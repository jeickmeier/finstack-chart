//! Versioned positional providers use the same immutable registry as other extensions.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, ValueSpace, extensions};
use crate::{
    ChartResult, DiagnosticCode, Limits,
    scales::{Bounds, OutsidePolicy, PositionalScale},
    state::AxisWindow,
};
use std::{collections::BTreeMap, sync::Arc};

/// Destination inputs for one positional scale, before any of its guides are resolved.
pub struct ScaleProviderInput<'a> {
    /// Exact declarative parameters; never executable code or implicit host resources.
    pub parameters: &'a serde_json::Value,
    /// Eligible post-stat numeric extent, including the chart's shared facet training.
    pub extent: Option<Bounds>,
    /// Prepared numeric/category/exact-timestamp meaning.
    pub space: &'a ValueSpace,
    /// Authored or destination-derived output endpoints.
    pub range: Bounds,
    /// Explicit captured navigation window, if any; unsupported windows must reject.
    pub window: Option<AxisWindow>,
    /// Requested omit/clamp/extrapolation policy; unsupported policies must reject.
    pub outside: OutsidePolicy,
    /// Hard collection and geometry limits for native implementation work.
    pub limits: Limits,
    /// Hard domain/category output limit, independent from any tick density hint.
    pub max_values: usize,
    /// Hard tick output limit for this layout.
    pub max_ticks: usize,
}

/// Trusted native factory behind an explicitly installed versioned positional scale.
pub trait CustomScale: Send + Sync {
    /// Frozen registration identity and portable/native-only declaration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate owned parameters without running mapping, ticks, formatting or host I/O.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Resolve one immutable mapping; guides share this result without retraining it.
    /// Implementations must honor supplied limits and explicitly reject unsupported inputs.
    fn resolve(&self, input: ScaleProviderInput<'_>) -> ChartResult<Arc<dyn PositionalScale>>;
}

#[derive(Clone)]
struct Registration {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomScale>,
}
impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ScaleRegistrations {
    entries: BTreeMap<(String, u64), Registration>,
}
impl ScaleRegistrations {
    fn registration(&self, operation: &OperationRef) -> ChartResult<&Registration> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Positional scale {} version {} is not registered.",
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
    ) -> ChartResult<()> {
        extensions::parameter_size(parameters)?;
        self.registration(operation)?
            .implementation
            .validate(parameters)
    }
    pub(crate) fn resolve(
        &self,
        operation: &OperationRef,
        input: ScaleProviderInput<'_>,
        portable: bool,
    ) -> ChartResult<Arc<dyn PositionalScale>> {
        self.validate(operation, input.parameters)?;
        let registration = self.registration(operation)?;
        if portable && !registration.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "A native-only positional provider cannot execute in headless publication.",
            ));
        }
        registration.implementation.resolve(input)
    }
}
impl ExtensionRegistry {
    /// Register one exact provider version without replacing a builtin scale family.
    pub fn register_scale(&mut self, implementation: Arc<dyn CustomScale>) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        let entries = &mut Arc::make_mut(&mut self.scales).entries;
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A positional provider version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "registered positional provider")?;
        entries.insert(
            key,
            Registration {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    /// Inspect captured registration metadata without invoking native code.
    pub fn scale_descriptor(&self, operation: &OperationRef) -> ChartResult<ExtensionDescriptor> {
        Ok(self.scales.registration(operation)?.descriptor.clone())
    }
    pub(crate) fn validate_scale_selections(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for axis in &definition.axes {
            if let crate::layout::AxisScale::Registered {
                operation,
                parameters,
            } = &axis.scale
            {
                self.scales.validate(operation, parameters)?;
                if portable && !self.scale_descriptor(operation)?.portable {
                    return Err(super::error(
                        DiagnosticCode::UnsupportedCapability,
                        "A native-only positional provider cannot serialize or execute as a portable chart.",
                    ));
                }
            }
        }
        Ok(())
    }
}
