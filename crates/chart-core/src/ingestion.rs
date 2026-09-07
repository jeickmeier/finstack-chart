//! Explicit bounded acceptance, separate from synchronous committed transactions.
//! The owner chooses when to drain; no worker, I/O, implicit retries or durable log exists.
use crate::transaction::{CommitOutcome, DataStore, Mutation, Transaction, TransactionId};
use crate::{ChartResult, Diagnostic, DiagnosticCode};
use std::collections::VecDeque;

/// Behavior when the next transaction cannot fit. Accepted queued work is never discarded.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverloadPolicy {
    /// Reject acceptance so the producer can retry the unchanged transaction later.
    #[default]
    Backpressure,
    /// Explicitly drop the arriving transaction and count it; never drop a queued transaction.
    DropNewest,
}
/// Simultaneous logical limits; capacity is not an allocator/RSS measurement.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct QueueLimits {
    /// Maximum waiting transactions, including metadata-only operations.
    pub transactions: usize,
    /// Maximum incoming/replacement/removal row operations waiting for commit.
    pub rows: usize,
    /// Maximum conservatively charged transaction bytes.
    pub bytes: usize,
    /// Explicit lossless or lossy overload response.
    #[serde(default)]
    pub overload: OverloadPolicy,
}
impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            transactions: 64,
            rows: 100_000,
            bytes: 16 * 1024 * 1024,
            overload: OverloadPolicy::Backpressure,
        }
    }
}
/// Bounded occupancy and cumulative accounting. Counters saturate at u64::MAX.
#[derive(serde::Serialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct QueueStatus {
    /// Waiting transactions.
    pub transactions: usize,
    /// Waiting row-operation charge.
    pub rows: usize,
    /// Waiting byte charge.
    pub bytes: usize,
    /// Newly queued transactions (duplicate queued retries do not increment).
    #[serde(with = "crate::portable::unsigned")]
    pub accepted: u64,
    /// Drained transactions returning Applied or AlreadyApplied.
    #[serde(with = "crate::portable::unsigned")]
    pub committed: u64,
    /// Drained transactions returning Rejected or Conflict.
    #[serde(with = "crate::portable::unsigned")]
    pub failed: u64,
    /// Lossless overload responses; not accepted and safe for the producer to retry.
    #[serde(with = "crate::portable::unsigned")]
    pub backpressured: u64,
    /// Explicit DropNewest responses.
    #[serde(with = "crate::portable::unsigned")]
    pub dropped: u64,
    /// Row-operation charge discarded by DropNewest.
    #[serde(with = "crate::portable::unsigned")]
    pub dropped_rows: u64,
}
/// Acceptance acknowledgement. Queued never means validated or committed.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub enum EnqueueOutcome {
    /// Accepted for ordered synchronous processing; expected bases are checked on drain.
    Queued,
    /// Identical transaction is already waiting; no additional capacity is consumed.
    AlreadyQueued,
    /// Not accepted; no data was lost and the producer retains responsibility.
    Backpressure,
    /// Explicit lossy policy discarded the arriving transaction.
    Dropped,
    /// A queued ID was reused with different content.
    Rejected(Diagnostic),
}
struct Pending {
    transaction: Transaction,
    rows: usize,
    bytes: usize,
}
/// A FIFO accepting immutable transactions within explicit transaction/row/byte bounds.
pub struct IngestionQueue {
    pending: VecDeque<Pending>,
    limits: QueueLimits,
    status: QueueStatus,
}
impl IngestionQueue {
    /// Validate positive limits before accepting any work.
    pub fn new(limits: QueueLimits) -> ChartResult<Self> {
        if limits.transactions == 0 || limits.rows == 0 || limits.bytes == 0 {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Queue capacities must be positive.",
                "Supply explicit positive transaction, row and byte capacities.",
            ));
        }
        Ok(Self {
            pending: VecDeque::new(),
            limits,
            status: QueueStatus::default(),
        })
    }
    /// Limits selected by the owner; changing policy requires an empty new queue.
    pub fn limits(&self) -> QueueLimits {
        self.limits
    }
    /// Current occupancy and cumulative acceptance/commit/loss counters.
    pub fn status(&self) -> &QueueStatus {
        &self.status
    }
    /// Accept without committing. Invalid schema/revisions may still fail on drain.
    pub fn enqueue(&mut self, transaction: Transaction) -> EnqueueOutcome {
        if let Some(p) = self
            .pending
            .iter()
            .find(|p| p.transaction.id == transaction.id)
        {
            return if p.transaction == transaction {
                EnqueueOutcome::AlreadyQueued
            } else {
                EnqueueOutcome::Rejected(Diagnostic::error(
                    DiagnosticCode::TransactionReuse,
                    "A queued transaction ID has different content.",
                    "Retry the identical transaction or assign a new ID.",
                ))
            };
        }
        let bytes = transaction.payload_bytes();
        let rows = transaction.operations.iter().fold(0usize, |n, op| {
            n.saturating_add(match &op.mutation {
                Mutation::AppendBatch(b)
                | Mutation::UpsertByKey(b)
                | Mutation::ReplaceSnapshot(b) => b.len(),
                Mutation::RemoveKeys(keys) => keys.len(),
                _ => 0,
            })
        });
        if self.status.transactions >= self.limits.transactions
            || rows > self.limits.rows.saturating_sub(self.status.rows)
            || bytes > self.limits.bytes.saturating_sub(self.status.bytes)
        {
            return match self.limits.overload {
                OverloadPolicy::Backpressure => {
                    self.status.backpressured = self.status.backpressured.saturating_add(1);
                    EnqueueOutcome::Backpressure
                }
                OverloadPolicy::DropNewest => {
                    self.status.dropped = self.status.dropped.saturating_add(1);
                    self.status.dropped_rows = self.status.dropped_rows.saturating_add(rows as u64);
                    EnqueueOutcome::Dropped
                }
            };
        }
        self.status.transactions += 1;
        self.status.rows += rows;
        self.status.bytes += bytes;
        self.status.accepted = self.status.accepted.saturating_add(1);
        self.pending.push_back(Pending {
            transaction,
            rows,
            bytes,
        });
        EnqueueOutcome::Queued
    }
    /// Commit one queued transaction with current store fences, returning its ID and final outcome.
    /// Failures free capacity and are observable; no automatic retry or base rewriting occurs.
    pub fn commit_next(&mut self, store: &mut DataStore) -> Option<(TransactionId, CommitOutcome)> {
        self.commit_next_checked(store, |_| Ok(()))
    }
    pub(crate) fn commit_next_checked(
        &mut self,
        store: &mut DataStore,
        check: impl FnOnce(&crate::data::SnapshotHandle<crate::data::StoreSnapshot>) -> ChartResult<()>,
    ) -> Option<(TransactionId, CommitOutcome)> {
        let p = self.pending.pop_front()?;
        self.status.transactions -= 1;
        self.status.rows -= p.rows;
        self.status.bytes -= p.bytes;
        let id = p.transaction.id.clone();
        let outcome = store.apply_checked(p.transaction, check);
        match outcome {
            CommitOutcome::Applied(_) | CommitOutcome::AlreadyApplied(_) => {
                self.status.committed = self.status.committed.saturating_add(1)
            }
            _ => self.status.failed = self.status.failed.saturating_add(1),
        }
        Some((id, outcome))
    }
}
