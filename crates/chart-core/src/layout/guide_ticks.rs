//! Tick selection over retained scales. This module never trains a scale or projects marks.
use super::*;
use crate::{ChartResult, DiagnosticCode, composition::ScaleValue, scales::*};

pub(super) fn resolve(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
    if style.uses_tick_configuration() {
        return configured(chart, axis, style, r);
    }
    if matches!(axis.scale, ResolvedScale::Provider(_))
        && (style.number_format.is_some()
            || style.numeric_format.is_some()
            || style.time_format.is_some())
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Registered providers use their declared formatter or explicit guide labels; built-in formatter overrides require a built-in scale.",
        ));
    }
    if (style.number_format.is_some() || style.numeric_format.is_some())
        && !matches!(
            axis.scale,
            ResolvedScale::Linear(_)
                | ResolvedScale::Numeric(_)
                | ResolvedScale::Nonlinear(_)
                | ResolvedScale::Secondary { .. }
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric formatting requires a numeric guide.",
        ));
    }
    if style.time_format.is_some()
        && !matches!(
            axis.scale,
            ResolvedScale::Utc(_) | ResolvedScale::Calendar(_)
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Time formatting requires a UTC or calendar time axis.",
        ));
    }
    let mut ticks = Vec::new();
    if let Some(values) = &style.guide_ticks {
        let mut bytes = r.limits.max_text_bytes;
        crate::limits::require_within(values.len() <= r.max_ticks, "custom guide tick")?;
        for tick in values {
            crate::limits::require_within(tick.label.len() <= bytes, "custom guide label byte")?;
            bytes -= tick.label.len();
            if tick.label.is_empty() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Custom guide labels must be nonempty.",
                ));
            }
            if let Some(position) = axis.map_value(&tick.value)?
                && style.visible
            {
                ticks.push(GuideTick {
                    value: tick.value.clone(),
                    position,
                    label: tick.label.clone(),
                });
            }
        }
        ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
        return Ok(ticks);
    }
    if !style.visible {
        return Ok(ticks);
    }
    macro_rules! numeric {
        ($scale:expr, $family:expr) => {{
            let s = $scale;
            let formatter =
                super::axes::guide_formatter(style, $family, s.viewport(), r.target_ticks as f64)?;
            for t in s.ticks(r.target_ticks, r.max_ticks)? {
                if let Some(position) = s.map(t.value)? {
                    ticks.push(GuideTick {
                        value: ScaleValue::Number(t.value),
                        position,
                        label: super::axes::numeric_label(
                            style,
                            t.value,
                            t.label,
                            formatter.as_ref(),
                        )?,
                    });
                }
            }
        }};
    }
    macro_rules! categorical {
        ($scale:expr) => {{
            let s = $scale;
            let stride = s.visible_domain().len().div_ceil(r.max_ticks).max(1);
            for label in s.visible_domain().iter().step_by(stride) {
                if let Some(position) = s.center(label)? {
                    ticks.push(GuideTick {
                        value: ScaleValue::Category(label.clone()),
                        position,
                        label: label.clone(),
                    });
                }
            }
        }};
    }
    match &axis.scale {
        ResolvedScale::Provider(s) => {
            let arguments = GuideTickArguments {
                count: Some(r.target_ticks as f64),
                ..Default::default()
            };
            let values = s.ticks(&arguments, r.max_ticks)?;
            let labels = s.labels(&values, &arguments, r.max_ticks)?;
            for (value, label) in values.into_iter().zip(labels) {
                if let Some(position) = s.map(&value)? {
                    ticks.push(GuideTick {
                        value,
                        position,
                        label,
                    });
                }
            }
        }
        ResolvedScale::Linear(s) => numeric!(s, NumericFamily::Linear),
        ResolvedScale::Numeric(s) => numeric!(s, s.family()),
        ResolvedScale::Nonlinear(s) => {
            let family = match s.transform() {
                ScaleTransform::Log { base } => NumericFamily::Log { base },
                ScaleTransform::Symlog { threshold } => NumericFamily::Symlog {
                    constant: threshold,
                },
            };
            numeric!(s, family);
        }
        ResolvedScale::Band(s) => categorical!(s),
        ResolvedScale::Point(s) => categorical!(s),
        ResolvedScale::Session(s) => {
            for t in s.ticks(r.target_ticks, r.max_ticks)? {
                if let Some(position) = s.map(t.value)? {
                    ticks.push(GuideTick {
                        value: ScaleValue::Timestamp {
                            value: t.value,
                            unit: s.calendar().unit,
                        },
                        position,
                        label: t.label,
                    });
                }
            }
        }
        ResolvedScale::Calendar(s) => {
            let interval = match &axis.spec.scale {
                AxisScale::Calendar { interval, .. } => *interval,
                _ => None,
            };
            let selection = interval.map_or(
                CalendarTicks::Count((r.target_ticks as f64).into()),
                CalendarTicks::Interval,
            );
            let formatter = style
                .time_format
                .clone()
                .unwrap_or_default()
                .prepare(s.calendar().clone())?;
            for value in s.ticks(selection, r.max_ticks)? {
                if let Some(position) = s.map(value)? {
                    ticks.push(GuideTick {
                        value: ScaleValue::Timestamp {
                            value,
                            unit: s.unit(),
                        },
                        position,
                        label: formatter.format(value, s.unit())?,
                    });
                }
            }
        }
        ResolvedScale::Utc(s) => {
            let interval = match axis.spec.scale {
                AxisScale::Utc { interval, .. } => interval,
                _ => None,
            };
            let formatter = style
                .time_format
                .as_ref()
                .map(|f| f.prepare(Calendar::new(CalendarZone::Utc)?))
                .transpose()?;
            for t in s.ticks(
                interval.unwrap_or(s.auto_interval(r.target_ticks)?),
                r.max_ticks,
            )? {
                if let Some(position) = s.map(t.value)? {
                    let label = formatter
                        .as_ref()
                        .map_or_else(|| Ok(t.label.clone()), |f| f.format(t.value, s.unit()))?;
                    ticks.push(GuideTick {
                        value: ScaleValue::Timestamp {
                            value: t.value,
                            unit: s.unit(),
                        },
                        position,
                        label,
                    });
                }
            }
        }
        ResolvedScale::Secondary { .. } => {
            // Secondary units inherit primary positions and already retain exact converted values.
            // Independent replacement formatting requires a primary positional scale.
            if style != &axis.spec.guide {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Additional guides must reference a primary positional scale, not an alternate-unit guide.",
                ));
            }
            ticks.clone_from(&axis.ticks);
        }
    }
    ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
    Ok(ticks)
}

fn configured(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
    super::axes::validate_style(style, r.limits)?;
    let arguments = style.tick_arguments.clone().unwrap_or_default();
    arguments.validate(r.limits.max_text_bytes)?;
    // Select before enumeration: even an empty explicit list bypasses scale ticks.
    let values = if let Some(values) = &style.tick_values {
        crate::limits::require_within(values.len() <= r.max_ticks, "selected guide tick")?;
        values.clone()
    } else if let Some(ticks) = &style.guide_ticks {
        crate::limits::require_within(ticks.len() <= r.max_ticks, "selected guide tick")?;
        ticks.iter().map(|tick| tick.value.clone()).collect()
    } else if style.visible {
        super::guide_selection::values(axis, &arguments, r.max_ticks)?
    } else {
        Vec::new()
    };
    crate::limits::require_within(values.len() <= r.max_ticks, "selected guide tick")?;
    let mut bytes = r.limits.max_text_bytes;
    for value in &values {
        if let ScaleValue::Category(label) = value {
            crate::limits::require_within(label.len() <= bytes, "selected guide category byte")?;
            bytes -= label.len();
        }
    }
    // Validate every semantic value before any per-value formatter. Mapping omission
    // does not alter the complete callback list or its occurrence indices.
    let positions = values
        .iter()
        .map(|value| axis.map_value(value))
        .collect::<ChartResult<Vec<_>>>()?;
    let numeric = match &style.tick_format {
        Some(GuideFormatter::Numeric(format)) => Some(format.as_ref()),
        _ => style.numeric_format.as_ref(),
    };
    let time = match &style.tick_format {
        Some(GuideFormatter::Time(format)) => Some(format.as_ref()),
        _ => style.time_format.as_ref(),
    };
    let labels = if let Some(ticks) = &style.guide_ticks {
        ticks.iter().map(|tick| tick.label.clone()).collect()
    } else {
        match &style.tick_format {
            Some(GuideFormatter::Labels(labels)) => {
                if labels.len() != values.len() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Explicit guide labels must match the complete selected value list.",
                    ));
                }
                labels.clone()
            }
            Some(GuideFormatter::Registered {
                operation,
                parameters,
            }) => chart.guide_registrations.labels(
                operation,
                parameters,
                &values,
                r.limits,
                r.units == crate::services::Units::Points,
            )?,
            _ if style.number_format.is_some() => {
                let format = style.number_format.as_ref().expect("checked");
                values
                    .iter()
                    .map(|value| match value {
                        ScaleValue::Number(n) => format.format(*n),
                        _ => Err(error(
                            DiagnosticCode::SchemaConflict,
                            "Numeric formatting requires numeric guide values.",
                        )),
                    })
                    .collect::<ChartResult<Vec<_>>>()?
            }
            _ => super::guide_selection::labels(
                axis,
                &values,
                &arguments,
                numeric,
                time,
                r.max_ticks,
            )?,
        }
    };
    let mut bytes = r.limits.max_text_bytes;
    for label in &labels {
        crate::limits::require_within(label.len() <= bytes, "configured guide label byte")?;
        bytes -= label.len();
    }
    if !style.visible {
        return Ok(Vec::new());
    }
    Ok(values
        .into_iter()
        .zip(labels)
        .zip(positions)
        .filter_map(|((value, label), position)| {
            position.map(|position| GuideTick {
                value,
                position,
                label,
            })
        })
        .collect())
}
