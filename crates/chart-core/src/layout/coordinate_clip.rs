//! Shared annulus/sector clipping before native, publication and hit indexing.
use super::coordinate_map::CoordinateMap;
use crate::grammar::{CoordinateClip, CoordinateSpec, RadialMode};
use crate::path::{Path, PathGeometry};
use crate::scene::{FillRule, Primitive};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point};
fn charge(left: &mut usize, n: usize) -> ChartResult<()> {
    *left = left.checked_sub(n).ok_or_else(|| {
        Diagnostic::error(
            DiagnosticCode::ResourceLimit,
            "Coordinate clipping exceeds its explicit polygon work budget.",
            "Reduce the geometry or increase the caller's path budget.",
        )
    })?;
    Ok(())
}
fn cross(a: Point, b: Point, c: Point) -> f64 {
    (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
}
fn intersection(a: Point, b: Point, p: Point, q: Point) -> ChartResult<Point> {
    let u = cross(p, q, a);
    let v = cross(p, q, b);
    let t = u / (u - v);
    Point::new(a.x() + t * (b.x() - a.x()), a.y() + t * (b.y() - a.y()))
}
fn polygon(
    mut subject: Vec<Point>,
    triangle: &[Point; 3],
    remaining: &mut usize,
) -> ChartResult<Vec<Point>> {
    let sign = if cross(triangle[0], triangle[1], triangle[2]) >= 0. {
        1.
    } else {
        -1.
    };
    for i in 0..3 {
        if subject.is_empty() {
            break;
        }
        charge(remaining, subject.len())?;
        let p = triangle[i];
        let q = triangle[(i + 1) % 3];
        let mut out = vec![];
        let mut a = *subject.last().expect("nonempty");
        for b in subject {
            let ai = cross(p, q, a) * sign >= 0.;
            let bi = cross(p, q, b) * sign >= 0.;
            if ai != bi {
                out.push(intersection(a, b, p, q)?);
            }
            if bi {
                out.push(b);
            }
            a = b;
        }
        subject = out;
    }
    Ok(subject)
}
fn triangles(
    map: &CoordinateMap,
    error: f64,
    remaining: &mut usize,
) -> ChartResult<Vec<[Point; 3]>> {
    let CoordinateSpec::Radial(v) = &map.spec else {
        return Ok(vec![]);
    };
    let direction = if v.mode == RadialMode::Polar {
        f64::from(v.direction)
    } else {
        1.
    };
    let a = map.arc[0].min(map.arc[1]);
    let span = (map.arc[1] - map.arc[0]).abs().min(std::f64::consts::TAU);
    let outer = map.radii[0].max(map.radii[1]);
    let inner = map.radii[0].min(map.radii[1]);
    let radius = outer
        * (map.plot.width() / (map.bbox[0][1] - map.bbox[0][0]))
            .max(map.plot.height() / (map.bbox[1][1] - map.bbox[1][0]));
    let step =
        (2. * libm::acos((1. - error / radius).clamp(-1., 1.))).min(std::f64::consts::FRAC_PI_4);
    let count = (libm::ceil(span / step) as usize).max(1);
    charge(remaining, count.saturating_mul(6))?;
    let point = |angle: f64, r: f64| {
        Point::new(
            map.plot.origin().x()
                + ((0.5 + r * libm::sin(angle * direction)) - map.bbox[0][0])
                    / (map.bbox[0][1] - map.bbox[0][0])
                    * map.plot.width(),
            map.plot.max_y()
                - ((0.5 + r * libm::cos(angle * direction)) - map.bbox[1][0])
                    / (map.bbox[1][1] - map.bbox[1][0])
                    * map.plot.height(),
        )
    };
    let mut out = vec![];
    for i in 0..count {
        let x = a + span * i as f64 / count as f64;
        let y = a + span * (i + 1) as f64 / count as f64;
        let p = point(x, inner)?;
        let q = point(x, outer)?;
        let r = point(y, outer)?;
        let s = point(y, inner)?;
        out.push([p, q, r]);
        if inner > 0. {
            out.push([p, r, s]);
        }
    }
    Ok(out)
}
fn wholly_inside(map: &CoordinateMap, points: &[Point], error: f64) -> bool {
    if (map.arc[1] - map.arc[0]).abs() < std::f64::consts::TAU {
        return false;
    }
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for p in points {
        let v = [
            map.bbox[0][0]
                + (p.x() - map.plot.origin().x()) / map.plot.width()
                    * (map.bbox[0][1] - map.bbox[0][0])
                - 0.5,
            map.bbox[1][0]
                + (map.plot.max_y() - p.y()) / map.plot.height()
                    * (map.bbox[1][1] - map.bbox[1][0])
                - 0.5,
        ];
        for i in 0..2 {
            low[i] = low[i].min(v[i]);
            high[i] = high[i].max(v[i]);
        }
    }
    let near = libm::hypot(0_f64.clamp(low[0], high[0]), 0_f64.clamp(low[1], high[1]));
    let far = libm::hypot(
        low[0].abs().max(high[0].abs()),
        low[1].abs().max(high[1].abs()),
    );
    let inner = map.radii[0].min(map.radii[1]);
    let outer = map.radii[0].max(map.radii[1]);
    if near >= inner && far <= outer {
        return true;
    }
    // A wrapped compound contour can enclose the annulus while its bounds
    // contain the hole. Check each edge and the center winding instead of
    // clipping every edge against every sector triangle.
    let tolerance = error
        * ((map.bbox[0][1] - map.bbox[0][0]) / map.plot.width())
            .min((map.bbox[1][1] - map.bbox[1][0]) / map.plot.height());
    let normalized = |p: Point| {
        [
            (p.x() - map.plot.origin().x()) / map.plot.width() * (map.bbox[0][1] - map.bbox[0][0])
                + map.bbox[0][0]
                - 0.5,
            (map.plot.max_y() - p.y()) / map.plot.height() * (map.bbox[1][1] - map.bbox[1][0])
                + map.bbox[1][0]
                - 0.5,
        ]
    };
    let mut winding = 0_i32;
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        let a = normalized(*a);
        let b = normalized(*b);
        let d = [b[0] - a[0], b[1] - a[1]];
        let length = d[0] * d[0] + d[1] * d[1];
        let t = if length == 0. {
            0.
        } else {
            (-(a[0] * d[0] + a[1] * d[1]) / length).clamp(0., 1.)
        };
        if libm::hypot(a[0] + t * d[0], a[1] + t * d[1]) < inner - tolerance
            || libm::hypot(a[0], a[1]) > outer + tolerance
        {
            return false;
        }
        let cross = a[0] * b[1] - a[1] * b[0];
        if a[1] <= 0. && b[1] > 0. && cross > 0. {
            winding += 1;
        }
        if a[1] > 0. && b[1] <= 0. && cross < 0. {
            winding -= 1;
        }
    }
    inner == 0. || winding == 0
}
fn append(path: &mut Path, vertices: &[Point], remaining: &mut usize) -> ChartResult<()> {
    if vertices.len() < 3 {
        return Ok(());
    }
    charge(remaining, vertices.len() + 1)?;
    path.move_to(vertices[0].x(), vertices[0].y())?;
    for p in &vertices[1..] {
        path.line_to(p.x(), p.y())?;
    }
    path.close_path()
}
fn overlap(points: &[Point], triangle: &[Point; 3]) -> bool {
    let extent = |values: &[Point]| {
        values.iter().fold(
            [
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ],
            |v, p| {
                [
                    v[0].min(p.x()),
                    v[1].max(p.x()),
                    v[2].min(p.y()),
                    v[3].max(p.y()),
                ]
            },
        )
    };
    let a = extent(points);
    let b = extent(triangle);
    a[0] <= b[1] && b[0] <= a[1] && a[2] <= b[3] && b[2] <= a[3]
}
fn filled(
    geometry: PathGeometry,
    fill: crate::scene::Color,
    anchors: Vec<Point>,
    rule: FillRule,
    map: &CoordinateMap,
    error: f64,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    let flat = geometry.flatten(error / 2., *remaining)?;
    let clip = triangles(map, error / 2., remaining)?;
    let mut path = Path::new();
    for sub in flat.subpaths {
        if sub.points.len() < 3 {
            continue;
        }
        if wholly_inside(map, &sub.points, error / 2.) {
            append(&mut path, &sub.points, remaining)?;
            continue;
        }
        for triangle in &clip {
            charge(remaining, 1)?;
            if !overlap(&sub.points, triangle) {
                continue;
            }
            let vertices = polygon(sub.points.clone(), triangle, remaining)?;
            if vertices.len() < 3 {
                continue;
            }
            charge(remaining, vertices.len() + 1)?;
            path.move_to(vertices[0].x(), vertices[0].y())?;
            for p in &vertices[1..] {
                path.line_to(p.x(), p.y())?;
            }
            path.close_path()?;
        }
    }
    let geometry = path.geometry();
    if !geometry.has_segments() {
        return Ok(vec![]);
    }
    Ok(vec![Primitive::ShapePath {
        geometry,
        fill: Some(fill),
        stroke: None,
        dashes: vec![],
        anchors,
        fill_rule: rule,
    }])
}
pub(super) fn clip(
    primitive: Primitive,
    map: &CoordinateMap,
    targets: usize,
    error: f64,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    if !matches!(&map.spec,CoordinateSpec::Radial(v) if v.mode==RadialMode::Radial)
        || map.clip() == CoordinateClip::Off
    {
        return Ok(vec![primitive]);
    }
    match primitive {
        Primitive::FilledPath { commands, fill } => filled(
            PathGeometry::from_beziers(&commands)?,
            fill,
            super::project::command_anchors(&commands, targets),
            FillRule::NonZero,
            map,
            error,
            remaining,
        ),
        Primitive::Rectangle { bounds, fill } => {
            let mut p = Path::new();
            p.rect(
                bounds.origin().x(),
                bounds.origin().y(),
                bounds.width(),
                bounds.height(),
            )?;
            filled(
                p.geometry(),
                fill,
                vec![],
                FillRule::NonZero,
                map,
                error,
                remaining,
            )
        }
        Primitive::Point {
            center,
            radius,
            fill,
        } => filled(
            super::stroke_outline::circle(center, radius, *remaining)?,
            fill,
            vec![center],
            FillRule::NonZero,
            map,
            error,
            remaining,
        ),
        Primitive::Symbol {
            center,
            radius,
            kind,
            fill,
        } => filled(
            PathGeometry::from_beziers(&crate::scene::symbol_path(center, radius, kind)?)?,
            fill,
            vec![center],
            FillRule::NonZero,
            map,
            error,
            remaining,
        ),
        Primitive::ShapePath {
            geometry,
            fill: Some(fill),
            stroke: None,
            anchors,
            fill_rule,
            ..
        } => filled(geometry, fill, anchors, fill_rule, map, error, remaining),
        Primitive::VectorPath {
            geometry,
            fill: Some(fill),
            stroke: None,
            ..
        } => filled(
            geometry,
            fill,
            vec![],
            FillRule::NonZero,
            map,
            error,
            remaining,
        ),
        Primitive::GlyphRun {
            origin,
            rotation,
            run,
            color,
        } => filled(
            PathGeometry::from_beziers(&crate::typography::placed_outlines(
                &run, origin, rotation,
            )?)?,
            color,
            vec![origin; targets],
            FillRule::NonZero,
            map,
            error,
            remaining,
        ),
        primitive @ (Primitive::ShapePath {
            stroke: Some(_), ..
        }
        | Primitive::VectorPath {
            stroke: Some(_), ..
        }
        | Primitive::Rule { .. }
        | Primitive::Path { .. }
        | Primitive::DashedPath { .. }) => {
            let primitive = match primitive {
                Primitive::ShapePath {
                    geometry,
                    fill,
                    stroke: Some(stroke),
                    dashes,
                    anchors,
                    fill_rule,
                } if stroke.color.alpha == 0 => Primitive::ShapePath {
                    geometry,
                    fill,
                    stroke: None,
                    dashes,
                    anchors,
                    fill_rule,
                },
                Primitive::VectorPath {
                    geometry,
                    fill,
                    stroke: Some(stroke),
                    dashes,
                } if stroke.color.alpha == 0 => Primitive::VectorPath {
                    geometry,
                    fill,
                    stroke: None,
                    dashes,
                },
                Primitive::Rule { stroke, .. }
                | Primitive::Path { stroke, .. }
                | Primitive::DashedPath { stroke, .. }
                    if stroke.color.alpha == 0 =>
                {
                    return Ok(vec![]);
                }
                other => other,
            };
            let pieces = super::stroke_outline::expand(
                primitive,
                targets,
                Some(crate::grammar::LineEnd::Butt),
                Some(crate::grammar::LineJoin::Miter),
                remaining,
            )?;
            let mut result = vec![];
            for p in pieces {
                result.extend(clip(p, map, targets, error, remaining)?);
            }
            Ok(result)
        }
        other => Ok(vec![other]),
    }
}

/// Clip panel furniture to its sector independently of layer overflow policy.
pub(super) fn panel(
    primitive: Primitive,
    map: &CoordinateMap,
    error: f64,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    let mut map = map.clone();
    if let CoordinateSpec::Radial(v) = &mut map.spec {
        v.clip = Some(CoordinateClip::On);
    }
    clip(primitive, &map, 0, error, remaining)
}
