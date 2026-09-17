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
    /// Presentation sampling for continuous colorbars; ignored by other guide kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colorbar_options: Option<Box<GgplotColorbarOptions>>,
    /// Ordered reference aesthetics whose theme palette may replace this fallback.
    /// Empty means the authored palette is explicit and bypasses theme lookup.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette_theme_aesthetics: Vec<String>,
    /// Pure vector OOB operation in transformed reference coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oob_function: Option<Box<crate::grammar::ScaleVectorOperation>>,
    /// Pure vector rescaler before palette evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rescaler_function: Option<Box<crate::grammar::ScaleVectorOperation>>,
    /// Trained discrete slots that use na.value because no palette element matched.
    /// Discrete returned NA elements remain distinct from replacement slots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette_fallback_indices: Vec<usize>,
    /// Treat absent reference color outputs as NA, distinct from transparent paint.
    /// Explicitly authored color replacement clears this setting; numeric outputs
    /// already retain missing values independently.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub missing_paint_is_na: bool,
    /// Registered count or vector palette over an eligible reference domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette_function: Option<Box<crate::grammar::ScalePaletteOperation>>,
    /// Numeric callback output in source coordinates, retaining its length and missing values.
    /// Populated by eligible training; replacement training recomputes it from source data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_numeric_limits: Option<Box<Vec<Number>>>,
    /// Distinguish a resolved NULL identity domain from a typed empty limit vector.
    /// Replacement training recomputes this flag from the registered operation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub resolved_discrete_limits_null: bool,
    /// Transformed bounds captured by population training for vector transforms.
    /// Replacement training recomputes these from the complete source population.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trained_transformed_bounds: Option<[Number; 2]>,
    /// Pure registered break selection after population training.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breaks_function: Option<Box<crate::grammar::ScaleBreaksOperation>>,
    /// Pure registered limit replacement after population training.
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
    pub(crate) fn preserves_palette_missing(&self) -> bool {
        self.palette_function.is_some()
            || matches!(
                self.ggplot.as_deref(),
                Some(GgplotScalePolicy::Discrete {
                    palette: GgplotDiscretePalette::Values(_) | GgplotDiscretePalette::Named(_),
                    ..
                })
            )
    }
    /// Construct a scale using its explicit authored domain.
    pub fn authored(function: ScaleFunctionSpec) -> Self {
        Self {
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
            breaks_function: None,
            limits_function: None,
            guide: None,
            ggplot: None,
            catalog: None,
            function,
            training: ScaleTraining::Authored,
        }
    }
    /// Retain the current palette as fallback and request ordered theme lookup.
    pub fn with_theme_palette(mut self, aesthetics: Vec<String>) -> ChartResult<Self> {
        self.palette_theme_aesthetics = aesthetics;
        self.validate_training()?;
        Ok(self)
    }
    /// Select a pure OOB vector function over transformed observations.
    pub fn with_oob_function(
        mut self,
        operation: crate::grammar::ScaleVectorOperation,
    ) -> ChartResult<Self> {
        self.oob_function = Some(Box::new(operation));
        self.validate_training()?;
        Ok(self)
    }
    /// Select a pure rescaler over the complete OOB output vector.
    pub fn with_rescaler_function(
        mut self,
        operation: crate::grammar::ScaleVectorOperation,
    ) -> ChartResult<Self> {
        self.rescaler_function = Some(Box::new(operation));
        self.validate_training()?;
        Ok(self)
    }
    /// Select a pure installed palette over a trained count or normalized vector.
    pub fn with_palette_function(
        mut self,
        operation: crate::grammar::ScalePaletteOperation,
    ) -> ChartResult<Self> {
        self.palette_theme_aesthetics.clear();
        self.palette_function = Some(Box::new(operation));
        self.validate_training()?;
        Ok(self)
    }
    /// Select a registered break function over the prepared limits.
    pub fn with_breaks_function(
        mut self,
        operation: crate::grammar::ScaleBreaksOperation,
    ) -> ChartResult<Self> {
        self.breaks_function = Some(Box::new(operation));
        self.validate_training()?;
        Ok(self)
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
    /// Retain authored raster colorbar sampling. Invalid counts are rejected when
    /// a visible guide demands samples, preserving hidden/no-break behavior.
    pub fn with_colorbar_options(mut self, options: GgplotColorbarOptions) -> Self {
        self.colorbar_options = Some(Box::new(options));
        self
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
            Some(
                GgplotScaleGuide::Continuous(g)
                | GgplotScaleGuide::Colorbar(g)
                | GgplotScaleGuide::ContinuousBins(g)
                | GgplotScaleGuide::ContinuousSteps(g),
            ) => Some(g),
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
        self.palette_theme_aesthetics.clear();
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
    pub(crate) fn ggplot_transform_mut(&mut self) -> Option<&mut super::GgplotTransform> {
        match &mut self.function {
            ScaleFunctionSpec::Continuous(s) => s.family.ggplot_transform_mut(),
            ScaleFunctionSpec::Interpolated(s) => s.normalization.ggplot_transform_mut(),
            ScaleFunctionSpec::GgplotNumericIdentity(s) => s
                .transform
                .as_mut()
                .and_then(super::ScaleTransform::ggplot_transform_mut),
            _ => None,
        }
    }
    pub(crate) fn ggplot_transform(&self) -> Option<&super::GgplotTransform> {
        match &self.function {
            ScaleFunctionSpec::Continuous(s) => s.family.ggplot_transform(),
            ScaleFunctionSpec::Interpolated(s) => s.normalization.ggplot_transform(),
            ScaleFunctionSpec::GgplotNumericIdentity(s) => s
                .transform
                .as_ref()
                .and_then(super::ScaleTransform::ggplot_transform),
            _ => None,
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
        if self
            .ggplot_transform()
            .is_some_and(super::GgplotTransform::needs_resolution)
        {
            let mut captured = self.clone();
            captured
                .ggplot_transform_mut()
                .unwrap()
                .resolve_registrations(&registry.transforms_function, false)?;
            return captured.trained_with_registry(values, registry);
        }
        if self.breaks_function.is_some()
            && matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
        {
            return super::ggplot_binned_breaks::train(self, values, registry);
        }
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
        next.resolved_discrete_limits_null = false;
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
                next.resolved_discrete_limits_null = resolved.is_none();
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
            policy.train_keys(&mut next, keys, Some(registry))?;
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
        if let Some(t) = spec.ggplot_transform_mut()
            && t.needs_resolution()
        {
            t.resolve_registrations(&registry.transforms_function, false)?;
        }
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
        if let Some(options) = &self.colorbar_options {
            options.validate()?;
        }
        if self.trained_transformed_bounds.is_some()
            && (!matches!(&self.function, ScaleFunctionSpec::Interpolated(s)
                if matches!(&s.normalization, NormalizationSpec::Ggplot {
                    family: NumericFamily::Ggplot { transform }, ..
                } if transform.has_registered()))
                || !matches!(
                    self.ggplot.as_deref(),
                    Some(GgplotScalePolicy::Continuous { .. } | GgplotScalePolicy::Binned(_))
                ))
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Trained transformed bounds require a registered reference transform.",
            ));
        }
        if !self.palette_theme_aesthetics.is_empty()
            && (self.ggplot.is_none()
                || self.palette_theme_aesthetics.len() > 16
                || self
                    .palette_theme_aesthetics
                    .iter()
                    .any(|a| !crate::theme::palette_aesthetic(a)))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Theme palette lookup requires a reference scale and up to sixteen aesthetic names.",
            ));
        }
        if !self.palette_fallback_indices.is_empty() {
            let valid = match &self.function {
                ScaleFunctionSpec::Ordinal(s) => {
                    self.preserves_palette_missing()
                        && self
                            .palette_fallback_indices
                            .iter()
                            .all(|i| *i < s.range.len())
                        && self
                            .palette_fallback_indices
                            .windows(2)
                            .all(|w| w[0] < w[1])
                }
                _ => false,
            };
            if !valid {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Palette fallback slots must be ordered indices in a discrete palette range.",
                ));
            }
        }

        if (self.oob_function.is_some() || self.rescaler_function.is_some())
            && !matches!(
                self.ggplot.as_deref(),
                Some(GgplotScalePolicy::Continuous { .. } | GgplotScalePolicy::Binned(_))
            )
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered vector pipelines require a continuous or binned reference scale.",
            ));
        }
        if self.missing_paint_is_na
            && !matches!(
                self.ggplot.as_deref(),
                Some(
                    GgplotScalePolicy::Discrete { .. }
                        | GgplotScalePolicy::Binned(_)
                        | GgplotScalePolicy::Continuous { .. }
                )
            )
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Missing NA paint requires a reference discrete, continuous or binned scale.",
            ));
        }
        if self.palette_function.is_some()
            && (self.training != ScaleTraining::Eligible
                || self.catalog.is_some()
                || !matches!(
                    self.ggplot.as_deref(),
                    Some(
                        GgplotScalePolicy::Discrete { .. }
                            | GgplotScalePolicy::Binned(_)
                            | GgplotScalePolicy::Continuous { .. }
                    )
                ))
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered palettes require an eligible reference discrete, continuous or binned scale without named catalog metadata.",
            ));
        }
        if self.breaks_function.is_some() {
            let supported = match self.ggplot.as_deref() {
                None if matches!(self.function, ScaleFunctionSpec::GgplotNumericIdentity(_)) => {
                    matches!(
                        self.guide.as_deref(),
                        None | Some(GgplotScaleGuide::Continuous(_) | GgplotScaleGuide::Hidden)
                    ) && !self.continuous_guide().is_some_and(|g| g.breaks.is_some())
                }
                Some(GgplotScalePolicy::Continuous { .. }) => {
                    matches!(
                        self.guide.as_deref(),
                        None | Some(
                            GgplotScaleGuide::Continuous(_)
                                | GgplotScaleGuide::Colorbar(_)
                                | GgplotScaleGuide::ContinuousBins(_)
                                | GgplotScaleGuide::TemporalBins(_)
                                | GgplotScaleGuide::ContinuousSteps(_)
                                | GgplotScaleGuide::TemporalSteps(_)
                                | GgplotScaleGuide::Temporal(_)
                                | GgplotScaleGuide::TemporalColorbar(_)
                                | GgplotScaleGuide::Hidden
                        )
                    ) && !self.continuous_guide().is_some_and(|g| g.breaks.is_some())
                }
                Some(GgplotScalePolicy::Binned(policy)) => {
                    !matches!(policy.breaks, GgplotBreaks::Explicit(_))
                        && matches!(
                            self.guide.as_deref(),
                            None | Some(
                                GgplotScaleGuide::Binned(_)
                                    | GgplotScaleGuide::BinnedLegend(_)
                                    | GgplotScaleGuide::BinnedBins(_)
                                    | GgplotScaleGuide::BinnedSteps(_)
                                    | GgplotScaleGuide::Hidden
                            )
                        )
                }
                None if matches!(self.function, ScaleFunctionSpec::GgplotDiscreteIdentity(_)) => {
                    matches!(
                        self.guide.as_deref(),
                        None | Some(GgplotScaleGuide::Discrete(_) | GgplotScaleGuide::Hidden)
                    ) && !self
                        .discrete_guide()
                        .is_some_and(|g| g.breaks.is_some() || g.break_names.is_some())
                }
                Some(GgplotScalePolicy::Discrete { .. }) => {
                    matches!(
                        self.guide.as_deref(),
                        None | Some(GgplotScaleGuide::Discrete(_) | GgplotScaleGuide::Hidden)
                    ) && !self
                        .discrete_guide()
                        .is_some_and(|g| g.breaks.is_some() || g.break_names.is_some())
                }
                _ => false,
            };
            if !supported {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Registered breaks require a reference continuous, discrete or binned scale without fixed breaks.",
                ));
            }
        }
        if self.resolved_discrete_limits_null
            && (self.limits_function.is_none()
                || !matches!(&self.function,
                ScaleFunctionSpec::GgplotDiscreteIdentity(identity) if identity.limits.as_ref().is_some_and(Vec::is_empty)))
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A resolved NULL identity domain requires an empty discrete identity limit result and its operation.",
            ));
        }
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
        if matches!(
            self.guide.as_deref(),
            Some(
                GgplotScaleGuide::Colorbar(_)
                    | GgplotScaleGuide::TemporalColorbar(_)
                    | GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) && !(matches!(
            self.ggplot.as_deref(),
            Some(GgplotScalePolicy::Continuous { .. })
        ) && matches!(&self.function, ScaleFunctionSpec::Interpolated(scale)
                    if matches!(scale.normalization, NormalizationSpec::Ggplot { .. } | NormalizationSpec::Sequential { .. })))
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Continuous colorbar and interval guides require a reference ggplot normalization.",
            ));
        }
        if let Some(guide) = &self.guide {
            guide.validate()?;
            let matching = match guide.as_ref() {
                GgplotScaleGuide::Hidden => true,
                GgplotScaleGuide::Discrete(_) => self.categorical(),
                GgplotScaleGuide::Binned(_)
                | GgplotScaleGuide::BinnedLegend(_)
                | GgplotScaleGuide::BinnedBins(_)
                | GgplotScaleGuide::BinnedSteps(_) => {
                    matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
                }
                GgplotScaleGuide::Continuous(_)
                | GgplotScaleGuide::Colorbar(_)
                | GgplotScaleGuide::ContinuousBins(_)
                | GgplotScaleGuide::TemporalBins(_)
                | GgplotScaleGuide::ContinuousSteps(_)
                | GgplotScaleGuide::TemporalSteps(_)
                | GgplotScaleGuide::Temporal(_)
                | GgplotScaleGuide::TemporalColorbar(_) => {
                    matches!(
                        self.function,
                        ScaleFunctionSpec::Continuous(_)
                            | ScaleFunctionSpec::Interpolated(_)
                            | ScaleFunctionSpec::GgplotNumericIdentity(_)
                    ) && !matches!(self.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(_)))
                        && !matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization,NormalizationSpec::Quantile {..}))
                }
            };
            if let GgplotScaleGuide::Binned(labels)
            | GgplotScaleGuide::BinnedLegend(labels)
            | GgplotScaleGuide::BinnedBins(labels)
            | GgplotScaleGuide::BinnedSteps(labels) = guide.as_ref()
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
                GgplotScalePolicy::Discrete { .. } => policy.train_keys(&mut empty, &[], None)?,
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
    /// Color-only guide selection is suppressed before a non-color scale trains.
    pub(crate) fn trained_value_population(
        &self,
        population: Option<&ScalePopulation>,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        if matches!(
            self.guide.as_deref(),
            Some(GgplotScaleGuide::BinnedSteps(_))
        ) {
            let mut scale = self.clone();
            scale.guide = Some(Box::new(GgplotScaleGuide::Hidden));
            scale.trained_population(population, registry)
        } else {
            self.trained_population(population, registry)
        }
    }
    pub(crate) fn trained_population(
        &self,
        population: Option<&ScalePopulation>,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        if self.training == ScaleTraining::Authored {
            return Ok(self.clone());
        }
        if self.limits_function.is_some()
            && matches!(self.guide.as_deref(), Some(GgplotScaleGuide::Hidden))
            && matches!(
                self.ggplot.as_deref(),
                Some(GgplotScalePolicy::Continuous { .. } | GgplotScalePolicy::Binned(_))
            )
            && match population {
                None => true,
                Some(ScalePopulation::Numbers { values, .. }) => values.is_empty(),
                _ => false,
            }
        {
            // Primary builds with no observations or guide keys never request
            // function limits. Retain the operation as unresolved rather than
            // forcing it merely to prepare an empty layer.
            let mut base = self.clone();
            let call = base.limits_function.take();
            base.resolved_numeric_limits = None;
            let mut trained = base.trained_with_registry(&[], registry)?;
            trained.limits_function = call;
            return Ok(trained);
        }
        if matches!(&self.function, ScaleFunctionSpec::GgplotDiscreteIdentity(identity) if !identity.guide)
        {
            let mut base = self.clone();
            let call = base.limits_function.take();
            base.resolved_discrete_limits_null = false;
            let keys = match population {
                Some(ScalePopulation::Keys(keys)) => keys.as_slice(),
                _ => &[],
            };
            let mut trained = base.trained_keys_with_registry(keys, registry)?;
            trained.limits_function = call;
            return Ok(trained);
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
            (
                _,
                Some(ScalePopulation::Numbers {
                    values, batches, ..
                }),
            ) => {
                let mut captured = self.clone();
                if let Some(transform) = captured.ggplot_transform_mut() {
                    transform.resolve_registrations(&registry.transforms_function, false)?;
                }
                if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &mut captured.function {
                    if captured.limits_function.is_none() || !identity.guide {
                        identity.train_batches(values, batches)?;
                        return Ok(captured);
                    }
                    return super::ggplot_numeric_limits::train_batches(
                        &captured, values, batches, registry,
                    );
                }
                if captured
                    .ggplot_transform()
                    .is_some_and(|t| !t.is_pointwise())
                    && let Some(policy) = captured.ggplot.clone()
                {
                    if captured.breaks_function.is_some()
                        && matches!(*policy, GgplotScalePolicy::Binned(_))
                    {
                        return super::ggplot_binned_breaks::train_batches(
                            &captured, values, batches, registry,
                        );
                    }
                    if captured.limits_function.is_some() {
                        return super::ggplot_numeric_limits::train_batches(
                            &captured, values, batches, registry,
                        );
                    }
                    policy.train_number_batches(&mut captured, values, batches)?;
                    Ok(captured)
                } else {
                    captured.trained_with_registry(values, registry)
                }
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
    Numbers {
        values: Vec<Option<Number>>,
        batches: Vec<usize>,
        sources: Vec<ScaleLayerBatch>,
    },
    Keys(Vec<ScaleKey>),
}
/// Stable row identity within one layer/input population.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScaleRowIdentity {
    Source(crate::RowKey),
    Generated(Option<crate::grammar::PanelKey>, u64),
}
/// Rows retain layer/input boundaries and source or generated panel order.
#[derive(Clone, Debug)]
pub(crate) struct ScaleLayerBatch {
    pub layer: crate::LayerId,
    pub input: crate::grammar::ColorInput,
    pub rows: std::collections::BTreeMap<usize, (ScaleRowIdentity, Option<Number>)>,
    pub generated_panels: std::collections::BTreeSet<Option<crate::grammar::PanelKey>>,
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
    vector_pipeline: Option<super::ggplot_vector::VectorPipeline>,
    palette_registry:
        std::sync::Arc<crate::grammar::scale_palette_extensions::ScalePaletteRegistrations>,
    breaks_registry:
        std::sync::Arc<crate::grammar::scale_break_extensions::ScaleBreakRegistrations>,
    registry: std::sync::Arc<crate::grammar::guide_extensions::GuideRegistrations>,
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
        if let Some(t) = spec.ggplot_transform_mut()
            && t.needs_resolution()
        {
            t.resolve_registrations(&registry.transforms_function, false)?;
        }
        GgplotScalePolicy::canonicalize(&mut spec)?;
        spec.validate_interpolations(&registry.interpolations, false)?;
        if let Some(GgplotGuideLabels::Registered {
            operation,
            parameters,
        }) = spec.guide.as_deref().and_then(GgplotScaleGuide::labels)
        {
            registry.guides.validate(operation, parameters, false)?;
        }
        // Validate population policy without discarding an already trained population.
        if let Some(call) = &spec.limits_function {
            registry.limits_function.validate(call, false)?;
        }
        for call in [&spec.oob_function, &spec.rescaler_function]
            .into_iter()
            .flatten()
        {
            registry.scale_vectors.validate(call, false)?;
        }
        if let Some(call) = &spec.palette_function {
            registry.palette_function.validate(call, false)?;
        }
        if let Some(call) = &spec.breaks_function {
            registry.breaks_function.validate(call, false)?;
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
        let mut mapping = if let Some(GgplotScalePolicy::Binned(_)) = spec.ggplot.as_deref() {
            PreparedMapping::Binned(super::ggplot_bins::GgplotBinnedMapping::new(
                &spec, registry, &mut pool,
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
                        // Custom palettes do not cycle a short range. Retain the
                        // domain index and check it only when it is sampled.
                        range: if spec.palette_function.is_some() {
                            (0..s.domain.len()).collect()
                        } else {
                            range
                        },
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
            } = &scale.spec().normalization
            && let Some(bounds) = policy.population_bounds(family.clone(), *reverse)
        {
            scale.set_reference_bounds(bounds);
        }
        if let (Some(bounds), PreparedMapping::Interpolated(scale)) =
            (spec.trained_transformed_bounds, &mut mapping)
        {
            scale.set_reference_bounds(bounds.map(|v| v.0));
        }
        Ok(Self {
            registry: registry.guides.clone(),
            breaks_registry: registry.breaks_function.clone(),
            vector_pipeline: super::ggplot_vector::VectorPipeline::new(&spec, registry),
            palette_registry: registry.palette_function.clone(),
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
                // A reference palette has a fixed paint output type. Only a
                // genuinely empty population skips evaluation; an all-NA vector
                // still forces the reference palette's position validation.
                let empty_reference_palette = matches!(
                    scale.spec.ggplot.as_deref(),
                    Some(GgplotScalePolicy::Continuous {
                        empty_population: true,
                        ..
                    })
                ) && matches!(
                    &s.spec().output,
                    ScaleRangeFunction::Interpolate(
                        crate::interpolate::InterpolationSpec::GgplotPalette { .. }
                    )
                );
                if !scale.has_continuous_palette() && !empty_reference_palette {
                    s.validate_color_output()?;
                }
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
                if !scale.has_continuous_palette() {
                    s.validate_numeric_output()?;
                }
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
        // Empty binned layers can omit mapping even when standalone sampling of
        // their function limits would fail. Guides and actual samples validate
        // independently; preparation alone must not invoke the map contract.
        if (matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden))
            || self.spec.resolved_numeric_limits.is_some())
            && matches!(self.spec.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(p)) if p.empty_population)
        {
            return Ok(());
        }
        self.validate_sampling()
    }
    fn validate_sampling(&self) -> ChartResult<()> {
        if let Some(values) = &self.spec.resolved_numeric_limits
            && self.vector_pipeline.is_none()
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
        if self.spec.preserves_palette_missing()
            && matches!(self.mapping, PreparedMapping::Ordinal(_))
            && key.is_none_or(|k| *k == ScaleKey::Null)
        {
            return None;
        }
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
        if let Some(call) = &self.spec.palette_function {
            // Empty charts never map a mark. A direct lookup of an untrained
            // reference scale uses the source's fallback limits [0,1].
            if matches!(
                self.spec.ggplot.as_deref(),
                Some(GgplotScalePolicy::Discrete {
                    empty_population: true,
                    limits: None,
                    ..
                })
            ) {
                let domain = [ScaleKey::Text("0".into()), ScaleKey::Text("1".into())];
                let values =
                    super::ggplot_palette::discrete_values(&self.palette_registry, call, &domain)?
                        .values;
                let index =
                    key.and_then(super::ggplot_palette::text_key)
                        .and_then(|k| match k.as_str() {
                            "0" => Some(0),
                            "1" => Some(1),
                            _ => None,
                        });
                return index.map_or(Ok(Some(Value::Missing)), |i| {
                    values
                        .get(i)
                        .cloned()
                        .map(Some)
                        .ok_or_else(palette_lookup_error)
                });
            }
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
    pub(crate) fn has_continuous_palette(&self) -> bool {
        (self.spec.palette_function.is_some()
            || self.spec.oob_function.is_some()
            || self.spec.rescaler_function.is_some()
            || self
                .spec
                .ggplot_transform()
                .is_some_and(|transform| !transform.is_pointwise()))
            && matches!(
                self.spec.ggplot.as_deref(),
                Some(GgplotScalePolicy::Continuous { .. })
            )
    }
    fn numeric_input(&self, input: Option<f64>) -> Option<f64> {
        if let Some(values) = &self.spec.resolved_numeric_limits {
            super::ggplot_numeric_limits::input(&self.spec, values, input)
        } else {
            self.spec
                .ggplot
                .as_ref()
                .map_or(input, |p| p.input(input, &self.spec.function))
        }
    }
    pub(crate) fn row_palette_batch(
        &self,
        inputs: &[Option<f64>],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        let mut batch = self.palette_batch(inputs)?;
        if matches!(self.mapping, PreparedMapping::Binned(_))
            && let Some(batch) = &mut batch
            && batch.values.is_none()
        {
            // Binned NULL assignment creates missing rows, unlike the removed
            // vectors returned by continuous and discrete palette functions.
            batch.values = Some(vec![Value::Missing]);
            batch.indices = vec![0; inputs.len()];
        }
        Ok(batch)
    }
    pub(crate) fn palette_batch(
        &self,
        inputs: &[Option<f64>],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        if let PreparedMapping::GgplotNumericIdentity(identity) = &self.mapping
            && self
                .spec
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise())
        {
            let values = identity.transform_values(
                &inputs
                    .iter()
                    .map(|v| v.unwrap_or(f64::NAN))
                    .collect::<Vec<_>>(),
            )?;
            let values = values
                .into_iter()
                .zip(inputs)
                .map(|(v, input)| {
                    if v.is_nan() && input.is_none() {
                        Value::Missing
                    } else {
                        Value::Number(Number(v))
                    }
                })
                .collect();
            return Ok(Some(super::ggplot_palette::PaletteBatch {
                values: Some(values),
                indices: (0..inputs.len()).collect(),
            }));
        }
        if let PreparedMapping::Binned(mapping) = &self.mapping {
            if inputs.is_empty() {
                return Ok(Some(super::ggplot_palette::PaletteBatch {
                    values: Some(vec![]),
                    indices: vec![],
                }));
            }
            self.validate_sampling()?;
            if mapping.has_null_palette() {
                return Ok(Some(super::ggplot_palette::PaletteBatch {
                    values: None,
                    indices: vec![],
                }));
            }
            if let Some(batch) = mapping.pipeline_batch(inputs)? {
                return Ok(Some(batch));
            }
            return Ok(Some(super::ggplot_palette::PaletteBatch {
                values: Some(self.values.clone()),
                indices: mapping
                    .batch_indices(inputs.iter().map(|input| self.numeric_input(*input))),
            }));
        }
        if !self.has_continuous_palette() {
            return Ok(None);
        }
        self.validate_sampling()?;
        let PreparedMapping::Interpolated(scale) = &self.mapping else {
            unreachable!("continuous palette normalizer")
        };
        if inputs.is_empty() {
            return Ok(Some(super::ggplot_palette::PaletteBatch {
                values: Some(vec![]),
                indices: vec![],
            }));
        }
        let samples = if let Some(pipeline) = &self.vector_pipeline {
            pipeline.map(scale.normalizer(), inputs)?
        } else {
            inputs
                .iter()
                .map(|input| {
                    Number(
                        scale
                            .normalizer()
                            .parameter(self.numeric_input(*input))
                            .unwrap_or(f64::NAN),
                    )
                })
                .collect()
        };
        self.normalized_palette_batch(scale, samples)
    }
    fn transformed_palette_batch(
        &self,
        values: &[Number],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        if values.is_empty() {
            return Ok(Some(super::ggplot_palette::PaletteBatch {
                values: Some(vec![]),
                indices: vec![],
            }));
        }
        self.validate_sampling()?;
        if let PreparedMapping::Binned(mapping) = &self.mapping {
            return mapping.pipeline_transformed_batch(values);
        }
        let PreparedMapping::Interpolated(scale) = &self.mapping else {
            unreachable!("continuous guide normalization")
        };
        let pipeline = self
            .vector_pipeline
            .as_ref()
            .expect("continuous guide pipeline");
        let samples = pipeline.map_transformed(scale.normalizer(), values)?;
        self.normalized_palette_batch(scale, samples)
    }
    fn binned_guide_bounds(&self) -> ChartResult<[f64; 2]> {
        let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
            unreachable!("binned mapping")
        };
        let NormalizationSpec::Ggplot {
            domain,
            ref family,
            reverse,
            ..
        } = scale.normalization
        else {
            unreachable!("binned normalization")
        };
        self.interval_bounds(domain, family, reverse)
    }
    fn interval_bounds(
        &self,
        domain: [Number; 2],
        family: &NumericFamily,
        reverse: bool,
    ) -> ChartResult<[f64; 2]> {
        if self
            .spec
            .guide
            .as_deref()
            .and_then(GgplotScaleGuide::temporal)
            .is_some()
        {
            return Ok(self
                .spec
                .resolved_numeric_limits
                .as_ref()
                .map_or(domain.map(|v| v.0), |v| {
                    [v[0].0, v.get(1).unwrap_or(&v[0]).0]
                }));
        }
        if matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) {
            let PreparedMapping::Interpolated(scale) = &self.mapping else {
                unreachable!("continuous interval normalization")
            };
            let bounds = self
                .vector_pipeline
                .as_ref()
                .expect("continuous interval pipeline")
                .limits(scale.normalizer())?;
            let lower = bounds.first().map_or(f64::NAN, |v| v.0);
            return Ok([lower, bounds.get(1).map_or(lower, |v| v.0)]);
        }
        Ok(domain.map(|v| super::ggplot_continuous_guide::forward(family, reverse, v.0)))
    }
    fn interval_palette_values(&self, values: &[Number]) -> Vec<Number> {
        if let Some(g) = self
            .spec
            .guide
            .as_deref()
            .and_then(GgplotScaleGuide::temporal)
        {
            let normalization = GgplotTimestampNormalization {
                origin: g.origin,
                unit: g.unit,
                date: g.arguments.date,
            };
            values
                .iter()
                .map(|v| Number(normalization.absolute(v.0)))
                .collect()
        } else {
            values.to_vec()
        }
    }
    fn interval_midpoint_batch(
        &self,
        boundaries: &[f64],
        family: &NumericFamily,
        reverse: bool,
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        let values = boundaries
            .windows(2)
            .map(|v| Number(v[0].midpoint(v[1])))
            .collect::<Vec<_>>();
        if matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) {
            let values = self.interval_palette_values(&values);
            self.transformed_palette_batch(&values)
        } else {
            let inputs = values
                .iter()
                .map(|v| {
                    Some(super::ggplot_continuous_guide::inverse(
                        family, reverse, v.0,
                    ))
                })
                .collect::<Vec<_>>();
            self.palette_batch(&inputs)
        }
    }
    fn normalized_palette_batch(
        &self,
        scale: &InterpolatedScale,
        samples: Vec<Number>,
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        if let Some(call) = self.spec.palette_function.as_deref() {
            super::ggplot_palette::continuous_values(
                &self.palette_registry,
                call,
                samples.into_iter(),
                &scale.spec().unknown,
            )
            .map(Some)
        } else {
            let mut values = samples
                .iter()
                .map(|v| {
                    if v.0.is_nan() {
                        Ok(scale.spec().unknown.clone())
                    } else {
                        scale.sample_parameter(v.0)
                    }
                })
                .collect::<ChartResult<Vec<_>>>()?;
            let indices = (0..values.len()).collect();
            if values.is_empty() {
                values.push(scale.spec().unknown.clone());
            }
            Ok(Some(super::ggplot_palette::PaletteBatch {
                indices,
                values: Some(values),
            }))
        }
    }

    pub(crate) fn palette_paints(
        &self,
        inputs: &[Option<f64>],
        missing: crate::color::Paint,
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Option<crate::color::Paint>>>> {
        let batch = match self.palette_batch(inputs)? {
            Some(batch) => batch,
            None if self.spec.missing_paint_is_na
                && matches!(
                    self.mapping,
                    PreparedMapping::Continuous(_) | PreparedMapping::Interpolated(_)
                ) =>
            {
                // Continuous built-in outputs do not have pooled paint indices.
                // Retain their missing slots in the same batch as their sampled paint.
                super::ggplot_palette::PaletteBatch {
                    values: Some(
                        inputs
                            .iter()
                            .map(|input| self.numeric(*input))
                            .collect::<ChartResult<Vec<_>>>()?,
                    ),
                    indices: (0..inputs.len()).collect(),
                }
            }
            None => return Ok(None),
        };
        if matches!(self.mapping, PreparedMapping::Binned(_)) && batch.values.is_none() {
            return Ok(Some(super::ggplot_palette::PaletteBatch {
                values: Some(vec![None]),
                indices: vec![0; inputs.len()],
            }));
        }
        let values = batch
            .values
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        reference_value_paint(value, true).map(|paint| {
                            paint.or_else(|| (!self.spec.missing_paint_is_na).then_some(missing))
                        })
                    })
                    .collect::<ChartResult<Vec<_>>>()
            })
            .transpose()?;
        Ok(Some(super::ggplot_palette::PaletteBatch {
            values,
            indices: batch.indices,
        }))
    }
    /// Map one observation batch. NULL palettes return no aesthetic vector.
    /// Continuous callbacks receive unique normalized inputs in first-seen order.
    pub fn numeric_batch(&self, inputs: &[Option<f64>]) -> ChartResult<Option<Vec<Value>>> {
        if let Some(batch) = self.palette_batch(inputs)? {
            return Ok(batch
                .values
                .map(|values| batch.indices.iter().map(|i| values[*i].clone()).collect()));
        }
        inputs
            .iter()
            .map(|input| self.numeric(*input))
            .collect::<ChartResult<Vec<_>>>()
            .map(Some)
    }
    /// Owned numerical or typed output with reference-specific discrete key matching.
    pub fn numeric(&self, input: Option<f64>) -> ChartResult<Value> {
        if let Some(batch) = self.palette_batch(&[input])? {
            let batch = batch.for_rows(1)?;
            return Ok(batch
                .values
                .map(|values| values[batch.indices[0]].clone())
                .unwrap_or(Value::Missing));
        }
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
                self.pooled_value(self.index(input, key.as_ref()))
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
        self.pooled_value(self.index(None, key))
    }
    fn pooled_value(&self, index: Option<usize>) -> ChartResult<Value> {
        index.map_or(Ok(Value::Missing), |i| {
            self.values.get(i).cloned().ok_or_else(palette_lookup_error)
        })
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
        ) && !self.spec.missing_paint_is_na
            && !self.suppresses_discrete_missing()
        {
            return false;
        }

        let number = input
            .filter(|v| !v.is_nan())
            .map(|v| ScaleKey::Number(Number(v)));
        if self.spec.preserves_palette_missing()
            && matches!(self.mapping, PreparedMapping::Ordinal(_))
        {
            let index = self.index(input, key.or(number.as_ref()));
            if index.is_some_and(|i| i >= self.values.len()) {
                return false;
            }
            return match index {
                Some(i)
                    if self
                        .spec
                        .palette_fallback_indices
                        .binary_search(&i)
                        .is_err() =>
                {
                    self.paints.as_ref().is_none_or(|p| p[i].is_none())
                }
                _ => self.spec.missing_paint_is_na || self.suppresses_discrete_missing(),
            };
        }
        let index = self.index(input, key.or(number.as_ref()));
        if self.spec.palette_function.is_some()
            && matches!(self.mapping, PreparedMapping::Binned(_))
        {
            return self.spec.missing_paint_is_na
                && index
                    .and_then(|i| {
                        self.paints
                            .as_ref()
                            .and_then(|p| p.get(i))
                            .copied()
                            .flatten()
                    })
                    .is_none();
        }
        if index.is_some_and(|i| i >= self.values.len()) {
            return false;
        }
        index
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
        if let Some(batch) = self.palette_paints(&[input], missing)? {
            let batch = batch.for_rows(1)?;
            return Ok(batch
                .values
                .and_then(|values| values[batch.indices[0]])
                .unwrap_or(crate::color::parse_r("NA")?));
        }
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
        let missing = if self.spec.missing_paint_is_na || self.suppresses_discrete_missing() {
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
                match self.index(input, key.or(numeric_key.as_ref())) {
                    Some(i) => paints
                        .get(i)
                        .ok_or_else(palette_lookup_error)?
                        .unwrap_or_else(|| {
                            if self.spec.preserves_palette_missing()
                                && matches!(self.mapping, PreparedMapping::Ordinal(_))
                                && self
                                    .spec
                                    .palette_fallback_indices
                                    .binary_search(&i)
                                    .is_err()
                            {
                                Color {
                                    red: 0,
                                    green: 0,
                                    blue: 0,
                                    alpha: 0,
                                }
                                .into()
                            } else {
                                missing
                            }
                        }),
                    None => missing,
                }
            }
        })
    }
    /// Binned scale cuts and vector-wide labels before color-step composition.
    /// Cuts keep their authored order and duplicates; the bin mapping owns selection.
    /// Registered labels receive even an empty vector here. Color-step composition
    /// separately omits empty guides and prepares censored callback inputs.
    pub fn binned_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        let labels = match self.spec.guide.as_deref() {
            Some(
                GgplotScaleGuide::Binned(labels)
                | GgplotScaleGuide::BinnedLegend(labels)
                | GgplotScaleGuide::BinnedBins(labels)
                | GgplotScaleGuide::BinnedSteps(labels),
            ) => labels,
            Some(GgplotScaleGuide::TemporalBins(g) | GgplotScaleGuide::TemporalSteps(g)) => {
                g.interval_labels()
            }
            _ => &GgplotGuideLabels::Automatic,
        };
        let mut entries = self.binned_guide_entries_using(
            budget,
            label_budget,
            if matches!(labels, GgplotGuideLabels::Registered { .. }) {
                &GgplotGuideLabels::Hidden
            } else {
                labels
            },
        )?;
        self.map_binned_candidates(&mut entries)?;
        if matches!(labels, GgplotGuideLabels::Registered { .. }) {
            self.registered_guide_entries(entries, labels, label_budget, false)
        } else {
            Ok(entries)
        }
    }
    fn map_binned_candidates(
        &self,
        entries: &mut Option<Vec<GgplotContinuousGuideEntry>>,
    ) -> ChartResult<()> {
        if self.vector_pipeline.is_none()
            && !matches!(
                self.spec.guide.as_deref(),
                Some(GgplotScaleGuide::BinnedLegend(_))
            )
        {
            return Ok(());
        }
        if let Some(entries) = entries {
            let inputs = entries
                .iter()
                .map(|entry| Some(entry.value.0))
                .collect::<Vec<_>>();
            let batch = if matches!(
                self.spec.guide.as_deref(),
                Some(GgplotScaleGuide::ContinuousSteps(_) | GgplotScaleGuide::TemporalSteps(_))
            ) {
                self.transformed_palette_batch(
                    &self.interval_palette_values(
                        &entries
                            .iter()
                            .map(|entry| entry.transformed)
                            .collect::<Vec<_>>(),
                    ),
                )?
            } else if self
                .spec
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise())
            {
                self.transformed_palette_batch(
                    &entries
                        .iter()
                        .map(|entry| entry.transformed)
                        .collect::<Vec<_>>(),
                )?
            } else {
                self.palette_batch(&inputs)?
            };
            if let Some(batch) = batch {
                if batch.indices.len() != entries.len() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Binned pipeline output must match the guide candidate count.",
                    ));
                }
                let values = batch.values.ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "A NULL palette cannot train a visible binned guide.",
                    )
                })?;
                for (entry, i) in entries.iter_mut().zip(batch.indices) {
                    entry.mapped = Some(values[i].clone());
                }
            }
        }
        Ok(())
    }

    fn binned_color_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
        steps: &mut Vec<ColorGuideStep>,
        positions: &mut Vec<Option<Number>>,
        missing: Color,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedBins(_)
                    | GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
            )
        ) {
            return self.binned_value_guide_entries(budget, label_budget);
        }
        if matches!(
            self.spec.guide.as_deref(),
            Some(GgplotScaleGuide::BinnedLegend(_))
        ) {
            let mut entries = self.binned_guide_entries(budget, label_budget)?;
            if let Some(entries) = &mut entries {
                entries.retain(|e| e.visible);
            }
            return Ok(entries);
        }
        let labels = match self.spec.guide.as_deref() {
            Some(
                GgplotScaleGuide::Binned(labels)
                | GgplotScaleGuide::BinnedLegend(labels)
                | GgplotScaleGuide::BinnedBins(labels)
                | GgplotScaleGuide::BinnedSteps(labels),
            ) => labels,
            Some(GgplotScaleGuide::ContinuousBins(g) | GgplotScaleGuide::ContinuousSteps(g)) => {
                &g.labels
            }
            Some(GgplotScaleGuide::TemporalBins(g) | GgplotScaleGuide::TemporalSteps(g)) => {
                g.interval_labels()
            }
            _ => &GgplotGuideLabels::Automatic,
        };
        if self
            .spec
            .colorbar_options
            .as_deref()
            .is_some_and(|o| !o.even_steps)
        {
            let entries = if let Some(GgplotScaleGuide::ContinuousSteps(g)) =
                self.spec.guide.as_deref()
            {
                self.continuous_guide_entries_using(
                    budget,
                    label_budget,
                    Some(&GgplotScaleGuide::Continuous(g.clone())),
                )?
            } else if let Some(GgplotScaleGuide::TemporalSteps(g)) = self.spec.guide.as_deref() {
                self.continuous_guide_entries_using(
                    budget,
                    label_budget,
                    Some(&GgplotScaleGuide::Temporal(g.clone())),
                )?
            } else {
                self.binned_guide_entries(budget, label_budget)?
            };
            if let Some(entries) = &entries
                && entries.iter().any(|e| e.visible)
            {
                let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
                    unreachable!("interval mapping")
                };
                let NormalizationSpec::Ggplot {
                    domain,
                    ref family,
                    reverse,
                    ..
                } = scale.normalization
                else {
                    unreachable!("interval normalization")
                };
                let mut boundaries = self
                    .interval_bounds(domain, family, reverse)?
                    .into_iter()
                    .chain(entries.iter().map(|e| e.transformed.0))
                    .filter(|v| !v.is_nan())
                    .collect::<Vec<_>>();
                boundaries.sort_by(f64::total_cmp);
                boundaries.dedup_by(|a, b| *a == *b);
                let batch = self.interval_midpoint_batch(&boundaries, family, reverse)?;
                *steps = self.color_step_cells(&boundaries, batch, family, reverse, missing)?;
                if let (Some(first), Some(last)) = (boundaries.first(), boundaries.last()) {
                    *positions = entries
                        .iter()
                        .map(|e| {
                            let p = (e.transformed.0 - first) / (last - first);
                            (e.visible && p.is_finite() && (0. ..=1.).contains(&p))
                                .then_some(Number(p))
                        })
                        .collect();
                }
            }

            return Ok(entries);
        }
        if matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedSteps(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) {
            let mut entries =
                self.binned_key_candidates(budget, label_budget, labels, true, positions)?;
            if let Some(entries) = &mut entries {
                let mut index = 0;
                positions.retain(|_| {
                    let keep = !entries[index].value.0.is_nan();
                    index += 1;
                    keep
                });
                entries.retain(|e| !e.value.0.is_nan());
                if !entries.is_empty() {
                    let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
                        unreachable!("binned mapping")
                    };
                    let NormalizationSpec::Ggplot {
                        domain,
                        ref family,
                        reverse,
                        ..
                    } = scale.normalization
                    else {
                        unreachable!("binned normalization")
                    };
                    let bounds = self.interval_bounds(domain, family, reverse)?;
                    let mut boundaries = bounds
                        .into_iter()
                        .chain(entries.iter().map(|e| e.transformed.0))
                        .collect::<Vec<_>>();
                    boundaries.sort_by(f64::total_cmp);
                    boundaries.dedup_by(|a, b| *a == *b);
                    let batch = self.interval_midpoint_batch(&boundaries, family, reverse)?;
                    *steps = self.color_step_cells(&boundaries, batch, family, reverse, missing)?;
                    for entry in entries {
                        if bounds.contains(&entry.transformed.0) {
                            entry.mapped = Some(Value::Missing);
                        }
                    }
                }
            }
            return Ok(entries);
        }
        let named = matches!(self.spec.ggplot.as_deref(), Some(GgplotScalePolicy::Binned(p)) if p.prepared_break_names.is_some());
        if !matches!(labels, GgplotGuideLabels::Registered { .. })
            && !(named && matches!(labels, GgplotGuideLabels::Automatic))
        {
            let entries = self.binned_guide_entries(budget, label_budget)?;
            if let (Some(entries), PreparedMapping::Binned(mapping)) = (&entries, &self.mapping)
                && !mapping.bounds.is_empty()
            {
                if !entries.is_empty() {
                    mapping.validate_guide_mapping()?;
                    if mapping.bounds.len() == 1
                        && entries.len() > 1
                        && !matches!(
                            self.spec.guide.as_deref(),
                            Some(GgplotScaleGuide::Binned(_))
                        )
                    {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Binned color guide values exceed the constant mapped key count.",
                        ));
                    }
                }
                let mut keys = entries.clone();
                *positions = super::ggplot_bins::prepare_color_label_inputs(
                    &mut keys,
                    self.binned_guide_bounds()?,
                );
            }
            return Ok(entries);
        }
        self.binned_key_candidates(budget, label_budget, labels, true, positions)
    }
    fn binned_key_candidates(
        &self,
        budget: usize,
        label_budget: usize,
        labels: &GgplotGuideLabels,
        color: bool,
        positions: &mut Vec<Option<Number>>,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        let mut entries = if let Some(
            GgplotScaleGuide::ContinuousBins(g) | GgplotScaleGuide::ContinuousSteps(g),
        ) = self.spec.guide.as_deref()
        {
            let mut raw = g.clone();
            raw.labels = GgplotGuideLabels::Hidden;
            self.continuous_guide_entries_using(
                budget,
                label_budget,
                Some(&GgplotScaleGuide::Continuous(raw)),
            )?
        } else if let Some(GgplotScaleGuide::TemporalBins(g) | GgplotScaleGuide::TemporalSteps(g)) =
            self.spec.guide.as_deref()
        {
            let mut raw = g.clone();
            if !matches!(labels, GgplotGuideLabels::Automatic) {
                raw.arguments.labels = GgplotGuideLabels::Hidden;
                raw.arguments.format = None;
            }
            self.continuous_guide_entries_using(
                budget,
                label_budget,
                Some(&GgplotScaleGuide::Temporal(raw)),
            )?
        } else {
            self.binned_guide_entries_using(budget, label_budget, &GgplotGuideLabels::Hidden)?
        };
        if let Some(entries) = &mut entries {
            // Reference binned guides parse function/default-labelled cuts before get_labels:
            // drop missing transformed cuts, then censor only finite outsiders.
            // The standalone scale candidate API retains its original raw cuts.
            if matches!(
                labels,
                GgplotGuideLabels::Registered { .. } | GgplotGuideLabels::Automatic
            ) {
                entries.retain(|entry| !entry.transformed.0.is_nan());
            }
            // Typed Date/POSIX break vectors are non-numeric to the reference parser,
            // so both automatic and explicitly selected temporal endpoints are masked.
            let mask_endpoints = match self.spec.guide.as_deref() {
                Some(
                    GgplotScaleGuide::ContinuousBins(g) | GgplotScaleGuide::ContinuousSteps(g),
                ) => g.breaks.is_none(),
                Some(GgplotScaleGuide::TemporalBins(_) | GgplotScaleGuide::TemporalSteps(_)) => {
                    true
                }
                _ => false,
            };
            if mask_endpoints {
                let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
                    unreachable!("interval scale")
                };
                let NormalizationSpec::Ggplot {
                    domain,
                    ref family,
                    reverse,
                    ..
                } = scale.normalization
                else {
                    unreachable!("interval normalization")
                };
                let bounds = self.interval_bounds(domain, family, reverse)?;
                for entry in entries.iter_mut() {
                    if bounds.contains(&entry.transformed.0) {
                        entry.value = Number(super::ggplot_palette::missing_number());
                        entry.transformed = Number(super::ggplot_palette::missing_number());
                        entry.label = None;
                        entry.visible = false;
                    }
                }
            }
            if !entries.is_empty()
                && let PreparedMapping::Binned(mapping) = &self.mapping
            {
                mapping.validate_guide_mapping()?;
                if color && mapping.bounds.len() == 1 && entries.len() > 1 {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Binned color guide values exceed the constant mapped key count.",
                    ));
                }
            }
            let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
                unreachable!("validated binned range")
            };
            let NormalizationSpec::Ggplot {
                domain,
                ref family,
                reverse,
                ..
            } = scale.normalization
            else {
                unreachable!("validated binned normalization")
            };
            let bounds = self.interval_bounds(domain, family, reverse)?;
            *positions = super::ggplot_bins::prepare_color_label_inputs(entries, bounds);
            let names = entries
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<Option<Vec<_>>>();
            if let Some(names) = names.filter(|_| {
                !entries.is_empty()
                    && matches!(labels, GgplotGuideLabels::Automatic)
                    && !self
                        .spec
                        .guide
                        .as_deref()
                        .and_then(GgplotScaleGuide::temporal)
                        .is_some_and(|g| g.arguments.format.is_some())
            }) {
                super::ggplot_continuous_guide::apply_break_names(
                    entries,
                    &names,
                    true,
                    label_budget,
                )?;
            } else if self
                .spec
                .guide
                .as_deref()
                .and_then(GgplotScaleGuide::temporal)
                .is_some()
                && matches!(labels, GgplotGuideLabels::Automatic)
            {
                // Keep the common calendar selector's pretty labels after censoring.
            } else if !matches!(labels, GgplotGuideLabels::Registered { .. }) {
                *entries = super::ggplot_continuous_guide::numeric_guide_entries(
                    entries
                        .iter()
                        .map(|entry| {
                            if entry.value.0.is_nan() {
                                f64::NAN
                            } else {
                                entry.transformed.0
                            }
                        })
                        .collect(),
                    bounds,
                    family.clone(),
                    reverse,
                    labels,
                    label_budget,
                )?;
            }
        }
        let steps = matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedSteps(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        );
        if steps {
            self.map_binned_candidates(&mut entries)?;
        }
        let mut entries = if matches!(labels, GgplotGuideLabels::Registered { .. })
            && !matches!(
                self.spec.guide.as_deref(),
                Some(
                    GgplotScaleGuide::BinnedBins(_)
                        | GgplotScaleGuide::ContinuousBins(_)
                        | GgplotScaleGuide::TemporalBins(_)
                )
            ) {
            self.registered_guide_entries(entries, labels, label_budget, true)?
        } else {
            entries
        };
        if !steps
            && !matches!(
                self.spec.guide.as_deref(),
                Some(
                    GgplotScaleGuide::BinnedBins(_)
                        | GgplotScaleGuide::ContinuousBins(_)
                        | GgplotScaleGuide::TemporalBins(_)
                )
            )
        {
            self.map_binned_candidates(&mut entries)?;
        }
        Ok(entries)
    }

    /// Non-color binned guide keys, including separately labelled limits.
    /// Entries retain source/scale values; guide composition places keys at equally
    /// spaced interval boundaries and owns endpoint visibility.
    pub fn binned_value_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedSteps(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) {
            return Ok(Some(vec![]));
        }
        if matches!(
            self.spec.guide.as_deref(),
            Some(GgplotScaleGuide::BinnedLegend(_))
        ) {
            let mut entries = self.binned_guide_entries(budget, label_budget)?;
            if let Some(entries) = &mut entries {
                entries.retain(|e| e.visible);
            }
            return Ok(entries);
        }
        let labels = match self.spec.guide.as_deref() {
            Some(
                GgplotScaleGuide::Binned(labels)
                | GgplotScaleGuide::BinnedLegend(labels)
                | GgplotScaleGuide::BinnedBins(labels)
                | GgplotScaleGuide::BinnedSteps(labels),
            ) => labels,
            Some(GgplotScaleGuide::ContinuousBins(g) | GgplotScaleGuide::ContinuousSteps(g)) => {
                &g.labels
            }
            Some(GgplotScaleGuide::TemporalBins(g) | GgplotScaleGuide::TemporalSteps(g)) => {
                g.interval_labels()
            }
            _ => &GgplotGuideLabels::Automatic,
        };
        let bins = matches!(
            self.spec.guide.as_deref(),
            None | Some(GgplotScaleGuide::Binned(_))
        ) || matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedBins(_)
                    | GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
            )
        );
        let defer_labels = matches!(
            self.spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedBins(_)
                    | GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
            )
        ) && matches!(labels, GgplotGuideLabels::Registered { .. });
        let Some(mut entries) =
            self.binned_key_candidates(budget, label_budget, labels, false, &mut vec![])?
        else {
            return Ok(None);
        };
        if entries.is_empty() {
            return Ok(Some(entries));
        }
        let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
            unreachable!("validated binned range")
        };
        let NormalizationSpec::Ggplot {
            domain,
            ref family,
            reverse,
            ..
        } = scale.normalization
        else {
            unreachable!("validated binned normalization")
        };
        let bounds = self.interval_bounds(domain, family, reverse)?;
        let mut boundaries = bounds
            .into_iter()
            .chain(
                entries
                    .iter()
                    .filter(|entry| !entry.value.0.is_nan())
                    .map(|entry| entry.transformed.0),
            )
            .filter(|v| !v.is_nan())
            .collect::<Vec<_>>();
        boundaries.sort_by(f64::total_cmp);
        boundaries.dedup_by(|a, b| *a == *b);
        // A constant map returns one value even for zero interval midpoints;
        // GuideBins appends a final missing key, producing two equal boundaries.
        if boundaries.len() == 1
            && !matches!(
                self.spec.guide.as_deref(),
                Some(GgplotScaleGuide::ContinuousBins(_) | GgplotScaleGuide::TemporalBins(_))
            )
        {
            boundaries.push(boundaries[0]);
        }
        let mapped = if bins {
            let mut values =
                if let Some(batch) = self.interval_midpoint_batch(&boundaries, family, reverse)? {
                    let values = batch.values.ok_or_else(|| {
                        error(
                            DiagnosticCode::Validation,
                            "A NULL palette cannot train a visible binned guide.",
                        )
                    })?;
                    batch
                        .indices
                        .into_iter()
                        .map(|i| values[i].clone())
                        .collect::<Vec<_>>()
                } else {
                    unreachable!("binned scales always retain a palette batch")
                };
            values.push(Value::Missing);
            if values.len() != boundaries.len() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Binned guide mapping and boundaries have incompatible lengths.",
                ));
            }
            Some(values)
        } else {
            None
        };
        if defer_labels {
            entries = self
                .registered_guide_entries(Some(entries), labels, label_budget, true)?
                .expect("binned key vector");
        }
        let endpoint_labels = match labels {
            GgplotGuideLabels::Automatic => labels,
            _ => &GgplotGuideLabels::Hidden,
        };
        let temporal_endpoint_labels = self
            .spec
            .guide
            .as_deref()
            .and_then(GgplotScaleGuide::temporal)
            .filter(|_| matches!(endpoint_labels, GgplotGuideLabels::Automatic))
            .map(|g| {
                g.interval_endpoint_labels(&bounds)
                    .map(GgplotGuideLabels::Explicit)
            })
            .transpose()?;
        let endpoint_labels = temporal_endpoint_labels.as_ref().unwrap_or(endpoint_labels);
        let endpoint_entries = super::ggplot_continuous_guide::numeric_guide_entries(
            bounds.to_vec(),
            bounds,
            family.clone(),
            reverse,
            endpoint_labels,
            label_budget,
        )?;
        let endpoint_entries = if matches!(labels, GgplotGuideLabels::Registered { .. }) {
            self.registered_guide_entries(Some(endpoint_entries), labels, label_budget, false)?
                .expect("endpoint vector")
        } else {
            endpoint_entries
        };
        entries.retain(|entry| !entry.value.0.is_nan());
        if entries.is_empty() {
            return Err(error(
                DiagnosticCode::Validation,
                "No binned guide breaks remain after censoring.",
            ));
        }
        let prepend = !bounds.contains(&entries[0].transformed.0);
        let append = !bounds.contains(&entries.last().expect("retained breaks").transformed.0);
        if prepend {
            entries.insert(0, endpoint_entries[0].clone());
        }
        if append {
            entries.push(endpoint_entries[1].clone());
        }
        if entries.is_empty() || !boundaries.len().is_multiple_of(entries.len()) {
            return Err(error(
                DiagnosticCode::Validation,
                "Binned guide keys and labels have incompatible lengths.",
            ));
        }
        crate::limits::require_within(boundaries.len() <= budget, "binned guide")?;
        // A continuous zero-width interval has key position 0/0. The reference
        // removes that key after evaluating break and endpoint label callbacks.
        if boundaries.len() == 1
            && matches!(
                self.spec.guide.as_deref(),
                Some(GgplotScaleGuide::ContinuousBins(_) | GgplotScaleGuide::TemporalBins(_))
            )
        {
            return Ok(Some(vec![]));
        }
        let labels = entries
            .iter()
            .map(|entry| entry.label.clone())
            .collect::<Vec<_>>();
        Ok(Some(
            boundaries
                .into_iter()
                .enumerate()
                .map(|(i, transformed)| GgplotContinuousGuideEntry {
                    mapped: mapped.as_ref().map(|v| v[i].clone()),
                    name: None,
                    value: if matches!(
                        self.spec.guide.as_deref(),
                        Some(
                            GgplotScaleGuide::ContinuousBins(_) | GgplotScaleGuide::TemporalBins(_)
                        )
                    ) {
                        entries[i % entries.len()].value
                    } else {
                        Number(super::ggplot_continuous_guide::inverse(
                            family,
                            reverse,
                            transformed,
                        ))
                    },
                    transformed: Number(transformed),
                    label: labels[i % labels.len()].clone(),
                    visible: true,
                })
                .collect(),
        ))
    }
    fn binned_guide_entries_using(
        &self,
        budget: usize,
        label_budget: usize,
        labels: &GgplotGuideLabels,
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
            ref family,
            domain,
            reverse,
            ..
        } = s.normalization
        else {
            unreachable!("validated binned normalization")
        };
        if !super::ggplot_continuous_guide::comparable_limits(policy.limits, family.clone())
            .is_some_and(|limits| limits.iter().all(Option::is_some))
        {
            if policy.empty_population && self.spec.resolved_numeric_limits.is_none() {
                if policy.limits.is_some() {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Empty binned populations cannot replace missing authored limits.",
                    ));
                }
                if matches!(labels, GgplotGuideLabels::Automatic) {
                    super::ggplot_continuous_guide::transform_numeric_labels(
                        family.ggplot_transform(),
                        &[],
                        label_budget,
                    )?;
                }
                return Ok(Some(vec![]));
            }
            if policy.nonfinite_population {
                let mut entries = policy.nonfinite_guide_entries(
                    family.clone(),
                    reverse,
                    labels,
                    budget,
                    label_budget,
                )?;
                if let Some(names) = &policy.prepared_break_names {
                    super::ggplot_continuous_guide::apply_break_names(
                        &mut entries,
                        names,
                        matches!(labels, GgplotGuideLabels::Automatic),
                        label_budget,
                    )?;
                }
                return Ok(Some(entries));
            }
        }
        let (domain, cuts) = if let Some(values) = &self.spec.resolved_numeric_limits {
            policy.resolve(
                [
                    *values.first().unwrap_or(&Number(f64::NAN)),
                    *values.get(1).unwrap_or(&Number(f64::NAN)),
                ],
                family.clone(),
                reverse,
            )?
        } else if let Some(cuts) = &policy.prepared_breaks {
            (domain, cuts.clone())
        } else {
            policy.resolve(domain, family.clone(), reverse)?
        };
        if self.spec.palette_function.is_some()
            && !cuts.is_empty()
            && let PreparedMapping::Binned(mapping) = &self.mapping
        {
            mapping.validate_guide_mapping()?;
        }
        if cuts.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Binned guide exceeds the tick budget.",
            ));
        }
        let forward = |v: Number| super::ggplot_continuous_guide::forward(family, reverse, v.0);
        let bounds = self
            .spec
            .trained_transformed_bounds
            .map_or_else(|| domain.map(forward), |v| v.map(|v| v.0));
        let transformed_cuts = super::ggplot_continuous_guide::forward_values(
            family,
            reverse,
            &cuts.iter().map(|v| v.0).collect::<Vec<_>>(),
        )?;
        let mut entries = super::ggplot_continuous_guide::numeric_guide_entries(
            transformed_cuts,
            bounds,
            family.clone(),
            reverse,
            labels,
            label_budget,
        )?;
        if let Some(names) = &policy.prepared_break_names {
            super::ggplot_continuous_guide::apply_break_names(
                &mut entries,
                names,
                matches!(labels, GgplotGuideLabels::Automatic),
                label_budget,
            )?;
        }
        Ok(Some(entries))
    }
    /// Reference continuous guide candidates from this exact prepared scale.
    /// Outside candidates remain available because they affect vector-wide labels.
    pub fn continuous_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        let mut entries = self.selected_continuous_guide_entries(budget, label_budget)?;
        if matches!(self.mapping, PreparedMapping::GgplotNumericIdentity(_))
            && self
                .spec
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise())
        {
            if let Some(entries) = &mut entries {
                for entry in entries {
                    entry.mapped = Some(Value::Number(entry.transformed));
                }
            }
            return Ok(entries);
        }
        if let Some(entries) = &mut entries {
            let inputs = entries
                .iter()
                .map(|entry| Some(entry.value.0))
                .collect::<Vec<_>>();
            let batch = if self
                .spec
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise())
            {
                self.transformed_palette_batch(
                    &entries
                        .iter()
                        .map(|entry| entry.transformed)
                        .collect::<Vec<_>>(),
                )?
            } else {
                self.palette_batch(&inputs)?
            };
            if let Some(batch) = batch {
                let values = batch.values.ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "A NULL continuous palette cannot supply guide keys.",
                    )
                })?;
                if batch.indices.len() != entries.len() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Continuous pipeline output must match the guide candidate count.",
                    ));
                }
                for (entry, i) in entries.iter_mut().zip(batch.indices) {
                    entry.mapped = Some(values[i].clone());
                }
            }
        }
        Ok(entries)
    }
    fn selected_continuous_guide_entries(
        &self,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if !matches!(
            self.spec.continuous_guide().map(|g| &g.labels),
            Some(GgplotGuideLabels::Registered { .. })
        ) {
            return self.continuous_guide_entries_using(
                budget,
                label_budget,
                self.spec.guide.as_deref(),
            );
        }
        let mut guide = self.spec.guide.as_deref().cloned();
        let labels = if let Some(
            GgplotScaleGuide::Continuous(guide)
            | GgplotScaleGuide::Colorbar(guide)
            | GgplotScaleGuide::ContinuousBins(guide)
            | GgplotScaleGuide::ContinuousSteps(guide),
        ) = &mut guide
            && matches!(guide.labels, GgplotGuideLabels::Registered { .. })
        {
            Some(std::mem::replace(
                &mut guide.labels,
                GgplotGuideLabels::Hidden,
            ))
        } else {
            None
        };
        let entries = self.continuous_guide_entries_using(budget, label_budget, guide.as_ref())?;
        if let Some(labels) = labels {
            self.registered_guide_entries(entries, &labels, label_budget, true)
        } else {
            Ok(entries)
        }
    }
    fn registered_guide_entries(
        &self,
        mut entries: Option<Vec<GgplotContinuousGuideEntry>>,
        labels: &GgplotGuideLabels,
        label_budget: usize,
        skip_empty: bool,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if let Some(entries) = &mut entries {
            if skip_empty && entries.is_empty() {
                return Ok(Some(vec![]));
            }
            let values = entries
                .iter()
                .map(|entry| crate::composition::ScaleValue::Number(entry.value.0))
                .collect::<Vec<_>>();
            let names = entries
                .iter()
                .map(|e| e.name.clone())
                .collect::<Option<Vec<_>>>();
            let labels = labels.registered_values(
                &values,
                names.as_deref(),
                self.spec
                    .guide
                    .as_deref()
                    .and_then(GgplotScaleGuide::temporal)
                    .map(|g| crate::grammar::GuideTemporalContext {
                        normalization: GgplotTimestampNormalization {
                            origin: g.origin,
                            unit: g.unit,
                            date: g.arguments.date,
                        },
                        zone: &g.zone,
                    }),
                &self.registry,
                label_budget,
            )?;
            if labels.len() != entries.len() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Breaks and labels have different lengths.",
                ));
            }
            for (entry, label) in entries.iter_mut().zip(labels) {
                entry.label = label;
            }
        }
        Ok(entries)
    }
    fn continuous_guide_entries_using(
        &self,
        budget: usize,
        label_budget: usize,
        guide: Option<&GgplotScaleGuide>,
    ) -> ChartResult<Option<Vec<GgplotContinuousGuideEntry>>> {
        if !self.spec.reference_guides() {
            return Ok(None);
        }
        if matches!(guide, Some(GgplotScaleGuide::Hidden)) {
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
                    s.spec().family.clone(),
                    false,
                )
            }
            PreparedMapping::Interpolated(s) => match &s.spec().normalization {
                NormalizationSpec::Sequential { family, domain, .. } => {
                    (*domain, family.clone(), false)
                }
                NormalizationSpec::Ggplot {
                    family,
                    domain,
                    reverse,
                    ..
                } => (*domain, family.clone(), *reverse),
                NormalizationSpec::Diverging { family, domain, .. } => {
                    ([domain[0], domain[2]], family.clone(), false)
                }
                NormalizationSpec::Quantile { .. } => return Ok(None),
            },
            PreparedMapping::GgplotNumericIdentity(s) => {
                if !s.guide {
                    return Ok(Some(vec![]));
                }
                let transformed = s.domain()?.map(|v| v.0);
                let domain = if let Some(ScaleTransform::Ggplot { transform }) = &s.transform {
                    let values = transform.inverse_population(&transformed)?;
                    [Number(values[0]), Number(values[1])]
                } else {
                    transformed
                        .map(|v| Number(s.transform.as_ref().map_or(v, |t| t.inverse_raw(v))))
                };
                let family = match &s.transform {
                    Some(ScaleTransform::Ggplot { transform }) => NumericFamily::Ggplot {
                        transform: transform.clone(),
                    },
                    Some(ScaleTransform::Log { base }) => NumericFamily::Log { base: *base },
                    Some(ScaleTransform::Sqrt) => NumericFamily::Pow { exponent: 0.5 },
                    Some(ScaleTransform::Symlog { threshold }) => NumericFamily::Symlog {
                        constant: *threshold,
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
            if let Some(
                GgplotScaleGuide::Temporal(guide)
                | GgplotScaleGuide::TemporalColorbar(guide)
                | GgplotScaleGuide::TemporalBins(guide)
                | GgplotScaleGuide::TemporalSteps(guide),
            ) = guide
            {
                return self
                    .resolve_temporal_breaks(guide, domain.map(|v| v.0), budget, label_budget)
                    .map(Some);
            }
            let default_guide = GgplotContinuousGuide::default();
            let guide = match guide {
                Some(
                    GgplotScaleGuide::Continuous(g)
                    | GgplotScaleGuide::Colorbar(g)
                    | GgplotScaleGuide::ContinuousBins(g)
                    | GgplotScaleGuide::ContinuousSteps(g),
                ) => g,
                _ => &default_guide,
            };
            if values.len() == 1 {
                crate::limits::require_within(budget > 0, "continuous guide")?;
                let value = super::ggplot_continuous_guide::forward(&family, reverse, values[0].0);
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
            return self
                .resolve_continuous_breaks(
                    guide,
                    super::ggplot_continuous_guide::forward_values(
                        &family,
                        reverse,
                        &domain.map(|v| v.0),
                    )?
                    .try_into()
                    .expect("two transformed bounds"),
                    family,
                    reverse,
                    budget,
                    label_budget,
                )
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
            && !super::ggplot_continuous_guide::finite_limits(limits, family.clone(), reverse)
        {
            if guide.is_none()
                || matches!(guide, Some(GgplotScaleGuide::Continuous(g) | GgplotScaleGuide::Colorbar(g) | GgplotScaleGuide::ContinuousBins(g) | GgplotScaleGuide::ContinuousSteps(g)) if matches!(g.labels, GgplotGuideLabels::Automatic))
            {
                super::ggplot_continuous_guide::transform_numeric_labels(
                    family.ggplot_transform(),
                    &[],
                    label_budget,
                )?;
            }
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
        if let Some(
            GgplotScaleGuide::Temporal(guide)
            | GgplotScaleGuide::TemporalColorbar(guide)
            | GgplotScaleGuide::TemporalBins(guide)
            | GgplotScaleGuide::TemporalSteps(guide),
        ) = guide
        {
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
            return self
                .resolve_temporal_breaks(guide, bounds, budget, label_budget)
                .map(Some);
        }
        let default_guide = GgplotContinuousGuide::default();
        let guide = match guide {
            Some(
                GgplotScaleGuide::Continuous(g)
                | GgplotScaleGuide::Colorbar(g)
                | GgplotScaleGuide::ContinuousBins(g)
                | GgplotScaleGuide::ContinuousSteps(g),
            ) => g,
            _ => &default_guide,
        };
        let bounds = nonfinite_limits.map_or_else(
            || domain.map(|v| super::ggplot_continuous_guide::forward(&family, reverse, v.0)),
            |limits| {
                super::ggplot_continuous_guide::authored_bounds(
                    limits,
                    family.clone(),
                    reverse,
                    [f64::INFINITY, f64::NEG_INFINITY],
                )
            },
        );
        let bounds = if let PreparedMapping::GgplotNumericIdentity(identity) = &self.mapping
            && self
                .spec
                .ggplot_transform()
                .is_some_and(|t| !t.is_pointwise())
        {
            identity.domain()?.map(|v| v.0)
        } else {
            self.spec
                .trained_transformed_bounds
                .map_or(bounds, |v| v.map(|v| v.0))
        };
        self.resolve_continuous_breaks(guide, bounds, family, reverse, budget, label_budget)
            .map(Some)
    }
    fn resolve_temporal_breaks(
        &self,
        guide: &GgplotTemporalGuide,
        bounds: [f64; 2],
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        match &self.spec.breaks_function {
            Some(call) => guide.resolve_breaks_using(
                bounds,
                budget,
                label_budget,
                &self.registry,
                &self.breaks_registry,
                call,
            ),
            None => guide.resolve_using(bounds, budget, label_budget, &self.registry),
        }
    }
    fn resolve_continuous_breaks(
        &self,
        guide: &GgplotContinuousGuide,
        bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        let censor_labels = !matches!(self.mapping, PreparedMapping::GgplotNumericIdentity(_));
        let Some(call) = &self.spec.breaks_function else {
            return guide.resolve_bounds_with_label_censor(
                bounds,
                family,
                reverse,
                budget,
                label_budget,
                censor_labels,
            );
        };
        if super::ggplot::zero_range(bounds[0], bounds[1]) {
            return guide.resolve_bounds_with_label_censor(
                bounds,
                family,
                reverse,
                budget,
                label_budget,
                censor_labels,
            );
        }
        let domain = super::ggplot_continuous_guide::inverse_values(&family, reverse, &bounds)?
            .into_iter()
            .map(|v| ScaleKey::Number(Number(v)))
            .collect::<Vec<_>>();
        let output = self.breaks_registry.evaluate(call, &domain, guide.count)?;
        if output.values.is_none() {
            super::ggplot_continuous_guide::forward_null(&family, reverse)?;
        }
        let values = output.values.unwrap_or_default();
        crate::limits::require_within(values.len() <= budget, "continuous break function")?;
        let values = values
            .into_iter()
            .map(|v| match v {
                ScaleKey::Number(v) => Ok(v.0),
                ScaleKey::Null => Ok(f64::NAN),
                _ => Err(error(
                    DiagnosticCode::Validation,
                    "Continuous break functions must return numeric values.",
                )),
            })
            .collect::<ChartResult<Vec<_>>>()?;
        let values = super::ggplot_continuous_guide::forward_values(&family, reverse, &values)?;
        let mut entries = super::ggplot_continuous_guide::numeric_guide_entries_with_label_censor(
            values,
            bounds,
            family,
            reverse,
            &guide.labels,
            label_budget,
            censor_labels,
        )?;
        if let Some(names) = output.names {
            super::ggplot_continuous_guide::apply_break_names(
                &mut entries,
                &names,
                matches!(guide.labels, GgplotGuideLabels::Automatic),
                label_budget,
            )?;
        }
        Ok(entries)
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
            let color = if let Some(value) = &entry.mapped {
                reference_value_paint(value, true)?.map_or(missing, crate::color::Paint::resolve)
            } else {
                self.color(Some(entry.value.0), None, missing)?
            };
            entries.push((label, color));
        }
        Ok(())
    }
    /// Selected discrete keys and vector-wide labels from the prepared mapping.
    /// Hidden and disabled identity guides return no entries and invoke no formatter.
    pub fn discrete_guide_entries(&self) -> ChartResult<Option<Vec<GgplotDiscreteGuideEntry>>> {
        let entries = self.selected_discrete_guide_entries()?;
        if self.spec.palette_function.is_some() {
            for entry in entries.iter().flatten() {
                self.pooled_value(self.index(None, Some(&entry.key)))?;
            }
        }
        Ok(entries)
    }
    fn selected_discrete_guide_entries(
        &self,
    ) -> ChartResult<Option<Vec<GgplotDiscreteGuideEntry>>> {
        if !self.spec.reference_guides()
            || matches!(self.spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden))
        {
            return Ok(None);
        }
        if self
            .spec
            .discrete_guide()
            .is_some_and(|guide| guide.breaks.as_ref().is_some_and(Vec::is_empty))
        {
            return Ok(Some(vec![]));
        }
        let domain = match &self.mapping {
            PreparedMapping::Ordinal(scale) => scale.spec().domain.clone(),
            PreparedMapping::GgplotDiscreteIdentity(_) => {
                let ScaleFunctionSpec::GgplotDiscreteIdentity(scale) = &self.spec.function else {
                    unreachable!("prepared identity mapping")
                };
                if !scale.guide {
                    return Ok(None);
                }
                scale.domain()?
            }
            _ => return Ok(None),
        };
        let default = GgplotDiscreteGuide::default();
        let guide = self.spec.discrete_guide().unwrap_or(&default);
        if let Some(call) = &self.spec.breaks_function {
            if domain.is_empty()
                && !(self.spec.limits_function.is_some()
                    && matches!(self.mapping, PreparedMapping::GgplotDiscreteIdentity(_)))
            {
                return Ok(Some(vec![]));
            }
            let result = self.breaks_registry.evaluate_optional(
                call,
                (!self.spec.resolved_discrete_limits_null).then_some(domain.as_slice()),
                None,
            )?;
            let mut selection = guide.clone();
            selection.breaks = Some(result.values.unwrap_or_default());
            selection.break_names = result.names;
            return selection
                .resolve_break_function_using(&domain, &self.registry)
                .map(Some);
        }
        guide.resolve_using(&domain, &self.registry).map(Some)
    }
    fn discrete_entries(
        &self,
        domain: &[ScaleKey],
        missing: Color,
        entries: &mut Vec<(String, Color)>,
        keys: &mut Vec<ScaleKey>,
    ) -> ChartResult<()> {
        if self.spec.reference_guides() {
            for entry in self.discrete_guide_entries()?.unwrap_or_default() {
                keys.push(entry.key.clone());
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
                keys.push(key.clone());
                entries.push((key_label(key), self.color(None, Some(key), missing)?));
            }
        }
        Ok(())
    }
    fn guide_paint(&self, value: &Value, missing: Color) -> ChartResult<Color> {
        let paint = reference_value_paint(value, true)?;
        Ok(self.guide_color(paint, missing))
    }
    fn guide_color(&self, paint: Option<crate::color::Paint>, missing: Color) -> Color {
        let mut color = paint.map_or(missing, crate::color::Paint::resolve);
        if let Some(alpha) = self.spec.colorbar_options.as_deref().and_then(|o| o.alpha)
            && (paint.is_some() || !self.spec.missing_paint_is_na)
        {
            color.alpha = crate::color::d65::alpha_byte(alpha.0);
        }
        color
    }
    fn color_step_cells(
        &self,
        boundaries: &[f64],
        batch: Option<super::ggplot_palette::PaletteBatch<Value>>,
        family: &NumericFamily,
        reverse: bool,
        missing: Color,
    ) -> ChartResult<Vec<ColorGuideStep>> {
        let count = boundaries.len().saturating_sub(1);
        crate::limits::require_within(
            count <= crate::interpolate::MAX_VALUES,
            "color guide interval",
        )?;
        let mapped = if let Some(batch) = batch {
            if batch.indices.len() != count {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Color-step output must match the interval count.",
                ));
            }
            let values = batch.values.ok_or_else(|| {
                error(
                    DiagnosticCode::Validation,
                    "A NULL palette cannot supply a stepped guide.",
                )
            })?;
            Some((values, batch.indices))
        } else {
            None
        };
        boundaries
            .windows(2)
            .enumerate()
            .filter(|(_, v)| v[0] < v[1])
            .map(|(i, v)| {
                let color = if let Some((values, indices)) = &mapped {
                    self.guide_paint(&values[indices[i]], missing)?
                } else {
                    let input = Some(super::ggplot_continuous_guide::inverse(
                        family,
                        reverse,
                        v[0].midpoint(v[1]),
                    ));
                    let paint = self.paint(input, None, missing.into())?;
                    self.guide_color((!self.missing_paint(input, None)).then_some(paint), missing)
                };
                Ok(ColorGuideStep {
                    start: Number(v[0]),
                    end: Number(v[1]),
                    color,
                })
            })
            .collect()
    }
    /// Guide entries and interval metadata from the same prepared mapping as marks.
    fn colorbar_samples(
        &self,
        has_keys: bool,
        missing: Color,
    ) -> ChartResult<Vec<ColorGuideSample>> {
        if !has_keys
            || !matches!(
                self.spec.guide.as_deref(),
                Some(GgplotScaleGuide::Colorbar(_) | GgplotScaleGuide::TemporalColorbar(_))
            )
        {
            return Ok(vec![]);
        }
        let PreparedMapping::Interpolated(scale) = &self.mapping else {
            unreachable!("validated colorbar normalization")
        };
        let pipeline = self
            .vector_pipeline
            .as_ref()
            .expect("validated colorbar pipeline");
        let limits = pipeline.limits(scale.normalizer())?;
        let lower = limits.first().map_or(f64::NAN, |v| v.0);
        let upper = limits.get(1).map_or(lower, |v| v.0);
        if !lower.is_finite() || !upper.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Colorbar limits must be finite.",
            ));
        }
        let defaults = GgplotColorbarOptions::default();
        let options = self.spec.colorbar_options.as_deref().unwrap_or(&defaults);
        let count = options.sample_count()?;
        // Zero-length reference sequences fall back to unique scale limits.
        let count = if options.nbin() == 0. && lower == upper {
            1
        } else {
            count
        };
        let values = (0..count)
            .map(|i| {
                Number(if count == 1 {
                    lower
                } else if i == count - 1 {
                    upper
                } else {
                    lower + (upper - lower) * (i as f64 / (count - 1) as f64)
                })
            })
            .collect::<Vec<_>>();
        let batch = self
            .transformed_palette_batch(&values)?
            .expect("colorbar palette batch");
        let paints = batch.values.ok_or_else(|| {
            error(
                DiagnosticCode::Validation,
                "A NULL palette cannot supply a colorbar.",
            )
        })?;
        if batch.indices.len() != values.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Colorbar output must match the ramp sample count.",
            ));
        }
        let mut samples = values
            .into_iter()
            .zip(batch.indices)
            .map(|(value, index)| {
                let color = self.guide_paint(&paints[index], missing)?;
                Ok(ColorGuideSample { value, color })
            })
            .collect::<ChartResult<Vec<_>>>()?;
        if options.reverse {
            samples.reverse();
        }
        Ok(samples)
    }
    /// Guide entries and retained ramp samples from the same prepared mapping as marks.
    pub fn legend(&self, id: crate::ScaleId, missing: Color) -> ChartResult<ColorLegend> {
        let missing = if self.spec.missing_paint_is_na {
            Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            }
        } else {
            missing
        };
        let mut entries = vec![];
        let mut discrete_keys = vec![];
        let mut intervals = vec![];
        let mut midpoint = None;
        let mut colorsteps = vec![];
        let mut colorstep_positions = vec![];
        let mut numeric_breaks = if let Some(candidates) = self.binned_color_guide_entries(
            crate::interpolate::MAX_VALUES,
            1_048_576,
            &mut colorsteps,
            &mut colorstep_positions,
            missing,
        )? {
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
                        self.discrete_entries(
                            &s.domain()?,
                            missing,
                            &mut entries,
                            &mut discrete_keys,
                        )?;
                    }
                }
                PreparedMapping::Binned(_)
                    if matches!(
                        self.spec.guide.as_deref(),
                        Some(GgplotScaleGuide::BinnedLegend(_))
                    ) =>
                {
                    self.continuous_entries(&numeric_breaks, missing, &mut entries)?;
                }
                PreparedMapping::Binned(_) if numeric_breaks.is_empty() => {}
                PreparedMapping::Binned(s) => {
                    let paints = if let Some(values) = s.pipeline_palette() {
                        values
                            .iter()
                            .map(|value| reference_value_paint(value, true))
                            .collect::<ChartResult<Vec<_>>>()?
                    } else {
                        s.range()
                            .iter()
                            .map(|index| self.paints.as_ref().and_then(|p| p[*index]))
                            .collect()
                    };
                    let guide_bounds = self.binned_guide_bounds()?;
                    let count = paints.len();
                    for (i, paint) in paints.into_iter().enumerate() {
                        let a = s.bounds[i.min(s.bounds.len() - 1)].0;
                        let b = s.bounds[(i + 1).min(s.bounds.len() - 1)].0;
                        entries.push((
                            format!(
                                "{} – {}",
                                crate::number::ecmascript(a.min(b)),
                                crate::number::ecmascript(a.max(b))
                            ),
                            paint.map_or(missing, crate::color::Paint::resolve),
                        ));
                        intervals.push(GuideInterval {
                            lower: Some(ScaleKey::Number(Number(a.min(b)))),
                            upper: Some(ScaleKey::Number(Number(a.max(b)))),
                            entry: i,
                            closure: if i == 0 && s.right || i + 1 == count && !s.right {
                                IntervalClosure::Both
                            } else if s.right == (a <= b) {
                                IntervalClosure::Right
                            } else {
                                IntervalClosure::Left
                            },
                        });
                        // Guide cells reuse the classifier palette, but finite outside
                        // cuts collapse onto the scale limits. Infinite edge cells retain
                        // their cached missing paint and join the nearest limit.
                        let clip = |value: f64| {
                            if value.is_finite() && guide_bounds.iter().all(|v| v.is_finite()) {
                                value.clamp(
                                    guide_bounds[0].min(guide_bounds[1]),
                                    guide_bounds[0].max(guide_bounds[1]),
                                )
                            } else {
                                value
                            }
                        };
                        let (a, b) = (clip(a), clip(b));
                        if matches!(
                            self.spec.guide.as_deref(),
                            None | Some(GgplotScaleGuide::Binned(_))
                        ) && !self
                            .spec
                            .colorbar_options
                            .as_deref()
                            .is_some_and(|o| !o.even_steps)
                            && !a.is_nan()
                            && !b.is_nan()
                            && a != b
                        {
                            colorsteps.push(ColorGuideStep {
                                start: Number(a),
                                end: Number(b),
                                color: self.guide_color(paint, missing),
                            });
                        }
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
                    self.discrete_entries(
                        &s.spec().domain,
                        missing,
                        &mut entries,
                        &mut discrete_keys,
                    )?;
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
        if !colorsteps.is_empty()
            && colorstep_positions.iter().any(Option::is_some)
            && self
                .spec
                .colorbar_options
                .as_deref()
                .is_some_and(|o| o.show_limits)
        {
            let labels = self
                .spec
                .guide
                .as_deref()
                .and_then(GgplotScaleGuide::labels)
                .unwrap_or(&GgplotGuideLabels::Automatic);
            if !matches!(
                labels,
                GgplotGuideLabels::Explicit(_) | GgplotGuideLabels::Named(_)
            ) {
                let ScaleFunctionSpec::Interpolated(scale) = &self.spec.function else {
                    unreachable!("interval mapping")
                };
                let NormalizationSpec::Ggplot {
                    domain,
                    ref family,
                    reverse,
                    ..
                } = scale.normalization
                else {
                    unreachable!("interval normalization")
                };
                let bounds = self.interval_bounds(domain, family, reverse)?;
                let temporal = self
                    .spec
                    .guide
                    .as_deref()
                    .and_then(GgplotScaleGuide::temporal)
                    .filter(|_| matches!(labels, GgplotGuideLabels::Automatic))
                    .map(|g| {
                        g.interval_endpoint_labels(&bounds)
                            .map(GgplotGuideLabels::Explicit)
                    })
                    .transpose()?;
                let mut endpoints = super::ggplot_continuous_guide::numeric_guide_entries(
                    bounds.to_vec(),
                    bounds,
                    family.clone(),
                    reverse,
                    temporal.as_ref().unwrap_or(
                        if matches!(labels, GgplotGuideLabels::Registered { .. }) {
                            &GgplotGuideLabels::Hidden
                        } else {
                            labels
                        },
                    ),
                    1_048_576,
                )?;
                if matches!(labels, GgplotGuideLabels::Registered { .. }) {
                    endpoints = self
                        .registered_guide_entries(Some(endpoints), labels, 1_048_576, false)?
                        .expect("limit labels");
                }
                for e in &mut endpoints {
                    e.mapped = Some(Value::Missing);
                }
                if colorstep_positions.first().copied().flatten() != Some(Number(0.)) {
                    numeric_breaks.insert(0, endpoints[0].clone());
                    colorstep_positions.insert(0, Some(Number(0.)));
                }
                if colorstep_positions.last().copied().flatten() != Some(Number(1.)) {
                    numeric_breaks.push(endpoints[1].clone());
                    colorstep_positions.push(Some(Number(1.)));
                }
            }
        }
        let colorbar =
            self.colorbar_samples(numeric_breaks.iter().any(|entry| entry.visible), missing)?;
        if self
            .spec
            .colorbar_options
            .as_deref()
            .is_some_and(|o| o.reverse)
        {
            for position in colorstep_positions.iter_mut().flatten() {
                position.0 = 1. - position.0;
            }
            colorsteps.reverse();
            for step in &mut colorsteps {
                std::mem::swap(&mut step.start, &mut step.end);
            }
        }
        Ok(ColorLegend {
            discrete_keys,
            title: None,
            id,
            entries,
            numeric_breaks,
            colorbar,
            colorsteps,
            colorstep_positions,
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
pub(super) fn discrete_text_key(key: &ScaleKey) -> Option<String> {
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
            || matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(i) if i.wire_version()>=3) || matches!(s.normalization,NormalizationSpec::Ggplot {..}))
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

fn palette_lookup_error() -> crate::Diagnostic {
    error(
        DiagnosticCode::Validation,
        "Discrete palette lookup exceeds the returned vector and its missing-value slot.",
    )
}
