use super::*;
use crate::data::{DatasetSnapshot, SnapshotHandle, StoreSnapshot};
use crate::state::ChartState;
use crate::{ChartResult, DiagnosticCode, FieldId};
use std::{collections::BTreeSet, sync::Arc};

/// Stable typed facet identity, independent of panel order, bounds and source revisions.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct PanelKey {
    /// One exact non-null value for each declared facet field.
    pub values: Vec<GroupValue>,
}

/// Panel arrangement; grids retain explicit row and column ordering from the key catalog.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum FacetLayout {
    /// Fill successive rows using this positive number of columns.
    Wrap {
        /// Maximum panels across one row.
        columns: usize,
    },
    /// Exactly two fields; first identifies rows and second identifies columns.
    Grid,
}

/// Whether panels with no matched source population occupy their authored slots.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum EmptyPanels {
    /// Preserve authored empty panels.
    Keep,
    /// Omit empty panels; wrap packs remaining panels, grid preserves row/column coordinates.
    Drop,
}

/// Independent horizontal and vertical training policies across the figure.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacetScales {
    /// Train x separately in each panel when true.
    pub free_x: bool,
    /// Train y separately in each panel when true.
    pub free_y: bool,
}

/// Explicit panel catalog and figure layout. Undeclared non-null observed keys reject.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacetSpec {
    /// Optional reference facet semantics; absent retains explicit legacy catalogs and targeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<FacetPolicy>,
    /// One field for wrap, or two fields for a row/column grid.
    pub fields: Vec<FieldId>,
    /// Exact panel order and retained empty keys; keys must be unique.
    pub order: Vec<PanelKey>,
    /// Wrap or basic row/column grid.
    pub layout: FacetLayout,
    /// Explicit empty-panel policy.
    pub empty: EmptyPanels,
    /// Shared scales by default; independent policies require explicit opt-in.
    pub scales: FacetScales,
    /// Nonnegative gap in destination units.
    pub gap: f64,
    /// Collect compatible color legends once outside the panels.
    pub collect_guides: bool,
}

/// A layer or transform must explicitly opt into input data that lacks facet fields.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum FacetTarget {
    /// Filter source rows to the panel key; every facet field must exist.
    #[default]
    Match,
    /// Repeat the complete filtered input in all panels.
    Broadcast,
    /// Repeat the complete filtered input only in these stable panels.
    Panels(Vec<PanelKey>),
}

/// Statistical population scope; source filters always run before statistics.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum StatScope {
    /// Configured groups within each matched facet population.
    #[default]
    Group,
    /// One population per facet, overriding the statistic's grouping.
    Facet,
    /// One chart population, overriding facet filtering and statistical grouping.
    Chart,
}

/// One immutable panel, retaining the same coherent source snapshot as its figure.
#[derive(Clone, Debug)]
pub struct PreparedPanel {
    /// Stable horizontal scale-sharing population.
    pub x_group: GroupValue,
    /// Stable vertical scale-sharing population.
    pub y_group: GroupValue,
    /// Stable semantic panel key.
    pub key: PanelKey,
    /// Zero-based layout row, independent of source row positions.
    pub row: usize,
    /// Zero-based layout column.
    pub column: usize,
    /// Prepared layers and transforms for this population.
    pub chart: Arc<PreparedChart>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PanelScope {
    pub fields: Vec<FieldId>,
    pub key: PanelKey,
    pub names: Vec<String>,
    pub axis_group: bool,
}
impl PanelScope {
    pub(crate) fn new(spec: &FacetSpec, key: PanelKey) -> Self {
        Self {
            axis_group: false,
            fields: spec.fields.clone(),
            key,
            names: spec
                .reference
                .as_ref()
                .map(|p| p.field_names.clone())
                .unwrap_or_default(),
        }
    }
}

pub(crate) fn scoped_stat(stat: &Statistic, scope: StatScope) -> Statistic {
    let mut stat = stat.clone();
    if scope != StatScope::Group {
        match &mut stat.parameters {
            StatParameters::Identity => {}
            StatParameters::Distribution(s) => s.grouping = Grouping::All,
            StatParameters::Spatial(s) => s.grouping = Grouping::All,
            StatParameters::Model(s) => s.grouping = Grouping::All,
            StatParameters::Univariate(s) => s.grouping = Grouping::All,
            StatParameters::Custom(s) => s.grouping = Grouping::All,
            StatParameters::AutoBin(s) => s.grouping = Grouping::All,
            StatParameters::Bin(s) => s.grouping = Grouping::All,
            StatParameters::Count(s) => s.grouping = Grouping::All,
            StatParameters::Summary(s) => s.grouping = Grouping::All,
            StatParameters::Ols(s) => s.grouping = Grouping::All,
        }
    }
    stat
}

pub(crate) fn row_matches(row: crate::data::RowView<'_>, scope: &PanelScope) -> bool {
    scope
        .fields
        .iter()
        .zip(&scope.key.values)
        .enumerate()
        .all(|(index, (field, value))| {
            if scope.names.is_empty() {
                return stats::group_value(row, &Grouping::Field(*field)).as_ref() == Some(value);
            }
            if *value == GroupValue::All {
                return true;
            }
            let schema = row.chunk.batch().schema();
            let Some(actual) = schema
                .fields()
                .iter()
                .find(|f| f.name == scope.names[index])
                .map(|f| f.id)
            else {
                return true;
            };
            super::facet_policy::row_value(row, actual) == *value
        })
}

pub(crate) fn filter_panel(
    table: Arc<PreparedTable>,
    data: &DatasetSnapshot,
    scope: Option<&PanelScope>,
    target: &FacetTarget,
    stat_scope: StatScope,
) -> ChartResult<Arc<PreparedTable>> {
    let Some(scope) = scope else { return Ok(table) };
    if stat_scope == StatScope::Chart || *target != FacetTarget::Match {
        return Ok(table);
    }
    let PreparedRows::Source(rows) = &table.rows else {
        return Ok(table);
    };
    for (index, field) in scope.fields.iter().enumerate() {
        let actual = if scope.names.is_empty() {
            Some(*field)
        } else {
            data.schema()
                .fields()
                .iter()
                .find(|f| f.name == scope.names[index])
                .map(|f| f.id)
        };
        if let Some(field) = actual {
            stats::validate_group(
                data,
                &if scope.names.is_empty() {
                    Grouping::Field(field)
                } else {
                    Grouping::Interaction(vec![field])
                },
            )?;
        }
    }
    let rows = rows
        .iter()
        .filter(|row| data.row(row.key).is_some_and(|row| row_matches(row, scope)))
        .cloned()
        .collect();
    Ok(Arc::new(PreparedTable {
        rows: PreparedRows::Source(rows),
        ..(*table).clone()
    }))
}

pub(crate) fn targeted(target: &FacetTarget, scope: Option<&PanelScope>) -> bool {
    match (target, scope) {
        (FacetTarget::Panels(keys), Some(scope)) => {
            keys.contains(&scope.key)
                || scope.axis_group
                    && keys.iter().any(|k| {
                        k.values
                            .iter()
                            .zip(&scope.key.values)
                            .all(|(actual, wanted)| *wanted == GroupValue::All || actual == wanted)
                    })
        }
        _ => true,
    }
}

pub(super) fn validate_facets(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    limits: CompileLimits,
    extensions: &ExtensionRegistry,
) -> ChartResult<(Vec<crate::Diagnostic>, Vec<GroupValue>, Vec<GroupValue>)> {
    let spec = definition
        .facets
        .as_ref()
        .ok_or_else(|| error(DiagnosticCode::Validation, "Missing facet specification."))?;
    let expected_fields = if spec.reference.is_some() {
        spec.fields.len()
    } else if matches!(spec.layout, FacetLayout::Grid) {
        2
    } else {
        1
    };
    if expected_fields == 0
        || expected_fields > 32
        || spec.reference.as_ref().is_some_and(|p| {
            p.field_names.len() != expected_fields
                || p.row_fields > expected_fields
                || p.margins.iter().any(|i| *i >= expected_fields)
        })
        || spec.fields.len() != expected_fields
        || spec.fields.iter().collect::<BTreeSet<_>>().len() != expected_fields
        || spec.order.is_empty()
        || spec.order.len() > limits.max_groups.min(256)
        || !spec.gap.is_finite()
        || spec.gap < 0.
        || matches!(spec.layout, FacetLayout::Wrap { columns: 0 })
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Facets need one wrap/two grid fields, 1..256 bounded panels, positive columns and a finite nonnegative gap.",
        ));
    }
    if let Some(policy) = &spec.reference {
        if policy.levels.len() > expected_fields
            || policy.margins.iter().collect::<BTreeSet<_>>().len() != policy.margins.len()
            || policy.field_names.iter().any(|n| n.len() > 4096)
            || policy.labeller.separator.len() > 4096
            || policy.labeller.wrap_width == Some(0)
            || matches!(spec.layout, FacetLayout::Wrap { .. }) && policy.space == FacetSpace::Free
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Facet policy catalog, label or wrap-space controls are invalid.",
            ));
        }
        let bytes = policy
            .labeller
            .lookup
            .iter()
            .map(|(k, v)| k.len() + v.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>())
            .sum::<usize>();
        if bytes > 1024 * 1024 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Facet label lookup exceeds byte budget.",
            ));
        }
    }
    let mut keys = BTreeSet::new();
    for key in &spec.order {
        if key.values.len() != expected_fields
            || key.values.iter().any(|v| {
                (spec.reference.is_none() && matches!(v, GroupValue::All))
                    || matches!(v, GroupValue::Text(s) if s.len() > 4096)
            })
            || !keys.insert(key)
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Facet keys must be unique exact values matching the field list; labels are bounded to 4096 bytes.",
            ));
        }
    }
    for target in definition
        .layers
        .iter()
        .map(|l| &l.facet)
        .chain(definition.transforms.iter().map(|t| &t.facet))
    {
        if let FacetTarget::Panels(targets) = target
            && (targets.is_empty()
                || targets.len() > keys.len()
                || targets.iter().collect::<BTreeSet<_>>().len() != targets.len()
                || targets.iter().any(|k| !keys.contains(k)))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Target panels must be distinct keys from the figure catalog.",
            ));
        }
    }
    let population_diagnostics =
        validate_population(definition, snapshot, spec, limits, extensions)?;
    let mut rows = Vec::new();
    let mut columns = Vec::new();
    if matches!(spec.layout, FacetLayout::Grid) {
        for key in &spec.order {
            let row = if let Some(policy) = &spec.reference {
                super::facet_policy::side_key(key, 0, policy.row_fields)
            } else {
                key.values[0].clone()
            };
            let column = if let Some(policy) = &spec.reference {
                super::facet_policy::side_key(key, policy.row_fields, spec.fields.len())
            } else {
                key.values[1].clone()
            };
            if !rows.contains(&row) {
                rows.push(row);
            }
            if !columns.contains(&column) {
                columns.push(column);
            }
        }
        if rows.len().checked_mul(columns.len()) != Some(spec.order.len()) {
            return Err(error(
                DiagnosticCode::Validation,
                "A grid catalog must declare every row/column combination; use empty-panel policy for unpopulated cells.",
            ));
        }
    }
    Ok((population_diagnostics, rows, columns))
}

pub(crate) fn prepare_facets(
    compiler: &mut Compiler,
    definition: &ChartDefinition,
    source: &SnapshotHandle<StoreSnapshot>,
    state: &ChartState,
    limits: CompileLimits,
) -> ChartResult<PreparedChart> {
    let resolved =
        super::facet_policy::resolve(definition, source.get()?, limits, &compiler.extensions)?;
    let definition = resolved.as_ref();
    let (mut population_diagnostics, rows, columns) =
        validate_facets(definition, source.get()?, limits, &compiler.extensions)?;
    let spec = definition.facets.as_ref().expect("validated facets");
    if let Some(policy) = &spec.reference
        && let FacetLayout::Wrap { columns } = &spec.layout
        && policy.space != FacetSpace::Fixed
        && (*columns != spec.order.len() || policy.space == FacetSpace::FreeY)
    {
        let mut diagnostic = error(
            DiagnosticCode::Validation,
            "Reference free-space wrap normalizes the authored row/column arrangement; use one row for free_x or one column for free_y.",
        );
        diagnostic.severity = crate::Severity::Warning;
        population_diagnostics.push(diagnostic);
    }
    let mut child = definition.clone();
    child.facets = None;
    let mut panels = vec![];
    let mut metrics = PreparationMetrics::default();
    let mut remaining = limits.max_prepared_rows;
    let mut vertices = limits.max_vertices;
    let mut populations = vec![];
    let mut definitions = vec![];
    for key in &spec.order {
        let scope = PanelScope {
            axis_group: false,
            fields: spec.fields.clone(),
            names: spec
                .reference
                .as_ref()
                .map(|p| p.field_names.clone())
                .unwrap_or_default(),
            key: key.clone(),
        };
        let mut panel_definition = if super::semantics::needs_panel_training(definition) {
            super::semantics::resolve_scoped(
                definition,
                source.get()?,
                limits,
                &compiler.extensions,
                Some(&scope),
                false,
            )?
            .into_owned()
        } else {
            child.clone()
        };
        panel_definition.facets = None;
        definitions.push(panel_definition);
    }
    if definition
        .axes
        .iter()
        .any(super::positional_vectors::selected)
    {
        let panel_axes = definitions
            .iter()
            .map(|d| d.axes.clone())
            .collect::<Vec<_>>();
        let mut cache = super::positional_vectors::SourceCache::default();
        let mut transform_caches = std::collections::BTreeMap::new();
        for (panel_index, panel) in definitions.iter_mut().enumerate() {
            let mut context = definition.clone();
            context.axes = panel.axes.clone();
            super::positional_vectors::source_transforms(
                panel,
                &context,
                source.get()?,
                &compiler.extensions,
                limits,
                Some((panel_index, &panel_axes)),
                &mut transform_caches,
            )?;
            for layer in &mut panel.layers {
                super::positional_vectors::source_layer(
                    layer,
                    &context,
                    source.get()?,
                    &compiler.extensions,
                    limits,
                    Some((panel_index, &panel_axes)),
                    &mut cache,
                )?;
            }
        }
    }
    super::ggplot_bin_training::resolve(&mut definitions, source.get()?, limits, Some(spec))?;
    for (key, panel_definition) in spec.order.iter().zip(&definitions) {
        let scope = PanelScope {
            axis_group: false,
            fields: spec.fields.clone(),
            names: spec
                .reference
                .as_ref()
                .map(|p| p.field_names.clone())
                .unwrap_or_default(),
            key: key.clone(),
        };
        let population = compiler.prepare_scope(
            panel_definition,
            source,
            CompileLimits {
                max_prepared_rows: remaining,
                ..limits
            },
            Some(&scope),
        )?;
        remaining = remaining
            .checked_sub(population.work_units())
            .ok_or_else(|| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "Figure prepared-row budget exceeded.",
                )
            })?;
        populations.push((key, population));
    }
    if definition.profile() == Profile::Ggplot2_4_0_3 {
        let mut summaries = std::collections::BTreeMap::new();
        for (_, population) in &populations {
            for (layer, table) in population.layer_tables() {
                if table
                    .population_operation()
                    .is_some_and(|op| matches!(op.parameters, StatParameters::Summary(_)))
                {
                    let populated = matches!(table.rows(), PreparedRows::Statistical(rows) if rows.iter().any(|row| row.count > 0));
                    *summaries.entry(layer.id).or_insert(false) |= populated
                        || source
                            .get()?
                            .dataset(table.input().dataset)?
                            .rows()
                            .next()
                            .is_none();
                }
            }
        }
        if summaries.values().any(|populated| !populated) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "A faceted summary has no usable observations in any panel.",
            ));
        }
    }
    let samples = super::colors::shared_samples(
        &child,
        source.get()?,
        populations.iter().flat_map(|(_, p)| p.layer_tables()),
        limits,
    )?;
    for ((_, population), panel_definition) in populations.iter_mut().zip(&definitions) {
        compiler.encode_scope(panel_definition, source, limits, population)?;
    }
    super::compiler::transform_generated_scopes(
        &mut populations
            .iter_mut()
            .zip(&definitions)
            .map(|((_, population), definition)| (definition, population))
            .collect::<Vec<_>>(),
    )?;
    super::compiler::synchronize_position_populations(
        &mut definitions,
        &populations
            .iter()
            .map(|(_, population)| population)
            .collect::<Vec<_>>(),
    );
    let mut positioned = Vec::new();
    let mut keys = Vec::new();
    for ((key, population), panel_definition) in populations.into_iter().zip(&definitions) {
        keys.push(key);
        positioned.push(compiler.position_scope(
            panel_definition,
            source,
            state,
            limits,
            population,
            &samples,
        )?);
    }
    super::compiler::retain_facet_raw_domains(
        definition,
        &definitions,
        &mut positioned,
        source.get()?,
        limits,
    )?;
    super::compiler::train_facet_positioned_scopes(
        definition,
        &definitions,
        &mut positioned,
        &compiler.extensions,
        limits,
    )?;
    for ((key, population), panel_definition) in keys.into_iter().zip(positioned).zip(&definitions)
    {
        let chart = compiler.finish_positioned_scope(
            panel_definition,
            source,
            state,
            CompileLimits {
                max_vertices: vertices,
                ..limits
            },
            population,
            &samples,
        )?;
        metrics.evaluated_transforms += chart.metrics.evaluated_transforms;
        metrics.reused_transforms += chart.metrics.reused_transforms;
        metrics.evaluated_layers += chart.metrics.evaluated_layers;
        metrics.reused_layers += chart.metrics.reused_layers;
        let used_vertices = chart
            .layers
            .iter()
            .flat_map(|l| l.marks.iter())
            .map(|m| match &m.geometry {
                PreparedGeometry::Point(_) | PreparedGeometry::UnboundedPoint(_) => 1,
                PreparedGeometry::ShapePath { geometry, .. } => {
                    geometry.commands().len().saturating_add(1)
                }
                PreparedGeometry::ShapePathRun {
                    geometry, anchors, ..
                } => geometry.commands().len().saturating_add(anchors.len()),
                PreparedGeometry::Rectangle { .. } | PreparedGeometry::Bar { .. } => 4,
                PreparedGeometry::LineRun(v) | PreparedGeometry::Polygon(v) => v.len(),
                PreparedGeometry::BandRun { lower, upper }
                | PreparedGeometry::StackBandRun { lower, upper, .. } => lower.len() + upper.len(),
                _ => 2,
            })
            .sum::<usize>();
        vertices = vertices.checked_sub(used_vertices).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Figure geometry budget exceeded.",
            )
        })?;
        let populated = chart.layers.iter().any(|layer| {
            let matched = definition
                .layers
                .iter()
                .find(|l| l.id == layer.id)
                .is_some_and(|l| l.facet == FacetTarget::Match && l.scope != StatScope::Chart);
            matched
                && match layer.table.rows() {
                    PreparedRows::Source(rows) => !rows.is_empty(),
                    _ => layer.table.population_operation().is_some_and(|op| {
                        op.counts.input > op.counts.filtered + op.counts.invalid_filter
                    }),
                }
        });
        if spec.empty == EmptyPanels::Drop && !populated {
            population_diagnostics.extend_from_slice(chart.diagnostics());
            continue;
        }
        let (row, column) = match spec.layout {
            FacetLayout::Wrap { .. } => {
                super::facet_policy::coordinates(spec, panels.len(), spec.order.len())
            }
            FacetLayout::Grid => (
                rows.iter()
                    .position(|v| {
                        v == &spec
                            .reference
                            .as_ref()
                            .map(|p| super::facet_policy::side_key(key, 0, p.row_fields))
                            .unwrap_or_else(|| key.values[0].clone())
                    })
                    .unwrap_or(0),
                columns
                    .iter()
                    .position(|v| {
                        v == &spec
                            .reference
                            .as_ref()
                            .map(|p| {
                                super::facet_policy::side_key(key, p.row_fields, spec.fields.len())
                            })
                            .unwrap_or_else(|| key.values[1].clone())
                    })
                    .unwrap_or(0),
            ),
        };
        let row = if matches!(spec.layout, FacetLayout::Grid)
            && spec.reference.as_ref().is_some_and(|p| !p.as_table)
        {
            rows.len() - 1 - row
        } else {
            row
        };
        panels.push(PreparedPanel {
            x_group: super::facet_policy::sharing_group(spec, key, true),
            y_group: super::facet_policy::sharing_group(spec, key, false),
            key: key.clone(),
            row,
            column,
            chart: Arc::new(chart),
        });
    }
    let mut result = PreparedChart {
        positional_limits: Default::default(),
        positional_empty: Default::default(),
        scale_registrations: compiler.extensions.scales.clone(),
        key_registrations: compiler.extensions.keys.clone(),
        coordinate_registrations: compiler.extensions.coordinates.clone(),
        guide_drawing: compiler.extensions.guide_drawing.clone(),
        palette_registrations: compiler.extensions.palette_function.clone(),
        break_registrations: compiler.extensions.breaks_function.clone(),
        guide_registrations: compiler.extensions.guides.clone(),
        definition: Arc::new(definition.clone()),
        source: source.clone(),
        state: state.clone(),
        layers: vec![],
        transforms: Default::default(),
        domains: Default::default(),
        scale_domains: Default::default(),
        diagnostics: population_diagnostics,
        metrics,
        panels,
        shared_training: None,
    };
    let mut grouped = std::collections::BTreeMap::<
        GroupValue,
        std::collections::BTreeMap<crate::ScaleId, DomainContributions>,
    >::new();
    if spec.reference.is_some() {
        for panel in &result.panels {
            for layer in panel.chart.layers() {
                let d = compiler::eligible_domains(layer);
                for (id, horizontal, group) in [
                    (layer.scales.x, true, &panel.x_group),
                    (layer.scales.y, false, &panel.y_group),
                ] {
                    compiler::merge_axis(
                        grouped.entry(group.clone()).or_default(),
                        id,
                        horizontal,
                        &d,
                    )?;
                }
            }
        }
        for panel in &mut result.panels {
            let chart = Arc::make_mut(&mut panel.chart);
            for group in [&panel.x_group, &panel.y_group] {
                if let Some(domains) = grouped.get(group) {
                    for (id, d) in domains {
                        chart.scale_domains.insert(*id, d.clone());
                    }
                }
            }
        }
    }
    for panel in &result.panels {
        result
            .diagnostics
            .extend_from_slice(panel.chart.diagnostics());
        for layer in panel.chart.layers() {
            let d = compiler::eligible_domains(layer);
            for (id, horizontal) in [(layer.scales.x, true), (layer.scales.y, false)] {
                if (horizontal && !spec.scales.free_x) || (!horizontal && !spec.scales.free_y) {
                    compiler::merge_axis(&mut result.scale_domains, id, horizontal, &d)?;
                }
            }
            result.layers.push(layer.clone());
        }
    }
    Ok(result)
}

pub(crate) fn operation_scope(
    kind: &str,
    id: u64,
    stat: StatScope,
    panel: Option<&PanelScope>,
) -> String {
    let base = format!("{kind}:{id}");
    match panel.filter(|_| stat != StatScope::Chart) {
        Some(panel) => format!("{base}:panel:{panel:?}"),
        None => base,
    }
}

fn validate_population(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    spec: &FacetSpec,
    limits: CompileLimits,
    extensions: &ExtensionRegistry,
) -> ChartResult<Vec<crate::Diagnostic>> {
    #[derive(Clone)]
    struct Input {
        dataset: crate::DatasetId,
        filters: Vec<SourceFilter>,
        generated: bool,
        faceted: bool,
    }
    let order = compiler::validate_definition(definition, snapshot, limits, extensions)?;
    let mut inputs = std::collections::BTreeMap::<crate::TransformId, Input>::new();
    let resolve =
        |input: DataRef, inputs: &std::collections::BTreeMap<crate::TransformId, Input>| -> Input {
            match input {
                DataRef::Dataset(dataset) => Input {
                    dataset,
                    filters: vec![],
                    generated: false,
                    faceted: false,
                },
                DataRef::Transform(id) => inputs[&id].clone(),
            }
        };
    let validate_scope = |input: &Input,
                          target: &FacetTarget,
                          scope: StatScope|
     -> ChartResult<()> {
        if scope == StatScope::Chart && input.faceted {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A chart-scope operation cannot recover observations from a facet-scoped dependency.",
            ));
        }
        if *target == FacetTarget::Match {
            if input.generated && !input.faceted {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Chart-wide aggregate outputs cannot be refaceted; explicitly broadcast or target their presentation layer.",
                ));
            }
            let data = snapshot.dataset(input.dataset)?;
            for (index, field) in spec.fields.iter().enumerate() {
                if let Some(policy) = &spec.reference {
                    if let Some(field) = data
                        .schema()
                        .fields()
                        .iter()
                        .find(|f| f.name == policy.field_names[index])
                        .map(|f| f.id)
                    {
                        stats::validate_group(data, &Grouping::Interaction(vec![field]))?;
                    }
                    continue;
                }
                if data.schema().field(*field).is_none() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "A layer missing a facet field must explicitly Broadcast or name target Panels.",
                    ));
                }
                stats::validate_group(data, &Grouping::Field(*field))?;
            }
        }
        Ok(())
    };
    for index in order {
        let node = &definition.transforms[index];
        let mut input = resolve(node.input, &inputs);
        validate_scope(&input, &node.facet, node.scope)?;
        input.filters.extend_from_slice(&node.filters);
        input.generated |= !matches!(node.statistic.parameters, StatParameters::Identity);
        input.faceted |= node.facet == FacetTarget::Match && node.scope != StatScope::Chart;
        inputs.insert(node.id, input);
    }
    let mut diagnostics = vec![];
    for layer in &definition.layers {
        let mut input = resolve(layer.data, &inputs);
        validate_scope(&input, &layer.facet, layer.scope)?;
        if layer.facet != FacetTarget::Match {
            continue;
        }
        input.filters.extend_from_slice(&layer.filters);
        let data = snapshot.dataset(input.dataset)?;
        let mut invalid = 0;
        let mut samples = vec![];
        for row in data.rows().filter(|row| {
            input
                .filters
                .iter()
                .all(|filter| stats::filter_matches(*row, filter) == Some(true))
        }) {
            if spec.reference.is_some() {
                if !spec
                    .order
                    .iter()
                    .any(|key| row_matches(row, &PanelScope::new(spec, key.clone())))
                {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Observed reference facet values are absent from the panel catalog.",
                    ));
                }
                continue;
            }
            let values = spec
                .fields
                .iter()
                .map(|field| stats::group_value(row, &Grouping::Field(*field)))
                .collect::<Option<Vec<_>>>();
            let Some(values) = values else {
                invalid += 1;
                if samples.len() < 32 {
                    samples.push(row.key());
                }
                continue;
            };
            if !spec.order.contains(&PanelKey { values }) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "An observed filtered facet key is absent from the explicit panel catalog.",
                ));
            }
        }
        let before = diagnostics.len();
        stats::warning(
            layer.invalid,
            invalid,
            samples,
            "Null facet values are excluded from explicitly keyed panels.",
            &mut diagnostics,
        )?;
        for diagnostic in &mut diagnostics[before..] {
            diagnostic.context.dataset = Some(input.dataset);
            diagnostic.context.layer = Some(layer.id);
            diagnostic.context.dataset_revision = Some(data.version().revision);
            diagnostic.context.schema_version = Some(data.version().schema_version);
        }
    }
    Ok(diagnostics)
}

pub(crate) fn population_panel<'a>(
    scope: Option<&'a PanelScope>,
    target: &FacetTarget,
    stat: StatScope,
) -> Option<&'a PanelKey> {
    scope
        .filter(|_| stat != StatScope::Chart && *target == FacetTarget::Match)
        .map(|s| &s.key)
}
pub(crate) fn source_table(
    data: &DatasetSnapshot,
    max_rows: usize,
    scope: Option<&PanelScope>,
    target: &FacetTarget,
    stat: StatScope,
) -> ChartResult<Arc<PreparedTable>> {
    let Some(scope) = scope.filter(|_| stat != StatScope::Chart && *target == FacetTarget::Match)
    else {
        return stats::source_table(data, max_rows);
    };
    for (index, field) in scope.fields.iter().enumerate() {
        let actual = if scope.names.is_empty() {
            Some(*field)
        } else {
            data.schema()
                .fields()
                .iter()
                .find(|f| f.name == scope.names[index])
                .map(|f| f.id)
        };
        if let Some(field) = actual {
            stats::validate_group(
                data,
                &if scope.names.is_empty() {
                    Grouping::Field(field)
                } else {
                    Grouping::Interaction(vec![field])
                },
            )?;
        }
    }
    stats::source_table_where(data, max_rows, |row| row_matches(row, scope))
}
