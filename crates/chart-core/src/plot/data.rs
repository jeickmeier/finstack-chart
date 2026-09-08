//! Owned authoring data; materialization preserves exact source representations.
use super::{error, fresh_id};
use crate::data::*;
use crate::{ChartResult, DatasetId, DiagnosticCode, FieldId, RowKey, SchemaVersion};
use std::sync::Arc;

/// Owned column input with independent null/display metadata.
#[derive(Clone, Debug)]
pub struct ColumnData {
    values: ColumnValues,
    kind: FieldKind,
    validity: Vec<bool>,
    formatted: Option<Vec<Option<String>>>,
    unit: Option<String>,
    label: Option<String>,
    nullable: bool,
}
impl ColumnData {
    fn new(values: ColumnValues, kind: FieldKind, validity: Vec<bool>, nullable: bool) -> Self {
        Self {
            values,
            kind,
            validity,
            nullable,
            formatted: None,
            unit: None,
            label: None,
        }
    }
    /// Set independent source validity; the payload is retained even for null cells.
    pub fn validity(mut self, validity: Vec<bool>) -> Self {
        self.validity = validity;
        self.nullable = true;
        self
    }
    /// Retain exact original display strings separately from calculation coordinates.
    pub fn formatted(mut self, values: Vec<Option<String>>) -> Self {
        self.formatted = Some(values);
        self
    }
    /// Declare a source unit without performing conversion.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
    /// Set an optional human-readable field label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Declare whether future batches may contain nulls even if this batch does not.
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}
macro_rules! columns {
    ($ty:ty, $kind:ident, $values:ident) => {
        impl From<Vec<$ty>> for ColumnData {
            fn from(values: Vec<$ty>) -> Self {
                let valid = vec![true; values.len()];
                Self::new(
                    ColumnValues::$values(values),
                    FieldKind::$kind,
                    valid,
                    false,
                )
            }
        }
        impl From<Vec<Option<$ty>>> for ColumnData {
            fn from(values: Vec<Option<$ty>>) -> Self {
                let valid = values.iter().map(Option::is_some).collect();
                Self::new(
                    ColumnValues::$values(
                        values.into_iter().map(Option::unwrap_or_default).collect(),
                    ),
                    FieldKind::$kind,
                    valid,
                    true,
                )
            }
        }
    };
}
columns!(f64, Float64, Float64);
columns!(i64, Int64, Int64);
columns!(u64, UInt64, UInt64);
columns!(bool, Boolean, Boolean);
columns!(String, Utf8, Utf8);
macro_rules! widen_columns {
    ($($source:ty => $target:ty),* $(,)?) => {$(
        impl From<Vec<$source>> for ColumnData {
            fn from(values: Vec<$source>) -> Self { values.into_iter().map(|v| v as $target).collect::<Vec<_>>().into() }
        }
        impl From<Vec<Option<$source>>> for ColumnData {
            fn from(values: Vec<Option<$source>>) -> Self { values.into_iter().map(|v| v.map(|v| v as $target)).collect::<Vec<_>>().into() }
        }
    )*};
}
widen_columns!(i8 => i64, i16 => i64, i32 => i64, isize => i64, u8 => u64, u16 => u64, u32 => u64, usize => u64, f32 => f64);
impl<T, const N: usize> From<[T; N]> for ColumnData
where
    Vec<T>: Into<ColumnData>,
{
    fn from(values: [T; N]) -> Self {
        Vec::from(values).into()
    }
}
impl From<Vec<&str>> for ColumnData {
    fn from(values: Vec<&str>) -> Self {
        values
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
            .into()
    }
}
impl From<Vec<Option<&str>>> for ColumnData {
    fn from(values: Vec<Option<&str>>) -> Self {
        values
            .into_iter()
            .map(|v| v.map(str::to_owned))
            .collect::<Vec<_>>()
            .into()
    }
}
/// Enrich ordinary column values with units, display labels, null masks or original text.
pub fn column(values: impl Into<ColumnData>) -> ColumnData {
    values.into()
}
/// Exact signed timestamp ticks with a required unit and explicit timezone metadata.
pub fn timestamps(values: Vec<i64>, unit: TimeUnit, timezone: impl Into<String>) -> ColumnData {
    let valid = vec![true; values.len()];
    ColumnData::new(
        ColumnValues::Timestamp(values),
        FieldKind::Timestamp(TimestampType {
            unit,
            timezone: timezone.into(),
        }),
        valid,
        false,
    )
}
/// Nullable timestamp ticks with independent validity.
pub fn nullable_timestamps(
    values: Vec<Option<i64>>,
    unit: TimeUnit,
    timezone: impl Into<String>,
) -> ColumnData {
    let validity = values.iter().map(Option::is_some).collect();
    timestamps(
        values.into_iter().map(Option::unwrap_or_default).collect(),
        unit,
        timezone,
    )
    .validity(validity)
}
/// Stable string categories; first occurrence defines the batch dictionary order.
pub fn categorical(values: impl IntoIterator<Item = impl Into<String>>) -> ColumnData {
    let mut dictionary = vec![];
    let mut lookup = std::collections::BTreeMap::new();
    let codes: Vec<u32> = values
        .into_iter()
        .map(|v| {
            let value = v.into();
            *lookup.entry(value.clone()).or_insert_with(|| {
                let code = dictionary.len() as u32;
                dictionary.push(value);
                code
            })
        })
        .collect();
    let valid = vec![true; codes.len()];
    ColumnData::new(
        ColumnValues::Categorical { codes, dictionary },
        FieldKind::Categorical,
        valid,
        false,
    )
}
/// Field identity scoped to the data that created it; foreign-owner use rejects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldHandle {
    pub(super) dataset: DatasetId,
    pub(super) field: FieldId,
}
impl FieldHandle {
    /// Exact immutable source field identity for specialist inspection.
    pub fn id(self) -> FieldId {
        self.field
    }
    /// Owning dataset identity.
    pub fn dataset(self) -> DatasetId {
        self.dataset
    }
}
/// Validated owned immutable data, cheap to clone without copying source columns.
#[derive(Clone, Debug)]
pub struct Data {
    pub(super) id: DatasetId,
    pub(super) batch: Arc<NormalizedBatch>,
    pub(super) name: String,
    pub(super) timestamp_origins: Vec<(TimestampType, i64)>,
}
impl Data {
    /// Start a named column batch; values may be nullable and retain exact scalar kinds.
    pub fn columns() -> ColumnsBuilder {
        ColumnsBuilder::default()
    }
    /// Materialize typed accessors once per row/field when added to this builder.
    pub fn rows<T>(rows: impl IntoIterator<Item = T>) -> RowsBuilder<T> {
        RowsBuilder {
            rows: rows.into_iter().collect(),
            columns: Self::columns(),
        }
    }
    /// Stable dataset identity retained by clones and plot edits.
    pub fn id(&self) -> DatasetId {
        self.id
    }
    /// Authored dataset name, defaulting to `data`.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Inspect exact immutable values, keys and metadata.
    pub fn batch(&self) -> &NormalizedBatch {
        &self.batch
    }
    /// Resolve a field once to an owner-scoped handle.
    pub fn field(&self, name: &str) -> ChartResult<FieldHandle> {
        self.batch
            .schema()
            .fields()
            .iter()
            .find(|f| f.name == name)
            .map(|f| FieldHandle {
                dataset: self.id,
                field: f.id,
            })
            .ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("Dataset '{}' has no field '{name}'.", self.name),
                )
            })
    }
}
/// Collect named columns and optional exact row keys; `.build()` validates atomically.
#[derive(Clone, Default)]
pub struct ColumnsBuilder {
    name: Option<String>,
    identity: Option<DatasetId>,
    columns: Vec<(String, ColumnData)>,
    keys: Option<Vec<RowKey>>,
    limits: DataLimits,
    schema_version: Option<SchemaVersion>,
}
impl ColumnsBuilder {
    /// Preserve an imported dataset identity, including identity-seeded jitter behavior.
    /// Ordinary authors omit this. Reuse the same Data clone for references within a plot.
    pub fn identity(mut self, identity: u64) -> Self {
        self.identity = Some(DatasetId::new(identity));
        self
    }
    /// Give the data a name for multi-dataset authoring and diagnostics.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Add one named typed column. Duplicate names and mismatched lengths reject at build.
    pub fn column(mut self, name: impl Into<String>, values: impl Into<ColumnData>) -> Self {
        self.columns.push((name.into(), values.into()));
        self
    }
    /// Supply exact durable row keys instead of automatic fresh allocation.
    pub fn keys(mut self, keys: impl IntoIterator<Item = u64>) -> Self {
        self.keys = Some(keys.into_iter().map(RowKey::new).collect());
        self
    }
    /// Configure explicit data/schema/payload budgets.
    pub fn limits(mut self, limits: DataLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Set an explicit schema migration version; ordinary compatible batches keep version one.
    pub fn schema_version(mut self, version: SchemaVersion) -> Self {
        self.schema_version = Some(version);
        self
    }
    /// Validate and own a batch; no compiler, statistics, layout or host resources are used.
    pub fn build(self) -> ChartResult<Data> {
        let name = self.name.unwrap_or_else(|| "data".into());
        super::validate_name(&name)?;
        let id = match self.identity {
            Some(id) => id,
            None => DatasetId::new(fresh_id()?),
        };
        let len = self.columns.first().map_or(0, |(_, c)| c.values.len());
        if self.columns.len() > self.limits.max_fields || len > self.limits.max_batch_rows {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Data fields/rows exceed configured limits.",
            ));
        }
        let keys = match self.keys {
            Some(keys) => keys,
            None => (0..len)
                .map(|_| fresh_id().map(RowKey::new))
                .collect::<ChartResult<_>>()?,
        };
        let mut fields = vec![];
        let mut columns = vec![];
        for (index, (name, column)) in self.columns.into_iter().enumerate() {
            super::validate_name(&name)?;
            fields.push(Field {
                id: FieldId::new(index as u64 + 1),
                name,
                kind: column.kind,
                nullable: column.nullable,
                unit: column.unit,
                label: column.label,
            });
            columns.push(Column::new(
                column.values,
                column.validity,
                column.formatted,
            ));
        }
        let schema = Arc::new(Schema::new(
            self.schema_version.unwrap_or(SchemaVersion::new(1)),
            fields,
        )?);
        let batch = NormalizedBatch::new(schema, keys, columns, self.limits)?;
        Ok(Data {
            id,
            batch: Arc::new(batch),
            name,
            timestamp_origins: vec![],
        })
    }
}

pub(super) fn align_timestamp_origins(datasets: &mut [Data]) {
    let mut origins: Vec<(TimestampType, i64)> = vec![];
    for data in datasets.iter() {
        for (field, column) in data
            .batch
            .schema()
            .fields()
            .iter()
            .zip(data.batch.columns())
        {
            if let (FieldKind::Timestamp(kind), ColumnValues::Timestamp(values)) =
                (&field.kind, column.values())
                && !origins.iter().any(|(seen, _)| seen == kind)
                && let Some(value) = values
                    .iter()
                    .zip(column.validity())
                    .find_map(|(v, valid)| valid.then_some(*v))
            {
                origins.push((kind.clone(), value));
            }
        }
    }
    for data in datasets {
        data.timestamp_origins.clone_from(&origins);
    }
}
/// Typed native rows. Accessors are consumed during materialization and never serialized.
pub struct RowsBuilder<T> {
    rows: Vec<T>,
    columns: ColumnsBuilder,
}
impl<T> RowsBuilder<T> {
    /// Preserve an explicitly imported dataset identity; ordinary rows use automatic allocation.
    pub fn identity(mut self, identity: u64) -> Self {
        self.columns = self.columns.identity(identity);
        self
    }
    /// Set the dataset name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.columns = self.columns.name(name);
        self
    }
    /// Materialize one accessor into an exact scalar or nullable column.
    pub fn field<V>(mut self, name: impl Into<String>, accessor: impl Fn(&T) -> V) -> Self
    where
        Vec<V>: Into<ColumnData>,
    {
        let values: Vec<V> = self.rows.iter().map(accessor).collect();
        self.columns = self.columns.column(name, values);
        self
    }
    /// Materialize exact timestamp ticks with declared unit/timezone metadata.
    pub fn timestamp(
        mut self,
        name: impl Into<String>,
        accessor: impl Fn(&T) -> i64,
        unit: TimeUnit,
        timezone: impl Into<String>,
    ) -> Self {
        self.columns = self.columns.column(
            name,
            timestamps(self.rows.iter().map(accessor).collect(), unit, timezone),
        );
        self
    }
    /// Materialize categorical labels; values retain first-seen dictionary identity.
    pub fn category(mut self, name: impl Into<String>, accessor: impl Fn(&T) -> String) -> Self {
        self.columns = self
            .columns
            .column(name, categorical(self.rows.iter().map(accessor)));
        self
    }
    /// Materialize supplied durable keys once per source row.
    pub fn keys(mut self, accessor: impl Fn(&T) -> u64) -> Self {
        self.columns = self.columns.keys(self.rows.iter().map(accessor));
        self
    }
    /// Configure data budgets.
    pub fn limits(mut self, limits: DataLimits) -> Self {
        self.columns = self.columns.limits(limits);
        self
    }
    /// Validate all materialized fields and own the immutable batch.
    pub fn build(self) -> ChartResult<Data> {
        self.columns.build()
    }
}
