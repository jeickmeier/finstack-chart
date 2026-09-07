use chart_core::{data::*, grammar::*, transaction::*, *};
use std::sync::Arc;
pub const DATA: DatasetId = DatasetId::new(1);
pub const X: FieldId = FieldId::new(1);
pub const Y: FieldId = FieldId::new(2);
pub const GROUP: FieldId = FieldId::new(3);
pub fn limits() -> DataLimits {
    DataLimits {
        max_batch_rows: 1_100_000,
        max_dataset_rows: 1_100_000,
        max_batch_bytes: 256 * 1024 * 1024,
        max_dataset_bytes: 256 * 1024 * 1024,
        ..Default::default()
    }
}
pub fn batch(series: usize, rows: usize, phase: usize) -> ChartResult<NormalizedBatch> {
    let schema = Arc::new(Schema::new(
        SchemaVersion::new(1),
        vec![
            Field {
                id: X,
                name: "x".into(),
                kind: FieldKind::Float64,
                nullable: false,
                unit: None,
                label: None,
            },
            Field {
                id: Y,
                name: "y".into(),
                kind: FieldKind::Float64,
                nullable: true,
                unit: None,
                label: None,
            },
            Field {
                id: GROUP,
                name: "series".into(),
                kind: FieldKind::UInt64,
                nullable: false,
                unit: None,
                label: None,
            },
        ],
    )?);
    let count = series * rows;
    let mut keys = Vec::with_capacity(count);
    let mut x = Vec::with_capacity(count);
    let mut y = Vec::with_capacity(count);
    let mut groups = Vec::with_capacity(count);
    let mut valid = Vec::with_capacity(count);
    for s in 0..series {
        for i in 0..rows {
            keys.push(RowKey::new((s * rows + i + 1) as u64));
            x.push(i as f64);
            y.push(if i % 997 == 0 {
                100. + s as f64
            } else {
                ((i * 13 + s * 7 + phase) % 97) as f64
            });
            groups.push(s as u64);
            valid.push(i % 1729 != 1728);
        }
    }
    NormalizedBatch::new(
        schema,
        keys,
        vec![
            Column::new(ColumnValues::Float64(x), vec![true; count], None),
            Column::new(ColumnValues::Float64(y), valid, None),
            Column::new(ColumnValues::UInt64(groups), vec![true; count], None),
        ],
        limits(),
    )
}
pub fn definition() -> ChartDefinition {
    ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        DATA,
        Geom::Line {
            order: LineOrder::X,
            connect_gaps: false,
        },
        SourceAes::new().x(X).y(Y).group(GROUP),
    ))
}
pub fn store(series: usize, rows: usize) -> ChartResult<DataStore> {
    DataStore::new(
        SourceEpoch::new(1),
        vec![(DATA, batch(series, rows, 0)?)],
        limits(),
    )
}
