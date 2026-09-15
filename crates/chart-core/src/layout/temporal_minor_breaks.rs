//! Typed temporal minor callbacks share primary major selection and temporal limits.
use super::{LayoutRequest, MinorGuideTick, ResolvedAxis, ResolvedScale};
use crate::{
    ChartResult, DiagnosticCode,
    composition::ScaleValue,
    grammar::{PreparedChart, ScaleBreaksOperation},
    interpolate::Number,
    scales::{ScaleKey, error, ggplot_zero_range},
};

pub(super) fn registered(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    values: &[ScaleValue],
    names: Option<&[String]>,
    call: &ScaleBreaksOperation,
    request: &LayoutRequest,
) -> ChartResult<Vec<MinorGuideTick>> {
    let (context, bounds) = super::guide_breaks::temporal_context(axis)?;
    let n = context.normalization;
    if bounds[0].is_finite() && ggplot_zero_range(n.absolute(bounds[0]), n.absolute(bounds[1])) {
        return Ok(vec![]);
    }
    let (_, inputs) = super::guide_ticks::temporal_label_input(axis, values, true, false)?;
    let mut major = Vec::new();
    let mut major_names = names.map(|_| Vec::new());
    for (index, value) in inputs.iter().enumerate() {
        let ScaleValue::Number(value) = value else {
            unreachable!()
        };
        if value.is_finite() {
            major.push(ScaleKey::Number(Number(*value)));
            if let (Some(source), Some(target)) = (names, &mut major_names) {
                target.push(source[index].clone());
            }
        }
    }
    let domain = bounds.map(|v| ScaleKey::Number(Number(v)));
    let output = chart.break_registrations.evaluate_temporal_minor(
        call,
        &domain,
        &major,
        major_names.as_deref(),
        context,
    )?;
    if output.values.is_none() || output.temporal != Some(n) {
        return Err(error(
            DiagnosticCode::Validation,
            "Temporal minor functions must return typed source timestamp offsets.",
        ));
    }
    let values = output.values.unwrap();
    crate::limits::require_within(values.len() <= request.max_ticks, "temporal minor breaks")?;
    values
        .into_iter()
        .map(|value| {
            let offset = match value {
                ScaleKey::Number(n) => n.0,
                ScaleKey::Null => f64::NAN,
                _ => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Temporal minor breaks require numeric timestamp offsets.",
                    ));
                }
            };
            if !offset.is_finite()
                || n.absolute(offset) < n.absolute(bounds[0])
                || n.absolute(offset) > n.absolute(bounds[1])
            {
                return Ok(None);
            }
            let position = match &axis.scale {
                ResolvedScale::Calendar(s) => {
                    // Normalize within the callback's common origin before moving
                    // into the scale frame; adding distant epochs loses endpoint bits.
                    let view = s.relative_viewport();
                    let t = (offset - bounds[0]) / (bounds[1] - bounds[0]);
                    s.map_relative_guide(view.minimum() + t * (view.maximum() - view.minimum()))?
                }
                ResolvedScale::Utc(s) => {
                    let range = s.range();
                    Some(
                        range.start()
                            + (range.end() - range.start()) * (offset - bounds[0])
                                / (bounds[1] - bounds[0]),
                    )
                }
                ResolvedScale::Unbounded(s) => s.map(offset)?,
                _ => unreachable!("validated temporal context"),
            };
            // Fractional Date values are valid; do not floor the returned minor vector.
            let value = if offset.fract() == 0. && offset.abs() <= 9_007_199_254_740_992. {
                ScaleValue::Timestamp {
                    value: crate::scales::absolute(offset, n.origin)?,
                    unit: n.unit,
                }
            } else {
                ScaleValue::Number(offset)
            };
            Ok(position.map(|position| MinorGuideTick {
                value: Some(value),
                position,
            }))
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|v| v.into_iter().flatten().collect())
}
