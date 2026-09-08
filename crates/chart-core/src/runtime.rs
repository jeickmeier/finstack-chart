//! Typed synchronous runtime over the existing store, compiler and action reducer.
//! Hosts own execution and acknowledge real presentations explicitly.
mod presentation;
use crate::data::{SnapshotHandle, StoreSnapshot};
use crate::grammar::{ChartDefinition, CompileLimits, Compiler, ExtensionRegistry, PreparedChart};
use crate::ingestion::{EnqueueOutcome, IngestionQueue, QueueLimits, QueueStatus};
use crate::inspection::Inspector;
use crate::layout::LaidOutChart;
use crate::state::*;
use crate::transaction::{CommitOutcome, DataStore, DedupHorizon, Transaction, TransactionId};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Revision, SceneStamp};
use std::sync::Arc;

fn error(code: DiagnosticCode, message: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the current owner/revision and the typed chart operation; external sources commit through their own store.",
    )
}

enum Source {
    Owned(DataStore),
    External(SnapshotHandle<StoreSnapshot>),
}
impl Source {
    fn snapshot(&self) -> SnapshotHandle<StoreSnapshot> {
        match self {
            Self::Owned(store) => store.snapshot(),
            Self::External(source) => source.clone(),
        }
    }
    fn store(&mut self) -> ChartResult<&mut DataStore> {
        match self {
            Self::Owned(store) => Ok(store),
            Self::External(_) => Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This chart consumes an external source; commit through its owner.",
            )),
        }
    }
}

/// Retained chart with independent view state and one explicit source authority.
/// Construction validates structure only. Preparation, commit and presentation are distinct.
pub struct Chart {
    definition: ChartDefinition,
    source: Source,
    reducer: ActionReducer,
    compiler: Compiler,
    inspectors: Vec<Inspector>,
    queue: IngestionQueue,
    reconciliation: Option<Reconciliation>,
    limits: CompileLimits,
    disposed: bool,
    painted_state: Option<ChartState>,
    painted_layout: Option<crate::layout::LayoutRequest>,
    pub(crate) plot_owner: Option<u64>,
    pub(crate) data_names: std::collections::BTreeMap<String, crate::DatasetId>,
}
impl Chart {
    /// Create a retained owned chart from the primary immutable plot.
    pub fn new(plot: crate::plot::Plot) -> ChartResult<Self> {
        plot.chart()
    }
    /// Adopt an existing single-writer store without copying it or its replay/retention state.
    pub fn from_store(
        definition: ChartDefinition,
        store: DataStore,
        extensions: Arc<ExtensionRegistry>,
    ) -> ChartResult<Self> {
        Self::from_source(
            definition,
            Source::Owned(store),
            extensions,
            CompileLimits::default(),
        )
    }
    /// Adopt a single-writer store with explicit shared preparation limits.
    pub fn from_store_with_limits(
        definition: ChartDefinition,
        store: DataStore,
        extensions: Arc<ExtensionRegistry>,
        limits: CompileLimits,
    ) -> ChartResult<Self> {
        Self::from_source(definition, Source::Owned(store), extensions, limits)
    }
    /// Consume committed immutable snapshots; this chart never creates a writable store copy.
    pub fn from_external(
        definition: ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        extensions: Arc<ExtensionRegistry>,
    ) -> ChartResult<Self> {
        Self::from_source(
            definition,
            Source::External(source),
            extensions,
            CompileLimits::default(),
        )
    }
    /// Consume one external source with explicit preparation limits and no writable store copy.
    pub fn from_external_with_limits(
        definition: ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        extensions: Arc<ExtensionRegistry>,
        limits: CompileLimits,
    ) -> ChartResult<Self> {
        Self::from_source(definition, Source::External(source), extensions, limits)
    }
    fn from_source(
        definition: ChartDefinition,
        source: Source,
        extensions: Arc<ExtensionRegistry>,
        limits: CompileLimits,
    ) -> ChartResult<Self> {
        let compiler = Compiler::with_extensions(extensions);
        compiler.validate(&definition, &source.snapshot(), limits)?;
        Ok(Self {
            definition,
            source,
            compiler,
            limits,
            reducer: ActionReducer::default(),
            inspectors: vec![],
            queue: IngestionQueue::new(QueueLimits::default())?,
            reconciliation: None,
            disposed: false,
            painted_state: None,
            painted_layout: None,
            plot_owner: None,
            data_names: Default::default(),
        })
    }
    fn active(&self) -> ChartResult<()> {
        if self.disposed {
            Err(error(
                DiagnosticCode::DisposedHandle,
                "This chart is disposed.",
            ))
        } else {
            Ok(())
        }
    }
    /// Validated normalized definition, available for immutable specialist inspection.
    pub fn definition(&self) -> &ChartDefinition {
        &self.definition
    }
    /// Cheap coherent source handle; later commits do not mutate it.
    pub fn source(&self) -> SnapshotHandle<StoreSnapshot> {
        self.source.snapshot()
    }
    /// Current independent view state.
    pub fn state(&self) -> &ChartState {
        self.reducer.state()
    }
    /// Immutable registry retained for preparation and publication.
    pub fn extensions(&self) -> &Arc<ExtensionRegistry> {
        self.compiler.extensions()
    }
    /// Current reducer including acknowledged and pinned presentation ownership.
    pub fn reducer(&self) -> &ActionReducer {
        &self.reducer
    }
    /// Most recent accepted source reconciliation, separate from preparation/presentation.
    pub fn reconciliation(&self) -> Option<&Reconciliation> {
        self.reconciliation.as_ref()
    }
    /// Whether this chart owns ingestion or consumes an external committed source.
    pub fn owns_ingestion(&self) -> bool {
        matches!(self.source, Source::Owned(_))
    }
    /// Create an independent view over this coherent source without another writable store.
    /// View state starts at its defaults. Later commits are admitted explicitly with accept_from.
    pub fn external_view(&self) -> ChartResult<Self> {
        self.active()?;
        let mut view = Self::from_external_with_limits(
            self.definition.clone(),
            self.source(),
            self.extensions().clone(),
            self.limits,
        )?;
        view.plot_owner = self.plot_owner;
        view.data_names.clone_from(&self.data_names);
        Ok(view)
    }
    /// Admit another chart's latest committed source without copying its state or preparation.
    pub fn accept_from(&mut self, source: &Self) -> ChartResult<Reconciliation> {
        source.active()?;
        self.accept_source(source.source())
    }
    /// Apply an immutable edit from this chart's original Plot, preserving current ingestion.
    pub fn apply_plot(
        &mut self,
        plot: &crate::plot::Plot,
        expected: Revision,
    ) -> ChartResult<bool> {
        if self.plot_owner != Some(plot.owner) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "The edited plot belongs to another chart definition owner.",
            ));
        }
        self.set_definition(plot.definition().clone(), expected)
    }
    /// Resolve a primary dataset name to a stable handle without retaining historical rows.
    pub fn data(&self, name: &str) -> ChartResult<crate::plot::DatasetHandle> {
        self.active()?;
        self.data_names
            .get(name)
            .copied()
            .map(crate::plot::DatasetHandle)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    "This chart has no dataset with that authored name.",
                )
            })
    }
    /// Capture current transaction fences once, before collecting ordered updates.
    pub fn transaction(&self) -> ChartResult<crate::plot::TransactionBuilder> {
        self.active()?;
        if !self.owns_ingestion() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "External source owners construct and commit their own transactions.",
            ));
        }
        Ok(crate::plot::TransactionBuilder::new(
            self.source(),
            self.data_names.clone(),
        ))
    }
    /// Existing store replay horizon; external stores expose this through their owner.
    pub fn dedup_horizon(&self) -> Option<DedupHorizon> {
        match &self.source {
            Source::Owned(store) => Some(store.dedup_horizon()),
            Source::External(_) => None,
        }
    }
    /// Replace a definition against current data, retaining store/keys/replay/queued work.
    /// The expected revision is checked before validation; effective edits advance exactly once.
    pub fn set_definition(
        &mut self,
        mut definition: ChartDefinition,
        expected: Revision,
    ) -> ChartResult<bool> {
        self.active()?;
        if expected != self.definition.revision {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Definition edit is stale.",
            ));
        }
        definition.revision = expected;
        if definition == self.definition {
            return Ok(false);
        }
        definition.revision = expected.checked_next()?;
        self.compiler
            .validate(&definition, &self.source(), self.limits)?;
        let mut next = self.reducer.clone();
        if next.state().active_gesture().is_some() {
            let request = next.request(
                &self.definition,
                ChartAction::CancelGesture(CancelReason::TargetRemoved),
                ActionOrigin::Programmatic,
            );
            next.dispatch(&self.definition, request)?;
        }
        let reconciliation = next.reconcile_source(&definition, &self.source())?;
        self.definition = definition;
        self.reducer = next;
        self.reconciliation = Some(reconciliation);
        self.prune_inspectors();
        Ok(true)
    }
    /// Accept a newer coherent source from the external owner, without running statistics.
    pub fn accept_source(
        &mut self,
        source: SnapshotHandle<StoreSnapshot>,
    ) -> ChartResult<Reconciliation> {
        self.active()?;
        self.validate_external_source(&source)?;
        let mut next = self.reducer.clone();
        let reconciliation = next.reconcile_source(&self.definition, &source)?;
        self.source = Source::External(source);
        self.reducer = next;
        self.reconciliation = Some(reconciliation.clone());
        self.prune_inspectors();
        Ok(reconciliation)
    }
    fn validate_external_source(&self, source: &SnapshotHandle<StoreSnapshot>) -> ChartResult<()> {
        if self.owns_ingestion() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "An owned chart changes data through transactions.",
            ));
        }
        let old = self.source();
        let before = old.get()?;
        let after = source.get()?;
        if after.epoch() < before.epoch()
            || (after.epoch() == before.epoch()
                && (after.revision() < before.revision()
                    || (after.revision() == before.revision() && !std::ptr::eq(before, after))))
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "External source regresses or reuses a revision for different content.",
            ));
        }
        Ok(())
    }
    /// Advance the owned source epoch once, retaining rows and invalidating old retry fences.
    /// Queued transactions keep their original epochs and return explicit conflicts on drain.
    pub fn reset_epoch(&mut self) -> ChartResult<Reconciliation> {
        self.active()?;
        let value = self
            .source()
            .get()?
            .epoch()
            .get()
            .checked_add(1)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::RevisionOverflow,
                    "Source epoch is exhausted.",
                )
            })?;
        let mut next = self.reducer.clone();
        let mut reconciliation = None;
        let definition = &self.definition;
        self.source
            .store()?
            .reset_epoch_checked(crate::SourceEpoch::new(value), |source| {
                reconciliation = Some(next.reconcile_source(definition, source)?);
                Ok(())
            })?;
        self.reducer = next;
        let reconciliation = reconciliation.expect("accepted epoch has reconciliation");
        self.reconciliation = Some(reconciliation.clone());
        self.prune_inspectors();
        Ok(reconciliation)
    }
    /// Apply an atomic typed transaction; preparation failure cannot undo a committed source.
    pub fn apply_transaction(&mut self, transaction: Transaction) -> ChartResult<CommitOutcome> {
        self.active()?;
        let mut next = self.reducer.clone();
        let mut reconciliation = None;
        let definition = &self.definition;
        let outcome = self.source.store()?.apply_checked(transaction, |source| {
            reconciliation = Some(next.reconcile_source(definition, source)?);
            Ok(())
        });
        if matches!(outcome, CommitOutcome::Applied(_)) {
            self.reducer = next;
            self.reconciliation = reconciliation;
            self.prune_inspectors();
        }
        Ok(outcome)
    }
    /// Configure the owned ingestion queue through explicit typed stream options.
    pub fn stream(&mut self, options: crate::plot::StreamOptions) -> ChartResult<()> {
        self.configure_queue(options.build()?)
    }
    /// Pin the active gesture basis, or otherwise the last actually presented scene.
    pub fn navigation(&self) -> ChartResult<crate::navigation::Navigator> {
        self.active()?;
        self.reducer
            .gesture_basis()
            .or_else(|| self.reducer.presented())
            .cloned()
            .map(crate::navigation::Navigator::new)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::UnsupportedCapability,
                    "Navigation requires an acknowledged scene.",
                )
            })
    }
    /// Configure an empty queue; accepted transactions are never silently dropped.
    pub fn configure_queue(&mut self, limits: QueueLimits) -> ChartResult<()> {
        self.active()?;
        self.source.store()?;
        if self.queue.status().transactions != 0 {
            return Err(error(
                DiagnosticCode::Validation,
                "Drain accepted transactions before changing queue policy.",
            ));
        }
        self.queue = IngestionQueue::new(limits)?;
        Ok(())
    }
    /// Current queue limits.
    pub fn queue_limits(&self) -> QueueLimits {
        self.queue.limits()
    }
    /// Current queue occupancy and cumulative accounting.
    pub fn queue_status(&self) -> &QueueStatus {
        self.queue.status()
    }
    /// Accept a typed transaction into the bounded queue, separately from commit.
    pub fn enqueue(&mut self, transaction: Transaction) -> ChartResult<EnqueueOutcome> {
        self.active()?;
        self.source.store()?;
        Ok(self.queue.enqueue(transaction))
    }
    /// Commit the next accepted transaction through the same checked store/reducer boundary.
    pub fn commit_next(&mut self) -> ChartResult<Option<(TransactionId, CommitOutcome)>> {
        self.active()?;
        let mut next = self.reducer.clone();
        let mut reconciliation = None;
        let definition = &self.definition;
        let result = self
            .queue
            .commit_next_checked(self.source.store()?, |source| {
                reconciliation = Some(next.reconcile_source(definition, source)?);
                Ok(())
            });
        if matches!(&result, Some((_, CommitOutcome::Applied(_)))) {
            self.reducer = next;
            self.reconciliation = reconciliation;
            self.prune_inspectors();
        }
        Ok(result)
    }
    /// Capture an explicit state/definition/scene fence for later dispatch.
    pub fn request(&self, action: ChartAction, origin: ActionOrigin) -> ActionRequest {
        self.reducer.request(&self.definition, action, origin)
    }
    /// Dispatch a typed action with explicit fences and origin.
    pub fn dispatch(&mut self, request: ActionRequest) -> ChartResult<DispatchOutcome> {
        self.active()?;
        let result = self.reducer.dispatch(&self.definition, request)?;
        self.prune_inspectors();
        Ok(result)
    }
    /// Apply a programmatic action at the current revision, without silently retrying conflicts.
    pub fn act(&mut self, action: ChartAction) -> ChartResult<DispatchOutcome> {
        self.dispatch(self.request(action, ActionOrigin::Programmatic))
    }
    /// Restore controlled durable state through the existing reducer fences.
    pub fn restore_state(&mut self, mut next: ChartState, expected: Revision) -> ChartResult<()> {
        self.active()?;
        if expected != self.state().revision() {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Stale state restore.",
            ));
        }
        next.retain_transient_from(self.state());
        if next.revision() < self.state().revision()
            || next.viewport_revision() < self.state().viewport_revision()
            || (next.revision() == self.state().revision() && &next != self.state())
            || (next.viewport_revision() == self.state().viewport_revision()
                && next.viewport() != self.state().viewport())
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "State restore would regress or reuse a revision for different content.",
            ));
        }
        self.reducer
            .accept_controlled(&self.definition, expected, next)?;
        self.prune_inspectors();
        Ok(())
    }
    /// Acknowledge the caller's actual scene; this does not prepare or commit data.
    pub fn present(&mut self, scene: Arc<LaidOutChart>) {
        if !self.disposed {
            self.painted_layout = None;
            self.painted_state = Some(
                scene
                    .prepared()
                    .state()
                    .with_painted_inspection(self.state()),
            );
            self.reducer.present(scene);
            self.prune_inspectors();
        }
    }
    fn prune_inspectors(&mut self) {
        let reducer = &self.reducer;
        self.inspectors.retain(|i| {
            [
                reducer.presented(),
                reducer.gesture_basis(),
                reducer.frozen_scene(),
                reducer.pinned_scene(),
            ]
            .into_iter()
            .flatten()
            .any(|s| Arc::ptr_eq(s, i.presented()))
        });
    }
    /// Query handle sharing the presented scene's retained geometry index, never compiling stats.
    pub fn inspector(&mut self, stamp: SceneStamp, gesture: bool) -> ChartResult<Inspector> {
        self.active()?;
        self.prune_inspectors();
        let scene = if gesture {
            self.reducer.gesture_basis()
        } else {
            self.reducer.presented()
        }
        .ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "No requested presented/gesture scene.",
            )
        })?
        .clone();
        if scene.scene().stamp() != stamp {
            return Err(error(
                DiagnosticCode::Superseded,
                "Query scene differs from its presented/pinned basis.",
            ));
        }
        if let Some(inspector) = self
            .inspectors
            .iter()
            .find(|i| Arc::ptr_eq(i.presented(), &scene))
        {
            return Ok(inspector.clone());
        }
        let inspector = Inspector::new(scene, 10., 32)?;
        self.inspectors.push(inspector.clone());
        Ok(inspector)
    }
    /// Shared preparation with compiler caches and post-preparation reconciliation retained.
    pub fn prepare(&mut self) -> ChartResult<Arc<PreparedChart>> {
        self.active()?;
        let source = self.source();
        let mut prepared =
            self.compiler
                .prepare(&self.definition, &source, self.reducer.state(), self.limits)?;
        let result = self.reducer.reconcile_prepared(&prepared)?;
        if result.transition.outcome.changed {
            prepared = self.compiler.prepare(
                &self.definition,
                &source,
                self.reducer.state(),
                self.limits,
            )?;
            self.reconciliation = Some(result);
            self.prune_inspectors();
        }
        Ok(Arc::new(prepared))
    }
    /// Configure preparation budgets without running statistics or replacing valid state on error.
    pub fn set_compile_limits(&mut self, limits: CompileLimits) -> ChartResult<()> {
        self.active()?;
        self.compiler
            .validate(&self.definition, &self.source(), limits)?;
        self.limits = limits;
        Ok(())
    }
    /// Release retained presentation/index/compiler/queued ownership; captured inputs stay valid.
    pub fn dispose(&mut self) -> ChartResult<DispatchOutcome> {
        self.active()?;
        let outcome = self.reducer.dispose(&self.definition)?;
        self.inspectors.clear();
        self.painted_state = None;
        self.painted_layout = None;
        self.compiler.clear_cache();
        self.queue = IngestionQueue::new(self.queue.limits())?;
        let mut source = self.source();
        source.dispose();
        self.source = Source::External(source);
        self.compiler = Compiler::new();
        self.disposed = true;
        Ok(outcome)
    }
}
