//! Explicit versioned interpolation factories, sharing the canonical scalar/color kernels.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{
        CustomInterpolationFactory, ExtensionDescriptor, ExtensionRegistry, InterpolationInput,
    },
    interpolate::{ColorInterpolator, ColorRoute, Sample, ScalarInterpolator, Value},
};
use std::sync::Arc;

/// Install portable and native-only examples. No host callback execution is implied.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_interpolation(Arc::new(Factory { portable: true }))?;
    registry.register_interpolation(Arc::new(Factory { portable: false }))
}
struct Factory {
    portable: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    mode: Mode,
}
#[derive(serde::Deserialize)]
enum Mode {
    SquaredNumber,
    LabColor,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
impl CustomInterpolationFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.interpolation"
            } else {
                "example.native_interpolation"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn compile(
        &self,
        input: InterpolationInput<'_>,
    ) -> ChartResult<Arc<dyn Sample<Value> + Send + Sync>> {
        Ok(match parameters(input.parameters)?.mode {
            Mode::SquaredNumber => {
                let (Value::Number(a), Value::Number(b)) = (input.source, input.target) else {
                    return Err(super::error(
                        DiagnosticCode::SchemaConflict,
                        "SquaredNumber requires numeric endpoints.",
                    ));
                };
                if !a.0.is_finite() || !b.0.is_finite() {
                    return Err(super::error(
                        DiagnosticCode::NumericalDomain,
                        "SquaredNumber requires finite endpoints.",
                    ));
                }
                Arc::new(Squared(ScalarInterpolator::number(a.0, b.0)))
            }
            Mode::LabColor => {
                fn color(value: &Value) -> ChartResult<chart_core::color::ColorValue> {
                    match value {
                        Value::Color(value) => Ok(*value),
                        Value::Text(text) => chart_core::color::parse(text)?.ok_or_else(|| {
                            super::error(DiagnosticCode::Validation, "Expected a color.")
                        }),
                        _ => Err(super::error(
                            DiagnosticCode::SchemaConflict,
                            "LabColor requires color endpoints.",
                        )),
                    }
                }
                Arc::new(Lab(ColorInterpolator::new(
                    ColorRoute::Lab,
                    color(input.source)?,
                    color(input.target)?,
                    None,
                )?))
            }
        })
    }
}
struct Squared(ScalarInterpolator);
impl Sample<Value> for Squared {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        self.0.sample(t * t).map(Value::number)
    }
}
struct Lab(ColorInterpolator);
impl Sample<Value> for Lab {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        self.0.sample_color(t).map(Value::Color)
    }
}
