//! FIX-AUTH01: typed execution, legacy parity, source ownership and structural-only build.
use chart_core::{data::*, grammar::*, portable::*, runtime::Chart, state::*, transaction::*, *};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
const CHART: &str = include_str!("../../../fixtures/bindings/chart.json");
const DATA: &str = include_str!("../../../fixtures/bindings/data.json");
const TX: &str = include_str!("../../../fixtures/bindings/correction.json");

#[test]
fn primary_edits_preserve_live_rows_retry_history_and_component_identity() {
    use chart_core::prelude::*;
    let batch = |xs: Vec<f64>| {
        Data::columns()
            .name("prices")
            .column("x", xs.clone())
            .column("y", xs)
            .build()
            .unwrap()
    };
    let initial = batch(vec![1., 2.]);
    let original = plot(initial.clone())
        .aes(aes().x("x").y("y"))
        .layer(points().name("prices"))
        .build()
        .unwrap();
    let id = original.layer("prices").unwrap();
    let mut chart = original.chart().unwrap();
    let incoming = batch(vec![3.]);
    let tx = chart
        .transaction()
        .unwrap()
        .id("append-one")
        .append(chart.data("prices").unwrap(), &incoming)
        .build()
        .unwrap();
    let stale = chart
        .transaction()
        .unwrap()
        .append("prices", batch(vec![4.]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx.clone()).unwrap(),
        CommitOutcome::Applied(_)
    ));
    chart.enqueue(tx.clone()).unwrap();
    let source = chart.source();
    let history = chart.dedup_horizon();
    let edited = original
        .edit()
        .title(title("Live prices"))
        .layer("prices", line())
        .axis(x_axis().label("Time"))
        .build()
        .unwrap();
    assert_eq!(edited.layer("prices").unwrap(), id);
    let revision = chart.definition().revision;
    assert!(chart.apply_plot(&edited, revision).unwrap());
    assert!(std::ptr::eq(
        source.get().unwrap(),
        chart.source().get().unwrap()
    ));
    assert_eq!(chart.dedup_horizon(), history);
    assert_eq!(chart.queue_status().transactions, 1);
    let prepared = chart.prepare().unwrap();
    let PreparedGeometry::LineRun(vertices) = &prepared.layers()[0].marks()[0].geometry else {
        panic!("line")
    };
    assert_eq!(
        vertices.iter().map(|p| (p.x(), p.y())).collect::<Vec<_>>(),
        [(1., 1.), (2., 2.), (3., 3.)]
    );
    assert!(matches!(
        chart.apply_transaction(stale).unwrap(),
        CommitOutcome::Conflict(_)
    ));
    assert!(matches!(
        chart.commit_next().unwrap(),
        Some((_, CommitOutcome::AlreadyApplied(_)))
    ));
    assert_eq!(
        chart.apply_plot(&original, revision).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    let unchanged = edited.edit().build().unwrap();
    assert_eq!(unchanged.definition(), edited.definition());
    let current_revision = chart.definition().revision;
    assert!(!chart.apply_plot(&unchanged, current_revision).unwrap());
    assert!(
        edited
            .edit()
            .layer("prices", points().data(batch(vec![9.])))
            .build()
            .is_err()
    );
}

#[test]
fn primary_transaction_rollback_retention_and_epoch_fences_are_atomic() {
    use chart_core::prelude::*;
    let batch = |values: Vec<f64>| {
        Data::columns()
            .name("observations")
            .column("x", values)
            .build()
            .unwrap()
    };
    let initial = batch(vec![1., 2., 3.]);
    let mut chart = plot(initial.clone())
        .aes(aes().x("x").y(0.))
        .layer(points())
        .build()
        .unwrap()
        .chart()
        .unwrap();
    let before = chart.source();
    let incompatible = Data::columns().column("wrong", vec![1.]).build().unwrap();
    let bad = chart
        .transaction()
        .unwrap()
        .remove("observations", [initial.batch().keys()[0].get()])
        .append("observations", incompatible)
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(bad).unwrap(),
        CommitOutcome::Rejected(_)
    ));
    assert!(std::ptr::eq(
        before.get().unwrap(),
        chart.source().get().unwrap()
    ));
    let retained = chart
        .transaction()
        .unwrap()
        .retain_count("observations", Some(2))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(retained).unwrap(),
        CommitOutcome::Applied(_)
    ));
    let retained_keys = chart
        .source()
        .get()
        .unwrap()
        .dataset(initial.id())
        .unwrap()
        .rows()
        .map(|r| r.key())
        .collect::<Vec<_>>();
    assert_eq!(retained_keys, initial.batch().keys()[1..]);
    let queued = chart
        .transaction()
        .unwrap()
        .append("observations", batch(vec![4.]))
        .build()
        .unwrap();
    chart.enqueue(queued).unwrap();
    let epoch = chart.source().get().unwrap().epoch();
    chart.reset_epoch().unwrap();
    assert_eq!(chart.source().get().unwrap().epoch().get(), epoch.get() + 1);
    assert_eq!(chart.queue_status().transactions, 1);
    assert!(matches!(
        chart.commit_next().unwrap(),
        Some((_, CommitOutcome::Conflict(_)))
    ));
    assert_eq!(
        chart
            .source()
            .get()
            .unwrap()
            .dataset(initial.id())
            .unwrap()
            .rows()
            .map(|r| r.key())
            .collect::<Vec<_>>(),
        retained_keys
    );
    chart.dispose().unwrap();
    assert!(chart.transaction().is_err());
    assert!(chart.data("observations").is_err());
}
fn definition() -> ChartDefinition {
    decode::<ChartEnvelope>(CHART).unwrap().definition
}
fn store() -> DataStore {
    let data: DataEnvelope = decode(DATA).unwrap();
    DataStore::new(
        data.epoch,
        data.datasets
            .into_iter()
            .map(|d| (d.id, d.batch.into_batch().unwrap()))
            .collect(),
        DataLimits::default(),
    )
    .unwrap()
}
fn transaction() -> Transaction {
    decode::<TransactionEnvelope>(TX)
        .unwrap()
        .into_transaction()
        .unwrap()
}
fn counts(chart: &mut Chart) -> Vec<u64> {
    let prepared = chart.prepare().unwrap();
    let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
        panic!("binned output")
    };
    rows.iter().map(|r| r.count).collect()
}
#[test]
fn typed_and_legacy_transactions_match_independent_bin_counts_and_replay() {
    let mut typed = Chart::from_store(definition(), store(), Arc::default()).unwrap();
    let mut legacy = Session::new(CHART, DATA).unwrap();
    assert_eq!(counts(&mut typed), [2, 1]);
    assert_eq!(counts(legacy.runtime_mut()), [2, 1]);
    assert!(matches!(
        typed.apply_transaction(transaction()).unwrap(),
        CommitOutcome::Applied(_)
    ));
    legacy.apply_transaction(TX).unwrap();
    assert_eq!(counts(&mut typed), [1, 2]);
    assert_eq!(counts(legacy.runtime_mut()), [1, 2]);
    assert!(matches!(
        typed.apply_transaction(transaction()).unwrap(),
        CommitOutcome::AlreadyApplied(_)
    ));
    assert_eq!(typed.state(), legacy.state());
    assert_eq!(
        typed.source().get().unwrap().revision(),
        legacy.source().get().unwrap().revision()
    );
}
#[test]
fn definition_only_edits_retain_current_source_replay_and_pending_queue() {
    let mut chart = Chart::from_store(definition(), store(), Arc::default()).unwrap();
    let mut original = chart.definition().clone();
    chart.apply_transaction(transaction()).unwrap();
    chart.enqueue(transaction()).unwrap();
    let source = chart.source();
    let horizon = chart.dedup_horizon();
    original.figure = Some(chart_core::composition::FigureComposition {
        title: Some(chart_core::typography::RichText::plain("Changed title")),
        ..Default::default()
    });
    let revision = chart.definition().revision;
    assert!(chart.set_definition(original.clone(), revision).unwrap());
    assert!(std::ptr::eq(
        chart.source().get().unwrap(),
        source.get().unwrap()
    ));
    assert_eq!(chart.dedup_horizon(), horizon);
    assert_eq!(chart.queue_status().transactions, 1);
    assert_eq!(counts(&mut chart), [1, 2]);
    assert_eq!(
        chart.set_definition(original, revision).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    let mut invalid = chart.definition().clone();
    invalid.layers[0].data = DataRef::Dataset(DatasetId::new(u64::MAX));
    assert!(
        chart
            .set_definition(invalid, chart.definition().revision)
            .is_err()
    );
    assert!(std::ptr::eq(
        chart.source().get().unwrap(),
        source.get().unwrap()
    ));
    assert_eq!(chart.queue_status().transactions, 1);
    assert!(matches!(
        chart.commit_next().unwrap(),
        Some((_, CommitOutcome::AlreadyApplied(_)))
    ));
}
#[test]
fn external_views_share_committed_source_but_keep_independent_state() {
    let mut owner = store();
    let source = owner.snapshot();
    let mut a = Chart::from_external(definition(), source.clone(), Arc::default()).unwrap();
    let b = Chart::from_external(definition(), source.clone(), Arc::default()).unwrap();
    assert!(!a.owns_ingestion());
    assert_eq!(
        a.apply_transaction(transaction()).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    assert!(matches!(
        owner.apply(transaction()),
        CommitOutcome::Applied(_)
    ));
    a.accept_source(owner.snapshot()).unwrap();
    assert_eq!(counts(&mut a), [1, 2]);
    assert_eq!(b.source().get().unwrap().revision(), Revision::INITIAL);
    assert!(std::ptr::eq(
        b.source().get().unwrap(),
        source.get().unwrap()
    ));
    a.act(ChartAction::SetLegendVisible(false)).unwrap();
    assert_eq!(b.state(), &ChartState::default());
    assert!(a.accept_source(source).is_err());
}
struct CountingStat(Arc<AtomicUsize>);
impl CustomStat for CountingStat {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.counting", Revision::new(1), false)
    }
    fn schema(
        &self,
        _: &DatasetSnapshot,
        _: &ExtensionParameters,
        _: CompileLimits,
    ) -> ChartResult<Vec<StatColumn>> {
        Ok(vec![StatColumn {
            field: StatField::Count,
            kind: GeneratedKind::UInt64,
            nullable: false,
            space: ValueSpace::Data,
        }])
    }
    fn evaluate(&self, _: CustomStatInput<'_>) -> ChartResult<CustomStatOutput> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(Diagnostic::error(
            DiagnosticCode::Validation,
            "Deliberate execution failure",
            "Counting fixture",
        ))
    }
}
#[test]
fn structural_construction_does_not_execute_or_reject_native_only_stats() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ExtensionRegistry::new();
    let stat = CountingStat(calls.clone());
    let op = stat.descriptor().operation;
    registry.register_stat(Arc::new(stat)).unwrap();
    let registry = Arc::new(registry);
    let store = store();
    let source = store.snapshot();
    let id = source
        .get()
        .unwrap()
        .datasets()
        .next()
        .unwrap()
        .version()
        .dataset;
    let definition = ChartDefinition::new(Revision::INITIAL).layer(Layer::statistical(
        LayerId::new(1),
        id,
        Statistic::custom(op, ExtensionParameters::new(serde_json::json!({}))),
        Geom::Point,
        StatAes::new(StatField::Count, StatField::Count),
    ));
    let mut chart = Chart::from_external(definition.clone(), source, registry.clone()).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let json = encode(&ChartEnvelope {
        version: VERSION,
        definition,
    })
    .unwrap();
    assert_eq!(
        Session::with_extensions(&json, DATA, registry)
            .err()
            .unwrap()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(chart.prepare().is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(chart.source().get().unwrap().revision(), Revision::INITIAL);
}

#[test]
fn worker_admission_preserves_newer_committed_source_and_reuses_worker_tables() {
    let mut store = store();
    let registry = Arc::new(ExtensionRegistry::new());
    let mut chart = Chart::from_external(definition(), store.snapshot(), registry.clone()).unwrap();
    let mut worker = Compiler::with_extensions(registry);
    let prepared = worker
        .prepare(
            chart.definition(),
            &chart.source(),
            chart.state(),
            CompileLimits::default(),
        )
        .unwrap();
    assert!(matches!(
        store.apply(transaction()),
        CommitOutcome::Applied(_)
    ));
    let committed = store.snapshot();
    chart.accept_source(committed.clone()).unwrap();
    let admitted = chart.admit_preparation(prepared, &mut worker).unwrap();
    assert!(std::ptr::eq(
        chart.source().get().unwrap(),
        committed.get().unwrap()
    ));
    assert!(admitted.source().get().unwrap().revision() < chart.source().get().unwrap().revision());
    let PreparedRows::Binned(rows) = admitted.layers()[0].table().rows() else {
        panic!("bins");
    };
    assert_eq!(rows.iter().map(|r| r.count).collect::<Vec<_>>(), [2, 1]);
    assert_eq!(counts(&mut chart), [1, 2]);
    let prepared = worker
        .prepare(
            chart.definition(),
            &chart.source(),
            chart.state(),
            CompileLimits::default(),
        )
        .unwrap();
    chart.dispose().unwrap();
    assert_eq!(
        chart
            .admit_preparation(prepared, &mut worker)
            .unwrap_err()
            .code,
        DiagnosticCode::DisposedHandle
    );
}

#[test]
fn primary_external_views_share_source_and_preserve_names_but_own_state() {
    use chart_core::prelude::*;
    let batch = |values: Vec<f64>| {
        Data::columns()
            .column("x", values.clone())
            .column("y", values)
            .build()
            .unwrap()
    };
    let original = plot(batch(vec![1., 2.]))
        .aes(aes().x("x").y("y"))
        .layer(points().name("observations"))
        .build()
        .unwrap();
    let mut owner = original.chart().unwrap();
    owner.act(ChartAction::SetLegendVisible(false)).unwrap();
    let mut left = owner.external_view().unwrap();
    let right = owner.external_view().unwrap();
    assert!(owner.owns_ingestion());
    assert!(!left.owns_ingestion());
    assert!(left.state().legend_visible());
    assert!(std::ptr::eq(
        owner.source().get().unwrap(),
        left.source().get().unwrap()
    ));
    assert_eq!(owner.data("data").unwrap(), left.data("data").unwrap());
    assert_eq!(
        left.transaction().err().unwrap().code,
        DiagnosticCode::UnsupportedCapability
    );
    let tx = owner
        .transaction()
        .unwrap()
        .append("data", batch(vec![3.]))
        .build()
        .unwrap();
    owner.apply_transaction(tx).unwrap();
    assert_eq!(left.source().get().unwrap().revision(), Revision::INITIAL);
    left.accept_from(&owner).unwrap();
    assert!(std::ptr::eq(
        owner.source().get().unwrap(),
        left.source().get().unwrap()
    ));
    assert_eq!(right.source().get().unwrap().revision(), Revision::INITIAL);
    let edited = original
        .edit()
        .title(title("Independent view"))
        .build()
        .unwrap();
    left.apply_plot(&edited, Revision::INITIAL).unwrap();
    assert_ne!(left.definition().revision, owner.definition().revision);
    owner.dispose().unwrap();
    assert_eq!(left.prepare().unwrap().layers()[0].marks().len(), 3);
    assert_eq!(
        left.accept_from(&owner).unwrap_err().code,
        DiagnosticCode::DisposedHandle
    );
}
