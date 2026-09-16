//! Original symmetric reduction and close-eigenvalue-aware eigensystem.
//! Householder reduction separates rank-deficient trailing subspaces before rotations.
pub(crate) fn eigen(mut a: Vec<f64>, n: usize) -> crate::ChartResult<(Vec<f64>, Vec<f64>)> {
    if !(2..=64).contains(&n)
        || n.checked_mul(n) != Some(a.len())
        || a.iter().any(|v| !v.is_finite())
    {
        return Err(super::error(
            crate::DiagnosticCode::NumericalDomain,
            "Invalid symmetric eigensystem input.",
        ));
    }
    let mut q = vec![0.; n * n];
    for i in 0..n {
        q[i * n + i] = 1.;
    }
    for column in 0..n.saturating_sub(2) {
        let start = column + 1;
        let m = n - start;
        let mut v: Vec<_> = (start..n).map(|i| a[i * n + column]).collect();
        let tail = libm::sqrt(v[1..].iter().fold(0., |s, v| libm::fma(*v, *v, s)));
        let largest = v[0].abs().max(tail);
        let smallest = v[0].abs().min(tail);
        let norm = if largest == 0. {
            0.
        } else {
            largest * libm::sqrt(libm::fma(smallest / largest, smallest / largest, 1.))
        };
        if norm == 0. {
            continue;
        }
        let beta = -norm.copysign(v[0]);
        let tau = (beta - v[0]) / beta;
        let divisor = 1. / (v[0] - beta);
        for value in &mut v[1..] {
            *value *= divisor;
        }
        v[0] = 1.;
        let mut w = vec![0.; m];
        for j in 0..m {
            let scaled = tau * v[j];
            let mut dot = 0.;
            w[j] = libm::fma(scaled, a[(start + j) * n + start + j], w[j]);
            for i in j + 1..m {
                w[i] = libm::fma(scaled, a[(start + i) * n + start + j], w[i]);
                dot = libm::fma(a[(start + i) * n + start + j], v[i], dot);
            }
            w[j] = libm::fma(tau, dot, w[j]);
        }
        let correction = -0.5 * tau * w.iter().zip(&v).fold(0., |s, (a, b)| libm::fma(*a, *b, s));
        for i in 0..m {
            w[i] = libm::fma(correction, v[i], w[i]);
        }
        for i in 0..m {
            for j in i..m {
                let value = libm::fma(
                    w[j],
                    -v[i],
                    libm::fma(v[j], -w[i], a[(start + i) * n + start + j]),
                );
                a[(start + i) * n + start + j] = value;
                a[(start + j) * n + start + i] = value;
            }
        }
        a[start * n + column] = beta;
        a[column * n + start] = beta;
        for i in start + 1..n {
            a[i * n + column] = 0.;
            a[column * n + i] = 0.;
        }
        for row in 0..n {
            let dot = (0..m).map(|i| q[row * n + start + i] * v[i]).sum::<f64>();
            for i in 0..m {
                q[row * n + start + i] -= tau * dot * v[i];
            }
        }
    }

    let mut d: Vec<_> = (0..n).map(|i| a[i * n + i]).collect();
    let norm = d.iter().map(|v| v.abs()).fold(0., f64::max)
        + (0..n - 1)
            .map(|i| a[i * n + i + 1].abs())
            .fold(0., f64::max)
            * 2.;
    // Absolute deflation separates numerically disconnected representations.
    for i in 0..n - 1 {
        if a[i * n + i + 1].abs() <= f64::EPSILON * norm {
            a[i * n + i + 1] = 0.;
            a[(i + 1) * n + i] = 0.;
        }
    }
    let mut blocks = vec![];
    let mut start = 0;
    for end in 1..=n {
        if end == n || a[(end - 1) * n + end] == 0. {
            let diagonal: Vec<_> = (start..end).map(|i| a[i * n + i]).collect();
            let off: Vec<_> = (start..end - 1).map(|i| a[i * n + i + 1]).collect();
            let mut values = vec![];
            for rank in 0..end - start {
                let (mut lo, mut hi) = (-norm, norm);
                for _ in 0..256 {
                    let mid = (lo + hi) / 2.;
                    if mid == lo || mid == hi {
                        break;
                    }
                    let mut pivot = diagonal[0] - mid;
                    let mut count = usize::from(pivot < 0.);
                    for i in 1..diagonal.len() {
                        if pivot == 0. {
                            pivot = -f64::MIN_POSITIVE;
                        }
                        pivot = diagonal[i] - mid - off[i - 1] * off[i - 1] / pivot;
                        count += usize::from(pivot < 0.);
                    }
                    if count > rank {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                values.push((lo + hi) / 2.);
            }
            blocks.push((start, end, values));
            start = end;
        }
    }
    for _ in 0..100 * n * n {
        let mut pair = (0, 1);
        let mut largest = 0.;
        for i in 0..n {
            for j in i + 1..n {
                if a[i * n + j].abs() > largest {
                    largest = a[i * n + j].abs();
                    pair = (i, j);
                }
            }
        }
        if largest < 1e-15 * norm {
            break;
        }
        let (p, r) = pair;
        let angle = 0.5 * libm::atan2(2. * a[p * n + r], a[r * n + r] - a[p * n + p]);
        let c = libm::cos(angle);
        let s = libm::sin(angle);
        for i in 0..n {
            let x = a[i * n + p];
            let y = a[i * n + r];
            a[i * n + p] = c * x - s * y;
            a[i * n + r] = s * x + c * y;
        }
        for j in 0..n {
            let x = a[p * n + j];
            let y = a[r * n + j];
            a[p * n + j] = c * x - s * y;
            a[r * n + j] = s * x + c * y;
        }
        for i in 0..n {
            let x = q[i * n + p];
            let y = q[i * n + r];
            q[i * n + p] = c * x - s * y;
            q[i * n + r] = s * x + c * y;
        }
    }
    for (start, end, values) in blocks {
        let mut order: Vec<_> = (start..end).collect();
        order.sort_by(|i, j| a[i * n + i].total_cmp(&a[j * n + j]));
        for (i, value) in order.into_iter().zip(values) {
            d[i] = value;
        }
    }
    let mut order: Vec<_> = (0..n).collect();
    order.sort_by(|a, b| d[*a].total_cmp(&d[*b]));
    let values = order.iter().map(|i| d[*i]).collect();
    let vectors = (0..n)
        .flat_map(|i| order.iter().map(|j| q[i * n + j]).collect::<Vec<_>>())
        .collect();
    Ok((values, vectors))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_cluster_order_under_translations() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/gam-nullspace-controls.json"
        ))
        .unwrap();
        for case in f["cases"].as_array().unwrap() {
            let rows = case["raw"].as_array().unwrap();
            let n = rows.len();
            let raw: Vec<_> = rows
                .iter()
                .flat_map(|r| r.as_array().unwrap().iter().map(|v| v.as_f64().unwrap()))
                .collect();
            let (mut values, vectors) = eigen(raw.clone(), n).unwrap();
            for j in 0..n {
                let residual = (0..n)
                    .map(|i| {
                        let r = (0..n)
                            .map(|k| raw[i * n + k] * vectors[k * n + j])
                            .sum::<f64>()
                            - values[j] * vectors[i * n + j];
                        r * r
                    })
                    .sum::<f64>();
                assert!(residual < 1e-18);
            }
            values[1] = values[2] * 0.1;
            values[0] = values[2] * 0.01;
            for i in 0..n {
                for j in 0..n {
                    let actual = (0..n)
                        .map(|k| vectors[i * n + k] * values[k] * vectors[j * n + k])
                        .sum::<f64>();
                    let expected = case["shrunk"][i][j].as_f64().unwrap();
                    assert!(
                        (actual - expected).abs() < 1e-10,
                        "shift{} shrink delta{}",
                        case["shift"],
                        actual - expected
                    );
                }
            }
        }
    }
}
