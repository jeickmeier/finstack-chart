//! Bounded reference univariate kernels, independent of datasets and renderers.
use super::*;
use crate::{ChartResult, DiagnosticCode};

fn invalid(message: &str) -> crate::Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
fn bounded(n: usize, limit: usize) -> ChartResult<()> {
    if n > limit {
        Err(error(
            DiagnosticCode::ResourceLimit,
            "Univariate output budget exceeded.",
        ))
    } else {
        Ok(())
    }
}
pub(super) fn grid(range: [f64; 2], n: usize, limit: usize) -> ChartResult<Vec<f64>> {
    bounded(n, limit)?;
    if range.iter().any(|x| !x.is_finite()) {
        return Err(invalid("Function grid requires finite endpoints."));
    }
    Ok((0..n)
        .map(|i| {
            if n <= 1 {
                range[0]
            } else if i + 1 == n {
                range[1]
            } else {
                let t = i as f64 / (n - 1) as f64;
                range[0] * (1. - t) + range[1] * t
            }
        })
        .collect())
}
pub(super) struct EcdfResult {
    pub points: Vec<[f64; 2]>,
    pub replaced_weights: usize,
    pub near_zero: bool,
}
pub(super) fn ecdf(
    values: &[f64],
    weights: Option<&[Option<f64>]>,
    n: Option<usize>,
    pad: bool,
    limit: usize,
) -> ChartResult<EcdfResult> {
    if weights.is_some_and(|w| w.len() != values.len()) || values.iter().any(|v| !v.is_finite()) {
        return Err(invalid("ECDF requires aligned finite samples."));
    }
    if values.is_empty() {
        return Ok(EcdfResult {
            points: vec![],
            replaced_weights: 0,
            near_zero: false,
        });
    }
    let mut replaced = 0;
    let weights: Vec<_> = (0..values.len())
        .map(|i| match weights {
            None => 1.,
            Some(w) => w[i].filter(|v| v.is_finite()).unwrap_or_else(|| {
                replaced += 1;
                0.
            }),
        })
        .collect();
    let total = super::statistics::sum(weights.iter().copied())?;
    if total == 0. {
        return Err(invalid("ECDF total weight must be nonzero."));
    }
    let mut query = if let Some(n) = n {
        grid(
            [
                values.iter().copied().fold(f64::INFINITY, f64::min),
                values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ],
            n,
            limit,
        )?
    } else {
        let mut query = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for &value in values {
            if seen.insert(if value == 0. { 0 } else { value.to_bits() }) {
                query.push(value);
            }
        }
        bounded(query.len(), limit)?;
        query
    };
    if pad {
        bounded(
            query
                .len()
                .checked_add(2)
                .ok_or_else(|| invalid("ECDF grid overflow."))?,
            limit,
        )?;
        query.insert(0, f64::NEG_INFINITY);
        query.push(f64::INFINITY);
    }
    let mut order: Vec<_> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut x = Vec::new();
    let mut prefix = Vec::new();
    let mut sum = 0.;
    for i in order {
        sum += weights[i];
        if x.last() == Some(&values[i]) {
            *prefix.last_mut().unwrap() = sum / total;
        } else {
            x.push(values[i]);
            prefix.push(sum / total);
        }
    }
    let points = query
        .into_iter()
        .map(|v| {
            let k = x.partition_point(|x| *x <= v);
            [v, if k == 0 { 0. } else { prefix[k - 1] }]
        })
        .collect();
    Ok(EcdfResult {
        points,
        replaced_weights: replaced,
        near_zero: total.abs() < f64::EPSILON.sqrt(),
    })
}
pub(super) fn plotting_positions(n: usize) -> Vec<f64> {
    let a = if n <= 10 { 0.375 } else { 0.5 };
    (1..=n)
        .map(|i| (i as f64 - a) / (n as f64 + 1. - 2. * a))
        .collect()
}
pub(super) fn connect(
    input: &[[f64; 2]],
    fractions: &[[f64; 2]],
    limit: usize,
) -> ChartResult<Vec<[f64; 2]>> {
    if input.len() < 2 {
        return Ok(vec![]);
    }
    if fractions.is_empty() || fractions.iter().flatten().any(|x| !x.is_finite()) {
        return Err(invalid(
            "Connection matrix requires finite coordinate pairs.",
        ));
    }
    bounded(
        (input.len() - 1)
            .checked_mul(fractions.len())
            .and_then(|v| v.checked_add(2))
            .ok_or_else(|| invalid("Connection size overflow."))?,
        limit,
    )?;
    let mut data = input.to_vec();
    data.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut result = Vec::new();
    if fractions[0] != [0., 0.] {
        result.push(data[0]);
    }
    for pair in data.windows(2) {
        for [fx, fy] in fractions {
            result.push([
                pair[0][0] * (1. - fx) + pair[1][0] * fx,
                pair[0][1] * (1. - fy) + pair[1][1] * fy,
            ]);
        }
    }
    if fractions.last() != Some(&[1., 1.]) {
        result.push(*data.last().unwrap());
    }
    Ok(result)
}
pub(super) type AlignedGroup = Vec<([f64; 2], bool)>;
pub(super) fn align(groups: &[Vec<[f64; 2]>], limit: usize) -> ChartResult<Vec<AlignedGroup>> {
    if groups.len() <= 1 {
        return Ok(groups
            .iter()
            .map(|g| g.iter().copied().map(|p| (p, false)).collect())
            .collect());
    }
    let mut locations: Vec<f64> = groups.iter().flatten().map(|v| v[0]).collect();
    for group in groups {
        for p in group.windows(2) {
            if (p[0][1] < 0.) != (p[1][1] < 0.) {
                let crossing = -p[0][1] * (p[1][0] - p[0][0]) / (p[1][1] - p[0][1]) + p[0][0];
                if crossing.is_finite() {
                    locations.push(crossing);
                }
            }
        }
    }
    locations.sort_by(f64::total_cmp);
    locations.dedup();
    if locations.len() < 2 {
        return Ok(vec![vec![]; groups.len()]);
    }
    let adjust = ((locations[locations.len() - 1] - locations[0]) * 0.001).min(
        locations
            .windows(2)
            .map(|x| (x[1] - x[0]) / 3.)
            .fold(f64::INFINITY, f64::min),
    );
    bounded(
        locations
            .len()
            .checked_mul(3)
            .ok_or_else(|| invalid("Alignment grid overflow."))?,
        limit,
    )?;
    let mut query: Vec<_> = locations
        .into_iter()
        .flat_map(|x| [x - adjust, x, x + adjust])
        .collect();
    query.sort_by(f64::total_cmp);
    query.dedup();
    let mut output = Vec::new();
    let mut remaining = limit;
    for group in groups {
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| a[0].total_cmp(&b[0]));
        if sorted.first().map(|x| x[0]) == sorted.last().map(|x| x[0]) {
            output.push(vec![]);
            continue;
        }
        let mut points = Vec::new();
        let mut i = 0;
        while i < sorted.len() {
            let mut end = i + 1;
            while end < sorted.len() && sorted[end][0] == sorted[i][0] {
                end += 1;
            }
            if end - i > 1 {
                points.push([sorted[i][0] - adjust, sorted[i][1]]);
            }
            points.push(sorted[end - 1]);
            i = end;
        }
        points.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let mut result = Vec::new();
        for &x in &query {
            if x < points[0][0] || x > points[points.len() - 1][0] {
                continue;
            }
            let k = points.partition_point(|p| p[0] < x);
            let y = if k < points.len() && points[k][0] == x {
                points[k][1]
            } else {
                let a = points[k - 1];
                let b = points[k];
                a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0])
            };
            result.push(([x, y], false));
        }
        if !result.is_empty() {
            let left = result[0].0[0] - adjust;
            let right = result.last().unwrap().0[0] + adjust;
            result.insert(0, ([left, 0.], true));
            result.push(([right, 0.], true));
        }
        remaining = remaining.checked_sub(result.len()).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Aligned output exceeds row budget.",
            )
        })?;
        output.push(result);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn number(v: &serde_json::Value) -> f64 {
        v.as_f64().unwrap_or_else(|| match v.as_str().unwrap() {
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            _ => f64::NAN,
        })
    }
    #[test]
    fn gg09_ecdf_matches_pinned_weighted_and_degenerate_cases() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/univariate-controls.json"
        ))
        .unwrap();
        let mut checked = 0;
        for case in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["family"] == "ecdf")
        {
            let rows = case["input"].as_array().unwrap();
            let x: Vec<_> = rows.iter().map(|r| number(&r["x"])).collect();
            let weights: Vec<_> = rows.iter().map(|r| r["w"].as_f64()).collect();
            let weighted = case["params"]["weighted"].as_bool().unwrap();
            let n = case["params"]["n"].as_u64().map(|v| v as usize);
            let result = ecdf(
                &x,
                weighted.then_some(weights.as_slice()),
                n,
                case["params"]["pad"].as_bool().unwrap(),
                10000,
            );
            let Some(expected) = case["rows"].as_array() else {
                assert!(result.is_err(), "{}", case["name"]);
                checked += 1;
                continue;
            };
            if expected.is_empty() {
                assert!(result.is_err(), "{}", case["name"]);
                assert!(
                    case["warnings"].is_string()
                        || case["warnings"].as_array().is_some_and(|w| !w.is_empty())
                );
                checked += 1;
                continue;
            }
            let result = result.unwrap();
            assert_eq!(result.points.len(), expected.len(), "{}", case["name"]);
            for (actual, expected) in result.points.iter().zip(expected) {
                for (v, key) in actual.iter().zip(["x", "y"]) {
                    let want = number(&expected[key]);
                    assert!(
                        *v == want || (*v - want).abs() <= 1e-12 * want.abs().max(1.),
                        "{} {key}: {v} vs {want}",
                        case["name"]
                    );
                }
            }
            checked += 1;
        }
        assert_eq!(checked, 12);
        assert!(ecdf(&[1., 2.], Some(&[Some(0.), Some(0.)]), None, false, 10).is_err());
        assert!(ecdf(&[1., 2.], None, Some(3), true, 4).is_err());
    }
    #[test]
    fn gg09_alignment_matches_pinned_groups_and_duplicate_x() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/univariate-controls.json"
        ))
        .unwrap();
        let mut checked = 0;
        for case in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["family"] == "align")
        {
            let rows = case["input"].as_array().unwrap();
            let groups: Vec<Vec<[f64; 2]>> = ["A", "B"]
                .iter()
                .map(|g| {
                    rows.iter()
                        .filter(|r| r["g"] == *g)
                        .map(|r| [number(&r["x"]), number(&r["y"])])
                        .collect()
                })
                .collect();
            let result = align(&groups, 10000).unwrap();
            let actual: Vec<_> = result.iter().flatten().collect();
            let expected = case["rows"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{}", case["name"]);
            for ((point, padding), want) in actual.iter().zip(expected) {
                for (v, key) in point.iter().zip(["x", "y"]) {
                    assert!(
                        (*v - number(&want[key])).abs() < 1e-11,
                        "{} {key}: {v} vs {}",
                        case["name"],
                        want[key]
                    );
                }
                assert_eq!(*padding, want["align_padding"].as_bool().unwrap());
            }
            checked += 1;
        }
        assert_eq!(checked, 2);
    }
}
