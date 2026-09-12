//! Shared typed aesthetic-scale preparation. All arithmetic remains in family owners.
use super::*;
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{Number, Value},
    scene::Color,
};
use serde::{Deserialize, Serialize};

/// Typed scale families available to chart aesthetics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleFunctionSpec {
    /// Raw reference category values with independently trained guides.
    GgplotDiscreteIdentity(GgplotDiscreteIdentity),
    /// Raw transformed numeric values with separate ggplot2 guide training.
    GgplotNumericIdentity(GgplotNumericIdentity),
    /// Arbitrarily spaced domain and typed range knots.
    Continuous(ContinuousScaleSpec),
    /// Sequential, diverging or empirical rank normalization and interpolation.
    Interpolated(InterpolatedScaleSpec),
    /// Sample quantile or equal-width quantize output buckets.
    Classifier(ClassifierSpec<Value>),
    /// Ordered typed cutpoints.
    Threshold(ThresholdSpec<ScaleKey, Value>),
    /// Frozen category catalog with arbitrary typed range values.
    Ordinal(OrdinalSpec<ScaleKey, Value>),
}
/// Population ownership, separate from viewport and output configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleTraining {
    /// Use the population retained in the descriptor.
    #[default]
    Authored,
    /// Train quantiles/ranks or ordinal keys from eligible post-stat observations sharing the identity.
    Eligible,
}
/// Shared mapping descriptor for color and numeric aesthetics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappedScaleSpec {
    /// Numeric callback output in source coordinates, retaining its length and missing values.
    /// Populated by eligible training; replacement training recomputes it from source data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_numeric_limits: Option<Box<Vec<Number>>>,
    /// Pure registered limit replacement after population training.
    /// It takes precedence over fixed limits and is reevaluated for replacement data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits_function: Option<Box<crate::grammar::ScaleLimitsOperation>>,
    /// Optional reference guide arguments; hidden binned guides preserve first-map limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide: Option<Box<GgplotScaleGuide>>,
    /// Optional ggplot2 population/limit policy; legacy D3 descriptors omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ggplot: Option<Box<GgplotScalePolicy>>,
    /// Named discrete range identity; its canonical values must match the range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<chromatic::SchemeSpec>,
    /// Typed range mapping.
    pub function: ScaleFunctionSpec,
    /// Quantile/rank/ordinal population source; other families use authored training.
    #[serde(default)]
    pub training: ScaleTraining,
}
impl MappedScaleSpec {
    /// Construct a scale using its explicit authored domain.
    pub fn authored(function: ScaleFunctionSpec) -> Self {
        Self {
            resolved_numeric_limits: None,
            limits_function: None,
            guide: None,
            ggplot: None,
            catalog: None,
            function,
            training: ScaleTraining::Authored,
        }
    }
    /// Select a registered limit function over the trained domain.
    pub fn with_limits_function(
        mut self,
        operation: crate::grammar::ScaleLimitsOperation,
    ) -> ChartResult<Self> {
        self.limits_function = Some(Box::new(operation));
        self.validate_training()?;
        Ok(self)
    }
    pub(crate) fn timestamp_normalization(
        &mut self,
        origin: i64,
        unit: crate::data::TimeUnit,
        date: bool,
    ) -> ChartResult<()> {
        GgplotScalePolicy::canonicalize(self)?;
        let ScaleFunctionSpec::Interpolated(scale) = &mut self.function else {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Timestamp aesthetics require reference interpolation.",
            ));
        };
        let NormalizationSpec::Ggplot { timestamp, .. } = &mut scale.normalization else {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Timestamp aesthetics require reference normalization.",
            ));
        };
        *timestamp = Some(GgplotTimestampNormalization { origin, unit, date });
        Ok(())
    }
    /// Evaluate a reference temporal palette using absolute Date/datetime arithmetic.
    /// The context survives later guide replacement and does not alter source identity.
    pub fn with_timestamp_normalization(
        mut self,
        context: GgplotTimestampNormalization,
    ) -> ChartResult<Self> {
        self.timestamp_normalization(context.origin, context.unit, context.date)?;
        self.validate_training()?;
        Ok(self)
    }
    /// Retain scale-level break/label arguments after choosing a reference scale policy.
    pub fn with_guide(mut self, guide: GgplotScaleGuide) -> ChartResult<Self> {
        self.guide = Some(Box::new(guide));
        self.validate_training()?;
        Ok(self)
    }
    pub(super) fn discrete_guide(&self) -> Option<&GgplotDiscreteGuide> {
        match self.guide.as_deref() {
            Some(GgplotScaleGuide::Discrete(g)) => Some(g),
            _ => None,
        }
    }
    pub(crate) fn reference_guides(&self) -> bool {
        self.ggplot.is_some()
            || self.guide.is_some()
            || matches!(
                self.function,
                ScaleFunctionSpec::GgplotNumericIdentity(_)
                    | ScaleFunctionSpec::GgplotDiscreteIdentity(_)
            )
            || matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization,NormalizationSpec::Ggplot {..}))
    }
    fn continuous_guide(&self) -> Option<&GgplotContinuousGuide> {
        match self.guide.as_deref() {
            Some(GgplotScaleGuide::Continuous(g)) => Some(g),
            _ => None,
        }
    }
    /// Retain a ggplot2 population policy, trained on the complete eligible rows.
    pub fn with_ggplot(mut self, policy: GgplotScalePolicy) -> ChartResult<Self> {
        self.ggplot = Some(Box::new(policy));
        self.training = ScaleTraining::Eligible;
        self.validate_training()?;
        Ok(self)
    }
    /// Replace a discrete/continuous range with an exact named palette and retain its identity.
    pub fn with_palette(mut self, palette: chromatic::SchemeSpec) -> ChartResult<Self> {
        let values = palette.values()?;
        match &mut self.function {
            ScaleFunctionSpec::Continuous(s) => s.range = values,
            ScaleFunctionSpec::Ordinal(s) => s.range = values,
            ScaleFunctionSpec::Classifier(s) => s.range = values,
            ScaleFunctionSpec::Threshold(s) => s.range = values,
            ScaleFunctionSpec::Interpolated(_)
            | ScaleFunctionSpec::GgplotNumericIdentity(_)
            | ScaleFunctionSpec::GgplotDiscreteIdentity(_) => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Use a named chromatic interpolator for sequential or diverging output.",
                ));
            }
        }
        self.catalog = Some(palette);
        MappedScale::new(self.clone())?;
        Ok(self)
    }
    fn validate_catalog(&self) -> ChartResult<()> {
        let Some(palette) = self.catalog else {
            return Ok(());
        };
        let expected = palette.values()?;
        let actual = match &self.function {
            ScaleFunctionSpec::Continuous(s) => Some(&s.range),
            ScaleFunctionSpec::Ordinal(s) => Some(&s.range),
            ScaleFunctionSpec::Classifier(s) => Some(&s.range),
            ScaleFunctionSpec::Threshold(s) => Some(&s.range),
            ScaleFunctionSpec::Interpolated(_)
            | ScaleFunctionSpec::GgplotNumericIdentity(_)
            | ScaleFunctionSpec::GgplotDiscreteIdentity(_) => None,
        };
        if actual != Some(&expected) {
            return Err(error(
                DiagnosticCode::Validation,
                "Named palette metadata and canonical scale range disagree.",
            ));
        }
        Ok(())
    }
    /// Whether this mapping requires the version-six catalog capability.
    pub fn has_chromatic(&self) -> bool {
        self.catalog.is_some()
            || self.has_ggplot()
            || matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(crate::interpolate::InterpolationSpec::Chromatic {..})))
    }
    /// Whether source fields should retain exact keys instead of numerical conversion.
    pub fn categorical(&self) -> bool {
        match &self.function {
            ScaleFunctionSpec::Ordinal(_) | ScaleFunctionSpec::GgplotDiscreteIdentity(_) => true,
            ScaleFunctionSpec::Threshold(s) => {
                s.domain.is_empty()
                    || !s
                        .domain
                        .iter()
                        .all(|key| matches!(key, ScaleKey::Number(_)))
            }
            _ => false,
        }
    }
    /// Maximum authored guide entries (rank guides sample five population quantiles).
    pub fn guide_entries(&self) -> usize {
        if self.reference_guides()
            && matches!(
                self.function,
                ScaleFunctionSpec::Continuous(_) | ScaleFunctionSpec::Interpolated(_)
            )
            && !matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
        {
            let guide = self.continuous_guide();
            return guide.and_then(|g| g.breaks.as_ref()).map_or_else(
                || guide.and_then(|g| g.count).unwrap_or(5.).ceil().max(0.) as usize,
                Vec::len,
            );
        }

        if let Some(GgplotScalePolicy::Binned(policy)) = self.ggplot.as_deref() {
            if let Some(cuts) = &policy.prepared_breaks {
                return cuts.len().saturating_add(1);
            }
            match &policy.breaks {
                GgplotBreaks::Explicit(cuts) => return cuts.len().saturating_add(1),
                GgplotBreaks::Equal(count) => {
                    return ((count + 2.).ceil() as usize).saturating_sub(1);
                }
                // Nice cuts depend on the trained population. Check their bound
                // again after training, before preparing palette values.
                GgplotBreaks::Nice(_) => {}
            }
        }
        match &self.function {
            ScaleFunctionSpec::GgplotDiscreteIdentity(s) => {
                if s.guide {
                    s.domain().map_or(usize::MAX, |v| v.len())
                } else {
                    0
                }
            }
            ScaleFunctionSpec::GgplotNumericIdentity(_) => 0,
            ScaleFunctionSpec::Continuous(s) => s.domain.len(),
            ScaleFunctionSpec::Interpolated(s) => match s.normalization {
                NormalizationSpec::Sequential { .. } | NormalizationSpec::Ggplot { .. } => 2,
                NormalizationSpec::Diverging { .. } => 3,
                NormalizationSpec::Quantile { .. } => 5,
            },
            ScaleFunctionSpec::Classifier(s) => s.range.len(),
            ScaleFunctionSpec::Threshold(s) => s.range.len(),
            ScaleFunctionSpec::Ordinal(s) => s.domain.len(),
        }
    }
    /// Return a new descriptor trained from the complete eligible numeric population.
    pub fn trained(&self, values: &[Option<Number>]) -> ChartResult<Self> {
        self.trained_with_registry(values, &crate::grammar::ExtensionRegistry::new())
    }
    /// Train numeric limits with explicitly installed pure domain functions.
    pub fn trained_with_registry(
        &self,
        values: &[Option<Number>],
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        if self.limits_function.is_some() {
            return super::ggplot_numeric_limits::train(self, values, registry);
        }
        let mut next = self.clone();
        if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &mut next.function {
            if self.training == ScaleTraining::Eligible {
                identity.train(values)?;
            }
            return Ok(next);
        }
        if let Some(policy) = &self.ggplot {
            policy.train_numbers(&mut next, values)?;
            return Ok(next);
        }
        if self.training == ScaleTraining::Eligible {
            match &mut next.function {
                ScaleFunctionSpec::Classifier(ClassifierSpec {
                    domain: ClassifierDomain::Quantile(samples),
                    ..
                }) => *samples = values.to_vec(),
                ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                    normalization: NormalizationSpec::Quantile { samples },
                    ..
                }) => *samples = values.to_vec(),
                _ => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Eligible population training is only defined for quantile and empirical rank scales.",
                    ));
                }
            }
        }
        Ok(next)
    }
    /// Extend an eligible ordinal catalog in first-seen order, starting from authored keys.
    /// Explicit ordinal unknown policies do not add new keys.
    pub fn trained_keys(&self, keys: &[ScaleKey]) -> ChartResult<Self> {
        self.trained_keys_with_registry(keys, &crate::grammar::ExtensionRegistry::new())
    }
    /// Train keys with explicitly installed pure domain functions.
    pub fn trained_keys_with_registry(
        &self,
        keys: &[ScaleKey],
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let mut next = self.clone();
        if let Some(call) = &self.limits_function {
            self.validate_training()?;
            let (levels, drop, na_translate) = match (&self.function, self.ggplot.as_deref()) {
                (
                    _,
                    Some(GgplotScalePolicy::Discrete {
                        levels,
                        drop,
                        na_translate,
                        ..
                    }),
                ) => (levels.as_deref(), *drop, *na_translate),
                (ScaleFunctionSpec::GgplotDiscreteIdentity(identity), _) => (
                    identity.levels.as_deref(),
                    identity.drop,
                    identity.na_translate,
                ),
                _ => {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Discrete limit training requires a categorical population.",
                    ));
                }
            };
            // Identity scales without a guide do not train a reference domain.
            // Their raw observed keys are retained separately for value pooling.
            let callback_keys = if matches!(&self.function,
                ScaleFunctionSpec::GgplotDiscreteIdentity(identity) if !identity.guide)
            {
                &[][..]
            } else {
                keys
            };
            let domain =
                super::ggplot::discrete_domain(callback_keys, None, levels, drop, na_translate);
            let resolved = registry.limits_function.evaluate(
                call,
                (!callback_keys.is_empty()).then_some(domain.as_slice()),
                None,
            )?;
            if let Some(GgplotScalePolicy::Discrete { limits, .. }) = next.ggplot.as_deref_mut() {
                *limits = Some(resolved.ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "A discrete palette requires a vector limit result, not NULL.",
                    )
                })?);
            } else if let ScaleFunctionSpec::GgplotDiscreteIdentity(identity) = &mut next.function {
                identity.limits = Some(resolved.unwrap_or_default());
            }
        }
        if let ScaleFunctionSpec::GgplotDiscreteIdentity(identity) = &mut next.function {
            if self.training == ScaleTraining::Eligible {
                identity.train(keys)?;
            }
            return Ok(next);
        }
        if let Some(policy) = next.ggplot.clone() {
            policy.train_keys(&mut next, keys)?;
            return Ok(next);
        }
        if self.training == ScaleTraining::Eligible {
            let ScaleFunctionSpec::Ordinal(spec) = &mut next.function else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Typed key populations require an ordinal scale.",
                ));
            };
            *spec = OrdinalScale::new(spec.clone())
                .train(
                    keys.iter()
                        .filter(|key| !self.has_chromatic() || finite_key(key))
                        .cloned(),
                )
                .spec()
                .clone();
        }
        Ok(next)
    }
    pub(crate) fn validate_definition_with_registry(
        &self,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<()> {
        if let Some(call) = &self.limits_function {
            registry.limits_function.validate(call, false)?;
        }
        let mut spec = self.clone();
        GgplotScalePolicy::canonicalize(&mut spec)?;
        spec.validate_training()?;
        spec.validate_catalog()?;
        if let Some(GgplotScalePolicy::Binned(policy)) = spec.ggplot.as_deref() {
            policy.validate_break_budget(4096)?;
            let ScaleFunctionSpec::Interpolated(function) = &spec.function else {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Binned scales require an interpolated range.",
                ));
            };
            // The named scale has no post-stat population during authoring. Compile
            // its normalizer and range without selecting cuts on a placeholder domain.
            InterpolatedScale::new_with_registry(function.clone(), registry)?;
            return Ok(());
        }
        MappedScale::new_with_registry(spec, registry).map(|_| ())
    }
    pub(crate) fn validate_training(&self) -> ChartResult<()> {
        if let Some(values) = &self.resolved_numeric_limits {
            if self.limits_function.is_none() || self.categorical() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Resolved numeric limits require a numeric limit function.",
                ));
            }
            crate::limits::require_within(
                values.len() <= crate::interpolate::MAX_VALUES,
                "numeric function limits",
            )?;
        }
        if self.limits_function.is_some()
            && (self.training != ScaleTraining::Eligible
                || !matches!(
                    self.ggplot.as_deref(),
                    Some(
                        GgplotScalePolicy::Discrete { .. }
                            | GgplotScalePolicy::Continuous { .. }
                            | GgplotScalePolicy::Binned(_)
                    )
                ) && !matches!(
                    self.function,
                    ScaleFunctionSpec::GgplotDiscreteIdentity(_)
                        | ScaleFunctionSpec::GgplotNumericIdentity(_)
                ))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Registered limits require an eligible reference scale.",
            ));
        }
        if let Some(guide) = &self.guide {
            guide.validate()?;
            let matching = match guide.as_ref() {
                GgplotScaleGuide::Hidden => true,
                GgplotScaleGuide::Discrete(_) => self.categorical(),
                GgplotScaleGuide::Binned(_) => {
                    matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
                }
                GgplotScaleGuide::Continuous(_) | GgplotScaleGuide::Temporal(_) => {
                    matches!(
                        self.function,
                        ScaleFunctionSpec::Continuous(_)
                            | ScaleFunctionSpec::Interpolated(_)
                            | ScaleFunctionSpec::GgplotNumericIdentity(_)
                    ) && !matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
                        && !matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization,NormalizationSpec::Quantile {..}))
                }
            };
            if let GgplotScaleGuide::Binned(labels) = guide.as_ref()
                && let Some(GgplotScalePolicy::Binned(policy)) = self.ggplot.as_deref()
                && let GgplotBreaks::Explicit(breaks) = &policy.breaks
            {
                labels.validate(Some(breaks.len()))?;
            }
            if !matching || !self.has_ggplot() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Reference guide arguments and scale family disagree.",
                ));
            }
        }
        if let ScaleFunctionSpec::GgplotDiscreteIdentity(identity) = &self.function {
            if self.ggplot.is_some() || self.catalog.is_some() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Identity mapping cannot carry a rescaling policy or palette.",
                ));
            }
            return identity.validate();
        }
        if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &self.function {
            if self.ggplot.is_some() || self.catalog.is_some() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Identity mapping cannot carry a rescaling policy or palette.",
                ));
            }
            return identity.validate();
        }
        if let Some(policy) = &self.ggplot {
            if self.training != ScaleTraining::Eligible {
                return Err(error(
                    DiagnosticCode::Validation,
                    "ggplot2 policy requires eligible population training.",
                ));
            }
            let mut empty = self.clone();
            match policy.as_ref() {
                GgplotScalePolicy::Continuous { .. } | GgplotScalePolicy::Binned(_) => {
                    // Structural validation must not execute a break search on
                    // a fabricated empty population. A count can be valid for
                    // the actual constant domain and invalid for that fallback.
                    policy.validate_numbers(&mut empty)?;
                }
                GgplotScalePolicy::Discrete { .. } => policy.train_keys(&mut empty, &[])?,
            }
        }

        if self.ggplot.is_none()
            && self.training == ScaleTraining::Eligible
            && !matches!(
                self.function,
                ScaleFunctionSpec::Ordinal(_)
                    | ScaleFunctionSpec::Classifier(ClassifierSpec {
                        domain: ClassifierDomain::Quantile(_),
                        ..
                    })
                    | ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                        normalization: NormalizationSpec::Quantile { .. },
                        ..
                    })
            )
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Eligible training requires a quantile, empirical rank or ordinal scale.",
            ));
        }
        Ok(())
    }
    pub(crate) fn trained_population(
        &self,
        population: Option<&ScalePopulation>,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        if self.training == ScaleTraining::Authored {
            return Ok(self.clone());
        }
        match (&self.function, population) {
            (
                ScaleFunctionSpec::Ordinal(_) | ScaleFunctionSpec::GgplotDiscreteIdentity(_),
                Some(ScalePopulation::Keys(keys)),
            ) => self.trained_keys_with_registry(keys, registry),
            (
                ScaleFunctionSpec::Ordinal(_) | ScaleFunctionSpec::GgplotDiscreteIdentity(_),
                None,
            ) => self.trained_keys_with_registry(&[], registry),
            (_, Some(ScalePopulation::Numbers(values))) => {
                self.trained_with_registry(values, registry)
            }
            (_, None) => self.trained_with_registry(&[], registry),
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Shared scale population kinds disagree.",
            )),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum ScalePopulation {
    Numbers(Vec<Option<Number>>),
    Keys(Vec<ScaleKey>),
}
#[derive(Clone, Debug)]
enum PreparedMapping {
    GgplotDiscreteIdentity(OrdinalScale<ScaleKey, usize>),
    GgplotNumericIdentity(GgplotNumericIdentity),
    Binned(super::ggplot_bins::GgplotBinnedMapping),
    Continuous(ContinuousScale),
    Interpolated(InterpolatedScale),
    Classifier(ClassifierScale<usize>),
    Threshold(ThresholdScale<ScaleKey, usize>),
    Ordinal(OrdinalScale<ScaleKey, usize>),
}
/// Compiled aesthetic mapping; discrete outputs and their paint conversions are pooled once.
#[derive(Clone, Debug)]
pub struct MappedScale {
    spec: MappedScaleSpec,
    mapping: PreparedMapping,
    values: Vec<Value>,
    paints: Option<Vec<Option<crate::color::Paint>>>,
}
impl MappedScale {
    /// Compile an authored or already trained descriptor.
    pub fn new(spec: MappedScaleSpec) -> ChartResult<Self> {
        Self::new_with_registry(spec, &crate::grammar::ExtensionRegistry::new())
    }
    /// Compile against an explicit registry; no callbacks are resolved per mark.
    pub fn new_with_registry(
        mut spec: MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        GgplotScalePolicy::canonicalize(&mut spec)?;
        spec.validate_interpolations(&registry.interpolations, false)?;
        // Validate population policy without discarding an already trained population.
        if let Some(call) = &spec.limits_function {
            registry.limits_function.validate(call, false)?;
        }
        spec.validate_training()?;
        spec.validate_catalog()?;
        let mut values = vec![];
        let mut pool = |value: &Value| -> ChartResult<usize> {
            value.validate()?;
            let i = values.len();
            values.push(value.clone());
            Ok(i)
        };
        let mut mapping = if let Some(GgplotScalePolicy::Binned(policy)) = spec.ggplot.as_deref() {
            PreparedMapping::Binned(super::ggplot_bins::GgplotBinnedMapping::new(
                policy,
                &spec.function,
                spec.resolved_numeric_limits.as_deref().map(Vec::as_slice),
                registry,
                &mut pool,
            )?)
        } else {
            match &spec.function {
                ScaleFunctionSpec::GgplotDiscreteIdentity(s) => {
                    let mut keys = s.observed.clone();
                    if s.guide {
                        keys.extend(s.domain()?);
                    }
                    keys.push(ScaleKey::Null);
                    let keys: Vec<_> = keys
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    let range = keys
                        .iter()
                        .map(|k| pool(&GgplotDiscreteIdentity::map(Some(k))?))
                        .collect::<ChartResult<_>>()?;
                    PreparedMapping::GgplotDiscreteIdentity(OrdinalScale::new(OrdinalSpec {
                        domain: keys,
                        range,
                        unknown: OrdinalUnknown::Explicit(None),
                    }))
                }
                ScaleFunctionSpec::GgplotNumericIdentity(s) => {
                    PreparedMapping::GgplotNumericIdentity(s.clone())
                }
                ScaleFunctionSpec::Continuous(s) => PreparedMapping::Continuous(
                    ContinuousScale::new_with_registry(s.clone(), registry)?,
                ),
                ScaleFunctionSpec::Interpolated(s) => PreparedMapping::Interpolated(
                    InterpolatedScale::new_with_registry(s.clone(), registry)?,
                ),
                ScaleFunctionSpec::Classifier(s) => {
                    PreparedMapping::Classifier(ClassifierScale::new(ClassifierSpec {
                        domain: s.domain.clone(),
                        range: s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?,
                        unknown: s.unknown.as_ref().map(&mut pool).transpose()?,
                    })?)
                }
                ScaleFunctionSpec::Threshold(s) => {
                    PreparedMapping::Threshold(ThresholdScale::new(ThresholdSpec {
                        domain: s.domain.clone(),
                        range: s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?,
                        unknown: s.unknown.as_ref().map(&mut pool).transpose()?,
                    })?)
                }
                ScaleFunctionSpec::Ordinal(s) => {
                    let range = s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?;
                    let unknown = match &s.unknown {
                        OrdinalUnknown::Implicit => OrdinalUnknown::Implicit,
                        OrdinalUnknown::Explicit(v) => {
                            OrdinalUnknown::Explicit(v.as_ref().map(&mut pool).transpose()?)
                        }
                    };
                    PreparedMapping::Ordinal(OrdinalScale::new(OrdinalSpec {
                        domain: s.domain.clone(),
                        range,
                        unknown,
                    }))
                }
            }
        };
        if spec
            .resolved_numeric_limits
            .as_ref()
            .is_some_and(|v| v.len() == 1)
            && let PreparedMapping::Interpolated(scale) = &mut mapping
            && matches!(
                scale.spec().normalization,
                NormalizationSpec::Ggplot {
                    rescaler: GgplotRescaler::Range | GgplotRescaler::Midpoint(_),
                    ..
                }
            )
        {
            // R zero_range() is true for every singleton, including NA/Inf.
            scale.set_reference_bounds([0., 0.]);
        }
        if let (Some(policy), PreparedMapping::Interpolated(scale)) =
            (spec.ggplot.as_deref(), &mut mapping)
            && let NormalizationSpec::Ggplot {
                family, reverse, ..
            } = scale.spec().normalization
            && let Some(bounds) = policy.population_bounds(family, reverse)
        {
            scale.set_reference_bounds(bounds);
        }
        Ok(Self {
            spec,
            mapping,
            values,
            paints: None,
        })
    }
    /// Compile and validate color outputs; category colors are parsed/quantized once.
    pub fn for_colors(spec: MappedScaleSpec) -> ChartResult<Self> {
        Self::for_colors_with_registry(spec, &crate::grammar::ExtensionRegistry::new())
    }
    /// Compile floating colors against explicitly registered interpolation factories.
    pub fn for_colors_with_registry(
        spec: MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let mut scale = Self::new_with_registry(spec, registry)?;
        scale.validate_prepared_sampling()?;
        scale.paints = Some(
            scale
                .values
                .iter()
                .map(|value| {
                    if matches!(scale.mapping, PreparedMapping::GgplotDiscreteIdentity(_))
                        && matches!(value, Value::Missing)
                    {
                        return crate::color::parse_r("NA").map(Some);
                    }
                    reference_value_paint(value, scale.spec.has_ggplot())
                })
                .collect::<ChartResult<_>>()?,
        );
        match &scale.mapping {
            PreparedMapping::GgplotNumericIdentity(_) => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Numeric identity scales require numeric aesthetic outputs.",
                ));
            }
            PreparedMapping::Continuous(s) => {
                scale.paints = Some(vec![value_paint(&s.spec().unknown)?]);
                s.validate_color_output()?;
            }
            PreparedMapping::Interpolated(s) => {
                scale.paints = Some(vec![value_paint(&s.spec().unknown)?]);
                s.validate_color_output()?;
            }
            _ => {}
        }
        Ok(scale)
    }
    /// Compile and validate numeric outputs even when the current layer has no observations.
    pub fn for_numbers(spec: MappedScaleSpec) -> ChartResult<Self> {
        Self::for_numbers_with_registry(spec, &crate::grammar::ExtensionRegistry::new())
    }
    /// Compile numeric outputs against explicitly registered interpolation factories.
    pub fn for_numbers_with_registry(
        spec: MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let scale = Self::new_with_registry(spec, registry)?;
        scale.validate_prepared_sampling()?;
        for value in &scale.values {
            numeric_output(value)?;
        }
        match &scale.mapping {
            PreparedMapping::Continuous(s) => {
                numeric_output(&s.spec().unknown)?;
                s.validate_numeric_output()?;
            }
            PreparedMapping::Interpolated(s) => {
                numeric_output(&s.spec().unknown)?;
                s.validate_numeric_output()?;
            }
            _ => {}
        }
        Ok(scale)
    }
    /// Retained descriptor with the exact prepared population.
    pub fn spec(&self) -> &MappedScaleSpec {
        &self.spec
    }
    pub(crate) fn validate_prepared_sampling(&self) -> ChartResult<()> {
        // Empty layers with no guide never invoke R's binned map. Keep a compiled
        // descriptor, but actual numeric/color sampling still checks validity.
        if matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden))
            && matches!(self.spec.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(p)) if p.empty_population)
        {
            return Ok(());
        }
        if matches!(
            self.spec.guide.as_deref(),
            Some(GgplotScaleGuide::Temporal(_))
        ) {
            self.continuous_guide_entries(crate::interpolate::MAX_VALUES, 1_048_576)?;
        }
        self.validate_sampling()
    }
    fn validate_sampling(&self) -> ChartResult<()> {
        if let Some(values) = &self.spec.resolved_numeric_limits
            && !matches!(self.mapping, PreparedMapping::GgplotNumericIdentity(_))
            && !super::ggplot_numeric_limits::maximum(&self.spec)
        {
            super::ggplot_numeric_limits::validate_result(values)?;
        }
        if let PreparedMapping::Binned(mapping) = &self.mapping {
            mapping.validate_sampling()?;
        }
        Ok(())
    }
    fn index(&self, numeric: Option<f64>, key: Option<&ScaleKey>) -> Option<usize> {
        match &self.mapping {
            PreparedMapping::Binned(s) => s.index(numeric),
            PreparedMapping::Classifier(s) => s.map(numeric).copied(),
            PreparedMapping::Threshold(s) => s.map_key(key).copied(),
            PreparedMapping::Ordinal(s) | PreparedMapping::GgplotDiscreteIdentity(s) => key
                .map_or_else(
                    || match s.spec().unknown {
                        OrdinalUnknown::Explicit(Some(i)) => Some(i),
                        _ => None,
                    },
                    |k| {
                        s.map(k).copied().or_else(|| {
                            if matches!(
                                self.spec.ggplot.as_deref(),
                                Some(GgplotScalePolicy::Discrete { .. })
                            ) {
                                discrete_text_key(k)
                                    .and_then(|text| s.map(&ScaleKey::Text(text)).copied())
                            } else {
                                None
                            }
                        })
                    },
                ),
            _ => None,
        }
    }
    fn untrained_discrete_value(&self, key: Option<&ScaleKey>) -> ChartResult<Option<Value>> {
        if !matches!(&self.spec.function, ScaleFunctionSpec::Ordinal(s) if s.domain.is_empty()) {
            return Ok(None);
        }
        let Some(values) = self
            .spec
            .ggplot
            .as_deref()
            .and_then(|policy| policy.untrained_discrete_values(self.spec.guide.as_deref()))
        else {
            return Ok(None);
        };
        let values = values?;
        let index = match key {
            Some(ScaleKey::Text(text)) => match text.as_str() {
                "0" => Some(0),
                "1" => Some(1),
                _ => None,
            },
            Some(key) => discrete_text_key(key).and_then(|text| match text.as_str() {
                "0" => Some(0),
                "1" => Some(1),
                _ => None,
            }),
            None => None,
        };
        Ok(Some(
            index.map_or(Value::Missing, |index| values[index].clone()),
        ))
    }
    /// Owned numerical or typed output with reference-specific discrete key matching.
    pub fn numeric(&self, input: Option<f64>) -> ChartResult<Value> {
        self.validate_sampling()?;
        let input = if let Some(values) = &self.spec.resolved_numeric_limits {
            super::ggplot_numeric_limits::input(&self.spec, values, input)
        } else {
            self.spec
                .ggplot
                .as_ref()
                .map_or(input, |p| p.input(input, &self.spec.function))
        };
        match &self.mapping {
            PreparedMapping::GgplotNumericIdentity(s) => s.map(input),
            PreparedMapping::GgplotDiscreteIdentity(_) => {
                GgplotDiscreteIdentity::map(input.map(|v| ScaleKey::Number(Number(v))).as_ref())
            }
            PreparedMapping::Continuous(s) => s.map(input),
            PreparedMapping::Interpolated(s) => s.map(input),
            _ => {
                let key = input
                    .filter(|x| !x.is_nan())
                    .map(|v| ScaleKey::Number(Number(v)));
                if let Some(value) = self.untrained_discrete_value(key.as_ref())? {
                    return Ok(value);
                }
                Ok(self
                    .index(input, key.as_ref())
                    .map_or(Value::Missing, |i| self.values[i].clone()))
            }
        }
    }
    /// Category/threshold output; ggplot numeric/logical queries can match text levels.
    pub fn category(&self, key: Option<&ScaleKey>) -> ChartResult<Value> {
        self.validate_sampling()?;
        if matches!(self.mapping, PreparedMapping::GgplotDiscreteIdentity(_)) {
            return GgplotDiscreteIdentity::map(key);
        }
        if let Some(value) = self.untrained_discrete_value(key)? {
            return Ok(value);
        }
        Ok(self
            .index(None, key)
            .map_or(Value::Missing, |i| self.values[i].clone()))
    }
    fn suppresses_discrete_missing(&self) -> bool {
        let Some(GgplotScalePolicy::Discrete {
            na_translate: false,
            empty_population,
            limits,
            palette,
            ..
        }) = self.spec.ggplot.as_deref()
        else {
            return false;
        };
        let ScaleFunctionSpec::Ordinal(spec) = &self.spec.function else {
            return false;
        };
        spec.domain.iter().any(|key| *key != ScaleKey::Null)
            // An untrained ordinary scale has reference fallback limits [0,1].
            // Named manual palettes instead install an implicit limits function.
            || (*empty_population && spec.domain.is_empty() && limits.is_none()
                && !matches!(palette, GgplotDiscretePalette::Manual { names: Some(_), .. }))
    }
    pub(super) fn missing_paint(&self, input: Option<f64>, key: Option<&ScaleKey>) -> bool {
        // Identity preserves raw missingness independently of its guide controls.
        // The literal R colour "NA" is transparent paint, not a missing key.
        if matches!(self.mapping, PreparedMapping::GgplotDiscreteIdentity(_)) {
            return key.is_none_or(|key| *key == ScaleKey::Null);
        }
        // Reference mapping returns na.value immediately for an empty palette,
        // before applying na.translate to unmatched inputs.
        if matches!(
            self.spec.ggplot.as_deref(),
            Some(GgplotScalePolicy::Discrete {
                na_translate: false,
                ..
            })
        ) && !self.suppresses_discrete_missing()
        {
            return false;
        }

        let number = input
            .filter(|v| !v.is_nan())
            .map(|v| ScaleKey::Number(Number(v)));
        self.index(input, key.or(number.as_ref()))
            .and_then(|i| self.paints.as_ref().and_then(|p| p[i]))
            .is_none()
    }
    /// Lower a sampled paint at the scene boundary.
    pub fn color(
        &self,
        input: Option<f64>,
        key: Option<&ScaleKey>,
        missing: Color,
    ) -> ChartResult<Color> {
        self.paint(input, key, missing.into())
            .map(crate::color::Paint::resolve)
    }
    /// Prepared paint sampling; no per-row CSS parsing or range-factory construction.
    pub fn paint(
        &self,
        input: Option<f64>,
        key: Option<&ScaleKey>,
        missing: crate::color::Paint,
    ) -> ChartResult<crate::color::Paint> {
        if self.spec.resolved_numeric_limits.is_some() {
            self.validate_sampling()?;
        }
        let input = if let Some(values) = &self.spec.resolved_numeric_limits {
            super::ggplot_numeric_limits::input(&self.spec, values, input)
        } else {
            self.spec
                .ggplot
                .as_ref()
                .map_or(input, |p| p.input(input, &self.spec.function))
        };
        // Named chart palettes use the declared missing color for all non-finite
        // observations; direct ramp calls keep their explicit rejection contract.
        let input = if self.spec.has_chromatic() && !self.spec.has_ggplot() {
            input.filter(|value| value.is_finite())
        } else {
            input
        };
        let key = key.filter(|key| !self.spec.has_chromatic() || finite_key(key));
        let paints = self.paints.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::Validation,
                "Prepare this mapped scale for color outputs before paint sampling.",
            )
        })?;
        let missing = if self.suppresses_discrete_missing() {
            crate::scene::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            }
            .into()
        } else {
            missing
        };
        let numeric_key = input
            .filter(|value| !value.is_nan())
            .map(|value| ScaleKey::Number(Number(value)));
        if let Some(value) = self.untrained_discrete_value(key.or(numeric_key.as_ref()))? {
            return Ok(reference_value_paint(&value, true)?.unwrap_or(missing));
        }
        Ok(match &self.mapping {
            PreparedMapping::GgplotDiscreteIdentity(_) => {
                let key = key.unwrap_or(&ScaleKey::Null);
                if let Some(index) = self.index(None, Some(key)) {
                    paints[index].unwrap_or(missing)
                } else {
                    reference_value_paint(&GgplotDiscreteIdentity::map(Some(key))?, true)?
                        .unwrap_or(crate::color::parse_r("NA")?)
                }
            }
            PreparedMapping::Continuous(s) => s
                .map_color(input)?
                .map_or(paints.first().copied().flatten().unwrap_or(missing), |v| {
                    v.into()
                }),
            PreparedMapping::Interpolated(s) => s
                .map_color(input)?
                .map_or(paints.first().copied().flatten().unwrap_or(missing), |v| {
                    v.into()
                }),
            _ => {
                let numeric_key = input
                    .filter(|x| !x.is_nan())
                    .map(|v| ScaleKey::Number(Number(v)));
                self.index(input, key.or(numeric_key.as_ref()))
                    .and_then(|i| paints[i])
                    .unwrap_or(missing)
            }
        })
    }
    /// Binned scale cuts and vector-wide labels before color-step composition.
    /// Cuts keep their authored order and duplicates; the bin mapping owns selection.
    pub fn binned_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        let Some(GgplotScalePolicy::Binned(policy)) = self.spec.ggplot.as_deref() else {
            return Ok(None);
        };
        if matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden)) {
            return Ok(Some(vec![]));
        }
        let ScaleFunctionSpec::Interpolated(s) = &self.spec.function else {
            unreachable!("validated binned mapping")
        };
        let NormalizationSpec::Ggplot {
            family,
            domain,
            reverse,
            ..
        } = s.normalization
        else {
            unreachable!("validated binned normalization")
        };
        let labels = match self.spec.guide.as_deref() {
            Some(GgplotScaleGuide::Binned(labels)) => labels,
            _ => &GgplotGuideLabels::Automatic,
        };
        if !super::ggplot_continuous_guide::comparable_limits(policy.limits, family)
            .is_some_and(|limits| limits.iter().all(Option::is_some))
        {
            if policy.empty_population {
                if policy.limits.is_some() {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Empty binned populations cannot replace missing authored limits.",
                    ));
                }
                return Ok(Some(vec![]));
            }
            if policy.nonfinite_population {
                return policy
                    .nonfinite_guide_entries(family, reverse, labels, budget, label_budget)
                    .map(Some);
            }
        }
        let (domain, cuts) = if let Some(values) = &self.spec.resolved_numeric_limits {
            policy.resolve(
                [
                    *values.first().unwrap_or(&Number(f64::NAN)),
                    *values.get(1).unwrap_or(&Number(f64::NAN)),
                ],
                family,
                reverse,
            )?
        } else if let Some(cuts) = &policy.prepared_breaks {
            (domain, cuts.clone())
        } else {
            policy.resolve(domain, family, reverse)?
        };
        if cuts.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Binned guide exceeds the tick budget.",
            ));
        }
        let forward = |v: Number| super::ggplot_continuous_guide::forward(family, reverse, v.0);
        let bounds = domain.map(forward);
        super::ggplot_continuous_guide::numeric_guide_entries(
            cuts.into_iter().map(forward).collect(),
            bounds,
            family,
            reverse,
            labels,
            label_budget,
        )
        .map(Some)
    }
    /// Reference continuous guide candidates from this exact prepared scale.
    /// Outside candidates remain available because they affect vector-wide labels.
    pub fn continuous_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if !self.spec.reference_guides() {
            return Ok(None);
        }
        if matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden)) {
            return Ok(Some(vec![]));
        }
        let (mut domain, family, reverse) = match &self.mapping {
            PreparedMapping::Continuous(s) => {
                let domain = &s.spec().domain;
                if domain.is_empty() {
                    return Ok(Some(vec![]));
                }
                (
                    [domain[0], domain[domain.len() - 1]],
                    s.spec().family,
                    false,
                )
            }
            PreparedMapping::Interpolated(s) => match s.spec().normalization {
                NormalizationSpec::Sequential { family, domain, .. } => (domain, family, false),
                NormalizationSpec::Ggplot {
                    family,
                    domain,
                    reverse,
                    ..
                } => (domain, family, reverse),
                NormalizationSpec::Diverging { family, domain, .. } => {
                    ([domain[0], domain[2]], family, false)
                }
                NormalizationSpec::Quantile { .. } => return Ok(None),
            },
            PreparedMapping::GgplotNumericIdentity(s) => {
                if !s.guide {
                    return Ok(Some(vec![]));
                }
                let domain = s
                    .domain()?
                    .map(|v| Number(s.transform.map_or(v.0, |t| t.inverse_raw(v.0))));
                let family = match s.transform {
                    Some(ScaleTransform::Log { base }) => NumericFamily::Log { base },
                    Some(ScaleTransform::Sqrt) => NumericFamily::Pow { exponent: 0.5 },
                    Some(ScaleTransform::Symlog { threshold }) => NumericFamily::Symlog {
                        constant: threshold,
                    },
                    _ => NumericFamily::Linear,
                };
                (domain, family, s.transform == Some(ScaleTransform::Reverse))
            }
            _ => return Ok(None),
        };
        if let Some(values) = &self.spec.resolved_numeric_limits {
            super::ggplot_numeric_limits::validate_result(values)?;
            domain = [values[0], *values.get(1).unwrap_or(&values[0])];
            if let Some(GgplotScaleGuide::Temporal(guide)) = self.spec.guide.as_deref() {
                return guide
                    .resolve(domain.map(|v| v.0), budget, label_budget)
                    .map(Some);
            }
            let default_guide = GgplotContinuousGuide::default();
            let guide = self.spec.continuous_guide().unwrap_or(&default_guide);
            if values.len() == 1 {
                crate::limits::require_within(budget > 0, "continuous guide")?;
                let value = super::ggplot_continuous_guide::forward(family, reverse, values[0].0);
                return super::ggplot_continuous_guide::numeric_guide_entries(
                    vec![value],
                    [value; 2],
                    family,
                    reverse,
                    &guide.labels,
                    label_budget,
                )
                .map(Some);
            }
            return guide
                .resolve_prepared(domain, family, reverse, budget, label_budget)
                .map(Some);
        }
        let empty_limits = match self.spec.ggplot.as_deref() {
            Some(GgplotScalePolicy::Continuous {
                empty_population: true,
                limits,
                ..
            }) => Some(*limits),
            _ => match &self.mapping {
                PreparedMapping::GgplotNumericIdentity(s)
                    if !s.has_population && s.trained.is_none() =>
                {
                    Some(s.limits)
                }
                _ => None,
            },
        };
        if let Some(limits) = empty_limits
            && !super::ggplot_continuous_guide::finite_limits(limits, family, reverse)
        {
            return Ok(Some(vec![]));
        }
        let nonfinite_limits = match self.spec.ggplot.as_deref() {
            Some(GgplotScalePolicy::Continuous {
                nonfinite_population: true,
                limits,
                ..
            }) => Some(*limits),
            _ => match &self.mapping {
                PreparedMapping::GgplotNumericIdentity(s)
                    if s.has_population && s.trained.is_none() =>
                {
                    Some(s.limits)
                }
                _ => None,
            },
        };
        if let Some(GgplotScaleGuide::Temporal(guide)) = self.spec.guide.as_deref() {
            if family != NumericFamily::Linear || reverse {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Temporal guides require a linear timestamp mapping.",
                ));
            }
            let bounds = nonfinite_limits.map_or(domain.map(|v| v.0), |limits| {
                super::ggplot_continuous_guide::authored_bounds(
                    limits,
                    family,
                    false,
                    [f64::INFINITY, f64::NEG_INFINITY],
                )
            });
            return guide.resolve(bounds, budget, label_budget).map(Some);
        }
        let default_guide = GgplotContinuousGuide::default();
        let guide = self.spec.continuous_guide().unwrap_or(&default_guide);
        if let Some(limits) = nonfinite_limits {
            return guide
                .resolve_nonfinite(limits, family, reverse, budget, label_budget)
                .map(Some);
        }
        guide
            .resolve_prepared(domain, family, reverse, budget, label_budget)
            .map(Some)
    }
    fn continuous_entries(
        &self,
        candidates: &[GgplotContinuousGuideEntry],
        missing: Color,
        entries: &mut Vec<(String, Color)>,
    ) -> ChartResult<()> {
        for entry in candidates.iter().filter(|entry| entry.visible) {
            let label = entry.label.clone().unwrap_or_else(|| {
                if matches!(
                    self.spec.continuous_guide().map(|g| &g.labels),
                    Some(GgplotGuideLabels::Hidden)
                ) {
                    String::new()
                } else {
                    "NA".into()
                }
            });
            entries.push((label, self.color(Some(entry.value.0), None, missing)?));
        }
        Ok(())
    }
    fn discrete_entries(
        &self,
        domain: &[ScaleKey],
        missing: Color,
        entries: &mut Vec<(String, Color)>,
    ) -> ChartResult<()> {
        if self.spec.reference_guides() {
            for entry in self
                .spec
                .discrete_guide()
                .unwrap_or(&GgplotDiscreteGuide::default())
                .resolve(domain)?
            {
                let label = entry.label.unwrap_or_else(|| {
                    if matches!(
                        self.spec.discrete_guide().map(|g| &g.labels),
                        Some(GgplotGuideLabels::Hidden)
                    ) {
                        String::new()
                    } else {
                        "NA".into()
                    }
                });
                entries.push((label, self.color(None, Some(&entry.key), missing)?));
            }
        } else {
            for key in domain {
                entries.push((key_label(key), self.color(None, Some(key), missing)?));
            }
        }
        Ok(())
    }
    /// Guide entries and interval metadata from the same prepared mapping as marks.
    pub fn legend(&self, id: crate::ScaleId, missing: Color) -> ChartResult<ColorLegend> {
        let mut entries = vec![];
        let mut intervals = vec![];
        let mut midpoint = None;
        let numeric_breaks = if let Some(candidates) =
            self.binned_guide_entries(crate::interpolate::MAX_VALUES, 1_048_576)?
        {
            candidates
        } else {
            self.continuous_guide_entries(crate::interpolate::MAX_VALUES, 1_048_576)?
                .unwrap_or_default()
        };
        let continuous = matches!(
            self.mapping,
            PreparedMapping::Continuous(_) | PreparedMapping::Interpolated(_)
        );
        if !matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden)) {
            match &self.mapping {
                PreparedMapping::GgplotNumericIdentity(_) => {}
                PreparedMapping::GgplotDiscreteIdentity(_) => {
                    if let ScaleFunctionSpec::GgplotDiscreteIdentity(s) = &self.spec.function
                        && s.guide
                    {
                        self.discrete_entries(&s.domain()?, missing, &mut entries)?;
                    }
                }
                PreparedMapping::Binned(_) if numeric_breaks.is_empty() => {}
                PreparedMapping::Binned(s) => {
                    for (i, index) in s.range().iter().enumerate() {
                        let a = s.bounds[i.min(s.bounds.len() - 1)].0;
                        let b = s.bounds[(i + 1).min(s.bounds.len() - 1)].0;
                        let paint = self
                            .paints
                            .as_ref()
                            .and_then(|p| p[*index])
                            .map_or(missing, crate::color::Paint::resolve);
                        entries.push((
                            format!(
                                "{} – {}",
                                crate::number::ecmascript(a.min(b)),
                                crate::number::ecmascript(a.max(b))
                            ),
                            paint,
                        ));
                        intervals.push(GuideInterval {
                            lower: Some(ScaleKey::Number(Number(a.min(b)))),
                            upper: Some(ScaleKey::Number(Number(a.max(b)))),
                            entry: i,
                            closure: if i == 0 && s.right || i + 1 == s.range().len() && !s.right {
                                IntervalClosure::Both
                            } else if s.right == (a <= b) {
                                IntervalClosure::Right
                            } else {
                                IntervalClosure::Left
                            },
                        });
                    }
                }
                PreparedMapping::Classifier(s) => {
                    let formatter = interval_formatter(s.spec().range.iter().flat_map(|index| {
                        let e = s.invert_extent(index);
                        [e.lower, e.upper].into_iter().flatten()
                    }))?;
                    for (i, index) in s.spec().range.iter().enumerate() {
                        let e = s.invert_extent(index);
                        let label = interval_text(
                            e.lower.map(|n| formatter.format(n.0)),
                            e.upper.map(|n| formatter.format(n.0)),
                        );
                        let paint = self
                            .paints
                            .as_ref()
                            .and_then(|p| p[*index])
                            .map(crate::color::Paint::resolve)
                            .unwrap_or(missing);
                        entries.push((label, paint));
                        intervals.push(GuideInterval {
                            closure: IntervalClosure::Left,
                            lower: e.lower.map(ScaleKey::Number),
                            upper: e.upper.map(ScaleKey::Number),
                            entry: i,
                        });
                    }
                }
                PreparedMapping::Threshold(s) => {
                    for (i, index) in s.spec().range.iter().enumerate() {
                        let e = s.invert_extent(index);
                        entries.push((
                            interval_text(
                                e.lower.as_ref().map(key_label),
                                e.upper.as_ref().map(key_label),
                            ),
                            self.paints
                                .as_ref()
                                .and_then(|p| p[*index])
                                .map(crate::color::Paint::resolve)
                                .unwrap_or(missing),
                        ));
                        intervals.push(GuideInterval {
                            closure: IntervalClosure::Left,
                            lower: e.lower,
                            upper: e.upper,
                            entry: i,
                        });
                    }
                }
                PreparedMapping::Ordinal(s) => {
                    self.discrete_entries(&s.spec().domain, missing, &mut entries)?;
                }
                PreparedMapping::Continuous(s) => {
                    if self.spec.reference_guides() {
                        self.continuous_entries(&numeric_breaks, missing, &mut entries)?;
                    } else {
                        for d in &s.spec().domain {
                            entries.push((
                                crate::number::ecmascript(d.0),
                                self.color(Some(d.0), None, missing)?,
                            ));
                        }
                    }
                }
                PreparedMapping::Interpolated(s) => {
                    let domain = s.normalizer().domain();
                    if matches!(s.spec().normalization, NormalizationSpec::Diverging { .. }) {
                        midpoint = domain.get(1).copied();
                    }
                    if self.spec.reference_guides()
                        && !matches!(s.spec().normalization, NormalizationSpec::Quantile { .. })
                    {
                        self.continuous_entries(&numeric_breaks, missing, &mut entries)?;
                    } else {
                        let points: Vec<_> =
                            if matches!(s.spec().normalization, NormalizationSpec::Quantile { .. })
                            {
                                s.normalizer()
                                    .quantiles(4.)?
                                    .into_iter()
                                    .flatten()
                                    .collect()
                            } else {
                                domain.to_vec()
                            };
                        for d in points {
                            entries.push((
                                crate::number::ecmascript(d.0),
                                self.color(Some(d.0), None, missing)?,
                            ));
                        }
                    }
                }
            }
        }
        Ok(ColorLegend {
            title: None,
            id,
            entries,
            numeric_breaks,
            continuous,
            missing,
            intervals,
            midpoint,
            mapping: Some(self.spec.clone()),
        })
    }
}
fn reference_value_paint(value: &Value, ggplot: bool) -> ChartResult<Option<crate::color::Paint>> {
    if ggplot && let Value::Text(text) = value {
        return crate::color::parse_r(text).map(Some);
    }
    value_paint(value)
}
fn value_paint(value: &Value) -> ChartResult<Option<crate::color::Paint>> {
    match value {
        Value::Missing | Value::Null => Ok(None),
        Value::Color(c) => Ok(Some((*c).into())),
        Value::Text(s) => crate::color::parse(s)
            .map(|v| v.map(Into::into))
            .and_then(|v| {
                v.ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "A color scale range contains invalid color text.",
                    )
                })
            })
            .map(Some),
        _ => Err(error(
            DiagnosticCode::SchemaConflict,
            "A color scale requires color-valued outputs.",
        )),
    }
}
// Labels are presentation: start with six significant digits, increasing precision
// until distinct finite cutpoints remain distinct. Exact extents stay in GuideInterval.
fn interval_formatter(
    values: impl Iterator<Item = Number>,
) -> ChartResult<crate::typography::NumericFormatter> {
    let mut values: Vec<_> = values.map(|n| n.0).filter(|n| n.is_finite()).collect();
    values.sort_by(f64::total_cmp);
    values.dedup_by(|a, b| *a == *b);
    for precision in 6..=17 {
        let format = crate::typography::NumericFormat {
            specifier: format!(".{precision}~g"),
            locale: crate::typography::NumericLocale {
                minus: "-".into(),
                ..Default::default()
            },
        }
        .prepare()?;
        if precision == 17
            || values
                .windows(2)
                .all(|v| format.format(v[0]) != format.format(v[1]))
        {
            return Ok(format);
        }
    }
    unreachable!("binary64 needs at most seventeen significant decimal digits")
}
fn interval_text(a: Option<String>, b: Option<String>) -> String {
    match (a, b) {
        (None, None) => "All values".into(),
        (None, Some(b)) => format!("< {b}"),
        (Some(a), None) => format!(">= {a}"),
        (Some(a), Some(b)) => format!("{a} – {b}"),
    }
}
fn discrete_text_key(key: &ScaleKey) -> Option<String> {
    match key {
        ScaleKey::Number(value) => Some(crate::number::ecmascript(value.0)),
        ScaleKey::Integer(value) => Some(value.to_string()),
        ScaleKey::Unsigned(value) => Some(value.to_string()),
        ScaleKey::Boolean(value) => Some(if *value { "TRUE" } else { "FALSE" }.into()),
        _ => None,
    }
}
fn key_label(key: &ScaleKey) -> String {
    match key {
        ScaleKey::Null => "null".into(),
        ScaleKey::Boolean(v) => v.to_string(),
        ScaleKey::Number(v) => crate::number::ecmascript(v.0),
        ScaleKey::Integer(v) | ScaleKey::Timestamp(v) => v.to_string(),
        ScaleKey::Unsigned(v) => v.to_string(),
        ScaleKey::Text(v) => v.clone(),
    }
}
/// A guide entry's real data interval; labels are presentation only.
/// Interval membership; the default preserves the existing D3 left-closed contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntervalClosure {
    /// Inclusive lower endpoint.
    #[default]
    Left,
    /// Inclusive upper endpoint.
    Right,
    /// Both finite endpoints included.
    Both,
}
impl IntervalClosure {
    fn is_left(&self) -> bool {
        *self == Self::Left
    }
}
/// Exact interval bounds and membership, independent of formatted labels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuideInterval {
    /// Which finite endpoints belong to this interval.
    #[serde(default, skip_serializing_if = "IntervalClosure::is_left")]
    pub closure: IntervalClosure,
    /// Inclusive lower cut, absent for an unbounded tail.
    pub lower: Option<ScaleKey>,
    /// Upper cut, absent for an unbounded tail.
    pub upper: Option<ScaleKey>,
    /// Index in the legend's entries.
    pub entry: usize,
}

pub(super) fn numeric_output(value: &Value) -> ChartResult<()> {
    if matches!(value, Value::Number(_) | Value::Missing | Value::Null) {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric aesthetic scales require numeric or missing range outputs.",
        ))
    }
}

// Categorical numeric keys retain exact identity, but named chart palettes treat
// exceptional source numbers as missing just like numerical color observations.
fn finite_key(key: &ScaleKey) -> bool {
    !matches!(key, ScaleKey::Number(value) if !value.0.is_finite())
}

impl MappedScaleSpec {
    /// Whether this scale requires the ggplot2 palette capability.
    pub fn has_ggplot(&self) -> bool {
        self.ggplot.is_some()
            || matches!(
                &self.function,
                ScaleFunctionSpec::GgplotNumericIdentity(_)
                    | ScaleFunctionSpec::GgplotDiscreteIdentity(_)
            )
            || matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(i) if i.wire_version()==3) || matches!(s.normalization,NormalizationSpec::Ggplot {..}))
    }
    /// Whether this scale requires versioned interpolation code installation.
    pub fn has_registered_interpolation(&self) -> bool {
        match &self.function {
            ScaleFunctionSpec::Continuous(s) => s.factory.has_registration(),
            ScaleFunctionSpec::Interpolated(s) => {
                matches!(&s.output, ScaleRangeFunction::Interpolate(i) if i.has_registration())
            }
            _ => false,
        }
    }
    pub(crate) fn validate_interpolations(
        &self,
        registrations: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        match &self.function {
            ScaleFunctionSpec::Continuous(s) => {
                s.factory.validate_registration(registrations, portable)
            }
            ScaleFunctionSpec::Interpolated(s) => match &s.output {
                ScaleRangeFunction::Interpolate(i) => {
                    i.validate_registrations(registrations, portable)
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}
