use super::super::{SummaryBins, SummaryFunction, ggplot_stats};
use crate::ChartResult;
use std::collections::{BTreeMap, BTreeSet};
/// One occupied or explicitly empty cell with exact indices into the input population.
pub(crate) struct Cell {
    pub center: [f64; 2],
    pub size: [f64; 2],
    pub value: Option<f64>,
    pub members: Vec<usize>,
}
pub(crate) fn rectangular(
    points: &[[f64; 2]],
    values: &[f64],
    axes: &[SummaryBins; 2],
    ranges: [[f64; 2]; 2],
    summary: Option<SummaryFunction>,
    drop: bool,
    budget: usize,
) -> ChartResult<Vec<Cell>> {
    if points.len() != values.len() {
        return Err(super::invalid(
            "Spatial values must match coordinate length.",
        ));
    }
    let mut edges = [Vec::new(), Vec::new()];
    for axis in 0..2 {
        edges[axis] = match &axes[axis].breaks {
            Some(v) => v.clone(),
            None => ggplot_stats::automatic_edges(
                ranges[axis][0],
                ranges[axis][1],
                axes[axis].bins,
                &axes[axis].options,
            )?,
        };
        if edges[axis].len() < 2
            || edges[axis].len() > budget.saturating_add(1)
            || edges[axis].iter().any(|v| !v.is_finite())
            || edges[axis].windows(2).any(|v| v[0] >= v[1])
        {
            return Err(super::invalid(
                "Spatial bin edges must be finite, ordered and bounded.",
            ));
        }
    }
    let fuzzy = [
        ggplot_stats::fuzzy_edges(&edges[0], axes[0].options.closed),
        ggplot_stats::fuzzy_edges(&edges[1], axes[1].options.closed),
    ];
    let mut groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut used = [BTreeSet::new(), BTreeSet::new()];
    for (i, p) in points.iter().enumerate() {
        let mut bin = [0; 2];
        let mut valid = true;
        for axis in 0..2 {
            let k = fuzzy[axis].partition_point(|e| *e < p[axis]);
            if !p[axis].is_finite() || k == 0 || k >= fuzzy[axis].len() {
                valid = false;
                break;
            }
            bin[axis] = k - 1;
        }
        if valid {
            used[0].insert(bin[0]);
            used[1].insert(bin[1]);
            groups.entry((bin[1], bin[0])).or_default().push(i);
        }
    }
    if used[0]
        .len()
        .checked_mul(used[1].len())
        .is_none_or(|n| n > budget)
    {
        return Err(super::invalid("Spatial cell budget exceeded."));
    }
    let mut out = Vec::new();
    for y in &used[1] {
        for x in &used[0] {
            let members = groups.remove(&(*y, *x)).unwrap_or_default();
            if drop && members.is_empty() {
                continue;
            }
            let sample = members.iter().map(|i| values[*i]).collect::<Vec<_>>();
            let value = match summary {
                Some(function) => super::response_summary(function, &sample)?,
                None => {
                    let sum = sample.iter().sum::<f64>();
                    sum.is_finite().then_some(sum)
                }
            };
            out.push(Cell {
                center: [
                    edges[0][*x].midpoint(edges[0][x + 1]),
                    edges[1][*y].midpoint(edges[1][y + 1]),
                ],
                size: [
                    edges[0][x + 1] - edges[0][*x],
                    edges[1][y + 1] - edges[1][*y],
                ],
                value,
                members,
            });
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{BinClosure, GgplotBinOptions};
    #[test]
    fn pinned_rectangular_membership_and_empty_cells() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            if !name.starts_with("bin2d-left-")
                && !name.starts_with("bin2d-right-")
                && name != "bin2d-missing-weight"
            {
                continue;
            }
            let rows = case["input"].as_array().unwrap();
            let points = rows
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let weights = rows
                .iter()
                .map(|r| r["w"].as_f64().unwrap_or(f64::NAN))
                .collect::<Vec<_>>();
            let control = SummaryBins {
                options: GgplotBinOptions {
                    closed: if name.contains("left") {
                        BinClosure::Left
                    } else {
                        BinClosure::Right
                    },
                    binwidth: Some(1.),
                    boundary: Some(0.),
                    ..Default::default()
                },
                ..Default::default()
            };
            let actual = rectangular(
                &points,
                &weights,
                &[control.clone(), control],
                [[0., 2.], [0., 2.]],
                None,
                name.ends_with("TRUE") || name == "bin2d-missing-weight",
                100,
            )
            .unwrap();
            let counts = case["built"]["count"].as_array().unwrap();
            assert_eq!(actual.len(), counts.len());
            for (i, cell) in actual.iter().enumerate() {
                assert_eq!(cell.value, counts[i].as_f64(), "{name}");
                assert_eq!(
                    cell.center,
                    [
                        case["built"]["x"][i].as_f64().unwrap(),
                        case["built"]["y"][i].as_f64().unwrap()
                    ]
                );
                assert_eq!(cell.size, [1., 1.]);
            }
            assert_eq!(
                actual.iter().map(|c| c.members.len()).sum::<usize>(),
                points.len()
            );
        }
    }
}
