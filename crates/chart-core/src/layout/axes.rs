use super::{project, *};
use crate::grammar::{DomainContributions, PreparedChart, ValueSpace};
use crate::scales::*;
use crate::state::AxisWindow;
use crate::{ChartResult, DiagnosticCode, Rect, ScaleId};

pub(super) fn validate_specs(axes: &[AxisSpec], limits: crate::Limits) -> ChartResult<()> {
    use std::collections::BTreeSet;
    crate::limits::require_within(axes.len() <= 4, "independent axis count (four)")?;
    let mut ids = BTreeSet::new();
    for a in axes {
        validate_style(&a.guide, limits)?;
        if let Some(values) = &a.temporal_limits
            && (a.numeric_limits.is_some()
                || a.limits_function.is_some()
                || !matches!(
                    a.scale,
                    AxisScale::Date { .. } | AxisScale::Utc { .. } | AxisScale::Calendar { .. }
                )
                || values
                    .iter()
                    .flatten()
                    .any(|v| !matches!(v, crate::composition::ScaleValue::Timestamp { .. })))
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Temporal limits require timestamp endpoints and a temporal axis without numeric limits or a limit function.",
            ));
        }
        if a.numeric_limits.is_some()
            && (a.limits_function.is_some()
                || !matches!(
                    a.scale,
                    AxisScale::Auto
                        | AxisScale::Linear(_)
                        | AxisScale::Duration(_)
                        | AxisScale::Nonlinear { .. }
                ))
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Authored numeric limits require a numeric axis without a limit function.",
            ));
        }

        if let Some(values) = &a.continuous_limits {
            crate::limits::require_within(
                values.len() <= limits.max_items,
                "continuous category limits",
            )?;
            if values.iter().any(|v| v.0.is_nan()) && values.len() != 2 {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Continuous category limits containing missing values require two endpoints.",
                ));
            }
            if !matches!(
                a.scale,
                AxisScale::Auto | AxisScale::Band(_) | AxisScale::Point(_)
            ) {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Continuous category limits require a reference band or point axis.",
                ));
            }
        }
        if let Some(expansion) = a.expansion {
            expansion.expand(Bounds::new(0., 1.)?)?;
            if !matches!(
                a.scale,
                AxisScale::Auto
                    | AxisScale::Linear(_)
                    | AxisScale::Duration(_)
                    | AxisScale::Binned { .. }
                    | AxisScale::Nonlinear { .. }
                    | AxisScale::Band(_)
                    | AxisScale::Point(_)
                    | AxisScale::Utc { .. }
                    | AxisScale::Date { .. }
                    | AxisScale::Calendar { .. }
            ) {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Reference expansion requires a linear, nonlinear, band, point or time axis.",
                ));
            }
        }
        if !ids.insert(a.id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Positional scales need unique IDs.",
            ));
        }
        if let Some(v) = a.viewport {
            v.distinct()?;
        }
        if let Some(v) = a.range {
            v.distinct()?;
        }
        if let AxisScale::Secondary {
            source,
            factor,
            offset,
            ref transform,
        } = a.scale
        {
            if let Some(transform) = transform {
                secondary_mapping(transform)?;
            }
            if !factor.is_finite()
                || factor == 0.
                || !offset.is_finite()
                || a.viewport.is_some()
                || a.range.is_some()
                || a.outside != OutsidePolicy::Extend
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Secondary axes need finite nonzero factor/finite offset and inherit viewport/range/outside policies.",
                ));
            }
            let primary = axes
                .iter()
                .find(|p| p.id == source && p.side.horizontal() == a.side.horizontal())
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Secondary axis requires a source axis in the same orientation.",
                    )
                })?;
            if !matches!(
                primary.scale,
                AxisScale::Auto
                    | AxisScale::Band(_)
                    | AxisScale::Point(_)
                    | AxisScale::Linear(_)
                    | AxisScale::Duration(_)
                    | AxisScale::Numeric(_)
                    | AxisScale::Nonlinear { .. }
                    | AxisScale::Utc { .. }
                    | AxisScale::Date { .. }
                    | AxisScale::Calendar { .. }
            ) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary axes require a primary numeric, time or reference categorical scale directly.",
                ));
            }
        }
        let range = Bounds::new(0., 1.)?;
        match &a.scale {
            AxisScale::Calendar { spec, interval } => {
                TimeAxisScale::validate_axis_spec(spec)?;
                if let Some(interval) = interval {
                    interval.validate()?;
                }
            }
            AxisScale::Numeric(spec) => {
                NumericScale::new(spec.clone())?;
            }
            AxisScale::Binned { spec, prepared } => {
                spec.validate()?;
                if let Some(prepared) = prepared {
                    prepared.validate()?;
                }
            }
            AxisScale::Linear(domain) | AxisScale::Duration(domain) => {
                domain.resolve(None)?;
            }
            AxisScale::Nonlinear {
                transform: transform @ ScaleTransform::Ggplot { .. },
                domain,
            } => {
                transform.validate()?;
                domain.resolve(None)?;
                for bounds in domain.explicit.into_iter().chain(a.viewport) {
                    for value in [bounds.start(), bounds.end()] {
                        if transform.forward(value)?.is_none() {
                            return Err(error(
                                DiagnosticCode::NumericalDomain,
                                "Authored axis limits are outside the transform domain.",
                            ));
                        }
                    }
                }
            }
            AxisScale::Nonlinear { transform, domain } => {
                NonlinearScale::resolve(
                    None,
                    *domain,
                    transform.clone(),
                    range,
                    if a.numeric_limits.is_some() {
                        None
                    } else {
                        a.viewport
                    },
                    a.outside,
                )?;
            }
            AxisScale::D3Band(options) => {
                BandScale::resolve_d3(&[], options, range)?;
            }
            AxisScale::D3Point(options) => {
                PointScale::resolve_d3(&[], options, range)?;
            }
            AxisScale::Band(options) => {
                BandScale::resolve(&[], options, range)?;
            }
            AxisScale::Point(options) => {
                PointScale::resolve(&[], options, range)?;
            }
            AxisScale::Session(calendar) => {
                SessionScale::new(calendar.clone(), range, None, a.outside)?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn validate_style(a: &GuideStyle, limits: crate::Limits) -> ChartResult<()> {
    if let Some(options) = &a.ggplot_axis {
        options.validate()?;
    }
    if a.breaks_function.is_some() && (a.tick_values.is_some() || a.guide_ticks.is_some()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Registered break selection cannot compete with fixed guide values.",
        ));
    }
    if let Some(components) = &a.components {
        components.validate(limits)?;
    }
    if let Some(geometry) = &a.geometry {
        geometry.validate()?;
    }
    if let Some(super::MinorBreaks::TimeWidth(width)) = &a.minor_breaks {
        crate::limits::require_within(width.len() <= limits.max_text_bytes, "minor time width")?;
    }
    if let Some(super::MinorBreaks::Numeric(values)) = &a.minor_breaks {
        crate::limits::require_within(values.len() <= limits.max_items, "authored minor break")?;
    }
    if let Some(values) = &a.tick_values {
        crate::limits::require_within(values.len() <= limits.max_items, "authored guide tick")?;
    }
    if let Some(arguments) = &a.tick_arguments {
        arguments.validate(limits.max_text_bytes)?;
    }
    if a.tick_format.is_some()
        && (a.number_format.is_some()
            || a.numeric_format.is_some()
            || a.time_format.is_some()
            || a.guide_ticks.is_some())
        || a.tick_values.is_some() && a.guide_ticks.is_some()
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Guide value/formatter controls cannot conflict with coupled legacy labels or other formatters.",
        ));
    }
    if let Some(format) = &a.tick_format {
        match format {
            GuideFormatter::Numeric(f) => {
                f.prepare()?;
            }
            GuideFormatter::Time(f) => {
                f.prepare(Calendar::new(CalendarZone::Utc)?)?;
            }
            GuideFormatter::GgplotTime(f) => {
                f.prepare(Calendar::new(CalendarZone::Utc)?)?;
            }
            GuideFormatter::Labels(labels) => {
                crate::limits::require_within(
                    labels.len() <= limits.max_items,
                    "authored guide label",
                )?;
                let mut bytes = limits.max_text_bytes;
                for label in labels {
                    crate::limits::require_within(
                        label.len() <= bytes,
                        "explicit guide label byte",
                    )?;
                    bytes -= label.len();
                }
            }
            GuideFormatter::Registered { parameters, .. } => {
                crate::grammar::guide_extensions::validate_parameters(parameters)?;
            }
        }
    }
    if !a.label_rotation.is_finite() || a.label_rotation.abs() > 360. {
        return Err(error(
            DiagnosticCode::Validation,
            "Axis label rotation must be -360..360 degrees.",
        ));
    }
    if let Some(t) = &a.typography {
        t.validate(limits)?;
    }
    if let Some(t) = &a.title {
        t.validate(limits)?;
    }

    if let Some(f) = &a.number_format {
        f.validate()?;
    }
    if let Some(f) = &a.numeric_format {
        f.prepare()?;
        if a.number_format.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "An axis may select only one numeric formatter.",
            ));
        }
    }
    if let Some(f) = &a.time_format {
        f.prepare(Calendar::new(CalendarZone::Utc)?)?;
        if a.number_format.is_some() || a.numeric_format.is_some() || a.guide_ticks.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Time formatting cannot be combined with numeric or custom guide labels.",
            ));
        }
    }
    if let Some(ticks) = &a.guide_ticks {
        if a.number_format.is_some() || a.numeric_format.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Explicit custom labels cannot also request a numeric formatter.",
            ));
        }
        if a.profile == GuideProfile::LibraryV1 && ticks.iter().any(|tick| tick.label.is_empty()) {
            return Err(error(
                DiagnosticCode::Validation,
                "Custom guide labels must be nonempty.",
            ));
        }
    }
    Ok(())
}

pub(super) const MAX_GUIDES: usize = 64;
pub(super) fn validate_guides(
    axes: &[AxisSpec],
    guides: &[GuideSpec],
    limits: crate::Limits,
) -> ChartResult<()> {
    crate::limits::require_within(
        axes.len().saturating_add(guides.len()) <= MAX_GUIDES,
        "positional guide count (64)",
    )?;
    let mut ids: std::collections::BTreeSet<_> =
        axes.iter().map(|a| a.default_guide().id).collect();
    for guide in guides {
        if !ids.insert(guide.id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Positional guides need unique guide identities.",
            ));
        }
        if guide.translation.iter().any(|v| !v.is_finite()) {
            return Err(error(
                DiagnosticCode::Validation,
                "Guide translation must be finite.",
            ));
        }
        validate_style(&guide.style, limits)?;
        let scale = axes.iter().find(|a| a.id == guide.scale).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Guide references an absent positional scale.",
            )
        })?;
        if scale.side.horizontal() != guide.side.horizontal() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Guide orientation differs from its shared scale.",
            ));
        }
        if matches!(scale.scale, AxisScale::Secondary { .. }) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Additional guides require a primary positional scale.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_definition_axes(
    definition: &crate::grammar::ChartDefinition,
) -> ChartResult<()> {
    if definition.axes.is_empty() {
        let axes = [
            AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
            AxisSpec::new(ScaleId::new(1), AxisSide::Left),
        ];
        return validate_guides(&axes, &definition.guides, crate::Limits::default());
    }
    validate_guides(
        &definition.axes,
        &definition.guides,
        crate::Limits::default(),
    )?;
    validate_specs(&definition.axes, crate::Limits::default())?;
    for layer in &definition.layers {
        for (id, horizontal) in [(layer.scales.x, true), (layer.scales.y, false)] {
            let axis = definition
                .axes
                .iter()
                .find(|a| a.id == id && a.side.horizontal() == horizontal)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Layer scale binding requires an axis in the same orientation.",
                    )
                })?;
            if matches!(axis.scale, AxisScale::Secondary { .. }) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary axes are guide-only; bind layers to the primary scale.",
                ));
            }
        }
    }
    Ok(())
}

fn resolve_axis_inner(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    plot: Rect,
    major_values: &mut super::guide_ticks::SelectedGuideValues,
) -> ChartResult<ResolvedAxis> {
    if let Some(f) = &spec.number_format {
        f.validate()?;
    }
    let windows = chart.state().axis_windows();
    let window = windows.get(&spec.id);
    if let AxisScale::Secondary {
        source,
        factor,
        offset,
        ref transform,
    } = spec.scale
    {
        if !factor.is_finite()
            || factor == 0.
            || !offset.is_finite()
            || spec.viewport.is_some()
            || window.is_some()
            || spec.range.is_some()
            || spec.outside != OutsidePolicy::Extend
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Secondary axes need a finite nonzero multiplier and finite offset, and inherit source viewport/range/outside policies.",
            ));
        }
        if chart.scale_domains().contains_key(&spec.id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Secondary axes are guide-only; layers bind their primary numeric scale.",
            ));
        }
        let primary = r
            .axes
            .iter()
            .find(|a| a.id == source && a.side.horizontal() == spec.side.horizontal())
            .ok_or_else(|| {
                error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary axis requires a source axis in the same orientation.",
                )
            })?;
        if matches!(primary.scale, AxisScale::Secondary { .. }) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Secondary axes must reference a primary scale directly.",
            ));
        }
        let mut primary = primary.clone();
        primary.visible = spec.visible;
        let resolved = resolve_axis_inner(chart, r, &primary, plot, &mut Default::default())?;
        if matches!(
            resolved.scale,
            ResolvedScale::Utc(_) | ResolvedScale::Calendar(_)
        ) {
            return resolve_secondary_time(
                chart,
                r,
                spec,
                source,
                factor,
                offset,
                transform.is_some(),
                resolved,
            );
        }
        if resolved.space.is_categorical()
            && chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        {
            if factor != 1. || offset != 0. || transform.is_some() {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Discrete secondary axes require an identity transformation.",
                ));
            }
            let range = match &resolved.scale {
                ResolvedScale::Band(s) => s.range(),
                ResolvedScale::Point(s) => s.range(),
                ResolvedScale::Provider(s) => s.range(),
                _ => unreachable!(),
            };
            let viewport = super::secondary_discrete::viewport(&resolved)?;
            let mut axis = ResolvedAxis {
                spec: spec.clone(),
                space: ValueSpace::Data,
                ticks: vec![],
                scale: ResolvedScale::SecondaryDiscrete {
                    source,
                    primary: Box::new(resolved),
                    range,
                    viewport,
                },
            };
            axis.ticks = super::guide_ticks::resolve_with_values(
                chart,
                &axis,
                &spec.guide,
                r,
                major_values,
            )?;
            return Ok(axis);
        }
        let (domain, view, range) = match &resolved.scale {
            ResolvedScale::Linear(s) => (s.domain(), s.viewport(), s.range()),
            ResolvedScale::Numeric(s) => (s.domain(), s.viewport(), s.range()),
            ResolvedScale::Nonlinear(s) => (s.domain(), s.viewport(), s.range()),
            ResolvedScale::Unbounded(s)
                if matches!(s.transform(), Some(ScaleTransform::Ggplot { .. })) =>
            {
                (
                    s.invertible_domain().ok_or_else(|| {
                        error(
                            DiagnosticCode::NumericalDomain,
                            "Secondary unit mapping requires a finite invertible source domain.",
                        )
                    })?,
                    s.invertible_viewport().ok_or_else(|| {
                        error(
                            DiagnosticCode::NumericalDomain,
                            "Secondary unit mapping requires a finite invertible source viewport.",
                        )
                    })?,
                    s.range(),
                )
            }
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary unit mappings require an invertible numeric primary scale.",
                ));
            }
        };
        let conversion = transform
            .as_deref()
            .map(secondary_mapping)
            .transpose()?
            .map(Box::new);
        let convert = |v: f64| -> ChartResult<f64> {
            let v = conversion
                .as_ref()
                .map_or(Ok(Some(v)), |s| s.map_finite(v))?
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::NumericalDomain,
                        "Primary domain is outside the secondary transform's finite domain.",
                    )
                })?;
            let value = v.mul_add(factor, offset);
            if value.is_finite() {
                Ok(value)
            } else {
                Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Secondary unit conversion exceeds finite range.",
                ))
            }
        };
        let ggplot = chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3;
        let convert_bounds = |bounds: Bounds| -> ChartResult<Bounds> {
            let converted = Bounds::new(convert(bounds.start())?, convert(bounds.end())?)?;
            if bounds.start() != bounds.end() && converted.start() == converted.end() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Secondary conversion collapses distinct primary endpoints.",
                ));
            }
            Ok(converted)
        };
        let domain = convert_bounds(domain)?;
        let mut view = convert_bounds(view)?;
        if !ggplot {
            domain.distinct()?;
            view.distinct()?;
        }
        let formatter = guide_formatter(spec, NumericFamily::Linear, view, r.target_ticks as f64)?;
        let ticks = resolved
            .ticks
            .iter()
            .map(|t| {
                let crate::composition::ScaleValue::Number(value) = t.value else {
                    unreachable!()
                };
                Ok(GuideTick {
                    value: crate::composition::ScaleValue::Number(convert(value)?),
                    position: t.position,
                    label: numeric_label(
                        spec,
                        convert(value)?,
                        format_nonlinear_tick(convert(value)?),
                        formatter.as_ref(),
                    )?,
                })
            })
            .collect::<ChartResult<_>>()?;
        let guide_inverse = if ggplot {
            let mut samples = Vec::with_capacity(1000);
            for i in 0..1000 {
                let position = range.start() + (range.end() - range.start()) * (i as f64 / 999.);
                let crate::composition::ScaleValue::Number(value) =
                    resolved.invert_value(position)?
                else {
                    unreachable!()
                };
                samples.push([convert(value)?, value]);
            }
            // The reference samples the expanded primary range. Its transformed
            // extrema can differ from the transformed endpoints (for example x^2
            // when an empty plot's expansion extends below zero).
            let mut samples = crate::scales::ggplot_approx::knots(samples);
            if samples.len() == 1 {
                samples.push(samples[0]);
            }
            view = Bounds::new(samples[0][0], samples[samples.len() - 1][0])?;
            Some(Box::new(NumericScale::new(NumericScaleSpec {
                domain: samples.iter().map(|p| p[0].into()).collect(),
                range: samples.iter().map(|p| p[1].into()).collect(),
                ..NumericScaleSpec::d3(NumericFamily::Linear)
            })?))
        } else {
            None
        };
        let mut axis = ResolvedAxis {
            spec: spec.clone(),
            space: ValueSpace::Data,
            scale: ResolvedScale::Secondary {
                source,
                domain,
                view,
                primary: Box::new(resolved),
                conversion,
                guide_inverse,
                range,
            },
            ticks,
        };
        if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
            || spec.guide.uses_tick_configuration()
        {
            axis.ticks = super::guide_ticks::resolve_with_values(
                chart,
                &axis,
                &spec.guide,
                r,
                major_values,
            )?;
        }
        if ggplot && view.start() == view.end() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Reference secondary interpolation requires two distinct sample values.",
            ));
        }
        return Ok(axis);
    }
    let empty = DomainContributions::default();
    let d = chart.scale_domains().get(&spec.id).unwrap_or(&empty);
    let horizontal = spec.side.horizontal();
    let (extent, space) = if horizontal {
        (d.x, d.x_space.clone())
    } else {
        (d.y, d.y_space.clone())
    };
    let space = space.unwrap_or(ValueSpace::Data);
    let range = spec.range.unwrap_or(Bounds::new(
        if horizontal {
            plot.origin().x()
        } else {
            plot.max_y()
        },
        if horizontal {
            plot.max_x()
        } else {
            plot.origin().y()
        },
    )?);
    let state = chart.state().viewport();
    let inherited = if spec.id == ScaleId::new(0) {
        state.x
    } else if spec.id == ScaleId::new(1) {
        state.y
    } else {
        None
    };
    let viewport = match window {
        Some(AxisWindow::Numeric(a, b))
            if matches!(
                space,
                ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. }
            ) =>
        {
            Some(Bounds::new(*a, *b)?)
        }
        Some(AxisWindow::Timestamp(..)) if matches!(space, ValueSpace::Timestamp { .. }) => None,
        Some(AxisWindow::Category { .. })
            if matches!(
                space,
                ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
            ) =>
        {
            None
        }
        Some(_) => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Navigation window does not match this axis value space.",
            ));
        }
        None => spec
            .viewport
            .or(inherited.map(|(a, b)| Bounds::new(a, b)).transpose()?),
    };
    let time_window = match window {
        Some(AxisWindow::Timestamp(start, end)) => Some(TimeBounds {
            start: *start,
            end: *end,
        }),
        _ => None,
    };
    if matches!(spec.scale, AxisScale::Binned { .. }) {
        let prepared = positional_bins(spec, &space)?;
        if prepared
            .function_limits()
            .is_some_and(|limits| limits.iter().all(|v| v.0.is_nan()))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Positional bin dimensions require a comparable function limit.",
            ));
        }
    }
    if let Some(limits) = chart.positional_limits.get(&spec.id) {
        if limits.len() > 2
            || (limits.iter().any(|v| v.0.is_nan()) && limits.iter().any(|v| v.0.is_finite()))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Positional function limits cannot resolve this endpoint vector.",
            ));
        }
        limits.iter().find(|v| !v.0.is_nan()).ok_or_else(|| {
            error(
                DiagnosticCode::NumericalDomain,
                "Positional function limits have no comparable endpoints.",
            )
        })?;
    }
    if let AxisScale::Binned { spec: bins, .. } = &spec.scale {
        let prepared = positional_bins(spec, &space)?;
        let limits = prepared.panel_limits();
        let view = viewport.map(|view| {
            [view.start(), view.end()].map(|v| {
                crate::interpolate::Number(bins.transform.as_ref().map_or(v, |t| t.forward_raw(v)))
            })
        });
        if limits.iter().any(|v| !v.0.is_finite())
            || view.is_some_and(|v| v.iter().any(|v| !v.0.is_finite()))
        {
            let mut scale = crate::scales::GgplotUnboundedScale::new(
                limits,
                range,
                bins.transform.clone(),
                spec.outside,
            )?;
            if let Some(view) = view {
                scale = scale.with_transformed_viewport(view)?;
            }
            let mut axis = ResolvedAxis {
                spec: spec.clone(),
                space,
                scale: ResolvedScale::Unbounded(scale),
                ticks: vec![],
            };
            axis.ticks = super::guide_ticks::resolve_with_values(
                chart,
                &axis,
                &spec.guide,
                r,
                major_values,
            )?;
            return Ok(axis);
        }
    }
    // Reference transforms retain the panel in transformed units, including
    // intervals whose inverse has an infinite or undefined endpoint.
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && let AxisScale::Nonlinear {
            transform: transform @ ScaleTransform::Ggplot { .. },
            domain,
        } = &spec.scale
    {
        let transformed = |bounds: Bounds| {
            Bounds::new(
                transform.forward_raw(bounds.start()),
                transform.forward_raw(bounds.end()),
            )
        };
        let domain = ContinuousDomain {
            explicit: domain.explicit.map(transformed).transpose()?,
            baseline: match domain.baseline {
                Baseline::None => Baseline::None,
                Baseline::Zero => Baseline::Value(transform.forward_raw(0.)),
                Baseline::Value(v) => Baseline::Value(transform.forward_raw(v)),
            },
            ..*domain
        };
        let extent = if matches!(space, ValueSpace::Scaled { .. }) {
            extent
        } else {
            extent.map(|e| {
                let a = transform.forward_raw(e.minimum);
                let b = transform.forward_raw(e.maximum);
                crate::grammar::Extent {
                    minimum: a.min(b),
                    maximum: a.max(b),
                }
            })
        };
        let limits = if let Some(limits) = chart.positional_limits.get(&spec.id) {
            [
                limits.iter().map(|v| v.0).fold(f64::INFINITY, f64::min),
                limits.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max),
            ]
        } else {
            let b = if domain.explicit.is_none() && (domain.nice || domain.padding != 0.) {
                domain.resolve(extent)?
            } else {
                domain.trained_limits(extent)?
            };
            [b.start(), b.end()]
        };
        let view = if let Some(view) = viewport {
            let b = transformed(view)?;
            [b.start(), b.end()]
        } else if limits.iter().all(|v| v.is_finite()) {
            let b = spec
                .expansion
                .unwrap_or_default()
                .continuous_viewport(Bounds::new(limits[0], limits[1])?)?;
            [b.start(), b.end()]
        } else {
            limits
        };
        let scale = GgplotUnboundedScale::new(
            limits.map(crate::interpolate::Number),
            range,
            Some(transform.clone()),
            spec.outside,
        )?
        .with_transformed_viewport(view.map(crate::interpolate::Number))?;
        let mut axis = ResolvedAxis {
            spec: spec.clone(),
            space,
            scale: ResolvedScale::Unbounded(scale),
            ticks: vec![],
        };
        axis.ticks =
            super::guide_ticks::resolve_with_values(chart, &axis, &spec.guide, r, major_values)?;
        return Ok(axis);
    }
    let mut function_spec;
    let spec = if let Some(limits) = chart.positional_limits.get(&spec.id)
        && !matches!(spec.scale, AxisScale::Binned { .. })
    {
        let first = limits
            .iter()
            .find(|v| !v.0.is_nan())
            .expect("validated endpoints")
            .0;
        let (low, high) = limits
            .iter()
            .filter(|v| !v.0.is_nan())
            .fold((first, first), |(lo, hi), v| (lo.min(v.0), hi.max(v.0)));
        let transform = match &spec.scale {
            AxisScale::Nonlinear { transform, .. } => Some(transform),
            _ => None,
        };
        if !low.is_finite()
            || !high.is_finite()
            || viewport.is_some_and(|view| {
                [view.start(), view.end()].into_iter().any(|v| {
                    !transform
                        .as_ref()
                        .map_or(v, |t| t.forward_raw(v))
                        .is_finite()
                })
            })
        {
            let mut scale = crate::scales::GgplotUnboundedScale::new(
                [
                    crate::interpolate::Number(low),
                    crate::interpolate::Number(high),
                ],
                range,
                transform.cloned(),
                spec.outside,
            )?;
            if let Some(view) = viewport {
                scale = scale.with_transformed_viewport([view.start(), view.end()].map(|v| {
                    crate::interpolate::Number(transform.as_ref().map_or(v, |t| t.forward_raw(v)))
                }))?;
            }
            let mut axis = ResolvedAxis {
                spec: spec.clone(),
                space,
                scale: ResolvedScale::Unbounded(scale),
                ticks: vec![],
            };
            axis.ticks = super::guide_ticks::resolve_with_values(
                chart,
                &axis,
                &spec.guide,
                r,
                major_values,
            )?;
            return Ok(axis);
        }
        if let ValueSpace::Timestamp {
            origin,
            representation,
        } = &space
        {
            let time = crate::scales::GgplotTimestampNormalization {
                origin: *origin,
                unit: representation.unit,
                date: matches!(spec.scale, AxisScale::Date { .. }),
            };
            let mut resolved_spec = spec.clone();
            resolved_spec.resolved_temporal = Some(Box::new(time));
            let mut axis = ResolvedAxis {
                spec: resolved_spec,
                space,
                scale: ResolvedScale::Calendar(Box::new(TimeAxisScale::resolve_function(
                    time,
                    match &spec.scale {
                        AxisScale::Calendar { spec, .. } => spec.zone.clone(),
                        _ => CalendarZone::Utc,
                    },
                    Bounds::new(low, high)?,
                    range,
                    time_window.or(viewport
                        .map(|view| {
                            Ok(TimeBounds {
                                start: project::timestamp(view.start(), time.origin)?,
                                end: project::timestamp(view.end(), time.origin)?,
                            })
                        })
                        .transpose()?),
                    spec.outside,
                    spec.expansion.unwrap_or_default(),
                )?)),
                ticks: vec![],
            };
            axis.ticks = super::guide_ticks::resolve_with_values(
                chart,
                &axis,
                &spec.guide,
                r,
                major_values,
            )?;
            return Ok(axis);
        }
        function_spec = spec.clone();
        let raw = [low, high].map(|v| transform.as_ref().map_or(Ok(v), |t| t.inverse(v)));
        let domain = ContinuousDomain::explicit(Bounds::new(raw[0].clone()?, raw[1].clone()?)?);
        function_spec.scale = if let Some(transform) = transform {
            AxisScale::Nonlinear {
                transform: transform.clone(),
                domain,
            }
        } else if matches!(spec.scale, AxisScale::Duration(_)) {
            AxisScale::Duration(domain)
        } else {
            AxisScale::Linear(domain)
        };
        &function_spec
    } else {
        spec
    };
    let family = match (&spec.scale, &space) {
        (
            AxisScale::Auto,
            ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. },
        ) => AxisScale::Band(BandOptions::default()),
        (AxisScale::Auto, ValueSpace::Timestamp { .. }) => AxisScale::Utc {
            domain: None,
            interval: None,
        },
        (AxisScale::Auto, _) => AxisScale::Linear(ContinuousDomain::default()),
        (AxisScale::Binned { spec: bins, .. }, _) => {
            let prepared = positional_bins(spec, &space)?;
            let raw = prepared
                .panel_limits()
                .map(|v| bins.transform.as_ref().map_or(Ok(v.0), |t| t.inverse(v.0)))
                .into_iter()
                .collect::<ChartResult<Vec<_>>>()?;
            let domain = ContinuousDomain::explicit(Bounds::new(raw[0], raw[1])?);
            if let Some(transform) = &bins.transform {
                AxisScale::Nonlinear {
                    transform: transform.clone(),
                    domain,
                }
            } else {
                AxisScale::Linear(domain)
            }
        }
        (s, _) => s.clone(),
    };
    if let Some(limits) = &spec.continuous_limits {
        crate::limits::require_within(
            limits.len() <= r.limits.max_items,
            "continuous category limits",
        )?;
        if !matches!(family, AxisScale::Band(_) | AxisScale::Point(_)) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Continuous category limits require a reference band or point axis.",
            ));
        }
        if window.is_some() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Continuous category limits cannot be combined with a categorical navigation window.",
            ));
        }
    }
    if spec.discrete.is_some()
        && (!space.is_categorical() || !matches!(family, AxisScale::Band(_) | AxisScale::Point(_)))
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Reference discrete policies require a band or point category axis.",
        ));
    }
    let is_date = matches!(family, AxisScale::Date { .. });
    let family = match family {
        AxisScale::Date { domain } => AxisScale::Utc {
            domain,
            interval: None,
        },
        family => family,
    };
    // Character training in the pinned reference uses C collation. Keep source
    // ordinals intact; only the resolved automatic domain changes order.
    let mut family = family;
    if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && let ValueSpace::Categorical { categories } = &space
    {
        let domain = match &mut family {
            AxisScale::Band(options) => Some(&mut options.domain),
            AxisScale::Point(options) => Some(&mut options.domain),
            _ => None,
        };
        if let Some(domain) = domain
            && domain.is_none()
            && spec.discrete.is_none()
        {
            let mut labels = categories.clone();
            labels.sort_unstable();
            *domain = Some(labels);
        }
    }
    let expansion = spec.expansion.or_else(|| {
        (is_date
            || spec.continuous_limits.is_some()
            || chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3)
            .then(|| {
                if matches!(family, AxisScale::Band(_) | AxisScale::Point(_)) {
                    GgplotExpansion::discrete_default()
                } else {
                    GgplotExpansion::default()
                }
            })
    });
    if spec.expansion.is_some()
        && !matches!(
            family,
            AxisScale::Linear(_)
                | AxisScale::Duration(_)
                | AxisScale::Nonlinear { .. }
                | AxisScale::Band(_)
                | AxisScale::Point(_)
                | AxisScale::Utc { .. }
                | AxisScale::Calendar { .. }
        )
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Reference expansion requires a linear, nonlinear, band, point or time axis.",
        ));
    }
    if (spec.number_format.is_some() || spec.numeric_format.is_some())
        && !matches!(
            space,
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. }
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric formatting requires a numeric guide; time and categories retain their own labels.",
        ));
    }
    if spec.time_format.is_some()
        && !matches!(
            family,
            AxisScale::Utc { .. } | AxisScale::Calendar { .. } | AxisScale::Duration(_)
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Time formatting requires a UTC or calendar time axis.",
        ));
    }
    if spec.population_missing.is_some()
        && spec.limits_function.is_none()
        && let AxisScale::Nonlinear {
            transform: ScaleTransform::Sqrt,
            domain,
        } = &family
        && domain.explicit.is_none()
        && matches!(space, ValueSpace::Scaled { .. })
        && let Some(extent) = extent.filter(|e| e.minimum < 0.)
    {
        let limits = Bounds::new(extent.minimum, extent.maximum)?;
        let view = match viewport {
            Some(view) => Bounds::new(
                ScaleTransform::Sqrt.forward_raw(view.start()),
                ScaleTransform::Sqrt.forward_raw(view.end()),
            )?,
            None => expansion.unwrap_or_default().continuous_viewport(limits)?,
        };
        let mut axis = ResolvedAxis {
            spec: spec.clone(),
            space,
            scale: ResolvedScale::Nonlinear(NonlinearScale::from_reference_transformed(
                limits,
                view,
                ScaleTransform::Sqrt,
                range,
                spec.outside,
            )?),
            ticks: vec![],
        };
        axis.ticks =
            super::guide_ticks::resolve_with_values(chart, &axis, &spec.guide, r, major_values)?;
        axis.ticks
            .retain(|tick| tick.position >= range.minimum() && tick.position <= range.maximum());
        return Ok(axis);
    }
    let scale = match (family, &space) {
        (
            AxisScale::Registered {
                operation,
                parameters,
            },
            _,
        ) => {
            let window = match window {
                Some(window) => Some(window.clone()),
                None => viewport
                    .map(|view| -> ChartResult<AxisWindow> {
                        match &space {
                            ValueSpace::Timestamp { origin, .. } => Ok(AxisWindow::Timestamp(
                                project::timestamp(view.start(), *origin)?,
                                project::timestamp(view.end(), *origin)?,
                            )),
                            _ => Ok(AxisWindow::Numeric(view.start(), view.end())),
                        }
                    })
                    .transpose()?,
            };
            let provider = chart.scale_registrations.resolve(
                &operation,
                crate::grammar::ScaleProviderInput {
                    parameters: &parameters,
                    extent: extent
                        .map(|e| Bounds::new(e.minimum, e.maximum))
                        .transpose()?,
                    space: &space,
                    range,
                    window,
                    outside: spec.outside,
                    limits: r.limits,
                    max_values: r.max_categories,
                    max_ticks: r.max_ticks,
                },
                r.units == crate::services::Units::Points,
            )?;
            ResolvedScale::Provider(CheckedPositionalScale::new(
                provider,
                space.clone(),
                r.limits,
                r.max_categories,
            )?)
        }
        (family @ (AxisScale::Band(_) | AxisScale::Point(_)), category_space)
            if matches!(category_space, ValueSpace::NullableCategorical { .. })
                || spec.discrete.is_some() && category_space.is_categorical() =>
        {
            if viewport.is_some() || window.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Nullable positional categories currently require the automatic reference viewport.",
                ));
            }
            let (limits, inner, outer, point) = match &family {
                AxisScale::Band(options) => (
                    options.domain.as_ref(),
                    options.inner_padding,
                    options.outer_padding,
                    false,
                ),
                AxisScale::Point(options) => (options.domain.as_ref(), 1., options.padding, true),
                _ => unreachable!(),
            };
            let keys = (0..category_space.category_count().unwrap())
                .map(|i| match category_space.category_value(i as f64).unwrap() {
                    crate::composition::ScaleValue::Category(value) => {
                        crate::scales::ScaleKey::Text(value)
                    }
                    crate::composition::ScaleValue::MissingCategory => {
                        crate::scales::ScaleKey::Null
                    }
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            let mut policy = spec.discrete.as_deref().cloned().unwrap_or_default();
            if let Some(limits) = limits {
                if spec.discrete.is_some() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Reference discrete policies own their category limits.",
                    ));
                }
                policy.limits = Some(
                    limits
                        .iter()
                        .cloned()
                        .map(crate::scales::ScaleKey::Text)
                        .collect(),
                );
            }
            if let Some(expansion) = spec.expansion {
                policy.expansion = expansion;
            }
            if matches!(policy.guide.labels, GgplotGuideLabels::Registered { .. }) {
                policy.guide.labels = GgplotGuideLabels::Hidden;
            }
            let provider = policy
                .train_using(
                    &keys,
                    spec.continuous_limits.as_deref(),
                    &chart.palette_registrations,
                )?
                .into_provider(range, inner, outer, point)?;
            ResolvedScale::Provider(CheckedPositionalScale::new(
                std::sync::Arc::new(provider),
                ValueSpace::NullableCategorical { categories: vec![] },
                r.limits,
                r.max_categories,
            )?)
        }
        (
            family @ (AxisScale::D3Band(_) | AxisScale::D3Point(_)),
            ValueSpace::NullableCategorical { categories },
        ) => {
            if viewport.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "D3 category axes require category windows and extend behavior.",
                ));
            }
            let labels = categories.iter().flatten().cloned().collect::<Vec<_>>();
            match family {
                AxisScale::D3Band(options) => {
                    let mut scale = BandScale::resolve_d3(&labels, &options, range)?;
                    if let Some(AxisWindow::Category { first, last }) = window {
                        scale = scale.with_window(first, last)?;
                    }
                    ResolvedScale::Band(scale)
                }
                AxisScale::D3Point(options) => {
                    let mut scale = PointScale::resolve_d3(&labels, &options, range)?;
                    if let Some(AxisWindow::Category { first, last }) = window {
                        scale = scale.with_window(first, last)?;
                    }
                    ResolvedScale::Point(scale)
                }
                _ => unreachable!(),
            }
        }
        (AxisScale::Numeric(options), ValueSpace::Data | ValueSpace::Transformed { .. }) => {
            let scale = NumericAxisScale::resolve(options, range, viewport, spec.outside)?;

            ResolvedScale::Numeric(scale)
        }
        (
            AxisScale::Linear(options) | AxisScale::Duration(options),
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. },
        ) => {
            let reference_viewport = match viewport {
                Some(_) => None,
                None => expansion
                    .map(|e| {
                        let limits = if options.explicit.is_none()
                            && (options.nice || options.padding != 0.)
                        {
                            options.resolve(extent)?
                        } else {
                            options.trained_limits(extent)?
                        };
                        e.continuous_viewport(Bounds::new(limits.minimum(), limits.maximum())?)
                    })
                    .transpose()?,
            };
            let mut scale = LinearScale::resolve(extent, options, range, viewport, spec.outside)?;
            if let Some(view) = reference_viewport {
                scale = scale.with_reference_viewport(view);
            }

            ResolvedScale::Linear(scale)
        }
        (
            AxisScale::Nonlinear { transform, domain },
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. },
        ) => {
            let contribution = if let ValueSpace::Scaled { scale, .. } = &space {
                if scale.transform != Some(transform.clone()) || scale.id != spec.id {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Prepared scale stage differs from the bound axis transformation.",
                    ));
                }
                extent
                    .filter(|_| domain.explicit.is_none())
                    .map(|e| -> ChartResult<crate::grammar::Extent> {
                        let a = transform.inverse(e.minimum)?;
                        let b = transform.inverse(e.maximum)?;
                        Ok(crate::grammar::Extent {
                            minimum: a.min(b),
                            maximum: a.max(b),
                        })
                    })
                    .transpose()?
            } else if matches!(transform, ScaleTransform::Log { .. } | ScaleTransform::Sqrt) {
                eligible_transform_extent(chart, spec, transform.clone())?
            } else {
                extent
            };
            let reference_viewport = match viewport {
                Some(_) => None,
                None => expansion
                    .map(|e| {
                        e.nonlinear_transformed_viewport(contribution, domain, transform.clone())
                    })
                    .transpose()?,
            };
            let mut scale = NonlinearScale::resolve(
                contribution,
                domain,
                transform,
                range,
                viewport,
                spec.outside,
            )?;

            if let Some(view) = reference_viewport {
                scale = scale.with_reference_transformed_viewport(view)?;
            }
            ResolvedScale::Nonlinear(scale)
        }
        (
            family @ (AxisScale::Point(_) | AxisScale::D3Point(_)),
            ValueSpace::Categorical { categories },
        ) => {
            if viewport.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Point scales use category domains without numeric viewport policies.",
                ));
            }
            let mut scale = match &family {
                AxisScale::Point(options) => PointScale::resolve(categories, options, range)?,
                AxisScale::D3Point(options) => PointScale::resolve_d3(categories, options, range)?,
                _ => unreachable!(),
            };
            if let Some(AxisWindow::Category { first, last }) = window {
                scale = scale.with_window(first, last)?;
            } else if let Some(expansion) =
                expansion.filter(|_| matches!(family, AxisScale::Point(_)))
            {
                scale = scale.with_reference_expansion(
                    expansion,
                    spec.continuous_limits.as_deref(),
                    categories,
                )?;
            }

            ResolvedScale::Point(scale)
        }
        (
            AxisScale::Session(calendar),
            ValueSpace::Timestamp {
                representation,
                origin,
            },
        ) => {
            if calendar.unit != representation.unit {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Session calendar and source timestamp units must agree.",
                ));
            }
            let view = viewport
                .map(|b| {
                    Ok(TimeBounds {
                        start: project::timestamp(b.start(), *origin)?,
                        end: project::timestamp(b.end(), *origin)?,
                    })
                })
                .transpose()?;
            let scale = SessionScale::new(calendar, range, time_window.or(view), spec.outside)?;

            ResolvedScale::Session(scale)
        }
        (
            family @ (AxisScale::Band(_) | AxisScale::D3Band(_)),
            ValueSpace::Categorical { categories },
        ) => {
            if viewport.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Band scales use explicit category domains; numeric viewport/clamp/omit policies are unsupported.",
                ));
            }
            let mut scale = match &family {
                AxisScale::Band(options) => BandScale::resolve(categories, options, range)?,
                AxisScale::D3Band(options) => BandScale::resolve_d3(categories, options, range)?,
                _ => unreachable!(),
            };
            if let Some(AxisWindow::Category { first, last }) = window {
                scale = scale.with_window(first, last)?;
            } else if let Some(expansion) =
                expansion.filter(|_| matches!(family, AxisScale::Band(_)))
            {
                scale = scale.with_reference_expansion(
                    expansion,
                    spec.continuous_limits.as_deref(),
                    categories,
                )?;
            }

            ResolvedScale::Band(scale)
        }
        (
            AxisScale::Calendar {
                spec: options,
                interval: _,
            },
            ValueSpace::Timestamp {
                representation,
                origin,
            },
        ) => {
            if representation.unit != options.unit {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Calendar axis and timestamp source units must agree.",
                ));
            }
            let viewport = viewport
                .map(|b| {
                    Ok(TimeBounds {
                        start: project::timestamp(b.start(), *origin)?,
                        end: project::timestamp(b.end(), *origin)?,
                    })
                })
                .transpose()?;
            let mut options = options;
            if options.domain.is_empty() {
                options.domain = match extent {
                    Some(e) => vec![
                        project::timestamp(e.minimum, *origin)?,
                        project::timestamp(e.maximum, *origin)?,
                    ],
                    None => vec![
                        0,
                        if expansion.is_some() {
                            ticks_per_second(representation.unit) as i64
                        } else {
                            1
                        },
                    ],
                };
            }
            let scale = TimeAxisScale::resolve_reference(
                options,
                range,
                time_window.or(viewport),
                spec.outside,
                expansion,
            )?;

            ResolvedScale::Calendar(Box::new(scale))
        }
        (
            AxisScale::Utc { domain, interval },
            ValueSpace::Timestamp {
                representation,
                origin,
            },
        ) => {
            if interval.is_some_and(|i| match i {
                UtcInterval::Ticks(n) => n == 0,
                UtcInterval::Seconds(n)
                | UtcInterval::Days(n)
                | UtcInterval::Weeks(n)
                | UtcInterval::Months(n)
                | UtcInterval::Years(n) => n == 0,
            }) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "UTC tick interval must be positive.",
                ));
            }
            let domain = match domain {
                Some(d) => d,
                None => match extent {
                    Some(e) => TimeBounds {
                        start: project::timestamp(e.minimum, *origin)?,
                        end: project::timestamp(e.maximum, *origin)?,
                    },
                    None => TimeBounds {
                        start: 0,
                        end: if is_date {
                            (ticks_per_second(representation.unit) * 86400) as i64
                        } else if expansion.is_some() {
                            ticks_per_second(representation.unit) as i64
                        } else {
                            1
                        },
                    },
                },
            };
            let viewport = viewport
                .map(|b| {
                    Ok(TimeBounds {
                        start: project::timestamp(b.start(), *origin)?,
                        end: project::timestamp(b.end(), *origin)?,
                    })
                })
                .transpose()?;
            if expansion.is_some() {
                ResolvedScale::Calendar(Box::new(TimeAxisScale::resolve_reference_unit(
                    TimeScaleSpec {
                        domain: vec![domain.start, domain.end],
                        unit: representation.unit,
                        ..Default::default()
                    },
                    range,
                    time_window.or(viewport),
                    spec.outside,
                    expansion,
                    if is_date { 86400. } else { 1. },
                )?))
            } else {
                ResolvedScale::Utc(UtcScale::new(
                    domain,
                    time_window.or(viewport),
                    representation.unit,
                    range,
                    spec.outside,
                )?)
            }
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Axis family does not match the prepared numeric/category/timestamp space.",
            ));
        }
    };
    let mut axis = ResolvedAxis {
        spec: spec.clone(),
        space,
        scale,
        ticks: vec![],
    };
    axis.ticks =
        super::guide_ticks::resolve_with_values(chart, &axis, &spec.guide, r, major_values)?;
    Ok(axis)
}

pub(super) fn resolve_axis(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    plot: Rect,
    major_values: &mut super::guide_ticks::SelectedGuideValues,
) -> ChartResult<ResolvedAxis> {
    resolve_axis_inner(chart, r, spec, plot, major_values).map_err(|mut e: crate::Diagnostic| {
        e.message = format!("Scale {}: {}", spec.id.get(), e.message);
        e
    })
}

// Domain training scans eligible post-position geometry, never the visible viewport.
fn eligible_transform_extent(
    chart: &PreparedChart,
    axis: &AxisSpec,
    transform: ScaleTransform,
) -> ChartResult<Option<crate::grammar::Extent>> {
    use crate::grammar::{Extent, PreparedGeometry};
    let mut extent: Option<Extent> = None;
    let chart = chart
        .shared_training
        .as_deref()
        .filter(|shared| shared.scale_domains().contains_key(&axis.id))
        .unwrap_or(chart);
    for layer in chart.layers() {
        let binding = if axis.side.horizontal() {
            layer.scales().x
        } else {
            layer.scales().y
        };
        if binding != axis.id {
            continue;
        }
        let authored = chart
            .definition()
            .layers
            .iter()
            .find(|l| l.id == layer.id())
            .expect("prepared layer exists");
        let coordinate = |point: crate::Point| {
            if axis.side.horizontal() {
                point.x()
            } else {
                point.y()
            }
        };
        let mut include = |value: f64, baseline_dependent: bool| -> ChartResult<()> {
            if transform.forward(value)?.is_none() {
                if baseline_dependent || authored.invalid == crate::data::InvalidPolicy::Strict {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Geometry coordinates and declared interval baselines must lie inside the scale transform domain.",
                    ));
                }
                return Ok(());
            }
            extent = Some(match extent {
                None => Extent {
                    minimum: value,
                    maximum: value,
                },
                Some(e) => Extent {
                    minimum: e.minimum.min(value),
                    maximum: e.maximum.max(value),
                },
            });
            Ok(())
        };
        for mark in layer.marks() {
            match &mark.geometry {
                PreparedGeometry::UnboundedPoint(p) => {
                    let value = p[usize::from(!axis.side.horizontal())].0;
                    if value.is_finite() {
                        include(value, false)?;
                    }
                }
                PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {}
                PreparedGeometry::Point(p)
                | PreparedGeometry::ShapePath { center: p, .. }
                | PreparedGeometry::ShapePathRun { center: p, .. } => {
                    include(coordinate(*p), false)?
                }
                PreparedGeometry::Polygon(points) => {
                    for p in points {
                        include(coordinate(*p), true)?;
                    }
                }
                PreparedGeometry::BandRun { lower, upper }
                | PreparedGeometry::StackBandRun { lower, upper, .. } => {
                    for p in lower.iter().chain(upper) {
                        include(coordinate(*p), true)?;
                    }
                }
                PreparedGeometry::LineRun(points) => {
                    for p in points {
                        include(coordinate(*p), false)?;
                    }
                }
                PreparedGeometry::Rule { from, to }
                | PreparedGeometry::Rectangle { from, to }
                | PreparedGeometry::Bar { from, to, .. }
                | PreparedGeometry::NativePaint { from, to, .. } => {
                    include(coordinate(*from), true)?;
                    include(coordinate(*to), true)?;
                }
            }
        }
    }
    Ok(extent)
}

pub(super) fn guide_formatter(
    spec: &GuideStyle,
    family: NumericFamily,
    view: Bounds,
    count: f64,
) -> ChartResult<Option<crate::typography::NumericFormatter>> {
    spec.numeric_format
        .as_ref()
        .map(|f| {
            family.tick_format(
                &[
                    crate::interpolate::Number(view.start()),
                    crate::interpolate::Number(view.end()),
                ],
                count,
                Some(&f.specifier),
                f.locale.clone(),
            )
        })
        .transpose()
}
pub(super) fn numeric_label(
    spec: &GuideStyle,
    value: f64,
    default: String,
    formatter: Option<&crate::typography::NumericFormatter>,
) -> ChartResult<String> {
    if let Some(f) = formatter {
        return Ok(f.format(value));
    }
    if let Some(f) = &spec.number_format {
        f.format(value)
    } else {
        Ok(default)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Existing secondary specification plus its resolved primary."
)]
fn resolve_secondary_time(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    source: crate::ScaleId,
    factor: f64,
    offset: f64,
    nonlinear: bool,
    mut time: ResolvedAxis,
) -> ChartResult<ResolvedAxis> {
    if factor != 1. || nonlinear {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Date/datetime secondary guides support additive time offsets only.",
        ));
    }
    let unit = match &time.scale {
        ResolvedScale::Utc(s) => s.unit(),
        ResolvedScale::Calendar(s) => s.unit(),
        _ => unreachable!(),
    };
    let units = match unit {
        crate::data::TimeUnit::Seconds => 1.,
        crate::data::TimeUnit::Milliseconds => 1_000.,
        crate::data::TimeUnit::Microseconds => 1_000_000.,
        crate::data::TimeUnit::Nanoseconds => 1_000_000_000.,
    };
    let delta = offset
        * units
        * if matches!(time.spec.scale, AxisScale::Date { .. }) {
            86400.
        } else {
            1.
        };
    if !delta.is_finite()
        || delta.fract() != 0.
        || !(-9_223_372_036_854_775_808. ..9_223_372_036_854_775_808.).contains(&delta)
    {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Secondary time offset must be representable in exact source timestamp units.",
        ));
    }
    let delta = delta as i64;
    time.scale = match &time.scale {
        ResolvedScale::Utc(s) => ResolvedScale::Utc(s.shifted(delta)?),
        ResolvedScale::Calendar(s) => ResolvedScale::Calendar(Box::new(s.shifted(delta)?)),
        _ => unreachable!(),
    };
    if let ValueSpace::Timestamp { origin, .. } = &mut time.space {
        *origin = origin.checked_add(delta).ok_or_else(|| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Secondary time origin exceeds the timestamp range.",
            )
        })?;
    }
    // Retain the primary Date/calendar family for shared guide selection while
    // replacing all presentation with the independently authored secondary guide.
    let family = match (&time.spec.scale, &time.scale) {
        (AxisScale::Date { .. }, ResolvedScale::Calendar(s)) => AxisScale::Date {
            domain: Some(s.domain()),
        },
        (_, ResolvedScale::Calendar(s)) => AxisScale::Calendar {
            spec: s.spec().clone(),
            interval: None,
        },
        (_, ResolvedScale::Utc(s)) => AxisScale::Utc {
            domain: Some(s.domain()),
            interval: None,
        },
        _ => unreachable!(),
    };
    time.spec = spec.clone();
    time.spec.scale = family;
    time.ticks.clear();
    time.ticks = super::guide_ticks::resolve(chart, &time, &spec.guide, r)?;
    Ok(ResolvedAxis {
        spec: spec.clone(),
        space: time.space.clone(),
        ticks: time.ticks.clone(),
        scale: ResolvedScale::SecondaryTime {
            source,
            axis: Box::new(time),
        },
    })
}

pub(super) fn positional_bins(
    spec: &AxisSpec,
    space: &ValueSpace,
) -> ChartResult<std::sync::Arc<PreparedGgplotBinnedPosition>> {
    if let ValueSpace::Scaled { scale, .. } = space
        && scale.id == spec.id
        && let Some(binned) = &scale.binned
    {
        return Ok(binned.clone());
    }
    let AxisScale::Binned {
        spec: bins,
        prepared,
    } = &spec.scale
    else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "This axis has no positional bin policy.",
        ));
    };
    prepared
        .clone()
        .map_or_else(|| bins.train(&[]).map(std::sync::Arc::new), Ok)
}
