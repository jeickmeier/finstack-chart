//! A versioned diamond key receives resolved aesthetics through the public protocol.
use chart_core::{ChartResult, Revision, grammar::*, path::Path};
use std::sync::Arc;
/// Portable example identity.
pub const DIAMOND: &str = "example.diamond_key";
/// Native-only example identity, rejected at portable serialization.
pub const NATIVE_DIAMOND: &str = "example.native_diamond_key";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    padding: f64,
}
struct Diamond {
    portable: bool,
}
impl CustomKeyGlyph for Diamond {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                DIAMOND
            } else {
                NATIVE_DIAMOND
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()> {
        let p: Parameters = serde_json::from_value(parameters.clone()).map_err(|_| {
            chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Diamond key requires only a numeric padding parameter.",
                "Supply padding in 0..0.5.",
            )
        })?;
        if !p.padding.is_finite() || !(0. ..0.5).contains(&p.padding) {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Diamond padding must be in 0..0.5.",
                "Choose a finite local fraction.",
            ));
        }
        Ok(())
    }
    fn draw(&self, input: KeyGlyphInput<'_>) -> ChartResult<KeyGlyphOutput> {
        self.validate(input.parameters)?;
        let p: Parameters =
            serde_json::from_value(input.parameters.clone()).expect("validated parameters");
        let w = input.bounds.width();
        let h = input.bounds.height();
        let dx = w * p.padding;
        let dy = h * p.padding;
        let mut path = Path::new();
        path.move_to(w / 2., dy)?;
        path.line_to(w - dx, h / 2.)?;
        path.line_to(w / 2., h - dy)?;
        path.line_to(dx, h / 2.)?;
        path.close_path()?;
        Ok(KeyGlyphOutput {
            size: [w, h],
            paths: vec![CustomGuidePath {
                geometry: path.geometry(),
                fill: Some(input.fill.unwrap_or(input.color).into()),
                stroke: (input.stroke_width > 0.).then_some(chart_core::scene::Stroke {
                    color: input.stroke,
                    width: input.stroke_width,
                }),
            }],
        })
    }
}
/// Install both identities into the shared registry used by Rust/Python/WASM proofs.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_key_glyph(Arc::new(Diamond { portable: true }))?;
    registry.register_key_glyph(Arc::new(Diamond { portable: false }))
}
