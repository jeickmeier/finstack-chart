//! Pure palette functions over shared count or normalized vector inputs.
use super::{
    ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions,
    registry::{VersionedMap, reject_native_only},
};
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{Number, Value},
};
use std::sync::Arc;

/// Select an installed pure function; JSON never contains executable code.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalePaletteOperation {
    /// Captured qualified identity and exact version.
    pub operation: OperationRef,
    /// Bounded declarative configuration.
    pub parameters: serde_json::Value,
}
/// The scale family determines the callback's complete input.
#[derive(Clone, Copy, Debug)]
pub enum ScalePaletteDomain<'a> {
    /// Resolved category count, including unobserved authored limits.
    /// Nonpositional palettes omit missing levels; positional palettes include them.
    Count(usize),
    /// Ordered normalized samples; IEEE NaN represents a missing input.
    Normalized(&'a [Number]),
}
/// Borrowed input for one pure palette evaluation.
pub struct ScalePaletteInput<'a> {
    /// Count or complete vector, without per-element callback evaluation.
    pub domain: ScalePaletteDomain<'a>,
    /// Authored parameters for this operation.
    pub parameters: &'a serde_json::Value,
}
/// Palette results retain names and NULL separately from an empty vector.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScalePaletteOutput {
    /// None represents NULL; some empty vector represents a zero-length palette.
    pub values: Option<Vec<Value>>,
    /// Discrete scales match these names by category; numeric scales ignore names.
    pub names: Option<Vec<String>>,
}
/// Trusted native function. Evaluation must be pure and honor the bounded output
/// contract; core cannot preempt native code or undo implementation side effects.
pub trait CustomScalePalette: Send + Sync {
    /// Immutable identity and portable/native capability.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters without inspecting data or running the function.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Return the complete palette for the count or ordered vector.
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput>;
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ScalePaletteRegistrations(VersionedMap<Arc<dyn CustomScalePalette>>);
impl ScalePaletteRegistrations {
    fn registration(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomScalePalette>>> {
        self.0.get_fmt(operation, || {
            format!(
                "Scale palette {} version {} are not registered.",
                operation.id,
                operation.version.get()
            )
        })
    }
    pub(crate) fn validate(&self, call: &ScalePaletteOperation, portable: bool) -> ChartResult<()> {
        extensions::parameter_size(&call.parameters)?;
        let registration = self.registration(&call.operation)?;
        reject_native_only(
            portable,
            registration.descriptor.portable,
            "Native-only scale palettes cannot serialize or execute in portable publication.",
        )?;
        registration.implementation.validate(&call.parameters)
    }
    pub(crate) fn evaluate(
        &self,
        call: &ScalePaletteOperation,
        domain: ScalePaletteDomain<'_>,
    ) -> ChartResult<ScalePaletteOutput> {
        self.validate(call, false)?;
        crate::limits::require_within(
            match domain {
                ScalePaletteDomain::Count(count) => count,
                ScalePaletteDomain::Normalized(values) => values.len(),
            } <= crate::interpolate::MAX_VALUES,
            "palette input count",
        )?;
        let result = self
            .registration(&call.operation)?
            .implementation
            .evaluate(ScalePaletteInput {
                domain,
                parameters: &call.parameters,
            })?;
        let values = result.values.as_deref().unwrap_or_default();
        // Validate the whole vector under one aggregate byte/value budget.
        let mut nodes = 1;
        let mut bytes = 0;
        for value in values {
            value.budget(1, &mut nodes, &mut bytes)?;
        }
        if let Some(names) = &result.names {
            if names.len() != values.len() {
                return Err(super::error(
                    DiagnosticCode::Validation,
                    "Palette names and values have different lengths.",
                ));
            }
            crate::limits::require_within(
                names
                    .iter()
                    .fold(bytes, |n, name| n.saturating_add(name.len()))
                    <= crate::interpolate::MAX_VALUE_BYTES,
                "palette names",
            )?;
        }
        Ok(result)
    }
}
impl ExtensionRegistry {
    /// Install a pure palette function without replacing an existing version.
    pub fn register_scale_palette(
        &mut self,
        implementation: Arc<dyn CustomScalePalette>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        Arc::make_mut(&mut self.palette_function).0.insert(
            descriptor,
            implementation,
            "A scale palette function version is already registered.",
            "registered scale palette function",
        )
    }
    /// Read the captured identity without running the installed function.
    pub fn scale_palette_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .palette_function
            .registration(operation)?
            .descriptor
            .clone())
    }
}
