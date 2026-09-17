//! Direct PostScript paths over the already-resolved immutable publication tree.
use crate::{
    PublicationProfile,
    devices::{AlphaPolicy, BoundedString, Leaf, LeafPaint, quad_to_cubic},
    error,
};
use chart_core::{
    ChartResult, DiagnosticCode,
    scene::{Primitive, Scene},
};

/// Color conversion performed by the PostScript emitter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VectorColorModel {
    /// Calibrated sRGB, the reference default.
    #[default]
    Srgb,
    /// Calibrated sRGB with exact neutral colors emitted as gray.
    SrgbGray,
    /// Uncalibrated RGB with neutral colors emitted as gray.
    RgbGray,
    /// Uncalibrated device RGB colors.
    Rgb,
    /// Luminance using the reference 0.213/0.715/0.072 coefficients.
    Gray,
    /// Device CMYK with common black extraction.
    Cmyk,
}
/// Explicit treatment of alpha unsupported by PostScript.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VectorAlphaPolicy {
    /// Reject partial transparency instead of changing appearance.
    #[default]
    Reject,
    /// Omit partially transparent paints, matching the reference device limitation.
    OmitTranslucent,
}
/// Options for retained-vector PostScript/EPS output.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VectorDeviceOptions {
    /// Explicit color space.
    pub color_model: VectorColorModel,
    /// Partial-alpha handling; fully transparent paints are always omitted.
    pub alpha: VectorAlphaPolicy,
}
struct Writer {
    data: BoundedString,
    options: VectorDeviceOptions,
}
impl Writer {
    fn add(&mut self, args: std::fmt::Arguments<'_>) -> ChartResult<()> {
        self.data
            .add(args, "PostScript exceeds the output byte budget.")
    }
    fn components(&self, c: usvg::Color) -> Vec<f64> {
        let [r, g, b] = [c.red, c.green, c.blue].map(|v| f64::from(v) / 255.);
        match self.options.color_model {
            VectorColorModel::Rgb
            | VectorColorModel::RgbGray
            | VectorColorModel::Srgb
            | VectorColorModel::SrgbGray => vec![r, g, b],
            VectorColorModel::Gray => vec![0.213 * r + 0.715 * g + 0.072 * b],
            VectorColorModel::Cmyk => {
                let k = 1. - r.max(g).max(b);
                if k == 1. {
                    vec![0., 0., 0., 1.]
                } else {
                    vec![
                        (1. - r - k) / (1. - k),
                        (1. - g - k) / (1. - k),
                        (1. - b - k) / (1. - k),
                        k,
                    ]
                }
            }
        }
    }
    fn color(&mut self, c: usvg::Color) -> ChartResult<()> {
        if matches!(
            self.options.color_model,
            VectorColorModel::RgbGray | VectorColorModel::SrgbGray
        ) && c.red == c.green
            && c.green == c.blue
        {
            return self.add(format_args!("{} setgray\n", f64::from(c.red) / 255.));
        }
        if matches!(
            self.options.color_model,
            VectorColorModel::Srgb | VectorColorModel::SrgbGray
        ) {
            self.add(format_args!("ChartSRGB setcolorspace\n"))?;
        }
        for value in self.components(c) {
            self.add(format_args!("{value} "))?;
        }
        self.add(format_args!(
            "{}\n",
            match self.options.color_model {
                VectorColorModel::Rgb | VectorColorModel::RgbGray => "setrgbcolor",
                VectorColorModel::Srgb | VectorColorModel::SrgbGray => "setcolor",
                VectorColorModel::Gray => "setgray",
                VectorColorModel::Cmyk => "setcmykcolor",
            }
        ))
    }
    fn transform(&mut self, t: usvg::Transform) -> ChartResult<()> {
        self.add(format_args!(
            "[{} {} {} {} {} {}] concat\n",
            t.sx, t.ky, t.kx, t.sy, t.tx, t.ty
        ))
    }
    fn path(&mut self, p: &usvg::tiny_skia_path::Path) -> ChartResult<()> {
        use usvg::tiny_skia_path::PathSegment::*;
        let mut current = (0_f32, 0_f32);
        let mut start = current;
        self.add(format_args!("newpath\n"))?;
        for command in p.segments() {
            match command {
                MoveTo(p) => {
                    self.add(format_args!("{} {} moveto\n", p.x, p.y))?;
                    current = (p.x, p.y);
                    start = current;
                }
                LineTo(p) => {
                    self.add(format_args!("{} {} lineto\n", p.x, p.y))?;
                    current = (p.x, p.y);
                }
                QuadTo(a, b) => {
                    let [a1, a2] = quad_to_cubic((current.0, current.1), (a.x, a.y), (b.x, b.y));
                    self.add(format_args!(
                        "{} {} {} {} {} {} curveto\n",
                        a1.0, a1.1, a2.0, a2.1, b.x, b.y
                    ))?;
                    current = (b.x, b.y);
                }
                CubicTo(a, b, c) => {
                    self.add(format_args!(
                        "{} {} {} {} {} {} curveto\n",
                        a.x, a.y, b.x, b.y, c.x, c.y
                    ))?;
                    current = (c.x, c.y);
                }
                Close => {
                    self.add(format_args!("closepath\n"))?;
                    current = start;
                }
            }
        }
        Ok(())
    }
    fn stroke(&mut self, s: &usvg::Stroke) -> ChartResult<()> {
        self.add(format_args!(
            "{} setlinewidth\n{} setmiterlimit\n{} setlinecap\n{} setlinejoin\n[",
            s.width().get(),
            s.miterlimit().get(),
            match s.linecap() {
                usvg::LineCap::Butt => 0,
                usvg::LineCap::Round => 1,
                usvg::LineCap::Square => 2,
            },
            match s.linejoin() {
                usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => 0,
                usvg::LineJoin::Round => 1,
                usvg::LineJoin::Bevel => 2,
            }
        ))?;
        for v in s.dasharray().unwrap_or_default() {
            self.add(format_args!("{v} "))?;
        }
        self.add(format_args!("] {} setdash\n", s.dashoffset()))
    }
    fn paint(
        &mut self,
        alpha: &mut AlphaPolicy,
        paint: &usvg::Paint,
        stroke: bool,
        even: bool,
    ) -> ChartResult<()> {
        match paint {
            usvg::Paint::Color(c) => {
                self.color(*c)?;
                self.add(format_args!(
                    "{}\n",
                    if stroke {
                        "stroke"
                    } else if even {
                        "eofill"
                    } else {
                        "fill"
                    }
                ))
            }
            usvg::Paint::LinearGradient(g) => self.gradient(
                alpha,
                g,
                Some([g.x1(), g.y1(), g.x2(), g.y2()]),
                None,
                stroke,
                even,
            ),
            usvg::Paint::RadialGradient(g) => self.gradient(
                alpha,
                g,
                None,
                Some([g.fx(), g.fy(), g.fr().get(), g.cx(), g.cy(), g.r().get()]),
                stroke,
                even,
            ),
            usvg::Paint::Pattern(_) => Err(error(
                DiagnosticCode::ExportFidelity,
                "PostScript publication encountered an unqualified pattern paint.",
            )),
        }
    }
    fn gradient(
        &mut self,
        alpha: &mut AlphaPolicy,
        g: &usvg::BaseGradient,
        linear: Option<[f32; 4]>,
        radial: Option<[f32; 6]>,
        stroke: bool,
        even: bool,
    ) -> ChartResult<()> {
        if g.spread_method() != usvg::SpreadMethod::Pad {
            return Err(error(
                DiagnosticCode::ExportFidelity,
                "PostScript gradient requires pad spreading.",
            ));
        }
        for s in g.stops() {
            if !alpha.admit(s.opacity().get())? {
                return Ok(());
            }
        }
        if stroke {
            self.add(format_args!("strokepath\n"))?;
        }
        self.add(format_args!(
            "{} newpath\n",
            if even { "eoclip" } else { "clip" }
        ))?;
        self.transform(g.transform())?;
        self.add(format_args!(
            "<< /ShadingType {} /ColorSpace {} /Coords [",
            if linear.is_some() { 2 } else { 3 },
            match self.options.color_model {
                VectorColorModel::Rgb | VectorColorModel::RgbGray => "/DeviceRGB",
                VectorColorModel::Srgb | VectorColorModel::SrgbGray => "ChartSRGB",
                VectorColorModel::Gray => "/DeviceGray",
                VectorColorModel::Cmyk => "/DeviceCMYK",
            }
        ))?;
        if let Some(coords) = linear {
            for v in coords {
                self.add(format_args!("{v} "))?;
            }
        }
        if let Some(coords) = radial {
            for v in coords {
                self.add(format_args!("{v} "))?;
            }
        }
        self.add(format_args!(
            "] /Extend [true true] /Function << /FunctionType 3 /Domain [0 1] /Functions [\n"
        ))?;
        let mut stops: Vec<_> = g
            .stops()
            .iter()
            .map(|s| (s.offset().get(), s.color()))
            .collect();
        if stops.is_empty() {
            return Err(error(
                DiagnosticCode::ExportFidelity,
                "PostScript gradient has no colors.",
            ));
        }
        if stops[0].0 > 0. {
            stops.insert(0, (0., stops[0].1));
        }
        if stops.last().unwrap().0 < 1. {
            stops.push((1., stops.last().unwrap().1));
        }
        let intervals: Vec<_> = stops
            .windows(2)
            .filter(|pair| pair[0].0 < pair[1].0)
            .collect();
        for pair in &intervals {
            self.add(format_args!("<< /FunctionType 2 /Domain [0 1] /C0 ["))?;
            for value in self.components(pair[0].1) {
                self.add(format_args!("{value} "))?;
            }
            self.add(format_args!("] /C1 ["))?;
            for value in self.components(pair[1].1) {
                self.add(format_args!("{value} "))?;
            }
            self.add(format_args!("] /N 1 >>\n"))?;
        }
        self.add(format_args!("] /Bounds ["))?;
        for pair in &intervals[..intervals.len().saturating_sub(1)] {
            self.add(format_args!("{} ", pair[1].0))?;
        }
        self.add(format_args!("] /Encode ["))?;
        for _ in &intervals {
            self.add(format_args!("0 1 "))?;
        }
        self.add(format_args!("] >> >> shfill\n"))
    }
    fn leaf(&mut self, alpha: &mut AlphaPolicy, leaf: &Leaf<'_>) -> ChartResult<()> {
        self.add(format_args!("gsave\n"))?;
        self.transform(leaf.transform)?;
        self.path(leaf.path)?;
        let (stroke, even) = match leaf.kind {
            LeafPaint::Stroke(s) => {
                self.stroke(s)?;
                (true, false)
            }
            LeafPaint::Fill { even_odd } => (false, even_odd),
        };
        self.paint(alpha, leaf.paint, stroke, even)?;
        self.add(format_args!("grestore\n"))
    }
}

pub(crate) fn postscript_pages(
    pages: &[(&Scene, &usvg::Tree, &PublicationProfile)],
    eps: bool,
) -> ChartResult<(Vec<u8>, usize)> {
    if pages.is_empty() || (eps && pages.len() != 1) {
        return Err(error(
            DiagnosticCode::Validation,
            "PostScript requires at least one page; EPS requires exactly one.",
        ));
    }
    let p = pages[0].2;
    if pages.len() > p.layout.limits.max_items {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "PostScript page count exceeds the publication item budget.",
        ));
    }
    let width = pages.iter().map(|v| v.2.page.width()).fold(0., f64::max);
    let height = pages.iter().map(|v| v.2.page.height()).fold(0., f64::max);
    let mut w = Writer {
        data: BoundedString::new(pages.iter().map(|v| v.2.max_output_bytes).min().unwrap()),
        options: p.vector_device.clone().unwrap_or_default(),
    };
    let mut alpha = AlphaPolicy::new(w.options.alpha, "PostScript");
    w.add(format_args!("%!PS-Adobe-3.0{}\n%%Creator: finstack-chart\n%%LanguageLevel: 3\n%%BoundingBox: 0 0 {} {}\n%%HiResBoundingBox: 0 0 {} {}\n%%Pages: {}\n%%DocumentData: Clean7Bit\n%%EndComments\n",if eps{" EPSF-3.0"}else{""},width.ceil(),height.ceil(),width,height,pages.len()))?;
    w.add(format_args!("/ChartSRGB [/CIEBasedABC << /DecodeABC [{{dup 0.04045 le {{12.92 div}}{{0.055 add 1.055 div 2.4 exp}} ifelse}} bind dup dup] /MatrixABC [0.4124564 0.2126729 0.0193339 0.3575761 0.7151522 0.119192 0.1804375 0.072175 0.9503041] /WhitePoint [0.95047 1 1.08883] /BlackPoint [0 0 0] >>] def\n"))?;
    for (i, (scene, tree, p)) in pages.iter().enumerate() {
        w.options = p.vector_device.clone().unwrap_or_default();
        alpha.policy = w.options.alpha;
        page(&mut w, &mut alpha, scene, tree, p, eps, i + 1)?;
    }
    let omitted = alpha.omitted;
    w.add(format_args!(
        "%%Trailer\n%%ChartOmittedTranslucentPaints: {}\n%%EOF\n",
        omitted
    ))?;
    Ok((w.data.text.into_bytes(), alpha.omitted))
}
fn page(
    w: &mut Writer,
    alpha: &mut AlphaPolicy,
    scene: &Scene,
    tree: &usvg::Tree,
    p: &PublicationProfile,
    eps: bool,
    number: usize,
) -> ChartResult<()> {
    if !eps {
        w.add(format_args!(
            "<< /PageSize [{} {}] >> setpagedevice\n",
            p.page.width(),
            p.page.height()
        ))?;
    }
    w.add(format_args!(
        "%%Page: {} {}\ngsave\n0 {} translate 1 -1 scale\n",
        number,
        number,
        p.page.height()
    ))?;
    for (index, item) in scene.items().iter().enumerate() {
        if crate::snapshot::point_is_clipped(item, scene.bounds()) {
            continue;
        }
        let clip = item.clip.unwrap_or(scene.bounds());
        if clip.width() == 0. || clip.height() == 0. {
            continue;
        }
        w.add(format_args!(
            "gsave\n{} {} {} {} rectclip\n",
            clip.origin().x(),
            clip.origin().y(),
            clip.width(),
            clip.height()
        ))?;
        match &item.primitive {
            Primitive::RasterImage {
                bounds,
                raster,
                interpolate,
                ..
            } => {
                if raster.pixels.iter().all(|c| c.alpha == 255) {
                    w.add(format_args!("gsave {} {} translate {} {} scale /DeviceRGB setcolorspace\n<< /ImageType 1 /Width {} /Height {} /BitsPerComponent 8 /Decode [0 1 0 1 0 1] /Interpolate {} /ImageMatrix [{} 0 0 {} 0 0] /DataSource currentfile /ASCIIHexDecode filter >> image\n",bounds.origin().x(),bounds.origin().y(),bounds.width(),bounds.height(),raster.width,raster.height,interpolate,raster.width,raster.height))?;
                    for color in &raster.pixels {
                        w.add(format_args!(
                            "{:02X}{:02X}{:02X}",
                            color.red, color.green, color.blue
                        ))?;
                    }
                    w.add(format_args!(">\ngrestore\n"))?;
                } else {
                    if *interpolate {
                        return Err(error(
                            DiagnosticCode::ExportFidelity,
                            "PostScript interpolated raster images require opaque pixels, as in the reference device.",
                        ));
                    }
                    // Explicit cells preserve alpha holes and exact nearest-image color without
                    // changing other page marks into raster content.
                    let dx = bounds.width() / raster.width as f64;
                    let dy = bounds.height() / raster.height as f64;
                    for (i, c) in raster.pixels.iter().enumerate() {
                        if !alpha.admit(f32::from(c.alpha) / 255.)? {
                            continue;
                        }
                        w.color(usvg::Color::new_rgb(c.red, c.green, c.blue))?;
                        w.add(format_args!(
                            "{} {} {} {} rectfill\n",
                            bounds.origin().x() + (i % raster.width) as f64 * dx,
                            bounds.origin().y() + (i / raster.width) as f64 * dy,
                            dx,
                            dy
                        ))?;
                    }
                }
            }
            _ => {
                if let Some(node) = tree.node_by_id(&format!("item-{index}")) {
                    crate::devices::walk(node, alpha, &mut |leaf, alpha| w.leaf(alpha, &leaf))?;
                }
            }
        }
        w.add(format_args!("grestore\n"))?;
    }
    w.add(format_args!("grestore\nshowpage\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn writer(model: VectorColorModel) -> Writer {
        Writer {
            data: BoundedString::new(100_000),
            options: VectorDeviceOptions {
                color_model: model,
                ..Default::default()
            },
        }
    }
    fn walk_all(w: &mut Writer, root: &usvg::Group) -> ChartResult<()> {
        let mut alpha = AlphaPolicy::new(w.options.alpha, "PostScript");
        crate::devices::walk_group(root, &mut alpha, &mut |leaf, alpha| w.leaf(alpha, &leaf))
    }
    #[test]
    fn pinned_reference_color_models_keep_calibration_and_neutral_policy() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/parity/ggplot2/postscript-controls.json"
        ))
        .unwrap();
        assert_eq!(fixture["cases"].as_array().unwrap().len(), 6);
        let c = usvg::Color::new_rgb(128, 64, 32);
        let gray = writer(VectorColorModel::Gray).components(c)[0];
        assert!((gray - 0.2954).abs() < 0.00005);
        assert_eq!(
            writer(VectorColorModel::Cmyk).components(c),
            vec![0., 0.5, 0.75, 127. / 255.]
        );
        for model in [VectorColorModel::SrgbGray, VectorColorModel::RgbGray] {
            let mut w = writer(model);
            w.color(usvg::Color::new_rgb(128, 128, 128)).unwrap();
            assert!(w.data.text.ends_with(" setgray\n"));
        }
        let mut w = writer(VectorColorModel::Srgb);
        w.color(c).unwrap();
        assert!(w.data.text.starts_with("ChartSRGB setcolorspace\n"));
    }
    #[test]
    fn quadratic_paths_and_strokes_remain_vector_commands() {
        let tree=usvg::Tree::from_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><path d="M0 0 Q3 6 6 0" fill="none" stroke="red" stroke-width="2" stroke-linecap="round" stroke-linejoin="bevel" stroke-dasharray="3 2"/></svg>"#,&usvg::Options::default()).unwrap();
        let mut w = writer(VectorColorModel::Rgb);
        walk_all(&mut w, tree.root()).unwrap();
        assert!(w.data.text.contains("2 4 4 4 6 0 curveto"));
        assert!(w.data.text.contains("2 setlinewidth"));
        assert!(w.data.text.contains("1 setlinecap"));
        assert!(w.data.text.contains("2 setlinejoin"));
        assert!(w.data.text.contains("[3 2 ] 0 setdash"));
        assert!(!w.data.text.contains("image"));
    }
    #[test]
    fn gradient_shading_preserves_resolved_stop_intervals_and_endpoint_padding() {
        let tree=usvg::Tree::from_str(r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><linearGradient id="g"><stop offset=".2" stop-color="red"/><stop offset=".5" stop-color="red"/><stop offset=".5" stop-color="blue"/><stop offset=".8" stop-color="blue"/></linearGradient></defs><rect width="100" height="100" fill="url(#g)"/></svg>"##,&usvg::Options::default()).unwrap();
        let mut w = writer(VectorColorModel::Rgb);
        walk_all(&mut w, tree.root()).unwrap();
        assert!(w.data.text.contains("/ShadingType 2"));
        assert!(
            w.data.text.contains("/Bounds [0.2 0.49999988 0.5 0.8 ]"),
            "{}",
            w.data.text
        );
        assert!(w.data.text.contains("clip newpath"));
        assert_eq!(w.data.text.matches("/FunctionType 2").count(), 5);
    }
    #[test]
    fn unsupported_alpha_is_explicit_and_writer_budget_prevents_growth() {
        let mut w = writer(VectorColorModel::Rgb);
        let mut alpha = AlphaPolicy::new(w.options.alpha, "PostScript");
        assert_eq!(
            alpha.admit(0.5).unwrap_err().code,
            DiagnosticCode::ExportFidelity
        );
        assert!(!alpha.admit(0.).unwrap());
        alpha.policy = VectorAlphaPolicy::OmitTranslucent;
        assert!(!alpha.admit(0.5).unwrap());
        w.data.limit = 2;
        assert_eq!(
            w.add(format_args!("long")).unwrap_err().code,
            DiagnosticCode::ResourceLimit
        );
        assert!(w.data.text.len() <= 2);
    }
}
