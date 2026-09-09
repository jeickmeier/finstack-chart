//! Named radial parameters and authored runs over the shared shape generators.
use super::{compiler::EncodedRow, *};
use crate::provenance::Target;
use crate::{
    ChartResult, DiagnosticCode, Point,
    path::PathLimits,
    shape::{AreaPoint, AreaRadial, LineRadial, LinkRadial, ShapeLimits, SymbolPaint},
};
use std::sync::Arc;

/// Constant polar boundaries, overridden by named shape channels in destination units.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RadialParameters {
    /// Start/source angle in radians clockwise from twelve o'clock.
    pub start_angle: f64,
    /// Optional end/target angle; None reuses the start angle.
    pub end_angle: Option<f64>,
    /// Inner/source radius; radial lines use this as their radius.
    pub inner_radius: f64,
    /// Optional outer/target radius; None reuses the inner radius.
    pub outer_radius: Option<f64>,
}
impl Default for RadialParameters {
    fn default() -> Self {
        Self {
            start_angle: 0.,
            end_angle: None,
            inner_radius: 0.,
            outer_radius: Some(40.),
        }
    }
}
impl RadialParameters {
    /// Validate finite constants without clamping signed radii or wrapping angles.
    pub fn validate(self) -> ChartResult<()> {
        if [
            Some(self.start_angle),
            self.end_angle,
            Some(self.inner_radius),
            self.outer_radius,
        ]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Radial parameters must be finite.",
            ));
        }
        Ok(())
    }
    fn pair(self) -> AreaPoint {
        AreaPoint {
            lower: [self.start_angle, self.inner_radius],
            upper: [
                self.end_angle.unwrap_or(self.start_angle),
                self.outer_radius.unwrap_or(self.inner_radius),
            ],
        }
    }
}
pub(super) fn defaults(geom: Geom) -> Option<RadialParameters> {
    match geom {
        Geom::ShapeLineRadial { parameters, .. }
        | Geom::ShapeAreaRadial { parameters, .. }
        | Geom::ShapeLinkRadial { parameters } => Some(parameters),
        _ => None,
    }
}
pub(super) fn set(parameters: &mut RadialParameters, channel: NumericAesthetic, value: f64) {
    use NumericAesthetic as A;
    match channel {
        A::Angle => {
            parameters.start_angle = value;
            parameters.end_angle = None;
        }
        A::Radius => {
            parameters.inner_radius = value;
            parameters.outer_radius = None;
        }
        A::StartAngle => parameters.start_angle = value,
        A::EndAngle => parameters.end_angle = Some(value),
        A::InnerRadius => parameters.inner_radius = value,
        A::OuterRadius => parameters.outer_radius = Some(value),
        _ => unreachable!("validated radial channel"),
    }
}
fn limits(limits: CompileLimits) -> ShapeLimits {
    ShapeLimits {
        max_points: limits.max_prepared_rows,
        path: PathLimits {
            max_commands: limits.max_vertices,
            ..Default::default()
        },
    }
}
pub(super) fn link_geometry(
    geom: Geom,
    row: &EncodedRow,
    center: Point,
    compile: CompileLimits,
) -> ChartResult<PreparedGeometry> {
    let parameters = super::shape_encoding::radial_parameters(geom, row);
    let pair = parameters.pair();
    let geometry = LinkRadial::new()
        .limits(limits(compile))
        .generate_by(&pair, |p| Ok([p.lower, p.upper]))?
        .geometry();
    let anchors = [pair.lower, pair.upper]
        .into_iter()
        .map(|p| {
            let a = crate::shape::point_radial(p[0], p[1])?;
            Point::new(a[0], a[1])
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(PreparedGeometry::ShapePathRun {
        paint: SymbolPaint::Stroke,
        center,
        geometry,
        anchors,
    })
}
pub(super) fn emit_block(
    prepared: &mut PreparedLayer,
    rows: &mut Vec<EncodedRow>,
    group: &GroupValue,
    style: Style,
    geom: Geom,
    vertices: &mut usize,
    compile: CompileLimits,
) -> ChartResult<()> {
    let (order, connect) = geom.run().expect("radial run");
    let (curve, area) = match geom {
        Geom::ShapeLineRadial { curve, .. } => (curve, false),
        Geom::ShapeAreaRadial { curve, .. } => (curve, true),
        _ => unreachable!("radial run"),
    };
    if order == LineOrder::X {
        rows.sort_by(|a, b| {
            super::shape_encoding::radial_parameters(geom, a)
                .start_angle
                .partial_cmp(&super::shape_encoding::radial_parameters(geom, b).start_angle)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.ordinal.cmp(&b.ordinal))
        });
    }
    let mut points = vec![];
    let mut targets = vec![];
    let mut center = None;
    let flush = |points: &mut Vec<AreaPoint>,
                 targets: &mut Vec<Target>,
                 center: &mut Option<Point>,
                 prepared: &mut PreparedLayer,
                 vertices: &mut usize|
     -> ChartResult<()> {
        if points.is_empty() {
            return Ok(());
        }
        let geometry = if area {
            AreaRadial::new()
                .curve(curve)?
                .limits(limits(compile))
                .generate_with(
                    points,
                    prepared
                        .shape_protocols
                        .curve()
                        .map_or(&curve as &dyn crate::shape::CurveFactory, |p| p),
                    |p, _, _| Ok(Some(*p)),
                )?
        } else {
            LineRadial::new()
                .curve(curve)?
                .limits(limits(compile))
                .generate_with(
                    points,
                    prepared
                        .shape_protocols
                        .curve()
                        .map_or(&curve as &dyn crate::shape::CurveFactory, |p| p),
                    |p, _, _| Ok(Some(p.lower)),
                )?
        }
        .geometry();
        if !geometry.commands().is_empty() {
            let anchors = points
                .iter()
                .map(|p| {
                    let polar = if area { p.upper } else { p.lower };
                    let p = crate::shape::curve_point_radial(polar[0], polar[1])?;
                    Point::new(p[0], p[1])
                })
                .collect::<ChartResult<Vec<_>>>()?;
            super::compiler::charge(
                vertices,
                geometry.commands().len().saturating_add(anchors.len()),
                "radial path and source anchor",
            )?;
            let center = center.expect("nonempty radial run");
            Extent::include(&mut prepared.domains.x, center.x());
            Extent::include(&mut prepared.domains.y, center.y());
            Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                geometry: PreparedGeometry::ShapePathRun {
                    paint: if area {
                        SymbolPaint::Fill
                    } else {
                        SymbolPaint::Stroke
                    },
                    center,
                    geometry,
                    anchors,
                },
                targets: std::mem::take(targets),
                group: group.clone(),
                style,
            });
        }
        points.clear();
        targets.clear();
        *center = None;
        Ok(())
    };
    for row in rows.drain(..) {
        if let (Some(x), Some(y)) = (row.x, row.y) {
            let next = Point::new(x, y)?;
            if center.is_some_and(|p| p != next) {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Each radial run requires one common x/y center; split distinct centers into groups.",
                ));
            }
            center = Some(next);
            points.push(super::shape_encoding::radial_parameters(geom, &row).pair());
            targets.push(row.target);
        } else if !connect {
            flush(&mut points, &mut targets, &mut center, prepared, vertices)?;
        }
    }
    flush(&mut points, &mut targets, &mut center, prepared, vertices)
}
