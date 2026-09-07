//! Versioned headless theme cascade. Presentation never changes statistical populations.
use crate::scene::Color;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Construct an opaque sRGB token.
pub const fn rgb(red: u8, green: u8, blue: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha: 255,
    }
}
/// Supplied complete destination-independent themes.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedTheme {
    /// Light editorial publication.
    Editorial,
    /// Dense dark desktop.
    Terminal,
    /// Monochrome print.
    Grayscale,
}
/// Explicit presentation color policy; original mapped color values remain in semantic metadata.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Preserve resolved sRGB colors.
    #[default]
    Preserve,
    /// Convert final paint to sRGB luminance for monochrome output.
    Grayscale,
}
/// A point symbol, lowered to ordinary shared vector primitives.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Symbol {
    /// Circular symbol.
    #[default]
    Circle,
    /// Axis-aligned square.
    Square,
    /// Four-vertex diamond.
    Diamond,
    /// Upward triangle.
    Triangle,
}
/// Presentation overrides; absence inherits the previous cascade level.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ThemePatch {
    /// Explicit final color conversion policy, including mapped paints and legend swatches.
    pub color_mode: Option<ColorMode>,
    /// Optional full-panel gradient, replacing the solid panel fill.
    pub gradient: Option<crate::scene::LinearGradient>,
    /// Figure background.
    pub background: Option<Color>,
    /// Panel fill.
    pub panel: Option<Color>,
    /// Text/guide foreground.
    pub foreground: Option<Color>,
    /// Grid color (alpha zero disables grids).
    pub grid: Option<Color>,
    /// Explicit constant mark override; mapped colors remain authoritative.
    pub mark: Option<Color>,
    /// Annotation text/callout color.
    pub annotation: Option<Color>,
    /// Visible focus accent.
    pub focus: Option<Color>,
    /// Selection accent.
    pub selection: Option<Color>,
    /// Label font size in destination units.
    pub font_size: Option<f64>,
    /// Figure/panel padding.
    pub padding: Option<f64>,
    /// Label/legend separation.
    pub gap: Option<f64>,
    /// Tick length.
    pub tick_length: Option<f64>,
    /// Axis/mark stroke width; mapped sizes remain authoritative.
    pub stroke_width: Option<f64>,
    /// Alternating positive on/off lengths, empty means solid; at most 16 entries.
    pub dashes: Option<Vec<f64>>,
    /// Point symbol; size continues to come from the layer encoding.
    pub symbol: Option<Symbol>,
}
impl ThemePatch {
    /// Overlay explicitly present values without string heuristics.
    pub fn overlay(&mut self, next: &Self) {
        macro_rules! copy {($($field:ident),*)=>{$(if next.$field.is_some(){self.$field=next.$field;})*};}
        copy!(
            color_mode,
            gradient,
            background,
            panel,
            foreground,
            grid,
            mark,
            annotation,
            focus,
            selection,
            font_size,
            padding,
            gap,
            tick_length,
            stroke_width,
            symbol
        );
        if let Some(d) = &next.dashes {
            self.dashes = Some(d.clone());
        }
    }
    /// Reject nonfinite, negative and unbounded typography/paint parameters.
    pub fn validate(&self) -> ChartResult<()> {
        let bad_positive = [self.font_size, self.stroke_width]
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite() || v <= 0. || v > 4096.);
        let bad_spacing = [self.padding, self.gap, self.tick_length]
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite() || !(0. ..=4096.).contains(&v));
        let bad_dash = self.dashes.as_ref().is_some_and(|d| {
            d.len() > 16
                || d.len() % 2 != 0
                || d.iter().any(|v| !v.is_finite() || *v <= 0. || *v > 4096.)
        });
        if bad_positive || bad_spacing || bad_dash {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Invalid theme typography, spacing or dash tokens.",
                "Use bounded finite sizes and even positive on/off dash lengths.",
            ));
        }
        Ok(())
    }
}
impl NamedTheme {
    /// Complete named tokens; mark colors are deliberately inherited from explicit encodings.
    pub fn tokens(self) -> ThemePatch {
        let dark = self == Self::Terminal;
        let foreground = if dark {
            rgb(225, 234, 242)
        } else {
            rgb(40, 45, 50)
        };
        ThemePatch {
            gradient: None,
            color_mode: Some(if self == Self::Grayscale {
                ColorMode::Grayscale
            } else {
                ColorMode::Preserve
            }),
            background: Some(if dark {
                rgb(17, 23, 31)
            } else {
                rgb(255, 255, 255)
            }),
            panel: Some(if dark {
                rgb(23, 31, 41)
            } else {
                rgb(255, 255, 255)
            }),
            foreground: Some(foreground),
            grid: Some(if dark {
                rgb(54, 66, 79)
            } else {
                rgb(229, 232, 235)
            }),
            annotation: Some(foreground),
            focus: Some(if dark {
                rgb(255, 213, 79)
            } else {
                rgb(28, 91, 162)
            }),
            selection: Some(if dark {
                rgb(82, 197, 173)
            } else {
                rgb(116, 66, 150)
            }),
            font_size: Some(if dark { 10. } else { 12. }),
            padding: Some(if dark { 6. } else { 10. }),
            gap: Some(if dark { 3. } else { 5. }),
            tick_length: Some(4.),
            stroke_width: Some(1.),
            dashes: Some(vec![]),
            symbol: Some(Symbol::Circle),
            mark: None,
        }
    }
}
/// Version-one named theme plus portable plot/layer overrides.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeSpec {
    /// Supported version is one.
    pub version: u32,
    /// Optional named theme; absent preserves the host/default cascade.
    pub named: Option<NamedTheme>,
    /// Plot-level overrides.
    #[serde(default)]
    pub plot: ThemePatch,
    /// Layer overrides applied after plot tokens and before interaction/output patches.
    #[serde(default)]
    pub layers: BTreeMap<LayerId, ThemePatch>,
}
impl ThemeSpec {
    /// Select a supplied theme.
    pub fn named(named: NamedTheme) -> Self {
        Self {
            version: 1,
            named: Some(named),
            plot: ThemePatch::default(),
            layers: BTreeMap::new(),
        }
    }
    /// Defaults -> host -> named -> plot; layer/interaction/output resolve at presentation.
    pub fn resolve(&self, host: &ThemePatch) -> ChartResult<ThemePatch> {
        if self.layers.len() > 256 {
            return Err(Diagnostic::error(
                DiagnosticCode::ResourceLimit,
                "Theme layer override count exceeds 256.",
                "Use a bounded chart theme.",
            ));
        }
        if self.version != 1 {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported theme version.",
                "Use theme version one.",
            ));
        }
        host.validate()?;
        self.plot.validate()?;
        for layer in self.layers.values() {
            layer.validate()?;
        }
        let mut t = ThemePatch::default();
        t.overlay(host);
        if let Some(n) = self.named {
            t.overlay(&n.tokens());
        }
        t.overlay(&self.plot);
        Ok(t)
    }
}

/// Apply the declared monochrome conversion after resolving constant or mapped colors.
pub fn paint_color(c: Color, mode: Option<ColorMode>) -> Color {
    if mode != Some(ColorMode::Grayscale) {
        return c;
    }
    let y = (0.2126 * f64::from(c.red) + 0.7152 * f64::from(c.green) + 0.0722 * f64::from(c.blue))
        .round() as u8;
    Color {
        red: y,
        green: y,
        blue: y,
        alpha: c.alpha,
    }
}
