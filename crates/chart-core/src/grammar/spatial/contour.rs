use crate::ChartResult;
use std::collections::BTreeMap;
/// One contour path; points retain deterministic grid edge order and closure.
pub(crate) struct ContourPath {
    pub points: Vec<[f64; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(usize, usize);
/// Marching-square cell connections with the reference arithmetic-center saddle policy.
pub(super) fn connections(v: [f64; 4], level: f64) -> Vec<(usize, usize)> {
    let mut crossed = Vec::new();
    for e in 0..4 {
        if (v[e] >= level) != (v[(e + 1) % 4] >= level) {
            crossed.push(e);
        }
    }
    if crossed.len() == 2 {
        return vec![(crossed[0], crossed[1])];
    }
    if crossed.len() != 4 {
        return Vec::new();
    }
    let central = (v[0] + v[1] + v[2] + v[3]) / 4.;
    if (central >= level) == (v[0] >= level) {
        vec![(0, 1), (2, 3)]
    } else {
        vec![(0, 3), (1, 2)]
    }
}
pub(crate) fn isolines(
    x: &[f64],
    y: &[f64],
    z: &[Option<f64>],
    level: f64,
    budget: usize,
) -> ChartResult<Vec<ContourPath>> {
    let size = x.len().checked_mul(y.len());
    if x.len() < 2
        || y.len() < 2
        || size != Some(z.len())
        || size.is_none_or(|n| n > budget)
        || !level.is_finite()
        || x.iter().chain(y).any(|v| !v.is_finite())
        || x.windows(2).chain(y.windows(2)).any(|p| p[0] >= p[1])
    {
        return Err(super::invalid(
            "Contours require a bounded ordered rectangular grid.",
        ));
    }
    let mut positions = BTreeMap::new();
    let mut neighbors: BTreeMap<Edge, Vec<Edge>> = BTreeMap::new();
    let mut segment_count = 0usize;
    for row in 0..y.len() - 1 {
        for col in 0..x.len() - 1 {
            let nodes = [
                row * x.len() + col,
                row * x.len() + col + 1,
                (row + 1) * x.len() + col + 1,
                (row + 1) * x.len() + col,
            ];
            let mut v = [0.; 4];
            let mut valid = true;
            for i in 0..4 {
                match z[nodes[i]] {
                    Some(value) if value.is_finite() => v[i] = value,
                    _ => {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid {
                continue;
            }
            let p = [
                [x[col], y[row]],
                [x[col + 1], y[row]],
                [x[col + 1], y[row + 1]],
                [x[col], y[row + 1]],
            ];
            let mut intersection = |e: usize| {
                let j = (e + 1) % 4;
                let (a, b) = if nodes[e] < nodes[j] { (e, j) } else { (j, e) };
                let key = Edge(nodes[a], nodes[b]);
                let t = (level - v[a]) / (v[b] - v[a]);
                positions.entry(key).or_insert([
                    p[a][0] + t * (p[b][0] - p[a][0]),
                    p[a][1] + t * (p[b][1] - p[a][1]),
                ]);
                key
            };
            for (a, b) in connections(v, level) {
                let a = intersection(a);
                let b = intersection(b);
                neighbors.entry(a).or_default().push(b);
                neighbors.entry(b).or_default().push(a);
                segment_count += 1;
                if segment_count > budget {
                    return Err(super::invalid("Contour segment budget exceeded."));
                }
            }
        }
    }
    let mut output = Vec::new();
    while !neighbors.is_empty() {
        let start = neighbors
            .iter()
            .find(|(_, v)| v.len() == 1)
            .map(|(k, _)| *k)
            .unwrap_or(*neighbors.first_key_value().unwrap().0);
        let mut key = start;
        let mut points = vec![positions[&start]];
        while let Some(next) = neighbors.get(&key).and_then(|v| v.first()).copied() {
            for (a, b) in [(key, next), (next, key)] {
                if let Some(v) = neighbors.get_mut(&a) {
                    if let Some(i) = v.iter().position(|v| *v == b) {
                        v.remove(i);
                    }
                    if v.is_empty() {
                        neighbors.remove(&a);
                    }
                }
            }
            key = next;
            points.push(positions[&key]);
            if key == start {
                break;
            }
        }
        if points.len() > 1 {
            output.push(ContourPath { points });
        }
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_saddles_rings_and_missing_cells() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
        ))
        .unwrap();
        for case in f["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let levels = match name {
                "contour-saddle" => vec![0.5, 1., 1.5],
                "contour-rings" | "contour-missing-center" | "contour-rotated" => vec![1., 2., 4.],
                _ => continue,
            };
            let rows = case["input"].as_array().unwrap();
            let points = rows
                .iter()
                .map(|r| [r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap()])
                .collect::<Vec<_>>();
            let values = rows.iter().map(|r| r["z"].as_f64()).collect::<Vec<_>>();
            let grid = super::super::grid::grid(&points, &values, 1000).unwrap();
            for level in levels {
                let mut actual = isolines(&grid.x, &grid.y, &grid.z, level, 1000).unwrap();
                for path in &mut actual {
                    path.points = super::super::grid::rotate(&path.points, grid.angle);
                }
                let b = &case["built"];
                let mut expected = Vec::new();
                let n = b["x"].as_array().unwrap().len();
                for i in 1..n {
                    if b["level"][i].as_f64() != Some(level) || b["group"][i] != b["group"][i - 1] {
                        continue;
                    }
                    expected.push((
                        [
                            b["x"][i - 1].as_f64().unwrap(),
                            b["y"][i - 1].as_f64().unwrap(),
                        ],
                        [b["x"][i].as_f64().unwrap(), b["y"][i].as_f64().unwrap()],
                    ));
                }
                let mut got = actual
                    .iter()
                    .flat_map(|p| p.points.windows(2).map(|v| (v[0], v[1])))
                    .collect::<Vec<_>>();
                assert_eq!(got.len(), expected.len(), "{name} {level}");
                for (a, b) in expected {
                    let near = |a: [f64; 2], b: [f64; 2]| {
                        (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12
                    };
                    let i = got
                        .iter()
                        .position(|(c, d)| {
                            (near(a, *c) && near(b, *d)) || (near(a, *d) && near(b, *c))
                        })
                        .expect(name);
                    got.remove(i);
                }
            }
        }
    }
}
