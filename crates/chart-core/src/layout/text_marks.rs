//! Portable row labels share rich shaping, resources, scene budgets and provenance.
use super::{LayoutRequest, project::Output};
use crate::grammar::{PreparedMark, TextGeom, ValueAesthetic as A};
use crate::interpolate::{Number, Value};
use crate::scene::{Primitive, SceneItem, Stroke};
use crate::services::TextMeasurer;
use crate::{ChartResult, DiagnosticCode, LayerId, Point, Rect};

fn number(mark: &PreparedMark, channel: A, fallback: f64) -> f64 {
    match mark.aesthetics.get(&channel) {
        Some(Value::Number(Number(n))) => *n,
        _ => fallback,
    }
}
fn overlap(a: &[Point; 4], b: &[Point; 4]) -> bool {
    for polygon in [a, b] {
        for edge in 0..2 {
            let p = polygon[edge];
            let q = polygon[edge + 1];
            let axis = [-(q.y() - p.y()), q.x() - p.x()];
            let extent = |points: &[Point; 4]| {
                points
                    .iter()
                    .map(|p| p.x() * axis[0] + p.y() * axis[1])
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                        (lo.min(v), hi.max(v))
                    })
            };
            let (al, ah) = extent(a);
            let (bl, bh) = extent(b);
            if ah <= bl || bh <= al {
                return false;
            }
        }
    }
    true
}
#[allow(clippy::too_many_arguments)]
pub(super) fn project(
    options: &TextGeom,
    mark: &PreparedMark,
    layer: LayerId,
    anchor: Point,
    clip: Option<Rect>,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    occupied: &mut Vec<[Point; 4]>,
    bytes: &mut usize,
    output: &mut Output,
) -> ChartResult<()> {
    let label = match mark.aesthetics.get(&A::Label) {
        Some(Value::Text(s)) => s.clone(),
        Some(Value::Number(Number(n))) => n.to_string(),
        Some(Value::Boolean(v)) => if *v { "TRUE" } else { "FALSE" }.into(),
        _ => {
            output.omitted += 1;
            return Ok(());
        }
    };
    *bytes = bytes.saturating_add(label.len());
    crate::limits::require_within(*bytes <= request.limits.max_text_bytes, "row label UTF-8")?;
    if label.is_empty() {
        return Ok(());
    }
    let family = match mark.aesthetics.get(&A::FontFamily) {
        Some(Value::Text(s)) => Some(s.as_str()),
        _ => None,
    };
    let face = match mark.aesthetics.get(&A::FontFace) {
        Some(Value::Text(s)) => s.as_str(),
        _ => "plain",
    };
    let selected = options
        .fonts
        .iter()
        .find(|f| f.family.as_str() == family.unwrap_or("") && f.face == face)
        .or_else(|| {
            if options.font.is_some() && family.is_none() && face == "plain" {
                return None;
            }
            request
                .resolved_theme
                .as_ref()?
                .fonts
                .iter()
                .find(|f| f.family.as_str() == family.unwrap_or("") && f.face == face)
        });
    if (family.is_some() || face != "plain") && selected.is_none() {
        return Err(crate::scales::error(
            DiagnosticCode::MissingResource,
            "Mapped font family and face must select an explicit supplied TextFont resource.",
        ));
    }
    let factor = options.units.factor(request.units);
    let size = number(mark, A::TextSize, options.size) * factor;
    let angle = number(mark, A::TextAngle, options.angle);
    let hjust = number(mark, A::HJust, options.hjust);
    let vjust = number(mark, A::VJust, options.vjust);
    let mut text = crate::typography::RichText::plain("");
    text.lines = label
        .split('\n')
        .map(|line| {
            let mut run = crate::typography::RichRun::new(line);
            run.size = size / request.font_size;
            run.font = selected.map(|f| f.font).or(options.font);
            if let Some(font) = selected {
                run.weight = font.weight;
            }
            vec![run]
        })
        .collect();
    if let Some(fonts) = &options.math {
        let mut parsed = crate::typography::RichText::math(label.clone(), fonts.clone())?;
        parsed.lines[0][0].size = size / request.font_size;
        parsed.lines[0][0].font = selected.map(|f| f.font).or(options.font);
        text = parsed;
    }
    text.line_spacing = number(mark, A::LineHeight, options.line_height);
    let block = super::text::measure(&text, request, measurer, mark.style.color)?;
    output.diagnostics.extend(block.diagnostics);
    let pad = if options.fill.is_some() {
        [options.padding[0] * size, options.padding[1] * size]
    } else {
        [0.; 2]
    };
    let width = block.bounds.width() + 2. * pad[0];
    let height = block.bounds.height() + 2. * pad[1];
    let (sin, cos) = angle.to_radians().sin_cos();
    let transform = |x: f64, y: f64| {
        Point::new(
            anchor.x() + cos * x - sin * y,
            anchor.y() + sin * x + cos * y,
        )
    };
    let left = -hjust * width;
    let top = -(1. - vjust) * height;
    let corners = [
        transform(left, top)?,
        transform(left + width, top)?,
        transform(left + width, top + height)?,
        transform(left, top + height)?,
    ];
    if options.check_overlap && occupied.iter().any(|other| overlap(other, &corners)) {
        output.omitted += 1;
        return Ok(());
    }
    occupied.push(corners);
    let output_start = output.items.len();
    let interaction = crate::grammar::GeometryInteraction {
        hit: crate::grammar::HitGeometry::Polygon(corners.to_vec()),
        values: vec![("label".into(), crate::grammar::SemanticValue::Text(label))],
        selection: crate::grammar::SelectionPolicy::AtomicTarget,
        keyboard_order: (occupied.len() - 1) as u64,
    };
    if let Some(fill) = options.fill {
        use crate::path::Command;
        let geometry = crate::path::PathGeometry::from_commands(
            vec![
                Command::MoveTo([corners[0].x(), corners[0].y()]),
                Command::LineTo([corners[1].x(), corners[1].y()]),
                Command::LineTo([corners[2].x(), corners[2].y()]),
                Command::LineTo([corners[3].x(), corners[3].y()]),
                Command::Close,
            ],
            request.limits.max_path_commands,
        )?;
        output.push(
            SceneItem {
                guide: None,
                layer: Some(layer),
                clip,
                primitive: Primitive::VectorPath {
                    geometry,
                    fill: Some(fill),
                    stroke: (options.border_width > 0.).then_some(Stroke {
                        color: mark.style.color,
                        width: options.border_width * factor,
                    }),
                    dashes: vec![],
                },
            },
            mark.targets.clone(),
            request,
        )?;
    }
    for mut item in block.items {
        if let Primitive::GlyphRun {
            origin, rotation, ..
        } = &mut item.primitive
        {
            *origin = transform(
                origin.x() - block.bounds.origin().x() + left + pad[0],
                origin.y() - block.bounds.origin().y() + top + pad[1],
            )?;
            *rotation = angle;
        }
        if let Primitive::Path { commands, .. } = &mut item.primitive {
            super::text::transform_commands(commands, &|p| {
                transform(
                    p.x() - block.bounds.origin().x() + left + pad[0],
                    p.y() - block.bounds.origin().y() + top + pad[1],
                )
            })?;
        }
        item.layer = Some(layer);
        item.clip = clip;
        output.push(item, mark.targets.clone(), request)?;
    }
    for index in output_start..output.items.len() {
        output.interaction(index, interaction.clone(), request)?;
    }
    Ok(())
}
