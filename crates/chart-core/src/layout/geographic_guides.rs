//! Geographic graticule candidates and edge intersections over shared guide measurement.
use super::{AxisSide, GuideTick, LayoutRequest, ResolvedGuide, coordinate_map::CoordinateMap};
use crate::grammar::{CoordinateSpec, GeoAxisLabel, GeoAxisOrder, GeoTransform};
use crate::path::Path;
use crate::scene::PathCommand;
use crate::{ChartResult, DiagnosticCode, GuideId, Point};
use std::collections::BTreeMap;
fn spec(map: &CoordinateMap) -> &crate::grammar::GeographicCoordinate {
    let CoordinateSpec::Geographic(v) = &map.spec else {
        unreachable!("geographic guide")
    };
    v
}
fn datum_bounds(map: &CoordinateMap) -> ChartResult<Option<[[f64; 2]; 2]>> {
    let s = spec(map);
    let Some(datum) = &s.graticule.datum else {
        return Ok(None);
    };
    let transform = GeoTransform::new(s.source_crs(), datum, GeoAxisOrder::XY)?;
    let mut bounds = [[f64::INFINITY, f64::NEG_INFINITY]; 2];
    let add = |bounds: &mut [[f64; 2]; 2], p: [f64; 2]| {
        if let Ok(p) = transform.transform(p) {
            for i in 0..2 {
                bounds[i][0] = bounds[i][0].min(p[i]);
                bounds[i][1] = bounds[i][1].max(p[i]);
            }
        }
    };
    for i in 0..s.graticule.subdivisions {
        let t = i as f64 / (s.graticule.subdivisions - 1) as f64;
        let x = map.plot.origin().x() + map.plot.width() * t;
        let y = map.plot.origin().y() + map.plot.height() * t;
        for p in [
            [x, map.plot.origin().y()],
            [x, map.plot.max_y()],
            [map.plot.origin().x(), y],
            [map.plot.max_x(), y],
        ] {
            if let Some(points) = map.inverse(Point::new(p[0], p[1])?, 1)? {
                for p in points {
                    add(&mut bounds, map.source_values(p));
                }
            }
        }
    }
    if bounds.iter().flatten().any(|v| !v.is_finite()) {
        for x in map.view[0] {
            for y in map.view[1] {
                add(&mut bounds, [x, y]);
            }
        }
    }
    if bounds.iter().flatten().any(|v| !v.is_finite()) {
        return Ok(None);
    }
    Ok(Some(bounds))
}
fn pretty(bounds: [f64; 2], budget: usize) -> ChartResult<Vec<f64>> {
    let span = bounds[1] - bounds[0];
    if span <= 0. {
        return Ok(vec![bounds[0]]);
    }
    let cell = span / 6.;
    let base = libm::pow(10., libm::floor(libm::log10(cell)));
    let mut step = base;
    if 2. * base - cell < 1.5 * (cell - step) {
        step = 2. * base;
    }
    if 5. * base - cell < 2.75 * (cell - step) {
        step = 5. * base;
    }
    if 10. * base - cell < 1.5 * (cell - step) {
        step = 10. * base;
    }
    crate::scales::tick_candidates(bounds[0], bounds[1], span / step, budget)
}
pub(super) fn levels(
    map: &CoordinateMap,
    longitude: bool,
    r: &LayoutRequest,
) -> ChartResult<Vec<f64>> {
    let s = spec(map);
    let Some(bounds) = datum_bounds(map)? else {
        return Ok(vec![]);
    };
    let explicit = if longitude {
        &s.graticule.longitude
    } else {
        &s.graticule.latitude
    };
    let mut values = if let Some(values) = explicit {
        values.clone()
    } else if longitude && bounds[0][1] <= 180. && bounds[0][0] < -170. && bounds[0][1] > 170. {
        (-3..=3).map(|i| f64::from(i) * 60.).collect()
    } else {
        pretty(bounds[usize::from(!longitude)], r.max_ticks)?
    };
    if values.len() > r.max_ticks {
        return Err(crate::scales::error(
            DiagnosticCode::ResourceLimit,
            "Geographic graticule candidates exceed the caller tick budget.",
        ));
    }
    let datum = s.graticule.datum.as_ref().expect("datum bounds");
    if GeoTransform::new(datum, datum, GeoAxisOrder::XY)?.source_is_geographic() {
        if longitude {
            let positive = values.iter().copied().reduce(f64::min).unwrap_or(0.) >= -15.
                && values.iter().copied().reduce(f64::max).unwrap_or(0.) > 195.;
            values.retain(|v| {
                if positive {
                    *v >= 0. && *v <= 360.
                } else {
                    *v >= -180. && *v <= 180.
                }
            });
        } else {
            values.retain(|v| *v > -90. && *v < 90.);
        }
    }
    Ok(values)
}
pub(super) fn path(
    map: &CoordinateMap,
    longitude: bool,
    value: f64,
    r: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<PathCommand>> {
    let s = spec(map);
    let Some(mut bounds) = datum_bounds(map)? else {
        return Ok(vec![]);
    };
    if matches!(
        s.projection,
        crate::grammar::GeoProjectionSelection::Mapproj(_)
    ) {
        bounds = map.view;
        for (axis, bound) in bounds.iter_mut().enumerate() {
            let span = bound[1] - bound[0];
            let mid = bound[0] / 2. + bound[1] / 2.;
            let half = if axis == 0 { 180. } else { 90. };
            bound[0] = (bound[0] - span * 0.2).max(mid - half);
            bound[1] = (bound[1] + span * 0.2).min(mid + half);
        }
    }
    let datum = s.graticule.datum.as_ref().expect("datum");
    let transform = GeoTransform::new(datum, s.source_crs(), GeoAxisOrder::XY)?;
    // Include supplied edge levels just as the source expands its sampled graticule box.
    for (axis, bound) in bounds.iter_mut().enumerate() {
        for v in levels(map, axis == 0, r)? {
            bound[0] = bound[0].min(v);
            bound[1] = bound[1].max(v);
        }
    }
    let mut path = Path::new();
    let mut open = false;
    for i in 0..s.graticule.subdivisions {
        *remaining = remaining.checked_sub(1).ok_or_else(|| {
            crate::scales::error(
                DiagnosticCode::ResourceLimit,
                "Geographic graticule source samples exceed the path budget.",
            )
        })?;
        let t = i as f64 / (s.graticule.subdivisions - 1) as f64;
        let other = usize::from(longitude);
        let v = bounds[other][0] + (bounds[other][1] - bounds[other][0]) * t;
        let raw = if longitude { [value, v] } else { [v, value] };
        match transform.transform(raw) {
            Ok(p) => {
                let old = Point::new(
                    map.axes[0].range[0]
                        + (p[0] - map.axes[0].viewport[0])
                            / (map.axes[0].viewport[1] - map.axes[0].viewport[0])
                            * (map.axes[0].range[1] - map.axes[0].range[0]),
                    map.axes[1].range[0]
                        + (p[1] - map.axes[1].viewport[0])
                            / (map.axes[1].viewport[1] - map.axes[1].viewport[0])
                            * (map.axes[1].range[1] - map.axes[1].range[0]),
                )?;
                if open {
                    path.line_to(old.x(), old.y())?;
                } else {
                    path.move_to(old.x(), old.y())?;
                    open = true;
                }
            }
            Err(e) if e.code == DiagnosticCode::NumericalDomain => open = false,
            Err(e) => return Err(e),
        }
    }
    super::coordinate_path::project(
        &path.geometry(),
        map,
        super::coordinate_path::tolerance(r),
        remaining,
    )?
    .lower(super::coordinate_path::tolerance(r), *remaining)
}
fn labels(values: &[f64], longitude: bool) -> Vec<String> {
    let values = values
        .iter()
        .map(|v| {
            if longitude && *v > 180. {
                *v - 360.
            } else {
                *v
            }
        })
        .collect::<Vec<_>>();
    let maximum = values.iter().map(|v| v.abs()).fold(0., f64::max);
    let decimals = if maximum == 0. {
        0
    } else {
        (9. - libm::floor(libm::log10(maximum))).clamp(0., 15.) as usize
    };
    let mut numbers = values
        .iter()
        .map(|v| {
            let number = format!("{:.*}", decimals, v.abs());
            if decimals == 0 {
                number
            } else {
                number
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        })
        .collect::<Vec<_>>();
    for n in &mut numbers {
        if n.is_empty() {
            *n = "0".into();
        }
    }
    let width = numbers.iter().map(String::len).max().unwrap_or(0);
    values
        .into_iter()
        .zip(numbers)
        .map(|(v, n)| {
            format!(
                "{:>width$}°{}",
                n,
                if v == 0. || longitude && v.abs() == 180. {
                    ""
                } else if longitude {
                    if v < 0. { "W" } else { "E" }
                } else if v < 0. {
                    "S"
                } else {
                    "N"
                }
            )
        })
        .collect()
}
fn intersections(commands: &[PathCommand], map: &CoordinateMap, side: AxisSide) -> Vec<f64> {
    let horizontal = side.horizontal();
    let boundary = match side {
        AxisSide::Bottom => map.plot.max_y(),
        AxisSide::Top => map.plot.origin().y(),
        AxisSide::Left => map.plot.origin().x(),
        AxisSide::Right => map.plot.max_x(),
    };
    let mut previous = None;
    let mut result = vec![];
    for command in commands {
        match command {
            PathCommand::MoveTo(p) => previous = Some(*p),
            PathCommand::LineTo(b) => {
                if let Some(a) = previous {
                    let (from, to) = if horizontal {
                        (a.y(), b.y())
                    } else {
                        (a.x(), b.x())
                    };
                    if from != to {
                        let t = (boundary - from) / (to - from);
                        if (-1e-10..=1. + 1e-10).contains(&t) {
                            let v = if horizontal {
                                a.x() + t * (b.x() - a.x())
                            } else {
                                a.y() + t * (b.y() - a.y())
                            };
                            let range = if horizontal {
                                [map.plot.origin().x(), map.plot.max_x()]
                            } else {
                                [map.plot.origin().y(), map.plot.max_y()]
                            };
                            if v >= range[0] - 1e-8
                                && v <= range[1] + 1e-8
                                && !result.iter().any(|p: &f64| (*p - v).abs() < 1e-8)
                            {
                                result.push(v);
                            }
                        }
                    }
                }
                previous = Some(*b);
            }
            _ => previous = None,
        }
    }
    result
}
/// Install requested geographic edge guides before side-specific theme resolution.
pub(super) fn configure(
    chart: &crate::grammar::PreparedChart,
    request: &mut LayoutRequest,
) -> ChartResult<()> {
    let Some(CoordinateSpec::Geographic(spec)) = chart.definition().coordinate.as_ref() else {
        return Ok(());
    };
    if matches!(
        spec.projection,
        crate::grammar::GeoProjectionSelection::Mapproj(_)
    ) {
        return Ok(());
    }
    let mut guides = request
        .axes
        .iter()
        .map(|a| a.default_guide())
        .chain(request.guides.iter().cloned())
        .collect::<Vec<_>>();
    for (index, side) in [
        AxisSide::Bottom,
        AxisSide::Left,
        AxisSide::Top,
        AxisSide::Right,
    ]
    .into_iter()
    .enumerate()
    {
        if spec.graticule.label_axes[index] == GeoAxisLabel::None
            || guides.iter().any(|g| g.side == side)
        {
            continue;
        }
        if let Some(mut guide) = guides
            .iter()
            .find(|g| g.visible && g.side.horizontal() == side.horizontal())
            .cloned()
        {
            let mut value = u64::MAX;
            while guides.iter().any(|g| g.id == GuideId::new(value)) {
                value = value.checked_sub(1).ok_or_else(|| {
                    crate::scales::error(
                        DiagnosticCode::ResourceLimit,
                        "Geographic guide identity space is exhausted.",
                    )
                })?;
            }
            guide.id = GuideId::new(value);
            guide.side = side;
            guide.style.title = None;
            request.guides.push(guide.clone());
            guides.push(guide);
        }
    }
    Ok(())
}
pub(super) fn relocate(
    map: &CoordinateMap,
    guides: &mut BTreeMap<GuideId, ResolvedGuide>,
    r: &LayoutRequest,
) -> ChartResult<()> {
    let s = spec(map);
    if matches!(
        s.projection,
        crate::grammar::GeoProjectionSelection::Mapproj(_)
    ) {
        for guide in guides.values_mut() {
            let axis = usize::from(!guide.spec.side.horizontal());
            let mut ticks = Vec::new();
            for mut tick in std::mem::take(&mut guide.ticks) {
                let mut p = map.view.map(|v| v[0]);
                p[axis] = map.axes[axis].viewport[0]
                    + (tick.position - map.axes[axis].range[0])
                        / (map.axes[axis].range[1] - map.axes[axis].range[0])
                        * (map.axes[axis].viewport[1] - map.axes[axis].viewport[0]);
                let old = Point::new(
                    map.axes[0].range[0]
                        + (p[0] - map.axes[0].viewport[0])
                            / (map.axes[0].viewport[1] - map.axes[0].viewport[0])
                            * (map.axes[0].range[1] - map.axes[0].range[0]),
                    map.axes[1].range[0]
                        + (p[1] - map.axes[1].viewport[0])
                            / (map.axes[1].viewport[1] - map.axes[1].viewport[0])
                            * (map.axes[1].range[1] - map.axes[1].range[0]),
                )?;
                if let Some(p) = map.project(old)? {
                    tick.position = if axis == 0 { p.x() } else { p.y() };
                    ticks.push(tick);
                }
            }
            guide.ticks = ticks;
            guide.minor_ticks.clear();
            guide.tick_indices = (0..guide.ticks.len()).collect();
        }
        return Ok(());
    }
    let mut remaining = r.limits.max_path_commands;
    let longitude = levels(map, true, r)?;
    let latitude = levels(map, false, r)?;
    let geographic = s
        .graticule
        .datum
        .as_ref()
        .map(|datum| {
            GeoTransform::new(datum, datum, GeoAxisOrder::XY).map(|v| v.source_is_geographic())
        })
        .transpose()?
        .unwrap_or(false);
    let label = |values: &[f64], longitude| {
        if geographic {
            labels(values, longitude)
        } else {
            let numbers = values.iter().map(ToString::to_string).collect::<Vec<_>>();
            let width = numbers.iter().map(String::len).max().unwrap_or(0);
            numbers
                .into_iter()
                .map(|v| format!("{v:>width$}"))
                .collect()
        }
    };
    let lon_labels = label(&longitude, true);
    let lat_labels = label(&latitude, false);
    for guide in guides.values_mut() {
        let index = match guide.spec.side {
            AxisSide::Bottom => 0,
            AxisSide::Left => 1,
            AxisSide::Top => 2,
            AxisSide::Right => 3,
        };
        let policy = s.graticule.label_axes[index];
        guide.ticks.clear();
        guide.minor_ticks.clear();
        for (is_lon, values, text) in [
            (true, &longitude, &lon_labels),
            (false, &latitude, &lat_labels),
        ] {
            if policy == GeoAxisLabel::None
                || is_lon && policy == GeoAxisLabel::Latitude
                || !is_lon && policy == GeoAxisLabel::Longitude
            {
                continue;
            }
            for (value, label) in values.iter().zip(text) {
                let commands = path(map, is_lon, *value, r, &mut remaining)?;
                for position in intersections(&commands, map, guide.spec.side) {
                    guide.ticks.push(GuideTick {
                        value: crate::composition::ScaleValue::Number(*value),
                        position,
                        label: label.clone(),
                    });
                }
            }
        }
        guide.tick_indices = (0..guide.ticks.len()).collect();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::coordinate_map::CoordinateDomain;
    use super::*;
    use crate::grammar::{GeoCrs, GeoProjectionSelection, GeographicCoordinate};
    #[test]
    fn pinned_sf_graticule_levels_and_projected_edge_intersections() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/geography-controls.json"
        ))
        .unwrap();
        let case = &fixture["coordinates"][0];
        let x: [f64; 2] = serde_json::from_value(case["x_range"].clone()).unwrap();
        let y: [f64; 2] = serde_json::from_value(case["y_range"].clone()).unwrap();
        let view = [[0., 15.], [-2., 8.]];
        let axes = view.map(|v| CoordinateDomain {
            domain: v,
            viewport: v,
            range: v,
        });
        let map = CoordinateMap::with_transformed_bounds(
            CoordinateSpec::Geographic(GeographicCoordinate {
                projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
                default_crs: Some(GeoCrs::Wgs84),
                ..Default::default()
            }),
            axes,
            view,
            Some([x, y]),
            crate::Rect::new(0., 0., 600., 400.).unwrap(),
        )
        .unwrap();
        let request = LayoutRequest::new(
            map.plot,
            crate::services::Units::Points,
            crate::services::ResourceDescriptor {
                id: crate::ResourceId::new(1),
                revision: crate::Revision::INITIAL,
                kind: crate::services::ResourceKind::Font,
                byte_len: 1,
            },
        );
        for longitude in [true, false] {
            let expected = case["graticule"]["degree"]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["graticule"]["type"].as_array().unwrap())
                .filter(|(_, kind)| kind.as_str() == Some(if longitude { "E" } else { "N" }))
                .map(|(v, _)| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            let actual = levels(&map, longitude, &request).unwrap();
            assert_eq!(actual, expected);
            for value in actual {
                let commands = path(&map, longitude, value, &request, &mut 100000).unwrap();
                let side = if longitude {
                    AxisSide::Bottom
                } else {
                    AxisSide::Left
                };
                let positions = intersections(&commands, &map, side);
                assert_eq!(positions.len(), 1, "{longitude} {value}");
                let index = case["graticule"]["degree"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(case["graticule"]["type"].as_array().unwrap())
                    .position(|(v, kind)| {
                        v.as_f64() == Some(value)
                            && kind.as_str() == Some(if longitude { "E" } else { "N" })
                    })
                    .unwrap();
                let expected = if longitude {
                    (case["graticule"]["x_start"][index].as_f64().unwrap() - x[0]) / (x[1] - x[0])
                        * 600.
                } else {
                    (1. - (case["graticule"]["y_start"][index].as_f64().unwrap() - y[0])
                        / (y[1] - y[0]))
                        * 400.
                };
                assert!((positions[0] - expected).abs() < 1e-8);
            }
        }
        assert_eq!(labels(&[0., 2., 14.], true), [" 0°", " 2°E", "14°E"]);
        assert_eq!(
            labels(&[-180., 0., 180., 240.], true),
            ["180°", "  0°", "180°", "120°W"]
        );
    }
}
