//! Resolve typed coordinate views without changing statistical scale training.
use super::coordinate_map::{CoordinateDomain, CoordinateMap};
use super::{ResolvedAxis, ResolvedScale};
use crate::composition::ScaleValue;
use crate::grammar::{CoordinateLimits, CoordinateSpec, RadialMode, ThetaAxis};
use crate::scales::{Bounds, GgplotExpansion};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Rect};
fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::UnsupportedCapability,
        message,
        "Use coordinate limits compatible with the retained positional scale capability.",
    )
}
fn pair(bounds: Bounds) -> [f64; 2] {
    [bounds.start(), bounds.end()]
}
fn domain(axis: &ResolvedAxis) -> ChartResult<CoordinateDomain> {
    let (domain, viewport, range) = match &axis.scale {
        ResolvedScale::Linear(s) => (pair(s.domain()), pair(s.viewport()), s.range()),
        ResolvedScale::Numeric(s) => (
            pair(s.coordinate_domain()?),
            pair(s.coordinate_viewport()),
            s.range(),
        ),
        ResolvedScale::Calendar(s) => (
            pair(s.coordinate_domain()?),
            pair(s.coordinate_viewport()),
            s.range(),
        ),
        ResolvedScale::Nonlinear(s) => {
            let d = pair(s.domain());
            let transform = s.transform();
            (
                [
                    transform
                        .forward(d[0])?
                        .ok_or_else(|| error("Coordinate domain transform is undefined."))?,
                    transform
                        .forward(d[1])?
                        .ok_or_else(|| error("Coordinate domain transform is undefined."))?,
                ],
                pair(s.transformed_viewport()),
                s.range(),
            )
        }
        ResolvedScale::Unbounded(s) => (
            s.limits().map(|v| v.0),
            s.viewport().map(|v| v.0),
            s.range(),
        ),
        ResolvedScale::Utc(s) => {
            let offset = |x: i64| (i128::from(x) - i128::from(s.origin())) as f64;
            (
                [offset(s.domain().start), offset(s.domain().end)],
                [offset(s.viewport().start), offset(s.viewport().end)],
                s.range(),
            )
        }
        ResolvedScale::Session(s) => (
            pair(s.coordinate_domain()),
            pair(s.coordinate_viewport()),
            s.range(),
        ),
        ResolvedScale::Band(s) => (
            [1., s.domain().len() as f64],
            s.reference_viewport()
                .map(|v| v.map(|n| n.0))
                .unwrap_or([0.5, s.domain().len() as f64 + 0.5]),
            s.range(),
        ),
        ResolvedScale::Point(s) => (
            [1., s.domain().len() as f64],
            s.reference_viewport()
                .map(|v| v.map(|n| n.0))
                .unwrap_or([1., s.domain().len() as f64]),
            s.range(),
        ),
        ResolvedScale::Provider(s) => {
            if let Some(view) = s.category_viewport() {
                ([1., s.domain().len() as f64], view.map(|v| v.0), s.range())
            } else {
                return Err(error(
                    "Coordinate projection requires a provider with retained coordinate-space endpoints.",
                ));
            }
        }
        ResolvedScale::Secondary { primary, .. }
        | ResolvedScale::SecondaryDiscrete { primary, .. } => return domain(primary),
        ResolvedScale::SecondaryTime { axis, .. } => return domain(axis),
    };
    Ok(CoordinateDomain {
        domain,
        viewport,
        range: pair(range),
    })
}
pub(super) fn limit(
    axis: &ResolvedAxis,
    domain: CoordinateDomain,
    value: &ScaleValue,
) -> ChartResult<f64> {
    if axis.space.is_categorical()
        && let ScaleValue::Number(v) = value
    {
        return Ok(*v);
    }
    // Transform direct numeric limits without applying out-of-bounds censoring.
    if let ScaleValue::Number(v) = value {
        match &axis.scale {
            ResolvedScale::Linear(_) | ResolvedScale::Unbounded(_) => return Ok(*v),
            ResolvedScale::Nonlinear(s) => {
                return s.transform().forward(*v)?.ok_or_else(|| {
                    error("Coordinate limit is outside the scale transform domain.")
                });
            }
            ResolvedScale::Numeric(s) => return s.coordinate(*v),
            _ => {}
        }
    }
    if let ScaleValue::Timestamp { value, unit } = value {
        match &axis.scale {
            ResolvedScale::Utc(s) if s.unit() == *unit => {
                return Ok((i128::from(*value) - i128::from(s.origin())) as f64);
            }
            ResolvedScale::Calendar(s) if s.unit() == *unit => return s.coordinate_value(*value),
            ResolvedScale::Session(s) if s.calendar().unit == *unit => {
                return s.coordinate_value(*value)?.ok_or_else(|| {
                    error("Coordinate timestamp limit lies outside an active session.")
                });
            }
            _ => {}
        }
    }
    let p = axis
        .map_value(value)?
        .ok_or_else(|| error("Coordinate limit cannot be mapped through this typed scale."))?;
    Ok(domain.viewport[0]
        + (p - domain.range[0]) / (domain.range[1] - domain.range[0])
            * (domain.viewport[1] - domain.viewport[0]))
}
#[allow(clippy::too_many_arguments)]
fn view(
    axis: &ResolvedAxis,
    domain: CoordinateDomain,
    limits: &CoordinateLimits,
    enabled: [bool; 2],
    expansion: GgplotExpansion,
    transform: Option<&crate::scales::GgplotTransform>,
    preserve: bool,
    invert: bool,
) -> ChartResult<[f64; 2]> {
    if preserve && limits.is_none() && enabled == [true; 2] && transform.is_none() {
        return Ok(domain.viewport);
    }
    let mut values = domain.domain;
    if let Some(limits) = limits {
        for i in 0..2 {
            if let Some(value) = &limits[i] {
                values[i] = limit(axis, domain, value)?;
            }
        }
    }
    if let Some(t) = transform {
        values = values.map(|v| t.forward(v));
    }
    let expanded = pair(expansion.continuous_viewport(Bounds::new(values[0], values[1])?)?);
    for i in 0..2 {
        if enabled[i] {
            values[i] = expanded[i];
        }
    }
    if invert && let Some(t) = transform {
        values = values.map(|v| t.inverse(v));
    }
    Ok(values)
}
pub(super) fn resolve(
    spec: &CoordinateSpec,
    axes: [&ResolvedAxis; 2],
    plot: Rect,
) -> ChartResult<CoordinateMap> {
    resolve_with_windows(spec, axes, plot, None)
}
pub(super) fn resolve_with_windows(
    spec: &CoordinateSpec,
    axes: [&ResolvedAxis; 2],
    plot: Rect,
    windows: Option<&crate::state::AxisWindows>,
) -> ChartResult<CoordinateMap> {
    let domains = [domain(axes[0])?, domain(axes[1])?];
    let mut views = [domains[0].viewport, domains[1].viewport];
    for i in 0..2 {
        let normal = axes[i].spec.expansion.unwrap_or_else(|| {
            if axes[i].space.is_categorical() {
                GgplotExpansion::discrete_default()
            } else {
                GgplotExpansion::default()
            }
        });
        views[i] = match spec {
            CoordinateSpec::Geographic(v) => view(
                axes[i],
                domains[i],
                if i == 0 { &v.view.xlim } else { &v.view.ylim },
                if matches!(
                    v.projection,
                    crate::grammar::GeoProjectionSelection::Mapproj(_)
                ) {
                    if i == 0 {
                        [v.view.expand[3], v.view.expand[1]]
                    } else {
                        [v.view.expand[2], v.view.expand[0]]
                    }
                } else {
                    [false; 2]
                },
                normal,
                None,
                false,
                true,
            )?,
            CoordinateSpec::Cartesian(v) => view(
                axes[i],
                domains[i],
                if i == 0 { &v.xlim } else { &v.ylim },
                if i == 0 {
                    [v.expand[3], v.expand[1]]
                } else {
                    [v.expand[2], v.expand[0]]
                },
                normal,
                None,
                true,
                true,
            )?,
            CoordinateSpec::Transformed(v) => view(
                axes[i],
                domains[i],
                if i == 0 { &v.view.xlim } else { &v.view.ylim },
                if i == 0 {
                    [v.view.expand[3], v.view.expand[1]]
                } else {
                    [v.view.expand[2], v.view.expand[0]]
                },
                normal,
                Some(if i == 0 { &v.x } else { &v.y }),
                false,
                false,
            )?,
            CoordinateSpec::Radial(v) => {
                let theta = i == usize::from(v.theta == ThetaAxis::Y);
                let expansion = if v.mode == RadialMode::Polar {
                    axes[i].spec.expansion.unwrap_or(GgplotExpansion {
                        mult: [0.; 2],
                        add: [if theta && axes[i].space.is_categorical() {
                            0.5
                        } else {
                            0.
                        }; 2],
                    })
                } else {
                    normal
                };
                view(
                    axes[i],
                    domains[i],
                    if theta { &v.thetalim } else { &v.rlim },
                    [v.expand; 2],
                    expansion,
                    None,
                    v.mode == RadialMode::Radial,
                    true,
                )?
            }
        };
    }
    for i in 0..2 {
        if windows.is_some_and(|w| w.contains_key(&axes[i].spec.id)) {
            views[i] = if let CoordinateSpec::Transformed(v) = spec {
                domains[i].viewport.map(|x| {
                    if i == 0 {
                        v.x.forward(x)
                    } else {
                        v.y.forward(x)
                    }
                })
            } else {
                domains[i].viewport
            };
        }
    }
    if let CoordinateSpec::Geographic(v) = spec {
        let map = super::geographic::GeographicMap::new(v)?;
        let mut projected = match map.bounds(views, v.limits_method) {
            Ok(bounds) => bounds,
            Err(error)
                if error.code == DiagnosticCode::NumericalDomain
                    && matches!(v.projection, crate::grammar::GeoProjectionSelection::Crs(_)) =>
            {
                // coord_sf falls back to geometry-trained bounds when an authored limit
                // cannot be projected. The retained domain remains separate from that limit.
                views = domains.map(|axis| axis.domain);
                map.bounds(views, crate::grammar::GeoLimitsMethod::GeometryBounds)?
            }
            Err(error) => return Err(error),
        };
        for i in 0..2 {
            if matches!(
                v.projection,
                crate::grammar::GeoProjectionSelection::Mapproj(_)
            ) {
                break;
            }
            let enabled = if i == 0 {
                [v.view.expand[3], v.view.expand[1]]
            } else {
                [v.view.expand[2], v.view.expand[0]]
            };
            let expanded = axes[i]
                .spec
                .expansion
                .unwrap_or_default()
                .continuous_viewport(Bounds::new(projected[i][0], projected[i][1])?)?;
            for j in 0..2 {
                if enabled[j] {
                    projected[i][j] = [expanded.start(), expanded.end()][j];
                }
            }
        }
        return CoordinateMap::with_transformed_bounds(
            spec.clone(),
            domains,
            views,
            Some(projected),
            plot,
        );
    }
    if matches!(spec, CoordinateSpec::Transformed(_)) {
        CoordinateMap::with_transformed_bounds(
            spec.clone(),
            domains,
            [domains[0].domain, domains[1].domain],
            Some(views),
            plot,
        )
    } else {
        CoordinateMap::new(spec.clone(), domains, views, plot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ScaleId,
        grammar::{CartesianCoordinate, ValueSpace},
        layout::AxisSpec,
        scales::{LinearScale, OutsidePolicy},
        state::AxisWindow,
    };
    #[test]
    fn geographic_failed_limits_fall_back_to_trained_feature_domain() {
        use crate::grammar::{
            GeoAxisOrder, GeoCrs, GeoProjectionSelection, GeoTransform, GeographicCoordinate,
        };
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/geography-controls.json"
        ))
        .unwrap();
        let axes = [10., 30.]
            .into_iter()
            .enumerate()
            .map(|(index, end)| ResolvedAxis {
                spec: AxisSpec::new(
                    ScaleId::new(index as u64),
                    if index == 0 {
                        super::super::AxisSide::Bottom
                    } else {
                        super::super::AxisSide::Left
                    },
                ),
                space: ValueSpace::Data,
                scale: ResolvedScale::Linear(
                    LinearScale::from_domains(
                        Bounds::new(0., end).unwrap(),
                        Bounds::new(0., end).unwrap(),
                        Bounds::new(0., 100.).unwrap(),
                        OutsidePolicy::Extend,
                    )
                    .unwrap(),
                ),
                ticks: vec![],
            })
            .collect::<Vec<_>>();
        let spec = CoordinateSpec::Geographic(GeographicCoordinate {
            projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
            default_crs: Some(GeoCrs::Wgs84),
            view: CartesianCoordinate {
                xlim: Some([Some(ScaleValue::Number(0.)), Some(ScaleValue::Number(10.))]),
                ylim: Some([
                    Some(ScaleValue::Number(100.)),
                    Some(ScaleValue::Number(110.)),
                ]),
                ..Default::default()
            },
            ..Default::default()
        });
        let map = resolve(
            &spec,
            [&axes[0], &axes[1]],
            Rect::new(0., 0., 100., 100.).unwrap(),
        )
        .unwrap();
        let transform =
            GeoTransform::new(&GeoCrs::Wgs84, &GeoCrs::WebMercator, GeoAxisOrder::XY).unwrap();
        for (edge, p) in [[0., 100.], [100., 0.]].into_iter().enumerate() {
            let inverse = map
                .inverse(crate::Point::new(p[0], p[1]).unwrap(), 1)
                .unwrap()
                .unwrap();
            let projected = transform.transform(map.source_values(inverse[0])).unwrap();
            for (axis, key) in ["x_range", "y_range"].into_iter().enumerate() {
                let expected = fixture["failed_limits"][key][edge].as_f64().unwrap();
                assert!(
                    (projected[axis] - expected).abs() < 1e-7,
                    "{axis} {edge}: {} {expected}",
                    projected[axis]
                );
            }
        }
    }
    #[test]
    fn timestamp_coordinate_limits_preserve_origin_precision_outside_censored_view() {
        use crate::{
            data::TimeUnit,
            scales::{TimeBounds, UtcScale},
        };
        let origin = 1_700_000_000_000_000_001_i64;
        let scale = UtcScale::new(
            TimeBounds {
                start: origin,
                end: origin + 1000,
            },
            Some(TimeBounds {
                start: origin + 400,
                end: origin + 600,
            }),
            TimeUnit::Nanoseconds,
            Bounds::new(0., 100.).unwrap(),
            OutsidePolicy::Omit,
        )
        .unwrap();
        let axis = ResolvedAxis {
            spec: AxisSpec::new(ScaleId::new(0), crate::layout::AxisSide::Bottom),
            space: ValueSpace::Timestamp {
                origin,
                representation: crate::data::TimestampType {
                    unit: TimeUnit::Nanoseconds,
                    timezone: "UTC".into(),
                },
            },
            scale: ResolvedScale::Utc(scale),
            ticks: vec![],
        };
        let value = ScaleValue::Timestamp {
            value: origin + 17,
            unit: TimeUnit::Nanoseconds,
        };
        assert_eq!(axis.map_value(&value).unwrap(), None);
        let d = domain(&axis).unwrap();
        assert_eq!(limit(&axis, d, &value).unwrap() - d.domain[0], 17.);
    }
    #[test]
    fn runtime_window_overrides_authored_coordinate_zoom_without_expansion() {
        let scale = LinearScale::from_domains(
            Bounds::new(0., 1.).unwrap(),
            Bounds::new(0.4, 0.6).unwrap(),
            Bounds::new(0., 100.).unwrap(),
            OutsidePolicy::Extend,
        )
        .unwrap();
        let axes = [
            ResolvedAxis {
                spec: AxisSpec::new(ScaleId::new(0), crate::layout::AxisSide::Bottom),
                space: ValueSpace::Data,
                scale: ResolvedScale::Linear(scale.clone()),
                ticks: vec![],
            },
            ResolvedAxis {
                spec: AxisSpec::new(ScaleId::new(1), crate::layout::AxisSide::Left),
                space: ValueSpace::Data,
                scale: ResolvedScale::Linear(scale),
                ticks: vec![],
            },
        ];
        let spec = CoordinateSpec::Cartesian(CartesianCoordinate {
            xlim: Some([Some(ScaleValue::Number(0.2)), Some(ScaleValue::Number(0.8))]),
            expand: [false; 4],
            ..Default::default()
        });
        let window = [(ScaleId::new(0), AxisWindow::Numeric(0.4, 0.6))]
            .into_iter()
            .collect();
        let result = resolve_with_windows(
            &spec,
            [&axes[0], &axes[1]],
            Rect::new(0., 0., 100., 100.).unwrap(),
            Some(&window),
        )
        .unwrap();
        assert_eq!(result.view[0], [0.4, 0.6]);
    }
}
