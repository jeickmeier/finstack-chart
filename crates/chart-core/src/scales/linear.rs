use super::{Bounds, OutsidePolicy, ScaleCapabilities, error};
use crate::grammar::Extent;
use crate::{ChartResult, DiagnosticCode};

/// Baseline/zero contribution policy, applied to automatically trained domains only.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Baseline {
    /// No additional domain contribution.
    #[default]
    None,
    /// Include zero.
    Zero,
    /// Include a finite declared baseline.
    Value(f64),
}

/// Continuous domain policy. Explicit nonconstant domains remain exact and retain direction.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContinuousDomain {
    /// Exact requested domain; baseline/padding/nice apply only when this is absent.
    pub explicit: Option<Bounds>,
    /// Additional auto-domain baseline.
    pub baseline: Baseline,
    /// Nonnegative fraction of auto-domain width added to each side.
    pub padding: f64,
    /// Expand automatic domains to the fixed 1/2/5 tick-step grid.
    pub nice: bool,
    /// Fixed training target for nice policy, independent of layout/viewport (2–128).
    pub nice_ticks: usize,
}
impl Default for ContinuousDomain {
    fn default() -> Self {
        Self {
            explicit: None,
            baseline: Baseline::None,
            padding: 0.,
            nice: false,
            nice_ticks: 6,
        }
    }
}
impl ContinuousDomain {
    /// Exact nonconstant domain, without automatic baseline/padding/nice changes.
    pub fn explicit(bounds: Bounds) -> Self {
        Self {
            explicit: Some(bounds),
            ..Self::default()
        }
    }
    /// Validate and train limits before constant-range, padding and nice policies.
    pub(crate) fn trained_limits(self, contribution: Option<Extent>) -> ChartResult<Bounds> {
        if !self.padding.is_finite() || self.padding < 0. || !(2..=128).contains(&self.nice_ticks) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Padding must be finite/nonnegative and nice tick target must be 2–128.",
            ));
        }
        if let Baseline::Value(v) = self.baseline
            && !v.is_finite()
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Baseline must be finite.",
            ));
        }
        let bounds = if let Some(explicit) = self.explicit {
            explicit
        } else {
            let mut b = if let Some(e) = contribution {
                if e.minimum > e.maximum {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Domain contributions must have ordered extrema.",
                    ));
                }
                Bounds::new(e.minimum, e.maximum)?
            } else {
                Bounds::new(0., 1.)?
            };
            if let Some(value) = match self.baseline {
                Baseline::None => None,
                Baseline::Zero => Some(0.),
                Baseline::Value(v) => Some(v),
            } {
                b = Bounds::new(b.minimum().min(value), b.maximum().max(value))?;
            }
            b
        };
        Ok(bounds)
    }
    /// Resolve empty/constant/descending domains with checked finite arithmetic.
    pub fn resolve(self, contribution: Option<Extent>) -> ChartResult<Bounds> {
        let mut bounds = self.trained_limits(contribution)?;
        // Symmetric 5% expansion for nonzero constants; zero expands to [-1,1].
        // At subnormal magnitudes use the smallest positive representable step.
        if bounds.start() == bounds.end() {
            let center = bounds.start();
            let delta = if center == 0. {
                1.
            } else {
                (center.abs() * 0.05).max(f64::from_bits(1))
            };
            bounds = Bounds::new(center - delta, center + delta)
                .map_err(|_| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "A symmetric constant-domain expansion cannot remain finite.",
                    )
                })?
                .distinct()?;
        }
        if self.explicit.is_none() {
            if self.padding > 0. {
                let pad = (bounds.maximum() * 0.5 - bounds.minimum() * 0.5) * self.padding * 2.;
                bounds =
                    Bounds::new(bounds.minimum() - pad, bounds.maximum() + pad).map_err(|_| {
                        error(
                            DiagnosticCode::PrecisionLoss,
                            "Domain padding exceeds finite precision.",
                        )
                    })?;
            }
            if self.nice {
                let step = tick_step(bounds, self.nice_ticks)?;
                bounds = Bounds::new(
                    (bounds.minimum() / step).floor() * step,
                    (bounds.maximum() / step).ceil() * step,
                )
                .map_err(|_| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "Nice-domain expansion exceeds finite precision.",
                    )
                })?;
            }
        }
        bounds.distinct()
    }
}

/// Immutable affine scale with separate trained domain, viewport and destination range.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearScale {
    domain: Bounds,
    view: Bounds,
    range: Bounds,
    outside: OutsidePolicy,
}
impl LinearScale {
    /// Resolve training once; viewport and range never alter the contributing population.
    pub fn resolve(
        contribution: Option<Extent>,
        options: ContinuousDomain,
        range: Bounds,
        viewport: Option<Bounds>,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        let domain = options.resolve(contribution)?;
        Self::from_domains(domain, viewport.unwrap_or(domain), range, outside)
    }
    pub(crate) fn from_domains(
        domain: Bounds,
        view: Bounds,
        range: Bounds,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        Ok(Self {
            domain: domain.distinct()?,
            view: view.distinct()?,
            range: range.distinct()?,
            outside,
        })
    }
    // Reference expansion may deliberately collapse a viewport. Keep authored viewport
    // validation in resolve/from_domains while retaining ggplot2 midpoint projection.
    pub(crate) fn with_reference_viewport(mut self, view: Bounds) -> Self {
        self.view = view;
        self
    }
    /// Trained full domain, independent of viewport/resize.
    pub fn domain(&self) -> Bounds {
        self.domain
    }
    /// Visible domain used for projection and ticks.
    pub fn viewport(&self) -> Bounds {
        self.view
    }
    /// Explicit destination endpoints; descending ranges are valid.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Continuous inversion is available; category lookup is not.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: true,
            category_lookup: false,
        }
    }
    /// Map a finite value, respecting declared out-of-domain policy.
    pub fn map(&self, value: f64) -> ChartResult<Option<f64>> {
        if !value.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Cannot project a non-finite coordinate.",
            ));
        }
        let value = match self.outside {
            OutsidePolicy::Omit if !self.view.contains(value) => return Ok(None),
            OutsidePolicy::Clamp => value.clamp(self.view.minimum(), self.view.maximum()),
            _ => value,
        };
        if self.view.start() == self.view.end() {
            return interpolate(self.range, 0.5).map(Some);
        }
        super::numeric::legacy_map(self.view, self.range, value).map(Some)
    }
    /// Invert a finite destination coordinate; extrapolation is explicit and unclamped.
    pub fn invert(&self, position: f64) -> ChartResult<f64> {
        super::numeric::legacy_map(self.range, self.view, position)
    }
    /// Bounded 1/2/5 ticks in visible-domain order, with unique round-trip-safe labels.
    pub fn ticks(&self, target: usize, max_ticks: usize) -> ChartResult<Vec<NumericTick>> {
        if self.view.start() == self.view.end() {
            if max_ticks == 0 || max_ticks > 4096 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Tick output budget must be 1–4096.",
                ));
            }
            return Ok(vec![NumericTick {
                value: self.view.start(),
                label: self.view.start().to_string(),
            }]);
        }
        numeric_ticks(self.view, target, max_ticks)
    }
}

pub(crate) fn fraction(bounds: Bounds, value: f64) -> ChartResult<f64> {
    if !value.is_finite() {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Scale coordinate must be finite.",
        ));
    }
    let denominator = bounds.end() - bounds.start();
    let numerator = value - bounds.start();
    let t = if denominator.is_finite() && numerator.is_finite() {
        numerator / denominator
    } else {
        (value * 0.5 - bounds.start() * 0.5) / (bounds.end() * 0.5 - bounds.start() * 0.5)
    };
    if t.is_finite() {
        Ok(t)
    } else {
        Err(error(
            DiagnosticCode::PrecisionLoss,
            "Scale normalization cannot preserve a finite coordinate at this domain resolution.",
        ))
    }
}
pub(crate) fn interpolate(bounds: Bounds, t: f64) -> ChartResult<f64> {
    let value = if t == 0. {
        bounds.start()
    } else if t == 1. {
        bounds.end()
    } else if (0. ..=1.).contains(&t) {
        bounds.start() * (1. - t) + bounds.end() * t
    } else {
        bounds.start() + (bounds.end() - bounds.start()) * t
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(error(
            DiagnosticCode::PrecisionLoss,
            "Projected or inverted coordinate exceeds finite precision.",
        ))
    }
}

/// Numeric tick value in data units and its deterministic portable label.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericTick {
    /// Exact selected binary64 tick.
    pub value: f64,
    /// Locale-independent label using Unicode minus, never machine locale formatting.
    pub label: String,
}
pub(crate) fn tick_step(bounds: Bounds, target: usize) -> ChartResult<f64> {
    if !(2..=128).contains(&target) {
        return Err(error(
            DiagnosticCode::Validation,
            "Tick target must be 2–128.",
        ));
    }
    let raw = (bounds.maximum() * 0.5 - bounds.minimum() * 0.5) / (target - 1) as f64 * 2.;
    if raw == 0. {
        return Ok(f64::from_bits(1));
    }
    if !raw.is_finite() || raw < 0. {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Tick step cannot be represented.",
        ));
    }
    let power = 10_f64.powf(raw.log10().floor()).max(f64::from_bits(1));
    let ratio = raw / power;
    let multiplier = if ratio <= 1. {
        1.
    } else if ratio <= 2. {
        2.
    } else if ratio <= 5. {
        5.
    } else {
        10.
    };
    let step = power * multiplier;
    if step.is_finite() && step > 0. {
        Ok(step)
    } else {
        Err(error(
            DiagnosticCode::PrecisionLoss,
            "Tick step exceeds finite precision.",
        ))
    }
}
pub(super) fn numeric_ticks(
    bounds: Bounds,
    target: usize,
    max_ticks: usize,
) -> ChartResult<Vec<NumericTick>> {
    if max_ticks == 0 || max_ticks > 4096 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Tick output budget must be 1–4096.",
        ));
    }
    let step = tick_step(bounds, target)?;
    let first = (bounds.minimum() / step).ceil() * step;
    let mut values = vec![];
    for i in 0..=max_ticks {
        let raw = first + i as f64 * step;
        // Decimal 1/2/5 grids should label 0.3, not accumulated 0.30000000000000004.
        // Round only to the grid's declared decimal precision, before membership checks.
        let decimals = (-step.log10().floor()).max(0.);
        let value = if raw.is_finite() && decimals <= 15. {
            format!("{raw:.precision$}", precision = decimals as usize)
                .parse::<f64>()
                .map_err(|_| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "Numeric tick rounding failed.",
                    )
                })?
        } else {
            raw
        };
        if !value.is_finite() || value > bounds.maximum() {
            break;
        }
        if value < bounds.minimum() {
            continue;
        }
        if values.last().is_some_and(|v| *v >= value) {
            // A decimal grid can coalesce at a large binary64 origin. Keep only
            // distinct representable ticks; candidate work is still bounded below.
            continue;
        }
        if values.len() == max_ticks {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Tick count exceeds the explicit budget.",
            ));
        }
        values.push(if value == 0. { 0. } else { value });
    }
    if values.is_empty() {
        values.push(bounds.minimum());
        if max_ticks > 1 {
            values.push(bounds.maximum());
        }
    }
    if bounds.start() > bounds.end() {
        values.reverse();
    }
    let mut output = vec![];
    for value in values {
        output.push(NumericTick {
            value,
            label: format_number(value),
        });
    }
    Ok(output)
}
/// Exact shortest-round-trip scalar formatting, with Unicode minus and normalized zero.
pub fn format_number(value: f64) -> String {
    if value == 0. {
        return "0".into();
    }
    let magnitude = value.abs();
    let text = if !(1e-6..1e9).contains(&magnitude) {
        format!("{value:e}")
    } else {
        value.to_string()
    };
    text.replace('-', "−")
}
