//! Destination geometry for reference rugs and grid-style curves.
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};

fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::NumericalDomain,
        message,
        "Supply valid reference recipe controls.",
    )
}

/// Construct grid's angle-skewed circular control polygon in y-up physical units.
/// The endpoints remain interpolation points; internal points have X-spline shape 0.5.
pub(crate) fn curve_controls(
    from: Point,
    to: Point,
    curvature: f64,
    angle: f64,
    ncp: usize,
) -> ChartResult<Vec<Point>> {
    if !curvature.is_finite() || !angle.is_finite() || ncp == 0 || ncp > 4096 {
        return Err(invalid(
            "Curve curvature/angle must be finite and ncp must be in 1..=4096.",
        ));
    }
    if from == to {
        return Err(invalid("Curve endpoints must not be identical."));
    }
    if curvature == 0. || !(1. ..=179.).contains(&angle) {
        return Ok(vec![from, to]);
    }
    let dx = to.x() - from.x();
    let dy = to.y() - from.y();
    let (sa, ca) = (libm::sin(angle.to_radians()), libm::cos(angle.to_radians()));
    let corner_x = (dx * (1. - ca) + dy * sa) / 2.;
    let corner_y = (dy * (1. - ca) - dx * sa) / 2.;
    let beta = -libm::atan(corner_y / corner_x);
    let (sb, cb) = (libm::sin(beta), libm::cos(beta));
    let rx = dx * cb - dy * sb;
    let ry = dy * cb + dx * sb;
    let scale = ry / rx;
    // In the scaled frame both components of the endpoint are ry.
    let origin = (curvature - 1. / curvature) / 2.;
    let ox = ry * (1. + origin) / 2.;
    let oy = ry * (1. - origin) / 2.;
    let direction = curvature.signum();
    let sweep = direction
        * (std::f64::consts::PI + (origin * direction).signum() * 2. * libm::atan(origin.abs()));
    let mut result = Vec::with_capacity(ncp + 2);
    result.push(from);
    for i in 1..=ncp {
        let (s, c) = {
            let theta = sweep * i as f64 / (ncp + 1) as f64;
            (libm::sin(theta), libm::cos(theta))
        };
        let x = (ox - ox * c + oy * s) / scale;
        let y = oy - oy * c - ox * s;
        result.push(Point::new(
            from.x() + x * cb + y * sb,
            from.y() + y * cb - x * sb,
        )?);
    }
    result.push(to);
    Ok(result)
}

// Positive-shape rational X-spline basis from Blanc & Schlick, SIGGRAPH 1995.
// Same mathematical basis as R's src/main/xspline.c, with translated knot index
// eliminated. End shapes are zero; interior shapes are one half.
fn basis(u: f64, denominator: f64) -> f64 {
    let p = 2. * denominator * denominator;
    let v = u / denominator;
    v * v * v * (10. - p + (2. * p - 15.) * v + (6. - p) * v * v)
}
fn sample(points: [Point; 4], s1: f64, s2: f64, t: f64) -> ChartResult<Point> {
    let weights = [
        if t < s1 { basis(t - s1, -1. - s1) } else { 0. },
        basis(t - 1. - s2, -1. - s2),
        basis(t + s1, 1. + s1),
        if t > 1. - s2 {
            basis(t - 1. + s2, 1. + s2)
        } else {
            0.
        },
    ];
    let sum: f64 = weights.iter().sum();
    Point::new(
        points
            .iter()
            .zip(weights)
            .map(|(p, w)| p.x() * w)
            .sum::<f64>()
            / sum,
        points
            .iter()
            .zip(weights)
            .map(|(p, w)| p.y() * w)
            .sum::<f64>()
            / sum,
    )
}
/// Flatten the actual rational X-spline, with a bounded destination-space error.
/// Input/output coordinates are y-down; the control polygon uses grid's y-up frame.
pub(crate) fn curve(
    from: Point,
    to: Point,
    curvature: f64,
    angle: f64,
    ncp: usize,
    budget: usize,
) -> ChartResult<Vec<Point>> {
    if budget < 2 {
        return Err(Diagnostic::error(
            DiagnosticCode::ResourceLimit,
            "Curve requires at least two destination vertices.",
            "Increase the configured vertex budget.",
        ));
    }
    let controls = curve_controls(
        Point::new(from.x(), -from.y())?,
        Point::new(to.x(), -to.y())?,
        curvature,
        angle,
        ncp,
    )?;
    if controls.len() == 2 {
        return Ok(vec![from, to]);
    }
    let mut result = vec![from];
    for i in 0..controls.len() - 1 {
        let points = [
            controls[i.saturating_sub(1)],
            controls[i],
            controls[i + 1],
            controls[(i + 2).min(controls.len() - 1)],
        ];
        let s1 = if i == 0 { 0. } else { 0.5 };
        let s2 = if i + 1 == controls.len() - 1 { 0. } else { 0.5 };
        let start = sample(points, s1, s2, 0.)?;
        let end = sample(points, s1, s2, 1.)?;
        let mut pending = vec![(0., 1., start, end, 0u8)];
        while let Some((a, b, p, q, depth)) = pending.pop() {
            let middle = sample(points, s1, s2, (a + b) / 2.)?;
            let mut deviation: f64 = 0.;
            for fraction in [0.25, 0.5, 0.75] {
                let actual = sample(points, s1, s2, a + (b - a) * fraction)?;
                deviation = deviation.max(libm::hypot(
                    actual.x() - (p.x() + (q.x() - p.x()) * fraction),
                    actual.y() - (p.y() + (q.y() - p.y()) * fraction),
                ));
            }
            if deviation > 0.01 && depth < 20 {
                pending.push(((a + b) / 2., b, middle, q, depth + 1));
                pending.push((a, (a + b) / 2., p, middle, depth + 1));
            } else {
                if deviation > 0.01 {
                    return Err(invalid(
                        "Curve subdivision could not reach destination tolerance.",
                    ));
                }
                if result.len() >= budget {
                    return Err(Diagnostic::error(
                        DiagnosticCode::ResourceLimit,
                        "Curve exceeds the destination vertex budget.",
                        "Increase the configured vertex budget.",
                    ));
                }
                result.push(Point::new(q.x(), -q.y())?);
            }
        }
    }
    Ok(result)
}
/// Rug lengths are fractions of physical panel dimensions, independently per side.
pub(crate) fn rug(
    center: Point,
    panel: Rect,
    sides: &str,
    length: f64,
    outside: bool,
) -> ChartResult<Vec<(Point, Point)>> {
    if !length.is_finite() || length < 0. || sides.chars().any(|c| !"bltr".contains(c)) {
        return Err(invalid(
            "Rug length must be nonnegative and sides must contain only b/l/t/r.",
        ));
    }
    let sign = if outside { -1. } else { 1. };
    let mut result = Vec::new();
    for side in ['b', 'l', 't', 'r'] {
        if !sides.contains(side) {
            continue;
        }
        let (a, b) = match side {
            'b' => (
                Point::new(center.x(), panel.max_y())?,
                Point::new(center.x(), panel.max_y() - sign * length * panel.height())?,
            ),
            't' => (
                Point::new(center.x(), panel.origin().y())?,
                Point::new(
                    center.x(),
                    panel.origin().y() + sign * length * panel.height(),
                )?,
            ),
            'l' => (
                Point::new(panel.origin().x(), center.y())?,
                Point::new(
                    panel.origin().x() + sign * length * panel.width(),
                    center.y(),
                )?,
            ),
            _ => (
                Point::new(panel.max_x(), center.y())?,
                Point::new(panel.max_x() - sign * length * panel.width(), center.y())?,
            ),
        };
        result.push((a, b));
    }
    Ok(result)
}

pub(super) fn project(
    recipe: &crate::grammar::PreparedMarkRecipe,
    mark: &crate::grammar::PreparedMark,
    layer: &crate::grammar::Layer,
    request: &super::LayoutRequest,
    panel: Rect,
    map: &dyn Fn(Point) -> ChartResult<Option<Point>>,
    map_axis: &dyn Fn(f64, bool) -> ChartResult<Option<f64>>,
) -> ChartResult<Vec<crate::scene::Primitive>> {
    use crate::{
        grammar::{LineType, PreparedMarkRecipe},
        scene::{PathCommand, Primitive},
    };
    let style = mark.style;
    let line = style.line_type.unwrap_or(LineType::Solid);
    if line == LineType::Blank {
        return Ok(vec![]);
    }
    let stroke = super::recipe_intervals::stroke(mark, layer, request);
    let width = stroke.width;
    let (paths, arrow) = match recipe {
        PreparedMarkRecipe::Rug { x, y, controls } => {
            let x = x.map(|x| map_axis(x, true)).transpose()?.flatten();
            let y = y.map(|y| map_axis(y, false)).transpose()?.flatten();
            let sides = controls
                .sides
                .chars()
                .filter(|s| {
                    if "bt".contains(*s) {
                        x.is_some()
                    } else {
                        y.is_some()
                    }
                })
                .collect::<String>();
            let center = Point::new(
                x.unwrap_or(panel.origin().x()),
                y.unwrap_or(panel.origin().y()),
            )?;
            (
                rug(center, panel, &sides, controls.length, controls.outside)?
                    .into_iter()
                    .map(|(a, b)| vec![a, b])
                    .collect::<Vec<_>>(),
                None,
            )
        }
        PreparedMarkRecipe::Curve { from, to, controls } => {
            let (Some(from), Some(to)) = (map(*from)?, map(*to)?) else {
                return Ok(vec![]);
            };
            (
                vec![curve(
                    from,
                    to,
                    controls.curvature,
                    controls.angle,
                    controls.ncp,
                    request.limits.max_path_commands,
                )?],
                controls.arrow.as_ref(),
            )
        }
        PreparedMarkRecipe::Spoke { from, to, controls } => {
            let (Some(from), Some(to)) = (map(*from)?, map(*to)?) else {
                return Ok(vec![]);
            };
            (vec![vec![from, to]], controls.arrow.as_ref())
        }
    };
    let primitives = paths
        .into_iter()
        .map(|points| {
            let commands = points
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == 0 {
                        PathCommand::MoveTo(p)
                    } else {
                        PathCommand::LineTo(p)
                    }
                })
                .collect();
            if line == LineType::Solid {
                Ok(Primitive::Path { commands, stroke })
            } else {
                Ok(Primitive::DashedPath {
                    commands,
                    stroke,
                    dashes: line.pattern(width)?,
                })
            }
        })
        .collect::<ChartResult<Vec<_>>>()?;
    super::recipe_intervals::with_arrows(primitives, arrow, &style, request)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_curve_control_polygons_and_rational_paths() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/recipe-mark-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let curvature = case["curvature"].as_f64().unwrap();
            let angle = case["angle"].as_f64().unwrap();
            let ncp = case["ncp"].as_u64().unwrap() as usize;
            let controls = curve_controls(
                Point::new(1., 1.).unwrap(),
                Point::new(4., 3.).unwrap(),
                curvature,
                angle,
                ncp,
            )
            .unwrap();
            for (actual, expected) in controls.iter().zip(case["controls"].as_array().unwrap()) {
                assert!(
                    (actual.x() - expected[0].as_f64().unwrap()).abs() < 2e-12,
                    "{case}"
                );
                assert!(
                    (actual.y() - expected[1].as_f64().unwrap()).abs() < 2e-12,
                    "{case}"
                );
            }
            // R points are in physical inches. A 96-unit inch makes this a
            // destination-space test independent of R's adaptive sampling times.
            let actual = curve(
                Point::new(96., -96.).unwrap(),
                Point::new(384., -288.).unwrap(),
                curvature,
                angle,
                ncp,
                100_000,
            )
            .unwrap();
            for expected in case["points"].as_array().unwrap() {
                let x = 96. * expected[0].as_f64().unwrap();
                let y = -96. * expected[1].as_f64().unwrap();
                let distance = actual
                    .windows(2)
                    .map(|p| {
                        let dx = p[1].x() - p[0].x();
                        let dy = p[1].y() - p[0].y();
                        let t = (((x - p[0].x()) * dx + (y - p[0].y()) * dy) / (dx * dx + dy * dy))
                            .clamp(0., 1.);
                        (x - p[0].x() - t * dx).hypot(y - p[0].y() - t * dy)
                    })
                    .fold(f64::INFINITY, f64::min);
                assert!(
                    distance < 0.015,
                    "distance={distance}, curvature={curvature}, angle={angle}, ncp={ncp}"
                );
            }
        }
    }
    #[test]
    fn straight_and_curved_recipes_obey_identical_vertex_budgets() {
        let a = Point::new(0., 0.).unwrap();
        let b = Point::new(10., 10.).unwrap();
        for curvature in [0., 0.5] {
            assert!(curve(a, b, curvature, 90., 5, 1).is_err());
        }
        assert_eq!(curve(a, b, 0., 90., 5, 2).unwrap(), vec![a, b]);
        assert!(curve(a, a, 0., 90., 5, 2).is_err());
    }
    #[test]
    fn rugs_use_each_physical_panel_extent_and_outside_direction() {
        let p = Rect::new(10., 20., 200., 100.).unwrap();
        let center = Point::new(80., 50.).unwrap();
        let ticks = rug(center, p, "bltr", 0.04, false).unwrap();
        assert_eq!(ticks.len(), 4);
        assert_eq!(
            ticks[0],
            (
                Point::new(80., 120.).unwrap(),
                Point::new(80., 116.).unwrap()
            )
        );
        assert_eq!(
            ticks[1],
            (Point::new(10., 50.).unwrap(), Point::new(18., 50.).unwrap())
        );
        let outside = rug(center, p, "b", 0.04, true).unwrap();
        assert_eq!(outside[0].1.y(), 124.);
    }
}
