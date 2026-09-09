//! Pie layout preserves source data and input-array order independently of angular order.
use super::{ShapeLimits, domain, limit};
use crate::ChartResult;
use std::{cmp::Ordering, f64::consts::TAU};

/// Fallible comparison for registered portable pie protocols. Exact source metadata is
/// available alongside values; implementations must provide a consistent total order.
pub trait PieComparator {
    /// Compare two source observations without modifying them.
    fn compare(
        &self,
        left: &serde_json::Value,
        left_value: f64,
        right: &serde_json::Value,
        right_value: f64,
    ) -> ChartResult<Ordering>;
}

/// Portable angular ordering; native datum/value comparators use explicit methods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PieOrder {
    /// Descending positive or nonpositive authored value; stable ties.
    #[default]
    ValuesDescending,
    /// Ascending authored value; stable ties.
    ValuesAscending,
    /// Preserve source order around the sweep.
    Input,
}
/// Resolved pie angles, also accepted from a native whole-input accessor.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PieAngles {
    /// Start angle in radians.
    pub start_angle: f64,
    /// End angle before the sweep is clamped to one turn.
    pub end_angle: f64,
    /// Requested gap per datum; retained even for zero/negative values.
    pub pad_angle: f64,
}
impl Default for PieAngles {
    fn default() -> Self {
        Self {
            start_angle: 0.,
            end_angle: TAU,
            pad_angle: 0.,
        }
    }
}
impl PieAngles {
    fn validate(self) -> ChartResult<()> {
        if [self.start_angle, self.end_angle, self.pad_angle]
            .into_iter()
            .all(f64::is_finite)
        {
            Ok(())
        } else {
            Err(domain("Pie angles must be finite."))
        }
    }
}
/// One output in original input-array order, including its stable angular rank.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PieSlice<T> {
    /// Owned original source datum, with all identifiers and metadata preserved.
    pub data: T,
    /// Rank in angular order, independent of this entry's input position.
    pub index: usize,
    /// Original resolved value, including zero or negative values.
    pub value: f64,
    /// Starting angle of this datum's allocated interval.
    pub start_angle: f64,
    /// Ending angle, including its allocated padding.
    pub end_angle: f64,
    /// Unsigned-with-respect-to-sweep pad angle (authored negative pads remain negative).
    pub pad_angle: f64,
}
impl<T> PieSlice<T> {
    /// Materialize arc input with explicit radii; layout and geometry remain separate.
    pub fn arc_datum(&self, inner_radius: f64, outer_radius: f64) -> super::ArcDatum {
        super::ArcDatum {
            inner_radius,
            outer_radius,
            start_angle: self.start_angle,
            end_angle: self.end_angle,
            pad_angle: self.pad_angle,
        }
    }
}
/// Reusable pie layout over finite values, with checked resource and ownership boundaries.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Pie {
    value: Option<f64>,
    order: PieOrder,
    angles: PieAngles,
    limits: ShapeLimits,
}
impl Pie {
    /// Default identity value, descending value order and a full turn from zero.
    pub fn new() -> Self {
        Self::default()
    }
    /// Override numeric input values with a constant, or restore the identity accessor.
    pub fn value(mut self, value: Option<f64>) -> Self {
        self.value = value;
        self
    }
    /// Select a portable order; it replaces any prior portable order.
    pub fn order(mut self, order: PieOrder) -> Self {
        self.order = order;
        self
    }
    /// Set the constant start angle.
    pub fn start_angle(mut self, value: f64) -> Self {
        self.angles.start_angle = value;
        self
    }
    /// Set the constant end angle.
    pub fn end_angle(mut self, value: f64) -> Self {
        self.angles.end_angle = value;
        self
    }
    /// Set the constant per-datum pad angle.
    pub fn pad_angle(mut self, value: f64) -> Self {
        self.angles.pad_angle = value;
        self
    }
    /// Set source-entry work bounds.
    pub fn limits(mut self, limits: ShapeLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Validate a deserialized reusable configuration.
    pub fn validate(&self) -> ChartResult<()> {
        self.angles.validate()?;
        if self.value.is_none_or(f64::is_finite) {
            Ok(())
        } else {
            Err(domain("Pie value constants must be finite."))
        }
    }
    /// Layout numeric inputs, applying the configured constant or identity value accessor.
    pub fn layout(&self, data: &[f64]) -> ChartResult<Vec<PieSlice<f64>>> {
        self.layout_by(data, |d, _, _| Ok(self.value.unwrap_or(*d)))
    }
    /// Layout arbitrary owned data using a native value accessor and configured order/angles.
    pub fn layout_by<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.build(data, value, self.angles, None)
    }
    /// Layout arbitrary data with exact-length materialized values, honoring a configured constant.
    pub fn layout_materialized<T: Clone>(
        &self,
        data: &[T],
        values: &[f64],
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.validate_len(data.len())?;
        if values.len() != data.len() {
            return Err(super::invalid(
                "Pie values must match the source-entry count.",
            ));
        }
        self.layout_by(data, |_, i, _| Ok(self.value.unwrap_or(values[i])))
    }
    /// A native datum comparator replaces configured value sorting; stable ties retain input order.
    pub fn layout_by_comparator<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
        mut compare: impl FnMut(&T, &T) -> Ordering,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.build(
            data,
            value,
            self.angles,
            Some(&mut |i, j, _| Ok(compare(&data[i], &data[j]))),
        )
    }
    /// Fallible datum-and-value comparison through bounded stable sorting.
    pub fn layout_by_fallible_comparator<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
        mut compare: impl FnMut(&T, f64, &T, f64) -> ChartResult<Ordering>,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.build(
            data,
            value,
            self.angles,
            Some(&mut |i, j, v| compare(&data[i], v[i], &data[j], v[j])),
        )
    }
    /// A native value comparator replaces configured sorting; stable ties retain input order.
    pub fn layout_by_value_comparator<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
        mut compare: impl FnMut(f64, f64) -> Ordering,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.build(
            data,
            value,
            self.angles,
            Some(&mut |i, j, v| Ok(compare(v[i], v[j]))),
        )
    }
    /// Resolve start/end/padding once from the whole input before running the value accessor.
    pub fn layout_by_angles<T: Clone>(
        &self,
        data: &[T],
        value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
        angles: impl FnOnce(&[T]) -> ChartResult<PieAngles>,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.validate_len(data.len())?;
        self.build(data, value, angles(data)?, None)
    }
    /// Layout owned JSON data through a fallible registered comparator and the shared allocator.
    pub fn layout_materialized_with(
        &self,
        data: &[serde_json::Value],
        values: &[f64],
        compare: &dyn PieComparator,
    ) -> ChartResult<Vec<PieSlice<serde_json::Value>>> {
        self.validate_len(data.len())?;
        if values.len() != data.len() {
            return Err(super::invalid(
                "Pie values must match the source-entry count.",
            ));
        }
        self.build(
            data,
            |_, i, _| Ok(self.value.unwrap_or(values[i])),
            self.angles,
            Some(&mut |i, j, v| compare.compare(&data[i], v[i], &data[j], v[j])),
        )
    }
    fn validate_len(&self, n: usize) -> ChartResult<()> {
        self.validate()?;
        if n > self.limits.max_points {
            Err(limit("Pie source-entry limit exceeded."))
        } else {
            Ok(())
        }
    }
    fn build<T: Clone>(
        &self,
        data: &[T],
        mut value: impl FnMut(&T, usize, &[T]) -> ChartResult<f64>,
        angles: PieAngles,
        mut compare: Option<&mut IndexComparator<'_>>,
    ) -> ChartResult<Vec<PieSlice<T>>> {
        self.validate_len(data.len())?;
        angles.validate()?;
        let n = data.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        let mut values = Vec::with_capacity(n);
        let mut sum = 0.;
        for (i, d) in data.iter().enumerate() {
            let v = value(d, i, data)?;
            if !v.is_finite() {
                return Err(domain("Pie values must be finite."));
            }
            if v > 0. {
                sum += v;
            }
            values.push(v);
        }
        if !sum.is_finite() {
            return Err(domain("Pie positive-value sum overflowed."));
        }
        let mut indices: Vec<usize> = (0..n).collect();
        if let Some(ref mut compare) = compare {
            stable_sort_by(&mut indices, |a, b| compare(a, b, &values))?;
        } else {
            match self.order {
                PieOrder::Input => {}
                PieOrder::ValuesAscending => {
                    indices.sort_by(|a, b| values[*a].partial_cmp(&values[*b]).expect("finite"))
                }
                PieOrder::ValuesDescending => {
                    indices.sort_by(|a, b| values[*b].partial_cmp(&values[*a]).expect("finite"))
                }
            }
        }
        let mut a0 = angles.start_angle;
        let da = (angles.end_angle - a0).clamp(-TAU, TAU);
        let p = (da.abs() / n as f64).min(angles.pad_angle);
        let pa = p * if da < 0. { -1. } else { 1. };
        let k = if sum != 0. {
            (da - n as f64 * pa) / sum
        } else {
            0.
        };
        let mut result: Vec<_> = data
            .iter()
            .zip(&values)
            .map(|(d, v)| PieSlice {
                data: d.clone(),
                index: 0,
                value: *v,
                start_angle: 0.,
                end_angle: 0.,
                pad_angle: p,
            })
            .collect();
        for (i, j) in indices.into_iter().enumerate() {
            let v = values[j];
            let a1 = a0 + if v > 0. { v * k } else { 0. } + pa;
            if !a0.is_finite() || !a1.is_finite() {
                return Err(domain("Pie angle allocation overflowed."));
            }
            result[j].index = i;
            result[j].start_angle = a0;
            result[j].end_angle = a1;
            a0 = a1;
        }
        Ok(result)
    }
}
type IndexComparator<'a> = dyn FnMut(usize, usize, &[f64]) -> ChartResult<Ordering> + 'a;

// Fallible stable merge sorting avoids exposing a partial ordering on callback errors.
// It makes at most n * ceil(log2(n)) comparisons, even for inconsistent callbacks.
fn stable_sort_by(
    indices: &mut [usize],
    mut compare: impl FnMut(usize, usize) -> ChartResult<Ordering>,
) -> ChartResult<()> {
    let n = indices.len();
    let mut work = indices.to_vec();
    let mut width = 1usize;
    while width < n {
        for start in (0..n).step_by(width.saturating_mul(2)) {
            let mid = start.saturating_add(width).min(n);
            let end = mid.saturating_add(width).min(n);
            let (mut left, mut right) = (start, mid);
            for item in &mut work[start..end] {
                if left < mid
                    && (right == end
                        || compare(indices[left], indices[right])? != Ordering::Greater)
                {
                    *item = indices[left];
                    left += 1;
                } else {
                    *item = indices[right];
                    right += 1;
                }
            }
        }
        indices.copy_from_slice(&work);
        width = width.saturating_mul(2);
    }
    Ok(())
}
