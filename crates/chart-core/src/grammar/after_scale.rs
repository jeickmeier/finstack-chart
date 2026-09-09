//! Nonpositional expression evaluation after scale mapping and semantic positions.
use super::*;
use crate::{ChartResult, theme::GeometryTheme};
/// Aesthetic outputs currently accepted by the shared post-scale expression stage.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum AfterScaleAesthetic {
    /// Point radius or rule stroke size, before destination-unit conversion.
    Size,
    /// Exact mapped or constant sRGB color.
    Color,
}
/// Theme-derived input available without a data-column dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ThemeRead {
    /// Foreground geometry color.
    Ink,
    /// Background geometry color.
    Paper,
    /// Accent geometry color.
    Accent,
    /// Geometry point-size default.
    PointSize,
    /// Geometry line-width default.
    LineWidth,
}
/// Snapshot of resolved aesthetics and theme tokens, never destination pixel positions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AfterScaleRead {
    /// Read a resolved aesthetic before any post-scale modifier is applied.
    Aesthetic(AfterScaleAesthetic),
    /// Read a geometry-theme token.
    Theme(ThemeRead),
}
fn kind(read: &AfterScaleRead) -> ChartResult<ExpressionType> {
    Ok(match read {
        AfterScaleRead::Aesthetic(AfterScaleAesthetic::Size)
        | AfterScaleRead::Theme(ThemeRead::PointSize | ThemeRead::LineWidth) => {
            ExpressionType::Number
        }
        _ => ExpressionType::Color,
    })
}
pub(super) fn validate(layer: &Layer) -> ChartResult<()> {
    for (aesthetic, expr) in &layer.after_scale {
        let expected = kind(&AfterScaleRead::Aesthetic(*aesthetic))?;
        if expr.validate(ExpressionLimits::default(), kind)? != expected {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Post-scale expression type does not match its output aesthetic.",
            ));
        }
    }
    if layer.after_scale.contains_key(&AfterScaleAesthetic::Size)
        && !matches!(layer.geom, Geom::Point | Geom::Rule)
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Post-scale size currently requires a point or rule geometry.",
        ));
    }
    Ok(())
}
pub(super) fn apply(
    layer: &Layer,
    theme: Option<&GeometryTheme<crate::color::Paint>>,
    mapped_size: bool,
    rows: &mut [super::compiler::EncodedRow],
) -> ChartResult<()> {
    if layer.after_scale.is_empty() {
        return Ok(());
    }
    validate(layer)?;
    let fallback = GeometryTheme::<crate::color::Paint>::default();
    let theme = theme.unwrap_or(&fallback);
    theme.validate()?;
    let constant_color = layer.style.color;
    let input = |read: &AfterScaleRead, i: usize| {
        use AfterScaleRead as R;
        use ExpressionValue as V;
        match read {
            R::Aesthetic(AfterScaleAesthetic::Color) => {
                V::Color(rows[i].color.unwrap_or(constant_color))
            }
            R::Aesthetic(AfterScaleAesthetic::Size) => {
                if mapped_size {
                    rows[i]
                        .size
                        .map_or(V::Missing(ExpressionType::Number), V::Number)
                } else {
                    V::Number(if matches!(layer.geom, Geom::Point) {
                        layer.style.radius
                    } else {
                        layer.style.stroke_width
                    })
                }
            }
            R::Theme(ThemeRead::Ink) => V::Color(theme.ink),
            R::Theme(ThemeRead::Paper) => V::Color(theme.paper),
            R::Theme(ThemeRead::Accent) => V::Color(theme.accent),
            R::Theme(ThemeRead::PointSize) => V::Number(theme.point_size),
            R::Theme(ThemeRead::LineWidth) => V::Number(theme.line_width),
        }
    };
    // Evaluate all outputs against the same input snapshot, then apply them simultaneously.
    let values = layer
        .after_scale
        .iter()
        .map(|(a, e)| {
            e.evaluate(rows.len(), ExpressionLimits::default(), kind, input)
                .map(|v| (*a, v))
        })
        .collect::<ChartResult<Vec<_>>>()?;
    for (aesthetic, values) in values {
        for (row, value) in rows.iter_mut().zip(values) {
            match aesthetic {
                AfterScaleAesthetic::Size => row.size = value.number(),
                AfterScaleAesthetic::Color => {
                    row.color = Some(match value {
                        ExpressionValue::Color(c) => c,
                        _ => crate::scene::Color {
                            red: 0,
                            green: 0,
                            blue: 0,
                            alpha: 0,
                        }
                        .into(),
                    })
                }
            }
        }
    }
    Ok(())
}
