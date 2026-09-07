//! Synchronous bounded preparation scheduling; executors and threads belong to the host.
use crate::{ChartResult, Diagnostic, DiagnosticCode, Revision, SourceEpoch};

/// Incompatible destination/semantic revisions. Data commits are deliberately separate.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompatibilityStamp {
    /// A new source epoch invalidates all old data revisions.
    pub epoch: SourceEpoch,
    /// Definition/computation generation; advance even when replacing equal external IDs.
    pub definition: Revision,
    /// Named/primary viewport generation.
    pub viewport: Revision,
    /// Destination bounds/layout generation.
    pub layout: Revision,
    /// Supplied font/renderer resources generation.
    pub resources: Revision,
    /// Other presentation changes, including palette/annotations/visibility.
    pub presentation: Revision,
}
/// Unique job identity; callers must return the exact token accompanying the work.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct JobToken {
    /// Scheduler-local strictly increasing identity.
    pub id: Revision,
    /// Captured compatibility requirements.
    pub compatibility: CompatibilityStamp,
    /// Captured coherent store commit revision.
    pub store: Revision,
}
/// One immutable request transferred to a host executor.
pub struct PreparationJob<T> {
    /// Completion fence.
    pub token: JobToken,
    /// Host-selected immutable input snapshot.
    pub input: T,
}
/// Submission accounting; coalescing never drops an accepted source transaction.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitOutcome {
    /// A request is waiting for an available executor.
    Pending,
    /// Replaced the older pending preparation; the active request continues.
    Coalesced,
}
/// Completion admission; ready is not an acknowledgement of actual presentation.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionOutcome {
    /// Compatible and no older than the last admitted result; host may render it.
    Ready,
    /// Changed specification/destination/resources, old data, unknown token or disposal.
    Stale,
    /// Host reported an execution/resource error; no result is admitted.
    Failed,
}
/// Observable work bounds and compatible presentation lag; counters saturate.
#[derive(serde::Serialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct SchedulerMetrics {
    /// Always zero or one.
    pub active: usize,
    /// Always zero or one.
    pub pending: usize,
    /// Accepted preparation requests; source transactions are counted by ingestion.
    #[serde(with = "crate::portable::unsigned")]
    pub submitted: u64,
    /// Replaced waiting preparation requests.
    #[serde(with = "crate::portable::unsigned")]
    pub coalesced: u64,
    /// Requests transferred to an executor.
    #[serde(with = "crate::portable::unsigned")]
    pub started: u64,
    /// Compatible successful completions.
    #[serde(with = "crate::portable::unsigned")]
    pub completed: u64,
    /// Incompatible/unknown/older completions.
    #[serde(with = "crate::portable::unsigned")]
    pub stale: u64,
    /// Matching jobs reported failed by their executor.
    #[serde(with = "crate::portable::unsigned")]
    pub failed: u64,
    /// Actual presentation acknowledgements.
    #[serde(with = "crate::portable::unsigned")]
    pub presentations: u64,
    /// Latest submitted coherent store revision in the current source epoch.
    pub committed: Option<Revision>,
    /// Most recently acknowledged compatible store revision.
    pub presented: Option<Revision>,
    /// Commit difference; unavailable until a compatible scene is acknowledged.
    #[serde(serialize_with = "serialize_lag")]
    pub lag: Option<u64>,
    /// Explicit disposal closes the scheduler permanently.
    pub disposed: bool,
}
/// One active plus one newest pending input. The active input is executor-owned;
/// pending ownership is released on replacement/disposal. There is no implicit worker.
pub struct PreparationScheduler<T> {
    compatibility: Option<CompatibilityStamp>,
    epoch: Option<SourceEpoch>,
    pending: Option<PreparationJob<T>>,
    active: Option<JobToken>,
    admitted: Option<JobToken>,
    next_id: Revision,
    metrics: SchedulerMetrics,
}
impl<T> Default for PreparationScheduler<T> {
    fn default() -> Self {
        Self {
            epoch: None,
            compatibility: None,
            pending: None,
            active: None,
            admitted: None,
            next_id: Revision::INITIAL,
            metrics: SchedulerMetrics::default(),
        }
    }
}
impl<T> PreparationScheduler<T> {
    /// Empty scheduler, with no resource or executor requirements.
    pub fn new() -> Self {
        Self::default()
    }
    /// Accept the latest immutable request. A context change invalidates old results,
    /// but never starts another job until the existing executor reports completion.
    pub fn submit(
        &mut self,
        compatibility: CompatibilityStamp,
        store: Revision,
        input: T,
    ) -> ChartResult<SubmitOutcome> {
        if self.metrics.disposed {
            return Err(error(
                DiagnosticCode::DisposedHandle,
                "Preparation scheduler is disposed.",
            ));
        }
        let next = self.next_id.checked_next()?;
        if self.epoch == Some(compatibility.epoch)
            && self.metrics.committed.is_some_and(|old| store < old)
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Cannot submit a regressing store revision within a source epoch.",
            ));
        }
        if self.compatibility != Some(compatibility) {
            let changed_epoch = self.epoch != Some(compatibility.epoch);
            self.admitted = None;
            self.metrics.presented = None;
            if changed_epoch {
                self.metrics.committed = None;
            }
            self.compatibility = Some(compatibility);
            self.epoch = Some(compatibility.epoch);
        }
        let outcome = if self.pending.is_some() {
            self.metrics.coalesced = self.metrics.coalesced.saturating_add(1);
            SubmitOutcome::Coalesced
        } else {
            SubmitOutcome::Pending
        };
        self.pending = Some(PreparationJob {
            token: JobToken {
                id: self.next_id,
                compatibility,
                store,
            },
            input,
        });
        self.next_id = next;
        self.metrics.submitted = self.metrics.submitted.saturating_add(1);
        self.metrics.committed = Some(store);
        self.refresh();
        Ok(outcome)
    }
    /// Transfer the one pending request only when no worker is active.
    pub fn start(&mut self) -> Option<PreparationJob<T>> {
        if self.metrics.disposed || self.active.is_some() {
            return None;
        }
        let job = self.pending.take()?;
        self.active = Some(job.token);
        self.metrics.started = self.metrics.started.saturating_add(1);
        self.refresh();
        Some(job)
    }
    /// Finish the matching worker. Data-only arrivals never make compatible progress stale.
    /// Unknown/duplicate completions cannot release another worker's active slot.
    pub fn complete(&mut self, token: JobToken, success: bool) -> CompletionOutcome {
        if self.active != Some(token) {
            self.metrics.stale = self.metrics.stale.saturating_add(1);
            return CompletionOutcome::Stale;
        }
        self.active = None;
        let outcome = if self.metrics.disposed
            || self.compatibility != Some(token.compatibility)
            || self
                .admitted
                .is_some_and(|last| token.store < last.store || token.id < last.id)
        {
            self.metrics.stale = self.metrics.stale.saturating_add(1);
            CompletionOutcome::Stale
        } else if !success {
            self.metrics.failed = self.metrics.failed.saturating_add(1);
            CompletionOutcome::Failed
        } else {
            self.admitted = Some(token);
            self.metrics.completed = self.metrics.completed.saturating_add(1);
            CompletionOutcome::Ready
        };
        self.refresh();
        outcome
    }
    /// Acknowledge actual presentation. A late paint cannot overwrite a newer admitted frame.
    pub fn present(&mut self, token: JobToken) -> bool {
        if self.metrics.disposed
            || self.admitted != Some(token)
            || self.compatibility != Some(token.compatibility)
            || self.metrics.presented.is_some_and(|p| p > token.store)
        {
            return false;
        }
        if self.metrics.presented != Some(token.store) {
            self.metrics.presentations = self.metrics.presentations.saturating_add(1);
        }
        self.metrics.presented = Some(token.store);
        self.refresh();
        true
    }
    /// Invalidate on a synchronous state/layout operation without losing the active job bound.
    /// The host must submit a compatible latest snapshot to resume preparation.
    pub fn invalidate(&mut self) {
        self.pending = None;
        self.compatibility = None;
        self.admitted = None;
        self.metrics.presented = None;
        self.refresh();
    }
    /// Release pending inputs and reject all future work/results. An already running host
    /// computation may retain its input until it cooperatively stops or finishes.
    pub fn dispose(&mut self) {
        self.pending = None;
        self.compatibility = None;
        self.admitted = None;
        self.metrics.disposed = true;
        self.metrics.presented = None;
        self.refresh();
    }
    /// Current bounded work/accounting snapshot.
    pub fn metrics(&self) -> &SchedulerMetrics {
        &self.metrics
    }
    fn refresh(&mut self) {
        self.metrics.active = usize::from(self.active.is_some());
        self.metrics.pending = usize::from(self.pending.is_some());
        self.metrics.lag = self
            .metrics
            .committed
            .zip(self.metrics.presented)
            .map(|(c, p)| c.get().saturating_sub(p.get()));
    }
}
fn error(code: DiagnosticCode, message: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use a live scheduler and a coherent non-regressing source snapshot.",
    )
}

fn serialize_lag<S: serde::Serializer>(
    value: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(&value.map(|v| v.to_string()), serializer)
}
