//! One bounded destination subdivision kernel for coordinate marks and guides.
use super::coordinate_map::CoordinateMap;
use crate::path::{Affine, Path, PathGeometry};
use crate::scene::PathCommand;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
fn budget() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ResourceLimit,
        "Coordinate path subdivision exceeds its explicit work or depth budget.",
        "Reduce the path or coordinate span, or raise the caller's path budget.",
    )
}
fn charge(left: &mut usize) -> ChartResult<()> {
    *left = left.checked_sub(1).ok_or_else(budget)?;
    Ok(())
}
fn distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let length = dx * dx + dy * dy;
    let t = if length == 0. {
        0.
    } else {
        ((p.x() - a.x()) * dx + (p.y() - a.y()) * dy) / length
    }
    .clamp(0., 1.);
    libm::hypot(p.x() - a.x() - t * dx, p.y() - a.y() - t * dy)
}
fn corners(rect: Rect) -> [Point; 4] {
    [
        rect.origin(),
        Point::new(rect.max_x(), rect.origin().y()).expect("rectangle"),
        Point::new(rect.max_x(), rect.max_y()).expect("rectangle"),
        Point::new(rect.origin().x(), rect.max_y()).expect("rectangle"),
    ]
}
fn split(points: &[Point]) -> (Vec<Point>, Vec<Point>) {
    let mut work = points.to_vec();
    let mut left = vec![work[0]];
    let mut right = vec![work[work.len() - 1]];
    while work.len() > 1 {
        work = work
            .windows(2)
            .map(|p| {
                Point::new(p[0].x().midpoint(p[1].x()), p[0].y().midpoint(p[1].y()))
                    .expect("finite midpoint")
            })
            .collect();
        left.push(work[0]);
        right.push(work[work.len() - 1]);
    }
    right.reverse();
    (left, right)
}
struct Subdivision<'a> {
    map: &'a CoordinateMap,
    path: Path,
    error: f64,
    source_error: f64,
    remaining: &'a mut usize,
    opened: bool,
}
impl Subdivision<'_> {
    // Geographic kernels use nested numerical deviation estimates, not analytic
    // interval enclosures. Angular/source-span guards prevent coarse sampling
    // from aliasing a projection branch; unresolved continuous pieces fail the
    // existing explicit depth/work budget.
    fn geographic_segment(&mut self, points: &[Point], depth: u8) -> ChartResult<()> {
        let mut pieces = vec![points.to_vec()];
        for _ in 0..3 {
            pieces = pieces
                .into_iter()
                .flat_map(|p| {
                    let (a, b) = split(&p);
                    [a, b]
                })
                .collect();
        }
        let samples = std::iter::once(points[0])
            .chain(pieces.iter().map(|p| *p.last().expect("segment")))
            .collect::<Vec<_>>();
        let mapped = samples
            .iter()
            .map(|p| self.map.project(*p))
            .collect::<ChartResult<Vec<_>>>()?;
        let mut low = [f64::INFINITY; 2];
        let mut high = [f64::NEG_INFINITY; 2];
        for p in points {
            for (i, v) in self.map.source_values(*p).into_iter().enumerate() {
                low[i] = low[i].min(v);
                high[i] = high[i].max(v);
            }
        }
        let angular = self
            .map
            .geographic
            .as_ref()
            .expect("geography")
            .angular_step();
        let small = (0..2).all(|i| {
            let step = ((self.map.view[i][1] - self.map.view[i][0]).abs() / 100.).max(f64::EPSILON);
            high[i] - low[i] <= angular.map_or(step, |v| step.min(v))
        });
        if small && mapped.iter().all(Option::is_none) {
            self.opened = false;
            return Ok(());
        }
        if small
            && let (Some(a), Some(b)) = (mapped[0], mapped[8])
            && mapped
                .iter()
                .all(|p| p.is_some_and(|p| distance(p, a, b) <= self.error / 4.))
        {
            if !self.opened {
                self.path.move_to(a.x(), a.y())?;
                self.opened = true;
            }
            return self.path.line_to(b.x(), b.y());
        }
        if depth >= 32 {
            // Domain/seam crossings have no finite connecting segment. Preserve a
            // disconnected path instead of joining opposite projection boundaries.
            if mapped.iter().any(Option::is_none) || mapped.windows(2).any(|p|matches!(p,[Some(a),Some(b)] if libm::hypot(a.x()-b.x(),a.y()-b.y())>libm::hypot(self.map.plot.width(),self.map.plot.height())/2.)){self.opened=false;return Ok(());}
            return Err(budget());
        }
        let (a, b) = split(points);
        self.segment(&a, depth + 1)?;
        self.segment(&b, depth + 1)
    }
    fn segment(&mut self, points: &[Point], depth: u8) -> ChartResult<()> {
        charge(self.remaining)?;
        if self.map.geographic.is_some() {
            return self.geographic_segment(points, depth);
        }
        let start = self.map.project(points[0])?;
        let end = self.map.project(points[points.len() - 1])?;
        // Arc lowering has a source-space enclosure error. Carry it into the
        // transformed hull so the final chord bound covers the analytic arc,
        // not merely the approximating cubic.
        let mut enclosure = points.to_vec();
        if self.source_error > 0. {
            for p in points {
                for dx in [-self.source_error, self.source_error] {
                    for dy in [-self.source_error, self.source_error] {
                        enclosure.push(Point::new(p.x() + dx, p.y() + dy)?);
                    }
                }
            }
        }
        let hull = self.map.hull(&enclosure)?;
        if let (Some(a), Some(b), Some(hull)) = (start, end, hull) {
            if corners(hull)
                .iter()
                .all(|p| distance(*p, a, b) <= self.error)
            {
                if !self.opened {
                    self.path.move_to(a.x(), a.y())?;
                    self.opened = true;
                }
                return self.path.line_to(b.x(), b.y());
            }
        } else if start.is_none()
            && end.is_none()
            && points
                .iter()
                .all(|p| self.map.project(*p).is_ok_and(|p| p.is_none()))
        {
            self.opened = false;
            return Ok(());
        }
        if depth >= 32 {
            return Err(budget());
        }
        let (left, right) = split(points);
        self.segment(&left, depth + 1)?;
        self.segment(&right, depth + 1)
    }
}
pub(super) fn project(
    geometry: &PathGeometry,
    map: &CoordinateMap,
    error: f64,
    remaining: &mut usize,
) -> ChartResult<PathGeometry> {
    if !error.is_finite() || error <= 0. {
        return Err(Diagnostic::error(
            DiagnosticCode::Validation,
            "Coordinate path error must be finite and positive.",
            "Supply a positive destination-space error bound.",
        ));
    }
    if map.geographic.as_ref().is_some_and(|g| g.vertices_only) {
        let mut output = Path::new();
        let mut open = false;
        let mut complete = false;
        for command in geometry.lower(error / 64., *remaining)? {
            charge(remaining)?;
            match command {
                PathCommand::MoveTo(p) => {
                    open = false;
                    complete = true;
                    if let Some(q) = map.project(p)? {
                        output.move_to(q.x(), q.y())?;
                        open = true;
                    } else {
                        complete = false;
                    }
                }
                PathCommand::LineTo(p) => {
                    if let Some(q) = map.project(p)? {
                        if open {
                            output.line_to(q.x(), q.y())?;
                        } else {
                            output.move_to(q.x(), q.y())?;
                            open = true;
                        }
                    } else {
                        open = false;
                        complete = false;
                    }
                }
                PathCommand::Close => {
                    if open && complete {
                        output.close_path()?;
                    }
                    open = false;
                }
                _ => {
                    return Err(Diagnostic::error(
                        DiagnosticCode::Validation,
                        "Geographic feature geometry must contain straight supplied vertices.",
                        "Use the ordinary overlay path API for curves.",
                    ));
                }
            }
        }
        return Ok(output.geometry());
    }
    if map.affine() {
        let origin = map
            .project(Point::new(0., 0.)?)?
            .expect("finite affine map");
        let x = map
            .project(Point::new(1., 0.)?)?
            .expect("finite affine map");
        let y = map
            .project(Point::new(0., 1.)?)?
            .expect("finite affine map");
        let result = geometry.transformed(
            Affine::new([
                x.x() - origin.x(),
                x.y() - origin.y(),
                y.x() - origin.x(),
                y.y() - origin.y(),
                origin.x(),
                origin.y(),
            ])?,
            error,
            *remaining,
        )?;
        *remaining = remaining
            .checked_sub(result.commands().len())
            .ok_or_else(budget)?;
        return Ok(result);
    }
    let source_error = if geometry
        .commands()
        .iter()
        .any(|c| matches!(c, crate::path::Command::Arc { .. }))
    {
        error / 64.
    } else {
        0.
    };
    let commands = geometry.lower(error / 64., *remaining)?;
    let mut state = Subdivision {
        map,
        path: Path::new(),
        error,
        source_error,
        remaining,
        opened: false,
    };
    let mut previous = None;
    let mut first = None;
    for command in commands {
        match command {
            PathCommand::MoveTo(p) => {
                previous = Some(p);
                first = Some(p);
                state.opened = false;
            }
            PathCommand::LineTo(p) => {
                if let Some(a) = previous {
                    state.segment(&[a, p], 0)?;
                }
                previous = Some(p);
            }
            PathCommand::QuadraticTo(c, p) => {
                if let Some(a) = previous {
                    state.segment(&[a, c, p], 0)?;
                }
                previous = Some(p);
            }
            PathCommand::CubicTo(c, d, p) => {
                if let Some(a) = previous {
                    state.segment(&[a, c, d, p], 0)?;
                }
                previous = Some(p);
            }
            PathCommand::Close => {
                if let (Some(a), Some(b)) = (previous, first) {
                    state.segment(&[a, b], 0)?;
                    if state.opened {
                        state.path.close_path()?;
                    }
                }
                previous = first;
            }
        }
    }
    Ok(state.path.geometry())
}

/// Shared maximum destination subdivision error: one tenth of a logical pixel.
pub(super) fn tolerance(request: &super::LayoutRequest) -> f64 {
    match request.units {
        crate::services::Units::Points => 0.075,
        crate::services::Units::LogicalPixels => 0.1,
    }
}

#[cfg(test)]
mod tests {
    use super::super::coordinate_map::CoordinateDomain;
    use super::*;
    use crate::grammar::{CoordinateSpec, RadialCoordinate, RadialMode};
    fn radial(turns: f64) -> CoordinateMap {
        CoordinateMap::new(
            CoordinateSpec::Radial(RadialCoordinate {
                mode: RadialMode::Radial,
                end: Some(std::f64::consts::TAU * turns),
                expand: false,
                ..Default::default()
            }),
            [CoordinateDomain {
                domain: [0., 1.],
                viewport: [0., 1.],
                range: [0., 1.],
            }; 2],
            [[0., 1.]; 2],
            Rect::new(0., 0., 400., 400.).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn full_turn_and_multiple_turns_cannot_alias_to_empty_chords() {
        for turns in [1., 4.] {
            let map = radial(turns);
            let mut line = Path::new();
            line.move_to(0., 1.).unwrap();
            line.line_to(1., 1.).unwrap();
            let result = project(&line.geometry(), &map, 0.1, &mut 100000).unwrap();
            let flat = result.flatten(0.01, 100000).unwrap();
            let points = &flat.subpaths[0].points;
            assert!(points.len() > 100);
            let length = points
                .windows(2)
                .map(|v| libm::hypot(v[1].x() - v[0].x(), v[1].y() - v[0].y()))
                .sum::<f64>();
            let expected = std::f64::consts::TAU * 160. * turns;
            assert!((length - expected).abs() / expected < 0.001);
            assert!(
                points
                    .iter()
                    .all(|p| (libm::hypot(p.x() - 200., p.y() - 200.) - 160.).abs() < 1e-10)
            );
        }
    }
    #[test]
    fn exhausted_subdivision_is_explicit_and_center_has_nonunique_inverse() {
        let map = radial(1.);
        let mut line = Path::new();
        line.move_to(0., 1.).unwrap();
        line.line_to(1., 1.).unwrap();
        assert!(project(&line.geometry(), &map, 0.1, &mut 2).is_err());
        let center = Point::new(200., 200.).unwrap();
        assert!(map.contains(center));
        assert!(map.inverse(center, 8).unwrap().is_none());
    }
    #[test]
    fn geographic_refinement_matches_dense_independent_mercator_curve_and_budget() {
        use crate::grammar::{GeoCrs, GeoProjectionSelection, GeographicCoordinate};
        let view = [[-60., 60.], [0., 70.]];
        let axes = view.map(|v| CoordinateDomain {
            domain: v,
            viewport: v,
            range: v,
        });
        let map = CoordinateMap::new(
            CoordinateSpec::Geographic(GeographicCoordinate {
                projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
                default_crs: Some(GeoCrs::Wgs84),
                ..Default::default()
            }),
            axes,
            view,
            Rect::new(0., 0., 600., 400.).unwrap(),
        )
        .unwrap();
        let mut line = Path::new();
        line.move_to(-50., 10.).unwrap();
        line.line_to(50., 60.).unwrap();
        let projected = project(&line.geometry(), &map, 0.075, &mut 100000).unwrap();
        let flat = projected.flatten(0.001, 100000).unwrap();
        let segments = flat
            .subpaths
            .iter()
            .flat_map(|p| p.points.windows(2))
            .collect::<Vec<_>>();
        assert!(segments.len() > 100);
        let mercator = |lat: f64| {
            libm::log(libm::tan(
                std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.,
            ))
        };
        let top = mercator(70.);
        for i in 0..=10000 {
            let t = i as f64 / 10000.;
            let p = Point::new(
                ((-50. + 100. * t) + 60.) / 120. * 600.,
                (1. - mercator(10. + 50. * t) / top) * 400.,
            )
            .unwrap();
            let error = segments
                .iter()
                .map(|v| distance(p, v[0], v[1]))
                .fold(f64::INFINITY, f64::min);
            assert!(error <= 0.075, "dense deviation {error}");
        }
        assert_eq!(
            project(&line.geometry(), &map, 0.075, &mut 4)
                .unwrap_err()
                .code,
            DiagnosticCode::ResourceLimit
        );
    }
}
