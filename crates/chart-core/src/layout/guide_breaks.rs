//! Pure reference break selection over retained positional scales.
use super::{AxisScale, GuideStyle, ResolvedAxis, ResolvedScale};
use crate::{
    ChartResult, DiagnosticCode,
    composition::ScaleValue,
    grammar::PreparedChart,
    interpolate::Number,
    scales::{ScaleKey, error, ggplot_zero_range},
};

pub(super) struct SelectedBreaks {
    /// A NULL numeric secondary result bypasses labels; an empty vector does not.
    pub null_result: bool,
    pub values: Vec<ScaleValue>,
    pub names: Option<Vec<String>>,
}
pub(super) fn numeric(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    budget: usize,
    secondary: bool,
) -> ChartResult<Option<SelectedBreaks>> {
    let Some(call) = &style.breaks_function else {
        return Ok(None);
    };
    if chart.definition().profile() != crate::grammar::Profile::Ggplot2_4_0_3
        || style.profile != super::GuideProfile::LibraryV1
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Positional break functions require the reference guide profile.",
        ));
    }
    if matches!(axis.space, crate::grammar::ValueSpace::Timestamp { .. }) {
        return temporal(chart, axis, style, budget, secondary).map(Some);
    }
    if !matches!(
        axis.spec.scale,
        AxisScale::Auto
            | AxisScale::Linear(_)
            | AxisScale::Nonlinear { .. }
            | AxisScale::Secondary { .. }
    ) {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Positional break functions currently require a numeric continuous primary scale.",
        ));
    }
    let (bounds, transform) = match &axis.scale {
        ResolvedScale::Linear(s) => ([s.viewport().minimum(), s.viewport().maximum()], None),
        ResolvedScale::Secondary { view, .. } => ([view.minimum(), view.maximum()], None),
        ResolvedScale::Nonlinear(s) => (
            [
                s.transformed_viewport().minimum(),
                s.transformed_viewport().maximum(),
            ],
            Some(s.transform()),
        ),
        ResolvedScale::Unbounded(s) => (
            s.viewport().map(|n| n.0),
            match &axis.spec.scale {
                AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
                _ => None,
            },
        ),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Break functions require numeric panel limits.",
            ));
        }
    };
    let bounds = [bounds[0].min(bounds[1]), bounds[0].max(bounds[1])];
    let (family, reverse) =
        crate::scales::ggplot_numeric_limits::positional_coordinates(transform.clone());
    let domain = crate::scales::ggplot_inverse_values(&family, reverse, &bounds)?
        .into_iter()
        .map(|v| ScaleKey::Number(Number(v)))
        .collect::<Vec<_>>();
    if ggplot_zero_range(bounds[0], bounds[1]) {
        let ScaleKey::Number(value) = domain[0] else {
            unreachable!()
        };
        return Ok(Some(SelectedBreaks {
            null_result: false,
            values: vec![ScaleValue::Number(value.0)],
            names: None,
        }));
    }
    let output = chart.break_registrations.evaluate(
        call,
        &domain,
        style.tick_arguments.as_ref().and_then(|a| a.count),
    )?;
    let null_result = if output.values.is_none() {
        crate::scales::ggplot_forward_null(&family, reverse)?.is_none()
    } else {
        false
    };
    let values = output.values.unwrap_or_default();
    crate::limits::require_within(values.len() <= budget, "positional break function")?;
    let values = values
        .into_iter()
        .map(|v| match v {
            ScaleKey::Number(n) => Ok(ScaleValue::Number(n.0)),
            ScaleKey::Null => Ok(ScaleValue::Number(f64::NAN)),
            _ => Err(error(
                DiagnosticCode::Validation,
                "Numeric positional breaks must be numeric values.",
            )),
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(Some(SelectedBreaks {
        null_result,
        values,
        names: output.names,
    }))
}

fn temporal(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    budget: usize,
    secondary: bool,
) -> ChartResult<SelectedBreaks> {
    use crate::scales::*;
    let (context, bounds) = temporal_context(axis)?;
    let normalization = context.normalization;
    let origin = normalization.origin;
    let domain = bounds.map(|v| ScaleKey::Number(Number(v)));
    let output = if bounds[0].is_finite()
        && ggplot_zero_range(
            normalization.absolute(bounds[0]),
            normalization.absolute(bounds[1]),
        ) {
        crate::grammar::ScaleBreaksOutput {
            values: Some(vec![domain[0].clone()]),
            names: None,
            temporal: Some(normalization),
        }
    } else {
        chart.break_registrations.evaluate_temporal(
            style.breaks_function.as_ref().unwrap(),
            &domain,
            style.tick_arguments.as_ref().and_then(|a| a.count),
            context,
        )?
    };
    if output.values.is_none() || output.temporal != Some(normalization) {
        return Err(error(
            DiagnosticCode::Validation,
            "Temporal break functions must return typed source timestamp offsets.",
        ));
    }
    let raw = output.values.unwrap();
    crate::limits::require_within(raw.len() <= budget, "temporal positional break function")?;
    let values = raw
        .into_iter()
        .map(|v| {
            let offset = match v {
                ScaleKey::Number(n) => n.0,
                ScaleKey::Null => f64::NAN,
                _ => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Temporal breaks require numeric timestamp offsets.",
                    ));
                }
            };
            if !offset.is_finite() || ((!normalization.date || secondary) && offset.fract() != 0.) {
                return Ok(ScaleValue::Number(offset));
            }
            Ok(ScaleValue::Timestamp {
                value: crate::scales::absolute_number(offset.floor(), origin)?,
                unit: normalization.unit,
            })
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(SelectedBreaks {
        null_result: false,
        values,
        names: output.names,
    })
}

/// Reference fixed vectors still reduce to the single value on a zero range.
pub(super) fn numeric_constant(axis: &ResolvedAxis) -> Option<ScaleValue> {
    let (bounds, transform) = match &axis.scale {
        ResolvedScale::Linear(s) => ([s.viewport().minimum(), s.viewport().maximum()], None),
        ResolvedScale::Nonlinear(s) => (
            [
                s.transformed_viewport().minimum(),
                s.transformed_viewport().maximum(),
            ],
            Some(s.transform()),
        ),
        _ => return None,
    };
    ggplot_zero_range(bounds[0], bounds[1])
        .then(|| ScaleValue::Number(transform.map_or(bounds[0], |t| t.inverse_raw(bounds[0]))))
}

/// Shared typed limits for primary temporal major and minor selectors.
pub(super) fn temporal_context(
    axis: &ResolvedAxis,
) -> ChartResult<(crate::grammar::GuideTemporalContext<'_>, [f64; 2])> {
    use crate::scales::*;
    let crate::grammar::ValueSpace::Timestamp {
        origin,
        representation,
    } = &axis.space
    else {
        unreachable!()
    };
    let normalization = GgplotTimestampNormalization {
        origin: *origin,
        unit: representation.unit,
        date: matches!(axis.spec.scale, AxisScale::Date { .. }),
    };
    let zone = match &axis.spec.scale {
        AxisScale::Calendar { spec, .. } => &spec.zone,
        _ => &CalendarZone::Utc,
    };
    let bounds = match &axis.scale {
        ResolvedScale::Calendar(s) => {
            let delta = (i128::from(s.origin()) - i128::from(*origin)) as f64;
            [
                s.relative_viewport().minimum() + delta,
                s.relative_viewport().maximum() + delta,
            ]
        }
        ResolvedScale::Utc(s) => {
            let v = s.viewport();
            [
                (i128::from(v.start) - i128::from(*origin)) as f64,
                (i128::from(v.end) - i128::from(*origin)) as f64,
            ]
        }
        ResolvedScale::Unbounded(s) => s.viewport().map(|n| n.0),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Temporal break functions require temporal panel limits.",
            ));
        }
    };
    Ok((
        crate::grammar::GuideTemporalContext {
            normalization,
            zone,
        },
        bounds,
    ))
}
