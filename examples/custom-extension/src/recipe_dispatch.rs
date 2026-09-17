//! Application-specific typed payload dispatch through ordinary Data and Plot builders.
use chart_core::{
    Diagnostic, DiagnosticCode, Revision,
    data::DataLimits,
    grammar::{ExtensionDescriptor, ExtensionRegistry},
    plot::{AuthoringInput, CustomAuthoring},
    prelude::*,
};
use std::sync::Arc;
/// Public recipe identity.
pub const XY: &str = "example.xy_recipe";
/// Native-only recipe identity.
pub const NATIVE_XY: &str = "example.native_xy_recipe";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    x: Vec<f64>,
    y: Vec<f64>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    line: bool,
}
struct Xy {
    portable: bool,
}
fn invalid() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        "Invalid XY recipe schema.",
        "Supply x/y numeric arrays for materialization and a line boolean for layers.",
    )
}
impl CustomAuthoring for Xy {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable { XY } else { NATIVE_XY },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        serde_json::from_value::<Parameters>(p.clone())
            .map(|_| ())
            .map_err(|_| invalid())
    }
    fn materialize(&self, payload: &serde_json::Value, limits: DataLimits) -> ChartResult<Data> {
        let p: Payload = serde_json::from_value(payload.clone()).map_err(|_| invalid())?;
        Data::columns()
            .limits(limits)
            .column("x", p.x)
            .column("y", p.y)
            .build()
    }
    fn layers(&self, input: AuthoringInput<'_>) -> ChartResult<Vec<LayerBuilder>> {
        let p: Parameters =
            serde_json::from_value(input.parameters.clone()).map_err(|_| invalid())?;
        let a = aes().x(input.data.field("x")?).y(input.data.field("y")?);
        let mut layers = vec![points().aes(a.clone()).name("observations")];
        if p.line {
            layers.push(line().aes(a).name("trend"));
        }
        Ok(layers)
    }
}
/// Install the same portable adapter in each proof host; runtime callbacks are Rust-owned.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_authoring(Arc::new(Xy { portable: true }))?;
    registry.register_authoring(Arc::new(Xy { portable: false }))
}
