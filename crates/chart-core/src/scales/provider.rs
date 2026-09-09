//! Checked positional provider results, shared by mark projection and independent guides.
use super::{Bounds, CalendarInterval, ScaleCapabilities, error};
use crate::{ChartResult, DiagnosticCode, Limits, composition::ScaleValue, grammar::ValueSpace};
use std::sync::Arc;

/// Typed scale tick arguments; a missing count uses the provider's own default.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GuideTickArguments {
    /// Finite density hint, independent from the hard tick/work budget.
    pub count: Option<f64>,
    /// Optional scale formatter argument, retained independently of value selection.
    pub specifier: Option<String>,
    /// Explicit calendar interval in place of a numeric density hint.
    pub interval: Option<CalendarInterval>,
}
impl GuideTickArguments {
    /// Check argument shape and work bounds before invoking a provider.
    pub fn validate(&self, max_bytes: usize) -> ChartResult<()> {
        if self.count.is_some_and(|n| !n.is_finite())
            || (self.count.is_some() && self.interval.is_some())
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Tick arguments require one finite count or calendar interval.",
            ));
        }
        if let Some(interval) = self.interval {
            interval.validate()?;
        }
        crate::limits::require_within(
            self.specifier.as_ref().is_none_or(|s| s.len() <= max_bytes),
            "provider tick argument byte",
        )
    }
}

/// Optional band-start mapping metadata. Point providers may report zero bandwidth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProviderBand {
    /// Nonnegative finite bandwidth in the same units as the returned positions.
    pub bandwidth: f64,
    /// Whether guide band centering rounds its nonnegative center displacement.
    pub round: bool,
}

/// Immutable native positional mapping. It has no compulsory inverse or host objects.
///
/// Implementations are trusted native code and must honor the supplied collection/work
/// limits. Core checks returned values and bounds map/tick calls through its existing
/// geometry and guide budgets; it cannot preempt a native callback. A provider maps band
/// starts when `band` is present; the shared adapter owns mark/guide centering.
pub trait PositionalScale: Send + Sync {
    /// Owned semantic domain in its declared order, including exact timestamp units.
    fn domain(&self) -> &[ScaleValue];
    /// Actual finite range endpoints, independent of whether they are increasing.
    fn range(&self) -> Bounds;
    /// Forward numeric output; `None` deliberately omits this value.
    fn map(&self, value: &ScaleValue) -> ChartResult<Option<f64>>;
    /// Optional band-start/rounding capability.
    fn band(&self) -> Option<ProviderBand> {
        None
    }
    /// Optional automatic ticks. `None` falls back to the retained domain order.
    fn ticks(
        &self,
        _arguments: &GuideTickArguments,
        _max_ticks: usize,
    ) -> ChartResult<Option<Vec<ScaleValue>>> {
        Ok(None)
    }
    /// Optional formatter over original semantic values and the complete selected list.
    fn format(
        &self,
        _value: &ScaleValue,
        _index: usize,
        _values: &[ScaleValue],
        _arguments: &GuideTickArguments,
    ) -> ChartResult<Option<String>> {
        Ok(None)
    }
    /// Whether this provider supports a semantic inverse. Never inferred from numeric output.
    fn has_inverse(&self) -> bool {
        false
    }
    /// Optional inverse in the same semantic units as `domain` and `map`.
    fn invert(&self, _position: f64) -> ChartResult<ScaleValue> {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "This positional provider has no inverse.",
        ))
    }
}

/// Checked retained provider, with metadata captured once for all guides on its scale.
#[derive(Clone)]
pub struct CheckedPositionalScale {
    provider: Arc<dyn PositionalScale>,
    domain: Vec<ScaleValue>,
    range: Bounds,
    band: Option<ProviderBand>,
    inverse: bool,
    space: ValueSpace,
    limits: Limits,
}
impl std::fmt::Debug for CheckedPositionalScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedPositionalScale")
            .field("domain", &self.domain)
            .field("range", &self.range)
            .field("band", &self.band)
            .field("inverse", &self.inverse)
            .finish_non_exhaustive()
    }
}
impl CheckedPositionalScale {
    /// Validate metadata before any mapping, tick or formatting callbacks are run.
    pub fn new(
        provider: Arc<dyn PositionalScale>,
        space: ValueSpace,
        limits: Limits,
        max_values: usize,
    ) -> ChartResult<Self> {
        let domain = provider.domain();
        crate::limits::require_within(domain.len() <= max_values, "provider domain value")?;
        let supplied_range = provider.range();
        let range = Bounds::new(supplied_range.start(), supplied_range.end())?;
        let band = provider.band();
        if band.is_some_and(|b| !b.bandwidth.is_finite() || b.bandwidth < 0.) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Provider bandwidth must be finite and nonnegative.",
            ));
        }
        let result = Self {
            domain: domain.to_vec(),
            range,
            band,
            inverse: provider.has_inverse(),
            provider,
            space,
            limits,
        };
        result.validate_values(&result.domain, max_values)?;
        Ok(result)
    }
    fn validate_value(&self, value: &ScaleValue) -> ChartResult<()> {
        let valid = match (&self.space, value) {
            (
                ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. },
                ScaleValue::Number(n),
            ) => n.is_finite(),
            (ValueSpace::Categorical { .. }, ScaleValue::Category(_)) => true,
            (ValueSpace::Timestamp { representation, .. }, ScaleValue::Timestamp { unit, .. }) => {
                representation.unit == *unit
            }
            _ => false,
        };
        if !valid {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Provider value differs from its prepared semantic space or is nonfinite.",
            ));
        }
        if let ScaleValue::Category(label) = value {
            crate::limits::require_within(
                label.len() <= self.limits.max_text_bytes,
                "provider category byte",
            )?;
        }
        Ok(())
    }
    fn validate_values(&self, values: &[ScaleValue], max_values: usize) -> ChartResult<()> {
        crate::limits::require_within(values.len() <= max_values, "provider selected value")?;
        let mut remaining = self.limits.max_text_bytes;
        for value in values {
            self.validate_value(value)?;
            if let ScaleValue::Category(label) = value {
                crate::limits::require_within(
                    label.len() <= remaining,
                    "provider domain/tick category byte",
                )?;
                remaining -= label.len();
            }
        }
        Ok(())
    }
    /// Frozen domain values; requesting them does not retrain the provider.
    pub fn domain(&self) -> &[ScaleValue] {
        &self.domain
    }
    /// Actual retained range endpoints for guide geometry.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Retained optional band metadata.
    pub fn band(&self) -> Option<ProviderBand> {
        self.band
    }
    /// Report optional inversion independently of categorical lookup.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: self.inverse && !matches!(self.space, ValueSpace::Categorical { .. }),
            category_lookup: matches!(self.space, ValueSpace::Categorical { .. }),
        }
    }
    fn raw_map(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
        self.validate_value(value)?;
        let position = self.provider.map(value)?;
        if position.is_some_and(|n| !n.is_finite()) {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Provider mapping returned a nonfinite position.",
            ));
        }
        Ok(position)
    }
    /// Map a semantic mark center, without applying any guide pixel offset.
    pub fn map(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
        self.center(value, self.band.map_or(0., |b| b.bandwidth / 2.))
    }
    fn center(&self, value: &ScaleValue, center: f64) -> ChartResult<Option<f64>> {
        if !center.is_finite() {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Provider center displacement exceeds finite coordinates.",
            ));
        }
        self.raw_map(value)?
            .map(|p| {
                let result = p + center;
                if result.is_finite() {
                    Ok(result)
                } else {
                    Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Provider centering exceeds finite coordinates.",
                    ))
                }
            })
            .transpose()
    }
    /// Center a guide band using the reference offset/rounding rule; offset is painted separately.
    pub fn guide_position(&self, value: &ScaleValue, offset: f64) -> ChartResult<Option<f64>> {
        if !offset.is_finite() {
            return Err(error(
                DiagnosticCode::Validation,
                "Guide offset must be finite.",
            ));
        }
        let center = self.band.map_or(0., |b| {
            let center = ((b.bandwidth - offset * 2.).max(0.)) / 2.;
            if b.round { center.round() } else { center }
        });
        self.center(value, center)
    }
    /// Select bounded semantic ticks, or domain values when automatic ticks are absent.
    pub fn ticks(
        &self,
        arguments: &GuideTickArguments,
        max_ticks: usize,
    ) -> ChartResult<Vec<ScaleValue>> {
        arguments.validate(self.limits.max_text_bytes)?;
        let values = self
            .provider
            .ticks(arguments, max_ticks)?
            .unwrap_or_else(|| self.domain.clone());
        self.validate_values(&values, max_ticks)?;
        Ok(values)
    }
    /// Format a checked list once, preserving full semantic callback context and a shared byte budget.
    pub fn labels(
        &self,
        values: &[ScaleValue],
        arguments: &GuideTickArguments,
        max_ticks: usize,
    ) -> ChartResult<Vec<String>> {
        arguments.validate(self.limits.max_text_bytes)?;
        self.validate_values(values, max_ticks)?;
        let mut remaining = self.limits.max_text_bytes;
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let label = self
                    .provider
                    .format(value, index, values, arguments)?
                    .unwrap_or_else(|| match value {
                        ScaleValue::Number(n) => crate::number::ecmascript(*n),
                        ScaleValue::Category(label) => label.clone(),
                        ScaleValue::Timestamp { value, .. } => value.to_string(),
                    });
                crate::limits::require_within(
                    label.len() <= remaining,
                    "provider formatted label byte",
                )?;
                remaining -= label.len();
                Ok(label)
            })
            .collect()
    }
    /// Invoke an explicitly supported inverse and check its semantic output.
    pub fn invert(&self, position: f64) -> ChartResult<ScaleValue> {
        if !self.inverse {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This positional provider has no inverse.",
            ));
        }
        if !position.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Provider inverse position must be finite.",
            ));
        }
        let value = self.provider.invert(position)?;
        self.validate_value(&value)?;
        Ok(value)
    }
}
