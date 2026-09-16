//! Canonical-family iteratively reweighted least squares, sharing weighted QR.
use super::model_linear::{self, LinearFit};
use crate::{ChartResult, DiagnosticCode};
#[derive(Clone, Copy, Debug)]
pub(crate) enum Family {
    Gaussian,
    Binomial,
    Poisson,
}
impl Family {
    pub fn inverse(self, eta: f64) -> f64 {
        match self {
            Self::Gaussian => eta,
            Self::Poisson => libm::exp(eta).max(f64::EPSILON),
            Self::Binomial => {
                let e = libm::exp(eta.clamp(-30., 30.));
                (e / (1. + e)).clamp(f64::EPSILON, 1. - f64::EPSILON)
            }
        }
    }
    fn link(self, mu: f64) -> f64 {
        match self {
            Self::Gaussian => mu,
            Self::Binomial => libm::log(mu / (1. - mu)),
            Self::Poisson => libm::log(mu),
        }
    }
    fn variance(self, mu: f64) -> f64 {
        match self {
            Self::Gaussian => 1.,
            Self::Binomial => mu * (1. - mu),
            Self::Poisson => mu,
        }
    }
    fn deviance(self, y: &[f64], mu: &[f64], w: &[f64]) -> f64 {
        y.iter()
            .zip(mu)
            .zip(w)
            .map(|((&y, &m), &w)| {
                w * match self {
                    Self::Gaussian => (y - m) * (y - m),
                    Self::Poisson => {
                        2. * (if y == 0. {
                            m
                        } else {
                            y * libm::log(y / m) - (y - m)
                        })
                    }
                    Self::Binomial => {
                        2. * (if y == 0. { 0. } else { y * libm::log(y / m) }
                            + if y == 1. {
                                0.
                            } else {
                                (1. - y) * libm::log((1. - y) / (1. - m))
                            })
                    }
                }
            })
            .sum()
    }
}
#[derive(Clone, Debug)]
pub(crate) struct GlmFit {
    pub linear: LinearFit,
    pub family: Family,
    pub dispersion: f64,
    pub converged: bool,
    pub iterations: usize,
    pub boundary: bool,
}
impl GlmFit {
    pub fn predict_link(&self, row: &[f64]) -> ChartResult<(f64, f64)> {
        self.linear.predict(row, self.dispersion)
    }
}
pub(crate) fn fit(
    x: &[f64],
    y: &[f64],
    weights: &[f64],
    p: usize,
    family: Family,
    epsilon: f64,
    max_iterations: usize,
) -> ChartResult<GlmFit> {
    let fail = |s| super::error(DiagnosticCode::NumericalDomain, s);
    if !epsilon.is_finite()
        || epsilon <= 0.
        || max_iterations == 0
        || max_iterations > 10000
        || weights.len() != y.len()
        || weights.iter().any(|w| !w.is_finite() || *w < 0.)
        || y.iter().any(|y| {
            !y.is_finite()
                || match family {
                    Family::Binomial => !(0. ..=1.).contains(y),
                    Family::Poisson => *y < 0.,
                    Family::Gaussian => false,
                }
        })
    {
        return Err(fail("Invalid GLM controls or response."));
    }
    // Initial family means follow the canonical R-family initialization contract.
    let mut mu: Vec<_> = y
        .iter()
        .zip(weights)
        .map(|(&y, &w)| match family {
            Family::Gaussian => y,
            Family::Poisson => y + 0.1,
            Family::Binomial => (w * y + 0.5) / (w + 1.),
        })
        .collect();
    let mut eta: Vec<_> = mu.iter().map(|m| family.link(*m)).collect();
    let mut deviance = family.deviance(y, &mu, weights);
    let mut converged = false;
    let mut result = None;
    let mut iterations = 0;
    let mut boundary = false;
    let mut previous: Option<Vec<Option<f64>>> = None;
    for iteration in 1..=max_iterations {
        let derivative: Vec<_> = mu.iter().map(|m| family.variance(*m)).collect();
        let z: Vec<_> = (0..y.len())
            .map(|i| eta[i] + (y[i] - mu[i]) / derivative[i])
            .collect();
        let working: Vec<_> = (0..y.len()).map(|i| weights[i] * derivative[i]).collect();
        let mut fit = model_linear::fit(x, &z, &working, p)?;
        let mut next_eta;
        let mut next_mu;
        let mut next_deviance;
        let mut halving = 0;
        loop {
            next_eta = x
                .chunks_exact(p)
                .map(|row| {
                    row.iter()
                        .zip(&fit.coefficients)
                        .map(|(v, b)| v * b.unwrap_or(0.))
                        .sum::<f64>()
                })
                .collect::<Vec<_>>();
            next_mu = next_eta
                .iter()
                .map(|e| family.inverse(*e))
                .collect::<Vec<_>>();
            next_deviance = family.deviance(y, &next_mu, weights);
            if next_deviance.is_finite()
                && next_eta.iter().all(|v| v.is_finite())
                && next_mu.iter().all(|v| v.is_finite())
            {
                break;
            }
            let Some(previous) = &previous else {
                return Err(fail("No valid initial GLM step."));
            };
            if halving >= max_iterations {
                return Err(fail("GLM step halving failed to find finite deviance."));
            }
            for (value, old) in fit.coefficients.iter_mut().zip(previous) {
                if let (Some(value), Some(old)) = (value, old) {
                    *value = 0.5 * (*value + *old);
                }
            }
            halving += 1;
            boundary = true;
        }
        previous = Some(fit.coefficients.clone());
        iterations = iteration;
        converged = (next_deviance - deviance).abs() / (0.1 + next_deviance.abs()) < epsilon;
        eta = next_eta;
        mu = next_mu;
        deviance = next_deviance;
        result = Some(fit);
        if converged {
            break;
        }
    }
    let mut linear = result.unwrap();
    linear.residual_df = weights
        .iter()
        .filter(|w| **w > 0.)
        .count()
        .saturating_sub(linear.rank);
    let dispersion = match family {
        Family::Gaussian => deviance / linear.residual_df as f64,
        _ => 1.,
    };
    linear.residual_variance = dispersion;
    Ok(GlmFit {
        linear,
        family,
        dispersion,
        converged,
        iterations,
        boundary,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_glm_link_predictions_covariance_and_iterations() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
            c["name"].as_str().unwrap().starts_with("glm-") && c["name"] != "glm-nonconverged"
        }) {
            let data = case["controls"]["data"].as_array().unwrap();
            let family = match case["controls"]["family"].as_str().unwrap() {
                "gaussian" => Family::Gaussian,
                "binomial" => Family::Binomial,
                _ => Family::Poisson,
            };
            let x: Vec<_> = data
                .iter()
                .flat_map(|r| [1., r["x"].as_f64().unwrap()])
                .collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap()).collect();
            let fit = fit(&x, &y, &w, 2, family, 1e-8, 25).unwrap();
            let expected = &case["result"]["value"];
            assert!(fit.converged);
            assert_eq!(
                fit.iterations,
                expected["iterations"].as_u64().unwrap() as usize
            );
            for (i, x) in case["controls"]["xseq"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let (y, se) = fit.predict_link(&[1., x.as_f64().unwrap()]).unwrap();
                assert!(
                    (y - expected["link_prediction"]["fit"][i].as_f64().unwrap()).abs() < 1e-9,
                    "{} y {y}",
                    case["name"]
                );
                assert!(
                    (se - expected["link_prediction"]["se.fit"][i].as_f64().unwrap()).abs() < 1e-9,
                    "{} se {se}",
                    case["name"]
                );
                assert!(
                    (fit.family.inverse(y) - expected["built"]["values"][i]["y"].as_f64().unwrap())
                        .abs()
                        < 1e-9
                );
            }
        }
    }
}
#[cfg(test)]
mod edge_tests {
    use super::*;
    #[test]
    fn nonconvergence_is_retained_and_invalid_responses_fail() {
        let d = serde_json::from_str::<serde_json::Value>(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        let case = d["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "glm-nonconverged")
            .unwrap();
        let rows = case["controls"]["data"].as_array().unwrap();
        let x: Vec<_> = rows
            .iter()
            .flat_map(|r| [1., r["x"].as_f64().unwrap()])
            .collect();
        let y: Vec<_> = rows.iter().map(|r| r["y"].as_f64().unwrap()).collect();
        let w: Vec<_> = rows.iter().map(|r| r["w"].as_f64().unwrap()).collect();
        let m = fit(&x, &y, &w, 2, Family::Poisson, 1e-8, 1).unwrap();
        assert!(!m.converged);
        assert!(!m.boundary);
        assert_eq!(m.iterations, 1);
        for row in case["result"]["value"]["values"].as_array().unwrap() {
            let (eta, se) = m.predict_link(&[1., row["x"].as_f64().unwrap()]).unwrap();
            assert!((m.family.inverse(eta) - row["y"].as_f64().unwrap()).abs() < 1e-9);
            assert!((se - row["se"].as_f64().unwrap()).abs() < 1e-9);
        }
        assert!(fit(&x, &y, &w, 2, Family::Binomial, 1e-8, 25).is_err());
    }
}
