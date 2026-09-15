//! Owned portable scale operations. Preparation and arithmetic stay in the family kernels.
use super::*;
use crate::{
    interpolate::{Number, Value},
    portable,
    typography::{NumericFormatter, NumericLocale},
};
use serde::{Deserialize, Serialize};

/// One versioned configuration surface for native and host standalone scales.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StandaloneScaleSpec {
    /// Numeric output, including identity and radial transforms.
    Numeric(NumericScaleSpec),
    /// Typed piecewise interpolation.
    Continuous(ContinuousScaleSpec),
    /// Sequential, diverging or rank-based interpolation.
    Interpolated(InterpolatedScaleSpec),
    /// Explicitly trained typed categories.
    Ordinal(OrdinalSpec<ScaleKey, Value>),
    /// Categorical band starts and extents.
    Band {
        /// Spacing and domain.
        spec: BandSpec<ScaleKey>,
        /// Destination endpoints.
        range: Bounds,
    },
    /// Categorical points.
    Point {
        /// Spacing and domain.
        spec: PointSpec<ScaleKey>,
        /// Destination endpoints.
        range: Bounds,
    },
    /// Quantile or quantize classification.
    Classifier(ClassifierSpec<Value>),
    /// Typed threshold classification.
    Threshold(ThresholdSpec<ScaleKey, Value>),
    /// Integer calendar scale with explicit timezone resources.
    Time(TimeScaleSpec),
}

/// Typed scale input; missing values and null category identities remain distinct.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleInput {
    /// Absent input.
    Missing,
    /// Binary64 input, including explicitly classified exceptional values.
    Number(Number),
    /// Typed categorical or threshold identity.
    Key(ScaleKey),
    /// Exact timestamp in the scale's source unit.
    Time(#[serde(with = "crate::portable::signed")] i64),
}
/// Inverse output. Identity scales may return their arbitrary configured unknown value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleInverse {
    /// Numeric result or owned identity unknown output.
    Value(Value),
    /// Exact source-unit timestamp.
    Time(#[serde(with = "crate::portable::signed")] i64),
}

#[derive(Clone, Debug)]
enum Prepared {
    Numeric(NumericScale),
    Continuous {
        scale: ContinuousScale,
        inverse: Option<NumericScale>,
    },
    Interpolated(InterpolatedScale),
    Ordinal(OrdinalScale<ScaleKey, Value>),
    Category(CategoryScale<ScaleKey>),
    Classifier(ClassifierScale<Value>),
    Threshold(ThresholdScale<ScaleKey, Value>),
    Time(TimeScale),
}
/// Immutable prepared scale shared by Rust, Python and WASM.
#[derive(Clone, Debug)]
pub struct StandaloneScale {
    pub(crate) registrations:
        std::sync::Arc<crate::grammar::interpolation_extensions::InterpolationRegistrations>,
    pub(crate) transforms:
        std::sync::Arc<crate::grammar::transform_extensions::TransformRegistrations>,
    spec: StandaloneScaleSpec,
    prepared: Box<Prepared>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u32,
    spec: StandaloneScaleSpec,
}

fn capability(operation: &str) -> crate::Diagnostic {
    error(
        DiagnosticCode::UnsupportedCapability,
        format!("This scale does not support {operation}."),
    )
}
impl StandaloneScale {
    /// Validate and prepare one configuration atomically.
    pub fn new(spec: StandaloneScaleSpec) -> ChartResult<Self> {
        Self::new_with_registrations(spec, Default::default())
    }
    /// Prepare with an explicitly installed registry retained by copies and reconfiguration.
    pub fn new_with_registry(
        spec: StandaloneScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        Self::new_with_all_registrations(
            spec,
            registry.interpolations.clone(),
            registry.transforms_function.clone(),
            false,
        )
    }
    pub(crate) fn new_with_registrations(
        spec: StandaloneScaleSpec,
        registrations: std::sync::Arc<
            crate::grammar::interpolation_extensions::InterpolationRegistrations,
        >,
    ) -> ChartResult<Self> {
        Self::new_with_all_registrations(spec, registrations, Default::default(), false)
    }
    pub(crate) fn new_with_all_registrations(
        mut spec: StandaloneScaleSpec,
        registrations: std::sync::Arc<
            crate::grammar::interpolation_extensions::InterpolationRegistrations,
        >,
        transforms: std::sync::Arc<crate::grammar::transform_extensions::TransformRegistrations>,
        portable: bool,
    ) -> ChartResult<Self> {
        use StandaloneScaleSpec as S;
        // Check the same aggregate wire budget for native and transported descriptors.
        let encoded = portable::encode(&spec)?;
        let _: StandaloneScaleSpec = portable::decode(&encoded)?;
        if let Some(transform) = spec.ggplot_transform_mut() {
            transform.resolve_registrations(&transforms, portable)?;
        }
        let prepared = match &spec {
            S::Numeric(s) => Prepared::Numeric(NumericScale::new(s.clone())?),
            S::Continuous(s) => {
                let inverse = s
                    .range
                    .iter()
                    .map(|v| {
                        if let Value::Number(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|range| {
                        NumericScale::new(NumericScaleSpec {
                            compatibility: ScaleCompatibility::D3,
                            family: s.family.clone(),
                            domain: s.domain.clone(),
                            range,
                            clamp: s.clamp,
                            round: false,
                            unknown: Value::Missing,
                        })
                    })
                    .transpose()?;
                Prepared::Continuous {
                    scale: ContinuousScale::new_with_registrations(
                        s.clone(),
                        registrations.clone(),
                    )?,
                    inverse,
                }
            }
            S::Interpolated(s) => Prepared::Interpolated(
                InterpolatedScale::new_with_registrations(s.clone(), registrations.clone())?,
            ),
            S::Ordinal(s) => {
                for v in &s.range {
                    v.validate()?;
                }
                if let OrdinalUnknown::Explicit(Some(v)) = &s.unknown {
                    v.validate()?;
                }
                Prepared::Ordinal(OrdinalScale::new(s.clone()))
            }
            S::Band { spec, range } => Prepared::Category(CategoryScale::band(
                spec.clone(),
                Bounds::new(range.start(), range.end())?,
            )?),
            S::Point { spec, range } => Prepared::Category(CategoryScale::point(
                spec.clone(),
                Bounds::new(range.start(), range.end())?,
            )?),
            S::Classifier(s) => {
                for v in s.range.iter().chain(s.unknown.iter()) {
                    v.validate()?;
                }
                Prepared::Classifier(ClassifierScale::new(s.clone())?)
            }
            S::Threshold(s) => {
                for v in s.range.iter().chain(s.unknown.iter()) {
                    v.validate()?;
                }
                Prepared::Threshold(ThresholdScale::new(s.clone())?)
            }
            S::Time(s) => Prepared::Time(TimeScale::new_with_registrations(
                s.clone(),
                registrations.clone(),
            )?),
        };
        let mut result = Self {
            spec,
            prepared: Box::new(prepared),
            registrations,
            transforms,
        };
        // Preserve authored family while exposing normalized catalogs and parameters.
        match (&mut result.spec, &*result.prepared) {
            (S::Numeric(s), Prepared::Numeric(p)) => *s = p.spec().clone(),
            (S::Ordinal(s), Prepared::Ordinal(p)) => *s = p.spec().clone(),
            (S::Classifier(s), Prepared::Classifier(p)) => *s = p.spec().clone(),
            (S::Band { spec, .. }, Prepared::Category(p)) => *spec = p.spec().clone(),
            (S::Point { spec, .. }, Prepared::Category(p)) => {
                spec.domain = Some(p.domain().to_vec());
                spec.padding = p.spec().padding_outer;
                spec.align = p.spec().align;
            }
            _ => {}
        }
        Ok(result)
    }
    /// Normalized immutable configuration, including full authored knots.
    pub fn spec(&self) -> &StandaloneScaleSpec {
        &self.spec
    }
    /// Prepare a replacement without mutating this scale.
    pub fn reconfigure(&self, spec: StandaloneScaleSpec) -> ChartResult<Self> {
        Self::new_with_all_registrations(
            spec,
            self.registrations.clone(),
            self.transforms.clone(),
            false,
        )
    }
    /// Strict bounded version-one transport.
    pub fn to_json(&self) -> ChartResult<String> {
        self.spec
            .validate_registrations(&self.registrations, true)?;
        if let Some(transform) = self.spec.ggplot_transform() {
            transform.validate_portable()?;
        }
        portable::encode(&Wire {
            version: self.spec.wire_version(),
            spec: self.spec.clone(),
        })
    }
    /// Reject unknown versions and descriptors before preparing any scale.
    pub fn from_json(text: &str) -> ChartResult<Self> {
        Self::from_json_with_registry(text, &crate::grammar::ExtensionRegistry::new())
    }
    /// Decode a versioned scale only with explicitly installed portable factory versions.
    pub fn from_json_with_registry(
        text: &str,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let wire: Wire = portable::decode(text)?;
        if wire.version != wire.spec.wire_version() {
            return Err(capability("this descriptor version"));
        }
        wire.spec
            .validate_registrations(&registry.interpolations, true)?;
        Self::new_with_all_registrations(
            wire.spec,
            registry.interpolations.clone(),
            registry.transforms_function.clone(),
            true,
        )
    }
    /// Pure lookup; implicit ordinal growth occurs only through explicit `train`.
    pub fn map(&self, input: ScaleInput) -> ChartResult<Value> {
        let number = match input {
            ScaleInput::Missing => Some(None),
            ScaleInput::Number(n) => Some(Some(n.0)),
            _ => None,
        };
        let mapped = match (&*self.prepared, &input) {
            (Prepared::Numeric(s), _) if number.is_some() => {
                return s.map(number.expect("checked"));
            }
            (Prepared::Continuous { scale, .. }, _) if number.is_some() => {
                return scale.map(number.expect("checked"));
            }
            (Prepared::Interpolated(s), _) if number.is_some() => {
                return s.map(number.expect("checked"));
            }
            (Prepared::Classifier(s), _) if number.is_some() => {
                s.map(number.expect("checked")).cloned()
            }
            (Prepared::Ordinal(s), ScaleInput::Key(k)) => s.map(k).cloned(),
            (Prepared::Ordinal(s), ScaleInput::Missing) => match &s.spec().unknown {
                OrdinalUnknown::Explicit(v) => v.clone(),
                OrdinalUnknown::Implicit => None,
            },
            (Prepared::Category(s), ScaleInput::Key(k)) => s.map(k)?.map(Value::number),
            (Prepared::Category(_), ScaleInput::Missing) => None,
            (Prepared::Threshold(s), ScaleInput::Key(k)) => s.map_key(Some(k)).cloned(),
            (Prepared::Threshold(s), ScaleInput::Missing) => s.map_key(None).cloned(),
            (Prepared::Time(s), ScaleInput::Time(t)) => return s.map(Some(*t)),
            (Prepared::Time(s), ScaleInput::Missing) => return s.map(None),
            _ => return Err(capability("this input type")),
        };
        Ok(mapped.unwrap_or(Value::Missing))
    }
    /// Numeric inverse or exact timestamp; unsupported families diagnose explicitly.
    pub fn invert(&self, position: f64) -> ChartResult<ScaleInverse> {
        match &*self.prepared {
            Prepared::Numeric(s)
            | Prepared::Continuous {
                inverse: Some(s), ..
            } => Ok(ScaleInverse::Value(s.invert_output(Some(position))?)),
            Prepared::Time(s) => Ok(ScaleInverse::Time(s.invert(position)?)),
            _ => Err(capability("a continuous inverse")),
        }
    }
    /// Classifier interval; absent output membership differs from unbounded tails.
    pub fn invert_extent(&self, value: &Value) -> ChartResult<ScaleExtent<ScaleKey>> {
        match &*self.prepared {
            Prepared::Classifier(s) => {
                let e = s.invert_extent(value);
                Ok(ScaleExtent {
                    found: e.found,
                    lower: e.lower.map(ScaleKey::Number),
                    upper: e.upper.map(ScaleKey::Number),
                })
            }
            Prepared::Threshold(s) => Ok(s.invert_extent(value)),
            _ => Err(capability("invertExtent")),
        }
    }
    /// Unthinned numeric ticks with an independent hard output budget.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<Number>> {
        let ticks = match &*self.prepared {
            Prepared::Numeric(s) => s.ticks(count, budget),
            Prepared::Continuous { scale, .. } => scale.ticks(count, budget),
            Prepared::Interpolated(s) => s.ticks(count, budget),
            Prepared::Classifier(s) => s.ticks(count, budget),
            _ => Err(capability("numeric ticks")),
        }?;
        Ok(ticks.into_iter().map(Number).collect())
    }
    /// Numeric label preparation shared with chart guides.
    pub fn tick_format(
        &self,
        count: f64,
        specifier: Option<&str>,
        locale: NumericLocale,
    ) -> ChartResult<NumericFormatter> {
        match &*self.prepared {
            Prepared::Numeric(s) => s.tick_format(count, specifier, locale),
            Prepared::Continuous { scale, .. } => scale.tick_format(count, specifier, locale),
            Prepared::Interpolated(s) => s.tick_format(count, specifier, locale),
            Prepared::Classifier(s) => s.tick_format(count, specifier, locale),
            _ => Err(capability("numeric formatting")),
        }
    }
    /// Return an independently prepared scale with nice numeric endpoints.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        let spec = match &*self.prepared {
            Prepared::Numeric(s) => StandaloneScaleSpec::Numeric(s.nice(count)?.spec().clone()),
            Prepared::Continuous { scale, .. } => {
                StandaloneScaleSpec::Continuous(scale.nice(count)?.spec().clone())
            }
            Prepared::Interpolated(s) => {
                StandaloneScaleSpec::Interpolated(s.nice(count)?.spec().clone())
            }
            Prepared::Classifier(s) => {
                StandaloneScaleSpec::Classifier(s.nice(count)?.spec().clone())
            }
            _ => return Err(capability("numeric nice")),
        };
        Self::new_with_all_registrations(
            spec,
            self.registrations.clone(),
            self.transforms.clone(),
            false,
        )
    }
    /// Explicit immutable ordinal training, preserving first-seen order.
    pub fn train(&self, keys: Vec<ScaleKey>) -> ChartResult<Self> {
        let Prepared::Ordinal(s) = &*self.prepared else {
            return Err(capability("ordinal training"));
        };
        if keys.len() > crate::interpolate::MAX_VALUES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Ordinal training exceeds its input budget.",
            ));
        }
        Self::new_with_all_registrations(
            StandaloneScaleSpec::Ordinal(s.train(keys).spec().clone()),
            self.registrations.clone(),
            self.transforms.clone(),
            false,
        )
    }
    /// Prepared numeric classifier cuts.
    pub fn thresholds(&self) -> ChartResult<Vec<Option<Number>>> {
        if let Prepared::Classifier(s) = &*self.prepared {
            Ok(s.thresholds().to_vec())
        } else {
            Err(capability("classifier thresholds"))
        }
    }
    /// Rank-based sequential quantile sample boundaries.
    pub fn quantiles(&self, count: f64) -> ChartResult<Vec<Option<Number>>> {
        if let Prepared::Interpolated(s) = &*self.prepared {
            s.normalizer().quantiles(count)
        } else {
            Err(capability("sequential quantiles"))
        }
    }
    /// Prepared categorical metrics and lookup, using band starts for mapping.
    pub fn category(&self) -> ChartResult<&CategoryScale<ScaleKey>> {
        if let Prepared::Category(s) = &*self.prepared {
            Ok(s)
        } else {
            Err(capability("categorical spacing"))
        }
    }
    /// Prepared calendar operations with the same resource and timestamp unit.
    pub fn time(&self) -> ChartResult<&TimeScale> {
        if let Prepared::Time(s) = &*self.prepared {
            Ok(s)
        } else {
            Err(capability("calendar operations"))
        }
    }
    /// Effective sampled range for a sequential/diverging interpolator.
    pub fn sampled_range(&self) -> ChartResult<Vec<Value>> {
        if let Prepared::Interpolated(s) = &*self.prepared {
            s.range()
        } else {
            Err(capability("interpolated range sampling"))
        }
    }
    /// Observable normalized domain; quantile populations are sorted and ordinal catalogs trained.
    pub fn domain(&self) -> Vec<ScaleInput> {
        let numbers = match &*self.prepared {
            Prepared::Numeric(s) => &s.spec().domain,
            Prepared::Continuous { scale, .. } => &scale.spec().domain,
            Prepared::Interpolated(s) => s.normalizer().domain(),
            Prepared::Classifier(s) => s.domain(),
            Prepared::Ordinal(s) => {
                return s
                    .spec()
                    .domain
                    .iter()
                    .cloned()
                    .map(ScaleInput::Key)
                    .collect();
            }
            Prepared::Category(s) => {
                return s.domain().iter().cloned().map(ScaleInput::Key).collect();
            }
            Prepared::Threshold(s) => {
                return s
                    .spec()
                    .domain
                    .iter()
                    .cloned()
                    .map(ScaleInput::Key)
                    .collect();
            }
            Prepared::Time(s) => {
                return s
                    .spec()
                    .domain
                    .iter()
                    .copied()
                    .map(ScaleInput::Time)
                    .collect();
            }
        };
        numbers.iter().copied().map(ScaleInput::Number).collect()
    }
    /// Authored output knots, or sampled interpolator endpoints/ranks for interpolated families.
    pub fn range(&self) -> ChartResult<Vec<Value>> {
        Ok(match &*self.prepared {
            Prepared::Numeric(s) => s.spec().range.iter().copied().map(Value::Number).collect(),
            Prepared::Continuous { scale, .. } => scale.spec().range.clone(),
            Prepared::Interpolated(s) => return s.range(),
            Prepared::Classifier(s) => s.spec().range.clone(),
            Prepared::Ordinal(s) => s.spec().range.clone(),
            Prepared::Category(s) => vec![
                Value::number(s.range().start()),
                Value::number(s.range().end()),
            ],
            Prepared::Threshold(s) => s.spec().range.clone(),
            Prepared::Time(s) => s.spec().range.clone(),
        })
    }
    /// Share a supported family with chart color/size/opacity consumers.
    pub fn mapped(&self, training: ScaleTraining) -> ChartResult<MappedScaleSpec> {
        let function = match &self.spec {
            StandaloneScaleSpec::Numeric(s)
                if !matches!(s.family, NumericFamily::Identity | NumericFamily::Radial) =>
            {
                ScaleFunctionSpec::Continuous(ContinuousScaleSpec {
                    family: s.family.clone(),
                    domain: s.domain.clone(),
                    range: s.range.iter().copied().map(Value::Number).collect(),
                    factory: crate::interpolate::InterpolationFactory::new(if s.round {
                        crate::interpolate::FactoryKind::Round
                    } else {
                        crate::interpolate::FactoryKind::Value
                    }),
                    clamp: s.clamp,
                    unknown: s.unknown.clone(),
                })
            }
            StandaloneScaleSpec::Continuous(s) => ScaleFunctionSpec::Continuous(s.clone()),
            StandaloneScaleSpec::Interpolated(s) => ScaleFunctionSpec::Interpolated(s.clone()),
            StandaloneScaleSpec::Ordinal(s) => ScaleFunctionSpec::Ordinal(s.clone()),
            StandaloneScaleSpec::Classifier(s) => ScaleFunctionSpec::Classifier(s.clone()),
            StandaloneScaleSpec::Threshold(s) => ScaleFunctionSpec::Threshold(s.clone()),
            _ => return Err(capability("a mapped aesthetic descriptor")),
        };
        let spec = MappedScaleSpec {
            colorbar_options: None,
            palette_theme_aesthetics: vec![],
            oob_function: None,
            rescaler_function: None,
            palette_fallback_indices: vec![],
            missing_paint_is_na: false,
            palette_function: None,
            resolved_numeric_limits: None,
            resolved_discrete_limits_null: false,
            trained_transformed_bounds: None,
            limits_function: None,
            breaks_function: None,
            guide: None,
            ggplot: None,
            catalog: None,
            function,
            training,
        };
        spec.validate_training()?;
        Ok(spec)
    }
}

impl StandaloneScaleSpec {
    fn ggplot_transform(&self) -> Option<&GgplotTransform> {
        match self {
            Self::Numeric(s) => s.family.ggplot_transform(),
            Self::Continuous(s) => s.family.ggplot_transform(),
            Self::Interpolated(s) => s.normalization.ggplot_transform(),
            _ => None,
        }
    }
    fn ggplot_transform_mut(&mut self) -> Option<&mut GgplotTransform> {
        let family = match self {
            Self::Numeric(s) => &mut s.family,
            Self::Continuous(s) => &mut s.family,
            Self::Interpolated(s) => match &mut s.normalization {
                NormalizationSpec::Ggplot { family, .. }
                | NormalizationSpec::Sequential { family, .. }
                | NormalizationSpec::Diverging { family, .. } => family,
                _ => return None,
            },
            _ => return None,
        };
        match family {
            NumericFamily::Ggplot { transform } => Some(transform),
            _ => None,
        }
    }
    /// Minimum standalone scale envelope version, preserving builtin version one.
    pub fn wire_version(&self) -> u32 {
        let transform = match self {
            Self::Numeric(s) => s.family.ggplot_transform(),
            Self::Continuous(s) => s.family.ggplot_transform(),
            Self::Interpolated(s) => s.normalization.ggplot_transform(),
            _ => None,
        };
        if let Some(transform) = transform {
            return if transform.has_registered() {
                10
            } else if matches!(transform, super::GgplotTransform::Compose { .. }) {
                9
            } else {
                8
            };
        }
        if matches!(self,Self::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(i) if i.wire_version()==5))
        {
            return 7;
        }
        if matches!(self,Self::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(i) if i.wire_version()==4))
        {
            return 6;
        }
        if matches!(self, Self::Interpolated(s) if matches!(s.normalization, NormalizationSpec::Ggplot { timestamp: Some(GgplotTimestampNormalization { date: true, .. }), .. }))
        {
            return 5;
        }
        if matches!(self, Self::Interpolated(s) if matches!(s.normalization, NormalizationSpec::Ggplot { timestamp: Some(_), .. }))
        {
            return 4;
        }
        if matches!(self,Self::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(i) if i.wire_version()==3) || matches!(s.normalization,NormalizationSpec::Ggplot {..}))
        {
            return 3;
        }

        let registered = match self {
            Self::Continuous(s) => s.factory.has_registration(),
            Self::Time(s) => s.factory.has_registration(),
            Self::Interpolated(s) => {
                matches!(&s.output, ScaleRangeFunction::Interpolate(i) if i.has_registration())
            }
            _ => false,
        };
        if registered { 2 } else { 1 }
    }
    pub(crate) fn validate_registrations(
        &self,
        registry: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        match self {
            Self::Continuous(s) => s.factory.validate_registration(registry, portable),
            Self::Time(s) => s.factory.validate_registration(registry, portable),
            Self::Interpolated(s) => match &s.output {
                ScaleRangeFunction::Interpolate(i) => i.validate_registrations(registry, portable),
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}
