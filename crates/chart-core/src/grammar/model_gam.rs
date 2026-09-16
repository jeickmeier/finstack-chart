//! Univariate Gaussian shrinkage cubic regression splines with REML selection.
use crate::{ChartResult, DiagnosticCode};
fn fail(s: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::NumericalDomain, s)
}
fn cholesky(a: &[f64], n: usize) -> ChartResult<Vec<f64>> {
    let mut l = vec![0.; n * n];
    for i in 0..n {
        for j in 0..=i {
            let v = a[i * n + j] - (0..j).map(|k| l[i * n + k] * l[j * n + k]).sum::<f64>();
            if i == j {
                if v <= 0. || !v.is_finite() {
                    return Err(fail("GAM matrix is not positive definite."));
                }
                l[i * n + j] = libm::sqrt(v);
            } else {
                l[i * n + j] = v / l[j * n + j];
            }
        }
    }
    Ok(l)
}
fn solve(l: &[f64], b: &[f64]) -> Vec<f64> {
    let n = b.len();
    let mut x = b.to_vec();
    for i in 0..n {
        x[i] = (x[i] - (0..i).map(|j| l[i * n + j] * x[j]).sum::<f64>()) / l[i * n + i];
    }
    for i in (0..n).rev() {
        x[i] = (x[i] - ((i + 1)..n).map(|j| l[j * n + i] * x[j]).sum::<f64>()) / l[i * n + i];
    }
    x
}
fn inverse(l: &[f64], n: usize) -> Vec<f64> {
    let mut a = vec![0.; n * n];
    for j in 0..n {
        let mut b = vec![0.; n];
        b[j] = 1.;
        for (i, v) in solve(l, &b).into_iter().enumerate() {
            a[i * n + j] = v;
        }
    }
    a
}
use super::model_symmetric::eigen;
#[derive(Clone, Debug)]
pub(crate) struct Controls {
    pub basis_dimension: usize,
    pub knots: Option<Vec<f64>>,
    pub iterations: usize,
    pub tolerance: f64,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            basis_dimension: 10,
            knots: None,
            iterations: 120,
            tolerance: 1e-9,
        }
    }
}
#[derive(Clone, Debug)]
struct Basis {
    knots: Vec<f64>,
    second: Vec<f64>,
}
impl Basis {
    fn new(knots: Vec<f64>) -> ChartResult<Self> {
        let k = knots.len();
        let q = k - 2;
        let h: Vec<_> = knots.windows(2).map(|v| v[1] - v[0]).collect();
        let mut diagonal: Vec<_> = (0..q).map(|i| (h[i] + h[i + 1]) / 3.).collect();
        let mut lower: Vec<_> = (1..q).map(|i| h[i] / 6.).collect();
        for i in 0..q - 1 {
            let off = lower[i];
            lower[i] /= diagonal[i];
            diagonal[i + 1] = libm::fma(-lower[i], off, diagonal[i + 1]);
        }
        let mut second = vec![0.; k * k];
        for column in 0..k {
            let mut response: Vec<_> = (0..q)
                .map(|i| {
                    if column == i {
                        1. / h[i]
                    } else if column == i + 1 {
                        -1. / h[i] - 1. / h[i + 1]
                    } else if column == i + 2 {
                        1. / h[i + 1]
                    } else {
                        0.
                    }
                })
                .collect();
            for i in 1..q {
                response[i] = libm::fma(-lower[i - 1], response[i - 1], response[i]);
            }
            response[q - 1] /= diagonal[q - 1];
            for i in (0..q - 1).rev() {
                response[i] = libm::fma(-lower[i], response[i + 1], response[i] / diagonal[i]);
            }
            for i in 0..q {
                second[(i + 1) * k + column] = response[i];
            }
        }
        Ok(Self { knots, second })
    }
    fn row(&self, x: f64) -> Vec<f64> {
        let k = self.knots.len();
        let i = self.knots.partition_point(|v| *v < x).clamp(1, k - 1) - 1;
        let h = self.knots[i + 1] - self.knots[i];
        let t = (x - self.knots[i]) / h;
        (0..k)
            .map(|j| {
                if t < 0. {
                    f64::from(j == 0)
                        + (x - self.knots[0])
                            * ((f64::from(j == 1) - f64::from(j == 0)) / h
                                - h * self.second[k + j] / 6.)
                } else if t > 1. {
                    f64::from(j == k - 1)
                        + (x - self.knots[k - 1])
                            * ((f64::from(j == k - 1) - f64::from(j == k - 2)) / h
                                + h * self.second[(k - 2) * k + j] / 6.)
                } else {
                    let a = 1. - t;
                    a * f64::from(j == i)
                        + t * f64::from(j == i + 1)
                        + h * h / 6.
                            * ((a * a * a - a) * self.second[i * k + j]
                                + (t * t * t - t) * self.second[(i + 1) * k + j])
                }
            })
            .collect()
    }
    fn penalty(&self) -> Vec<f64> {
        let k = self.knots.len();
        let mut s = vec![0.; k * k];
        let h: Vec<_> = self.knots.windows(2).map(|v| v[1] - v[0]).collect();
        for i in 0..k {
            for j in 0..k {
                let mut terms = vec![];
                if i >= 2 {
                    terms.push((self.second[(i - 1) * k + j], 1. / h[i - 1]));
                }
                if i >= 1 && i < k - 1 {
                    let coefficient = -1. / h[i - 1] - 1. / h[i];
                    terms.push((self.second[i * k + j], coefficient));
                }
                if i < k - 2 {
                    terms.push((self.second[(i + 1) * k + j], 1. / h[i]));
                }
                s[i * k + j] = match terms.as_slice() {
                    [(a, b)] => a * b,
                    [(a, b), (c, d)] => libm::fma(*a, *b, c * d),
                    [(a, b), (c, d), (e, f)] => libm::fma(*e, *f, libm::fma(*a, *b, c * d)),
                    _ => 0.,
                };
            }
        }
        for i in 0..k {
            for j in 0..i {
                let v = (s[i * k + j] + s[j * k + i]) / 2.;
                s[i * k + j] = v;
                s[j * k + i] = v;
            }
        }
        s
    }
}
#[derive(Clone, Debug)]
pub(crate) struct GamFit {
    basis: Basis,
    constraint: Vec<f64>,
    coefficients: Vec<f64>,
    covariance: Vec<f64>,
    pub smoothing_parameter: f64,
    pub residual_variance: f64,
    pub effective_df: f64,
    pub converged: bool,
}
fn constrained(b: &[f64], c: &[f64]) -> Vec<f64> {
    let k = b.len();
    std::iter::once(1.)
        .chain((0..k - 1).map(|i| b[i] - b[k - 1] * c[i] / c[k - 1]))
        .collect()
}
impl GamFit {
    pub fn predict(&self, grid: &[f64]) -> ChartResult<Vec<(f64, f64)>> {
        let k = self.coefficients.len();
        grid.iter()
            .map(|x| {
                if !x.is_finite() {
                    return Err(fail("Invalid GAM prediction grid."));
                }
                let b = constrained(&self.basis.row(*x), &self.constraint);
                let y = b.iter().zip(&self.coefficients).map(|(a, b)| a * b).sum();
                let variance = (0..k)
                    .map(|i| {
                        (0..k)
                            .map(|j| b[i] * self.covariance[i * k + j] * b[j])
                            .sum::<f64>()
                    })
                    .sum::<f64>();
                Ok((y, libm::sqrt(variance.max(0.))))
            })
            .collect()
    }
}
pub(crate) fn fit(
    x: &[f64],
    y: &[f64],
    w: &[f64],
    controls: Controls,
    max_cells: usize,
) -> ChartResult<GamFit> {
    let n = x.len();
    let k = controls.basis_dimension;
    if k < 3
        || k > 64
        || n <= k
        || y.len() != n
        || w.len() != n
        || x.iter().chain(y).chain(w).any(|v| !v.is_finite())
        || w.iter().any(|v| *v < 0.)
        || controls.iterations == 0
        || controls.iterations > 1000
        || !controls.tolerance.is_finite()
        || controls.tolerance <= 0.
    {
        return Err(fail("Invalid canonical GAM inputs or controls."));
    }
    if y.iter().all(|v| *v == y[0]) {
        return Err(fail(
            "GAM REML cannot estimate scale from a constant response.",
        ));
    }
    if n.checked_mul(k).is_none_or(|v| v > max_cells) {
        return Err(fail("GAM design exceeds allocation budget."));
    }
    let mut unique = x.to_vec();
    unique.sort_by(f64::total_cmp);
    unique.dedup();
    if unique.len() < k {
        return Err(fail("Insufficient unique values for GAM knots."));
    }
    let knots = if let Some(knots) = controls.knots {
        knots
    } else {
        (0..k)
            .map(|i| super::distributions::quantile(&unique, i as f64 * (1. / (k - 1) as f64), 7))
            .collect::<ChartResult<_>>()?
    };
    if knots.len() != k
        || knots.iter().any(|v| !v.is_finite())
        || knots.windows(2).any(|v| v[0] >= v[1])
    {
        return Err(fail("Invalid GAM knots."));
    }
    let basis = Basis::new(knots)?;
    let raw: Vec<_> = x.iter().map(|x| basis.row(*x)).collect();
    let constraint: Vec<_> = (0..k)
        .map(|i| raw.iter().map(|r| r[i]).sum::<f64>() / n as f64)
        .collect();
    let design: Vec<_> = raw.iter().map(|r| constrained(r, &constraint)).collect();
    let (mut eigenvalues, vectors) = eigen(basis.penalty(), k)?;
    eigenvalues[1] = eigenvalues[2] * 0.1;
    eigenvalues[0] = eigenvalues[2] * 0.01;
    let shrink: Vec<_> = (0..k)
        .flat_map(|i| {
            (0..k)
                .map(|j| {
                    (0..k)
                        .map(|a| vectors[i * k + a] * eigenvalues[a] * vectors[j * k + a])
                        .sum::<f64>()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let mut penalty = vec![0.; k * k];
    for i in 1..k {
        for j in 1..k {
            let a = constraint[i - 1] / constraint[k - 1];
            let b = constraint[j - 1] / constraint[k - 1];
            penalty[i * k + j] = shrink[(i - 1) * k + j - 1]
                - a * shrink[(k - 1) * k + j - 1]
                - b * shrink[(i - 1) * k + k - 1]
                + a * b * shrink[k * k - 1];
        }
    }
    let mut cross = vec![0.; k * k];
    let mut rhs = vec![0.; k];
    for r in 0..n {
        for i in 0..k {
            rhs[i] += w[r] * design[r][i] * y[r];
            for j in 0..k {
                cross[i * k + j] += w[r] * design[r][i] * design[r][j];
            }
        }
    }
    let objective = |log_lambda: f64| -> ChartResult<(f64, Vec<f64>, Vec<f64>, f64, f64)> {
        let lambda = libm::exp(log_lambda);
        let matrix: Vec<_> = cross
            .iter()
            .zip(&penalty)
            .map(|(x, s)| x + lambda * s)
            .collect();
        let l = cholesky(&matrix, k)?;
        let beta = solve(&l, &rhs);
        let rss = (0..n)
            .map(|r| {
                let d = y[r] - design[r].iter().zip(&beta).map(|(x, b)| x * b).sum::<f64>();
                w[r] * d * d
            })
            .sum::<f64>();
        let roughness = (0..k)
            .map(|i| {
                (0..k)
                    .map(|j| beta[i] * penalty[i * k + j] * beta[j])
                    .sum::<f64>()
            })
            .sum::<f64>();
        let variance = (rss + lambda * roughness) / (n - 1) as f64;
        let criterion = (n - 1) as f64 * libm::log(variance)
            + 2. * (0..k).map(|i| libm::log(l[i * k + i])).sum::<f64>()
            - (k - 1) as f64 * log_lambda;
        let inv = inverse(&l, k);
        let trace = (0..k)
            .map(|i| {
                (0..k)
                    .map(|j| inv[i * k + j] * penalty[j * k + i])
                    .sum::<f64>()
            })
            .sum::<f64>();
        let derivative = -((k - 1) as f64) + lambda * trace + lambda * roughness / variance;
        Ok((criterion, beta, l, variance, derivative))
    };
    let (mut lo, mut hi) = (-100., 100.);
    for _ in 0..controls.iterations {
        if hi - lo < controls.tolerance {
            break;
        }
        let mid = (lo + hi) / 2.;
        if mid == lo || mid == hi {
            break;
        }
        if objective(mid)?.4 > 0. {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let log_lambda = (lo + hi) / 2.;
    let (_, coefficients, l, residual_variance, _) = objective(log_lambda)?;
    let inv = inverse(&l, k);
    let effective_df = (0..k)
        .map(|i| {
            (0..k)
                .map(|j| inv[i * k + j] * cross[j * k + i])
                .sum::<f64>()
        })
        .sum();
    let covariance = inv.iter().map(|v| v * residual_variance).collect();
    Ok(GamFit {
        basis,
        constraint,
        coefficients,
        covariance,
        smoothing_parameter: libm::exp(log_lambda),
        residual_variance,
        effective_df,
        converged: hi - lo < controls.tolerance,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_canonical_reml() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/gam-controls.json"
        ))
        .unwrap();
        let grid: Vec<_> = f["grid"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        for case in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"] != "constant" && c["name"] != "insufficient")
        {
            let data = case["data"].as_array().unwrap();
            let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap()).collect();
            let knots = case["result"]["knots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let basis = Basis::new(knots).unwrap();
            let k = basis.knots.len();
            let mut second_delta: f64 = 0.;
            for i in 0..k {
                for j in 0..k {
                    let reference = case["result"]["raw_F"][i * k + j].as_f64().unwrap();
                    second_delta = second_delta.max((basis.second[i * k + j] - reference).abs());
                }
            }
            assert!(second_delta < 1e-12);
            let (mut values, vectors) = eigen(basis.penalty(), k).unwrap();
            values[1] = values[2] * 0.1;
            values[0] = values[2] * 0.01;
            let scale = case["result"]["penalty_scale"].as_f64().unwrap();
            let mut delta: f64 = 0.;
            for i in 0..k {
                for j in 0..k {
                    let actual = (0..k)
                        .map(|a| vectors[i * k + a] * values[a] * vectors[j * k + a])
                        .sum::<f64>();
                    let expected = case["result"]["penalty"][i][j].as_f64().unwrap() * scale;
                    delta = delta.max((actual - expected).abs());
                }
            }
            assert!(delta < 1e-10, "penalty delta {delta}");
            let m = fit(
                &x,
                &y,
                &w,
                Controls {
                    basis_dimension: case["k"].as_u64().unwrap() as usize,

                    ..Default::default()
                },
                10000,
            )
            .unwrap();
            let p = m.predict(&grid).unwrap();
            assert!(m.converged);
            assert!(m.smoothing_parameter.is_finite());
            if case["name"] != "linear" {
                let reference = case["result"]["sp"].as_f64().unwrap()
                    / case["result"]["model_scale"].as_f64().unwrap();
                assert!(
                    (m.smoothing_parameter - reference).abs() < 1e-7 * reference.abs(),
                    "smoothing selection"
                );
            }
            assert!((m.residual_variance - case["result"]["scale"].as_f64().unwrap()).abs() < 1e-8);
            assert!((m.effective_df - case["result"]["edf"].as_f64().unwrap()).abs() < 1e-7);
            for (i, (y, se)) in p.iter().enumerate() {
                assert!(
                    (y - case["result"]["prediction"]["fit"][i].as_f64().unwrap()).abs() < 1e-7,
                    "{} mean {} !={}",
                    case["name"],
                    y,
                    case["result"]["prediction"]["fit"][i]
                );
                assert!(
                    (se - case["result"]["prediction"]["se.fit"][i].as_f64().unwrap()).abs() < 1e-7
                );
            }
        }
    }
}
#[cfg(test)]
mod automatic_tests {
    use super::*;
    #[test]
    fn pinned_automatic_large_group_route() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        let case = f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "automatic-1000")
            .unwrap();
        let data = case["controls"]["data"].as_array().unwrap();
        let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
        let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
        let m = fit(&x, &y, &vec![1.; x.len()], Controls::default(), 100000).unwrap();
        let expected = case["result"]["value"]["values"].as_array().unwrap();
        let grid: Vec<_> = expected.iter().map(|r| r["x"].as_f64().unwrap()).collect();
        for ((y, se), expected) in m.predict(&grid).unwrap().iter().zip(expected) {
            assert!(
                (y - expected["y"].as_f64().unwrap()).abs() < 1e-7,
                "mean{y} !={}",
                expected["y"]
            );
            assert!((se - expected["se"].as_f64().unwrap()).abs() < 1e-7);
        }
    }
}
#[cfg(test)]
mod invalid_tests {
    use super::*;
    #[test]
    fn source_degeneracy_and_budgets() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/gam-controls.json"
        ))
        .unwrap();
        for case in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["result"]["ok"] == false)
        {
            let data = case["data"].as_array().unwrap();
            let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w = vec![1.; x.len()];
            assert!(fit(&x, &y, &w, Controls::default(), 100000).is_err());
        }
        let x: Vec<_> = (0..20).map(f64::from).collect();
        let y: Vec<_> = x.iter().map(|x| libm::sin(*x)).collect();
        let mut weights = vec![1.; 20];
        assert!(fit(&x, &y, &weights, Controls::default(), 199).is_err());
        weights[0] = -1.;
        assert!(fit(&x, &y, &weights, Controls::default(), 1000).is_err());
    }
}
