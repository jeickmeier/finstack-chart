//! Pure scale limit functions over shared population training.
use super::{
    ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions,
    registry::{VersionedMap, reject_native_only},
};
use crate::{ChartResult, scales::ScaleKey};
use std::sync::Arc;

/// Select an installed pure function; JSON never contains executable code.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleLimitsOperation {
    /// Captured qualified identity and exact version.
    pub operation: OperationRef,
    /// Bounded declarative configuration.
    pub parameters: serde_json::Value,
}
/// Source domain supplied after population training, before limit replacement.
/// Discrete scales preserve typed keys; numeric scales supply inverse-transformed
/// endpoints as numbers, including nonfinite values for a trained empty range.
pub struct ScaleLimitsInput<'a> {
    /// None denotes an untrained domain, distinct from an explicitly empty vector.
    pub domain: Option<&'a [ScaleKey]>,
    /// When present, numeric domain values and results are offsets in this unit
    /// from the exact origin. `date` distinguishes Date from datetime semantics.
    /// Nonfinite offsets remain representable without rounding the epoch.
    pub temporal: Option<crate::scales::GgplotTimestampNormalization>,
    /// Authored parameters for this operation.
    pub parameters: &'a serde_json::Value,
}
/// Trusted native function. Evaluation must be pure and honor the bounded output
/// contract; core cannot preempt native code or undo implementation side effects.
pub trait CustomScaleLimits: Send + Sync {
    /// Immutable identity and portable/native capability.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters without inspecting data or running the function.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Whether evaluation reads the trained domain. A constant function may
    /// return false to avoid forcing an unavailable inverse-transformed domain.
    /// The default preserves eager domain validation for existing functions.
    fn requires_domain(&self, _parameters: &serde_json::Value) -> bool {
        true
    }
    /// Replace the trained domain. None preserves a reference NULL result;
    /// Some(empty) deliberately selects no levels.
    fn evaluate(&self, input: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>>;
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ScaleLimitRegistrations(VersionedMap<Arc<dyn CustomScaleLimits>>);
impl ScaleLimitRegistrations {
    pub(crate) fn requires_domain(&self, call: &ScaleLimitsOperation) -> ChartResult<bool> {
        self.validate(call, false)?;
        Ok(self
            .registration(&call.operation)?
            .implementation
            .requires_domain(&call.parameters))
    }
    fn registration(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomScaleLimits>>> {
        self.0.get_fmt(operation, || {
            format!(
                "Scale limits {} version {} are not registered.",
                operation.id,
                operation.version.get()
            )
        })
    }
    pub(crate) fn validate(&self, call: &ScaleLimitsOperation, portable: bool) -> ChartResult<()> {
        extensions::parameter_size(&call.parameters)?;
        let registration = self.registration(&call.operation)?;
        reject_native_only(
            portable,
            registration.descriptor.portable,
            "Native-only scale limits cannot serialize or execute in portable publication.",
        )?;
        registration.implementation.validate(&call.parameters)
    }
    pub(crate) fn evaluate(
        &self,
        call: &ScaleLimitsOperation,
        domain: Option<&[ScaleKey]>,
        temporal: Option<crate::scales::GgplotTimestampNormalization>,
    ) -> ChartResult<Option<Vec<ScaleKey>>> {
        self.validate(call, false)?;
        let result = self
            .registration(&call.operation)?
            .implementation
            .evaluate(ScaleLimitsInput {
                domain,
                temporal,
                parameters: &call.parameters,
            })?;
        if let Some(keys) = &result {
            crate::limits::require_within(
                keys.len() <= crate::interpolate::MAX_VALUES,
                "scale limit output",
            )?;
            let mut bytes = 0usize;
            for key in keys {
                let value = crate::scales::GgplotDiscreteIdentity::map(Some(key))?;
                value.validate()?;
                if let ScaleKey::Text(text) = key {
                    bytes = bytes.saturating_add(text.len());
                    crate::limits::require_within(
                        bytes <= crate::interpolate::MAX_VALUE_BYTES,
                        "scale limit text",
                    )?;
                }
            }
        }
        Ok(result)
    }
}
impl ExtensionRegistry {
    /// Install a pure domain function without replacing an existing version.
    pub fn register_scale_limits(
        &mut self,
        implementation: Arc<dyn CustomScaleLimits>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        Arc::make_mut(&mut self.limits_function).0.insert(
            descriptor,
            implementation,
            "A scale limit function version is already registered.",
            "registered scale limit function",
        )
    }
    /// Read the captured identity without running the installed function.
    pub fn scale_limits_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .limits_function
            .registration(operation)?
            .descriptor
            .clone())
    }
}
