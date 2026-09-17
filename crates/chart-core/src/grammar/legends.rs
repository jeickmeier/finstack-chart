//! Portable guide composition and per-layer key selection.
use crate::{ChartResult, DiagnosticCode, color::Paint};
/// A nonpositional channel that can contribute a legend key.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum LegendAesthetic {
    /// Shared color channel.
    Color,
    /// Interior paint.
    Fill,
    /// Outline paint.
    Stroke,
    /// Point size.
    Size,
    /// Symbol area.
    AreaSize,
    /// Paint alpha.
    Alpha,
    /// Opacity multiplier.
    Opacity,
    /// Line width.
    StrokeWidth,
    /// Point topology.
    Shape,
    /// Line pattern.
    LineType,
}
/// Layer key topology; automatic uses the actual geometry family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyGlyph {
    /// The layer geometry's key.
    #[default]
    Auto,
    /// Circle/shape key.
    Point,
    /// Line sample.
    Line,
    /// Filled rectangle.
    Rectangle,
    /// No key ink.
    None,
}
/// Normalized position of an inside legend box, or an outside side.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LegendPosition {
    /// Right outside column.
    Right,
    /// Left outside column.
    Left,
    /// Top outside row.
    Top,
    /// Bottom outside row.
    Bottom,
    /// Fractional left/top of the available area.
    Inside {
        /// Horizontal fraction in `0..=1`.
        x: f64,
        /// Vertical fraction in `0..=1`.
        y: f64,
    },
}
/// Guide-only aesthetic replacements; mark mappings are untouched.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyOverrides {
    /// Key color.
    pub color: Option<Paint>,
    /// Key fill.
    pub fill: Option<Paint>,
    /// Key outline.
    pub stroke: Option<Paint>,
    /// Point size in millimeters.
    pub size: Option<f64>,
    /// Alpha in `0..=1`.
    pub alpha: Option<f64>,
    /// Line width in millimeters.
    pub width: Option<f64>,
    /// Reference shape code 0 through 25.
    pub shape: Option<u8>,
    /// Line pattern.
    pub line_type: Option<super::LineType>,
}
/// Per-guide presentation, separate from scale training.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LegendOptions {
    /// Registered drawing of the fully trained guide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered: Option<super::GuideDrawingSelection>,
    /// Parse title and labels as mathematical notation using explicitly supplied faces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub math: Option<crate::typography::MathFonts>,
    /// Explicit title; empty omits it.
    pub title: Option<String>,
    /// Smaller explicit orders are placed first; zero sorts as 99. Ties retain source order.
    pub order: u32,
    /// Reverse keys without changing scale outputs.
    pub reverse: Option<bool>,
    /// Guide box destination.
    pub position: Option<LegendPosition>,
    /// Key flow direction; outside top/bottom defaults to horizontal.
    pub direction: Option<crate::scene::GradientDirection>,
    /// Requested key row count.
    pub nrow: Option<usize>,
    /// Requested key column count.
    pub ncol: Option<usize>,
    /// Fill the grid across rows rather than down columns.
    pub by_row: bool,
    /// Guide-only key overrides.
    pub override_aes: KeyOverrides,
}
/// A layer's guide inclusion and key glyph policy.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayerLegend {
    /// Registered vector key; a None builtin key still suppresses it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_key: Option<super::KeyGlyphSelection>,
    /// None uses mapped-channel inclusion; false omits this layer from guides.
    pub show: Option<bool>,
    /// Channel-specific inclusion overrides the whole-layer policy.
    pub aesthetics: std::collections::BTreeMap<LegendAesthetic, bool>,
    /// Key topology override.
    pub key_glyph: KeyGlyph,
}
impl LayerLegend {
    pub(crate) fn includes(&self, a: LegendAesthetic) -> bool {
        self.aesthetics
            .get(&a)
            .copied()
            .or(self.show)
            .unwrap_or(true)
    }
}
impl LegendOptions {
    pub(crate) fn validate(&self) -> ChartResult<()> {
        let o = &self.override_aes;
        if self.nrow == Some(0)
            || self.ncol == Some(0)
            || self.nrow.is_some_and(|n| n > 1_000_000)
            || self.ncol.is_some_and(|n| n > 1_000_000)
            || self.title.as_ref().is_some_and(|s| s.len() > 4096)
            || o.size.is_some_and(|v| !v.is_finite() || v < 0.)
            || o.width.is_some_and(|v| !v.is_finite() || v < 0.)
            || o.alpha
                .is_some_and(|v| !v.is_finite() || !(0. ..=1.).contains(&v))
            || o.shape.is_some_and(|v| v > 25)
            || matches!(self.position,Some(LegendPosition::Inside{x,y}) if !x.is_finite()||!y.is_finite()||!(0. ..=1.).contains(&x)||!(0. ..=1.).contains(&y))
        {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Invalid legend title, key override or inside position.",
            ));
        }
        Ok(())
    }
}

/// One portable vector path in a custom guide, in destination units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomGuidePath {
    /// Shared path topology; text can use supplied glyph outlines.
    pub geometry: crate::path::PathGeometry,
    /// Optional interior paint.
    pub fill: Option<Paint>,
    /// Optional outline.
    pub stroke: Option<crate::scene::Stroke>,
}
/// A custom vector guide with explicit dimensions, independent of data mappings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomLegend {
    /// Stable decoration identity, distinct from other custom guides.
    pub id: crate::ScaleId,
    /// Local bounds and intrinsic dimensions in destination units.
    pub bounds: [f64; 4],
    /// Ordered portable vector content.
    pub paths: Vec<CustomGuidePath>,
    /// Shared title, order and placement policies.
    #[serde(default)]
    pub options: LegendOptions,
}
impl CustomLegend {
    pub(crate) fn validate(&self, limits: crate::Limits) -> ChartResult<()> {
        self.options.validate()?;
        crate::Rect::new(
            self.bounds[0],
            self.bounds[1],
            self.bounds[2],
            self.bounds[3],
        )?;
        crate::geometry::positive(self.bounds[2], "Custom guide width must be positive.")?;
        crate::geometry::positive(self.bounds[3], "Custom guide height must be positive.")?;
        crate::limits::require_within(self.paths.len() <= limits.max_items, "custom guide paths")?;
        crate::limits::require_within(
            self.paths
                .iter()
                .map(|p| p.geometry.commands().len())
                .fold(0usize, usize::saturating_add)
                <= limits.max_path_commands,
            "custom guide path commands",
        )?;
        for path in &self.paths {
            path.geometry.bounds(0.01, limits.max_path_commands)?;
            if let Some(stroke) = path.stroke {
                crate::geometry::positive(stroke.width, "Custom guide stroke must be positive.")?;
            }
        }
        Ok(())
    }
}
