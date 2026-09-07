use super::TransactionEnvelope;
use crate::ingestion::QueueLimits;
/// Versioned synchronous ingestion operations. Queue acceptance is distinct from commit.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamEnvelope {
    /// Supported envelope version.
    pub version: u32,
    /// Queue, commit or observation command.
    pub operation: StreamOperation,
}
/// All hosts execute these through the same retained core session.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StreamOperation {
    /// Set positive capacities/overload policy; only an empty queue may be reconfigured.
    ConfigureQueue(QueueLimits),
    /// Bounded acceptance only; no source revisions advance.
    Enqueue(TransactionEnvelope),
    /// Commit the oldest queued transaction, returning its identity and typed final outcome.
    CommitNext,
    /// Read current capacity/occupancy/accounting, revisions and last reconciliation.
    Status,
    /// Describe the retained pinned value, labeled historical when inputs have changed.
    Pinned,
}
