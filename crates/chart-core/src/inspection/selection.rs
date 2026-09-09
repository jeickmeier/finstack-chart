use super::*;
use crate::grammar::{HitGeometry, PanelKey, SelectionPolicy};
use crate::state::MarkTarget;
use std::collections::BTreeSet;

/// Selection expressed in exact presented coordinates; querying never filters source rows.
#[derive(Clone, Debug)]
pub enum SelectionRegion {
    /// Select the frontmost inspected target, using the configured hover radius.
    Point(Point),
    /// Select all visible targets of a layer in one panel.
    Series {
        /// Series/layer identity.
        layer: LayerId,
        /// Stable panel identity.
        panel: Option<PanelKey>,
    },
    /// Inclusive horizontal brush, across the presented scene height.
    XRange(f64, f64),
    /// Inclusive vertical brush, across the presented scene width.
    YRange(f64, f64),
    /// Inclusive rectangular brush. Scatter/line vertices test centers; bars intersect.
    Rectangle(Rect),
    /// Closed even-odd lasso with 3..4096 vertices, including its boundary.
    Lasso(Vec<Point>),
}
impl Inspector {
    /// Query bounded provenance-aware identities against the exact presented stamp.
    /// A result exceeding limit is rejected in full; there is no silent partial selection.
    pub fn select(
        &self,
        stamp: SceneStamp,
        region: &SelectionRegion,
        limit: usize,
    ) -> ChartResult<Vec<MarkTarget>> {
        if stamp != self.presented.scene().stamp() {
            return Err(error(
                DiagnosticCode::Superseded,
                "Selection belongs to another presented scene.",
            ));
        }
        if !(1..=4096).contains(&limit) {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Selection limit must be 1..4096.",
            ));
        }
        let epoch = self.presented.prepared().source().get()?.epoch();
        if let SelectionRegion::Point(p) = region {
            return Ok(self
                .query(*p, InspectionMode::Auto)
                .hits
                .into_iter()
                .take(1)
                .filter(|h| h.selection != SelectionPolicy::Disabled)
                .map(|h| MarkTarget::from_inspected(&h, epoch))
                .collect());
        }
        let scene = self.presented.scene().bounds();
        let polygon = match region {
            SelectionRegion::XRange(a, b) | SelectionRegion::YRange(a, b) => {
                if !a.is_finite() || !b.is_finite() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Brush endpoints must be finite.",
                    ));
                }
                if matches!(region, SelectionRegion::XRange(..)) {
                    corners(Bounds {
                        x0: a.min(*b),
                        x1: a.max(*b),
                        y0: scene.origin().y(),
                        y1: scene.max_y(),
                    })?
                } else {
                    corners(Bounds {
                        x0: scene.origin().x(),
                        x1: scene.max_x(),
                        y0: a.min(*b),
                        y1: a.max(*b),
                    })?
                }
            }
            SelectionRegion::Rectangle(r) => corners((*r).into())?,
            SelectionRegion::Lasso(points) => {
                if !(3..=4096).contains(&points.len()) {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Lasso needs 3..4096 vertices.",
                    ));
                }
                points.clone()
            }
            _ => corners(scene.into())?,
        };
        let shape = HitGeometry::Polygon(polygon.clone());
        let bounds = Bounds::from(shape.bounds()?);
        let mut selected = BTreeSet::new();
        let mut overflow = false;
        self.index.visit(bounds, |i| {
            let c = &self.index.candidates[i];
            if c.shape.is_some() || overflow || c.hit.selection == SelectionPolicy::Disabled {
                return;
            }
            let matches = if let SelectionRegion::Series { layer, panel } = region {
                c.hit.layer == *layer && c.hit.panel == *panel
            } else if c.clamped_anchor {
                false
            } else if let Some(custom) = &c.custom {
                match &custom.hit {
                    HitGeometry::Point { center, .. } => {
                        contains(c.clip, *center) && shape.contains(*center)
                    }
                    HitGeometry::Rectangle { .. } | HitGeometry::Polygon(_) => {
                        let vertices = match &custom.hit {
                            HitGeometry::Polygon(p) => p.clone(),
                            _ => {
                                corners(custom.hit.bounds().expect("validated hit geometry").into())
                                    .expect("validated corners")
                            }
                        };
                        let clipped = clip_polygon(vertices, c.clip);
                        polygons_intersect(&polygon, &clipped)
                    }
                }
            } else if c.rectangle.is_some() {
                polygons_intersect(&polygon, &corners(c.bounds).expect("validated rectangle"))
            } else if let Some((a, b, _)) = c.segment {
                super::index::clip_segment(a, b, c.clip).is_some_and(|(a, b)| {
                    shape.contains(a)
                        || shape.contains(b)
                        || edges(&polygon).any(|(u, v)| segments_intersect(a, b, u, v))
                })
            } else {
                shape.contains(c.hit.position)
            };
            if matches {
                selected.insert(MarkTarget::from_inspected(&c.hit, epoch));
                overflow = selected.len() > limit;
            }
        });
        if overflow {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Selection exceeds the configured target limit.",
            ));
        }
        Ok(selected.into_iter().collect())
    }
}
fn corners(b: Bounds) -> ChartResult<Vec<Point>> {
    Ok(vec![
        Point::new(b.x0, b.y0)?,
        Point::new(b.x1, b.y0)?,
        Point::new(b.x1, b.y1)?,
        Point::new(b.x0, b.y1)?,
    ])
}
fn edges(points: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
}
fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    fn cross(a: Point, b: Point, c: Point) -> f64 {
        (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
    }
    fn on(a: Point, b: Point, c: Point) -> bool {
        c.x() >= a.x().min(b.x())
            && c.x() <= a.x().max(b.x())
            && c.y() >= a.y().min(b.y())
            && c.y() <= a.y().max(b.y())
    }
    let (ab_c, ab_d, cd_a, cd_b) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (ab_c == 0. && on(a, b, c))
        || (ab_d == 0. && on(a, b, d))
        || (cd_a == 0. && on(c, d, a))
        || (cd_b == 0. && on(c, d, b))
        || ((ab_c > 0.) != (ab_d > 0.) && (cd_a > 0.) != (cd_b > 0.))
}
fn polygons_intersect(a: &[Point], b: &[Point]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    HitGeometry::Polygon(a.to_vec()).contains(b[0])
        || HitGeometry::Polygon(b.to_vec()).contains(a[0])
        || edges(a).any(|(u, v)| edges(b).any(|(s, t)| segments_intersect(u, v, s, t)))
}
fn clip_polygon(mut points: Vec<Point>, r: Rect) -> Vec<Point> {
    for (axis, value, lower) in [
        (true, r.origin().x(), true),
        (true, r.max_x(), false),
        (false, r.origin().y(), true),
        (false, r.max_y(), false),
    ] {
        let coord = |p: Point| if axis { p.x() } else { p.y() };
        let inside = |p: Point| {
            if lower {
                coord(p) >= value
            } else {
                coord(p) <= value
            }
        };
        let mut next = vec![];
        for (a, b) in edges(&points) {
            if inside(a) != inside(b) {
                let t = (value - coord(a)) / (coord(b) - coord(a));
                if let Ok(p) = Point::new(
                    if axis {
                        value
                    } else {
                        a.x() + t * (b.x() - a.x())
                    },
                    if axis {
                        a.y() + t * (b.y() - a.y())
                    } else {
                        value
                    },
                ) {
                    next.push(p)
                }
            }
            if inside(b) {
                next.push(b)
            }
        }
        points = next;
    }
    points
}
