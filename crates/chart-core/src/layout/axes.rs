use super::{project, *};
use crate::grammar::{DomainContributions, PreparedChart, ValueSpace};
use crate::scales::*;
use crate::{ChartResult, DiagnosticCode, Rect, ScaleId};

fn resolve_axis_inner(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    plot: Rect,
) -> ChartResult<ResolvedAxis> {
    if let Some(f) = &spec.number_format {
        f.validate()?;
    }
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
        Bounds::new(convert(view.start())?, convert(view.end())?)?.distinct()?;
        let ticks = resolved
            .ticks
            .iter()
            .map(|t| {
                let value = match &resolved.scale {
                    ResolvedScale::Linear(s) => s.invert(t.position)?,
                    ResolvedScale::Nonlinear(s) => s.invert(t.position)?,
                    _ => unreachable!(),
                };
                Ok(GuideTick {
                    position: t.position,
                    label: numeric_label(
                        spec,
                        convert(value)?,
                        format_nonlinear_tick(convert(value)?),
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
    let viewport = spec
        .viewport
        .or(inherited.map(|(a, b)| Bounds::new(a, b)).transpose()?);
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
    if spec.number_format.is_some()
        && !matches!(space, ValueSpace::Data | ValueSpace::Transformed { .. })
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric formatting requires a numeric guide; time and categories retain their own labels.",
        ));
    }
    let mut ticks = vec![];
    let scale = match (family, &space) {
        (AxisScale::Linear(options), ValueSpace::Data | ValueSpace::Transformed { .. }) => {
            let scale = LinearScale::resolve(extent, options, range, viewport, spec.outside)?;
            if spec.visible {
                for t in scale.ticks(r.target_ticks, r.max_ticks)? {
                    if let Some(position) = scale.map(t.value)? {
                        ticks.push(GuideTick {
                            position,
                            label: numeric_label(spec, t.value, t.label)?,
                        });
                    }
                }
            }
            ResolvedScale::Linear(scale)
        }
        (
            AxisScale::Nonlinear { transform, domain },
            ValueSpace::Data | ValueSpace::Transformed { .. },
        ) => {
            let contribution = if matches!(transform, ScaleTransform::Log { .. }) {
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
            if spec.visible {
                for t in scale.ticks(r.target_ticks, r.max_ticks)? {
                    if let Some(position) = scale.map(t.value)? {
                        ticks.push(GuideTick {
                            position,
                            label: numeric_label(spec, t.value, t.label)?,
                        });
                    }
                }
            }
            ResolvedScale::Nonlinear(scale)
        }
        (AxisScale::Point(options), ValueSpace::Categorical { categories }) => {
            if viewport.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Point scales use category domains without numeric viewport policies.",
                ));
            }
            let scale = PointScale::resolve(categories, &options, range)?;
            if spec.visible {
                let stride = scale.domain().len().div_ceil(r.max_ticks).max(1);
                for label in scale.domain().iter().step_by(stride) {
                    if let Some(position) = scale.center(label)? {
                        ticks.push(GuideTick {
                            position,
                            label: label.clone(),
                        });
                    }
                }
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
            let scale = SessionScale::new(calendar, range, view, spec.outside)?;
            if spec.visible {
                for t in scale.ticks(r.target_ticks, r.max_ticks)? {
                    if let Some(position) = scale.map(t.value)? {
                        ticks.push(GuideTick {
                            position,
                            label: t.label,
                        });
                    }
                }
            }
            ResolvedScale::Session(scale)
        }
        (AxisScale::Band(options), ValueSpace::Categorical { categories }) => {
            if viewport.is_some() || spec.outside != OutsidePolicy::Extend {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Band scales use explicit category domains; numeric viewport/clamp/omit policies are unsupported.",
                ));
            }
            let scale = BandScale::resolve(categories, &options, range)?;
            if spec.visible {
                let stride = scale.domain().len().div_ceil(r.max_ticks).max(1);
                for label in scale.domain().iter().step_by(stride) {
                    if let Some(position) = scale.center(label)? {
                        ticks.push(GuideTick {
                            position,
                            label: label.clone(),
                        });
                    }
                }
            }
            ResolvedScale::Band(scale)
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
            let scale = UtcScale::new(domain, viewport, representation.unit, range, spec.outside)?;
            if spec.visible {
                for t in scale.ticks(
                    interval.unwrap_or(scale.auto_interval(r.target_ticks)?),
                    r.max_ticks,
                )? {
                    if let Some(position) = scale.map(t.value)? {
                        ticks.push(GuideTick {
                            position,
                            label: t.label,
                        });
                    }
                }
            }
            ResolvedScale::Utc(scale)
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Axis family does not match the prepared numeric/category/timestamp space.",
            ));
        }
    };
    ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
    Ok(ResolvedAxis {
        spec: spec.clone(),
        space,
        scale,
        ticks,
    })
}

pub(super) fn resolve_axis(
    chart: &PreparedChart,
    r: &LayoutRequest,
    spec: &AxisSpec,
    plot: Rect,
) -> ChartResult<ResolvedAxis> {
    let result = (|| {
        if let Some(ticks) = &spec.guide_ticks
            && (ticks.len() > r.max_ticks
                || ticks.iter().map(|t| t.label.len()).sum::<usize>() > r.limits.max_text_bytes)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom guide exceeds tick/text budgets.",
            ));
        }
        let mut axis = resolve_axis_inner(chart, r, spec, plot)?;
        if let Some(ticks) = &spec.guide_ticks {
            if spec.number_format.is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Explicit custom labels cannot also request a numeric formatter.",
                ));
            }
            axis.ticks.clear();
            for tick in ticks {
                if tick.label.is_empty() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Custom guide labels must be nonempty.",
                    ));
                }
                if let Some(position) = axis.map_value(&tick.value)?
                    && spec.visible
                {
                    axis.ticks.push(GuideTick {
                        position,
                        label: tick.label.clone(),
                    });
                }
            }
            axis.ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
        }
        Ok(axis)
    })();
    result.map_err(|mut e: crate::Diagnostic| {
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
                PreparedGeometry::Point(p) => include(*p, false)?,
                PreparedGeometry::Polygon(points) => {
                    for p in points {
                        include(*p, true)?;
                    }
                }
                PreparedGeometry::BandRun { lower, upper } => {
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

fn numeric_label(spec: &AxisSpec, value: f64, default: String) -> ChartResult<String> {
    if let Some(f) = &spec.number_format {
        f.format(value)
    } else {
        Ok(default)
    }
}
