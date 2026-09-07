use super::stats::{group_value, number, numeric_space, validate_group};
use super::*;
use crate::data::{DatasetSnapshot, SnapshotHandle, StoreSnapshot};
use crate::provenance::{SourceRef, Target};
use crate::state::ChartState;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Point, RowKey, TransformId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

struct CachedGraph {
    scope: Option<facets::PanelScope>,
    source: SnapshotHandle<StoreSnapshot>,
    definitions: Vec<TransformDefinition>,
    limits: CompileLimits,
    tables: BTreeMap<TransformId, Arc<PreparedTable>>,
    diagnostics: Vec<Diagnostic>,
    rows: usize,
}

/// One synchronous preparation route for typed-normalized authoring, recipes and layers.
/// A bounded graph cache retains each authored panel scope for the current source/graph.
/// New source or transform definitions release prior cache entries; prepared owners stay valid.
#[derive(Default)]
pub struct Compiler {
    cache: BTreeMap<Option<PanelKey>, CachedGraph>,
    presentation: Option<(PreparedChart, CompileLimits)>,
}
impl Compiler {
    /// Empty compiler; no host services, threads or I/O are needed for data preparation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Release cached graph/source ownership; existing prepared charts remain valid.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.presentation = None;
    }
    /// Validate, filter/map, compute stats, bind outputs, position, collect domains and emit
    /// immutable data-space geometry. Scale/range/layout preparation follows in WP-06.
    pub fn prepare(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
    ) -> ChartResult<PreparedChart> {
        let snapshot = source.get()?;
        if let Some((previous, old_limits)) = &self.presentation {
            let old = previous.definition();
            if *old_limits == limits
                && previous
                    .source
                    .get()
                    .is_ok_and(|p| std::ptr::eq(p, snapshot))
                && previous.state() == state
                && old.mappings == definition.mappings
                && old.transforms == definition.transforms
                && old.layers == definition.layers
                && old.facets == definition.facets
            {
                validate_definition(definition, snapshot, limits)?;
                let mut result = previous.clone();
                rebind_presentation(&mut result, &Arc::new(definition.clone()));
                self.presentation = Some((result.clone(), limits));
                return Ok(result);
            }
        }
        self.presentation = None;
        if self.cache.values().any(|c| {
            c.definitions != definition.transforms
                || !c.source.get().is_ok_and(|old| std::ptr::eq(old, snapshot))
        }) {
            self.cache.clear();
        }
        let result = if definition.facets.is_some() {
            self.cache.retain(|key, _| {
                key.as_ref().is_some_and(|key| {
                    definition
                        .facets
                        .as_ref()
                        .is_some_and(|spec| spec.order.contains(key))
                })
            });
            facets::prepare_facets(self, definition, source, state, limits)
        } else {
            self.cache.retain(|key, _| key.is_none());
            self.prepare_scoped(definition, source, state, limits, None)
        }?;
        self.presentation = Some((result.clone(), limits));
        Ok(result)
    }
    pub(crate) fn prepare_scoped(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
        scope: Option<&facets::PanelScope>,
    ) -> ChartResult<PreparedChart> {
        let snapshot = source.get()?;
        let order = validate_definition(definition, snapshot, limits)?;
        let cache_key = scope.map(|s| s.key.clone());
        let reuse = self.cache.get(&cache_key).is_some_and(|c| {
            c.scope.as_ref() == scope
                && c.definitions == definition.transforms
                && c.limits == limits
                && c.source.get().is_ok_and(|old| std::ptr::eq(old, snapshot))
        });
        let mut remaining = limits.max_prepared_rows;
        let (tables, mut diagnostics, graph_rows) = if reuse {
            let c = self
                .cache
                .get(&cache_key)
                .ok_or_else(|| error(DiagnosticCode::Validation, "Graph cache is absent."))?;
            charge(&mut remaining, c.rows, "prepared row")?;
            (c.tables.clone(), c.diagnostics.clone(), c.rows)
        } else {
            let mut tables = BTreeMap::new();
            let mut diagnostics = vec![];
            for id in order {
                let node = &definition.transforms[id];
                if !facets::targeted(&node.facet, scope) {
                    continue;
                }
                let input = resolve_input(
                    node.input,
                    snapshot,
                    &tables,
                    remaining,
                    scope,
                    &node.facet,
                    node.scope,
                )?;
                let data = snapshot.dataset(input.input.dataset)?;
                let input = facets::filter_panel(input, data, scope, &node.facet, node.scope)?;
                let statistic = facets::scoped_stat(&node.statistic, node.scope);
                let start = diagnostics.len();
                let table = stats::run(
                    input,
                    data,
                    stats::StatRequest {
                        stat: &statistic,
                        population: node.scope,
                        panel: facets::population_panel(scope, &node.facet, node.scope),
                        filters: &node.filters,
                        policy: node.invalid,
                        scope: &facets::operation_scope(
                            "transform",
                            node.id.get(),
                            node.scope,
                            scope,
                        ),
                    },
                    CompileLimits {
                        max_prepared_rows: remaining,
                        ..limits
                    },
                    &mut diagnostics,
                )
                .map_err(|e| context(e, data, None))?;
                for e in &mut diagnostics[start..] {
                    *e = context(e.clone(), data, None);
                }
                charge(&mut remaining, table.work_units(), "prepared value")?;
                tables.insert(node.id, table);
            }
            (tables, diagnostics, limits.max_prepared_rows - remaining)
        };
        let graph_diagnostics = diagnostics.clone();
        let mut layers = vec![];
        let mut colors = BTreeMap::new();
        let mut scale_domains = BTreeMap::new();
        let mut budget = GeometryBudget {
            limits,
            vertices: limits.max_vertices,
        };
        for layer in &definition.layers {
            if !facets::targeted(&layer.facet, scope) {
                continue;
            }
            let input = resolve_input(
                layer.data,
                snapshot,
                &tables,
                remaining,
                scope,
                &layer.facet,
                layer.scope,
            )?;
            let data = snapshot.dataset(input.input.dataset)?;
            let input = facets::filter_panel(input, data, scope, &layer.facet, layer.scope)?;
            let statistic = facets::scoped_stat(&layer.statistic, layer.scope);
            let start = diagnostics.len();
            let table = stats::run(
                input,
                data,
                stats::StatRequest {
                    stat: &statistic,
                    population: layer.scope,
                    panel: facets::population_panel(scope, &layer.facet, layer.scope),
                    filters: &layer.filters,
                    policy: layer.invalid,
                    scope: &facets::operation_scope("layer", layer.id.get(), layer.scope, scope),
                },
                CompileLimits {
                    max_prepared_rows: remaining,
                    ..limits
                },
                &mut diagnostics,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            charge(&mut remaining, table.work_units(), "prepared value")?;
            let prepared = prepare_layer(
                layer,
                table,
                data,
                &definition.mappings,
                state,
                &mut budget,
                &mut diagnostics,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            for e in &mut diagnostics[start..] {
                *e = context(e.clone(), data, Some(layer.id));
            }
            merge_named(
                &mut scale_domains,
                layer.scales,
                &eligible_domains(&prepared),
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            if let Some(legend) = &prepared.color_legend {
                if colors.get(&legend.id).is_some_and(|old| old != legend) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Layers sharing a color scale ID must resolve compatible domains and palettes; supply an explicit shared domain.",
                    ));
                }
                colors.insert(legend.id, legend.clone());
            }
            layers.push(prepared);
        }
        let mut domains = DomainContributions::default();
        if let Some(d) = scale_domains.get(&crate::ScaleId::new(0)) {
            domains.x = d.x;
            domains.x_space = d.x_space.clone();
        }
        if let Some(d) = scale_domains.get(&crate::ScaleId::new(1)) {
            domains.y = d.y;
            domains.y_space = d.y_space.clone();
        }
        let result = PreparedChart {
            panels: vec![],
            shared_training: None,
            definition: Arc::new(definition.clone()),
            source: source.clone(),
            state: state.clone(),
            layers,
            transforms: tables.clone(),
            domains,
            scale_domains,
            diagnostics,
            metrics: PreparationMetrics {
                evaluated_transforms: if reuse { 0 } else { tables.len() },
                reused_transforms: if reuse { tables.len() } else { 0 },
            },
        };
        self.cache.insert(
            cache_key,
            CachedGraph {
                scope: scope.cloned(),
                source: source.clone(),
                definitions: definition.transforms.clone(),
                limits,
                tables,
                diagnostics: graph_diagnostics,
                rows: graph_rows,
            },
        );
        Ok(result)
    }
}
fn context(mut e: Diagnostic, data: &DatasetSnapshot, layer: Option<LayerId>) -> Diagnostic {
    e.context.dataset = Some(data.version().dataset);
    e.context.dataset_revision = Some(data.version().revision);
    e.context.schema_version = Some(data.version().schema_version);
    e.context.layer = layer;
    e
}
fn charge(remaining: &mut usize, count: usize, kind: &str) -> ChartResult<()> {
    *remaining = remaining.checked_sub(count).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            format!("Compiler {kind} budget exceeded."),
        )
    })?;
    Ok(())
}
fn resolve_input(
    input: DataRef,
    snapshot: &StoreSnapshot,
    tables: &BTreeMap<TransformId, Arc<PreparedTable>>,
    remaining: usize,
    scope: Option<&facets::PanelScope>,
    target: &FacetTarget,
    stat: StatScope,
) -> ChartResult<Arc<PreparedTable>> {
    match input {
        DataRef::Dataset(id) => {
            facets::source_table(snapshot.dataset(id)?, remaining, scope, target, stat)
        }
        DataRef::Transform(id) => tables.get(&id).cloned().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("Transform {} is unavailable.", id.get()),
            )
        }),
    }
}
pub(super) fn validate_definition(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<usize>> {
    if let Some(theme) = &definition.theme {
        theme.resolve(&crate::theme::ThemePatch::default())?;
        if theme
            .layers
            .keys()
            .any(|id| !definition.layers.iter().any(|l| &l.id == id))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Theme names an absent layer.",
            ));
        }
    }
    if let Some(figure) = &definition.figure {
        figure.validate(crate::Limits::default())?;
    }
    if definition.layers.len() > limits.max_layers
        || definition.transforms.len() > limits.max_transforms
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Definition layer/transform count budget exceeded.",
        ));
    }
    let mut layers = BTreeSet::new();
    let mut nodes = BTreeMap::new();
    for (i, node) in definition.transforms.iter().enumerate() {
        if nodes.insert(node.id, i).is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Transform IDs must be unique.",
            ));
        }
        stats::validate_stat(&node.statistic, limits)?;
        stats::validate_filters(&node.filters, limits)?;
    }
    for layer in &definition.layers {
        if !layers.insert(layer.id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Layer IDs must be unique.",
            ));
        }
        if let Geom::Bar { width, .. } | Geom::Ohlc { width } = layer.geom
            && (!width.is_finite() || width <= 0.)
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Bar/candle width must be finite and positive.",
            ));
        }
        stats::validate_stat(&layer.statistic, limits)?;
        stats::validate_filters(&layer.filters, limits)?;
        if !layer.style.radius.is_finite()
            || layer.style.radius <= 0.
            || !layer.style.stroke_width.is_finite()
            || layer.style.stroke_width <= 0.
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Constant radii and stroke widths must be finite and positive.",
            ));
        }
    }
    for input in definition
        .transforms
        .iter()
        .map(|t| t.input)
        .chain(definition.layers.iter().map(|l| l.data))
    {
        match input {
            DataRef::Dataset(id) => {
                snapshot.dataset(id)?;
            }
            DataRef::Transform(id) if !nodes.contains_key(&id) => {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    format!("Unknown transform {}.", id.get()),
                ));
            }
            _ => {}
        }
    }
    // Iterative topological order avoids call-stack dependence on user graph depth.
    let mut visited = BTreeSet::new();
    let mut order = vec![];
    while order.len() < nodes.len() {
        let before = order.len();
        for (i, node) in definition.transforms.iter().enumerate() {
            if visited.contains(&node.id) {
                continue;
            }
            if matches!(node.input, DataRef::Dataset(_))
                || matches!(node.input,DataRef::Transform(id) if visited.contains(&id))
            {
                visited.insert(node.id);
                order.push(i);
            }
        }
        if order.len() == before {
            return Err(error(
                DiagnosticCode::Validation,
                "Transform dependency cycle: layout/viewport feedback and cyclic statistical inputs are unsupported.",
            ));
        }
    }
    preflight_schemas(definition, snapshot, &order, limits)?;
    Ok(order)
}

pub(super) struct EncodedRow {
    pub(super) x: Option<f64>,
    pub(super) y: Option<f64>,
    pub(super) x2: Option<f64>,
    pub(super) y2: Option<f64>,
    pub(super) color: Option<crate::scene::Color>,
    pub(super) low: Option<f64>,
    pub(super) high: Option<f64>,
    pub(super) size: Option<f64>,
    pub(super) group: Option<GroupValue>,
    pub(super) ordinal: u64,
    pub(super) target: Target,
    pub(super) key: Option<RowKey>,
}
fn coordinate_space(data: &DatasetSnapshot, value: &Numeric) -> ChartResult<ValueSpace> {
    if let Numeric::Category(id) = value {
        return data
            .categories(*id)
            .map(|v| ValueSpace::Categorical {
                categories: v.to_vec(),
            })
            .ok_or_else(|| {
                error(
                    DiagnosticCode::SchemaConflict,
                    "Band encoding requires a categorical source field.",
                )
            });
    }
    numeric_space(data, value)
}
fn source_space(data: &DatasetSnapshot, value: &Numeric) -> ChartResult<Option<ValueSpace>> {
    let space = coordinate_space(data, value)?;
    Ok(if matches!(value, Numeric::Literal(_)) {
        None
    } else {
        Some(space)
    })
}
fn bin_space(space: &ValueSpace, value: &BinNumeric) -> ChartResult<Option<ValueSpace>> {
    match value {
        BinNumeric::Literal(v) if !v.is_finite() => Err(error(
            DiagnosticCode::NumericalDomain,
            "Generated literal mappings must be finite.",
        )),
        BinNumeric::Literal(_) => Ok(None),
        BinNumeric::Field(BinField::Count) => Ok(Some(ValueSpace::Data)),
        BinNumeric::Field(_) => Ok(Some(space.clone())),
    }
}
fn bin_number(row: &BinnedRow, value: &BinNumeric) -> Option<f64> {
    match value {
        BinNumeric::Literal(v) => Some(*v),
        BinNumeric::Field(BinField::Start) => Some(row.start),
        BinNumeric::Field(BinField::End) => Some(row.end),
        BinNumeric::Field(BinField::Midpoint) => Some(row.start.midpoint(row.end)),
        BinNumeric::Field(BinField::Count) if row.count <= 1_u64 << 53 => Some(row.count as f64),
        _ => None,
    }
    .filter(|v| v.is_finite())
}
fn merge_space(a: &mut Option<ValueSpace>, b: &Option<ValueSpace>) -> ChartResult<()> {
    if let Some(space) = b {
        if a.as_ref().is_some_and(|prior| prior != space) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Incompatible calculation spaces or timestamp origins share an axis; use compatible mappings or independently named scales.",
            ));
        }
        *a = Some(space.clone());
    }
    Ok(())
}
// Train categorical membership from eligible post-stat geometry, ordered by the retained
// source catalog. Keep the layer catalog unchanged so geometry ordinals still decode exactly.
pub(super) fn eligible_domains(layer: &PreparedLayer) -> DomainContributions {
    let mut d = layer.domains.clone();
    for horizontal in [true, false] {
        let space = if horizontal {
            &mut d.x_space
        } else {
            &mut d.y_space
        };
        if let Some(ValueSpace::Categorical { categories }) = space {
            let mut used = BTreeSet::new();
            let mut include = |p: Point| {
                used.insert(if horizontal { p.x() } else { p.y() } as usize);
            };
            for mark in layer.marks.iter() {
                match &mark.geometry {
                    PreparedGeometry::Point(p) => include(*p),
                    PreparedGeometry::BandRun { lower, upper } => {
                        for p in lower.iter().chain(upper) {
                            include(*p);
                        }
                    }
                    PreparedGeometry::LineRun(points) => {
                        for p in points {
                            include(*p);
                        }
                    }
                    PreparedGeometry::Rule { from, to }
                    | PreparedGeometry::Rectangle { from, to }
                    | PreparedGeometry::Bar { from, to, .. } => {
                        include(*from);
                        include(*to);
                    }
                }
            }
            *categories = categories
                .iter()
                .enumerate()
                .filter(|(i, _)| used.contains(i))
                .map(|(_, s)| s.clone())
                .collect();
        }
    }
    d
}
pub(super) fn merge_named(
    all: &mut BTreeMap<crate::ScaleId, DomainContributions>,
    bindings: ScaleBindings,
    d: &DomainContributions,
) -> ChartResult<()> {
    for (id, horizontal) in [(bindings.x, true), (bindings.y, false)] {
        merge_axis(all, id, horizontal, d)?;
    }
    Ok(())
}
pub(super) fn merge_axis(
    all: &mut BTreeMap<crate::ScaleId, DomainContributions>,
    id: crate::ScaleId,
    horizontal: bool,
    d: &DomainContributions,
) -> ChartResult<()> {
    let a = all.entry(id).or_default();
    if (horizontal && a.y_space.is_some()) || (!horizontal && a.x_space.is_some()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "A named scale cannot serve both x and y.",
        ));
    }
    let b = if horizontal {
        DomainContributions {
            x: d.x,
            x_space: Some(d.x_space.clone().unwrap_or(ValueSpace::Data)),
            ..Default::default()
        }
    } else {
        DomainContributions {
            y: d.y,
            y_space: Some(d.y_space.clone().unwrap_or(ValueSpace::Data)),
            ..Default::default()
        }
    };
    // Union layer catalogs only here: each layer retains its own ordinal-to-label mapping.
    let (prior, next) = if horizontal {
        (&mut a.x_space, &b.x_space)
    } else {
        (&mut a.y_space, &b.y_space)
    };
    if let (
        Some(ValueSpace::Categorical { categories: p }),
        Some(ValueSpace::Categorical { categories: n }),
    ) = (prior, next)
    {
        let mut seen: BTreeSet<String> = p.iter().cloned().collect();
        p.extend(n.iter().filter(|v| seen.insert((*v).clone())).cloned());
        let extent = if horizontal { &mut a.x } else { &mut a.y };
        if let Some(e) = if horizontal { b.x } else { b.y } {
            Extent::include(extent, e.minimum);
            Extent::include(extent, e.maximum);
        }
    } else {
        merge_domains(a, &b)?;
    }
    Ok(())
}

fn merge_domains(a: &mut DomainContributions, b: &DomainContributions) -> ChartResult<()> {
    merge_space(&mut a.x_space, &b.x_space)?;
    merge_space(&mut a.y_space, &b.y_space)?;
    if let Some(v) = b.x {
        Extent::include(&mut a.x, v.minimum);
        Extent::include(&mut a.x, v.maximum);
    }
    if let Some(v) = b.y {
        Extent::include(&mut a.y, v.minimum);
        Extent::include(&mut a.y, v.maximum);
    }
    Ok(())
}
fn source_binding(
    layer: &Layer,
    authored: &SourceAes,
    inherited: &SourceAes,
    data: &DatasetSnapshot,
) -> ChartResult<(SourceAes, DomainContributions)> {
    let endpoints = matches!(layer.geom, Geom::Rule | Geom::Rectangle);
    let mut domains = DomainContributions::default();
    let aes = if layer.inherit {
        authored.inherit(inherited)
    } else {
        authored.clone()
    };
    let (Some(x), Some(y)) = (&aes.x, &aes.y) else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Geometry requires x and y source mappings.",
        ));
    };
    if endpoints && (aes.x2.is_none() || aes.y2.is_none()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Rules/rectangles require both second endpoints; baselines must be explicit.",
        ));
    }
    if matches!(layer.geom, Geom::Bar { .. } | Geom::Ohlc { .. }) && aes.y2.is_none() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Bars/candles require an explicit y2 baseline/close.",
        ));
    }
    if matches!(layer.geom, Geom::Ohlc { .. }) && (aes.low.is_none() || aes.high.is_none()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires low and high numeric bounds.",
        ));
    }
    // Validate even unused inherited fields so unrelated schemas require explicit overrides.
    for value in [&aes.x, &aes.y, &aes.x2, &aes.y2, &aes.low, &aes.high]
        .into_iter()
        .flatten()
    {
        coordinate_space(data, value)?;
    }
    if let Some(size) = &aes.size {
        numeric_space(data, size)?;
    }
    let grouping = aes.group.map_or(Grouping::All, Grouping::Field);
    validate_group(data, &grouping)?;
    domains.x_space = source_space(data, x)?;
    domains.y_space = source_space(data, y)?;
    if endpoints {
        if let Some(value) = &aes.x2 {
            merge_space(&mut domains.x_space, &source_space(data, value)?)?;
        }
        if let Some(value) = &aes.y2 {
            merge_space(&mut domains.y_space, &source_space(data, value)?)?;
        }
    }
    if matches!(
        layer.geom,
        Geom::Ribbon { .. } | Geom::Bar { .. } | Geom::Ohlc { .. }
    ) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut domains.y_space, &source_space(data, y2)?)?;
    }
    for bound in [&aes.low, &aes.high].into_iter().flatten() {
        merge_space(&mut domains.y_space, &source_space(data, bound)?)?;
    }
    if matches!(layer.geom, Geom::Ohlc { .. } | Geom::Bar { .. })
        && matches!(
            domains.y_space,
            Some(ValueSpace::Categorical { .. } | ValueSpace::Timestamp { .. })
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Bar/OHLC values must be numeric.",
        ));
    }
    validate_line_size(layer, aes.size.is_some())?;
    Ok((aes, domains))
}
fn bin_binding(
    layer: &Layer,
    aes: &BinAes,
    space: &ValueSpace,
) -> ChartResult<DomainContributions> {
    if matches!(layer.geom, Geom::Ohlc { .. }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires source open/close/low/high mappings.",
        ));
    }
    let endpoints = matches!(layer.geom, Geom::Rule | Geom::Rectangle);
    let mut domains = DomainContributions::default();
    if endpoints && (aes.x2.is_none() || aes.y2.is_none()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Generated rule/rectangle mappings require both endpoints.",
        ));
    }
    domains.x_space = bin_space(space, &aes.x)?;
    domains.y_space = bin_space(space, &aes.y)?;
    for value in [&aes.x2, &aes.y2, &aes.size].into_iter().flatten() {
        bin_space(space, value)?;
    }
    if endpoints {
        if let Some(value) = &aes.x2 {
            merge_space(&mut domains.x_space, &bin_space(space, value)?)?;
        }
        if let Some(value) = &aes.y2 {
            merge_space(&mut domains.y_space, &bin_space(space, value)?)?;
        }
    }
    if matches!(layer.geom, Geom::Ribbon { .. } | Geom::Bar { .. }) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a generated y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut domains.y_space, &bin_space(space, y2)?)?;
    }
    validate_line_size(layer, aes.size.is_some())?;
    Ok(domains)
}
fn validate_line_size(layer: &Layer, mapped: bool) -> ChartResult<()> {
    if (layer.geom.run().is_some() || matches!(layer.geom, Geom::Rectangle)) && mapped {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Mapped size currently supports point radius and rule stroke width; use constant styling for lines and rectangles.",
        ));
    }
    Ok(())
}

struct GeometryBudget {
    limits: CompileLimits,
    vertices: usize,
}
fn prepare_layer(
    layer: &Layer,
    table: Arc<PreparedTable>,
    data: &DatasetSnapshot,
    inherited: &SourceAes,
    state: &ChartState,
    budget: &mut GeometryBudget,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<PreparedLayer> {
    let limits = budget.limits;
    let vertices = &mut budget.vertices;
    let mut domains;
    let (mut encoded, mapped_size): (Vec<EncodedRow>, bool) = match (&table.rows, &layer.mappings) {
        (PreparedRows::Source(rows), Mappings::Source(authored)) => {
            let (aes, bound_domains) = source_binding(layer, authored, inherited, data)?;
            domains = bound_domains;
            let (Some(x), Some(y)) = (&aes.x, &aes.y) else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Missing checked source mappings.",
                ));
            };
            let grouping = aes.group.map_or(Grouping::All, Grouping::Field);
            let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            let catalogs: BTreeMap<_, BTreeMap<&str, f64>> = [&aes.x, &aes.y, &aes.x2, &aes.y2]
                .into_iter()
                .flatten()
                .filter_map(|v| {
                    if let Numeric::Category(id) = v {
                        Some((
                            *id,
                            data.categories(*id)
                                .unwrap_or_default()
                                .iter()
                                .enumerate()
                                .map(|(i, s)| (s.as_str(), i as f64))
                                .collect(),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            let coordinate = |row: crate::data::RowView<'_>, value: &Numeric| {
                if let Numeric::Category(id) = value {
                    if let Some(crate::data::ValueRef::Category(label)) = row.value(*id) {
                        catalogs.get(id)?.get(label).copied()
                    } else {
                        None
                    }
                } else {
                    number(row, value)
                }
            };
            let encoded = rows
                .iter()
                .map(|r| {
                    let row = index[&r.key];
                    EncodedRow {
                        x: coordinate(row, x),
                        y: coordinate(row, y),
                        x2: aes.x2.as_ref().and_then(|v| coordinate(row, v)),
                        y2: aes.y2.as_ref().and_then(|v| coordinate(row, v)),
                        color: None,
                        low: aes.low.as_ref().and_then(|v| coordinate(row, v)),
                        high: aes.high.as_ref().and_then(|v| coordinate(row, v)),
                        size: aes.size.as_ref().and_then(|v| coordinate(row, v)),
                        group: group_value(row, &grouping),
                        ordinal: r.ordinal,
                        target: Target::Source(SourceRef {
                            dataset: table.input.dataset,
                            key: r.key,
                        }),
                        key: Some(r.key),
                    }
                })
                .collect::<Vec<_>>();
            (encoded, aes.size.is_some())
        }
        (PreparedRows::Statistical(rows), Mappings::Statistical(aes)) => {
            let OutputSchema::Statistical { fields, .. } = &table.schema else {
                unreachable!()
            };
            domains = statistical_binding(layer, aes, fields)?;
            let value = |r: &StatisticalRow, n: &StatNumeric| match n {
                StatNumeric::Literal(v) => Some(*v),
                StatNumeric::Field(StatField::Group) => fields
                    .iter()
                    .find(|c| c.field == StatField::Group)
                    .and_then(|c| {
                        if let ValueSpace::Categorical { categories } = &c.space {
                            categories
                                .iter()
                                .position(|v| v == &super::statistics::group_label(&r.group))
                                .map(|i| i as f64)
                        } else {
                            None
                        }
                    }),
                StatNumeric::Field(f) => r.value(f),
            };
            (
                rows.iter()
                    .enumerate()
                    .map(|(i, r)| EncodedRow {
                        x: value(r, &aes.x),
                        y: value(r, &aes.y),
                        x2: aes.x2.as_ref().and_then(|v| value(r, v)),
                        y2: aes.y2.as_ref().and_then(|v| value(r, v)),
                        color: None,
                        low: None,
                        high: None,
                        size: aes.size.as_ref().and_then(|v| value(r, v)),
                        group: Some(r.group.clone()),
                        ordinal: i as u64,
                        target: r.target.clone(),
                        key: None,
                    })
                    .collect(),
                aes.size.is_some(),
            )
        }
        (PreparedRows::Binned(rows), Mappings::Binned(aes)) => {
            domains = bin_binding(layer, aes, &table.space)?;
            let encoded = rows
                .iter()
                .enumerate()
                .map(|(i, r)| EncodedRow {
                    x: bin_number(r, &aes.x),
                    y: bin_number(r, &aes.y),
                    x2: aes.x2.as_ref().and_then(|v| bin_number(r, v)),
                    y2: aes.y2.as_ref().and_then(|v| bin_number(r, v)),
                    color: None,
                    low: None,
                    high: None,
                    size: aes.size.as_ref().and_then(|v| bin_number(r, v)),
                    group: Some(r.group.clone()),
                    ordinal: i as u64,
                    target: r.target.clone(),
                    key: None,
                })
                .collect();
            (encoded, aes.size.is_some())
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Aesthetic stage does not match the statistic output schema; use BinAes for bins and SourceAes for observations.",
            ));
        }
    };
    if layer.geom.run().is_some() && mapped_size {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Mapped size currently supports point radius and rule stroke width; use constant styling for lines and rectangles.",
        ));
    }
    if let Geom::Area { baseline, .. } = layer.geom {
        if !baseline.is_finite()
            || matches!(
                domains.y_space,
                Some(ValueSpace::Categorical { .. } | ValueSpace::Timestamp { .. })
            )
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Area baseline requires a finite numeric calculation space.",
            ));
        }
        for row in &mut encoded {
            row.y2 = Some(baseline);
        }
    }
    if matches!(layer.geom, Geom::Ribbon { .. }) {
        for row in &mut encoded {
            if !matches!((row.y, row.y2), (Some(a), Some(b)) if a <= b) {
                row.y = None;
                row.y2 = None;
            }
        }
    }
    if let Geom::Bar { nonnegative, .. } = layer.geom {
        for row in &mut encoded {
            row.x2 = row.x;
            if nonnegative && row.y.is_some_and(|y| y < 0.) {
                row.y = None;
            }
        }
    }
    let color_legend = super::colors::apply(layer, data, &table, &mut encoded, limits)?;
    super::positions::apply(layer, &domains, &mut encoded, limits)?;
    super::positions::output_space(layer, &mut domains);
    let mut prepared = PreparedLayer {
        color_legend,
        position: layer.position.clone(),
        id: layer.id,
        scales: layer.scales,
        clip: layer.clip,
        table,
        marks: Arc::new(vec![]),
        domains,
        invalid_geometry: 0,
        visible: state.is_visible(layer.id),
    };
    let mut samples = vec![];
    if let Some((_, connect_gaps)) = layer.geom.run() {
        let mut groups: Vec<(GroupValue, Vec<EncodedRow>)> = vec![];
        let mut indexes = BTreeMap::new();
        let mut boundary = 0;
        for row in encoded {
            let valid = row.x.is_some() && row.y.is_some() && row.group.is_some();
            if !valid {
                prepared.invalid_geometry += 1;
                if samples.len() < 32
                    && let Some(key) = row.key
                {
                    samples.push(key);
                }
            }
            let Some(group) = &row.group else {
                if !connect_gaps {
                    boundary += 1;
                }
                continue;
            };
            let index_key = (group.clone(), boundary);
            let i = if let Some(&i) = indexes.get(&index_key) {
                i
            } else {
                if groups.len() >= limits.max_groups {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Geometry group/run boundary budget exceeded.",
                    ));
                }
                let i = groups.len();
                indexes.insert(index_key, i);
                groups.push((group.clone(), vec![]));
                i
            };
            groups[i].1.push(row);
        }
        for (group, rows) in groups {
            let mut style = layer.style;
            let mut color = None;
            for row in rows.iter().filter(|r| r.x.is_some() && r.y.is_some()) {
                if let Some(c) = row.color {
                    if color.is_some_and(|old| old != c) {
                        return Err(error(
                            DiagnosticCode::UnsupportedCapability,
                            "Filled/line runs require color constant within each group; use a group color mapping.",
                        ));
                    }
                    color = Some(c);
                }
            }
            if let Some(c) = color {
                style.color = c;
            }

            // Missing x has no sortable position. It separates authored blocks before x ordering.
            let mut block = vec![];
            for row in rows {
                if row.x.is_none() {
                    if !connect_gaps {
                        emit_line_block(
                            &mut prepared,
                            &mut block,
                            &group,
                            style,
                            layer.geom,
                            vertices,
                        )?;
                    }
                } else {
                    block.push(row);
                }
            }
            emit_line_block(
                &mut prepared,
                &mut block,
                &group,
                style,
                layer.geom,
                vertices,
            )?;
        }
    } else {
        for row in encoded {
            let base_style = Style {
                color: row.color.unwrap_or(layer.style.color),
                ..layer.style
            };
            let style = if mapped_size {
                row.size.filter(|v| *v > 0.).map(|size| Style {
                    radius: size,
                    stroke_width: size,
                    ..base_style
                })
            } else {
                Some(base_style)
            };
            if let Geom::Ohlc { width } = layer.geom
                && let (
                    Some(x),
                    Some(open),
                    Some(close),
                    Some(low),
                    Some(high),
                    Some(group),
                    Some(style),
                ) = (
                    row.x,
                    row.y,
                    row.y2,
                    row.low,
                    row.high,
                    row.group.as_ref(),
                    style,
                )
                && low <= open
                && open <= high
                && low <= close
                && close <= high
            {
                charge(vertices, 6, "OHLC vertex")?;
                for geometry in [
                    PreparedGeometry::Rule {
                        from: Point::new(x, low)?,
                        to: Point::new(x, high)?,
                    },
                    PreparedGeometry::Bar {
                        from: Point::new(x, open)?,
                        to: Point::new(x, close)?,
                        width,
                    },
                ] {
                    include_geometry(&mut prepared.domains, &geometry);
                    Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                        geometry,
                        targets: vec![row.target.clone()],
                        group: group.clone(),
                        style,
                    });
                }
                continue;
            }
            let geometry = match (row.x, row.y) {
                (Some(x), Some(y)) => match layer.geom {
                    Geom::Point => Some(PreparedGeometry::Point(Point::new(x, y)?)),
                    Geom::Bar { width, .. } => row
                        .y2
                        .map(|y2| {
                            Ok(PreparedGeometry::Bar {
                                from: Point::new(x, y)?,
                                to: Point::new(x, y2)?,
                                width,
                            })
                        })
                        .transpose()?,
                    Geom::Ohlc { .. } => None,
                    Geom::Rule => match (row.x2, row.y2) {
                        (Some(x2), Some(y2)) => Some(PreparedGeometry::Rule {
                            from: Point::new(x, y)?,
                            to: Point::new(x2, y2)?,
                        }),
                        _ => None,
                    },
                    Geom::Rectangle => match (row.x2, row.y2) {
                        (Some(x2), Some(y2)) => Some(PreparedGeometry::Rectangle {
                            from: Point::new(x, y)?,
                            to: Point::new(x2, y2)?,
                        }),
                        _ => None,
                    },
                    Geom::Line { .. } | Geom::Area { .. } | Geom::Ribbon { .. } => None,
                },
                _ => None,
            };
            if let (Some(geometry), Some(style), Some(group)) = (geometry, style, row.group) {
                let n = match geometry {
                    PreparedGeometry::Point(_) => 1,
                    PreparedGeometry::Rule { .. } => 2,
                    PreparedGeometry::Rectangle { .. } | PreparedGeometry::Bar { .. } => 4,
                    PreparedGeometry::LineRun(_) | PreparedGeometry::BandRun { .. } => 0,
                };
                charge(vertices, n, "vertex")?;
                include_geometry(&mut prepared.domains, &geometry);
                Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                    geometry,
                    targets: vec![row.target],
                    group,
                    style,
                });
            } else {
                prepared.invalid_geometry += 1;
                if samples.len() < 32
                    && let Some(key) = row.key
                {
                    samples.push(key);
                }
            }
        }
    }
    stats::warning(
        layer.invalid,
        prepared.invalid_geometry,
        samples,
        format!(
            "Geometry excluded {} rows with invalid required coordinates, groups or sizes; gaps are not zero values.",
            prepared.invalid_geometry
        ),
        diagnostics,
    )?;
    Ok(prepared)
}
fn emit_line_block(
    prepared: &mut PreparedLayer,
    rows: &mut Vec<EncodedRow>,
    group: &GroupValue,
    style: Style,
    geom: Geom,
    vertices: &mut usize,
) -> ChartResult<()> {
    let (order, connect) = geom.run().expect("run geometry");
    if order == LineOrder::X {
        rows.sort_by(|a, b| {
            // Signed zeros are equal x values and therefore use insertion ordinal.
            a.x.partial_cmp(&b.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.ordinal.cmp(&b.ordinal))
        });
    }
    let mut points = vec![];
    let mut upper = vec![];
    let mut targets = vec![];
    for row in rows.drain(..) {
        if let (Some(x), Some(y)) = (row.x, row.y) {
            charge(
                vertices,
                if matches!(geom, Geom::Line { .. }) {
                    1
                } else {
                    2
                },
                "vertex",
            )?;
            points.push(Point::new(x, y)?);
            if !matches!(geom, Geom::Line { .. }) {
                upper.push(Point::new(x, row.y2.expect("validated boundary"))?);
            }
            targets.push(row.target);
        } else if !connect {
            push_run(
                prepared,
                &mut points,
                &mut upper,
                &mut targets,
                group,
                style,
            );
        }
    }
    push_run(
        prepared,
        &mut points,
        &mut upper,
        &mut targets,
        group,
        style,
    );
    Ok(())
}
fn push_run(
    prepared: &mut PreparedLayer,
    points: &mut Vec<Point>,
    upper: &mut Vec<Point>,
    targets: &mut Vec<Target>,
    group: &GroupValue,
    style: Style,
) {
    if !points.is_empty() {
        let geometry = if upper.is_empty() {
            PreparedGeometry::LineRun(std::mem::take(points))
        } else {
            PreparedGeometry::BandRun {
                lower: std::mem::take(points),
                upper: std::mem::take(upper),
            }
        };
        include_geometry(&mut prepared.domains, &geometry);
        Arc::make_mut(&mut prepared.marks).push(PreparedMark {
            geometry,
            targets: std::mem::take(targets),
            group: group.clone(),
            style,
        });
    }
}
fn include_geometry(domains: &mut DomainContributions, geometry: &PreparedGeometry) {
    let mut include = |p: Point| {
        Extent::include(&mut domains.x, p.x());
        Extent::include(&mut domains.y, p.y());
    };
    match geometry {
        PreparedGeometry::Point(p) => include(*p),
        PreparedGeometry::BandRun { lower, upper } => {
            for p in lower.iter().chain(upper) {
                include(*p);
            }
        }
        PreparedGeometry::LineRun(points) => {
            for p in points {
                include(*p);
            }
        }
        PreparedGeometry::Rule { from, to } => {
            include(*from);
            include(*to);
        }
        PreparedGeometry::Rectangle { from, to } | PreparedGeometry::Bar { from, to, .. } => {
            include(*from);
            include(*to);
        }
    }
}

#[derive(Clone)]
struct OutputShape {
    dataset: crate::DatasetId,
    bins: Option<ValueSpace>,
    statistical: Option<Vec<StatColumn>>,
}
fn preflight_schemas(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    order: &[usize],
    limits: CompileLimits,
) -> ChartResult<()> {
    let mut shapes = BTreeMap::<TransformId, OutputShape>::new();
    for &i in order {
        let node = &definition.transforms[i];
        let input = input_shape(node.input, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(
            input,
            data,
            &facets::scoped_stat(&node.statistic, node.scope),
            &node.filters,
            limits,
        )
        .map_err(|e| context(e, data, None))?;
        shapes.insert(node.id, output);
    }
    let mut domains = BTreeMap::new();
    for layer in &definition.layers {
        let input = input_shape(layer.data, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(
            input,
            data,
            &facets::scoped_stat(&layer.statistic, layer.scope),
            &layer.filters,
            limits,
        )
        .map_err(|e| context(e, data, Some(layer.id)))?;
        let mut binding = match (&output.bins, &output.statistical, &layer.mappings) {
            (None, None, Mappings::Source(aes)) => {
                source_binding(layer, aes, &definition.mappings, data).map(|(_, domain)| domain)
            }
            (Some(space), None, Mappings::Binned(aes)) => bin_binding(layer, aes, space),
            (None, Some(fields), Mappings::Statistical(aes)) => {
                statistical_binding(layer, aes, fields)
            }
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Aesthetic stage does not match the declared source/generated output schema.",
            )),
        }
        .map_err(|e| context(e, data, Some(layer.id)))?;
        super::positions::validate(layer, &binding, limits)?;
        super::positions::output_space(layer, &mut binding);
        merge_named(&mut domains, layer.scales, &binding)
            .map_err(|e| context(e, data, Some(layer.id)))?;
    }
    Ok(())
}
fn input_shape(
    input: DataRef,
    shapes: &BTreeMap<TransformId, OutputShape>,
) -> ChartResult<OutputShape> {
    match input {
        DataRef::Dataset(dataset) => Ok(OutputShape {
            dataset,
            bins: None,
            statistical: None,
        }),
        DataRef::Transform(id) => shapes.get(&id).cloned().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Dependency schema is absent.",
            )
        }),
    }
}
fn stat_shape(
    mut input: OutputShape,
    data: &DatasetSnapshot,
    stat: &Statistic,
    filters: &[SourceFilter],
    limits: CompileLimits,
) -> ChartResult<OutputShape> {
    if !filters.is_empty() && (input.bins.is_some() || input.statistical.is_some()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Filter source observations before generating statistical rows.",
        ));
    }
    for filter in filters {
        numeric_space(data, &filter.value)?;
    }
    let automatic = if let StatParameters::AutoBin(s) = &stat.parameters {
        Some(BinSpec {
            input: s.input.clone(),
            edges: vec![],
            grouping: s.grouping.clone(),
            space: s.space.clone(),
            outliers: OutlierPolicy::Exclude,
        })
    } else {
        None
    };
    let bin = match &stat.parameters {
        StatParameters::Bin(s) => Some(s),
        _ => automatic.as_ref(),
    };
    if !matches!(stat.parameters, StatParameters::Identity)
        && (input.bins.is_some() || input.statistical.is_some())
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Nonidentity statistics require source observations.",
        ));
    }
    if matches!(
        stat.parameters,
        StatParameters::Count(_) | StatParameters::Summary(_) | StatParameters::Ols(_)
    ) {
        input.statistical = Some(super::statistics::schema(stat, data, limits)?);
    }
    if let Some(spec) = bin {
        if input.bins.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Bins require source observations rather than generated bins.",
            ));
        }
        let space = numeric_space(data, &spec.input)?;
        validate_group(data, &spec.grouping)?;
        input.bins = Some(match &spec.space {
            StatSpace::Data => space,
            StatSpace::Transformed(transform) => ValueSpace::Transformed {
                input: Box::new(space),
                transform: transform.clone(),
            },
        });
    }
    Ok(input)
}

fn statistical_binding(
    layer: &Layer,
    aes: &StatAes,
    fields: &[StatColumn],
) -> ChartResult<DomainContributions> {
    if matches!(layer.geom, Geom::Ohlc { .. }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires source open/close/low/high mappings.",
        ));
    }
    let space = |value: &StatNumeric| -> ChartResult<Option<ValueSpace>> {
        match value {
            StatNumeric::Literal(v) if v.is_finite() => Ok(None),
            StatNumeric::Literal(_) => Err(error(
                DiagnosticCode::NumericalDomain,
                "Generated literal must be finite.",
            )),
            StatNumeric::Field(f) => fields
                .iter()
                .find(|c| &c.field == f)
                .map(|c| Some(c.space.clone()))
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Generated field is absent from this statistic's schema.",
                    )
                }),
        }
    };
    validate_line_size(layer, aes.size.is_some())?;
    let mut d = DomainContributions {
        x_space: space(&aes.x)?,
        y_space: space(&aes.y)?,
        ..Default::default()
    };
    for v in [&aes.x2, &aes.y2].into_iter().flatten() {
        space(v)?;
    }
    if let Some(v) = &aes.size
        && matches!(space(v)?, Some(ValueSpace::Categorical { .. }))
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Mapped generated size requires a numeric field.",
        ));
    }
    if matches!(layer.geom, Geom::Rule | Geom::Rectangle) {
        let (Some(x2), Some(y2)) = (&aes.x2, &aes.y2) else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Interval geometry requires both generated endpoints.",
            ));
        };
        merge_space(&mut d.x_space, &space(x2)?)?;
        merge_space(&mut d.y_space, &space(y2)?)?;
    }
    if matches!(layer.geom, Geom::Ribbon { .. } | Geom::Bar { .. }) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a generated y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut d.y_space, &space(y2)?)?;
    }
    Ok(d)
}

// Theme/furniture/guide changes reuse immutable rows, marks and provenance without recomputation.
fn rebind_presentation(chart: &mut PreparedChart, definition: &Arc<ChartDefinition>) {
    chart.definition = definition.clone();
    chart.metrics = PreparationMetrics {
        evaluated_transforms: 0,
        reused_transforms: chart.metrics.evaluated_transforms + chart.metrics.reused_transforms,
    };
    for panel in &mut chart.panels {
        rebind_presentation(Arc::make_mut(&mut panel.chart), definition);
    }
}
