use crate::{FontResources, PublicationProfile, error};
use base64::Engine;
use chart_core::scene::{Color, GuideComponent, GuideRole, PathCommand, Primitive, Scene};
use chart_core::{ChartResult, DiagnosticCode};
use std::collections::BTreeMap;
use std::fmt::Write;

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
struct Writer {
    text: String,
    limit: usize,
}
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self
            .text
            .len()
            .checked_add(s.len())
            .is_none_or(|n| n > self.limit)
        {
            return Err(std::fmt::Error);
        }
        self.text.push_str(s);
        Ok(())
    }
}
fn color(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue)
}
fn alpha(c: Color) -> f64 {
    f64::from(c.alpha) / 255.
}
fn component_attributes(component: Option<&GuideComponent>) -> String {
    match component {
        Some(c) => {
            let role = match c.role {
                GuideRole::LegendTitle => "legend-title",
                GuideRole::LegendKey => "legend-key",
                GuideRole::LegendLabel => "legend-label",
                GuideRole::LegendBar => "legend-bar",
                GuideRole::LegendTick => "legend-tick",
                GuideRole::Domain => "domain",
                GuideRole::Line => "tick-line",
                GuideRole::Label => "label",
            };
            let mut attrs = format!(" class=\"{role}\" data-guide-role=\"{role}\"");
            if let Some(label) = &c.label {
                write!(attrs, " data-label=\"{}\"", escape(label)).unwrap();
            }
            attrs
        }
        None => String::new(),
    }
}
fn guide_groups<'a>(
    out: &mut Writer,
    active: &mut Option<&'a GuideComponent>,
    next: Option<&'a GuideComponent>,
    index: usize,
) -> Result<(), std::fmt::Error> {
    let same_axis = active
        .zip(next)
        .is_some_and(|(a, b)| a.guide == b.guide && a.scope == b.scope && a.side == b.side);
    let same_tick = same_axis
        && active.zip(next).is_some_and(|(a, b)| {
            a.tick == b.tick && a.animation.map(|v| v.identity) == b.animation.map(|v| v.identity)
        });
    if active.is_some_and(|a| a.tick.is_some()) && !same_tick {
        out.write_str("</g>")?;
    }
    if active.is_some() && !same_axis {
        out.write_str("</g>")?;
    }
    if let Some(c) = next {
        if !same_axis {
            let scope = chart_core::portable::encode(&c.scope).map_err(|_| std::fmt::Error)?;
            write!(
                out,
                "<g id=\"axis-{index}\" class=\"axis\" data-guide-id=\"{}\" data-guide-scope=\"{}\">",
                c.guide.get(),
                escape(&scope)
            )?;
        }
        if let Some(tick) = &c.tick
            && !same_tick
        {
            let value = chart_core::portable::encode(&tick.value).map_err(|_| std::fmt::Error)?;
            write!(
                out,
                "<g id=\"tick-{index}\" class=\"tick\" data-value=\"{}\" data-occurrence=\"{}\" data-index=\"{}\" data-label=\"{}\"{}>",
                escape(&value),
                tick.occurrence,
                c.index.unwrap_or_default(),
                escape(c.label.as_deref().unwrap_or_default()),
                c.animation
                    .map(|a| format!(
                        " data-animation-id=\"{}\" opacity=\"{}\"",
                        a.identity, a.opacity
                    ))
                    .unwrap_or_default()
            )?;
        }
    }
    *active = next;
    Ok(())
}
pub(crate) fn build(
    scene: &Scene,
    fonts: &FontResources,
    p: &PublicationProfile,
    embed: bool,
) -> ChartResult<String> {
    build_with_outlines(scene, fonts, p, embed, None)
}
fn build_with_outlines(
    scene: &Scene,
    fonts: &FontResources,
    p: &PublicationProfile,
    embed: bool,
    outlines: Option<&BTreeMap<usize, &str>>,
) -> ChartResult<String> {
    scene.require_portable_paint()?;
    let mut remaining_path_bytes = p.max_output_bytes;
    // The parsed tree uses bounded shared lowering. Standalone SVG retains analytic arcs.
    let vector_paths = scene
        .items()
        .iter()
        .map(|item| {
            if let Primitive::VectorPath { geometry, .. } | Primitive::ShapePath { geometry, .. } =
                &item.primitive
            {
                let lowered;
                let geometry = if embed {
                    geometry
                } else {
                    lowered =
                        chart_core::path::PathGeometry::from_beziers(&lower_path(geometry, p)?)?;
                    &lowered
                };
                let text = geometry
                    .to_svg(chart_core::path::Precision::Unrounded, remaining_path_bytes)?;
                remaining_path_bytes -= text.len();
                Ok(Some(text))
            } else {
                Ok(None)
            }
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let mut out = Writer {
        text: String::new(),
        limit: p.max_output_bytes,
    };
    let result = (|| -> Result<(), std::fmt::Error> {
        write!(
            out,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}pt\" height=\"{}pt\" viewBox=\"0 0 {} {}\">",
            p.page.width(),
            p.page.height(),
            p.page.width(),
            p.page.height()
        )?;
        if embed {
            out.write_str("<defs><style>")?;
            for f in fonts.iter() {
                write!(
                    out,
                    "@font-face{{font-family:'{}';src:url(data:{};base64,",
                    f.alias(),
                    f.embedding_type().0
                )?;
                // Bound the expansion before making the temporary encoded font allocation.
                let n = f
                    .bytes
                    .len()
                    .checked_add(2)
                    .and_then(|n| (n / 3).checked_mul(4))
                    .ok_or(std::fmt::Error)?;
                if n > out.limit - out.text.len() {
                    return Err(std::fmt::Error);
                }
                out.write_str(&base64::engine::general_purpose::STANDARD.encode(&f.bytes))?;
                write!(
                    out,
                    ") format('{}');font-weight:normal;font-style:normal;}}",
                    f.embedding_type().1
                )?;
            }
            out.write_str("</style></defs>")?;
        }
        let mut active: Option<&GuideComponent> = None;
        for (index, item) in scene.items().iter().enumerate() {
            if crate::snapshot::point_is_clipped(item, scene.bounds()) {
                continue;
            }
            guide_groups(&mut out, &mut active, item.guide.as_ref(), index)?;
            let attrs = component_attributes(item.guide.as_ref());
            let clip = item.clip.unwrap_or(scene.bounds());
            write!(
                out,
                "<defs><clipPath id=\"clip-{index}\" clipPathUnits=\"userSpaceOnUse\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath></defs><g clip-path=\"url(#clip-{index})\">",
                clip.origin().x(),
                clip.origin().y(),
                clip.width(),
                clip.height()
            )?;
            match &item.primitive {
                Primitive::VectorPath {
                    fill,
                    stroke,
                    dashes,
                    ..
                }
                | Primitive::ShapePath {
                    fill,
                    stroke,
                    dashes,
                    ..
                } => {
                    write!(
                        out,
                        "<path id=\"item-{index}\"{attrs} d=\"{}\" fill-rule=\"nonzero\"",
                        vector_paths[index].as_deref().unwrap_or_default()
                    )?;
                    if let Some(c) = fill {
                        write!(
                            out,
                            " fill=\"{}\" fill-opacity=\"{}\"",
                            color(*c),
                            alpha(*c)
                        )?;
                    } else {
                        out.write_str(" fill=\"none\"")?;
                    }
                    if let Some(s) = stroke {
                        write!(
                            out,
                            " stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"",
                            color(s.color),
                            s.width,
                            alpha(s.color)
                        )?;
                    }
                    if stroke.is_some() && !dashes.is_empty() {
                        out.write_str(" stroke-dasharray=\"")?;
                        for (i, dash) in dashes.iter().enumerate() {
                            if i > 0 {
                                out.write_str(",")?;
                            }
                            write!(out, "{dash}")?;
                        }
                        out.write_str("\"")?;
                    }
                    out.write_str("/>")?;
                }
                Primitive::NativePaint { .. } => {
                    unreachable!("portable paint checked before encoding")
                }
                Primitive::Symbol {
                    center,
                    radius,
                    kind,
                    fill,
                } => {
                    write!(
                        out,
                        "<path id=\"item-{index}\"{attrs} fill=\"{}\" fill-opacity=\"{}\" d=\"",
                        color(*fill),
                        alpha(*fill)
                    )?;
                    for c in chart_core::scene::symbol_path(*center, *radius, *kind)
                        .map_err(|_| std::fmt::Error)?
                    {
                        write_command(&mut out, &c)?;
                    }
                    out.write_str("\"/>")?;
                }
                Primitive::GlyphRun {
                    origin,
                    rotation,
                    run,
                    color: c,
                } => {
                    // Exact positioned outlines are the display truth; logical content stays explicit.
                    write!(
                        out,
                        "<g id=\"item-{index}\"{attrs} role=\"img\" aria-label=\"{}\"><desc>{}</desc><path fill=\"{}\" fill-opacity=\"{}\" fill-rule=\"nonzero\" d=\"",
                        escape(&run.text),
                        escape(&run.text),
                        color(*c),
                        alpha(*c)
                    )?;
                    let commands = chart_core::typography::placed_outlines(run, *origin, *rotation)
                        .map_err(|_| std::fmt::Error)?;
                    for c in &commands {
                        write_command(&mut out, c)?;
                    }
                    out.write_str("\"/></g>")?;
                }
                Primitive::SampledGradientRectangle {
                    bounds,
                    direction,
                    colors,
                    mode,
                } => {
                    let (x2, y2) = match direction {
                        chart_core::scene::GradientDirection::Horizontal => (1, 0),
                        chart_core::scene::GradientDirection::Vertical => (0, 1),
                    };
                    write!(
                        out,
                        "<defs><linearGradient id=\"gradient-{index}\" x1=\"0\" y1=\"0\" x2=\"{x2}\" y2=\"{y2}\" color-interpolation=\"sRGB\" spreadMethod=\"pad\">"
                    )?;
                    for (offset, sample) in mode.stops(colors) {
                        write!(
                            out,
                            "<stop offset=\"{}\" stop-color=\"{}\" stop-opacity=\"{}\"/>",
                            offset,
                            color(sample),
                            alpha(sample)
                        )?;
                    }
                    write!(
                        out,
                        "</linearGradient></defs><rect id=\"item-{index}\"{attrs} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#gradient-{index})\"/>",
                        bounds.origin().x(),
                        bounds.origin().y(),
                        bounds.width(),
                        bounds.height()
                    )?;
                }
                Primitive::GradientRectangle { bounds, gradient } => {
                    let (x2, y2) = match gradient.direction {
                        chart_core::scene::GradientDirection::Horizontal => (1, 0),
                        chart_core::scene::GradientDirection::Vertical => (0, 1),
                    };
                    write!(
                        out,
                        "<defs><linearGradient id=\"gradient-{index}\" x1=\"0\" y1=\"0\" x2=\"{x2}\" y2=\"{y2}\" color-interpolation=\"sRGB\"><stop offset=\"0\" stop-color=\"{}\" stop-opacity=\"{}\"/><stop offset=\"1\" stop-color=\"{}\" stop-opacity=\"{}\"/></linearGradient></defs><rect id=\"item-{index}\"{attrs} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#gradient-{index})\"/>",
                        color(gradient.start),
                        alpha(gradient.start),
                        color(gradient.end),
                        alpha(gradient.end),
                        bounds.origin().x(),
                        bounds.origin().y(),
                        bounds.width(),
                        bounds.height()
                    )?;
                }
                Primitive::Rectangle { bounds, fill } => write!(
                    out,
                    "<rect id=\"item-{index}\"{attrs} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    bounds.origin().x(),
                    bounds.origin().y(),
                    bounds.width(),
                    bounds.height(),
                    color(*fill),
                    alpha(*fill)
                )?,
                Primitive::Point {
                    center,
                    radius,
                    fill,
                } => write!(
                    out,
                    "<circle id=\"item-{index}\"{attrs} cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    center.x(),
                    center.y(),
                    radius,
                    color(*fill),
                    alpha(*fill)
                )?,
                Primitive::Rule { from, to, stroke } => {
                    if item
                        .guide
                        .as_ref()
                        .is_some_and(|g| g.role == GuideRole::Line)
                    {
                        write!(
                            out,
                            "<line id=\"item-{index}\"{attrs} x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>",
                            from.x(),
                            from.y(),
                            to.x(),
                            to.y(),
                            color(stroke.color),
                            stroke.width,
                            alpha(stroke.color)
                        )?;
                    } else {
                        write!(
                            out,
                            "<path id=\"item-{index}\"{attrs} d=\"M {} {} L {} {}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>",
                            from.x(),
                            from.y(),
                            to.x(),
                            to.y(),
                            color(stroke.color),
                            stroke.width,
                            alpha(stroke.color)
                        )?;
                    }
                }
                Primitive::DashedPath {
                    commands,
                    stroke,
                    dashes,
                } if item
                    .guide
                    .as_ref()
                    .is_some_and(|g| g.role == GuideRole::Line) =>
                {
                    let [PathCommand::MoveTo(from), PathCommand::LineTo(to)] = commands.as_slice()
                    else {
                        return Err(std::fmt::Error);
                    };
                    write!(
                        out,
                        "<line id=\"item-{index}\"{attrs} x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\" stroke-dasharray=\"",
                        from.x(),
                        from.y(),
                        to.x(),
                        to.y(),
                        color(stroke.color),
                        stroke.width,
                        alpha(stroke.color)
                    )?;
                    for (i, dash) in dashes.iter().enumerate() {
                        if i > 0 {
                            out.write_str(",")?;
                        }
                        write!(out, "{dash}")?;
                    }
                    out.write_str("\"/>")?;
                }
                Primitive::Path { commands, .. }
                | Primitive::FilledPath { commands, .. }
                | Primitive::DashedPath { commands, .. } => {
                    write!(out, "<path id=\"item-{index}\"{attrs} d=\"")?;
                    for c in commands {
                        write_command(&mut out, c)?;
                    }
                    match &item.primitive {
                        Primitive::DashedPath { stroke, dashes, .. } => {
                            write!(
                                out,
                                "\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\" stroke-dasharray=\"",
                                color(stroke.color),
                                stroke.width,
                                alpha(stroke.color)
                            )?;
                            for v in dashes {
                                write!(out, "{v} ")?;
                            }
                            out.write_str("\"/>")?;
                        }
                        Primitive::Path { stroke, .. } => write!(
                            out,
                            "\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>",
                            color(stroke.color),
                            stroke.width,
                            alpha(stroke.color)
                        )?,
                        Primitive::FilledPath { fill, .. } => write!(
                            out,
                            "\" fill=\"{}\" fill-opacity=\"{}\" fill-rule=\"nonzero\"/>",
                            color(*fill),
                            alpha(*fill)
                        )?,
                        _ => unreachable!(),
                    }
                }
                Primitive::Text {
                    origin,
                    text,
                    font,
                    font_size,
                    color: c,
                } => {
                    if let Some(outlines) = outlines {
                        if let Some(fragment) = outlines.get(&index) {
                            let end = fragment.find('>').ok_or(std::fmt::Error)?;
                            out.write_str(&fragment[..end])?;
                            out.write_str(&attrs)?;
                            out.write_str(&fragment[end..])?;
                        } else {
                            // Empty or fully transparent text may be pruned by the renderer.
                            write!(
                                out,
                                "<g id=\"item-{index}\"{attrs}><desc>{}</desc></g>",
                                escape(text)
                            )?;
                        }
                        out.write_str("</g>")?;
                        continue;
                    }
                    // Resource and glyph preflight already ran, so this lookup is only formatting.
                    let alias = fonts
                        .iter()
                        .find(|f| f.descriptor.id == *font)
                        .map(|f| f.alias())
                        .ok_or(std::fmt::Error)?;
                    write!(
                        out,
                        "<text id=\"item-{index}\"{attrs} x=\"{}\" y=\"{}\" font-family=\"{alias}\" font-size=\"{font_size}\" xml:space=\"preserve\" fill=\"{}\" fill-opacity=\"{}\">{}</text>",
                        origin.x(),
                        origin.y(),
                        color(*c),
                        alpha(*c),
                        escape(text)
                    )?;
                }
            }
            out.write_str("</g>")?;
        }
        guide_groups(&mut out, &mut active, None, scene.items().len())?;
        out.write_str("</svg>")
    })();
    result.map_err(|_| {
        error(
            DiagnosticCode::ResourceLimit,
            "SVG exceeds the encoded output budget.",
        )
    })?;
    Ok(out.text)
}
/// Reserve half a quarter-pixel budget for lowering and half for binary32 projection.
pub(crate) fn lower_path(
    geometry: &chart_core::path::PathGeometry,
    p: &PublicationProfile,
) -> ChartResult<Vec<PathCommand>> {
    let error = (0.125 * 72. / f64::from(p.dpi)).min(p.precision);
    let commands = geometry.lower(error, p.layout.limits.max_path_commands)?;
    let check = |point: chart_core::Point| -> ChartResult<()> {
        for value in [point.x(), point.y()] {
            let rounded = p.f32(value)?;
            if (value - f64::from(rounded)).abs() > error / std::f64::consts::SQRT_2 {
                return Err(crate::error(
                    DiagnosticCode::PrecisionLoss,
                    "Retained path exceeds its destination pixel precision.",
                ));
            }
        }
        Ok(())
    };
    for command in &commands {
        match *command {
            PathCommand::MoveTo(a) | PathCommand::LineTo(a) => check(a)?,
            PathCommand::QuadraticTo(a, b) => {
                check(a)?;
                check(b)?;
            }
            PathCommand::CubicTo(a, b, c) => {
                check(a)?;
                check(b)?;
                check(c)?;
            }
            PathCommand::Close => (),
        }
    }
    Ok(commands)
}
/// Outline serialization keeps the exact point viewBox despite SVG's default CSS DPI.
pub(crate) fn outline(
    tree: &usvg::Tree,
    scene: &Scene,
    fonts: &FontResources,
    p: &PublicationProfile,
) -> ChartResult<Vec<u8>> {
    let text = tree.to_string(&usvg::WriteOptions::default());
    if scene.items().iter().any(|item| item.guide.is_some()) {
        // Reuse the already positioned glyph outlines, not a second font/layout route.
        let document = usvg::roxmltree::Document::parse(&text)
            .map_err(|e| error(DiagnosticCode::ExportFidelity, e.to_string()))?;
        let by_id: BTreeMap<_, _> = document
            .descendants()
            .filter_map(|node| node.attribute("id").map(|id| (id, node.range())))
            .collect();
        let mut fragments = BTreeMap::new();
        for (index, item) in scene.items().iter().enumerate() {
            if matches!(item.primitive, Primitive::Text { .. })
                && let Some(range) = by_id.get(format!("item-{index}").as_str())
            {
                fragments.insert(index, &text[range.clone()]);
            }
        }
        return super::encode::bounded(
            build_with_outlines(scene, fonts, p, false, Some(&fragments))?.into_bytes(),
            p,
        );
    }
    let (_, body) = text.split_once('>').ok_or_else(|| {
        error(
            DiagnosticCode::ExportFidelity,
            "Outline writer omitted the SVG root.",
        )
    })?;
    let result=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}pt\" height=\"{}pt\" viewBox=\"0 0 {} {}\">{body}",p.page.width(),p.page.height(),tree.size().width(),tree.size().height()).into_bytes();
    super::encode::bounded(result, p)
}

fn write_command(out: &mut Writer, c: &PathCommand) -> Result<(), std::fmt::Error> {
    match c {
        PathCommand::MoveTo(a) => write!(out, "M {} {} ", a.x(), a.y()),
        PathCommand::LineTo(a) => write!(out, "L {} {} ", a.x(), a.y()),
        PathCommand::QuadraticTo(a, b) => write!(out, "Q {} {} {} {} ", a.x(), a.y(), b.x(), b.y()),
        PathCommand::CubicTo(a, b, c) => write!(
            out,
            "C {} {} {} {} {} {} ",
            a.x(),
            a.y(),
            b.x(),
            b.y(),
            c.x(),
            c.y()
        ),
        PathCommand::Close => out.write_str("Z "),
    }
}
