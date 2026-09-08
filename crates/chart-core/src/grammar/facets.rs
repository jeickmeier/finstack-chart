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
}

pub(crate) fn scoped_stat(stat: &Statistic, scope: StatScope) -> Statistic {
    let mut stat = stat.clone();
    if scope != StatScope::Group {
        match &mut stat.parameters {
            StatParameters::Identity => {}
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
    for field in &scope.fields {
        stats::validate_group(data, &Grouping::Field(*field))?;
    }
    let rows = rows
        .iter()
        .filter(|row| {
            data.row(row.key).is_some_and(|row| {
                scope
                    .fields
                    .iter()
                    .zip(&scope.key.values)
                    .all(|(field, value)| {
                        stats::group_value(row, &Grouping::Field(*field)).as_ref() == Some(value)
                    })
            })
        })
        .cloned()
        .collect();
    Ok(Arc::new(PreparedTable {
        rows: PreparedRows::Source(rows),
        ..(*table).clone()
    }))
}

pub(crate) fn targeted(target: &FacetTarget, scope: Option<&PanelScope>) -> bool {
    match (target, scope) {
        (FacetTarget::Panels(keys), Some(scope)) => keys.contains(&scope.key),
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
    let expected_fields = if matches!(spec.layout, FacetLayout::Grid) {
        2
    } else {
        1
    };
    if spec.fields.len() != expected_fields
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
    let mut keys = BTreeSet::new();
    for key in &spec.order {
        if key.values.len() != expected_fields
            || key.values.iter().any(|v| {
                matches!(v, GroupValue::All) || matches!(v, GroupValue::Text(s) if s.len() > 4096)
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
            if !rows.contains(&key.values[0]) {
                rows.push(key.values[0].clone());
            }
            if !columns.contains(&key.values[1]) {
                columns.push(key.values[1].clone());
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
    let (mut population_diagnostics, rows, columns) =
        validate_facets(definition, source.get()?, limits, &compiler.extensions)?;
    let spec = definition.facets.as_ref().expect("validated facets");
    let mut child = definition.clone();
    child.facets = None;
    let mut panels = vec![];
    let mut metrics = PreparationMetrics::default();
    let mut remaining = limits.max_prepared_rows;
    let mut vertices = limits.max_vertices;
    for key in &spec.order {
        let scope = PanelScope {
            fields: spec.fields.clone(),
            key: key.clone(),
        };
        let chart = compiler.prepare_scoped(
            &child,
            source,
            state,
            CompileLimits {
                max_prepared_rows: remaining,
                max_vertices: vertices,
                ..limits
            },
            Some(&scope),
        )?;
        metrics.evaluated_transforms += chart.metrics.evaluated_transforms;
        metrics.reused_transforms += chart.metrics.reused_transforms;
        let used = chart
            .layers
            .iter()
            .map(|l| l.table.work_units())
            .chain(chart.transforms.values().map(|t| t.work_units()))
            .sum::<usize>();
        remaining = remaining.checked_sub(used).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Figure prepared-row budget exceeded.",
            )
        })?;
        let used_vertices = chart
            .layers
            .iter()
            .flat_map(|l| l.marks.iter())
            .map(|m| match &m.geometry {
                PreparedGeometry::Point(_) => 1,
                PreparedGeometry::LineRun(v) => v.len(),
                PreparedGeometry::BandRun { lower, upper } => lower.len() + upper.len(),
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
                    _ => layer
                        .table
                        .operations()
                        .iter()
                        .rev()
                        .find(|op| !matches!(op.parameters, StatParameters::Identity))
                        .is_some_and(|op| {
                            op.counts.input > op.counts.filtered + op.counts.invalid_filter
                        }),
                }
        });
        if spec.empty == EmptyPanels::Drop && !populated {
            population_diagnostics.extend_from_slice(chart.diagnostics());
            continue;
        }
        let (row, column) = match spec.layout {
            FacetLayout::Wrap { columns } => (panels.len() / columns, panels.len() % columns),
            FacetLayout::Grid => (
                rows.iter().position(|v| v == &key.values[0]).unwrap_or(0),
                columns
                    .iter()
                    .position(|v| v == &key.values[1])
                    .unwrap_or(0),
            ),
        };
        panels.push(PreparedPanel {
            key: key.clone(),
            row,
            column,
            chart: Arc::new(chart),
        });
    }
    let mut result = PreparedChart {
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
            for field in &spec.fields {
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
    for field in &scope.fields {
        stats::validate_group(data, &Grouping::Field(*field))?;
    }
    stats::source_table_where(data, max_rows, |row| {
        scope
            .fields
            .iter()
            .zip(&scope.key.values)
            .all(|(field, value)| {
                stats::group_value(row, &Grouping::Field(*field)).as_ref() == Some(value)
            })
    })
}
