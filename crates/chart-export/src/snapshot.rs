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
use serde_json::{Value, json};
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
    /// Exact immutable displayed input when publication explicitly bypassed relayout.
    pub displayed_layout: Option<Arc<LaidOutChart>>,
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
impl Reproducibility {
    /// Emit captured definition/state/profile, exact revisions/font hashes and dependency identities.
    pub fn manifest(&self) -> Value {
        json!({"version":1,"displayed":displayed_manifest(self.displayed_layout.as_deref()),"engines":self.engines,"stamp":self.stamp,"origin_scene":self.origin_scene,"origin_layout":self.origin_layout,"definition":self.definition,"source_epoch":self.source_epoch,"datasets":self.datasets,"state":state_manifest(&self.definition,&self.captured_state),"effective_state":state_manifest(&self.definition,&self.effective_state),"interaction_policy":self.interaction,"fonts":self.fonts.iter().map(|f:&FontManifest|json!({"id":f.id,"revision":f.revision,"sha256":f.sha256})).collect::<Vec<_>>(),"profile":self.profile,"compile_limits":self.compile_limits})
    }
}
pub(crate) fn state_manifest(definition: &ChartDefinition, state: &ChartState) -> Value {
    json!({"revision":state.revision(),"viewport_revision":state.viewport_revision(),"viewport":state.viewport(),"committed_viewport":state.committed_viewport(),"windows":state.axis_windows(),"hidden_layers":definition.layers.iter().filter(|l|!state.is_visible(l.id)).map(|l|l.id).collect::<Vec<_>>(),"interaction":state.interaction_snapshot(),"hover":state.hover(),"focus":state.focus(),"active_gesture":state.active_gesture(),"effective_annotations":state.annotations(definition)})
}
pub(crate) fn displayed_manifest(layout: Option<&chart_core::layout::LaidOutChart>) -> Value {
    layout.map_or(Value::Null,|l|json!({"unit_policy":"one scene unit per point","scene":l.scene().items(),"guides":l.guide_presentation()}))
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
impl ExportArtifact {
    /// Explicit host filesystem save of already encoded bytes, with the original I/O error.
    /// Parent directory creation and path selection remain the application's responsibility.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, &self.bytes)
    }
}
/// Immutable publication layout/resources built synchronously from a coherent source snapshot.
/// Clone shares the retained figure; later data/definition/annotation edits cannot enter it.
#[derive(Clone)]
pub struct FigureSnapshot(Arc<Figure>);
struct Figure {
    extensions: Arc<chart_core::grammar::ExtensionRegistry>,
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
        extensions.validate_portable_interpolations(definition)?;
        extensions.validate_portable_hierarchies(definition)?;
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
        let laid_out = if let Some(displayed) = &request.displayed_layout {
            displayed.clone()
        } else {
            let prepared = Arc::new(Compiler::with_extensions(extensions).prepare(
                &effective_definition,
                &source,
                &effective_state,
                request.compile_limits,
            )?);
            Arc::new(layout(prepared, &profile.layout, &fonts)?)
        };
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
                guide: None,
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
                displayed_layout: request.displayed_layout.clone(),
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
                extensions: request.extensions.clone(),
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
            let mut diagnostics = self.0.layout.diagnostics().to_vec();
            let bytes = match format {
                Format::Svg if profile.text == TextMode::Preserve => {
                    crate::svg::build(&self.0.scene, &self.0.fonts, profile, true)?.into_bytes()
                }
                Format::Svg => {
                    crate::svg::outline(&self.0.tree, &self.0.scene, &self.0.fonts, profile)?
                }
                Format::Pdf => {
                    crate::encode::pdf(&self.0.scene, &self.0.tree, &self.0.fonts, profile)?
                }
                Format::Png => crate::encode::png(&self.0.tree, profile)?,
                Format::PostScript | Format::Eps => {
                    let (bytes, omitted) = crate::devices_vector::postscript(
                        &self.0.scene,
                        &self.0.tree,
                        profile,
                        format == Format::Eps,
                    )?;
                    if omitted > 0 {
                        let mut warning = error(
                            DiagnosticCode::ExportFidelity,
                            format!(
                                "Explicit PostScript policy omitted {omitted} partially transparent paints."
                            ),
                        );
                        warning.severity = chart_core::Severity::Warning;
                        diagnostics.push(warning);
                    }
                    bytes
                }
                Format::Emf => {
                    let (bytes, omitted) =
                        crate::devices_metafile::encode(&self.0.scene, &self.0.tree, profile)?;
                    if omitted > 0 {
                        let mut warning = error(
                            DiagnosticCode::ExportFidelity,
                            format!(
                                "Explicit EMF policy omitted {omitted} partially transparent paints."
                            ),
                        );
                        warning.severity = chart_core::Severity::Warning;
                        diagnostics.push(warning);
                    }
                    bytes
                }
                Format::PicTeX => {
                    if profile.text == TextMode::Outline {
                        return Err(error(
                            DiagnosticCode::ExportFidelity,
                            "Historical PicTeX uses device fonts and cannot promise supplied-font outlines.",
                        ));
                    }
                    let bytes = crate::devices_pictex::encode(
                        &self.0.scene,
                        &self.0.tree,
                        &self.0.fonts,
                        profile,
                    )?;
                    let mut warning = error(
                        DiagnosticCode::ExportFidelity,
                        "Historical PicTeX emits monochrome outlines, ignores fill and ordinary linewidth, and uses Computer Modern device fonts; non-right-angle text rotation is not retained.",
                    );
                    warning.severity = chart_core::Severity::Warning;
                    diagnostics.push(warning);
                    bytes
                }
                Format::Jpeg | Format::Tiff | Format::Bmp => {
                    crate::devices_raster::encode(&self.0.tree, profile, format)?
                }
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
                diagnostics,
                metadata: self.0.metadata.clone(),
            })
        })();
        result.map_err(|mut e: Diagnostic| {
            e.context.stamp = Some(self.0.scene.stamp());
            e
        })
    }
    /// Encode captured pages as one PDF, PostScript or TIFF document (EPS requires one page).
    /// Each snapshot retains its own reproduction metadata; no current chart is read.
    pub fn export_pages(pages: &[Self], format: Format) -> ChartResult<Vec<u8>> {
        if !matches!(
            format,
            Format::Pdf | Format::PostScript | Format::Eps | Format::Tiff
        ) {
            return Err(error(
                DiagnosticCode::Validation,
                "This multi-page encoder requires PDF, PostScript, EPS or TIFF.",
            ));
        }
        if format == Format::Tiff {
            let input = pages
                .iter()
                .map(|p| (&p.0.tree, &p.0.metadata.profile))
                .collect::<Vec<_>>();
            return crate::devices_raster::tiff_pages(&input);
        }
        for page in pages {
            for font in page.0.fonts.iter() {
                font.check_mode(
                    format,
                    if format == Format::Pdf {
                        page.0.metadata.profile.text
                    } else {
                        TextMode::Outline
                    },
                )?;
            }
        }
        if format == Format::Pdf {
            let input = pages
                .iter()
                .map(|p| (&p.0.scene, &p.0.tree, &p.0.fonts, &p.0.metadata.profile))
                .collect::<Vec<_>>();
            return crate::encode::pdf_pages(&input);
        }
        let input: Vec<_> = pages
            .iter()
            .map(|p| (&p.0.scene, &p.0.tree, &p.0.metadata.profile))
            .collect();
        crate::devices_vector::postscript_pages(&input, format == Format::Eps).map(|v| v.0)
    }
    /// Vector preview of this same publication tree. Glyphs are already positioned/outlined;
    /// a host scales the point viewBox uniformly without native font remeasurement.
    pub fn preview_svg(&self) -> ChartResult<Vec<u8>> {
        crate::svg::outline(
            &self.0.tree,
            &self.0.scene,
            &self.0.fonts,
            &self.0.metadata.profile,
        )
    }
    /// Coherent guide values, labels and configuration, including exact timestamp strings.
    /// The records use this retained publication layout and never resolve a second state.
    pub fn presentation_json(&self) -> ChartResult<String> {
        chart_core::portable::encode(&self.layout().guide_presentation())
    }
    /// Coherent selected semantic guide values and labels.
    pub fn guides_json(&self) -> ChartResult<String> {
        chart_core::portable::encode(
            &json!({"version": 1, "guides": self.layout().guide_snapshots()}),
        )
    }
    /// Owned versioned scene values with exact targets, font identities and diagnostics.
    pub fn scene_json(&self) -> ChartResult<String> {
        // The publication adds a background and optional decorations around core items.
        // Preserve one target entry per actual publication item, including decorative empties.
        let targets: Vec<_> = std::iter::once(Vec::new())
            .chain(self.layout().targets().iter().cloned())
            .chain(std::iter::repeat_with(Vec::new).take(self.metadata().profile.annotations.len()))
            .collect();
        let item_panels: Vec<_> = std::iter::once(None)
            .chain(self.layout().item_panels().iter().cloned())
            .chain(std::iter::repeat_n(
                None,
                self.metadata().profile.annotations.len(),
            ))
            .collect();
        let panels = self.layout().panels().iter().map(|p|json!({"key":p.key,"row":p.row,"column":p.column,"bounds":p.bounds,"plot":p.chart.plot()})).collect::<Vec<_>>();
        let insets = self
            .layout()
            .insets()
            .iter()
            .map(|i| json!({"id":i.id,"panel":i.panel,"bounds":i.bounds,"plot":i.chart.plot()}))
            .collect::<Vec<_>>();
        let interactions: std::collections::BTreeMap<_, _> = self
            .layout()
            .interactions()
            .iter()
            .map(|(i, v)| (i + 1, v))
            .collect();
        let mut value = json!({"interactions":interactions,"insets":insets,"version":self.scene().wire_version(),"stamp":self.scene().stamp(),"units":self.scene().units(),"bounds":self.scene().bounds(),"items":self.scene().items(),"resources":self.scene().resources(),"targets":targets,"item_panels":item_panels,"panels":panels,"diagnostics":self.layout().diagnostics(),"fonts":self.metadata().fonts.iter().map(|f|json!({"id":f.id,"revision":f.revision,"sha256":f.sha256})).collect::<Vec<_>>() });
        let hierarchies = self.layout().hierarchy_snapshots()?;
        if !hierarchies.is_empty() {
            value["version"] = json!(15);
            value["hierarchies"] = json!({"version": 1, "snapshots": hierarchies});
        }
        chart_core::portable::encode(&value)
    }
}
/// Fully clipped circular points require no backend coordinate conversion. Keep
/// the original scene intact, including its retained semantic geometry.
pub(crate) fn point_is_clipped(item: &SceneItem, bounds: Rect) -> bool {
    let Primitive::Point { center, radius, .. } = &item.primitive else {
        return false;
    };
    let clip = item.clip.unwrap_or(bounds);
    clip.width() == 0.
        || clip.height() == 0.
        || center.x() + radius < clip.origin().x()
        || center.x() - radius > clip.max_x()
        || center.y() + radius < clip.origin().y()
        || center.y() - radius > clip.max_y()
}

pub(crate) fn command_points(
    commands: &[PathCommand],
) -> impl Iterator<Item = chart_core::Point> + '_ {
    commands.iter().flat_map(|c| -> Vec<chart_core::Point> {
        match *c {
            PathCommand::MoveTo(a) | PathCommand::LineTo(a) => vec![a],
            PathCommand::QuadraticTo(a, b) => vec![a, b],
            PathCommand::CubicTo(a, b, c) => vec![a, b, c],
            PathCommand::Close => vec![],
        }
    })
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
        if point_is_clipped(item, scene.bounds()) {
            continue;
        }
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
                    for p in command_points(&chart_core::typography::placed_outlines(
                        run, *origin, *rotation,
                    )?) {
                        point(p)?;
                    }
                }
                Primitive::Rule { from, to, stroke } => {
                    point(*from)?;
                    point(*to)?;
                    profile.f32(stroke.width)?;
                }
                Primitive::RasterImage { bounds, .. }
                | Primitive::Rectangle { bounds, .. }
                | Primitive::GradientRectangle { bounds, .. }
                | Primitive::SampledGradientRectangle { bounds, .. } => rect(*bounds)?,
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
                    for p in command_points(commands) {
                        point(p)?;
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

/// One compiled guide plan with retained target publication resources. Hosts own clocks;
/// sampling performs no source compilation or layout and can outlive an output handle.
#[derive(Clone)]
pub struct FigureTransition {
    target: FigureSnapshot,
    plan: chart_core::layout::LayoutGuideTransition,
}
impl FigureSnapshot {
    /// Compile a transition from an immutable prior (possibly interrupted) figure.
    pub fn guide_transition(&self, previous: &FigureSnapshot) -> ChartResult<FigureTransition> {
        Ok(FigureTransition {
            target: self.clone(),
            plan: chart_core::layout::LayoutGuideTransition::new(
                previous.layout().clone(),
                self.layout().clone(),
                self.metadata().profile.layout.limits,
            )?,
        })
    }
}
impl FigureTransition {
    /// Sample an explicit fraction in `[0, 1]`. Passing 1 implements reduced motion;
    /// dropping this owner cancels future samples without affecting already captured figures.
    pub fn sample(&self, fraction: f64) -> ChartResult<FigureSnapshot> {
        let layout = self.plan.sample(fraction)?;
        let p = layout.prepared();
        crate::FigureRequest::new(
            p.definition().clone(),
            p.source().clone(),
            self.target.metadata().effective_state.clone(),
            self.target.0.fonts.clone(),
            self.target.metadata().profile.clone(),
            chart_core::state::InteractionCapture::ALL,
        )?
        .with_extensions(self.target.0.extensions.clone())
        .with_compile_limits(self.target.metadata().compile_limits)
        .with_displayed_layout(layout)?
        .prepare()
    }
}
