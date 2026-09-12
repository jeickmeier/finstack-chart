//! Minor selection over retained major values and scale ranges; no population training.
use super::{
    AxisScale, GuideProfile, GuideStyle, GuideTick, LayoutRequest, MinorBreaks, MinorGuideTick,
    ResolvedAxis, ResolvedScale,
};
use crate::{
    ChartResult, DiagnosticCode,
    composition::ScaleValue,
    grammar::{PreparedChart, Profile},
    interpolate::Number,
    scales::{ScaleTransform, error, ggplot_minor_breaks},
};

pub(super) fn resolve(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    major: &[GuideTick],
    request: &LayoutRequest,
) -> ChartResult<Vec<MinorGuideTick>> {
    let automatic = MinorBreaks::Automatic;
    let policy = match &style.minor_breaks {
        Some(value) => value,
        None if chart.definition().profile() == Profile::Ggplot2_4_0_3
            && style.profile == GuideProfile::LibraryV1 =>
        {
            &automatic
        }
        None => return Ok(vec![]),
    };
    if let MinorBreaks::Timestamps(values) = policy {
        crate::limits::require_within(
            values.len() <= request.max_ticks,
            "explicit timestamp minor breaks",
        )?;
        if !matches!(
            axis.scale,
            ResolvedScale::Calendar(_) | ResolvedScale::Utc(_)
        ) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Timestamp minor breaks require a date or datetime axis.",
            ));
        }
        return values
            .iter()
            .flatten()
            .map(|candidate| {
                let ScaleValue::Timestamp { value, unit } = candidate else {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Timestamp minor candidates must be timestamps.",
                    ));
                };
                let position = match &axis.scale {
                    ResolvedScale::Calendar(scale) => {
                        if *unit != scale.unit() {
                            return Err(error(
                                DiagnosticCode::SchemaConflict,
                                "Timestamp minor units must match the axis.",
                            ));
                        }
                        let relative = scale.relative_guide(*value)?;
                        if !scale.relative_viewport().contains(relative) {
                            None
                        } else {
                            scale.map_relative_guide(relative)?
                        }
                    }
                    ResolvedScale::Utc(scale) => {
                        if *unit != scale.unit() {
                            return Err(error(
                                DiagnosticCode::SchemaConflict,
                                "Timestamp minor units must match the axis.",
                            ));
                        }
                        let view = scale.viewport();
                        if !(view.start.min(view.end)..=view.start.max(view.end)).contains(value) {
                            None
                        } else {
                            scale.map(*value)?
                        }
                    }
                    _ => unreachable!(),
                };
                Ok(position.map(|position| MinorGuideTick {
                    value: Some(candidate.clone()),
                    position,
                }))
            })
            .collect::<ChartResult<Vec<_>>>()
            .map(|values| values.into_iter().flatten().collect());
    }
    if let MinorBreaks::TimeWidth(width) = policy {
        let arguments = crate::scales::GuideTickArguments {
            width: Some(width.clone()),
            ..Default::default()
        }
        .resolve_width(
            super::guide_selection::is_duration(axis),
            matches!(axis.spec.scale, AxisScale::Date { .. }),
        )?;
        arguments.validate(request.limits.max_text_bytes)?;
        let range = match &axis.scale {
            ResolvedScale::Utc(s) => s.range(),
            ResolvedScale::Calendar(s) => s.range(),
            ResolvedScale::Linear(s) if super::guide_selection::is_duration(axis) => s.range(),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Time-width minor breaks require a date, datetime or duration axis.",
                ));
            }
        };
        return super::guide_selection::values(axis, &arguments, request.max_ticks, true)?
            .into_iter()
            .map(|value| {
                Ok(axis
                    .guide_value_position(&value)?
                    .filter(|p| range.contains(*p))
                    .map(|position| MinorGuideTick {
                        value: Some(value),
                        position,
                    }))
            })
            .collect::<ChartResult<Vec<_>>>()
            .map(|values| values.into_iter().flatten().collect());
    }
    if matches!(policy, MinorBreaks::Hidden) {
        return Ok(vec![]);
    }
    if matches!(axis.spec.scale, AxisScale::Binned { .. }) {
        return Ok(vec![]);
    }
    if axis.space.is_categorical() {
        if matches!(policy, MinorBreaks::Automatic) {
            return Ok(vec![]);
        }
        let MinorBreaks::Numeric(values) = policy else {
            unreachable!("hidden and width handled above");
        };
        crate::limits::require_within(
            values.len() <= request.max_ticks,
            "explicit categorical minor breaks",
        )?;
        return values
            .iter()
            .map(|value| {
                let position = match &axis.scale {
                    ResolvedScale::Band(scale) => scale.reference_minor(value.0)?,
                    ResolvedScale::Point(scale) => scale.reference_minor(value.0)?,
                    ResolvedScale::Provider(scale) => scale.category_minor(value.0)?,
                    _ => unreachable!(),
                };
                Ok(position.map(|position| MinorGuideTick {
                    value: Some(ScaleValue::Number(value.0)),
                    position,
                }))
            })
            .collect::<ChartResult<Vec<_>>>()
            .map(|values| values.into_iter().flatten().collect());
    }
    if let ResolvedScale::Calendar(scale) = &axis.scale {
        if !matches!(policy, MinorBreaks::Automatic) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Temporal minor breaks require Automatic, Hidden or TimeWidth.",
            ));
        }
        let limits = scale.relative_viewport();
        let major = major
            .iter()
            .map(|tick| {
                let ScaleValue::Timestamp { value, unit } = tick.value else {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Temporal minor breaks require timestamp major values.",
                    ));
                };
                if unit != scale.unit() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Temporal minor major units must match the axis.",
                    ));
                }
                scale.relative_guide(value).map(Number)
            })
            .collect::<ChartResult<Vec<_>>>()?
            .into_iter()
            .filter(|v| limits.contains(v.0))
            .collect::<Vec<_>>();
        return ggplot_minor_breaks(
            &major,
            [Number(limits.start()), Number(limits.end())],
            false,
            request.max_ticks,
        )?
        .into_iter()
        .map(|v| {
            let value = scale
                .guide_timestamp(v.0)
                .map(|(value, unit)| ScaleValue::Timestamp { value, unit });
            Ok(scale
                .map_relative_guide(v.0)?
                .map(|position| MinorGuideTick { value, position }))
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|values| values.into_iter().flatten().collect());
    }
    let (limits, transform) = match &axis.scale {
        ResolvedScale::Linear(s) => (s.viewport(), None),
        ResolvedScale::Nonlinear(s) => (s.transformed_viewport(), Some(s.transform())),
        _ if style.minor_breaks.is_none() => return Ok(vec![]),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This minor-break policy currently requires a continuous numeric axis.",
            ));
        }
    };
    let forward = |v: f64| -> ChartResult<Option<f64>> {
        if !v.is_finite() {
            return Ok(None);
        }
        transform.map_or(Ok(Some(v)), |t| t.forward(v))
    };
    let selected = match policy {
        MinorBreaks::Automatic => {
            let major = major
                .iter()
                .map(|t| match t.value {
                    ScaleValue::Number(v) => forward(v),
                    _ => Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Numeric minor breaks require numeric major values.",
                    )),
                })
                .collect::<ChartResult<Vec<_>>>()?
                .into_iter()
                .flatten()
                .map(Number)
                .filter(|v| limits.contains(v.0))
                .collect::<Vec<_>>();
            ggplot_minor_breaks(
                &major,
                [Number(limits.start()), Number(limits.end())],
                transform == Some(ScaleTransform::Reverse),
                request.max_ticks,
            )?
        }
        MinorBreaks::Numeric(values) => {
            crate::limits::require_within(
                values.len() <= request.max_ticks,
                "explicit minor breaks",
            )?;
            values
                .iter()
                .map(|v| forward(v.0))
                .collect::<ChartResult<Vec<_>>>()?
                .into_iter()
                .flatten()
                .filter(|v| limits.contains(*v))
                .map(Number)
                .collect()
        }
        MinorBreaks::Hidden | MinorBreaks::TimeWidth(_) | MinorBreaks::Timestamps(_) => {
            unreachable!()
        }
    };
    selected
        .into_iter()
        .map(|v| {
            let raw = transform
                .map_or(Ok(v.0), |t| t.inverse(v.0))
                .ok()
                .filter(|v| v.is_finite());
            let value = raw.map(ScaleValue::Number);
            let position = match &axis.scale {
                ResolvedScale::Nonlinear(s) => s.map_transformed(v.0)?,
                ResolvedScale::Linear(s) => s.map(v.0)?,
                _ => unreachable!(),
            };
            Ok(position.map(|position| MinorGuideTick { value, position }))
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|values| values.into_iter().flatten().collect())
}
