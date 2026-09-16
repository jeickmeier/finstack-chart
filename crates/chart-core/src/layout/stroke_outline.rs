//! Optional portable stroke expansion, shared by every renderer and inspection.
//! Same-winding component contours paint their union once, including translucent strokes.
use crate::{
    ChartResult, Diagnostic, DiagnosticCode, Point,
    grammar::{LineEnd, LineJoin},
    path::{Path, PathGeometry},
    scene::{FillRule, PathCommand, Primitive},
};
fn budget() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ResourceLimit,
        "Portable stroke outline exceeds the path budget.",
        "Reduce path complexity or increase the explicit path budget.",
    )
}
struct Builder {
    path: Path,
    left: usize,
}
impl Builder {
    fn charge(&mut self, n: usize) -> ChartResult<()> {
        self.left = self.left.checked_sub(n).ok_or_else(budget)?;
        Ok(())
    }
    fn polygon(&mut self, mut points: Vec<Point>) -> ChartResult<()> {
        if points.len() < 3 {
            return Ok(());
        }
        self.charge(points.len() + 1)?;
        let origin = points[0];
        let area = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(a, b)| {
                (a.x() - origin.x()) * (b.y() - origin.y())
                    - (b.x() - origin.x()) * (a.y() - origin.y())
            })
            .sum::<f64>();
        if area < 0. {
            points.reverse();
        }
        self.path.move_to(points[0].x(), points[0].y())?;
        for p in &points[1..] {
            self.path.line_to(p.x(), p.y())?;
        }
        self.path.close_path()
    }
    fn circle(&mut self, p: Point, r: f64) -> ChartResult<()> {
        // Circular cubic error is O(r * angle^6); this conservative bound is
        // over fifty times the quarter-circle radial error. Keep destination
        // error below 0.01 even for unusually large authored stroke widths.
        let angle = libm::pow(10. / r, 1. / 6.).min(std::f64::consts::FRAC_PI_2);
        let count = libm::ceil(std::f64::consts::TAU / angle) as usize;
        self.charge(count.checked_add(2).ok_or_else(budget)?)?;
        let step = std::f64::consts::TAU / count as f64;
        let k = 4. / 3. * libm::tan(step / 4.);
        self.path.move_to(p.x() + r, p.y())?;
        for i in 0..count {
            let a = i as f64 * step;
            let z = (i + 1) as f64 * step;
            let (ca, sa) = (libm::cos(a), libm::sin(a));
            let (cz, sz) = if i + 1 == count {
                (1., 0.)
            } else {
                (libm::cos(z), libm::sin(z))
            };
            self.path.bezier_curve_to(
                p.x() + r * (ca - k * sa),
                p.y() + r * (sa + k * ca),
                p.x() + r * (cz + k * sz),
                p.y() + r * (sz - k * cz),
                p.x() + r * cz,
                p.y() + r * sz,
            )?;
        }
        self.path.close_path()
    }
}
/// Deterministic adaptive circle geometry shared by distribution dots and stroke caps.
pub(super) fn circle(center: Point, radius: f64, max_commands: usize) -> ChartResult<PathGeometry> {
    let mut builder = Builder {
        path: Path::new(),
        left: max_commands,
    };
    builder.circle(center, radius)?;
    Ok(builder.path.geometry())
}

fn offset(p: Point, x: f64, y: f64) -> ChartResult<Point> {
    Point::new(p.x() + x, p.y() + y)
}
fn direction(a: Point, b: Point) -> Option<[f64; 2]> {
    let x = b.x() - a.x();
    let y = b.y() - a.y();
    let n = libm::hypot(x, y);
    (n > 0.).then(|| [x / n, y / n])
}
fn cap(b: &mut Builder, p: Point, d: [f64; 2], half: f64, end: LineEnd) -> ChartResult<()> {
    match end {
        LineEnd::Butt => Ok(()),
        LineEnd::Round => b.circle(p, half),
        LineEnd::Square => {
            let n = [-d[1] * half, d[0] * half];
            let extension = [d[0] * half, d[1] * half];
            b.polygon(vec![
                offset(p, n[0], n[1])?,
                offset(p, -n[0], -n[1])?,
                offset(p, -n[0] + extension[0], -n[1] + extension[1])?,
                offset(p, n[0] + extension[0], n[1] + extension[1])?,
            ])
        }
    }
}
fn join(
    b: &mut Builder,
    p: Point,
    a: [f64; 2],
    c: [f64; 2],
    half: f64,
    kind: LineJoin,
) -> ChartResult<()> {
    let cross = a[0] * c[1] - a[1] * c[0];
    if cross.abs() < 1e-14 {
        return if a[0] * c[0] + a[1] * c[1] < 0. && kind == LineJoin::Round {
            b.circle(p, half)
        } else {
            Ok(())
        };
    }
    if kind == LineJoin::Round {
        return b.circle(p, half);
    }
    let side = if cross > 0. { -1. } else { 1. };
    let n1 = [-a[1] * side, a[0] * side];
    let n2 = [-c[1] * side, c[0] * side];
    let p1 = offset(p, n1[0] * half, n1[1] * half)?;
    let p2 = offset(p, n2[0] * half, n2[1] * half)?;
    let denominator = 1. + a[0] * c[0] + a[1] * c[1];
    if kind == LineJoin::Miter && denominator > 0. {
        let x = (n1[0] + n2[0]) * half / denominator;
        let y = (n1[1] + n2[1]) * half / denominator;
        if libm::hypot(x, y) <= 10. * half {
            return b.polygon(vec![p, p1, offset(p, x, y)?, p2]);
        }
    }
    b.polygon(vec![p, p1, p2])
}
/// Expand only explicitly requested controls. Existing unchecked/default primitives bypass it.
pub(super) fn expand(
    primitive: Primitive,
    targets: usize,
    end: Option<LineEnd>,
    corner: Option<LineJoin>,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    if end.is_none() && corner.is_none() {
        return Ok(vec![primitive]);
    }
    let (geometry, stroke, dashes, anchors, fill) = match &primitive {
        Primitive::Rule { from, to, stroke } => {
            let commands = vec![PathCommand::MoveTo(*from), PathCommand::LineTo(*to)];
            (
                PathGeometry::from_beziers(&commands)?,
                *stroke,
                vec![],
                super::project::command_anchors(&commands, targets),
                false,
            )
        }
        Primitive::Path { commands, stroke }
        | Primitive::DashedPath {
            commands, stroke, ..
        } => (
            PathGeometry::from_beziers(commands)?,
            *stroke,
            if let Primitive::DashedPath { dashes, .. } = &primitive {
                dashes.clone()
            } else {
                vec![]
            },
            super::project::command_anchors(commands, targets),
            false,
        ),
        Primitive::ShapePath {
            geometry,
            stroke: Some(stroke),
            dashes,
            anchors,
            fill,
            ..
        } => (
            geometry.clone(),
            *stroke,
            dashes.clone(),
            anchors.clone(),
            fill.is_some(),
        ),
        Primitive::VectorPath {
            geometry,
            stroke: Some(stroke),
            dashes,
            fill,
        } => (
            geometry.clone(),
            *stroke,
            dashes.clone(),
            super::project::command_anchors(&geometry.lower(0.01, *remaining)?, targets),
            fill.is_some(),
        ),
        _ => return Ok(vec![primitive]),
    };
    if stroke.color.alpha == 0 {
        return Ok(vec![primitive]);
    }
    if !stroke.width.is_finite() || stroke.width <= 0. {
        return Err(budget());
    }
    let tolerance = (stroke.width * 0.01).clamp(0.0001, 0.01);
    let geometry = if dashes.is_empty() {
        geometry
    } else {
        PathGeometry::from_beziers(&geometry.dashed(&dashes, tolerance, *remaining)?)?
    };
    let flat = geometry.flatten(tolerance, *remaining)?;
    let mut builder = Builder {
        path: Path::new(),
        left: *remaining,
    };
    let half = stroke.width / 2.;
    let end = end.unwrap_or_default();
    let corner = corner.unwrap_or_default();
    for sub in flat.subpaths {
        let had_segment = sub.points.len() > 1;
        let mut points = sub.points;
        points.dedup();
        if sub.closed && points.len() > 1 && points.first() == points.last() {
            points.pop();
        }
        if points.len() < 2 {
            if had_segment
                && !sub.closed
                && let Some(p) = points.first()
            {
                cap(&mut builder, *p, [1., 0.], half, end)?;
                if end == LineEnd::Square {
                    cap(&mut builder, *p, [-1., 0.], half, end)?;
                }
            }
            continue;
        }
        let count = if sub.closed {
            points.len()
        } else {
            points.len() - 1
        };
        let mut directions = Vec::with_capacity(count);
        for i in 0..count {
            let a = points[i];
            let c = points[(i + 1) % points.len()];
            let d = direction(a, c).expect("distinct consecutive stroke vertices");
            directions.push(d);
            let n = [-d[1] * half, d[0] * half];
            builder.polygon(vec![
                offset(a, n[0], n[1])?,
                offset(c, n[0], n[1])?,
                offset(c, -n[0], -n[1])?,
                offset(a, -n[0], -n[1])?,
            ])?;
        }
        if sub.closed {
            for i in 0..points.len() {
                join(
                    &mut builder,
                    points[i],
                    directions[(i + count - 1) % count],
                    directions[i],
                    half,
                    corner,
                )?;
            }
        } else {
            for i in 1..points.len() - 1 {
                join(
                    &mut builder,
                    points[i],
                    directions[i - 1],
                    directions[i],
                    half,
                    corner,
                )?;
            }
            cap(
                &mut builder,
                points[0],
                [-directions[0][0], -directions[0][1]],
                half,
                end,
            )?;
            cap(
                &mut builder,
                *points.last().unwrap(),
                *directions.last().unwrap(),
                half,
                end,
            )?;
        }
    }
    let mut result = vec![];
    if fill {
        let mut original = primitive;
        match &mut original {
            Primitive::ShapePath { stroke, dashes, .. }
            | Primitive::VectorPath { stroke, dashes, .. } => {
                *stroke = None;
                dashes.clear();
            }
            _ => {}
        }
        result.push(original);
    }
    let geometry = builder.path.geometry();
    builder.charge(anchors.len())?;
    *remaining = builder.left;
    if geometry.has_segments() {
        result.push(Primitive::ShapePath {
            geometry,
            fill: Some(stroke.color),
            stroke: None,
            dashes: vec![],
            anchors,
            fill_rule: FillRule::NonZero,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Color, Stroke};
    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }
    fn path(points: &[(f64, f64)], dashes: Vec<f64>) -> Primitive {
        let commands = points
            .iter()
            .enumerate()
            .map(|(i, (x, y))| {
                if i == 0 {
                    PathCommand::MoveTo(p(*x, *y))
                } else {
                    PathCommand::LineTo(p(*x, *y))
                }
            })
            .collect();
        Primitive::DashedPath {
            commands,
            stroke: Stroke {
                color: Color {
                    red: 200,
                    green: 30,
                    blue: 20,
                    alpha: 128,
                },
                width: 2.,
            },
            dashes,
        }
    }
    fn outline(
        points: &[(f64, f64)],
        end: LineEnd,
        join: LineJoin,
        dashes: Vec<f64>,
    ) -> PathGeometry {
        let mut budget = 10000;
        let mut result = expand(
            path(points, dashes),
            points.len(),
            Some(end),
            Some(join),
            &mut budget,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        let Primitive::ShapePath {
            geometry,
            fill,
            stroke,
            anchors,
            fill_rule,
            ..
        } = result.remove(0)
        else {
            panic!("filled outline")
        };
        assert_eq!(fill.unwrap().alpha, 128);
        assert!(stroke.is_none());
        assert_eq!(anchors.len(), points.len());
        assert_eq!(fill_rule, FillRule::NonZero);
        geometry
    }
    fn contains(g: &PathGeometry, x: f64, y: f64) -> bool {
        g.flatten(0.0001, 10000)
            .unwrap()
            .contains(p(x, y), true, None)
    }
    #[test]
    fn portable_caps_preserve_union_winding_and_distinct_endpoint_shapes() {
        for end in [LineEnd::Butt, LineEnd::Round, LineEnd::Square] {
            let g = outline(&[(0., 0.), (10., 0.)], end, LineJoin::Miter, vec![]);
            assert!(contains(&g, 0.5, 0.5)); // cap/body overlap must never cancel.
            assert_eq!(contains(&g, -0.5, 0.), end != LineEnd::Butt);
            assert_eq!(contains(&g, -0.9, 0.9), end == LineEnd::Square);
            assert!(!contains(&g, -1.1, 0.));
        }
    }
    #[test]
    fn portable_joins_union_both_turns_and_bound_miters() {
        for sign in [-1., 1.] {
            for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
                let g = outline(
                    &[(0., 0.), (10., 0.), (10., sign * 10.)],
                    LineEnd::Butt,
                    join,
                    vec![],
                );
                assert!(contains(&g, 9.5, sign * 0.5)); // two segment rectangles overlap here.
                assert!(contains(&g, 10.3, -sign * 0.3)); // join/segment union.
                assert_eq!(contains(&g, 10.8, -sign * 0.8), join == LineJoin::Miter);
            }
        }
        let sharp = outline(
            &[(0., 0.), (10., 0.), (0., 0.1)],
            LineEnd::Butt,
            LineJoin::Miter,
            vec![],
        );
        assert!(!contains(&sharp, 25., -0.5)); // limit10 falls back to bevel.
    }
    #[test]
    fn portable_dash_caps_extend_each_run_but_retain_gaps() {
        for end in [LineEnd::Butt, LineEnd::Round, LineEnd::Square] {
            let g = outline(&[(0., 0.), (20., 0.)], end, LineJoin::Miter, vec![4., 4.]);
            assert_eq!(contains(&g, 4.5, 0.), end != LineEnd::Butt);
            assert!(!contains(&g, 6., 0.));
            assert!(contains(&g, 8.5, 0.5));
        }
    }
    #[test]
    fn portable_closed_rectangle_uses_joins_without_open_caps() {
        let mut path = Path::new();
        path.rect(0., 0., 10., 10.).unwrap();
        let primitive = Primitive::ShapePath {
            geometry: path.geometry(),
            fill: None,
            stroke: Some(Stroke {
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 128,
                },
                width: 2.,
            }),
            dashes: vec![],
            anchors: vec![p(5., 5.)],
            fill_rule: FillRule::NonZero,
        };
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            let a = expand(
                primitive.clone(),
                1,
                Some(LineEnd::Square),
                Some(join),
                &mut 10000,
            )
            .unwrap();
            let b = expand(
                primitive.clone(),
                1,
                Some(LineEnd::Butt),
                Some(join),
                &mut 10000,
            )
            .unwrap();
            assert_eq!(a, b);
            let Primitive::ShapePath { geometry, .. } = &a[0] else {
                panic!()
            };
            assert_eq!(contains(geometry, 10.8, -0.8), join == LineJoin::Miter);
            assert!(contains(geometry, 9.5, 0.5));
            assert!(!contains(geometry, 5., 5.));
        }
    }
    #[test]
    fn portable_circle_error_is_bounded_for_small_and_large_radii() {
        for r in [0.001, 1., 1e6, 1e9] {
            let mut builder = Builder {
                path: Path::new(),
                left: 10000,
            };
            builder.circle(p(0., 0.), r).unwrap();
            let geometry = builder.path.geometry();
            assert_eq!(10000 - builder.left, geometry.commands().len());
            let mut start = [r, 0.];
            for command in geometry.commands() {
                if let crate::path::Command::CubicTo(v) = command {
                    for i in 0..=100 {
                        let t = i as f64 / 100.;
                        let q = 1. - t;
                        let x = q * q * q * start[0]
                            + 3. * q * q * t * v[0]
                            + 3. * q * t * t * v[2]
                            + t * t * t * v[4];
                        let y = q * q * q * start[1]
                            + 3. * q * q * t * v[1]
                            + 3. * q * t * t * v[3]
                            + t * t * t * v[5];
                        assert!((libm::hypot(x, y) - r).abs() < 0.01, "radius {r}");
                    }
                    start = [v[4], v[5]];
                }
            }
        }
    }
    #[test]
    fn portable_controls_are_optional_and_charge_generated_complexity() {
        let original = path(&[(0., 0.), (10., 0.)], vec![]);
        let mut budget = 0;
        assert_eq!(
            expand(original.clone(), 2, None, None, &mut budget).unwrap(),
            vec![original.clone()]
        );
        assert!(expand(original, 2, Some(LineEnd::Round), None, &mut budget).is_err());
    }
}
