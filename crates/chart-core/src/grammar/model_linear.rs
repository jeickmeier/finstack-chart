//! Weighted least squares using rank-revealing, twice-orthogonalized QR.
use crate::{ChartResult, DiagnosticCode};
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::NumericalDomain, message)
}
#[derive(Clone, Debug)]
pub(crate) struct LinearFit {
    pub coefficients: Vec<Option<f64>>,
    pub rank: usize,
    pub residual_df: usize,
    pub residual_variance: f64,
    pub covariance: Vec<f64>,
    aliases: Vec<(usize, Vec<f64>)>,
}
impl LinearFit {
    /// Mean and mean-estimation standard error; covariance excludes residual variance.
    pub fn predict(&self, row: &[f64], dispersion: f64) -> ChartResult<(f64, f64)> {
        let p = self.coefficients.len();
        if row.len() != p || row.iter().any(|x| !x.is_finite()) {
            return Err(invalid("Invalid prediction design."));
        }
        for (column, relation) in &self.aliases {
            let expected = row.iter().zip(relation).map(|(a, b)| a * b).sum::<f64>();
            if (row[*column] - expected).abs() > 1e-7 * (1. + row[*column].abs() + expected.abs()) {
                return Err(invalid(
                    "Prediction lies outside the estimable model subspace.",
                ));
            }
        }
        let value = row
            .iter()
            .zip(&self.coefficients)
            .map(|(x, b)| x * b.unwrap_or(0.))
            .sum::<f64>();
        let variance = (0..p)
            .map(|i| {
                (0..p)
                    .map(|j| row[i] * self.covariance[i * p + j] * row[j])
                    .sum::<f64>()
            })
            .sum::<f64>()
            * dispersion;
        Ok((
            value,
            libm::sqrt(if variance.is_nan() {
                f64::NAN
            } else {
                variance.max(0.)
            }),
        ))
    }
}
/// Positive weights contribute to residual degrees of freedom; zero weights do not.
pub(crate) fn fit(x: &[f64], y: &[f64], weights: &[f64], p: usize) -> ChartResult<LinearFit> {
    let n = y.len();
    if p == 0
        || p > 64
        || n == 0
        || n.checked_mul(p) != Some(x.len())
        || weights.len() != n
        || x.iter().chain(y).chain(weights).any(|v| !v.is_finite())
        || weights.iter().any(|w| *w < 0.)
    {
        return Err(invalid("Invalid weighted model inputs."));
    }
    let positive = weights.iter().filter(|w| **w > 0.).count();
    if positive == 0 {
        return Err(invalid("Model has no positive observation weights."));
    }
    let roots: Vec<_> = weights.iter().map(|w| libm::sqrt(*w)).collect();
    let wy: Vec<_> = y.iter().zip(&roots).map(|(y, w)| y * w).collect();
    let mut q: Vec<Vec<f64>> = vec![];
    let mut accepted = vec![];
    let mut r = vec![0.; p * p];
    for col in 0..p {
        let mut v: Vec<_> = (0..n).map(|i| x[i * p + col] * roots[i]).collect();
        let norm = libm::sqrt(v.iter().map(|v| v * v).sum());
        for _ in 0..2 {
            for (k, basis) in q.iter().enumerate() {
                let projection = v.iter().zip(basis).map(|(a, b)| a * b).sum::<f64>();
                r[k * p + col] += projection;
                for (v, b) in v.iter_mut().zip(basis) {
                    *v -= projection * b;
                }
            }
        }
        let remaining = libm::sqrt(v.iter().map(|v| v * v).sum());
        if !remaining.is_finite() || !norm.is_finite() {
            return Err(invalid("Weighted QR overflow."));
        }
        if remaining > norm * 1e-7 && remaining > 0. {
            r[q.len() * p + col] = remaining;
            for v in &mut v {
                *v /= remaining;
            }
            q.push(v);
            accepted.push(col);
        }
    }
    let rank = q.len();
    if rank == 0 {
        return Err(invalid("Model design has zero rank."));
    }
    let mut beta = vec![0.; rank];
    for i in (0..rank).rev() {
        beta[i] = (q[i].iter().zip(&wy).map(|(a, b)| a * b).sum::<f64>()
            - ((i + 1)..rank)
                .map(|j| r[i * p + accepted[j]] * beta[j])
                .sum::<f64>())
            / r[i * p + accepted[i]];
    }
    let mut coefficients = vec![None; p];
    for (i, &col) in accepted.iter().enumerate() {
        coefficients[col] = Some(beta[i]);
    }
    let rss = (0..n)
        .map(|i| {
            let residual = y[i]
                - (0..rank)
                    .map(|j| x[i * p + accepted[j]] * beta[j])
                    .sum::<f64>();
            weights[i] * residual * residual
        })
        .sum::<f64>();
    let residual_df = positive.saturating_sub(rank);
    let mut inverse = vec![0.; rank * rank];
    for col in 0..rank {
        for i in (0..rank).rev() {
            inverse[i * rank + col] = (f64::from(i == col)
                - ((i + 1)..rank)
                    .map(|j| r[i * p + accepted[j]] * inverse[j * rank + col])
                    .sum::<f64>())
                / r[i * p + accepted[i]];
        }
    }
    let mut aliases = vec![];
    for col in 0..p {
        if coefficients[col].is_none() {
            let mut relation = vec![0.; p];
            for i in (0..rank).rev() {
                relation[accepted[i]] = (r[i * p + col]
                    - ((i + 1)..rank)
                        .map(|j| r[i * p + accepted[j]] * relation[accepted[j]])
                        .sum::<f64>())
                    / r[i * p + accepted[i]];
            }
            aliases.push((col, relation));
        }
    }
    let mut covariance = vec![0.; p * p];
    for i in 0..rank {
        for j in 0..rank {
            covariance[accepted[i] * p + accepted[j]] = (0..rank)
                .map(|k| inverse[i * rank + k] * inverse[j * rank + k])
                .sum();
        }
    }
    if coefficients.iter().flatten().any(|v| !v.is_finite())
        || covariance.iter().any(|v| !v.is_finite())
        || !rss.is_finite()
    {
        return Err(invalid("Weighted model arithmetic overflow."));
    }
    Ok(LinearFit {
        coefficients,
        rank,
        residual_df,
        residual_variance: rss / residual_df as f64,
        covariance,
        aliases,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_weighted_polynomial_fits_and_standard_errors() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("lm-formula-"))
        {
            let data = case["controls"]["data"].as_array().unwrap();
            let formula = case["controls"]["formula"].as_str().unwrap();
            let design = |x: f64| {
                if formula.contains("0 +") {
                    vec![x]
                } else if formula.contains("x^2") {
                    vec![1., x, x * x]
                } else if formula.contains("2 * x") {
                    vec![1., x, 2. * x]
                } else {
                    vec![1., x]
                }
            };
            let x: Vec<_> = data
                .iter()
                .flat_map(|r| design(r["x"].as_f64().unwrap()))
                .collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap()).collect();
            let fit = fit(&x, &y, &w, x.len() / y.len()).unwrap();
            let expected = &case["result"]["value"];
            assert_eq!(fit.rank, expected["rank"].as_u64().unwrap() as usize);
            assert_eq!(fit.residual_df, expected["df"].as_u64().unwrap() as usize);
            for row in expected["prediction"].as_array().unwrap() {
                let (y, se) = fit
                    .predict(&design(row["x"].as_f64().unwrap()), fit.residual_variance)
                    .unwrap();
                assert!((y - row["y"].as_f64().unwrap()).abs() < 1e-10, "{formula}");
                assert!(
                    (se - row["se"].as_f64().unwrap()).abs() < 1e-10,
                    "{formula}"
                );
            }
        }
    }
}
#[cfg(test)]
mod edge_tests {
    use super::*;
    #[test]
    fn aliases_prediction_subspace_and_invalid_weights() {
        let x = [1., 0., 0., 1., 1., 2., 1., 2., 4.];
        let m = fit(&x, &[1., 2., 4.], &[1.; 3], 3).unwrap();
        assert_eq!(m.rank, 2);
        assert_eq!(m.coefficients[2], None);
        assert!(m.predict(&[1., 3., 6.], m.residual_variance).is_ok());
        assert!(m.predict(&[1., 3., 7.], m.residual_variance).is_err());
        assert!(fit(&x, &[1., 2., 4.], &[1., -1., 1.], 3).is_err());
        assert!(fit(&x, &[1., 2., 4.], &[0.; 3], 3).is_err());
        let saturated = fit(&[1., 0., 1., 1.], &[1., 2.], &[1.; 2], 2).unwrap();
        assert_eq!(saturated.residual_df, 0);
        assert!(
            !saturated
                .predict(&[1., 0.5], saturated.residual_variance)
                .unwrap()
                .1
                .is_finite()
        );
    }
}
