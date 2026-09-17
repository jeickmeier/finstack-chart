//! One-dimensional LOESS local polynomials and interpolated Hermite surfaces.
use super::model_linear;
use crate::{ChartResult, DiagnosticCode};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Surface {
    Interpolate,
    Direct,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Family {
    Gaussian,
    Symmetric,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Controls {
    pub exact_statistics: bool,
    pub approximate_trace: bool,
    pub span: f64,
    pub degree: usize,
    pub cell: f64,
    pub surface: Surface,
    pub family: Family,
    pub iterations: usize,
    pub normalize: bool,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            exact_statistics: false,
            approximate_trace: false,
            span: 0.75,
            degree: 2,
            cell: 0.2,
            surface: Surface::Interpolate,
            family: Family::Gaussian,
            iterations: 4,
            normalize: true,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct LoessFit {
    x: Vec<f64>,
    y: Vec<f64>,
    weights: Vec<f64>,
    vertices: Vec<(f64, f64, f64)>,
    controls: Controls,
    pub robust: Vec<f64>,
    prior: Vec<f64>,
}
fn fail(message: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::NumericalDomain, message)
}
fn local(
    x: &[f64],
    y: &[f64],
    weights: &[f64],
    at: f64,
    c: Controls,
) -> ChartResult<(f64, f64, Vec<[f64; 2]>)> {
    let p = c.degree + 1;
    let n = x.len();
    let nf = ((n as f64 * c.span + 1e-5).floor() as usize).min(n);
    if nf < p {
        return Err(fail(
            "LOESS neighborhood is too small for polynomial degree.",
        ));
    }
    let mut order: Vec<_> = (0..n).collect();
    order.sort_by(|&a, &b| {
        (x[a] - at)
            .abs()
            .total_cmp(&(x[b] - at).abs())
            .then(a.cmp(&b))
    });
    let radius = (x[order[nf - 1]] - at).abs() * libm::sqrt(c.span.max(1.));
    if radius <= 0. || !radius.is_finite() {
        return Err(fail("LOESS neighborhood has zero or invalid radius."));
    }
    let mut design = Vec::with_capacity(nf * p);
    let mut response = Vec::with_capacity(nf);
    let mut w = Vec::with_capacity(nf);
    for &i in &order[..nf] {
        let dx = x[i] - at;
        design.push(1.);
        if c.degree >= 1 {
            design.push(dx);
        }
        if c.degree == 2 {
            design.push(dx * dx);
        }
        response.push(y[i]);
        let u = (dx / radius).abs();
        let v = 1. - u * u * u;
        w.push(weights[i] * v * v * v);
    }
    let fit = model_linear::fit(&design, &response, &w, p)?;
    let mut influence = vec![[0.; 2]; n];
    for (j, &i) in order[..nf].iter().enumerate() {
        for (k, value) in influence[i].iter_mut().enumerate().take(p.min(2)) {
            *value = (0..p)
                .map(|c| fit.covariance[k * p + c] * design[j * p + c])
                .sum::<f64>()
                * w[j];
        }
    }
    Ok((
        fit.coefficients[0].unwrap_or(0.),
        fit.coefficients.get(1).copied().flatten().unwrap_or(0.),
        influence,
    ))
}
fn split(
    sorted: &[f64],
    threshold: usize,
    vertices: &mut Vec<f64>,
    budget: usize,
) -> ChartResult<()> {
    if sorted.len() <= threshold.max(1) || sorted.first() == sorted.last() {
        return Ok(());
    }
    let mut middle = (sorted.len() - 1) / 2;
    while middle + 1 < sorted.len() && sorted[middle + 1] == sorted[middle] {
        middle += 1;
    }
    if middle + 1 == sorted.len() {
        return Ok(());
    }
    if vertices.len() >= budget {
        return Err(fail("LOESS interpolation vertex budget exceeded."));
    }
    vertices.push(sorted[middle]);
    split(&sorted[..=middle], threshold, vertices, budget)?;
    split(&sorted[middle + 1..], threshold, vertices, budget)?;
    Ok(())
}
impl LoessFit {
    pub fn predict(&self, grid: &[f64]) -> ChartResult<Vec<Option<f64>>> {
        grid.iter()
            .map(|&at| {
                if !at.is_finite() {
                    return Err(fail("LOESS prediction grid must be finite."));
                }
                if self.controls.surface == Surface::Direct {
                    return local(&self.x, &self.y, &self.weights, at, self.controls)
                        .map(|x| Some(x.0));
                }
                let lo = self.x.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = self.x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                if at < lo || at > hi {
                    return Ok(None);
                }
                let i = self
                    .vertices
                    .partition_point(|v| v.0 < at)
                    .clamp(1, self.vertices.len() - 1);
                let (a, ya, da) = self.vertices[i - 1];
                let (b, yb, db) = self.vertices[i];
                Ok(Some(hermite(at, a, b, ya, da, yb, db)))
            })
            .collect()
    }
}
pub(crate) fn fit(
    x: &[f64],
    y: &[f64],
    weights: &[f64],
    controls: Controls,
    max_vertices: usize,
) -> ChartResult<LoessFit> {
    let n = x.len();
    if n < 2
        || y.len() != n
        || weights.len() != n
        || x.iter().chain(y).chain(weights).any(|v| !v.is_finite())
        || weights.iter().any(|w| *w < 0.)
        || !controls.span.is_finite()
        || controls.span <= 0.
        || !controls.cell.is_finite()
        || controls.cell <= 0.
        || controls.degree > 2
        || controls.iterations == 0
        || controls.iterations > 100
    {
        return Err(fail("Invalid LOESS inputs or controls."));
    }
    // R normalizes predictors only for multivariate designs; one-dimensional normalization is identity.
    let _ = controls.normalize;
    let mut sorted = x.to_vec();
    sorted.sort_by(f64::total_cmp);
    let lo = sorted[0];
    let hi = sorted[n - 1];
    let padding = 0.005 * (hi - lo).max(1e-10 * lo.abs().max(hi.abs()) + 1e-30);
    let mut knots = vec![lo - padding, hi + padding];
    if controls.surface == Surface::Interpolate {
        split(
            &sorted,
            (n as f64 * controls.span * controls.cell).floor() as usize,
            &mut knots,
            max_vertices,
        )?;
    }
    knots.sort_by(f64::total_cmp);
    knots.dedup();
    if knots.len() > max_vertices {
        return Err(fail("LOESS interpolation vertex budget exceeded."));
    }
    let mut out = LoessFit {
        x: x.to_vec(),
        y: y.to_vec(),
        weights: weights.to_vec(),
        vertices: vec![],
        controls,
        robust: vec![1.; n],
        prior: weights.to_vec(),
    };
    let iterations = if controls.family == Family::Gaussian {
        1
    } else {
        controls.iterations
    };
    for iteration in 0..iterations {
        out.weights = weights
            .iter()
            .zip(&out.robust)
            .map(|(a, b)| a * b)
            .collect();
        out.vertices = knots
            .iter()
            .map(|&at| local(x, y, &out.weights, at, controls).map(|(y, d, _)| (at, y, d)))
            .collect::<ChartResult<_>>()?;
        if iteration + 1 < iterations {
            let fitted = out.predict(x)?;
            let residual: Vec<_> = y
                .iter()
                .zip(fitted)
                .map(|(y, f)| (y - f.unwrap()).abs())
                .collect();
            let mut sorted = residual.clone();
            sorted.sort_by(f64::total_cmp);
            let median = if n.is_multiple_of(2) {
                (sorted[n / 2 - 1] + sorted[n / 2]) / 2.
            } else {
                sorted[n / 2]
            };
            let cutoff = 6. * median;
            out.robust = residual
                .iter()
                .map(|r| {
                    if cutoff < f64::MIN_POSITIVE {
                        1.
                    } else if *r > cutoff * 0.999 {
                        0.
                    } else if *r < cutoff * 0.001 {
                        1.
                    } else {
                        let u = r / cutoff;
                        (1. - u * u) * (1. - u * u)
                    }
                })
                .collect();
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_loess_mean_surfaces() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
            c["name"].as_str().unwrap().starts_with("loess-") || c["name"] == "automatic-999"
        }) {
            let data = case["controls"]["data"].as_array().unwrap();
            let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap_or(1.)).collect();
            let controls = Controls {
                span: case["controls"]["span"].as_f64().unwrap_or(0.75),
                degree: case["controls"]["method_args"]["degree"]
                    .as_u64()
                    .unwrap_or(2) as usize,
                family: if case["name"] == "loess-symmetric" {
                    Family::Symmetric
                } else {
                    Family::Gaussian
                },
                ..Default::default()
            };
            let fit = fit(&x, &y, &w, controls, 10000).unwrap();
            let expected = case["result"]["value"]["values"].as_array().unwrap();
            let grid: Vec<_> = expected.iter().map(|r| r["x"].as_f64().unwrap()).collect();
            let actual = fit.predict(&grid).unwrap();
            let uncertainty = fit.uncertainty(&grid, 2_000_000).unwrap();
            assert!(uncertainty.df > 0. && uncertainty.residual_scale > 0.);
            for (se, b) in uncertainty.standard_errors.iter().zip(expected) {
                if let Some(se) = se {
                    assert!(
                        (se - b["se"].as_f64().unwrap()).abs() < 1e-9,
                        "{} se {se} != {}",
                        case["name"],
                        b["se"]
                    );
                }
            }
            for (a, b) in actual.iter().zip(expected) {
                match (a, b["y"].as_f64()) {
                    (Some(a), Some(b)) => {
                        assert!((a - b).abs() < 1e-9, "{}: {a} != {b}", case["name"])
                    }
                    (None, None) => {}
                    _ => panic!("missing mismatch"),
                }
            }
        }
    }
}
fn hermite(at: f64, a: f64, b: f64, ya: f64, da: f64, yb: f64, db: f64) -> f64 {
    let h = b - a;
    let t = (at - a) / h;
    (2. * t * t * t - 3. * t * t + 1.) * ya
        + (t * t * t - 2. * t * t + t) * h * da
        + (-2. * t * t * t + 3. * t * t) * yb
        + (t * t * t - t * t) * h * db
}
// Numerical calibration data for Cleveland/Grosse/Shyu approximate residual traces.
// Provenance: R stats loessf.f ehg141/ehg176; these are model coefficients, not solver code.
fn deltas(n: usize, degree: usize, trace: f64) -> (f64, f64) {
    let degree = degree.max(1);
    let k = (degree + 1) as f64;
    let correction = libm::sqrt(k / n as f64);
    let z = ((libm::sqrt(k / trace) - correction) / (1. - correction)).clamp(0., 1.);
    let knots = [
        (-0.005, -0.090572, 4.4844),
        (0.1204, 0.095807, -0.7978),
        (0.2017, 0.026152, -0.7286),
        (0.2815, -0.031926, -0.4457),
        (0.3705, -0.053718, -0.3495),
        (0.4536, -0.06417, 0.032813),
        (0.5591, -0.058387, 0.1611),
        (0.7132, -0.020636, 0.335),
        (0.8751, 0.040172, -0.041032),
        (1.005, -0.010856, -0.7736),
    ];
    let i = knots.partition_point(|v| v.0 < z).clamp(1, knots.len() - 1);
    let a = knots[i - 1];
    let b = knots[i];
    let correction = libm::exp(hermite(z, a.0, b.0, a.1, a.2, b.1, b.2));
    let eval = |c: [f64; 3]| {
        n as f64
            - trace * libm::exp(c[0] * libm::pow(z, c[1]) * libm::pow(1. - z, c[2]) * correction)
    };
    let linear = [
        eval([0.297162, 0.380266, 0.5886043]),
        eval([0.2848308, 0.2254512, 0.2914126]),
    ];
    let quadratic = [
        eval([0.1611761, 0.3091323, 0.4401023]),
        eval([0.207567, 0.2822574, 0.2369957]),
    ];
    let t = degree as f64 - 1.;
    (
        (1. - t) * linear[0] + t * quadratic[0],
        (1. - t) * linear[1] + t * quadratic[1],
    )
}
#[derive(Clone, Debug)]
pub(crate) struct Uncertainty {
    pub fitted: Vec<Option<f64>>,
    pub standard_errors: Vec<Option<f64>>,
    pub df: f64,
    pub residual_scale: f64,
}
impl LoessFit {
    fn influences(&self, grid: &[f64]) -> ChartResult<Vec<Vec<f64>>> {
        if self.controls.surface == Surface::Direct {
            return grid
                .iter()
                .map(|at| {
                    local(&self.x, &self.y, &self.prior, *at, self.controls)
                        .map(|l| l.2.into_iter().map(|v| v[0]).collect())
                })
                .collect();
        }
        let vertices = self
            .vertices
            .iter()
            .map(|v| local(&self.x, &self.y, &self.prior, v.0, self.controls).map(|v| v.2))
            .collect::<ChartResult<Vec<_>>>()?;
        Ok(grid
            .iter()
            .map(|at| {
                let i = self
                    .vertices
                    .partition_point(|v| v.0 < *at)
                    .clamp(1, self.vertices.len() - 1);
                (0..self.x.len())
                    .map(|j| {
                        hermite(
                            *at,
                            self.vertices[i - 1].0,
                            self.vertices[i].0,
                            vertices[i - 1][j][0],
                            vertices[i - 1][j][1],
                            vertices[i][j][0],
                            vertices[i][j][1],
                        )
                    })
                    .collect()
            })
            .collect())
    }
    pub fn uncertainty(&self, grid: &[f64], max_cells: usize) -> ChartResult<Uncertainty> {
        let n = self.x.len();
        if n.checked_mul(n.max(grid.len()))
            .is_none_or(|v| v > max_cells)
        {
            return Err(fail("LOESS influence matrix exceeds budget."));
        }
        let influence = self.influences(&self.x)?;
        let trace = if self.controls.approximate_trace
            && self.controls.surface == Surface::Interpolate
            && !self.controls.exact_statistics
        {
            (self.controls.degree + 1) as f64
                * (1. + ((1.09875 - self.controls.span) / self.controls.span).max(0.))
        } else {
            (0..n).map(|i| influence[i][i]).sum()
        };
        let (d1, d2) = if self.controls.exact_statistics {
            let residual: Vec<Vec<f64>> = influence
                .iter()
                .enumerate()
                .map(|(i, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(j, v)| v - f64::from(i == j))
                        .collect()
                })
                .collect();
            let mut first = 0.;
            let mut second = 0.;
            for i in 0..n {
                for j in 0..n {
                    let covariance = residual[i]
                        .iter()
                        .zip(&residual[j])
                        .map(|(a, b)| a * b)
                        .sum::<f64>();
                    if i == j {
                        first += covariance;
                    }
                    second += covariance * covariance;
                }
            }
            (first, second)
        } else {
            deltas(n, self.controls.degree, trace)
        };
        let fitted = self.predict(&self.x)?;
        let mut response = self.y.clone();
        if self.controls.family == Family::Symmetric {
            let mut absolute: Vec<_> = (0..n)
                .map(|i| (self.y[i] - fitted[i].unwrap()).abs() * libm::sqrt(self.prior[i]))
                .collect();
            absolute.sort_by(f64::total_cmp);
            let mad = if n.is_multiple_of(2) {
                (absolute[n / 2 - 1] + absolute[n / 2]) / 2.
            } else {
                absolute[n / 2]
            };
            let cutoff = 36. * mad * mad / 5.;
            let divisor = (0..n)
                .map(|i| {
                    let r = self.y[i] - fitted[i].unwrap();
                    (1. - r * r * self.prior[i] / cutoff) * libm::sqrt(self.robust[i])
                })
                .sum::<f64>();
            let c = n as f64 / divisor;
            for i in 0..n {
                let f = fitted[i].unwrap();
                response[i] = f + c * self.robust[i] * (self.y[i] - f);
            }
        }
        let rss = (0..n)
            .map(|i| {
                let f = influence[i]
                    .iter()
                    .zip(&response)
                    .map(|(a, b)| a * b)
                    .sum::<f64>();
                self.prior[i] * (response[i] - f) * (response[i] - f)
            })
            .sum::<f64>();
        let residual_scale = libm::sqrt(rss / d1);
        let mut predictions = self.predict(grid)?;
        let mut l = self.influences(grid)?;
        if self.controls.family == Family::Symmetric && self.controls.surface == Surface::Direct {
            predictions = grid
                .iter()
                .map(|at| {
                    local(&self.x, &self.y, &self.robust, *at, self.controls).map(|v| Some(v.0))
                })
                .collect::<ChartResult<_>>()?;
            l = grid
                .iter()
                .map(|at| {
                    local(&self.x, &self.y, &self.weights, *at, self.controls)
                        .map(|v| v.2.into_iter().map(|v| v[0]).collect())
                })
                .collect::<ChartResult<_>>()?;
        }
        let standard_errors = l
            .iter()
            .zip(&predictions)
            .map(|(row, p)| {
                p.map(|_| {
                    residual_scale
                        * libm::sqrt(row.iter().zip(&self.prior).map(|(v, w)| v * v / w).sum())
                })
            })
            .collect();
        Ok(Uncertainty {
            fitted: predictions,
            standard_errors,
            df: d1 * d1 / d2,
            residual_scale,
        })
    }
}
#[cfg(test)]
mod control_tests {
    use super::*;
    #[test]
    fn pinned_direct_interpolated_and_uncertainty_controls() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/loess-controls.json"
        ))
        .unwrap();
        let data = f["data"].as_array().unwrap();
        let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
        let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
        let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap()).collect();
        let grid: Vec<_> = f["grid"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        for (index, case) in f["cases"].as_array().unwrap().iter().enumerate() {
            let c = &case["controls"];
            let controls = Controls {
                degree: c["degree"].as_u64().unwrap() as usize,
                span: c["span"].as_f64().unwrap(),
                cell: c["cell"].as_f64().unwrap(),
                normalize: c["normalize"].as_bool().unwrap(),
                exact_statistics: c["statistics"] == "exact",
                approximate_trace: c["trace"] == "approximate",
                surface: if c["surface"] == "direct" {
                    Surface::Direct
                } else {
                    Surface::Interpolate
                },
                family: if c["family"] == "symmetric" {
                    Family::Symmetric
                } else {
                    Family::Gaussian
                },
                ..Default::default()
            };
            let fit = fit(&x, &y, &w, controls, 10000).unwrap();
            let uncertainty = fit.uncertainty(&grid, 2_000_000).unwrap();
            let mean_only = fit.predict(&grid).unwrap();
            for (i, value) in mean_only.iter().enumerate() {
                if let Some(value) = value {
                    assert!(
                        (value - case["result"]["mean_only"][i].as_f64().unwrap()).abs() < 1e-9,
                        "case {index} mean-only"
                    );
                } else {
                    assert!(case["result"]["mean_only"][i].is_null());
                }
            }
            let values = &uncertainty.fitted;
            let expected = &case["result"]["prediction"];
            for i in 0..grid.len() {
                if let Some(value) = values[i] {
                    assert!(
                        (value - expected["fit"][i].as_f64().unwrap()).abs() < 1e-9,
                        "case {index} mean {value} != {}",
                        expected["fit"][i]
                    );
                    assert!(
                        (uncertainty.standard_errors[i].unwrap()
                            - expected["se.fit"][i].as_f64().unwrap())
                        .abs()
                            < 1e-9,
                        "case {index} se {} != {}",
                        uncertainty.standard_errors[i].unwrap(),
                        expected["se.fit"][i]
                    );
                } else {
                    assert!(expected["fit"][i].is_null());
                }
            }
            assert!(
                (uncertainty.df - expected["df"].as_f64().unwrap()).abs() < 1e-8,
                "case {index} df {} != {}",
                uncertainty.df,
                expected["df"]
            );
            assert!(
                (uncertainty.residual_scale - case["result"]["scale"].as_f64().unwrap()).abs()
                    < 1e-9
            );
        }
    }
}
#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn checked_controls_and_influence_budgets() {
        let x: Vec<_> = (0..20).map(f64::from).collect();
        let y: Vec<_> = x.iter().map(|x| libm::sin(*x)).collect();
        let w = vec![1.; 20];
        assert!(fit(&x, &y, &w, Controls::default(), 2).is_err());
        assert!(
            fit(
                &x,
                &y,
                &w,
                Controls {
                    span: 0.01,
                    ..Default::default()
                },
                100
            )
            .is_err()
        );
        let fit = fit(&x, &y, &w, Controls::default(), 100).unwrap();
        assert!(fit.uncertainty(&[2.], 399).is_err());
        assert!(fit.predict(&[f64::INFINITY]).is_err());
    }
}
