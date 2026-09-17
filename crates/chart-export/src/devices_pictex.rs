//! Restricted historical PicTeX device: monochrome outlines and Computer Modern text.
use crate::{FontResources, PublicationProfile, error};
use chart_core::{
    ChartResult, DiagnosticCode, Point, Rect,
    path::{Command, PathGeometry},
    scene::{Primitive, Scene},
};
const TEX_POINTS_PER_BIG_POINT: f64 = 72.27 / 72.;
struct Writer {
    text: crate::devices::BoundedString,
    remaining: usize,
    height: f64,
}
impl Writer {
    fn add(&mut self, args: std::fmt::Arguments<'_>) -> ChartResult<()> {
        self.text
            .add(args, "PicTeX exceeds the output byte budget.")
    }
    fn point(&self, p: Point) -> [f64; 2] {
        [
            p.x() * TEX_POINTS_PER_BIG_POINT,
            (self.height - p.y()) * TEX_POINTS_PER_BIG_POINT,
        ]
    }
    fn line(&mut self, a: Point, b: Point, clip: Rect) -> ChartResult<()> {
        let Some((a, b)) = clip_segment(a, b, clip)? else {
            return Ok(());
        };
        let a = self.point(a);
        let b = self.point(b);
        self.add(format_args!(
            "\\plot {} {} {} {} /\n",
            a[0], a[1], b[0], b[1]
        ))
    }
    fn node(&mut self, node: &usvg::Node, clip: Rect) -> ChartResult<()> {
        match node {
            usvg::Node::Group(g) => {
                for n in g.children() {
                    self.node(n, clip)?;
                }
            }
            usvg::Node::Text(_) => {
                return Err(error(
                    DiagnosticCode::ExportFidelity,
                    "PicTeX text requires its retained logical scene payload.",
                ));
            }
            usvg::Node::Image(_) => {
                return Err(error(
                    DiagnosticCode::ExportFidelity,
                    "The historical PicTeX device does not support raster images.",
                ));
            }
            usvg::Node::Path(p) => {
                if !p.is_visible() {
                    return Ok(());
                }
                let Some(path) = p.data().clone().transform(p.abs_transform()) else {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "PicTeX path transform overflowed.",
                    ));
                };
                let commands = path
                    .segments()
                    .map(|s| {
                        use usvg::tiny_skia_path::PathSegment::*;
                        match s {
                            MoveTo(a) => Command::MoveTo([a.x.into(), a.y.into()]),
                            LineTo(a) => Command::LineTo([a.x.into(), a.y.into()]),
                            QuadTo(a, b) => Command::QuadraticTo([
                                a.x.into(),
                                a.y.into(),
                                b.x.into(),
                                b.y.into(),
                            ]),
                            CubicTo(a, b, c) => Command::CubicTo([
                                a.x.into(),
                                a.y.into(),
                                b.x.into(),
                                b.y.into(),
                                c.x.into(),
                                c.y.into(),
                            ]),
                            Close => Command::Close,
                        }
                    })
                    .collect();
                let geometry = PathGeometry::from_commands(commands, self.remaining)?;
                let flat = geometry.flatten(0.1, self.remaining)?;
                let count = flat.subpaths.iter().map(|s| s.points.len()).sum::<usize>();
                self.remaining = self.remaining.checked_sub(count).ok_or_else(|| {
                    error(
                        DiagnosticCode::ResourceLimit,
                        "PicTeX path vertex budget exceeded.",
                    )
                })?;
                if let Some(dashes) = p.stroke().and_then(|s| s.dasharray()) {
                    self.add(format_args!("\\setdashpattern <"))?;
                    for (i, dash) in dashes.iter().enumerate() {
                        self.add(format_args!(
                            "{}{}pt",
                            if i == 0 { "" } else { ", " },
                            f64::from(*dash) * TEX_POINTS_PER_BIG_POINT
                        ))?;
                    }
                    self.add(format_args!(">\n"))?;
                } else {
                    self.add(format_args!("\\setsolid\n"))?;
                }
                for sub in flat.subpaths {
                    for pair in sub.points.windows(2) {
                        self.line(pair[0], pair[1], clip)?;
                    }
                    if sub.closed && sub.points.len() > 1 {
                        self.line(*sub.points.last().unwrap(), sub.points[0], clip)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn text(
        &mut self,
        text: &str,
        origin: Point,
        size: f64,
        rotation: f64,
        face: &str,
        clip: Rect,
    ) -> ChartResult<()> {
        if origin.x() < clip.origin().x()
            || origin.x() > clip.max_x()
            || origin.y() < clip.origin().y()
            || origin.y() > clip.max_y()
        {
            return Ok(());
        }
        if !text.is_ascii() {
            return Err(error(
                DiagnosticCode::ExportFidelity,
                "Historical PicTeX supports literal ASCII device text, not Unicode or mathematical glyphs.",
            ));
        }
        let p = self.point(origin);
        self.add(format_args!(
            "\\font\\picfont {} at {}pt\\picfont\n\\put {{",
            face, size
        ))?;
        if rotation.rem_euclid(360.) == 270. {
            self.add(format_args!("\\rotatebox{{90}}{{"))?;
        }
        // Every non-space character is a numeric literal. No source label can inject TeX.
        for c in text.bytes() {
            match c {
                b' ' | b'\t' | b'\n' | b'\r' => self.add(format_args!("\\ "))?,
                32..=126 => self.add(format_args!("\\char{}{{}}", c))?,
                _ => {
                    return Err(error(
                        DiagnosticCode::ExportFidelity,
                        "PicTeX text contains a control character.",
                    ));
                }
            }
        }
        if rotation.rem_euclid(360.) == 270. {
            self.add(format_args!("}}"))?;
        }
        self.add(format_args!("}} [lB] <0pt,0pt> at {} {}\n", p[0], p[1]))
    }
}
fn clip_segment(a: Point, b: Point, r: Rect) -> ChartResult<Option<(Point, Point)>> {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let mut lower: f64 = 0.;
    let mut upper: f64 = 1.;
    for (p, q) in [
        (-dx, a.x() - r.origin().x()),
        (dx, r.max_x() - a.x()),
        (-dy, a.y() - r.origin().y()),
        (dy, r.max_y() - a.y()),
    ] {
        if p == 0. {
            if q < 0. {
                return Ok(None);
            }
            continue;
        }
        let t = q / p;
        if p < 0. {
            lower = lower.max(t);
        } else {
            upper = upper.min(t);
        }
        if lower > upper {
            return Ok(None);
        }
    }
    Ok(Some((
        Point::new(a.x() + lower * dx, a.y() + lower * dy)?,
        Point::new(a.x() + upper * dx, a.y() + upper * dy)?,
    )))
}
fn font_face(fonts: &FontResources, id: chart_core::ResourceId) -> ChartResult<&'static str> {
    use skrifa::{FontRef, MetadataProvider};
    let resource = fonts
        .iter()
        .find(|f| f.descriptor.id == id)
        .ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "PicTeX text font metadata is missing.",
            )
        })?;
    let font = FontRef::new(&resource.bytes)
        .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
    let a = font.attributes();
    Ok(if a.weight.value() >= 600. {
        "cmssbx10"
    } else if a.style != skrifa::attribute::Style::Normal {
        "cmssi10"
    } else {
        "cmss10"
    })
}
pub(crate) fn encode(
    scene: &Scene,
    tree: &usvg::Tree,
    fonts: &FontResources,
    p: &PublicationProfile,
) -> ChartResult<Vec<u8>> {
    let mut w = Writer {
        text: crate::devices::BoundedString::new(p.max_output_bytes),
        remaining: p.layout.limits.max_path_commands,
        height: p.page.height(),
    };
    w.add(format_args!("% finstack-chart historical PicTeX device: requires pictex and graphicx.\n% Monochrome outlines; fills and ordinary linewidths are not represented.\n\\hbox{{\\beginpicture\n\\setcoordinatesystem units <1pt,1pt>\n\\setplotarea x from 0 to {}, y from 0 to {}\n\\setlinear\n",p.page.width()*TEX_POINTS_PER_BIG_POINT,p.page.height()*TEX_POINTS_PER_BIG_POINT))?;
    // The first scene item is the publication page background. The historical
    // device ignores background filling rather than drawing its boundary.
    for (i, item) in scene.items().iter().enumerate().skip(1) {
        let clip = item.clip.unwrap_or(scene.bounds());
        if clip.width() == 0. || clip.height() == 0. {
            continue;
        }
        match &item.primitive {
            Primitive::Text {
                origin,
                font,
                font_size,
                text,
                ..
            } => w.text(
                text,
                *origin,
                *font_size,
                0.,
                font_face(fonts, *font)?,
                clip,
            )?,
            Primitive::GlyphRun {
                origin,
                rotation,
                run,
                ..
            } => w.text(
                &run.text,
                *origin,
                run.font_size,
                *rotation,
                font_face(fonts, run.font.id)?,
                clip,
            )?,
            Primitive::RasterImage { .. } => {
                return Err(error(
                    DiagnosticCode::ExportFidelity,
                    "The reference PicTeX device does not support raster images.",
                ));
            }
            _ => {
                if let Some(node) = tree.node_by_id(&format!("item-{i}")) {
                    w.node(node, clip)?;
                }
            }
        }
    }
    w.add(format_args!("\\endpicture\n}}\n"))?;
    Ok(w.text.text.into_bytes())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipped_segment_and_safe_text_are_bounded() {
        let r = Rect::new(0., 0., 10., 10.).unwrap();
        let (a, b) = clip_segment(
            Point::new(-5., 5.).unwrap(),
            Point::new(15., 5.).unwrap(),
            r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.x(), 0.);
        assert_eq!(b.x(), 10.);
        let mut w = Writer {
            text: crate::devices::BoundedString::new(10_000),
            remaining: 100,
            height: 10.,
        };
        w.text(
            "\\input{bad}_$",
            Point::new(1., 1.).unwrap(),
            10.,
            0.,
            "cmss10",
            r,
        )
        .unwrap();
        assert!(!w.text.text.contains("\\input"));
        assert!(w.text.text.contains("\\char92{}"));
    }
}
