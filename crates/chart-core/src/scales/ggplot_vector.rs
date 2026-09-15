//! Shared ordered numeric stages for reference aesthetic scale callbacks.
use super::*;
use crate::{
    grammar::{
        ScaleVectorOperation, ScaleVectorStage, scale_vector_extensions::ScaleVectorRegistrations,
    },
    interpolate::Number,
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(super) struct VectorPipeline {
    oob: Option<Box<ScaleVectorOperation>>,
    rescaler: Option<Box<ScaleVectorOperation>>,
    registry: Arc<ScaleVectorRegistrations>,
    default_oob: GgplotOob,
    limits: Option<Vec<Number>>,
}
impl VectorPipeline {
    pub fn new(
        spec: &MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> Option<Self> {
        if !matches!(
            spec.guide.as_deref(),
            Some(
                GgplotScaleGuide::Colorbar(_)
                    | GgplotScaleGuide::TemporalColorbar(_)
                    | GgplotScaleGuide::ContinuousBins(_)
                    | GgplotScaleGuide::TemporalBins(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        ) && spec.oob_function.is_none()
            && spec.rescaler_function.is_none()
            && !spec
                .ggplot_transform()
                .is_some_and(|transform| !transform.is_pointwise())
        {
            return None;
        }
        Self::reference(spec, registry)
    }
    /// Retain the same ordered stages when built-in palette evaluation is deferred.
    pub fn reference(
        spec: &MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> Option<Self> {
        let default_oob = match spec.ggplot.as_deref()? {
            GgplotScalePolicy::Continuous { oob, .. } => *oob,
            GgplotScalePolicy::Binned(policy) => policy.oob,
            _ => return None,
        };
        Some(Self {
            oob: spec.oob_function.clone(),
            rescaler: spec.rescaler_function.clone(),
            registry: registry.scale_vectors.clone(),
            default_oob,
            limits: spec.resolved_numeric_limits.as_deref().cloned(),
        })
    }
    pub fn map(
        &self,
        normalizer: &ScaleNormalizer,
        inputs: &[Option<f64>],
    ) -> ChartResult<Vec<Number>> {
        let transformed = normalizer.reference_inputs(
            &inputs
                .iter()
                .map(|v| v.unwrap_or_else(super::ggplot_palette::missing_number))
                .collect::<Vec<_>>(),
        )?;
        self.map_transformed(normalizer, &transformed)
    }
    pub fn map_transformed(
        &self,
        normalizer: &ScaleNormalizer,
        transformed: &[Number],
    ) -> ChartResult<Vec<Number>> {
        let limits = self.limits(normalizer)?;
        let bounded = if let Some(call) = &self.oob {
            self.registry
                .evaluate(call, ScaleVectorStage::OutOfBounds, transformed, &limits)?
                .unwrap_or_default()
        } else {
            transformed
                .iter()
                .map(|v| {
                    Number(if v.0.is_nan() {
                        v.0
                    } else {
                        self.default_oob
                            .apply(
                                Some(v.0),
                                [
                                    limits.first().map_or(f64::NAN, |v| v.0),
                                    limits.get(1).map_or(f64::NAN, |v| v.0),
                                ],
                            )
                            .unwrap_or_else(super::ggplot_palette::missing_number)
                    })
                })
                .collect()
        };
        self.rescale(normalizer, &bounded)
    }
    pub fn rescale(
        &self,
        normalizer: &ScaleNormalizer,
        inputs: &[Number],
    ) -> ChartResult<Vec<Number>> {
        if let Some(call) = &self.rescaler {
            self.registry
                .evaluate(
                    call,
                    ScaleVectorStage::Rescale,
                    inputs,
                    &self.limits(normalizer)?,
                )
                .map(Option::unwrap_or_default)
        } else {
            if let Some(limits) = &self.limits {
                super::ggplot_numeric_limits::validate_result(limits)?;
            }
            Ok(inputs
                .iter()
                .map(|v| {
                    Number(if v.0.is_nan() {
                        v.0
                    } else {
                        normalizer.reference_parameter(v.0)
                    })
                })
                .collect())
        }
    }
    pub fn limits(&self, normalizer: &ScaleNormalizer) -> ChartResult<Vec<Number>> {
        self.limits.as_ref().map_or_else(
            || Ok(normalizer.reference_limits()),
            |values| normalizer.reference_inputs(&values.iter().map(|v| v.0).collect::<Vec<_>>()),
        )
    }
}
