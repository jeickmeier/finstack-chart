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
    let mut ticks = vec![];
    let scale = match (family, &space) {
        (AxisScale::Linear(options), ValueSpace::Data | ValueSpace::Transformed { .. }) => {
            let scale = LinearScale::resolve(extent, options, range, viewport, spec.outside)?;
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
            ResolvedScale::Linear(scale)
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
    resolve_axis_inner(chart, r, spec, plot).map_err(|mut e| {
        e.message = format!("Scale {}: {}", spec.id.get(), e.message);
        e
    })
}
