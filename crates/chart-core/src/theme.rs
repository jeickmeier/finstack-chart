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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(
    default,
    deny_unknown_fields,
    bound(deserialize = "P: Deserialize<'de>")
)]
pub struct ThemePatch<P = Color> {
    /// Explicit final color conversion policy, including mapped paints and legend swatches.
    pub color_mode: Option<ColorMode>,
    /// Optional full-panel gradient, replacing the solid panel fill.
    pub gradient: Option<crate::scene::LinearGradient<P>>,
    /// Figure background.
    pub background: Option<P>,
    /// Panel fill.
    pub panel: Option<P>,
    /// Text/guide foreground.
    pub foreground: Option<P>,
    /// Grid color (alpha zero disables grids).
    pub grid: Option<P>,
    /// Explicit constant mark override; mapped colors remain authoritative.
    pub mark: Option<P>,
    /// Annotation text/callout color.
    pub annotation: Option<P>,
    /// Visible focus accent.
    pub focus: Option<P>,
    /// Selection accent.
    pub selection: Option<P>,
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
impl<P: Copy> ThemePatch<P> {
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
/// A retained reference theme palette, converted only when a scale selects it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ThemeScalePalette {
    /// Case-insensitive identity from the pinned reference palette registry.
    Named(String),
    /// Pure installed count/vector function; retains the v45 wire representation.
    Registered(crate::grammar::ScalePaletteOperation),
    /// Reference color vector. Missing entries differ from transparent color text.
    Colors(Vec<Option<String>>),
}
impl From<crate::grammar::ScalePaletteOperation> for ThemeScalePalette {
    fn from(value: crate::grammar::ScalePaletteOperation) -> Self {
        Self::Registered(value)
    }
}

/// Versioned named theme, palette selections and portable plot/layer overrides.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeSpec {
    /// Explicit supplied family/face resources for reference element text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<crate::grammar::TextFont>,
    /// Optional reference element hierarchy, resolved before flat presentation overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<ElementTheme>,
    /// Reference palettes keyed by `palette.<aesthetic>.<continuous|discrete>`.
    /// They are evaluated by the common scale engine, before guide preparation.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scale_palettes: BTreeMap<String, ThemeScalePalette>,
    /// Version-two geometry defaults available to theme-derived expressions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryTheme<crate::color::Paint>>,
    /// One for legacy, two for geometry, three for registered palettes, four for vectors, five for names.
    pub version: u32,
    /// Optional named theme; absent preserves the host/default cascade.
    pub named: Option<NamedTheme>,
    /// Plot-level overrides.
    #[serde(default)]
    pub plot: ThemePatch<crate::color::Paint>,
    /// Layer overrides applied after plot tokens and before interaction/output patches.
    #[serde(default)]
    pub layers: BTreeMap<LayerId, ThemePatch<crate::color::Paint>>,
}
impl ThemeSpec {
    pub(crate) fn required_version(&self) -> u32 {
        if self.hierarchy.is_some() || !self.fonts.is_empty() {
            6
        } else if self.scale_palettes.values().any(|p| {
            matches!(p, ThemeScalePalette::Named(_))
                || matches!(p, ThemeScalePalette::Colors(colors) if colors.len() == 1)
        }) {
            5
        } else if self
            .scale_palettes
            .values()
            .any(|p| matches!(p, ThemeScalePalette::Colors(_)))
        {
            4
        } else if !self.scale_palettes.is_empty() {
            3
        } else if self.geometry.is_some() {
            2
        } else {
            1
        }
    }
    /// Select a supplied theme.
    pub fn named(named: NamedTheme) -> Self {
        Self {
            version: 1,
            hierarchy: None,
            fonts: Vec::new(),
            scale_palettes: BTreeMap::new(),
            geometry: None,
            named: Some(named),
            plot: ThemePatch::default(),
            layers: BTreeMap::new(),
        }
    }
    /// Defaults -> host -> named -> plot; layer/interaction/output resolve at presentation.
    pub fn resolve<P: Copy + Into<crate::color::Paint>>(
        &self,
        host: &ThemePatch<P>,
    ) -> ChartResult<ThemePatch> {
        if self.layers.len() > 256 {
            return Err(Diagnostic::error(
                DiagnosticCode::ResourceLimit,
                "Theme layer override count exceeds 256.",
                "Use a bounded chart theme.",
            ));
        }
        let version = self.required_version();
        if self.version != version {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported theme version.",
                "Use version four for color vectors, three for registered palettes, two for geometry defaults, or one for legacy themes.",
            ));
        }
        if self.scale_palettes.len() > 128
            || self.scale_palettes.keys().any(|key| {
                key.strip_prefix("palette.")
                    .and_then(|key| key.rsplit_once('.'))
                    .is_none_or(|(aesthetic, family)| {
                        !palette_aesthetic(aesthetic)
                            || !matches!(family, "continuous" | "discrete")
                    })
            })
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Invalid reference theme palette keys or palette count.",
                "Supply at most 128 palette.<aesthetic>.<continuous|discrete> registrations.",
            ));
        }
        let font_options = crate::grammar::TextGeom {
            fonts: self.fonts.clone(),
            ..Default::default()
        };
        font_options.validate()?;
        if let Some(hierarchy) = &self.hierarchy {
            hierarchy.validate()?;
        }
        if let Some(geometry) = &self.geometry {
            geometry.validate()?;
        }
        host.validate()?;
        self.plot.validate()?;
        for layer in self.layers.values() {
            layer.validate()?;
        }
        let mut t = ThemePatch::default();
        t.overlay(&host.clone().map_colors(|p| p.into().resolve()));
        if let Some(n) = self.named {
            t.overlay(&n.tokens());
        }
        if let Some(elements) = self.resolved_elements()? {
            let transparent = Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            };
            for (node, token) in [
                ("plot.background", &mut t.background),
                ("panel.background", &mut t.panel),
            ] {
                *token = if elements.blank(node) {
                    Some(transparent)
                } else {
                    elements
                        .paint(node, "fill")?
                        .map(crate::color::Paint::resolve)
                };
            }
            t.foreground = elements
                .paint("text", "colour")?
                .map(crate::color::Paint::resolve);
            t.annotation = t.foreground;
            t.grid = if elements.blank("panel.grid.major") {
                Some(transparent)
            } else {
                elements
                    .paint("panel.grid.major", "colour")?
                    .map(crate::color::Paint::resolve)
            };
        }
        t.overlay(&self.plot.resolve());
        Ok(t)
    }
}

pub(crate) fn palette_aesthetic(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
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

/// Geometry theme values corresponding to ggplot2 4.0.3 element_geom defaults.
/// Physical size conversion belongs to the aesthetic and destination contracts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GeometryTheme<P = Color> {
    /// Foreground geometry color.
    pub ink: P,
    /// Background geometry color.
    pub paper: P,
    /// Accent geometry color.
    pub accent: P,
    /// Nonnegative semantic point size.
    pub point_size: f64,
    /// Nonnegative semantic line width; zero follows the shared reference hairline policy.
    pub line_width: f64,
}
impl<P: From<Color>> Default for GeometryTheme<P> {
    fn default() -> Self {
        Self {
            ink: rgb(0, 0, 0).into(),
            paper: rgb(255, 255, 255).into(),
            accent: rgb(51, 102, 255).into(),
            point_size: 1.5,
            line_width: 0.5,
        }
    }
}
impl<P> GeometryTheme<P> {
    /// Reject invalid size tokens before evaluating any geometry expression.
    pub fn validate(&self) -> ChartResult<()> {
        if !self.point_size.is_finite()
            || self.point_size < 0.
            || !self.line_width.is_finite()
            || self.line_width < 0.
        {
            return Err(Diagnostic::error(
                DiagnosticCode::NumericalDomain,
                "Geometry theme sizes must be finite and nonnegative.",
                "Supply nonnegative point size and line width.",
            ));
        }
        Ok(())
    }
    /// Transform colors without changing semantic size tokens.
    pub fn map_colors<Q>(self, mut map: impl FnMut(P) -> Q) -> GeometryTheme<Q> {
        GeometryTheme {
            ink: map(self.ink),
            paper: map(self.paper),
            accent: map(self.accent),
            point_size: self.point_size,
            line_width: self.line_width,
        }
    }
}
impl GeometryTheme<crate::color::Paint> {
    /// Prepare geometry colors for post-scale expression reads.
    pub fn resolve(&self) -> GeometryTheme {
        self.clone().map_colors(crate::color::Paint::resolve)
    }
}

impl<P> Default for ThemePatch<P> {
    fn default() -> Self {
        Self {
            gradient: None,
            background: None,
            panel: None,
            foreground: None,
            grid: None,
            mark: None,
            annotation: None,
            focus: None,
            selection: None,
            color_mode: None,
            font_size: None,
            padding: None,
            gap: None,
            tick_length: None,
            stroke_width: None,
            dashes: None,
            symbol: None,
        }
    }
}
impl<P> ThemePatch<P> {
    /// Transform only color inputs, preserving all cascade and typography fields.
    pub fn map_colors<Q>(self, mut map: impl FnMut(P) -> Q) -> ThemePatch<Q> {
        ThemePatch {
            gradient: self.gradient.map(|g| g.map_colors(&mut map)),
            background: self.background.map(&mut map),
            panel: self.panel.map(&mut map),
            foreground: self.foreground.map(&mut map),
            grid: self.grid.map(&mut map),
            mark: self.mark.map(&mut map),
            annotation: self.annotation.map(&mut map),
            focus: self.focus.map(&mut map),
            selection: self.selection.map(&mut map),
            color_mode: self.color_mode,
            font_size: self.font_size,
            padding: self.padding,
            gap: self.gap,
            tick_length: self.tick_length,
            stroke_width: self.stroke_width,
            dashes: self.dashes,
            symbol: self.symbol,
        }
    }
}
impl ThemePatch<crate::color::Paint> {
    /// Prepare all constant colors before iterating scene items.
    pub fn resolve(&self) -> ThemePatch {
        self.clone().map_colors(crate::color::Paint::resolve)
    }
    /// Whether this patch retains any floating input.
    pub fn has_floating(&self) -> bool {
        [
            self.background,
            self.panel,
            self.foreground,
            self.grid,
            self.mark,
            self.annotation,
            self.focus,
            self.selection,
        ]
        .into_iter()
        .flatten()
        .any(crate::color::Paint::is_floating)
            || self
                .gradient
                .is_some_and(|g| g.start.is_floating() || g.end.is_floating())
    }
}
impl ThemeSpec {
    /// Whether the retained theme requires the floating-color definition capability.
    pub fn has_floating_paint(&self) -> bool {
        self.plot.has_floating()
            || self.layers.values().any(ThemePatch::has_floating)
            || self.geometry.as_ref().is_some_and(|g| {
                [g.ink, g.paper, g.accent]
                    .into_iter()
                    .any(crate::color::Paint::is_floating)
            })
    }
}

mod elements;
mod hierarchy;
pub use elements::{ElementKind, ElementTheme, ThemeElement, ThemeEntry, ThemeLength, ThemeValue};
pub use hierarchy::{ThemeContext, ThemePreset};

mod presets;
pub use presets::ThemePresetOptions;
mod consumers;
pub use consumers::ResolvedElements;
