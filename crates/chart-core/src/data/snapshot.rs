use super::{Column, DataLimits, FieldKind, NormalizedBatch, Schema, ValueRef, error};
use crate::{
    ChartResult, DatasetId, DiagnosticCode, FieldId, Revision, RowKey, SchemaVersion, SourceEpoch,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Owning immutable handle with explicit local disposal. Clones remain independently valid.
/// Thread transfer requires the normal Rust `T: Send + Sync` bounds; none are forced here.
#[derive(Debug)]
pub struct SnapshotHandle<T> {
    value: Option<Arc<T>>,
}
impl<T> Clone for SnapshotHandle<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}
impl<T> SnapshotHandle<T> {
    pub(crate) fn from_arc(value: Arc<T>) -> Self {
        Self { value: Some(value) }
    }
    /// Access the retained snapshot or report a controlled disposed-handle error.
    pub fn get(&self) -> ChartResult<&T> {
        self.value.as_deref().ok_or_else(|| {
            error(
                DiagnosticCode::DisposedHandle,
                "This snapshot handle has been disposed.",
            )
        })
    }
    /// Release this handle's ownership. Repeated disposal is harmless; clones stay valid.
    pub fn dispose(&mut self) {
        self.value = None;
    }
    /// Whether this particular handle has released its snapshot.
    pub fn is_disposed(&self) -> bool {
        self.value.is_none()
    }
}

/// Typed authoring snapshot. Callers must not mutate logical row values through interior
/// mutability. Normalization into immutable columns is the portable preparation boundary.
#[derive(Debug)]
pub struct TypedRows<T> {
    dataset: DatasetId,
    revision: Revision,
    keys: Vec<RowKey>,
    rows: Vec<T>,
}
impl<T> TypedRows<T> {
    /// Own typed rows without requiring `Clone`, `Send`, an interpreter or a renderer.
    pub fn snapshot(
        dataset: DatasetId,
        revision: Revision,
        keys: Vec<RowKey>,
        rows: Vec<T>,
        max_rows: usize,
    ) -> ChartResult<SnapshotHandle<Self>> {
        if rows.len() > max_rows {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Typed row budget exceeded.",
            ));
        }
        if keys.len() != rows.len() || keys.iter().collect::<BTreeSet<_>>().len() != keys.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Typed rows require one unique stable key per row.",
            ));
        }
        Ok(SnapshotHandle::from_arc(Arc::new(Self {
            dataset,
            revision,
            keys,
            rows,
        })))
    }
    /// Source identity.
    pub fn dataset(&self) -> DatasetId {
        self.dataset
    }
    /// Explicit immutable input revision.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Stable keys in authored order.
    pub fn keys(&self) -> &[RowKey] {
        &self.keys
    }
    /// Borrow the typed rows; their concrete type is erased only at normalization.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }
}

/// Revision-qualified dataset/schema handle used by transactions and provenance.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DatasetVersion {
    /// Dataset identity.
    pub dataset: DatasetId,
    /// Data/ordering/retention revision.
    pub revision: Revision,
    /// Schema version.
    pub schema_version: SchemaVersion,
}

/// Implemented retention policies. Event-time/watermark retention belongs to WP-18.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPolicy {
    /// Keep all rows within the caller's data budget.
    Unbounded,
    /// Keep at most this many rows, evicting oldest insertion ordinals first.
    Count(usize),
}

#[derive(Debug)]
struct ChunkData {
    batch: NormalizedBatch,
    ordinals: Vec<u64>,
}

/// Immutable stored batch plus stable insertion ordinals. Cloning shares row payloads.
#[derive(Clone, Debug)]
pub struct DataChunk(Arc<ChunkData>);
impl DataChunk {
    pub(crate) fn new(batch: NormalizedBatch, ordinals: Vec<u64>) -> Self {
        Self(Arc::new(ChunkData { batch, ordinals }))
    }
    /// Immutable column batch, useful for vectorized preparation and storage-sharing checks.
    pub fn batch(&self) -> &NormalizedBatch {
        &self.0.batch
    }
    /// Stable ordinal for each batch row; independent of key, event time and physical order.
    pub fn ordinals(&self) -> &[u64] {
        &self.0.ordinals
    }
    pub(crate) fn selected(&self, indexes: &[usize]) -> Self {
        Self::new(
            self.batch().selected(indexes),
            indexes.iter().map(|&i| self.ordinals()[i]).collect(),
        )
    }
    pub(crate) fn replaced(
        &self,
        batch: &NormalizedBatch,
        replacements: &BTreeMap<RowKey, usize>,
    ) -> ChartResult<Self> {
        Ok(Self::new(
            self.batch().replaced(batch, replacements)?,
            self.ordinals().to_vec(),
        ))
    }
    fn shares(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// A source row borrowed from an owning immutable dataset snapshot.
#[derive(Clone, Copy, Debug)]
pub struct RowView<'a> {
    pub(crate) chunk: &'a DataChunk,
    pub(crate) index: usize,
}
impl<'a> RowView<'a> {
    /// Durable source key.
    pub fn key(self) -> RowKey {
        self.chunk.batch().keys()[self.index]
    }
    /// Stable insertion ordinal for equal-x tie breaking.
    pub fn ordinal(self) -> u64 {
        self.chunk.ordinals()[self.index]
    }
    /// Read a valid source value by field identity; null/missing fields yield `None`.
    pub fn value(self, field: FieldId) -> Option<ValueRef<'a>> {
        self.column(field)?.value(self.index)
    }
    /// Recover original display text for this cell.
    pub fn formatted(self, field: FieldId) -> Option<&'a str> {
        self.column(field)?.formatted(self.index)
    }
    fn column(self, field: FieldId) -> Option<&'a Column> {
        self.chunk.batch().column(field)
    }
    pub(crate) fn equals(self, other: Self) -> bool {
        self.ordinal() == other.ordinal()
            && self
                .chunk
                .batch()
                .row_eq(self.index, other.chunk.batch(), other.index)
    }
}

/// Immutable dataset state; edits create a new value sharing unchanged chunks.
#[derive(Clone, Debug)]
pub struct DatasetSnapshot {
    pub(crate) id: DatasetId,
    pub(crate) revision: Revision,
    pub(crate) schema: Arc<Schema>,
    pub(crate) chunks: Vec<DataChunk>,
    pub(crate) categories: BTreeMap<FieldId, Arc<[String]>>,
    pub(crate) next_ordinal: u64,
    pub(crate) retention: RetentionPolicy,
}

impl DatasetSnapshot {
    pub(crate) fn empty(id: DatasetId, schema: Arc<Schema>) -> Self {
        Self {
            id,
            revision: Revision::INITIAL,
            categories: schema
                .fields()
                .iter()
                .filter(|f| f.kind == FieldKind::Categorical)
                .map(|f| (f.id, Arc::from([])))
                .collect(),
            schema,
            chunks: Vec::new(),
            next_ordinal: 0,
            retention: RetentionPolicy::Unbounded,
        }
    }
    /// Dataset/schema input stamp.
    pub fn version(&self) -> DatasetVersion {
        DatasetVersion {
            dataset: self.id,
            revision: self.revision,
            schema_version: self.schema.version(),
        }
    }
    /// Immutable schema.
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }
    /// Stored chunks in authored row order. Append clones chunk handles, never old values.
    pub fn chunks(&self) -> &[DataChunk] {
        &self.chunks
    }
    /// Number of retained rows.
    pub fn len(&self) -> usize {
        self.chunks.iter().map(|c| c.batch().len()).sum()
    }
    /// Whether no rows are retained.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
    /// Retained source rows in authored order. No implicit sorting by x or timestamp.
    pub fn rows(&self) -> impl Iterator<Item = RowView<'_>> {
        self.chunks
            .iter()
            .flat_map(|chunk| (0..chunk.batch().len()).map(move |index| RowView { chunk, index }))
    }
    /// Exact key lookup; currently scans chunks, without copying retained row payloads.
    pub fn row(&self, key: RowKey) -> Option<RowView<'_>> {
        self.rows().find(|row| row.key() == key)
    }
    /// First-seen category labels, retained across updates/removals until explicit reset.
    pub fn categories(&self, field: FieldId) -> Option<&[String]> {
        self.categories.get(&field).map(AsRef::as_ref)
    }
    /// Return an explicit domain exactly when supplied, otherwise retained first-seen order.
    /// This does not filter or mutate source data; an explicit domain may include absent labels.
    pub fn category_order(
        &self,
        field: FieldId,
        explicit: Option<&[String]>,
    ) -> ChartResult<Vec<String>> {
        let observed = self
            .categories(field)
            .ok_or_else(|| error(DiagnosticCode::SchemaConflict, "Field is not categorical."))?;
        let order = explicit.unwrap_or(observed);
        if order.iter().collect::<BTreeSet<_>>().len() != order.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Explicit category order contains duplicates.",
            ));
        }
        Ok(order.to_vec())
    }
    /// Current count-retention policy.
    pub fn retention(&self) -> RetentionPolicy {
        self.retention
    }
    pub(crate) fn add_categories(
        &mut self,
        batch: &NormalizedBatch,
        limits: DataLimits,
    ) -> ChartResult<()> {
        for field in batch
            .schema()
            .fields()
            .iter()
            .filter(|f| f.kind == FieldKind::Categorical)
        {
            let old = self
                .categories
                .get(&field.id)
                .cloned()
                .unwrap_or_else(|| Arc::from([]));
            let mut seen: BTreeSet<&str> = old.iter().map(String::as_str).collect();
            let mut added = Vec::new();
            if let Some(column) = batch.column(field.id) {
                for row in 0..batch.len() {
                    if let Some(ValueRef::Category(label)) = column.value(row)
                        && seen.insert(label)
                    {
                        added.push(label);
                    }
                }
            }
            if seen.len() > limits.max_categories {
                let mut e = error(
                    DiagnosticCode::ResourceLimit,
                    "Remembered category budget exceeded; explicitly reset order or raise the budget.",
                );
                e.context.field = Some(field.id);
                return Err(e);
            }
            if !added.is_empty() {
                let mut labels = old.to_vec();
                labels.extend(added.into_iter().map(str::to_owned));
                self.categories.insert(field.id, labels.into());
            }
        }
        Ok(())
    }
    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        if self.schema != other.schema
            || self.next_ordinal != other.next_ordinal
            || self.retention != other.retention
            || self.categories != other.categories
            || self.len() != other.len()
        {
            return false;
        }
        if self.chunks.len() == other.chunks.len()
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .all(|(a, b)| a.shares(b))
        {
            return true;
        }
        self.rows().zip(other.rows()).all(|(a, b)| a.equals(b))
    }
}

/// Coherent immutable view of every dataset at one store commit revision.
#[derive(Clone, Debug)]
pub struct StoreSnapshot {
    pub(crate) epoch: SourceEpoch,
    pub(crate) revision: Revision,
    pub(crate) datasets: BTreeMap<DatasetId, Arc<DatasetSnapshot>>,
    pub(crate) order: Vec<DatasetId>,
}
impl StoreSnapshot {
    /// Captured source epoch.
    pub fn epoch(&self) -> SourceEpoch {
        self.epoch
    }
    /// One revision per effective multi-dataset commit.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Resolve an existing dataset without narrowing identity.
    pub fn dataset(&self, id: DatasetId) -> ChartResult<&DatasetSnapshot> {
        self.datasets.get(&id).map(AsRef::as_ref).ok_or_else(|| {
            let mut e = error(
                DiagnosticCode::MissingResource,
                "Dataset is absent from this snapshot.",
            );
            e.context.dataset = Some(id);
            e
        })
    }
    /// Stable dataset registration order, independent of numeric identity ordering.
    pub fn datasets(&self) -> impl Iterator<Item = &DatasetSnapshot> {
        self.order
            .iter()
            .filter_map(|id| self.datasets.get(id).map(AsRef::as_ref))
    }
}
