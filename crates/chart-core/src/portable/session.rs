use super::error;
use super::*;
use crate::{data::*, grammar::*, state::*, transaction::*, *};
use serde_json::json;
use std::sync::Arc;

/// Owned synchronous portable core session. Hosts own its lifetime and interpreter detachment.
pub struct Session {
    definition: ChartEnvelope,
    store: DataStore,
    reducer: ActionReducer,
    compiler: Compiler,
    inspectors: Vec<crate::inspection::Inspector>,
}
impl Session {
    /// Decode and validate versioned chart/data inputs, including builtin operations and schemas.
    pub fn new(chart: &str, data: &str) -> ChartResult<Self> {
        Self::with_extensions(chart, data, Arc::new(ExtensionRegistry::new()))
    }
    /// Execute only registered portable extensions; arbitrary callbacks are never decoded.
    pub fn with_extensions(
        chart: &str,
        data: &str,
        extensions: Arc<ExtensionRegistry>,
    ) -> ChartResult<Self> {
        let definition: ChartEnvelope = decode(chart)?;
        extensions.validate_portable(&definition.definition)?;
        definition.validate()?;
        let data: DataEnvelope = decode(data)?;
        let mut session = Self {
            definition,
            store: data.into_store()?,
            reducer: ActionReducer::default(),
            compiler: Compiler::with_extensions(extensions),
            inspectors: vec![],
        };
        session.prepare()?;
        Ok(session)
    }
    /// Validated captured definition, including exact operation IDs and parameters.
    pub fn definition(&self) -> &ChartDefinition {
        &self.definition.definition
    }
    /// Owned coherent immutable source handle; later commits do not mutate it.
    pub fn source(&self) -> SnapshotHandle<StoreSnapshot> {
        self.store.snapshot()
    }
    /// Current minimal state; mutations use revision-fenced actions.
    pub fn state(&self) -> &ChartState {
        self.reducer.state()
    }
    /// Immutable registrations retained for coherent publication capture.
    pub fn extensions(&self) -> &Arc<ExtensionRegistry> {
        self.compiler.extensions()
    }
    /// Serialize the authored definition with its envelope version.
    pub fn chart_json(&self) -> ChartResult<String> {
        self.compiler
            .extensions()
            .validate_portable(self.definition())?;
        encode(&self.definition)
    }
    /// Serialize exact state/revisions in their separate envelope.
    pub fn state_json(&self) -> ChartResult<String> {
        encode(&StateEnvelope::capture(
            self.definition(),
            self.reducer.state(),
        ))
    }
    /// Restore an explicit state snapshot, guarded by the current state's expected revision.
    pub fn restore_state(&mut self, input: &str, expected: Revision) -> ChartResult<()> {
        if expected != self.reducer.state().revision() {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Stale state restore",
            ));
        }
        let state: StateEnvelope = decode(input)?;
        let mut next = state.into_state(self.definition())?;
        next.retain_transient_from(self.reducer.state());
        if next.revision() < self.reducer.state().revision()
            || next.viewport_revision() < self.reducer.state().viewport_revision()
            || (next.revision() == self.reducer.state().revision() && &next != self.reducer.state())
            || (next.viewport_revision() == self.reducer.state().viewport_revision()
                && next.viewport() != self.reducer.state().viewport())
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "State restore would regress or reuse a revision for different content",
            ));
        }
        self.reducer
            .accept_controlled(&self.definition.definition, expected, next)?;
        self.prune_inspectors();
        Ok(())
    }
    /// Validate then apply the existing atomic transaction; typed outcomes preserve replay/conflicts.
    pub fn apply_transaction(&mut self, input: &str) -> ChartResult<CommitOutcome> {
        let envelope: TransactionEnvelope = decode(input)?;
        Ok(self.store.apply(envelope.into_transaction()?))
    }
    /// Validate exact definition/state fences, then use the common action reducer.
    pub fn apply_action(&mut self, input: &str) -> ChartResult<ActionOutcome> {
        let envelope: ActionEnvelope = decode(input)?;
        version(envelope.version)?;
        if envelope.definition_revision != self.definition().revision
            || envelope.expected_state != self.reducer.state().revision()
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Action definition/state revision is stale",
            ));
        }
        let request = self.reducer.request(
            &self.definition.definition,
            envelope.action,
            ActionOrigin::Programmatic,
        );
        let result = self
            .reducer
            .dispatch(&self.definition.definition, request)?
            .outcome;
        self.prune_inspectors();
        Ok(result)
    }
    /// Acknowledge the caller's actual scene before scene-dependent actions.
    pub fn present(&mut self, scene: Arc<crate::layout::LaidOutChart>) {
        self.reducer.present(scene);
        self.prune_inspectors();
    }
    /// Full shared reducer with explicit origin/state/scene fences and effective events.
    pub fn dispatch(&mut self, input: &str) -> ChartResult<DispatchOutcome> {
        let request: ActionRequest = decode(input)?;
        let result = self
            .reducer
            .dispatch(&self.definition.definition, request)?;
        self.prune_inspectors();
        Ok(result)
    }
    fn prune_inspectors(&mut self) {
        let reducer = &self.reducer;
        self.inspectors.retain(|i| {
            [
                reducer.presented(),
                reducer.gesture_basis(),
                reducer.frozen_scene(),
            ]
            .into_iter()
            .flatten()
            .any(|s| Arc::ptr_eq(s, i.presented()))
        });
    }
    /// Pure presented-scene query. Geometry indexes are shared across calls and retained only for
    /// the current, frozen and active-gesture scenes; input never compiles statistics.
    pub fn query(&mut self, input: &str) -> ChartResult<String> {
        use crate::inspection::Inspector;
        use crate::navigation::Navigator;
        let request: InputQuery = decode(input)?;
        self.prune_inspectors();
        let reducer = &self.reducer;
        let scene = if request.gesture {
            reducer.gesture_basis()
        } else {
            reducer.presented()
        }
        .ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "No requested presented/gesture scene.",
            )
        })?
        .clone();
        if request.scene != scene.scene().stamp() {
            return Err(error(
                DiagnosticCode::Superseded,
                "Query scene differs from its presented/pinned basis.",
            ));
        }
        let index = match self
            .inspectors
            .iter()
            .position(|i| Arc::ptr_eq(i.presented(), &scene))
        {
            Some(i) => i,
            None => {
                self.inspectors
                    .push(Inspector::new(scene.clone(), 10., 32)?);
                self.inspectors.len() - 1
            }
        };
        let inspector = &self.inspectors[index];
        match request.query {
            InputOperation::Describe { offset, limit } => {
                encode(&inspector.accessible_page(self.reducer.state(), offset, limit)?)
            }
            InputOperation::EditAnnotation {
                id,
                part,
                constraints,
                delta,
            } => {
                let editor = crate::editing::AnnotationEditor::new(scene, &id, part, constraints)?;
                encode(&json!({"annotation": editor.preview(request.scene, delta[0], delta[1])?}))
            }
            InputOperation::LinkCapture {
                origin,
                event,
                axes,
                panel,
                selection,
            } => encode(
                &json!({"message":crate::linking::LinkMessage::from_event(&origin,&event,inspector,self.reducer.state(),&axes,panel.as_ref(),selection)?}),
            ),
            InputOperation::LinkResolve {
                message,
                mappings,
                panel,
                missing,
            } => {
                let update = message.resolve(inspector, &mappings, panel.as_ref(), missing)?;
                encode(
                    &json!({"action":update.action,"origin":update.origin,"unmatched":update.unmatched}),
                )
            }
            InputOperation::Inspect {
                point,
                radius,
                max_grouped,
                mode,
            } => {
                let result = inspector
                    .with_options(radius, max_grouped)?
                    .query(super::input::point(point)?, mode);
                let epoch = scene.prepared().source().get()?.epoch();
                let targets: Vec<_> = result
                    .hits
                    .iter()
                    .map(|h| MarkTarget::from_inspected(h, epoch))
                    .collect();
                encode(
                    &json!({"hits":result.hits,"targets":targets,"examined":result.examined,"nodes":result.nodes}),
                )
            }
            InputOperation::Select { region, limit } => {
                encode(&json!({"targets":inspector.select(request.scene,&region.region()?,limit)?}))
            }
            InputOperation::Navigate {
                axes,
                panel,
                action,
                boundary,
            } => encode(
                &json!({"windows":Navigator::new(scene).navigate(request.scene,&axes,panel.as_ref(),action.navigation()?,boundary)?}),
            ),
            InputOperation::SetRange {
                axis,
                panel,
                window,
            } => encode(
                &json!({"windows":Navigator::new(scene).set_range(request.scene,axis,panel.as_ref(),window)?}),
            ),
        }
    }
    /// Inspect runtime ownership without exposing mutable state.
    pub fn reducer(&self) -> &ActionReducer {
        &self.reducer
    }
    /// Shared preparation; no binding-specific chart or stat algorithm.
    pub fn prepare(&mut self) -> ChartResult<Arc<PreparedChart>> {
        self.compiler
            .prepare(
                &self.definition.definition,
                &self.store.snapshot(),
                self.reducer.state(),
                CompileLimits::default(),
            )
            .map(Arc::new)
    }
    /// Semantic result DTO for runtime comparison: domains, generated rows, targets, exact sources.
    pub fn semantics_json(&mut self) -> ChartResult<String> {
        let prepared = self.prepare()?;
        let source = self.store.snapshot();
        let data = source.get()?;
        let datasets=data.datasets().map(|d| json!({
            "version":d.version(),"schema":d.schema(),
            "chunks":d.chunks().iter().map(|c|BatchWire::from_batch(c.batch())).collect::<Vec<_>>(),
        })).collect::<Vec<_>>();
        let layers = prepared
            .layers()
            .iter()
            .map(|l| {
                json!({
                    "id":l.id(),"visible":l.visible(),"domains":l.domains(),"rows":l.table().rows(),
                    "schema":l.table().schema(),"operations":l.table().operations(),
                    "color_legend":l.color_legend(),"invalid_geometry":l.invalid_geometry(),
                    "targets":l.marks().iter().flat_map(|m|m.targets.iter()).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();
        let panels = prepared.panels().iter().map(|p| json!({
            "key":p.key,"row":p.row,"column":p.column,"scale_domains":p.chart.scale_domains(),
            "layers":p.chart.layers().iter().map(|l| json!({
                "id":l.id(),"visible":l.visible(),"domains":l.domains(),"rows":l.table().rows(),
                "schema":l.table().schema(),"operations":l.table().operations(),
                "color_legend":l.color_legend(),"invalid_geometry":l.invalid_geometry(),
                "targets":l.marks().iter().flat_map(|m|m.targets.iter()).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>();
        encode(
            &json!({"version":VERSION,"definition_revision":prepared.definition_revision(),"store_revision":data.revision(),"state":StateEnvelope::capture(self.definition(),self.reducer.state()),"datasets":datasets,"layers":layers,"panels":panels}),
        )
    }
}
