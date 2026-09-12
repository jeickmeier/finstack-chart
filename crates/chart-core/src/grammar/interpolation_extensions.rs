//! Captured native interpolation factories behind bounded versioned descriptors.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions};
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{Sample, Value},
};
use std::{collections::BTreeMap, sync::Arc};

/// One pair supplied to a registered factory. Preparation runs before sampling.
pub struct InterpolationInput<'a> {
    /// Original source endpoint, with typed missing/nonfinite classifications.
    pub source: &'a Value,
    /// Original target endpoint; custom factories declare their own result shape.
    pub target: &'a Value,
    /// Bounded declarative configuration.
    pub parameters: &'a serde_json::Value,
}

/// Trusted pure native factory. Each returned sampler owns its prepared state.
/// Core validates input/output value budgets, but cannot preempt native code.
pub trait CustomInterpolationFactory: Send + Sync {
    /// Stable descriptor captured at installation; no runtime code loading is implied.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate declarative configuration before any pair is compiled.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Compile one pair once. Sampling receives finite explicit parameters and no clock.
    fn compile(
        &self,
        input: InterpolationInput<'_>,
    ) -> ChartResult<Arc<dyn Sample<Value> + Send + Sync>>;
}

#[derive(Clone)]
struct Registration {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomInterpolationFactory>,
}
impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct InterpolationRegistrations {
    entries: BTreeMap<(String, u64), Registration>,
}
impl InterpolationRegistrations {
    fn registration(&self, operation: &OperationRef) -> ChartResult<&Registration> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Interpolation factory {} version {} is not registered.",
                        operation.id,
                        operation.version.get()
                    ),
                )
            })
    }
    pub(crate) fn validate(
        &self,
        operation: &OperationRef,
        parameters: &serde_json::Value,
        portable: bool,
    ) -> ChartResult<()> {
        extensions::parameter_size(parameters)?;
        let registration = self.registration(operation)?;
        if portable && !registration.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "A native-only interpolation factory cannot serialize or execute in headless publication.",
            ));
        }
        registration.implementation.validate(parameters)
    }
    pub(crate) fn compile(
        &self,
        operation: &OperationRef,
        input: InterpolationInput<'_>,
    ) -> ChartResult<RegisteredInterpolator> {
        self.validate(operation, input.parameters, false)?;
        input.source.validate()?;
        input.target.validate()?;
        let registration = self.registration(operation)?;
        Ok(RegisteredInterpolator {
            descriptor: registration.descriptor.clone(),
            sampler: registration.implementation.compile(input)?,
        })
    }
}

/// Checked prepared native sampler; its descriptor cannot drift after registration.
#[derive(Clone)]
pub(crate) struct RegisteredInterpolator {
    descriptor: ExtensionDescriptor,
    sampler: Arc<dyn Sample<Value> + Send + Sync>,
}
impl std::fmt::Debug for RegisteredInterpolator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
impl RegisteredInterpolator {
    pub(crate) fn portable(&self) -> bool {
        self.descriptor.portable
    }
}
impl Sample<Value> for RegisteredInterpolator {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        if !t.is_finite() {
            return Err(super::error(
                DiagnosticCode::NumericalDomain,
                "Registered interpolation requires a finite parameter.",
            ));
        }
        let value = self.sampler.sample(t)?;
        value.validate()?;
        Ok(value)
    }
}
impl ExtensionRegistry {
    /// Install one exact native factory version. Existing prepared snapshots are unchanged.
    pub fn register_interpolation(
        &mut self,
        implementation: Arc<dyn CustomInterpolationFactory>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        let entries = &mut Arc::make_mut(&mut self.interpolations).entries;
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "An interpolation factory version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "registered interpolation factory")?;
        entries.insert(
            key,
            Registration {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    /// Captured metadata without invoking the implementation.
    pub fn interpolation_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .interpolations
            .registration(operation)?
            .descriptor
            .clone())
    }
    /// Select a checked factory without compiling endpoints or exposing callback objects.
    pub fn interpolation_factory(
        &self,
        operation: OperationRef,
        parameters: serde_json::Value,
    ) -> ChartResult<crate::interpolate::InterpolationFactory> {
        self.interpolations
            .validate(&operation, &parameters, false)?;
        Ok(crate::interpolate::InterpolationFactory::registered(
            operation, parameters,
        ))
    }
}

pub(crate) fn mapped_scales(
    definition: &super::ChartDefinition,
) -> impl Iterator<Item = &crate::scales::MappedScaleSpec> {
    definition.layers.iter().flat_map(|layer| {
        layer
            .numeric_scales
            .values()
            .map(|encoding| &encoding.scale)
            .chain(layer.value_scales.values().map(|encoding| &encoding.scale))
            .chain(
                super::colors::encodings(layer).filter_map(|encoding| match &encoding.scale {
                    crate::scales::ColorScale::Mapped { scale, .. } => Some(scale),
                    _ => None,
                }),
            )
    })
}
impl ExtensionRegistry {
    /// Reject unavailable or native-only mapped interpolation before headless preparation.
    /// Other extension families retain their existing destination-specific checks.
    pub fn validate_portable_interpolations(
        &self,
        definition: &super::ChartDefinition,
    ) -> ChartResult<()> {
        self.validate_interpolation_selections(definition, true)
    }
    pub(crate) fn validate_interpolation_selections(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for scale in mapped_scales(definition) {
            if let Some(call) = &scale.limits_function {
                self.limits_function.validate(call, portable)?;
            }
            scale.validate_interpolations(&self.interpolations, portable)?;
        }
        Ok(())
    }
}
