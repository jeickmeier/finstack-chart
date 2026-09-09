//! Reference stack layouts retain key order independently of stacking rank.
use super::{domain, invalid, limit};
use crate::{ChartResult, interpolate::Number};
use std::cmp::Ordering;

/// Explicit stack allocation and reference arithmetic work budgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StackLimits {
    /// Maximum series, including when the sample array is empty.
    pub max_series: usize,
    /// Maximum series times samples.
    pub max_cells: usize,
    /// Maximum built-in offset work units; wiggle preserves reference summation order.
    pub max_work: usize,
}
impl Default for StackLimits {
    fn default() -> Self {
        Self {
            max_series: 4096,
            max_cells: 1_000_000,
            max_work: 100_000_000,
        }
    }
}
/// Ordered stack rank policy. Result series stay in configured key order.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StackOrder {
    /// Input key order.
    #[default]
    None,
    /// Reverse key order.
    Reverse,
    /// Stable increasing sum of values.
    Ascending,
    /// Reverse of increasing sum, including reversed ties.
    Descending,
    /// Stable increasing index of the first maximum.
    Appearance,
    /// Balance appearance-ordered sums on bottom and top.
    InsideOut,
    /// Exact permutation of zero-based key indexes.
    Explicit(Vec<usize>),
}
/// Baseline and normalization policy, separate from the legacy signed stack position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StackOffset {
    /// Cumulative signed values from zero.
    #[default]
    None,
    /// Divide by each column's signed sum, then accumulate.
    Expand,
    /// Accumulate positive and negative values separately, with ordered endpoints.
    Diverging,
    /// Center the signed column total about zero.
    Silhouette,
    /// Reference streamgraph baseline with ordered slope summation.
    Wiggle,
}
/// Missing-cell policy; absent values never implicitly become source observations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StackMissing {
    /// Reference NaN gap: the following series resumes from its preceding baseline.
    #[default]
    Gap,
    /// Use a zero height for a missing cell.
    Zero,
    /// Reject missing values.
    Error,
}
/// Native ordering protocol over initial [zero, value] points.
pub trait StackOrdering {
    /// Declared work checked before accessing data; custom sorting may raise this bound.
    fn work_units(&self, series: usize, samples: usize) -> Option<usize> {
        series.checked_mul(samples)
    }
    /// Return a complete permutation; the caller validates it before assigning ranks.
    fn order(&self, series: &[Vec<[f64; 2]>]) -> ChartResult<Vec<usize>>;
}
/// Native offset protocol over rectangular series and a validated rank permutation.
pub trait StackOffsetting {
    /// Declared work bound checked before allocation; custom protocols may raise it.
    fn work_units(&self, series: usize, samples: usize) -> Option<usize> {
        series.checked_mul(samples)
    }
    /// Change endpoints without changing series/sample lengths. Custom work is caller-owned.
    fn offset(&self, series: &mut [Vec<[f64; 2]>], order: &[usize]) -> ChartResult<()>;
}
/// One stack cell, including the original owned source datum.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackPoint<T> {
    /// Original datum; exact identifiers and metadata are retained.
    pub data: T,
    /// Lower/reference starting endpoint, with explicit exceptional-number transport.
    pub y0: Number,
    /// Upper/reference ending endpoint, including an explicit NaN for a missing gap.
    pub y1: Number,
}
/// One series in configured key order, regardless of its stacking rank.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackSeries<T> {
    /// Configured series key.
    pub key: String,
    /// Rank in the order passed to the offset operation.
    pub index: usize,
    /// Cells in original sample order.
    pub points: Vec<StackPoint<T>>,
}
/// Reusable checked stack layout; tidy chart adaptation uses this same kernel.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Stack {
    pub(super) keys: Vec<String>,
    pub(super) order: StackOrder,
    pub(super) offset: StackOffset,
    missing: StackMissing,
    value: Option<f64>,
    limits: StackLimits,
}
impl Stack {
    /// Empty keys, input rank order, cumulative offset and explicit missing gaps.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set result keys, including their stable output order.
    pub fn keys(mut self, keys: Vec<String>) -> Self {
        self.keys = keys;
        self
    }
    /// Set the stacking rank policy.
    pub fn order(mut self, order: StackOrder) -> Self {
        self.order = order;
        self
    }
    /// Set the baseline/normalization policy.
    pub fn offset(mut self, offset: StackOffset) -> Self {
        self.offset = offset;
        self
    }
    /// Set the absent-cell policy.
    pub fn missing(mut self, missing: StackMissing) -> Self {
        self.missing = missing;
        self
    }
    /// Set a constant value, or restore the supplied value accessor.
    pub fn value(mut self, value: Option<f64>) -> Self {
        self.value = value;
        self
    }
    /// Set bounded allocation and built-in work limits.
    pub fn limits(mut self, limits: StackLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Validate options independently of input population.
    pub fn validate(&self) -> ChartResult<()> {
        if self.value.is_some_and(|v| !v.is_finite()) {
            return Err(domain("Stack constant must be finite."));
        }
        if self.keys.len() > self.limits.max_series || self.keys.iter().any(|k| k.len() > 4096) {
            return Err(limit("Stack key catalog exceeds its budget."));
        }
        if let StackOrder::Explicit(order) = &self.order {
            permutation(order, self.keys.len())?;
        }
        Ok(())
    }
    /// Consume a rectangular sample-by-key matrix with explicit absent cells.
    pub fn generate(
        &self,
        data: &[Vec<Option<f64>>],
    ) -> ChartResult<Vec<StackSeries<Vec<Option<f64>>>>> {
        self.layout_materialized(data, data)
    }
    /// Layout materialized values with separately owned original source data.
    pub fn layout_materialized<T: Clone>(
        &self,
        data: &[T],
        values: &[Vec<Option<f64>>],
    ) -> ChartResult<Vec<StackSeries<T>>> {
        if data.len() != values.len() || values.iter().any(|r| r.len() != self.keys.len()) {
            return Err(invalid(
                "Stack values must match source sample count and configured key count.",
            ));
        }
        self.generate_with(
            data,
            |_, key, sample, _| Ok(values[sample][key]),
            &self.order,
            &self.offset,
        )
    }
    /// Native value accessors receive datum, key index, sample index and the full input.
    pub fn generate_by<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, usize, &[T]) -> ChartResult<Option<f64>>,
    ) -> ChartResult<Vec<StackSeries<T>>> {
        self.generate_with(data, value, &self.order, &self.offset)
    }
    /// Use native ordering and offset protocols with checked permutation and output bounds.
    pub fn generate_with<T: Clone>(
        &self,
        data: &[T],
        mut value: impl FnMut(&T, usize, usize, &[T]) -> ChartResult<Option<f64>>,
        ordering: &dyn StackOrdering,
        offset: &dyn StackOffsetting,
    ) -> ChartResult<Vec<StackSeries<T>>> {
        self.validate()?;
        let n = self.keys.len();
        let m = data.len();
        let cells = n
            .checked_mul(m)
            .ok_or_else(|| limit("Stack cell count overflow."))?;
        if cells > self.limits.max_cells || m > self.limits.max_cells {
            return Err(limit("Stack cell budget exceeded."));
        }
        if ordering
            .work_units(n, m)
            .is_none_or(|v| v > self.limits.max_work)
        {
            return Err(limit("Stack order work budget exceeded."));
        }
        let work = offset.work_units(n, m);
        if work.is_none_or(|v| v > self.limits.max_work) {
            return Err(limit("Stack offset work budget exceeded."));
        }
        let mut series = vec![Vec::with_capacity(m); n];
        for (j, datum) in data.iter().enumerate() {
            for (i, s) in series.iter_mut().enumerate() {
                let v = match self.value {
                    Some(v) => Some(v),
                    None => value(datum, i, j, data)?,
                };
                let v = match v {
                    Some(v) if v.is_finite() => v,
                    Some(_) => {
                        return Err(domain(
                            "Stack values must be finite; represent missing cells explicitly.",
                        ));
                    }
                    None => match self.missing {
                        StackMissing::Gap => f64::NAN,
                        StackMissing::Zero => 0.,
                        StackMissing::Error => {
                            return Err(domain(
                                "Stack missing-cell policy rejects an absent value.",
                            ));
                        }
                    },
                };
                s.push([0., v]);
            }
        }
        let missing: Vec<Vec<bool>> = series
            .iter()
            .map(|s| s.iter().map(|p| p[1].is_nan()).collect())
            .collect();
        let order = ordering.order(&series)?;
        permutation(&order, n)?;
        let mut ranks = vec![0; n];
        for (rank, &i) in order.iter().enumerate() {
            ranks[i] = rank;
        }
        offset.offset(&mut series, &order)?;
        for (i, s) in series.iter().enumerate() {
            if s.len() != m {
                return Err(invalid("Stack offset changed sample count."));
            }
            for (j, p) in s.iter().enumerate() {
                if !p[0].is_finite() || !(p[1].is_finite() || (missing[i][j] && p[1].is_nan())) {
                    return Err(domain("Stack offset produced a nonrepresentable endpoint."));
                }
            }
        }
        Ok(series
            .into_iter()
            .enumerate()
            .map(|(i, s)| StackSeries {
                key: self.keys[i].clone(),
                index: ranks[i],
                points: s
                    .into_iter()
                    .zip(data)
                    .map(|(p, data)| StackPoint {
                        data: data.clone(),
                        y0: Number(p[0]),
                        y1: Number(p[1]),
                    })
                    .collect(),
            })
            .collect())
    }
}
fn permutation(order: &[usize], n: usize) -> ChartResult<()> {
    if order.len() != n {
        return Err(invalid(
            "Stack order must contain every series exactly once.",
        ));
    }
    let mut seen = vec![false; n];
    for &i in order {
        if i >= n || seen[i] {
            return Err(invalid(
                "Stack order must be a permutation of series indexes.",
            ));
        }
        seen[i] = true;
    }
    Ok(())
}
fn finite(value: f64) -> ChartResult<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(domain("Stack arithmetic exceeds finite range."))
    }
}
fn or_zero(value: f64) -> f64 {
    if value.is_nan() { 0. } else { value }
}
fn sums(series: &[Vec<[f64; 2]>]) -> ChartResult<Vec<f64>> {
    series
        .iter()
        .map(|s| s.iter().try_fold(0., |sum, p| finite(sum + or_zero(p[1]))))
        .collect()
}
fn appearance(series: &[Vec<[f64; 2]>]) -> Vec<usize> {
    let peaks: Vec<_> = series
        .iter()
        .map(|s| {
            let (mut j, mut max) = (0, f64::NEG_INFINITY);
            for (i, p) in s.iter().enumerate() {
                if p[1] > max {
                    max = p[1];
                    j = i;
                }
            }
            j
        })
        .collect();
    let mut order: Vec<_> = (0..series.len()).collect();
    order.sort_by_key(|&i| peaks[i]);
    order
}
impl StackOrdering for StackOrder {
    fn order(&self, series: &[Vec<[f64; 2]>]) -> ChartResult<Vec<usize>> {
        let mut order: Vec<_> = (0..series.len()).collect();
        match self {
            Self::None => {}
            Self::Reverse => order.reverse(),
            Self::Explicit(o) => {
                permutation(o, series.len())?;
                return Ok(o.clone());
            }
            Self::Ascending | Self::Descending => {
                let sums = sums(series)?;
                order.sort_by(|&a, &b| sums[a].partial_cmp(&sums[b]).unwrap_or(Ordering::Equal));
                if *self == Self::Descending {
                    order.reverse();
                }
            }
            Self::Appearance => return Ok(appearance(series)),
            Self::InsideOut => {
                let sums = sums(series)?;
                let (mut top, mut bottom) = (0., 0.);
                let (mut tops, mut bottoms) = (vec![], vec![]);
                for j in appearance(series) {
                    if top < bottom {
                        top = finite(top + sums[j])?;
                        tops.push(j);
                    } else {
                        bottom = finite(bottom + sums[j])?;
                        bottoms.push(j);
                    }
                }
                bottoms.reverse();
                bottoms.extend(tops);
                return Ok(bottoms);
            }
        }
        Ok(order)
    }
}
fn cumulative(series: &mut [Vec<[f64; 2]>], order: &[usize]) {
    for rank in 1..order.len() {
        let a = order[rank - 1];
        let b = order[rank];
        for j in 0..series[b].len() {
            let prior = series[a][j];
            let base = if prior[1].is_nan() {
                prior[0]
            } else {
                prior[1]
            };
            series[b][j][0] = base;
            series[b][j][1] += base;
        }
    }
}
impl StackOffsetting for StackOffset {
    fn work_units(&self, series: usize, samples: usize) -> Option<usize> {
        let cells = series.checked_mul(samples)?;
        if *self == Self::Wiggle {
            cells.checked_mul(series)
        } else {
            Some(cells)
        }
    }
    fn offset(&self, series: &mut [Vec<[f64; 2]>], order: &[usize]) -> ChartResult<()> {
        permutation(order, series.len())?;
        let n = series.len();
        if n == 0 {
            return Ok(());
        }
        let m = series[0].len();
        if series.iter().any(|s| s.len() != m) {
            return Err(invalid("Stack offset requires rectangular series."));
        }
        match self {
            Self::None => cumulative(series, order),
            Self::Expand => {
                for j in 0..m {
                    let total = series
                        .iter()
                        .try_fold(0., |sum, s| finite(sum + or_zero(s[j][1])))?;
                    if total != 0. {
                        for s in series.iter_mut() {
                            s[j][1] /= total;
                        }
                    }
                }
                cumulative(series, order);
            }
            Self::Diverging => {
                // Columns are independent; the inner loop follows the explicit series permutation.
                #[allow(clippy::needless_range_loop)]
                for j in 0..m {
                    let (mut positive, mut negative) = (0., 0.);
                    for &i in order {
                        let p = &mut series[i][j];
                        let height = p[1] - p[0];
                        if height > 0. {
                            p[0] = positive;
                            positive += height;
                            p[1] = positive;
                        } else if height < 0. {
                            p[1] = negative;
                            negative += height;
                            p[0] = negative;
                        } else {
                            p[0] = 0.;
                            p[1] = height;
                        }
                    }
                }
            }
            Self::Silhouette => {
                for j in 0..m {
                    let total = series
                        .iter()
                        .try_fold(0., |sum, s| finite(sum + or_zero(s[j][1])))?;
                    let p = &mut series[order[0]][j];
                    p[0] = -total / 2.;
                    p[1] += p[0];
                }
                cumulative(series, order);
            }
            Self::Wiggle => {
                if m == 0 {
                    return Ok(());
                }
                let mut y = 0.;
                for j in 1..m {
                    let (mut s1, mut s2) = (0., 0.);
                    for (i, &key) in order.iter().enumerate() {
                        let current = or_zero(series[key][j][1]);
                        let previous = or_zero(series[key][j - 1][1]);
                        let mut s3 = (current - previous) / 2.;
                        for &prior in &order[..i] {
                            s3 += or_zero(series[prior][j][1]) - or_zero(series[prior][j - 1][1]);
                        }
                        s1 = finite(s1 + current)?;
                        s2 = finite(s2 + s3 * current)?;
                    }
                    let p = &mut series[order[0]][j - 1];
                    p[0] = y;
                    p[1] += y;
                    if s1 != 0. {
                        y = finite(y - s2 / s1)?;
                    }
                }
                let p = &mut series[order[0]][m - 1];
                p[0] = y;
                p[1] += y;
                cumulative(series, order);
            }
        }
        Ok(())
    }
}
