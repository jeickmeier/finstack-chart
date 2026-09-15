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
    selected_major: Option<&super::guide_ticks::SelectedGuideValues>,
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
    if let MinorBreaks::Registered(call) = policy {
        if matches!(axis.spec.scale, AxisScale::Binned { .. }) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "The reference binned positional constructor does not accept minor-break callbacks.",
            ));
        }
        let fallback;
        let values = if let Some(selected) = selected_major {
            &selected.values
        } else {
            fallback = major
                .iter()
                .map(|tick| tick.value.clone())
                .collect::<Vec<_>>();
            &fallback
        };
        if axis.space.is_categorical() {
            return registered_discrete(chart, axis, style, values, call, request);
        }
        if matches!(axis.space, crate::grammar::ValueSpace::Timestamp { .. }) {
            return super::temporal_minor_breaks::registered(
                chart,
                axis,
                values,
                selected_major.and_then(|s| s.names.as_deref()),
                call,
                request,
            );
        }
        return registered(
            chart,
            axis,
            values,
            selected_major.and_then(|s| s.transformed.as_deref()),
            call,
            request,
        );
    }
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
                if let (
                    ScaleValue::Number(value),
                    crate::grammar::ValueSpace::Timestamp { origin, .. },
                ) = (&tick.value, &axis.space)
                {
                    return Ok(Number(scale.reference_relative(
                        *value + (i128::from(*origin) - i128::from(scale.origin())) as f64,
                        matches!(axis.spec.scale, super::AxisScale::Date { .. }),
                    )));
                }
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
        ResolvedScale::Linear(s) => ([s.viewport().start(), s.viewport().end()], None),
        ResolvedScale::Nonlinear(s) => (
            [
                s.transformed_viewport().start(),
                s.transformed_viewport().end(),
            ],
            Some(s.transform()),
        ),
        ResolvedScale::Unbounded(s) => (
            s.viewport().map(|v| v.0),
            match &axis.spec.scale {
                AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
                AxisScale::Auto | AxisScale::Linear(_) => None,
                _ => return Ok(vec![]),
            },
        ),
        _ if style.minor_breaks.is_none() => return Ok(vec![]),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This minor-break policy currently requires a continuous numeric axis.",
            ));
        }
    };
    let contains = |v: f64| (limits[0].min(limits[1])..=limits[0].max(limits[1])).contains(&v);
    let forward = |v: f64| -> ChartResult<Option<f64>> {
        if !v.is_finite() {
            return Ok(None);
        }
        transform.as_ref().map_or(Ok(Some(v)), |t| t.forward(v))
    };
    let selected = match policy {
        MinorBreaks::Automatic => {
            let fallback;
            let values = if let Some(selected) = selected_major {
                &selected.values
            } else {
                fallback = major.iter().map(|t| t.value.clone()).collect::<Vec<_>>();
                &fallback
            };
            let transformed = if let Some(v) = selected_major.and_then(|s| s.transformed.as_ref()) {
                v.clone()
            } else {
                values
                    .iter()
                    .map(|value| match value {
                        ScaleValue::Number(v) => forward(*v),
                        _ => Err(error(
                            DiagnosticCode::SchemaConflict,
                            "Numeric minor breaks require numeric major values.",
                        )),
                    })
                    .collect::<ChartResult<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
            };
            let major = transformed
                .into_iter()
                .filter(|v| v.is_finite())
                .map(Number)
                .filter(|v| contains(v.0))
                .collect::<Vec<_>>();
            let bounds = [Number(limits[0]), Number(limits[1])];
            let custom = if crate::scales::ggplot_zero_range(limits[0], limits[1]) {
                Some(vec![])
            } else if let Some(ScaleTransform::Ggplot { transform }) = &transform {
                transform.default_minor_breaks(&major, bounds, request.max_ticks)?
            } else {
                None
            };
            if let Some(values) = custom {
                values
                    .into_iter()
                    .filter(|v| v.0.is_finite() && contains(v.0))
                    .collect()
            } else {
                ggplot_minor_breaks(
                    &major,
                    bounds,
                    transform == Some(ScaleTransform::Reverse),
                    request.max_ticks,
                )?
            }
        }
        MinorBreaks::Numeric(values) => {
            crate::limits::require_within(
                values.len() <= request.max_ticks,
                "explicit minor breaks",
            )?;
            let (family, reverse) =
                crate::scales::ggplot_numeric_limits::positional_coordinates(transform.clone());
            crate::scales::ggplot_forward_values(
                &family,
                reverse,
                &values.iter().map(|v| v.0).collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter(|v| v.is_finite() && contains(*v))
            .map(Number)
            .collect()
        }
        MinorBreaks::Registered(_)
        | MinorBreaks::Hidden
        | MinorBreaks::TimeWidth(_)
        | MinorBreaks::Timestamps(_) => {
            unreachable!()
        }
    };
    selected
        .into_iter()
        .map(|v| {
            let raw = transform
                .as_ref()
                .map_or(Ok(v.0), |t| t.inverse(v.0))
                .ok()
                .filter(|v| v.is_finite());
            let value = raw.map(ScaleValue::Number);
            let position = match &axis.scale {
                ResolvedScale::Nonlinear(s) => s.map_transformed(v.0)?,
                ResolvedScale::Linear(s) => s.map(v.0)?,
                ResolvedScale::Unbounded(s) => s.map_transformed(v.0)?,
                _ => unreachable!(),
            };
            Ok(position.map(|position| MinorGuideTick { value, position }))
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|values| values.into_iter().flatten().collect())
}

fn registered(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    values: &[ScaleValue],
    transformed_major: Option<&[f64]>,
    call: &crate::grammar::ScaleBreaksOperation,
    request: &LayoutRequest,
) -> ChartResult<Vec<MinorGuideTick>> {
    use crate::scales::{ScaleKey, ggplot_zero_range};
    let (bounds, transform) = match &axis.scale {
        ResolvedScale::Linear(s) => ([s.viewport().minimum(), s.viewport().maximum()], None),
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
                AxisScale::Auto | AxisScale::Linear(_) => None,
                AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Registered minor breaks currently require numeric primary axes.",
                    ));
                }
            },
        ),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered minor breaks currently require numeric primary axes.",
            ));
        }
    };
    if ggplot_zero_range(bounds[0], bounds[1]) {
        return Ok(vec![]);
    }
    let (family, reverse) = crate::scales::ggplot_numeric_limits::positional_coordinates(transform);
    let keys = |values: Vec<f64>| {
        values
            .into_iter()
            .map(|v| ScaleKey::Number(Number(v)))
            .collect::<Vec<_>>()
    };
    let domain = keys(crate::scales::ggplot_inverse_values(
        &family, reverse, &bounds,
    )?);
    let transformed = if let Some(values) = transformed_major {
        values.to_vec()
    } else {
        let raw = values
            .iter()
            .map(|value| match value {
                ScaleValue::Number(v) => Ok(*v),
                _ => Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Numeric minor callbacks require numeric major breaks.",
                )),
            })
            .collect::<ChartResult<Vec<_>>>()?;
        crate::scales::ggplot_forward_values(&family, reverse, &raw)?
    };
    let major = transformed
        .into_iter()
        .filter(|v| v.is_finite() && *v >= bounds[0] && *v <= bounds[1])
        .collect::<Vec<_>>();
    let major = keys(crate::scales::ggplot_inverse_values(
        &family, reverse, &major,
    )?);
    let selected = chart
        .break_registrations
        .evaluate_minor(call, &domain, Some(&major))?;
    if selected.values.is_none() {
        crate::scales::ggplot_forward_null(&family, reverse)?;
    }
    let values = selected.values.unwrap_or_default();
    crate::limits::require_within(values.len() <= request.max_ticks, "registered minor breaks")?;
    let raw = values
        .into_iter()
        .map(|value| match value {
            ScaleKey::Number(n) => Ok(n.0),
            ScaleKey::Null => Ok(f64::NAN),
            _ => Err(error(
                DiagnosticCode::Validation,
                "Numeric minor callbacks must return numeric values.",
            )),
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let transformed = crate::scales::ggplot_forward_values(&family, reverse, &raw)?;
    raw.into_iter()
        .zip(transformed)
        .map(|(raw, transformed)| {
            if !transformed.is_finite() || transformed < bounds[0] || transformed > bounds[1] {
                return Ok(None);
            }
            let position = match &axis.scale {
                ResolvedScale::Linear(s) => s.map(transformed)?,
                ResolvedScale::Nonlinear(s) => s.map_transformed(transformed)?,
                ResolvedScale::Unbounded(s) => s.map_transformed(transformed)?,
                _ => unreachable!(),
            };
            Ok(position.map(|position| MinorGuideTick {
                value: raw.is_finite().then_some(ScaleValue::Number(raw)),
                position,
            }))
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|v| v.into_iter().flatten().collect())
}

/// Discrete callbacks receive numeric category coordinates without a zero-range bypass.
fn registered_discrete(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    values: &[ScaleValue],
    call: &crate::grammar::ScaleBreaksOperation,
    request: &LayoutRequest,
) -> ChartResult<Vec<MinorGuideTick>> {
    use crate::scales::ScaleKey;
    let has_domain = match &axis.scale {
        ResolvedScale::Band(s) => !s.domain().is_empty(),
        ResolvedScale::Point(s) => !s.domain().is_empty(),
        ResolvedScale::Provider(s) => !s.domain().is_empty(),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered discrete minor breaks require a primary category scale.",
            ));
        }
    };
    let domain = super::secondary_discrete::reference_viewport(axis)?.map(ScaleKey::Number);
    let major = values
        .iter()
        .map(|value| {
            let coordinate = match value {
                ScaleValue::Number(value) => Some(*value),
                _ => super::secondary_discrete::coordinate(axis, value)?,
            };
            Ok(ScaleKey::Number(Number(coordinate.unwrap_or(f64::NAN))))
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let selected = chart.break_registrations.evaluate_minor(
        call,
        &domain,
        if has_domain && style.guide_ticks.as_ref().is_some_and(Vec::is_empty) {
            None
        } else {
            Some(&major)
        },
    )?;
    let values = selected.values.unwrap_or_default();
    crate::limits::require_within(
        values.len() <= request.max_ticks,
        "registered discrete minor breaks",
    )?;
    values
        .into_iter()
        .map(|value| {
            let value = match value {
                ScaleKey::Number(n) => n.0,
                ScaleKey::Null => f64::NAN,
                _ => {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Discrete minor callbacks must return numeric category positions.",
                    ));
                }
            };
            Ok(
                super::secondary_discrete::position(axis, value)?.map(|position| MinorGuideTick {
                    value: Some(ScaleValue::Number(value)),
                    position,
                }),
            )
        })
        .collect::<ChartResult<Vec<_>>>()
        .map(|values| values.into_iter().flatten().collect())
}
