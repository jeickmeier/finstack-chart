use crate::{FontResources, Format, PublicationProfile, TextMode, ViewMode, error};
use chart_core::data::{DatasetVersion, SnapshotHandle, StoreSnapshot};
use chart_core::grammar::{ChartDefinition, CompileLimits, Compiler};
use chart_core::layout::{LaidOutChart, layout};
use chart_core::scene::{PathCommand, Primitive, Scene, SceneItem};
use chart_core::state::ChartState;
use chart_core::{
    ChartResult, Diagnostic, DiagnosticCode, Rect, ResourceId, Revision, SceneStamp, SchemaVersion,
    SourceEpoch,
};
use std::sync::Arc;

/// Exact font bytes identity alongside its reproducibility digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontManifest {
    /// Stable identity.
    pub id: ResourceId,
    /// Captured revision.
    pub revision: Revision,
    /// Lowercase SHA-256 of the supplied bytes.
    pub sha256: String,
}
/// Reproduction inputs, returned with bytes. This is not the WP-09 portable wire envelope.
#[derive(Clone, Debug)]
pub struct Reproducibility {
    /// Complete authored definition captured before any later edits.
    pub definition: ChartDefinition,
    /// Captured numeric preparation budgets.
    pub compile_limits: CompileLimits,
    /// Explicit state inclusion policy; legacy direct captures preserve all state.
    pub interaction: chart_core::state::InteractionCapture,
    /// Actual presented scene used by the host, when capture came from presentation.
    pub origin_scene: Option<SceneStamp>,
    /// Actual original destination policy, independently of publication point dimensions.
    pub origin_layout: Option<chart_core::layout::LayoutRequest>,
    /// Exact encoding/font engine versions and crate version.
    pub engines: String,
    /// Effective publication scene stamp.
    pub stamp: SceneStamp,
    /// Original captured interaction state, before the explicit full-domain policy.
    pub captured_state: ChartState,
    /// Effective state actually prepared after inclusion and full-domain policies.
    pub effective_state: ChartState,
    /// Coherent source transaction epoch.
    pub source_epoch: SourceEpoch,
    /// Captured per-dataset versions and schema versions.
    pub datasets: Vec<(DatasetVersion, SchemaVersion)>,
    /// Exact fonts held by the snapshot.
    pub fonts: Vec<FontManifest>,
    /// Captured physical dimensions, viewport policy, annotations, text/quality settings.
    pub profile: PublicationProfile,
}
/// Returned publication bytes, diagnostics and reproduction inputs; saving belongs to the host.
#[derive(Clone, Debug)]
pub struct ExportArtifact {
    /// Encoded representation.
    pub format: Format,
    /// Format and text-mode representation guarantees.
    pub capabilities: crate::ExportCapabilities,
    /// Complete owned payload.
    pub bytes: Vec<u8>,
    /// Core preparation/layout diagnostics, including explicit no-space pressure.
    pub diagnostics: Vec<Diagnostic>,
    /// Captured reproducibility settings and exact identities.
    pub metadata: Reproducibility,
}
/// Immutable publication layout/resources built synchronously from a coherent source snapshot.
/// Clone shares the retained figure; later data/definition/annotation edits cannot enter it.
#[derive(Clone)]
pub struct FigureSnapshot(Arc<Figure>);
struct Figure {
    layout: Arc<LaidOutChart>,
    scene: Scene,
    fonts: FontResources,
    tree: usvg::Tree,
    metadata: Reproducibility,
}
impl FigureSnapshot {
    /// Capture supported definition/theme/state, data, font bytes, annotations and output quality,
    /// then rebuild shared layout in physical points. No screen envelope is used as source data.
    pub fn capture(
        definition: &ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        fonts: FontResources,
        profile: PublicationProfile,
    ) -> ChartResult<Self> {
        Self::capture_with_extensions(
            definition,
            source,
            state,
            fonts,
            profile,
            Arc::new(chart_core::grammar::ExtensionRegistry::new()),
        )
    }
    /// Capture with explicitly retained versioned implementations; native-only paint rejects.
    pub fn capture_with_extensions(
        definition: &ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        fonts: FontResources,
        profile: PublicationProfile,
        extensions: Arc<chart_core::grammar::ExtensionRegistry>,
    ) -> ChartResult<Self> {
        crate::FigureRequest::new(
            definition.clone(),
            source,
            state.clone(),
            fonts,
            profile,
            chart_core::state::InteractionCapture::ALL,
        )?
        .with_extensions(extensions)
        .prepare()
    }
    /// Prepare one previously acquired immutable request on a caller-selected executor.
    pub fn capture_request(request: &crate::FigureRequest) -> ChartResult<Self> {
        let definition = &request.definition;
        let source = request.source.clone();
        let fonts = request.fonts.clone();
        let mut profile = request.profile.clone();
        let extensions = request.extensions.clone();
        let state = &request.state;
        profile.validate()?;
        profile.layout.bounds = Rect::new(0., 0., profile.page.width(), profile.page.height())?;
        let captured_profile = profile.clone();
        let captured_state = state.clone();
        let mut effective_definition = definition.clone();
        let mut effective_state = state.capture_interaction(request.interaction);
        if profile.view == ViewMode::FullDomain {
            effective_state = effective_state.capture_full_domain();
            for axis in &mut effective_definition.axes {
                axis.viewport = None;
            }
            for axis in &mut profile.layout.axes {
                axis.viewport = None;
            }
        }
        fonts.get(&profile.layout.font)?;
        let prepared = Arc::new(Compiler::with_extensions(extensions).prepare(
            &effective_definition,
            &source,
            &effective_state,
            request.compile_limits,
        )?);
        let laid_out = Arc::new(layout(prepared, &profile.layout, &fonts)?);
        let result = (|| {
            let mut resources: Vec<_> = fonts.iter().map(|f| f.descriptor).collect();
            resources.sort_by_key(|f| f.id);
            let count = laid_out
                .scene()
                .items()
                .len()
                .checked_add(profile.annotations.len())
                .and_then(|n| n.checked_add(1));
            if count.is_none_or(|n| n > profile.layout.limits.max_items) {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Figure plus annotation item budget exceeded.",
                ));
            }
            let mut items = Vec::with_capacity(count.unwrap_or(0));
            items.push(SceneItem {
                layer: None,
                clip: None,
                primitive: Primitive::Rectangle {
                    bounds: profile.layout.bounds,
                    fill: profile.background.resolve(),
                },
            });
            items.extend_from_slice(laid_out.scene().items());
            items.extend_from_slice(&profile.annotations);
            let scene = Scene::new(
                laid_out.scene().stamp(),
                profile.layout.units,
                profile.layout.bounds,
                &items,
                &resources,
                profile.layout.limits,
            )?;
            preflight(&scene, &fonts, &profile)?;
            let text = crate::svg::build(&scene, &fonts, &profile, false)?;
            let tree = usvg::Tree::from_str(&text, &fonts.options())
                .map_err(|e| error(DiagnosticCode::ExportFidelity, e.to_string()))?;
            // Fail instead of accepting a parser's silent text/font substitution.
            for (i, item) in scene.items().iter().enumerate() {
                if let Primitive::Text { font, text, .. } = &item.primitive {
                    if text.is_empty() {
                        continue;
                    }
                    let descriptor = scene
                        .resources()
                        .iter()
                        .find(|r| r.id == *font)
                        .ok_or_else(|| {
                            error(DiagnosticCode::MissingResource, "Text resource is absent.")
                        })?;
                    let resource = fonts.get(descriptor)?;
                    let result = (|| {
                        let Some(usvg::Node::Text(node)) = tree.node_by_id(&format!("item-{i}"))
                        else {
                            return Err(error(
                                DiagnosticCode::ExportFidelity,
                                "Renderer omitted a nonempty text run.",
                            ));
                        };
                        if node
                            .layouted()
                            .iter()
                            .flat_map(|s| &s.positioned_glyphs)
                            .any(|g| g.id.0 == 0 || !fonts.matches(resource, g.font))
                        {
                            return Err(error(
                                DiagnosticCode::MissingResource,
                                "Renderer substituted a missing/different glyph.",
                            ));
                        }
                        Ok(())
                    })();
                    result.map_err(|mut e| {
                        e.context.layer = item.layer;
                        e.context.resource = Some(*font);
                        e.context.resource_revision = Some(descriptor.revision);
                        e
                    })?;
                }
            }
            let data = source.get()?;
            let metadata = Reproducibility {
                definition: definition.clone(),
                compile_limits: request.compile_limits,
                interaction: request.interaction,
                origin_scene: request.origin_scene,
                origin_layout: request.origin_layout.clone(),
                engines: format!(
                    "chart-export {}; chart-text 0.1.0; usvg/resvg 0.48.1; harfrust 0.12.0; krilla 0.8.2; skrifa 0.44.0/0.42.1; PNG 0.17.16",
                    env!("CARGO_PKG_VERSION")
                ),
                stamp: scene.stamp(),
                captured_state,
                effective_state,
                source_epoch: data.epoch(),
                datasets: data
                    .datasets()
                    .map(|d| (d.version(), d.schema().version()))
                    .collect(),
                fonts: fonts
                    .iter()
                    .map(|f| FontManifest {
                        id: f.descriptor.id,
                        revision: f.descriptor.revision,
                        sha256: f.hash.clone(),
                    })
                    .collect(),
                profile: captured_profile,
            };
            Ok(Self(Arc::new(Figure {
                layout: laid_out.clone(),
                scene,
                fonts,
                tree,
                metadata,
            })))
        })();
        result.map_err(|mut e| {
            e.context.stamp = Some(laid_out.scene().stamp());
            e
        })
    }
    /// Core publication layout retains exact prepared semantics and source/aggregate targets.
    pub fn layout(&self) -> &Arc<LaidOutChart> {
        &self.0.layout
    }
    /// Publication scene, including page background and portable decoration items.
    pub fn scene(&self) -> &Scene {
        &self.0.scene
    }
    /// Captured reproduction inputs; no mutable access exists.
    pub fn metadata(&self) -> &Reproducibility {
        &self.0.metadata
    }
    /// Encode this captured figure without consulting current application state.
    pub fn export(&self, format: Format) -> ChartResult<ExportArtifact> {
        let profile = &self.0.metadata.profile;
        let result = (|| {
            for font in self.0.fonts.iter() {
                font.check_mode(format, profile.text)?;
            }
            let bytes = match format {
                Format::Svg if profile.text == TextMode::Preserve => {
                    crate::svg::build(&self.0.scene, &self.0.fonts, profile, true)?.into_bytes()
                }
                Format::Svg => crate::svg::outline(&self.0.tree, profile)?,
                Format::Pdf => {
                    crate::encode::pdf(&self.0.scene, &self.0.tree, &self.0.fonts, profile)?
                }
                Format::Png => crate::encode::png(&self.0.tree, profile)?,
            };
            Ok(ExportArtifact {
                format,
                capabilities: {
                    let mut c = format.capabilities(profile.text);
                    if format == Format::Svg
                        && profile.text == TextMode::Preserve
                        && self
                            .0
                            .scene
                            .items()
                            .iter()
                            .any(|i| matches!(i.primitive, Primitive::GlyphRun { .. }))
                    {
                        c.text = crate::TextRepresentation::MixedPositionedOutlines;
                    }
                    c
                },
                bytes: crate::encode::bounded(bytes, profile)?,
                diagnostics: self.0.layout.diagnostics().to_vec(),
                metadata: self.0.metadata.clone(),
            })
        })();
        result.map_err(|mut e: Diagnostic| {
            e.context.stamp = Some(self.0.scene.stamp());
            e
        })
    }
    /// Vector preview of this same publication tree. Glyphs are already positioned/outlined;
    /// a host scales the point viewBox uniformly without native font remeasurement.
    pub fn preview_svg(&self) -> ChartResult<Vec<u8>> {
        crate::svg::outline(&self.0.tree, &self.0.metadata.profile)
    }
}
fn preflight(
    scene: &Scene,
    fonts: &FontResources,
    profile: &PublicationProfile,
) -> ChartResult<()> {
    let point = |p: chart_core::Point| -> ChartResult<()> {
        profile.f32(p.x())?;
        profile.f32(p.y())?;
        Ok(())
    };
    let rect = |r: Rect| -> ChartResult<()> {
        point(r.origin())?;
        profile.f32(r.width())?;
        profile.f32(r.height())?;
        profile.f32(r.origin().x() + r.width())?;
        profile.f32(r.origin().y() + r.height())?;
        Ok(())
    };
    for item in scene.items() {
        let result = (|| {
            rect(item.clip.unwrap_or(scene.bounds()))?;
            match &item.primitive {
                Primitive::VectorPath {
                    geometry, stroke, ..
                }
                | Primitive::ShapePath {
                    geometry, stroke, ..
                } => {
                    crate::svg::lower_path(geometry, profile)?;
                    if let Some(stroke) = stroke {
                        profile.f32(stroke.width)?;
                    }
                }
                Primitive::NativePaint { painter, .. } => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        format!(
                            "Native painter {} version {} has no SVG/PDF/PNG representation; replace it with portable geometry.",
                            painter.id,
                            painter.version.get()
                        ),
                    ));
                }
                Primitive::GlyphRun {
                    origin,
                    rotation,
                    run,
                    ..
                } => {
                    point(*origin)?;
                    profile.f32(run.font_size)?;
                    fonts.get(&run.font)?.validate_text(&run.text)?;
                    for glyph in &run.glyphs {
                        point(glyph.position)?;
                    }
                    for c in chart_core::typography::placed_outlines(run, *origin, *rotation)? {
                        match c {
                            PathCommand::MoveTo(a) | PathCommand::LineTo(a) => point(a)?,
                            PathCommand::QuadraticTo(a, b) => {
                                point(a)?;
                                point(b)?;
                            }
                            PathCommand::CubicTo(a, b, c) => {
                                point(a)?;
                                point(b)?;
                                point(c)?;
                            }
                            PathCommand::Close => {}
                        }
                    }
                }
                Primitive::Rule { from, to, stroke } => {
                    point(*from)?;
                    point(*to)?;
                    profile.f32(stroke.width)?;
                }
                Primitive::Rectangle { bounds, .. }
                | Primitive::GradientRectangle { bounds, .. } => rect(*bounds)?,
                Primitive::Point { center, radius, .. }
                | Primitive::Symbol { center, radius, .. } => {
                    point(*center)?;
                    profile.f32(*radius)?;
                }
                Primitive::Path { commands, .. }
                | Primitive::FilledPath { commands, .. }
                | Primitive::DashedPath { commands, .. } => {
                    if let Primitive::Path { stroke, .. } | Primitive::DashedPath { stroke, .. } =
                        &item.primitive
                    {
                        profile.f32(stroke.width)?;
                    }
                    if let Primitive::DashedPath { dashes, .. } = &item.primitive {
                        for dash in dashes {
                            profile.f32(*dash)?;
                        }
                    }
                    for c in commands {
                        match c {
                            PathCommand::MoveTo(a) | PathCommand::LineTo(a) => point(*a)?,
                            PathCommand::QuadraticTo(a, b) => {
                                point(*a)?;
                                point(*b)?;
                            }
                            PathCommand::CubicTo(a, b, c) => {
                                point(*a)?;
                                point(*b)?;
                                point(*c)?;
                            }
                            PathCommand::Close => {}
                        }
                    }
                }
                Primitive::Text {
                    text,
                    font,
                    font_size,
                    origin,
                    ..
                } => {
                    point(*origin)?;
                    profile.f32(*font_size)?;
                    let descriptor = scene
                        .resources()
                        .iter()
                        .find(|r| r.id == *font)
                        .ok_or_else(|| {
                            error(DiagnosticCode::MissingResource, "Text resource is absent.")
                        })?;
                    fonts.get(descriptor)?.validate_text(text)?;
                }
            }
            Ok(())
        })();
        result.map_err(|mut e: Diagnostic| {
            e.context.layer = item.layer;
            e
        })?;
    }
    Ok(())
}
