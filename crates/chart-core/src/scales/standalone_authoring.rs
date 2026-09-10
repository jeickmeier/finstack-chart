//! Named standalone constructors and checked options, shared by all host facades.
use super::*;
use crate::{
    data::TimeUnit,
    interpolate::{FactoryKind, InterpolationFactory, InterpolationSpec, Number, Value},
};
use serde::{Deserialize, Serialize};

/// Named reference families. Sqrt constructors retain a power-family descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleConstructor {
    /// Piecewise affine scale.
    Linear,
    /// Logarithmic scale.
    Log,
    /// Sign-preserving power scale.
    Pow,
    /// Power with exponent one half.
    Sqrt,
    /// Symmetric logarithmic scale.
    Symlog,
    /// Identity mapping.
    Identity,
    /// Signed square-area radius mapping.
    Radial,
    /// Typed category catalog.
    Ordinal,
    /// Categorical bands.
    Band,
    /// Categorical points.
    Point,
    /// Sample quantile classifier.
    Quantile,
    /// Equal-width classifier.
    Quantize,
    /// Typed cutpoint classifier.
    Threshold,
    /// Sequential affine normalization.
    Sequential,
    /// Sequential logarithmic normalization.
    SequentialLog,
    /// Sequential power normalization.
    SequentialPow,
    /// Sequential square-root normalization.
    SequentialSqrt,
    /// Sequential symmetric logarithmic normalization.
    SequentialSymlog,
    /// Sequential empirical ranks.
    SequentialQuantile,
    /// Diverging affine normalization.
    Diverging,
    /// Diverging logarithmic normalization.
    DivergingLog,
    /// Diverging power normalization.
    DivergingPow,
    /// Diverging square-root normalization.
    DivergingSqrt,
    /// Diverging symmetric logarithmic normalization.
    DivergingSymlog,
    /// Explicit UTC calendar.
    Utc,
    /// Local calendar with a required supplied timezone resource.
    Local,
}

/// Constructor/reconfiguration options. Unsupported options diagnose rather than disappear.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScaleOptions {
    /// Typed numeric, categorical or exact timestamp domain.
    pub domain: Option<Vec<ScaleInput>>,
    /// Owned output values, or two numeric destination endpoints for categories.
    pub range: Option<Vec<Value>>,
    /// Continuous input and inverse clamping.
    pub clamp: Option<bool>,
    /// Round output coordinates (or categorical spacing).
    pub round: Option<bool>,
    /// Explicit missing-input or unknown-category output.
    pub unknown: Option<Value>,
    /// Enable authoring-time implicit ordinal training; mutually exclusive with unknown.
    pub implicit: Option<bool>,
    /// Logarithm base.
    pub base: Option<f64>,
    /// Power exponent.
    pub exponent: Option<f64>,
    /// Symlog constant.
    pub constant: Option<f64>,
    /// Set both band paddings or point outer padding.
    pub padding: Option<f64>,
    /// Band inner padding, applied after shorthand padding.
    pub padding_inner: Option<f64>,
    /// Band outer padding, applied after shorthand padding.
    pub padding_outer: Option<f64>,
    /// Categorical leftover-space alignment.
    pub align: Option<f64>,
    /// Built-in interpolation factory for continuous ranges.
    pub factory: Option<InterpolationFactory>,
    /// Explicit output interpolator for sequential/diverging/rank families.
    pub interpolator: Option<InterpolationSpec>,
    /// Exact timestamp source unit.
    pub unit: Option<TimeUnit>,
    /// Shared timezone rules used for all local operations.
    pub zone: Option<CalendarZone>,
}
fn invalid(message: &str) -> crate::Diagnostic {
    error(DiagnosticCode::UnsupportedCapability, message)
}
fn numbers(domain: Vec<ScaleInput>, missing: bool) -> ChartResult<Vec<Option<Number>>> {
    domain
        .into_iter()
        .map(|v| match v {
            ScaleInput::Number(n) => Ok(Some(n)),
            ScaleInput::Missing if missing => Ok(None),
            _ => Err(invalid("This domain requires numeric inputs.")),
        })
        .collect()
}
fn keys(domain: Vec<ScaleInput>) -> ChartResult<Vec<ScaleKey>> {
    domain
        .into_iter()
        .map(|v| match v {
            ScaleInput::Key(k) => Ok(k),
            _ => Err(invalid("This domain requires typed category keys.")),
        })
        .collect()
}
fn numeric_values(values: Vec<Value>) -> ChartResult<Vec<Number>> {
    values
        .into_iter()
        .map(|v| {
            if let Value::Number(n) = v {
                Ok(n)
            } else {
                Err(invalid("This range requires numeric values."))
            }
        })
        .collect()
}
fn endpoints<const N: usize>(domain: Vec<ScaleInput>) -> ChartResult<[Number; N]> {
    let values = numbers(domain, false)?;
    Ok(std::array::from_fn(|i| {
        values.get(i).copied().flatten().unwrap_or(Number(f64::NAN))
    }))
}
impl ScaleConstructor {
    /// Reference defaults prepared through the same checked option path as reconfiguration.
    pub fn create(self, options: ScaleOptions) -> ChartResult<StandaloneScale> {
        self.create_with_registry(options, &crate::grammar::ExtensionRegistry::new())
    }
    /// Construct against explicitly installed native range factories.
    pub fn create_with_registry(
        self,
        mut options: ScaleOptions,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<StandaloneScale> {
        use ScaleConstructor as C;
        let family = match self {
            C::Log | C::SequentialLog | C::DivergingLog => NumericFamily::Log { base: 10. },
            C::Pow | C::SequentialPow | C::DivergingPow => NumericFamily::Pow { exponent: 1. },
            C::Sqrt | C::SequentialSqrt | C::DivergingSqrt => NumericFamily::Pow { exponent: 0.5 },
            C::Symlog | C::SequentialSymlog | C::DivergingSymlog => {
                NumericFamily::Symlog { constant: 1. }
            }
            C::Identity => NumericFamily::Identity,
            C::Radial => NumericFamily::Radial,
            _ => NumericFamily::Linear,
        };
        let spec = match self {
            C::Ordinal => StandaloneScaleSpec::Ordinal(OrdinalSpec::default()),
            C::Band => StandaloneScaleSpec::Band {
                spec: BandSpec::default(),
                range: Bounds::new(0., 1.)?,
            },
            C::Point => StandaloneScaleSpec::Point {
                spec: PointSpec::default(),
                range: Bounds::new(0., 1.)?,
            },
            C::Quantile => StandaloneScaleSpec::Classifier(ClassifierSpec::quantile()),
            C::Quantize => StandaloneScaleSpec::Classifier(ClassifierSpec::quantize()),
            C::Threshold => StandaloneScaleSpec::Threshold(ThresholdSpec::d3()),
            C::Sequential
            | C::SequentialLog
            | C::SequentialPow
            | C::SequentialSqrt
            | C::SequentialSymlog => {
                StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec::sequential(family))
            }
            C::Diverging
            | C::DivergingLog
            | C::DivergingPow
            | C::DivergingSqrt
            | C::DivergingSymlog => {
                StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec::diverging(family))
            }
            C::SequentialQuantile => {
                StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec::quantile())
            }
            C::Utc | C::Local => {
                if self == C::Utc
                    && options
                        .zone
                        .as_ref()
                        .is_some_and(|z| *z != CalendarZone::Utc)
                {
                    return Err(invalid(
                        "Use the local constructor for supplied timezone rules.",
                    ));
                }
                let zone = if self == C::Local {
                    options.zone.take().ok_or_else(|| {
                        invalid("Local time requires a supplied timezone resource.")
                    })?
                } else {
                    CalendarZone::Utc
                };
                let mut spec = if options.domain.is_some() {
                    TimeScaleSpec {
                        zone,
                        ..Default::default()
                    }
                } else {
                    TimeScaleSpec::local(zone)?
                };
                if options.domain.is_none()
                    && let Some(unit) = options.unit
                {
                    let calendar = Calendar::new(spec.zone.clone())?;
                    spec.domain = spec
                        .domain
                        .iter()
                        .map(|t| {
                            calendar.from_components(
                                calendar.components(*t, TimeUnit::Milliseconds)?,
                                unit,
                            )
                        })
                        .collect::<ChartResult<_>>()?;
                }
                StandaloneScaleSpec::Time(spec)
            }
            _ => StandaloneScaleSpec::Numeric(NumericScaleSpec::d3(family)),
        };
        options.apply_spec(spec, registry.interpolations.clone())
    }
}
impl ScaleOptions {
    /// Atomically apply only explicitly supplied fields, preserving family and other settings.
    pub fn apply(self, scale: &StandaloneScale) -> ChartResult<StandaloneScale> {
        self.apply_spec(scale.spec().clone(), scale.registrations.clone())
    }
    fn apply_spec(
        mut self,
        mut spec: StandaloneScaleSpec,
        registrations: std::sync::Arc<
            crate::grammar::interpolation_extensions::InterpolationRegistrations,
        >,
    ) -> ChartResult<StandaloneScale> {
        use StandaloneScaleSpec as S;
        if self.interpolator.is_some()
            && (self.range.is_some() || self.factory.is_some() || self.round.is_some())
        {
            return Err(invalid(
                "Choose an output interpolator or range/factory options, not both.",
            ));
        }
        if self.unknown.is_some() && self.implicit.is_some() {
            return Err(invalid(
                "Choose explicit unknown output or ordinal implicit policy.",
            ));
        }
        // Generic continuous ranges reuse the shared kernel; identity/radial remain numeric.
        if let S::Numeric(s) = &spec {
            let typed = self.factory.is_some()
                || self
                    .range
                    .as_ref()
                    .is_some_and(|r| r.iter().any(|v| !matches!(v, Value::Number(_))));
            if typed {
                if matches!(s.family, NumericFamily::Identity | NumericFamily::Radial) {
                    return Err(invalid("Identity and radial ranges are numeric."));
                }
                spec = S::Continuous(ContinuousScaleSpec {
                    family: s.family,
                    domain: s.domain.clone(),
                    range: s.range.iter().copied().map(Value::Number).collect(),
                    factory: InterpolationFactory::new(if s.round {
                        FactoryKind::Round
                    } else {
                        FactoryKind::Value
                    }),
                    clamp: s.clamp,
                    unknown: s.unknown.clone(),
                });
            }
        }
        let family = match &mut spec {
            S::Numeric(s) => Some(&mut s.family),
            S::Continuous(s) => Some(&mut s.family),
            S::Interpolated(s) => match &mut s.normalization {
                NormalizationSpec::Sequential { family, .. }
                | NormalizationSpec::Diverging { family, .. } => Some(family),
                _ => None,
            },
            _ => None,
        };
        if let Some(family) = family {
            match family {
                NumericFamily::Log { base } => {
                    if let Some(v) = self.base.take() {
                        *base = v;
                    }
                }
                NumericFamily::Pow { exponent } => {
                    if let Some(v) = self.exponent.take() {
                        *exponent = v;
                    }
                }
                NumericFamily::Symlog { constant } => {
                    if let Some(v) = self.constant.take() {
                        *constant = v;
                    }
                }
                _ => {}
            }
        }
        if let Some(domain) = self.domain.take() {
            match &mut spec {
                S::Numeric(s) => s.domain = numbers(domain, false)?.into_iter().flatten().collect(),
                S::Continuous(s) => {
                    s.domain = numbers(domain, false)?.into_iter().flatten().collect()
                }
                S::Ordinal(s) => s.domain = keys(domain)?,
                S::Band { spec, .. } => spec.domain = Some(keys(domain)?),
                S::Point { spec, .. } => spec.domain = Some(keys(domain)?),
                S::Threshold(s) => s.domain = keys(domain)?,
                S::Classifier(s) => {
                    s.domain = match &s.domain {
                        ClassifierDomain::Quantile(_) => {
                            ClassifierDomain::Quantile(numbers(domain, true)?)
                        }
                        ClassifierDomain::Quantize(_) => {
                            ClassifierDomain::Quantize(endpoints(domain)?)
                        }
                    }
                }
                S::Interpolated(s) => match &mut s.normalization {
                    NormalizationSpec::Sequential { domain: d, .. } => *d = endpoints(domain)?,
                    NormalizationSpec::Diverging { domain: d, .. } => *d = endpoints(domain)?,
                    NormalizationSpec::Quantile { samples } => *samples = numbers(domain, true)?,
                },
                S::Time(s) => {
                    s.domain = domain
                        .into_iter()
                        .map(|v| {
                            if let ScaleInput::Time(t) = v {
                                Ok(t)
                            } else {
                                Err(invalid("Time domains require exact timestamp inputs."))
                            }
                        })
                        .collect::<ChartResult<_>>()?
                }
            }
        }
        if let Some(range) = self.range.take() {
            match &mut spec {
                S::Numeric(s) => {
                    s.range = numeric_values(range)?;
                    if s.family == NumericFamily::Identity {
                        s.domain = s.range.clone();
                    }
                }
                S::Continuous(s) => s.range = range,
                S::Ordinal(s) => s.range = range,
                S::Classifier(s) => s.range = range,
                S::Threshold(s) => s.range = range,
                S::Time(s) => s.range = range,
                S::Band { range: r, .. } | S::Point { range: r, .. } => {
                    let n = numeric_values(range)?;
                    if n.len() != 2 {
                        return Err(invalid("Categorical ranges require two endpoints."));
                    }
                    *r = Bounds::new(n[0].0, n[1].0)?;
                }
                S::Interpolated(s) => {
                    let factory = self
                        .factory
                        .take()
                        .unwrap_or_else(|| InterpolationFactory::new(FactoryKind::Value));
                    let factory = if self.round.take().unwrap_or(false) {
                        InterpolationFactory::new(FactoryKind::Round)
                    } else {
                        factory
                    };
                    s.output = ScaleRangeFunction::Interpolate(match s.normalization {
                        NormalizationSpec::Sequential { .. } => InterpolationSpec::Between {
                            factory,
                            a: range.first().cloned().unwrap_or(Value::Missing),
                            b: range.get(1).cloned().unwrap_or(Value::Missing),
                        },
                        NormalizationSpec::Diverging { .. } => InterpolationSpec::Piecewise {
                            factory,
                            values: (0..3)
                                .map(|i| range.get(i).cloned().unwrap_or(Value::Missing))
                                .collect(),
                        },
                        _ => {
                            return Err(invalid(
                                "Rank scales use an interpolator rather than a range setter.",
                            ));
                        }
                    });
                }
            }
        }
        if let Some(clamp) = self.clamp.take() {
            match &mut spec {
                S::Numeric(s) if s.family != NumericFamily::Identity => s.clamp = clamp,
                S::Continuous(s) => s.clamp = clamp,
                S::Time(s) => s.clamp = clamp,
                S::Interpolated(s) => match &mut s.normalization {
                    NormalizationSpec::Sequential { clamp: c, .. }
                    | NormalizationSpec::Diverging { clamp: c, .. } => *c = clamp,
                    _ => return Err(invalid("Rank scales do not clamp.")),
                },
                _ => return Err(invalid("This scale does not support clamp.")),
            }
        }
        if let Some(round) = self.round.take() {
            match &mut spec {
                S::Numeric(s) if s.family != NumericFamily::Identity => s.round = round,
                S::Continuous(s) => {
                    s.factory = InterpolationFactory::new(if round {
                        FactoryKind::Round
                    } else {
                        FactoryKind::Value
                    })
                }
                S::Time(s) => {
                    s.factory = InterpolationFactory::new(if round {
                        FactoryKind::Round
                    } else {
                        FactoryKind::Value
                    })
                }
                S::Band { spec, .. } => spec.round = round,
                S::Point { spec, .. } => spec.round = round,
                _ => {
                    return Err(invalid(
                        "This scale requires a numeric range-round operation.",
                    ));
                }
            }
        }
        if let Some(factory) = self.factory.take() {
            match &mut spec {
                S::Continuous(s) => s.factory = factory,
                S::Time(s) => s.factory = factory,
                _ => {
                    return Err(invalid(
                        "This scale does not accept a binary interpolation factory here.",
                    ));
                }
            }
        }
        if let Some(unknown) = self.unknown.take() {
            match &mut spec {
                S::Numeric(s) => s.unknown = unknown,
                S::Continuous(s) => s.unknown = unknown,
                S::Time(s) => s.unknown = unknown,
                S::Ordinal(s) => s.unknown = OrdinalUnknown::Explicit(Some(unknown)),
                S::Classifier(s) => s.unknown = Some(unknown),
                S::Threshold(s) => s.unknown = Some(unknown),
                S::Interpolated(s)
                    if !matches!(s.normalization, NormalizationSpec::Quantile { .. }) =>
                {
                    s.unknown = unknown
                }
                _ => return Err(invalid("This family has no configurable unknown output.")),
            }
        }
        if let Some(implicit) = self.implicit.take() {
            if let S::Ordinal(s) = &mut spec {
                s.unknown = if implicit {
                    OrdinalUnknown::Implicit
                } else {
                    OrdinalUnknown::Explicit(None)
                };
            } else {
                return Err(invalid("Implicit training is an ordinal option."));
            }
        }
        match &mut spec {
            S::Band { spec, .. } => {
                if let Some(p) = self.padding.take() {
                    spec.padding_inner = p;
                    spec.padding_outer = p;
                }
                if let Some(p) = self.padding_inner.take() {
                    spec.padding_inner = p;
                }
                if let Some(p) = self.padding_outer.take() {
                    spec.padding_outer = p;
                }
                if let Some(a) = self.align.take() {
                    spec.align = a;
                }
            }
            S::Point { spec, .. } => {
                if let Some(p) = self.padding.take() {
                    spec.padding = p;
                }
                if let Some(a) = self.align.take() {
                    spec.align = a;
                }
            }
            S::Interpolated(s) => {
                if let Some(i) = self.interpolator.take() {
                    s.output = ScaleRangeFunction::Interpolate(i);
                }
            }
            S::Time(s) => {
                if let Some(unit) = self.unit.take() {
                    s.unit = unit;
                }
                if let Some(zone) = self.zone.take() {
                    s.zone = zone;
                }
            }
            _ => {}
        }
        if self != Self::default() {
            return Err(invalid("An option is not supported by this scale family."));
        }
        StandaloneScale::new_with_registrations(spec, registrations)
    }
}
