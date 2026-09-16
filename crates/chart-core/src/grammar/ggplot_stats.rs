//! GG-06 reference bin arithmetic, shared by batch and incremental preparation.
use super::*;
use crate::{ChartResult, DiagnosticCode};

/// Reference interval closure; both outer endpoints remain included.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BinClosure {
    /// `(left, right]`, including the first left endpoint.
    #[default]
    Right,
    /// `[left, right)`, including the last right endpoint.
    Left,
}
/// Explicit ggplot bin controls. Absence retains the legacy bin contract.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotBinOptions {
    /// Reference fuzzy interval closure.
    #[serde(default)]
    pub closed: BinClosure,
    /// Optional source weights; missing weights contribute zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<Numeric>,
    /// Add empty bins at both ends after classifying the original population.
    #[serde(default)]
    pub pad: bool,
    /// Positive width for automatic breaks; takes precedence over bin count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binwidth: Option<f64>,
    /// Align automatic breaks to this bin center.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<f64>,
    /// Align automatic breaks to this boundary; mutually exclusive with center.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<f64>,
}
impl GgplotBinOptions {
    pub(super) fn validate(&self) -> ChartResult<()> {
        if self.binwidth.is_some_and(|v| !v.is_finite() || v <= 0.)
            || self.center.is_some_and(|v| !v.is_finite())
            || self.boundary.is_some_and(|v| !v.is_finite())
            || self.center.is_some() && self.boundary.is_some()
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Bin width must be positive and finite; choose a finite center or boundary, not both.",
            ));
        }
        Ok(())
    }
}
/// Generated weighted bin values; source membership count remains separately exact.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct BinStatistics {
    /// Sum of source weights (one per row if unweighted).
    pub count: f64,
    /// Signed count divided by width and total absolute count.
    pub density: Option<f64>,
    /// Count divided by maximum absolute count.
    pub ncount: Option<f64>,
    /// Density divided by maximum absolute density.
    pub ndensity: Option<f64>,
}
pub(super) fn fuzzy_edges(edges: &[f64], closed: BinClosure) -> Vec<f64> {
    let mut widths = edges.windows(2).map(|v| v[1] - v[0]).collect::<Vec<_>>();
    widths.sort_by(f64::total_cmp);
    let middle = widths.len() / 2;
    let median = if widths.len() % 2 == 0 {
        widths[middle - 1].midpoint(widths[middle])
    } else {
        widths[middle]
    };
    let fuzz = if median.is_finite() {
        median * 1e-8
    } else {
        f64::EPSILON * 1000.
    };
    edges
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let positive = match closed {
                BinClosure::Right => i > 0,
                BinClosure::Left => i + 1 == edges.len(),
            };
            *v + if positive { fuzz } else { -fuzz }
        })
        .collect()
}
pub(super) fn automatic_edges(
    lo: f64,
    hi: f64,
    bins: usize,
    options: &GgplotBinOptions,
) -> ChartResult<Vec<f64>> {
    options.validate()?;
    let mut boundary = options.boundary;
    let mut center = options.center;
    let width = if let Some(width) = options.binwidth {
        width
    } else if lo == hi {
        0.1
    } else if bins == 1 {
        boundary = Some(lo);
        center = None;
        hi - lo
    } else {
        let mut width = (hi - lo) / (bins - 1) as f64;
        if center.is_none() {
            boundary = Some(boundary.unwrap_or(lo - width / 2.));
        }
        if boundary.is_some_and(|v| {
            lo.rem_euclid(width) == v.rem_euclid(width)
                || hi.rem_euclid(width) == v.rem_euclid(width)
        }) {
            width = (hi - lo) / bins as f64;
        }
        width
    };
    if !width.is_finite() || width <= 0. {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Automatic bin width is not representable.",
        ));
    }
    let boundary = boundary.unwrap_or_else(|| center.map_or(width / 2., |v| v - width / 2.));
    let origin = boundary + ((lo - boundary) / width).floor() * width;
    let intervals = ((hi + (1. - 1e-8) * width - origin) / width)
        .floor()
        .max(1.);
    if !intervals.is_finite() || intervals > 1_000_000. {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Automatic bin edge budget exceeded.",
        ));
    }
    let result = (0..=intervals as usize)
        .map(|i| origin + i as f64 * width)
        .collect::<Vec<_>>();
    if result.iter().any(|v| !v.is_finite()) || result.windows(2).any(|v| v[0] >= v[1]) {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Automatic bin edges are not distinctly representable.",
        ));
    }
    Ok(result)
}

pub(super) fn statistics(counts: &[f64], edges: &[f64]) -> ChartResult<Vec<BinStatistics>> {
    let total = super::statistics::sum(counts.iter().map(|v| v.abs()))?;
    let max_count = counts.iter().map(|v| v.abs()).fold(0., f64::max);
    let densities = counts
        .iter()
        .zip(edges.windows(2))
        .map(|(n, e)| *n / (e[1] - e[0]) / total)
        .collect::<Vec<_>>();
    let max_density = densities.iter().map(|v| v.abs()).fold(0., f64::max);
    let finite = |value: f64| value.is_finite().then_some(value);
    Ok(counts
        .iter()
        .zip(densities)
        .map(|(count, density)| BinStatistics {
            count: *count,
            density: finite(density),
            ncount: finite(*count / max_count),
            ndensity: finite(density / max_density),
        })
        .collect())
}
