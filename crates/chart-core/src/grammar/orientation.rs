//! Oriented families reuse the vertical statistical/positioning kernel and transpose once.
use super::*;
use crate::{ChartResult, Point};
use std::{borrow::Cow, sync::Arc};
/// Independent-axis direction for statistics and oriented geometry families.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Orientation {
    /// Independent x with dependent y; preserves legacy recipes.
    #[default]
    Vertical,
    /// Independent y with dependent x.
    Horizontal,
}
impl Orientation {
    pub(crate) fn is_vertical(&self) -> bool {
        *self == Self::Vertical
    }
}
pub(crate) fn transpose_source(a: &mut SourceAes) {
    std::mem::swap(&mut a.x, &mut a.y);
    std::mem::swap(&mut a.x2, &mut a.y2);
}
pub(crate) fn transpose_mappings(a: &mut Mappings) {
    match a {
        Mappings::Source(a) => transpose_source(a),
        Mappings::Statistical(a) => {
            std::mem::swap(&mut a.x, &mut a.y);
            std::mem::swap(&mut a.x2, &mut a.y2);
        }
        Mappings::Binned(a) => {
            std::mem::swap(&mut a.x, &mut a.y);
            std::mem::swap(&mut a.x2, &mut a.y2);
        }
    }
}
pub(super) fn resolve(definition: &ChartDefinition) -> ChartResult<Cow<'_, ChartDefinition>> {
    if definition
        .layers
        .iter()
        .all(|l| l.orientation == Orientation::Vertical)
    {
        return Ok(Cow::Borrowed(definition));
    }
    let mut result = definition.clone();
    for layer in &mut result.layers {
        if layer.orientation == Orientation::Horizontal {
            if layer.geometry_extension.is_some() {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Registered geometry must declare its own orientation contract before using horizontal normalization.",
                ));
            }
            if let Mappings::Source(a) = &mut layer.mappings
                && layer.inherit
            {
                *a = a.inherit(&definition.mappings);
                layer.inherit = false;
            }
            transpose_mappings(&mut layer.mappings);
            if let Some(g) = &mut layer.grammar {
                transpose_source(&mut g.source);
            }
            if let Position::Jitter(s) = &mut layer.position {
                std::mem::swap(&mut s.x, &mut s.y);
            }
        }
    }
    Ok(Cow::Owned(result))
}
pub(super) fn domains(domains: &mut DomainContributions) {
    std::mem::swap(&mut domains.x, &mut domains.y);
    std::mem::swap(&mut domains.x_space, &mut domains.y_space);
}
fn point(p: Point) -> ChartResult<Point> {
    Point::new(p.y(), p.x())
}
fn geometry(geometry: &mut PreparedGeometry) -> ChartResult<()> {
    match geometry {
        PreparedGeometry::UnboundedPoint(p) => p.swap(0, 1),
        PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {}
        PreparedGeometry::Point(p)
        | PreparedGeometry::ShapePath { center: p, .. }
        | PreparedGeometry::ShapePathRun { center: p, .. } => *p = point(*p)?,
        PreparedGeometry::LineRun(p) | PreparedGeometry::Polygon(p) => {
            for v in p {
                *v = point(*v)?;
            }
        }
        PreparedGeometry::BandRun { lower, upper }
        | PreparedGeometry::StackBandRun { lower, upper, .. } => {
            for v in lower.iter_mut().chain(upper) {
                *v = point(*v)?;
            }
        }
        PreparedGeometry::Rule { from, to }
        | PreparedGeometry::Rectangle { from, to }
        | PreparedGeometry::Bar { from, to, .. }
        | PreparedGeometry::NativePaint { from, to, .. } => {
            *from = point(*from)?;
            *to = point(*to)?;
        }
    }
    Ok(())
}
pub(super) fn output(layer: &mut PreparedLayer) -> ChartResult<()> {
    if layer.orientation == Orientation::Horizontal {
        for mark in Arc::make_mut(&mut layer.marks) {
            geometry(&mut mark.geometry)?;
        }
        domains(&mut layer.domains);
        if let Position::Jitter(s) = &mut layer.position {
            std::mem::swap(&mut s.x, &mut s.y);
        }
    }
    Ok(())
}
