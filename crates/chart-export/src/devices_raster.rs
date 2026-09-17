//! Additional raster devices share the PNG point-to-pixel projection and budgets.
use crate::{Format, PublicationProfile, error};
use chart_core::{ChartResult, DiagnosticCode, scene::Color};
use std::io::{self, Cursor, Seek, SeekFrom, Write};

/// TIFF strip compression, with no hidden fallback to another codec.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TiffCompression {
    /// Uncompressed strips, matching the reference default.
    #[default]
    None,
    /// Lempel-Ziv-Welch compression.
    Lzw,
    /// ZIP/Deflate compression.
    Deflate,
    /// TIFF PackBits byte-run compression.
    PackBits,
    /// JPEG strips, flattened against the explicit matte (lossy).
    Jpeg,
    /// XZ/LZMA2 lossless strips.
    Lzma,
    /// Zstandard lossless strips using the portable fastest compression level.
    Zstd,
    /// Lossless WebP strips preserving straight alpha.
    Webp,
}
/// Explicit device options for JPEG, TIFF and BMP; PNG retains its original encoder.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RasterDeviceOptions {
    /// JPEG quality in 0..=100; the reference default is 75.
    pub jpeg_quality: u8,
    /// Opaque matte for formats without alpha (JPEG/BMP), applied after scene rendering.
    pub matte: Color,
    /// TIFF strip compression.
    pub tiff_compression: TiffCompression,
    /// Horizontal differencing for TIFF LZW/Deflate compression.
    pub tiff_predictor: bool,
}
impl Default for RasterDeviceOptions {
    fn default() -> Self {
        Self {
            jpeg_quality: 75,
            matte: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            },
            tiff_compression: TiffCompression::None,
            tiff_predictor: false,
        }
    }
}
impl RasterDeviceOptions {
    pub(crate) fn validate(&self) -> ChartResult<()> {
        if self.jpeg_quality > 100
            || self.matte.alpha != 255
            || (self.tiff_predictor
                && !matches!(
                    self.tiff_compression,
                    TiffCompression::Lzw | TiffCompression::Deflate
                ))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Raster device requires JPEG quality 0..100, an opaque matte, and a TIFF predictor only with LZW/Deflate.",
            ));
        }
        Ok(())
    }
}
struct Buffer {
    inner: Cursor<Vec<u8>>,
    limit: usize,
    exceeded: bool,
}
impl Buffer {
    fn new(limit: usize) -> Self {
        Self {
            inner: Cursor::new(Vec::new()),
            limit,
            exceeded: false,
        }
    }
    fn fail(&mut self) -> io::Error {
        self.exceeded = true;
        io::Error::other("Encoded publication exceeds output byte budget")
    }
    fn diagnostic(&self, e: impl std::fmt::Display) -> chart_core::Diagnostic {
        error(
            if self.exceeded {
                DiagnosticCode::ResourceLimit
            } else {
                DiagnosticCode::ExportFidelity
            },
            e.to_string(),
        )
    }
}
impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .inner
            .position()
            .checked_add(bytes.len() as u64)
            .is_none_or(|end| end > self.limit as u64)
        {
            return Err(self.fail());
        }
        self.inner.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Seek for Buffer {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(v) => i128::from(v),
            SeekFrom::End(v) => self.inner.get_ref().len() as i128 + i128::from(v),
            SeekFrom::Current(v) => i128::from(self.inner.position()) + i128::from(v),
        };
        if position < 0 || position > self.limit as i128 {
            return Err(self.fail());
        }
        self.inner.seek(SeekFrom::Start(position as u64))
    }
}
fn flatten(rgba: &[u8], matte: Color) -> Vec<u8> {
    rgba.chunks_exact(4)
        .flat_map(|p| {
            let alpha = u32::from(p[3]);
            [matte.red, matte.green, matte.blue]
                .into_iter()
                .enumerate()
                .map(move |(i, b)| {
                    ((u32::from(p[i]) * alpha + u32::from(b) * (255 - alpha) + 127) / 255) as u8
                })
        })
        .collect()
}
pub(crate) fn encode(
    tree: &usvg::Tree,
    p: &PublicationProfile,
    format: Format,
) -> ChartResult<Vec<u8>> {
    if format == Format::Tiff {
        return tiff_pages(&[(tree, p)]);
    }
    let options = p.raster_device.clone().unwrap_or_default();
    options.validate()?;
    let (width, height) = p.raster_dimensions()?;
    let density = (f64::from(p.dpi) / 0.0254).round();
    if (format == Format::Jpeg
        && (p.dpi > u32::from(u16::MAX)
            || width > u32::from(u16::MAX)
            || height > u32::from(u16::MAX)))
        || (format == Format::Bmp
            && (density > f64::from(i32::MAX)
                || width > i32::MAX as u32
                || height > i32::MAX as u32))
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Raster device dimensions or physical density exceed its metadata representation.",
        ));
    }
    let rgba = crate::encode::raster_rgba(tree, p, width, height)?;
    let mut buffer = Buffer::new(p.max_output_bytes);
    match format {
        Format::Jpeg => {
            let rgb = flatten(&rgba, options.matte);
            let result = {
                let mut encoder =
                    jpeg_encoder::Encoder::new(&mut buffer, options.jpeg_quality.max(1));
                encoder.set_density(jpeg_encoder::PixelDensity::dpi(p.dpi as u16));
                encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::F_2_2);
                encoder.encode(
                    &rgb,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
            };
            result.map_err(|e| buffer.diagnostic(e))?;
        }
        Format::Bmp => {
            let rgb = flatten(&rgba, options.matte);
            let stride = (u64::from(width) * 3 + 3) & !3;
            let pixels = stride * u64::from(height);
            let size = u32::try_from(pixels + 54).map_err(|_| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "BMP file length exceeds its 32-bit header.",
                )
            })?;
            if size as usize > p.max_output_bytes {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "BMP exceeds the output byte budget.",
                ));
            }
            let mut header = [0_u8; 54];
            header[..2].copy_from_slice(b"BM");
            header[2..6].copy_from_slice(&size.to_le_bytes());
            header[10..14].copy_from_slice(&54_u32.to_le_bytes());
            header[14..18].copy_from_slice(&40_u32.to_le_bytes());
            header[18..22].copy_from_slice(&(width as i32).to_le_bytes());
            header[22..26].copy_from_slice(&(height as i32).to_le_bytes());
            header[26..28].copy_from_slice(&1_u16.to_le_bytes());
            header[28..30].copy_from_slice(&24_u16.to_le_bytes());
            header[34..38].copy_from_slice(&(pixels as u32).to_le_bytes());
            header[38..42].copy_from_slice(&(density as i32).to_le_bytes());
            header[42..46].copy_from_slice(&(density as i32).to_le_bytes());
            buffer
                .write_all(&header)
                .map_err(|e| buffer.diagnostic(e))?;
            let padding = [0; 3];
            let pad = (stride - u64::from(width) * 3) as usize;
            for row in rgb.chunks_exact(width as usize * 3).rev() {
                for c in row.chunks_exact(3) {
                    buffer
                        .write_all(&[c[2], c[1], c[0]])
                        .map_err(|e| buffer.diagnostic(e))?;
                }
                buffer
                    .write_all(&padding[..pad])
                    .map_err(|e| buffer.diagnostic(e))?;
            }
        }

        Format::Tiff => unreachable!("TIFF uses the shared page encoder"),
        _ => {
            return Err(error(
                DiagnosticCode::Validation,
                "Additional raster encoder requires JPEG, TIFF or BMP.",
            ));
        }
    }
    Ok(buffer.inner.into_inner())
}

/// Encode pages sequentially, retaining only one raster allocation at a time.
pub(crate) fn tiff_pages(pages: &[(&usvg::Tree, &PublicationProfile)]) -> ChartResult<Vec<u8>> {
    use tiff::{
        encoder::{Compression, Predictor, Rational, TiffEncoder, colortype::RGBA8},
        tags::Tag,
    };
    if pages.is_empty() || pages.len() > 1024 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "TIFF requires 1..=1024 pages.",
        ));
    }
    let limit = pages.iter().map(|(_, p)| p.max_output_bytes).min().unwrap();
    for (_, p) in pages {
        p.raster_device.clone().unwrap_or_default().validate()?;
        p.raster_dimensions()?;
    }
    let mut buffer = Buffer::new(limit);
    // Preserve the buffer's resource-limit diagnostic even if a codec wraps its I/O error.
    let result = (|| -> ChartResult<()> {
        let codec = |e: tiff::TiffError| error(DiagnosticCode::ExportFidelity, e.to_string());
        let mut encoder = TiffEncoder::new(&mut buffer).map_err(codec)?;
        for (tree, p) in pages {
            let options = p.raster_device.clone().unwrap_or_default();
            if matches!(
                options.tiff_compression,
                TiffCompression::Jpeg
                    | TiffCompression::Lzma
                    | TiffCompression::Zstd
                    | TiffCompression::Webp
            ) {
                let (width, height) = p.raster_dimensions()?;
                let rgba = crate::encode::raster_rgba(tree, p, width, height)?;
                advanced_tiff_page(&mut encoder, &rgba, width, height, p, &options)?;
                continue;
            }
            let compression = match options.tiff_compression {
                TiffCompression::None => Compression::Uncompressed,
                TiffCompression::Lzw => Compression::Lzw,
                TiffCompression::Deflate => Compression::Deflate(Default::default()),
                TiffCompression::PackBits => Compression::Packbits,
                _ => unreachable!("advanced codecs handled above"),
            };
            encoder =
                encoder
                    .with_compression(compression)
                    .with_predictor(if options.tiff_predictor {
                        Predictor::Horizontal
                    } else {
                        Predictor::None
                    });
            let (width, height) = p.raster_dimensions()?;
            let rgba = crate::encode::raster_rgba(tree, p, width, height)?;
            let mut image = encoder.new_image::<RGBA8>(width, height).map_err(codec)?;
            image
                .encoder()
                .write_tag(Tag::ResolutionUnit, 2u16)
                .map_err(codec)?;
            image
                .encoder()
                .write_tag(Tag::XResolution, Rational { n: p.dpi, d: 1 })
                .map_err(codec)?;
            image
                .encoder()
                .write_tag(Tag::YResolution, Rational { n: p.dpi, d: 1 })
                .map_err(codec)?;
            image
                .encoder()
                .write_tag(Tag::ExtraSamples, &[2u16][..])
                .map_err(codec)?;
            image.write_data(&rgba).map_err(codec)?;
        }
        Ok(())
    })();
    if buffer.exceeded {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "TIFF exceeds the output byte budget.",
        ));
    }
    result?;
    Ok(buffer.inner.into_inner())
}

fn advanced_tiff_page(
    encoder: &mut tiff::encoder::TiffEncoder<&mut Buffer>,
    rgba: &[u8],
    width: u32,
    height: u32,
    p: &PublicationProfile,
    options: &RasterDeviceOptions,
) -> ChartResult<()> {
    use tiff::{encoder::Rational, tags::Tag};
    let codec = |e: &dyn std::fmt::Display| error(DiagnosticCode::ExportFidelity, e.to_string());
    let mut encoded = Buffer::new(p.max_output_bytes);
    let (compression, samples, photometric) = match options.tiff_compression {
        TiffCompression::Jpeg => {
            if width > u16::MAX as u32 || height > u16::MAX as u32 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "JPEG TIFF dimensions exceed16-bit codec limit.",
                ));
            }
            let rgb = flatten(rgba, options.matte);
            let mut writer = jpeg_encoder::Encoder::new(&mut encoded, options.jpeg_quality.max(1));
            writer.set_sampling_factor(jpeg_encoder::SamplingFactor::F_2_2);
            writer
                .encode(
                    &rgb,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
                .map_err(|e| encoded.diagnostic(e))?;
            (7u16, 3u16, 6u16)
        }
        TiffCompression::Lzma => {
            let result = (|| -> io::Result<()> {
                let mut writer =
                    lzma_rust2::XzWriter::new(&mut encoded, lzma_rust2::XzOptions::default())?;
                writer.write_all(rgba)?;
                writer.finish()?;
                Ok(())
            })();
            result.map_err(|e| encoded.diagnostic(e))?;
            (34925, 4, 2)
        }
        TiffCompression::Zstd => {
            // The upstream compressor unwraps writes. This bounded sink consumes excess
            // bytes without retaining them, then reports the resource failure explicitly.
            struct Sink<'a>(&'a mut Buffer);
            impl Write for Sink<'_> {
                fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                    if !self.0.exceeded {
                        let _ = self.0.write_all(b);
                    }
                    Ok(b.len())
                }
                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }
            }
            ruzstd::encoding::compress(
                rgba,
                Sink(&mut encoded),
                ruzstd::encoding::CompressionLevel::Fastest,
            );
            (50000, 4, 2)
        }
        TiffCompression::Webp => {
            image_webp::WebPEncoder::new(&mut encoded)
                .encode(rgba, width, height, image_webp::ColorType::Rgba8)
                .map_err(|e| encoded.diagnostic(e))?;
            (50001, 4, 2)
        }
        _ => unreachable!("advanced TIFF codec"),
    };
    if encoded.exceeded {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "TIFF strip exceeds output budget.",
        ));
    }
    let bytes = encoded.inner.into_inner();
    let mut directory = encoder.image_directory().map_err(|e| codec(&e))?;
    let offset = directory
        .write_data(bytes.as_slice())
        .map_err(|e| codec(&e))?;
    let offset = u32::try_from(offset).map_err(|e| codec(&e))?;
    let length = u32::try_from(bytes.len()).map_err(|e| codec(&e))?;
    let mut tag = |t, v: u32| directory.write_tag(t, v).map_err(|e| codec(&e));
    tag(Tag::ImageWidth, width)?;
    tag(Tag::ImageLength, height)?;
    tag(Tag::RowsPerStrip, height)?;
    tag(Tag::StripOffsets, offset)?;
    tag(Tag::StripByteCounts, length)?;
    directory
        .write_tag(Tag::BitsPerSample, &vec![8u16; samples as usize][..])
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::Compression, compression)
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::PhotometricInterpretation, photometric)
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::SamplesPerPixel, samples)
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::PlanarConfiguration, 1u16)
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::ResolutionUnit, 2u16)
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::XResolution, Rational { n: p.dpi, d: 1 })
        .map_err(|e| codec(&e))?;
    directory
        .write_tag(Tag::YResolution, Rational { n: p.dpi, d: 1 })
        .map_err(|e| codec(&e))?;
    if samples == 4 {
        directory
            .write_tag(Tag::ExtraSamples, &[2u16][..])
            .map_err(|e| codec(&e))?;
    } else {
        directory
            .write_tag(Tag::ChromaSubsampling, &[2u16, 2][..])
            .map_err(|e| codec(&e))?;
    }
    directory.finish().map_err(|e| codec(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> (usvg::Tree, PublicationProfile) {
        let tree=usvg::Tree::from_str(r##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><path d="M0 0H1V1H0Z" fill="#ff0000" fill-opacity="0.5"/><path d="M1 0H2V1H1Z" fill="#00ff00"/><path d="M1 1H2V2H1Z" fill="#0000ff" fill-opacity="0.25"/></svg>"##,&usvg::Options::default()).unwrap();
        let mut p = PublicationProfile::new(
            crate::PageSize::points(2., 2.).unwrap(),
            chart_core::services::ResourceDescriptor {
                id: chart_core::ResourceId::new(1),
                revision: chart_core::Revision::INITIAL,
                kind: chart_core::services::ResourceKind::Font,
                byte_len: 1,
            },
        )
        .unwrap();
        p.dpi = 72;
        p.background = Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }
        .into();
        (tree, p)
    }
    #[test]
    fn bmp_preserves_rows_padding_flattening_and_density() {
        let (tree, p) = input();
        let bytes = encode(&tree, &p, Format::Bmp).unwrap();
        assert_eq!(&bytes[..2], b"BM");
        assert_eq!(i32::from_le_bytes(bytes[38..42].try_into().unwrap()), 2835);
        assert_eq!(u32::from_le_bytes(bytes[2..6].try_into().unwrap()), 70);
        assert_eq!(i32::from_le_bytes(bytes[18..22].try_into().unwrap()), 2);
        assert_eq!(i32::from_le_bytes(bytes[22..26].try_into().unwrap()), 2);
        // Independent literal RGB24 BMP scanlines: bottom row first, BGR order,
        // two zero padding bytes per six-byte row.
        assert_eq!(
            &bytes[54..],
            &[
                255, 255, 255, 255, 191, 191, 0, 0, 127, 127, 255, 0, 255, 0, 0, 0
            ]
        );
    }
    #[test]
    fn tiff_straight_alpha_lossless_compression_and_physical_tags() {
        let (tree, mut p) = input();
        for compression in [
            TiffCompression::None,
            TiffCompression::Lzw,
            TiffCompression::Deflate,
            TiffCompression::PackBits,
        ] {
            for predictor in [false, true] {
                if predictor
                    && !matches!(compression, TiffCompression::Lzw | TiffCompression::Deflate)
                {
                    continue;
                }
                p.raster_device = Some(RasterDeviceOptions {
                    tiff_compression: compression,
                    tiff_predictor: predictor,
                    ..Default::default()
                });
                let bytes = encode(&tree, &p, Format::Tiff).unwrap();
                let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).unwrap();
                assert_eq!(decoder.dimensions().unwrap(), (2, 2));
                assert_eq!(
                    decoder
                        .get_tag_u32(tiff::tags::Tag::ResolutionUnit)
                        .unwrap(),
                    2
                );
                assert_eq!(
                    decoder
                        .get_tag_u32_vec(tiff::tags::Tag::ExtraSamples)
                        .unwrap(),
                    [2]
                );
                let tiff::decoder::DecodingResult::U8(pixels) = decoder.read_image().unwrap()
                else {
                    panic!("expected eight-bit RGBA")
                };
                assert_eq!(
                    pixels,
                    [255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 0, 0, 0, 0, 255, 64]
                );
            }
        }
    }
    #[test]
    fn tiff_pages_retain_distinct_dimensions_pixels_codecs_and_budget() {
        let (tree, first) = input();
        let mut second = first.clone();
        second.dpi = 144;
        second.raster_device = Some(RasterDeviceOptions {
            tiff_compression: TiffCompression::Lzw,
            tiff_predictor: true,
            ..Default::default()
        });
        let single = encode(&tree, &first, Format::Tiff).unwrap();
        assert_eq!(single, tiff_pages(&[(&tree, &first)]).unwrap());
        let bytes = tiff_pages(&[(&tree, &first), (&tree, &second)]).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(Cursor::new(&bytes)).unwrap();
        assert_eq!(decoder.dimensions().unwrap(), (2, 2));
        assert_eq!(
            decoder.get_tag_u32(tiff::tags::Tag::Compression).unwrap(),
            1
        );
        let tiff::decoder::DecodingResult::U8(pixels) = decoder.read_image().unwrap() else {
            panic!()
        };
        assert_eq!(
            pixels,
            [255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 0, 0, 0, 0, 255, 64]
        );
        assert!(decoder.more_images());
        decoder.next_image().unwrap();
        assert_eq!(decoder.dimensions().unwrap(), (4, 4));
        assert_eq!(
            decoder.get_tag_u32(tiff::tags::Tag::Compression).unwrap(),
            5
        );
        let tiff::decoder::DecodingResult::U8(pixels) = decoder.read_image().unwrap() else {
            panic!()
        };
        assert_eq!(pixels.len(), 64);
        assert!(!decoder.more_images());
        second.max_output_bytes = bytes.len() - 1;
        assert_eq!(
            tiff_pages(&[(&tree, &first), (&tree, &second)])
                .unwrap_err()
                .code,
            DiagnosticCode::ResourceLimit
        );
    }
    #[test]
    fn advanced_tiff_strip_codecs_preserve_pixels_and_enforce_budgets() {
        use std::io::Read;
        let (tree, mut p) = input();
        let expected = [255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 0, 0, 0, 0, 255, 64];
        for (compression, tag) in [
            (TiffCompression::Jpeg, 7),
            (TiffCompression::Lzma, 34925),
            (TiffCompression::Zstd, 50000),
            (TiffCompression::Webp, 50001),
        ] {
            p.raster_device = Some(RasterDeviceOptions {
                tiff_compression: compression,
                ..Default::default()
            });
            let bytes = encode(&tree, &p, Format::Tiff).unwrap();
            let mut decoder = tiff::decoder::Decoder::new(Cursor::new(&bytes)).unwrap();
            assert_eq!(
                decoder.get_tag_u32(tiff::tags::Tag::Compression).unwrap(),
                tag
            );
            let offset = decoder.get_tag_u32(tiff::tags::Tag::StripOffsets).unwrap() as usize;
            let count = decoder
                .get_tag_u32(tiff::tags::Tag::StripByteCounts)
                .unwrap() as usize;
            let strip = &bytes[offset..offset + count];
            let mut decoded = Vec::new();
            match compression {
                TiffCompression::Jpeg => {
                    let mut jpeg = zune_jpeg::JpegDecoder::new(Cursor::new(strip));
                    decoded = jpeg.decode().unwrap();
                    assert_eq!(jpeg.dimensions(), Some((2, 2)));
                    assert_eq!(decoded.len(), 12);
                }
                TiffCompression::Lzma => {
                    lzma_rust2::XzReader::new(strip, true)
                        .read_to_end(&mut decoded)
                        .unwrap();
                }
                TiffCompression::Zstd => {
                    ruzstd::decoding::StreamingDecoder::new(strip)
                        .unwrap()
                        .read_to_end(&mut decoded)
                        .unwrap();
                }
                TiffCompression::Webp => {
                    decoded.resize(16, 0);
                    image_webp::WebPDecoder::new(Cursor::new(strip))
                        .unwrap()
                        .read_image(&mut decoded)
                        .unwrap();
                }
                _ => unreachable!(),
            }
            if compression != TiffCompression::Jpeg {
                assert_eq!(decoded, expected);
            }
            let mut small = p.clone();
            small.max_output_bytes = 16;
            assert_eq!(
                encode(&tree, &small, Format::Tiff).unwrap_err().code,
                DiagnosticCode::ResourceLimit
            );
        }
    }
    #[test]
    fn jpeg_has_real_jfif_density_and_independent_decoder() {
        let (tree, mut p) = input();
        p.dpi = 144;
        let bytes = encode(&tree, &p, Format::Jpeg).unwrap();
        assert_eq!(&bytes[..2], &[255, 216]);
        let jfif = bytes.windows(5).position(|b| b == b"JFIF\0").unwrap();
        assert_eq!(bytes[jfif + 7], 1);
        assert_eq!(
            u16::from_be_bytes(bytes[jfif + 8..jfif + 10].try_into().unwrap()),
            144
        );
        let mut decoder = zune_jpeg::JpegDecoder::new(std::io::Cursor::new(&bytes));
        let decoded = decoder.decode().unwrap();
        assert_eq!(decoder.dimensions(), Some((4, 4)));
        assert_eq!(decoded.len(), 4 * 4 * 3);
    }
    #[test]
    fn jpeg_quality_extremes_keep_pinned_reference_chroma_sampling() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/parity/ggplot2/jpeg-controls.json"
        ))
        .unwrap();
        let (tree, mut p) = input();
        for case in fixture["cases"].as_array().unwrap() {
            p.raster_device = Some(RasterDeviceOptions {
                jpeg_quality: case["quality"].as_u64().unwrap() as u8,
                ..Default::default()
            });
            let bytes = encode(&tree, &p, Format::Jpeg).unwrap();
            let mut i = 2;
            loop {
                assert_eq!(bytes[i], 255);
                let marker = bytes[i + 1];
                let length = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
                if marker == 192 {
                    let sampling: Vec<_> = (0..bytes[i + 9] as usize)
                        .map(|channel| bytes[i + 11 + 3 * channel] as u64)
                        .collect();
                    let expected: Vec<_> = case["result"]["sampling"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap())
                        .collect();
                    assert_eq!(sampling, expected);
                    break;
                }
                i += length + 2;
            }
        }
    }
    #[test]
    fn encoder_output_budget_and_invalid_options_reject() {
        let (tree, mut p) = input();
        p.max_output_bytes = 16;
        for format in [Format::Jpeg, Format::Tiff, Format::Bmp] {
            assert_eq!(
                encode(&tree, &p, format).unwrap_err().code,
                DiagnosticCode::ResourceLimit
            );
        }
        p.raster_device = Some(RasterDeviceOptions {
            jpeg_quality: 101,
            ..Default::default()
        });
        assert_eq!(
            encode(&tree, &p, Format::Jpeg).unwrap_err().code,
            DiagnosticCode::Validation
        );
    }
}
