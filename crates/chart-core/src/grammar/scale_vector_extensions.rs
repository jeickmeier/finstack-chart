//! Pure numeric vector operations used by reference scale pipelines.
use super::{
    ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions,
    registry::{VersionedMap, reject_native_only},
};
use crate::{ChartResult, interpolate::Number};
use std::sync::Arc;

/// Select an installed numeric vector function by exact version.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleVectorOperation {
    /// Captured qualified identity and version.
    pub operation: OperationRef,
    /// Bounded declarative configuration, without executable code.
    pub parameters: serde_json::Value,
}
/// Pipeline stage that supplies the vector to a registered operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleVectorStage {
    /// Out-of-bounds handling in transformed scale coordinates.
    OutOfBounds,
    /// Rescaling observations or binned boundaries into palette coordinates.
    Rescale,
}
/// Borrowed complete numeric vector; IEEE NaN represents missing values.
pub struct ScaleVectorInput<'a> {
    /// Pipeline stage, independent of the installed function identity.
    pub stage: ScaleVectorStage,
    /// Ordered observations or break boundaries, before deduplication.
    pub values: &'a [Number],
    /// Reference limits in the same transformed coordinate space.
    pub limits: &'a [Number],
    /// Authored parameters.
    pub parameters: &'a serde_json::Value,
}
/// Trusted pure native operation. Core bounds input and output storage but cannot
/// preempt native code or undo side effects inside an implementation.
pub trait CustomScaleVector: Send + Sync {
    /// Immutable identity and portable/native capability.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate declarative configuration without running the operation.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Preserve result arity; None is NULL and Some(empty) is an empty vector.
    fn evaluate(&self, input: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>>;
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ScaleVectorRegistrations(VersionedMap<Arc<dyn CustomScaleVector>>);
impl ScaleVectorRegistrations {
    fn registration(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomScaleVector>>> {
        self.0.get_fmt(operation, || {
            format!(
                "Scale vector function {} version {} is not registered.",
                operation.id,
                operation.version.get()
            )
        })
    }
    pub(crate) fn validate(&self, call: &ScaleVectorOperation, portable: bool) -> ChartResult<()> {
        extensions::parameter_size(&call.parameters)?;
        let registration = self.registration(&call.operation)?;
        reject_native_only(
            portable,
            registration.descriptor.portable,
            "Native-only scale vector functions cannot execute in portable publication.",
        )?;
        registration.implementation.validate(&call.parameters)
    }
    pub(crate) fn evaluate(
        &self,
        call: &ScaleVectorOperation,
        stage: ScaleVectorStage,
        values: &[Number],
        limits: &[Number],
    ) -> ChartResult<Option<Vec<Number>>> {
        self.validate(call, false)?;
        crate::limits::require_within(
            values.len().saturating_add(limits.len()) <= crate::interpolate::MAX_VALUES,
            "scale vector input count",
        )?;
        let result = self
            .registration(&call.operation)?
            .implementation
            .evaluate(ScaleVectorInput {
                stage,
                values,
                limits,
                parameters: &call.parameters,
            })?;
        crate::limits::require_within(
            result
                .as_ref()
                .is_none_or(|v| v.len() <= crate::interpolate::MAX_VALUES),
            "scale vector output count",
        )?;
        Ok(result)
    }
}
impl ExtensionRegistry {
    /// Register a pure numeric vector operation without replacing an existing version.
    pub fn register_scale_vector(
        &mut self,
        implementation: Arc<dyn CustomScaleVector>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        Arc::make_mut(&mut self.scale_vectors).0.insert(
            descriptor,
            implementation,
            "A scale vector function version is already registered.",
            "registered scale vector function",
        )
    }
    /// Read an installed descriptor without evaluating the function.
    pub fn scale_vector_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .scale_vectors
            .registration(operation)?
            .descriptor
            .clone())
    }
}
