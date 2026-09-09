//! Bounded geometric flattening shared by inspection; no source targets are synthesized.
use super::{PathGeometry, domain, require};
use crate::{ChartResult, Point, scene::PathCommand};

/// One independently opened polyline; fill closes it implicitly, stroke closes it explicitly.
#[derive(Clone, Debug)]
pub struct FlatSubpath {
    /// Destination vertices, including any repeated authored endpoint.
    pub points: Vec<Point>,
    /// Whether a close command joins the final vertex to the first.
    pub closed: bool,
}
/// Bounded polyline approximation, retaining subpaths and nonzero winding semantics.
#[derive(Clone, Debug)]
pub struct FlattenedPath {
    /// Subpaths in authored order.
    pub subpaths: Vec<FlatSubpath>,
    /// Maximum Euclidean deviation from the input path, in destination units.
    pub max_error: f64,
}
impl PathGeometry {
    /// Flatten arcs and Béziers with an explicit geometric error and output-vertex budget.
    /// De Casteljau subdivision accepts only a control hull within tolerance of its chord.
    pub fn flatten(&self, max_error: f64, max_vertices: usize) -> ChartResult<FlattenedPath> {
        if !max_error.is_finite() || max_error <= 0. {
            return Err(domain("Flattening error must be finite and positive"));
        }
        let commands = self.lower(max_error / 2., max_vertices)?;
        let mut result = FlattenedPath {
            subpaths: vec![],
            max_error,
        };
        let mut remaining = max_vertices;
        for command in commands {
            if let PathCommand::MoveTo(p) = command {
                push_subpath(&mut result, p, &mut remaining)?;
                continue;
            }
            let Some(sub) = result.subpaths.last_mut() else {
                continue;
            };
            let start = *sub.points.last().expect("opened subpath");
            match command {
                PathCommand::LineTo(p) => push(&mut sub.points, p, &mut remaining)?,
                PathCommand::QuadraticTo(c, p) => flatten_curve(
                    &[start, c, p],
                    max_error / 2.,
                    &mut sub.points,
                    &mut remaining,
                    0,
                )?,
                PathCommand::CubicTo(c, d, p) => flatten_curve(
                    &[start, c, d, p],
                    max_error / 2.,
                    &mut sub.points,
                    &mut remaining,
                    0,
                )?,
                PathCommand::Close => sub.closed = true,
                PathCommand::MoveTo(_) => unreachable!(),
            }
        }
        for sub in &mut result.subpaths {
            sub.points.dedup();
            if sub.closed && sub.points.len() > 1 && sub.points.first() == sub.points.last() {
                sub.points.pop();
            }
        }
        Ok(result)
    }
}
fn push_subpath(out: &mut FlattenedPath, p: Point, remaining: &mut usize) -> ChartResult<()> {
    let mut points = vec![];
    push(&mut points, p, remaining)?;
    out.subpaths.push(FlatSubpath {
        points,
        closed: false,
    });
    Ok(())
}
fn push(out: &mut Vec<Point>, p: Point, remaining: &mut usize) -> ChartResult<()> {
    require(*remaining > 0, "Flattened path vertex limit exceeded")?;
    *remaining -= 1;
    out.push(p);
    Ok(())
}
fn midpoint(a: Point, b: Point) -> Point {
    Point::new(a.x().midpoint(b.x()), a.y().midpoint(b.y())).expect("finite midpoint")
}
fn flatten_curve(
    p: &[Point],
    error: f64,
    out: &mut Vec<Point>,
    remaining: &mut usize,
    depth: u8,
) -> ChartResult<()> {
    let last = p[p.len() - 1];
    if p[1..p.len() - 1]
        .iter()
        .all(|c| distance(*c, p[0], last) <= error)
    {
        return push(out, last, remaining);
    }
    require(depth < 32, "Curve flattening subdivision depth exceeded")?;
    // Both halves must produce at least one endpoint, before recursive work begins.
    require(*remaining >= 2, "Flattened path vertex limit exceeded")?;
    let mut triangle = p.to_vec();
    let mut left = vec![p[0]];
    let mut right = vec![last];
    for n in (1..p.len()).rev() {
        for i in 0..n {
            triangle[i] = midpoint(triangle[i], triangle[i + 1]);
        }
        left.push(triangle[0]);
        right.push(triangle[n - 1]);
    }
    right.reverse();
    flatten_curve(&left, error, out, remaining, depth + 1)?;
    flatten_curve(&right, error, out, remaining, depth + 1)
}
fn distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let length = dx.hypot(dy);
    if length == 0. {
        return (p.x() - a.x()).hypot(p.y() - a.y());
    }
    let t = (((p.x() - a.x()) * (dx / length) + (p.y() - a.y()) * (dy / length)) / length)
        .clamp(0., 1.);
    (p.x() - (a.x() + dx * t)).hypot(p.y() - (a.y() + dy * t))
}
fn cross(a: Point, b: Point, p: Point) -> f64 {
    (b.x() - a.x()) * (p.y() - a.y()) - (b.y() - a.y()) * (p.x() - a.x())
}
fn winding(points: &[Point], p: Point) -> i64 {
    let mut value = 0;
    for (&a, &b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if a.y() <= p.y() {
            if b.y() > p.y() && cross(a, b, p) > 0. {
                value += 1;
            }
        } else if b.y() <= p.y() && cross(a, b, p) < 0. {
            value -= 1;
        }
    }
    value
}
fn stroke_segment(p: Point, a: Point, b: Point, w: f64) -> bool {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let length = dx.hypot(dy);
    if length == 0. {
        return false;
    }
    let x = (p.x() - a.x()) * (dx / length) + (p.y() - a.y()) * (dy / length);
    let y = (p.x() - a.x()) * (-dy / length) + (p.y() - a.y()) * (dx / length);
    x >= 0. && x <= length && y.abs() <= w
}
fn join(p: Point, a: Point, b: Point, c: Point, w: f64) -> bool {
    let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
    let (ex, ey) = (c.x() - b.x(), c.y() - b.y());
    let (d, e) = (dx.hypot(dy), ex.hypot(ey));
    if d == 0. || e == 0. {
        return false;
    }
    let (dx, dy, ex, ey) = (dx / d, dy / d, ex / e, ey / e);
    let det = dx * ey - dy * ex;
    if det == 0. {
        return false;
    }
    let sign = if det > 0. { -1. } else { 1. };
    let n = [-dy * sign, dx * sign];
    let m = [-ey * sign, ex * sign];
    let q = Point::new(b.x() + w * n[0], b.y() + w * n[1]);
    let r = Point::new(b.x() + w * m[0], b.y() + w * m[1]);
    let (Ok(q), Ok(r)) = (q, r) else {
        return false;
    };
    let divisor = 1. + dx * ex + dy * ey;
    let tip = Point::new(
        b.x() + w * (n[0] + m[0]) / divisor,
        b.y() + w * (n[1] + m[1]) / divisor,
    );
    let mut polygon = vec![b, q];
    if let Ok(tip) = tip
        && (tip.x() - b.x()).hypot(tip.y() - b.y()) <= 4. * w
    {
        polygon.push(tip);
    }
    polygon.push(r);
    winding(&polygon, p) != 0
}
impl FlattenedPath {
    /// Test nonzero fill and/or butt-cap, miter-join (limit 4) stroke of the approximation.
    /// Queries within max_error of a boundary have the declared flattening uncertainty.
    pub fn contains(&self, p: Point, fill: bool, stroke_width: Option<f64>) -> bool {
        if fill
            && self
                .subpaths
                .iter()
                .map(|s| winding(&s.points, p))
                .sum::<i64>()
                != 0
        {
            return true;
        }
        let Some(w) = stroke_width
            .filter(|w| w.is_finite() && *w > 0.)
            .map(|w| w / 2.)
        else {
            return false;
        };
        for sub in &self.subpaths {
            let points = &sub.points;
            let n = points.len();
            if n < 2 {
                continue;
            }
            let count = if sub.closed { n } else { n - 1 };
            for i in 0..count {
                if stroke_segment(p, points[i], points[(i + 1) % n], w) {
                    return true;
                }
            }
            let indices = if sub.closed { 0..n } else { 1..n - 1 };
            for i in indices {
                if join(
                    p,
                    points[(i + n - 1) % n],
                    points[i],
                    points[(i + 1) % n],
                    w,
                ) {
                    return true;
                }
            }
        }
        false
    }
}
