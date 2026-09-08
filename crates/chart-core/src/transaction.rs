//! Synchronous, ordered, atomic data commits with revision fences and bounded replay.
//! No queue or background acceptance is implied: returned Applied means committed.

use crate::data::{
    DataChunk, DataLimits, DatasetSnapshot, DatasetVersion, NormalizedBatch, RetentionPolicy,
    SnapshotHandle, StoreSnapshot,
};
use crate::provenance::SourceRef;
use crate::{
    ChartResult, DatasetId, Diagnostic, DiagnosticCode, FieldId, Revision, RowKey, SourceEpoch,
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the operation or resynchronize the source epoch, dataset and schema revisions before retrying.",
    )
}

/// Opaque, case-sensitive transaction identity with a bounded UTF-8 representation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransactionId(String);
impl TransactionId {
    /// Accept 1–128 UTF-8 bytes; IDs have no ordering/sequence semantics.
    pub fn new(value: impl Into<String>) -> ChartResult<Self> {
        let value = value.into();
        if value.is_empty() || value.len() > 128 {
            return Err(error(
                DiagnosticCode::Validation,
                "Transaction IDs must contain 1–128 UTF-8 bytes.",
            ));
        }
        Ok(Self(value))
    }
    /// Unmodified opaque identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One whole-row or metadata operation. A transaction applies these in declared order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mutation {
    /// Insert only absent keys; duplicate existing keys reject the transaction.
    AppendBatch(NormalizedBatch),
    /// Replace complete existing rows and insert absent keys; partial patches are unsupported.
    UpsertByKey(NormalizedBatch),
    /// Remove distinct keys; repeated keys in this vector collapse to one request.
    RemoveKeys(Vec<RowKey>),
    /// Replace authored row order. Reused keys keep ordinals; new keys get new ordinals.
    /// Schema changes require a strictly greater schema version.
    ReplaceSnapshot(NormalizedBatch),
    /// Update and immediately enforce retention in declared operation order.
    SetRetention(RetentionPolicy),
    /// Advance the supplied watermark of an existing event-time retention policy.
    AdvanceWatermark(i64),
    /// Rebuild one first-seen category order from currently retained authored rows.
    ResetCategoryOrder(FieldId),
}

/// Dataset-scoped operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operation {
    /// Existing dataset identity.
    pub dataset: DatasetId,
    /// Operation payload.
    pub mutation: Mutation,
}

/// Atomic source transaction. Exactly one expected base is required per touched dataset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    /// Deduplication identity.
    pub id: TransactionId,
    /// Source fence; old epochs cannot be reused after reset.
    pub epoch: SourceEpoch,
    /// Base data/schema versions, checked before any ordered operation runs.
    pub expected: Vec<DatasetVersion>,
    /// Ordered operations across one or more datasets.
    pub operations: Vec<Operation>,
}
impl Transaction {
    /// Conservative logical payload charge used by transaction and ingestion budgets.
    pub fn payload_bytes(&self) -> usize {
        self.operations.iter().fold(
            self.id
                .0
                .len()
                .saturating_add(self.expected.len().saturating_mul(32)),
            |n, op| {
                n.saturating_add(32).saturating_add(match &op.mutation {
                    Mutation::AppendBatch(b)
                    | Mutation::UpsertByKey(b)
                    | Mutation::ReplaceSnapshot(b) => b.payload_bytes(),
                    Mutation::RemoveKeys(keys) => keys.len().saturating_mul(8),
                    Mutation::SetRetention(_) => 64,
                    _ => 16,
                })
            },
        )
    }
}

/// Counts for one operation, before later operations may undo its row changes.
#[derive(serde::Serialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct OperationCounts {
    /// New keys inserted, including keys subsequently evicted by this operation.
    pub inserted: usize,
    /// Existing whole rows whose logical values/display strings changed.
    pub updated: usize,
    /// Present keys explicitly removed (including replacement omissions).
    pub removed: usize,
    /// Distinct requested removal keys that were already absent.
    pub absent: usize,
    /// Keys removed by automatic count retention.
    pub evicted: usize,
    /// Incoming rows explicitly discarded by the event-time late-drop policy.
    pub late_dropped: usize,
}

/// Completed transaction acknowledgement, including net revision and removal effects.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub struct CommitReceipt {
    /// Source epoch at commit.
    pub epoch: SourceEpoch,
    /// Resulting coherent store revision.
    pub store_revision: Revision,
    /// False for an empty or net-no-op transaction.
    pub changed: bool,
    /// Resulting versions of all touched datasets, in dataset-ID order.
    pub datasets: Vec<DatasetVersion>,
    /// One count record per operation, in the original order.
    pub operations: Vec<OperationCounts>,
    /// Previously retained keys absent in the final snapshot. Hosts use these to prune
    /// active source selections; explicit pinned historical snapshots remain resolvable.
    pub removed_sources: Vec<SourceRef>,
}
impl CommitReceipt {
    fn bytes(&self) -> usize {
        64usize
            .saturating_add(self.datasets.len().saturating_mul(32))
            .saturating_add(self.operations.len().saturating_mul(48))
            .saturating_add(self.removed_sources.len().saturating_mul(16))
    }
}

/// Observed fences accompanying a conflict; sources must resynchronize.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    /// Recoverable conflict explanation and available revision context.
    pub diagnostic: Diagnostic,
    /// Store's observed source epoch.
    pub epoch: SourceEpoch,
    /// Observed versions of referenced datasets that still exist.
    pub observed: Vec<DatasetVersion>,
}

/// Synchronous acknowledgement. Rejections/conflicts leave data and replay history intact.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub enum CommitOutcome {
    /// Committed, including an explicitly unchanged result for a no-op.
    Applied(CommitReceipt),
    /// Identical remembered transaction; original receipt returned without reapplication.
    AlreadyApplied(CommitReceipt),
    /// Validation/resource/payload error.
    Rejected(Diagnostic),
    /// Source/data/schema fence mismatch.
    Conflict(Conflict),
}

/// Observable FIFO replay horizon. Reads/retries do not extend it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DedupHorizon {
    /// Number of remembered committed transactions, including no-ops.
    pub entries: usize,
    /// Configured entry cap.
    pub capacity: usize,
    /// Conservatively charged payload and receipt bytes.
    pub bytes: usize,
    /// Configured byte cap.
    pub byte_capacity: usize,
    /// Oldest retained ID, if any.
    pub oldest: Option<TransactionId>,
    /// Newest retained ID, if any.
    pub newest: Option<TransactionId>,
}
struct Remembered {
    transaction: Transaction,
    receipt: CommitReceipt,
    bytes: usize,
}

/// Single-writer synchronous store. Snapshots retain immutable state across commits/disposal.
pub struct DataStore {
    state: Arc<StoreSnapshot>,
    limits: DataLimits,
    replay: VecDeque<Remembered>,
    replay_bytes: usize,
}

impl DataStore {
    /// Register a fixed set of dataset identities at revision zero, preserving registration
    /// and authored row order. Creation/destruction during transactions is not yet exposed.
    pub fn new(
        epoch: SourceEpoch,
        datasets: Vec<(DatasetId, NormalizedBatch)>,
        limits: DataLimits,
    ) -> ChartResult<Self> {
        limits.validate()?;
        if datasets.len() > limits.max_datasets {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Dataset count budget exceeded.",
            ));
        }
        let mut state = StoreSnapshot {
            epoch,
            revision: Revision::INITIAL,
            datasets: BTreeMap::new(),
            order: Vec::new(),
        };
        for (id, batch) in datasets {
            if state.datasets.contains_key(&id) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Dataset IDs must be unique.",
                ));
            }
            batch.validate(limits)?;
            let mut data = DatasetSnapshot::empty(id, batch.schema().clone());
            append(
                &mut data,
                &batch,
                &(0..batch.len()).collect::<Vec<_>>(),
                limits,
            )?;
            data.add_categories(&batch, limits)?;
            check_size(&data, limits)?;
            state.order.push(id);
            state.datasets.insert(id, Arc::new(data));
        }
        Ok(Self {
            state: Arc::new(state),
            limits,
            replay: VecDeque::new(),
            replay_bytes: 0,
        })
    }
    /// Capture one coherent revision cheaply; cloning the handle does not clone source data.
    pub fn snapshot(&self) -> SnapshotHandle<StoreSnapshot> {
        SnapshotHandle::from_arc(self.state.clone())
    }
    /// Current bounded replay horizon.
    pub fn dedup_horizon(&self) -> DedupHorizon {
        DedupHorizon {
            entries: self.replay.len(),
            capacity: self.limits.dedup_entries,
            bytes: self.replay_bytes,
            byte_capacity: self.limits.dedup_bytes,
            oldest: self.replay.front().map(|v| v.transaction.id.clone()),
            newest: self.replay.back().map(|v| v.transaction.id.clone()),
        }
    }
    /// Explicitly advance the source fence and clear replay history, preserving data/revisions.
    /// Epochs must strictly increase, so an old epoch cannot become valid again.
    pub fn reset_epoch(&mut self, epoch: SourceEpoch) -> ChartResult<()> {
        self.reset_epoch_checked(epoch, |_| Ok(()))
    }
    pub(crate) fn reset_epoch_checked(
        &mut self,
        epoch: SourceEpoch,
        check: impl FnOnce(&SnapshotHandle<StoreSnapshot>) -> ChartResult<()>,
    ) -> ChartResult<()> {
        if epoch.get() <= self.state.epoch.get() {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "A source epoch reset must strictly increase the epoch.",
            ));
        }
        let mut candidate = self.state.as_ref().clone();
        candidate.epoch = epoch;
        let state = Arc::new(candidate);
        check(&SnapshotHandle::from_arc(state.clone()))?;
        self.state = state;
        self.replay.clear();
        self.replay_bytes = 0;
        Ok(())
    }
    /// Validate and stage everything, then publish one atomic commit or return unchanged state.
    pub fn apply(&mut self, transaction: Transaction) -> CommitOutcome {
        self.apply_checked(transaction, |_| Ok(()))
    }
    pub(crate) fn apply_checked(
        &mut self,
        transaction: Transaction,
        check: impl FnOnce(&SnapshotHandle<StoreSnapshot>) -> ChartResult<()>,
    ) -> CommitOutcome {
        if transaction.operations.len() > self.limits.max_operations
            || transaction.expected.len() > self.limits.max_datasets
            || transaction.payload_bytes() > self.limits.max_transaction_bytes
        {
            return CommitOutcome::Rejected(error(
                DiagnosticCode::ResourceLimit,
                "Transaction operation/base/payload budget exceeded.",
            ));
        }
        let touched: BTreeSet<_> = transaction.operations.iter().map(|o| o.dataset).collect();
        if transaction.epoch != self.state.epoch {
            return self.conflict(
                error(DiagnosticCode::RevisionConflict, "Source epoch is stale."),
                &touched,
            );
        }
        if let Some(entry) = self
            .replay
            .iter()
            .find(|entry| entry.transaction.id == transaction.id)
        {
            return if entry.transaction == transaction {
                CommitOutcome::AlreadyApplied(entry.receipt.clone())
            } else {
                CommitOutcome::Rejected(error(
                    DiagnosticCode::TransactionReuse,
                    "A remembered transaction ID was reused with different payload or expected bases.",
                ))
            };
        }
        let expected_ids: BTreeSet<_> = transaction.expected.iter().map(|e| e.dataset).collect();
        if expected_ids.len() != transaction.expected.len() || expected_ids != touched {
            return CommitOutcome::Rejected(error(
                DiagnosticCode::Validation,
                "Supply exactly one expected base for every touched dataset, with no extras.",
            ));
        }
        for expected in &transaction.expected {
            let observed = match self.state.dataset(expected.dataset) {
                Ok(data) => data.version(),
                Err(e) => return CommitOutcome::Rejected(e),
            };
            if observed != *expected {
                let mut e = error(
                    DiagnosticCode::RevisionConflict,
                    "Dataset or schema base is stale.",
                );
                e.context.dataset = Some(observed.dataset);
                e.context.dataset_revision = Some(observed.revision);
                e.context.schema_version = Some(observed.schema_version);
                return self.conflict(e, &touched);
            }
        }
        let staged = self.stage(&transaction, &touched);
        let (next, receipt) = match staged {
            Ok(value) => value,
            Err(e) => return CommitOutcome::Rejected(e),
        };
        let bytes = transaction.payload_bytes().saturating_add(receipt.bytes());
        if bytes > self.limits.dedup_bytes {
            return CommitOutcome::Rejected(error(
                DiagnosticCode::ResourceLimit,
                "Transaction plus receipt cannot fit the replay horizon; raise the budget or reduce the payload.",
            ));
        }
        let next = if receipt.changed {
            Arc::new(next)
        } else {
            self.state.clone()
        };
        if let Err(e) = check(&SnapshotHandle::from_arc(next.clone())) {
            return CommitOutcome::Rejected(e);
        }
        while self.replay.len() >= self.limits.dedup_entries
            || self.replay_bytes.saturating_add(bytes) > self.limits.dedup_bytes
        {
            if let Some(old) = self.replay.pop_front() {
                self.replay_bytes -= old.bytes;
            }
        }
        if receipt.changed {
            self.state = next;
        }
        self.replay_bytes += bytes;
        self.replay.push_back(Remembered {
            transaction,
            receipt: receipt.clone(),
            bytes,
        });
        CommitOutcome::Applied(receipt)
    }
    fn conflict(&self, diagnostic: Diagnostic, touched: &BTreeSet<DatasetId>) -> CommitOutcome {
        CommitOutcome::Conflict(Conflict {
            diagnostic,
            epoch: self.state.epoch,
            observed: touched
                .iter()
                .filter_map(|id| self.state.datasets.get(id).map(|d| d.version()))
                .collect(),
        })
    }
    fn stage(
        &self,
        transaction: &Transaction,
        touched: &BTreeSet<DatasetId>,
    ) -> ChartResult<(StoreSnapshot, CommitReceipt)> {
        let mut next = (*self.state).clone();
        let mut receipt = CommitReceipt {
            epoch: next.epoch,
            store_revision: next.revision,
            changed: false,
            datasets: Vec::new(),
            operations: Vec::new(),
            removed_sources: Vec::new(),
        };
        for op in &transaction.operations {
            let data = next
                .datasets
                .get_mut(&op.dataset)
                .ok_or_else(|| error(DiagnosticCode::MissingResource, "Dataset is absent."))?;
            let data = Arc::make_mut(data);
            let counts = mutate(data, &op.mutation, self.limits).map_err(|mut e| {
                e.context.dataset = Some(op.dataset);
                e.context.dataset_revision = Some(data.revision);
                e.context.schema_version = Some(data.schema.version());
                e
            })?;
            receipt.operations.push(counts);
        }
        for id in touched {
            let before = self.state.dataset(*id)?;
            let after = next
                .datasets
                .get_mut(id)
                .ok_or_else(|| error(DiagnosticCode::MissingResource, "Dataset is absent."))?;
            if before.equivalent(after) {
                *after = self.state.datasets[id].clone();
            } else {
                Arc::make_mut(after).revision =
                    before.revision.checked_next().map_err(|mut e| {
                        e.context.dataset = Some(*id);
                        e.context.dataset_revision = Some(before.revision);
                        e
                    })?;
                receipt.changed = true;
                let retained: BTreeSet<_> = after.rows().map(|r| r.key()).collect();
                receipt.removed_sources.extend(
                    before
                        .rows()
                        .filter(|r| !retained.contains(&r.key()))
                        .map(|r| SourceRef {
                            dataset: *id,
                            key: r.key(),
                        }),
                );
            }
            receipt.datasets.push(after.version());
        }
        if receipt.changed {
            next.revision = next.revision.checked_next()?;
            receipt.store_revision = next.revision;
        }
        Ok((next, receipt))
    }
}

fn check_size(data: &DatasetSnapshot, limits: DataLimits) -> ChartResult<()> {
    let bytes = data
        .categories
        .values()
        .flat_map(|v| v.iter())
        .fold(data.schema.payload_bytes(), |n, s| {
            n.saturating_add(24).saturating_add(s.len())
        });
    let bytes = data.chunks.iter().fold(bytes, |n, c| {
        n.saturating_add(c.batch().payload_bytes())
            .saturating_add(c.ordinals().len().saturating_mul(8))
    });
    if data.len() > limits.max_dataset_rows || bytes > limits.max_dataset_bytes {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Retained dataset row/payload budget exceeded.",
        ));
    }
    Ok(())
}

fn append(
    data: &mut DatasetSnapshot,
    batch: &NormalizedBatch,
    indexes: &[usize],
    limits: DataLimits,
) -> ChartResult<()> {
    let end = data
        .next_ordinal
        .checked_add(u64::try_from(indexes.len()).map_err(|_| {
            error(
                DiagnosticCode::ResourceLimit,
                "Row count cannot fit insertion ordinals.",
            )
        })?)
        .ok_or_else(|| {
            error(
                DiagnosticCode::RevisionOverflow,
                "Insertion ordinal counter exhausted.",
            )
        })?;
    for slice in indexes.chunks(limits.chunk_rows) {
        let owned = if slice.len() == batch.len() && slice.iter().enumerate().all(|(i, &v)| i == v)
        {
            batch.clone()
        } else {
            batch.selected(slice)
        };
        let start = data.next_ordinal;
        data.next_ordinal += slice.len() as u64;
        data.chunks
            .push(DataChunk::new(owned, (start..data.next_ordinal).collect()));
    }
    debug_assert_eq!(end, data.next_ordinal);
    Ok(())
}

fn remove(data: &mut DatasetSnapshot, keys: &BTreeSet<RowKey>) -> usize {
    let before = data.len();
    data.chunks = data
        .chunks
        .iter()
        .filter_map(|chunk| {
            let keep: Vec<_> = chunk
                .batch()
                .keys()
                .iter()
                .enumerate()
                .filter(|(_, key)| !keys.contains(key))
                .map(|(i, _)| i)
                .collect();
            if keep.is_empty() {
                None
            } else if keep.len() == chunk.batch().len() {
                Some(chunk.clone())
            } else {
                Some(chunk.selected(&keep))
            }
        })
        .collect();
    before - data.len()
}

fn mutate(
    data: &mut DatasetSnapshot,
    mutation: &Mutation,
    limits: DataLimits,
) -> ChartResult<OperationCounts> {
    data.lookup.take();
    let mut counts = OperationCounts::default();
    // Validate the whole incoming representation before any explicitly lossy filtering.
    let filtered;
    let mutation = if let RetentionPolicy::EventTime(window) = data.retention {
        match mutation {
            Mutation::AppendBatch(batch)
            | Mutation::UpsertByKey(batch)
            | Mutation::ReplaceSnapshot(batch) => {
                batch.validate(limits)?;
                if data.schema().field(window.field).map(|(_, f)| &f.kind)
                    != batch.schema().field(window.field).map(|(_, f)| &f.kind)
                {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Disable event-time retention explicitly before changing its timestamp representation.",
                    ));
                }
                if matches!(mutation, Mutation::AppendBatch(_)) {
                    let existing: BTreeSet<_> = data.rows().map(|r| r.key()).collect();
                    if batch.keys().iter().any(|k| existing.contains(k)) {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Append key already exists, including a row that would be dropped as late.",
                        ));
                    }
                }
                let (batch, dropped) = window.filter(batch)?;
                counts.late_dropped = dropped;
                filtered = match mutation {
                    Mutation::AppendBatch(_) => Mutation::AppendBatch(batch),
                    Mutation::UpsertByKey(_) => Mutation::UpsertByKey(batch),
                    _ => Mutation::ReplaceSnapshot(batch),
                };
                &filtered
            }
            _ => mutation,
        }
    } else {
        mutation
    };
    match mutation {
        Mutation::AppendBatch(batch) | Mutation::UpsertByKey(batch) => {
            batch.validate(limits)?;
            if batch.schema() != &data.schema {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Append/upsert requires the exact current schema, including metadata.",
                ));
            }
            let existing: BTreeMap<_, _> = data.rows().map(|row| (row.key(), row)).collect();
            let mut inserted = Vec::new();
            let mut replacements = BTreeMap::new();
            for (i, key) in batch.keys().iter().enumerate() {
                if let Some(row) = existing.get(key) {
                    if matches!(mutation, Mutation::AppendBatch(_)) {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Append key already exists.",
                        ));
                    }
                    if !row.chunk.batch().row_eq(row.index, batch, i) {
                        replacements.insert(*key, i);
                    }
                } else {
                    inserted.push(i);
                }
            }
            counts.inserted = inserted.len();
            counts.updated = replacements.len();
            data.chunks = data
                .chunks
                .iter()
                .map(|chunk| {
                    if chunk
                        .batch()
                        .keys()
                        .iter()
                        .any(|k| replacements.contains_key(k))
                    {
                        chunk.replaced(batch, &replacements)
                    } else {
                        Ok(chunk.clone())
                    }
                })
                .collect::<ChartResult<_>>()?;
            append(data, batch, &inserted, limits)?;
            data.add_categories(batch, limits)?;
        }
        Mutation::RemoveKeys(keys) => {
            let keys = keys.iter().copied().collect::<BTreeSet<_>>();
            counts.removed = remove(data, &keys);
            counts.absent = keys.len() - counts.removed;
        }
        Mutation::ReplaceSnapshot(batch) => {
            batch.validate(limits)?;
            if batch.schema() != &data.schema
                && batch.schema().version().get() <= data.schema.version().get()
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "A changed replacement schema needs a strictly newer version.",
                ));
            }
            let mut next_ordinal = data.next_ordinal;
            let old: BTreeMap<_, _> = data.rows().map(|r| (r.key(), (r.ordinal(), r))).collect();
            let mut ordinals = Vec::with_capacity(batch.len());
            for (index, key) in batch.keys().iter().enumerate() {
                if let Some((ordinal, row)) = old.get(key) {
                    ordinals.push(*ordinal);
                    if data.schema != *batch.schema()
                        || !row.chunk.batch().row_eq(row.index, batch, index)
                    {
                        counts.updated += 1;
                    }
                } else {
                    ordinals.push(next_ordinal);
                    next_ordinal = next_ordinal.checked_add(1).ok_or_else(|| {
                        error(
                            DiagnosticCode::RevisionOverflow,
                            "Insertion ordinal counter exhausted.",
                        )
                    })?;
                    counts.inserted += 1;
                }
            }
            counts.removed = old.len() - (batch.len() - counts.inserted);
            data.next_ordinal = next_ordinal;
            if data.schema != *batch.schema() {
                data.categories = batch
                    .schema()
                    .fields()
                    .iter()
                    .filter(|f| f.kind == crate::data::FieldKind::Categorical)
                    .map(|f| {
                        (
                            f.id,
                            data.categories
                                .get(&f.id)
                                .cloned()
                                .unwrap_or_else(|| Arc::from([])),
                        )
                    })
                    .collect();
            }
            data.schema = batch.schema().clone();
            data.chunks.clear();
            for start in (0..batch.len()).step_by(limits.chunk_rows) {
                let end = (start + limits.chunk_rows).min(batch.len());
                let indexes: Vec<_> = (start..end).collect();
                let part = if start == 0 && end == batch.len() {
                    batch.clone()
                } else {
                    batch.selected(&indexes)
                };
                data.chunks
                    .push(DataChunk::new(part, ordinals[start..end].to_vec()));
            }
            data.add_categories(batch, limits)?;
        }
        Mutation::SetRetention(policy) => {
            if let RetentionPolicy::EventTime(next) = policy {
                next.validate(data)?;
                if let RetentionPolicy::EventTime(old) = data.retention
                    && old.field == next.field
                    && next.watermark < old.watermark
                {
                    return Err(error(
                        DiagnosticCode::RevisionConflict,
                        "Event-time watermark cannot regress; changing width/lateness does not reset it.",
                    ));
                }
            }
            data.retention = *policy;
        }
        Mutation::AdvanceWatermark(watermark) => {
            let RetentionPolicy::EventTime(mut window) = data.retention else {
                return Err(error(
                    DiagnosticCode::Validation,
                    "AdvanceWatermark requires an event-time retention policy.",
                ));
            };
            if *watermark < window.watermark {
                return Err(error(
                    DiagnosticCode::RevisionConflict,
                    "Event-time watermark cannot regress.",
                ));
            }
            window.watermark = *watermark;
            data.retention = RetentionPolicy::EventTime(window);
        }
        Mutation::ResetCategoryOrder(field) => {
            if !data.categories.contains_key(field) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Category reset requires a categorical field.",
                ));
            }
            let mut seen = BTreeSet::new();
            let mut labels = Vec::new();
            for row in data.rows() {
                if let Some(crate::data::ValueRef::Category(label)) = row.value(*field)
                    && seen.insert(label)
                {
                    labels.push(label.to_owned());
                }
            }
            data.categories.insert(*field, labels.into());
        }
    }
    if let RetentionPolicy::Count(max) = data.retention
        && data.len() > max
    {
        let mut oldest: Vec<_> = data.rows().map(|r| (r.ordinal(), r.key())).collect();
        oldest.sort_unstable();
        let keys = oldest
            .iter()
            .take(data.len() - max)
            .map(|(_, key)| *key)
            .collect();
        counts.evicted = remove(data, &keys);
    }
    if let RetentionPolicy::EventTime(window) = data.retention {
        window.validate(data)?;
        let mut keys = BTreeSet::new();
        for row in data.rows() {
            let Some(crate::data::ValueRef::Timestamp(t)) = row.value(window.field) else {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Retained observations must have valid event timestamps.",
                ));
            };
            if i128::from(t) < window.cutoff() {
                keys.insert(row.key());
            }
        }
        counts.evicted = remove(data, &keys);
    }
    check_size(data, limits)?;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, ColumnValues, Field, FieldKind, Schema};
    use crate::{FieldId, SchemaVersion};

    #[test]
    fn exhausted_store_dataset_and_ordinal_counters_roll_back() {
        let id = DatasetId::new(1);
        let schema = Arc::new(
            Schema::new(
                SchemaVersion::new(0),
                vec![Field {
                    id: FieldId::new(1),
                    name: "x".into(),
                    kind: FieldKind::Float64,
                    nullable: false,
                    unit: None,
                    label: None,
                }],
            )
            .unwrap(),
        );
        let batch = |keys: Vec<RowKey>| {
            let len = keys.len();
            NormalizedBatch::new(
                schema.clone(),
                keys,
                vec![Column::new(
                    ColumnValues::Float64(vec![1.; len]),
                    vec![true; len],
                    None,
                )],
                DataLimits::default(),
            )
            .unwrap()
        };
        for counter in 0..3 {
            let mut store = DataStore::new(
                SourceEpoch::new(0),
                vec![(id, batch(vec![]))],
                DataLimits::default(),
            )
            .unwrap();
            let state = Arc::make_mut(&mut store.state);
            let data = Arc::make_mut(state.datasets.get_mut(&id).unwrap());
            match counter {
                0 => state.revision = Revision::new(u64::MAX),
                1 => data.revision = Revision::new(u64::MAX),
                _ => data.next_ordinal = u64::MAX,
            }
            let old = store.snapshot();
            let transaction = Transaction {
                id: TransactionId::new("overflow").unwrap(),
                epoch: SourceEpoch::new(0),
                expected: vec![store.state.dataset(id).unwrap().version()],
                operations: vec![Operation {
                    dataset: id,
                    mutation: Mutation::AppendBatch(batch(vec![RowKey::new(1)])),
                }],
            };
            assert!(
                matches!(store.apply(transaction),CommitOutcome::Rejected(e) if e.code==DiagnosticCode::RevisionOverflow)
            );
            assert!(std::ptr::eq(
                old.get().unwrap(),
                store.snapshot().get().unwrap()
            ));
            assert_eq!(store.dedup_horizon().entries, 0);
        }
    }
}
