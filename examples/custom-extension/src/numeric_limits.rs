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
    #[serde(default)]
    bounds: Option<[Number; 2]>,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    let parsed: Parameters = serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))?;
    if parsed.bounds.is_some() && !matches!(parsed.mode, Mode::Fixed) {
        return Err(super::error(
            DiagnosticCode::Validation,
            "Only fixed limits accept bounds.",
        ));
    }
    Ok(parsed)
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
            Mode::Fixed => Some(
                parameters(input.parameters)?
                    .bounds
                    .unwrap_or([Number(0.), Number(10.)])
                    .map(|v| number(v.0))
                    .to_vec(),
            ),
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
    registry.register_scale_limits(Arc::new(Limits))?;
    registry.register_scale_limits(Arc::new(ConstantLimits))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstantParameters {
    values: Option<Vec<Option<Number>>>,
}
fn constant_parameters(value: &serde_json::Value) -> ChartResult<ConstantParameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
/// Captured constant limit vector; this operation never reads its input domain.
pub struct ConstantLimits;
impl CustomScaleLimits for ConstantLimits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.constant_numeric_limits", Revision::new(1), true)
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        constant_parameters(value).map(|_| ())
    }
    fn requires_domain(&self, _: &serde_json::Value) -> bool {
        false
    }
    fn evaluate(&self, input: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
        Ok(constant_parameters(input.parameters)?.values.map(|values| {
            values
                .into_iter()
                .map(|v| v.map_or(ScaleKey::Null, ScaleKey::Number))
                .collect()
        }))
    }
}
