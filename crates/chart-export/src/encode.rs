use crate::{FontResources, PublicationProfile, TextMode, error};
use chart_core::scene::{Color, Primitive, Scene};
use chart_core::{ChartResult, DiagnosticCode};
use krilla::{
    geom::{Path, PathBuilder, Point, Transform},
    num::NormalizedF32,
    paint::{Fill, FillRule, Stroke},
    text::{Font, GlyphId, KrillaGlyph},
};
use resvg_export::tiny_skia;
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn bounded(bytes: Vec<u8>, profile: &PublicationProfile) -> ChartResult<Vec<u8>> {
    if bytes.len() > profile.max_output_bytes {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Encoded publication exceeds the output byte budget.",
        ));
    }
    Ok(bytes)
}
pub(crate) fn png(tree: &usvg::Tree, p: &PublicationProfile) -> ChartResult<Vec<u8>> {
    let (width, height) = p.raster_dimensions()?;
    let ppm = (f64::from(p.dpi) / 0.0254).round();
    if ppm > f64::from(u32::MAX) {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "PNG physical density overflows its integer metadata.",
        ));
    }
    let mut pixels = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "Raster allocation failed."))?;
    let c = p.background;
    if c.alpha == 255 {
        pixels.fill(tiny_skia::Color::from_rgba8(
            c.red, c.green, c.blue, c.alpha,
        ));
    }
    // Uniform physical scale: fractional-pixel rounding never stretches x and y separately.
    resvg_export::render(
        tree,
        tiny_skia::Transform::from_scale(p.dpi as f32 / 72., p.dpi as f32 / 72.),
        &mut pixels.as_mut(),
    );
    let rgba = pixels.take_demultiplied();
    let mut bytes = vec![];
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_pixel_dims(Some(png::PixelDimensions {
            xppu: ppm as u32,
            yppu: ppm as u32,
            unit: png::Unit::Meter,
        }));
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&rgba))
            .map_err(|e| error(DiagnosticCode::ExportFidelity, e.to_string()))?;
    }
    bounded(bytes, p)
}
fn opacity(c: Color) -> NormalizedF32 {
    NormalizedF32::new(f32::from(c.alpha) / 255.).unwrap_or(NormalizedF32::ZERO)
}
fn fill(c: Color) -> Fill {
    Fill {
        paint: krilla::color::rgb::Color::new(c.red, c.green, c.blue).into(),
        opacity: opacity(c),
        rule: FillRule::NonZero,
    }
}
fn path(data: &usvg::tiny_skia_path::Path) -> ChartResult<Path> {
    let mut builder = PathBuilder::new();
    for s in data.segments() {
        match s {
            usvg::tiny_skia_path::PathSegment::MoveTo(a) => builder.move_to(a.x, a.y),
            usvg::tiny_skia_path::PathSegment::LineTo(a) => builder.line_to(a.x, a.y),
            usvg::tiny_skia_path::PathSegment::QuadTo(a, b) => builder.quad_to(a.x, a.y, b.x, b.y),
            usvg::tiny_skia_path::PathSegment::CubicTo(a, b, c) => {
                builder.cubic_to(a.x, a.y, b.x, b.y, c.x, c.y)
            }
            usvg::tiny_skia_path::PathSegment::Close => builder.close(),
        }
    }
    builder.finish().ok_or_else(|| {
        error(
            DiagnosticCode::ExportFidelity,
            "PDF path is empty or invalid.",
        )
    })
}
pub(crate) fn pdf(
    scene: &Scene,
    tree: &usvg::Tree,
    fonts: &FontResources,
    p: &PublicationProfile,
) -> ChartResult<Vec<u8>> {
    let mut document = krilla::Document::new();
    let settings =
        krilla::page::PageSettings::from_wh(p.f32(p.page.width())?, p.f32(p.page.height())?)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::Validation,
                    "PDF page dimensions are unsupported.",
                )
            })?;
    let mut page = document.start_page_with(settings);
    let mut surface = page.surface();
    let mut pdf_fonts = BTreeMap::new();
    for f in fonts.iter() {
        let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(f.bytes.clone());
        let font = Font::new(data.into(), 0).ok_or_else(|| {
            let mut e = error(
                DiagnosticCode::InvalidResource,
                "PDF font loader rejected the supplied resource.",
            );
            e.context.resource = Some(f.descriptor.id);
            e.context.resource_revision = Some(f.descriptor.revision);
            e
        })?;
        pdf_fonts.insert(f.descriptor.id, font);
    }
    for (index, item) in scene.items().iter().enumerate() {
        let clip = item.clip.unwrap_or(scene.bounds());
        // Empty clips paint nothing, including annotations; do not turn them into unbounded paint.
        if clip.width() == 0. || clip.height() == 0. {
            continue;
        }
        let result = (|| {
            let mut builder = PathBuilder::new();
            let r = krilla::geom::Rect::from_xywh(
                p.f32(clip.origin().x())?,
                p.f32(clip.origin().y())?,
                p.f32(clip.width())?,
                p.f32(clip.height())?,
            )
            .ok_or_else(|| {
                error(
                    DiagnosticCode::PrecisionLoss,
                    "PDF clip cannot represent its bounds.",
                )
            })?;
            builder.push_rect(r);
            let clip = builder
                .finish()
                .ok_or_else(|| error(DiagnosticCode::ExportFidelity, "PDF clip has no path."))?;
            surface.push_clip_path(&clip, &FillRule::NonZero);
            let node = tree.node_by_id(&format!("item-{index}"));
            match (&item.primitive, node) {
                (
                    Primitive::GlyphRun {
                        origin,
                        rotation,
                        run,
                        color,
                    },
                    _,
                ) => {
                    surface.set_fill(Some(fill(*color)));
                    surface.set_stroke(None);
                    let font = pdf_fonts.get(&run.font.id).ok_or_else(|| {
                        error(
                            DiagnosticCode::MissingResource,
                            "PDF rich text font is absent.",
                        )
                    })?;
                    let (sin, cos) = rotation.to_radians().sin_cos();
                    surface.push_transform(&Transform::from_row(
                        p.f32(cos)?,
                        p.f32(sin)?,
                        p.f32(-sin)?,
                        p.f32(cos)?,
                        p.f32(origin.x())?,
                        p.f32(origin.y())?,
                    ));
                    let mut x = 0.;
                    let mut y = 0.;
                    let glyphs = run
                        .glyphs
                        .iter()
                        .map(|g| {
                            let glyph = KrillaGlyph::new(
                                GlyphId::new(u32::from(g.id)),
                                p.f32(g.advance.x() / run.font_size)?,
                                p.f32((g.position.x() - x) / run.font_size)?,
                                p.f32(-(g.position.y() - y) / run.font_size)?,
                                p.f32(-g.advance.y() / run.font_size)?,
                                g.start..g.end,
                                None,
                            );
                            x += g.advance.x();
                            y += g.advance.y();
                            Ok(glyph)
                        })
                        .collect::<ChartResult<Vec<_>>>()?;
                    surface.draw_glyphs(
                        Point::from_xy(0., 0.),
                        &glyphs,
                        font.clone(),
                        &run.text,
                        p.f32(run.font_size)?,
                        p.text == TextMode::Outline,
                    );
                    surface.pop();
                }
                (Primitive::Text { font, color, .. }, Some(usvg::Node::Text(text))) => {
                    surface.set_fill(Some(fill(*color)));
                    surface.set_stroke(None);
                    let font = pdf_fonts.get(font).ok_or_else(|| {
                        error(
                            DiagnosticCode::MissingResource,
                            "PDF text references an absent font.",
                        )
                    })?;
                    for span in text.layouted() {
                        for glyph in &span.positioned_glyphs {
                            // usvg already shaped and positioned every glyph. Undo only its em scale;
                            // krilla applies the declared font size to this exact glyph ID.
                            let ts = glyph.transform().pre_concat(usvg::Transform::from_scale(
                                font.units_per_em() / span.font_size.get(),
                                font.units_per_em() / span.font_size.get(),
                            ));
                            surface.push_transform(&Transform::from_row(
                                ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty,
                            ));
                            surface.draw_glyphs(
                                Point::from_xy(0., 0.),
                                &[KrillaGlyph::new(
                                    GlyphId::new(glyph.id.0),
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0..glyph.text.len(),
                                    None,
                                )],
                                font.clone(),
                                &glyph.text,
                                span.font_size.get(),
                                p.text == TextMode::Outline,
                            );
                            surface.pop();
                        }
                    }
                }
                (primitive, Some(usvg::Node::Path(node))) => {
                    match primitive {
                        Primitive::GradientRectangle { bounds, gradient } => {
                            let x1 = p.f32(bounds.origin().x())?;
                            let y1 = p.f32(bounds.origin().y())?;
                            let (x2, y2) = match gradient.direction {
                                chart_core::scene::GradientDirection::Horizontal => {
                                    (p.f32(bounds.max_x())?, y1)
                                }
                                chart_core::scene::GradientDirection::Vertical => {
                                    (x1, p.f32(bounds.max_y())?)
                                }
                            };
                            let stop = |c: Color, offset: NormalizedF32| krilla::paint::Stop {
                                offset,
                                color: krilla::color::rgb::Color::new(c.red, c.green, c.blue)
                                    .into(),
                                opacity: opacity(c),
                            };
                            surface.set_fill(Some(Fill {
                                paint: krilla::paint::LinearGradient {
                                    x1,
                                    y1,
                                    x2,
                                    y2,
                                    transform: Transform::identity(),
                                    spread_method: krilla::paint::SpreadMethod::Pad,
                                    stops: vec![
                                        stop(gradient.start, NormalizedF32::ZERO),
                                        stop(gradient.end, NormalizedF32::ONE),
                                    ],
                                    anti_alias: true,
                                }
                                .into(),
                                opacity: NormalizedF32::ONE,
                                rule: FillRule::NonZero,
                            }));
                            surface.set_stroke(None);
                        }
                        Primitive::Rectangle { fill: c, .. }
                        | Primitive::Point { fill: c, .. }
                        | Primitive::Symbol { fill: c, .. }
                        | Primitive::FilledPath { fill: c, .. } => {
                            surface.set_fill(Some(fill(*c)));
                            surface.set_stroke(None);
                        }
                        Primitive::Rule { stroke, .. }
                        | Primitive::Path { stroke, .. }
                        | Primitive::DashedPath { stroke, .. } => {
                            surface.set_fill(None);
                            surface.set_stroke(Some(Stroke {
                                paint: fill(stroke.color).paint,
                                width: p.f32(stroke.width)?,
                                opacity: opacity(stroke.color),
                                miter_limit: 4.,
                                dash: if let Primitive::DashedPath { dashes, .. } = primitive {
                                    Some(krilla::paint::StrokeDash {
                                        array: dashes
                                            .iter()
                                            .map(|v| p.f32(*v))
                                            .collect::<ChartResult<_>>()?,
                                        offset: 0.,
                                    })
                                } else {
                                    None
                                },
                                ..Default::default()
                            }));
                        }
                        Primitive::Text { .. } | Primitive::GlyphRun { .. } => {
                            return Err(error(
                                DiagnosticCode::ExportFidelity,
                                "Expected shaped PDF text, found a path.",
                            ));
                        }
                    }
                    surface.draw_path(&path(node.data())?);
                }
                (_, None) => { /* usvg omits empty/zero-alpha/degenerate marks; no geometry is invented. */
                }
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Unexpected SVG effect/group in the core primitive PDF path.",
                    ));
                }
            }
            surface.pop();
            Ok(())
        })();
        result.map_err(|mut e: chart_core::Diagnostic| {
            e.context.layer = item.layer;
            if let Primitive::Text { font, .. } = item.primitive {
                e.context.resource = Some(font);
            }
            e
        })?;
    }
    surface.finish();
    page.finish();
    let bytes = document.finish().map_err(|e| {
        error(
            DiagnosticCode::ExportFidelity,
            format!("PDF encoding failed: {e:?}"),
        )
    })?;
    bounded(bytes, p)
}
