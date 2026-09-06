//! WP-04 public data portions of FIX-08/09/16; host/chart semantics remain later work.

use chart_core::data::*;
use chart_core::transaction::*;
use chart_core::{
    DatasetId, DiagnosticCode, FieldId, Revision, RowKey, SchemaVersion, SourceEpoch,
};
use std::{collections::BTreeSet, sync::Arc};

const A: DatasetId = DatasetId::new(9_007_199_254_740_993);
const B: DatasetId = DatasetId::new(2);
const X: FieldId = FieldId::new(9_007_199_254_740_995);

fn schema(version: u64, kind: FieldKind) -> Arc<Schema> {
    Arc::new(
        Schema::new(
            SchemaVersion::new(version),
            vec![Field {
                id: X,
                name: "value".into(),
                kind,
                nullable: true,
                unit: None,
                label: None,
            }],
        )
        .unwrap(),
    )
}
fn floats_with(schema: Arc<Schema>, rows: &[(u64, f64)]) -> NormalizedBatch {
    NormalizedBatch::new(
        schema,
        rows.iter().map(|(key, _)| RowKey::new(*key)).collect(),
        vec![Column::new(
            ColumnValues::Float64(rows.iter().map(|(_, v)| *v).collect()),
            vec![true; rows.len()],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap()
}
fn floats(rows: &[(u64, f64)]) -> NormalizedBatch {
    floats_with(schema(0, FieldKind::Float64), rows)
}
fn store(rows: &[(u64, f64)], limits: DataLimits) -> DataStore {
    DataStore::new(SourceEpoch::new(0), vec![(A, floats(rows))], limits).unwrap()
}
fn tx(store: &DataStore, id: &str, operations: Vec<(DatasetId, Mutation)>) -> Transaction {
    let snapshot = store.snapshot();
    let view = snapshot.get().unwrap();
    let touched: BTreeSet<_> = operations.iter().map(|(id, _)| *id).collect();
    Transaction {
        id: TransactionId::new(id).unwrap(),
        epoch: view.epoch(),
        expected: touched
            .iter()
            .map(|id| view.dataset(*id).unwrap().version())
            .collect(),
        operations: operations
            .into_iter()
            .map(|(dataset, mutation)| Operation { dataset, mutation })
            .collect(),
    }
}
fn applied(outcome: CommitOutcome) -> CommitReceipt {
    match outcome {
        CommitOutcome::Applied(receipt) => receipt,
        other => panic!("expected committed outcome, got {other:?}"),
    }
}
fn values(snapshot: &StoreSnapshot, id: DatasetId) -> Vec<(u64, f64, u64)> {
    snapshot
        .dataset(id)
        .unwrap()
        .rows()
        .map(|row| {
            let Some(ValueRef::Float64(value)) = row.value(X) else {
                panic!("expected float")
            };
            (row.key().get(), value, row.ordinal())
        })
        .collect()
}

#[test]
fn fix08_ordered_atomic_counts_and_replay() {
    let mut store = DataStore::new(
        SourceEpoch::new(0),
        vec![(A, floats(&[(1, 10.), (2, 20.)])), (B, floats(&[(9, 90.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let old = store.snapshot();
    let transaction = tx(
        &store,
        "multi",
        vec![
            (A, Mutation::AppendBatch(floats(&[(3, 30.)]))),
            (A, Mutation::UpsertByKey(floats(&[(1, 11.), (4, 40.)]))),
            (
                A,
                Mutation::RemoveKeys(vec![RowKey::new(2), RowKey::new(99), RowKey::new(99)]),
            ),
            (B, Mutation::AppendBatch(floats(&[(10, 100.)]))),
        ],
    );
    let receipt = applied(store.apply(transaction.clone()));
    assert!(receipt.changed);
    assert_eq!(receipt.store_revision, Revision::new(1));
    assert_eq!(
        receipt.operations,
        vec![
            OperationCounts {
                inserted: 1,
                ..Default::default()
            },
            OperationCounts {
                inserted: 1,
                updated: 1,
                ..Default::default()
            },
            OperationCounts {
                removed: 1,
                absent: 1,
                ..Default::default()
            },
            OperationCounts {
                inserted: 1,
                ..Default::default()
            }
        ]
    );
    assert_eq!(
        values(store.snapshot().get().unwrap(), A),
        vec![(1, 11., 0), (3, 30., 2), (4, 40., 3)]
    );
    assert_eq!(
        values(store.snapshot().get().unwrap(), B),
        vec![(9, 90., 0), (10, 100., 1)]
    );
    assert_eq!(
        values(old.get().unwrap(), A),
        vec![(1, 10., 0), (2, 20., 1)]
    );
    assert!(
        receipt
            .datasets
            .iter()
            .all(|v| v.revision == Revision::new(1))
    );
    assert_eq!(
        store.apply(transaction.clone()),
        CommitOutcome::AlreadyApplied(receipt)
    );
    let mut different = transaction;
    different.operations.pop();
    assert!(
        matches!(store.apply(different),CommitOutcome::Rejected(e) if e.code==DiagnosticCode::TransactionReuse)
    );
}

#[test]
fn invalid_last_dataset_rolls_back_everything_and_does_not_remember_id() {
    let mut store = DataStore::new(
        SourceEpoch::new(0),
        vec![(A, floats(&[(1, 10.)])), (B, floats(&[(9, 90.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let transaction = tx(
        &store,
        "retryable",
        vec![
            (A, Mutation::AppendBatch(floats(&[(2, 20.)]))),
            (B, Mutation::AppendBatch(floats(&[(9, 91.)]))),
        ],
    );
    assert!(
        matches!(store.apply(transaction),CommitOutcome::Rejected(e) if e.context.dataset==Some(B))
    );
    assert_eq!(
        store.snapshot().get().unwrap().revision(),
        Revision::INITIAL
    );
    assert_eq!(
        values(store.snapshot().get().unwrap(), A),
        vec![(1, 10., 0)]
    );
    assert_eq!(
        values(store.snapshot().get().unwrap(), B),
        vec![(9, 90., 0)]
    );
    assert_eq!(store.dedup_horizon().entries, 0);
    let corrected = tx(
        &store,
        "retryable",
        vec![(A, Mutation::AppendBatch(floats(&[(2, 20.)])))],
    );
    assert!(applied(store.apply(corrected)).changed);
}

#[test]
fn net_noops_keep_revisions_and_nan_payload_replay_is_bit_exact() {
    let mut store = store(&[(1, 10.)], DataLimits::default());
    let noop = tx(
        &store,
        "undo",
        vec![
            (A, Mutation::UpsertByKey(floats(&[(1, 12.)]))),
            (A, Mutation::UpsertByKey(floats(&[(1, 10.)]))),
            (A, Mutation::RemoveKeys(vec![RowKey::new(88)])),
        ],
    );
    let receipt = applied(store.apply(noop));
    assert!(!receipt.changed);
    assert_eq!(receipt.store_revision, Revision::INITIAL);
    assert_eq!(receipt.operations[0].updated, 1);
    assert_eq!(receipt.operations[1].updated, 1);
    let transaction = tx(
        &store,
        "nan",
        vec![(
            A,
            Mutation::AppendBatch(floats(&[(2, f64::from_bits(0x7ff8_0000_0000_0001))])),
        )],
    );
    let receipt = applied(store.apply(transaction.clone()));
    assert_eq!(
        store.apply(transaction),
        CommitOutcome::AlreadyApplied(receipt)
    );
}

#[test]
fn fix16_timestamp_origin_validity_and_bounded_diagnostics() {
    let origin = 9_000_000_000_000_000_000_i64;
    let time = schema(
        17,
        FieldKind::Timestamp(TimestampType {
            unit: TimeUnit::Nanoseconds,
            timezone: "America/Toronto".into(),
        }),
    );
    let batch = NormalizedBatch::new(
        time,
        vec![RowKey::new(u64::MAX), RowKey::new(2), RowKey::new(3)],
        vec![Column::new(
            ColumnValues::Timestamp(vec![origin + 17, origin + 257, i64::MIN]),
            vec![true, true, false],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap();
    assert_eq!(batch.keys()[0].get(), u64::MAX);
    assert_eq!(
        batch.column(X).unwrap().value(0),
        Some(ValueRef::Timestamp(origin + 17))
    );
    let projected = batch
        .project_numeric(X, InvalidPolicy::Exclude, Some(origin), 1)
        .unwrap();
    assert_eq!(projected.values, vec![Some(17.), Some(257.), None]);
    assert_eq!(projected.nulls, 1);
    assert_eq!(
        projected.diagnostic.unwrap().context.row_samples,
        vec![RowKey::new(3)]
    );
    assert!(
        batch
            .project_numeric(X, InvalidPolicy::Strict, Some(origin), 1)
            .is_err()
    );
    assert_eq!(
        batch
            .project_numeric(X, InvalidPolicy::Exclude, Some(i64::MIN), 1)
            .unwrap()
            .precision_loss,
        2
    );
    let bad = floats(&[(1, f64::NAN), (2, f64::INFINITY), (3, 1.)]);
    let result = bad
        .project_numeric(X, InvalidPolicy::Exclude, None, 1)
        .unwrap();
    assert_eq!(result.values, vec![None, None, Some(1.)]);
    assert_eq!(result.non_finite, 2);
    assert_eq!(
        result.diagnostic.unwrap().context.row_samples,
        vec![RowKey::new(1)]
    );
}

#[test]
fn expected_bases_epoch_fences_and_empty_commits() {
    let mut store = store(&[(1, 1.)], DataLimits::default());
    let base = tx(&store, "bases", vec![(A, Mutation::RemoveKeys(vec![]))]);
    for expected in [
        vec![],
        vec![base.expected[0], base.expected[0]],
        vec![DatasetVersion {
            dataset: B,
            ..base.expected[0]
        }],
    ] {
        let mut invalid = base.clone();
        invalid.expected = expected;
        assert!(matches!(store.apply(invalid), CommitOutcome::Rejected(_)));
    }
    for schema_conflict in [false, true] {
        let mut stale = base.clone();
        if schema_conflict {
            stale.expected[0].schema_version = SchemaVersion::new(1);
        } else {
            stale.expected[0].revision = Revision::new(1);
        }
        let CommitOutcome::Conflict(conflict) = store.apply(stale) else {
            panic!("expected fence conflict")
        };
        assert_eq!(conflict.observed, base.expected);
        assert_eq!(conflict.diagnostic.context.dataset, Some(A));
    }
    let empty = tx(&store, "empty", vec![]);
    assert!(!applied(store.apply(empty)).changed);
    let old = store.snapshot();
    store.reset_epoch(SourceEpoch::new(7)).unwrap();
    assert_eq!(store.dedup_horizon().entries, 0);
    assert_eq!(old.get().unwrap().epoch(), SourceEpoch::new(0));
    assert!(matches!(store.apply(base), CommitOutcome::Conflict(_)));
    assert!(store.reset_epoch(SourceEpoch::new(7)).is_err());
    assert!(store.reset_epoch(SourceEpoch::new(6)).is_err());
}

#[test]
fn replay_eviction_is_fifo_and_old_effective_commit_conflicts() {
    let mut store = store(
        &[],
        DataLimits {
            dedup_entries: 2,
            ..Default::default()
        },
    );
    let first = tx(
        &store,
        "first",
        vec![(A, Mutation::AppendBatch(floats(&[(1, 1.)])))],
    );
    let first_receipt = applied(store.apply(first.clone()));
    let second = tx(
        &store,
        "second",
        vec![(A, Mutation::AppendBatch(floats(&[(2, 2.)])))],
    );
    applied(store.apply(second));
    assert_eq!(
        store.apply(first.clone()),
        CommitOutcome::AlreadyApplied(first_receipt)
    );
    let third = tx(
        &store,
        "third",
        vec![(A, Mutation::AppendBatch(floats(&[(3, 3.)])))],
    );
    applied(store.apply(third));
    assert_eq!(
        store.dedup_horizon().oldest,
        Some(TransactionId::new("second").unwrap())
    );
    assert!(matches!(store.apply(first), CommitOutcome::Conflict(_)));
    assert_eq!(store.snapshot().get().unwrap().revision(), Revision::new(3));
}

#[test]
fn resource_budgets_reject_before_publication_and_byte_horizon_evicts() {
    for limits in [
        DataLimits {
            max_dataset_rows: 1,
            ..Default::default()
        },
        DataLimits {
            max_operations: 0,
            ..Default::default()
        },
        DataLimits {
            max_transaction_bytes: 1,
            ..Default::default()
        },
        DataLimits {
            dedup_bytes: 1,
            ..Default::default()
        },
        DataLimits {
            max_dataset_bytes: 250,
            ..Default::default()
        },
    ] {
        let mut store = store(&[(1, 1.)], limits);
        let t = tx(
            &store,
            "too-big",
            vec![(A, Mutation::AppendBatch(floats(&[(2, 2.)])))],
        );
        assert!(
            matches!(store.apply(t), CommitOutcome::Rejected(e) if e.code == DiagnosticCode::ResourceLimit)
        );
        assert_eq!(values(store.snapshot().get().unwrap(), A), vec![(1, 1., 0)]);
        assert_eq!(store.dedup_horizon().entries, 0);
    }
    let mut probe = store(&[], DataLimits::default());
    let t = tx(
        &probe,
        "one",
        vec![(A, Mutation::AppendBatch(floats(&[(1, 1.)])))],
    );
    applied(probe.apply(t));
    let budget = probe.dedup_horizon().bytes;
    let mut bounded = store(
        &[],
        DataLimits {
            dedup_bytes: budget,
            ..Default::default()
        },
    );
    for (name, key) in [("one", 1), ("two", 2)] {
        let t = tx(
            &bounded,
            name,
            vec![(A, Mutation::AppendBatch(floats(&[(key, key as f64)])))],
        );
        applied(bounded.apply(t));
        assert_eq!(bounded.dedup_horizon().entries, 1);
        assert!(bounded.dedup_horizon().bytes <= budget);
    }
    assert_eq!(
        bounded.dedup_horizon().oldest,
        Some(TransactionId::new("two").unwrap())
    );
}

#[test]
fn replacement_reorders_without_reassigning_keys_and_retention_uses_ordinals() {
    let mut store = store(&[(1, 10.), (2, 20.), (3, 30.)], DataLimits::default());
    let old = store.snapshot();
    let t = tx(
        &store,
        "replace",
        vec![
            (
                A,
                Mutation::ReplaceSnapshot(floats(&[(3, 31.), (1, 10.), (4, 40.), (2, 20.)])),
            ),
            (A, Mutation::SetRetention(RetentionPolicy::Count(2))),
        ],
    );
    let receipt = applied(store.apply(t));
    assert_eq!(
        values(store.snapshot().get().unwrap(), A),
        vec![(3, 31., 2), (4, 40., 3)]
    );
    assert_eq!(
        receipt.operations[0],
        OperationCounts {
            inserted: 1,
            updated: 1,
            ..Default::default()
        }
    );
    assert_eq!(receipt.operations[1].evicted, 2);
    assert_eq!(
        receipt
            .removed_sources
            .iter()
            .map(|s| s.key.get())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        values(old.get().unwrap(), A),
        vec![(1, 10., 0), (2, 20., 1), (3, 30., 2)]
    );
    let t = tx(
        &store,
        "zero",
        vec![(A, Mutation::SetRetention(RetentionPolicy::Count(0)))],
    );
    assert_eq!(applied(store.apply(t)).operations[0].evicted, 2);
    let t = tx(
        &store,
        "retained-zero",
        vec![(A, Mutation::AppendBatch(floats(&[(9, 9.)])))],
    );
    let r = applied(store.apply(t));
    assert!(r.changed); // Consumed stable insertion ordinal is effective metadata.
    assert_eq!(r.operations[0].evicted, 1);
    assert!(
        store
            .snapshot()
            .get()
            .unwrap()
            .dataset(A)
            .unwrap()
            .is_empty()
    );
}

fn categories(version: u64, keys: &[u64], codes: &[u32], dictionary: &[&str]) -> NormalizedBatch {
    NormalizedBatch::new(
        schema(version, FieldKind::Categorical),
        keys.iter().map(|k| RowKey::new(*k)).collect(),
        vec![Column::new(
            ColumnValues::Categorical {
                codes: codes.to_vec(),
                dictionary: dictionary.iter().map(|s| (*s).into()).collect(),
            },
            vec![true; keys.len()],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap()
}

#[test]
fn category_identity_survives_code_reassignment_removal_and_schema_version() {
    let mut store = DataStore::new(
        SourceEpoch::new(0),
        vec![(A, categories(0, &[1, 2], &[1, 0], &["z", "a"]))],
        DataLimits::default(),
    )
    .unwrap();
    let t = tx(
        &store,
        "recode",
        vec![(
            A,
            Mutation::UpsertByKey(categories(0, &[1, 2], &[0, 1], &["a", "z"])),
        )],
    );
    assert!(!applied(store.apply(t)).changed);
    let t = tx(
        &store,
        "category",
        vec![
            (A, Mutation::RemoveKeys(vec![RowKey::new(1)])),
            (A, Mutation::AppendBatch(categories(0, &[3], &[0], &["b"]))),
            (
                A,
                Mutation::ReplaceSnapshot(categories(1, &[3, 2], &[0, 1], &["b", "z"])),
            ),
        ],
    );
    applied(store.apply(t));
    let snapshot = store.snapshot();
    let data = snapshot.get().unwrap().dataset(A).unwrap();
    assert_eq!(data.categories(X).unwrap(), &["a", "z", "b"]);
    assert_eq!(
        data.row(RowKey::new(2)).unwrap().value(X),
        Some(ValueRef::Category("z"))
    );
    assert_eq!(
        data.category_order(X, Some(&["b".into(), "absent".into()]))
            .unwrap(),
        vec!["b", "absent"]
    );
    assert!(
        data.category_order(X, Some(&["b".into(), "b".into()]))
            .is_err()
    );
    let t = tx(&store, "reset", vec![(A, Mutation::ResetCategoryOrder(X))]);
    applied(store.apply(t));
    assert_eq!(
        store
            .snapshot()
            .get()
            .unwrap()
            .dataset(A)
            .unwrap()
            .categories(X)
            .unwrap(),
        &["b", "z"]
    );
}

#[test]
fn schema_migration_is_explicit_and_ordered() {
    let mut store = store(&[(1, 1.)], DataLimits::default());
    let strings = |version| {
        NormalizedBatch::new(
            schema(version, FieldKind::Utf8),
            vec![RowKey::new(1)],
            vec![Column::new(
                ColumnValues::Utf8(vec!["exact".into()]),
                vec![true],
                Some(vec![Some("1.000000000000000001".into())]),
            )],
            DataLimits::default(),
        )
        .unwrap()
    };
    let invalid = tx(
        &store,
        "same-version",
        vec![(A, Mutation::ReplaceSnapshot(strings(0)))],
    );
    assert!(
        matches!(store.apply(invalid), CommitOutcome::Rejected(e) if e.code == DiagnosticCode::SchemaConflict)
    );
    let old = store.snapshot();
    let t = tx(
        &store,
        "migrate",
        vec![
            (A, Mutation::ReplaceSnapshot(strings(1))),
            (A, Mutation::UpsertByKey(strings(1))),
        ],
    );
    let r = applied(store.apply(t));
    assert_eq!(r.datasets[0].schema_version, SchemaVersion::new(1));
    assert_eq!(r.operations[1].updated, 0);
    let snapshot = store.snapshot();
    let row = snapshot
        .get()
        .unwrap()
        .dataset(A)
        .unwrap()
        .row(RowKey::new(1))
        .unwrap();
    assert_eq!(row.value(X), Some(ValueRef::Utf8("exact")));
    assert_eq!(row.formatted(X), Some("1.000000000000000001"));
    assert_eq!(row.ordinal(), 0);
    assert_eq!(values(old.get().unwrap(), A), vec![(1, 1., 0)]);
}

#[test]
fn fix09_provenance_resolves_only_declared_scope_and_pinned_history() {
    use chart_core::provenance::{ResolvedTarget, SourceRef, Target};
    use chart_core::{AggregateId, DerivedId};
    let mut store = store(&[(1, 1.), (2, 2.)], DataLimits::default());
    let old = store.snapshot();
    let input = old.get().unwrap().dataset(A).unwrap().version();
    let source = Target::Source(SourceRef {
        dataset: A,
        key: RowKey::new(1),
    });
    let aggregate = Target::Aggregate {
        id: AggregateId::new(99),
        group: "bucket".into(),
        input,
        members: Arc::from([RowKey::new(2), RowKey::new(1)]),
    };
    let derived = Target::Derived {
        id: DerivedId::new(42),
        model: "fit".into(),
        model_version: Revision::new(7),
        inputs: vec![input],
    };
    let ResolvedTarget::Aggregate { members, .. } = aggregate.resolve(old.get().unwrap()).unwrap()
    else {
        panic!("aggregate")
    };
    assert_eq!(
        members.iter().map(|r| r.key().get()).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(
        matches!(derived.resolve(old.get().unwrap()).unwrap(), ResolvedTarget::Derived { model_version, .. } if model_version == Revision::new(7))
    );
    let t = tx(
        &store,
        "evict",
        vec![(A, Mutation::SetRetention(RetentionPolicy::Count(1)))],
    );
    applied(store.apply(t));
    let current = store.snapshot();
    assert!(matches!(
        source.resolve(current.get().unwrap()).unwrap(),
        ResolvedTarget::MissingSource(_)
    ));
    assert!(matches!(
        source.resolve(old.get().unwrap()).unwrap(),
        ResolvedTarget::Source(_)
    ));
    assert_eq!(
        aggregate.resolve(current.get().unwrap()).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(
        derived.resolve(current.get().unwrap()).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    for members in [vec![RowKey::new(1), RowKey::new(1)], vec![RowKey::new(999)]] {
        let invalid = Target::Aggregate {
            id: AggregateId::new(1),
            group: "bad".into(),
            input,
            members: members.into(),
        };
        assert!(invalid.resolve(old.get().unwrap()).is_err());
    }
    let invalid = Target::Derived {
        id: DerivedId::new(1),
        model: "fit".into(),
        model_version: Revision::INITIAL,
        inputs: vec![input, input],
    };
    assert!(invalid.resolve(old.get().unwrap()).is_err());
}

#[test]
fn fix16_typed_and_normalized_handles_pin_then_release_ownership() {
    use std::{cell::Cell, rc::Rc};
    #[derive(Debug)]
    struct LocalRow(Rc<Cell<usize>>);
    impl Drop for LocalRow {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut handle = TypedRows::snapshot(
        A,
        Revision::new(9),
        vec![RowKey::new(u64::MAX)],
        vec![LocalRow(drops.clone())],
        1,
    )
    .unwrap();
    let mut other = handle.clone();
    handle.dispose();
    handle.dispose();
    assert_eq!(
        handle.get().unwrap_err().code,
        DiagnosticCode::DisposedHandle
    );
    assert_eq!(other.get().unwrap().keys(), &[RowKey::new(u64::MAX)]);
    assert_eq!(drops.get(), 0);
    other.dispose();
    assert_eq!(drops.get(), 1);
    fn send_sync<T: Send + Sync>() {}
    send_sync::<SnapshotHandle<StoreSnapshot>>();
    let initial = schema(0, FieldKind::Float64);
    let weak = Arc::downgrade(&initial);
    let mut store = DataStore::new(
        SourceEpoch::new(0),
        vec![(A, floats_with(initial, &[(1, 1.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let mut old = store.snapshot();
    let t = tx(
        &store,
        "new-schema",
        vec![(
            A,
            Mutation::ReplaceSnapshot(floats_with(schema(1, FieldKind::Float64), &[(2, 2.)])),
        )],
    );
    applied(store.apply(t));
    assert!(weak.upgrade().is_some());
    assert_eq!(values(old.get().unwrap(), A), vec![(1, 1., 0)]);
    old.dispose();
    assert!(weak.upgrade().is_none());
}

#[test]
fn append_shares_every_retained_chunk_and_upsert_copies_only_affected_chunk() {
    let initial: Vec<_> = (0..10_000).map(|i| (i, i as f64)).collect();
    let mut store = store(
        &initial,
        DataLimits {
            chunk_rows: 1000,
            ..Default::default()
        },
    );
    let old = store.snapshot();
    for n in 0..10 {
        let t = tx(
            &store,
            &format!("append-{n}"),
            vec![(A, Mutation::AppendBatch(floats(&[(10_000 + n, n as f64)])))],
        );
        applied(store.apply(t));
        let next = store.snapshot();
        let before = old.get().unwrap().dataset(A).unwrap().chunks();
        let after = next.get().unwrap().dataset(A).unwrap().chunks();
        for (a, b) in before.iter().zip(after) {
            assert!(std::ptr::eq(a.batch(), b.batch()));
        }
    }
    let before = store.snapshot();
    let t = tx(
        &store,
        "edit",
        vec![(A, Mutation::UpsertByKey(floats(&[(1500, -1.)])))],
    );
    applied(store.apply(t));
    let after = store.snapshot();
    for (i, (a, b)) in before
        .get()
        .unwrap()
        .dataset(A)
        .unwrap()
        .chunks()
        .iter()
        .zip(after.get().unwrap().dataset(A).unwrap().chunks())
        .enumerate()
    {
        assert_eq!(std::ptr::eq(a.batch(), b.batch()), i != 1);
    }
    assert_eq!(
        old.get()
            .unwrap()
            .dataset(A)
            .unwrap()
            .row(RowKey::new(1500))
            .unwrap()
            .value(X),
        Some(ValueRef::Float64(1500.))
    );
}

#[test]
fn all_portable_kinds_preserve_exact_values_and_original_display() {
    let kinds = vec![
        FieldKind::Float64,
        FieldKind::Int64,
        FieldKind::UInt64,
        FieldKind::Boolean,
        FieldKind::Utf8,
        FieldKind::Categorical,
        FieldKind::Timestamp(TimestampType {
            unit: TimeUnit::Microseconds,
            timezone: "UTC".into(),
        }),
    ];
    let fields = kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| Field {
            id: FieldId::new(i as u64),
            name: format!("f{i}"),
            kind,
            nullable: true,
            unit: Some("unit".into()),
            label: Some("label".into()),
        })
        .collect();
    let schema = Arc::new(Schema::new(SchemaVersion::new(u64::MAX), fields).unwrap());
    let columns = vec![
        ColumnValues::Float64(vec![1.25]),
        ColumnValues::Int64(vec![i64::MIN]),
        ColumnValues::UInt64(vec![u64::MAX]),
        ColumnValues::Boolean(vec![true]),
        ColumnValues::Utf8(vec!["12.0000000000000000001".into()]),
        ColumnValues::Categorical {
            codes: vec![0],
            dictionary: vec!["category".into()],
        },
        ColumnValues::Timestamp(vec![i64::MAX]),
    ]
    .into_iter()
    .map(|v| Column::new(v, vec![true], Some(vec![Some("original".into())])))
    .collect();
    let batch = NormalizedBatch::new(
        schema,
        vec![RowKey::new(u64::MAX)],
        columns,
        DataLimits::default(),
    )
    .unwrap();
    let expected = [
        ValueRef::Float64(1.25),
        ValueRef::Int64(i64::MIN),
        ValueRef::UInt64(u64::MAX),
        ValueRef::Boolean(true),
        ValueRef::Utf8("12.0000000000000000001"),
        ValueRef::Category("category"),
        ValueRef::Timestamp(i64::MAX),
    ];
    for (i, value) in expected.into_iter().enumerate() {
        let col = batch.column(FieldId::new(i as u64)).unwrap();
        assert_eq!(col.value(0), Some(value));
        assert_eq!(col.formatted(0), Some("original"));
    }
    for field in [1, 2] {
        let result = batch
            .project_numeric(FieldId::new(field), InvalidPolicy::Exclude, None, 1)
            .unwrap();
        assert_eq!(result.values, vec![None]);
        assert_eq!(result.precision_loss, 1);
    }
    assert_eq!(
        batch
            .project_numeric(
                FieldId::new(6),
                InvalidPolicy::Strict,
                Some(i64::MAX - 1),
                1
            )
            .unwrap()
            .values,
        vec![Some(1.)]
    );
    assert!(
        batch
            .project_numeric(FieldId::new(6), InvalidPolicy::Strict, Some(i64::MIN), 1)
            .is_err()
    );
}

#[test]
fn malformed_schemas_batches_and_null_dictionary_codes_are_checked() {
    let field = schema(0, FieldKind::Float64).fields()[0].clone();
    for duplicate in [
        field.clone(),
        Field {
            id: FieldId::new(7),
            ..field.clone()
        },
        Field {
            name: "different".into(),
            ..field.clone()
        },
    ] {
        assert!(Schema::new(SchemaVersion::new(0), vec![field.clone(), duplicate]).is_err());
    }
    assert!(
        Schema::new(
            SchemaVersion::new(0),
            vec![Field {
                kind: FieldKind::Timestamp(TimestampType {
                    unit: TimeUnit::Seconds,
                    timezone: " ".into()
                }),
                ..field.clone()
            }]
        )
        .is_err()
    );
    for column in [
        Column::new(ColumnValues::Float64(vec![]), vec![true], None),
        Column::new(ColumnValues::Float64(vec![1.]), vec![], None),
        Column::new(ColumnValues::Float64(vec![1.]), vec![true], Some(vec![])),
        Column::new(ColumnValues::Int64(vec![1]), vec![true], None),
    ] {
        assert!(
            NormalizedBatch::new(
                schema(0, FieldKind::Float64),
                vec![RowKey::new(1)],
                vec![column],
                DataLimits::default()
            )
            .is_err()
        );
    }
    let required = Arc::new(
        Schema::new(
            SchemaVersion::new(0),
            vec![Field {
                nullable: false,
                ..field
            }],
        )
        .unwrap(),
    );
    assert!(
        NormalizedBatch::new(
            required,
            vec![RowKey::new(1)],
            vec![Column::new(
                ColumnValues::Float64(vec![0.]),
                vec![false],
                None
            )],
            DataLimits::default()
        )
        .is_err()
    );
    assert!(
        NormalizedBatch::new(
            schema(0, FieldKind::Float64),
            vec![RowKey::new(1), RowKey::new(1)],
            vec![Column::new(
                ColumnValues::Float64(vec![1., 2.]),
                vec![true, true],
                None
            )],
            DataLimits::default()
        )
        .is_err()
    );
    for (valid, dictionary, expected_ok) in [
        (false, vec![], true),
        (true, vec![], false),
        (false, vec!["a".into(), "a".into()], false),
    ] {
        let result = NormalizedBatch::new(
            schema(0, FieldKind::Categorical),
            vec![RowKey::new(1)],
            vec![Column::new(
                ColumnValues::Categorical {
                    codes: vec![u32::MAX],
                    dictionary,
                },
                vec![valid],
                None,
            )],
            DataLimits::default(),
        );
        assert_eq!(result.is_ok(), expected_ok);
        if let Ok(batch) = result {
            assert_eq!(batch.column(X).unwrap().value(0), None);
        }
    }
    for limits in [
        DataLimits {
            max_batch_rows: 0,
            ..Default::default()
        },
        DataLimits {
            max_fields: 0,
            ..Default::default()
        },
        DataLimits {
            max_batch_bytes: 0,
            ..Default::default()
        },
    ] {
        assert!(
            NormalizedBatch::new(
                schema(0, FieldKind::Float64),
                vec![RowKey::new(1)],
                vec![Column::new(
                    ColumnValues::Float64(vec![1.]),
                    vec![true],
                    None
                )],
                limits
            )
            .is_err()
        );
    }
}

#[test]
fn deterministic_operation_sequence_matches_independent_batch_model() {
    // A simple whole-row Vec oracle. It shares no transaction/storage implementation.
    let mut store = store(
        &[],
        DataLimits {
            chunk_rows: 3,
            ..Default::default()
        },
    );
    let mut rows: Vec<(u64, f64, u64)> = vec![];
    let mut next_ordinal = 0;
    let mut retained: Option<usize> = None;
    let mut revision = 0;
    let mut random = 0x1234_5678_u64;
    for step in 0..250 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key = (random >> 32) % 23;
        let value = ((random >> 16) % 100) as f64;
        let before = (rows.clone(), next_ordinal, retained);
        let mutation = match step % 6 {
            0 => {
                let key = 1000 + step;
                rows.push((key, value, next_ordinal));
                next_ordinal += 1;
                Mutation::AppendBatch(floats(&[(key, value)]))
            }
            1 => {
                if let Some(row) = rows.iter_mut().find(|r| r.0 == key) {
                    row.1 = value;
                } else {
                    rows.push((key, value, next_ordinal));
                    next_ordinal += 1;
                }
                Mutation::UpsertByKey(floats(&[(key, value)]))
            }
            2 => {
                rows.retain(|r| r.0 != key);
                Mutation::RemoveKeys(vec![RowKey::new(key)])
            }
            3 => {
                let old_rows = rows.clone();
                let inputs = [(key, value), ((key + 1) % 23, value + 1.)];
                rows = inputs
                    .iter()
                    .map(|&(k, v)| {
                        let ordinal = old_rows
                            .iter()
                            .find(|r| r.0 == k)
                            .map(|r| r.2)
                            .unwrap_or_else(|| {
                                let n = next_ordinal;
                                next_ordinal += 1;
                                n
                            });
                        (k, v, ordinal)
                    })
                    .collect();
                Mutation::ReplaceSnapshot(floats(&inputs))
            }
            4 => {
                retained = Some((key % 4) as usize);
                Mutation::SetRetention(RetentionPolicy::Count(retained.unwrap()))
            }
            _ => {
                retained = None;
                Mutation::SetRetention(RetentionPolicy::Unbounded)
            }
        };
        if let Some(max) = retained {
            while rows.len() > max {
                let index = rows.iter().enumerate().min_by_key(|(_, r)| r.2).unwrap().0;
                rows.remove(index);
            }
        }
        let changed = before != (rows.clone(), next_ordinal, retained);
        revision += u64::from(changed);
        let t = tx(&store, &format!("oracle-{step}"), vec![(A, mutation)]);
        let receipt = applied(store.apply(t));
        assert_eq!(receipt.changed, changed, "step {step}");
        assert_eq!(
            receipt.store_revision,
            Revision::new(revision),
            "step {step}"
        );
        assert_eq!(
            values(store.snapshot().get().unwrap(), A),
            rows,
            "step {step}"
        );
    }
}
