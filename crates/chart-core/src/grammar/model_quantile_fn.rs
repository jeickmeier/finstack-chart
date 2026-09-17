//! Weighted quantile regression via a primal-dual interior-point system.
//! The bounded dual is X' u = (1-tau) X' 1, 0 < u < 1.
use super::model_symmetric::{cholesky, solve};
use crate::{ChartResult, DiagnosticCode};
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::NumericalDomain, message)
}
#[derive(Debug)]
pub(crate) struct Fit {
    pub coefficients: Vec<f64>,
    pub iterations: usize,
    pub converged: bool,
}
struct Direction {
    beta: Vec<f64>,
    u: Vec<f64>,
    lower: Vec<f64>,
    upper: Vec<f64>,
}
#[allow(clippy::too_many_arguments)]
fn direction(
    x: &[f64],
    y: &[f64],
    p: usize,
    tau: f64,
    beta: &[f64],
    u: &[f64],
    lower: &[f64],
    upper: &[f64],
    target: f64,
    affine: Option<&Direction>,
) -> ChartResult<Direction> {
    let n = y.len();
    let mut inverse_d = vec![0.; n];
    let mut v = vec![0.; n];
    let mut rc_l = vec![0.; n];
    let mut rc_u = vec![0.; n];
    for i in 0..n {
        rc_l[i] = target - u[i] * lower[i];
        rc_u[i] = target - (1. - u[i]) * upper[i];
        if let Some(a) = affine {
            rc_l[i] -= a.u[i] * a.lower[i];
            rc_u[i] += a.u[i] * a.upper[i];
        }
        inverse_d[i] = 1. / (lower[i] / u[i] + upper[i] / (1. - u[i]));
        let residual =
            (0..p).map(|j| x[i * p + j] * beta[j]).sum::<f64>() - y[i] - lower[i] + upper[i];
        v[i] = -residual + rc_l[i] / u[i] - rc_u[i] / (1. - u[i]);
    }
    let mut h = vec![0.; p * p];
    let mut rhs = vec![0.; p];
    for j in 0..p {
        for i in 0..n {
            rhs[j] += x[i * p + j] * (u[i] - (1. - tau) + inverse_d[i] * v[i]);
        }
        for k in 0..p {
            h[j * p + k] = (0..n)
                .map(|i| x[i * p + j] * inverse_d[i] * x[i * p + k])
                .sum();
        }
    }
    let db = solve(&cholesky(&h, p)?, &rhs);
    let mut du = vec![0.; n];
    let mut dl = vec![0.; n];
    let mut dz = vec![0.; n];
    for i in 0..n {
        du[i] = inverse_d[i] * (v[i] - (0..p).map(|j| x[i * p + j] * db[j]).sum::<f64>());
        dl[i] = rc_l[i] / u[i] - lower[i] / u[i] * du[i];
        dz[i] = rc_u[i] / (1. - u[i]) + upper[i] / (1. - u[i]) * du[i];
    }
    Ok(Direction {
        beta: db,
        u: du,
        lower: dl,
        upper: dz,
    })
}
fn step(u: &[f64], lower: &[f64], upper: &[f64], d: &Direction) -> (f64, f64) {
    let mut primal: f64 = f64::INFINITY;
    let mut dual: f64 = f64::INFINITY;
    for i in 0..u.len() {
        if d.u[i] < 0. {
            primal = primal.min(-u[i] / d.u[i]);
        } else if d.u[i] > 0. {
            primal = primal.min((1. - u[i]) / d.u[i]);
        }
        if d.lower[i] < 0. {
            dual = dual.min(-lower[i] / d.lower[i]);
        }
        if d.upper[i] < 0. {
            dual = dual.min(-upper[i] / d.upper[i]);
        }
    }
    ((0.99995 * primal).min(1.), (0.99995 * dual).min(1.))
}
pub(crate) fn fit(
    design: &[f64],
    response: &[f64],
    weights: &[f64],
    p: usize,
    tau: f64,
    max_iterations: usize,
) -> ChartResult<Fit> {
    let n = response.len();
    if n == 0
        || !(1..=64).contains(&p)
        || n.checked_mul(p) != Some(design.len())
        || weights.len() != n
        || !tau.is_finite()
        || !(0. ..=1.).contains(&tau)
        || max_iterations == 0
        || max_iterations > 1_000_000
        || design.iter().chain(response).any(|v| !v.is_finite())
        || weights.iter().any(|w| !w.is_finite() || *w < 0.)
    {
        return Err(invalid("Invalid interior-point quantile input."));
    }
    let endpoint = libm::pow(f64::EPSILON, 2. / 3.);
    let tau = tau.clamp(endpoint, 1. - endpoint);
    let tolerance = 1e-6;
    let x: Vec<_> = design
        .chunks_exact(p)
        .zip(weights)
        .flat_map(|(row, w)| row.iter().map(move |v| v * w))
        .collect();
    let y: Vec<_> = response.iter().zip(weights).map(|(y, w)| y * w).collect();
    let mut h = vec![0.; p * p];
    let mut rhs = vec![0.; p];
    for j in 0..p {
        rhs[j] = (0..n).map(|i| x[i * p + j] * y[i]).sum();
        for k in 0..p {
            h[j * p + k] = (0..n).map(|i| x[i * p + j] * x[i * p + k]).sum();
        }
    }
    let mut beta = solve(&cholesky(&h, p)?, &rhs);
    let mut u = vec![1. - tau; n];
    let mut lower = vec![0.; n];
    let mut upper = vec![0.; n];
    for i in 0..n {
        let residual = (0..p).map(|j| x[i * p + j] * beta[j]).sum::<f64>() - y[i];
        let offset = if residual.abs() < tolerance {
            tolerance
        } else {
            0.
        };
        lower[i] = residual.max(0.) + offset;
        upper[i] = (-residual).max(0.) + offset;
    }
    let mut iterations = 0;
    let mut converged = false;
    for iteration in 0..max_iterations {
        let gap = (0..n)
            .map(|i| u[i] * lower[i] + (1. - u[i]) * upper[i])
            .sum::<f64>();
        if gap < tolerance {
            converged = true;
            break;
        }
        let mu = gap / (2 * n) as f64;
        let affine = direction(&x, &y, p, tau, &beta, &u, &lower, &upper, 0., None)?;
        let (ap, ad) = step(&u, &lower, &upper, &affine);
        let d = if ap.min(ad) < 1. {
            let affine_mu = (0..n)
                .map(|i| {
                    (u[i] + ap * affine.u[i]) * (lower[i] + ad * affine.lower[i])
                        + (1. - u[i] - ap * affine.u[i]) * (upper[i] + ad * affine.upper[i])
                })
                .sum::<f64>()
                / (2 * n) as f64;
            let ratio = (affine_mu / mu).clamp(0., 1.);
            direction(
                &x,
                &y,
                p,
                tau,
                &beta,
                &u,
                &lower,
                &upper,
                mu * ratio * ratio * ratio,
                Some(&affine),
            )?
        } else {
            affine
        };
        let (ap, ad) = step(&u, &lower, &upper, &d);
        for (coefficient, delta) in beta.iter_mut().zip(&d.beta) {
            *coefficient += ad * delta;
        }
        for i in 0..n {
            u[i] += ap * d.u[i];
            lower[i] += ad * d.lower[i];
            upper[i] += ad * d.upper[i];
        }
        iterations = iteration + 1;
    }
    if beta.iter().any(|v| !v.is_finite()) {
        return Err(invalid(
            "Interior-point quantile coefficients are not finite.",
        ));
    }
    Ok(Fit {
        coefficients: beta,
        iterations,
        converged,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_weighted_fn_coefficients_and_nonunique_selection() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        let mut count = 0;
        for case in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("quantile-fn-"))
        {
            let data = case["controls"]["data"].as_array().unwrap();
            let x: Vec<_> = data
                .iter()
                .flat_map(|r| [1., r["x"].as_f64().unwrap()])
                .collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap()).collect();
            for expected in case["result"]["value"]["fits"].as_array().unwrap() {
                let tau = expected["tau"].as_f64().unwrap();
                let fit = fit(&x, &y, &w, 2, tau, 500).unwrap();
                assert!(fit.converged);
                assert!(fit.iterations > 0);
                for (actual, expected) in fit
                    .coefficients
                    .iter()
                    .zip(expected["coefficients"].as_array().unwrap())
                {
                    assert!((actual - expected.as_f64().unwrap()).abs() < 1e-9);
                }
                let objective: f64 = y
                    .iter()
                    .zip(&w)
                    .enumerate()
                    .map(|(i, (y, w))| {
                        let residual = y - fit.coefficients[0] - x[2 * i + 1] * fit.coefficients[1];
                        w * residual * (if residual < 0. { tau - 1. } else { tau })
                    })
                    .sum();
                assert!((objective - expected["objective"].as_f64().unwrap()).abs() < 1e-9);
                count += 1;
            }
        }
        assert_eq!(count, 6);
    }
    #[test]
    fn invalid_rank_weights_and_iteration_limit() {
        assert!(fit(&[1., 1., 1., 1.], &[0., 1.], &[1., 1.], 2, 0.5, 500).is_err());
        assert!(fit(&[1., 1.], &[0., 1.], &[-1., 1.], 1, 0.5, 500).is_err());
        let fit = fit(
            &[1., 0., 1., 1., 1., 2., 1., 3.],
            &[0., 4., 1., 5.],
            &[1.; 4],
            2,
            0.25,
            1,
        )
        .unwrap();
        assert!(!fit.converged);
        assert_eq!(fit.iterations, 1);
    }
}
