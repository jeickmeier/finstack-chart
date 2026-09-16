//! Bounded weighted quantile regression tableau, shared by distribution/model stats.
//! The mathematical pivot policy is qualified against external quantreg fixtures.
use crate::{ChartResult, DiagnosticCode};
#[derive(Clone, Debug)]
pub(crate) struct QuantileFit {
    pub coefficients: Vec<f64>,
    pub nonunique: bool,
}
fn fail(message: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::NumericalDomain, message)
}
/// Minimize weighted check loss using signed residual variables and a compact tableau.
/// `design` is row-major; zero-weight rows retain their original order.
pub(crate) fn fit(
    design: &[f64],
    response: &[f64],
    weights: &[f64],
    columns: usize,
    tau: f64,
    max_iterations: usize,
) -> ChartResult<QuantileFit> {
    let n = response.len();
    let p = columns;
    if p == 0
        || n < p
        || design.len()
            != n.checked_mul(p)
                .ok_or_else(|| fail("Quantile design size overflow."))?
        || weights.len() != n
        || !(0. ..=1.).contains(&tau)
        || design
            .iter()
            .chain(response)
            .chain(weights)
            .any(|v| !v.is_finite())
        || weights.iter().any(|v| *v < 0.)
    {
        return Err(fail(
            "Quantile design, finite observations, nonnegative weights and probability are required.",
        ));
    }
    // The formula wrapper checks weighted design rank before optimization.
    // Relative column residuals keep that decision independent of overall units.
    let magnitude = design
        .iter()
        .enumerate()
        .map(|(i, x)| x * weights[i / p])
        .fold(0_f64, |m, v| m.max(v.abs()));
    if !magnitude.is_finite() || magnitude == 0. {
        return Err(fail(
            "Quantile weighted design is singular or nonrepresentable.",
        ));
    }
    let mut orthogonal: Vec<Vec<f64>> = Vec::with_capacity(p);
    for column in 0..p {
        let mut v = (0..n)
            .map(|i| design[i * p + column] * weights[i] / magnitude)
            .collect::<Vec<_>>();
        let original = libm::sqrt(v.iter().map(|x| x * x).sum());
        for _ in 0..2 {
            for basis in &orthogonal {
                let projection = v.iter().zip(basis).map(|(a, b)| a * b).sum::<f64>();
                for (a, b) in v.iter_mut().zip(basis) {
                    *a -= projection * b;
                }
            }
        }
        let norm = libm::sqrt(v.iter().map(|x| x * x).sum());
        if norm <= 1e-7 * original || norm == 0. {
            return Err(fail("Quantile weighted design is rank deficient."));
        }
        for x in &mut v {
            *x /= norm;
        }
        orthogonal.push(v);
    }
    let tol = libm::pow(f64::EPSILON, 2. / 3.);
    let tau = if tau == 0. {
        tol
    } else if tau == 1. {
        1. - tol
    } else {
        tau
    };
    let mut rows = vec![vec![0.; p + 1]; n];
    let mut labels = Vec::with_capacity(n);
    let mut columns_id = (1..=p).map(|v| v as isize).collect::<Vec<_>>();
    let mut cost = vec![0.; p + 1];
    for i in 0..n {
        let b = response[i] * weights[i];
        if !b.is_finite() {
            return Err(fail("Weighted quantile response is not representable."));
        }
        let sign = if b < 0. { -1. } else { 1. };
        labels.push(sign as isize * (p + i + 1) as isize);
        for j in 0..p {
            let a = design[i * p + j] * weights[i];
            rows[i][j] = sign * a;
            cost[j] += 2. * a * (tau - if b < 0. { 1. } else { 0. });
        }
        rows[i][p] = b * sign;
    }
    if cost.iter().any(|v| !v.is_finite()) {
        return Err(fail("Quantile objective is not representable."));
    }
    let mut basic = 0;
    let mut iterations = 0;
    loop {
        let initializing = basic < p;
        let entering = if initializing {
            (0..p)
                .filter(|j| columns_id[*j].unsigned_abs() <= p)
                .max_by(|a, b| {
                    cost[*a]
                        .abs()
                        .total_cmp(&cost[*b].abs())
                        .then_with(|| b.cmp(a))
                })
        } else {
            (0..p)
                .filter_map(|j| {
                    let improvement = if cost[j] >= 0. {
                        cost[j]
                    } else {
                        -cost[j] - 2.
                    };
                    (improvement > tol).then_some((j, improvement))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
                .map(|v| v.0)
        };
        let Some(col) = entering else {
            break;
        };
        if cost[col] < 0. {
            for row in &mut rows {
                row[col] = -row[col];
            }
            cost[col] = -cost[col];
            columns_id[col] = -columns_id[col];
            if !initializing {
                cost[col] -= 2.;
            }
        }
        let mut candidates = (basic..n)
            .filter(|i| rows[*i][col] > tol)
            .collect::<Vec<_>>();
        let leaving = loop {
            let Some((slot, &idx)) = candidates.iter().enumerate().min_by(|a, b| {
                let ra = rows[*a.1][p] / rows[*a.1][col];
                let rb = rows[*b.1][p] / rows[*b.1][col];
                ra.total_cmp(&rb).then_with(|| a.0.cmp(&b.0))
            }) else {
                return Err(fail(
                    "Quantile design is rank deficient or has no bounded pivot.",
                ));
            };
            let pivot = rows[idx][col];
            if cost[col] <= 2. * pivot + tol {
                break idx;
            }
            for (j, c) in cost.iter_mut().enumerate() {
                *c -= 2. * rows[idx][j];
                rows[idx][j] = -rows[idx][j];
            }
            labels[idx] = -labels[idx];
            candidates.swap_remove(slot);
        };
        iterations += 1;
        if iterations > max_iterations {
            return Err(super::error(
                DiagnosticCode::ResourceLimit,
                "Quantile pivot iteration budget exceeded.",
            ));
        }
        let pivot = rows[leaving][col];
        let mut normalized = rows[leaving].clone();
        for (j, v) in normalized.iter_mut().enumerate() {
            if j != col {
                *v /= pivot;
            }
        }
        normalized[col] = 1. / pivot;
        for (i, row) in rows.iter_mut().enumerate() {
            if i == leaving {
                continue;
            }
            let factor = row[col];
            for j in 0..=p {
                if j != col {
                    row[j] -= factor * normalized[j];
                }
            }
            row[col] = -factor / pivot;
        }
        let factor = cost[col];
        for j in 0..=p {
            if j != col {
                cost[j] -= factor * normalized[j];
            }
        }
        cost[col] = -factor / pivot;
        if normalized.iter().chain(cost.iter()).any(|v| !v.is_finite()) {
            return Err(fail("Quantile pivot is not representable."));
        }
        rows[leaving] = normalized;
        std::mem::swap(&mut labels[leaving], &mut columns_id[col]);
        if initializing {
            rows.swap(leaving, basic);
            labels.swap(leaving, basic);
            basic += 1;
        }
    }
    if basic < p {
        return Err(fail("Quantile design is rank deficient."));
    }
    let mut coefficients = vec![0.; p];
    for i in 0..basic {
        let label = labels[i];
        if label.unsigned_abs() <= p {
            coefficients[label.unsigned_abs() - 1] = rows[i][p] * if label < 0 { -1. } else { 1. };
        }
    }
    if coefficients.iter().any(|v| !v.is_finite()) {
        return Err(fail("Quantile coefficients are not representable."));
    }
    Ok(QuantileFit {
        coefficients,
        nonunique: cost[..p]
            .iter()
            .any(|d| d.abs() <= tol || (2. - d.abs()) <= tol),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_intercept_ties_weights_and_permutations() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/weighted-intercept-quantiles.json"
        ))
        .unwrap();
        let vals = |v: &serde_json::Value| {
            v.as_array().map_or_else(
                || vec![v.as_f64().unwrap()],
                |a| a.iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>(),
            )
        };
        for c in f["cases"].as_array().unwrap() {
            let y = vals(&c["y"]);
            let w = vals(&c["weight"]);
            for (i, tau) in [0., 0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
                let result = fit(&vec![1.; y.len()], &y, &w, 1, tau, 1000);
                if c["result"]["ok"] == false {
                    assert!(result.is_err(), "{}", c["name"]);
                } else {
                    let result = result.unwrap_or_else(|e| panic!("{} tau{tau}: {e:?}", c["name"]));
                    assert!(
                        (result.coefficients[0] - c["result"]["quantiles"][i].as_f64().unwrap())
                            .abs()
                            < 1e-10,
                        "{} tau{tau}: {:?}",
                        c["name"],
                        result
                    );
                }
            }
        }
    }
    #[test]
    fn pinned_weighted_multivariate_br_objectives() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        for c in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("quantile-br-"))
        {
            let d = c["controls"]["data"].as_array().unwrap();
            let design = d
                .iter()
                .flat_map(|r| [1., r["x"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let y = d
                .iter()
                .map(|r| r["y"].as_f64().unwrap())
                .collect::<Vec<_>>();
            let w = d
                .iter()
                .map(|r| r["w"].as_f64().unwrap())
                .collect::<Vec<_>>();
            for row in c["result"]["value"]["fits"].as_array().unwrap() {
                let r = fit(&design, &y, &w, 2, row["tau"].as_f64().unwrap(), 1000).unwrap();
                assert!(
                    (loss(
                        &design,
                        &y,
                        &w,
                        2,
                        row["tau"].as_f64().unwrap(),
                        &r.coefficients
                    ) - row["objective"].as_f64().unwrap())
                    .abs()
                        < 1e-8,
                    "{:?}",
                    r
                );
                for (a, b) in r
                    .coefficients
                    .iter()
                    .zip(row["coefficients"].as_array().unwrap())
                {
                    assert!((a - b.as_f64().unwrap()).abs() < 1e-8, "{:?}", r);
                }
            }
        }
    }
}
#[cfg(test)]
mod additional_tests {
    use super::*;
    #[test]
    fn weighted_polynomial_designs_and_rank_deficiency() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/br-tableau-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let design_rows = case["design"].as_array().unwrap();
            let p = design_rows[0].as_array().unwrap().len();
            let design = design_rows
                .iter()
                .flat_map(|r| r.as_array().unwrap().iter().map(|v| v.as_f64().unwrap()))
                .collect::<Vec<_>>();
            let numbers = |key: &str| {
                case[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect::<Vec<_>>()
            };
            let result = fit(
                &design,
                &numbers("response"),
                &numbers("weight"),
                p,
                case["tau"].as_f64().unwrap(),
                10000,
            );
            if case["result"]["ok"] == false {
                assert!(result.is_err(), "{} {result:?}", case["name"]);
            } else {
                let result = result.unwrap();
                for (a, b) in result
                    .coefficients
                    .iter()
                    .zip(case["result"]["coefficients"].as_array().unwrap())
                {
                    assert!(
                        (a - b.as_f64().unwrap()).abs() < 1e-9,
                        "{} {result:?}",
                        case["name"]
                    );
                }
                assert!(
                    (loss(
                        &design,
                        &numbers("response"),
                        &numbers("weight"),
                        p,
                        case["tau"].as_f64().unwrap(),
                        &result.coefficients
                    ) - case["result"]["objective"].as_f64().unwrap())
                    .abs()
                        < 1e-9
                );
            }
        }
    }
}

#[cfg(test)]
fn loss(
    design: &[f64],
    response: &[f64],
    weights: &[f64],
    p: usize,
    tau: f64,
    coefficients: &[f64],
) -> f64 {
    response
        .iter()
        .enumerate()
        .map(|(i, y)| {
            let residual = y - design[i * p..(i + 1) * p]
                .iter()
                .zip(coefficients)
                .map(|(x, b)| x * b)
                .sum::<f64>();
            weights[i] * residual * (tau - if residual < 0. { 1. } else { 0. })
        })
        .sum()
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn rank_and_pivot_budgets_are_explicit() {
        assert!(fit(&[1., 1.], &[0., 1.], &[1., 1.], 1, 0.5, 0).is_err());
        assert!(fit(&[1., 1.], &[f64::MAX, 1.], &[2., 1.], 1, 0.5, 100).is_err());
        assert!(fit(&[1., 1., 1., 1.], &[0., 1.], &[1., 1.], 2, 0.5, 100).is_err());
    }
}
