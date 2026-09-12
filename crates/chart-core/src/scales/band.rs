use super::{Bounds, ScaleCapabilities, error};
use crate::{ChartResult, DiagnosticCode};
use std::collections::{BTreeMap, BTreeSet};

/// Stable categorical-domain and band-space policy.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
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
    window: std::ops::Range<usize>,
    index: BTreeMap<String, usize>,
    spacing: super::spacing::Spacing,
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
        let spacing = super::spacing::Spacing::new(
            labels.len(),
            range,
            &super::BandSpec::<String> {
                domain: None,
                padding_inner: options.inner_padding,
                padding_outer: options.outer_padding,
                align: 0.5,
                round: false,
            },
            super::ScaleCompatibility::Legacy,
            false,
        )?;
        Ok(Self {
            labels: labels.to_vec(),
            window: 0..labels.len(),
            index: labels
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), i))
                .collect(),
            spacing,
        })
    }
    /// Resolve D3 spacing while preserving the chart's stable source-category catalog.
    pub fn resolve_d3(
        first_seen: &[String],
        options: &super::BandSpec,
        range: Bounds,
    ) -> ChartResult<Self> {
        let prepared = super::CategoryScale::band(
            super::BandSpec {
                domain: Some(options.domain.as_deref().unwrap_or(first_seen).to_vec()),
                ..options.clone()
            },
            range,
        )?;
        let labels = prepared.domain().to_vec();
        Ok(Self {
            index: labels
                .iter()
                .enumerate()
                .map(|(i, k)| (k.clone(), i))
                .collect(),
            window: 0..labels.len(),
            spacing: super::spacing::Spacing::new(
                labels.len(),
                range,
                prepared.spec(),
                super::ScaleCompatibility::D3,
                false,
            )?,
            labels,
        })
    }
    /// Project a numeric minor candidate in the reference category-index space.
    pub(crate) fn reference_viewport(&self) -> Option<[crate::interpolate::Number; 2]> {
        self.spacing.reference_viewport()
    }
    pub(crate) fn reference_minor(&self, value: f64) -> ChartResult<Option<f64>> {
        self.spacing.reference_minor(value)
    }
    pub(crate) fn with_reference_expansion(
        mut self,
        expansion: super::GgplotExpansion,
        limits: Option<&[crate::interpolate::Number]>,
        observed: &[String],
    ) -> ChartResult<Self> {
        let continuous = super::spacing::observed_extent(
            observed
                .iter()
                .filter_map(|label| self.index.get(label).copied()),
            !observed.is_empty(),
        );
        self.spacing = self
            .spacing
            .with_reference_expansion(expansion, limits, continuous)?;
        Ok(self)
    }
    /// Nonnegative interval between neighboring band starts.
    pub fn step(&self) -> f64 {
        self.spacing.step()
    }
    /// Nonnegative band width, including zero-width D3 bands.
    pub fn bandwidth(&self) -> f64 {
        self.spacing.bandwidth()
    }
    /// Lower band start, independent of range orientation.
    pub fn start(&self, label: &str) -> ChartResult<Option<f64>> {
        self.extent(label).map(|e| e.map(Bounds::minimum))
    }
    /// Explicit resolved category order, including absent labels retained in the domain.
    pub fn domain(&self) -> &[String] {
        &self.labels
    }
    /// Current category window, preserving the complete trained domain separately.
    pub fn visible_domain(&self) -> &[String] {
        &self.labels[self.window.clone()]
    }
    /// Restrict presentation to an inclusive stable-label window without changing training.
    pub fn with_window(mut self, first: &str, last: &str) -> ChartResult<Self> {
        self.window = super::category_window(&self.labels, first, last)?;
        self.spacing = self.spacing.resize(self.window.len())?;
        Ok(self)
    }
    /// Authored destination range, including descending direction.
    pub fn range(&self) -> Bounds {
        self.spacing.range
    }
    /// Categorical lookup is available; a continuous numeric inverse is not.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: false,
            category_lookup: self.spacing.supports_lookup(),
        }
    }
    /// Map a known label to its center; absent labels are explicitly missing.
    pub fn center(&self, label: &str) -> ChartResult<Option<f64>> {
        let Some(&i) = self.index.get(label) else {
            return Ok(None);
        };
        if !self.window.contains(&i) {
            return Ok(None);
        }
        let i = i - self.window.start;
        self.spacing.center(i)
    }
    /// Guide-only D3 centering; data marks retain the band's full center.
    pub fn guide_position(&self, label: &str, offset: f64) -> ChartResult<Option<f64>> {
        if !offset.is_finite() {
            return Err(error(
                DiagnosticCode::Validation,
                "Guide offset must be finite.",
            ));
        }
        self.extent(label)?
            .map(|bounds| {
                let center = (self.bandwidth() - 2. * offset).max(0.) / 2.;
                let center = if self.spacing.round {
                    center.round()
                } else {
                    center
                };
                let start = bounds.start().min(bounds.end());
                let value = start + center;
                if !value.is_finite() {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Guide center exceeds finite positions.",
                    ));
                }
                Ok(value)
            })
            .transpose()
    }
    /// Oriented destination edges of a band; widths are not fabricated for absent labels.
    pub fn extent(&self, label: &str) -> ChartResult<Option<Bounds>> {
        let Some(&i) = self.index.get(label) else {
            return Ok(None);
        };
        if !self.window.contains(&i) {
            return Ok(None);
        }
        let i = i - self.window.start;
        self.spacing.extent(i)
    }
    /// Locate a band containing a destination position; gaps/outside positions yield `None`.
    pub fn category_at(&self, position: f64) -> ChartResult<Option<&str>> {
        Ok(self
            .spacing
            .band_at(position)?
            .map(|i| self.labels[self.window.start + i].as_str()))
    }
}
