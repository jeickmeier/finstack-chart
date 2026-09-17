//! Typed facet formatting through the existing registered guide protocol.
use chart_core::{ChartResult, DiagnosticCode, Revision, grammar::*};
use std::sync::Arc;
/// Portable context-aware facet formatter.
pub const LABELS: &str = "example.facet_labels";
/// Identical native-only implementation for portable-boundary proofs.
pub const NATIVE_LABELS: &str = "example.native_facet_labels";
struct Labels(bool);
impl CustomGuideFormatter for Labels {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.0 { LABELS } else { NATIVE_LABELS },
            Revision::new(1),
            self.0,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        match p.as_str() {
            Some("context" | "short" | "oversize" | "missing") => Ok(()),
            _ => Err(super::error(
                DiagnosticCode::Validation,
                "Expected facet label proof mode.",
            )),
        }
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        let facet = input.facet.ok_or_else(|| {
            super::error(
                DiagnosticCode::Validation,
                "Facet formatter requires typed facet context.",
            )
        })?;
        let names = input.names.ok_or_else(|| {
            super::error(
                DiagnosticCode::Validation,
                "Facet variable names are required.",
            )
        })?;
        if names.len() != facet.values.len() || input.values.len() != facet.values.len() {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "Facet values and names must align.",
            ));
        }
        if input.parameters == "short" {
            return Ok(vec![]);
        }
        if input.parameters == "oversize" {
            return Ok(vec![
                Some("x".repeat(input.limits.max_text_bytes + 1));
                facet.values.len()
            ]);
        }
        Ok(facet
            .values
            .iter()
            .zip(names)
            .map(|(value, name)| {
                if input.parameters == "missing" {
                    return None;
                }
                let kind = match value {
                    GroupValue::Missing => "missing",
                    GroupValue::All => "margin",
                    GroupValue::Text(_) => "text",
                    GroupValue::Int(_) => "int",
                    GroupValue::UInt(_) => "uint",
                    GroupValue::Boolean(_) => "bool",
                    GroupValue::Number(_) => "number",
                    GroupValue::Interaction(_) => "interaction",
                };
                Some(format!(
                    "{name}={kind}:{} [{},{} {:?} {}]",
                    value.label(),
                    facet.location.0,
                    facet.location.1,
                    facet.side,
                    if facet.grid { "grid" } else { "wrap" }
                ))
            })
            .collect())
    }
}
/// Install the same trusted code in every explicit proof registry.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_guide_formatter(Arc::new(Labels(true)))?;
    registry.register_guide_formatter(Arc::new(Labels(false)))
}
