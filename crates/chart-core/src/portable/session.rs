use super::error;
use super::*;
use crate::{data::*, grammar::*, state::*, transaction::*, *};
use serde_json::json;
use std::sync::Arc;

/// Owned synchronous portable core session. Hosts own its lifetime and interpreter detachment.
pub struct Session {
    definition: ChartEnvelope,
    store: DataStore,
    state: ChartState,
    compiler: Compiler,
}
impl Session {
    /// Decode and validate versioned chart/data inputs, including builtin operations and schemas.
    pub fn new(chart: &str, data: &str) -> ChartResult<Self> {
        let definition: ChartEnvelope = decode(chart)?;
        definition.validate()?;
        let data: DataEnvelope = decode(data)?;
        let mut session = Self {
            definition,
            store: data.into_store()?,
            state: ChartState::default(),
            compiler: Compiler::default(),
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
        &self.state
    }
    /// Serialize the authored definition with its envelope version.
    pub fn chart_json(&self) -> ChartResult<String> {
        encode(&self.definition)
    }
    /// Serialize exact state/revisions in their separate envelope.
    pub fn state_json(&self) -> ChartResult<String> {
        encode(&StateEnvelope::capture(self.definition(), &self.state))
    }
    /// Restore an explicit state snapshot, guarded by the current state's expected revision.
    pub fn restore_state(&mut self, input: &str, expected: Revision) -> ChartResult<()> {
        if expected != self.state.revision() {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Stale state restore",
            ));
        }
        let state: StateEnvelope = decode(input)?;
        let next = state.into_state(self.definition())?;
        if next.revision() < self.state.revision()
            || next.viewport_revision() < self.state.viewport_revision()
            || (next.revision() == self.state.revision() && next != self.state)
            || (next.viewport_revision() == self.state.viewport_revision()
                && next.viewport() != self.state.viewport())
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "State restore would regress or reuse a revision for different content",
            ));
        }
        self.state = next;
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
            || envelope.expected_state != self.state.revision()
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Action definition/state revision is stale",
            ));
        }
        self.state
            .apply(&self.definition.definition, envelope.action)
    }
    /// Shared preparation; no binding-specific chart or stat algorithm.
    pub fn prepare(&mut self) -> ChartResult<Arc<PreparedChart>> {
        self.compiler
            .prepare(
                &self.definition.definition,
                &self.store.snapshot(),
                &self.state,
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
                    "targets":l.marks().iter().flat_map(|m|m.targets.iter()).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();
        encode(
            &json!({"version":VERSION,"definition_revision":prepared.definition_revision(),"store_revision":data.revision(),"state":StateEnvelope::capture(self.definition(),&self.state),"datasets":datasets,"layers":layers}),
        )
    }
}
