//! Planar geographic label locations, with multipart and hole identity preserved.
use super::*;
use crate::{ChartResult, Point};
fn contours(g: &PreparedGeometry) -> Option<&[Vec<Point>]> {
    if let PreparedGeometry::Recipe(recipe) = g
        && let PreparedRecipe::Surface(PreparedSurface::Polygon { contours, .. }) = recipe.as_ref()
    {
        Some(contours)
    } else {
        None
    }
}
fn ring_moment(ring: &[Point]) -> (f64, f64, f64) {
    let (mut area, mut x, mut y) = (0., 0., 0.);
    for edge in ring.windows(2) {
        let a = edge[0];
        let b = edge[1];
        let cross = a.x() * b.y() - b.x() * a.y();
        area += cross;
        x += (a.x() + b.x()) * cross;
        y += (a.y() + b.y()) * cross;
    }
    if area == 0. {
        (0., 0., 0.)
    } else {
        (area.abs() / 2., x / (3. * area), y / (3. * area))
    }
}
fn polygon_moment(rings: &[Vec<Point>]) -> (f64, f64, f64) {
    let (mut total, mut x, mut y) = (0., 0., 0.);
    for (i, ring) in rings.iter().enumerate() {
        let (a, cx, cy) = ring_moment(ring);
        let a = if i == 0 { a } else { -a };
        total += a;
        x += a * cx;
        y += a * cy;
    }
    (total, x, y)
}
/// Widest interior interval on a scanline avoiding vertices; holes subtract their intervals.
fn surface(rings: &[Vec<Point>]) -> Option<Point> {
    let exterior = rings.first()?;
    let min = exterior.iter().map(|p| p.y()).reduce(f64::min)?;
    let max = exterior.iter().map(|p| p.y()).reduce(f64::max)?;
    let centre = min / 2. + max / 2.;
    let mut below = min;
    let mut above = max;
    for p in rings.iter().flatten() {
        if p.y() <= centre {
            below = below.max(p.y());
        } else {
            above = above.min(p.y());
        }
    }
    let y = below / 2. + above / 2.;
    let mut cuts = vec![];
    for ring in rings {
        for edge in ring.windows(2) {
            let a = edge[0];
            let b = edge[1];
            if (a.y() > y) != (b.y() > y) {
                cuts.push(a.x() + (y - a.y()) / (b.y() - a.y()) * (b.x() - a.x()));
            }
        }
    }
    cuts.sort_by(f64::total_cmp);
    cuts.chunks_exact(2)
        .fold(None, |best: Option<&[f64]>, candidate| {
            if best.is_none_or(|v| candidate[1] - candidate[0] > v[1] - v[0]) {
                Some(candidate)
            } else {
                best
            }
        })
        .and_then(|v| Point::new(v[0] / 2. + v[1] / 2., y).ok())
}
pub(super) fn anchor(
    parts: &[PreparedGeometry],
    operation: GeoOperation,
) -> ChartResult<Option<Point>> {
    let polygons: Vec<_> = parts.iter().filter_map(contours).collect();
    if !polygons.is_empty() {
        if operation == GeoOperation::PointOnSurface {
            return Ok(polygons
                .iter()
                .max_by(|a, b| polygon_moment(a).0.total_cmp(&polygon_moment(b).0))
                .and_then(|rings| surface(rings)));
        }
        let (mut total, mut x, mut y) = (0., 0., 0.);
        for rings in polygons {
            let (a, mx, my) = polygon_moment(rings);
            total += a;
            x += mx;
            y += my;
        }
        if total != 0. {
            return Point::new(x / total, y / total).map(Some);
        }
    }
    let mut lines: Vec<&[Point]> = parts
        .iter()
        .filter_map(|g| {
            if let PreparedGeometry::LineRun(p) = g {
                Some(p.as_slice())
            } else {
                None
            }
        })
        .collect();
    for g in parts {
        if let Some(rings) = contours(g) {
            lines.extend(rings.iter().map(Vec::as_slice));
        }
    }
    if !lines.is_empty() {
        if operation == GeoOperation::PointOnSurface {
            let Some(centre) = anchor(parts, GeoOperation::Centroid)? else {
                return Ok(None);
            };
            let mut candidates: Vec<Point> = lines
                .iter()
                .filter(|p| p.len() > 2)
                .flat_map(|p| p[1..p.len() - 1].iter().copied())
                .collect();
            if candidates.is_empty() {
                candidates = lines.iter().flat_map(|p| p.iter().copied()).collect();
            }
            return Ok(nearest(&candidates, centre));
        }
        let (mut weight, mut x, mut y) = (0., 0., 0.);
        for line in lines {
            for edge in line.windows(2) {
                let a = edge[0];
                let b = edge[1];
                let w = libm::hypot(b.x() - a.x(), b.y() - a.y());
                weight += w;
                x += (a.x() / 2. + b.x() / 2.) * w;
                y += (a.y() / 2. + b.y() / 2.) * w;
            }
        }
        if weight != 0. {
            return Point::new(x / weight, y / weight).map(Some);
        }
    }
    let points: Vec<_> = parts
        .iter()
        .filter_map(|g| {
            if let PreparedGeometry::Point(p) = g {
                Some(*p)
            } else {
                None
            }
        })
        .collect();
    if operation == GeoOperation::PointOnSurface {
        let Some(centre) = anchor(parts, GeoOperation::Centroid)? else {
            return Ok(None);
        };
        return Ok(nearest(&points, centre));
    }
    if points.is_empty() {
        return Ok(None);
    }
    let n = points.len() as f64;
    Point::new(
        points.iter().map(|p| p.x() / n).sum(),
        points.iter().map(|p| p.y() / n).sum(),
    )
    .map(Some)
}

fn nearest(points: &[Point], centre: Point) -> Option<Point> {
    points.iter().copied().fold(None, |best: Option<Point>, p| {
        let distance = |q: Point| libm::hypot(q.x() - centre.x(), q.y() - centre.y());
        if best.is_none_or(|b| distance(p) < distance(b)) {
            Some(p)
        } else {
            best
        }
    })
}
