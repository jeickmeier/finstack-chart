use super::super::EllipseKind;
use crate::ChartResult;
use std::f64::consts::PI;
/// Weighted reference ellipse, with original group size retained for F degrees of freedom.
pub(crate) fn ellipse(
    points: &[[f64; 2]],
    weights: &[f64],
    kind: EllipseKind,
    level: f64,
    segments: usize,
    budget: usize,
) -> ChartResult<Vec<[f64; 2]>> {
    if points.len() != weights.len()
        || points.iter().flatten().any(|v| !v.is_finite())
        || weights.iter().any(|v| !v.is_finite() || *v < 0.)
        || !level.is_finite()
        || level < 0.
        || (kind != EllipseKind::Euclidean && level >= 1.)
        || segments == 0
        || segments.checked_add(1).is_none_or(|n| n > budget)
    {
        return Err(super::invalid(
            "Ellipse requires finite coordinates, nonnegative weights and bounded segments.",
        ));
    }
    if points.len() < 4 {
        return Ok(Vec::new());
    }
    let total = weights.iter().sum::<f64>();
    if total <= 0. {
        return Err(super::invalid("Ellipse requires positive total weight."));
    }
    let weights = weights.iter().map(|w| w / total).collect::<Vec<_>>();
    let mut center = [0.; 2];
    for (point, w) in points.iter().zip(&weights) {
        for axis in 0..2 {
            center[axis] += point[axis] * w;
        }
    }
    let covariance = |center: [f64; 2], w: &[f64], denominator: f64| {
        let mut c = [0.; 3];
        for (point, w) in points.iter().zip(w) {
            let dx = point[0] - center[0];
            let dy = point[1] - center[1];
            c[0] += w * dx * dx;
            c[1] += w * dx * dy;
            c[2] += w * dy * dy;
        }
        c.map(|v| v / denominator)
    };
    let mut cov;
    if kind == EllipseKind::T {
        // cov.trob uses nu=5, maxit=25, tol=0.01 and n-scaled observation weights.
        let wt = weights
            .iter()
            .map(|w| w * points.len() as f64)
            .collect::<Vec<_>>();
        let mut w = wt.iter().map(|w| w * 1.4).collect::<Vec<_>>();
        let mut last_center = center;
        for _ in 0..25 {
            last_center = center;
            let c = covariance(center, &w, w.iter().sum());
            let det = c[0] * c[2] - c[1] * c[1];
            if det <= 0. || !det.is_finite() {
                return Err(super::invalid("Ellipse covariance is singular."));
            }
            let mut next = Vec::with_capacity(w.len());
            for (p, wt) in points.iter().zip(&wt) {
                let x = p[0] - center[0];
                let y = p[1] - center[1];
                let q = (c[2] * x * x - 2. * c[1] * x * y + c[0] * y * y) / det;
                next.push(wt * 7. / (5. + q));
            }
            let sum = next.iter().sum::<f64>();
            center = [0.; 2];
            for (p, w) in points.iter().zip(&next) {
                center[0] += p[0] * w / sum;
                center[1] += p[1] * w / sum;
            }
            let converged = w.iter().zip(&next).all(|(a, b)| (a - b).abs() < 0.01);
            w = next;
            if converged {
                break;
            }
        }
        cov = covariance(last_center, &w, wt.iter().sum());
    } else {
        let denominator = 1. - weights.iter().map(|w| w * w).sum::<f64>();
        if denominator <= 0. {
            return Err(super::invalid(
                "Ellipse effective covariance degrees of freedom are zero.",
            ));
        }
        cov = covariance(center, &weights, denominator);
    }
    if kind == EllipseKind::Euclidean {
        let v = cov[0].min(cov[2]);
        cov = [v, 0., v];
    }
    let a = libm::sqrt(cov[0]);
    let b = cov[1] / a;
    let c = libm::sqrt(cov[2] - b * b);
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || a <= 0. || c <= 0. {
        return Err(super::invalid(
            "Ellipse covariance is not positive definite.",
        ));
    }
    let radius = if kind == EllipseKind::Euclidean {
        level / a.max(c)
    } else {
        let df = (points.len() - 1) as f64;
        libm::sqrt(df * libm::expm1(-2. / df * libm::log1p(-level)))
    };
    Ok((0..=segments)
        .map(|i| {
            let theta = i as f64 * 2. * PI / segments as f64;
            let x = libm::cos(theta);
            let y = libm::sin(theta);
            [
                center[0] + radius * x * a,
                center[1] + radius * (x * b + y * c),
            ]
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_weighted_three_ellipse_methods() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let kind = match case["name"].as_str().unwrap() {
                "ellipse-norm" => EllipseKind::Normal,
                "ellipse-t" => EllipseKind::T,
                "ellipse-euclid" => EllipseKind::Euclidean,
                _ => continue,
            };
            let rows = case["input"].as_array().unwrap();
            let points = rows
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let weights = rows
                .iter()
                .map(|r| r["w"].as_f64().unwrap())
                .collect::<Vec<_>>();
            let actual = ellipse(&points, &weights, kind, 0.8, 8, 9).unwrap();
            for (i, p) in actual.iter().enumerate() {
                for (axis, name) in ["x", "y"].iter().enumerate() {
                    let e = case["built"][name][i].as_f64().unwrap();
                    assert!(
                        (p[axis] - e).abs() < 2e-12,
                        "{kind:?} {i} {name}: {} vs {e}",
                        p[axis]
                    );
                }
            }
        }
        assert!(
            ellipse(&[[1., 1.]; 3], &[1.; 3], EllipseKind::Normal, 0.8, 8, 9)
                .unwrap()
                .is_empty()
        );
        assert!(ellipse(&[[1., 1.]; 4], &[1.; 4], EllipseKind::Normal, 0.8, 8, 9).is_err());
    }
}
