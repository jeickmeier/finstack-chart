//! Immutable typed rows and normalized columnar snapshots, independent of any host runtime.

mod columns;
mod retention;
mod schema;
mod snapshot;
pub use retention::{EventTimeWindow, LateDataPolicy};

pub use columns::{
    Column, ColumnValues, InvalidPolicy, NormalizedBatch, NumericProjection, ValueRef,
};
pub use schema::{DataLimits, Field, FieldKind, Schema, TimeUnit, TimestampType};
pub use snapshot::{
    DataChunk, DatasetSnapshot, DatasetVersion, RetentionPolicy, RowView, SnapshotHandle,
    StoreSnapshot, TypedRows,
};

use crate::{Diagnostic, DiagnosticCode};

pub(crate) fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the data/schema or resynchronize the expected snapshot before retrying.",
    )
}
