//! Ordered positional OOB callbacks before statistics and after generated mapping.
use super::*;
use crate::{
    ChartResult,
    data::StoreSnapshot,
    interpolate::Number,
    layout::{AxisScale, AxisSpec},
};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn selected(axis: &AxisSpec) -> bool {
    axis.oob_function.is_some()
        || match &axis.scale {
            AxisScale::Nonlinear { transform, .. } => transform
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise()),
            AxisScale::Binned { spec, .. } => spec
                .transform
                .as_ref()
                .and_then(|t| t.ggplot_transform())
                .is_some_and(|t| !t.is_pointwise()),
            _ => false,
        }
}

pub(super) fn authored_limits(axis: &AxisSpec) -> ChartResult<[Option<Number>; 2]> {
    let bounds = match axis.scale {
        AxisScale::Auto => None,
        AxisScale::Linear(d) | AxisScale::Duration(d) | AxisScale::Nonlinear { domain: d, .. } => {
            d.explicit
        }
        AxisScale::Date { domain } | AxisScale::Utc { domain, .. } => {
            return temporal_authored_limits(axis, domain.map(|d| [d.start, d.end]));
        }
        AxisScale::Calendar { ref spec, .. } => {
            return temporal_authored_limits(
                axis,
                spec.domain
                    .first()
                    .zip(spec.domain.last())
                    .map(|(a, b)| [*a, *b]),
            );
        }
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Positional vector functions require a numeric population scale.",
            ));
        }
    };
    Ok(bounds.map_or([None, None], |b| {
        [Some(Number(b.start())), Some(Number(b.end()))]
    }))
}
pub(super) fn train_limits(
    axis: &AxisSpec,
    transform: Option<crate::scales::ScaleTransform>,
    values: impl Iterator<Item = f64>,
    trained: bool,
) -> ChartResult<Vec<Number>> {
    let authored = authored_limits(axis)?;
    if !trained
        && authored.iter().all(Option::is_none)
        && let Some(time) = axis.resolved_temporal.as_deref()
    {
        return Ok([0., 1.].map(|v| Number(time.relative(v))).to_vec());
    }
    Ok(crate::scales::ggplot_numeric_limits::authored_transformed(
        authored, transform, values, trained,
    )?
    .to_vec())
}
fn temporal_authored_limits(
    axis: &AxisSpec,
    bounds: Option<[i64; 2]>,
) -> ChartResult<[Option<Number>; 2]> {
    let time = axis.resolved_temporal.as_deref().ok_or_else(|| {
        error(
            DiagnosticCode::SchemaConflict,
            "Temporal vector population units have not been prepared.",
        )
    })?;
    Ok(bounds.map_or([None, None], |bounds| {
        bounds.map(|v| Some(Number((i128::from(v) - i128::from(time.origin)) as f64)))
    }))
}
fn limits(axis: &AxisSpec) -> ChartResult<Vec<Number>> {
    if let AxisScale::Binned {
        prepared: Some(bins),
        ..
    } = &axis.scale
    {
        return Ok(axis
            .resolved_limits
            .as_deref()
            .cloned()
            .unwrap_or_else(|| bins.limits().to_vec()));
    }
    axis.resolved_limits.as_deref().cloned().ok_or_else(|| {
        error(
            DiagnosticCode::SchemaConflict,
            "Positional vector population limits have not been prepared.",
        )
    })
}
/// Apply one scale per ordered population, then restore layer row order before
/// the reference data-frame replacement/recycling rule. Empty populations are
/// invoked when another population in the same layer has observations.
pub(super) fn map_populations(
    populations: &[(&AxisSpec, Vec<(usize, Number)>)],
    count: usize,
    registry: &ExtensionRegistry,
) -> ChartResult<Vec<Number>> {
    if count == 0 {
        return Ok(vec![]);
    }
    let mut pieces = vec![];
    let mut indices = vec![];
    for (axis, population) in populations {
        let absolute = |v: Number| {
            Number(
                axis.resolved_temporal
                    .as_deref()
                    .map_or(v.0, |t| t.absolute(v.0)),
            )
        };
        let values = population
            .iter()
            .map(|(_, v)| absolute(*v))
            .collect::<Vec<_>>();
        let range = limits(axis)?.into_iter().map(absolute).collect::<Vec<_>>();
        let mut result = registry
            .scale_vectors
            .evaluate(
                axis.oob_function
                    .as_ref()
                    .expect("selected vector callback"),
                ScaleVectorStage::OutOfBounds,
                &values,
                &range,
            )?
            .unwrap_or_default();
        if let Some(time) = axis.resolved_temporal.as_deref() {
            for value in &mut result {
                value.0 = time.relative(value.0);
            }
        }
        if let AxisScale::Binned {
            prepared: Some(bins),
            ..
        } = &axis.scale
        {
            result = bins.project_vector_source(&result)?;
        } else if !matches!(axis.scale, AxisScale::Duration(_))
            && let Some(missing) = axis.population_missing
        {
            for value in &mut result {
                if value.0.is_nan() {
                    *value = missing;
                }
            }
        }
        pieces.extend(result);
        indices.extend(population.iter().map(|(i, _)| *i));
    }
    if pieces.is_empty() || pieces.len() > count || !count.is_multiple_of(pieces.len()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Combined positional vector output must be nonempty and divide the layer population length.",
        ));
    }
    let mut order = (0..indices.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| indices[*i]);
    let mapped = order
        .into_iter()
        .take(pieces.len())
        .map(|i| pieces.get(i).copied().unwrap_or(Number(f64::NAN)))
        .collect::<Vec<_>>();
    Ok((0..count).map(|i| mapped[i % mapped.len()]).collect())
}

struct SourcePopulation<'a> {
    input: DataRef,
    filters: Vec<&'a SourceFilter>,
    routes: Vec<(&'a FacetTarget, StatScope)>,
}
impl SourcePopulation<'_> {
    fn matched(&self) -> bool {
        self.routes
            .iter()
            .any(|(target, scope)| **target == FacetTarget::Match && *scope != StatScope::Chart)
    }
    fn targets(&self, key: &PanelKey) -> bool {
        self.routes.iter().all(|(target, _)| match target {
            FacetTarget::Panels(keys) => keys.contains(key),
            _ => true,
        })
    }
}
fn source_population<'a>(
    definition: &'a ChartDefinition,
    input: DataRef,
    filters: &'a [SourceFilter],
    target: &'a FacetTarget,
    scope: StatScope,
) -> SourcePopulation<'a> {
    let mut population = SourcePopulation {
        input,
        filters: filters.iter().collect(),
        routes: vec![(target, scope)],
    };
    for _ in 0..=definition.transforms.len() {
        let DataRef::Transform(id) = population.input else {
            break;
        };
        let node = definition
            .transforms
            .iter()
            .find(|node| node.id == id)
            .expect("validated transform dependency");
        if !matches!(node.statistic.parameters, StatParameters::Identity) {
            break;
        }
        population.filters.extend(&node.filters);
        population.routes.push((&node.facet, node.scope));
        population.input = node.input;
    }
    population
}
pub(super) fn matched_source(definition: &ChartDefinition, layer: &Layer) -> bool {
    source_population(
        definition,
        layer.data,
        &layer.filters,
        &layer.facet,
        layer.scope,
    )
    .matched()
}

type SourceSamples = Arc<BTreeMap<crate::RowKey, Number>>;
#[derive(Default)]
pub(super) struct SourceCache(Vec<(crate::LayerId, crate::ScaleId, Numeric, Vec<SourceSamples>)>);

pub(super) fn source_layer(
    layer: &mut Layer,
    definition: &ChartDefinition,
    source: &StoreSnapshot,
    registry: &ExtensionRegistry,
    budget: CompileLimits,
    panel_axes: Option<(usize, &[Vec<AxisSpec>])>,
    cache: &mut SourceCache,
) -> ChartResult<()> {
    let selected = definition
        .axes
        .iter()
        .filter(|a| [layer.scales.x, layer.scales.y].contains(&a.id) && selected(a))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Ok(());
    }
    if definition.profile() != Profile::Ggplot2_4_0_3
        || selected
            .iter()
            .any(|a| a.scale_stage == Some(ScaleStage::AfterStatistics))
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Positional vector functions require a ggplot2 pre-statistic scale.",
        ));
    }
    for axis in &selected {
        if axis.oob_function.is_some() {
            limits(axis)?;
        }
    }
    let population = source_population(
        definition,
        layer.data,
        &layer.filters,
        &layer.facet,
        layer.scope,
    );
    let matched = population.matched();
    let active = definition.facets.as_ref().map(|facets| {
        facets
            .order
            .iter()
            .enumerate()
            .filter_map(|(index, key)| population.targets(key).then_some(index))
            .collect::<Vec<_>>()
    });
    let input = population.input;
    let data = match input {
        DataRef::Dataset(id) => Some(source.dataset(id)?),
        DataRef::Transform(_) => None,
    };
    let mut remaining = budget.max_prepared_rows;
    let mut rows = vec![];
    if let Some(data) = data {
        for row in data.rows().filter(|r| {
            population
                .filters
                .iter()
                .all(|f| super::stats::filter_matches(*r, f) == Some(true))
                && (!matched
                    || definition.facets.as_ref().is_none_or(|facets| {
                        let active = active.as_ref().expect("facet targets");
                        active.len() == facets.order.len()
                            || active.iter().any(|index| {
                                super::facets::row_matches(
                                    *r,
                                    &super::facets::PanelScope {
                                        fields: facets.fields.clone(),
                                        key: facets.order[*index].clone(),
                                    },
                                )
                            })
                    }))
        }) {
            super::compiler::charge(&mut remaining, 1, "positional vector population")?;
            rows.push(row);
        }
    }
    let mut bind = |numeric: &mut Numeric| -> ChartResult<()> {
        let Numeric::Scaled {
            input,
            scale,
            samples,
        } = numeric
        else {
            return Ok(());
        };
        let Some(axis) = selected.iter().find(|a| a.id == scale.id) else {
            return Ok(());
        };
        if let Some((_, _, _, values)) = cache
            .0
            .iter()
            .find(|(id, axis, n, _)| *id == layer.id && *axis == scale.id && n == input.as_ref())
        {
            let panel = if values.len() == 1 {
                0
            } else {
                panel_axes.expect("panel vector cache").0
            };
            *samples = Some(values[panel].clone());
            return Ok(());
        }
        let data = data.ok_or_else(|| error(
                DiagnosticCode::UnsupportedCapability,
                "Positional source vectors require source observations; a statistical ancestor has generated fields.",
            ))?;
        super::stats::numeric_space(data, input)?;
        let inputs = rows
            .iter()
            .map(|r| super::stats::raw_number(*r, input).unwrap_or(f64::NAN))
            .collect::<Vec<_>>();
        let inputs = match scale.transform.as_ref().and_then(|t| t.ggplot_transform()) {
            Some(transform) => transform.forward_population(&inputs)?,
            None => inputs
                .into_iter()
                .map(|v| scale.transform.as_ref().map_or(v, |t| t.forward_raw(v)))
                .collect(),
        }
        .into_iter()
        .map(Number)
        .collect::<Vec<_>>();
        if axis.oob_function.is_none() {
            let values = Arc::new(
                rows.iter()
                    .zip(inputs)
                    .map(|(row, v)| {
                        (
                            row.key(),
                            Number(
                                scale
                                    .project_transformed_optional(Some(v.0))
                                    .unwrap_or(f64::NAN),
                            ),
                        )
                    })
                    .collect(),
            );
            *samples = Some(Arc::clone(&values));
            cache
                .0
                .push((layer.id, scale.id, input.as_ref().clone(), vec![values]));
            return Ok(());
        }
        let free = definition.facets.as_ref().is_some_and(|f| {
            if axis.side.horizontal() {
                f.scales.free_x
            } else {
                f.scales.free_y
            }
        });
        let values = if definition.facets.is_some() && !matched {
            let axes = panel_axes.expect("broadcast panel axes").1;
            let active = active.as_ref().expect("facet targets");
            let width = inputs.len();
            let count = rows.len().checked_mul(active.len()).ok_or_else(|| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "Broadcast positional vector population exceeds the row budget.",
                )
            })?;
            super::compiler::charge(
                &mut remaining,
                count,
                "broadcast positional vector population",
            )?;
            let populations = if free {
                active
                    .iter()
                    .enumerate()
                    .map(|(panel, index)| {
                        let axis = axes[*index]
                            .iter()
                            .find(|a| a.id == scale.id)
                            .expect("broadcast axis");
                        (
                            axis,
                            inputs
                                .iter()
                                .copied()
                                .enumerate()
                                .map(|(row, value)| (panel * rows.len() + row, value))
                                .collect(),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![(
                    *axis,
                    (0..active.len())
                        .flat_map(|panel| {
                            inputs
                                .iter()
                                .copied()
                                .enumerate()
                                .map(move |(row, value)| (panel * width + row, value))
                        })
                        .collect(),
                )]
            };
            let mapped = map_populations(&populations, count, registry)?;
            (0..axes.len())
                .map(|panel| {
                    let Some(offset) = active.iter().position(|index| *index == panel) else {
                        return Arc::new(BTreeMap::new());
                    };
                    Arc::new(
                        rows.iter()
                            .zip(&mapped[offset * rows.len()..(offset + 1) * rows.len()])
                            .map(|(row, value)| (row.key(), *value))
                            .collect(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            let populations = if free {
                let facets = definition.facets.as_ref().expect("free facet scale");
                let axes = panel_axes.expect("prepared panel axes").1;
                facets
                    .order
                    .iter()
                    .zip(axes)
                    .filter(|(key, _)| population.targets(key))
                    .map(|(key, axes)| {
                        let scope = super::facets::PanelScope {
                            fields: facets.fields.clone(),
                            key: key.clone(),
                        };
                        let axis = axes.iter().find(|a| a.id == scale.id).expect("panel axis");
                        let values = rows
                            .iter()
                            .zip(&inputs)
                            .enumerate()
                            .filter(|(_, (row, _))| super::facets::row_matches(**row, &scope))
                            .map(|(i, (_, value))| (i, *value))
                            .collect();
                        (axis, values)
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![(*axis, inputs.iter().copied().enumerate().collect())]
            };
            let values = map_populations(&populations, rows.len(), registry)?;
            let values = Arc::new(
                rows.iter()
                    .zip(values)
                    .map(|(r, v)| (r.key(), v))
                    .collect::<BTreeMap<_, _>>(),
            );
            vec![values]
        };
        let panel = if values.len() == 1 {
            0
        } else {
            panel_axes.expect("broadcast panel selection").0
        };
        *samples = Some(values[panel].clone());
        cache
            .0
            .push((layer.id, scale.id, input.as_ref().clone(), values));
        Ok(())
    };
    if let Mappings::Source(a) = &mut layer.mappings {
        for n in [
            &mut a.x,
            &mut a.y,
            &mut a.x2,
            &mut a.y2,
            &mut a.low,
            &mut a.high,
        ]
        .into_iter()
        .flatten()
        {
            bind(n)?;
        }
    }
    match &mut layer.statistic.parameters {
        StatParameters::Bin(s) => bind(&mut s.input)?,
        StatParameters::AutoBin(s) => bind(&mut s.input)?,
        StatParameters::Summary(s) => bind(&mut s.input)?,
        StatParameters::Ols(s) => {
            bind(&mut s.x)?;
            bind(&mut s.y)?;
        }
        StatParameters::Count(s) => {
            for n in &mut s.required {
                bind(n)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Bind each already-resolved shared source operation once. The per-node cache
/// also shares its ordered samples across panel definitions with different limits.
pub(super) fn source_transforms(
    resolved: &mut ChartDefinition,
    context: &ChartDefinition,
    source: &StoreSnapshot,
    registry: &ExtensionRegistry,
    budget: CompileLimits,
    panel_axes: Option<(usize, &[Vec<AxisSpec>])>,
    caches: &mut BTreeMap<crate::TransformId, SourceCache>,
) -> ChartResult<()> {
    if !context.axes.iter().any(selected) {
        return Ok(());
    }
    let mut consumers = BTreeMap::new();
    for consumer in &resolved.layers {
        let mut input = consumer.data;
        for _ in 0..=resolved.transforms.len() {
            let DataRef::Transform(id) = input else { break };
            consumers.entry(id).or_insert_with(|| consumer.clone());
            input = resolved
                .transforms
                .iter()
                .find(|n| n.id == id)
                .expect("validated transform dependency")
                .input;
        }
    }
    for node in &mut resolved.transforms {
        let Some(mut layer) = consumers.remove(&node.id) else {
            continue;
        };
        layer.data = node.input;
        layer.filters = node.filters.clone();
        layer.facet = node.facet.clone();
        layer.scope = node.scope;
        layer.statistic = node.statistic.clone();
        layer.mappings = Mappings::Source(SourceAes::default());
        // The statistic already carries its resolved source projections. The
        // consuming layer's generated aesthetics must not enter source binding.
        source_layer(
            &mut layer,
            context,
            source,
            registry,
            budget,
            panel_axes,
            caches.entry(node.id).or_default(),
        )?;
        node.statistic = layer.statistic;
    }
    Ok(())
}

pub(super) fn generated_rows(
    definition: &ChartDefinition,
    layer: &Layer,
    axes: &[AxisSpec],
    rows: &mut [&mut super::compiler::EncodedRow],
    registry: &ExtensionRegistry,
) -> ChartResult<()> {
    for axis in axes.iter().filter(|a| selected(a)) {
        generated_populations(
            definition,
            layer,
            &mut [(axis, rows.iter_mut().map(|r| &mut **r).collect())],
            registry,
        )?;
    }
    Ok(())
}

/// Map all panels of one layer/axis together, after all final domains are trained.
pub(super) fn generated_populations(
    definition: &ChartDefinition,
    layer: &Layer,
    groups: &mut [(&AxisSpec, Vec<&mut super::compiler::EncodedRow>)],
    registry: &ExtensionRegistry,
) -> ChartResult<()> {
    let Some((axis, _)) = groups.first() else {
        return Ok(());
    };
    let ids = if layer.orientation == Orientation::Horizontal {
        [layer.scales.y, layer.scales.x]
    } else {
        [layer.scales.x, layer.scales.y]
    };
    let Some(dimension) = ids.iter().position(|id| *id == axis.id) else {
        return Ok(());
    };
    let [x2, y2, low, high] = super::scale_stage::mapped_endpoints(layer);
    let present = |field| {
        groups
            .iter()
            .flat_map(|(_, r)| r)
            .any(|r| value(r, field).is_some())
    };
    let mut fields = vec![dimension];
    if dimension == 0 {
        if x2 || present(2) {
            fields.push(2);
        }
    } else {
        for (field, mapped) in [(3, y2), (4, low), (5, high)] {
            if mapped || present(field) {
                fields.push(field);
            }
        }
    }
    let mut order = groups
        .iter()
        .enumerate()
        .flat_map(|(g, (_, rows))| {
            rows.iter()
                .enumerate()
                .map(move |(r, row)| (g, r, row.ordinal, row.key.is_some()))
        })
        .collect::<Vec<_>>();
    if matched_source(definition, layer) && order.iter().all(|(_, _, _, source)| *source) {
        order.sort_by_key(|(_, _, ordinal, _)| *ordinal);
    }
    let mut indices = groups
        .iter()
        .map(|(_, rows)| vec![0; rows.len()])
        .collect::<Vec<_>>();
    for (index, (g, r, _, _)) in order.iter().enumerate() {
        indices[*g][*r] = index;
    }
    for field in fields {
        let inputs = groups
            .iter()
            .zip(&indices)
            .map(|((axis, rows), indices)| {
                let mut values = rows
                    .iter()
                    .zip(indices)
                    .map(|(r, i)| (*i, Number(value(r, field).unwrap_or(f64::NAN))))
                    .collect::<Vec<_>>();
                values.sort_by_key(|(index, _)| *index);
                (*axis, values)
            })
            .collect::<Vec<_>>();
        let values = map_populations(&inputs, order.len(), registry)?;
        for ((g, r, _, _), v) in order.iter().zip(values) {
            let row = &mut groups[*g].1[*r];
            let v = (!v.0.is_nan()).then_some(v.0);
            match field {
                0 => row.x = v,
                1 => row.y = v,
                2 => row.x2 = v,
                3 => row.y2 = v,
                4 => row.low = v,
                _ => row.high = v,
            };
        }
    }
    Ok(())
}
pub(super) fn value(row: &super::compiler::EncodedRow, field: usize) -> Option<f64> {
    match field {
        0 => row.x,
        1 => row.y,
        2 => row.x2,
        3 => row.y2,
        4 => row.low,
        _ => row.high,
    }
}
