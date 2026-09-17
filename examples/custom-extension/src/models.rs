//! Application fixture: a prescribed slope with a weighted residual intercept.
//! The envelope is explicitly configured, not a claim of a statistical confidence estimator.
use chart_core::{ChartResult, Diagnostic, DiagnosticCode, Revision, grammar::*};
use std::sync::Arc;
/// Portable model operation.
pub const PRESCRIBED_SLOPE: &str = "example.prescribed_slope";
/// Same implementation restricted to a native registration.
pub const NATIVE_PRESCRIBED_SLOPE: &str = "example.native_prescribed_slope";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    slope: f64,
    envelope: f64,
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Supply finite slope and nonnegative envelope parameters with aligned finite observations and positive total weight.",
    )
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    let p: Parameters = serde_json::from_value(value.clone())
        .map_err(|_| invalid("Invalid prescribed-slope parameter schema."))?;
    if !p.slope.is_finite() || !p.envelope.is_finite() || p.envelope < 0. {
        return Err(invalid("Invalid prescribed-slope parameter values."));
    }
    Ok(p)
}
/// Pure application-specific fit installed explicitly in each host.
pub struct PrescribedSlope {
    /// Whether definitions may be replayed on portable hosts.
    pub portable: bool,
}
impl CustomModel for PrescribedSlope {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                PRESCRIBED_SLOPE
            } else {
                NATIVE_PRESCRIBED_SLOPE
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn predict(
        &self,
        input: ModelInput<'_>,
        value: &serde_json::Value,
    ) -> ChartResult<Vec<ModelEstimate>> {
        let p = parameters(value)?;
        if input.x.len() != input.y.len()
            || input.x.len() != input.weights.len()
            || input.x.is_empty()
            || input
                .x
                .iter()
                .chain(input.y)
                .chain(input.weights)
                .chain(input.grid)
                .any(|v| !v.is_finite())
            || input.weights.iter().any(|v| *v < 0.)
        {
            return Err(invalid("Invalid prescribed-slope observation vectors."));
        }
        let total: f64 = input.weights.iter().sum();
        if !total.is_finite() || total <= 0. {
            return Err(invalid(
                "Prescribed-slope model requires positive total weight.",
            ));
        }
        let intercept = input
            .x
            .iter()
            .zip(input.y)
            .zip(input.weights)
            .map(|((&x, &y), &w)| (y - p.slope * x) * (w / total))
            .sum::<f64>();
        let half_width = p.envelope * input.options.level;
        Ok(input
            .grid
            .iter()
            .map(|&x| {
                let mean = p.slope * x + intercept;
                ModelEstimate {
                    mean: Some(mean),
                    standard_error: None,
                    lower: input.options.se.then_some(mean - half_width),
                    upper: input.options.se.then_some(mean + half_width),
                }
            })
            .collect())
    }
}
/// Register portable and native-only operation versions without implicit discovery.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_model(Arc::new(PrescribedSlope { portable: true }))?;
    registry.register_model(Arc::new(PrescribedSlope { portable: false }))
}
