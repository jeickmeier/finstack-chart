//! External live catalog planner using only the public facet protocol.
use chart_core::{ChartResult, Revision, grammar::*};
use std::sync::Arc;
/// Portable identity.
pub const REVERSE: &str = "example.reverse_facets";
/// Native-only counterpart.
pub const NATIVE_REVERSE: &str = "example.native_reverse_facets";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    columns: usize,
}
struct Reverse {
    portable: bool,
}
impl CustomFacet for Reverse {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                REVERSE
            } else {
                NATIVE_REVERSE
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        let value: Parameters = serde_json::from_value(p.clone()).map_err(|_| {
            chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Reverse facet planner requires columns.",
                "Supply a positive column count.",
            )
        })?;
        if !(1..=256).contains(&value.columns) {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Column count is outside 1..256.",
                "Supply a bounded count.",
            ));
        }
        Ok(())
    }
    fn plan(&self, input: FacetPlanInput<'_>) -> ChartResult<FacetSpec> {
        self.validate(input.parameters)?;
        let p: Parameters = serde_json::from_value(input.parameters.clone()).expect("validated");
        let mut result = input.base.clone();
        result.order.reverse();
        result.layout = FacetLayout::Wrap { columns: p.columns };
        Ok(result)
    }
}
/// Register the portable and explicit native-only protocols.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_facet(Arc::new(Reverse { portable: true }))?;
    registry.register_facet(Arc::new(Reverse { portable: false }))
}
