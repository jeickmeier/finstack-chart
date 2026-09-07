//! Focused exact-bin update work, including immutable output membership materialization.
//! This is a local comparison, not a full PERF acceptance runner.
use chart_core::{data::*, grammar::*, state::ChartState, transaction::*, *};
use std::time::Instant;
const DATA: DatasetId = DatasetId::new(1);
fn batch(start: u64, count: usize) -> NormalizedBatch {
    let rows = TypedRows::snapshot(
        DATA,
        Revision::INITIAL,
        (start..start + count as u64).map(RowKey::new).collect(),
        (start..start + count as u64)
            .map(|k| (k % 1000) as f64 / 10.)
            .collect(),
        count,
    )
    .unwrap();
    TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(FieldId::new(1), "value", |v| Some(*v))
        .finish(DataLimits::default())
        .unwrap()
}
fn main() -> ChartResult<()> {
    let row_count = 20_000;
    let updates = 64;
    let batch_rows = 128;
    let mut store = DataStore::new(
        SourceEpoch::new(1),
        vec![(DATA, batch(0, row_count))],
        DataLimits {
            chunk_rows: 256,
            ..DataLimits::default()
        },
    )?;
    let apply = |store: &mut DataStore, id: String, mutation| {
        let expected = vec![
            store
                .snapshot()
                .get()
                .unwrap()
                .dataset(DATA)
                .unwrap()
                .version(),
        ];
        let r = store.apply(Transaction {
            id: TransactionId::new(id).unwrap(),
            epoch: SourceEpoch::new(1),
            expected,
            operations: vec![Operation {
                dataset: DATA,
                mutation,
            }],
        });
        assert!(matches!(r, CommitOutcome::Applied(_)));
    };
    apply(
        &mut store,
        "retention".into(),
        Mutation::SetRetention(RetentionPolicy::Count(row_count)),
    );
    let definition = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        LayerId::new(1),
        DATA,
        BinSpec::new(FieldId::new(1), (0..=10).map(|n| n as f64 * 10.).collect()),
    ));
    let mut compiler = Compiler::new();
    compiler.prepare(
        &definition,
        &store.snapshot(),
        &ChartState::default(),
        CompileLimits::default(),
    )?;
    let (mut updated_ns, mut batch_ns, mut evaluated, mut reused) = (0u128, 0u128, 0usize, 0usize);
    for i in 0..updates {
        apply(
            &mut store,
            format!("update-{i}"),
            Mutation::AppendBatch(batch((row_count + i * batch_rows) as u64, batch_rows)),
        );
        let begin = Instant::now();
        let updated = compiler.prepare(
            &definition,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )?;
        updated_ns += begin.elapsed().as_nanos();
        let work = compiler.update_metrics();
        evaluated += work.evaluated_rows;
        reused += work.reused_rows;
        let begin = Instant::now();
        let full = Compiler::new().prepare(
            &definition,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )?;
        batch_ns += begin.elapsed().as_nanos();
        assert_eq!(updated.layers()[0].table(), full.layers()[0].table());
        assert_eq!(updated.domains(), full.domains());
        assert!(store.snapshot().get()?.dataset(DATA)?.len() <= row_count);
    }
    println!(
        "{}",
        serde_json::json!({"retained_rows":row_count,"updates":updates,"incoming_rows_per_update":batch_rows,"updated_prepare_ns":updated_ns.to_string(),"fresh_prepare_ns":batch_ns.to_string(),"updated_evaluated_rows":evaluated,"updated_reused_rows":reused,"fresh_evaluated_rows":row_count*updates,"cached_chunks_final":compiler.update_metrics().retained_chunks,"equivalent_tables_domains":true,"scope":"single release run; excludes transaction ingestion and native paint; immutable memberships materialized every prepare"})
    );
    Ok(())
}
