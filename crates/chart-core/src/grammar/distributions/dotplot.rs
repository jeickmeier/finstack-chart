use crate::ChartResult;
#[derive(Clone, Debug)]
pub(crate) struct DotBin {
    pub center: f64,
    pub width: f64,
    pub count: f64,
    pub members: Vec<usize>,
}
/// Wilkinson dot-density grouping; ties on the next boundary start a new bin.
pub(crate) fn dotdensity(
    values: &[f64],
    weights: Option<&[f64]>,
    binwidth: f64,
) -> ChartResult<Vec<DotBin>> {
    if !binwidth.is_finite()
        || values.iter().any(|v| !v.is_finite())
        || weights.is_some_and(|w| {
            w.len() != values.len()
                || w.iter()
                    .any(|v| !v.is_finite() || *v < 0. || v.fract() != 0.)
        })
    {
        return Err(super::invalid(
            "Dot bins require finite positions and width, and nonnegative integer weights.",
        ));
    }
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| values[*a].total_cmp(&values[*b]).then_with(|| a.cmp(b)));
    let mut bins: Vec<DotBin> = vec![];
    let mut end = f64::NEG_INFINITY;
    for i in order {
        if values[i] >= end {
            end = values[i] + binwidth;
            bins.push(DotBin {
                center: values[i],
                width: binwidth,
                count: 0.,
                members: vec![],
            });
        }
        let bin = bins.last_mut().expect("nonempty values start a bin");
        bin.members.push(i);
        bin.count += weights.map_or(1., |w| w[i]);
        bin.center = (values[bin.members[0]] + values[i]) / 2.;
    }
    if binwidth <= 0. {
        let mut merged: Vec<DotBin> = vec![];
        for bin in bins {
            if let Some(last) = merged.last_mut().filter(|last| last.center == bin.center) {
                last.count += bin.count;
                last.members.extend(bin.members);
            } else {
                merged.push(bin);
            }
        }
        return Ok(merged);
    }
    Ok(bins)
}
/// Histogram-style dots reuse the shared GG06 automatic edges and fuzzy closure.
pub(crate) fn histodot(
    values: &[f64],
    weights: Option<&[f64]>,
    range: [f64; 2],
    binwidth: Option<f64>,
    origin: Option<f64>,
    closed: super::super::BinClosure,
) -> ChartResult<Vec<DotBin>> {
    if values.iter().any(|v| !v.is_finite())
        || weights.is_some_and(|w| {
            w.len() != values.len()
                || w.iter()
                    .any(|v| !v.is_finite() || *v < 0. || v.fract() != 0.)
        })
    {
        return Err(super::invalid(
            "Histogram dots require finite positions and nonnegative integer weights.",
        ));
    }
    let options = super::super::GgplotBinOptions {
        binwidth,
        boundary: origin,
        closed,
        ..Default::default()
    };
    let edges = super::super::ggplot_stats::automatic_edges(range[0], range[1], 30, &options)?;
    let fuzzy = super::super::ggplot_stats::fuzzy_edges(&edges, closed);
    let mut bins = edges
        .windows(2)
        .map(|v| DotBin {
            center: v[0].midpoint(v[1]),
            width: v[1] - v[0],
            count: 0.,
            members: vec![],
        })
        .collect::<Vec<_>>();
    for (i, v) in values.iter().enumerate() {
        if *v < fuzzy[0] || *v > fuzzy[fuzzy.len() - 1] {
            continue;
        }
        let k = fuzzy
            .partition_point(|edge| {
                if closed == super::super::BinClosure::Right {
                    edge < v
                } else {
                    edge <= v
                }
            })
            .saturating_sub(1)
            .min(bins.len() - 1);
        bins[k].count += weights.map_or(1., |w| w[i]);
        bins[k].members.push(i);
    }
    Ok(bins)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_dot_bins_grouped_and_panel_shared() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        for c in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("dot-"))
        {
            let controls = &c["controls"];
            let rows = controls["data"].as_array().unwrap();
            let x = rows
                .iter()
                .map(|r| r["x"].as_f64().unwrap())
                .collect::<Vec<_>>();
            let w = rows
                .iter()
                .map(|r| r["w"].as_f64().unwrap())
                .collect::<Vec<_>>();
            let panel = dotdensity(&x, Some(&w), 1.).unwrap();
            for (group_id, group) in [(1, "A"), (2, "B")] {
                let members = rows
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r["g"] == group)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                let gx = members.iter().map(|i| x[*i]).collect::<Vec<_>>();
                let gw = members.iter().map(|i| w[*i]).collect::<Vec<_>>();
                let bins = if controls["method"] == "histodot" {
                    histodot(
                        &gx,
                        Some(&gw),
                        [0., 3.],
                        Some(1.),
                        None,
                        if controls["right"] == true {
                            super::super::super::BinClosure::Right
                        } else {
                            super::super::super::BinClosure::Left
                        },
                    )
                    .unwrap()
                } else if controls["binpositions"] == "all" {
                    panel
                        .iter()
                        .filter_map(|b| {
                            let m = b
                                .members
                                .iter()
                                .filter(|i| members.contains(i))
                                .copied()
                                .collect::<Vec<_>>();
                            (!m.is_empty()).then(|| DotBin {
                                center: b.center,
                                width: b.width,
                                count: m.iter().map(|i| w[*i]).sum(),
                                members: m,
                            })
                        })
                        .collect()
                } else {
                    dotdensity(&gx, Some(&gw), 1.).unwrap()
                };
                let position = if controls["binaxis"] == "x" { "x" } else { "y" };
                let bins = bins
                    .into_iter()
                    .filter(|b| b.count > 0.)
                    .collect::<Vec<_>>();
                let mut expected = c["result"]["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["group"].as_i64() == Some(group_id))
                    .collect::<Vec<_>>();
                expected.dedup_by(|a, b| a[position] == b[position]);
                assert_eq!(bins.len(), expected.len(), "{} {group}", c["name"]);
                for (a, b) in bins.iter().zip(expected) {
                    assert_eq!(a.center, b[position].as_f64().unwrap(), "{}", c["name"]);
                    assert_eq!(a.count, b["count"].as_f64().unwrap(), "{}", c["name"]);
                    assert_eq!(a.width, b["binwidth"].as_f64().unwrap(), "{}", c["name"]);
                }
            }
        }
    }
}

#[cfg(test)]
mod width_tests {
    use super::*;
    #[test]
    fn signed_and_zero_dotdensity_widths() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/dot-width-controls.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let x = c["x"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            let actual = dotdensity(&x, None, c["width"].as_f64().unwrap()).unwrap();
            let mut expected = c["values"].as_array().unwrap().iter().collect::<Vec<_>>();
            expected.dedup_by(|a, b| a["x"] == b["x"]);
            assert_eq!(actual.len(), expected.len());
            for (a, b) in actual.iter().zip(expected) {
                assert_eq!(a.center, b["x"].as_f64().unwrap());
                assert_eq!(a.count, b["count"].as_f64().unwrap());
                assert_eq!(a.width, b["binwidth"].as_f64().unwrap());
            }
        }
    }
}
