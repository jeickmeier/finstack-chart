//! Select guide values over a coordinate view without changing mark scales or statistics.
use super::*;
use crate::composition::ScaleValue;
use crate::grammar::PreparedChart;
use crate::scales::Bounds;
use crate::{ChartResult, GuideId, Rect, ScaleId};
use std::collections::BTreeMap;

pub(super) fn resolve(
    chart: &PreparedChart,
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    request: &LayoutRequest,
    plot: Rect,
    map: &super::coordinate_map::CoordinateMap,
) -> ChartResult<BTreeMap<GuideId, ResolvedGuide>> {
    let mut selection = request.clone();
    for spec in &mut selection.axes {
        let axis = &axes[&spec.id];
        if matches!(
            axis.scale,
            ResolvedScale::Secondary { .. }
                | ResolvedScale::SecondaryDiscrete { .. }
                | ResolvedScale::SecondaryTime { .. }
        ) || axis.space.is_categorical()
        {
            continue;
        }
        let dimension = usize::from(!spec.side.horizontal());
        let domain = map.axes[dimension];
        let view = map.guide_view(dimension);
        let positions = view.map(|value| {
            domain.range[0]
                + (value - domain.viewport[0]) / (domain.viewport[1] - domain.viewport[0])
                    * (domain.range[1] - domain.range[0])
        });
        let mut values = [0.; 2];
        for i in 0..2 {
            values[i] = match axis.invert_value(positions[i])? {
                ScaleValue::Number(v) => v,
                ScaleValue::Timestamp { value, .. } => {
                    let crate::grammar::ValueSpace::Timestamp { origin, .. } = axis.space else {
                        unreachable!()
                    };
                    (i128::from(value) - i128::from(origin)) as f64
                }
                _ => unreachable!(),
            };
        }
        spec.viewport = Some(Bounds::new(values[0], values[1])?);
        // Tick selection sees the new semantic view; positions remain in the original
        // Cartesian destination space for the one shared coordinate projection.
        spec.range = Some(Bounds::new(positions[0], positions[1])?);
    }
    let mut selected = BTreeMap::new();
    let mut guide_axes = BTreeMap::new();
    for spec in &selection.axes {
        guide_axes.insert(
            spec.id,
            super::axes::resolve_axis(
                chart,
                &selection,
                spec,
                plot,
                selected.entry(spec.id).or_default(),
            )?,
        );
    }
    super::engine::resolve_guides(chart, &guide_axes, &selected, &selection)
}
