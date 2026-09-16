//! Project compound surfaces once, retaining source anchors and cell hit coverage.
use crate::{
    ChartResult, Diagnostic, DiagnosticCode, Point, Rect,
    grammar::{AestheticUnits, Layer, LineType, PreparedMark, PreparedSurface, RasterAnnotation},
    layout::LayoutRequest,
    path::Path,
    scene::{Primitive, Stroke},
};
fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ResourceLimit,
        message,
        "Reduce raster resolution or increase the explicit scene budget.",
    )
}
pub(super) fn project(
    surface: &PreparedSurface,
    mark: &PreparedMark,
    _layer: &Layer,
    request: &LayoutRequest,
    _plot: Rect,
    map: &dyn Fn(Point) -> ChartResult<Option<Point>>,
) -> ChartResult<Vec<Primitive>> {
    match surface {
        PreparedSurface::Polygon {
            contours,
            anchors,
            rule,
        } => {
            let mut path = Path::new();
            for contour in contours {
                let mut started = false;
                for p in contour {
                    let Some(p) = map(*p)? else { return Ok(vec![]) };
                    if started {
                        path.line_to(p.x(), p.y())?;
                    } else {
                        path.move_to(p.x(), p.y())?;
                        started = true;
                    }
                }
                if started {
                    path.close_path()?;
                }
            }
            let Some(anchors) = anchors
                .iter()
                .map(|p| map(*p))
                .collect::<ChartResult<Option<Vec<_>>>>()?
            else {
                return Ok(vec![]);
            };
            let style = mark.style;
            let alpha = |mut c: crate::scene::Color| {
                if let Some(a) = style.alpha {
                    c.alpha = (a.clamp(0., 1.) * 255.).round() as u8;
                }
                c
            };
            let width = style.stroke_width
                * style
                    .units
                    .unwrap_or(AestheticUnits::Destination)
                    .factor(request.units);
            let line = style.line_type.unwrap_or(LineType::Solid);
            let stroke = style
                .stroke
                .map(alpha)
                .filter(|c| c.alpha > 0 && line != LineType::Blank)
                .map(|color| Stroke { color, width });
            Ok(vec![Primitive::ShapePath {
                geometry: path.geometry(),
                fill: Some(alpha(style.fill.unwrap_or(style.color))),
                stroke,
                dashes: if stroke.is_some() {
                    line.pattern(width)?
                } else {
                    vec![]
                },
                anchors,
                fill_rule: *rule,
            }])
        }
        PreparedSurface::Raster {
            cells,
            width,
            height,
            interpolate,
        } => {
            let (width, height) = (*width, *height);
            if cells.is_empty() {
                return Ok(vec![]);
            }
            let mut rects = Vec::with_capacity(cells.len());
            for cell in cells {
                let (Some(a), Some(b)) = (map(cell.corners[0])?, map(cell.corners[1])?) else {
                    return Ok(vec![]);
                };
                if a.x() == b.x() || a.y() == b.y() {
                    return Ok(vec![]);
                }
                rects.push(Rect::new(
                    a.x().min(b.x()),
                    a.y().min(b.y()),
                    (b.x() - a.x()).abs(),
                    (b.y() - a.y()).abs(),
                )?);
            }
            let x = rects
                .iter()
                .map(|r| r.origin().x())
                .reduce(f64::min)
                .unwrap();
            let y = rects
                .iter()
                .map(|r| r.origin().y())
                .reduce(f64::min)
                .unwrap();
            let maxx = rects.iter().map(|r| r.max_x()).reduce(f64::max).unwrap();
            let maxy = rects.iter().map(|r| r.max_y()).reduce(f64::max).unwrap();
            let count = width
                .checked_mul(height)
                .filter(|n| {
                    n.saturating_add(cells.len().saturating_mul(4))
                        <= request.limits.max_path_commands
                })
                .ok_or_else(|| {
                    error("Raster grid pixels and source cells exceed the scene budget.")
                })?;
            let mut pixels = vec![
                crate::scene::Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0
                };
                count
            ];
            let a = map(cells[0].corners[0])?.unwrap();
            let b = map(cells[0].corners[1])?.unwrap();
            let dx = (maxx - x) / width as f64;
            let dy = (maxy - y) / height as f64;
            for (cell, rect) in cells.iter().zip(&mut rects) {
                let col = if a.x() < b.x() {
                    cell.grid[0]
                } else {
                    width - 1 - cell.grid[0]
                };
                let row = if a.y() < b.y() {
                    cell.grid[1]
                } else {
                    height - 1 - cell.grid[1]
                };
                pixels[row * width + col] = cell.color;
                *rect = Rect::new(x + col as f64 * dx, y + row as f64 * dy, dx, dy)?;
            }
            Ok(vec![Primitive::RasterImage {
                bounds: Rect::new(x, y, maxx - x, maxy - y)?,
                raster: RasterAnnotation {
                    width,
                    height,
                    pixels,
                },
                interpolate: *interpolate,
                cells: rects,
            }])
        }
    }
}
