//! Tick selection over retained scales. This module never trains a scale or projects marks.
use super::*;
use crate::{ChartResult, DiagnosticCode, composition::ScaleValue, scales::*};

#[derive(Default)]
pub(super) struct SelectedGuideValues {
    pub values: Vec<ScaleValue>,
    pub names: Option<Vec<String>>,
    pub transformed: Option<Vec<f64>>,
}

/// Resolve drawable ticks and retain the semantic selection before projection.
pub(super) fn resolve_with_values(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
    values: &mut SelectedGuideValues,
) -> ChartResult<Vec<GuideTick>> {
    let mut selected = None;
    let ticks = resolve_inner(chart, axis, style, r, &mut selected)?;
    *values = selected.unwrap_or_else(|| SelectedGuideValues {
        values: ticks.iter().map(|t| t.value.clone()).collect(),
        names: None,
        transformed: None,
    });
    Ok(ticks)
}
pub(super) fn resolve(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
    resolve_inner(chart, axis, style, r, &mut None)
}
fn resolve_inner(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
    selected: &mut Option<SelectedGuideValues>,
) -> ChartResult<Vec<GuideTick>> {
    if style.breaks_function.is_some()
        && !matches!(
            axis.spec.scale,
            AxisScale::Auto
                | AxisScale::Linear(_)
                | AxisScale::Nonlinear { .. }
                | AxisScale::Band(_)
                | AxisScale::Point(_)
                | AxisScale::Date { .. }
                | AxisScale::Utc { .. }
                | AxisScale::Calendar { .. }
                | AxisScale::Secondary { .. }
        )
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Positional break functions require a continuous numeric or discrete primary scale.",
        ));
    }
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && style.profile == GuideProfile::LibraryV1
        && let Some(ticks) = &style.guide_ticks
        && matches!(&axis.spec.scale, AxisScale::Nonlinear { transform: ScaleTransform::Ggplot { transform }, .. } if !transform.is_pointwise())
    {
        let mut configured = style.clone();
        configured.tick_values = Some(ticks.iter().map(|tick| tick.value.clone()).collect());
        configured.tick_format = Some(GuideFormatter::Labels(
            ticks.iter().map(|tick| tick.label.clone()).collect(),
        ));
        configured.guide_ticks = None;
        return resolve_inner(chart, axis, &configured, r, selected);
    }
    if style.tick_format.is_none()
        && style.guide_ticks.is_none()
        && style.number_format.is_none()
        && style.numeric_format.is_none()
        && style.time_format.is_none()
    {
        let policy = match &axis.spec.scale {
            AxisScale::Binned { spec, .. } => Some(&spec.labels),
            _ => axis.spec.discrete.as_deref().map(|p| &p.guide.labels),
        };
        if let Some(GgplotGuideLabels::Registered {
            operation,
            parameters,
        }) = policy
        {
            let mut configured = style.clone();
            configured.tick_format = Some(GuideFormatter::Registered {
                operation: operation.clone(),
                parameters: parameters.clone(),
            });
            return resolve_inner(chart, axis, &configured, r, selected);
        }
    }
    let reference_labels = reference_numeric_labels(chart, axis, style)
        || reference_temporal_labels(chart, axis, style);
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && matches!(axis.spec.scale, AxisScale::Duration(domain) if domain.explicit.is_none() || chart.positional_empty.contains(&axis.spec.id))
        && axis.spec.limits_function.is_none()
        && axis.spec.numeric_limits.is_none()
        && style.tick_values.is_none()
        && style.guide_ticks.is_none()
        && (chart.positional_empty.contains(&axis.spec.id)
            || chart.scale_domains().get(&axis.spec.id).is_none_or(|d| {
                if axis.spec.side.horizontal() {
                    d.x.is_none()
                } else {
                    d.y.is_none()
                }
            }))
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Untrained duration guide breaks and labels cannot resolve a reference axis.",
        ));
    }
    if chart.positional_empty.contains(&axis.spec.id) {
        if reference_labels {
            let mut empty = style.clone();
            empty.tick_values = Some(Vec::new());
            empty.guide_ticks = None;
            empty.breaks_function = None;
            return configured_with_names(chart, axis, &empty, r, None, selected);
        }
        return Ok(vec![]);
    }
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && matches!(axis.scale, ResolvedScale::Unbounded(_))
        && matches!(axis.space, crate::grammar::ValueSpace::Timestamp { .. })
        && !(chart.positional_limits().contains_key(&axis.spec.id)
            && matches!(&axis.scale, ResolvedScale::Unbounded(scale)
                if scale.viewport().iter().all(|v| v.0 == f64::NEG_INFINITY)))
        && style.visible
        && style.tick_values.is_none()
        && style.guide_ticks.is_none()
        && style.breaks_function.is_none()
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Unbounded temporal axes cannot select automatic breaks.",
        ));
    }
    if !reference_labels
        && !matches!(
            axis.spec.scale,
            AxisScale::Nonlinear {
                transform: ScaleTransform::Ggplot { .. },
                ..
            }
        )
        && matches!(axis.scale, ResolvedScale::Unbounded(_))
        && axis.spec.limits_function.is_none()
        && axis.spec.numeric_limits.is_none()
        && !matches!(axis.spec.scale, AxisScale::Binned { .. })
    {
        return Ok(vec![]);
    }
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && !chart.positional_limits().contains_key(&axis.spec.id)
        && match axis.spec.scale {
            AxisScale::Auto => !axis.space.is_categorical(),
            AxisScale::Linear(domain)
            | AxisScale::Duration(domain)
            | AxisScale::Nonlinear { domain, .. } => domain.explicit.is_none(),
            _ => false,
        }
        && chart.scale_domains().get(&axis.spec.id).is_none_or(|d| {
            if axis.spec.side.horizontal() {
                d.x.is_none()
            } else {
                d.y.is_none()
            }
        })
    {
        if reference_labels {
            if matches!(axis.scale, ResolvedScale::Unbounded(_))
                && style.breaks_function.is_none()
                && style.tick_values.is_none()
                && style.guide_ticks.is_none()
                && !matches!(style.minor_breaks, Some(MinorBreaks::Registered(_)))
            {
                return configured_with_names(chart, axis, style, r, None, selected);
            }
            let mut empty = style.clone();
            empty.tick_values = Some(Vec::new());
            empty.guide_ticks = None;
            empty.breaks_function = None;
            return configured_with_names(chart, axis, &empty, r, None, selected);
        }
        return Ok(vec![]);
    }
    if matches!(axis.scale, ResolvedScale::SecondaryDiscrete { .. }) {
        if style.breaks_function.is_some() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Reference discrete secondary axes reject break callbacks.",
            ));
        }
        return super::secondary_discrete::resolve(chart, axis, style, r);
    }
    if let ResolvedScale::SecondaryTime { axis: time, .. } = &axis.scale {
        if style != &axis.spec.guide {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Additional guides must reference a primary positional scale.",
            ));
        }
        return resolve_inner(chart, time, style, r, selected);
    }
    if matches!(axis.spec.scale, AxisScale::Binned { .. })
        && style.guide_ticks.is_none()
        && style.tick_values.is_none()
        && style.tick_arguments.is_none()
    {
        let bins = super::axes::positional_bins(&axis.spec, &axis.space)?;
        let bounds = bins.panel_limits();
        let viewport = match &axis.scale {
            ResolvedScale::Linear(s) => [s.viewport().minimum(), s.viewport().maximum()],
            ResolvedScale::Nonlinear(s) => [
                s.transformed_viewport().minimum(),
                s.transformed_viewport().maximum(),
            ],
            ResolvedScale::Unbounded(s) => {
                let [a, b] = s.viewport().map(|v| v.0);
                [a.min(b), a.max(b)]
            }
            _ => bounds.map(|v| v.0),
        };
        let empty_entries = (bins.authored().bins.empty_population
            && bins.authored().bins.limits.is_none()
            && bins.function_limits().is_none())
        .then(|| {
            bins.empty_axis_guide_entries(
                viewport,
                r.max_ticks,
                if reference_numeric_labels(chart, axis, style) {
                    &GgplotGuideLabels::Hidden
                } else {
                    &bins.authored().labels
                },
                r.limits.max_text_bytes,
                match &axis.spec.scale {
                    AxisScale::Binned { spec, .. } => spec.breaks_function.as_deref().map(|call| {
                        let count = match spec.bins.breaks {
                            crate::scales::GgplotBreaks::Nice(n)
                            | crate::scales::GgplotBreaks::Equal(n) => n,
                            _ => unreachable!("validated function cuts"),
                        };
                        (call, chart.break_registrations.as_ref(), count)
                    }),
                    _ => None,
                },
            )
        })
        .transpose()?;
        let entries = empty_entries
            .as_deref()
            .unwrap_or_else(|| bins.guide_entries())
            .iter()
            .filter(|e| {
                reference_numeric_labels(chart, axis, style)
                    || (e.transformed.0.is_finite()
                        && e.transformed.0 >= viewport[0]
                        && e.transformed.0 <= viewport[1])
            })
            .collect::<Vec<_>>();
        let automatic_labels = if matches!(bins.authored().labels, GgplotGuideLabels::Automatic)
            && !reference_numeric_labels(chart, axis, style)
        {
            Some(if entries.iter().any(|e| e.name.is_some()) {
                entries
                    .iter()
                    .map(|e| e.name.clone().unwrap_or_default())
                    .collect()
            } else {
                crate::typography::ggplot_numeric_labels(
                    &entries.iter().map(|e| e.value.0).collect::<Vec<_>>(),
                    r.max_ticks,
                )?
            })
        } else {
            None
        };
        crate::limits::require_within(entries.len() <= r.max_ticks, "positional bin guide tick")?;
        let mut resolved = style.clone();
        if style.tick_format.is_some()
            || style.number_format.is_some()
            || style.numeric_format.is_some()
            || style.time_format.is_some()
        {
            resolved.tick_values = Some(
                entries
                    .iter()
                    .map(|e| ScaleValue::Number(e.value.0))
                    .collect(),
            );
        } else {
            let mut bytes = r.limits.max_text_bytes;
            let mut ticks = Vec::new();
            for (i, entry) in entries.iter().enumerate() {
                let label = automatic_labels.as_ref().map_or_else(
                    || entry.label.clone().unwrap_or_default(),
                    |labels| labels[i].clone(),
                );
                crate::limits::require_within(label.len() <= bytes, "binned guide label byte")?;
                bytes -= label.len();
                let value = ScaleValue::Number(entry.value.0);
                if let Some(position) = axis.map_value(&value)?
                    && style.visible
                {
                    ticks.push(GuideTick {
                        value,
                        position,
                        label,
                    });
                }
            }
            return Ok(ticks);
        }
        let names = entries.iter().any(|e| e.name.is_some()).then(|| {
            entries
                .iter()
                .map(|e| e.name.clone().unwrap_or_default())
                .collect::<Vec<_>>()
        });
        return configured_with_names(chart, axis, &resolved, r, names.as_deref(), selected);
    }
    let automatic_time_axis = matches!(
        axis.scale,
        ResolvedScale::Utc(_) | ResolvedScale::Calendar(_)
    ) && !matches!(
        axis.spec.scale,
        AxisScale::Utc {
            interval: Some(_),
            ..
        } | AxisScale::Calendar {
            interval: Some(_),
            ..
        }
    );
    if style.uses_tick_configuration()
        || reference_labels
        || ((chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
            || (matches!(axis.spec.scale, AxisScale::Date { .. })
                || super::guide_selection::is_duration(axis)))
            && style.profile == GuideProfile::LibraryV1
            && (automatic_time_axis
                || matches!(&axis.scale, ResolvedScale::Unbounded(scale) if matches!(scale.transform(), Some(ScaleTransform::Ggplot { transform }) if !transform.is_pointwise()))
                || matches!(
                    axis.scale,
                    ResolvedScale::Linear(_)
                        | ResolvedScale::Numeric(_)
                        | ResolvedScale::Nonlinear(_)
                        | ResolvedScale::Secondary { .. }
                )))
    {
        return configured_with_names(chart, axis, style, r, None, selected);
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
        ResolvedScale::Unbounded(scale)
            if chart.positional_limits().contains_key(&axis.spec.id)
                || matches!(
                    axis.spec.scale,
                    AxisScale::Nonlinear {
                        transform: ScaleTransform::Ggplot { .. },
                        ..
                    }
                ) =>
        {
            if matches!(axis.space, crate::grammar::ValueSpace::Timestamp { .. })
                || matches!(axis.spec.scale, AxisScale::Duration(_))
            {
                let bounds = scale.viewport();
                if bounds[0].0 != f64::NEG_INFINITY || bounds[1].0 != f64::NEG_INFINITY {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Automatic temporal guide endpoints must be finite.",
                    ));
                }
            }
            let transform = match &axis.spec.scale {
                AxisScale::Nonlinear { transform, .. } => Some(transform),
                _ => None,
            };
            let (family, reverse) =
                crate::scales::ggplot_numeric_limits::positional_coordinates(transform.cloned());
            let raw = scale
                .viewport()
                .map(|v| transform.as_ref().map_or(v.0, |t| t.inverse_raw(v.0)));
            let entries = if axis.spec.numeric_limits.is_some()
                || matches!(transform, Some(ScaleTransform::Ggplot { .. }))
            {
                // Authored limits already own the transformed panel range. An
                // inverse/forward round trip can invalidate its infinite bounds.
                let mut bounds = scale.viewport().map(|v| v.0);
                bounds.sort_by(f64::total_cmp);
                GgplotContinuousGuide::default().resolve_bounds(
                    bounds,
                    family,
                    reverse,
                    r.max_ticks,
                    r.limits.max_text_bytes,
                )?
            } else {
                GgplotContinuousGuide::default().resolve(
                    [
                        crate::interpolate::Number(raw[0]),
                        crate::interpolate::Number(raw[1]),
                    ],
                    family,
                    reverse,
                    r.max_ticks,
                    r.limits.max_text_bytes,
                )?
            };
            let duration_labels = if matches!(axis.spec.scale, AxisScale::Duration(_)) {
                Some(crate::typography::ggplot_duration_labels(
                    &entries.iter().map(|e| e.value.0).collect::<Vec<_>>(),
                    r.limits.max_text_bytes,
                )?)
            } else {
                None
            };
            *selected = Some(SelectedGuideValues {
                values: entries
                    .iter()
                    .map(|e| ScaleValue::Number(e.value.0))
                    .collect(),
                names: None,
                transformed: Some(entries.iter().map(|e| e.transformed.0).collect()),
            });
            for (i, entry) in entries.into_iter().enumerate() {
                if entry.value.0.is_nan()
                    || (scale.viewport().iter().all(|v| v.0.is_finite()) && !entry.visible)
                {
                    continue;
                }
                if let Some(position) = scale.map_transformed(entry.transformed.0)? {
                    ticks.push(GuideTick {
                        value: ScaleValue::Number(entry.value.0),
                        position,
                        label: duration_labels.as_ref().map_or_else(
                            || entry.label.unwrap_or_default(),
                            |labels| labels[i].clone(),
                        ),
                    });
                }
            }
        }
        ResolvedScale::Unbounded(_) => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unbounded reference guides require their retained break candidates.",
            ));
        }
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
                ScaleTransform::Ggplot { transform } => NumericFamily::Ggplot { transform },
                ScaleTransform::Reverse => NumericFamily::Linear,
                ScaleTransform::Sqrt => NumericFamily::Pow { exponent: 0.5 },
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
            if let AxisScale::Utc {
                interval: Some(interval),
                ..
            } = axis.spec.scale
            {
                let formatter = style
                    .time_format
                    .as_ref()
                    .map(|f| f.prepare(s.calendar().clone()))
                    .transpose()?;
                if let Some(bounds) = s.tick_bounds()? {
                    for tick in UtcScale::ticks_in(bounds, s.unit(), interval, r.max_ticks)? {
                        if let Some(position) = s.map(tick.value)? {
                            let label = formatter.as_ref().map_or_else(
                                || Ok(tick.label.clone()),
                                |f| f.format(tick.value, s.unit()),
                            )?;
                            ticks.push(GuideTick {
                                value: ScaleValue::Timestamp {
                                    value: tick.value,
                                    unit: s.unit(),
                                },
                                position,
                                label,
                            });
                        }
                    }
                }
                return Ok(ticks);
            }
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
        ResolvedScale::SecondaryTime { .. } | ResolvedScale::SecondaryDiscrete { .. } => {
            unreachable!("handled before time selection")
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
    if selected.is_none() {
        *selected = Some(SelectedGuideValues {
            values: ticks.iter().map(|t| t.value.clone()).collect(),
            names: None,
            transformed: None,
        });
    }
    ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
    Ok(ticks)
}

pub(super) fn configured(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
    configured_with_names(chart, axis, style, r, None, &mut None)
}
fn configured_with_names(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    r: &LayoutRequest,
    break_names: Option<&[String]>,
    selected: &mut Option<SelectedGuideValues>,
) -> ChartResult<Vec<GuideTick>> {
    super::axes::validate_style(style, r.limits)?;
    if let Some(ticks) = super::guide_discrete::registered(chart, axis, style, r)? {
        return Ok(ticks);
    }
    let arguments = style.tick_arguments.clone().unwrap_or_default();
    let ggplot = (chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        || (matches!(axis.spec.scale, AxisScale::Date { .. })
            || super::guide_selection::is_duration(axis)))
        && style.profile == GuideProfile::LibraryV1;
    arguments.validate(r.limits.max_text_bytes)?;
    let arguments = if reference_temporal_labels(chart, axis, style) && arguments.width.is_some() {
        arguments
    } else {
        arguments.resolve_width(
            super::guide_selection::is_duration(axis),
            matches!(axis.spec.scale, AxisScale::Date { .. }),
        )?
    };
    if matches!(axis.spec.scale, AxisScale::Date { .. })
        && (arguments.seconds.is_some()
            || arguments.width.as_deref().is_some_and(|width| {
                matches!(
                    crate::scales::parse_width(width),
                    Ok((CalendarUnit::Minute, _, _))
                )
            })
            || arguments.time_width.is_some_and(|width| {
                matches!(
                    width.unit,
                    CalendarUnit::Second | CalendarUnit::Minute | CalendarUnit::Hour
                )
            }))
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Date widths require day, week, month or year units.",
        ));
    }
    if (arguments.seconds.is_some() || arguments.time_width.is_some() || arguments.width.is_some())
        && !super::guide_selection::is_duration(axis)
        && !matches!(
            axis.scale,
            ResolvedScale::Utc(_) | ResolvedScale::Calendar(_)
        )
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Fixed-second breaks require a UTC or calendar time guide.",
        ));
    }
    let secondary = r
        .axes
        .iter()
        .any(|a| a.id == axis.spec.id && matches!(a.scale, AxisScale::Secondary { .. }));
    let selected_breaks = if style.tick_values.is_none()
        && style.guide_ticks.is_none()
        && arguments.seconds.is_none()
        && arguments.time_width.is_none()
        && arguments.width.is_none()
    {
        super::guide_breaks::numeric(chart, axis, style, r.max_ticks, secondary)?
    } else {
        None
    };
    if secondary && selected_breaks.as_ref().is_some_and(|b| b.null_result) {
        return Ok(Vec::new());
    }
    let temporal_labels = reference_temporal_labels(chart, axis, style);
    // Select before enumeration: even an empty explicit list bypasses scale ticks.
    let automatic_time = if ggplot
        && style.visible
        && style.tick_values.is_none()
        && style.guide_ticks.is_none()
        && selected_breaks.is_none()
    {
        super::guide_selection::reference_time_selection(
            axis,
            &arguments,
            r.max_ticks,
            !temporal_labels,
        )?
    } else {
        None
    };
    let reference_labels = reference_numeric_labels(chart, axis, style);
    let constant = if style.tick_values.is_some() && style.guide_ticks.is_none() {
        if reference_labels {
            super::guide_breaks::numeric_constant(axis)
        } else if temporal_labels {
            let (context, bounds) = super::guide_breaks::temporal_context(axis)?;
            let n = context.normalization;
            if bounds[0].is_finite()
                && ggplot_zero_range(n.absolute(bounds[0]), n.absolute(bounds[1]))
            {
                Some(ScaleValue::Timestamp {
                    value: crate::scales::absolute_number(bounds[0], n.origin)?,
                    unit: n.unit,
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let vector_entries = if ggplot
        && constant.is_none()
        && automatic_time.is_none()
        && arguments.interval.is_none()
        && arguments.seconds.is_none()
        && arguments.time_width.is_none()
        && arguments.width.is_none()
        && let ResolvedScale::Unbounded(scale) = &axis.scale
        && let Some(ScaleTransform::Ggplot { transform }) = scale.transform()
        && !transform.is_pointwise()
    {
        let mut bounds = scale.viewport().map(|v| v.0);
        bounds.sort_by(f64::total_cmp);
        Some(
            GgplotContinuousGuide {
                count: arguments.count,
                breaks: style
                    .tick_values
                    .clone()
                    .or_else(|| {
                        style
                            .guide_ticks
                            .as_ref()
                            .map(|ticks| ticks.iter().map(|tick| tick.value.clone()).collect())
                    })
                    .or_else(|| {
                        selected_breaks
                            .as_ref()
                            .map(|selected| selected.values.clone())
                    })
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| {
                                let ScaleValue::Number(v) = value else {
                                    return Err(error(
                                        DiagnosticCode::SchemaConflict,
                                        "Numeric vector guide breaks require numeric values.",
                                    ));
                                };
                                Ok(crate::interpolate::Number(*v))
                            })
                            .collect::<ChartResult<Vec<_>>>()
                    })
                    .transpose()?,
                ..Default::default()
            }
            .resolve_bounds(
                bounds,
                NumericFamily::Ggplot { transform },
                false,
                r.max_ticks,
                r.limits.max_text_bytes,
            )?,
        )
    } else {
        None
    };
    let mut values = if let Some(entries) = &vector_entries {
        entries
            .iter()
            .map(|entry| ScaleValue::Number(entry.value.0))
            .collect()
    } else if let Some(value) = constant {
        vec![value]
    } else if let Some(values) = &style.tick_values {
        crate::limits::require_within(values.len() <= r.max_ticks, "selected guide tick")?;
        values.clone()
    } else if let Some(ticks) = &style.guide_ticks {
        crate::limits::require_within(ticks.len() <= r.max_ticks, "selected guide tick")?;
        ticks.iter().map(|tick| tick.value.clone()).collect()
    } else if let Some(selected) = &selected_breaks {
        selected.values.clone()
    } else if let Some((unit, ticks)) = &automatic_time {
        ticks
            .values
            .iter()
            .map(|value| ScaleValue::Timestamp {
                value: *value,
                unit: *unit,
            })
            .collect()
    } else if temporal_labels && matches!(axis.scale, ResolvedScale::Unbounded(_)) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Unbounded temporal axes cannot select automatic breaks.",
        ));
    } else if reference_labels
        && matches!(axis.scale, ResolvedScale::Unbounded(_))
        && !matches!(
            axis.spec.scale,
            AxisScale::Nonlinear {
                transform: ScaleTransform::Ggplot { .. },
                ..
            }
        )
    {
        Vec::new()
    } else if style.visible && (reference_labels || temporal_labels) {
        super::guide_selection::label_values(axis, &arguments, r.max_ticks)?
    } else if style.visible {
        super::guide_selection::values(axis, &arguments, r.max_ticks, ggplot)?
    } else {
        Vec::new()
    };
    if ggplot
        && !secondary
        && chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && matches!(axis.spec.scale, super::AxisScale::Date { .. })
        && let super::ResolvedScale::Calendar(scale) = &axis.scale
    {
        for value in &mut values {
            if let ScaleValue::Timestamp { value, unit } = value
                && *unit == scale.unit()
            {
                *value = scale.floor_date_guide(*value)?;
            }
        }
    }
    if reference_labels && vector_entries.is_none() {
        censor_numeric_labels(axis, &mut values);
    }
    crate::limits::require_within(values.len() <= r.max_ticks, "selected guide tick")?;
    *selected = Some(SelectedGuideValues {
        values: values.clone(),
        transformed: vector_entries
            .as_ref()
            .map(|entries| entries.iter().map(|e| e.transformed.0).collect()),
        names: break_names
            .map(<[String]>::to_vec)
            .or_else(|| selected_breaks.as_ref().and_then(|s| s.names.clone()))
            .or_else(|| {
                automatic_time
                    .as_ref()
                    .map(|(_, ticks)| ticks.labels.clone())
            }),
    });
    let mut bytes = r.limits.max_text_bytes;
    for value in &values {
        if let ScaleValue::Category(label) = value {
            crate::limits::require_within(label.len() <= bytes, "selected guide category byte")?;
            bytes -= label.len();
        }
    }
    let temporal_input = temporal_labels
        .then(|| temporal_label_input(axis, &values, selected_breaks.is_some(), secondary))
        .transpose()?;
    // Validate every semantic value before any per-value formatter. Mapping omission
    // does not alter the complete callback list or its occurrence indices.
    let positions = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if let Some(entries) = &vector_entries
                && let ResolvedScale::Unbounded(scale) = &axis.scale
            {
                return if entries[index].visible {
                    scale.map_transformed(entries[index].transformed.0)
                } else {
                    Ok(None)
                };
            }
            if let Some((_, inputs)) = &temporal_input {
                let ScaleValue::Number(offset) = inputs[index] else {
                    unreachable!()
                };
                if !offset.is_finite() {
                    return Ok(None);
                }
                if matches!(axis.scale, ResolvedScale::Unbounded(_)) {
                    return axis.guide_value_position(&inputs[index]);
                }
                if let (ScaleValue::Number(_), ResolvedScale::Calendar(scale)) =
                    (value, &axis.scale)
                {
                    let origin = temporal_input.as_ref().unwrap().0.normalization.origin;
                    return scale.map_relative_guide(
                        offset + (i128::from(origin) - i128::from(scale.origin())) as f64,
                    );
                }
            }
            if reference_labels && matches!(value, ScaleValue::Number(v) if !v.is_finite()) {
                return Ok(None);
            }
            // Secondary reference guides format the complete candidate vector before
            // censoring. An outside candidate must not invoke an invalid inverse.
            if ggplot
                && let (ResolvedScale::Secondary { view, .. }, ScaleValue::Number(v)) =
                    (&axis.scale, value)
                && !view.contains(*v)
            {
                return Ok(None);
            }
            axis.guide_value_position(value)
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let numeric = match &style.tick_format {
        Some(GuideFormatter::Numeric(format)) => Some(format.as_ref()),
        _ => style.numeric_format.as_ref(),
    };
    let time = match &style.tick_format {
        Some(GuideFormatter::Time(format)) => Some(format.as_ref()),
        _ => style.time_format.as_ref(),
    };
    let labels = if let Some(entries) = &vector_entries
        && style.guide_ticks.is_none()
        && numeric.is_none()
        && style.tick_format.is_none()
    {
        if arguments.specifier.is_none()
            && let Some(names) = selected_breaks
                .as_ref()
                .and_then(|selected| selected.names.as_ref())
        {
            names.clone()
        } else {
            entries
                .iter()
                .map(|e| e.label.clone().unwrap_or_default())
                .collect()
        }
    } else if let Some(ticks) = &style.guide_ticks {
        ticks.iter().map(|tick| tick.label.clone()).collect()
    } else {
        match &style.tick_format {
            Some(GuideFormatter::GgplotTime(format)) => {
                super::guide_selection::reference_time_labels(
                    axis,
                    &values,
                    format,
                    selected_breaks.is_some(),
                )?
            }
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
                crate::grammar::GuideLabelsInput {
                    values: temporal_input
                        .as_ref()
                        .map_or(values.as_slice(), |(_, values)| values.as_slice()),
                    names: if break_names.is_some() {
                        break_names
                    } else if let Some(selected) = &selected_breaks {
                        selected.names.as_deref()
                    } else if temporal_labels {
                        automatic_time
                            .as_ref()
                            .map(|(_, ticks)| ticks.labels.as_slice())
                    } else {
                        None
                    },
                    temporal: temporal_input.as_ref().map(|(context, _)| *context),
                    parameters,
                    limits: r.limits,
                },
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
            _ if numeric.is_none()
                && time.is_none()
                && arguments.specifier.is_none()
                && selected_breaks
                    .as_ref()
                    .is_some_and(|selected| selected.names.is_some()) =>
            {
                selected_breaks.as_ref().unwrap().names.clone().unwrap()
            }
            _ if reference_labels
                && numeric.is_none()
                && time.is_none()
                && arguments.specifier.is_none() =>
            {
                let values = values
                    .iter()
                    .map(|v| {
                        let ScaleValue::Number(n) = v else {
                            unreachable!()
                        };
                        *n
                    })
                    .collect::<Vec<_>>();
                let transform = match &axis.scale {
                    ResolvedScale::Unbounded(s) => {
                        s.transform().and_then(|t| t.ggplot_transform().cloned())
                    }
                    ResolvedScale::Nonlinear(s) => s.transform().ggplot_transform().cloned(),
                    ResolvedScale::Numeric(s) => s.family().ggplot_transform().cloned(),
                    _ => None,
                };
                crate::scales::transform_numeric_labels(
                    transform.as_ref(),
                    &values,
                    r.limits.max_text_bytes,
                )?
                .into_iter()
                .map(Option::unwrap_or_default)
                .collect()
            }
            _ if temporal_labels
                && automatic_time.is_none()
                && numeric.is_none()
                && time.is_none() =>
            {
                let (context, inputs) = temporal_input.as_ref().unwrap();
                let guide = GgplotTemporalGuide {
                    origin: context.normalization.origin,
                    unit: context.normalization.unit,
                    zone: context.zone.clone(),
                    arguments: GgplotTemporalGuideArguments {
                        date: context.normalization.date,
                        ..Default::default()
                    },
                };
                guide
                    .default_labels(
                        &inputs
                            .iter()
                            .map(|v| {
                                let ScaleValue::Number(n) = v else {
                                    unreachable!()
                                };
                                n.floor()
                            })
                            .collect::<Vec<_>>(),
                    )?
                    .into_iter()
                    .map(Option::unwrap_or_default)
                    .collect()
            }
            _ if automatic_time.is_some() && numeric.is_none() && time.is_none() => automatic_time
                .as_ref()
                .expect("checked automatic selection")
                .1
                .labels
                .clone(),
            _ => super::guide_selection::labels(
                axis,
                &values,
                &arguments,
                numeric,
                time,
                r.max_ticks,
                ggplot.then_some(r.limits.max_text_bytes),
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

// Positional scale callbacks have a different censoring contract from color guides.
fn reference_numeric_labels(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
) -> bool {
    chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && style.profile == GuideProfile::LibraryV1
        && (matches!(style.tick_format, Some(GuideFormatter::Registered { .. }))
            || style.breaks_function.is_some()
            || style.tick_values.is_some()
            || matches!(style.minor_breaks, Some(MinorBreaks::Registered(_))))
        && matches!(
            axis.spec.scale,
            AxisScale::Auto
                | AxisScale::Linear(_)
                | AxisScale::Nonlinear { .. }
                | AxisScale::Binned { .. }
        )
        && matches!(
            axis.scale,
            ResolvedScale::Linear(_)
                | ResolvedScale::Numeric(_)
                | ResolvedScale::Nonlinear(_)
                | ResolvedScale::Unbounded(_)
        )
}
fn censor_numeric_labels(axis: &ResolvedAxis, values: &mut [ScaleValue]) {
    // Sorting a partly missing, infinite reference range retains one endpoint.
    // Its absent upper bound does not censor positive infinity.
    let one_sided = match &axis.space {
        crate::grammar::ValueSpace::Scaled { scale, .. } => scale
            .binned
            .as_ref()
            .and_then(|bins| bins.function_limits())
            .and_then(|limits| {
                let values = limits.iter().filter(|v| !v.0.is_nan()).collect::<Vec<_>>();
                (values.len() == 1 && values[0].0.is_infinite()).then(|| [values[0].0, f64::NAN])
            }),
        _ => None,
    };
    for value in values {
        if let ScaleValue::Number(v) = value {
            let (transformed, bounds) = match &axis.scale {
                ResolvedScale::Linear(s) => (*v, [s.viewport().minimum(), s.viewport().maximum()]),
                ResolvedScale::Numeric(s) => (*v, [s.viewport().minimum(), s.viewport().maximum()]),
                ResolvedScale::Nonlinear(s) => (
                    s.transform().forward_raw(*v),
                    [
                        s.transformed_viewport().minimum(),
                        s.transformed_viewport().maximum(),
                    ],
                ),
                ResolvedScale::Unbounded(s) => {
                    let transform = match &axis.spec.scale {
                        AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
                        AxisScale::Binned { spec, .. } => spec.transform.clone(),
                        _ => None,
                    };
                    let bounds = s.viewport().map(|x| x.0);
                    (
                        transform.map_or(*v, |t| t.forward_raw(*v)),
                        one_sided.unwrap_or([bounds[0].min(bounds[1]), bounds[0].max(bounds[1])]),
                    )
                }
                _ => continue,
            };
            if transformed.is_nan() || transformed < bounds[0] || transformed > bounds[1] {
                *v = f64::NAN;
            }
        }
    }
}

fn reference_temporal_labels(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
) -> bool {
    chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && style.profile == GuideProfile::LibraryV1
        && (matches!(style.tick_format, Some(GuideFormatter::Registered { .. }))
            || style.breaks_function.is_some()
            || matches!(style.minor_breaks, Some(MinorBreaks::Registered(_))))
        && matches!(
            axis.spec.scale,
            AxisScale::Auto
                | AxisScale::Date { .. }
                | AxisScale::Utc { .. }
                | AxisScale::Calendar { .. }
        )
        && matches!(axis.space, crate::grammar::ValueSpace::Timestamp { .. })
}
pub(super) fn temporal_label_input<'a>(
    axis: &'a ResolvedAxis,
    values: &[ScaleValue],
    selected_offsets: bool,
    secondary: bool,
) -> ChartResult<(crate::grammar::GuideTemporalContext<'a>, Vec<ScaleValue>)> {
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
    let numbers=values.iter().map(|value| {
        let offset=match value {
            ScaleValue::Timestamp { value, unit } if *unit == normalization.unit => (i128::from(*value)-i128::from(*origin)) as f64,
            ScaleValue::Number(v) if !v.is_finite() || selected_offsets => *v,
            _ => return Err(error(DiagnosticCode::SchemaConflict, "Temporal label values must use the source timestamp unit or explicit missing numbers.")),
        };
        let visible=secondary || match &axis.scale {
            ResolvedScale::Calendar(s) => {
                let relative=offset+(i128::from(*origin)-i128::from(s.origin())) as f64;
                let view = s.relative_viewport();
                let value = s.reference_relative(relative, normalization.date);
                value >= s.reference_relative(view.minimum(), normalization.date) && value <= s.reference_relative(view.maximum(), normalization.date)
            }
            ResolvedScale::Utc(s) => {
                let view=s.viewport();
                offset >= (i128::from(view.start)-i128::from(*origin)) as f64 && offset <= (i128::from(view.end)-i128::from(*origin)) as f64
            }
            ResolvedScale::Unbounded(s) => {
                let [a,b]=s.viewport().map(|v|v.0);
                offset>=a.min(b) && offset<=a.max(b)
            }
            _ => false,
        };
        Ok(ScaleValue::Number(if visible { offset } else { f64::NAN }))
    }).collect::<ChartResult<Vec<_>>>()?;
    Ok((
        crate::grammar::GuideTemporalContext {
            normalization,
            zone,
        },
        numbers,
    ))
}
