use super::super::SummaryFunction;
use super::rectangular::Cell;
use crate::ChartResult;
use std::collections::BTreeMap;
/// Nearest center on the reference offset triangular lattice, in normalized data units.
pub(crate) fn hexagonal(
    points: &[[f64; 2]],
    values: &[f64],
    width: [f64; 2],
    summary: Option<SummaryFunction>,
    drop: bool,
    budget: usize,
) -> ChartResult<Vec<Cell>> {
    if points.len() != values.len()
        || width.iter().any(|v| !v.is_finite() || *v <= 0.)
        || points.iter().flatten().any(|v| !v.is_finite())
    {
        return Err(super::invalid(
            "Hex bins require finite coordinates and positive dimensions.",
        ));
    }
    if points.is_empty() {
        return Ok(Vec::new());
    }
    let minimum = [
        points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
        points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
    ];
    let origin = [
        libm::floor(minimum[0] / width[0]) * width[0] - 1e-6,
        libm::floor(minimum[1] / width[1]) * width[1] - 1e-6,
    ];
    let step = libm::sqrt(3.) / 2.;
    let mut groups: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (index, p) in points.iter().enumerate() {
        let x = (p[0] - origin[0]) / width[0];
        let y = (p[1] - origin[1]) / width[1];
        if x > i64::MAX as f64 / 4. || y > i64::MAX as f64 / 4. {
            return Err(super::invalid(
                "Hex lattice index exceeds integer precision.",
            ));
        }
        let row = libm::floor(y / step) as i64;
        let mut best = (f64::INFINITY, 0, 0);
        for r in (row - 1)..=(row + 1) {
            let shift = if r.rem_euclid(2) == 0 { 0. } else { 0.5 };
            let column = libm::floor(x - shift + 0.5) as i64;
            let dx = x - column as f64 - shift;
            let dy = y - r as f64 * step;
            let distance = dx * dx + dy * dy;
            if distance < best.0 {
                best = (distance, r, column);
            }
        }
        groups.entry((best.1, best.2)).or_default().push(index);
        if groups.len() > budget {
            return Err(super::invalid("Hex cell budget exceeded."));
        }
    }
    let mut output = Vec::with_capacity(groups.len());
    for ((row, column), members) in groups {
        let sample = members.iter().map(|i| values[*i]).collect::<Vec<_>>();
        let value = match summary {
            Some(function) => super::response_summary(function, &sample)?,
            None => {
                let sum = sample.iter().sum::<f64>();
                sum.is_finite().then_some(sum)
            }
        };
        if drop && value.is_none() {
            continue;
        }
        output.push(Cell {
            center: [
                origin[0]
                    + width[0] * (column as f64 + if row.rem_euclid(2) == 0 { 0. } else { 0.5 }),
                origin[1] + width[1] * step * row as f64,
            ],
            size: width,
            value,
            members,
        });
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_hex_membership_and_response_summaries() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let summary = match case["name"].as_str().unwrap() {
                "hex-count" | "hex-missing-weight" => None,
                "summaryhex-mean" => Some(SummaryFunction::Mean),
                "summaryhex-sum" => Some(SummaryFunction::Sum),
                "summaryhex-median" => Some(SummaryFunction::Median),
                _ => continue,
            };
            let rows = case["input"].as_array().unwrap();
            let points = rows
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let values = rows
                .iter()
                .map(|r| {
                    r[if summary.is_none() { "w" } else { "z" }]
                        .as_f64()
                        .unwrap_or(f64::NAN)
                })
                .collect::<Vec<_>>();
            let actual = hexagonal(&points, &values, [1., 1.], summary, true, 100).unwrap();
            let built = &case["built"];
            assert_eq!(actual.len(), built["x"].as_array().unwrap().len());
            for (i, c) in actual.iter().enumerate() {
                for (axis, name) in ["x", "y"].iter().enumerate() {
                    assert!((c.center[axis] - built[name][i].as_f64().unwrap()).abs() < 1e-12);
                }
                assert_eq!(
                    c.value,
                    built[if summary.is_none() { "count" } else { "value" }][i].as_f64()
                );
            }
            if case["name"] != "hex-missing-weight" {
                assert_eq!(
                    actual.iter().map(|c| c.members.len()).sum::<usize>(),
                    points.len()
                );
            }
        }
    }
}
