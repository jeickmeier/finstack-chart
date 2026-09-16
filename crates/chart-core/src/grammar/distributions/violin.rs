use super::super::ViolinScale;
use super::{Bandwidth, DensityControls, DensityEstimate, DensityPoint};
use crate::ChartResult;
#[derive(Clone, Debug)]
pub(crate) struct ViolinPoint {
    pub density: DensityPoint,
    pub quantile: Option<f64>,
    pub violinwidth: f64,
}
#[derive(Clone, Debug)]
pub(crate) struct ViolinEstimate {
    pub points: Vec<ViolinPoint>,
    pub density: DensityEstimate,
    pub weighted_quantiles: bool,
}
/// Group computation precedes panel-wide area/count normalization.
pub(crate) fn violin(
    values: &[f64],
    weights: Option<&[f64]>,
    controls: &DensityControls,
    trim: bool,
    quantiles: &[f64],
    max_grid: usize,
) -> ChartResult<ViolinEstimate> {
    if quantiles.iter().any(|q| !(0. ..=1.).contains(q)) {
        return Err(super::invalid("Violin quantiles must lie in [0,1]."));
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    if values.len() < 2 {
        return Ok(ViolinEstimate {
            points: vec![],
            density: DensityEstimate {
                points: vec![],
                removed_bounds: false,
                too_few: true,
                boundary_minimum: false,
            },
            weighted_quantiles: weights.is_some() && !quantiles.is_empty(),
        });
    }
    let (bw, boundary) = super::bandwidth::select(values, controls.bandwidth)?;
    let mut settings = controls.clone();
    settings.bandwidth = Bandwidth::Fixed(bw);
    let extension = if trim { 0. } else { 3. * bw };
    let mut estimate = super::density(
        values,
        weights,
        sorted[0] - extension,
        sorted[sorted.len() - 1] + extension,
        &settings,
        max_grid,
    )?;
    estimate.boundary_minimum |= boundary;
    let mut inserted = vec![];
    for tau in quantiles {
        let value = super::quantile(&sorted, *tau, 7)?;
        let hi = estimate.points.partition_point(|p| p.x < value);
        let point = if hi == 0 {
            estimate.points.first().cloned()
        } else if hi == estimate.points.len() {
            estimate.points.last().cloned()
        } else {
            let a = &estimate.points[hi - 1];
            let b = &estimate.points[hi];
            let f = (value - a.x) / (b.x - a.x);
            let mix = |x: f64, y: f64| (1. - f) * x + f * y;
            Some(DensityPoint {
                x: value,
                density: mix(a.density, b.density),
                scaled: mix(a.scaled, b.scaled),
                count: mix(a.count, b.count),
                wdensity: mix(a.wdensity, b.wdensity),
                n: a.n,
            })
        };
        if let Some(mut point) = point {
            point.x = value;
            inserted.push(ViolinPoint {
                density: point,
                quantile: Some(*tau),
                violinwidth: 0.,
            });
        }
    }
    let mut points = estimate
        .points
        .iter()
        .filter(|p| !inserted.iter().any(|v| v.density.x == p.x))
        .cloned()
        .map(|density| ViolinPoint {
            density,
            quantile: None,
            violinwidth: 0.,
        })
        .collect::<Vec<_>>();
    points.extend(inserted);
    Ok(ViolinEstimate {
        points,
        density: estimate,
        weighted_quantiles: weights.is_some() && !quantiles.is_empty(),
    })
}
/// Normalize a complete panel; a caller must not normalize individual groups separately.
pub(crate) fn normalize_violin(groups: &mut [ViolinEstimate], scale: ViolinScale) {
    let maximum = groups
        .iter()
        .flat_map(|g| &g.points)
        .map(|p| p.density.density)
        .fold(0., f64::max);
    let nmax = groups
        .iter()
        .flat_map(|g| &g.points)
        .map(|p| p.density.n)
        .max()
        .unwrap_or(0) as f64;
    for group in groups {
        for point in &mut group.points {
            point.violinwidth = match scale {
                ViolinScale::Area => point.density.density / maximum,
                ViolinScale::Count => {
                    point.density.density / maximum * point.density.n as f64 / nmax
                }
                ViolinScale::Width => point.density.scaled,
            };
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_violin_normalization_and_quantile_insertions() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        for c in fixture["cases"].as_array().unwrap().iter().filter(|c| {
            c["name"].as_str().unwrap().starts_with("violin-")
                && !c["name"].as_str().unwrap().contains("singleton")
        }) {
            let rows = c["controls"]["data"].as_array().unwrap();
            let mut estimates = ["A", "B"]
                .into_iter()
                .map(|g| {
                    let y = rows
                        .iter()
                        .filter(|r| r["g"] == g)
                        .map(|r| r["y"].as_f64().unwrap())
                        .collect::<Vec<_>>();
                    violin(
                        &y,
                        None,
                        &DensityControls {
                            bandwidth: Bandwidth::Fixed(0.5),
                            ..Default::default()
                        },
                        c["controls"]["trim"].as_bool().unwrap(),
                        &[0.25, 0.5, 0.75],
                        1 << 20,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
            normalize_violin(
                &mut estimates,
                match c["controls"]["scale"].as_str().unwrap() {
                    "area" => ViolinScale::Area,
                    "count" => ViolinScale::Count,
                    "width" => ViolinScale::Width,
                    _ => unreachable!(),
                },
            );
            let actual = estimates.iter().flat_map(|g| &g.points).collect::<Vec<_>>();
            let expected = c["result"]["value"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, b) in actual.iter().zip(expected) {
                for (k, x) in [
                    ("y", a.density.x),
                    ("density", a.density.density),
                    ("violinwidth", a.violinwidth),
                ] {
                    assert!(
                        (x - b[k].as_f64().unwrap()).abs() < 1e-9,
                        "{} {k}: {x} expected{}",
                        c["name"],
                        b[k]
                    );
                }
            }
        }
    }
}
