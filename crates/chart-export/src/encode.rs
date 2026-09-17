use crate::{FontResources, PublicationProfile, TextMode, error};
use chart_core::scene::{Color, Primitive, Scene};
use chart_core::{ChartResult, DiagnosticCode, Rect};
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
    let rgba = raster_rgba(tree, p, width, height)?;
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
pub(crate) fn raster_rgba(
    tree: &usvg::Tree,
    p: &PublicationProfile,
    width: u32,
    height: u32,
) -> ChartResult<Vec<u8>> {
    let mut pixels = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "Raster allocation failed."))?;
    let c = p.background.resolve();
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
    Ok(pixels.take_demultiplied())
}
fn opacity(c: Color) -> NormalizedF32 {
    NormalizedF32::new(f32::from(c.alpha) / 255.).unwrap_or(NormalizedF32::ZERO)
}
fn linear_gradient(
    p: &PublicationProfile,
    bounds: Rect,
    direction: chart_core::scene::GradientDirection,
    stops: impl Iterator<Item = (f64, Color)>,
) -> ChartResult<Fill> {
    let x1 = p.f32(bounds.origin().x())?;
    let y1 = p.f32(bounds.origin().y())?;
    let (x2, y2) = match direction {
        chart_core::scene::GradientDirection::Horizontal => (p.f32(bounds.max_x())?, y1),
        chart_core::scene::GradientDirection::Vertical => (x1, p.f32(bounds.max_y())?),
    };
    let stops = stops
        .map(|(position, c)| {
            let offset = p.f32(position)?;
            Ok(krilla::paint::Stop {
                offset: NormalizedF32::new(offset).ok_or_else(|| {
                    error(
                        DiagnosticCode::ExportFidelity,
                        "Invalid gradient stop offset.",
                    )
                })?,
                color: krilla::color::rgb::Color::new(c.red, c.green, c.blue).into(),
                opacity: opacity(c),
            })
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(Fill {
        paint: krilla::paint::LinearGradient {
            x1,
            y1,
            x2,
            y2,
            transform: Transform::identity(),
            spread_method: krilla::paint::SpreadMethod::Pad,
            stops,
            anti_alias: true,
        }
        .into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    })
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
pub(crate) fn pdf_pages(
    pages: &[(&Scene, &usvg::Tree, &FontResources, &PublicationProfile)],
) -> ChartResult<Vec<u8>> {
    let Some(first) = pages.first() else {
        return Err(error(
            DiagnosticCode::Validation,
            "PDF requires at least one page.",
        ));
    };
    let limit = pages
        .iter()
        .map(|v| v.3.max_output_bytes)
        .min()
        .expect("nonempty");
    if pages.len() > 1024 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "PDF page budget exceeded.",
        ));
    }
    let mut document = krilla::Document::new();
    for &(scene, tree, fonts, p) in pages {
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
                crate::resource_error(
                    DiagnosticCode::InvalidResource,
                    "PDF font loader rejected the supplied resource.",
                    &f.descriptor,
                )
            })?;
            pdf_fonts.insert(f.descriptor.id, font);
        }
        let mut active: Option<&chart_core::scene::GuideComponent> = None;
        for (index, item) in scene.items().iter().enumerate() {
            if crate::snapshot::point_is_clipped(item, scene.bounds()) {
                continue;
            }
            let next = item.guide.as_ref().filter(|g| g.animation.is_some());
            let same = active.zip(next).is_some_and(|(a, b)| {
                a.scope == b.scope
                    && a.guide == b.guide
                    && a.side == b.side
                    && a.animation == b.animation
            });
            if !same {
                if active.is_some() {
                    surface.pop();
                }
                if let Some(next) = next {
                    surface.push_opacity(
                        NormalizedF32::new(next.animation.expect("sampled tick").opacity as f32)
                            .expect("validated opacity"),
                    );
                }
            }
            active = next;
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
                let clip = builder.finish().ok_or_else(|| {
                    error(DiagnosticCode::ExportFidelity, "PDF clip has no path.")
                })?;
                surface.push_clip_path(&clip, &FillRule::NonZero);
                let node = tree.node_by_id(&format!("item-{index}"));
                match (&item.primitive, node) {
                    (
                        Primitive::RasterImage {
                            bounds,
                            raster,
                            interpolate,
                            ..
                        },
                        _,
                    ) => {
                        surface.push_transform(&Transform::from_translate(
                            p.f32(bounds.origin().x())?,
                            p.f32(bounds.origin().y())?,
                        ));
                        let size = krilla::geom::Size::from_wh(
                            p.f32(bounds.width())?,
                            p.f32(bounds.height())?,
                        )
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::PrecisionLoss,
                                "Raster PDF dimensions cannot be represented.",
                            )
                        })?;
                        surface.draw_image(crate::raster::pdf(raster, *interpolate)?, size);
                        surface.pop();
                    }

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
                            Primitive::RasterImage { .. } => {
                                unreachable!("raster handled before vector paths")
                            }
                            Primitive::VectorPath {
                                fill: c,
                                stroke,
                                dashes,
                                ..
                            }
                            | Primitive::ShapePath {
                                fill: c,
                                stroke,
                                dashes,
                                ..
                            } => {
                                surface.set_fill(c.map(|c| {
                                    let mut f = fill(c);
                                    if matches!(
                                        primitive,
                                        Primitive::ShapePath {
                                            fill_rule: chart_core::scene::FillRule::EvenOdd,
                                            ..
                                        }
                                    ) {
                                        f.rule = FillRule::EvenOdd;
                                    }
                                    f
                                }));
                                surface.set_stroke(
                                    stroke
                                        .map(|s| -> ChartResult<Stroke> {
                                            Ok(Stroke {
                                                paint: fill(s.color).paint,
                                                width: p.f32(s.width)?,
                                                opacity: opacity(s.color),
                                                miter_limit: 4.,
                                                dash: if dashes.is_empty() {
                                                    None
                                                } else {
                                                    Some(krilla::paint::StrokeDash {
                                                        array: dashes
                                                            .iter()
                                                            .map(|v| p.f32(*v))
                                                            .collect::<ChartResult<_>>()?,
                                                        offset: 0.,
                                                    })
                                                },
                                                ..Stroke::default()
                                            })
                                        })
                                        .transpose()?,
                                );
                            }
                            Primitive::NativePaint { .. } => {
                                return Err(error(
                                    DiagnosticCode::UnsupportedCapability,
                                    "Native painters cannot be encoded as PDF.",
                                ));
                            }
                            Primitive::SampledGradientRectangle {
                                bounds,
                                direction,
                                colors,
                                mode,
                            } => {
                                surface.set_fill(Some(linear_gradient(
                                    p,
                                    *bounds,
                                    *direction,
                                    mode.stops(colors),
                                )?));
                                surface.set_stroke(None);
                            }
                            Primitive::GradientRectangle { bounds, gradient } => {
                                surface.set_fill(Some(linear_gradient(
                                    p,
                                    *bounds,
                                    gradient.direction,
                                    [(0., gradient.start), (1., gradient.end)].into_iter(),
                                )?));
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
        if active.is_some() {
            surface.pop();
        }
        surface.finish();
        page.finish();
    }
    let bytes = document.finish().map_err(|e| {
        error(
            DiagnosticCode::ExportFidelity,
            format!("PDF encoding failed: {e:?}"),
        )
    })?;
    if bytes.len() > limit {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "PDF document exceeds its output byte budget.",
        ));
    }
    bounded(bytes, first.3)
}
