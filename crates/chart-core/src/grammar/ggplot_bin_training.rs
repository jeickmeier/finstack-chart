//! Fixed-facet automatic bins share the filtered pre-statistic position population.
use super::*;
use crate::data::StoreSnapshot;
use crate::{ChartResult, DiagnosticCode, ScaleId};
use std::collections::BTreeMap;

fn input(stat: &Statistic) -> Option<(&Numeric, &StatSpace)> {
    match &stat.parameters {
        StatParameters::AutoBin(s) if s.ggplot.is_some() => Some((&s.input, &s.space)),
        StatParameters::Summary(s) => s
            .ggplot
            .as_ref()
            .filter(|g| g.bins.as_ref().is_some_and(|b| b.breaks.is_none()))
            .and_then(|g| g.position.as_ref())
            .map(|n| (n, &StatSpace::Data)),
        _ => None,
    }
}
fn target_axis(layer: &Layer) -> Option<ScaleId> {
    let normalized = match &layer.statistic.parameters {
        StatParameters::Distribution(s) => s.sample_axis(),
        StatParameters::Univariate(s)
            if matches!(
                s.kind,
                UnivariateKind::Function { .. }
                    | UnivariateKind::Qq {
                        line: true,
                        full_range: true,
                        ..
                    }
            ) =>
        {
            0
        }
        _ if input(&layer.statistic).is_some() => 0,
        _ => return None,
    };
    Some(
        if (normalized == 0) == (layer.orientation == Orientation::Vertical) {
            layer.scales.x
        } else {
            layer.scales.y
        },
    )
}
fn root_input<'a>(
    mut input: DataRef,
    definition: &'a ChartDefinition,
    filters: &mut Vec<&'a SourceFilter>,
) -> ChartResult<crate::DatasetId> {
    for _ in 0..=definition.transforms.len() {
        match input {
            DataRef::Dataset(id) => return Ok(id),
            DataRef::Transform(id) => {
                let t = definition
                    .transforms
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "Unknown automatic-bin source transform.",
                        )
                    })?;
                if !matches!(t.statistic.parameters, StatParameters::Identity) {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Automatic bins need source observations after identity/filter transforms.",
                    ));
                }
                filters.extend(&t.filters);
                input = t.input;
            }
        }
    }
    Err(error(
        DiagnosticCode::SchemaConflict,
        "Cyclic automatic-bin source graph.",
    ))
}
pub(super) fn needed(definition: &ChartDefinition) -> bool {
    definition.layers.iter().any(|l| target_axis(l).is_some())
}
fn independent_axis(layer: &Layer) -> ScaleId {
    if layer.orientation == Orientation::Horizontal {
        layer.scales.y
    } else {
        layer.scales.x
    }
}
fn source_inputs(layer: &Layer, normalized_x: bool) -> Vec<(&Numeric, &StatSpace)> {
    if let Mappings::Source(a) = &layer.mappings {
        let fields = if normalized_x {
            vec![a.x.as_ref(), a.x2.as_ref()]
        } else {
            vec![a.y.as_ref(), a.y2.as_ref(), a.low.as_ref(), a.high.as_ref()]
        };
        fields
            .into_iter()
            .flatten()
            .map(|n| (n, &StatSpace::Data))
            .collect()
    } else {
        vec![]
    }
}
fn training_inputs(layer: &Layer, axis: ScaleId) -> Vec<(&Numeric, &StatSpace)> {
    let normalized = usize::from(axis != independent_axis(layer));
    match &layer.statistic.parameters {
        StatParameters::Distribution(s) => {
            return if normalized == s.sample_axis() {
                vec![(&s.input, &s.space)]
            } else {
                s.position
                    .as_ref()
                    .map(|p| vec![(p, &s.position_space)])
                    .unwrap_or_default()
            };
        }
        StatParameters::Univariate(s) => {
            return if normalized == s.sample_axis() {
                vec![(&s.input, &s.space)]
            } else {
                s.second
                    .as_ref()
                    .map(|p| vec![(p, &s.second_space)])
                    .unwrap_or_default()
            };
        }
        _ => {}
    }
    if axis != independent_axis(layer) {
        return match &layer.statistic.parameters {
            StatParameters::Ols(s) => vec![(&s.y, &s.y_space)],
            StatParameters::Summary(s) => vec![(&s.input, &s.space)],
            _ => source_inputs(layer, false),
        };
    }
    if let Some(v) = input(&layer.statistic) {
        return vec![v];
    }
    match &layer.statistic.parameters {
        StatParameters::Bin(s) => vec![(&s.input, &s.space)],
        StatParameters::Ols(s) => vec![(&s.x, &s.x_space)],
        StatParameters::Count(s) => s
            .ggplot
            .as_ref()
            .and_then(|g| g.position.as_ref())
            .map(|n| vec![(n, &StatSpace::Data)])
            .unwrap_or_default(),
        StatParameters::Summary(s) => s
            .ggplot
            .as_ref()
            .and_then(|g| g.position.as_ref())
            .map(|n| vec![(n, &StatSpace::Data)])
            .unwrap_or_default(),
        _ => source_inputs(layer, true),
    }
}
pub(super) fn resolve(
    definitions: &mut [ChartDefinition],
    source: &StoreSnapshot,
    limits: CompileLimits,
    spec: Option<&FacetSpec>,
) -> ChartResult<()> {
    for layer in definitions.iter().flat_map(|d| &d.layers) {
        if target_axis(layer).is_some() {
            super::stats::validate_stat(&layer.statistic, limits)?;
        }
    }
    let targets = definitions
        .iter()
        .flat_map(|d| d.layers.iter())
        .filter_map(|l| target_axis(l).map(|axis| (axis, axis == l.scales.x)))
        .collect::<BTreeMap<_, _>>();
    if targets.is_empty() {
        return Ok(());
    }
    let mut ranges: BTreeMap<(usize, ScaleId), (f64, f64)> = BTreeMap::new();
    let mut authored = BTreeMap::new();
    let key = |index: usize, id: ScaleId| {
        (
            if let Some(spec) = spec.filter(|s| s.reference.is_some()) {
                let group =
                    super::facet_policy::sharing_group(spec, &spec.order[index], targets[&id]);
                spec.order
                    .iter()
                    .position(|key| {
                        super::facet_policy::sharing_group(spec, key, targets[&id]) == group
                    })
                    .unwrap_or(index)
            } else if spec.is_some_and(|s| {
                if targets[&id] {
                    s.scales.free_x
                } else {
                    s.scales.free_y
                }
            }) {
                index
            } else {
                0
            },
            id,
        )
    };
    for (panel_index, definition) in definitions.iter().enumerate() {
        let scope = spec.map(|spec| super::facets::PanelScope {
            axis_group: false,
            fields: spec.fields.clone(),
            names: spec
                .reference
                .as_ref()
                .map(|p| p.field_names.clone())
                .unwrap_or_default(),
            key: spec.order[panel_index].clone(),
        });
        for layer in &definition.layers {
            if !super::facets::targeted(&layer.facet, scope.as_ref()) {
                continue;
            }
            for axis in [layer.scales.x, layer.scales.y] {
                if !targets.contains_key(&axis) {
                    continue;
                }
                let inputs = training_inputs(layer, axis);
                if inputs.is_empty() {
                    continue;
                }
                let mut filters = layer.filters.iter().collect::<Vec<_>>();
                let id = root_input(layer.data, definition, &mut filters)?;
                let data = source.dataset(id)?;
                for (numeric, space) in inputs {
                    let mut extend = |v: f64| {
                        if v.is_finite() {
                            ranges
                                .entry(key(panel_index, axis))
                                .and_modify(|r| {
                                    r.0 = r.0.min(v);
                                    r.1 = r.1.max(v);
                                })
                                .or_insert((v, v));
                        }
                    };
                    for row in data.rows().filter(|_| !matches!(&layer.statistic.parameters, StatParameters::Univariate(s) if s.default_input)) {
                        if layer.scope != StatScope::Chart
                            && layer.facet == FacetTarget::Match
                            && scope
                                .as_ref()
                                .is_some_and(|s| !super::facets::row_matches(row, s))
                        {
                            continue;
                        }
                        if !filters
                            .iter()
                            .all(|f| super::stats::filter_matches(row, f) == Some(true))
                        {
                            continue;
                        }
                        if let Some(mut v) = super::stats::number(row, numeric) {
                            if let StatSpace::Transformed(t) = space {
                                v = v * t.factor + t.offset;
                            }
                            extend(v);
                        }
                    }
                    if let Numeric::Scaled { scale, .. } = numeric
                        && let Some(bounds) = scale.limits
                        && let (Some(a), Some(b)) =
                            (scale.project(bounds.start()), scale.project(bounds.end()))
                    {
                        authored.insert(key(panel_index, axis), (a.min(b), a.max(b)));
                    }
                    if let Numeric::Scaled { scale, .. } = numeric
                        && let Some(endpoints) = &scale.function_limits
                        && let (Some(a), Some(b)) = (endpoints.first(), endpoints.last())
                        && a.0.is_finite()
                        && b.0.is_finite()
                    {
                        authored.insert(key(panel_index, axis), (a.0.min(b.0), a.0.max(b.0)));
                    }
                }
            }
        }
    }
    ranges.extend(authored);
    for (panel_index, definition) in definitions.iter_mut().enumerate() {
        for layer in &mut definition.layers {
            let Some(axis) = target_axis(layer) else {
                continue;
            };
            let Some((lo, hi)) = ranges.get(&key(panel_index, axis)).copied() else {
                continue;
            };
            match &mut layer.statistic.parameters {
                StatParameters::Distribution(s) => s.training_range = Some([lo, hi]),
                StatParameters::Univariate(s) => s.training_range = Some([lo, hi]),
                StatParameters::AutoBin(s) if s.ggplot.is_some() => {
                    let edges = super::ggplot_stats::automatic_edges(
                        lo,
                        hi,
                        s.bins,
                        s.ggplot.as_ref().unwrap(),
                    )?;
                    if edges.len() > limits.max_edges {
                        return Err(error(
                            DiagnosticCode::ResourceLimit,
                            "Shared automatic bin edges exceed budget.",
                        ));
                    }
                    layer.statistic = Statistic::bin(BinSpec {
                        input: s.input.clone(),
                        edges,
                        ggplot: s.ggplot.clone(),
                        outliers: OutlierPolicy::Exclude,
                        grouping: s.grouping.clone(),
                        space: s.space.clone(),
                    });
                }
                StatParameters::Summary(s) => {
                    if let Some(g) = &mut s.ggplot
                        && let Some(b) = &mut g.bins
                        && b.breaks.is_none()
                    {
                        let edges =
                            super::ggplot_stats::automatic_edges(lo, hi, b.bins, &b.options)?;
                        if edges.len() > limits.max_edges {
                            return Err(error(
                                DiagnosticCode::ResourceLimit,
                                "Shared summary edges exceed budget.",
                            ));
                        }
                        b.breaks = Some(edges);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
