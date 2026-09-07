use super::error;
use crate::data::{
    Column, ColumnValues, DataLimits, Field, FieldKind, NormalizedBatch, Schema, TimestampType,
    TypedRows,
};
use crate::{ChartResult, DiagnosticCode, FieldId, SchemaVersion};
use std::{collections::BTreeMap, sync::Arc};

type ExtractColumn<'a, T> = Box<dyn Fn(&[T]) -> Column + 'a>;
struct NativeColumn<'a, T> {
    field: Field,
    extract: ExtractColumn<'a, T>,
}

/// Typed native authoring boundary. Accessors consume `&T`, then materialize immutable
/// normalized values for the same compiler as portable field-based definitions.
/// Closures are not retained in the output or claimed to be serializable computations.
/// Accessors need neither `Send` nor `Sync`; callback execution is synchronous.
pub struct TypedDataBuilder<'a, T> {
    rows: &'a TypedRows<T>,
    version: SchemaVersion,
    columns: Vec<NativeColumn<'a, T>>,
}
macro_rules! column_method {
    ($method:ident,$value:ty,$kind:ident,$description:literal) => {
        #[doc=$description]
        pub fn $method(
            mut self,
            id: FieldId,
            name: impl Into<String>,
            accessor: impl Fn(&T) -> Option<$value> + 'a,
        ) -> Self {
            let field = Field {
                id,
                name: name.into(),
                kind: FieldKind::$kind,
                nullable: true,
                unit: None,
                label: None,
            };
            self.columns.push(NativeColumn {
                field,
                extract: Box::new(move |rows| {
                    let source: Vec<_> = rows.iter().map(&accessor).collect();
                    let validity = source.iter().map(Option::is_some).collect();
                    let values = source.into_iter().map(Option::unwrap_or_default).collect();
                    Column::new(ColumnValues::$kind(values), validity, None)
                }),
            });
            self
        }
    };
}
impl<'a, T> TypedDataBuilder<'a, T> {
    /// Native accessors have no portable executable representation. Materialize a normalized
    /// batch and serialize that owned data instead of attempting to serialize closures.
    pub fn to_portable_spec(&self) -> ChartResult<String> {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Native accessors cannot be serialized as operations; materialize the batch and use portable field mappings.",
        ))
    }

    /// Materialize a typed snapshot into a caller-versioned schema; source keys stay exact.
    pub fn new(rows: &'a TypedRows<T>, version: SchemaVersion) -> Self {
        Self {
            rows,
            version,
            columns: vec![],
        }
    }
    column_method!(
        float,
        f64,
        Float64,
        "Add a nullable binary64 accessor over the exact typed row."
    );
    column_method!(
        int64,
        i64,
        Int64,
        "Add an exact nullable signed-integer accessor."
    );
    column_method!(
        uint64,
        u64,
        UInt64,
        "Add an exact nullable unsigned-integer accessor."
    );
    column_method!(boolean, bool, Boolean, "Add a nullable boolean accessor.");
    column_method!(
        utf8,
        String,
        Utf8,
        "Add owned UTF-8 values, including amounts requiring exact decimal display."
    );
    /// Add categorical labels with a first-seen batch-local dictionary.
    pub fn category(
        mut self,
        id: FieldId,
        name: impl Into<String>,
        accessor: impl Fn(&T) -> Option<String> + 'a,
    ) -> Self {
        let field = Field {
            id,
            name: name.into(),
            kind: FieldKind::Categorical,
            nullable: true,
            unit: None,
            label: None,
        };
        self.columns.push(NativeColumn {
            field,
            extract: Box::new(move |rows| {
                let mut dictionary = vec![];
                let mut known = BTreeMap::new();
                let mut codes = vec![];
                let mut validity = vec![];
                for row in rows {
                    if let Some(label) = accessor(row) {
                        let code = if let Some(code) = known.get(&label) {
                            *code
                        } else {
                            // `finish` caps row count at the u32 representation before callbacks run.
                            let code = dictionary.len() as u32;
                            known.insert(label.clone(), code);
                            dictionary.push(label);
                            code
                        };
                        codes.push(code);
                        validity.push(true);
                    } else {
                        codes.push(0);
                        validity.push(false);
                    }
                }
                Column::new(
                    ColumnValues::Categorical { codes, dictionary },
                    validity,
                    None,
                )
            }),
        });
        self
    }
    /// Add integer timestamp ticks with explicit unit and timezone metadata.
    pub fn timestamp(
        mut self,
        id: FieldId,
        name: impl Into<String>,
        representation: TimestampType,
        accessor: impl Fn(&T) -> Option<i64> + 'a,
    ) -> Self {
        let field = Field {
            id,
            name: name.into(),
            kind: FieldKind::Timestamp(representation),
            nullable: true,
            unit: None,
            label: None,
        };
        self.columns.push(NativeColumn {
            field,
            extract: Box::new(move |rows| {
                let source: Vec<_> = rows.iter().map(&accessor).collect();
                let validity = source.iter().map(Option::is_some).collect();
                Column::new(
                    ColumnValues::Timestamp(
                        source.into_iter().map(Option::unwrap_or_default).collect(),
                    ),
                    validity,
                    None,
                )
            }),
        });
        self
    }
    /// Check row/schema budgets before invoking callbacks, then own immutable normalized columns.
    /// Dataset identity/revision remains on the caller's `TypedRows`; this does not commit data.
    pub fn finish(self, limits: DataLimits) -> ChartResult<NormalizedBatch> {
        if self.rows.rows().len() > limits.max_batch_rows
            || self.rows.rows().len() > u32::MAX as usize
            || self.columns.len() > limits.max_fields
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Typed normalization row/field budget exceeded before accessor execution.",
            ));
        }
        let schema = Arc::new(Schema::new(
            self.version,
            self.columns.iter().map(|c| c.field.clone()).collect(),
        )?);
        let columns = self
            .columns
            .iter()
            .map(|c| (c.extract)(self.rows.rows()))
            .collect();
        NormalizedBatch::new(schema, self.rows.keys().to_vec(), columns, limits)
    }
}
