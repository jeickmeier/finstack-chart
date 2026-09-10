//! Cheap immutable input acquisition, separate from publication preparation/encoding.
use crate::{
    FigureSnapshot, FontManifest, FontResources, PublicationProfile, Reproducibility, error,
};
use chart_core::{
    ChartResult, DiagnosticCode, SceneStamp,
    data::{SnapshotHandle, StoreSnapshot},
    grammar::{ChartDefinition, ExtensionRegistry},
    state::{ChartState, InteractionCapture},
};
use serde_json::{Value, json};
use std::sync::Arc;

/// Owned inputs for a later publication worker. Capturing clones handles/definition/state,
/// not source values; it does no numeric preparation, layout, shaping, encoding or I/O.
#[derive(Clone)]
pub struct FigureRequest {
    pub(crate) definition: ChartDefinition,
    pub(crate) source: SnapshotHandle<StoreSnapshot>,
    pub(crate) state: ChartState,
    pub(crate) fonts: FontResources,
    pub(crate) profile: PublicationProfile,
    pub(crate) extensions: Arc<ExtensionRegistry>,
    pub(crate) interaction: InteractionCapture,
    pub(crate) origin_scene: Option<SceneStamp>,
    pub(crate) origin_layout: Option<chart_core::layout::LayoutRequest>,
    pub(crate) compile_limits: chart_core::grammar::CompileLimits,
}
impl FigureRequest {
    /// Capture the exact supplied coherent source and state. Source validation/computation
    /// runs later in prepare; this step validates resource/profile availability only.
    pub fn new(
        definition: ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        state: ChartState,
        fonts: FontResources,
        profile: PublicationProfile,
        interaction: InteractionCapture,
    ) -> ChartResult<Self> {
        source.get()?;
        profile.validate()?;
        fonts.get(&profile.layout.font)?;
        Ok(Self {
            definition,
            source,
            state,
            fonts,
            profile,
            interaction,
            extensions: Arc::new(ExtensionRegistry::new()),
            origin_scene: None,
            origin_layout: None,
            compile_limits: Default::default(),
        })
    }
    /// Capture explicit numeric preparation budgets for the later publication worker.
    pub fn with_compile_limits(mut self, limits: chart_core::grammar::CompileLimits) -> Self {
        self.compile_limits = limits;
        self
    }
    /// Retain known immutable extension implementations for the later worker.
    pub fn with_extensions(mut self, extensions: Arc<ExtensionRegistry>) -> Self {
        self.extensions = extensions;
        self
    }
    /// Label a request derived from an actually presented scene. Public source/definition
    /// identities must agree; the captured inspection state may have newer overlay revisions.
    pub fn with_origin_scene(mut self, stamp: SceneStamp) -> ChartResult<Self> {
        let source = self.source.get()?;
        if stamp.store != source.revision() || stamp.definition != self.definition.revision {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Presented capture stamp disagrees with captured data or definition.",
            ));
        }
        self.origin_scene = Some(stamp);
        Ok(self)
    }
    /// Retain the acknowledged destination policy as provenance, independent of output sizing.
    pub fn with_origin_layout(mut self, layout: chart_core::layout::LayoutRequest) -> Self {
        self.origin_layout = Some(layout);
        self
    }
    /// Original immutable source; later source retention does not mutate this handle.
    pub fn source(&self) -> &SnapshotHandle<StoreSnapshot> {
        &self.source
    }
    /// Captured publication policy, including fonts, themes, dimensions and quality.
    pub fn profile(&self) -> &PublicationProfile {
        &self.profile
    }
    /// Rebuild the exact captured data at publication resolution, with no native density loss.
    pub fn prepare(&self) -> ChartResult<FigureSnapshot> {
        FigureSnapshot::capture_request(self)
    }
    /// Portable reproducibility metadata, including full profile and definition but not data/font bytes.
    pub fn manifest(&self) -> ChartResult<Value> {
        let data = self.source.get()?;
        Ok(
            json!({"version":1,"origin_scene":self.origin_scene,"origin_layout":self.origin_layout,"definition":self.definition,"source_epoch":data.epoch(),"store":data.revision(),"datasets":data.datasets().map(|d|json!({"version":d.version(),"schema":d.schema()})).collect::<Vec<_>>(),"state":state_manifest(&self.definition,&self.state),"interaction_policy":self.interaction,"profile":self.profile,"compile_limits":self.compile_limits,"fonts":self.fonts.iter().map(|f|json!({"id":f.descriptor.id,"revision":f.descriptor.revision,"sha256":f.hash})).collect::<Vec<_>>() }),
        )
    }
    pub(crate) fn charge(&self) -> ChartResult<(usize, usize)> {
        let data = self.source.get()?;
        let mut rows = 0usize;
        let mut bytes = 0usize;
        for d in data.datasets() {
            rows = rows.checked_add(d.len()).ok_or_else(budget)?;
            for chunk in d.chunks() {
                bytes = bytes
                    .checked_add(chunk.batch().payload_bytes())
                    .ok_or_else(budget)?;
            }
            // Charge ordinal and exact-key lookup capacity separately from column payload.
            bytes = bytes
                .checked_add(d.len().checked_mul(72).ok_or_else(budget)?)
                .ok_or_else(budget)?;
        }
        for font in self.fonts.iter() {
            bytes = bytes.checked_add(font.bytes.len()).ok_or_else(budget)?;
        }
        bytes = bytes
            .checked_add(
                serde_json::to_vec(&self.manifest()?)
                    .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?
                    .len(),
            )
            .ok_or_else(budget)?;
        Ok((rows, bytes))
    }
}
fn budget() -> chart_core::Diagnostic {
    error(
        DiagnosticCode::ResourceLimit,
        "Captured input accounting overflowed the export budget.",
    )
}
pub(crate) fn state_manifest(definition: &ChartDefinition, state: &ChartState) -> Value {
    json!({"revision":state.revision(),"viewport_revision":state.viewport_revision(),"viewport":state.viewport(),"committed_viewport":state.committed_viewport(),"windows":state.axis_windows(),"hidden_layers":definition.layers.iter().filter(|l|!state.is_visible(l.id)).map(|l|l.id).collect::<Vec<_>>(),"interaction":state.interaction_snapshot(),"hover":state.hover(),"focus":state.focus(),"active_gesture":state.active_gesture(),"effective_annotations":state.annotations(definition)})
}
impl Reproducibility {
    /// Emit captured definition/state/profile, exact revisions/font hashes and dependency identities.
    pub fn manifest(&self) -> Value {
        json!({"version":1,"engines":self.engines,"stamp":self.stamp,"origin_scene":self.origin_scene,"origin_layout":self.origin_layout,"definition":self.definition,"source_epoch":self.source_epoch,"datasets":self.datasets,"state":state_manifest(&self.definition,&self.captured_state),"effective_state":state_manifest(&self.definition,&self.effective_state),"interaction_policy":self.interaction,"fonts":self.fonts.iter().map(|f:&FontManifest|json!({"id":f.id,"revision":f.revision,"sha256":f.sha256})).collect::<Vec<_>>(),"profile":self.profile,"compile_limits":self.compile_limits})
    }
}
