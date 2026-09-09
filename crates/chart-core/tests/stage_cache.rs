//! STM-05: numeric, presentation and exact immutable source-index ownership.
use chart_core::{data::*, grammar::*, state::*, transaction::*, *};
use std::sync::Arc;
const A: DatasetId = DatasetId::new(1);
const B: DatasetId = DatasetId::new(2);
const V: FieldId = FieldId::new(1);
fn batch(id: DatasetId, rows: &[(u64, f64)]) -> NormalizedBatch {
    let r = TypedRows::snapshot(
        id,
        Revision::new(1),
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        rows.iter().map(|r| r.1).collect(),
        rows.len(),
    )
    .unwrap();
    TypedDataBuilder::new(r.get().unwrap(), SchemaVersion::new(1))
        .float(V, "value", |v| Some(*v))
        .finish(DataLimits::default())
        .unwrap()
}
fn apply(s: &mut DataStore, d: DatasetId, id: &str, mutation: Mutation) {
    let t = Transaction {
        id: TransactionId::new(id).unwrap(),
        epoch: SourceEpoch::new(1),
        expected: vec![s.snapshot().get().unwrap().dataset(d).unwrap().version()],
        operations: vec![Operation {
            dataset: d,
            mutation,
        }],
    };
    assert!(matches!(s.apply(t), CommitOutcome::Applied(_)));
}
#[test]
fn unrelated_dataset_updates_reuse_exact_named_and_layer_tables_but_capture_new_coherent_store() {
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![
            (A, batch(A, &[(1, 1.), (2, 2.)])),
            (B, batch(B, &[(1, 3.), (2, 4.)])),
        ],
        DataLimits::default(),
    )
    .unwrap();
    let d = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(
            TransformId::new(1),
            A,
            Statistic::identity(),
        ))
        .transform(TransformDefinition::new(
            TransformId::new(2),
            B,
            Statistic::identity(),
        ))
        .layer(Layer::new(
            LayerId::new(1),
            TransformId::new(1),
            Geom::Point,
            SourceAes::new().x(V).y(V),
        ))
        .layer(Layer::new(
            LayerId::new(2),
            TransformId::new(2),
            Geom::Point,
            SourceAes::new().x(V).y(V),
        ));
    let mut c = Compiler::new();
    let p = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    apply(
        &mut s,
        B,
        "correct",
        Mutation::UpsertByKey(batch(B, &[(2, 9.)])),
    );
    let next = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(
        next.metrics(),
        PreparationMetrics {
            evaluated_transforms: 1,
            reused_transforms: 1,
            evaluated_layers: 1,
            reused_layers: 1
        }
    );
    assert!(Arc::ptr_eq(
        p.transform(TransformId::new(1)).unwrap(),
        next.transform(TransformId::new(1)).unwrap()
    ));
    assert!(Arc::ptr_eq(p.layers()[0].table(), next.layers()[0].table()));
    assert!(!Arc::ptr_eq(
        p.layers()[1].table(),
        next.layers()[1].table()
    ));
    assert_eq!(next.source().get().unwrap().revision(), Revision::new(1));
    assert_eq!(p.source().get().unwrap().revision(), Revision::new(0));
    let fresh = Compiler::new()
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    for (a, b) in next.layers().iter().zip(fresh.layers()) {
        assert_eq!(a.table(), b.table());
        assert_eq!(a.marks(), b.marks());
    }
    // A separately constructed store with identical IDs/revisions cannot impersonate cached payloads.
    let other = DataStore::new(
        SourceEpoch::new(1),
        vec![(A, batch(A, &[(1, 100.)])), (B, batch(B, &[(1, 200.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let p = c
        .prepare(
            &d,
            &other.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(p.metrics().evaluated_layers, 2);
    assert_eq!(p.domains().y.unwrap().maximum, 200.);
}
#[test]
fn palette_changes_reuse_statistics_and_navigation_reuses_exact_marks() {
    let s = DataStore::new(
        SourceEpoch::new(1),
        vec![(A, batch(A, &[(1, 0.), (2, 1.), (3, 2.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let mut d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        LayerId::new(1),
        A,
        BinSpec::new(V, vec![0., 1., 2.]),
    ));
    let mut c = Compiler::new();
    let p = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    d.layers[0].style.color = chart_core::scene::Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 255,
    }
    .into();
    let recolored = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert!(Arc::ptr_eq(
        p.layers()[0].table(),
        recolored.layers()[0].table()
    ));
    assert_eq!(recolored.metrics().evaluated_layers, 0);
    assert_ne!(
        p.layers()[0].marks()[0].style,
        recolored.layers()[0].marks()[0].style
    );
    let mut state = ChartState::default();
    state
        .apply(
            &d,
            ChartAction::SetViewport(Viewport {
                x: Some((0.25, 1.75)),
                y: None,
            }),
        )
        .unwrap();
    let zoomed = c
        .prepare(&d, &s.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert!(std::ptr::eq(
        recolored.layers()[0].marks(),
        zoomed.layers()[0].marks()
    ));
    assert_eq!(zoomed.domains(), p.domains());
    assert_eq!(zoomed.state().viewport().x, Some((0.25, 1.75)));
}
#[test]
fn exact_key_indexes_survive_old_snapshots_and_rebuild_after_correction_reorder_and_retention() {
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(A, batch(A, &[(9007199254743001, 1.), (9, 2.), (3, 3.)]))],
        DataLimits {
            chunk_rows: 2,
            ..Default::default()
        },
    )
    .unwrap();
    let old = s.snapshot();
    let data = old.get().unwrap().dataset(A).unwrap();
    assert_eq!(data.lookup_entries(), 0);
    data.prepare_lookup();
    assert_eq!(data.lookup_entries(), 3);
    assert_eq!(
        data.row(RowKey::new(9007199254743001)).unwrap().value(V),
        Some(ValueRef::Float64(1.))
    );
    apply(
        &mut s,
        A,
        "replace",
        Mutation::ReplaceSnapshot(batch(A, &[(3, 33.), (9007199254743001, 11.), (9, 22.)])),
    );
    let next = s.snapshot();
    assert_eq!(
        next.get()
            .unwrap()
            .dataset(A)
            .unwrap()
            .row(RowKey::new(3))
            .unwrap()
            .value(V),
        Some(ValueRef::Float64(33.))
    );
    assert_eq!(
        data.row(RowKey::new(3)).unwrap().value(V),
        Some(ValueRef::Float64(3.))
    );
    apply(
        &mut s,
        A,
        "retain",
        Mutation::SetRetention(RetentionPolicy::Count(1)),
    );
    let current = s.snapshot();
    let current = current.get().unwrap().dataset(A).unwrap();
    assert!(current.row(RowKey::new(9)).is_none());
    assert_eq!(
        current.row(RowKey::new(3)).unwrap().value(V),
        Some(ValueRef::Float64(33.))
    );
    assert_eq!(current.lookup_entries(), 1);
    assert_eq!(data.lookup_entries(), 3);
}
