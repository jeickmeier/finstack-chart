//! Registered semantic guide formatters, independent of positional scale ownership.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions};
use crate::{ChartResult, DiagnosticCode, Limits, composition::ScaleValue};
use std::{collections::BTreeMap, sync::Arc};

/// Exact inputs to one native label callback. The list precedes mapping/omission.
pub struct GuideFormatInput<'a> {
    /// Original value, including exact timestamp units.
    pub value: &'a ScaleValue,
    /// Authored occurrence index, even when values or labels repeat.
    pub index: usize,
    /// Complete selected semantic values in their authored/generated order.
    pub values: &'a [ScaleValue],
    /// Explicit bounded declarative configuration.
    pub parameters: &'a serde_json::Value,
    /// Remaining text budget and the destination's other resource bounds.
    pub limits: Limits,
}

/// Trusted native label callback behind an explicitly installed versioned identity.
/// Implementations must honor the supplied resource bounds and perform no implicit
/// locale, calendar or host-object lookup. Native code cannot be preempted by core.
pub trait CustomGuideFormatter: Send + Sync {
    /// Identity and portability captured once at registration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Check bounded parameters without invoking per-value callbacks.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Format one original semantic value. Empty and repeated strings are valid.
    fn format(&self, input: GuideFormatInput<'_>) -> ChartResult<String>;
}

#[derive(Clone)]
struct Registration {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomGuideFormatter>,
}
impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct GuideRegistrations {
    entries: BTreeMap<(String, u64), Registration>,
}
impl GuideRegistrations {
    fn registration(&self, operation: &OperationRef) -> ChartResult<&Registration> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Guide formatter {} version {} is not registered.",
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
                "A native-only guide formatter cannot serialize or execute in headless publication.",
            ));
        }
        registration.implementation.validate(parameters)
    }
    pub(crate) fn labels(
        &self,
        operation: &OperationRef,
        parameters: &serde_json::Value,
        values: &[ScaleValue],
        mut limits: Limits,
        portable: bool,
    ) -> ChartResult<Vec<String>> {
        self.validate(operation, parameters, portable)?;
        let registration = self.registration(operation)?;
        let mut labels = Vec::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            let label = registration.implementation.format(GuideFormatInput {
                value,
                index,
                values,
                parameters,
                limits,
            })?;
            crate::limits::require_within(
                label.len() <= limits.max_text_bytes,
                "registered guide label byte",
            )?;
            limits.max_text_bytes -= label.len();
            labels.push(label);
        }
        Ok(labels)
    }
}
impl ExtensionRegistry {
    /// Install one checked version; snapshots retain their previous registrations.
    pub fn register_guide_formatter(
        &mut self,
        implementation: Arc<dyn CustomGuideFormatter>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        let entries = &mut Arc::make_mut(&mut self.guides).entries;
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A guide formatter version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "registered guide formatter")?;
        entries.insert(
            key,
            Registration {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    /// Captured identity and portability; does not invoke callback code.
    pub fn guide_formatter_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self.guides.registration(operation)?.descriptor.clone())
    }
    pub(crate) fn validate_guide_selections(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for style in definition
            .axes
            .iter()
            .map(|axis| &axis.guide)
            .chain(definition.guides.iter().map(|guide| &guide.style))
        {
            if let Some(crate::layout::GuideFormatter::Registered {
                operation,
                parameters,
            }) = &style.tick_format
            {
                self.guides.validate(operation, parameters, portable)?;
            }
        }
        Ok(())
    }
}

pub(crate) fn validate_parameters(parameters: &serde_json::Value) -> ChartResult<()> {
    extensions::parameter_size(parameters).map(|_| ())
}
