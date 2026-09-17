//! Destination-independent vector and nearest raster lowering; no image I/O in core.
use super::{LayoutRequest, project::Output};
use crate::{
    ChartResult, LayerId, Point, Rect,
    grammar::{AnnotationContent, PreparedMark, RowAnnotation},
    scene::{Primitive, SceneItem},
};
pub(super) fn project(
    annotation: &RowAnnotation,
    mark: &PreparedMark,
    layer: LayerId,
    anchor: Point,
    clip: Option<Rect>,
    request: &LayoutRequest,
    output: &mut Output,
) -> ChartResult<()> {
    annotation.validate(request.limits.max_path_commands)?;
    let factor = annotation.units.factor(request.units);
    let mut push = |primitive| {
        output.push(
            SceneItem {
                guide: None,
                layer: Some(layer),
                clip,
                primitive,
            },
            mark.targets.clone(),
            request,
        )
    };
    match &annotation.content {
        AnnotationContent::Vector {
            geometry,
            fill,
            stroke,
        } => {
            let geometry = geometry.transformed(
                crate::path::Affine::new([factor, 0., 0., factor, anchor.x(), anchor.y()])?,
                0.01,
                request.limits.max_path_commands,
            )?;
            push(Primitive::ShapePath {
                fill_rule: crate::scene::FillRule::NonZero,
                geometry,
                fill: *fill,
                stroke: stroke.map(|mut s| {
                    s.width *= factor;
                    s
                }),
                dashes: vec![],
                anchors: vec![anchor],
            })?;
        }
        AnnotationContent::Raster {
            raster,
            bounds,
            interpolate,
        } => {
            let [x, y, w, h] = *bounds;
            push(Primitive::RasterImage {
                hits: vec![],
                cells: vec![],
                bounds: Rect::new(
                    anchor.x() + factor * x,
                    anchor.y() + factor * y,
                    w * factor,
                    h * factor,
                )?,
                raster: raster.clone(),
                interpolate: *interpolate,
            })?;
        }
    }
    Ok(())
}
