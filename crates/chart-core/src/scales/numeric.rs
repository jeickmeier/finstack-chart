//! Prepared numerical scale families and shared piecewise normalization.
//! D3 algorithms are adapted from d3-scale 4.0.2; see `LICENSE` and ADR-018.
use super::{Bounds, error};
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{MAX_VALUES, Number, ScalarInterpolator, Value},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Preparation and inverse semantics, explicit at the portable boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleCompatibility {
    /// Established finite two-endpoint recipe projection with unbounded inverse.
    Legacy,
    /// D3-compatible knots, defaults, rounding and inverse clamping.
    D3,
}
/// Numeric domain or radius transformation. Sqrt is power with exponent 0.5.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NumericFamily {
    /// Affine or piecewise numeric interpolation.
    Linear,
    /// Sign-preserving power; finite exponents include zero and negative values.
    Pow {
        /// Authored exponent.
        exponent: f64,
    },
    /// Sign-consistent logarithmic branch; base configures ticks rather than mapping.
    Log {
        /// Finite authored logarithm base.
        base: f64,
    },
    /// Sign-preserving log1p transformation.
    Symlog {
        /// Finite transition constant; singular operations diagnose individually.
        constant: f64,
    },
    /// Identity mapping; domain and range are the same metadata.
    Identity,
    /// Interpolate signed squared radii and take the signed square root afterwards.
    Radial,
}
/// Immutable portable numeric scale configuration. No destination or host objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericScaleSpec {
    /// Explicit compatibility policy.
    pub compatibility: ScaleCompatibility,
    /// Numeric family and parameters.
    pub family: NumericFamily,
    /// Authored domain knots in monotone order; repeated and empty knots are explicit.
    pub domain: Vec<Number>,
    /// Numeric output knots; retained even when longer than the effective domain.
    pub range: Vec<Number>,
    /// Clamp forward input and, in D3 mode, inverse output to effective domain ends.
    pub clamp: bool,
    /// Round mapped numbers (after square root for radial scales).
    pub round: bool,
    /// Owned result for a missing/NaN input. Geometry consumers still require finite numbers.
    pub unknown: Value,
}
impl NumericScaleSpec {
    /// Reference constructor defaults for the selected family.
    pub fn d3(family: NumericFamily) -> Self {
        Self {
            compatibility: ScaleCompatibility::D3,
            family,
            domain: if matches!(family, NumericFamily::Log { .. }) {
                vec![1.0.into(), 10.0.into()]
            } else {
                vec![0.0.into(), 1.0.into()]
            },
            range: vec![0.0.into(), 1.0.into()],
            clamp: false,
            round: false,
            unknown: Value::Missing,
        }
    }
}
/// Shared segment selection and normalization for numeric and typed continuous ranges.
#[derive(Clone, Debug)]
pub(super) struct KnotMapping {
    domain: Vec<f64>,
    count: usize,
}
impl KnotMapping {
    pub(super) fn new(mut domain: Vec<f64>, outputs: usize) -> (Self, bool) {
        let n = domain.len().min(outputs);
        let count = if n > 2 { n - 1 } else { 1 };
        let reversed = if n > 2 {
            if domain[n - 1] < domain[0] {
                domain.reverse();
                true
            } else {
                false
            }
        } else if domain.get(1).is_some_and(|b| *b < domain[0]) {
            domain.swap(0, 1);
            true
        } else {
            false
        };
        (Self { domain, count }, reversed)
    }
    pub(super) fn count(&self) -> usize {
        self.count
    }
    pub(super) fn parameter(&self, x: f64) -> (usize, f64) {
        let i = if self.count == 1 {
            0
        } else {
            self.domain[1..self.count].partition_point(|d| *d <= x)
        };
        let a = self.domain.get(i).copied().unwrap_or(f64::NAN);
        let b = self.domain.get(i + 1).copied().unwrap_or(f64::NAN);
        let width = b - a;
        (i, if width == 0. { 0.5 } else { (x - a) / width })
    }
}
#[derive(Clone, Debug)]
struct Piecewise {
    knots: KnotMapping,
    segments: Vec<ScalarInterpolator>,
}
impl Piecewise {
    fn new(domain: Vec<f64>, mut range: Vec<f64>, round: bool) -> Self {
        let n = domain.len().min(range.len());
        let (knots, reversed) = KnotMapping::new(domain, range.len());
        if reversed {
            if n > 2 {
                range.reverse();
            } else if range.len() > 1 {
                range.swap(0, 1);
            }
        }
        let segments = (0..knots.count())
            .map(|i| {
                let a = range.get(i).copied().unwrap_or(f64::NAN);
                let b = range.get(i + 1).copied().unwrap_or(f64::NAN);
                if round {
                    ScalarInterpolator::round(a, b)
                } else {
                    ScalarInterpolator::number(a, b)
                }
            })
            .collect();
        Self { knots, segments }
    }
    fn sample(&self, x: f64) -> ChartResult<f64> {
        let (i, t) = self.knots.parameter(x);
        if t.is_nan() {
            return Ok(f64::NAN);
        }
        self.segments[i].sample(t)
    }
}
/// Prepared numeric mapping. Copies retain immutable compiled segments and owned descriptors.
#[derive(Clone, Debug)]
pub struct NumericScale {
    spec: Arc<NumericScaleSpec>,
    forward: Arc<Piecewise>,
    inverse: Arc<Piecewise>,
    effective_domain: Option<(f64, f64)>,
    negative_log: bool,
}
impl PartialEq for NumericScale {
    fn eq(&self, rhs: &Self) -> bool {
        self.spec == rhs.spec
    }
}
impl NumericScale {
    /// Validate and compile once; no per-sample allocation for ordinary numeric values.
    pub fn new(mut spec: NumericScaleSpec) -> ChartResult<Self> {
        if spec.domain.len().saturating_add(spec.range.len()) > MAX_VALUES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Scale knots exceed the aggregate budget.",
            ));
        }
        spec.unknown.validate()?;
        let parameter = match spec.family {
            NumericFamily::Pow { exponent } => exponent,
            NumericFamily::Log { base } => base,
            NumericFamily::Symlog { constant } => constant,
            _ => 1.,
        };
        if !parameter.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Scale parameters must be finite.",
            ));
        }
        let domain: Vec<_> = spec.domain.iter().map(|x| x.0).collect();
        if domain.iter().any(|v| !v.is_finite()) || spec.range.iter().any(|v| !v.0.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Authored numeric scale knots must be finite.",
            ));
        }
        let ascending = domain.windows(2).all(|p| p[0] <= p[1]);
        let descending = domain.windows(2).all(|p| p[0] >= p[1]);
        if !ascending && !descending {
            return Err(error(
                DiagnosticCode::Validation,
                "Scale domain knots must be monotone; repeated knots are permitted.",
            ));
        }
        if matches!(spec.family, NumericFamily::Identity) {
            spec.range = spec.domain.clone();
        }
        if spec.compatibility == ScaleCompatibility::Legacy
            && (domain.len() != 2
                || spec.range.len() != 2
                || domain[0] == domain[1]
                || spec.range[0] == spec.range[1]
                || spec.family != NumericFamily::Linear
                || spec.round)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Legacy numerical kernel requires distinct finite linear endpoints.",
            ));
        }
        let negative_log = domain.first().is_some_and(|x| *x < 0.);
        let transformed: Vec<_> = domain
            .iter()
            .map(|x| transform(spec.family, negative_log, *x, false))
            .collect();
        let range: Vec<_> = spec
            .range
            .iter()
            .map(|x| {
                if matches!(spec.family, NumericFamily::Radial) {
                    square(x.0)
                } else {
                    x.0
                }
            })
            .collect();
        let forward = Piecewise::new(
            transformed.clone(),
            range.clone(),
            spec.round && !matches!(spec.family, NumericFamily::Radial),
        );
        let inverse = Piecewise::new(range, transformed, false);
        let n = domain.len().min(spec.range.len());
        let effective_domain = if n > 0 {
            Some((domain[0].min(domain[n - 1]), domain[0].max(domain[n - 1])))
        } else {
            None
        };
        Ok(Self {
            spec: Arc::new(spec),
            forward: Arc::new(forward),
            inverse: Arc::new(inverse),
            effective_domain,
            negative_log,
        })
    }
    /// Reference linear constructor.
    pub fn linear() -> Self {
        Self::new(NumericScaleSpec::d3(NumericFamily::Linear)).expect("valid defaults")
    }
    /// Reference square-root constructor, sharing the power family.
    pub fn sqrt() -> Self {
        Self::new(NumericScaleSpec::d3(NumericFamily::Pow { exponent: 0.5 }))
            .expect("valid defaults")
    }
    /// Exact authored configuration, independent of compiled resources.
    pub fn spec(&self) -> &NumericScaleSpec {
        &self.spec
    }
    /// Raw unthinned candidates in authored data space, with an independent hard budget.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
        self.spec.family.ticks(&self.spec.domain, count, budget)
    }
    /// Independently prepared labels, including default precision and log suppression.
    pub fn tick_format(
        &self,
        count: f64,
        specifier: Option<&str>,
        locale: crate::typography::NumericLocale,
    ) -> ChartResult<crate::typography::NumericFormatter> {
        self.spec
            .family
            .tick_format(&self.spec.domain, count, specifier, locale)
    }
    /// Replace configuration atomically; the old scale remains usable on failure.
    pub fn reconfigure(&self, spec: NumericScaleSpec) -> ChartResult<Self> {
        Self::new(spec)
    }
    /// Independent domain edit; identity updates its range metadata as well.
    pub fn with_domain(&self, domain: impl IntoIterator<Item = f64>) -> ChartResult<Self> {
        let mut spec = (*self.spec).clone();
        spec.domain = domain.into_iter().map(Number).collect();
        Self::new(spec)
    }
    /// Independent range edit; identity updates its domain metadata as well.
    pub fn with_range(&self, range: impl IntoIterator<Item = f64>) -> ChartResult<Self> {
        let mut spec = (*self.spec).clone();
        spec.range = range.into_iter().map(Number).collect();
        if matches!(spec.family, NumericFamily::Identity) {
            spec.domain = spec.range.clone();
        }
        Self::new(spec)
    }
    /// Explicitly nice only the first/last domain knots; interior knots and the source remain unchanged.
    /// D3 count is a hint (including zero/fractional/IEEE values); legacy keeps its grid contract.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        if !count.is_finite() && self.spec.compatibility == ScaleCompatibility::Legacy {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Nice count must be finite.",
            ));
        }
        let mut spec = (*self.spec).clone();
        let n = spec.domain.len();
        if n < 2 {
            if let NumericFamily::Log { base } = spec.family {
                if n == 0 {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Log nice has no defined endpoint for an empty domain.",
                    ));
                }
                let (_, upper) = super::ticks::nice_log(spec.domain[0].0, spec.domain[0].0, base);
                spec.domain[0] = Number(upper);
                return Self::new(spec);
            }
            return Ok(self.clone());
        }
        let (a, b) = (spec.domain[0].0, spec.domain[n - 1].0);
        let (start, stop) = if spec.compatibility == ScaleCompatibility::Legacy {
            if count.fract() != 0. || !(2. ..=128.).contains(&count) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Legacy nice count must be an integer from 2 through 128.",
                ));
            }
            let options = super::ContinuousDomain {
                nice: true,
                nice_ticks: count as usize,
                ..Default::default()
            };
            let bounds = options.resolve(Some(crate::grammar::Extent {
                minimum: a.min(b),
                maximum: a.max(b),
            }))?;
            if b < a {
                (bounds.end(), bounds.start())
            } else {
                (bounds.start(), bounds.end())
            }
        } else if let NumericFamily::Log { base } = spec.family {
            super::ticks::nice_log(a, b, base)
        } else {
            super::ticks::nice(a, b, count)
        };
        spec.domain[0] = Number(start);
        spec.domain[n - 1] = Number(stop);
        Self::new(spec)
    }
    /// Evaluate a typed missing or numeric input; unknown outputs retain their original kind.
    pub fn map(&self, input: Option<f64>) -> ChartResult<Value> {
        let Some(mut x) = input.filter(|x| !x.is_nan()) else {
            return Ok(self.spec.unknown.clone());
        };
        if matches!(self.spec.family, NumericFamily::Identity) {
            return Ok(Value::number(x));
        }
        if self.spec.clamp {
            x = self.clamp(x);
        }
        if self.spec.compatibility == ScaleCompatibility::Legacy {
            let d = Bounds::new(self.spec.domain[0].0, self.spec.domain[1].0)?;
            let r = Bounds::new(self.spec.range[0].0, self.spec.range[1].0)?;
            return Ok(Value::number(legacy_map(d, r, x)?));
        }
        if self.spec.range.len() < 2
            && !self.spec.round
            && !matches!(self.spec.family, NumericFamily::Radial)
        {
            return Ok(Value::Missing);
        }
        let mut value =
            self.forward
                .sample(transform(self.spec.family, self.negative_log, x, false))?;
        if matches!(self.spec.family, NumericFamily::Radial) {
            value = unsquare(value);
            if value.is_nan() {
                return Ok(self.spec.unknown.clone());
            }
            if self.spec.round {
                value = crate::interpolate::js_round(value);
            }
        }
        Ok(Value::number(value))
    }
    /// Checked finite geometry boundary for numerical consumers.
    pub fn map_finite(&self, input: f64) -> ChartResult<Option<f64>> {
        match self.map(Some(input))? {
            Value::Missing => Ok(None),
            Value::Number(Number(v)) if v.is_finite() => Ok(Some(v)),
            _ => Err(error(
                DiagnosticCode::NumericalDomain,
                "Scale output is not a finite geometry coordinate.",
            )),
        }
    }
    /// Typed inverse output, retaining identity-scale unknown values for missing inputs.
    pub fn invert_output(&self, input: Option<f64>) -> ChartResult<Value> {
        if matches!(self.spec.family, NumericFamily::Identity) {
            return self.map(input);
        }
        match input {
            None => Ok(Value::Missing),
            Some(value) => self.invert(value).map(Value::number),
        }
    }
    /// Numeric inverse, with D3 clamp semantics. Rounding is deliberately not inverted.
    pub fn invert(&self, value: f64) -> ChartResult<f64> {
        self.inverse_value(
            value,
            self.spec.compatibility == ScaleCompatibility::D3 && self.spec.clamp,
        )
    }
    /// Explicit unbounded inverse for viewport navigation; never changes authored clamp policy.
    pub fn invert_unbounded(&self, value: f64) -> ChartResult<f64> {
        self.inverse_value(value, false)
    }
    fn inverse_value(&self, value: f64, clamp: bool) -> ChartResult<f64> {
        if matches!(self.spec.family, NumericFamily::Identity) {
            return Ok(value);
        }
        if self.spec.compatibility == ScaleCompatibility::Legacy {
            let d = Bounds::new(self.spec.domain[0].0, self.spec.domain[1].0)?;
            let r = Bounds::new(self.spec.range[0].0, self.spec.range[1].0)?;
            return legacy_map(r, d, value);
        }
        let value = if matches!(self.spec.family, NumericFamily::Radial) {
            square(value)
        } else {
            value
        };
        let value = transform(
            self.spec.family,
            self.negative_log,
            self.inverse.sample(value)?,
            true,
        );
        Ok(if clamp { self.clamp(value) } else { value })
    }
    fn clamp(&self, x: f64) -> f64 {
        if x.is_nan() {
            x
        } else {
            self.effective_domain
                .map_or(f64::NAN, |(a, b)| x.clamp(a, b))
        }
    }
    /// Strict standalone version-one descriptor.
    pub fn to_json(&self) -> ChartResult<String> {
        serde_json::to_string(&WireRef {
            version: 1,
            spec: &self.spec,
        })
        .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
    }
    /// Decode and prepare a bounded descriptor before exposing any usable scale.
    pub fn from_json(text: &str) -> ChartResult<Self> {
        if text.len() > crate::interpolate::MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Scale descriptor exceeds its byte budget.",
            ));
        }
        let wire: Wire = serde_json::from_str(text)
            .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?;
        if wire.version != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported numeric scale descriptor version.",
            ));
        }
        Self::new(wire.spec)
    }
}
#[derive(Serialize)]
struct WireRef<'a> {
    version: u32,
    spec: &'a NumericScaleSpec,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u32,
    spec: NumericScaleSpec,
}
fn square(x: f64) -> f64 {
    if x == 0. { x } else { x.signum() * x * x }
}
fn unsquare(x: f64) -> f64 {
    if x == 0. {
        x
    } else {
        x.signum() * x.abs().sqrt()
    }
}
fn power(x: f64, p: f64) -> f64 {
    if p == 1. {
        x
    } else if p == 0.5 {
        if x < 0. { -(-x).sqrt() } else { x.sqrt() }
    } else if x < 0. {
        -crate::number::ecma_pow(-x, p)
    } else {
        crate::number::ecma_pow(x, p)
    }
}
pub(super) fn transform(family: NumericFamily, negative: bool, x: f64, inverse: bool) -> f64 {
    match family {
        NumericFamily::Pow { exponent } => {
            if inverse && exponent == 0.5 {
                if x < 0. { -x * x } else { x * x }
            } else {
                power(x, if inverse { 1. / exponent } else { exponent })
            }
        }
        NumericFamily::Log { .. } => {
            if inverse {
                if negative {
                    -libm::exp(-x)
                } else {
                    libm::exp(x)
                }
            } else if negative {
                -libm::log(-x)
            } else {
                libm::log(x)
            }
        }
        NumericFamily::Symlog { constant } => {
            let sign = if x == 0. { x } else { x.signum() };
            sign * if inverse {
                libm::expm1(x.abs()) * constant
            } else {
                libm::log1p((x / constant).abs())
            }
        }
        _ => x,
    }
}

/// Destination projection of a prepared numeric scale, retaining all source knots.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericAxisScale {
    mapping: NumericScale,
    domain: Bounds,
    view: Bounds,
    range: Bounds,
    mapped_view: Bounds,
    outside: super::OutsidePolicy,
    clamp: bool,
}
impl NumericAxisScale {
    /// Resolve authored numerical outputs into destination units without dropping knots.
    pub fn resolve(
        mut spec: NumericScaleSpec,
        range: Bounds,
        viewport: Option<Bounds>,
        outside: super::OutsidePolicy,
    ) -> ChartResult<Self> {
        let clamp = spec.clamp;
        spec.clamp = false;
        let mapping = NumericScale::new(spec)?;
        let domain = Bounds::new(
            mapping
                .spec()
                .domain
                .first()
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::NumericalDomain,
                        "A chart axis needs an authored domain.",
                    )
                })?
                .0,
            mapping.spec().domain.last().expect("nonempty").0,
        )?;
        let view = viewport.unwrap_or(domain);
        let range = range.distinct()?;
        let mapped_view = if viewport.is_some() {
            Bounds::new(
                required(mapping.map_finite(view.start())?)?,
                required(mapping.map_finite(view.end())?)?,
            )?
        } else {
            let count = mapping.spec().domain.len().min(mapping.spec().range.len());
            if count < 2 {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "A chart axis needs two defined numeric range endpoints.",
                ));
            }
            Bounds::new(mapping.spec().range[0].0, mapping.spec().range[count - 1].0)?
        };
        Ok(Self {
            mapping,
            domain,
            view,
            range,
            mapped_view,
            outside,
            clamp,
        })
    }
    /// Complete authored data domain, independent of viewport and destination.
    pub fn domain(&self) -> Bounds {
        self.domain
    }
    /// Family used for data-space tick generation and formatting.
    pub fn family(&self) -> NumericFamily {
        self.mapping.spec.family
    }
    /// Visible data endpoints; original interior knots remain in the mapping.
    pub fn viewport(&self) -> Bounds {
        self.view
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Unclamped scale output used by navigation before destination projection.
    pub fn coordinate(&self, value: f64) -> ChartResult<f64> {
        required(self.mapping.map_finite(value)?)
    }
    /// Reverse the same shared numerical mapping for navigation.
    pub fn coordinate_inverse(&self, value: f64) -> ChartResult<f64> {
        finite(self.mapping.invert_unbounded(value)?)
    }
    /// Full domain in the scale's numerical output space.
    pub fn coordinate_domain(&self) -> ChartResult<Bounds> {
        Bounds::new(
            self.coordinate(self.domain.start())?,
            self.coordinate(self.domain.end())?,
        )
    }
    /// Visible interval in the scale's numerical output space.
    pub fn coordinate_viewport(&self) -> Bounds {
        self.mapped_view
    }
    /// Project a finite data coordinate using the retained mapping and viewport.
    pub fn map(&self, mut value: f64) -> ChartResult<Option<f64>> {
        finite(value)?;
        if self.outside == super::OutsidePolicy::Omit && !self.view.contains(value) {
            return Ok(None);
        }
        if self.outside == super::OutsidePolicy::Clamp {
            value = value.clamp(self.view.minimum(), self.view.maximum());
        }
        if self.clamp {
            value = self.mapping.clamp(value);
        }
        let mapped = self.coordinate(value)?;
        let t = if self.mapped_view.start() == self.mapped_view.end() {
            0.5
        } else {
            super::linear::fraction(self.mapped_view, mapped)?
        };
        Ok(Some(super::linear::interpolate(self.range, t)?))
    }
    /// Invert destination coordinates; explicit D3/outside clamping applies symmetrically.
    pub(crate) fn inverse_coordinate(&self, mut position: f64) -> ChartResult<f64> {
        if self.outside == super::OutsidePolicy::Clamp {
            position = position.clamp(self.range.minimum(), self.range.maximum());
        }
        super::linear::interpolate(
            self.mapped_view,
            super::linear::fraction(self.range, position)?,
        )
    }
    /// Invert destination coordinates with the declared outside and domain policy.
    pub fn invert(&self, position: f64) -> ChartResult<f64> {
        let mapped = self.inverse_coordinate(position)?;
        let value = self.coordinate_inverse(mapped)?;
        Ok(if self.clamp {
            self.mapping.clamp(value)
        } else {
            value
        })
    }
    /// Destination guide candidates; D3 raw candidates remain independently accessible on `NumericScale`.
    pub fn ticks(&self, target: usize, max_ticks: usize) -> ChartResult<Vec<super::NumericTick>> {
        if self.mapping.spec.compatibility == ScaleCompatibility::D3 {
            let domain = [Number(self.view.start()), Number(self.view.end())];
            let family = self.mapping.spec.family;
            let format = family.tick_format(&domain, target as f64, None, Default::default())?;
            return family
                .ticks(&domain, target as f64, max_ticks)
                .map(|ticks| {
                    ticks
                        .into_iter()
                        .map(|value| super::NumericTick {
                            value,
                            label: format.format(value),
                        })
                        .collect()
                });
        }
        if self.view.start() == self.view.end() {
            return Ok(vec![super::NumericTick {
                value: self.view.start(),
                label: crate::number::ecmascript(self.view.start()),
            }]);
        }
        super::linear::numeric_ticks(self.view, target, max_ticks)
    }
}
fn required(value: Option<f64>) -> ChartResult<f64> {
    value.ok_or_else(|| {
        error(
            DiagnosticCode::NumericalDomain,
            "A positional scale cannot project a missing numeric result.",
        )
    })
}
fn finite(value: f64) -> ChartResult<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(error(
            DiagnosticCode::NumericalDomain,
            "A positional scale requires a finite numerical result.",
        ))
    }
}

// The established geometry route and explicit Legacy descriptors share the same checked kernel.
pub(super) fn legacy_map(domain: Bounds, range: Bounds, value: f64) -> ChartResult<f64> {
    super::linear::interpolate(range, super::linear::fraction(domain, value)?)
}
