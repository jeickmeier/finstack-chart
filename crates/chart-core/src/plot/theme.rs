use super::LayerHandle;
use crate::{
    color::Paint,
    scene::LinearGradient,
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
            geometry: None,
            named: None,
            plot: ThemePatch::default(),
            layers: Default::default(),
        },
    }
}
/// Reusable plot/layer/destination style tokens.
#[derive(Clone, Debug, Default)]
pub struct StyleBuilder {
    pub(super) patch: ThemePatch<Paint>,
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
    /// Set the background color, retaining floating inputs until preparation.
    pub fn background(mut self, value: impl Into<Paint>) -> Self {
        self.patch.background = Some(value.into());
        self
    }
    /// Set the panel color, retaining floating inputs until preparation.
    pub fn panel(mut self, value: impl Into<Paint>) -> Self {
        self.patch.panel = Some(value.into());
        self
    }
    /// Set the foreground color, retaining floating inputs until preparation.
    pub fn foreground(mut self, value: impl Into<Paint>) -> Self {
        self.patch.foreground = Some(value.into());
        self
    }
    /// Set the grid color, retaining floating inputs until preparation.
    pub fn grid(mut self, value: impl Into<Paint>) -> Self {
        self.patch.grid = Some(value.into());
        self
    }
    /// Set the mark color, retaining floating inputs until preparation.
    pub fn mark(mut self, value: impl Into<Paint>) -> Self {
        self.patch.mark = Some(value.into());
        self
    }
    /// Set the annotation color, retaining floating inputs until preparation.
    pub fn annotation(mut self, value: impl Into<Paint>) -> Self {
        self.patch.annotation = Some(value.into());
        self
    }
    /// Set the focus color, retaining floating inputs until preparation.
    pub fn focus(mut self, value: impl Into<Paint>) -> Self {
        self.patch.focus = Some(value.into());
        self
    }
    /// Set the selection color, retaining floating inputs until preparation.
    pub fn selection(mut self, value: impl Into<Paint>) -> Self {
        self.patch.selection = Some(value.into());
        self
    }
    /// Set gradient endpoints without intermediate color quantization.
    pub fn gradient<P: Into<Paint>>(mut self, value: LinearGradient<P>) -> Self {
        self.patch.gradient = Some(value.map_colors(Into::into));
        self
    }

    tokens!(color_mode: ColorMode => "Set final output color conversion.", font_size: f64 => "Set label size in destination units.", padding: f64 => "Set figure/panel padding.", gap: f64 => "Set label/legend separation.", tick_length: f64 => "Set tick length.", stroke_width: f64 => "Set constant stroke width.", dashes: Vec<f64> => "Set alternating positive on/off lengths.", symbol: Symbol => "Set a constant point symbol.");
}
impl ThemeBuilder {
    /// Set geometry defaults and tokens available to theme-derived expressions.
    pub fn geometry<P: Into<Paint>>(mut self, geometry: crate::theme::GeometryTheme<P>) -> Self {
        self.spec.version = 2;
        self.spec.geometry = Some(geometry.map_colors(Into::into));
        self
    }
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
