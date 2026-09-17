//! Shared recipe projection and arrow generation in destination physical units.
use super::*;
use crate::scales::error;
use crate::{ChartResult, DiagnosticCode, Point, Rect};
use crate::{
    grammar::*,
    scene::{PathCommand, Primitive, Stroke},
    services::Units,
};
pub(super) fn stroke(mark: &PreparedMark, _layer: &Layer, request: &LayoutRequest) -> Stroke {
    let style = mark.style;
    let width = style.units.map_or_else(
        || crate::grammar::reference_linewidth(style.stroke_width, request.units),
        |u| style.stroke_width * u.factor(request.units),
    );
    Stroke {
        color: style.stroke.unwrap_or(style.color),
        width,
    }
}
pub(crate) fn with_arrows(
    mut primitives: Vec<Primitive>,
    arrow: Option<&ArrowSpec>,
    style: &Style,
    request: &LayoutRequest,
) -> ChartResult<Vec<Primitive>> {
    let Some(arrow) = arrow else {
        return Ok(primitives);
    };
    let length = arrow.length_mm
        * if request.units == Units::Points {
            72. / 25.4
        } else {
            96. / 25.4
        };
    let stroke = Stroke {
        color: style.stroke.unwrap_or(style.color),
        width: style.units.map_or_else(
            || crate::grammar::reference_linewidth(style.stroke_width, request.units),
            |u| style.stroke_width * u.factor(request.units),
        ),
    };
    let mut tips = vec![];
    for primitive in &primitives {
        let points = match primitive {
            Primitive::Rule { from, to, .. } => vec![*from, *to],
            Primitive::Path { commands, .. } | Primitive::DashedPath { commands, .. } => commands
                .iter()
                .flat_map(|c| match c {
                    PathCommand::MoveTo(p) | PathCommand::LineTo(p) => vec![*p],
                    PathCommand::QuadraticTo(a, b) => vec![*a, *b],
                    PathCommand::CubicTo(a, b, c) => vec![*a, *b, *c],
                    PathCommand::Close => vec![],
                })
                .collect(),
            Primitive::ShapePath { geometry, .. } | Primitive::VectorPath { geometry, .. } => {
                geometry
                    .lower(0.01, request.limits.max_path_commands)?
                    .into_iter()
                    .flat_map(|c| match c {
                        PathCommand::MoveTo(p) | PathCommand::LineTo(p) => vec![p],
                        PathCommand::QuadraticTo(a, b) => vec![a, b],
                        PathCommand::CubicTo(a, b, c) => vec![a, b, c],
                        PathCommand::Close => vec![],
                    })
                    .collect()
            }
            _ => vec![],
        };
        if points.len() < 2 {
            continue;
        }
        for (first, tip, other) in [
            (true, points[0], points[1]),
            (false, points[points.len() - 1], points[points.len() - 2]),
        ] {
            if first && arrow.ends == ArrowEnds::Last || !first && arrow.ends == ArrowEnds::First {
                continue;
            }
            let (dx, dy) = (other.x() - tip.x(), other.y() - tip.y());
            let norm = libm::hypot(dx, dy);
            if norm == 0. {
                continue;
            }
            let radians = arrow.angle * std::f64::consts::PI / 180.;
            let (sin, cos) = (libm::sin(radians), libm::cos(radians));
            let (dx, dy) = (dx / norm * length, dy / norm * length);
            let a = Point::new(tip.x() + dx * cos - dy * sin, tip.y() + dy * cos + dx * sin)?;
            let b = Point::new(tip.x() + dx * cos + dy * sin, tip.y() + dy * cos - dx * sin)?;
            let commands = vec![
                PathCommand::MoveTo(a),
                PathCommand::LineTo(tip),
                PathCommand::LineTo(b),
            ];
            tips.push(if arrow.closed {
                let mut path = crate::path::Path::new();
                path.move_to(a.x(), a.y())?;
                path.line_to(tip.x(), tip.y())?;
                path.line_to(b.x(), b.y())?;
                path.close_path()?;
                Primitive::ShapePath {
                    geometry: path.geometry(),
                    fill: Some(stroke.color),
                    stroke: Some(stroke),
                    dashes: vec![],
                    anchors: vec![tip],
                    fill_rule: crate::scene::FillRule::NonZero,
                }
            } else {
                Primitive::Path { commands, stroke }
            });
        }
    }
    primitives.extend(tips);
    Ok(primitives)
}
pub(super) fn project(
    mark: &PreparedMark,
    layer: &Layer,
    request: &LayoutRequest,
    plot: Rect,
    map: &dyn Fn(Point) -> ChartResult<Option<Point>>,
    axes: [&ResolvedAxis; 2],
    map_axis: &dyn Fn(f64, bool) -> ChartResult<Option<f64>>,
) -> ChartResult<Option<Vec<Primitive>>> {
    match &mark.geometry {
        PreparedGeometry::Recipe(recipe) => Ok(Some(match recipe.as_ref() {
            PreparedRecipe::Distribution(s) => {
                super::recipe_distributions::project(s, mark, layer, request, plot, map)?
            }
            PreparedRecipe::Surface(s) => {
                super::recipe_surfaces::project(s, mark, layer, request, plot, map)?
            }
            PreparedRecipe::Mark(s) => {
                super::recipe_marks::project(s, mark, layer, request, plot, map, map_axis)?
            }
            PreparedRecipe::Reference(s) => {
                let number = |v: crate::composition::ScaleValue| -> ChartResult<f64> {
                    if let crate::composition::ScaleValue::Number(v) = v {
                        Ok(v)
                    } else {
                        Err(error(
                            DiagnosticCode::UnsupportedCapability,
                            "Reference equations require numeric axes.",
                        ))
                    }
                };
                let xmin = number(axes[0].invert_value(plot.origin().x())?)?;
                let xmax = number(axes[0].invert_value(plot.max_x())?)?;
                let ymin = number(axes[1].invert_value(plot.max_y())?)?;
                let ymax = number(axes[1].invert_value(plot.origin().y())?)?;
                let (a, b) = match s.kind {
                    ReferenceKind::Abline => (
                        Point::new(xmin, s.slope * xmin + s.intercept)?,
                        Point::new(xmax, s.slope * xmax + s.intercept)?,
                    ),
                    ReferenceKind::Horizontal => (
                        Point::new(xmin, s.intercept)?,
                        Point::new(xmax, s.intercept)?,
                    ),
                    ReferenceKind::Vertical => (
                        Point::new(s.intercept, ymin)?,
                        Point::new(s.intercept, ymax)?,
                    ),
                };
                let project_reference = |p: Point| -> ChartResult<Option<Point>> {
                    let project = |v: f64, axis: &ResolvedAxis| match &axis.space {
                        ValueSpace::Scaled { scale, .. } => scale.project_optional(Some(v)),
                        _ => Some(v),
                    };
                    let (Some(x), Some(y)) = (project(p.x(), axes[0]), project(p.y(), axes[1]))
                    else {
                        return Ok(None);
                    };
                    map(Point::new(x, y)?)
                };
                if let (Some(from), Some(to)) = (project_reference(a)?, project_reference(b)?) {
                    with_arrows(
                        vec![Primitive::Rule {
                            from,
                            to,
                            stroke: stroke(mark, layer, request),
                        }],
                        s.arrow.as_ref(),
                        &mark.style,
                        request,
                    )?
                } else {
                    vec![]
                }
            }
        })),
        PreparedGeometry::Rule { from, to }
            if matches!(layer.recipe, Some(BuiltinRecipe::Segment { .. })) =>
        {
            let Some(BuiltinRecipe::Segment { arrow }) = &layer.recipe else {
                unreachable!()
            };
            Ok(Some(
                if let (Some(from), Some(to)) = (map(*from)?, map(*to)?) {
                    with_arrows(
                        vec![Primitive::Rule {
                            from,
                            to,
                            stroke: stroke(mark, layer, request),
                        }],
                        arrow.as_ref(),
                        &mark.style,
                        request,
                    )?
                } else {
                    vec![]
                },
            ))
        }
        _ => Ok(None),
    }
}
