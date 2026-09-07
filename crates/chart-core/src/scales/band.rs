use super::{Bounds, ScaleCapabilities, error};
use crate::{ChartResult, DiagnosticCode};
use std::collections::{BTreeMap, BTreeSet};

/// Stable categorical-domain and band-space policy.
#[derive(Clone, Debug, PartialEq)]
pub struct BandOptions {
    /// Exact domain/order when provided; otherwise retained first-seen labels are used.
    pub domain: Option<Vec<String>>,
    /// Fraction of a step reserved between bands, in [0,1).
    pub inner_padding: f64,
    /// Empty step fractions at either end, nonnegative.
    pub outer_padding: f64,
}
impl Default for BandOptions {
    fn default() -> Self {
        Self {
            domain: None,
            inner_padding: 0.1,
            outer_padding: 0.05,
        }
    }
}

/// Immutable categorical centers/extents. There is no pretended numeric inverse.
#[derive(Clone, Debug, PartialEq)]
pub struct BandScale {
    labels: Vec<String>,
    index: BTreeMap<String, usize>,
    range: Bounds,
    inner: f64,
    outer: f64,
    denominator: f64,
}
impl BandScale {
    /// Resolve explicit order or stable first-seen order; unknown explicit-domain labels do
    /// not filter source rows. Missing mapped categories return `None` at projection.
    pub fn resolve(
        first_seen: &[String],
        options: &BandOptions,
        range: Bounds,
    ) -> ChartResult<Self> {
        range.distinct()?;
        if !options.inner_padding.is_finite()
            || !(0. ..1.).contains(&options.inner_padding)
            || !options.outer_padding.is_finite()
            || options.outer_padding < 0.
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Band padding must be finite; inner is [0,1), outer is nonnegative.",
            ));
        }
        let labels = options.domain.as_deref().unwrap_or(first_seen);
        if labels.iter().collect::<BTreeSet<_>>().len() != labels.len() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Band domain labels must be unique.",
            ));
        }
        let denominator =
            (labels.len() as f64 - options.inner_padding + 2. * options.outer_padding).max(1.);
        if !denominator.is_finite() {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Band padding cannot preserve finite extents.",
            ));
        }
        Ok(Self {
            labels: labels.to_vec(),
            index: labels
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), i))
                .collect(),
            range,
            inner: options.inner_padding,
            outer: options.outer_padding,
            denominator,
        })
    }
    /// Explicit resolved category order, including absent labels retained in the domain.
    pub fn domain(&self) -> &[String] {
        &self.labels
    }
    /// Authored destination range, including descending direction.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Categorical lookup is available; a continuous numeric inverse is not.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: false,
            category_lookup: true,
        }
    }
    /// Map a known label to its center; absent labels are explicitly missing.
    pub fn center(&self, label: &str) -> ChartResult<Option<f64>> {
        let Some(&i) = self.index.get(label) else {
            return Ok(None);
        };
        Ok(Some(super::linear::interpolate(
            self.range,
            (self.outer + i as f64 + (1. - self.inner) * 0.5) / self.denominator,
        )?))
    }
    /// Oriented destination edges of a band; widths are not fabricated for absent labels.
    pub fn extent(&self, label: &str) -> ChartResult<Option<Bounds>> {
        let Some(&i) = self.index.get(label) else {
            return Ok(None);
        };
        let a = (self.outer + i as f64) / self.denominator;
        let b = (self.outer + i as f64 + 1. - self.inner) / self.denominator;
        Ok(Some(Bounds::new(
            super::linear::interpolate(self.range, a)?,
            super::linear::interpolate(self.range, b)?,
        )?))
    }
    /// Locate a band containing a destination position; gaps/outside positions yield `None`.
    pub fn category_at(&self, position: f64) -> ChartResult<Option<&str>> {
        if !position.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Category lookup needs a finite position.",
            ));
        }
        if !self.range.contains(position) {
            return Ok(None);
        }
        let slot = super::linear::fraction(self.range, position)? * self.denominator - self.outer;
        if slot < 0. {
            return Ok(None);
        }
        let i = slot.floor() as usize;
        if i >= self.labels.len() {
            // Include the last band's closed far edge, including a descending range.
            if let Some(last) = self.labels.last()
                && self
                    .extent(last)?
                    .is_some_and(|extent| position == extent.end())
            {
                return Ok(Some(last));
            }
            return Ok(None);
        }
        if slot - i as f64 > 1. - self.inner {
            return Ok(None);
        }
        Ok(Some(&self.labels[i]))
    }
}
