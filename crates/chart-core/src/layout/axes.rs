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
        } = a.scale
        {
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
                    | AxisScale::Linear(_)
                    | AxisScale::Numeric(_)
                    | AxisScale::Nonlinear { .. }
            ) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary axes require a primary numeric scale directly.",
                ));
            }
        }
        let range = Bounds::new(0., 1.)?;
        match &a.scale {
            AxisScale::Calendar { spec, interval } => {
                TimeAxisScale::resolve(spec.clone(), range, None, a.outside)?;
                if let Some(interval) = interval {
                    interval.validate()?;
                }
            }
            AxisScale::Numeric(spec) => {
                NumericScale::new(spec.clone())?;
            }
            AxisScale::Linear(domain) => {
                domain.resolve(None)?;
            }
            AxisScale::Nonlinear { transform, domain } => {
                NonlinearScale::resolve(None, *domain, *transform, range, a.viewport, a.outside)?;
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
        if ticks.iter().any(|tick| tick.label.is_empty()) {
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
        let resolved = resolve_axis_inner(chart, r, &primary, plot)?;
        let (domain, view) = match &resolved.scale {
            ResolvedScale::Linear(s) => (s.domain(), s.viewport()),
            ResolvedScale::Numeric(s) => (s.domain(), s.viewport()),
            ResolvedScale::Nonlinear(s) => (s.domain(), s.viewport()),
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Secondary unit mappings require an invertible numeric primary scale.",
                ));
            }
        };
        let convert = |v: f64| -> ChartResult<f64> {
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
        let domain = Bounds::new(convert(domain.start())?, convert(domain.end())?)?.distinct()?;
        let view = Bounds::new(convert(view.start())?, convert(view.end())?)?.distinct()?;
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
        return Ok(ResolvedAxis {
            spec: spec.clone(),
            space: ValueSpace::Data,
            scale: ResolvedScale::Secondary { source, domain },
            ticks,
        });
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
        Some(AxisWindow::Category { .. }) if matches!(space, ValueSpace::Categorical { .. }) => {
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
    let family = match (&spec.scale, &space) {
        (AxisScale::Auto, ValueSpace::Categorical { .. }) => {
            AxisScale::Band(BandOptions::default())
        }
        (AxisScale::Auto, ValueSpace::Timestamp { .. }) => AxisScale::Utc {
            domain: None,
            interval: None,
        },
        (AxisScale::Auto, _) => AxisScale::Linear(ContinuousDomain::default()),
        (s, _) => s.clone(),
    };
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
        && !matches!(family, AxisScale::Utc { .. } | AxisScale::Calendar { .. })
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Time formatting requires a UTC or calendar time axis.",
        ));
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
        (AxisScale::Numeric(options), ValueSpace::Data | ValueSpace::Transformed { .. }) => {
            let scale = NumericAxisScale::resolve(options, range, viewport, spec.outside)?;

            ResolvedScale::Numeric(scale)
        }
        (
            AxisScale::Linear(options),
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. },
        ) => {
            let scale = LinearScale::resolve(extent, options, range, viewport, spec.outside)?;

            ResolvedScale::Linear(scale)
        }
        (
            AxisScale::Nonlinear { transform, domain },
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. },
        ) => {
            let contribution = if let ValueSpace::Scaled { scale, .. } = &space {
                if scale.transform != Some(transform) || scale.id != spec.id {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Prepared scale stage differs from the bound axis transformation.",
                    ));
                }
                extent
                    .map(|e| -> ChartResult<crate::grammar::Extent> {
                        Ok(crate::grammar::Extent {
                            minimum: transform.inverse(e.minimum)?,
                            maximum: transform.inverse(e.maximum)?,
                        })
                    })
                    .transpose()?
            } else if matches!(transform, ScaleTransform::Log { .. }) {
                positive_extent(chart, spec)?
            } else {
                extent
            };
            let scale = NonlinearScale::resolve(
                contribution,
                domain,
                transform,
                range,
                viewport,
                spec.outside,
            )?;

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
            let mut scale = match family {
                AxisScale::Point(options) => PointScale::resolve(categories, &options, range)?,
                AxisScale::D3Point(options) => PointScale::resolve_d3(categories, &options, range)?,
                _ => unreachable!(),
            };
            if let Some(AxisWindow::Category { first, last }) = window {
                scale = scale.with_window(first, last)?;
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
            let mut scale = match family {
                AxisScale::Band(options) => BandScale::resolve(categories, &options, range)?,
                AxisScale::D3Band(options) => BandScale::resolve_d3(categories, &options, range)?,
                _ => unreachable!(),
            };
            if let Some(AxisWindow::Category { first, last }) = window {
                scale = scale.with_window(first, last)?;
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
            let scale =
                TimeAxisScale::resolve(options, range, time_window.or(viewport), spec.outside)?;

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
                    None => TimeBounds { start: 0, end: 1 },
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
            let scale = UtcScale::new(
                domain,
                time_window.or(viewport),
                representation.unit,
                range,
                spec.outside,
            )?;

            ResolvedScale::Utc(scale)
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
    axis.ticks = super::guide_ticks::resolve(&axis, &spec.guide, r)?;
    Ok(axis)
}

pub(super) fn resolve_axis(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    plot: Rect,
) -> ChartResult<ResolvedAxis> {
    resolve_axis_inner(chart, r, spec, plot).map_err(|mut e: crate::Diagnostic| {
        e.message = format!("Scale {}: {}", spec.id.get(), e.message);
        e
    })
}

// Domain training scans eligible post-position geometry, never the visible viewport.
fn positive_extent(
    chart: &PreparedChart,
    axis: &AxisSpec,
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
        let mut include = |point: crate::Point, baseline_dependent: bool| -> ChartResult<()> {
            let value = if axis.side.horizontal() {
                point.x()
            } else {
                point.y()
            };
            if value <= 0. {
                if baseline_dependent || authored.invalid == crate::data::InvalidPolicy::Strict {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Log geometry requires positive coordinates and a valid declared interval baseline.",
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
                PreparedGeometry::Point(p)
                | PreparedGeometry::ShapePath { center: p, .. }
                | PreparedGeometry::ShapePathRun { center: p, .. } => include(*p, false)?,
                PreparedGeometry::Polygon(points) => {
                    for p in points {
                        include(*p, true)?;
                    }
                }
                PreparedGeometry::BandRun { lower, upper }
                | PreparedGeometry::StackBandRun { lower, upper, .. } => {
                    for p in lower.iter().chain(upper) {
                        include(*p, true)?;
                    }
                }
                PreparedGeometry::LineRun(points) => {
                    for p in points {
                        include(*p, false)?;
                    }
                }
                PreparedGeometry::Rule { from, to }
                | PreparedGeometry::Rectangle { from, to }
                | PreparedGeometry::Bar { from, to, .. }
                | PreparedGeometry::NativePaint { from, to, .. } => {
                    include(*from, true)?;
                    include(*to, true)?;
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
