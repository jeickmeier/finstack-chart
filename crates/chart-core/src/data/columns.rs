use super::{DataLimits, FieldKind, Schema, error};
use crate::{ChartResult, Diagnostic, DiagnosticCode, FieldId, RowKey, Severity};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Physical column payload. Validity lives separately in [`Column`].
#[derive(Clone, Debug)]
pub enum ColumnValues {
    /// Source binary64 values, including recoverable non-finite values.
    Float64(Vec<f64>),
    /// Exact signed integers.
    Int64(Vec<i64>),
    /// Exact unsigned integers.
    UInt64(Vec<u64>),
    /// Booleans.
    Boolean(Vec<bool>),
    /// UTF-8 source text.
    Utf8(Vec<String>),
    /// Batch-local dictionary; only valid rows require an in-range code.
    Categorical {
        /// Batch-local codes, not durable category IDs.
        codes: Vec<u32>,
        /// Unique category labels; labels define category identity.
        dictionary: Vec<String>,
    },
    /// Integer timestamp ticks; schema carries unit and timezone.
    Timestamp(Vec<i64>),
}

impl PartialEq for ColumnValues {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float64(a), Self::Float64(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
            }
            (Self::Int64(a), Self::Int64(b)) | (Self::Timestamp(a), Self::Timestamp(b)) => a == b,
            (Self::UInt64(a), Self::UInt64(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Utf8(a), Self::Utf8(b)) => a == b,
            (
                Self::Categorical {
                    codes: a,
                    dictionary: ad,
                },
                Self::Categorical {
                    codes: b,
                    dictionary: bd,
                },
            ) => a == b && ad == bd,
            _ => false,
        }
    }
}
impl Eq for ColumnValues {}

impl ColumnValues {
    /// Physical row count.
    pub fn len(&self) -> usize {
        match self {
            Self::Float64(v) => v.len(),
            Self::Int64(v) | Self::Timestamp(v) => v.len(),
            Self::UInt64(v) => v.len(),
            Self::Boolean(v) => v.len(),
            Self::Utf8(v) => v.len(),
            Self::Categorical { codes, .. } => codes.len(),
        }
    }
    /// Whether this payload contains no rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn matches(&self, kind: &FieldKind) -> bool {
        matches!(
            (self, kind),
            (Self::Float64(_), FieldKind::Float64)
                | (Self::Int64(_), FieldKind::Int64)
                | (Self::UInt64(_), FieldKind::UInt64)
                | (Self::Boolean(_), FieldKind::Boolean)
                | (Self::Utf8(_), FieldKind::Utf8)
                | (Self::Categorical { .. }, FieldKind::Categorical)
                | (Self::Timestamp(_), FieldKind::Timestamp(_))
        )
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Utf8(v) => v
                .iter()
                .fold(v.len().saturating_mul(24), |n, s| n.saturating_add(s.len())),
            Self::Categorical { codes, dictionary } => dictionary
                .iter()
                .fold(codes.len().saturating_mul(4), |n, s| {
                    n.saturating_add(24).saturating_add(s.len())
                }),
            Self::Boolean(v) => v.len(),
            _ => self.len().saturating_mul(8),
        }
    }
    fn selected(&self, indexes: &[usize]) -> Self {
        match self {
            Self::Float64(v) => Self::Float64(indexes.iter().map(|&i| v[i]).collect()),
            Self::Int64(v) => Self::Int64(indexes.iter().map(|&i| v[i]).collect()),
            Self::UInt64(v) => Self::UInt64(indexes.iter().map(|&i| v[i]).collect()),
            Self::Boolean(v) => Self::Boolean(indexes.iter().map(|&i| v[i]).collect()),
            Self::Utf8(v) => Self::Utf8(indexes.iter().map(|&i| v[i].clone()).collect()),
            Self::Timestamp(v) => Self::Timestamp(indexes.iter().map(|&i| v[i]).collect()),
            Self::Categorical { codes, dictionary } => Self::Categorical {
                codes: indexes.iter().map(|&i| codes[i]).collect(),
                dictionary: dictionary.clone(),
            },
        }
    }
}

/// One source value borrowed from a valid cell. Non-finite floats are still source data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueRef<'a> {
    /// Floating source value.
    Float64(f64),
    /// Exact signed source value.
    Int64(i64),
    /// Exact unsigned source value.
    UInt64(u64),
    /// Boolean source value.
    Boolean(bool),
    /// Source UTF-8 text.
    Utf8(&'a str),
    /// Stable category label, independent of dictionary code.
    Category(&'a str),
    /// Exact source timestamp ticks.
    Timestamp(i64),
}

/// Values, independent null validity, and optional original formatted cell strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Column {
    values: ColumnValues,
    validity: Vec<bool>,
    formatted: Option<Vec<Option<String>>>,
}

impl Column {
    /// Assemble input; [`NormalizedBatch::new`] validates lengths, schema and codes.
    pub fn new(
        values: ColumnValues,
        validity: Vec<bool>,
        formatted: Option<Vec<Option<String>>>,
    ) -> Self {
        Self {
            values,
            validity,
            formatted,
        }
    }
    /// Immutable exact payload; consumers must honor the independent validity mask.
    pub fn values(&self) -> &ColumnValues {
        &self.values
    }
    /// True means non-null; it does not certify float finiteness for a numeric operation.
    pub fn validity(&self) -> &[bool] {
        &self.validity
    }
    /// Recover the original exact display text independently of rendered coordinates.
    pub fn formatted(&self, row: usize) -> Option<&str> {
        self.formatted.as_ref()?.get(row)?.as_deref()
    }
    /// Read a valid value; null or an out-of-range row returns `None`.
    pub fn value(&self, row: usize) -> Option<ValueRef<'_>> {
        if !self.validity.get(row).copied().unwrap_or(false) {
            return None;
        }
        Some(match &self.values {
            ColumnValues::Float64(v) => ValueRef::Float64(*v.get(row)?),
            ColumnValues::Int64(v) => ValueRef::Int64(*v.get(row)?),
            ColumnValues::UInt64(v) => ValueRef::UInt64(*v.get(row)?),
            ColumnValues::Boolean(v) => ValueRef::Boolean(*v.get(row)?),
            ColumnValues::Utf8(v) => ValueRef::Utf8(v.get(row)?),
            ColumnValues::Timestamp(v) => ValueRef::Timestamp(*v.get(row)?),
            ColumnValues::Categorical { codes, dictionary } => {
                ValueRef::Category(dictionary.get(usize::try_from(*codes.get(row)?).ok()?)?)
            }
        })
    }
    fn selected(&self, indexes: &[usize]) -> Self {
        Self {
            values: self.values.selected(indexes),
            validity: indexes.iter().map(|&i| self.validity[i]).collect(),
            formatted: self
                .formatted
                .as_ref()
                .map(|v| indexes.iter().map(|&i| v[i].clone()).collect()),
        }
    }
    fn replace(&mut self, to: usize, from: &Self, row: usize) -> ChartResult<()> {
        match (&mut self.values, &from.values) {
            (ColumnValues::Float64(a), ColumnValues::Float64(b)) => a[to] = b[row],
            (ColumnValues::Int64(a), ColumnValues::Int64(b))
            | (ColumnValues::Timestamp(a), ColumnValues::Timestamp(b)) => a[to] = b[row],
            (ColumnValues::UInt64(a), ColumnValues::UInt64(b)) => a[to] = b[row],
            (ColumnValues::Boolean(a), ColumnValues::Boolean(b)) => a[to] = b[row],
            (ColumnValues::Utf8(a), ColumnValues::Utf8(b)) => a[to].clone_from(&b[row]),
            (ColumnValues::Categorical { codes, dictionary }, ColumnValues::Categorical { .. }) => {
                codes[to] = if let Some(ValueRef::Category(label)) = from.value(row) {
                    let i = dictionary
                        .iter()
                        .position(|s| s == label)
                        .unwrap_or_else(|| {
                            dictionary.push(label.to_owned());
                            dictionary.len() - 1
                        });
                    u32::try_from(i).map_err(|_| {
                        error(
                            DiagnosticCode::ResourceLimit,
                            "Updated category dictionary exceeds its code representation.",
                        )
                    })?
                } else {
                    0
                };
            }
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Replacement column types differ.",
                ));
            }
        }
        self.validity[to] = from.validity[row];
        if from.formatted.is_some() && self.formatted.is_none() {
            self.formatted = Some(vec![None; self.validity.len()]);
        }
        if let Some(values) = &mut self.formatted {
            values[to] = from.formatted.as_ref().and_then(|v| v[row].clone());
        }
        Ok(())
    }
    fn row_eq(&self, row: usize, other: &Self, other_row: usize) -> bool {
        if self.validity[row] != other.validity[other_row]
            || self.formatted(row) != other.formatted(other_row)
        {
            return false;
        }
        if !self.validity[row] {
            return true;
        }
        match (self.value(row), other.value(other_row)) {
            (Some(ValueRef::Float64(a)), Some(ValueRef::Float64(b))) => a.to_bits() == b.to_bits(),
            (a, b) => a == b,
        }
    }
    fn bytes(&self) -> usize {
        self.values
            .bytes()
            .saturating_add(self.validity.len())
            .saturating_add(self.formatted.as_ref().map_or(0, |v| {
                v.iter().fold(v.len().saturating_mul(24), |n, s| {
                    n.saturating_add(s.as_ref().map_or(0, String::len))
                })
            }))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct BatchData {
    schema: Arc<Schema>,
    keys: Vec<RowKey>,
    columns: Vec<Column>,
}

/// Validated immutable column batch. Cloning shares its payload without cloning values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedBatch(Arc<BatchData>);

impl NormalizedBatch {
    /// Check limits, exact column kinds/lengths, validity, dictionaries and unique keys.
    pub fn new(
        schema: Arc<Schema>,
        keys: Vec<RowKey>,
        columns: Vec<Column>,
        limits: DataLimits,
    ) -> ChartResult<Self> {
        let batch = Self(Arc::new(BatchData {
            schema,
            keys,
            columns,
        }));
        batch.validate(limits)?;
        Ok(batch)
    }
    pub(crate) fn validate(&self, limits: DataLimits) -> ChartResult<()> {
        if self.len() > limits.max_batch_rows
            || self.schema().fields().len() > limits.max_fields
            || self.payload_bytes() > limits.max_batch_bytes
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Batch row, field or payload budget exceeded.",
            ));
        }
        if self.0.columns.len() != self.schema().fields().len() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Column count disagrees with the schema.",
            ));
        }
        if self.0.keys.iter().collect::<BTreeSet<_>>().len() != self.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Batch keys must be unique, including upsert batches.",
            ));
        }
        for (field, col) in self.schema().fields().iter().zip(&self.0.columns) {
            let invalid = col.values.len() != self.len()
                || col.validity.len() != self.len()
                || col
                    .formatted
                    .as_ref()
                    .is_some_and(|v| v.len() != self.len())
                || !col.values.matches(&field.kind)
                || (!field.nullable && col.validity.contains(&false));
            if invalid {
                let mut e = error(
                    DiagnosticCode::SchemaConflict,
                    "Column kind, length or validity disagrees with its field.",
                );
                e.context.field = Some(field.id);
                return Err(e);
            }
            if let ColumnValues::Categorical { codes, dictionary } = &col.values
                && (dictionary.len() > u32::MAX as usize
                    || dictionary.iter().collect::<BTreeSet<_>>().len() != dictionary.len()
                    || codes.iter().zip(&col.validity).any(|(&code, &valid)| {
                        valid && usize::try_from(code).map_or(true, |i| i >= dictionary.len())
                    }))
            {
                let mut e = error(
                    DiagnosticCode::SchemaConflict,
                    "Category labels must be unique and valid row codes must resolve.",
                );
                e.context.field = Some(field.id);
                return Err(e);
            }
        }
        Ok(())
    }
    /// Shared immutable schema.
    pub fn schema(&self) -> &Arc<Schema> {
        &self.0.schema
    }
    /// Stable caller-supplied keys in authored order.
    pub fn keys(&self) -> &[RowKey] {
        &self.0.keys
    }
    /// Immutable columns in schema order.
    pub fn columns(&self) -> &[Column] {
        &self.0.columns
    }
    /// Find a column by stable field identity.
    pub fn column(&self, field: FieldId) -> Option<&Column> {
        self.schema().field(field).map(|(i, _)| &self.0.columns[i])
    }
    /// Number of source rows.
    pub fn len(&self) -> usize {
        self.0.keys.len()
    }
    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.0.keys.is_empty()
    }
    /// Conservatively charged logical payload bytes for work/replay budgets.
    pub fn payload_bytes(&self) -> usize {
        self.0.columns.iter().fold(
            self.schema()
                .payload_bytes()
                .saturating_add(self.len().saturating_mul(8)),
            |n, c| n.saturating_add(c.bytes()),
        )
    }
    pub(crate) fn selected(&self, indexes: &[usize]) -> Self {
        Self(Arc::new(BatchData {
            schema: self.schema().clone(),
            keys: indexes.iter().map(|&i| self.0.keys[i]).collect(),
            columns: self.0.columns.iter().map(|c| c.selected(indexes)).collect(),
        }))
    }
    pub(crate) fn replaced(
        &self,
        incoming: &Self,
        replacements: &BTreeMap<RowKey, usize>,
    ) -> ChartResult<Self> {
        let mut columns = self.0.columns.clone();
        for (to, key) in self.keys().iter().enumerate() {
            if let Some(&from) = replacements.get(key) {
                for (a, b) in columns.iter_mut().zip(incoming.columns()) {
                    a.replace(to, b, from)?;
                }
            }
        }
        Ok(Self(Arc::new(BatchData {
            schema: self.schema().clone(),
            keys: self.keys().to_vec(),
            columns,
        })))
    }
    pub(crate) fn row_eq(&self, row: usize, other: &Self, other_row: usize) -> bool {
        self.keys()[row] == other.keys()[other_row]
            && self.columns().len() == other.columns().len()
            && self
                .columns()
                .iter()
                .zip(other.columns())
                .all(|(a, b)| a.row_eq(row, b, other_row))
    }
    /// Project one numeric column without converting null/non-finite values to zero.
    /// Timestamp projection requires an integer origin and returns relative source ticks.
    /// Integer magnitudes beyond 2^53 are rejected conservatively; source values remain intact.
    pub fn project_numeric(
        &self,
        field: FieldId,
        policy: InvalidPolicy,
        timestamp_origin: Option<i64>,
        sample_limit: usize,
    ) -> ChartResult<NumericProjection> {
        let column = self
            .column(field)
            .ok_or_else(|| error(DiagnosticCode::SchemaConflict, "Numeric field is absent."))?;
        let (_, descriptor) = self
            .schema()
            .field(field)
            .ok_or_else(|| error(DiagnosticCode::SchemaConflict, "Numeric field is absent."))?;
        if !matches!(
            descriptor.kind,
            FieldKind::Float64 | FieldKind::Int64 | FieldKind::UInt64 | FieldKind::Timestamp(_)
        ) || (matches!(descriptor.kind, FieldKind::Timestamp(_)) && timestamp_origin.is_none())
        {
            let mut e = error(
                DiagnosticCode::UnsupportedCapability,
                "Numeric projection needs a numeric field and an explicit integer timestamp origin.",
            );
            e.context.field = Some(field);
            return Err(e);
        }
        let mut projection = NumericProjection {
            values: Vec::with_capacity(self.len()),
            nulls: 0,
            non_finite: 0,
            precision_loss: 0,
            diagnostic: None,
        };
        let mut samples = Vec::new();
        for row in 0..self.len() {
            let value = match column.value(row) {
                None => {
                    projection.nulls += 1;
                    None
                }
                Some(ValueRef::Float64(v)) if v.is_finite() => Some(v),
                Some(ValueRef::Float64(_)) => {
                    projection.non_finite += 1;
                    None
                }
                Some(ValueRef::Int64(v)) if v.unsigned_abs() <= 1_u64 << 53 => Some(v as f64),
                Some(ValueRef::UInt64(v)) if v <= 1_u64 << 53 => Some(v as f64),
                Some(ValueRef::Timestamp(v)) => match timestamp_origin
                    .and_then(|origin| v.checked_sub(origin))
                    .filter(|delta| delta.unsigned_abs() <= 1_u64 << 53)
                {
                    Some(delta) => Some(delta as f64),
                    None => {
                        projection.precision_loss += 1;
                        None
                    }
                },
                _ => {
                    projection.precision_loss += 1;
                    None
                }
            };
            if value.is_none() && samples.len() < sample_limit.min(32) {
                samples.push(self.keys()[row]);
            }
            projection.values.push(value);
        }
        let count = projection.nulls + projection.non_finite + projection.precision_loss;
        if count > 0 {
            let mut e = error(
                if projection.precision_loss > 0 {
                    DiagnosticCode::PrecisionLoss
                } else {
                    DiagnosticCode::NumericalDomain
                },
                format!(
                    "Numeric projection excluded {count} rows: {} null, {} non-finite, {} precision loss.",
                    projection.nulls, projection.non_finite, projection.precision_loss
                ),
            );
            e.context.field = Some(field);
            e.context.affected_rows = count as u64;
            e.context.row_samples = samples;
            if policy == InvalidPolicy::Strict {
                return Err(e);
            }
            e.severity = Severity::Warning;
            projection.diagnostic = Some(e);
        }
        Ok(projection)
    }
}

/// Invalid channel policy for numeric projection only; ingestion preserves source data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidPolicy {
    /// Return gaps and a bounded aggregate diagnostic.
    Exclude,
    /// Reject the requested projection if any row is missing/invalid/imprecise.
    Strict,
}

/// Finite numeric values or explicit gaps, with separate reason counts.
#[derive(Clone, Debug)]
pub struct NumericProjection {
    /// One entry per input row; no gap is replaced by zero.
    pub values: Vec<Option<f64>>,
    /// Missing values.
    pub nulls: usize,
    /// Non-finite floating values.
    pub non_finite: usize,
    /// Values that require a more suitable integer origin/resolution.
    pub precision_loss: usize,
    /// One aggregate warning with at most 32 sampled row keys.
    pub diagnostic: Option<Diagnostic>,
}
