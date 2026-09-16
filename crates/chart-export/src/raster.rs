//! Encode the bounded core image payload without filesystem or external resource lookup.
use crate::error;
use chart_core::{ChartResult, DiagnosticCode, grammar::RasterAnnotation};
#[derive(Clone, Hash)]
struct RawImage {
    width: u32,
    height: u32,
    rgb: std::sync::Arc<Vec<u8>>,
    alpha: std::sync::Arc<Vec<u8>>,
}
impl krilla::image::CustomImage for RawImage {
    fn color_channel(&self) -> &[u8] {
        &self.rgb
    }
    fn alpha_channel(&self) -> Option<&[u8]> {
        Some(&self.alpha)
    }
    fn bits_per_component(&self) -> krilla::image::BitsPerComponent {
        krilla::image::BitsPerComponent::Eight
    }
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn icc_profile(&self) -> Option<&[u8]> {
        None
    }
    fn color_space(&self) -> krilla::image::ImageColorspace {
        krilla::image::ImageColorspace::Rgb
    }
}
fn dimensions(r: &RasterAnnotation) -> ChartResult<(u32, u32)> {
    Ok((
        u32::try_from(r.width)
            .map_err(|_| error(DiagnosticCode::ResourceLimit, "Raster width exceeds u32."))?,
        u32::try_from(r.height)
            .map_err(|_| error(DiagnosticCode::ResourceLimit, "Raster height exceeds u32."))?,
    ))
}
pub(crate) fn pdf(r: &RasterAnnotation, interpolate: bool) -> ChartResult<krilla::image::Image> {
    let (width, height) = dimensions(r)?;
    let mut rgb = Vec::with_capacity(r.pixels.len() * 3);
    let mut alpha = Vec::with_capacity(r.pixels.len());
    for c in &r.pixels {
        rgb.extend_from_slice(&[c.red, c.green, c.blue]);
        alpha.push(c.alpha);
    }
    krilla::image::Image::from_custom(
        RawImage {
            width,
            height,
            rgb: std::sync::Arc::new(rgb),
            alpha: std::sync::Arc::new(alpha),
        },
        interpolate,
    )
    .map_err(|e| error(DiagnosticCode::InvalidResource, e))
}
pub(crate) fn png(r: &RasterAnnotation) -> ChartResult<Vec<u8>> {
    let (width, height) = dimensions(r)?;
    let rgba: Vec<u8> = r
        .pixels
        .iter()
        .flat_map(|c| [c.red, c.green, c.blue, c.alpha])
        .collect();
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
        writer
            .write_image_data(&rgba)
            .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
    }
    Ok(bytes)
}
