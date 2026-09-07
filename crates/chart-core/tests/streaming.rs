//! FIX-08/09: independently expected event-time boundaries, queue outcomes and replay.
use chart_core::{data::*, ingestion::*, transaction::*, *};
use std::sync::Arc;
const D: DatasetId = DatasetId::new(1);
const T: FieldId = FieldId::new(1);
fn batch(rows: &[(u64, i64)]) -> NormalizedBatch {
    let schema = Arc::new(
        Schema::new(
            SchemaVersion::new(1),
            vec![Field {
                id: T,
                name: "time".into(),
                kind: FieldKind::Timestamp(TimestampType {
                    unit: TimeUnit::Nanoseconds,
                    timezone: "UTC".into(),
                }),
                nullable: false,
                unit: None,
                label: None,
            }],
        )
        .unwrap(),
    );
    NormalizedBatch::new(
        schema,
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        vec![Column::new(
            ColumnValues::Timestamp(rows.iter().map(|r| r.1).collect()),
            vec![true; rows.len()],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap()
}
fn store(rows: &[(u64, i64)]) -> DataStore {
    DataStore::new(
        SourceEpoch::new(1),
        vec![(D, batch(rows))],
        DataLimits::default(),
    )
    .unwrap()
}
fn tx(s: &DataStore, id: &str, operations: Vec<Mutation>) -> Transaction {
    Transaction {
        id: TransactionId::new(id).unwrap(),
        epoch: SourceEpoch::new(1),
        expected: vec![s.snapshot().get().unwrap().dataset(D).unwrap().version()],
        operations: operations
            .into_iter()
            .map(|mutation| Operation {
                dataset: D,
                mutation,
            })
            .collect(),
    }
}
fn applied(o: CommitOutcome) -> CommitReceipt {
    match o {
        CommitOutcome::Applied(r) => r,
        other => panic!("{other:?}"),
    }
}
fn keys(s: &DataStore) -> Vec<u64> {
    s.snapshot()
        .get()
        .unwrap()
        .dataset(D)
        .unwrap()
        .rows()
        .map(|r| r.key().get())
        .collect()
}
fn window(watermark: i64, late: LateDataPolicy) -> RetentionPolicy {
    RetentionPolicy::EventTime(EventTimeWindow {
        field: T,
        width: 10,
        allowed_lateness: 2,
        watermark,
        late,
    })
}

#[test]
fn supplied_watermark_exact_boundary_future_and_late_atomicity() {
    let origin = 9_007_199_254_741_001i64;
    let mut s = store(&[
        (1, origin - 13),
        (2, origin - 12),
        (3, origin),
        (4, origin + 1000),
    ]);
    let before = s.snapshot();
    let t = tx(
        &s,
        "policy",
        vec![Mutation::SetRetention(window(
            origin,
            LateDataPolicy::Reject,
        ))],
    );
    let r = applied(s.apply(t.clone()));
    assert_eq!(r.operations[0].evicted, 1);
    assert_eq!(keys(&s), [2, 3, 4]); // inclusive lower bound; future row did not move watermark
    assert_eq!(before.get().unwrap().dataset(D).unwrap().len(), 4);
    assert!(matches!(s.apply(t), CommitOutcome::AlreadyApplied(_)));
    let snapshot = s.snapshot();
    let t = tx(
        &s,
        "late",
        vec![
            Mutation::AppendBatch(batch(&[(5, origin + 1)])),
            Mutation::UpsertByKey(batch(&[(2, origin - 13)])),
        ],
    );
    assert!(matches!(s.apply(t), CommitOutcome::Rejected(_)));
    assert!(std::ptr::eq(
        snapshot.get().unwrap(),
        s.snapshot().get().unwrap()
    ));
    let t = tx(&s, "advance", vec![Mutation::AdvanceWatermark(origin + 1)]);
    let r = applied(s.apply(t));
    assert_eq!(r.operations[0].evicted, 1);
    assert_eq!(keys(&s), [3, 4]);
    let t = tx(&s, "regress", vec![Mutation::AdvanceWatermark(origin)]);
    assert!(matches!(s.apply(t), CommitOutcome::Rejected(_)));
    let t = tx(&s, "same", vec![Mutation::AdvanceWatermark(origin + 1)]);
    assert!(!applied(s.apply(t)).changed);
}
#[test]
fn explicit_late_drop_accounts_whole_rows_and_count_order_is_ordinal() {
    let mut s = store(&[(1, 90), (2, 100)]);
    let t = tx(
        &s,
        "policy",
        vec![Mutation::SetRetention(window(100, LateDataPolicy::Drop))],
    );
    applied(s.apply(t));
    let t = tx(
        &s,
        "late-drop",
        vec![Mutation::UpsertByKey(batch(&[(1, 87), (3, 89), (4, 86)]))],
    );
    let r = applied(s.apply(t));
    assert_eq!(r.operations[0].late_dropped, 2);
    assert_eq!(r.operations[0].inserted, 1);
    assert_eq!(r.operations[0].updated, 0);
    assert_eq!(keys(&s), [1, 2, 3]);
    let t = tx(
        &s,
        "duplicate-late",
        vec![Mutation::AppendBatch(batch(&[(1, 80)]))],
    );
    assert!(matches!(s.apply(t), CommitOutcome::Rejected(_)));
    let t = tx(
        &s,
        "count-reorder",
        vec![
            Mutation::SetRetention(RetentionPolicy::Count(2)),
            Mutation::ReplaceSnapshot(batch(&[(3, 89), (2, 100), (5, 95)])),
        ],
    );
    let r = applied(s.apply(t));
    assert_eq!(r.operations[0].evicted, 1);
    assert_eq!(r.operations[1].evicted, 1);
    assert_eq!(keys(&s), [3, 5]);
}
#[test]
fn event_time_arithmetic_schema_and_wire_are_checked_without_float_rounding() {
    let mut s = store(&[(1, i64::MIN), (2, i64::MAX)]);
    let t = tx(
        &s,
        "wide",
        vec![Mutation::SetRetention(RetentionPolicy::EventTime(
            EventTimeWindow {
                field: T,
                width: i64::MAX,
                allowed_lateness: i64::MAX,
                watermark: i64::MIN,
                late: LateDataPolicy::Reject,
            },
        ))],
    );
    applied(s.apply(t));
    assert_eq!(keys(&s), [1, 2]);
    let w = window(i64::MAX, LateDataPolicy::Reject);
    let RetentionPolicy::EventTime(w) = w else {
        unreachable!()
    };
    let json = chart_core::portable::encode(&w).unwrap();
    assert!(json.contains("\"9223372036854775807\""));
    assert_eq!(
        chart_core::portable::decode::<EventTimeWindow>(&json).unwrap(),
        w
    );
    assert!(
        chart_core::portable::decode::<EventTimeWindow>(
            &json.replace("\"9223372036854775807\"", "9223372036854775807")
        )
        .is_err()
    );
    for w in [
        EventTimeWindow { width: 0, ..w },
        EventTimeWindow {
            allowed_lateness: -1,
            ..w
        },
        EventTimeWindow {
            field: FieldId::new(999),
            ..w
        },
    ] {
        let t = tx(
            &s,
            "invalid",
            vec![Mutation::SetRetention(RetentionPolicy::EventTime(w))],
        );
        assert!(matches!(s.apply(t), CommitOutcome::Rejected(_)));
    }
}
#[test]
fn queued_is_not_committed_backpressure_retries_conflicts_and_loss_are_observable() {
    let mut s = store(&[]);
    let t = tx(&s, "a", vec![Mutation::AppendBatch(batch(&[(1, 10)]))]);
    let mut q = IngestionQueue::new(QueueLimits {
        transactions: 1,
        rows: 1,
        bytes: t.payload_bytes(),
        overload: OverloadPolicy::Backpressure,
    })
    .unwrap();
    assert_eq!(q.enqueue(t.clone()), EnqueueOutcome::Queued);
    assert!(keys(&s).is_empty());
    assert_eq!(q.enqueue(t.clone()), EnqueueOutcome::AlreadyQueued);
    assert_eq!(q.status().accepted, 1);
    let mut conflicting = t.clone();
    conflicting.operations = vec![Operation {
        dataset: D,
        mutation: Mutation::RemoveKeys(vec![]),
    }];
    assert!(matches!(
        q.enqueue(conflicting),
        EnqueueOutcome::Rejected(_)
    ));
    let b = tx(&s, "b", vec![Mutation::AppendBatch(batch(&[(2, 20)]))]);
    assert_eq!(q.enqueue(b.clone()), EnqueueOutcome::Backpressure);
    assert!(matches!(
        q.commit_next(&mut s),
        Some((_, CommitOutcome::Applied(_)))
    ));
    assert_eq!(keys(&s), [1]);
    assert_eq!(q.enqueue(b), EnqueueOutcome::Queued);
    assert!(matches!(
        q.commit_next(&mut s),
        Some((_, CommitOutcome::Conflict(_)))
    ));
    assert_eq!(q.status().failed, 1);
    assert_eq!(q.status().backpressured, 1);
    assert_eq!(q.status().bytes, 0);
    assert_eq!(q.status().rows, 0);
    assert_eq!(q.enqueue(t), EnqueueOutcome::Queued);
    assert!(matches!(
        q.commit_next(&mut s),
        Some((_, CommitOutcome::AlreadyApplied(_)))
    ));
    let mut q = IngestionQueue::new(QueueLimits {
        transactions: 1,
        rows: 1,
        bytes: 10000,
        overload: OverloadPolicy::DropNewest,
    })
    .unwrap();
    let t = tx(
        &s,
        "too-large",
        vec![Mutation::AppendBatch(batch(&[(2, 20), (3, 30)]))],
    );
    assert_eq!(q.enqueue(t), EnqueueOutcome::Dropped);
    assert_eq!(q.status().dropped_rows, 2);
    assert_eq!(q.status().accepted, 0);
    assert!(q.commit_next(&mut s).is_none());
}

fn stat_batch(rows: &[(u64, i64, f64)]) -> NormalizedBatch {
    use chart_core::grammar::*;
    let typed = TypedRows::snapshot(
        D,
        Revision::INITIAL,
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        rows.to_vec(),
        10000,
    )
    .unwrap();
    TypedDataBuilder::new(typed.get().unwrap(), SchemaVersion::new(1))
        .timestamp(
            T,
            "time",
            TimestampType {
                unit: TimeUnit::Nanoseconds,
                timezone: "UTC".into(),
            },
            |r| Some(r.1),
        )
        .float(FieldId::new(2), "value", |r| Some(r.2))
        .finish(DataLimits::default())
        .unwrap()
}
#[test]
fn exact_chunk_updates_preserve_bins_and_members_after_correction_removal_and_retention() {
    use chart_core::{grammar::*, state::ChartState};
    let origin = 9_007_199_254_741_001;
    let initial = [
        (1, origin, 0.),
        (2, origin + 1, 1.),
        (3, origin + 2, 2.),
        (4, origin + 3, 3.),
    ];
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(D, stat_batch(&initial))],
        DataLimits {
            chunk_rows: 2,
            ..DataLimits::default()
        },
    )
    .unwrap();
    let bins = BinSpec::new(FieldId::new(2), vec![0., 2., 4., 10.]);
    let d =
        ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(LayerId::new(1), D, bins));
    let mut c = Compiler::new();
    let first = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(c.update_metrics().evaluated_rows, 4);
    for (index, mutation, expected_evaluated) in [
        (
            0,
            Mutation::AppendBatch(stat_batch(&[(5, origin + 2, 5.)])),
            1,
        ), // out-of-order event time
        (
            1,
            Mutation::UpsertByKey(stat_batch(&[(2, origin + 1, 6.)])),
            2,
        ), // only one immutable two-row chunk changes
        (2, Mutation::RemoveKeys(vec![RowKey::new(3)]), 1),
        (3, Mutation::SetRetention(RetentionPolicy::Count(2)), 0), // whole remaining chunks retained
    ] {
        let transaction = tx(&s, &format!("stat-{index}"), vec![mutation]);
        applied(s.apply(transaction));
        let p = c
            .prepare(
                &d,
                &s.snapshot(),
                &ChartState::default(),
                CompileLimits::default(),
            )
            .unwrap();
        let work = c.update_metrics();
        assert_eq!(work.updated_operations, 1);
        assert_eq!(work.evaluated_rows, expected_evaluated);
        let reference = Compiler::new()
            .prepare(
                &d,
                &s.snapshot(),
                &ChartState::default(),
                CompileLimits::default(),
            )
            .unwrap();
        assert_eq!(p.layers()[0].table(), reference.layers()[0].table());
        assert_eq!(p.domains(), reference.domains());
        let mut expected = [vec![], vec![], vec![]];
        for r in s.snapshot().get().unwrap().dataset(D).unwrap().rows() {
            let Some(ValueRef::Float64(v)) = r.value(FieldId::new(2)) else {
                unreachable!()
            };
            let index = if v < 2. {
                0
            } else if v < 4. {
                1
            } else {
                2
            };
            expected[index].push(r.key());
        }
        for e in &mut expected {
            e.sort_unstable();
        }
        let PreparedRows::Binned(actual) = p.layers()[0].table().rows() else {
            unreachable!()
        };
        for (b, e) in actual.iter().zip(expected) {
            assert_eq!(b.count, e.len() as u64);
            let chart_core::provenance::Target::Aggregate { members, input, .. } = &b.target else {
                unreachable!()
            };
            assert_eq!(members.as_ref(), e);
            assert_eq!(
                *input,
                s.snapshot().get().unwrap().dataset(D).unwrap().version()
            );
        }
        assert!(
            work.retained_chunks
                <= s.snapshot()
                    .get()
                    .unwrap()
                    .dataset(D)
                    .unwrap()
                    .chunks()
                    .len()
        );
    }
    let PreparedRows::Binned(old) = first.layers()[0].table().rows() else {
        unreachable!()
    };
    assert_eq!(old.iter().map(|b| b.count).collect::<Vec<_>>(), [2, 2, 0]);
    c.clear_cache();
    assert_eq!(c.update_metrics().retained_chunks, 0);
}

struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> ChartResult<chart_core::typography::ShapedRun> {
        Ok(chart_core::typography::ShapedRun {
            text: r.run.text.clone(),
            font: *r.default_font,
            font_size: r.font_size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: chart_core::services::TextMetrics::new(r.run.text.len() as f64 * 5., 8., 2.)?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
}
fn laid_out(p: chart_core::grammar::PreparedChart) -> Arc<chart_core::layout::LaidOutChart> {
    use chart_core::{layout::*, services::*};
    let req = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    Arc::new(layout(Arc::new(p), &req, &Metrics).unwrap())
}
#[test]
fn eviction_removes_active_selection_cancels_preview_and_preserves_labeled_pin_until_unpin() {
    use chart_core::{grammar::*, state::*};
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(D, stat_batch(&[(1, 0, 1.), (2, 1, 2.), (3, 2, 3.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        D,
        Geom::Point,
        SourceAes::new()
            .x(Numeric::Timestamp {
                field: T,
                origin: 0,
            })
            .y(FieldId::new(2)),
    ));
    let mut c = Compiler::new();
    let mut r = ActionReducer::default();
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    r.reconcile_prepared(&p).unwrap();
    let old = laid_out(p);
    let weak = Arc::downgrade(&old);
    r.present(old.clone());
    let target = MarkTarget {
        epoch: SourceEpoch::new(1),
        layer: LayerId::new(1),
        panel: None,
        identity: TargetIdentity::Source {
            dataset: D,
            key: RowKey::new(1),
        },
    };
    let send = |r: &mut ActionReducer, a| {
        r.dispatch(&d, r.request(&d, a, ActionOrigin::Control))
            .unwrap()
    };
    send(
        &mut r,
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![target.clone()],
        },
    );
    send(&mut r, ChartAction::SetPinned(Some(target.clone())));
    send(
        &mut r,
        ChartAction::SetFollow(FollowMode::FreezePresentation),
    );
    send(
        &mut r,
        ChartAction::BeginGesture {
            id: Revision::new(1),
            kind: GestureKind::Selection,
        },
    );
    send(
        &mut r,
        ChartAction::PreviewGesture {
            id: Revision::new(1),
            preview: GesturePreview::Selection(vec![target.clone()]),
        },
    );
    let t = tx(
        &s,
        "evict",
        vec![Mutation::SetRetention(RetentionPolicy::Count(2))],
    );
    applied(s.apply(t));
    let result = r.reconcile_source(&d, &s.snapshot()).unwrap();
    assert_eq!(
        result.removed_selection.as_slice(),
        std::slice::from_ref(&target)
    );
    assert_eq!(
        result.transition.event.unwrap().cancellation,
        Some(CancelReason::TargetRemoved)
    );
    assert!(r.state().selection().is_empty());
    assert!(r.state().active_gesture().is_none());
    assert!(r.pinned_is_historical());
    assert!(Arc::ptr_eq(r.presented().unwrap(), &old));
    let desc = r.describe_pinned().unwrap().unwrap();
    assert!(desc.historical);
    assert_eq!(desc.target.target, target);
    let request = r.request(
        &d,
        ChartAction::Select {
            change: SelectionChange::Add,
            targets: vec![target],
        },
        ActionOrigin::Control,
    );
    assert!(r.dispatch(&d, request).is_err()); // frozen old marks cannot resurrect evicted active selections
    send(&mut r, ChartAction::ResumeLatest);
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    r.reconcile_prepared(&p).unwrap();
    r.present(laid_out(p));
    drop(old);
    assert!(weak.upgrade().is_some());
    send(&mut r, ChartAction::SetPinned(None));
    assert!(weak.upgrade().is_none());
    assert!(r.describe_pinned().unwrap().is_none());
}
#[test]
fn follow_preserves_horizontal_span_and_inspect_freeze_keep_view_identity() {
    use chart_core::{grammar::*, state::*};
    let origin = 9_007_199_254_741_001;
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(D, stat_batch(&[(1, origin, 1.), (2, origin + 10, 2.)]))],
        DataLimits::default(),
    )
    .unwrap();
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        D,
        Geom::Point,
        SourceAes::new()
            .x(Numeric::Timestamp { field: T, origin })
            .y(FieldId::new(2)),
    ));
    let mut c = Compiler::new();
    let mut r = ActionReducer::default();
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    r.reconcile_prepared(&p).unwrap();
    r.present(laid_out(p));
    let send = |r: &mut ActionReducer, a| {
        r.dispatch(&d, r.request(&d, a, ActionOrigin::Control))
            .unwrap()
    };
    send(
        &mut r,
        ChartAction::SetAxisWindows(
            [
                (
                    ScaleId::new(0),
                    AxisWindow::Timestamp(origin + 2, origin + 7),
                ),
                (ScaleId::new(1), AxisWindow::Numeric(0., 5.)),
            ]
            .into_iter()
            .collect(),
        ),
    );
    let saved = r.state().axis_windows().into_owned();
    let t = tx(
        &s,
        "append",
        vec![Mutation::AppendBatch(stat_batch(&[(3, origin + 20, 3.)]))],
    );
    applied(s.apply(t));
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    r.reconcile_prepared(&p).unwrap();
    assert_eq!(r.state().axis_windows().as_ref(), &saved);
    send(&mut r, ChartAction::ResumeLatest);
    let t = tx(
        &s,
        "next",
        vec![Mutation::AppendBatch(stat_batch(&[(4, origin + 30, 4.)]))],
    );
    applied(s.apply(t));
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    let result = r.reconcile_prepared(&p).unwrap();
    assert!(result.transition.outcome.viewport_changed);
    assert_eq!(
        r.state().axis_windows()[&ScaleId::new(0)],
        AxisWindow::Timestamp(origin + 25, origin + 30)
    );
    assert_eq!(
        r.state().axis_windows()[&ScaleId::new(1)],
        AxisWindow::Numeric(0., 5.)
    );
    let revision = r.state().revision();
    assert!(!r.reconcile_prepared(&p).unwrap().transition.outcome.changed);
    assert_eq!(r.state().revision(), revision);
}

#[test]
fn incremental_invalid_values_limits_and_failed_preparations_do_not_poison_reuse() {
    use chart_core::{grammar::*, state::ChartState};
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(
            D,
            stat_batch(&[(1, 0, f64::NAN), (2, 1, -1.), (3, 2, 1.), (4, 3, 5.)]),
        )],
        DataLimits {
            chunk_rows: 2,
            ..DataLimits::default()
        },
    )
    .unwrap();
    let mut d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        LayerId::new(1),
        D,
        BinSpec::new(FieldId::new(2), vec![0., 2., 4.]),
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
    let counts = &p.layers()[0].table().operations()[0].counts;
    assert_eq!((counts.invalid_stat, counts.below, counts.above), (1, 1, 1));
    d.layers[0].invalid = InvalidPolicy::Strict;
    assert!(
        c.prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default()
        )
        .is_err()
    );
    assert_eq!(c.update_metrics().retained_chunks, 0);
    let t = tx(
        &s,
        "correct",
        vec![Mutation::UpsertByKey(stat_batch(&[(1, 0, 3.)]))],
    );
    applied(s.apply(t));
    d.layers[0].invalid = InvalidPolicy::Exclude;
    let StatParameters::Bin(spec) = &mut d.layers[0].statistic.parameters else {
        unreachable!()
    };
    spec.outliers = OutlierPolicy::Overflow;
    let p = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    let PreparedRows::Binned(b) = p.layers()[0].table().rows() else {
        unreachable!()
    };
    assert_eq!(b.iter().map(|b| b.count).collect::<Vec<_>>(), [2, 2]);
    assert!(
        c.prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits {
                max_groups: 0,
                ..CompileLimits::default()
            }
        )
        .is_err()
    );
    let p = c
        .prepare(
            &d,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(
        p.layers()[0].table(),
        Compiler::new()
            .prepare(
                &d,
                &s.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap()
            .layers()[0]
            .table()
    );
}

#[test]
fn legacy_follow_window_is_used_by_prepared_timestamp_axis_after_reconciliation() {
    use chart_core::{grammar::*, layout::ResolvedScale, state::*};
    let origin = 9_007_199_254_741_001;
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(
            D,
            stat_batch(&[
                (1, origin + 15, 35.),
                (2, origin + 20, 45.),
                (3, origin + 25, 25.),
            ]),
        )],
        DataLimits::default(),
    )
    .unwrap();
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        D,
        Geom::Point,
        SourceAes::new()
            .x(Numeric::Timestamp { field: T, origin })
            .y(FieldId::new(2)),
    ));
    let mut c = Compiler::new();
    let mut r = ActionReducer::default();
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    r.reconcile_prepared(&p).unwrap();
    r.present(laid_out(p));
    r.dispatch(
        &d,
        r.request(
            &d,
            ChartAction::SetViewport(Viewport {
                x: Some((5., 20.)),
                y: None,
            }),
            ActionOrigin::Control,
        ),
    )
    .unwrap();
    r.dispatch(
        &d,
        r.request(&d, ChartAction::ResumeLatest, ActionOrigin::Control),
    )
    .unwrap();
    let t = tx(
        &s,
        "new-latest",
        vec![Mutation::AppendBatch(stat_batch(&[(4, origin + 50, 30.)]))],
    );
    applied(s.apply(t));
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    let result = r.reconcile_prepared(&p).unwrap();
    assert!(result.transition.outcome.changed);
    let p = c
        .prepare(&d, &s.snapshot(), r.state(), CompileLimits::default())
        .unwrap();
    assert_eq!(p.state().viewport().x, Some((35., 50.)));
    let layout = laid_out(p);
    let ResolvedScale::Utc(axis) = &layout.axes()[&ScaleId::new(0)].scale else {
        unreachable!()
    };
    assert_eq!(
        axis.viewport(),
        chart_core::scales::TimeBounds {
            start: origin + 35,
            end: origin + 50
        }
    );
}
