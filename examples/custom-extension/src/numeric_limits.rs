//! External numeric limit functions selected through the shared pure registry.
use chart_core::{
    ChartResult, DiagnosticCode, Revision, grammar::*, interpolate::Number, scales::ScaleKey,
};
use std::sync::Arc;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Identity,
    Reverse,
    Fixed,
    LowerZero,
    MissingLower,
    Empty,
    Single,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    mode: Mode,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
/// Portable example; source strings are never evaluated.
pub struct Limits;
impl CustomScaleLimits for Limits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.numeric_limits", Revision::new(1), true)
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn evaluate(&self, input: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
        let factor = input.temporal.map_or(1., |t| {
            use chart_core::data::TimeUnit;
            let unit = match t.unit {
                TimeUnit::Seconds => 1.,
                TimeUnit::Milliseconds => 1_000.,
                TimeUnit::Microseconds => 1_000_000.,
                TimeUnit::Nanoseconds => 1_000_000_000.,
            };
            unit * if t.date { 86_400. } else { 1. }
        });
        let number = |n| ScaleKey::Number(Number(n * factor));
        Ok(match parameters(input.parameters)?.mode {
            Mode::Identity => input.domain.map(<[ScaleKey]>::to_vec),
            Mode::Reverse => input.domain.map(|v| v.iter().rev().cloned().collect()),
            Mode::Fixed => Some(vec![number(0.), number(10.)]),
            Mode::LowerZero | Mode::MissingLower => {
                let first = if matches!(parameters(input.parameters)?.mode, Mode::LowerZero) {
                    number(0.)
                } else {
                    ScaleKey::Null
                };
                Some(if let Some(domain) = input.domain {
                    vec![first, domain.get(1).cloned().unwrap_or(ScaleKey::Null)]
                } else {
                    vec![first]
                })
            }
            Mode::Empty => Some(vec![]),
            Mode::Single => Some(vec![number(5.)]),
        })
    }
}
/// Select the installed callback with bounded declarative parameters.
pub fn operation(mode: &str) -> ScaleLimitsOperation {
    ScaleLimitsOperation {
        operation: OperationRef {
            id: "example.numeric_limits".into(),
            version: Revision::new(1),
        },
        parameters: serde_json::json!({"mode":mode}),
    }
}
/// Install one exact operation version.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_scale_limits(Arc::new(Limits))
}
