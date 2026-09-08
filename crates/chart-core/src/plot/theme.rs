use super::LayerHandle;
use crate::{
    scene::{Color, LinearGradient},
    theme::{ColorMode, NamedTheme, Symbol, ThemePatch, ThemeSpec},
};
/// Complete theme configuration over the existing theme cascade.
#[derive(Clone, Debug)]
pub struct ThemeBuilder {
    pub(super) spec: ThemeSpec,
}
/// Start with destination/default tokens and optional plot/layer overrides.
pub fn theme() -> ThemeBuilder {
    ThemeBuilder {
        spec: ThemeSpec {
            version: 1,
            named: None,
            plot: ThemePatch::default(),
            layers: Default::default(),
        },
    }
}
/// Reusable plot/layer/destination style tokens.
#[derive(Clone, Debug, Default)]
pub struct StyleBuilder {
    pub(super) patch: ThemePatch,
}
/// Start an inherited presentation style without changing data/statistics.
pub fn style() -> StyleBuilder {
    StyleBuilder::default()
}
macro_rules! tokens {
    ($($name:ident: $ty:ty => $doc:literal),* $(,)?) => {$(
        #[doc = $doc]
        pub fn $name(mut self, value: $ty) -> Self { self.patch.$name = Some(value); self }
    )*};
}
impl StyleBuilder {
    tokens!(color_mode: ColorMode => "Set final output color conversion.", gradient: LinearGradient => "Set a full-panel gradient.", background: Color => "Set figure background.", panel: Color => "Set panel background.", foreground: Color => "Set guide/text foreground.", grid: Color => "Set grid color; alpha zero disables it.", mark: Color => "Override constant mark color; mapped colors remain authoritative.", annotation: Color => "Set annotation/callout color.", focus: Color => "Set focus accent.", selection: Color => "Set selection accent.", font_size: f64 => "Set label size in destination units.", padding: f64 => "Set figure/panel padding.", gap: f64 => "Set label/legend separation.", tick_length: f64 => "Set tick length.", stroke_width: f64 => "Set constant stroke width.", dashes: Vec<f64> => "Set alternating positive on/off lengths.", symbol: Symbol => "Set a constant point symbol.");
}
impl ThemeBuilder {
    /// Select an existing complete named theme.
    pub fn preset(mut self, preset: NamedTheme) -> Self {
        self.spec.named = Some(preset);
        self
    }
    /// Overlay plot-wide style tokens.
    pub fn style(mut self, style: StyleBuilder) -> Self {
        self.spec.plot.overlay(&style.patch);
        self
    }
    /// Overlay tokens for one stable layer.
    pub fn layer(mut self, layer: LayerHandle, style: StyleBuilder) -> Self {
        self.spec
            .layers
            .entry(layer.0)
            .or_default()
            .overlay(&style.patch);
        self
    }
}
