//! Pure scale break functions over shared population training.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions};
use crate::{ChartResult, DiagnosticCode, scales::ScaleKey};
use std::{collections::BTreeMap, sync::Arc};

/// Select an installed pure function; JSON never contains executable code.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleBreaksOperation {
    /// Captured qualified identity and exact version.
    pub operation: OperationRef,
    /// Bounded declarative configuration.
    pub parameters: serde_json::Value,
}
/// Source limits supplied at guide selection, after population training and limit replacement.
pub struct ScaleBreaksInput<'a> {
    /// Inverse-transformed limits in reference order.
    pub domain: &'a [ScaleKey],
    /// True for reference NULL limits; false for a numeric vector, including an empty one.
    pub domain_is_null: bool,
    /// Major source breaks for a minor selector that accepts a second argument.
    /// None also preserves a suppressed discrete NULL major selection.
    pub major_breaks: Option<&'a [ScaleKey]>,
    /// Intrinsic names aligned with the supplied major breaks, before label formatting.
    pub major_break_names: Option<&'a [String]>,
    /// Desired count, supplied only when the implementation accepts the scale count argument.
    pub count: Option<f64>,
    /// Supplied argument identity; binned scales prefer `n.breaks` over `n`.
    pub count_argument: Option<&'static str>,
    /// Temporal interpretation of origin-relative limits, when selecting Date/datetime breaks.
    pub temporal: Option<super::GuideTemporalContext<'a>>,
    /// Authored parameters for this operation.
    pub parameters: &'a serde_json::Value,
}
/// Complete break vector, including duplicate, missing and nonfinite candidates.
#[derive(Clone, Debug, Default)]
pub struct ScaleBreaksOutput {
    /// None preserves NULL; an empty vector selects zero breaks.
    pub values: Option<Vec<ScaleKey>>,
    /// Typed temporal result representation; absent for ordinary numeric results.
    pub temporal: Option<crate::scales::GgplotTimestampNormalization>,
    /// Optional names aligned with the complete returned vector.
    pub names: Option<Vec<String>>,
}
/// Trusted pure native function. Core bounds results but cannot preempt native code.
pub trait CustomScaleBreaks: Send + Sync {
    /// Immutable identity and portable/native capability.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Whether continuous selection supplies its optional count argument.
    fn accepts_n(&self) -> bool {
        false
    }
    /// Whether binned selection accepts its preferred `n.breaks` count argument.
    fn accepts_n_breaks(&self) -> bool {
        false
    }
    /// Whether minor selection supplies its second, major-break vector argument.
    fn accepts_major_breaks(&self) -> bool {
        false
    }
    /// Validate parameters without evaluating the function.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Select candidates in source coordinates.
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput>;
}
#[derive(Clone)]
struct Registration {
    descriptor: ExtensionDescriptor,
    accepts_n: bool,
    accepts_n_breaks: bool,
    accepts_major_breaks: bool,
    implementation: Arc<dyn CustomScaleBreaks>,
}
impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ScaleBreakRegistrations {
    entries: BTreeMap<(String, u64), Registration>,
}
impl ScaleBreakRegistrations {
    fn registration(&self, operation: &OperationRef) -> ChartResult<&Registration> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Scale breaks {} version {} are not registered.",
                        operation.id,
                        operation.version.get()
                    ),
                )
            })
    }
    pub(crate) fn validate(&self, call: &ScaleBreaksOperation, portable: bool) -> ChartResult<()> {
        extensions::parameter_size(&call.parameters)?;
        let registration = self.registration(&call.operation)?;
        if portable && !registration.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only scale breaks cannot serialize or execute in portable publication.",
            ));
        }
        registration.implementation.validate(&call.parameters)
    }
    pub(crate) fn evaluate(
        &self,
        call: &ScaleBreaksOperation,
        domain: &[ScaleKey],
        count: Option<f64>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(call, Some(domain), count, false, None, None)
    }
    pub(crate) fn evaluate_optional(
        &self,
        call: &ScaleBreaksOperation,
        domain: Option<&[ScaleKey]>,
        count: Option<f64>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(call, domain, count, false, None, None)
    }
    pub(crate) fn evaluate_binned(
        &self,
        call: &ScaleBreaksOperation,
        domain: Option<&[ScaleKey]>,
        count: f64,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(call, domain, Some(count), true, None, None)
    }
    pub(crate) fn evaluate_temporal(
        &self,
        call: &ScaleBreaksOperation,
        domain: &[ScaleKey],
        count: Option<f64>,
        temporal: super::GuideTemporalContext<'_>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(call, Some(domain), count, false, Some(temporal), None)
    }
    pub(crate) fn evaluate_minor(
        &self,
        call: &ScaleBreaksOperation,
        domain: &[ScaleKey],
        major: Option<&[ScaleKey]>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(
            call,
            Some(domain),
            None,
            false,
            None,
            major.map(|values| (values, None)),
        )
    }
    pub(crate) fn evaluate_temporal_minor(
        &self,
        call: &ScaleBreaksOperation,
        domain: &[ScaleKey],
        major: &[ScaleKey],
        names: Option<&[String]>,
        temporal: super::GuideTemporalContext<'_>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.evaluate_count(
            call,
            Some(domain),
            None,
            false,
            Some(temporal),
            Some((major, names)),
        )
    }
    fn evaluate_count(
        &self,
        call: &ScaleBreaksOperation,
        domain: Option<&[ScaleKey]>,
        count: Option<f64>,
        binned: bool,
        temporal: Option<super::GuideTemporalContext<'_>>,
        major_breaks: Option<(&[ScaleKey], Option<&[String]>)>,
    ) -> ChartResult<ScaleBreaksOutput> {
        self.validate(call, false)?;
        let registration = self.registration(&call.operation)?;
        let count_argument = if count.is_none() {
            None
        } else if binned && registration.accepts_n_breaks {
            Some("n.breaks")
        } else if registration.accepts_n {
            Some("n")
        } else {
            None
        };
        let result = registration.implementation.evaluate(ScaleBreaksInput {
            domain: domain.unwrap_or(&[]),
            domain_is_null: domain.is_none(),
            major_breaks: major_breaks
                .filter(|_| registration.accepts_major_breaks)
                .map(|(values, _)| values),
            major_break_names: major_breaks
                .filter(|_| registration.accepts_major_breaks)
                .and_then(|(_, names)| names),
            count: count_argument.and(count),
            count_argument,
            temporal,
            parameters: &call.parameters,
        })?;
        if let Some(keys) = &result.values {
            crate::limits::require_within(
                keys.len() <= crate::interpolate::MAX_VALUES,
                "scale break output",
            )?;
            let mut bytes = 0usize;
            for key in keys {
                let value = crate::scales::GgplotDiscreteIdentity::map(Some(key))?;
                value.validate()?;
                if let ScaleKey::Text(text) = key {
                    bytes = bytes.saturating_add(text.len());
                    crate::limits::require_within(
                        bytes <= crate::interpolate::MAX_VALUE_BYTES,
                        "scale break text",
                    )?;
                }
            }
        }
        if let Some(names) = &result.names {
            crate::limits::require_within(
                names.len() == result.values.as_ref().map_or(0, Vec::len)
                    && names.len() <= crate::interpolate::MAX_VALUES,
                "scale break names",
            )?;
            crate::limits::require_within(
                names.iter().fold(0usize, |n, s| n.saturating_add(s.len()))
                    <= crate::interpolate::MAX_VALUE_BYTES,
                "scale break name bytes",
            )?;
        }
        Ok(result)
    }
}
impl ExtensionRegistry {
    /// Install a pure break function without replacing an existing version.
    pub fn register_scale_breaks(
        &mut self,
        implementation: Arc<dyn CustomScaleBreaks>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        let entries = &mut Arc::make_mut(&mut self.breaks_function).entries;
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A scale break function version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "registered scale break function")?;
        entries.insert(
            key,
            Registration {
                descriptor,
                accepts_n: implementation.accepts_n(),
                accepts_n_breaks: implementation.accepts_n_breaks(),
                accepts_major_breaks: implementation.accepts_major_breaks(),
                implementation,
            },
        );
        Ok(())
    }
    /// Read the captured identity without running the installed function.
    pub fn scale_breaks_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .breaks_function
            .registration(operation)?
            .descriptor
            .clone())
    }
}
