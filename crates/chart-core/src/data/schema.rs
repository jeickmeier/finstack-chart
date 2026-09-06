use super::error;
use crate::{ChartResult, DiagnosticCode, FieldId, SchemaVersion};
use std::{collections::BTreeSet, sync::Arc};

/// Source timestamp integer unit. No conversion to floating point is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeUnit {
    /// Whole seconds.
    Seconds,
    /// Thousandths of a second.
    Milliseconds,
    /// Millionths of a second.
    Microseconds,
    /// Billionths of a second.
    Nanoseconds,
}

/// Explicit timestamp representation; timezone is metadata, not a timezone database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimestampType {
    /// Unit of each signed 64-bit source value.
    pub unit: TimeUnit,
    /// Nonempty explicit timezone, for example `UTC` or `America/Toronto`.
    pub timezone: String,
}

/// Required portable column kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldKind {
    /// IEEE binary64 values; non-finite source values remain recoverable.
    Float64,
    /// Signed exact 64-bit source integers.
    Int64,
    /// Unsigned exact 64-bit source integers.
    UInt64,
    /// Boolean values.
    Boolean,
    /// Owned UTF-8 strings.
    Utf8,
    /// Dictionary codes whose category identity is the decoded label.
    Categorical,
    /// Signed integer timestamp with explicit unit and timezone.
    Timestamp(TimestampType),
}

/// Stable field identity plus optional presentation metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    /// Stable identity; schema order is not identity.
    pub id: FieldId,
    /// Nonempty unique name.
    pub name: String,
    /// Source representation.
    pub kind: FieldKind,
    /// Whether validity may mark a row null.
    pub nullable: bool,
    /// Optional unit label; no finance/unit conversion engine is implied.
    pub unit: Option<String>,
    /// Optional display label.
    pub label: Option<String>,
}

/// Validated immutable schema shared by batches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schema {
    version: SchemaVersion,
    fields: Arc<[Field]>,
}

impl Schema {
    /// Validate unique field IDs/names and explicit timestamp metadata, then own fields.
    pub fn new(version: SchemaVersion, fields: Vec<Field>) -> ChartResult<Self> {
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for field in &fields {
            if field.name.trim().is_empty() || !ids.insert(field.id) || !names.insert(&field.name) {
                let mut e = error(
                    DiagnosticCode::SchemaConflict,
                    "Field IDs and nonempty names must be unique.",
                );
                e.context.field = Some(field.id);
                return Err(e);
            }
            if let FieldKind::Timestamp(time) = &field.kind
                && time.timezone.trim().is_empty()
            {
                let mut e = error(
                    DiagnosticCode::SchemaConflict,
                    "A timestamp requires explicit timezone metadata.",
                );
                e.context.field = Some(field.id);
                return Err(e);
            }
        }
        Ok(Self {
            version,
            fields: fields.into(),
        })
    }
    /// Exact schema version.
    pub fn version(&self) -> SchemaVersion {
        self.version
    }
    /// Immutable fields in column order.
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
    /// Resolve an identity to its field and column position.
    pub fn field(&self, id: FieldId) -> Option<(usize, &Field)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.id == id)
    }
    pub(crate) fn payload_bytes(&self) -> usize {
        self.fields.iter().fold(16usize, |size, field| {
            size.saturating_add(
                64usize
                    .saturating_add(field.name.len())
                    .saturating_add(field.unit.as_ref().map_or(0, String::len))
                    .saturating_add(field.label.as_ref().map_or(0, String::len))
                    .saturating_add(if let FieldKind::Timestamp(t) = &field.kind {
                        t.timezone.len()
                    } else {
                        0
                    }),
            )
        })
    }
}

/// Explicit data/replay work budgets. Payload byte charges are conservative accounting,
/// not measured RSS; callers own allocation before passing input to core.
#[derive(Clone, Copy, Debug)]
pub struct DataLimits {
    /// Maximum datasets in one store.
    pub max_datasets: usize,
    /// Maximum fields in a schema.
    pub max_fields: usize,
    /// Maximum rows in an input batch.
    pub max_batch_rows: usize,
    /// Maximum retained rows per dataset after retention.
    pub max_dataset_rows: usize,
    /// Maximum charged retained chunk/category bytes per dataset.
    pub max_dataset_bytes: usize,
    /// Maximum remembered first-seen category labels per field, including removed rows.
    pub max_categories: usize,
    /// Maximum charged bytes in a normalized batch.
    pub max_batch_bytes: usize,
    /// Maximum rows copied when one stored chunk must change.
    pub chunk_rows: usize,
    /// Maximum ordered operations in one transaction.
    pub max_operations: usize,
    /// Maximum charged transaction payload bytes.
    pub max_transaction_bytes: usize,
    /// Maximum remembered committed transaction IDs (FIFO horizon).
    pub dedup_entries: usize,
    /// Maximum charged payload/receipt bytes in the deduplication horizon.
    pub dedup_bytes: usize,
}

impl Default for DataLimits {
    fn default() -> Self {
        Self {
            max_datasets: 256,
            max_fields: 1024,
            max_batch_rows: 100_000,
            max_dataset_rows: 1_000_000,
            max_dataset_bytes: 256 * 1024 * 1024,
            max_categories: 100_000,
            max_batch_bytes: 32 * 1024 * 1024,
            chunk_rows: 4096,
            max_operations: 1024,
            max_transaction_bytes: 32 * 1024 * 1024,
            dedup_entries: 128,
            dedup_bytes: 64 * 1024 * 1024,
        }
    }
}

impl DataLimits {
    pub(crate) fn validate(self) -> ChartResult<()> {
        if self.chunk_rows == 0 || self.dedup_entries == 0 || self.dedup_bytes == 0 {
            return Err(error(
                DiagnosticCode::Validation,
                "Chunk size and replay cache capacities must be positive.",
            ));
        }
        Ok(())
    }
}
