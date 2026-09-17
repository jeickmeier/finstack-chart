//! Registered pure functions shared by function sampling and QQ distributions.
use super::{
    registry::{VersionedMap, reject_native_only},
    *,
};
use crate::{ChartResult, DiagnosticCode};
use std::sync::Arc;

/// Trusted immutable numeric function, installed before portable execution.
pub trait CustomAnalyticFunction: Send + Sync {
    /// Stable identity, version and portability declaration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate bounded parameters before evaluating any coordinates.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Return exactly one output per input; missing results retain their position.
    fn evaluate(
        &self,
        input: &[f64],
        parameters: &serde_json::Value,
    ) -> ChartResult<Vec<Option<f64>>>;
}
#[derive(Clone, Default)]
pub(crate) struct AnalyticRegistrations(VersionedMap<Arc<dyn CustomAnalyticFunction>>);
impl ExtensionRegistry {
    /// Install one exact pure analytic function without permitting builtin overrides.
    pub fn register_analytic_function(
        &mut self,
        function: Arc<dyn CustomAnalyticFunction>,
    ) -> ChartResult<()> {
        let descriptor = function.descriptor();
        Arc::make_mut(&mut self.analytic_functions).0.insert(
            descriptor,
            function,
            "Analytic function version is already registered.",
            "Analytic function registration",
        )
    }
}
impl AnalyticRegistrations {
    fn registration(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomAnalyticFunction>>> {
        self.0.get_fmt(operation, || {
            format!(
                "Analytic function {} version {} is not registered.",
                operation.id,
                operation.version.get()
            )
        })
    }
}
impl AnalyticFunction {
    pub(super) fn validate_function(
        &self,
        registry: &ExtensionRegistry,
        portable: bool,
    ) -> ChartResult<()> {
        let valid = match self {
            Self::Expression(expression) => {
                if expression
                    .validate(ExpressionLimits::default(), |_| Ok(ExpressionType::Number))?
                    != ExpressionType::Number
                {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Analytic expression must return numbers.",
                    ));
                }
                true
            }
            Self::NormalQuantile { mean, sd } => mean.is_finite() && sd.is_finite() && *sd > 0.,
            Self::UniformQuantile { min, max } => min.is_finite() && max.is_finite() && max >= min,
            Self::LogisticQuantile { location, scale } => {
                location.is_finite() && scale.is_finite() && *scale > 0.
            }
            Self::Registered {
                operation,
                parameters,
            } => {
                extensions::parameter_size(parameters)?;
                let registration = registry.analytic_functions.registration(operation)?;
                reject_native_only(
                    portable,
                    registration.descriptor.portable,
                    "Native-only analytic function cannot execute in portable publication.",
                )?;
                registration.implementation.validate(parameters)?;
                true
            }
        };
        if valid {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::NumericalDomain,
                "Invalid analytic function distribution parameters.",
            ))
        }
    }
    pub(super) fn evaluate_function(
        &self,
        input: &[f64],
        registry: &ExtensionRegistry,
        max_rows: usize,
    ) -> ChartResult<Vec<Option<f64>>> {
        self.validate_function(registry, false)?;
        if input.len() > max_rows {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Analytic function output budget exceeded.",
            ));
        }
        let output = match self {
            Self::Expression(expression) => expression
                .evaluate(
                    input.len(),
                    ExpressionLimits::default(),
                    |_| Ok(ExpressionType::Number),
                    |_, i| ExpressionValue::Number(input[i]),
                )?
                .iter()
                .map(ExpressionValue::number)
                .collect(),
            Self::Registered {
                operation,
                parameters,
            } => registry
                .analytic_functions
                .registration(operation)?
                .implementation
                .evaluate(input, parameters)?,
            _ => input
                .iter()
                .map(|&p| {
                    let value = match *self {
                        Self::NormalQuantile { mean, sd } => {
                            crate::scales::GgplotTransform::Normal { mean, sd }.forward(p)
                        }
                        Self::UniformQuantile { min, max } if (0. ..=1.).contains(&p) => {
                            min * (1. - p) + max * p
                        }
                        Self::LogisticQuantile { location, scale } => {
                            crate::scales::GgplotTransform::Logistic { location, scale }.forward(p)
                        }
                        _ => f64::NAN,
                    };
                    (!value.is_nan()).then_some(value)
                })
                .collect(),
        };
        if output.len() != input.len() || output.iter().flatten().any(|v| v.is_nan()) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Analytic function must return aligned values with explicit missingness.",
            ));
        }
        Ok(output)
    }
}
