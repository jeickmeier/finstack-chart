//! Portable vector-label operation used by the independent ggplot2 callback proofs.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
};
use serde_json::Value;
use std::sync::Arc;

/// Install the versioned scale-label operation in each explicit proof registry.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_guide_formatter(Arc::new(Formatter))
}
/// Pure label function used by the installed proof hosts and recording tests.
pub struct Formatter;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Indexed,
    FixedTwo,
    Missing,
    Short,
    Empty,
    Named,
}
fn mode(value: &Value) -> ChartResult<Mode> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.scale_labels", Revision::new(1), true)
    }
    fn validate(&self, parameters: &Value) -> ChartResult<()> {
        mode(parameters).map(|_| ())
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        if matches!(mode(input.parameters)?, Mode::FixedTwo) {
            return input
                .values
                .iter()
                .map(|value| {
                    let chart_core::composition::ScaleValue::Number(value) = value else {
                        return Err(super::error(
                            DiagnosticCode::Validation,
                            "Fixed decimal callback requires numbers.",
                        ));
                    };
                    if !value.is_finite() {
                        return Ok(None);
                    }
                    let rounded = (value * 100.).round_ties_even() / 100.;
                    Ok(Some(format!(
                        "{:.2}",
                        if rounded == 0. { 0. } else { rounded }
                    )))
                })
                .collect();
        }
        let mut labels = if input.values.is_empty() {
            vec![Some("/0".into())]
        } else {
            (1..=input.values.len())
                .map(|i| Some(format!("{i}/{}", input.values.len())))
                .collect::<Vec<_>>()
        };
        match mode(input.parameters)? {
            Mode::Missing => {
                for (i, label) in labels.iter_mut().enumerate() {
                    if i % 2 == 1 {
                        *label = None;
                    }
                }
            }
            Mode::Short => labels.truncate(1),
            Mode::Empty => labels.clear(),
            Mode::Indexed | Mode::Named | Mode::FixedTwo => {}
        }
        Ok(labels)
    }
}
