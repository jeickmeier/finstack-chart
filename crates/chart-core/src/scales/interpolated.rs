//! Shared sequential/diverging/rank normalization, independent of output interpolation.
use super::{
    NumericFamily,
    classifier::{bisect_right, quantile_sorted, sorted_samples},
    error,
    numeric::transform,
};
use crate::{
    ChartResult, DiagnosticCode,
    color::ColorValue,
    interpolate::{InterpolationSpec, Interpolator, MAX_VALUES, Number, Sample, Value},
};
use serde::{Deserialize, Serialize};

/// Domain-to-parameter configuration. These families have no general numeric inverse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NormalizationSpec {
    /// Transform two endpoints, then normalize to zero and one.
    Sequential {
        /// Shared linear/log/pow/symlog family.
        family: NumericFamily,
        /// Authored endpoints; constants and descending order are supported.
        domain: [Number; 2],
        /// Saturate the interpolator parameter.
        clamp: bool,
    },
    /// Transform three endpoints, retaining the independently authored midpoint.
    Diverging {
        /// Shared linear/log/pow/symlog family.
        family: NumericFamily,
        /// Lower, central and upper domain entries in authored orientation.
        domain: [Number; 3],
        /// Saturate the interpolator parameter.
        clamp: bool,
    },
    /// Empirical rank over the complete eligible sample population.
    Quantile {
        /// Missing/NaN observations are excluded and remaining samples sorted once.
        samples: Vec<Option<Number>>,
    },
}
impl NormalizationSpec {
    /// D3-compatible sequential defaults, including the logarithmic domain.
    pub fn sequential(family: NumericFamily) -> Self {
        Self::Sequential {
            family,
            domain: if matches!(family, NumericFamily::Log { .. }) {
                [Number(1.), Number(10.)]
            } else {
                [Number(0.), Number(1.)]
            },
            clamp: false,
        }
    }
    /// D3-compatible diverging defaults with an explicit center.
    pub fn diverging(family: NumericFamily) -> Self {
        Self::Diverging {
            family,
            domain: if matches!(family, NumericFamily::Log { .. }) {
                [Number(0.1), Number(1.), Number(10.)]
            } else {
                [Number(0.), Number(0.5), Number(1.)]
            },
            clamp: false,
        }
    }
}
/// Prepared normalization usable with native custom `Sample` implementations.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleNormalizer {
    spec: NormalizationSpec,
    domain: Vec<Number>,
    transformed: Vec<f64>,
    factors: [f64; 2],
    negative: bool,
}
impl ScaleNormalizer {
    /// Validate configuration and prepare shared transformed endpoints or sorted samples.
    pub fn new(spec: NormalizationSpec) -> ChartResult<Self> {
        let (domain, family, diverging) = match &spec {
            NormalizationSpec::Sequential { family, domain, .. } => {
                (domain.to_vec(), Some(*family), false)
            }
            NormalizationSpec::Diverging { family, domain, .. } => {
                (domain.to_vec(), Some(*family), true)
            }
            NormalizationSpec::Quantile { samples } => {
                if samples.len() > MAX_VALUES {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Rank scale population exceeds its budget.",
                    ));
                }
                (sorted_samples(samples), None, false)
            }
        };
        if let Some(family) = family {
            let parameter = match family {
                NumericFamily::Linear => 1.,
                NumericFamily::Pow { exponent } => exponent,
                NumericFamily::Log { base } => base,
                NumericFamily::Symlog { constant } => constant,
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Identity and radial are separate numeric scales, not normalization transforms.",
                    ));
                }
            };
            if !parameter.is_finite() {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Scale transform parameters must be finite.",
                ));
            }
        }
        let negative = domain.first().is_some_and(|v| v.0 < 0.);
        let transformed: Vec<_> = domain
            .iter()
            .map(|v| family.map_or(v.0, |f| transform(f, negative, v.0, false)))
            .collect();
        let mut factors = [0.; 2];
        if family.is_some() {
            factors[0] = reciprocal(
                transformed[0],
                transformed[1],
                if diverging { 0.5 } else { 1. },
            );
            if diverging {
                factors[1] = reciprocal(transformed[1], transformed[2], 0.5);
            }
        }
        Ok(Self {
            spec,
            domain,
            transformed,
            factors,
            negative,
        })
    }
    /// Original descriptor.
    pub fn spec(&self) -> &NormalizationSpec {
        &self.spec
    }
    /// Authored domain or sorted rank population.
    pub fn domain(&self) -> &[Number] {
        &self.domain
    }
    fn tick_family(&self) -> ChartResult<NumericFamily> {
        match self.spec {
            NormalizationSpec::Sequential { family, .. }
            | NormalizationSpec::Diverging { family, .. } => Ok(family),
            NormalizationSpec::Quantile { .. } => Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Rank scales expose quantiles rather than continuous ticks or nice.",
            )),
        }
    }
    /// Unthinned data-space ticks; empirical rank scales expose `quantiles` instead.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
        self.tick_family()?.ticks(&self.domain, count, budget)
    }
    /// Independent inferred tick labels, including transformed-family log suppression.
    pub fn tick_format(
        &self,
        count: f64,
        specifier: Option<&str>,
        locale: crate::typography::NumericLocale,
    ) -> ChartResult<crate::typography::NumericFormatter> {
        self.tick_family()?
            .tick_format(&self.domain, count, specifier, locale)
    }
    /// Nice the two outer endpoints while preserving a diverging midpoint and the original scale.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        let family = self.tick_family()?;
        let start = self.domain[0].0;
        let stop = self.domain.last().expect("normalized endpoints").0;
        let (a, b) = if let NumericFamily::Log { base } = family {
            super::ticks::nice_log(start, stop, base)
        } else {
            super::ticks::nice(start, stop, count)
        };
        let mut spec = self.spec.clone();
        match &mut spec {
            NormalizationSpec::Sequential { domain, .. } => *domain = [Number(a), Number(b)],
            NormalizationSpec::Diverging { domain, .. } => {
                domain[0] = Number(a);
                domain[2] = Number(b);
            }
            NormalizationSpec::Quantile { .. } => unreachable!(),
        }
        Self::new(spec)
    }
    /// Normalized parameter. Missing/NaN source inputs remain absent; IEEE results are retained.
    pub fn parameter(&self, input: Option<f64>) -> Option<f64> {
        let x = input.filter(|x| !x.is_nan())?;
        let (t, clamp) = match self.spec {
            NormalizationSpec::Sequential { family, clamp, .. } => {
                let t = if self.factors[0] == 0. {
                    0.5
                } else {
                    (transform(family, self.negative, x, false) - self.transformed[0])
                        * self.factors[0]
                };
                (t, clamp)
            }
            NormalizationSpec::Diverging { family, clamp, .. } => {
                let x = transform(family, self.negative, x, false);
                let s = if self.transformed[1] < self.transformed[0] {
                    -1.
                } else {
                    1.
                };
                let side = usize::from(
                    s * x >= s * self.transformed[1] || x.is_nan() || self.transformed[1].is_nan(),
                );
                (0.5 + (x - self.transformed[1]) * self.factors[side], clamp)
            }
            NormalizationSpec::Quantile { .. } => {
                let rank = bisect_right(&self.domain, |d| d.0 <= x, 1);
                ((rank as f64 - 1.) / (self.domain.len() as f64 - 1.), false)
            }
        };
        Some(if clamp { t.clamp(0., 1.) } else { t })
    }
    /// Reuse the common native sampling contract for any owned output type.
    pub fn map_with<T>(
        &self,
        input: Option<f64>,
        interpolator: &impl Sample<T>,
    ) -> ChartResult<Option<T>> {
        self.parameter(input)
            .map(|t| interpolator.sample(t))
            .transpose()
    }
    /// Sample quantiles at `i/count`; fractional counts retain reference length semantics.
    pub fn quantiles(&self, count: f64) -> ChartResult<Vec<Option<Number>>> {
        if !matches!(self.spec, NormalizationSpec::Quantile { .. }) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Only empirical rank scales expose population quantiles.",
            ));
        }
        let length = if count.is_nan() || count <= -1. {
            0.
        } else {
            (count + 1.).floor()
        };
        if !length.is_finite() || length > MAX_VALUES as f64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Quantile request exceeds its sample budget.",
            ));
        }
        Ok((0..length as usize)
            .map(|i| quantile_sorted(&self.domain, i as f64 / count))
            .collect())
    }
}
fn reciprocal(a: f64, b: f64, width: f64) -> f64 {
    if a == b { 0. } else { width / (b - a) }
}

/// Output function compiled by the common interpolation owner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleRangeFunction {
    /// Reference default identity, preserving exceptional numbers and negative zero.
    Identity,
    /// A bounded built-in operation; no renderer or interpreter callbacks.
    Interpolate(InterpolationSpec),
}
/// Portable sequential/diverging/rank configuration with typed outputs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolatedScaleSpec {
    /// Domain normalization, independent of the output palette or interpolator.
    pub normalization: NormalizationSpec,
    /// Range function.
    pub output: ScaleRangeFunction,
    /// Missing input output. Rank scales preserve the reference's missing result.
    pub unknown: Value,
}
impl InterpolatedScaleSpec {
    /// Sequential defaults with the identity range function.
    pub fn sequential(family: NumericFamily) -> Self {
        Self {
            normalization: NormalizationSpec::sequential(family),
            output: ScaleRangeFunction::Identity,
            unknown: Value::Missing,
        }
    }
    /// Diverging defaults with the identity range function.
    pub fn diverging(family: NumericFamily) -> Self {
        Self {
            normalization: NormalizationSpec::diverging(family),
            output: ScaleRangeFunction::Identity,
            unknown: Value::Missing,
        }
    }
    /// Empty empirical rank population and identity range function.
    pub fn quantile() -> Self {
        Self {
            normalization: NormalizationSpec::Quantile { samples: vec![] },
            output: ScaleRangeFunction::Identity,
            unknown: Value::Missing,
        }
    }
}
/// Prepared typed range mapping and floating paint sampling.
#[derive(Clone, Debug)]
pub struct InterpolatedScale {
    spec: InterpolatedScaleSpec,
    normalization: ScaleNormalizer,
    output: Option<Interpolator>,
}
impl InterpolatedScale {
    /// Compile normalization and output once, before mapping any marks.
    pub fn new(spec: InterpolatedScaleSpec) -> ChartResult<Self> {
        spec.unknown.validate()?;
        if matches!(spec.normalization, NormalizationSpec::Quantile { .. })
            && spec.unknown != Value::Missing
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Sequential quantile has no configurable unknown output.",
            ));
        }
        let normalization = ScaleNormalizer::new(spec.normalization.clone())?;
        let output = match &spec.output {
            ScaleRangeFunction::Identity => None,
            ScaleRangeFunction::Interpolate(spec) => Some(Interpolator::new(spec.clone())?),
        };
        Ok(Self {
            spec,
            normalization,
            output,
        })
    }
    pub(super) fn validate_color_output(&self) -> ChartResult<()> {
        let output = self.output.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "A color scale requires a color interpolator.",
            )
        })?;
        output.scale_color(0.)?;
        Ok(())
    }
    pub(super) fn validate_numeric_output(&self) -> ChartResult<()> {
        for t in [0., 0.5, 1.] {
            super::mapped::numeric_output(&self.sample(t)?)?;
        }
        Ok(())
    }
    /// Immutable authored descriptor.
    pub fn spec(&self) -> &InterpolatedScaleSpec {
        &self.spec
    }
    /// Shared prepared normalization and population queries.
    pub fn normalizer(&self) -> &ScaleNormalizer {
        &self.normalization
    }
    /// Unthinned data-space candidates; rank scales have quantiles instead.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
        self.normalization.ticks(count, budget)
    }
    /// Independent labels with inferred precision.
    pub fn tick_format(
        &self,
        count: f64,
        specifier: Option<&str>,
        locale: crate::typography::NumericLocale,
    ) -> ChartResult<crate::typography::NumericFormatter> {
        self.normalization.tick_format(count, specifier, locale)
    }
    /// Copy with niced outer endpoints; the output function and midpoint are retained.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        let mut spec = self.spec.clone();
        spec.normalization = self.normalization.nice(count)?.spec;
        Self::new(spec)
    }
    /// Map a source number to an independently owned typed value.
    pub fn map(&self, input: Option<f64>) -> ChartResult<Value> {
        match self.normalization.parameter(input) {
            None => Ok(self.spec.unknown.clone()),
            Some(t) => self.sample(t),
        }
    }
    fn sample(&self, t: f64) -> ChartResult<Value> {
        self.output
            .as_ref()
            .map_or_else(|| Ok(Value::number(t)), |f| f.scale_sample(t))
    }
    /// Floating color without CSS or byte conversion. Missing inputs return no color.
    pub fn map_color(&self, input: Option<f64>) -> ChartResult<Option<ColorValue>> {
        let Some(t) = self.normalization.parameter(input) else {
            return Ok(None);
        };
        self.output
            .as_ref()
            .ok_or_else(|| {
                error(
                    DiagnosticCode::UnsupportedCapability,
                    "Identity output is not a color interpolator.",
                )
            })?
            .scale_color(t)
            .map(Some)
    }
    /// Reference range getter: endpoint/center samples or one sample per rank observation.
    pub fn range(&self) -> ChartResult<Vec<Value>> {
        let count = match self.spec.normalization {
            NormalizationSpec::Sequential { .. } => 2,
            NormalizationSpec::Diverging { .. } => 3,
            NormalizationSpec::Quantile { .. } => self.normalization.domain.len(),
        };
        (0..count)
            .map(|i| self.sample(i as f64 / (count as f64 - 1.)))
            .collect()
    }
}

/// Typed continuous outputs with arbitrarily spaced domain knots and a shared factory.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinuousScaleSpec {
    /// Shared linear/log/pow/symlog transform (identity/radial use numeric scales).
    pub family: NumericFamily,
    /// Ordered authored input knots.
    pub domain: Vec<Number>,
    /// Typed range knots; the effective mapping uses the shorter knot sequence.
    pub range: Vec<Value>,
    /// Shared interpolation factory, including round and floating color spaces.
    pub factory: crate::interpolate::InterpolationFactory,
    /// Clamp input to the effective domain endpoints.
    pub clamp: bool,
    /// Owned missing/NaN input result.
    pub unknown: Value,
}
impl ContinuousScaleSpec {
    /// Reference continuous defaults with target-kind interpolation and a numeric unit range.
    pub fn d3(family: NumericFamily) -> Self {
        let numeric = super::NumericScaleSpec::d3(family);
        Self {
            family,
            domain: numeric.domain,
            range: numeric.range.into_iter().map(Value::Number).collect(),
            factory: crate::interpolate::InterpolationFactory::new(
                crate::interpolate::FactoryKind::Value,
            ),
            clamp: false,
            unknown: Value::Missing,
        }
    }
}
/// Prepared generic continuous range with the same knot normalizer as numeric scales.
#[derive(Clone, Debug)]
pub struct ContinuousScale {
    spec: ContinuousScaleSpec,
    knots: super::numeric::KnotMapping,
    segments: Vec<Interpolator>,
    negative: bool,
    extent: Option<(f64, f64)>,
}
impl ContinuousScale {
    /// Compile each pair once; typed mapping does not advertise a numeric inverse.
    pub fn new(spec: ContinuousScaleSpec) -> ChartResult<Self> {
        if spec.domain.len().saturating_add(spec.range.len()) > MAX_VALUES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Continuous scale exceeds its knot budget.",
            ));
        }
        spec.unknown.validate()?;
        let first = spec.domain.first().copied().unwrap_or(Number(f64::NAN));
        ScaleNormalizer::new(NormalizationSpec::Sequential {
            family: spec.family,
            domain: [first, first],
            clamp: false,
        })?;
        if spec.domain.iter().any(|d| !d.0.is_finite())
            || !(spec.domain.windows(2).all(|p| p[0].0 <= p[1].0)
                || spec.domain.windows(2).all(|p| p[0].0 >= p[1].0))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Continuous domain knots must be finite and monotone.",
            ));
        }
        for v in &spec.range {
            v.validate()?;
        }
        let negative = first.0 < 0.;
        let n = spec.domain.len().min(spec.range.len());
        let extent = (n > 0).then(|| {
            (
                first.0.min(spec.domain[n - 1].0),
                first.0.max(spec.domain[n - 1].0),
            )
        });
        let domain = spec
            .domain
            .iter()
            .map(|d| transform(spec.family, negative, d.0, false))
            .collect();
        let (knots, reversed) = super::numeric::KnotMapping::new(domain, spec.range.len());
        let mut range = spec.range.clone();
        if reversed {
            if n > 2 {
                range.reverse();
            } else if range.len() > 1 {
                range.swap(0, 1);
            }
        }
        let segments = (0..knots.count())
            .map(|i| {
                spec.factory.between(
                    range.get(i).cloned().unwrap_or(Value::Missing),
                    range.get(i + 1).cloned().unwrap_or(Value::Missing),
                )
            })
            .collect::<ChartResult<_>>()?;
        Ok(Self {
            spec,
            knots,
            segments,
            negative,
            extent,
        })
    }
    pub(super) fn validate_color_output(&self) -> ChartResult<()> {
        for segment in &self.segments {
            segment.scale_color(0.)?;
        }
        Ok(())
    }
    pub(super) fn validate_numeric_output(&self) -> ChartResult<()> {
        for segment in &self.segments {
            super::mapped::numeric_output(&segment.scale_sample(0.)?)?;
        }
        Ok(())
    }
    /// Authored immutable configuration.
    pub fn spec(&self) -> &ContinuousScaleSpec {
        &self.spec
    }
    /// Unthinned data-space candidates independent of range value type.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
        self.spec.family.ticks(&self.spec.domain, count, budget)
    }
    /// Independent labels with inferred precision.
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
    /// Copy with niced outer endpoints; interpolation and interior knots remain unchanged.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        let mut numeric = super::NumericScaleSpec::d3(self.spec.family);
        numeric.domain = self.spec.domain.clone();
        let domain = super::NumericScale::new(numeric)?
            .nice(count)?
            .spec()
            .domain
            .clone();
        let mut spec = self.spec.clone();
        spec.domain = domain;
        Self::new(spec)
    }
    fn segment(&self, input: Option<f64>) -> Option<(&Interpolator, f64)> {
        let mut x = input.filter(|x| !x.is_nan())?;
        if self.spec.clamp {
            x = self.extent.map_or(f64::NAN, |(a, b)| x.clamp(a, b));
        }
        let (i, t) = self
            .knots
            .parameter(transform(self.spec.family, self.negative, x, false));
        Some((&self.segments[i], t))
    }
    /// Typed result through the shared interpolation engine.
    pub fn map(&self, input: Option<f64>) -> ChartResult<Value> {
        self.segment(input)
            .map_or_else(|| Ok(self.spec.unknown.clone()), |(f, t)| f.scale_sample(t))
    }
    /// Floating color before final scene-paint lowering.
    pub fn map_color(&self, input: Option<f64>) -> ChartResult<Option<ColorValue>> {
        self.segment(input)
            .map(|(f, t)| f.scale_color(t))
            .transpose()
    }
}
