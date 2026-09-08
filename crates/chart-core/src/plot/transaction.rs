use super::{Data, error, fresh_id};
use crate::{ChartResult, DatasetId, DiagnosticCode, RowKey, data::*, transaction::*};
use std::collections::{BTreeMap, BTreeSet};

/// Stable dataset identity for live operations without retaining old rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatasetHandle(pub(crate) DatasetId);
impl DatasetHandle {
    /// Exact dataset identity for immutable specialist inspection.
    pub fn id(self) -> DatasetId {
        self.0
    }
}
/// Dataset selection resolved once against the transaction's captured source.
#[derive(Clone, Debug)]
pub enum DatasetRef {
    /// Name from primary Plot authoring.
    Name(String),
    /// Exact handle from owned Data or the current Chart.
    Handle(DatasetHandle),
}
impl From<&str> for DatasetRef {
    fn from(value: &str) -> Self {
        Self::Name(value.into())
    }
}
impl From<String> for DatasetRef {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}
impl From<DatasetHandle> for DatasetRef {
    fn from(value: DatasetHandle) -> Self {
        Self::Handle(value)
    }
}
impl From<&Data> for DatasetRef {
    fn from(value: &Data) -> Self {
        Self::Handle(value.handle())
    }
}
impl Data {
    /// Stable live-operation handle that does not retain this batch.
    pub fn handle(&self) -> DatasetHandle {
        DatasetHandle(self.id)
    }
}
impl From<Data> for NormalizedBatch {
    fn from(value: Data) -> Self {
        value.batch.as_ref().clone()
    }
}
impl From<&Data> for NormalizedBatch {
    fn from(value: &Data) -> Self {
        value.batch.as_ref().clone()
    }
}

/// Atomic ordered updates with expected dataset/schema bases captured at construction.
/// Build never commits or retries; pass the immutable result to Chart::apply_transaction/enqueue.
#[derive(Clone)]
pub struct TransactionBuilder {
    source: SnapshotHandle<StoreSnapshot>,
    names: BTreeMap<String, DatasetId>,
    id: ChartResult<TransactionId>,
    operations: Vec<Operation>,
    failure: Option<crate::Diagnostic>,
}
impl TransactionBuilder {
    pub(crate) fn new(
        source: SnapshotHandle<StoreSnapshot>,
        names: BTreeMap<String, DatasetId>,
    ) -> Self {
        Self {
            source,
            names,
            id: fresh_id().and_then(|id| TransactionId::new(format!("transaction_{id}"))),
            operations: vec![],
            failure: None,
        }
    }
    /// Supply a durable retry identity; retry the same built transaction unchanged.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = TransactionId::new(id);
        self
    }
    fn resolve(&self, target: DatasetRef) -> ChartResult<DatasetId> {
        let id = match target {
            DatasetRef::Handle(handle) => handle.id(),
            DatasetRef::Name(name) => *self.names.get(&name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("Unknown dataset '{name}'."),
                )
            })?,
        };
        self.source.get()?.dataset(id)?;
        Ok(id)
    }
    fn push(mut self, target: impl Into<DatasetRef>, mutation: Mutation) -> Self {
        match self.resolve(target.into()) {
            Ok(dataset) => self.operations.push(Operation { dataset, mutation }),
            Err(e) => self.failure = Some(e),
        }
        self
    }
    /// Append absent exact keys; automatically keyed Data contributes fresh identities.
    pub fn append(self, target: impl Into<DatasetRef>, batch: impl Into<NormalizedBatch>) -> Self {
        self.push(target, Mutation::AppendBatch(batch.into()))
    }
    /// Replace complete keyed rows or insert absent keys in one ordered operation.
    pub fn upsert(self, target: impl Into<DatasetRef>, batch: impl Into<NormalizedBatch>) -> Self {
        self.push(target, Mutation::UpsertByKey(batch.into()))
    }
    /// Replace authored rows; only explicitly reused keys retain logical observation identity.
    pub fn replace(self, target: impl Into<DatasetRef>, batch: impl Into<NormalizedBatch>) -> Self {
        self.push(target, Mutation::ReplaceSnapshot(batch.into()))
    }
    /// Remove supplied exact observation keys without fabricating positional identity.
    pub fn remove(
        self,
        target: impl Into<DatasetRef>,
        keys: impl IntoIterator<Item = u64>,
    ) -> Self {
        self.push(
            target,
            Mutation::RemoveKeys(keys.into_iter().map(RowKey::new).collect()),
        )
    }
    /// Configure and immediately enforce retained authored count; None disables count retention.
    pub fn retain_count(self, target: impl Into<DatasetRef>, count: Option<usize>) -> Self {
        self.push(
            target,
            Mutation::SetRetention(
                count.map_or(RetentionPolicy::Unbounded, RetentionPolicy::Count),
            ),
        )
    }
    /// Configure any existing typed retention policy, including supplied event-time windows.
    pub fn retention(self, target: impl Into<DatasetRef>, policy: RetentionPolicy) -> Self {
        self.push(target, Mutation::SetRetention(policy))
    }
    /// Configure supplied event time by its authored field name and exact source ticks.
    pub fn retain_event_time(
        mut self,
        target: impl Into<DatasetRef>,
        field: &str,
        width: i64,
        watermark: i64,
        allowed_lateness: i64,
        late: LateDataPolicy,
    ) -> Self {
        let result = (|| {
            let dataset = self.resolve(target.into())?;
            let snapshot = self.source.get()?;
            let field = snapshot
                .dataset(dataset)?
                .schema()
                .fields()
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("Unknown event-time field '{field}'."),
                    )
                })?
                .id;
            Ok(Operation {
                dataset,
                mutation: Mutation::SetRetention(RetentionPolicy::EventTime(EventTimeWindow {
                    field,
                    width,
                    watermark,
                    allowed_lateness,
                    late,
                })),
            })
        })();
        match result {
            Ok(op) => self.operations.push(op),
            Err(e) => self.failure = Some(e),
        }
        self
    }
    /// Advance a previously supplied event-time watermark in exact source ticks.
    pub fn watermark(self, target: impl Into<DatasetRef>, watermark: i64) -> Self {
        self.push(target, Mutation::AdvanceWatermark(watermark))
    }
    /// Rebuild first-seen category order from retained rows in the selected named field.
    pub fn reset_categories(mut self, target: impl Into<DatasetRef>, field: &str) -> Self {
        let result = (|| {
            let id = self.resolve(target.into())?;
            let snapshot = self.source.get()?;
            let field = snapshot
                .dataset(id)?
                .schema()
                .fields()
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("Unknown category field '{field}'."),
                    )
                })?;
            Ok(Operation {
                dataset: id,
                mutation: Mutation::ResetCategoryOrder(field.id),
            })
        })();
        match result {
            Ok(op) => self.operations.push(op),
            Err(e) => self.failure = Some(e),
        }
        self
    }
    /// Materialize one version-fenced transaction without mutating the runtime.
    pub fn build(self) -> ChartResult<Transaction> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        let source = self.source.get()?;
        let expected = self
            .operations
            .iter()
            .map(|o| o.dataset)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| source.dataset(id).map(DatasetSnapshot::version))
            .collect::<ChartResult<_>>()?;
        Ok(Transaction {
            id: self.id?,
            epoch: source.epoch(),
            expected,
            operations: self.operations,
        })
    }
}
