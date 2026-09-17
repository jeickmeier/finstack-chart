//! Checked EMF records over the immutable publication tree (MS-EMF, version 1).
//! Text uses the shared supplied-font outlines; no native fonts or Windows APIs are required.
use crate::{PublicationProfile, VectorAlphaPolicy, error};
use chart_core::{
    ChartResult, DiagnosticCode,
    scene::{Primitive, Scene},
};
const SCALE: f64 = 1000.;
fn coordinate(v: f64) -> ChartResult<u32> {
    let v = (v * SCALE).round();
    if !v.is_finite() || v < f64::from(i32::MIN) || v > f64::from(i32::MAX) {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "EMF logical coordinate exceeds signed 32-bit range.",
        ));
    }
    Ok((v as i32) as u32)
}
struct Writer {
    bytes: Vec<u8>,
    limit: usize,
    records: u32,
    bounds: [u32; 4],
    alpha: VectorAlphaPolicy,
    omitted: usize,
}
impl Writer {
    fn record(&mut self, kind: u32, words: &[u32]) -> ChartResult<()> {
        let length = words
            .len()
            .checked_add(2)
            .and_then(|v| v.checked_mul(4))
            .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "EMF record length overflow."))?;
        if self
            .bytes
            .len()
            .checked_add(length)
            .is_none_or(|v| v > self.limit)
            || length > u32::MAX as usize
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "EMF output exceeds the byte budget.",
            ));
        }
        self.records = self
            .records
            .checked_add(1)
            .ok_or_else(|| error(DiagnosticCode::ResourceLimit, "EMF record count overflow."))?;
        self.bytes.extend_from_slice(&kind.to_le_bytes());
        self.bytes.extend_from_slice(&(length as u32).to_le_bytes());
        for word in words {
            self.bytes.extend_from_slice(&word.to_le_bytes());
        }
        Ok(())
    }
    fn alpha(&mut self, alpha: f32) -> ChartResult<bool> {
        if alpha == 0. {
            return Ok(false);
        }
        if alpha == 1. {
            return Ok(true);
        }
        match self.alpha {
            VectorAlphaPolicy::OmitTranslucent => {
                self.omitted += 1;
                Ok(false)
            }
            VectorAlphaPolicy::Reject => Err(error(
                DiagnosticCode::ExportFidelity,
                "EMF device does not preserve partial transparency; choose an explicit vector alpha policy.",
            )),
        }
    }
    fn path(&mut self, path: &usvg::tiny_skia_path::Path) -> ChartResult<()> {
        use usvg::tiny_skia_path::PathSegment::*;
        self.record(59, &[])?;
        let mut current = usvg::tiny_skia_path::Point::from_xy(0., 0.);
        let mut start = current;
        for segment in path.segments() {
            match segment {
                MoveTo(p) => {
                    self.record(
                        27,
                        &[coordinate(f64::from(p.x))?, coordinate(f64::from(p.y))?],
                    )?;
                    current = p;
                    start = p;
                }
                LineTo(p) => {
                    self.record(
                        54,
                        &[coordinate(f64::from(p.x))?, coordinate(f64::from(p.y))?],
                    )?;
                    current = p;
                }
                QuadTo(a, b) => {
                    let a1 = usvg::tiny_skia_path::Point::from_xy(
                        current.x + 2. / 3. * (a.x - current.x),
                        current.y + 2. / 3. * (a.y - current.y),
                    );
                    let a2 = usvg::tiny_skia_path::Point::from_xy(
                        b.x + 2. / 3. * (a.x - b.x),
                        b.y + 2. / 3. * (a.y - b.y),
                    );
                    self.cubic([a1, a2, b])?;
                    current = b;
                }
                CubicTo(a, b, c) => {
                    self.cubic([a, b, c])?;
                    current = c;
                }
                Close => {
                    self.record(61, &[])?;
                    current = start;
                }
            }
        }
        self.record(60, &[])
    }
    fn cubic(&mut self, points: [usvg::tiny_skia_path::Point; 3]) -> ChartResult<()> {
        let mut words = self.bounds.to_vec();
        words.push(3);
        for p in points {
            words.push(coordinate(f64::from(p.x))?);
            words.push(coordinate(f64::from(p.y))?);
        }
        self.record(5, &words)
    }
    fn fill(
        &mut self,
        path: &usvg::tiny_skia_path::Path,
        color: usvg::Color,
        even: bool,
    ) -> ChartResult<()> {
        let color =
            u32::from(color.red) | (u32::from(color.green) << 8) | (u32::from(color.blue) << 16);
        self.record(39, &[1, 0, color, 0])?;
        self.record(37, &[1])?;
        self.record(19, &[if even { 1 } else { 2 }])?;
        self.path(path)?;
        let bounds = self.bounds;
        self.record(62, &bounds)?;
        self.record(37, &[0x80000005])?;
        self.record(40, &[1])
    }
    fn node(&mut self, node: &usvg::Node) -> ChartResult<()> {
        match node {
            usvg::Node::Group(group) => {
                if !self.alpha(group.opacity().get())? {
                    return Ok(());
                }
                if group.mask().is_some() || !group.filters().is_empty() {
                    return Err(error(
                        DiagnosticCode::ExportFidelity,
                        "EMF cannot silently flatten masks or filters.",
                    ));
                }
                for node in group.children() {
                    self.node(node)?;
                }
                Ok(())
            }
            usvg::Node::Text(text) => self.group(text.flattened()),
            usvg::Node::Image(_) => Err(error(
                DiagnosticCode::ExportFidelity,
                "EMF image must be a retained raster scene item.",
            )),
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    return Ok(());
                }
                let paints = if path.paint_order() == usvg::PaintOrder::StrokeAndFill {
                    [true, false]
                } else {
                    [false, true]
                };
                for stroke in paints {
                    let (paint, alpha, even) = if stroke {
                        let Some(s) = path.stroke() else {
                            continue;
                        };
                        (s.paint(), s.opacity().get(), false)
                    } else {
                        let Some(f) = path.fill() else {
                            continue;
                        };
                        (
                            f.paint(),
                            f.opacity().get(),
                            f.rule() == usvg::FillRule::EvenOdd,
                        )
                    };
                    if !self.alpha(alpha)? {
                        continue;
                    }
                    let usvg::Paint::Color(color) = paint else {
                        return Err(error(
                            DiagnosticCode::ExportFidelity,
                            "EMF retained-vector device does not support gradient or pattern paint.",
                        ));
                    };
                    let geometry = if stroke {
                        let s = path.stroke().unwrap();
                        let style = usvg::tiny_skia_path::Stroke {
                            width: s.width().get(),
                            miter_limit: s.miterlimit().get(),
                            line_cap: match s.linecap() {
                                usvg::LineCap::Butt => usvg::tiny_skia_path::LineCap::Butt,
                                usvg::LineCap::Round => usvg::tiny_skia_path::LineCap::Round,
                                usvg::LineCap::Square => usvg::tiny_skia_path::LineCap::Square,
                            },
                            line_join: match s.linejoin() {
                                usvg::LineJoin::Round => usvg::tiny_skia_path::LineJoin::Round,
                                usvg::LineJoin::Bevel => usvg::tiny_skia_path::LineJoin::Bevel,
                                _ => usvg::tiny_skia_path::LineJoin::Miter,
                            },
                            dash: s.dasharray().and_then(|d| {
                                usvg::tiny_skia_path::StrokeDash::new(d.to_vec(), s.dashoffset())
                            }),
                        };
                        path.data().stroke(&style, 1.).ok_or_else(|| {
                            error(
                                DiagnosticCode::ExportFidelity,
                                "EMF stroke outline could not be resolved.",
                            )
                        })?
                    } else {
                        path.data().clone()
                    };
                    let geometry = geometry.transform(path.abs_transform()).ok_or_else(|| {
                        error(
                            DiagnosticCode::NumericalDomain,
                            "EMF path transform failed.",
                        )
                    })?;
                    self.fill(&geometry, *color, even)?;
                }
                Ok(())
            }
        }
    }
    fn group(&mut self, group: &usvg::Group) -> ChartResult<()> {
        for node in group.children() {
            self.node(node)?;
        }
        Ok(())
    }
    fn raster(
        &mut self,
        bounds: chart_core::Rect,
        raster: &chart_core::grammar::RasterAnnotation,
        interpolate: bool,
    ) -> ChartResult<()> {
        if raster.pixels.iter().any(|p| p.alpha != 255) {
            if interpolate {
                return Err(error(
                    DiagnosticCode::ExportFidelity,
                    "EMF interpolated rasters require opaque pixels.",
                ));
            }
            let dx = bounds.width() / raster.width as f64;
            let dy = bounds.height() / raster.height as f64;
            for (i, color) in raster.pixels.iter().enumerate() {
                if !self.alpha(f32::from(color.alpha) / 255.)? {
                    continue;
                }
                let rect = usvg::tiny_skia_path::Rect::from_xywh(
                    (bounds.origin().x() + (i % raster.width) as f64 * dx) as f32,
                    (bounds.origin().y() + (i / raster.width) as f64 * dy) as f32,
                    dx as f32,
                    dy as f32,
                )
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::NumericalDomain,
                        "EMF raster cell bounds failed.",
                    )
                })?;
                self.fill(
                    &usvg::tiny_skia_path::PathBuilder::from_rect(rect),
                    usvg::Color::new_rgb(color.red, color.green, color.blue),
                    false,
                )?;
            }
            return Ok(());
        }
        let pixel_bytes = raster.pixels.len().checked_mul(4).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "EMF raster byte count overflow.",
            )
        })?;
        if pixel_bytes > (u32::MAX as usize).saturating_sub(120)
            || pixel_bytes.checked_add(120).is_none_or(|n| {
                self.bytes
                    .len()
                    .checked_add(n)
                    .is_none_or(|n| n > self.limit)
            })
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "EMF raster exceeds output byte budget.",
            ));
        }
        let width = u32::try_from(raster.width)
            .map_err(|_| error(DiagnosticCode::ResourceLimit, "EMF raster width overflow."))?;
        let height = i32::try_from(raster.height)
            .map_err(|_| error(DiagnosticCode::ResourceLimit, "EMF raster height overflow."))?;
        self.record(21, &[if interpolate { 4 } else { 3 }])?;
        let mut words = self.bounds.to_vec();
        words.extend([
            coordinate(bounds.origin().x())?,
            coordinate(bounds.origin().y())?,
            0,
            0,
            width,
            height as u32,
            80,
            40,
            120,
            pixel_bytes as u32,
            0,
            0x00CC0020,
            coordinate(bounds.width())?,
            coordinate(bounds.height())?,
        ]);
        words.extend([
            40,
            width,
            (-height) as u32,
            0x00200001,
            0,
            pixel_bytes as u32,
            0,
            0,
            0,
            0,
        ]);
        words.extend(
            raster
                .pixels
                .iter()
                .map(|p| u32::from(p.blue) | (u32::from(p.green) << 8) | (u32::from(p.red) << 16)),
        );
        self.record(81, &words)
    }
}
/// One-page enhanced metafile. A .wmf extension is a reference alias, not classic WMF.
pub(crate) fn encode(
    scene: &Scene,
    tree: &usvg::Tree,
    profile: &PublicationProfile,
) -> ChartResult<(Vec<u8>, usize)> {
    if profile.vector_device.as_ref().is_some_and(|v| {
        !matches!(
            v.color_model,
            crate::VectorColorModel::Srgb | crate::VectorColorModel::Rgb
        )
    }) {
        return Err(error(
            DiagnosticCode::ExportFidelity,
            "EMF supports RGB colors; PostScript color-model conversions are not available for this device.",
        ));
    }
    let width = profile.page.width();
    let height = profile.page.height();
    let pixels = |v: f64| -> ChartResult<u32> {
        let v = (v * f64::from(profile.dpi) / 72.).round();
        if v < 1. || v > f64::from(i32::MAX) {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "EMF device bounds overflow.",
            ));
        }
        Ok(v as u32)
    };
    let [w, h] = [pixels(width)?, pixels(height)?];
    let mut writer = Writer {
        bytes: vec![],
        limit: profile.max_output_bytes,
        records: 0,
        bounds: [0, 0, w - 1, h - 1],
        alpha: profile.vector_device.clone().unwrap_or_default().alpha,
        omitted: 0,
    };
    let header = vec![
        0,
        0,
        w - 1,
        h - 1,
        0,
        0,
        (width / 72. * 2540.).round() as u32,
        (height / 72. * 2540.).round() as u32,
        0x464D4520,
        0x00010000,
        0,
        0,
        2,
        0,
        0,
        0,
        w,
        h,
        (width / 72. * 25.4).round().max(1.) as u32,
        (height / 72. * 25.4).round().max(1.) as u32,
        0,
        0,
        0,
        (width / 72. * 25400.).round() as u32,
        (height / 72. * 25400.).round() as u32,
    ];
    writer.record(1, &header)?;
    writer.record(17, &[8])?;
    writer.record(9, &[coordinate(width)?, coordinate(height)?])?;
    writer.record(11, &[w, h])?;
    writer.record(18, &[1])?;
    for (index, item) in scene.items().iter().enumerate() {
        if crate::snapshot::point_is_clipped(item, scene.bounds()) {
            continue;
        }
        let clip = item.clip.unwrap_or(scene.bounds());
        if clip.width() == 0. || clip.height() == 0. {
            continue;
        }
        writer.record(33, &[])?;
        writer.record(
            30,
            &[
                coordinate(clip.origin().x())?,
                coordinate(clip.origin().y())?,
                coordinate(clip.max_x())?,
                coordinate(clip.max_y())?,
            ],
        )?;
        if let Primitive::RasterImage {
            bounds,
            raster,
            interpolate,
            ..
        } = &item.primitive
        {
            writer.raster(*bounds, raster, *interpolate)?;
        } else if let Some(node) = tree.node_by_id(&format!("item-{index}")) {
            writer.node(node)?;
        }
        writer.record(34, &[u32::MAX])?;
    }
    writer.record(14, &[0, 0, 20])?;
    let size = u32::try_from(writer.bytes.len()).map_err(|_| {
        error(
            DiagnosticCode::ResourceLimit,
            "EMF total byte count overflow.",
        )
    })?;
    writer.bytes[48..52].copy_from_slice(&size.to_le_bytes());
    writer.bytes[52..56].copy_from_slice(&writer.records.to_le_bytes());
    Ok((writer.bytes, writer.omitted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Format, Output, PageSize, export_options};
    use chart_core::prelude::*;
    fn word(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn frame(alpha: f64, policy: VectorAlphaPolicy) -> crate::FigureSnapshot {
        let plot = plot(
            Data::columns()
                .column("x", [0., 1., 2.])
                .column("y", [0., 2., 1.])
                .build()
                .unwrap(),
        )
        .layer(points().aes(aes().x("x").y("y")).alpha(alpha))
        .build()
        .unwrap();
        let output = Output::new(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
        )
        .unwrap();
        output
            .request(
                &plot,
                export_options(PageSize::points(240., 160.).unwrap()).vector_device(
                    crate::VectorDeviceOptions {
                        alpha: policy,
                        ..Default::default()
                    },
                ),
            )
            .unwrap()
            .prepare()
            .unwrap()
    }
    #[test]
    fn emf_header_records_paths_and_resource_budget_are_checked() {
        let bytes = frame(1., VectorAlphaPolicy::Reject)
            .export(Format::Emf)
            .unwrap()
            .bytes;
        assert_eq!(word(&bytes, 0), 1);
        assert_eq!(word(&bytes, 4), 108);
        assert_eq!(word(&bytes, 40), 0x464D4520);
        assert_eq!(word(&bytes, 44), 0x10000);
        assert_eq!(word(&bytes, 48) as usize, bytes.len());
        assert_eq!(word(&bytes, 32), (240_f64 / 72. * 2540.).round() as u32);
        let mut cursor = 0;
        let mut records = 0;
        let mut kinds = std::collections::BTreeSet::new();
        while cursor < bytes.len() {
            let kind = word(&bytes, cursor);
            let length = word(&bytes, cursor + 4) as usize;
            assert!(length >= 8 && length.is_multiple_of(4) && cursor + length <= bytes.len());
            kinds.insert(kind);
            if kind == 14 {
                assert_eq!(cursor + length, bytes.len());
                assert_eq!(length, 20);
            }
            cursor += length;
            records += 1;
        }
        assert_eq!(records, word(&bytes, 52));
        for kind in [1, 5, 14, 27, 30, 33, 34, 37, 39, 40, 59, 60, 61, 62] {
            assert!(kinds.contains(&kind), "record {kind}");
        }
        assert_eq!(crate::host::format("wmf").unwrap(), Format::Emf);
        let mut writer = Writer {
            bytes: vec![],
            limit: 7,
            records: 0,
            bounds: [0; 4],
            alpha: VectorAlphaPolicy::Reject,
            omitted: 0,
        };
        assert_eq!(
            writer.record(59, &[]).unwrap_err().code,
            DiagnosticCode::ResourceLimit
        );
        assert!(writer.bytes.is_empty());
        assert!(coordinate(f64::MAX).is_err());
    }
    #[test]
    fn emf_alpha_requires_explicit_source_limitation_policy() {
        assert_eq!(
            frame(0.5, VectorAlphaPolicy::Reject)
                .export(Format::Emf)
                .unwrap_err()
                .code,
            DiagnosticCode::ExportFidelity
        );
        let export = frame(0.5, VectorAlphaPolicy::OmitTranslucent)
            .export(Format::Emf)
            .unwrap();
        assert!(
            export
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::ExportFidelity)
        );
    }
    #[test]
    fn emf_raster_offsets_and_top_down_bgra_are_independently_decodable() {
        let mut writer = Writer {
            bytes: vec![],
            limit: 1024,
            records: 0,
            bounds: [0, 0, 2, 1],
            alpha: VectorAlphaPolicy::Reject,
            omitted: 0,
        };
        let raster = chart_core::grammar::RasterAnnotation {
            width: 2,
            height: 1,
            pixels: vec![
                chart_core::scene::Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
                chart_core::scene::Color {
                    red: 0,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
            ],
        };
        writer
            .raster(
                chart_core::Rect::new(1., 2., 3., 4.).unwrap(),
                &raster,
                false,
            )
            .unwrap();
        let record = &writer.bytes[12..];
        assert_eq!(word(record, 0), 81);
        assert_eq!(word(record, 4), 128);
        assert_eq!(word(record, 48), 80);
        assert_eq!(word(record, 56), 120);
        assert_eq!(word(record, 60), 8);
        assert_eq!(word(record, 88), u32::MAX);
        assert_eq!(&record[120..128], &[0, 0, 255, 0, 0, 255, 0, 0]);
    }
}
