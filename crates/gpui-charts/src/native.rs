use chart_core::grammar::PreparedChart;
use chart_core::layout::{LaidOutChart, LayoutRequest, layout};
use chart_core::scene::{Color, PathCommand, Primitive};
use chart_core::services::{
    ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
};
use chart_core::{ChartResult, Diagnostic, DiagnosticCode, Limits, Point, Rect};
use gpui::{
    App, Bounds, ContentMask, Font, Path, PathBuilder, Pixels, ShapedLine, TextAlign, TextRun,
    Window, WindowTextSystem, fill, point, px, rgba, size,
};
use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

pub(crate) fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Supply a supported finite chart and the exact registered font resource, then prepare again.",
    )
}
/// Explicit immutable font loaded once into the host text registry and shared across charts.
/// Register outside render/mount loops; GPUI retains its process-level font cache.
#[derive(Clone)]
pub struct NativeFont {
    pub(crate) descriptor: ResourceDescriptor,
    bytes: Arc<[u8]>,
    font: Font,
    pub(crate) faces: BTreeMap<chart_core::ResourceId, (ResourceDescriptor, Arc<[u8]>)>,
}
#[derive(Default)]
struct FontRegistry(BTreeMap<String, NativeFont>);
impl gpui::Global for FontRegistry {}

impl NativeFont {
    /// Load supplied font bytes with an automatically allocated native resource identity.
    /// Reusing the same bytes/family returns the existing registered face.
    pub fn from_bytes(
        bytes: impl Into<Arc<[u8]>>,
        family: &str,
        cx: &mut App,
    ) -> ChartResult<Self> {
        let bytes = bytes.into();
        if let Some(existing) = cx.try_global::<FontRegistry>().and_then(|r| {
            r.0.values()
                .find(|font| font.bytes == bytes && font.font.family.as_ref() == family)
        }) {
            return Ok(existing.clone());
        }
        let id = cx
            .try_global::<FontRegistry>()
            .and_then(|r| r.0.values().map(|f| f.descriptor.id.get()).max())
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::RevisionOverflow,
                    "Native font resource identity is exhausted.",
                )
            })?;
        Self::load(
            ResourceDescriptor {
                id: chart_core::ResourceId::new(id),
                revision: chart_core::Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: bytes.len() as u64,
            },
            bytes,
            family,
            cx,
        )
    }

    /// Parse/check an explicit face, verify its family, then register these supplied bytes.
    /// Register before the host first resolves this family; GPUI caches resolved font selections.
    /// The host must reserve supplied faces in this family for this adapter. Reloading the
    /// same descriptor/bytes is idempotent; a different resource under the family rejects.
    /// No system-font lookup or silent missing-glyph substitution is requested by this adapter.
    pub fn load(
        descriptor: ResourceDescriptor,
        bytes: Arc<[u8]>,
        family: &str,
        cx: &mut App,
    ) -> ChartResult<Self> {
        let result = (|| {
            if descriptor.kind != ResourceKind::Font
                || descriptor.byte_len != bytes.len() as u64
                || bytes.is_empty()
            {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "Font descriptor and bytes disagree.",
                ));
            }
            if descriptor.byte_len > Limits::default().max_resource_bytes {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Native font exceeds the resource byte budget.",
                ));
            }
            let face = ttf_parser::Face::parse(&bytes, 0).map_err(|e| {
                error(
                    DiagnosticCode::InvalidResource,
                    format!("Font parsing failed: {e:?}"),
                )
            })?;
            if !face.names().into_iter().any(|n| {
                n.name_id == ttf_parser::name_id::FAMILY && n.to_string().as_deref() == Some(family)
            }) {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "The supplied font face does not match the requested family.",
                ));
            }
            let weight = face.weight().to_number();
            let italic = face.is_italic();
            let registry_key = format!("{family}:{weight}:{italic}");
            if let Some(existing) = cx
                .try_global::<FontRegistry>()
                .and_then(|r| r.0.get(&registry_key))
            {
                if existing.descriptor == descriptor && existing.bytes == bytes {
                    return Ok(existing.clone());
                }
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "This native family already names another resource; use a distinct font family.",
                ));
            }
            cx.text_system()
                .add_fonts(vec![Cow::Owned(bytes.to_vec())])
                .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
            if !cx
                .text_system()
                .all_font_names()
                .iter()
                .any(|s| s == family)
            {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    "Registered font family is unavailable in the native text system.",
                ));
            }
            let mut native = gpui::font(family.to_owned());
            native.weight = gpui::FontWeight(f32::from(weight));
            if italic {
                native.style = gpui::FontStyle::Italic;
            }
            let font = Self {
                descriptor,
                faces: BTreeMap::from([(descriptor.id, (descriptor, bytes.clone()))]),
                bytes,
                font: native,
            };
            cx.default_global::<FontRegistry>()
                .0
                .insert(registry_key, font.clone());
            Ok(font)
        })();
        result.map_err(|mut e| {
            e.context.resource = Some(descriptor.id);
            e.context.resource_revision = Some(descriptor.revision);
            e
        })
    }
    /// Exact core resource identity/revision used for measurement and painting.
    pub fn descriptor(&self) -> ResourceDescriptor {
        self.descriptor
    }
    /// Add an explicitly loaded face for rich runs and deliberate fallback. Duplicate identities
    /// must have identical descriptors/bytes; no system family lookup is added.
    pub fn with_face(mut self, other: &Self) -> ChartResult<Self> {
        for (id, (descriptor, bytes)) in &other.faces {
            if let Some((d, b)) = self.faces.get(id)
                && (d != descriptor || b != bytes)
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Native rich face identity has conflicting bytes/revision.",
                ));
            }
            self.faces.insert(*id, (*descriptor, bytes.clone()));
        }
        if self.faces.len() > Limits::default().max_resources
            || self.faces.values().map(|(d, _)| d.byte_len).sum::<u64>()
                > Limits::default().max_total_resource_bytes
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Native rich font bank exceeds resource limits.",
            ));
        }
        Ok(self)
    }
    fn shape(
        &self,
        text: &str,
        font_size: f64,
        color: Color,
        opacity: f32,
        system: &WindowTextSystem,
    ) -> ChartResult<ShapedLine> {
        let face = ttf_parser::Face::parse(&self.bytes, 0).map_err(|e| {
            error(
                DiagnosticCode::InvalidResource,
                format!("Font parsing failed: {e:?}"),
            )
        })?;
        if text
            .chars()
            .any(|c| c.is_control() || face.glyph_index(c).is_none())
        {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Plain chart text contains a control character or a glyph absent from the declared font.",
            ));
        }
        let run = TextRun {
            len: text.len(),
            font: self.font.clone(),
            color: {
                let mut c = native_color(color);
                c.a *= opacity;
                c.into()
            },
            ..Default::default()
        };
        let line = system.shape_line(text.to_owned().into(), pixel(font_size)?, &[run], None);
        let requested = system.resolve_font(&self.font);
        if line.runs.iter().any(|r| r.font_id != requested) {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Native shaping substituted a different font; implicit fallback is not supported.",
            ));
        }
        Ok(line)
    }
}
pub(crate) fn pixel(value: f64) -> ChartResult<Pixels> {
    let v = value as f32;
    if !v.is_finite() || (f64::from(v) - value).abs() > 0.25 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Native binary32 projection exceeds a quarter logical pixel.",
        ));
    }
    Ok(px(v))
}
pub(crate) fn native_color(c: Color) -> gpui::Rgba {
    rgba(
        (u32::from(c.red) << 24)
            | (u32::from(c.green) << 16)
            | (u32::from(c.blue) << 8)
            | u32::from(c.alpha),
    )
}
struct Metrics<'a> {
    font: &'a NativeFont,
    system: &'a WindowTextSystem,
}
impl TextMeasurer for Metrics<'_> {
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> ChartResult<chart_core::typography::ShapedRun> {
        if r.units != Units::LogicalPixels {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Native rich shaping requires logical pixels.",
            ));
        }
        r.run.validate(r.limits)?;
        let primary = r.run.font.as_ref().unwrap_or(r.default_font);
        for (index, descriptor) in std::iter::once(primary).chain(&r.run.fallback).enumerate() {
            let (_, bytes) = self
                .font
                .faces
                .get(&descriptor.id)
                .filter(|(d, _)| d == descriptor)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Native rich run references an unregistered exact font revision.",
                    )
                })?;
            if chart_text::supports(bytes, &r.run.text, r.run.weight)? {
                return chart_text::shape(bytes, *descriptor, r, index > 0);
            }
        }
        Err(error(
            DiagnosticCode::MissingResource,
            "No declared native face supplies the complete rich run/weight.",
        ))
    }

    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        if *r.font != self.font.descriptor {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Native layout requires the exact registered font revision.",
            ));
        }
        if r.units != Units::LogicalPixels {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Native layout requires logical pixel units.",
            ));
        }
        let line = self.font.shape(
            r.text,
            r.font_size,
            Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            1.,
            self.system,
        )?;
        TextMetrics::new(
            f64::from(f32::from(line.width)),
            f64::from(f32::from(line.ascent)),
            f64::from(f32::from(line.descent)),
        )
    }
}
fn sampled_gradient_image(
    direction: chart_core::scene::GradientDirection,
    colors: &[Color],
) -> ChartResult<Arc<gpui::RenderImage>> {
    let count = u32::try_from(colors.len().checked_add(2).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Gradient sample size overflow.",
        )
    })?)
    .map_err(|_| {
        error(
            DiagnosticCode::ResourceLimit,
            "Gradient sample count exceeds native image dimensions.",
        )
    })?;
    if colors.len() < 2 {
        return Err(error(
            DiagnosticCode::Validation,
            "A sampled gradient needs two colors.",
        ));
    }
    let (width, height) = match direction {
        chart_core::scene::GradientDirection::Horizontal => (count, 3),
        chart_core::scene::GradientDirection::Vertical => (3, count),
    };
    // Duplicate every edge pixel. GPUI filters inside a shared atlas; without
    // our own border, an expanded one-pixel strip samples neighboring images.
    let mut bytes = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let i = match direction {
                chart_core::scene::GradientDirection::Horizontal => x,
                chart_core::scene::GradientDirection::Vertical => y,
            };
            let c = colors[(i.saturating_sub(1) as usize).min(colors.len() - 1)];
            bytes.extend_from_slice(&[c.blue, c.green, c.red, c.alpha]);
        }
    }
    let image = image::RgbaImage::from_raw(width, height, bytes).ok_or_else(|| {
        error(
            DiagnosticCode::InvalidResource,
            "Invalid sampled gradient image size.",
        )
    })?;
    Ok(Arc::new(gpui::RenderImage::new(vec![image::Frame::new(
        image,
    )])))
}

/// An exact union of same-RGBA source pixels. Only the previous scanline's
/// runs stay in the lookup map; both map and output are bounded by raster cells.
#[derive(Clone, Copy, Debug)]
struct RasterRectangle {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    color: Color,
}
impl RasterRectangle {
    fn bounds(self, bounds: Rect, width: usize, height: usize) -> ChartResult<Rect> {
        let x0 = bounds.origin().x() + bounds.width() * self.x0 as f64 / width as f64;
        let x1 = bounds.origin().x() + bounds.width() * self.x1 as f64 / width as f64;
        let y0 = bounds.origin().y() + bounds.height() * self.y0 as f64 / height as f64;
        let y1 = bounds.origin().y() + bounds.height() * self.y1 as f64 / height as f64;
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
}
fn nearest_raster_rectangles(
    raster: &chart_core::grammar::RasterAnnotation,
) -> Vec<RasterRectangle> {
    let mut rectangles: Vec<RasterRectangle> = Vec::new();
    let mut previous: BTreeMap<(usize, usize, [u8; 4]), usize> = BTreeMap::new();
    for y in 0..raster.height {
        let mut current = BTreeMap::new();
        let mut x = 0;
        while x < raster.width {
            let color = raster.pixels[y * raster.width + x];
            let x0 = x;
            x += 1;
            while x < raster.width && raster.pixels[y * raster.width + x] == color {
                x += 1;
            }
            if color.alpha == 0 {
                continue;
            }
            let key = (x0, x, [color.red, color.green, color.blue, color.alpha]);
            let index = if let Some(&index) = previous.get(&key) {
                rectangles[index].y1 = y + 1;
                index
            } else {
                let index = rectangles.len();
                rectangles.push(RasterRectangle {
                    x0,
                    x1: x,
                    y0: y,
                    y1: y + 1,
                    color,
                });
                index
            };
            current.insert(key, index);
        }
        previous = current;
    }
    rectangles
}

fn raster_image(
    raster: &chart_core::grammar::RasterAnnotation,
) -> ChartResult<Arc<gpui::RenderImage>> {
    let width = u32::try_from(
        raster
            .width
            .checked_add(2)
            .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "Raster width overflow."))?,
    )
    .map_err(|_| error(DiagnosticCode::ResourceLimit, "Raster width overflow."))?;
    let height = u32::try_from(
        raster
            .height
            .checked_add(2)
            .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "Raster height overflow."))?,
    )
    .map_err(|_| error(DiagnosticCode::ResourceLimit, "Raster height overflow."))?;
    let mut bytes = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let row = (y.saturating_sub(1) as usize).min(raster.height - 1);
            let col = (x.saturating_sub(1) as usize).min(raster.width - 1);
            let c = raster.pixels[row * raster.width + col];
            bytes.extend_from_slice(&[c.blue, c.green, c.red, c.alpha]);
        }
    }
    let image = image::RgbaImage::from_raw(width, height, bytes)
        .ok_or_else(|| error(DiagnosticCode::InvalidResource, "Invalid raster size."))?;
    Ok(Arc::new(gpui::RenderImage::new(vec![image::Frame::new(
        image,
    )])))
}

enum Paint {
    Custom(std::rc::Rc<dyn crate::PreparedNativePaint>),
    Empty,
    Quad(gpui::PaintQuad),
    Quads(Vec<gpui::PaintQuad>),
    Image(Bounds<Pixels>, Bounds<Pixels>, Arc<gpui::RenderImage>),
    Path(Path<Pixels>, gpui::Rgba),
    Paths(Vec<(Path<Pixels>, gpui::Rgba)>),
    Text(Box<ShapedLine>, gpui::Point<Pixels>),
}
struct Item {
    clip: Bounds<Pixels>,
    paint: Paint,
}
pub(crate) struct NativeFrame {
    pub request: LayoutRequest,
    pub fonts: Vec<(ResourceDescriptor, Arc<[u8]>)>,
    pub job: Option<chart_core::scheduling::JobToken>,
    pub density: chart_core::dense::DensityMetrics,
    pub chart: Arc<LaidOutChart>,
    pub bounds: Bounds<Pixels>,
    font: NativeFont,
    painters: crate::NativePainterRegistry,
    density_options: chart_core::dense::DensityOptions,
    items: Vec<Item>,
}
fn point_at(p: Point, offset: gpui::Point<Pixels>) -> ChartResult<gpui::Point<Pixels>> {
    Ok(point(
        pixel(p.x() + f64::from(f32::from(offset.x)))?,
        pixel(p.y() + f64::from(f32::from(offset.y)))?,
    ))
}
fn rect_at(r: Rect, offset: gpui::Point<Pixels>) -> ChartResult<Bounds<Pixels>> {
    Ok(Bounds::new(
        point_at(r.origin(), offset)?,
        size(pixel(r.width())?, pixel(r.height())?),
    ))
}
impl NativeFrame {
    pub fn prepare(
        prepared: Arc<PreparedChart>,
        mut request: LayoutRequest,
        font: &NativeFont,
        painters: &crate::NativePainterRegistry,
        density: &chart_core::dense::DensityOptions,
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> ChartResult<Self> {
        request.device_scale = Some(f64::from(window.scale_factor()));
        request.bounds = Rect::new(
            0.,
            0.,
            f64::from(f32::from(bounds.size.width)),
            f64::from(f32::from(bounds.size.height)),
        )?;
        let chart = Arc::new(layout(
            prepared,
            &request,
            &Metrics {
                font,
                system: window.text_system(),
            },
        )?);
        Self::from_layout(chart, request, font, painters, density, bounds, window)
    }
    pub fn from_layout(
        chart: Arc<chart_core::layout::LaidOutChart>,
        request: LayoutRequest,
        font: &NativeFont,
        painters: &crate::NativePainterRegistry,
        density: &chart_core::dense::DensityOptions,
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> ChartResult<Self> {
        let reduced =
            chart_core::dense::DenseChart::prepare(chart.clone(), density, request.limits)?;
        let mut items = vec![];
        for item in reduced.scene().items() {
            let opacity = item
                .guide
                .as_ref()
                .and_then(|g| g.animation)
                .map_or(1., |a| a.opacity as f32);
            let native_color = |c: Color| {
                let mut rgba = native_color(c);
                rgba.a *= opacity;
                rgba
            };
            let result = (|| {
                let clip = rect_at(item.clip.unwrap_or(chart.scene().bounds()), bounds.origin)?;
                let outlined = if let Primitive::GlyphRun {
                    origin,
                    rotation,
                    run,
                    color,
                } = &item.primitive
                {
                    if self_font_missing(font, &run.font) {
                        return Err(error(
                            DiagnosticCode::MissingResource,
                            "Native rich scene has an unregistered face revision.",
                        ));
                    }
                    Some(Primitive::FilledPath {
                        commands: chart_core::typography::placed_outlines(run, *origin, *rotation)?,
                        fill: *color,
                    })
                } else if let Primitive::Symbol {
                    center,
                    radius,
                    kind,
                    fill,
                } = &item.primitive
                {
                    Some(Primitive::FilledPath {
                        commands: chart_core::scene::symbol_path(*center, *radius, *kind)?,
                        fill: *fill,
                    })
                } else if let Primitive::DashedPath {
                    commands,
                    stroke,
                    dashes,
                } = &item.primitive
                {
                    Some(Primitive::Path {
                        commands: chart_core::scene::dash_polyline(
                            commands,
                            dashes,
                            request.limits.max_path_commands,
                        )?,
                        stroke: *stroke,
                    })
                } else {
                    None
                };
                let primitive = outlined.as_ref().unwrap_or(&item.primitive);
                let paint = match primitive {
                    Primitive::VectorPath {
                        geometry,
                        fill,
                        stroke,
                        dashes,
                    }
                    | Primitive::ShapePath {
                        geometry,
                        fill,
                        stroke,
                        dashes,
                        ..
                    } => {
                        if !geometry.has_segments() {
                            Paint::Empty
                        } else {
                            let tolerance = 0.0625 / f64::from(window.scale_factor());
                            let commands =
                                geometry.lower(tolerance, request.limits.max_path_commands)?;
                            let dashed = if !dashes.is_empty() && stroke.is_some() {
                                Some(geometry.dashed(
                                    dashes,
                                    tolerance,
                                    request.limits.max_path_commands,
                                )?)
                            } else {
                                None
                            };
                            let mut paths = vec![];
                            for (color, width) in fill
                                .iter()
                                .map(|c| (*c, None))
                                .chain(stroke.iter().map(|s| (s.color, Some(s.width))))
                            {
                                let mut path = match width {
                                    Some(w) => PathBuilder::stroke(pixel(w)?).with_style(
                                        gpui::PathStyle::Stroke(
                                            gpui::StrokeOptions::default()
                                                .with_line_width(f32::from(pixel(w)?))
                                                .with_tolerance(tolerance as f32),
                                        ),
                                    ),
                                    None => PathBuilder::fill().with_style(gpui::PathStyle::Fill(
                                        gpui::FillOptions::default()
                                            .with_fill_rule(
                                                if matches!(
                                                    primitive,
                                                    Primitive::ShapePath {
                                                        fill_rule:
                                                            chart_core::scene::FillRule::EvenOdd,
                                                        ..
                                                    }
                                                ) {
                                                    gpui::FillRule::EvenOdd
                                                } else {
                                                    gpui::FillRule::NonZero
                                                },
                                            )
                                            .with_tolerance(tolerance as f32),
                                    )),
                                };
                                let at = |p| retained_point_at(p, bounds.origin, tolerance);
                                let commands = if width.is_some() {
                                    dashed.as_ref().unwrap_or(&commands)
                                } else {
                                    &commands
                                };
                                for c in commands {
                                    match *c {
                                        PathCommand::MoveTo(p) => path.move_to(at(p)?),
                                        PathCommand::LineTo(p) => path.line_to(at(p)?),
                                        PathCommand::QuadraticTo(a, b) => {
                                            path.curve_to(at(b)?, at(a)?)
                                        }
                                        PathCommand::CubicTo(a, b, c) => {
                                            path.cubic_bezier_to(at(c)?, at(a)?, at(b)?)
                                        }
                                        PathCommand::Close => path.close(),
                                    }
                                }
                                paths.push((
                                    path.build().map_err(|e| {
                                        error(DiagnosticCode::Validation, e.to_string())
                                    })?,
                                    native_color(color),
                                ));
                            }
                            Paint::Paths(paths)
                        }
                    }
                    Primitive::NativePaint {
                        bounds: r,
                        painter,
                        parameters,
                        fill,
                    } => Paint::Custom(painters.get(painter)?.prepare(
                        rect_at(*r, bounds.origin)?,
                        parameters,
                        *fill,
                    )?),
                    Primitive::GlyphRun { .. }
                    | Primitive::DashedPath { .. }
                    | Primitive::Symbol { .. } => unreachable!(),
                    Primitive::RasterImage {
                        bounds: r,
                        raster,
                        interpolate,
                        ..
                    } => {
                        if *interpolate {
                            let px = r.width() / raster.width as f64;
                            let py = r.height() / raster.height as f64;
                            let expanded = Rect::new(
                                r.origin().x() - px,
                                r.origin().y() - py,
                                r.width() + 2. * px,
                                r.height() + 2. * py,
                            )?;
                            Paint::Image(
                                rect_at(*r, bounds.origin)?,
                                rect_at(expanded, bounds.origin)?,
                                raster_image(raster)?,
                            )
                        } else {
                            let quads = nearest_raster_rectangles(raster)
                                .into_iter()
                                .map(|cell| {
                                    Ok(fill(
                                        rect_at(
                                            cell.bounds(*r, raster.width, raster.height)?,
                                            bounds.origin,
                                        )?,
                                        native_color(cell.color),
                                    ))
                                })
                                .collect::<ChartResult<Vec<_>>>()?;
                            Paint::Quads(quads)
                        }
                    }
                    Primitive::SampledGradientRectangle {
                        bounds: r,
                        direction,
                        colors,
                        mode,
                    } => {
                        if *mode == chart_core::scene::SampledGradientMode::Steps {
                            let quads = colors
                                .iter()
                                .enumerate()
                                .map(|(i, color)| {
                                    let start = i as f64 / colors.len() as f64;
                                    let end = (i + 1) as f64 / colors.len() as f64;
                                    let cell = match direction {
                                        chart_core::scene::GradientDirection::Horizontal => {
                                            Rect::new(
                                                r.origin().x() + start * r.width(),
                                                r.origin().y(),
                                                (end - start) * r.width(),
                                                r.height(),
                                            )?
                                        }
                                        chart_core::scene::GradientDirection::Vertical => {
                                            Rect::new(
                                                r.origin().x(),
                                                r.origin().y() + start * r.height(),
                                                r.width(),
                                                (end - start) * r.height(),
                                            )?
                                        }
                                    };
                                    Ok(fill(rect_at(cell, bounds.origin)?, native_color(*color)))
                                })
                                .collect::<ChartResult<Vec<_>>>()?;
                            Paint::Quads(quads)
                        } else {
                            let count = colors.len() as f64;
                            let axis_padding = match mode {
                                chart_core::scene::SampledGradientMode::CellCenters => 1. / count,
                                chart_core::scene::SampledGradientMode::Endpoints => {
                                    1.5 / (count - 1.)
                                }
                                chart_core::scene::SampledGradientMode::Steps => unreachable!(),
                            };
                            let (pad_x, pad_y) = match direction {
                                chart_core::scene::GradientDirection::Horizontal => {
                                    (r.width() * axis_padding, r.height())
                                }
                                chart_core::scene::GradientDirection::Vertical => {
                                    (r.width(), r.height() * axis_padding)
                                }
                            };
                            let image_bounds = Rect::new(
                                r.origin().x() - pad_x,
                                r.origin().y() - pad_y,
                                r.width() + 2. * pad_x,
                                r.height() + 2. * pad_y,
                            )?;
                            Paint::Image(
                                rect_at(*r, bounds.origin)?,
                                rect_at(image_bounds, bounds.origin)?,
                                sampled_gradient_image(*direction, colors)?,
                            )
                        }
                    }
                    Primitive::GradientRectangle {
                        bounds: r,
                        gradient,
                    } => Paint::Quad(fill(
                        rect_at(*r, bounds.origin)?,
                        gpui::linear_gradient(
                            match gradient.direction {
                                chart_core::scene::GradientDirection::Horizontal => 90.,
                                chart_core::scene::GradientDirection::Vertical => 180.,
                            },
                            gpui::linear_color_stop(native_color(gradient.start), 0.),
                            gpui::linear_color_stop(native_color(gradient.end), 1.),
                        )
                        .color_space(gpui::ColorSpace::Srgb),
                    )),

                    Primitive::Rectangle {
                        bounds: r,
                        fill: color,
                    } => Paint::Quad(fill(rect_at(*r, bounds.origin)?, native_color(*color))),
                    Primitive::Point {
                        center,
                        radius,
                        fill: color,
                    } => Paint::Quad(
                        fill(
                            rect_at(
                                Rect::new(
                                    center.x() - radius,
                                    center.y() - radius,
                                    2. * radius,
                                    2. * radius,
                                )?,
                                bounds.origin,
                            )?,
                            native_color(*color),
                        )
                        .corner_radii(pixel(*radius)?),
                    ),
                    Primitive::Text {
                        origin,
                        text,
                        font: font_id,
                        font_size,
                        color,
                    } => {
                        if *font_id != font.descriptor.id {
                            return Err(error(
                                DiagnosticCode::MissingResource,
                                "Scene text references another font.",
                            ));
                        }
                        let line =
                            font.shape(text, *font_size, *color, opacity, window.text_system())?;
                        let mut origin = point_at(*origin, bounds.origin)?;
                        origin.y -= line.ascent;
                        Paint::Text(Box::new(line), origin)
                    }
                    Primitive::Rule { from, to, stroke } => {
                        let mut path = PathBuilder::stroke(pixel(stroke.width)?);
                        path.move_to(point_at(*from, bounds.origin)?);
                        path.line_to(point_at(*to, bounds.origin)?);
                        Paint::Path(
                            path.build()
                                .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?,
                            native_color(stroke.color),
                        )
                    }
                    Primitive::FilledPath { commands, .. } | Primitive::Path { commands, .. }
                        if commands.is_empty() =>
                    {
                        Paint::Empty
                    }
                    Primitive::Path { commands, .. } | Primitive::FilledPath { commands, .. } => {
                        let (mut path, color) = match primitive {
                            Primitive::Path { stroke, .. } => {
                                (PathBuilder::stroke(pixel(stroke.width)?), stroke.color)
                            }
                            Primitive::FilledPath { fill, .. } => (
                                PathBuilder::fill().with_style(gpui::PathStyle::Fill(
                                    gpui::FillOptions::default()
                                        .with_fill_rule(gpui::FillRule::NonZero),
                                )),
                                *fill,
                            ),
                            _ => unreachable!(),
                        };
                        for c in commands {
                            match c {
                                PathCommand::MoveTo(p) => {
                                    path.move_to(point_at(*p, bounds.origin)?)
                                }
                                PathCommand::LineTo(p) => {
                                    path.line_to(point_at(*p, bounds.origin)?)
                                }
                                PathCommand::QuadraticTo(a, b) => path.curve_to(
                                    point_at(*b, bounds.origin)?,
                                    point_at(*a, bounds.origin)?,
                                ),
                                PathCommand::CubicTo(a, b, c) => path.cubic_bezier_to(
                                    point_at(*c, bounds.origin)?,
                                    point_at(*a, bounds.origin)?,
                                    point_at(*b, bounds.origin)?,
                                ),
                                PathCommand::Close => path.close(),
                            }
                        }
                        Paint::Path(
                            path.build()
                                .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?,
                            native_color(color),
                        )
                    }
                };
                Ok(Item { clip, paint })
            })()
            .map_err(|mut e: Diagnostic| {
                e.context.layer = item.layer;
                e.context.stamp = Some(chart.scene().stamp());
                e
            })?;
            items.push(result);
        }
        Ok(Self {
            request,
            fonts: font.faces.values().cloned().collect(),
            job: None,
            density: reduced.metrics().clone(),
            chart,
            bounds,
            font: font.clone(),
            painters: painters.clone(),
            density_options: density.clone(),
            items,
        })
    }
    /// Reproject the same frozen semantic input using its captured resources/style.
    pub fn reproject(
        &self,
        bounds: Bounds<Pixels>,
        revision: chart_core::Revision,
        window: &Window,
    ) -> ChartResult<Self> {
        let mut request = self.request.clone();
        request.revision = revision;
        Self::prepare(
            self.chart.prepared().clone(),
            request,
            &self.font,
            &self.painters,
            &self.density_options,
            bounds,
            window,
        )
    }
    pub fn paint(&self, window: &mut Window, cx: &mut App) -> ChartResult<()> {
        window.with_content_mask(
            Some(ContentMask {
                bounds: self.bounds,
            }),
            |window| {
                for item in &self.items {
                    window.with_content_mask(
                        Some(ContentMask { bounds: item.clip }),
                        |window| -> ChartResult<()> {
                            match &item.paint {
                                Paint::Custom(p) => p.paint(window, cx),
                                Paint::Empty => {}
                                Paint::Quad(q) => window.paint_quad(q.clone()),
                                Paint::Quads(quads) => {
                                    for q in quads {
                                        window.paint_quad(q.clone());
                                    }
                                }
                                Paint::Image(bounds, image_bounds, image) => window
                                    .paint_image(
                                        *bounds,
                                        *image_bounds,
                                        Default::default(),
                                        image.clone(),
                                        0,
                                        false,
                                    )
                                    .map_err(|e| {
                                        error(DiagnosticCode::InvalidResource, e.to_string())
                                    })?,
                                Paint::Path(p, c) => window.paint_path(p.clone(), *c),
                                Paint::Paths(paths) => {
                                    for (p, c) in paths {
                                        window.paint_path(p.clone(), *c);
                                    }
                                }
                                Paint::Text(line, p) => line
                                    .paint(
                                        *p,
                                        line.ascent + line.descent,
                                        TextAlign::Left,
                                        None,
                                        window,
                                        cx,
                                    )
                                    .map_err(|e| {
                                        error(DiagnosticCode::InvalidResource, e.to_string())
                                    })?,
                            };
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            },
        )
    }
}

fn retained_point_at(
    p: Point,
    offset: gpui::Point<Pixels>,
    tolerance: f64,
) -> ChartResult<gpui::Point<Pixels>> {
    let result = point_at(p, offset)?;
    for (value, rounded) in [
        (p.x() + f64::from(f32::from(offset.x)), result.x),
        (p.y() + f64::from(f32::from(offset.y)), result.y),
    ] {
        if (value - f64::from(f32::from(rounded))).abs() > tolerance / std::f64::consts::SQRT_2 {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Retained path exceeds its physical pixel precision.",
            ));
        }
    }
    Ok(result)
}

fn self_font_missing(font: &NativeFont, descriptor: &ResourceDescriptor) -> bool {
    font.faces
        .get(&descriptor.id)
        .is_none_or(|(d, _)| d != descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn verify_raster_rectangles(raster: &chart_core::grammar::RasterAnnotation) -> usize {
        let rectangles = nearest_raster_rectangles(raster);
        let mut reconstructed = vec![None; raster.pixels.len()];
        let bounds = Rect::new(7.25, -3.5, 317.125, 121.75).unwrap();
        for cell in &rectangles {
            assert!(cell.color.alpha > 0);
            assert!(cell.x0 < cell.x1 && cell.x1 <= raster.width);
            assert!(cell.y0 < cell.y1 && cell.y1 <= raster.height);
            let actual = cell.bounds(bounds, raster.width, raster.height).unwrap();
            assert_eq!(
                actual.origin().x(),
                bounds.origin().x() + bounds.width() * cell.x0 as f64 / raster.width as f64
            );
            assert_eq!(
                actual.origin().y(),
                bounds.origin().y() + bounds.height() * cell.y0 as f64 / raster.height as f64
            );
            assert!(
                (actual.max_x()
                    - (bounds.origin().x()
                        + bounds.width() * cell.x1 as f64 / raster.width as f64))
                    .abs()
                    < 1e-12
            );
            assert!(
                (actual.max_y()
                    - (bounds.origin().y()
                        + bounds.height() * cell.y1 as f64 / raster.height as f64))
                    .abs()
                    < 1e-12
            );
            for y in cell.y0..cell.y1 {
                for x in cell.x0..cell.x1 {
                    assert!(
                        reconstructed[y * raster.width + x]
                            .replace(cell.color)
                            .is_none(),
                        "overlap"
                    );
                }
            }
        }
        for (actual, expected) in reconstructed.iter().zip(&raster.pixels) {
            assert_eq!(*actual, (expected.alpha > 0).then_some(*expected));
        }
        rectangles.len()
    }
    #[test]
    fn nearest_raster_coalescing_preserves_holes_changes_alpha_and_geometry() {
        let palette = [
            Color {
                red: 123,
                green: 20,
                blue: 40,
                alpha: 0,
            },
            Color {
                red: 123,
                green: 20,
                blue: 40,
                alpha: 128,
            },
            Color {
                red: 123,
                green: 20,
                blue: 40,
                alpha: 255,
            },
        ];
        // Exhaust all 3x3 opaque/translucent/missing arrangements: splits,
        // disappearing runs, holes and equal RGB with different alpha.
        for mut code in 0..3_usize.pow(9) {
            let pixels = (0..9)
                .map(|_| {
                    let color = palette[code % 3];
                    code /= 3;
                    color
                })
                .collect();
            verify_raster_rectangles(&chart_core::grammar::RasterAnnotation {
                width: 3,
                height: 3,
                pixels,
            });
        }
        let solid = chart_core::grammar::RasterAnnotation {
            width: 7,
            height: 5,
            pixels: vec![palette[1]; 35],
        };
        assert_eq!(verify_raster_rectangles(&solid), 1);
        let empty = chart_core::grammar::RasterAnnotation {
            pixels: vec![palette[0]; 35],
            ..solid
        };
        assert_eq!(verify_raster_rectangles(&empty), 0);
    }
    #[test]
    fn nearest_raster_sector_gradient_diagnostic() {
        let (width, height) = (1000, 600);
        let pixels = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let dx = x as f64 - 500.;
                    let dy = y as f64 - 300.;
                    let radius = dx * dx + dy * dy;
                    Color {
                        red: (x / 4) as u8,
                        green: 70,
                        blue: 120,
                        alpha: if (10000. ..=90000.).contains(&radius) && x >= 500 {
                            180
                        } else {
                            0
                        },
                    }
                })
            })
            .collect();
        let raster = chart_core::grammar::RasterAnnotation {
            width,
            height,
            pixels,
        };
        let started = std::time::Instant::now();
        let rectangles = nearest_raster_rectangles(&raster);
        eprintln!(
            "nearest raster synthetic sector: {} cells -> {} rectangles; helper {:?}",
            width * height,
            rectangles.len(),
            started.elapsed()
        );
        assert!(rectangles.len() < width * height / 20);
        assert_eq!(verify_raster_rectangles(&raster), rectangles.len());
    }

    #[test]
    fn sampled_gradient_image_keeps_channel_order_alpha_and_direction() {
        use chart_core::scene::GradientDirection;
        let colors = [
            Color {
                red: 255,
                green: 10,
                blue: 30,
                alpha: 128,
            },
            Color {
                red: 20,
                green: 40,
                blue: 220,
                alpha: 255,
            },
        ];
        for (direction, width, height) in [
            (GradientDirection::Horizontal, 4, 3),
            (GradientDirection::Vertical, 3, 4),
        ] {
            let image = sampled_gradient_image(direction, &colors).unwrap();
            assert_eq!(image.size(0).width.0, width);
            assert_eq!(image.size(0).height.0, height);
            let expected = if direction == GradientDirection::Vertical {
                [[30, 10, 255, 128].repeat(6), [220, 40, 20, 255].repeat(6)].concat()
            } else {
                [
                    30, 10, 255, 128, 30, 10, 255, 128, 220, 40, 20, 255, 220, 40, 20, 255,
                ]
                .repeat(3)
            };
            assert_eq!(image.as_bytes(0).unwrap(), expected);
            assert_eq!(image.frame_count(), 1);
        }
    }

    #[test]
    fn native_projection_rejects_nonfinite_overflow_and_quarter_pixel_loss() {
        for x in [f64::NAN, f64::INFINITY, f64::MAX, 16_777_217.] {
            assert_eq!(pixel(x).unwrap_err().code, DiagnosticCode::PrecisionLoss);
        }
        assert_eq!(f32::from(pixel(320.125).unwrap()), 320.125);
    }
}
