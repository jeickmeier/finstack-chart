use crate::ChartResult;
use std::f64::consts::PI;
/// Rectangular KDE grid, x varying fastest, as in reference expand.grid output.
pub(crate) struct DensityGrid {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub values: Vec<f64>,
}
/// Product Gaussian KDE; reference bandwidths are four times kernel standard deviations.
pub(crate) fn density(
    points: &[[f64; 2]],
    bandwidth: Option<[f64; 2]>,
    adjust: [f64; 2],
    n: [usize; 2],
    ranges: [[f64; 2]; 2],
    budget: usize,
) -> ChartResult<DensityGrid> {
    let cells = n[0].checked_mul(n[1]).filter(|v| *v <= budget);
    if points.is_empty()
        || points.iter().flatten().any(|v| !v.is_finite())
        || n.contains(&0)
        || cells.is_none()
        || ranges
            .iter()
            .any(|r| r.iter().any(|v| !v.is_finite()) || r[0] > r[1])
        || adjust.iter().any(|v| !v.is_finite() || *v <= 0.)
    {
        return Err(super::invalid(
            "2D density requires finite observations, ordered ranges and a bounded positive grid.",
        ));
    }
    let h = match bandwidth {
        Some(h) => h,
        None => {
            let mut h = [0.; 2];
            for axis in 0..2 {
                let values = points.iter().map(|p| p[axis]).collect::<Vec<_>>();
                let (selected, _) = super::super::distributions::select_bandwidth(
                    &values,
                    super::super::Bandwidth::Nrd,
                )?;
                h[axis] = selected * 4.;
                if h[axis] == 0. && values.len() > 1 {
                    h[axis] = super::super::distributions::select_bandwidth(
                        &values,
                        super::super::Bandwidth::Nrd0,
                    )?
                    .0 * 4.;
                }
                h[axis] *= adjust[axis];
            }
            h
        }
    };
    if h.iter().any(|v| !v.is_finite() || *v <= 0.) {
        return Err(super::invalid(
            "2D density bandwidths must be finite and positive.",
        ));
    }
    let h = [h[0] / 4., h[1] / 4.];
    let sequence = |axis: usize| {
        (0..n[axis])
            .map(|i| {
                if n[axis] == 1 {
                    ranges[axis][0]
                } else if i + 1 == n[axis] {
                    ranges[axis][1]
                } else {
                    ranges[axis][0]
                        + i as f64 * (ranges[axis][1] - ranges[axis][0]) / (n[axis] - 1) as f64
                }
            })
            .collect::<Vec<_>>()
    };
    let x = sequence(0);
    let y = sequence(1);
    let normal = |v: f64| libm::exp(-0.5 * v * v) / libm::sqrt(2. * PI);
    let mut values = Vec::with_capacity(cells.unwrap());
    for gy in &y {
        for gx in &x {
            let sum = points
                .iter()
                .map(|p| normal((gx - p[0]) / h[0]) * normal((gy - p[1]) / h[1]))
                .sum::<f64>();
            values.push(sum / (points.len() as f64 * h[0] * h[1]));
        }
    }
    Ok(DensityGrid { x, y, values })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_product_kde_grid_and_ignored_weights() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            if !name.starts_with("kde2d-weight-") {
                continue;
            }
            let points = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let actual = density(
                &points,
                Some([1., 2.]),
                [1., 1.],
                [5, 5],
                [[0., 2.], [0., 2.]],
                25,
            )
            .unwrap();
            let expected = case["built"]["density"].as_array().unwrap();
            for (a, e) in actual.values.iter().zip(expected) {
                assert!(
                    (a - e.as_f64().unwrap()).abs() < 2e-14,
                    "{name}: {a} vs {e}"
                );
            }
            assert_eq!(actual.x, vec![0., 0.5, 1., 1.5, 2.]);
            assert_eq!(actual.y, actual.x);
        }
        assert!(
            density(
                &[[0., 0.]],
                Some([1., 1.]),
                [1., 1.],
                [100, 100],
                [[0., 1.], [0., 1.]],
                99
            )
            .is_err()
        );
    }
}
