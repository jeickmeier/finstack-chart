use crate::grammar::{
    compiler::{EncodedRow, charge, include_geometry, row_style},
    *,
};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, provenance::Target};
use std::sync::Arc;
pub(super) fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use finite distribution controls and a compatible source or generated schema.",
    )
}
pub(super) fn number(row: &EncodedRow, ch: RecipeAesthetic) -> Option<f64> {
    super::super::recipe_emit::number(row, ch)
}
pub(super) fn number_or(row: &EncodedRow, ch: RecipeAesthetic, default: f64) -> Option<f64> {
    super::super::recipe_emit::number_or(row, ch, default)
}
pub(super) fn width(row: &EncodedRow) -> Option<(f64, f64)> {
    Some((row.x?, row.x2?))
}
pub(super) fn point(x: f64, y: f64) -> ChartResult<Point> {
    Point::new(x, y)
}
pub(super) fn line(x1: f64, y1: f64, x2: f64, y2: f64) -> ChartResult<PreparedGeometry> {
    Ok(PreparedGeometry::Rule {
        from: point(x1, y1)?,
        to: point(x2, y2)?,
    })
}
pub(super) fn style(
    layer: &Layer,
    row: &EncodedRow,
    controls: &IntervalStroke,
    opaque: bool,
) -> ChartResult<Style> {
    let mut style = if opaque {
        let mut layer = layer.clone();
        let mut row = row.clone();
        layer.style.alpha = None;
        row.alpha = None;
        row_style(&layer, &row)?
    } else {
        row_style(layer, row)?
    };
    if let Some(color) = controls.color {
        style.color = color.resolve();
        style.stroke = Some(style.color);
    }
    if let Some(width) = controls.linewidth {
        style.stroke_width = width;
    }
    if let Some(line) = controls.line_type {
        style.line_type = Some(line);
    }
    if opaque {
        style.alpha = None;
    }
    if style.units.is_none() {
        style.stroke_width =
            super::super::reference_linewidth(style.stroke_width, crate::services::Units::Points);
        style.units = Some(AestheticUnits::Points);
    }
    style.stroke = Some(style.stroke.unwrap_or(style.color));
    Ok(style)
}
pub(super) fn push(
    prepared: &mut PreparedLayer,
    row: &EncodedRow,
    geometry: PreparedGeometry,
    style: Style,
    targets: Vec<Target>,
    vertices: &mut usize,
) -> ChartResult<()> {
    let count = match &geometry {
        PreparedGeometry::Recipe(r) => r.points().len(),
        PreparedGeometry::Polygon(v) | PreparedGeometry::LineRun(v) => v.len(),
        PreparedGeometry::Point(_) => 1,
        _ => 4,
    };
    charge(vertices, count.max(1), "distribution geometry vertex")?;
    include_geometry(&mut prepared.domains, &geometry);
    Arc::make_mut(&mut prepared.marks).push(PreparedMark {
        geometry,
        style,
        group: row.group.clone().unwrap_or(GroupValue::All),
        targets,
        aesthetics: row.values.clone(),
    });
    Ok(())
}
pub(super) fn stroke_valid(s: &IntervalStroke) -> bool {
    s.linewidth.is_none_or(|v| v.is_finite() && v >= 0.)
}
