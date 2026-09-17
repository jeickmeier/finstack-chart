use super::contour::{ContourPath, connections};
use crate::ChartResult;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Vertex(usize),
    Crossing(usize, usize, u8),
}
/// Band boundary comprises lower/upper contours plus clipped valid-grid perimeter edges.
pub(crate) fn isobands(
    x: &[f64],
    y: &[f64],
    z: &[Option<f64>],
    low: f64,
    high: f64,
    budget: usize,
) -> ChartResult<Vec<ContourPath>> {
    if x.len() < 2
        || y.len() < 2
        || x.len().checked_mul(y.len()) != Some(z.len())
        || z.len() > budget
        || !low.is_finite()
        || !high.is_finite()
        || low >= high
        || x.iter().chain(y).any(|v| !v.is_finite())
        || x.windows(2).chain(y.windows(2)).any(|v| v[0] >= v[1])
    {
        return Err(super::invalid(
            "Isobands require an ordered bounded grid and increasing finite thresholds.",
        ));
    }
    let columns = x.len() - 1;
    let rows = y.len() - 1;
    let nodes = |r: usize, c: usize| {
        [
            r * x.len() + c,
            r * x.len() + c + 1,
            (r + 1) * x.len() + c + 1,
            (r + 1) * x.len() + c,
        ]
    };
    let valid = (0..rows)
        .flat_map(|r| {
            (0..columns).map(move |c| {
                nodes(r, c)
                    .iter()
                    .all(|i| z[*i].is_some_and(f64::is_finite))
            })
        })
        .collect::<Vec<_>>();
    let mut positions = BTreeMap::new();
    let mut graph: BTreeMap<Node, Vec<Node>> = BTreeMap::new();
    let mut segments = 0usize;
    for row in 0..rows {
        for col in 0..columns {
            if !valid[row * columns + col] {
                continue;
            }
            let ids = nodes(row, col);
            let v = ids.map(|i| z[i].unwrap());
            let p = [
                [x[col], y[row]],
                [x[col + 1], y[row]],
                [x[col + 1], y[row + 1]],
                [x[col], y[row + 1]],
            ];
            let mut endpoint = |e: usize, t: f64, threshold: u8| {
                let j = (e + 1) % 4;
                let key = if t == 0. {
                    Node::Vertex(ids[e])
                } else if t == 1. {
                    Node::Vertex(ids[j])
                } else {
                    Node::Crossing(ids[e].min(ids[j]), ids[e].max(ids[j]), threshold)
                };
                // Evaluate in canonical edge direction to avoid adjacent-cell roundoff differences.
                let (a, b, t) = if ids[e] < ids[j] {
                    (e, j, t)
                } else {
                    (j, e, 1. - t)
                };
                positions.entry(key).or_insert([
                    p[a][0] + t * (p[b][0] - p[a][0]),
                    p[a][1] + t * (p[b][1] - p[a][1]),
                ]);
                key
            };
            let mut add = |a: Node, b: Node| -> ChartResult<()> {
                if a == b {
                    return Ok(());
                }
                graph.entry(a).or_default().push(b);
                graph.entry(b).or_default().push(a);
                segments += 1;
                if segments > budget {
                    return Err(super::invalid("Isoband segment budget exceeded."));
                }
                Ok(())
            };
            for (threshold, level) in [(0, low), (1, high)] {
                for (a, b) in connections(v, level) {
                    let a = endpoint(a, (level - v[a]) / (v[(a + 1) % 4] - v[a]), threshold);
                    let b = endpoint(b, (level - v[b]) / (v[(b + 1) % 4] - v[b]), threshold);
                    add(a, b)?;
                }
            }
            for e in 0..4 {
                let neighbor = match e {
                    0 => row.checked_sub(1).map(|r| (r, col)),
                    1 => (col + 1 < columns).then_some((row, col + 1)),
                    2 => (row + 1 < rows).then_some((row + 1, col)),
                    _ => col.checked_sub(1).map(|c| (row, c)),
                };
                if neighbor.is_some_and(|(r, c)| valid[r * columns + c]) {
                    continue;
                }
                let j = (e + 1) % 4;
                let mut cuts = vec![(0., 2), (1., 2)];
                if v[e] != v[j] {
                    for (threshold, level) in [(0, low), (1, high)] {
                        let t = (level - v[e]) / (v[j] - v[e]);
                        if t > 0. && t < 1. {
                            cuts.push((t, threshold));
                        }
                    }
                }
                cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
                for pair in cuts.windows(2) {
                    let t = pair[0].0.midpoint(pair[1].0);
                    let value = v[e] + t * (v[j] - v[e]);
                    if value >= low && value < high {
                        add(
                            endpoint(e, pair[0].0, pair[0].1),
                            endpoint(e, pair[1].0, pair[1].1),
                        )?;
                    }
                }
            }
        }
    }
    let mut result = Vec::new();
    while let Some((&start, _)) = graph.first_key_value() {
        let mut current = start;
        let mut points = vec![positions[&start]];
        loop {
            let Some(next) = graph.get(&current).and_then(|v| v.first()).copied() else {
                return Err(super::invalid("Isoband boundary unexpectedly open."));
            };
            for (a, b) in [(current, next), (next, current)] {
                if let Some(v) = graph.get_mut(&a) {
                    let i = v.iter().position(|v| *v == b).unwrap();
                    v.remove(i);
                    if v.is_empty() {
                        graph.remove(&a);
                    }
                }
            }
            current = next;
            points.push(positions[&current]);
            if current == start {
                break;
            }
        }
        if points.len() > 3 {
            result.push(ContourPath { points });
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn area(points: &[[f64; 2]]) -> f64 {
        points
            .windows(2)
            .map(|v| v[0][0] * v[1][1] - v[1][0] * v[0][1])
            .sum::<f64>()
            .abs()
            / 2.
    }
    #[test]
    fn pinned_band_holes_saddles_and_missing_cells() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in f["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let levels = match name {
                "isoband-saddle" => vec![0.5, 1., 1.5],
                "isoband-hole" | "isoband-missing-center" | "isoband-rotated" => vec![0.5, 2., 4.],
                _ => continue,
            };
            let rows = case["input"].as_array().unwrap();
            let points = rows
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let values = rows.iter().map(|r| r["z"].as_f64()).collect::<Vec<_>>();
            let grid = super::super::grid::grid(&points, &values, 1000).unwrap();
            for pair in levels.windows(2) {
                let mut actual =
                    isobands(&grid.x, &grid.y, &grid.z, pair[0], pair[1], 1000).unwrap();
                for path in &mut actual {
                    path.points = super::super::grid::rotate(&path.points, grid.angle);
                }
                let b = &case["built"];
                let n = b["x"].as_array().unwrap().len();
                let mut expected: BTreeMap<i64, Vec<[f64; 2]>> = BTreeMap::new();
                for i in 0..n {
                    if b["level_low"][i].as_f64() == Some(pair[0]) {
                        expected
                            .entry(b["subgroup"][i].as_i64().unwrap())
                            .or_default()
                            .push([b["x"][i].as_f64().unwrap(), b["y"][i].as_f64().unwrap()]);
                    }
                }
                let mut expected = expected
                    .into_values()
                    .map(|mut p| {
                        p.push(p[0]);
                        area(&p)
                    })
                    .collect::<Vec<_>>();
                let mut areas = actual.iter().map(|p| area(&p.points)).collect::<Vec<_>>();
                expected.sort_by(f64::total_cmp);
                areas.sort_by(f64::total_cmp);
                assert_eq!(areas.len(), expected.len(), "{name} {pair:?}");
                for (a, b) in areas.iter().zip(expected) {
                    assert!((a - b).abs() < 1e-12, "{name}: {a} vs {b}");
                }
            }
        }
    }
}
