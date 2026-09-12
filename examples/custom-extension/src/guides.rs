//! Portable guide-label example implemented through the public semantic callback contract.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideFormatInput},
};
use serde_json::Value;
use std::sync::Arc;

/// Install matching portable and native-only versions for host capability proofs.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_guide_formatter(Arc::new(Formatter { portable: true }))?;
    registry.register_guide_formatter(Arc::new(Formatter { portable: false }))
}
struct Formatter {
    portable: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    mode: Mode,
}
#[derive(serde::Deserialize)]
enum Mode {
    Blank,
    Same,
    Indexed,
    Context,
}
fn parameters(value: &Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.guide_format"
            } else {
                "example.native_guide_format"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn format(&self, input: GuideFormatInput<'_>) -> ChartResult<String> {
        let mode = parameters(input.parameters)?.mode;
        let value = match input.value {
            ScaleValue::Number(n) => n.to_string(),
            ScaleValue::Category(label) => label.clone(),
            ScaleValue::MissingCategory => "NA".into(),
            ScaleValue::Timestamp { value, .. } => value.to_string(),
        };
        Ok(match mode {
            Mode::Blank => String::new(),
            Mode::Same => "same".into(),
            Mode::Indexed => format!("{}:{value}", input.index),
            Mode::Context => format!("{}:{}:{value}", input.values.len(), input.index),
        })
    }
}
