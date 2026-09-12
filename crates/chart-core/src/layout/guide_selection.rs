//! Guide policy over shared scale ticks and formatters; never trains or replaces a scale.
use super::{ResolvedAxis, ResolvedScale};
use crate::{
    ChartResult, DiagnosticCode,
    composition::ScaleValue,
    scales::*,
    typography::{NumericFormat, NumericLocale},
};

fn unsupported(message: &str) -> crate::Diagnostic {
    crate::scales::error(DiagnosticCode::UnsupportedCapability, message)
}

pub(super) fn is_duration(axis: &ResolvedAxis) -> bool {
    matches!(axis.spec.scale, super::AxisScale::Duration(_))
        || matches!(&axis.scale, ResolvedScale::Secondary { primary, .. } if is_duration(primary))
}

/// Recover the retained numeric viewport, without reconstructing a mapping.
fn numeric(scale: &ResolvedScale) -> Option<(NumericFamily, Bounds)> {
    match scale {
        ResolvedScale::Linear(s) => Some((NumericFamily::Linear, s.viewport())),
        ResolvedScale::SecondaryDiscrete { viewport, .. } => {
            Some((NumericFamily::Linear, *viewport))
        }
        ResolvedScale::Numeric(s) => Some((s.family(), s.viewport())),
        ResolvedScale::Secondary { view, primary, .. } => {
            numeric(&primary.scale).map(|(family, _)| (family, *view))
        }
        ResolvedScale::Nonlinear(s) => Some((
            match s.transform() {
                ScaleTransform::Reverse => NumericFamily::Linear,
                ScaleTransform::Sqrt => NumericFamily::Pow { exponent: 0.5 },
                ScaleTransform::Log { base } => NumericFamily::Log { base },
                ScaleTransform::Symlog { threshold } => NumericFamily::Symlog {
                    constant: threshold,
                },
            },
            s.viewport(),
        )),
        _ => None,
    }
}

pub(super) fn reference_time_selection(
    axis: &ResolvedAxis,
    arguments: &GuideTickArguments,
    budget: usize,
) -> ChartResult<Option<(crate::data::TimeUnit, GgplotTimeBreaks)>> {
    if arguments.interval.is_some()
        || arguments.seconds.is_some()
        || arguments.time_width.is_some()
        || arguments.width.is_some()
    {
        return Ok(None);
    }
    let (origin, view, unit, calendar, bounds) = match &axis.scale {
        ResolvedScale::Calendar(s) => (
            s.origin(),
            s.relative_viewport(),
            s.unit(),
            s.calendar().clone(),
            s.tick_bounds()?,
        ),
        ResolvedScale::Utc(s) => {
            let view = s.viewport();
            let relative = Bounds::new(
                (i128::from(view.start) - i128::from(s.origin())) as f64,
                (i128::from(view.end) - i128::from(s.origin())) as f64,
            )?;
            (
                s.origin(),
                relative,
                s.unit(),
                Calendar::new(CalendarZone::Utc)?,
                Some(view),
            )
        }
        _ => return Ok(None),
    };
    if arguments.count.is_none() {
        // A formatter override must not silently replace the authored axis interval.
        match axis.spec.scale {
            super::AxisScale::Utc {
                interval: Some(interval),
                ..
            } => {
                let ticks = bounds.map_or_else(
                    || Ok(Vec::new()),
                    |b| UtcScale::ticks_in(b, unit, interval, budget),
                )?;
                return Ok(Some((
                    unit,
                    GgplotTimeBreaks {
                        values: ticks.iter().map(|t| t.value).collect(),
                        labels: ticks.into_iter().map(|t| t.label).collect(),
                    },
                )));
            }
            super::AxisScale::Calendar {
                interval: Some(interval),
                ..
            } => {
                let values = bounds.map_or_else(
                    || Ok(Vec::new()),
                    |b| calendar.ticks(b, unit, CalendarTicks::Interval(interval), budget),
                )?;
                let formatter = TimeFormat::default().prepare(calendar)?;
                let labels = values
                    .iter()
                    .map(|v| formatter.format(*v, unit))
                    .collect::<ChartResult<_>>()?;
                return Ok(Some((unit, GgplotTimeBreaks { values, labels })));
            }
            _ => {}
        }
    }
    if matches!(axis.spec.scale, super::AxisScale::Date { .. }) {
        return Ok(Some((
            unit,
            ggplot_breaks_pretty_date(origin, view, unit, arguments.count.unwrap_or(5.), budget)?,
        )));
    }
    Ok(Some((
        unit,
        ggplot_breaks_pretty_time(
            origin,
            view,
            unit,
            &calendar,
            arguments.count.unwrap_or(5.),
            budget,
        )?,
    )))
}

/// D3 count arguments are density hints; the collection bound is always independent.
pub(super) fn values(
    axis: &ResolvedAxis,
    arguments: &GuideTickArguments,
    budget: usize,
    ggplot: bool,
) -> ChartResult<Vec<ScaleValue>> {
    if ggplot
        && arguments.interval.is_none()
        && let Some((_, bounds)) = numeric(&axis.scale)
        && ggplot_zero_range(bounds.start(), bounds.end())
    {
        return Ok(vec![ScaleValue::Number(bounds.start())]);
    }
    let duration = is_duration(axis);
    let duration_width = if duration {
        arguments.seconds.or(arguments
            .time_width
            .map(|width| width_seconds(width.unit, f64::from(width.step)))
            .transpose()?)
    } else {
        None
    };
    if let Some(seconds) = duration_width
        && let Some((_, bounds)) = numeric(&axis.scale)
    {
        return Ok(ggplot_breaks_duration_width(bounds, seconds, budget)?
            .into_iter()
            .filter(|v| bounds.contains(*v))
            .map(ScaleValue::Number)
            .collect());
    }
    if let Some(width) = &arguments.width {
        let (width_unit, count, _) = parse_width(width)?;
        let (bounds, unit) = match &axis.scale {
            ResolvedScale::Utc(s) => (s.viewport(), s.unit()),
            ResolvedScale::Calendar(s) => (
                match s.tick_bounds()? {
                    Some(b) => b,
                    None => return Ok(vec![]),
                },
                s.unit(),
            ),
            _ => {
                return Err(unsupported(
                    "Reference width alignment requires a time guide.",
                ));
            }
        };
        let values = if matches!(axis.spec.scale, super::AxisScale::Date { .. }) {
            aligned_dates(bounds, unit, count, budget)?
        } else {
            aligned_seconds(
                bounds,
                unit,
                width_seconds(width_unit, count)?,
                width_seconds(width_unit, count.floor())?,
                budget,
            )?
        };
        return Ok(values
            .into_iter()
            .map(|value| ScaleValue::Timestamp { value, unit })
            .collect());
    }
    if let Some(width) = arguments.time_width {
        let (values, unit) = match &axis.scale {
            ResolvedScale::Utc(s) => (
                ggplot_breaks_width(
                    s.viewport(),
                    s.unit(),
                    &Calendar::new(CalendarZone::Utc)?,
                    width,
                    budget,
                )?,
                s.unit(),
            ),
            ResolvedScale::Calendar(s) => (
                s.ggplot_width_ticks(
                    width,
                    budget,
                    matches!(axis.spec.scale, super::AxisScale::Date { .. }),
                )?,
                s.unit(),
            ),
            _ => {
                return Err(unsupported(
                    "Reference time widths require a UTC or calendar time guide.",
                ));
            }
        };
        return Ok(values
            .into_iter()
            .map(|value| ScaleValue::Timestamp { value, unit })
            .collect());
    }
    if let Some(seconds) = arguments.seconds {
        let (bounds, unit) = match &axis.scale {
            ResolvedScale::Utc(s) => (s.viewport(), s.unit()),
            ResolvedScale::Calendar(s) => (
                match s.tick_bounds()? {
                    Some(b) => b,
                    None => return Ok(Vec::new()),
                },
                s.unit(),
            ),
            _ => {
                return Err(unsupported(
                    "Fixed-second breaks require a UTC or calendar time guide.",
                ));
            }
        };
        return Ok(ggplot_breaks_seconds(bounds, unit, seconds, budget)?
            .into_iter()
            .map(|value| ScaleValue::Timestamp { value, unit })
            .collect());
    }
    let count = arguments.count.unwrap_or(if ggplot { 5. } else { 10. });
    if let Some((family, domain)) = numeric(&axis.scale) {
        if arguments.interval.is_some() {
            return Err(unsupported("Calendar intervals require a time guide."));
        }
        if ggplot {
            let limits = [domain.minimum(), domain.maximum()];
            let values = if is_duration(axis) {
                ggplot_breaks_duration(limits, count, budget)?
            } else {
                match family {
                    NumericFamily::Log { base } => ggplot_breaks_log(limits, count, base, budget)?,
                    _ => ggplot_breaks_extended(limits, count, budget)?,
                }
            };
            return Ok(values
                .into_iter()
                .filter(|v| {
                    matches!(axis.scale, ResolvedScale::Secondary { .. }) || domain.contains(*v)
                })
                .map(ScaleValue::Number)
                .collect());
        }
        return Ok(family
            .ticks(&[domain.start().into(), domain.end().into()], count, budget)?
            .into_iter()
            .map(ScaleValue::Number)
            .collect());
    }
    let selection = arguments
        .interval
        .map_or(CalendarTicks::Count(count.into()), CalendarTicks::Interval);
    let timestamp = |values: Vec<i64>, unit| {
        values
            .into_iter()
            .map(|value| ScaleValue::Timestamp { value, unit })
            .collect()
    };
    let categories = |values: &[String]| -> ChartResult<Vec<ScaleValue>> {
        crate::limits::require_within(values.len() <= budget, "guide domain tick")?;
        Ok(values.iter().cloned().map(ScaleValue::Category).collect())
    };
    match &axis.scale {
        ResolvedScale::Provider(s) => s.ticks(arguments, budget),
        ResolvedScale::Band(s) => categories(s.visible_domain()),
        ResolvedScale::Point(s) => categories(s.visible_domain()),
        ResolvedScale::Calendar(s) => Ok(timestamp(s.ticks(selection, budget)?, s.unit())),
        ResolvedScale::Utc(s) => Ok(timestamp(
            Calendar::new(CalendarZone::Utc)?.ticks(s.viewport(), s.unit(), selection, budget)?,
            s.unit(),
        )),
        _ => Err(unsupported(
            "This scale has no D3 automatic tick policy; supply explicit semantic values.",
        )),
    }
}

/// Format original values with shared family precision and explicit locale/calendar resources.
/// This function never generates candidate values, including for an empty explicit list.
pub(super) fn labels(
    axis: &ResolvedAxis,
    values: &[ScaleValue],
    arguments: &GuideTickArguments,
    numeric_format: Option<&NumericFormat>,
    time_format: Option<&TimeFormat>,
    budget: usize,
    ggplot_bytes: Option<usize>,
) -> ChartResult<Vec<String>> {
    if let Some((family, domain)) = numeric(&axis.scale) {
        let duration = is_duration(axis);
        if duration && let Some(format) = time_format {
            let formatter = format.prepare(Calendar::new(CalendarZone::Utc)?)?;
            return values
                .iter()
                .map(|value| {
                    let ScaleValue::Number(seconds) = value else {
                        return Err(unsupported("Duration formatting requires numeric seconds."));
                    };
                    let nanos = seconds * 1_000_000_000.;
                    if !nanos.is_finite() || nanos.abs() > 9_007_199_254_740_991_000. {
                        return Err(crate::scales::error(
                            DiagnosticCode::PrecisionLoss,
                            "Duration pattern labels exceed the checked timestamp range.",
                        ));
                    }
                    formatter.format(
                        nanos.round_ties_even() as i64,
                        crate::data::TimeUnit::Nanoseconds,
                    )
                })
                .collect();
        }
        if time_format.is_some()
            || arguments.interval.is_some()
            || ((arguments.seconds.is_some() || arguments.time_width.is_some()) && !duration)
        {
            return Err(unsupported(
                "Time formatting/intervals require a time guide.",
            ));
        }
        if let Some(max_label_bytes) = ggplot_bytes
            && numeric_format.is_none()
            && arguments.specifier.is_none()
        {
            let values = values
                .iter()
                .map(|value| match value {
                    ScaleValue::Number(n) if n.is_finite() => Ok(*n),
                    _ => Err(unsupported(
                        "Numeric tick formatting requires finite numeric values.",
                    )),
                })
                .collect::<ChartResult<Vec<_>>>()?;
            return if is_duration(axis) {
                crate::typography::ggplot_duration_labels(&values, max_label_bytes)
            } else {
                crate::typography::ggplot_numeric_labels(&values, max_label_bytes)
            };
        }
        let formatter = family.tick_format(
            &[domain.start().into(), domain.end().into()],
            arguments.count.unwrap_or(10.),
            numeric_format
                .map(|f| f.specifier.as_str())
                .or(arguments.specifier.as_deref()),
            numeric_format.map_or_else(NumericLocale::default, |f| f.locale.clone()),
        )?;
        return values
            .iter()
            .map(|value| match value {
                ScaleValue::Number(n) if n.is_finite() => Ok(formatter.format(*n)),
                _ => Err(unsupported(
                    "Numeric tick formatting requires finite numeric values.",
                )),
            })
            .collect();
    }
    match &axis.scale {
        ResolvedScale::Provider(s) if numeric_format.is_none() && time_format.is_none() => {
            return s.labels(values, arguments, budget);
        }
        ResolvedScale::Calendar(_) | ResolvedScale::Utc(_) => {
            if numeric_format.is_some() {
                return Err(unsupported("Numeric formatting requires a numeric guide."));
            }
            // D3's time scale ignores the numeric formatter argument. A supplied time
            // descriptor is the independent override and uses this scale's calendar.
            let calendar = match &axis.scale {
                ResolvedScale::Calendar(s) => s.calendar().clone(),
                _ => Calendar::new(CalendarZone::Utc)?,
            };
            let date = matches!(axis.spec.scale, super::AxisScale::Date { .. });
            let mut format = time_format.cloned().unwrap_or_default();
            if date && format.pattern.is_none() {
                format.pattern = Some("%Y-%m-%d".into());
            }
            let formatter = format.prepare(calendar.clone())?;
            return values
                .iter()
                .map(|value| match value {
                    ScaleValue::Timestamp { value, unit } => {
                        let value = if date {
                            calendar.floor(
                                *value,
                                *unit,
                                CalendarInterval::new(CalendarUnit::Day),
                            )?
                        } else {
                            *value
                        };
                        formatter.format(value, *unit)
                    }
                    _ => Err(unsupported(
                        "Time tick formatting requires exact timestamps.",
                    )),
                })
                .collect();
        }
        _ => {}
    }
    if numeric_format.is_some() || time_format.is_some() {
        return Err(unsupported(
            "This guide cannot apply the requested built-in formatter.",
        ));
    }
    Ok(values
        .iter()
        .map(|value| match value {
            ScaleValue::Number(n) => crate::number::ecmascript(*n),
            ScaleValue::Category(s) => s.clone(),
            ScaleValue::MissingCategory => "NA".into(),
            ScaleValue::Timestamp { value, .. } => value.to_string(),
        })
        .collect())
}

/// Explicit R label policy uses the retained calendar and shared token renderer.
pub(super) fn reference_time_labels(
    axis: &ResolvedAxis,
    values: &[ScaleValue],
    format: &GgplotTimeFormat,
) -> ChartResult<Vec<String>> {
    if is_duration(axis) {
        let formatter = format.prepare(Calendar::new(CalendarZone::Utc)?)?;
        return values
            .iter()
            .map(|value| match value {
                ScaleValue::Number(seconds) => formatter.reference_seconds(*seconds),
                _ => Err(unsupported("R duration labels require numeric seconds.")),
            })
            .collect();
    }
    let calendar = match &axis.scale {
        ResolvedScale::Calendar(s) => s.calendar().clone(),
        ResolvedScale::Utc(_) => Calendar::new(CalendarZone::Utc)?,
        _ => {
            return Err(unsupported(
                "R date/time formatting requires a Date or datetime axis.",
            ));
        }
    };
    let formatter = format.prepare(calendar.clone())?;
    let date = matches!(axis.spec.scale, super::AxisScale::Date { .. });
    values
        .iter()
        .map(|value| {
            let ScaleValue::Timestamp { value, unit } = value else {
                return Err(unsupported(
                    "R date/time formatting requires exact timestamps.",
                ));
            };
            let value = if date {
                calendar.floor(*value, *unit, CalendarInterval::new(CalendarUnit::Day))?
            } else {
                *value
            };
            formatter.format(value, *unit)
        })
        .collect()
}
