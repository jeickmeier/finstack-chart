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
    /// Independent interior paint.
    Fill,
    /// Independent outline paint.
    Stroke,
    /// Replacement paint alpha.
    Alpha,
    /// Stroke width independent of point size.
    LineWidth,
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
        AfterScaleRead::Aesthetic(
            AfterScaleAesthetic::Size | AfterScaleAesthetic::Alpha | AfterScaleAesthetic::LineWidth,
        )
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
    profile: Profile,
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
        if let R::Aesthetic(a) = read
            && rows[i].is_missing(*a)
        {
            return V::Missing(kind(read).expect("validated aesthetic kind"));
        }
        match read {
            R::Aesthetic(AfterScaleAesthetic::Color) => {
                V::Color(rows[i].color.unwrap_or(constant_color))
            }
            R::Aesthetic(AfterScaleAesthetic::Fill) => V::Color(
                layer
                    .style
                    .fill
                    .or(rows[i].fill)
                    .unwrap_or(rows[i].color.unwrap_or(constant_color)),
            ),
            R::Aesthetic(AfterScaleAesthetic::Stroke) => V::Color(
                layer
                    .style
                    .stroke
                    .or(rows[i].stroke)
                    .unwrap_or(rows[i].color.unwrap_or(constant_color)),
            ),
            R::Aesthetic(AfterScaleAesthetic::Alpha) => {
                V::Number(layer.style.alpha.or(rows[i].alpha).unwrap_or(1.))
            }
            R::Aesthetic(AfterScaleAesthetic::LineWidth) => {
                V::Number(rows[i].stroke_width.unwrap_or(layer.style.stroke_width))
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
            let result = if profile == Profile::Ggplot2_4_0_3 {
                e.evaluate_reference(rows.len(), ExpressionLimits::default(), kind, input)
            } else {
                e.evaluate(rows.len(), ExpressionLimits::default(), kind, input)
            };
            result.map(|v| (*a, v))
        })
        .collect::<ChartResult<Vec<_>>>()?;
    for (aesthetic, values) in values {
        for (row, value) in rows.iter_mut().zip(values) {
            if profile == Profile::Ggplot2_4_0_3 {
                row.set_missing(aesthetic, matches!(value, ExpressionValue::Missing(_)));
            }
            match aesthetic {
                AfterScaleAesthetic::Size => {
                    row.size = value.number();
                    if profile == Profile::Ggplot2_4_0_3 {
                        row.set_missing(aesthetic, row.size.is_none());
                    }
                }
                AfterScaleAesthetic::Alpha => {
                    let alpha = value.number_with_infinite(profile == Profile::Ggplot2_4_0_3);
                    if profile != Profile::Ggplot2_4_0_3
                        && alpha.is_some_and(|v| !v.is_finite() || !(0. ..=1.).contains(&v))
                    {
                        return Err(error(
                            DiagnosticCode::NumericalDomain,
                            "Post-scale alpha must lie in the closed unit interval.",
                        ));
                    }
                    row.alpha = alpha;
                }
                AfterScaleAesthetic::LineWidth => {
                    row.stroke_width = value.number();
                    if profile == Profile::Ggplot2_4_0_3 {
                        row.set_missing(aesthetic, row.stroke_width.is_none());
                    }
                    if profile != Profile::Ggplot2_4_0_3 && row.stroke_width.is_none_or(|w| w <= 0.)
                    {
                        row.x = None;
                        row.y = None;
                    }
                }
                AfterScaleAesthetic::Color
                | AfterScaleAesthetic::Fill
                | AfterScaleAesthetic::Stroke => {
                    let paint = Some(match value {
                        ExpressionValue::Color(c) => c,
                        _ => crate::scene::Color {
                            red: 0,
                            green: 0,
                            blue: 0,
                            alpha: 0,
                        }
                        .into(),
                    });
                    match aesthetic {
                        AfterScaleAesthetic::Fill => row.fill = paint,
                        AfterScaleAesthetic::Stroke => row.stroke = paint,
                        _ => row.color = paint,
                    }
                }
            }
        }
    }
    Ok(())
}
