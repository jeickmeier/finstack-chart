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
}
#[derive(Default)]
struct FontRegistry(BTreeMap<String, NativeFont>);
impl gpui::Global for FontRegistry {}

impl NativeFont {
    /// Parse/check a regular face, verify its family, then register these supplied bytes.
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
            if face.is_bold()
                || face.is_italic()
                || !face.names().into_iter().any(|n| {
                    n.name_id == ttf_parser::name_id::FAMILY
                        && n.to_string().as_deref() == Some(family)
                })
            {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "The supplied regular font face does not match the requested family.",
                ));
            }
            if let Some(existing) = cx
                .try_global::<FontRegistry>()
                .and_then(|r| r.0.get(family))
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
            let font = Self {
                descriptor,
                bytes,
                font: gpui::font(family.to_owned()),
            };
            cx.default_global::<FontRegistry>()
                .0
                .insert(family.to_owned(), font.clone());
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
    fn shape(
        &self,
        text: &str,
        font_size: f64,
        color: Color,
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
            color: native_color(color).into(),
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
fn native_color(c: Color) -> gpui::Rgba {
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
            self.system,
        )?;
        TextMetrics::new(
            f64::from(f32::from(line.width)),
            f64::from(f32::from(line.ascent)),
            f64::from(f32::from(line.descent)),
        )
    }
}
enum Paint {
    Quad(gpui::PaintQuad),
    Path(Path<Pixels>, gpui::Rgba),
    Text(Box<ShapedLine>, gpui::Point<Pixels>),
}
struct Item {
    clip: Bounds<Pixels>,
    paint: Paint,
}
pub(crate) struct NativeFrame {
    pub chart: Arc<LaidOutChart>,
    pub bounds: Bounds<Pixels>,
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
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> ChartResult<Self> {
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
        let mut items = vec![];
        for item in chart.scene().items() {
            let result = (|| {
                let clip = rect_at(item.clip.unwrap_or(chart.scene().bounds()), bounds.origin)?;
                let paint = match &item.primitive {
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
                        let line = font.shape(text, *font_size, *color, window.text_system())?;
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
                    Primitive::Path { commands, stroke } => {
                        let mut path = PathBuilder::stroke(pixel(stroke.width)?);
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
                            native_color(stroke.color),
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
            chart,
            bounds,
            items,
        })
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
                                Paint::Quad(q) => window.paint_quad(q.clone()),
                                Paint::Path(p, c) => window.paint_path(p.clone(), *c),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_projection_rejects_nonfinite_overflow_and_quarter_pixel_loss() {
        for x in [f64::NAN, f64::INFINITY, f64::MAX, 16_777_217.] {
            assert_eq!(pixel(x).unwrap_err().code, DiagnosticCode::PrecisionLoss);
        }
        assert_eq!(f32::from(pixel(320.125).unwrap()), 320.125);
    }
}
