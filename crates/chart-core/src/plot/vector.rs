//! Checked vector conveniences. Results materialize into the existing typed data owner.
use super::{ChartResult, ColumnData, DiagnosticCode, error};
use serde::{Deserialize, Serialize};
const MAX_BINS: usize = 100_000;
/// How a numeric vector is partitioned into ordered intervals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CutSpec {
    /// Equal width over the finite observed range.
    Interval {
        /// Number of equal-width intervals.
        bins: usize,
    },
    /// Equal width aligned to multiples of the requested length.
    IntervalLength {
        /// Positive interval length.
        length: f64,
    },
    /// Equal population using the shared type-seven sample quantiles.
    Number {
        /// Number of equal-population intervals.
        bins: usize,
    },
    /// Fixed width, with either a center or a boundary (neither means half-width).
    Width {
        /// Positive bin width.
        width: f64,
        /// Optional center of one interval.
        center: Option<f64>,
        /// Optional interval boundary.
        boundary: Option<f64>,
    },
}
/// Label and endpoint controls shared by all interval helpers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CutOptions {
    /// Right-closed intervals; the outermost endpoints are always included.
    pub right: bool,
    /// Explicit labels, in interval order.
    pub labels: Option<Vec<String>>,
    /// Retain ordered-category metadata on this result.
    pub ordered: bool,
    /// Initial significant digits for generated labels (increased for collisions).
    pub digits: usize,
}
impl Default for CutOptions {
    fn default() -> Self {
        Self {
            right: true,
            labels: None,
            ordered: false,
            digits: 3,
        }
    }
}
/// Stable zero-based category codes, including unused levels and missing values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CutResult {
    /// None for missing or out-of-range observations.
    pub codes: Vec<Option<u32>>,
    /// Category dictionary in interval order.
    pub levels: Vec<String>,
    /// Numeric boundaries before text formatting.
    pub breaks: Vec<f64>,
    /// Whether the author requested an ordered category.
    pub ordered: bool,
}
impl CutResult {
    /// Materialize with the complete dictionary and original null positions.
    /// Use `levels` as an explicit scale domain when preserving factor order in guides;
    /// the existing reference character-scale default sorts labels.
    pub fn column(&self) -> ColumnData {
        super::data::categorical_codes(self.codes.clone(), self.levels.clone())
    }
}
fn invalid(message: &str) -> crate::Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
/// Partition a nullable numeric vector using one checked implementation.
pub fn cut(values: &[Option<f64>], spec: CutSpec, options: CutOptions) -> ChartResult<CutResult> {
    if values.len() > 1_000_000 || !(1..=17).contains(&options.digits) {
        return Err(invalid("Vector helper input or precision budget exceeded."));
    }
    let mut finite: Vec<_> = values
        .iter()
        .flatten()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    finite.sort_by(f64::total_cmp);
    let (Some(&low), Some(&high)) = (finite.first(), finite.last()) else {
        return Err(invalid("Cut requires a nonempty finite range."));
    };
    let sequence = |a: f64, b: f64, n: usize| -> ChartResult<Vec<f64>> {
        if n == 0 || n > MAX_BINS || !a.is_finite() || !b.is_finite() {
            return Err(invalid(
                "Cut requires a finite range and 1..100000 intervals.",
            ));
        }
        Ok((0..=n)
            .map(|i| {
                if i == n {
                    b
                } else {
                    a + (b - a) * (i as f64 / n as f64)
                }
            })
            .collect())
    };
    let breaks = match spec {
        CutSpec::Interval { bins } => sequence(low, high, bins)?,
        CutSpec::Number { bins } => {
            if bins == 0 || bins > MAX_BINS {
                return Err(invalid("Cut requires 1..100000 intervals."));
            }
            // Nonfinite observations are not silently discarded from sample quantiles.
            if values.iter().flatten().any(|x| x.is_infinite()) {
                return Err(invalid("Number cuts require finite sample values."));
            }
            (0..=bins)
                .map(|i| {
                    crate::grammar::distributions::quantile(&finite, i as f64 / bins as f64, 7)
                })
                .collect::<ChartResult<Vec<_>>>()?
        }
        CutSpec::IntervalLength { length } => {
            if !length.is_finite() || length <= 0. {
                return Err(invalid("Interval length must be positive and finite."));
            }
            let a = libm::floor(low / length) * length;
            let b = libm::ceil(high / length) * length;
            let n = libm::round((b - a) / length);
            if n < 1. || n > MAX_BINS as f64 {
                return Err(invalid("Interval length yields an invalid interval count."));
            }
            (0..=n as usize).map(|i| a + i as f64 * length).collect()
        }
        CutSpec::Width {
            width,
            center,
            boundary,
        } => {
            if !width.is_finite()
                || width <= 0.
                || center.is_some() && boundary.is_some()
                || center.into_iter().chain(boundary).any(|v| !v.is_finite())
            {
                return Err(invalid(
                    "Width must be positive and finite; choose at most one finite center or boundary.",
                ));
            }
            if values.iter().flatten().any(|x| *x == f64::INFINITY) {
                return Err(invalid("Width cuts cannot cover positive infinity."));
            }
            let origin = boundary.unwrap_or_else(|| center.map_or(width / 2., |c| c - width / 2.));
            let a = origin + libm::floor((low - origin) / width) * width;
            let n = libm::floor((high + (1. - 1e-8) * width - a) / width);
            if n < 1. || n > MAX_BINS as f64 {
                return Err(invalid("Width yields an invalid interval count."));
            }
            (0..=n as usize).map(|i| a + i as f64 * width).collect()
        }
    };
    if breaks.iter().any(|x| !x.is_finite()) || breaks.windows(2).any(|p| p[0] >= p[1]) {
        return Err(invalid(
            "Cut boundaries must be finite and unique; insufficient distinct data values.",
        ));
    }
    let count = breaks.len() - 1;
    let levels = if let Some(labels) = options.labels {
        if labels.len() != count {
            return Err(invalid("Cut labels must match the interval count."));
        }
        labels
    } else {
        let mut endpoints = Vec::new();
        for precision in options.digits..=12.max(options.digits) {
            endpoints = breaks
                .iter()
                .map(|v| crate::number::general_significant(*v, precision))
                .collect();
            if endpoints.windows(2).all(|p| p[0] != p[1]) {
                break;
            }
        }
        if endpoints.windows(2).any(|p| p[0] == p[1]) {
            (1..=count).map(|i| format!("Range_{i}")).collect()
        } else {
            (0..count)
                .map(|i| {
                    format!(
                        "{}{},{}{}",
                        if !options.right || i == 0 { '[' } else { '(' },
                        endpoints[i],
                        endpoints[i + 1],
                        if options.right || i + 1 == count {
                            ']'
                        } else {
                            ')'
                        }
                    )
                })
                .collect()
        }
    };
    let mut dictionary = Vec::new();
    let mut lookup = std::collections::BTreeMap::new();
    let level_codes: Vec<u32> = levels
        .into_iter()
        .map(|label| {
            *lookup.entry(label.clone()).or_insert_with(|| {
                let code = dictionary.len() as u32;
                dictionary.push(label);
                code
            })
        })
        .collect();
    let levels = dictionary;
    let codes = values
        .iter()
        .map(|value| {
            value.and_then(|x| {
                if x.is_nan() || x < breaks[0] || x > *breaks.last().unwrap() {
                    return None;
                }
                let index = if options.right {
                    breaks.partition_point(|b| *b < x).saturating_sub(1)
                } else {
                    breaks.partition_point(|b| *b <= x).saturating_sub(1)
                };
                Some(level_codes[index.min(count - 1)])
            })
        })
        .collect();
    Ok(CutResult {
        codes,
        levels,
        breaks,
        ordered: options.ordered,
    })
}
/// Reference resolution controls preserve integer and mapped-discrete semantics.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResolutionOptions {
    /// Add zero when finding the smallest meaningful gap.
    pub zero: bool,
    /// The source is integer typed, regardless of the observed values.
    pub integer: bool,
    /// Request unit resolution for a mapped-discrete vector.
    pub discrete: bool,
    /// Whether the vector carries mapped-discrete metadata.
    pub mapped_discrete: bool,
}
impl Default for ResolutionOptions {
    fn default() -> Self {
        Self {
            zero: true,
            integer: false,
            discrete: false,
            mapped_discrete: false,
        }
    }
}
/// Smallest gap above sqrt(machine epsilon); a constant or integer vector has unit resolution.
pub fn resolution(values: &[Option<f64>], options: ResolutionOptions) -> ChartResult<f64> {
    if values.len() > 1_000_000 {
        return Err(invalid("Resolution vector exceeds the input budget."));
    }
    if options.integer || options.discrete && options.mapped_discrete {
        return Ok(1.);
    }
    let mut sorted: Vec<_> = values
        .iter()
        .flatten()
        .copied()
        .filter(|x| !x.is_nan())
        .collect();
    sorted.sort_by(f64::total_cmp);
    if let (Some(&a), Some(&b)) = (sorted.first(), sorted.last()) {
        let m = a.abs().min(b.abs());
        if a == b
            || a.is_finite()
                && b.is_finite()
                && m > 0.
                && ((a - b) / m).abs() < 1000. * f64::EPSILON
        {
            return Ok(1.);
        }
    }
    if options.zero {
        sorted.push(0.);
        sorted.sort_by(f64::total_cmp);
    }
    sorted.dedup();
    Ok(sorted
        .windows(2)
        .map(|p| p[1] - p[0])
        .filter(|d| *d > f64::EPSILON.sqrt())
        .reduce(f64::min)
        .unwrap_or(1.))
}

/// Compute standalone summary bounds using exactly the built-in stat kernel.
/// Null and NaN observations are missing; infinities reject in the common kernel.
pub fn summarize(
    values: &[Option<f64>],
    helper: &crate::grammar::SummaryHelper,
) -> ChartResult<[Option<f64>; 3]> {
    if values.len() > 1_000_000 {
        return Err(invalid("Summary vector input budget exceeded."));
    }
    let population: Vec<_> = values
        .iter()
        .flatten()
        .copied()
        .filter(|v| !v.is_nan())
        .collect();
    crate::grammar::summary_values(helper, &population)
}
