//! Explicit immutable-font shaping for chart adapters. No I/O, system fonts or workers.
//! A run declares one language and direction; callers split mixed-direction paragraphs.
use chart_core::scene::PathCommand;
use chart_core::services::{ResourceDescriptor, ResourceKind, TextMetrics};
use chart_core::typography::{ShapeRequest, ShapedGlyph, ShapedRun, TextDirection};
use chart_core::{ChartResult, Diagnostic, DiagnosticCode, Point};
use std::collections::BTreeSet;

fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Supply a supported explicit static face and valid directional rich text.",
    )
}
/// Whether this exact face supplies every logical character in one run.
/// Fallback selection is whole-run to preserve joining and ligature context.
pub fn supports(bytes: &[u8], text: &str, weight: u16) -> ChartResult<bool> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| {
        error(
            DiagnosticCode::InvalidResource,
            format!("Invalid shaping font: {e:?}"),
        )
    })?;
    Ok(face.weight().to_number() == weight && text.chars().all(|c| face.glyph_index(c).is_some()))
}
struct Outline {
    commands: Vec<PathCommand>,
    scale: f64,
    x: f64,
    y: f64,
    remaining: usize,
    failed: bool,
}
impl Outline {
    fn point(&mut self, x: f32, y: f32) -> Point {
        match Point::new(
            self.x + f64::from(x) * self.scale,
            self.y - f64::from(y) * self.scale,
        ) {
            Ok(p) => p,
            Err(_) => {
                self.failed = true;
                Point::new(0., 0.).unwrap_or_else(|_| unreachable!())
            }
        }
    }
    fn push(&mut self, c: PathCommand) {
        if self.remaining == 0 {
            self.failed = true;
        } else {
            self.remaining -= 1;
            self.commands.push(c);
        }
    }
}
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.push(PathCommand::MoveTo(p));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.push(PathCommand::LineTo(p));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let a = self.point(x1, y1);
        let b = self.point(x, y);
        self.push(PathCommand::QuadraticTo(a, b));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let a = self.point(x1, y1);
        let b = self.point(x2, y2);
        let c = self.point(x, y);
        self.push(PathCommand::CubicTo(a, b, c));
    }
    fn close(&mut self) {
        self.push(PathCommand::Close);
    }
}
/// Shape and outline with the exact selected font. The adapter selects only declared fallback
/// resources and passes whether one was used; this function never looks up another face.
pub fn shape(
    bytes: &[u8],
    font: ResourceDescriptor,
    r: ShapeRequest<'_>,
    used_fallback: bool,
) -> ChartResult<ShapedRun> {
    let result = (|| {
        r.run.validate(r.limits)?;
        let font_size = r.font_size * r.run.size;
        if font.kind != ResourceKind::Font
            || font.byte_len != bytes.len() as u64
            || !font_size.is_finite()
            || font_size <= 0.
            || font_size > 262_144.
        {
            return Err(error(
                DiagnosticCode::InvalidResource,
                "Invalid shaping font descriptor or destination size.",
            ));
        }
        if !supports(bytes, &r.run.text, r.run.weight)? {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Explicit face lacks a requested glyph or its weight differs from the run.",
            ));
        }
        let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| {
            error(
                DiagnosticCode::InvalidResource,
                format!("Invalid face: {e:?}"),
            )
        })?;
        if face.is_variable() || face.tables().glyf.is_none() && face.tables().cff.is_none() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Rich shaping requires a static TrueType/CFF outline face.",
            ));
        }
        let scale = font_size / f64::from(face.units_per_em());
        let source = harfrust::FontRef::new(bytes)
            .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
        let data = harfrust::ShaperData::new(&source);
        let shaper = data.shaper(&source).build();
        let mut buffer = harfrust::UnicodeBuffer::new();
        buffer.push_str(&r.run.text);
        buffer.set_direction(match r.run.direction {
            TextDirection::LeftToRight => harfrust::Direction::LeftToRight,
            TextDirection::RightToLeft => harfrust::Direction::RightToLeft,
        });
        buffer.set_language(
            r.run
                .language
                .parse()
                .map_err(|_| error(DiagnosticCode::Validation, "Invalid shaping language tag."))?,
        );
        buffer.guess_segment_properties();
        let features = if r.run.tabular {
            vec![harfrust::Feature::new(harfrust::Tag::new(b"tnum"), 1, ..)]
        } else {
            vec![]
        };
        let output = shaper.shape(buffer, harfrust::ShapeOptions::new().features(&features));
        if output.len() > r.limits.max_path_commands {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Rich glyph count exceeds geometry budget.",
            ));
        }
        let mut ends: BTreeSet<usize> = output
            .glyph_infos()
            .iter()
            .map(|g| g.cluster as usize)
            .collect();
        ends.insert(r.run.text.len());
        let mut pen_x = 0.;
        let mut pen_y = 0.;
        let mut glyphs = Vec::with_capacity(output.len());
        let mut outlines = Outline {
            commands: vec![],
            scale,
            x: 0.,
            y: 0.,
            remaining: r.limits.max_path_commands - output.len(),
            failed: false,
        };
        for (info, pos) in output.glyph_infos().iter().zip(output.glyph_positions()) {
            let id = u16::try_from(info.glyph_id).map_err(|_| {
                error(
                    DiagnosticCode::InvalidResource,
                    "Glyph ID exceeds a static face index.",
                )
            })?;
            if id == 0 {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    "Shaper emitted an absent glyph.",
                ));
            }
            let start = info.cluster as usize;
            let end = ends
                .range((std::ops::Bound::Excluded(start), std::ops::Bound::Unbounded))
                .next()
                .copied()
                .unwrap_or(r.run.text.len());
            if !r.run.text.is_char_boundary(start) || !r.run.text.is_char_boundary(end) {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "Shaper returned invalid UTF-8 clusters.",
                ));
            }
            let position = Point::new(
                pen_x + f64::from(pos.x_offset) * scale,
                pen_y - f64::from(pos.y_offset) * scale,
            )?;
            outlines.x = position.x();
            outlines.y = position.y();
            face.outline_glyph(ttf_parser::GlyphId(id), &mut outlines);
            if outlines.failed {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Glyph outlines exceed finite geometry/command limits.",
                ));
            }
            let advance = Point::new(
                f64::from(pos.x_advance) * scale,
                -f64::from(pos.y_advance) * scale,
            )?;
            glyphs.push(ShapedGlyph {
                id,
                start,
                end,
                position,
                advance,
            });
            pen_x += advance.x();
            pen_y += advance.y();
        }
        let metrics = TextMetrics::new(
            pen_x.abs(),
            f64::from(face.ascender()).max(0.) * scale,
            -f64::from(face.descender()).min(0.) * scale,
        )?;
        Ok(ShapedRun {
            text: r.run.text.clone(),
            font,
            font_size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics,
            glyphs,
            outlines: outlines.commands,
            used_fallback,
        })
    })();
    result.map_err(|mut e: Diagnostic| {
        e.context.resource = Some(font.id);
        e.context.resource_revision = Some(font.revision);
        e
    })
}
