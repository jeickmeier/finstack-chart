use super::*;
use crate::grammar::Extent;

/// Explicit invertible nonlinear numeric coordinate transform.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ScaleTransform {
    /// Positive values, logarithm with finite base greater than one.
    Log {
        /// Logarithm base.
        base: f64,
    },
    /// sign(x) * ln(1 + abs(x)/threshold), linear near zero.
    Symlog {
        /// Positive finite linear threshold.
        threshold: f64,
    },
}
impl ScaleTransform {
    /// Validate authored parameters before reading data.
    pub fn validate(self) -> ChartResult<()> {
        let valid = match self {
            Self::Log { base } => base.is_finite() && base > 1.,
            Self::Symlog { threshold } => threshold.is_finite() && threshold > 0.,
        };
        if valid {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::NumericalDomain,
                "Log base must exceed one; symlog threshold must be finite and positive.",
            ))
        }
    }
    /// Transform one finite value. Nonpositive logarithmic values are ineligible.
    pub fn forward(self, x: f64) -> ChartResult<Option<f64>> {
        self.validate()?;
        if !x.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Nonlinear scale input must be finite.",
            ));
        }
        let v = match self {
            Self::Log { base } => {
                if x <= 0. {
                    return Ok(None);
                }
                if base == 10. {
                    x.log10()
                } else if base == 2. {
                    x.log2()
                } else {
                    x.ln() / base.ln()
                }
            }
            Self::Symlog { threshold } => {
                let ratio = x.abs() / threshold;
                x.signum()
                    * if ratio.is_finite() {
                        ratio.ln_1p()
                    } else {
                        x.abs().ln() - threshold.ln()
                    }
            }
        };
        if v.is_finite() && (x == 0. || v != 0. || matches!(self, Self::Log { .. })) {
            Ok(Some(v))
        } else {
            Err(error(
                DiagnosticCode::PrecisionLoss,
                "Nonlinear transform exceeds finite precision.",
            ))
        }
    }
    /// Invert a finite transformed value, checking the representable output.
    pub fn inverse(self, v: f64) -> ChartResult<f64> {
        self.validate()?;
        if !v.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Nonlinear inverse input must be finite.",
            ));
        }
        let x = match self {
            Self::Log { base } => base.powf(v),
            Self::Symlog { threshold } => {
                let exp = v.abs().exp_m1();
                v.signum()
                    * if exp.is_finite() {
                        exp * threshold
                    } else {
                        (v.abs() + threshold.ln()).exp()
                    }
            }
        };
        if x.is_finite() && (!matches!(self, Self::Log { .. }) || x > 0.) {
            Ok(x)
        } else {
            Err(error(
                DiagnosticCode::PrecisionLoss,
                "Nonlinear inverse cannot represent its result.",
            ))
        }
    }
}
/// Nonlinear scale retaining authored data-space domain and a separate viewport.
#[derive(Clone, Debug, PartialEq)]
pub struct NonlinearScale {
    transform: ScaleTransform,
    domain: Bounds,
    viewport: Bounds,
    inner: LinearScale,
}
impl NonlinearScale {
    /// Resolve policies in transformed space; explicit domains remain exact in source units.
    pub fn resolve(
        contribution: Option<Extent>,
        options: ContinuousDomain,
        transform: ScaleTransform,
        range: Bounds,
        viewport: Option<Bounds>,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        transform.validate()?;
        let forward = |v| {
            transform.forward(v)?.ok_or_else(|| {
                error(
                    DiagnosticCode::NumericalDomain,
                    "Logarithmic domains and baselines must be positive.",
                )
            })
        };
        let bounds = |b: Bounds| -> ChartResult<Bounds> {
            Bounds::new(forward(b.start())?, forward(b.end())?)
        };
        // Validate even ignored policy parameters without letting automatic policy alter an explicit domain.
        ContinuousDomain {
            explicit: Some(Bounds::new(0., 1.)?),
            ..options
        }
        .resolve(None)?;
        let baseline = if options.explicit.is_some() {
            Baseline::None
        } else {
            match options.baseline {
                Baseline::None => Baseline::None,
                Baseline::Zero => Baseline::Value(forward(0.)?),
                Baseline::Value(v) => Baseline::Value(forward(v)?),
            }
        };
        let expand = |b: Bounds| -> ChartResult<Bounds> {
            if matches!(transform, ScaleTransform::Log { .. }) && b.start() == b.end() {
                Bounds::new(b.start() - 0.5, b.end() + 0.5)
            } else {
                Ok(b)
            }
        };
        let explicit = options.explicit.map(|b| expand(bounds(b)?)).transpose()?;
        let input = if explicit.is_some() {
            None
        } else {
            contribution
                .map(|v| {
                    if v.minimum > v.maximum {
                        return Err(error(
                            DiagnosticCode::NumericalDomain,
                            "Domain contributions must be ordered.",
                        ));
                    }
                    let b = expand(bounds(Bounds::new(v.minimum, v.maximum)?)?)?;
                    Ok(Extent {
                        minimum: b.start(),
                        maximum: b.end(),
                    })
                })
                .transpose()?
        };
        let domain = ContinuousDomain {
            explicit,
            baseline,
            ..options
        }
        .resolve(input)?;
        let data = match options.explicit.filter(|b| b.start() != b.end()) {
            Some(b) => b,
            None => Bounds::new(
                transform.inverse(domain.start())?,
                transform.inverse(domain.end())?,
            )?,
        };
        let view = viewport.map(bounds).transpose()?.unwrap_or(domain);
        let inner = LinearScale::from_domains(domain, view, range, outside)?;
        Ok(Self {
            transform,
            domain: data,
            viewport: viewport.unwrap_or(data),
            inner,
        })
    }
    /// Full domain in source units.
    pub fn domain(&self) -> Bounds {
        self.domain
    }
    /// Visible domain in source units.
    pub fn viewport(&self) -> Bounds {
        self.viewport
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.inner.range()
    }
    /// Transformation identity, used by compatibility and inspection.
    pub fn transform(&self) -> ScaleTransform {
        self.transform
    }
    /// Map an eligible value; log-invalid and explicitly omitted inputs return missing.
    pub fn map(&self, x: f64) -> ChartResult<Option<f64>> {
        match self.transform.forward(x)? {
            Some(v) => self.inner.map(v),
            None => Ok(None),
        }
    }
    /// Invert destination coordinates to source units.
    pub fn invert(&self, p: f64) -> ChartResult<f64> {
        self.transform.inverse(self.inner.invert(p)?)
    }
    /// Bounded transformed-space numeric grid, labeled in source units.
    pub fn ticks(&self, target: usize, max_ticks: usize) -> ChartResult<Vec<NumericTick>> {
        self.inner
            .ticks(target, max_ticks)?
            .into_iter()
            .map(|t| {
                let v = self.transform.inverse(t.value)?;
                Ok(NumericTick {
                    value: v,
                    label: format_nonlinear_tick(v),
                })
            })
            .collect()
    }
}

// Guide-only rounding to twelve significant decimal digits suppresses transcendental
// floating noise without rounding mapping coordinates or authored source values.
pub(crate) fn format_nonlinear_tick(value: f64) -> String {
    if value == 0. {
        return "0".into();
    }
    let exponent = value.abs().log10().floor() as i32;
    let text = if (-6..9).contains(&exponent) {
        let precision = (11 - exponent).max(0) as usize;
        let text = format!("{value:.precision$}");
        if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            text
        }
    } else {
        let text = format!("{value:.11e}");
        let (mantissa, exponent) = text.split_once('e').expect("scientific format");
        format!(
            "{}e{exponent}",
            mantissa.trim_end_matches('0').trim_end_matches('.')
        )
    };
    text.replace('-', "−")
}
