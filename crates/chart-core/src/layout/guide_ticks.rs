//! Tick selection over retained scales. This module never trains a scale or projects marks.
use super::*;
use crate::{ChartResult, DiagnosticCode, composition::ScaleValue, scales::*};

pub(super) fn resolve(
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
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
