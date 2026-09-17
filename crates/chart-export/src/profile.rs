use crate::error;
use chart_core::layout::LayoutRequest;
use chart_core::scene::{Color, SceneItem};
use chart_core::services::{ResourceDescriptor, Units};
use chart_core::{ChartResult, DiagnosticCode, Rect, Revision};

/// Physical page size. All publication computation uses 72 points per inch.
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct PageSize {
    width: f64,
    height: f64,
}
impl PageSize {
    /// Construct positive finite point dimensions.
    pub fn points(width: f64, height: f64) -> ChartResult<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
            return Err(error(
                DiagnosticCode::Validation,
                "Page dimensions must be finite and positive.",
            ));
        }
        Ok(Self { width, height })
    }
    /// Construct physical dimensions without consulting a display or device scale.
    pub fn millimeters(width: f64, height: f64) -> ChartResult<Self> {
        Self::points(width / 25.4 * 72., height / 25.4 * 72.)
    }
    /// Width in points.
    pub fn width(self) -> f64 {
        self.width
    }
    /// Height in points.
    pub fn height(self) -> f64 {
        self.height
    }
}
/// Which captured domains are laid out; neither mode changes upstream statistics.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMode {
    /// Honor captured state and named-axis viewports.
    VisibleView,
    /// Clear primary and named-axis viewports, keeping authored domains and layer visibility.
    FullDomain,
}
/// Explicit editing/search tradeoff for vector outputs.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextMode {
    /// SVG embeds full permitted font bytes; PDF embeds subsets and Unicode maps.
    Preserve,
    /// Use shaped vector glyph outlines; logical strings remain in the snapshot/metadata.
    Outline,
}
/// Encoded output representation.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// Vector SVG.
    Svg,
    /// Vector PDF.
    Pdf,
    /// Raster PNG with explicit physical density.
    Png,
    /// Lossy JPEG raster with explicit matte, quality and DPI.
    Jpeg,
    /// TIFF raster with explicit compression, alpha policy and physical resolution.
    Tiff,
    /// Opaque 24-bit BMP raster with physical resolution.
    Bmp,
    /// Retained-vector PostScript Level 3 with supplied-font outlines.
    PostScript,
    /// Single-page encapsulated PostScript with exact physical bounds.
    Eps,
    /// Historical monochrome PicTeX output with Computer Modern device fonts.
    PicTeX,
    /// Single-page enhanced metafile with supplied-font vector outlines.
    Emf,
}
/// Text representation promised by the selected output mode.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextRepresentation {
    /// Plain text embeds fonts; rich runs use exact positioned outlines with logical SVG labels.
    MixedPositionedOutlines,
    /// Logical SVG text with full embedded font bytes; editor shaping can vary.
    EmbeddedFullFonts,
    /// PDF glyphs with embedded subsets and Unicode maps; search/extraction is preserved.
    EmbeddedSubsetFonts,
    /// Positioned vector glyph outlines; search/editable text is absent from the output.
    Outlines,
    /// Historical TeX device fonts; supplied font identity is not retained.
    DeviceFonts,
    /// Raster pixels; no selectable text or embedded font payload.
    Pixels,
}
/// Capabilities of this format/mode; no implicit localized or whole-figure fallback occurs.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportCapabilities {
    /// Native callback paint is unsupported in every headless format.
    pub native_painters: bool,
    /// Supported chart marks remain vectors in SVG/PDF.
    pub vector_marks: bool,
    /// Explicit text storage/editing tradeoff.
    pub text: TextRepresentation,
}
impl Format {
    /// Report representation before encoding. PNG is intentionally raster output.
    pub fn capabilities(self, text: TextMode) -> ExportCapabilities {
        let representation = match (self, text) {
            (Self::PicTeX, _) => TextRepresentation::DeviceFonts,
            (Self::Png | Self::Jpeg | Self::Tiff | Self::Bmp, _) => TextRepresentation::Pixels,
            (Self::PostScript | Self::Eps | Self::Emf, _) | (_, TextMode::Outline) => {
                TextRepresentation::Outlines
            }
            (Self::Svg, _) => TextRepresentation::EmbeddedFullFonts,
            (Self::Pdf, _) => TextRepresentation::EmbeddedSubsetFonts,
        };
        ExportCapabilities {
            native_painters: false,
            vector_marks: matches!(
                self,
                Self::Svg | Self::Pdf | Self::PostScript | Self::Eps | Self::PicTeX | Self::Emf
            ),
            text: representation,
        }
    }
}
/// Owned settings captured before publication layout. No native controls are exported.
#[derive(serde::Serialize, Clone, Debug)]
pub struct PublicationProfile {
    /// Physical output page. Actual layout bounds are derived from this size.
    pub page: PageSize,
    /// Point-based guide/font/scale policies; `bounds` is replaced by `page` on capture.
    pub layout: LayoutRequest,
    /// Captured visible interval or full trained domain, preserving layer visibility.
    pub view: ViewMode,
    /// Vector text policy; PNG always paints the same shaped tree.
    pub text: TextMode,
    /// Raster dots per inch; uniform scaling, nearest integer dimensions.
    pub dpi: u32,
    /// Optional additional raster-device controls; absence preserves legacy profile bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_device: Option<crate::RasterDeviceOptions>,
    /// Explicit retained-vector device policies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_device: Option<crate::VectorDeviceOptions>,
    /// Figure background; PNG correctly encodes straight alpha, including transparency.
    pub background: chart_core::color::Paint,
    /// Revision of the supplied point-space decoration/annotation items.
    pub annotation_revision: Revision,
    /// Initial portable decorations in figure point coordinates; full figure furniture is WP-13.
    pub annotations: Vec<SceneItem>,
    /// Maximum permitted binary32 coordinate conversion error, in points (0 < value <= 0.25).
    pub precision: f64,
    /// Pre-allocation raster pixel cap.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub max_raster_pixels: u64,
    /// Limit on each returned encoded payload, also used while constructing SVG.
    pub max_output_bytes: usize,
}
impl PublicationProfile {
    /// Plain two-axis point layout with a white page, 300 DPI and text-preserving output.
    pub fn new(page: PageSize, font: ResourceDescriptor) -> ChartResult<Self> {
        Ok(Self {
            page,
            layout: LayoutRequest::new(
                Rect::new(0., 0., page.width, page.height)?,
                Units::Points,
                font,
            ),
            view: ViewMode::VisibleView,
            text: TextMode::Preserve,
            dpi: 300,
            raster_device: None,
            vector_device: None,
            background: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            }
            .into(),
            annotation_revision: Revision::INITIAL,
            annotations: vec![],
            precision: 0.01,
            max_raster_pixels: 32_000_000,
            max_output_bytes: 64 * 1024 * 1024,
        })
    }
    pub(crate) fn validate(&self) -> ChartResult<()> {
        if let Some(options) = &self.raster_device {
            options.validate()?;
        }
        if self.layout.units != Units::Points {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Publication layout requires points, not native pixel metrics.",
            ));
        }
        if !self.precision.is_finite()
            || self.precision <= 0.
            || self.precision > 0.25
            || self.dpi == 0
            || self.max_output_bytes == 0
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Require positive DPI/output budget and a point precision in (0, 0.25].",
            ));
        }
        self.f32(self.page.width)?;
        self.f32(self.page.height)?;
        Ok(())
    }
    pub(crate) fn f32(&self, value: f64) -> ChartResult<f32> {
        let f = value as f32;
        if !f.is_finite()
            || (value != 0. && f == 0.)
            || (value - f64::from(f)).abs() > self.precision
        {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Publication binary32 projection exceeds the declared point precision.",
            ));
        }
        Ok(f)
    }
    /// Checked raster dimensions, before allocating the pixel buffer.
    pub fn raster_dimensions(&self) -> ChartResult<(u32, u32)> {
        self.validate()?;
        let w = (self.page.width * f64::from(self.dpi) / 72.).round();
        let h = (self.page.height * f64::from(self.dpi) / 72.).round();
        if !w.is_finite()
            || !h.is_finite()
            || w < 1.
            || h < 1.
            || w > f64::from(u32::MAX)
            || h > f64::from(u32::MAX)
            || w * h > self.max_raster_pixels as f64
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Raster dimensions are empty, overflow, or exceed the pixel budget.",
            ));
        }
        let (width, height) = (w as u32, h as u32);
        if u64::from(width) * u64::from(height) > self.max_raster_pixels {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Raster exceeds the exact pixel budget.",
            ));
        }
        Ok((width, height))
    }
}
