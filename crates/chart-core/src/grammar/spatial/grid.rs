//! Source-grid orientation is normalized before contour topology, then restored.
use crate::ChartResult;
/// Rotate with the reference zapsmall precision shared over each coordinate column.
pub(crate) fn rotate(points: &[[f64; 2]], angle: f64) -> Vec<[f64; 2]> {
    if angle == 0. {
        return points.to_vec();
    }
    let (s, c) = (libm::sin(angle), libm::cos(angle));
    let mut out = points
        .iter()
        .map(|p| [c * p[0] - s * p[1], s * p[0] + c * p[1]])
        .collect::<Vec<_>>();
    for axis in 0..2 {
        let maximum = out.iter().map(|p| p[axis].abs()).fold(0., f64::max);
        let digits = if maximum == 0. {
            13.
        } else {
            (13. - libm::floor(libm::log10(maximum))).max(0.)
        };
        let factor = libm::pow(10., digits);
        if factor.is_finite() {
            for p in &mut out {
                p[axis] = (p[axis] * factor).round_ties_even() / factor;
                if p[axis] == 0. {
                    p[axis] = 0.;
                }
            }
        }
    }
    out
}
fn angle(points: &[[f64; 2]]) -> f64 {
    if points.len() < 2 {
        return 0.;
    }
    let angles = points
        .iter()
        .take(20)
        .copied()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|p| libm::atan2(p[1][1] - p[0][1], p[1][0] - p[0][0]))
        .collect::<Vec<_>>();
    let mut winner = (0., 0);
    for a in &angles {
        let n = angles.iter().filter(|b| *b == a).count();
        if n > winner.1 {
            winner = (*a, n);
        }
    }
    let a = if winner.1 * 2 >= angles.len() {
        winner.0
    } else {
        let mut sorted = points.to_vec();
        sorted.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
        sorted.dedup();
        let cross = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
        };
        // Build each chain separately so the reverse pass cannot erase the lower hull.
        let chain = |iter: Vec<[f64; 2]>| {
            let mut h: Vec<[f64; 2]> = vec![];
            for p in iter {
                while h.len() >= 2 && cross(h[h.len() - 2], h[h.len() - 1], p) <= 0. {
                    h.pop();
                }
                h.push(p);
            }
            h
        };
        let mut hull = chain(sorted.clone());
        hull.pop();
        hull.extend(chain(sorted.into_iter().rev().collect()));
        if let Some(first) = hull.first().copied() {
            hull.push(first);
        }
        let edge = hull.windows(2).max_by(|a, b| {
            libm::hypot(a[1][0] - a[0][0], a[1][1] - a[0][1])
                .total_cmp(&libm::hypot(b[1][0] - b[0][0], b[1][1] - b[0][1]))
        });
        edge.map_or(0., |p| libm::atan2(p[1][1] - p[0][1], p[1][0] - p[0][0]))
    };
    if (-2..=2)
        .any(|i| (a - i as f64 * std::f64::consts::FRAC_PI_2).abs() < libm::sqrt(f64::EPSILON))
    {
        0.
    } else {
        a
    }
}
pub(crate) struct Grid {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<Option<f64>>,
    pub angle: f64,
}
pub(crate) fn grid(
    points: &[[f64; 2]],
    values: &[Option<f64>],
    budget: usize,
) -> ChartResult<Grid> {
    let angle = angle(points);
    let p = rotate(points, -angle);
    let mut x = p.iter().map(|p| p[0]).collect::<Vec<_>>();
    let mut y = p.iter().map(|p| p[1]).collect::<Vec<_>>();
    x.sort_by(f64::total_cmp);
    y.sort_by(f64::total_cmp);
    x.dedup();
    y.dedup();
    let size = x
        .len()
        .checked_mul(y.len())
        .filter(|n| *n <= budget)
        .ok_or_else(|| super::invalid("Contour grid budget exceeded."))?;
    let mut z = vec![None; size];
    for (p, v) in p.iter().zip(values) {
        let i = x.binary_search_by(|v| v.total_cmp(&p[0])).unwrap();
        let j = y.binary_search_by(|v| v.total_cmp(&p[1])).unwrap();
        z[j * x.len() + i] = *v;
    }
    Ok(Grid { x, y, z, angle })
}
