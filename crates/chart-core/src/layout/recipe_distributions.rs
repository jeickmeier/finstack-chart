//! One shared destination projection for distribution point glyphs and dot stacks.
use crate::{
    ChartResult, Point, Rect,
    grammar::{AestheticUnits, Layer, PreparedDistribution, PreparedMark},
    layout::LayoutRequest,
    scene::{Primitive, Stroke},
};
pub(super) fn project(
    payload: &PreparedDistribution,
    mark: &PreparedMark,
    _layer: &Layer,
    request: &LayoutRequest,
    _plot: Rect,
    map: &dyn Fn(Point) -> ChartResult<Option<Point>>,
) -> ChartResult<Vec<Primitive>> {
    let style = mark.style;
    let alpha = |mut c: crate::scene::Color| {
        if let Some(a) = style.alpha {
            c.alpha = (a.clamp(0., 1.) * 255.).round() as u8;
        }
        c
    };
    match payload {
        PreparedDistribution::Dot {
            center,
            bin,
            stack_position,
            stack_ratio,
            stack_offset,
            dot_size,
            horizontal,
        } => {
            let (Some(mut center), Some(a), Some(b)) = (map(*center)?, map(bin[0])?, map(bin[1])?)
            else {
                return Ok(vec![]);
            };
            let sign = if bin[1].x() != bin[0].x() {
                (bin[1].x() - bin[0].x()).signum()
            } else {
                (bin[1].y() - bin[0].y()).signum()
            };
            let diameter = libm::hypot(b.x() - a.x(), b.y() - a.y()) * dot_size * sign;
            let shift = diameter * (stack_position * stack_ratio + stack_offset);
            center = if *horizontal {
                Point::new(center.x() + shift, center.y())?
            } else {
                Point::new(center.x(), center.y() - shift)?
            };
            if diameter == 0. {
                return Ok(vec![]);
            }
            let geometry = super::stroke_outline::circle(
                center,
                diameter.abs() / 2.,
                request.limits.max_path_commands,
            )?;
            let width = style.stroke_width
                * style
                    .units
                    .unwrap_or(AestheticUnits::Destination)
                    .factor(request.units);
            Ok(vec![Primitive::ShapePath {
                geometry,
                fill: Some(alpha(style.fill.unwrap_or(style.color))),
                stroke: style
                    .stroke
                    .map(alpha)
                    .filter(|c| {
                        c.alpha > 0
                            && width > 0.
                            && style.line_type != Some(crate::grammar::LineType::Blank)
                    })
                    .map(|color| Stroke { color, width }),
                dashes: style
                    .line_type
                    .unwrap_or(crate::grammar::LineType::Solid)
                    .pattern(width)?,
                anchors: vec![center],
                fill_rule: crate::scene::FillRule::NonZero,
            }])
        }
        PreparedDistribution::Outlier {
            center,
            size,
            stroke,
            shape,
        } => {
            let Some(center) = map(*center)? else {
                return Ok(vec![]);
            };
            if *size <= 0. {
                return Ok(vec![]);
            }
            let factor = AestheticUnits::Millimeters.factor(request.units);
            let radius = crate::grammar::reference_point_radius(*size, *stroke) * factor;
            let width = if *stroke == 0. {
                0.01 * if request.units == crate::services::Units::LogicalPixels {
                    96. / 72.
                } else {
                    1.
                }
            } else {
                stroke * factor / 2.
            };
            let kind = crate::shape::SymbolKind::Ggplot(*shape);
            let policy = crate::shape::SymbolPaint::Auto.resolve(kind)?;
            let geometry = crate::shape::Symbol::new()
                .kind(kind)
                .size(std::f64::consts::PI * radius * radius)
                .generate()?
                .geometry()
                .transformed(
                    crate::path::Affine::new([1., 0., 0., 1., center.x(), center.y()])?,
                    0.01,
                    request.limits.max_path_commands,
                )?;
            let color = alpha(style.stroke.unwrap_or(style.color));
            let fill = alpha(if policy.color_fill() {
                style.color
            } else {
                style.fill.unwrap_or(style.color)
            });
            Ok(vec![Primitive::ShapePath {
                geometry,
                fill: policy.fills().then_some(fill),
                stroke: policy.strokes().then_some(Stroke { color, width }),
                dashes: vec![],
                anchors: vec![center],
                fill_rule: crate::scene::FillRule::NonZero,
            }])
        }
    }
}

pub(super) fn density_band(
    lower: &[Point],
    upper: &[Point],
    anchors: &[Point],
    style: crate::grammar::Style,
    outline: crate::grammar::AreaOutline,
) -> ChartResult<Vec<Primitive>> {
    use crate::{
        grammar::{AreaOutline, LineType},
        scene::{FillRule, PathCommand},
    };
    let stroke = style
        .stroke
        .filter(|c| c.alpha > 0)
        .filter(|_| style.line_type != Some(LineType::Blank) && style.stroke_width > 0.)
        .map(|color| Stroke {
            color,
            width: style.stroke_width,
        });
    let fill = style.fill.filter(|c| c.alpha > 0);
    let dashes = style
        .line_type
        .map(|l| l.pattern(style.stroke_width))
        .transpose()?
        .unwrap_or_default();
    let mut result = Vec::new();
    let polygon = || -> ChartResult<_> {
        let mut commands: Vec<_> = upper
            .iter()
            .chain(lower.iter().rev())
            .enumerate()
            .map(|(i, p)| {
                if i == 0 {
                    PathCommand::MoveTo(*p)
                } else {
                    PathCommand::LineTo(*p)
                }
            })
            .collect();
        commands.push(PathCommand::Close);
        crate::path::PathGeometry::from_beziers(&commands)
    };
    if fill.is_some() || outline == AreaOutline::Full {
        result.push(Primitive::ShapePath {
            geometry: polygon()?,
            fill,
            stroke: if outline == AreaOutline::Full {
                stroke
            } else {
                None
            },
            dashes: if outline == AreaOutline::Full {
                dashes.clone()
            } else {
                vec![]
            },
            anchors: anchors.to_vec(),
            fill_rule: FillRule::NonZero,
        });
    }
    if outline != AreaOutline::Full && stroke.is_some() {
        for (points, draw) in [
            (
                upper,
                matches!(outline, AreaOutline::Upper | AreaOutline::Both),
            ),
            (
                lower,
                matches!(outline, AreaOutline::Lower | AreaOutline::Both),
            ),
        ] {
            if !draw {
                continue;
            }
            let commands: Vec<_> = points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == 0 {
                        PathCommand::MoveTo(*p)
                    } else {
                        PathCommand::LineTo(*p)
                    }
                })
                .collect();
            result.push(Primitive::ShapePath {
                geometry: crate::path::PathGeometry::from_beziers(&commands)?,
                fill: None,
                stroke,
                dashes: dashes.clone(),
                anchors: points.to_vec(),
                fill_rule: FillRule::NonZero,
            });
        }
    }
    Ok(result)
}
