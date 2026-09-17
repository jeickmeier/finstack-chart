//! GG-07 surface setup in calculation space, preserving authored row identities.
use super::{
    compiler::{EncodedRow, charge, include_geometry, row_style},
    *,
};
use crate::{ChartResult, Point, provenance::Target, scene::FillRule};
use std::{collections::BTreeMap, sync::Arc};

/// Grouped polygon interior policy; subgroup mappings identify separate contours.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolygonRecipe {
    /// Even-odd matches geom_polygon's default; nonzero observes contour orientation.
    pub rule: FillRule,
}
impl Default for PolygonRecipe {
    fn default() -> Self {
        Self {
            rule: FillRule::EvenOdd,
        }
    }
}
/// Tile dimensions in the post-scale calculation space.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileRecipe {
    /// Constant width; mapped Width takes precedence, otherwise use x resolution.
    pub width: Option<f64>,
    /// Constant height; mapped Height takes precedence, otherwise use y resolution.
    pub height: Option<f64>,
}
/// Regular row-grid image placement and portable sampling policy.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RasterRecipe {
    /// Fraction of cell width right of the source x coordinate.
    pub hjust: f64,
    /// Fraction of cell height above the source y coordinate.
    pub vjust: f64,
    /// Smooth image interpolation; false preserves exact cell boundaries.
    pub interpolate: bool,
}
impl Default for RasterRecipe {
    fn default() -> Self {
        Self {
            hjust: 0.5,
            vjust: 0.5,
            interpolate: false,
        }
    }
}
/// A retained source cell in calculation coordinates, without native image objects.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SurfaceCell {
    /// Opposite finite calculation-space corners.
    pub corners: [Point; 2],
    /// Resolved sRGB fill with row alpha applied.
    pub color: crate::scene::Color,
    /// Integer column and row before destination-axis reversal.
    pub grid: [usize; 2],
}
/// Shared geometry before the destination coordinate projection.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum PreparedSurface {
    /// Compound contours with one source anchor for each retained target.
    Polygon {
        /// Contours in first-seen subgroup order, vertices in source order.
        contours: Vec<Vec<Point>>,
        /// Original retained source locations in target order.
        anchors: Vec<Point>,
        /// Compound contour interior policy.
        rule: FillRule,
    },
    /// Source-order cells; missing grid locations become transparent pixels.
    Raster {
        /// One original-row cell per target, in source order.
        cells: Vec<SurfaceCell>,
        /// Bounded grid column count.
        width: usize,
        /// Bounded grid row count.
        height: usize,
        /// Smooth destination sampling when true, nearest cell selection otherwise.
        interpolate: bool,
    },
}
impl PreparedSurface {
    pub(crate) fn points(&self) -> Vec<Point> {
        match self {
            Self::Polygon { contours, .. } => contours.iter().flatten().copied().collect(),
            Self::Raster { cells, .. } => cells.iter().flat_map(|c| c.corners).collect(),
        }
    }
    pub(crate) fn transpose(&mut self) -> ChartResult<()> {
        let swap = |p: &mut Point| -> ChartResult<()> {
            *p = Point::new(p.y(), p.x())?;
            Ok(())
        };
        match self {
            Self::Polygon {
                contours, anchors, ..
            } => {
                for p in contours.iter_mut().flatten().chain(anchors) {
                    swap(p)?;
                }
            }
            Self::Raster {
                cells,
                width,
                height,
                ..
            } => {
                std::mem::swap(width, height);
                for cell in cells {
                    cell.grid.swap(0, 1);
                    for p in &mut cell.corners {
                        swap(p)?;
                    }
                }
            }
        }
        Ok(())
    }
}
pub(super) fn validate(recipe: &BuiltinRecipe) -> ChartResult<()> {
    let valid = match recipe {
        BuiltinRecipe::Tile(s) | BuiltinRecipe::Hexagon(s) => [s.width, s.height]
            .into_iter()
            .flatten()
            .all(|v| v.is_finite() && v >= 0.),
        BuiltinRecipe::Raster(s) => s.hjust.is_finite() && s.vjust.is_finite(),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::Validation,
            "Surface dimensions must be finite and nonnegative; raster justification must be finite.",
        ))
    }
}
fn missing_dimensions(row: &EncodedRow) -> bool {
    [RecipeAesthetic::Width, RecipeAesthetic::Height]
        .into_iter()
        .any(|channel| super::recipe_emit::number_or(row, channel, 0.).is_none())
}
pub(super) fn setup(
    layer: &Layer,
    rows: &mut [EncodedRow],
    _limits: CompileLimits,
) -> ChartResult<()> {
    let Some(recipe) = &layer.recipe else {
        return Ok(());
    };
    let (width, height, hjust, vjust) = match recipe {
        BuiltinRecipe::Tile(s) | BuiltinRecipe::Hexagon(s) => (s.width, s.height, 0.5, 0.5),
        BuiltinRecipe::Raster(s) => (None, None, 1. - s.hjust, 1. - s.vjust),
        _ => return Ok(()),
    };
    let dx = super::ggplot_position::resolution(rows.iter().filter_map(|r| r.x).collect());
    let dy = super::ggplot_position::resolution(rows.iter().filter_map(|r| r.y).collect());
    for row in rows {
        if missing_dimensions(row) {
            continue;
        }
        let w = super::recipe_emit::number(row, RecipeAesthetic::Width)
            .or(width)
            .unwrap_or(dx);
        let h = super::recipe_emit::number(row, RecipeAesthetic::Height)
            .or(height)
            .unwrap_or(dy);
        let h = if matches!(recipe, BuiltinRecipe::Hexagon(_)) {
            h * 2. / libm::sqrt(3.)
        } else {
            h
        };
        if !w.is_finite() || !h.is_finite() || w < 0. || h < 0. {
            row.x = None;
            row.y = None;
            continue;
        }
        if let Some(x) = row.x {
            row.x = Some(x - w * hjust);
            row.x2 = Some(x + w * (1. - hjust));
        }
        if let Some(y) = row.y {
            row.y = Some(y - h * vjust);
            row.y2 = Some(y + h * (1. - vjust));
        }
    }
    Ok(())
}
pub(super) fn surface_style(layer: &Layer, row: &EncodedRow) -> ChartResult<Style> {
    let mut style = row_style(layer, row)?;
    if style.fill.is_none() {
        let color = crate::color::Paint::from(crate::scene::Color {
            red: 51,
            green: 51,
            blue: 51,
            alpha: 255,
        });
        style.fill = Some(super::numeric_aesthetics::apply_alpha(
            super::numeric_aesthetics::apply_opacity(color, row.opacity),
            layer.style.alpha.or(row.alpha),
        ));
    }
    if style.stroke.is_none() && row.color.is_some() {
        style.stroke = Some(style.color);
    }
    if style.units.is_none() {
        style.stroke_width =
            super::reference_linewidth(style.stroke_width, crate::services::Units::Points);
        style.units = Some(AestheticUnits::Points);
    }
    Ok(style)
}
fn surface_mark(
    layer: &Layer,
    row: &EncodedRow,
    geometry: PreparedGeometry,
    targets: Vec<Target>,
) -> ChartResult<PreparedMark> {
    Ok(PreparedMark {
        aesthetics: row.values.clone(),
        geometry,
        targets,
        group: row.group.clone().unwrap_or(GroupValue::All),
        style: surface_style(layer, row)?,
    })
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<bool> {
    let Some(recipe) = &layer.recipe else {
        return Ok(false);
    };
    match recipe {
        BuiltinRecipe::Hexagon(_) => {
            for row in rows {
                if missing_dimensions(row) {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                }
                if let (Some(x), Some(y), Some(x2), Some(y2)) = (row.x, row.y, row.x2, row.y2) {
                    let cx = x.midpoint(x2);
                    let cy = y.midpoint(y2);
                    let q = (y2 - y) / 4.;
                    let contour = vec![
                        Point::new(cx, y)?,
                        Point::new(x2, cy - q)?,
                        Point::new(x2, cy + q)?,
                        Point::new(cx, y2)?,
                        Point::new(x, cy + q)?,
                        Point::new(x, cy - q)?,
                    ];
                    charge(vertices, 6, "hexagon vertex")?;
                    let geometry = PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(
                        PreparedSurface::Polygon {
                            contours: vec![contour],
                            anchors: vec![Point::new(cx, cy)?],
                            rule: FillRule::EvenOdd,
                        },
                    )));
                    include_geometry(&mut prepared.domains, &geometry);
                    Arc::make_mut(&mut prepared.marks).push(surface_mark(
                        layer,
                        row,
                        geometry,
                        vec![row.target.clone()],
                    )?);
                }
            }
        }
        BuiltinRecipe::Tile(_) => {
            for row in rows {
                if missing_dimensions(row) {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                }
                if let (Some(x), Some(y), Some(x2), Some(y2)) = (row.x, row.y, row.x2, row.y2) {
                    charge(vertices, 4, "surface vertex")?;
                    let geometry = PreparedGeometry::Rectangle {
                        from: Point::new(x, y)?,
                        to: Point::new(x2, y2)?,
                    };
                    include_geometry(&mut prepared.domains, &geometry);
                    Arc::make_mut(&mut prepared.marks).push(surface_mark(
                        layer,
                        row,
                        geometry,
                        vec![row.target.clone()],
                    )?);
                }
            }
        }
        BuiltinRecipe::Polygon(s) => {
            let mut groups: BTreeMap<GroupValue, Vec<&EncodedRow>> = BTreeMap::new();
            for row in rows {
                groups
                    .entry(row.group.clone().unwrap_or(GroupValue::All))
                    .or_default()
                    .push(row);
            }
            for group in groups.values() {
                let mut contours: Vec<Vec<Point>> = vec![];
                let mut keys: BTreeMap<String, usize> = BTreeMap::new();
                let mut anchors = vec![];
                let mut targets = vec![];
                for row in group {
                    if let (Some(x), Some(y)) = (row.x, row.y) {
                        let key = match row.recipe_values.get(&RecipeAesthetic::Subgroup) {
                            Some(crate::interpolate::Value::Text(s)) => s.clone(),
                            Some(v) => serde_json::to_string(v).unwrap_or_default(),
                            None => String::new(),
                        };
                        let next = contours.len();
                        let i = *keys.entry(key).or_insert_with(|| {
                            contours.push(vec![]);
                            next
                        });
                        let p = Point::new(x, y)?;
                        contours[i].push(p);
                        anchors.push(p);
                        targets.push(row.target.clone());
                    }
                }
                if anchors.is_empty() {
                    continue;
                }
                charge(
                    vertices,
                    anchors
                        .len()
                        .saturating_mul(2)
                        .saturating_add(contours.len()),
                    "surface vertex",
                )?;
                let geometry = PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(
                    PreparedSurface::Polygon {
                        contours,
                        anchors,
                        rule: s.rule,
                    },
                )));
                include_geometry(&mut prepared.domains, &geometry);
                Arc::make_mut(&mut prepared.marks)
                    .push(surface_mark(layer, group[0], geometry, targets)?);
            }
        }
        BuiltinRecipe::Raster(s) => {
            let mut cells = vec![];
            let mut targets = vec![];
            let mut first = None;
            for row in rows {
                if missing_dimensions(row) {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                }
                if let (Some(x), Some(y), Some(x2), Some(y2)) = (row.x, row.y, row.x2, row.y2) {
                    let style = surface_style(layer, row)?;
                    let mut color = style.fill.unwrap_or(style.color);
                    if let Some(alpha) = style.alpha {
                        color.alpha = (alpha.clamp(0., 1.) * 255.).round() as u8;
                    }
                    cells.push(SurfaceCell {
                        corners: [Point::new(x, y)?, Point::new(x2, y2)?],
                        color,
                        grid: [0, 0],
                    });
                    targets.push(row.target.clone());
                    first.get_or_insert(row);
                }
            }
            if let Some(row) = first {
                let x = cells
                    .iter()
                    .map(|c| c.corners[0].x())
                    .reduce(f64::min)
                    .unwrap();
                let y = cells
                    .iter()
                    .map(|c| c.corners[0].y())
                    .reduce(f64::min)
                    .unwrap();
                let dx = super::ggplot_position::resolution(
                    cells.iter().map(|c| c.corners[0].x()).collect(),
                );
                let dy = super::ggplot_position::resolution(
                    cells.iter().map(|c| c.corners[0].y()).collect(),
                );
                for c in &mut cells {
                    c.grid = [
                        ((c.corners[0].x() - x) / dx) as usize,
                        ((c.corners[0].y() - y) / dy) as usize,
                    ];
                }
                let width = cells
                    .iter()
                    .map(|c| c.grid[0])
                    .max()
                    .unwrap()
                    .checked_add(1)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::ResourceLimit,
                            "Raster width exceeds budget.",
                        )
                    })?;
                let height = cells
                    .iter()
                    .map(|c| c.grid[1])
                    .max()
                    .unwrap()
                    .checked_add(1)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::ResourceLimit,
                            "Raster height exceeds budget.",
                        )
                    })?;
                let count = width.checked_mul(height).ok_or_else(|| {
                    error(DiagnosticCode::ResourceLimit, "Raster grid exceeds budget.")
                })?;
                charge(
                    vertices,
                    count.saturating_add(cells.len().saturating_mul(4)),
                    "surface vertex",
                )?;
                let geometry = PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(
                    PreparedSurface::Raster {
                        cells,
                        width,
                        height,
                        interpolate: s.interpolate,
                    },
                )));
                include_geometry(&mut prepared.domains, &geometry);
                Arc::make_mut(&mut prepared.marks)
                    .push(surface_mark(layer, row, geometry, targets)?);
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
