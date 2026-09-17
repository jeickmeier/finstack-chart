use super::LayoutRequest;
use crate::scene::{Color, PathCommand, Primitive, SceneItem};
use crate::services::TextMeasurer;
use crate::typography::{RichText, ShapeRequest, placed_outlines};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect, Severity};

#[derive(Clone)]
pub(super) struct Block {
    pub bounds: Rect,
    pub items: Vec<SceneItem>,
    pub diagnostics: Vec<Diagnostic>,
}
impl Block {
    /// Position the measured (rotated) bounding box at the requested top-left.
    pub fn items_at(&self, x: f64, y: f64, clip: Rect) -> ChartResult<Vec<SceneItem>> {
        self.items
            .iter()
            .map(|item| {
                let mut item = item.clone();
                item.clip = Some(clip);
                if let Primitive::GlyphRun { origin, .. } | Primitive::Text { origin, .. } =
                    &mut item.primitive
                {
                    *origin = Point::new(
                        origin.x() + x - self.bounds.origin().x(),
                        origin.y() + y - self.bounds.origin().y(),
                    )?;
                }
                if let Primitive::Path { commands, .. } = &mut item.primitive {
                    transform_commands(commands, &|p| {
                        Point::new(
                            p.x() + x - self.bounds.origin().x(),
                            p.y() + y - self.bounds.origin().y(),
                        )
                    })?;
                }
                Ok(item)
            })
            .collect()
    }
}
/// Plain multiline guide labels retain logical text while painting separate lines.
/// Each destination measures ordinary runs; no shaping capability is required.
pub(super) fn plain_lines(
    text: &str,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    color: Color,
) -> ChartResult<Block> {
    let mut lines = Vec::new();
    let mut width = 0_f64;
    let mut ascent = 0_f64;
    let mut descent = 0_f64;
    for line in text.split('\n') {
        let metrics = crate::services::measure_text(
            measurer,
            super::engine::text_request(r, if line.is_empty() { "M" } else { line }),
            r.limits,
        )?;
        if !line.is_empty() {
            width = width.max(metrics.width());
        }
        ascent = ascent.max(metrics.ascent());
        descent = descent.max(metrics.descent());
        lines.push((line, metrics));
    }
    let step = (ascent + descent) * 1.2;
    let height = ascent + descent + step * lines.len().saturating_sub(1) as f64;
    let bounds = Rect::new(0., -ascent, width, height)?;
    let items = lines
        .into_iter()
        .enumerate()
        .filter(|(_, (line, _))| !line.is_empty())
        .map(|(index, (line, metrics))| {
            Ok(SceneItem {
                guide: None,
                layer: None,
                clip: None,
                primitive: Primitive::Text {
                    origin: Point::new((width - metrics.width()) / 2., index as f64 * step)?,
                    text: line.into(),
                    font: r.font.id,
                    font_size: r.font_size,
                    color,
                },
            })
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(Block {
        bounds,
        items,
        diagnostics: Vec::new(),
    })
}
pub(super) fn measure(
    text: &RichText,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    color: Color,
) -> ChartResult<Block> {
    text.validate(r.limits)?;
    let mut shaped = vec![];
    let mut remaining = r.limits.max_path_commands;
    let mut diagnostics = vec![];
    for line in &text.lines {
        let mut runs = vec![];
        for run in line {
            let mut limits = r.limits;
            limits.max_path_commands = remaining;
            if let Some(expression) = &run.math {
                let result = crate::typography::math_layout::layout(
                    expression,
                    ShapeRequest {
                        run,
                        default_font: &r.font,
                        font_size: r.font_size,
                        units: r.units,
                        limits,
                    },
                    measurer,
                )?;
                let count = result
                    .paint
                    .iter()
                    .map(|paint| match paint {
                        crate::typography::math_layout::MathPaint::Glyph { run, .. } => {
                            run.glyphs.len() + run.outlines.len()
                        }
                        crate::typography::math_layout::MathPaint::Stroke { commands, .. } => {
                            commands.len()
                        }
                    })
                    .sum::<usize>();
                crate::limits::require_within(count <= remaining, "math block geometry")?;
                remaining -= count;
                runs.push((
                    result,
                    run.color.map(crate::color::Paint::resolve).unwrap_or(color),
                ));
                continue;
            }
            let result = measurer.shape(ShapeRequest {
                run,
                default_font: &r.font,
                font_size: r.font_size,
                units: r.units,
                limits,
            })?;
            crate::limits::require_within(
                result.outlines.len().saturating_add(result.glyphs.len()) <= remaining,
                "rich block outline command",
            )?;
            remaining -= result.outlines.len() + result.glyphs.len();
            if result.used_fallback {
                let mut d = Diagnostic::error(
                    DiagnosticCode::MissingResource,
                    "A declared fallback face was selected for a complete rich run.",
                    "Keep this face resource with the figure, or choose a primary face covering the run.",
                );
                d.severity = Severity::Warning;
                d.context.resource = Some(result.font.id);
                d.context.resource_revision = Some(result.font.revision);
                diagnostics.push(d);
            }
            runs.push((
                crate::typography::math_layout::MathBox {
                    width: result.metrics.width(),
                    ascent: result.metrics.ascent(),
                    descent: result.metrics.descent(),
                    italic: 0.,
                    paint: vec![crate::typography::math_layout::MathPaint::Glyph {
                        run: result,
                        x: 0.,
                        y: 0.,
                    }],
                },
                run.color.map(crate::color::Paint::resolve).unwrap_or(color),
            ));
        }
        shaped.push(runs);
    }
    let radians = text.rotation.to_radians();
    let (sin, cos) = (libm::sin(radians), libm::cos(radians));
    let rotate = |x: f64, y: f64| Point::new(cos * x - sin * y, sin * x + cos * y);
    let mut min_x = 0_f64;
    let mut min_y = 0_f64;
    let mut max_x = 0_f64;
    let mut max_y = 0_f64;
    let mut include = |p: Point| {
        min_x = min_x.min(p.x());
        min_y = min_y.min(p.y());
        max_x = max_x.max(p.x());
        max_y = max_y.max(p.y());
    };
    let mut items = vec![];
    let mut baseline = 0.;
    let mut previous_descent = 0.;
    for (index, line) in shaped.into_iter().enumerate() {
        let ascent = line.iter().map(|(s, _)| s.ascent).fold(0_f64, f64::max);
        let descent = line.iter().map(|(s, _)| s.descent).fold(0_f64, f64::max);
        if index > 0 {
            baseline += (previous_descent + ascent) * text.line_spacing;
        }
        let mut x = 0.;
        for (block, color) in line {
            for point in [
                rotate(x, baseline - block.ascent)?,
                rotate(x + block.width, baseline - block.ascent)?,
                rotate(x, baseline + block.descent)?,
                rotate(x + block.width, baseline + block.descent)?,
            ] {
                include(point);
            }
            for paint in block.paint {
                let primitive = match paint {
                    crate::typography::math_layout::MathPaint::Glyph { run, x: dx, y: dy } => {
                        let origin = rotate(x + dx, baseline + dy)?;
                        for command in placed_outlines(&run, origin, text.rotation)? {
                            match command {
                                PathCommand::MoveTo(a) | PathCommand::LineTo(a) => include(a),
                                PathCommand::QuadraticTo(a, b) => {
                                    include(a);
                                    include(b);
                                }
                                PathCommand::CubicTo(a, b, c) => {
                                    include(a);
                                    include(b);
                                    include(c);
                                }
                                PathCommand::Close => {}
                            }
                        }
                        Primitive::GlyphRun {
                            origin,
                            rotation: text.rotation,
                            run,
                            color,
                        }
                    }
                    crate::typography::math_layout::MathPaint::Stroke {
                        mut commands,
                        width,
                    } => {
                        transform_commands(&mut commands, &|p| {
                            rotate(x + p.x(), baseline + p.y())
                        })?;
                        for command in &commands {
                            let mut point = |p: Point| -> ChartResult<()> {
                                include(Point::new(p.x() - width / 2., p.y() - width / 2.)?);
                                include(Point::new(p.x() + width / 2., p.y() + width / 2.)?);
                                Ok(())
                            };
                            match command {
                                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => point(*p)?,
                                PathCommand::QuadraticTo(p, q) => {
                                    point(*p)?;
                                    point(*q)?;
                                }
                                PathCommand::CubicTo(p, q, r) => {
                                    point(*p)?;
                                    point(*q)?;
                                    point(*r)?;
                                }
                                PathCommand::Close => {}
                            }
                        }
                        Primitive::Path {
                            commands,
                            stroke: crate::scene::Stroke { color, width },
                        }
                    }
                };
                items.push(SceneItem {
                    guide: None,
                    layer: None,
                    clip: None,
                    primitive,
                });
            }
            x += block.width;
        }
        previous_descent = descent;
    }
    Ok(Block {
        bounds: Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)?,
        items,
        diagnostics,
    })
}
/// Exact resources used by shaped text, plus the destination's ordinary guide font.
pub(super) fn resources(
    items: &[SceneItem],
    r: &LayoutRequest,
) -> ChartResult<Vec<crate::services::ResourceDescriptor>> {
    let mut fonts = std::collections::BTreeMap::from([(r.font.id, r.font)]);
    for item in items {
        if let Primitive::GlyphRun { run, .. } = &item.primitive
            && fonts
                .insert(run.font.id, run.font)
                .is_some_and(|f| f != run.font)
        {
            return Err(Diagnostic::error(
                DiagnosticCode::SchemaConflict,
                "Figure mixes revisions of a rich font identity.",
                "Use distinct resource identities or one coherent revision.",
            ));
        }
    }
    Ok(fonts.into_values().collect())
}

// Four axis passes plus static furniture/legend work. This is a work budget, distinct
// from the final scene's stricter byte/command budget, and shared across panels/insets.
pub(super) struct BoundedMeasurer<'a> {
    pub inner: &'a dyn TextMeasurer,
    text: std::cell::Cell<usize>,
    commands: std::cell::Cell<usize>,
    calls: std::cell::Cell<usize>,
}
impl<'a> BoundedMeasurer<'a> {
    pub fn new(inner: &'a dyn TextMeasurer, limits: crate::Limits) -> ChartResult<Self> {
        let multiply = |value: usize| {
            value.checked_mul(6).ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::ResourceLimit,
                    "Layout measurement work budget overflows.",
                    "Use representable scene limits.",
                )
            })
        };
        Ok(Self {
            inner,
            text: std::cell::Cell::new(multiply(limits.max_text_bytes)?),
            commands: std::cell::Cell::new(multiply(limits.max_path_commands)?),
            calls: std::cell::Cell::new(multiply(limits.max_items)?),
        })
    }
    fn charge(cell: &std::cell::Cell<usize>, n: usize, name: &str) -> ChartResult<()> {
        crate::limits::require_within(n <= cell.get(), name)?;
        cell.set(cell.get() - n);
        Ok(())
    }
}
impl TextMeasurer for BoundedMeasurer<'_> {
    fn measure(
        &self,
        r: crate::services::TextRequest<'_>,
    ) -> ChartResult<crate::services::TextMetrics> {
        Self::charge(&self.calls, 1, "aggregate layout measurement call")?;
        Self::charge(&self.text, r.text.len(), "aggregate layout measured text")?;
        self.inner.measure(r)
    }
    fn shape(
        &self,
        mut r: crate::typography::ShapeRequest<'_>,
    ) -> ChartResult<crate::typography::ShapedRun> {
        Self::charge(&self.calls, 1, "aggregate rich shaping call")?;
        Self::charge(&self.text, r.run.text.len(), "aggregate rich measured text")?;
        r.limits.max_path_commands = r.limits.max_path_commands.min(self.commands.get());
        let result = self.inner.shape(r)?;
        Self::charge(
            &self.commands,
            result.outlines.len().saturating_add(result.glyphs.len()),
            "aggregate rich shaped geometry",
        )?;
        Ok(result)
    }
}

/// Apply one geometry mapping to all mathematical rule controls.
pub(super) fn transform_commands(
    commands: &mut [PathCommand],
    map: &dyn Fn(Point) -> ChartResult<Point>,
) -> ChartResult<()> {
    for command in commands {
        match command {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => *p = map(*p)?,
            PathCommand::QuadraticTo(p, q) => {
                *p = map(*p)?;
                *q = map(*q)?;
            }
            PathCommand::CubicTo(p, q, r) => {
                *p = map(*p)?;
                *q = map(*q)?;
                *r = map(*r)?;
            }
            PathCommand::Close => {}
        }
    }
    Ok(())
}
