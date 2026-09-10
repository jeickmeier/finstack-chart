//! Reusable publication destination for primary immutable plots and retained charts.
use crate::{
    ExportArtifact, FigureRequest, FontResource, FontResources, Format, PageSize,
    PublicationProfile, TextMode, ViewMode, error,
};
use chart_core::{
    ChartResult, DiagnosticCode, ResourceId, Revision,
    color::Paint,
    plot::Plot,
    runtime::Chart,
    services::{ResourceDescriptor, ResourceKind},
    state::{ChartState, InteractionCapture},
};
use std::sync::Arc;

impl ExportArtifact {
    /// Explicit host filesystem save of already encoded bytes, with the original I/O error.
    /// Parent directory creation and path selection remain the application's responsibility.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, &self.bytes)
    }
}

/// Which coherent live inputs to acquire, independently of projection and interaction policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize)]
pub enum CaptureBasis {
    /// Exact last acknowledged scene; rejects before first presentation.
    #[default]
    Presented,
    /// Current committed definition/data/state, including before first presentation.
    Current,
}
/// Typed optional publication settings; physical page dimensions are always explicit.
#[derive(Clone)]
pub struct ExportOptions {
    page: PageSize,
    layout: chart_core::plot::LayoutOptions,
    decorations: Vec<chart_core::scene::SceneItem>,
    decoration_revision: Revision,
    dpi: u32,
    text: TextMode,
    view: ViewMode,
    background: Paint,
    interaction: InteractionCapture,
    basis: CaptureBasis,
    precision: f64,
    max_raster_pixels: u64,
    max_output_bytes: usize,
}
/// Begin publication settings with the existing 300-DPI, clean visible-view defaults.
pub fn export_options(page: PageSize) -> ExportOptions {
    ExportOptions {
        page,
        layout: chart_core::plot::layout_options(),
        decorations: vec![],
        decoration_revision: Revision::INITIAL,
        dpi: 300,
        text: TextMode::Preserve,
        view: ViewMode::VisibleView,
        background: chart_core::theme::rgb(255, 255, 255).into(),
        interaction: InteractionCapture::default(),
        basis: CaptureBasis::Presented,
        precision: 0.01,
        max_raster_pixels: 32_000_000,
        max_output_bytes: 64 * 1024 * 1024,
    }
}
impl ExportOptions {
    /// Replace the explicit physical page without resetting any other publication option.
    pub fn page(mut self, page: PageSize) -> Self {
        self.page = page;
        self
    }
    /// Configure publication layout using the shared destination options, retaining point units.
    pub fn layout(mut self, layout: chart_core::plot::LayoutOptions) -> Self {
        self.layout = layout;
        self
    }
    /// Add supplied portable point-space decorations with their explicit revision.
    /// Ordinary authored annotations use labels/callout on Plot.
    pub fn decorations(
        mut self,
        revision: Revision,
        items: Vec<chart_core::scene::SceneItem>,
    ) -> Self {
        self.decorations = items;
        self.decoration_revision = revision;
        self
    }

    /// Select an exact coherent live capture basis.
    pub fn basis(mut self, basis: CaptureBasis) -> Self {
        self.basis = basis;
        self
    }
    /// Set physical raster density.
    pub fn dpi(mut self, dpi: u32) -> Self {
        self.dpi = dpi;
        self
    }
    /// Set vector text preservation versus outlines.
    pub fn text(mut self, text: TextMode) -> Self {
        self.text = text;
        self
    }
    /// Select visible or full trained domains independently of capture basis.
    pub fn view(mut self, view: ViewMode) -> Self {
        self.view = view;
        self
    }
    /// Set the output background, including transparent alpha.
    pub fn background(mut self, background: impl Into<Paint>) -> Self {
        self.background = background.into();
        self
    }
    /// Include only the explicitly selected interaction state.
    pub fn interaction(mut self, interaction: InteractionCapture) -> Self {
        self.interaction = interaction;
        self
    }
    /// Set maximum binary32 coordinate conversion error in points.
    pub fn precision(mut self, precision: f64) -> Self {
        self.precision = precision;
        self
    }
    /// Set the pre-allocation raster pixel cap.
    pub fn max_raster_pixels(mut self, pixels: u64) -> Self {
        self.max_raster_pixels = pixels;
        self
    }
    /// Set the encoded payload budget.
    pub fn max_output_bytes(mut self, bytes: usize) -> Self {
        self.max_output_bytes = bytes;
        self
    }
    fn profile(&self, font: ResourceDescriptor) -> ChartResult<PublicationProfile> {
        let mut profile = PublicationProfile::new(self.page, font)?;
        self.layout.apply(&mut profile.layout);
        profile.annotations.clone_from(&self.decorations);
        profile.annotation_revision = self.decoration_revision;
        profile.dpi = self.dpi;
        profile.text = self.text;
        profile.view = self.view;
        profile.background = self.background;
        profile.precision = self.precision;
        profile.max_raster_pixels = self.max_raster_pixels;
        profile.max_output_bytes = self.max_output_bytes;
        Ok(profile)
    }
}
/// Reusable headless destination retaining supplied fonts once for many plots.
#[derive(Clone)]
pub struct Output {
    fonts: FontResources,
    primary: ResourceDescriptor,
}
impl Output {
    /// Primary supplied face for explicit rich-text references.
    pub fn primary_font(&self) -> ResourceDescriptor {
        self.primary
    }
    /// Register another explicitly supplied face, retaining old requests and exact byte identity.
    pub fn register_font(
        &mut self,
        bytes: impl Into<Arc<[u8]>>,
    ) -> ChartResult<ResourceDescriptor> {
        let bytes = bytes.into();
        if let Some(font) = self
            .fonts
            .iter()
            .find(|f| f.bytes.as_ref() == bytes.as_ref())
        {
            return Ok(font.descriptor);
        }
        let id = self
            .fonts
            .iter()
            .map(|f| f.descriptor.id.get())
            .max()
            .unwrap_or_default()
            .checked_add(1)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::RevisionOverflow,
                    "Font resource identities are exhausted.",
                )
            })?;
        let descriptor = ResourceDescriptor {
            id: ResourceId::new(id),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: bytes.len() as u64,
        };
        let mut fonts = self.fonts.iter().cloned().collect::<Vec<_>>();
        fonts.push(FontResource::new(descriptor, bytes)?);
        self.fonts = FontResources::new(fonts)?;
        Ok(descriptor)
    }
    /// Configure one explicitly supplied font without IDs, filesystem I/O or system-font lookup.
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> ChartResult<Self> {
        let bytes = bytes.into();
        let descriptor = ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: bytes.len() as u64,
        };
        Ok(Self {
            fonts: FontResources::new(vec![FontResource::new(descriptor, bytes)?])?,
            primary: descriptor,
        })
    }
    /// Acquire cheap immutable static inputs; no statistics, layout, shaping or encoding occurs.
    pub fn request(&self, plot: &Plot, options: ExportOptions) -> ChartResult<FigureRequest> {
        FigureRequest::new(
            plot.definition().clone(),
            plot.source(),
            ChartState::default(),
            self.fonts.clone(),
            options.profile(self.primary)?,
            options.interaction,
        )
        .map(|r| {
            r.with_extensions(plot.extensions().clone())
                .with_compile_limits(plot.compile_limits())
        })
    }
    /// Acquire a coherent live basis independently of projection and interaction choices.
    pub fn live_request(
        &self,
        chart: &Chart,
        options: ExportOptions,
    ) -> ChartResult<FigureRequest> {
        let mut profile = options.profile(self.primary)?;
        match options.basis {
            CaptureBasis::Current => FigureRequest::new(
                chart.definition().clone(),
                chart.source(),
                chart.state().clone(),
                self.fonts.clone(),
                profile,
                options.interaction,
            )
            .map(|r| {
                r.with_extensions(chart.extensions().clone())
                    .with_compile_limits(chart.compile_limits())
            }),
            CaptureBasis::Presented => {
                let scene = chart.reducer().presented().ok_or_else(|| error(DiagnosticCode::UnsupportedCapability, "Presented capture requires an acknowledged frame; select Current before first presentation."))?;
                if let Some(layout) = chart.painted_layout() {
                    let (bounds, units, font) = (
                        profile.layout.bounds,
                        profile.layout.units,
                        profile.layout.font,
                    );
                    profile.layout = layout.clone();
                    profile.layout.bounds = bounds;
                    profile.layout.units = units;
                    profile.layout.font = font;
                    profile.layout.figure_bounds = None;
                    options.layout.apply(&mut profile.layout);
                }
                let mut request = FigureRequest::new(
                    scene.prepared().definition().clone(),
                    scene.prepared().source().clone(),
                    chart
                        .painted_state()
                        .unwrap_or(scene.prepared().state())
                        .clone(),
                    self.fonts.clone(),
                    profile,
                    options.interaction,
                )?
                .with_extensions(chart.extensions().clone())
                .with_compile_limits(chart.compile_limits())
                .with_origin_scene(scene.scene().stamp())?;
                if let Some(layout) = chart.painted_layout() {
                    request = request.with_origin_layout(layout.clone());
                }
                Ok(request)
            }
        }
    }
    /// Prepare and encode a primary static plot with explicit settings.
    pub fn export(
        &self,
        plot: &Plot,
        format: Format,
        options: ExportOptions,
    ) -> ChartResult<ExportArtifact> {
        self.request(plot, options)?.prepare()?.export(format)
    }
    /// Encode SVG bytes using default settings and explicit physical size.
    pub fn svg(&self, plot: &Plot, page: PageSize) -> ChartResult<Vec<u8>> {
        Ok(self.export(plot, Format::Svg, export_options(page))?.bytes)
    }
    /// Encode PDF bytes using default settings and explicit physical size.
    pub fn pdf(&self, plot: &Plot, page: PageSize) -> ChartResult<Vec<u8>> {
        Ok(self.export(plot, Format::Pdf, export_options(page))?.bytes)
    }
    /// Encode PNG bytes using default settings and explicit physical size.
    pub fn png(&self, plot: &Plot, page: PageSize) -> ChartResult<Vec<u8>> {
        Ok(self.export(plot, Format::Png, export_options(page))?.bytes)
    }
}
