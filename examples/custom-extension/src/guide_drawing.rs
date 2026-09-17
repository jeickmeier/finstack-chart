//! A compact strip guide demonstrating training context and shared vector output.
use chart_core::{ChartResult, Revision, grammar::*, path::Path};
use std::sync::Arc;
/// Portable guide identity.
pub const STRIP: &str = "example.strip_guide";
/// Native-only counterpart.
pub const NATIVE_STRIP: &str = "example.native_strip_guide";
struct Strip {
    portable: bool,
}
impl CustomGuideDrawing for Strip {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable { STRIP } else { NATIVE_STRIP },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        if p.as_object().is_none_or(|p| !p.is_empty()) {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Strip guide accepts an empty parameter object.",
                "Remove unknown parameters.",
            ));
        }
        Ok(())
    }
    fn draw(&self, input: GuideDrawingInput<'_>) -> ChartResult<KeyGlyphOutput> {
        let colors: Vec<_> = input
            .color_guide
            .filter(|c| !c.colorbar.is_empty())
            .map_or_else(
                || input.colors.to_vec(),
                |c| c.colorbar.iter().map(|s| s.color).collect(),
            );
        if colors.is_empty() {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Strip guide needs trained colors.",
                "Map a color or fill scale.",
            ));
        }
        let w = 4. * input.font_size;
        let h = input.font_size;
        let n = colors.len() as f64;
        let paths = colors
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                let mut p = Path::new();
                p.rect(i as f64 * w / n, 0., w / n, h)?;
                Ok(CustomGuidePath {
                    geometry: p.geometry(),
                    fill: Some(color.into()),
                    stroke: None,
                })
            })
            .collect::<ChartResult<Vec<_>>>()?;
        Ok(KeyGlyphOutput {
            size: [w, h],
            paths,
        })
    }
}
/// Install external portable/native implementations into the shared registry.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_guide_drawing(Arc::new(Strip { portable: true }))?;
    registry.register_guide_drawing(Arc::new(Strip { portable: false }))
}
