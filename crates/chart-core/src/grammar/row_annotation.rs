//! Bounded portable decoration over retained row anchors, shared by every destination.
use super::{AestheticUnits, Geom, Layer, error};
use crate::{ChartResult, DiagnosticCode};
/// Caller-owned RGBA pixels in top-to-bottom row-major order.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RasterAnnotation {
    /// Positive column count.
    pub width: usize,
    /// Positive row count.
    pub height: usize,
    /// Exactly width times height colors; alpha is retained unpremultiplied.
    pub pixels: Vec<crate::scene::Color>,
}
/// Portable payload positioned independently at every source/statistical row anchor.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AnnotationContent {
    /// Checked vector geometry in a local coordinate system.
    Vector {
        /// Shared path owner; no backend-specific commands.
        geometry: crate::path::PathGeometry,
        /// Optional filled interior.
        fill: Option<crate::scene::Color>,
        /// Optional outline in local size units.
        stroke: Option<crate::scene::Stroke>,
    },
    /// Nearest-neighbor pixels spanning local bounds without gaps or overlaps.
    Raster {
        /// Source pixels, bounded by compilation and destination item budgets.
        raster: RasterAnnotation,
        /// Interpolate pixels with the destination image sampler; false preserves cells.
        #[serde(default)]
        interpolate: bool,
        /// Local rectangle [x, y, width, height], allowing offset from anchor.
        bounds: [f64; 4],
    },
}
/// Portable custom/raster annotations; a single-row dataset authors a fixed annotation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowAnnotation {
    /// Explicit local units, independent of source-coordinate anchor projection.
    pub units: AestheticUnits,
    /// Destination-local payload.
    pub content: AnnotationContent,
}
impl RowAnnotation {
    pub(crate) fn validate(&self, max_items: usize) -> ChartResult<()> {
        match &self.content {
            AnnotationContent::Vector {
                geometry, stroke, ..
            } => {
                if geometry.commands().len() > max_items
                    || stroke.is_some_and(|s| !s.width.is_finite() || s.width <= 0.)
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Annotation vector exceeds its budget or has an invalid stroke.",
                    ));
                }
            }
            AnnotationContent::Raster { raster, bounds, .. } => {
                let count = raster.width.checked_mul(raster.height);
                if raster.width == 0 || raster.height == 0 || count != Some(raster.pixels.len()) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Raster dimensions must match its row-major pixel payload.",
                    ));
                }
                if raster.pixels.len() > max_items {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Raster annotation exceeds the pixel budget.",
                    ));
                }
                if bounds.iter().any(|v| !v.is_finite()) || bounds[2] <= 0. || bounds[3] <= 0. {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Raster annotation bounds must be finite and positive.",
                    ));
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn validate_layer(layer: &Layer, maximum: usize) -> ChartResult<()> {
    if let Some(a) = &layer.annotation {
        a.validate(maximum)?;
        if layer.geom != Geom::Point || layer.text.is_some() || layer.geometry_extension.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Portable annotations require exclusive point anchors.",
            ));
        }
    }
    Ok(())
}
