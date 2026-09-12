//! Pure domain functions implemented outside core and explicitly installed by proof hosts.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{
        CustomScaleLimits, ExtensionDescriptor, ExtensionRegistry, OperationRef, ScaleLimitsInput,
        ScaleLimitsOperation,
    },
    interpolate::Number,
    scales::ScaleKey,
};
use std::sync::Arc;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Identity,
    Reverse,
    Fixed,
    Append,
    Empty,
    Numeric,
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
/// External pure limit operation, with portable and native-only registrations.
pub struct Limits {
    /// Whether this registration can execute in portable consumers.
    pub portable: bool,
}
impl CustomScaleLimits for Limits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.discrete_limits"
            } else {
                "example.native_discrete_limits"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn evaluate(&self, input: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
        Ok(match parameters(input.parameters)?.mode {
            Mode::Identity => input.domain.map(<[ScaleKey]>::to_vec),
            Mode::Reverse => input.domain.map(|v| v.iter().rev().cloned().collect()),
            Mode::Fixed => Some(["c", "a"].map(|v| ScaleKey::Text(v.into())).to_vec()),
            Mode::Append => {
                let mut keys = input.domain.unwrap_or_default().to_vec();
                keys.push(ScaleKey::Text("extra".into()));
                Some(keys)
            }
            Mode::Empty => Some(vec![]),
            Mode::Numeric => Some([3., 1.].map(|v| ScaleKey::Number(Number(v))).to_vec()),
        })
    }
}
/// Select the compiled portable function through ordinary primary scale options.
pub fn operation(mode: &str) -> ScaleLimitsOperation {
    ScaleLimitsOperation {
        operation: OperationRef {
            id: "example.discrete_limits".into(),
            version: Revision::new(1),
        },
        parameters: serde_json::json!({"mode":mode}),
    }
}
/// Install both capabilities; no source code is accepted from a chart payload.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_scale_limits(Arc::new(Limits { portable: true }))?;
    registry.register_scale_limits(Arc::new(Limits { portable: false }))
}
