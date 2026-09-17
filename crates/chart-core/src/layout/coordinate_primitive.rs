//! Coordinate transformation of final styled marks; symbol dimensions remain destination units.
use super::{coordinate_map::CoordinateMap, coordinate_path};
use crate::path::{Affine, Path, PathGeometry};
use crate::scene::{FillRule, PathCommand, Primitive};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point};
fn rectangle(bounds: crate::Rect) -> ChartResult<PathGeometry> {
    let mut p = Path::new();
    p.rect(
        bounds.origin().x(),
        bounds.origin().y(),
        bounds.width(),
        bounds.height(),
    )?;
    Ok(p.geometry())
}
fn shape(
    geometry: PathGeometry,
    fill: Option<crate::scene::Color>,
    stroke: Option<crate::scene::Stroke>,
    dashes: Vec<f64>,
    anchors: Vec<Point>,
) -> Primitive {
    Primitive::ShapePath {
        geometry,
        fill,
        stroke,
        dashes,
        anchors,
        fill_rule: FillRule::NonZero,
    }
}
pub(super) fn project(
    primitive: Primitive,
    map: &CoordinateMap,
    fixed: Option<Point>,
    targets: usize,
    error: f64,
    remaining: &mut usize,
    request: &super::LayoutRequest,
) -> ChartResult<Vec<Primitive>> {
    let translate = fixed
        .map(|p| {
            map.project(p)
                .map(|mapped| mapped.map(|q| [q.x() - p.x(), q.y() - p.y()]))
        })
        .transpose()?
        .flatten();
    if fixed.is_some() && translate.is_none() {
        return Ok(vec![]);
    }
    let point = |p: Point| -> ChartResult<Option<Point>> {
        if let Some([x, y]) = translate {
            Point::new(p.x() + x, p.y() + y).map(Some)
        } else {
            map.project(p)
        }
    };
    let path = |geometry: &PathGeometry, remaining: &mut usize| {
        if let Some([x, y]) = translate {
            geometry.transformed(Affine::new([1., 0., 0., 1., x, y])?, error, *remaining)
        } else {
            coordinate_path::project(geometry, map, error, remaining)
        }
    };
    let anchors = |values: Vec<Point>| -> ChartResult<Vec<Point>> {
        values
            .into_iter()
            .map(|p| {
                point(p)?.ok_or_else(|| {
                    Diagnostic::error(
                        DiagnosticCode::NumericalDomain,
                        "A coordinate source anchor has no finite transform.",
                        "Filter undefined coordinate values or choose a valid transform domain.",
                    )
                })
            })
            .collect()
    };
    let result = match primitive {
        Primitive::Point {
            center,
            radius,
            fill,
        } => {
            let Some(center) = point(center)? else {
                return Ok(vec![]);
            };
            Primitive::Point {
                center,
                radius,
                fill,
            }
        }
        Primitive::Symbol {
            center,
            radius,
            kind,
            fill,
        } => {
            let Some(center) = point(center)? else {
                return Ok(vec![]);
            };
            Primitive::Symbol {
                center,
                radius,
                kind,
                fill,
            }
        }
        Primitive::ShapePath {
            geometry,
            fill,
            stroke,
            dashes,
            anchors: source,
            fill_rule,
        } => Primitive::ShapePath {
            geometry: path(&geometry, remaining)?,
            fill,
            stroke,
            dashes,
            anchors: anchors(source)?,
            fill_rule,
        },
        Primitive::VectorPath {
            geometry,
            fill,
            stroke,
            dashes,
        } => Primitive::VectorPath {
            geometry: path(&geometry, remaining)?,
            fill,
            stroke,
            dashes,
        },
        Primitive::Rule { from, to, stroke } => {
            let commands = vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)];
            shape(
                path(&PathGeometry::from_beziers(&commands)?, remaining)?,
                None,
                Some(stroke),
                vec![],
                anchors(super::project::command_anchors(&commands, targets))?,
            )
        }
        Primitive::Path { commands, stroke } => shape(
            path(&PathGeometry::from_beziers(&commands)?, remaining)?,
            None,
            Some(stroke),
            vec![],
            anchors(super::project::command_anchors(&commands, targets))?,
        ),
        Primitive::DashedPath {
            commands,
            stroke,
            dashes,
        } => shape(
            path(&PathGeometry::from_beziers(&commands)?, remaining)?,
            None,
            Some(stroke),
            dashes,
            anchors(super::project::command_anchors(&commands, targets))?,
        ),
        Primitive::FilledPath { commands, fill } => shape(
            path(&PathGeometry::from_beziers(&commands)?, remaining)?,
            Some(fill),
            None,
            vec![],
            anchors(super::project::command_anchors(&commands, targets))?,
        ),
        Primitive::Rectangle { bounds, fill } => shape(
            path(&rectangle(bounds)?, remaining)?,
            Some(fill),
            None,
            vec![],
            anchors(vec![
                Point::new(
                    bounds.origin().x() + bounds.width() / 2.,
                    bounds.origin().y() + bounds.height() / 2.
                )?;
                targets
            ])?,
        ),
        Primitive::Text {
            origin,
            text,
            font,
            font_size,
            color,
        } => {
            let Some(origin) = point(origin)? else {
                return Ok(vec![]);
            };
            Primitive::Text {
                origin,
                text,
                font,
                font_size,
                color,
            }
        }
        Primitive::GlyphRun {
            origin,
            mut rotation,
            run,
            color,
        } => {
            if let crate::grammar::CoordinateSpec::Radial(v) = &map.spec
                && v.rotate_angle
            {
                let theta = map.theta_angle(fixed.unwrap_or(origin)).unwrap_or(0.) * 180.
                    / std::f64::consts::PI;
                rotation -= theta;
                rotation = rotation.rem_euclid(360.);
                if rotation > 90. && rotation < 270. {
                    rotation = (rotation + 180.).rem_euclid(360.);
                }
            }
            let Some(origin) = point(origin)? else {
                return Ok(vec![]);
            };
            Primitive::GlyphRun {
                origin,
                rotation,
                run,
                color,
            }
        }
        image @ (Primitive::RasterImage { .. }
        | Primitive::GradientRectangle { .. }
        | Primitive::SampledGradientRectangle { .. }) => {
            return super::coordinate_raster::project(&image, map, request, remaining);
        }
        other @ Primitive::NativePaint { .. } => {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                format!(
                    "Coordinate raster/paint lowering is not available for {}.",
                    match other {
                        Primitive::RasterImage { .. } => "raster images",
                        Primitive::NativePaint { .. } => "native painters",
                        _ => "gradient rectangles",
                    }
                ),
                "Use the shared coordinate raster adapter or a supported vector primitive.",
            ));
        }
    };
    Ok(vec![result])
}
