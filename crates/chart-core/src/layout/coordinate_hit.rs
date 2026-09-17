//! Preserve extension interaction regions through the shared coordinate geometry path.
use super::{LayoutRequest, coordinate_map::CoordinateMap};
use crate::ChartResult;
use crate::grammar::HitGeometry;
use crate::path::{Path, PathGeometry};
use crate::scene::{Color, FillRule, Primitive};
pub(super) fn project(
    hit: HitGeometry,
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Option<HitGeometry>> {
    lower(hit, map, request, remaining, false)
}
pub(super) fn clip_presented(
    hit: HitGeometry,
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Option<HitGeometry>> {
    lower(hit, map, request, remaining, true)
}
fn lower(
    hit: HitGeometry,
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
    already_projected: bool,
) -> ChartResult<Option<HitGeometry>> {
    let original_anchor = hit.anchor()?;
    let paint = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    let primitive = match hit {
        HitGeometry::Point { center, radius } => Primitive::Point {
            center,
            radius,
            fill: paint,
        },
        HitGeometry::Rectangle { from, to } => Primitive::Rectangle {
            bounds: crate::Rect::new(
                from.x().min(to.x()),
                from.y().min(to.y()),
                (to.x() - from.x()).abs(),
                (to.y() - from.y()).abs(),
            )?,
            fill: paint,
        },
        HitGeometry::Polygon(points) => {
            let mut p = Path::new();
            p.move_to(points[0].x(), points[0].y())?;
            for q in &points[1..] {
                p.line_to(q.x(), q.y())?;
            }
            p.close_path()?;
            Primitive::ShapePath {
                geometry: p.geometry(),
                fill: Some(paint),
                stroke: None,
                dashes: vec![],
                anchors: vec![],
                fill_rule: FillRule::EvenOdd,
            }
        }
        HitGeometry::Path {
            geometry,
            fill_rule,
            ..
        } => Primitive::ShapePath {
            geometry,
            fill: Some(paint),
            stroke: None,
            dashes: vec![],
            anchors: vec![],
            fill_rule,
        },
    };
    let error = super::coordinate_path::tolerance(request);
    let projected = if already_projected {
        vec![primitive]
    } else {
        super::coordinate_primitive::project(primitive, map, None, 0, error, remaining, request)?
    };
    let mut pieces = vec![];
    for primitive in projected {
        pieces.extend(super::coordinate_clip::clip(
            primitive, map, 0, error, remaining,
        )?);
    }
    let mut commands = vec![];
    let mut rule = FillRule::NonZero;
    for p in pieces {
        match p {
            Primitive::Point { center, radius, .. } => {
                return Ok(Some(HitGeometry::Point { center, radius }));
            }
            Primitive::ShapePath {
                geometry,
                fill_rule,
                ..
            } => {
                commands.extend(geometry.commands().iter().copied());
                rule = fill_rule;
            }
            _ => {}
        }
    }
    if commands.is_empty() {
        return Ok(None);
    }
    let geometry = PathGeometry::from_commands(commands, request.limits.max_path_commands)?;
    let flat = geometry.flatten(error, request.limits.max_path_commands)?;
    let mapped = if already_projected {
        Some(original_anchor)
    } else {
        map.project(original_anchor)?
    };
    let anchor = mapped
        .filter(|p| flat.contains_with_rule(*p, true, None, rule))
        .or_else(|| {
            flat.subpaths
                .iter()
                .flat_map(|s| s.points.iter())
                .next()
                .copied()
        });
    Ok(anchor.map(|anchor| HitGeometry::Path {
        geometry,
        fill_rule: rule,
        anchor,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{CoordinateClip, CoordinateSpec, RadialCoordinate};
    use crate::layout::coordinate_map::CoordinateDomain;
    use crate::services::{ResourceDescriptor, ResourceKind, Units};
    use crate::{Point, Rect, ResourceId, Revision};
    #[test]
    fn compound_extension_hit_uses_same_annulus_and_sector_as_paint() {
        let plot = Rect::new(0., 0., 200., 200.).unwrap();
        let request = LayoutRequest::new(
            plot,
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        for end in [None, Some(std::f64::consts::PI)] {
            let map = CoordinateMap::new(
                CoordinateSpec::Radial(RadialCoordinate {
                    inner_radius: 0.5,
                    end,
                    clip: Some(CoordinateClip::On),
                    expand: false,
                    ..Default::default()
                }),
                [
                    CoordinateDomain {
                        domain: [0., 1.],
                        viewport: [0., 1.],
                        range: [0., 200.],
                    },
                    CoordinateDomain {
                        domain: [0., 1.],
                        viewport: [0., 1.],
                        range: [200., 0.],
                    },
                ],
                [[0., 1.], [0., 1.]],
                plot,
            )
            .unwrap();
            let hit = HitGeometry::Rectangle {
                from: Point::new(0., 0.).unwrap(),
                to: Point::new(200., 200.).unwrap(),
            };
            let transformed = project(hit, &map, &request, &mut 1_000_000)
                .unwrap()
                .unwrap();
            assert!(matches!(transformed, HitGeometry::Path { .. }));
            for theta in [0.2, 0.4, 0.6, 0.8] {
                let p = map.project_values([theta, 0.5]).unwrap().unwrap();
                assert!(transformed.contains(p), "visible point {theta}");
            }
            // A radial value inside the hole must never be recoverable from
            // the custom polygon's bounding box or semantic target anchor.
            let p = map.project_values([0.5, -0.5]).unwrap().unwrap();
            assert!(!transformed.contains(p));
        }
    }
}
