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
    source: SnapshotHandle<StoreSnapshot>,
    definitions: Vec<TransformDefinition>,
    limits: CompileLimits,
    tables: BTreeMap<TransformId, Arc<PreparedTable>>,
    diagnostics: Vec<Diagnostic>,
    rows: usize,
}

/// One synchronous preparation route for typed-normalized authoring, recipes and layers.
/// A bounded single graph cache retains at most the last successful source/graph snapshot.
#[derive(Default)]
pub struct Compiler {
    cache: Option<CachedGraph>,
}
impl Compiler {
    /// Empty compiler; no host services, threads or I/O are needed for data preparation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Release cached graph/source ownership; existing prepared charts remain valid.
    pub fn clear_cache(&mut self) {
        self.cache = None;
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
        let order = validate_definition(definition, snapshot, limits)?;
        let reuse = self.cache.as_ref().is_some_and(|c| {
            c.definitions == definition.transforms
                && c.limits == limits
                && c.source.get().is_ok_and(|old| std::ptr::eq(old, snapshot))
        });
        let mut remaining = limits.max_prepared_rows;
        let (tables, mut diagnostics, graph_rows) = if reuse {
            let c = self
                .cache
                .as_ref()
                .ok_or_else(|| error(DiagnosticCode::Validation, "Graph cache is absent."))?;
            charge(&mut remaining, c.rows, "prepared row")?;
            (c.tables.clone(), c.diagnostics.clone(), c.rows)
        } else {
            let mut tables = BTreeMap::new();
            let mut diagnostics = vec![];
            for id in order {
                let node = &definition.transforms[id];
                let input = resolve_input(node.input, snapshot, &tables, remaining)?;
                let data = snapshot.dataset(input.input.dataset)?;
                let start = diagnostics.len();
                let table = stats::run(
                    input,
                    data,
                    stats::StatRequest {
                        stat: &node.statistic,
                        filters: &node.filters,
                        policy: node.invalid,
                        scope: &format!("transform:{}", node.id.get()),
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
                charge(&mut remaining, table.rows.len(), "prepared row")?;
                tables.insert(node.id, table);
            }
            (tables, diagnostics, limits.max_prepared_rows - remaining)
        };
        let graph_diagnostics = diagnostics.clone();
        let mut layers = vec![];
        let mut domains = DomainContributions::default();
        let mut budget = GeometryBudget {
            limits,
            vertices: limits.max_vertices,
        };
        for layer in &definition.layers {
            let input = resolve_input(layer.data, snapshot, &tables, remaining)?;
            let data = snapshot.dataset(input.input.dataset)?;
            let start = diagnostics.len();
            let table = stats::run(
                input,
                data,
                stats::StatRequest {
                    stat: &layer.statistic,
                    filters: &layer.filters,
                    policy: layer.invalid,
                    scope: &format!("layer:{}", layer.id.get()),
                },
                CompileLimits {
                    max_prepared_rows: remaining,
                    ..limits
                },
                &mut diagnostics,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            charge(&mut remaining, table.rows.len(), "prepared row")?;
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
            merge_domains(&mut domains, &prepared.domains)
                .map_err(|e| context(e, data, Some(layer.id)))?;
            layers.push(prepared);
        }
        let result = PreparedChart {
            definition: Arc::new(definition.clone()),
            source: source.clone(),
            state: state.clone(),
            layers,
            transforms: tables.clone(),
            domains,
            diagnostics,
            metrics: PreparationMetrics {
                evaluated_transforms: if reuse { 0 } else { tables.len() },
                reused_transforms: if reuse { tables.len() } else { 0 },
            },
        };
        self.cache = Some(CachedGraph {
            source: source.clone(),
            definitions: definition.transforms.clone(),
            limits,
            tables,
            diagnostics: graph_diagnostics,
            rows: graph_rows,
        });
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
) -> ChartResult<Arc<PreparedTable>> {
    match input {
        DataRef::Dataset(id) => stats::source_table(snapshot.dataset(id)?, remaining),
        DataRef::Transform(id) => tables.get(&id).cloned().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("Transform {} is unavailable.", id.get()),
            )
        }),
    }
}
fn validate_definition(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<usize>> {
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
    preflight_schemas(definition, snapshot, &order)?;
    Ok(order)
}

struct EncodedRow {
    x: Option<f64>,
    y: Option<f64>,
    x2: Option<f64>,
    y2: Option<f64>,
    size: Option<f64>,
    group: Option<GroupValue>,
    ordinal: u64,
    target: Target,
    key: Option<RowKey>,
}
fn source_space(data: &DatasetSnapshot, value: &Numeric) -> ChartResult<Option<ValueSpace>> {
    let space = numeric_space(data, value)?;
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
                "Incompatible calculation spaces or timestamp origins share an axis; use compatible mappings or later independent scales.",
            ));
        }
        *a = Some(space.clone());
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
    // Validate even unused inherited fields so unrelated schemas require explicit overrides.
    for value in [&aes.x, &aes.y, &aes.x2, &aes.y2, &aes.size]
        .into_iter()
        .flatten()
    {
        numeric_space(data, value)?;
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
    validate_line_size(layer, aes.size.is_some())?;
    Ok((aes, domains))
}
fn bin_binding(
    layer: &Layer,
    aes: &BinAes,
    space: &ValueSpace,
) -> ChartResult<DomainContributions> {
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
    validate_line_size(layer, aes.size.is_some())?;
    Ok(domains)
}
fn validate_line_size(layer: &Layer, mapped: bool) -> ChartResult<()> {
    if matches!(layer.geom, Geom::Line { .. } | Geom::Rectangle) && mapped {
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
    let domains;
    let (encoded, mapped_size) = match (&table.rows, &layer.mappings) {
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
            let encoded = rows
                .iter()
                .map(|r| {
                    let row = index[&r.key];
                    EncodedRow {
                        x: number(row, x),
                        y: number(row, y),
                        x2: aes.x2.as_ref().and_then(|v| number(row, v)),
                        y2: aes.y2.as_ref().and_then(|v| number(row, v)),
                        size: aes.size.as_ref().and_then(|v| number(row, v)),
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
    if matches!(layer.geom, Geom::Line { .. }) && mapped_size {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Mapped size currently supports point radius and rule stroke width; use constant styling for lines and rectangles.",
        ));
    }
    let mut prepared = PreparedLayer {
        id: layer.id,
        table,
        marks: vec![],
        domains,
        invalid_geometry: 0,
        visible: state.is_visible(layer.id),
    };
    let mut samples = vec![];
    if let Geom::Line {
        order,
        connect_gaps,
    } = layer.geom
    {
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
            // Missing x has no sortable position. It separates authored blocks before x ordering.
            let mut block = vec![];
            for row in rows {
                if row.x.is_none() {
                    if !connect_gaps {
                        emit_line_block(
                            &mut prepared,
                            &mut block,
                            &group,
                            order,
                            connect_gaps,
                            layer.style,
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
                order,
                connect_gaps,
                layer.style,
                vertices,
            )?;
        }
    } else {
        for row in encoded {
            let style = if mapped_size {
                row.size.filter(|v| *v > 0.).map(|size| Style {
                    radius: size,
                    stroke_width: size,
                    ..layer.style
                })
            } else {
                Some(layer.style)
            };
            let geometry = match (row.x, row.y) {
                (Some(x), Some(y)) => match layer.geom {
                    Geom::Point => Some(PreparedGeometry::Point(Point::new(x, y)?)),
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
                    Geom::Line { .. } => None,
                },
                _ => None,
            };
            if let (Some(geometry), Some(style), Some(group)) = (geometry, style, row.group) {
                let n = match geometry {
                    PreparedGeometry::Point(_) => 1,
                    PreparedGeometry::Rule { .. } => 2,
                    PreparedGeometry::Rectangle { .. } => 4,
                    PreparedGeometry::LineRun(_) => 0,
                };
                charge(vertices, n, "vertex")?;
                include_geometry(&mut prepared.domains, &geometry);
                prepared.marks.push(PreparedMark {
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
    order: LineOrder,
    connect: bool,
    style: Style,
    vertices: &mut usize,
) -> ChartResult<()> {
    if order == LineOrder::X {
        rows.sort_by(|a, b| {
            // Signed zeros are equal x values and therefore use insertion ordinal.
            a.x.partial_cmp(&b.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.ordinal.cmp(&b.ordinal))
        });
    }
    let mut points = vec![];
    let mut targets = vec![];
    for row in rows.drain(..) {
        if let (Some(x), Some(y)) = (row.x, row.y) {
            charge(vertices, 1, "vertex")?;
            points.push(Point::new(x, y)?);
            targets.push(row.target);
        } else if !connect {
            push_run(prepared, &mut points, &mut targets, group, style);
        }
    }
    push_run(prepared, &mut points, &mut targets, group, style);
    Ok(())
}
fn push_run(
    prepared: &mut PreparedLayer,
    points: &mut Vec<Point>,
    targets: &mut Vec<Target>,
    group: &GroupValue,
    style: Style,
) {
    if !points.is_empty() {
        let geometry = PreparedGeometry::LineRun(std::mem::take(points));
        include_geometry(&mut prepared.domains, &geometry);
        prepared.marks.push(PreparedMark {
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
        PreparedGeometry::LineRun(points) => {
            for p in points {
                include(*p);
            }
        }
        PreparedGeometry::Rule { from, to } => {
            include(*from);
            include(*to);
        }
        PreparedGeometry::Rectangle { from, to } => {
            include(*from);
            include(*to);
        }
    }
}

#[derive(Clone)]
struct OutputShape {
    dataset: crate::DatasetId,
    bins: Option<ValueSpace>,
}
fn preflight_schemas(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    order: &[usize],
) -> ChartResult<()> {
    let mut shapes = BTreeMap::<TransformId, OutputShape>::new();
    for &i in order {
        let node = &definition.transforms[i];
        let input = input_shape(node.input, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(input, data, &node.statistic, &node.filters)
            .map_err(|e| context(e, data, None))?;
        shapes.insert(node.id, output);
    }
    let mut domains = DomainContributions::default();
    for layer in &definition.layers {
        let input = input_shape(layer.data, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(input, data, &layer.statistic, &layer.filters)
            .map_err(|e| context(e, data, Some(layer.id)))?;
        let binding = match (&output.bins, &layer.mappings) {
            (None, Mappings::Source(aes)) => {
                source_binding(layer, aes, &definition.mappings, data).map(|(_, domain)| domain)
            }
            (Some(space), Mappings::Binned(aes)) => bin_binding(layer, aes, space),
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Aesthetic stage does not match the declared source/generated output schema.",
            )),
        }
        .map_err(|e| context(e, data, Some(layer.id)))?;
        merge_domains(&mut domains, &binding).map_err(|e| context(e, data, Some(layer.id)))?;
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
) -> ChartResult<OutputShape> {
    if !filters.is_empty() && input.bins.is_some() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Filter source observations before generating statistical rows.",
        ));
    }
    for filter in filters {
        numeric_space(data, &filter.value)?;
    }
    if let StatParameters::Bin(spec) = &stat.parameters {
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
