//! Circular endpoint arcs become cubic Hermite segments under a coordinate-error bound.
use super::{ChartResult, Command, PathGeometry, domain, finite, require};
use crate::{Point, Rect, scene::PathCommand};
use std::f64::consts::{FRAC_PI_2, SQRT_2, TAU};

/// Checked affine map `[a,b,c,d,e,f]`: `(a*x+c*y+e, b*x+d*y+f)`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Affine([f64; 6]);
impl Default for Affine {
    fn default() -> Self {
        Self([1., 0., 0., 1., 0., 0.])
    }
}
impl Affine {
    /// Validate finite coefficients; singular maps are permitted.
    pub fn new(values: [f64; 6]) -> ChartResult<Self> {
        finite(&values)?;
        Ok(Self(values))
    }
    /// Owned coefficient copy.
    pub fn coefficients(self) -> [f64; 6] {
        self.0
    }
    /// Post-multiply by `right`: the right map acts on a point first. Rejects overflow.
    pub fn concatenate(self, right: Self) -> ChartResult<Self> {
        let [a, b, c, d, e, f] = self.0;
        let [g, h, i, j, k, l] = right.0;
        Self::new([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ])
    }
    /// Map one finite point, rejecting arithmetic overflow.
    pub fn point(self, p: [f64; 2]) -> ChartResult<[f64; 2]> {
        let [a, b, c, d, e, f] = self.0;
        let q = [
            a.mul_add(p[0], c.mul_add(p[1], e)),
            b.mul_add(p[0], d.mul_add(p[1], f)),
        ];
        finite(&q)?;
        Ok(q)
    }
    fn norm(self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        a.hypot(b).hypot(c.hypot(d))
    }
}

impl PathGeometry {
    /// Convert existing numeric Bézier commands without rounding or reparsing SVG.
    pub fn from_beziers(commands: &[PathCommand]) -> ChartResult<Self> {
        let commands = commands
            .iter()
            .map(|c| map_legacy(*c, Affine::default()))
            .collect::<ChartResult<Vec<_>>>()?;
        Ok(Self { commands })
    }
    /// Whether an explicit segment exists; a move/close-only path has no paint.
    pub fn has_segments(&self) -> bool {
        self.commands
            .iter()
            .any(|c| !matches!(c, Command::MoveTo(_) | Command::Close))
    }
    /// Lower only analytic arcs. Existing Béziers remain vector curves.
    ///
    /// `max_error` is a Euclidean coordinate-unit error bound, not a global epsilon.
    /// The cubic Hermite remainder is at most `sqrt(2)*r*h^4/384` per segment.
    /// Destination renderers divide their pixel budget by output scale before calling.
    pub fn lower(&self, max_error: f64, max_commands: usize) -> ChartResult<Vec<PathCommand>> {
        if !max_error.is_finite() || max_error <= 0. {
            return Err(domain(
                "Path lowering tolerance must be finite and positive",
            ));
        }
        self.validate_for_scene()?;
        require(
            self.commands.len() <= max_commands,
            "Path destination command limit",
        )?;
        let mut output = Vec::with_capacity(self.commands.len());
        let mut current = None;
        let mut start = None;
        for command in &self.commands {
            match *command {
                Command::MoveTo(p) => {
                    current = Some(p);
                    start = Some(p);
                    push(&mut output, PathCommand::MoveTo(point(p)?), max_commands)?;
                }
                Command::LineTo(p) => {
                    current = Some(p);
                    push(&mut output, PathCommand::LineTo(point(p)?), max_commands)?;
                }
                Command::QuadraticTo(p) => {
                    current = Some([p[2], p[3]]);
                    push(
                        &mut output,
                        PathCommand::QuadraticTo(point([p[0], p[1]])?, point([p[2], p[3]])?),
                        max_commands,
                    )?;
                }
                Command::CubicTo(p) => {
                    current = Some([p[4], p[5]]);
                    push(
                        &mut output,
                        PathCommand::CubicTo(
                            point([p[0], p[1]])?,
                            point([p[2], p[3]])?,
                            point([p[4], p[5]])?,
                        ),
                        max_commands,
                    )?;
                }
                Command::Horizontal(dx) => {
                    let [x, y] = current.expect("validated move");
                    let p = [x + dx, y];
                    current = Some(p);
                    push(&mut output, PathCommand::LineTo(point(p)?), max_commands)?;
                }
                Command::Vertical(dy) => {
                    let [x, y] = current.expect("validated move");
                    let p = [x, y + dy];
                    current = Some(p);
                    push(&mut output, PathCommand::LineTo(point(p)?), max_commands)?;
                }
                Command::Close => {
                    current = start;
                    push(&mut output, PathCommand::Close, max_commands)?;
                }
                Command::Arc {
                    radius,
                    large,
                    clockwise,
                    to,
                } => {
                    let from = current.expect("validated move");
                    if let Some(arc) = Arc::endpoint(from, to, radius, large, clockwise)? {
                        let max_angle = if max_error >= arc.radius {
                            FRAC_PI_2
                        } else {
                            (max_error / arc.radius * (384. / SQRT_2))
                                .powf(0.25)
                                .min(FRAC_PI_2)
                        };
                        let count = (arc.delta.abs() / max_angle).ceil().max(1.);
                        require(
                            count.is_finite()
                                && count <= (max_commands.saturating_sub(output.len())) as f64,
                            "Path arc subdivision limit",
                        )?;
                        let count = count as usize;
                        let step = arc.delta / count as f64;
                        let mut first = from;
                        for index in 0..count {
                            let a0 = arc.start + step * index as f64;
                            let a1 = arc.start + step * (index + 1) as f64;
                            let last = if index + 1 == count { to } else { arc.at(a1) };
                            let c1 = [
                                first[0] - step / 3. * arc.radius * a0.sin(),
                                first[1] + step / 3. * arc.radius * a0.cos(),
                            ];
                            let c2 = [
                                last[0] + step / 3. * arc.radius * a1.sin(),
                                last[1] - step / 3. * arc.radius * a1.cos(),
                            ];
                            push(
                                &mut output,
                                PathCommand::CubicTo(point(c1)?, point(c2)?, point(last)?),
                                max_commands,
                            )?;
                            first = last;
                        }
                    }
                    current = Some(to);
                }
            }
        }
        Ok(output)
    }
    /// Apply an affine map with bounded error for noncircular transformed arcs.
    /// Uniform orthogonal maps retain circular arcs analytically; other affine maps
    /// use the shared Hermite bound scaled by a conservative matrix norm.
    pub fn transformed(
        &self,
        map: Affine,
        max_error: f64,
        max_commands: usize,
    ) -> ChartResult<Self> {
        self.validate_for_scene()?;
        finite(&[map.norm()])?;
        if !max_error.is_finite() || max_error <= 0. {
            return Err(domain(
                "Path transform tolerance must be finite and positive",
            ));
        }
        let [a, b, c, d, _, _] = map.0;
        let scale = a.hypot(b);
        let circular = a * c + b * d == 0. && a.hypot(b) == c.hypot(d) && scale > 0.;
        if circular {
            require(
                self.commands.len() <= max_commands,
                "Path transformed command limit",
            )?;
            let mut output = vec![];
            let mut current = None;
            let mut start = None;
            for command in &self.commands {
                let c = match *command {
                    Command::MoveTo(p) => {
                        current = Some(p);
                        start = Some(p);
                        Command::MoveTo(map.point(p)?)
                    }
                    Command::LineTo(p) => {
                        current = Some(p);
                        Command::LineTo(map.point(p)?)
                    }
                    Command::QuadraticTo(p) => {
                        current = Some([p[2], p[3]]);
                        let c = map.point([p[0], p[1]])?;
                        let to = map.point([p[2], p[3]])?;
                        Command::QuadraticTo([c[0], c[1], to[0], to[1]])
                    }
                    Command::CubicTo(p) => {
                        current = Some([p[4], p[5]]);
                        let c1 = map.point([p[0], p[1]])?;
                        let c2 = map.point([p[2], p[3]])?;
                        let to = map.point([p[4], p[5]])?;
                        Command::CubicTo([c1[0], c1[1], c2[0], c2[1], to[0], to[1]])
                    }
                    Command::Arc {
                        radius,
                        large,
                        clockwise,
                        to,
                    } => {
                        current = Some(to);
                        finite(&[radius * scale])?;
                        Command::Arc {
                            radius: radius * scale,
                            large,
                            clockwise: if a * d - b * c < 0. {
                                !clockwise
                            } else {
                                clockwise
                            },
                            to: map.point(to)?,
                        }
                    }
                    Command::Horizontal(dx) => {
                        let p = current.expect("validated move");
                        let to = [p[0] + dx, p[1]];
                        current = Some(to);
                        Command::LineTo(map.point(to)?)
                    }
                    Command::Vertical(dy) => {
                        let p = current.expect("validated move");
                        let to = [p[0], p[1] + dy];
                        current = Some(to);
                        Command::LineTo(map.point(to)?)
                    }
                    Command::Close => {
                        current = start;
                        Command::Close
                    }
                };
                output.push(c);
            }
            Ok(Self { commands: output })
        } else {
            let norm = map.norm();
            let tolerance = if norm == 0. {
                max_error
            } else {
                max_error / norm
            };
            let commands = self
                .lower(tolerance, max_commands)?
                .iter()
                .map(|command| map_legacy(*command, map))
                .collect::<ChartResult<Vec<_>>>()?;
            Ok(Self { commands })
        }
    }
    /// Conservative destination bounds: Bézier control hulls plus the arc error bound.
    pub fn bounds(&self, max_error: f64, max_commands: usize) -> ChartResult<Option<Rect>> {
        let commands = self.lower(max_error, max_commands)?;
        let mut points = vec![];
        for command in commands {
            match command {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => points.push(p),
                PathCommand::QuadraticTo(a, b) => points.extend([a, b]),
                PathCommand::CubicTo(a, b, c) => points.extend([a, b, c]),
                PathCommand::Close => (),
            }
        }
        if points.is_empty() {
            return Ok(None);
        }
        let x0 = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min) - max_error;
        let y0 = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min) - max_error;
        let x1 = points
            .iter()
            .map(|p| p.x())
            .fold(f64::NEG_INFINITY, f64::max)
            + max_error;
        let y1 = points
            .iter()
            .map(|p| p.y())
            .fold(f64::NEG_INFINITY, f64::max)
            + max_error;
        Ok(Some(Rect::new(x0, y0, x1 - x0, y1 - y0)?))
    }
}
fn point(p: [f64; 2]) -> ChartResult<Point> {
    Point::new(p[0], p[1])
}
fn push(output: &mut Vec<PathCommand>, command: PathCommand, max: usize) -> ChartResult<()> {
    require(output.len() < max, "Path destination command limit")?;
    output.push(command);
    Ok(())
}
fn map_legacy(command: PathCommand, map: Affine) -> ChartResult<Command> {
    let p = |point: Point| map.point([point.x(), point.y()]);
    Ok(match command {
        PathCommand::MoveTo(a) => Command::MoveTo(p(a)?),
        PathCommand::LineTo(a) => Command::LineTo(p(a)?),
        PathCommand::QuadraticTo(a, b) => {
            let a = p(a)?;
            let b = p(b)?;
            Command::QuadraticTo([a[0], a[1], b[0], b[1]])
        }
        PathCommand::CubicTo(a, b, c) => {
            let a = p(a)?;
            let b = p(b)?;
            let c = p(c)?;
            Command::CubicTo([a[0], a[1], b[0], b[1], c[0], c[1]])
        }
        PathCommand::Close => Command::Close,
    })
}
struct Arc {
    center: [f64; 2],
    radius: f64,
    start: f64,
    delta: f64,
}
impl Arc {
    fn endpoint(
        from: [f64; 2],
        to: [f64; 2],
        radius: f64,
        large: bool,
        clockwise: bool,
    ) -> ChartResult<Option<Self>> {
        if from == to {
            return Ok(None);
        }
        if radius <= 0. {
            return Err(domain("Retained circular arc requires a positive radius"));
        }
        let dx = (from[0] - to[0]) / 2.;
        let dy = (from[1] - to[1]) / 2.;
        let half = dx.hypot(dy);
        finite(&[dx, dy, half])?;
        let radius = radius.max(half);
        // Avoid subtracting two nearly equal squares at a semicircle.
        let height = radius * (((radius - half) / radius) * (1. + half / radius)).sqrt();
        let sign = if large == clockwise { -1. } else { 1. };
        let center = [
            from[0].midpoint(to[0]) + sign * height * (dy / half),
            from[1].midpoint(to[1]) - sign * height * (dx / half),
        ];
        finite(&center)?;
        let start = (from[1] - center[1]).atan2(from[0] - center[0]);
        let end = (to[1] - center[1]).atan2(to[0] - center[0]);
        let mut delta = end - start;
        if clockwise && delta < 0. {
            delta += TAU;
        } else if !clockwise && delta > 0. {
            delta -= TAU;
        }
        Ok(Some(Self {
            center,
            radius,
            start,
            delta,
        }))
    }
    fn at(&self, angle: f64) -> [f64; 2] {
        [
            self.center[0] + self.radius * angle.cos(),
            self.center[1] + self.radius * angle.sin(),
        ]
    }
}
