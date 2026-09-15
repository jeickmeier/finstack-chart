//! Shared scalar and population arithmetic for the scales 1.4.0 compatibility profile.
//! Formula contracts and independent results: scale-transform-contracts.json.
use super::error;
use crate::{ChartResult, DiagnosticCode};

/// Owned built-in transformation for positional and aesthetic scales.
/// Domain metadata controls break generation; raw evaluation retains IEEE results.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum GgplotTransform {
    /// Explicitly installed pure operation.
    Registered {
        /// Versioned selection resolved through an immutable registry before use.
        selection: Box<crate::grammar::TransformSelection>,
    },
    /// Apply an owned sequence in order; invert in reverse order.
    Compose {
        /// At least one transform. The first owns the default break policy.
        transforms: Vec<GgplotTransform>,
    },
    /// Identity.
    Identity,
    /// Negation.
    Reverse,
    /// Nonnegative square root.
    Sqrt,
    /// Inverse hyperbolic sine.
    Asinh,
    /// Twice the arcsine of the square root.
    Asn,
    /// Inverse hyperbolic tangent.
    Atanh,
    /// Natural logarithm of one plus the input.
    Log1p,
    /// Reciprocal.
    Reciprocal,
    /// Logarithm with the supplied base.
    Log {
        /// Positive base other than one.
        base: f64,
    },
    /// Exponential with the supplied base.
    Exp {
        /// Positive base other than one.
        base: f64,
    },
    /// Box–Cox power transformation; negative shifted inputs reject a population.
    BoxCox {
        /// Power parameter.
        p: f64,
        /// Shift before exponentiation.
        offset: f64,
    },
    /// Signed shifted-power transformation.
    Modulus {
        /// Power parameter.
        p: f64,
        /// Positive shift.
        offset: f64,
    },
    /// Yeo–Johnson transformation.
    YeoJohnson {
        /// Power parameter.
        p: f64,
    },
    /// Smooth logarithmic transform based on inverse hyperbolic sine.
    PseudoLog {
        /// Positive transition width.
        sigma: f64,
        /// Positive base other than one.
        base: f64,
    },
    /// Logistic quantile; inverse is the logistic cumulative probability.
    Logistic {
        /// Location.
        location: f64,
        /// Positive scale.
        scale: f64,
    },
    /// Normal quantile; inverse is the normal cumulative probability.
    Normal {
        /// Mean.
        mean: f64,
        /// Positive standard deviation.
        sd: f64,
    },
}
impl GgplotTransform {
    /// Validate finite parameters and invertibility of authored descriptors.
    pub fn validate(&self) -> ChartResult<()> {
        let mut remaining = 4096usize;
        self.validate_bounded(0, &mut remaining, true)
    }
    pub(crate) fn validate_authoring(&self) -> ChartResult<()> {
        self.validate_bounded(0, &mut 4096, false)
    }
    fn validate_bounded(
        &self,
        depth: usize,
        remaining: &mut usize,
        resolved: bool,
    ) -> ChartResult<()> {
        if depth > crate::interpolate::MAX_VALUE_DEPTH || *remaining == 0 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Transform composition exceeds its depth or operation budget.",
            ));
        }
        *remaining -= 1;
        if let Self::Compose { transforms } = self {
            if transforms.is_empty() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Transform composition requires at least one transform.",
                ));
            }
            for transform in transforms {
                transform.validate_bounded(depth + 1, remaining, resolved)?;
            }
            if (resolved || !self.has_registered()) && self.domain().iter().any(|v| v.is_nan()) {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Transform composition has an invalid domain.",
                ));
            }
            return Ok(());
        }
        if let Self::Registered { selection } = self {
            selection.validate_authoring()?;
            if resolved {
                selection.checked()?;
            }
            return Ok(());
        }
        let base_ok = |b: f64| b.is_finite() && b > 0. && b != 1.;
        let valid = match *self {
            Self::Log { base } | Self::Exp { base } => base_ok(base),
            Self::BoxCox { p, offset } => p.is_finite() && offset.is_finite(),
            Self::Modulus { p, offset } => p.is_finite() && offset.is_finite() && offset > 0.,
            Self::YeoJohnson { p } => p.is_finite(),
            Self::PseudoLog { sigma, base } => sigma.is_finite() && sigma > 0. && base_ok(base),
            Self::Logistic { location, scale } => {
                location.is_finite() && scale.is_finite() && scale > 0.
            }
            Self::Normal { mean, sd } => mean.is_finite() && sd.is_finite() && sd > 0.,
            _ => true,
        };
        if valid {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::NumericalDomain,
                "Invalid built-in transform parameters.",
            ))
        }
    }
    /// Reference break-generation domain, which may differ from raw input validity.
    pub fn domain(&self) -> [f64; 2] {
        match *self {
            Self::Registered { ref selection } => selection
                .prepared
                .as_ref()
                .map_or([f64::NAN; 2], |p| p.domain.map(|v| v.0)),
            Self::Compose { ref transforms } => {
                let Some(first) = transforms.first() else {
                    return [f64::NAN; 2];
                };
                let mut range = first.domain_batch(first.domain(), false);
                for transform in transforms.iter().skip(1) {
                    if range.iter().any(|v| v.is_nan()) {
                        return [f64::NAN; 2];
                    }
                    let domain = transform.domain();
                    let lower = domain[0].min(domain[1]).max(range[0].min(range[1]));
                    let upper = domain[0].max(domain[1]).min(range[0].max(range[1]));
                    if domain.iter().any(|v| v.is_nan()) || lower > upper {
                        return [f64::NAN; 2];
                    }
                    range = transform.domain_batch([lower, upper], false);
                }
                for transform in transforms.iter().rev() {
                    range = transform.domain_batch(range, true);
                }
                if range.iter().any(|v| v.is_nan()) {
                    [f64::NAN; 2]
                } else {
                    [range[0].min(range[1]), range[0].max(range[1])]
                }
            }
            Self::Asn | Self::Logistic { .. } | Self::Normal { .. } => [0., 1.],
            Self::Atanh => [-1., 1.],
            Self::Sqrt | Self::BoxCox { .. } => [0., f64::INFINITY],
            Self::Log { .. } => [1e-100, f64::INFINITY],
            Self::Log1p => [-1. + f64::EPSILON, f64::INFINITY],
            _ => [f64::NEG_INFINITY, f64::INFINITY],
        }
    }
    // Domain composition calls arithmetic on a pair without recursively validating
    // the composition whose domain is currently being established.
    fn domain_batch(&self, values: [f64; 2], inverse: bool) -> [f64; 2] {
        if let Self::Registered { selection } = self {
            let Ok(prepared) = selection.checked() else {
                return [f64::NAN; 2];
            };
            let result = if inverse {
                prepared.kernel.inverse_batch(&values)
            } else {
                prepared.kernel.forward_batch(&values)
            };
            return result
                .ok()
                .and_then(|v| v.try_into().ok())
                .unwrap_or([f64::NAN; 2]);
        }
        if let Self::Compose { transforms } = self {
            return if inverse {
                transforms
                    .iter()
                    .rev()
                    .fold(values, |v, t| t.domain_batch(v, true))
            } else {
                transforms
                    .iter()
                    .fold(values, |v, t| t.domain_batch(v, false))
            };
        }
        values.map(|v| {
            if inverse {
                self.inverse(v)
            } else {
                self.forward(v)
            }
        })
    }
    /// Whether every arithmetic stage is independent of population contents and size.
    pub fn is_pointwise(&self) -> bool {
        match self {
            Self::Registered { selection } => selection
                .prepared
                .as_ref()
                .is_some_and(|p| p.kernel.is_pointwise()),
            Self::Compose { transforms } => transforms.iter().all(Self::is_pointwise),
            _ => true,
        }
    }
    /// Logarithmic break policy inherited from the first transform in a composition.
    pub(crate) fn break_log_base(&self) -> Option<f64> {
        match self {
            Self::Log { base } => Some(*base),
            Self::Compose { transforms } => transforms.first().and_then(Self::break_log_base),
            _ => None,
        }
    }
    pub(crate) fn default_breaks(
        &self,
        limits: [f64; 2],
        count: f64,
        budget: usize,
    ) -> ChartResult<Option<Vec<f64>>> {
        let values = match self {
            Self::Registered { selection } => selection.checked()?.kernel.breaks(limits, count)?,
            Self::Compose { transforms } => {
                return transforms
                    .first()
                    .map_or(Ok(None), |t| t.default_breaks(limits, 5., budget));
            }
            _ => None,
        };
        if let Some(values) = &values {
            crate::limits::require_within(values.len() <= budget, "transform break count")?;
        }
        Ok(values.map(|v| v.into_iter().map(|n| n.0).collect()))
    }
    pub(crate) fn default_minor_breaks(
        &self,
        major: &[crate::interpolate::Number],
        limits: [crate::interpolate::Number; 2],
        budget: usize,
    ) -> ChartResult<Option<Vec<crate::interpolate::Number>>> {
        let Self::Registered { selection } = self else {
            return Ok(None);
        };
        crate::limits::require_within(major.len() <= budget, "transform minor input")?;
        let values = selection.checked()?.kernel.minor_breaks(major, limits, 2)?;
        if let Some(values) = &values {
            crate::limits::require_within(values.len() <= budget, "transform minor count")?;
        }
        Ok(values)
    }
    pub(crate) fn default_labels(
        &self,
        values: &[f64],
        budget: usize,
    ) -> ChartResult<Option<Vec<Option<String>>>> {
        let Self::Registered { selection } = self else {
            return Ok(None);
        };
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "transform label input",
        )?;
        let labels = selection.checked()?.kernel.labels(
            &values
                .iter()
                .copied()
                .map(crate::interpolate::Number)
                .collect::<Vec<_>>(),
        )?;
        if let Some(labels) = &labels {
            if labels.len() != values.len() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Breaks and labels have different lengths.",
                ));
            }
            let mut remaining = budget;
            for label in labels.iter().flatten() {
                crate::limits::require_within(label.len() <= remaining, "transform label bytes")?;
                remaining -= label.len();
            }
        }
        Ok(labels)
    }
    fn any_registration(
        &self,
        predicate: impl Fn(&crate::grammar::TransformSelection) -> bool,
    ) -> bool {
        let mut stack = vec![(self, 0usize)];
        let mut remaining = 4096usize;
        while let Some((transform, depth)) = stack.pop() {
            // Invalid oversized descriptors must enter the checked resolution path.
            if depth > crate::interpolate::MAX_VALUE_DEPTH || remaining == 0 {
                return true;
            }
            remaining -= 1;
            match transform {
                Self::Registered { selection } if predicate(selection) => return true,
                Self::Compose { transforms } => {
                    if transforms.len() > remaining {
                        return true;
                    }
                    stack.extend(transforms.iter().map(|t| (t, depth + 1)));
                }
                _ => {}
            }
        }
        false
    }
    pub(crate) fn needs_resolution(&self) -> bool {
        self.any_registration(|s| s.prepared.is_none())
    }
    pub(crate) fn has_registered(&self) -> bool {
        self.any_registration(|_| true)
    }
    pub(crate) fn validate_portable(&self) -> ChartResult<()> {
        self.validate()?;
        match self {
            Self::Registered { selection } if !selection.checked()?.portable() => Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only transforms cannot serialize.",
            )),
            Self::Compose { transforms } => {
                for transform in transforms {
                    transform.validate_portable()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn validate_registration_references(
        &self,
        registrations: &crate::grammar::transform_extensions::TransformRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        let mut stack = vec![(self, 0usize)];
        let mut remaining = 4096usize;
        while let Some((transform, depth)) = stack.pop() {
            crate::limits::require_within(
                depth <= crate::interpolate::MAX_VALUE_DEPTH && remaining > 0,
                "transform reference budget",
            )?;
            remaining -= 1;
            match transform {
                Self::Registered { selection } => {
                    registrations.validate(&selection.call, portable)?
                }
                Self::Compose { transforms } => {
                    crate::limits::require_within(
                        transforms.len() <= remaining,
                        "transform reference count",
                    )?;
                    stack.extend(transforms.iter().map(|t| (t, depth + 1)));
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(crate) fn resolve_registrations(
        &mut self,
        registrations: &crate::grammar::transform_extensions::TransformRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        self.resolve_bounded(registrations, portable, 0, &mut 4096)
    }
    fn resolve_bounded(
        &mut self,
        registrations: &crate::grammar::transform_extensions::TransformRegistrations,
        portable: bool,
        depth: usize,
        remaining: &mut usize,
    ) -> ChartResult<()> {
        if depth > crate::interpolate::MAX_VALUE_DEPTH || *remaining == 0 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Transform resolution exceeds its depth or operation budget.",
            ));
        }
        *remaining -= 1;
        match self {
            Self::Registered { selection } => {
                selection.prepared = Some(registrations.compile(&selection.call, portable)?);
            }
            Self::Compose { transforms } => {
                for transform in transforms {
                    transform.resolve_bounded(registrations, portable, depth + 1, remaining)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    /// Forward evaluation, retaining NaN and infinity as compatibility values.
    /// Call `validate` first: unresolved registered selections yield NaN.
    pub fn forward(&self, x: f64) -> f64 {
        match *self {
            Self::Registered { ref selection } => selection
                .prepared
                .as_ref()
                .map_or(f64::NAN, |p| p.kernel.forward(x)),
            Self::Compose { ref transforms } => transforms.iter().fold(x, |x, t| t.forward(x)),
            Self::Identity => x,
            Self::Reverse => -x,
            Self::Sqrt => libm::sqrt(x),
            Self::Asinh => libm::asinh(x),
            Self::Asn => 2. * libm::asin(libm::sqrt(x)),
            Self::Atanh => libm::atanh(x),
            Self::Log1p => pxfm::f_log1p(x),
            Self::Reciprocal => 1. / x,
            Self::Log { base } => logarithm(x, base),
            Self::Exp { base } => pxfm::f_pow(base, x),
            Self::BoxCox { p, offset } => shifted_power(x + offset, p),
            Self::Modulus { p, offset } => sign(x) * shifted_power(x.abs() + offset, p),
            Self::YeoJohnson { p } => {
                if x == 0. {
                    x
                } else if x > 0. {
                    if p.abs() < 1e-7 {
                        pxfm::f_log1p(x)
                    } else {
                        shifted_power(x + 1., p)
                    }
                } else if (2. - p).abs() < 1e-7 {
                    -pxfm::f_log1p(-x)
                } else {
                    -shifted_power(1. - x, 2. - p)
                }
            }
            Self::PseudoLog { sigma, base } => libm::asinh(x / (2. * sigma)) / libm::log(base),
            Self::Logistic { location, scale } => {
                if (0. ..=1.).contains(&x) {
                    location + scale * (libm::log(x) - pxfm::f_log1p(-x))
                } else {
                    f64::NAN
                }
            }
            Self::Normal { mean, sd } => mean + sd * normal_quantile(x),
        }
    }
    /// Inverse evaluation, retaining reference exceptional-value behavior.
    pub fn inverse(&self, x: f64) -> f64 {
        match *self {
            Self::Registered { ref selection } => selection
                .prepared
                .as_ref()
                .map_or(f64::NAN, |p| p.kernel.inverse(x)),
            Self::Compose { ref transforms } => {
                transforms.iter().rev().fold(x, |x, t| t.inverse(x))
            }
            Self::Identity => x,
            Self::Reverse => -x,
            Self::Sqrt => {
                if x < 0. {
                    f64::NAN
                } else {
                    x * x
                }
            }
            Self::Asinh => libm::sinh(x),
            Self::Asn => libm::sin(x / 2.).powi(2),
            Self::Atanh => libm::tanh(x),
            Self::Log1p => pxfm::f_expm1(x),
            Self::Reciprocal => 1. / x,
            Self::Log { base } => libm::pow(base, x),
            Self::Exp { base } => logarithm(x, base),
            Self::BoxCox { p, offset } => inverse_power(x, p) - offset,
            Self::Modulus { p, offset } => sign(x) * (inverse_power(x.abs(), p) - offset),
            Self::YeoJohnson { p } => {
                if x == 0. {
                    x
                } else if x > 0. {
                    if p.abs() < 1e-7 {
                        pxfm::f_expm1(x)
                    } else {
                        inverse_power(x, p) - 1.
                    }
                } else if (2. - p).abs() < 1e-7 {
                    1. - libm::exp(-x)
                } else {
                    1. - inverse_power(-x, 2. - p)
                }
            }
            Self::PseudoLog { sigma, base } => 2. * sigma * libm::sinh(x * libm::log(base)),
            Self::Logistic { location, scale } => 1. / (1. + libm::exp(-(x - location) / scale)),
            Self::Normal { mean, sd } => {
                0.5 * libm::erfc(-(x - mean) / sd / std::f64::consts::SQRT_2)
            }
        }
    }
    /// Whether a finite source interval has a continuous, invertible branch.
    pub(crate) fn monotone_on(&self, values: &[f64]) -> bool {
        if !self.is_pointwise() || self.validate().is_err() || values.len() < 2 {
            return false;
        }
        if let Self::Compose { transforms } = self {
            let mut stage = values.to_vec();
            for transform in transforms {
                if !transform.monotone_on(&stage) {
                    return false;
                }
                stage = stage.into_iter().map(|v| transform.forward(v)).collect();
            }
            return true;
        }
        let low = values.iter().copied().fold(f64::INFINITY, f64::min);
        let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let branch = match *self {
            Self::Registered { ref selection } => selection
                .prepared
                .as_ref()
                .is_some_and(|p| p.kernel.monotone_on([low, high])),
            Self::Reciprocal => low > 0. || high < 0.,
            Self::Modulus { offset, .. } => offset == 1. || low > 0. || high < 0.,
            Self::BoxCox { offset, .. } => low + offset >= 0.,
            Self::Asn => low >= 0. && high <= 1.,
            _ => true,
        };
        branch
            && values
                .iter()
                .all(|x| x.is_finite() && self.forward(*x).is_finite())
            && self.forward(low) != self.forward(high)
    }
    /// Whether a stage can reject the whole population rather than return NaN.
    pub(crate) fn requires_population_validation(&self) -> bool {
        match self {
            Self::BoxCox { .. } | Self::Registered { .. } => true,
            Self::Compose { transforms } => {
                transforms.iter().any(Self::requires_population_validation)
            }
            _ => false,
        }
    }
    /// Validate the complete input population before missing-value filtering.
    pub(crate) fn validate_population(
        &self,
        values: impl IntoIterator<Item = f64>,
    ) -> ChartResult<()> {
        self.validate()?;
        if let Self::Compose { .. } = self {
            self.forward_population(&values.into_iter().collect::<Vec<_>>())?;
            return Ok(());
        }
        if let Self::Registered { selection } = self {
            let values = values.into_iter().collect::<Vec<_>>();
            crate::limits::require_within(
                values.len() <= crate::interpolate::MAX_VALUES,
                "transform population",
            )?;
            return selection.checked()?.kernel.validate_population(&values);
        }
        if let Self::BoxCox { offset, .. } = self
            && values.into_iter().any(|x| x + offset < 0.)
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Box-Cox requires nonnegative shifted inputs.",
            ));
        }
        Ok(())
    }
    /// Apply a population, including the reference Box–Cox negative-input rejection.
    pub fn forward_population(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "transform population",
        )?;
        if let Self::Compose { transforms } = self {
            self.validate()?;
            let mut result = values.to_vec();
            for transform in transforms {
                result = transform.forward_population(&result)?;
            }
            return Ok(result);
        }
        self.validate_population(values.iter().copied())?;
        if let Self::Registered { selection } = self {
            return checked_batch(values, selection.checked()?.kernel.forward_batch(values)?);
        }
        Ok(values.iter().map(|x| self.forward(*x)).collect())
    }
    /// Transform a NULL callback result until a transform produces a typed vector.
    pub(crate) fn forward_null(&self) -> ChartResult<Option<Vec<f64>>> {
        self.validate()?;
        match self {
            Self::Identity => Ok(None),
            Self::Reverse => Err(error(
                DiagnosticCode::NumericalDomain,
                "Reverse cannot transform a NULL limit result.",
            )),
            Self::Compose { transforms } => {
                let mut result = None;
                for transform in transforms {
                    result = match result {
                        None => transform.forward_null()?,
                        Some(values) => Some(transform.forward_population(&values)?),
                    };
                }
                Ok(result)
            }
            Self::Registered { selection } => selection
                .checked()?
                .kernel
                .forward_null()?
                .map(|values| checked_batch(&[], values))
                .transpose(),
            Self::Exp { .. } | Self::BoxCox { .. } | Self::PseudoLog { .. } | Self::Reciprocal => {
                self.forward_population(&[]).map(Some)
            }
            _ => Err(error(
                DiagnosticCode::NumericalDomain,
                "This numeric transformation cannot transform a NULL callback result.",
            )),
        }
    }
    /// Invert an untrained NULL guide domain, preserving its distinct demand
    /// behavior until a transform converts it to a typed numeric vector.
    pub(crate) fn inverse_null(&self) -> ChartResult<Option<Vec<f64>>> {
        self.validate()?;
        match self {
            Self::Identity => Ok(None),
            Self::Reverse => Err(error(
                DiagnosticCode::NumericalDomain,
                "Reverse cannot inverse-transform an untrained NULL domain.",
            )),
            Self::Compose { transforms } => {
                let mut result = None;
                for transform in transforms.iter().rev() {
                    result = match result {
                        None => transform.inverse_null()?,
                        Some(values) => Some(transform.inverse_population(&values)?),
                    };
                }
                Ok(result)
            }
            Self::Registered { selection } => selection
                .checked()?
                .kernel
                .inverse_null()?
                .map(|values| checked_batch(&[], values))
                .transpose(),
            Self::Asn
            | Self::Log { .. }
            | Self::Sqrt
            | Self::PseudoLog { .. }
            | Self::Reciprocal => self.inverse_population(&[]).map(Some),
            Self::BoxCox { p, .. } if p.abs() >= 1e-7 => self.inverse_population(&[]).map(Some),
            Self::YeoJohnson { p } if p.abs() >= 1e-7 && (2. - p).abs() >= 1e-7 => {
                self.inverse_population(&[]).map(Some)
            }
            _ => Err(error(
                DiagnosticCode::NumericalDomain,
                "This numeric transformation cannot inverse-transform an untrained NULL domain.",
            )),
        }
    }
    /// Invert a complete population in reverse composition order.
    /// Population-dependent inverses apply only to the supplied batch.
    pub fn inverse_population(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "transform population",
        )?;
        if let Self::Compose { transforms } = self {
            let mut result = values.to_vec();
            for transform in transforms.iter().rev() {
                result = transform.inverse_population(&result)?;
            }
            return Ok(result);
        }
        if let Self::Registered { selection } = self {
            return checked_batch(values, selection.checked()?.kernel.inverse_batch(values)?);
        }
        Ok(values.iter().map(|x| self.inverse(*x)).collect())
    }
}
fn checked_batch(input: &[f64], output: Vec<f64>) -> ChartResult<Vec<f64>> {
    crate::limits::require_within(
        output.len() <= crate::interpolate::MAX_VALUES,
        "transform population",
    )?;
    if output.len() != input.len() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Transform output must preserve the population length.",
        ));
    }
    Ok(output)
}
fn logarithm(x: f64, base: f64) -> f64 {
    if base == 10. {
        libm::log10(x)
    } else if base == 2. {
        libm::log2(x)
    } else {
        libm::log(x) / libm::log(base)
    }
}
fn shifted_power(x: f64, p: f64) -> f64 {
    if p.abs() < 1e-7 {
        pxfm::f_log(x)
    } else {
        (libm::pow(x, p) - 1.) / p
    }
}
fn inverse_power(x: f64, p: f64) -> f64 {
    if p.abs() < 1e-7 {
        pxfm::f_exp(x)
    } else {
        libm::pow(x * p + 1., 1. / p)
    }
}
fn sign(x: f64) -> f64 {
    if x == 0. { x } else { x.signum() }
}
// Invert the independent libm normal CDF using the smaller tail throughout.
// Bounded bisection covers all finite f64 probabilities without R runtime code.
fn normal_quantile(p: f64) -> f64 {
    if p == 0. {
        return f64::NEG_INFINITY;
    }
    if p == 1. {
        return f64::INFINITY;
    }
    if !(0. ..1.).contains(&p) {
        return f64::NAN;
    }
    if p == 0.5 {
        return 0.;
    }
    let tail = if p < 0.5 { p } else { 1. - p };
    let (mut low, mut high) = (0., 40.);
    for _ in 0..64 {
        let mid = (low + high) / 2.;
        if mid == low || mid == high {
            break;
        }
        if 0.5 * libm::erfc(mid / std::f64::consts::SQRT_2) > tail {
            low = mid;
        } else {
            high = mid;
        }
    }
    let q = (low + high) / 2.;
    if p < 0.5 { -q } else { q }
}

#[cfg(test)]
mod null_contract_tests {
    use super::GgplotTransform as T;

    #[test]
    fn builtin_null_and_typed_empty_protocols_match_29_source_configurations() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/transform-null-contracts.json"
        ))
        .unwrap();
        let transforms = [
            T::Asinh,
            T::Asn,
            T::Atanh,
            T::BoxCox { p: 0., offset: 0. },
            T::BoxCox { p: 0.5, offset: 0. },
            T::BoxCox { p: -1., offset: 2. },
            T::Exp {
                base: std::f64::consts::E,
            },
            T::Exp { base: 2. },
            T::Identity,
            T::Log {
                base: std::f64::consts::E,
            },
            T::Log { base: 0.5 },
            T::Log { base: 10. },
            T::Log { base: 2. },
            T::Log1p,
            T::Logistic {
                location: 0.,
                scale: 1.,
            },
            T::Normal { mean: 0., sd: 1. },
            T::Modulus { p: 0., offset: 1. },
            T::Modulus { p: 0.5, offset: 1. },
            T::Modulus { p: 2., offset: 2. },
            T::PseudoLog {
                sigma: 1.,
                base: std::f64::consts::E,
            },
            T::PseudoLog {
                sigma: 0.5,
                base: 10.,
            },
            T::Reciprocal,
            T::Reverse,
            T::Sqrt,
            T::YeoJohnson { p: 0. },
            T::YeoJohnson { p: 1. },
            T::YeoJohnson { p: 2. },
            T::Normal { mean: 2., sd: 3. },
            T::Logistic {
                location: 2.,
                scale: 3.,
            },
        ];
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), transforms.len());
        for (transform, case) in transforms.iter().zip(cases) {
            for (name, actual) in [
                ("forward_null", transform.forward_null()),
                ("inverse_null", transform.inverse_null()),
                ("forward_empty", transform.forward_population(&[]).map(Some)),
                ("inverse_empty", transform.inverse_population(&[]).map(Some)),
            ] {
                let expected = &case[name];
                assert_eq!(
                    actual.is_err(),
                    expected.get("error").is_some(),
                    "{transform:?}: {name}: {actual:?}"
                );
                if let Ok(actual) = actual {
                    assert_eq!(
                        actual.is_none(),
                        expected["is_null"].as_bool().unwrap(),
                        "{transform:?}: {name}"
                    );
                    assert!(actual.as_ref().is_none_or(Vec::is_empty));
                    assert!(expected["values"].as_array().unwrap().is_empty());
                }
            }
        }
    }
}
