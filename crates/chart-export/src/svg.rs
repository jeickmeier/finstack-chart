use crate::{FontResources, PublicationProfile, error};
use base64::Engine;
use chart_core::scene::{Color, PathCommand, Primitive, Scene};
use chart_core::{ChartResult, DiagnosticCode};
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
pub(crate) fn build(
    scene: &Scene,
    fonts: &FontResources,
    p: &PublicationProfile,
    embed: bool,
) -> ChartResult<String> {
    scene.require_portable_paint()?;
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
        for (index, item) in scene.items().iter().enumerate() {
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
                        "<path id=\"item-{index}\" fill=\"{}\" fill-opacity=\"{}\" d=\"",
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
                        "<g id=\"item-{index}\" role=\"img\" aria-label=\"{}\"><desc>{}</desc><path fill=\"{}\" fill-opacity=\"{}\" fill-rule=\"nonzero\" d=\"",
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
                Primitive::GradientRectangle { bounds, gradient } => {
                    let (x2, y2) = match gradient.direction {
                        chart_core::scene::GradientDirection::Horizontal => (1, 0),
                        chart_core::scene::GradientDirection::Vertical => (0, 1),
                    };
                    write!(
                        out,
                        "<defs><linearGradient id=\"gradient-{index}\" x1=\"0\" y1=\"0\" x2=\"{x2}\" y2=\"{y2}\" color-interpolation=\"sRGB\"><stop offset=\"0\" stop-color=\"{}\" stop-opacity=\"{}\"/><stop offset=\"1\" stop-color=\"{}\" stop-opacity=\"{}\"/></linearGradient></defs><rect id=\"item-{index}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#gradient-{index})\"/>",
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
                    "<rect id=\"item-{index}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>",
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
                    "<circle id=\"item-{index}\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    center.x(),
                    center.y(),
                    radius,
                    color(*fill),
                    alpha(*fill)
                )?,
                Primitive::Rule { from, to, stroke } => write!(
                    out,
                    "<path id=\"item-{index}\" d=\"M {} {} L {} {}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>",
                    from.x(),
                    from.y(),
                    to.x(),
                    to.y(),
                    color(stroke.color),
                    stroke.width,
                    alpha(stroke.color)
                )?,
                Primitive::Path { commands, .. }
                | Primitive::FilledPath { commands, .. }
                | Primitive::DashedPath { commands, .. } => {
                    write!(out, "<path id=\"item-{index}\" d=\"")?;
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
                    // Resource and glyph preflight already ran, so this lookup is only formatting.
                    let alias = fonts
                        .iter()
                        .find(|f| f.descriptor.id == *font)
                        .map(|f| f.alias())
                        .ok_or(std::fmt::Error)?;
                    write!(
                        out,
                        "<text id=\"item-{index}\" x=\"{}\" y=\"{}\" font-family=\"{alias}\" font-size=\"{font_size}\" xml:space=\"preserve\" fill=\"{}\" fill-opacity=\"{}\">{}</text>",
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
/// Outline serialization keeps the exact point viewBox despite SVG's default CSS DPI.
pub(crate) fn outline(tree: &usvg::Tree, p: &PublicationProfile) -> ChartResult<Vec<u8>> {
    let text = tree.to_string(&usvg::WriteOptions::default());
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
