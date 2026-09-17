//! Row-driven text consumes the shared typed aesthetic and destination text services.
use super::{Geom, Layer, error};
use crate::{ChartResult, DiagnosticCode};

/// Text/label controls applied independently to every retained source or generated row.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TextGeom {
    /// Physical size units; positional nudge remains in data coordinates.
    pub units: TextSizeUnit,
    /// Font size, overridden by the mapped TextSize aesthetic.
    pub size: f64,
    /// Clockwise angle, overridden by TextAngle.
    pub angle: f64,
    /// Horizontal justification, overridden by HJust.
    pub hjust: f64,
    /// Vertical justification (zero bottom, one top), overridden by VJust.
    pub vjust: f64,
    /// Line spacing, overridden by LineHeight.
    pub line_height: f64,
    /// Suppress later overlapping labels in retained row order.
    pub check_overlap: bool,
    /// Optional background fill; absent emits text alone.
    pub fill: Option<crate::scene::Color>,
    /// Box padding in multiples of the resolved font size, horizontal then vertical.
    pub padding: [f64; 2],
    /// Box border width in the declared size units.
    pub border_width: f64,
    /// Exact supplied face for all rows; absent uses the destination font.
    pub font: Option<crate::services::ResourceDescriptor>,
    /// Explicit family/face resources for mapped font selectors, without system lookup.
    pub fonts: Vec<TextFont>,
    /// Parse row labels as plotmath with these explicit faces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub math: Option<crate::typography::MathFonts>,
}
impl Default for TextGeom {
    fn default() -> Self {
        Self {
            units: TextSizeUnit::Millimeters,
            size: 3.88,
            angle: 0.,
            hjust: 0.5,
            vjust: 0.5,
            line_height: 1.2,
            check_overlap: false,
            fill: None,
            padding: [0.25; 2],
            border_width: 0.25,
            font: None,
            fonts: Vec::new(),
            math: None,
        }
    }
}
impl TextGeom {
    pub(crate) fn validate(&self) -> ChartResult<()> {
        if let Some(fonts) = &self.math {
            fonts.validate(crate::Limits::default())?;
        }
        let finite = [
            self.size,
            self.angle,
            self.hjust,
            self.vjust,
            self.line_height,
            self.padding[0],
            self.padding[1],
            self.border_width,
        ];
        if finite.iter().any(|x| !x.is_finite())
            || self.size <= 0.
            || self.angle.abs() > 360.
            || !(1. ..=8.).contains(&self.line_height)
            || self.padding.iter().any(|x| *x < 0.)
            || self.border_width < 0.
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid row text dimensions, spacing or rotation.",
            ));
        }
        if self
            .font
            .is_some_and(|font| font.kind != crate::services::ResourceKind::Font)
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Row text requires a font resource.",
            ));
        }
        if self.fonts.len() > 256 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Text font catalog exceeds 256 faces.",
            ));
        }
        let mut identities = std::collections::BTreeSet::new();
        for entry in &self.fonts {
            if entry.family.len() > 4096
                || !matches!(
                    entry.face.as_str(),
                    "plain" | "bold" | "italic" | "bold.italic"
                )
                || entry.font.kind != crate::services::ResourceKind::Font
                || !(100..=900).contains(&entry.weight)
                || !identities.insert((&entry.family, &entry.face))
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Invalid or duplicate explicit text face.",
                ));
            }
        }
        Ok(())
    }
}
pub(crate) fn validate_layer(layer: &Layer) -> ChartResult<()> {
    if let Some(text) = &layer.text {
        text.validate()?;
        if layer.geom != Geom::Point || layer.geometry_extension.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Row text requires point anchors without a geometry extension.",
            ));
        }
    }
    Ok(())
}

/// ggplot text-size units, including its TeX-point millimeter conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextSizeUnit {
    /// Already expressed in destination units.
    Destination,
    /// Reference millimeters: 72.27 / 25.4 device points.
    Millimeters,
    /// Device points, 72 per publication inch.
    Points,
    /// Ten reference millimeters.
    Centimeters,
    /// Reference inch, 72.27 device points.
    Inches,
    /// Twelve device points.
    Picas,
}
impl TextSizeUnit {
    /// Convert using the pinned reference unit policy and destination density.
    pub fn factor(self, units: crate::services::Units) -> f64 {
        let points = match self {
            Self::Destination => return 1.,
            Self::Millimeters => 72.27 / 25.4,
            Self::Points => 1.,
            Self::Centimeters => 72.27 / 2.54,
            Self::Inches => 72.27,
            Self::Picas => 12.,
        };
        points
            * if units == crate::services::Units::LogicalPixels {
                96. / 72.
            } else {
                1.
            }
    }
}

/// Exact supplied font face selected by mapped family and face values.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextFont {
    /// Authored family identity, independent of backend system catalogs.
    pub family: String,
    /// plain, bold, italic or bold.italic.
    pub face: String,
    /// Immutable caller-owned font bytes descriptor.
    pub font: crate::services::ResourceDescriptor,
    /// Actual OS/2 weight of that font resource.
    pub weight: u16,
}
