//! Bounded export ownership; executors, scheduling and I/O belong to callers.
use crate::{ExportArtifact, FigureRequest, Format, error};
use chart_core::{ChartResult, DiagnosticCode, Revision};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

/// Explicit bounds on simultaneous retained export requests, independent of input ingestion.
#[derive(Clone, Copy, Debug)]
pub struct ExportLimits {
    /// Pending plus executing requests; 1..=64.
    pub max_jobs: usize,
    /// Total conservatively charged input bytes across live jobs; shared inputs count per job.
    pub max_input_bytes: usize,
    /// Total retained rows across live jobs; shared snapshots count per job.
    pub max_rows: usize,
}
impl Default for ExportLimits {
    fn default() -> Self {
        Self {
            max_jobs: 2,
            max_input_bytes: 512 * 1024 * 1024,
            max_rows: 2_000_000,
        }
    }
}
/// Snapshot/resource accounting; input bytes are a documented logical charge, not process RSS.
#[derive(serde::Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportMetrics {
    /// Requests waiting for their caller's executor.
    pub pending: usize,
    /// Requests currently preparing/encoding.
    pub running: usize,
    /// Sum of retained rows, conservatively charged per live job.
    pub rows: usize,
    /// Source columns, key/ordinal allowance, fonts and serialized capture metadata.
    pub input_bytes: usize,
    /// Highest concurrent pending plus running jobs.
    pub peak_jobs: usize,
    /// Accepted jobs.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub submitted: u64,
    /// Successfully returned artifacts.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub completed: u64,
    /// Validation/preparation/encoding/observer failures.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub failed: u64,
    /// Canceled/dropped requests, including results discarded after an active phase.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub cancelled: u64,
    /// Submissions rejected by capacity or disposal.
    #[serde(serialize_with = "crate::serialize_u64")]
    pub rejected: u64,
    /// Owner closed the queue permanently.
    pub disposed: bool,
}
/// Observable boundaries; callbacks run on the chosen executor, never inside pool locks.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportPhase {
    /// Immutable input has been transferred to this worker; numeric/layout work has not begun.
    Captured,
    /// Exact publication layout and font tree have been built; encoding has not begun.
    Prepared,
    /// Bytes have been encoded; cancellation is checked again before returning them.
    Encoded,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
struct Book {
    metrics: ExportMetrics,
    next: Revision,
    jobs: BTreeMap<Revision, Weak<JobState>>,
}
struct Pool {
    limits: ExportLimits,
    book: Mutex<Book>,
}
struct JobInner {
    request: Option<FigureRequest>,
    status: Status,
    cancel: bool,
}
struct JobState {
    id: Revision,
    pool: Arc<Pool>,
    rows: usize,
    bytes: usize,
    inner: Mutex<JobInner>,
}
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn cancelled() -> chart_core::Diagnostic {
    error(
        DiagnosticCode::Cancelled,
        "Export was cancelled; no artifact is published.",
    )
}
impl JobState {
    fn release(&self, prior: Status, next: Status) {
        let mut b = lock(&self.pool.book);
        b.jobs.remove(&self.id);
        if prior == Status::Pending {
            b.metrics.pending -= 1;
        } else {
            b.metrics.running -= 1;
        }
        b.metrics.rows -= self.rows;
        b.metrics.input_bytes -= self.bytes;
        match next {
            Status::Completed => b.metrics.completed = b.metrics.completed.saturating_add(1),
            Status::Failed => b.metrics.failed = b.metrics.failed.saturating_add(1),
            _ => b.metrics.cancelled = b.metrics.cancelled.saturating_add(1),
        }
    }
    fn cancel(&self, abandon: bool) -> bool {
        let mut inner = lock(&self.inner);
        if !matches!(inner.status, Status::Pending | Status::Running) {
            return false;
        }
        let fresh = !inner.cancel;
        inner.cancel = true;
        if inner.status == Status::Pending || abandon {
            let prior = inner.status;
            inner.status = Status::Cancelled;
            let request = inner.request.take();
            self.release(prior, Status::Cancelled);
            drop(inner);
            drop(request);
        }
        fresh
    }
    fn check(&self) -> ChartResult<()> {
        if lock(&self.inner).cancel {
            Err(cancelled())
        } else {
            Ok(())
        }
    }
    fn finish(&self, result: ChartResult<ExportArtifact>) -> ChartResult<ExportArtifact> {
        let mut inner = lock(&self.inner);
        let result = if inner.cancel {
            Err(cancelled())
        } else {
            result
        };
        let prior = inner.status;
        let next = if inner.cancel {
            Status::Cancelled
        } else if result.is_ok() {
            Status::Completed
        } else {
            Status::Failed
        };
        inner.status = next;
        self.release(prior, next);
        result
    }
}
/// Cancellation handle contains no captured source/font resources after a job finishes.
#[derive(Clone)]
pub struct ExportCancellation(Arc<JobState>);
impl ExportCancellation {
    /// Pending capture releases immediately. Executing phases finish cooperatively and their
    /// output is discarded; no unsafe thread termination or partially encoded artifact occurs.
    pub fn cancel(&self) -> bool {
        self.0.cancel(false)
    }
}
/// One uniquely executable request. Dropping it releases its reservation and cancels it.
/// Hold a cancellation handle on the UI thread and move this value to a worker.
pub struct ExportJob {
    state: Arc<JobState>,
    format: Format,
}
impl ExportJob {
    /// Monotonic queue-local identity.
    pub fn id(&self) -> Revision {
        self.state.id
    }
    /// Non-owning-of-input-after-completion cancellation control.
    pub fn cancellation(&self) -> ExportCancellation {
        ExportCancellation(self.state.clone())
    }
    /// Execute synchronously wherever the host schedules this value.
    pub fn run(self) -> ChartResult<ExportArtifact> {
        self.run_with_observer(|_| Ok(()))
    }
    /// Execute with phase observations outside locks. Observers can report host errors and
    /// coordinate deterministic acceptance tests; production code need not supply one.
    pub fn run_with_observer(
        self,
        mut observe: impl FnMut(ExportPhase) -> ChartResult<()>,
    ) -> ChartResult<ExportArtifact> {
        let request = {
            let mut i = lock(&self.state.inner);
            if i.cancel {
                return Err(cancelled());
            }
            i.status = Status::Running;
            let request = i.request.take().expect("uniquely owned pending export");
            let mut b = lock(&self.state.pool.book);
            b.metrics.pending -= 1;
            b.metrics.running += 1;
            request
        };
        let result = (|| {
            self.state.check()?;
            observe(ExportPhase::Captured)?;
            self.state.check()?;
            let figure = request.prepare()?;
            self.state.check()?;
            observe(ExportPhase::Prepared)?;
            self.state.check()?;
            let artifact = figure.export(self.format)?;
            self.state.check()?;
            observe(ExportPhase::Encoded)?;
            Ok(artifact)
        })();
        // The phase-local figure and the input capture drop before the reservation is released.
        drop(request);
        self.state.finish(result)
    }
}
impl Drop for ExportJob {
    fn drop(&mut self) {
        self.state.cancel(true);
    }
}
/// Bounded export job owner. It starts no threads and performs no I/O. Disposal requests
/// cancellation of all accepted work; active reservations remain charged until workers stop.
pub struct ExportQueue {
    pool: Arc<Pool>,
}
impl ExportQueue {
    /// Create explicit capacity; the caller owns any upstream ingestion/backpressure policy.
    pub fn new(limits: ExportLimits) -> ChartResult<Self> {
        if !(1..=64).contains(&limits.max_jobs)
            || limits.max_input_bytes == 0
            || limits.max_rows == 0
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Export limits require 1..=64 jobs and positive input/row budgets.",
            ));
        }
        Ok(Self {
            pool: Arc::new(Pool {
                limits,
                book: Mutex::new(Book {
                    metrics: ExportMetrics::default(),
                    next: Revision::INITIAL,
                    jobs: BTreeMap::new(),
                }),
            }),
        })
    }
    /// Reserve one immutable request or reject before acceptance. Export rejection does not
    /// roll back, drop or pause any already accepted source transaction.
    pub fn submit(&self, request: FigureRequest, format: Format) -> ChartResult<ExportJob> {
        let (rows, bytes) = request.charge()?;
        let mut b = lock(&self.pool.book);
        let fail = if b.metrics.disposed {
            Some((DiagnosticCode::DisposedHandle, "Export queue is disposed."))
        } else if b.metrics.pending + b.metrics.running >= self.pool.limits.max_jobs
            || b.metrics
                .rows
                .checked_add(rows)
                .is_none_or(|v| v > self.pool.limits.max_rows)
            || b.metrics
                .input_bytes
                .checked_add(bytes)
                .is_none_or(|v| v > self.pool.limits.max_input_bytes)
        {
            Some((
                DiagnosticCode::ResourceLimit,
                "Export request exceeds available job/input/row capacity.",
            ))
        } else {
            None
        };
        if let Some((code, message)) = fail {
            b.metrics.rejected = b.metrics.rejected.saturating_add(1);
            return Err(error(code, message));
        }
        let next = b.next.checked_next()?;
        let id = b.next;
        b.next = next;
        let state = Arc::new(JobState {
            id,
            pool: self.pool.clone(),
            rows,
            bytes,
            inner: Mutex::new(JobInner {
                request: Some(request),
                status: Status::Pending,
                cancel: false,
            }),
        });
        b.jobs.insert(id, Arc::downgrade(&state));
        b.metrics.pending += 1;
        b.metrics.rows += rows;
        b.metrics.input_bytes += bytes;
        b.metrics.submitted = b.metrics.submitted.saturating_add(1);
        b.metrics.peak_jobs = b
            .metrics
            .peak_jobs
            .max(b.metrics.pending + b.metrics.running);
        Ok(ExportJob { state, format })
    }
    /// Current reservations and cumulative outcomes.
    pub fn metrics(&self) -> ExportMetrics {
        lock(&self.pool.book).metrics.clone()
    }
    /// Close permanently and cancel pending/running work without waiting on a worker phase.
    pub fn dispose(&self) {
        let jobs = {
            let mut b = lock(&self.pool.book);
            b.metrics.disposed = true;
            b.jobs
                .values()
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>()
        };
        for job in jobs {
            job.cancel(false);
        }
    }
}
impl Drop for ExportQueue {
    fn drop(&mut self) {
        self.dispose();
    }
}
