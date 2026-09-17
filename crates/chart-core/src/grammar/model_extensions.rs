//! Trusted portable model registrations supplement the required built-in algorithms.
use super::{registry::VersionedMap, *};
use crate::{ChartResult, DiagnosticCode};
use std::sync::Arc;
/// Inputs borrowed by one registered fit/predict operation.
pub struct ModelInput<'a> {
    /// Predictor observations in declared calculation space.
    pub x: &'a [f64],
    /// Aligned response observations.
    pub y: &'a [f64],
    /// Aligned nonnegative observation weights.
    pub weights: &'a [f64],
    /// Requested predictor coordinates, preserving authored order.
    pub grid: &'a [f64],
    /// Prediction and confidence controls.
    pub options: &'a ModelOptions,
    /// Resource limits also enforced on returned predictions.
    pub limits: CompileLimits,
}
/// One aligned registered model prediction; missing predictions retain their slot.
#[derive(Clone, Debug)]
pub struct ModelEstimate {
    /// Conditional mean or fitted value.
    pub mean: Option<f64>,
    /// Mean-estimation standard error.
    pub standard_error: Option<f64>,
    /// Lower confidence endpoint.
    pub lower: Option<f64>,
    /// Upper confidence endpoint.
    pub upper: Option<f64>,
}
/// Immutable pure model installed before compilation or portable replay.
pub trait CustomModel: Send + Sync {
    /// Stable operation identity and portability declaration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters before any observations are processed.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Fit and return exactly one estimate per requested coordinate.
    fn predict(
        &self,
        input: ModelInput<'_>,
        parameters: &serde_json::Value,
    ) -> ChartResult<Vec<ModelEstimate>>;
}
#[derive(Clone, Default)]
pub(crate) struct ModelRegistrations(VersionedMap<Arc<dyn CustomModel>>);
impl ExtensionRegistry {
    /// Install an exact model version without overriding built-in operations.
    pub fn register_model(&mut self, model: Arc<dyn CustomModel>) -> ChartResult<()> {
        let descriptor = model.descriptor();
        Arc::make_mut(&mut self.models).0.insert(
            descriptor,
            model,
            "Model version is already registered.",
            "model registrations",
        )
    }
}
fn registration<'a>(
    registry: &'a ExtensionRegistry,
    operation: &OperationRef,
) -> ChartResult<&'a super::registry::VersionedEntry<Arc<dyn CustomModel>>> {
    registry.models.0.get_fmt(operation, || {
        format!(
            "Model {} version {} is not registered.",
            operation.id,
            operation.version.get()
        )
    })
}
pub(super) fn validate(
    registry: &ExtensionRegistry,
    method: &ModelMethod,
    portable: bool,
) -> ChartResult<()> {
    if let ModelMethod::Registered {
        operation,
        parameters,
    } = method
    {
        extensions::parameter_size(parameters)?;
        let r = registration(registry, operation)?;
        super::registry::reject_native_only(
            portable,
            r.descriptor.portable,
            "Native-only model cannot enter portable publication.",
        )?;
        r.implementation.validate(parameters)?;
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn predict(
    registry: &ExtensionRegistry,
    operation: &OperationRef,
    parameters: &serde_json::Value,
    x: &[f64],
    y: &[f64],
    weights: &[f64],
    grid: &[f64],
    options: &ModelOptions,
    limits: CompileLimits,
) -> ChartResult<model_predict::ModelPredictionOutput> {
    validate(registry, &options.method, false)?;
    if weights.iter().any(|w| !w.is_finite() || *w < 0.) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Registered model weights must be finite and nonnegative.",
        ));
    }
    if grid.len() > limits.max_prepared_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Registered model grid exceeds budget.",
        ));
    }
    let output = registration(registry, operation)?.implementation.predict(
        ModelInput {
            x,
            y,
            weights,
            grid,
            options,
            limits,
        },
        parameters,
    )?;
    if output.len() != grid.len()
        || output.iter().any(|p| {
            [p.mean, p.standard_error, p.lower, p.upper]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite())
                || p.standard_error.is_some_and(|v| v < 0.)
                || p.lower.zip(p.upper).is_some_and(|(a, b)| a > b)
        })
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Registered model returned misaligned or invalid predictions.",
        ));
    }
    Ok(model_predict::ModelPredictionOutput {
        rows: output
            .into_iter()
            .zip(grid)
            .map(|(p, &x)| model_predict::Prediction {
                x,
                mean: p.mean,
                se: p.standard_error,
                lower: p.lower,
                upper: p.upper,
                quantile: None,
            })
            .collect(),
        diagnostics: vec![],
    })
}
