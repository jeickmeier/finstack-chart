//! FIX-14: real worker barriers, atomic multi-dataset updates and exact ownership release.
#[path = "../../../fixtures/publication/support.rs"]
mod fixture;
use chart_core::{composition::Annotation, data::*, grammar::*, state::*, transaction::*, *};
use chart_export::*;
use fixture::*;
use serde_json::json;
use std::sync::{Arc, Weak, mpsc};
fn annotation(label: &str) -> Annotation {
    serde_json::from_value(json!({"id":"live-label","anchor":{"Figure":{"x":0.1,"y":0.1}},"text":{"lines":[[{"text":label}]],"line_spacing":1.2,"rotation":0.}})).unwrap()
}
fn request() -> FigureRequest {
    FigureRequest::new(
        definition(),
        store().snapshot(),
        ChartState::default(),
        fonts(),
        profile(),
        InteractionCapture::default(),
    )
    .unwrap()
}
fn owned() -> (FigureRequest, Weak<[u8]>) {
    owned_profile(profile())
}
fn owned_profile(p: PublicationProfile) -> (FigureRequest, Weak<[u8]>) {
    let bytes: Arc<[u8]> = Arc::from(FONT_BYTES);
    let weak = Arc::downgrade(&bytes);
    let f = FontResource::new(font().descriptor(), bytes).unwrap();
    (
        FigureRequest::new(
            definition(),
            store().snapshot(),
            ChartState::default(),
            FontResources::new(vec![f]).unwrap(),
            p,
            InteractionCapture::default(),
        )
        .unwrap(),
        weak,
    )
}
fn empty(q: &ExportQueue) {
    let m = q.metrics();
    assert_eq!((m.pending, m.running, m.rows, m.input_bytes), (0, 0, 0, 0));
}
#[test]
fn worker_capture_survives_eighty_atomic_commits_annotation_theme_and_quality_edits() {
    let second = DatasetId::new(2);
    let mut store = DataStore::new(
        SourceEpoch::new(7),
        vec![
            (DATA, batch(&[(1, 0., Some(100.)), (2, 1., Some(101.))])),
            (second, batch(&[(1, 0., Some(200.)), (2, 1., Some(201.))])),
        ],
        DataLimits::default(),
    )
    .unwrap();
    let mut definition = ChartDefinition::new(Revision::new(1))
        .layer(Layer::new(
            LayerId::new(1),
            DATA,
            Geom::Point,
            SourceAes::new().x(X).y(Y),
        ))
        .layer(Layer::new(
            LayerId::new(2),
            second,
            Geom::Point,
            SourceAes::new().x(X).y(Y),
        ));
    let mut state = ChartState::default();
    state
        .apply(
            &definition,
            ChartAction::SetAnnotation(Box::new(annotation("Captured label"))),
        )
        .unwrap();
    let mut p = profile();
    let captured = FigureRequest::new(
        definition.clone(),
        store.snapshot(),
        state.clone(),
        fonts(),
        p.clone(),
        InteractionCapture::ALL,
    )
    .unwrap();
    let expected = captured.prepare().unwrap();
    let old_source = expected.layout().prepared().source().get().unwrap();
    assert_eq!(
        old_source
            .dataset(DATA)
            .unwrap()
            .row(RowKey::new(1))
            .unwrap()
            .value(Y),
        Some(ValueRef::Float64(100.))
    );
    assert_eq!(
        old_source
            .dataset(second)
            .unwrap()
            .row(RowKey::new(1))
            .unwrap()
            .value(Y),
        Some(ValueRef::Float64(200.))
    );
    let expected_bytes = expected.export(Format::Svg).unwrap().bytes;
    let queue = ExportQueue::new(ExportLimits::default()).unwrap();
    let job = queue.submit(captured, Format::Svg).unwrap();
    let (reached_tx, reached_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        job.run_with_observer(|phase| {
            if phase != ExportPhase::Encoded {
                reached_tx.send(phase).unwrap();
                resume_rx.recv().unwrap();
            }
            Ok(())
        })
    });
    let mut count = 0u64;
    for phase in [ExportPhase::Captured, ExportPhase::Prepared] {
        assert_eq!(reached_rx.recv().unwrap(), phase);
        assert_eq!(queue.metrics().running, 1);
        for _ in 0..40 {
            count += 1;
            let s = store.snapshot();
            let source = s.get().unwrap();
            let outcome = store.apply(Transaction {
                id: TransactionId::new(format!("during-export-{count}")).unwrap(),
                epoch: source.epoch(),
                expected: source.datasets().map(|d| d.version()).collect(),
                operations: vec![
                    Operation {
                        dataset: DATA,
                        mutation: Mutation::UpsertByKey(batch(&[(
                            1,
                            0.,
                            Some(1000. + count as f64),
                        )])),
                    },
                    Operation {
                        dataset: second,
                        mutation: Mutation::UpsertByKey(batch(&[(
                            1,
                            0.,
                            Some(2000. + count as f64),
                        )])),
                    },
                ],
            });
            assert!(matches!(outcome, CommitOutcome::Applied(_)));
            state
                .apply(
                    &definition,
                    ChartAction::SetAnnotation(Box::new(annotation(&format!(
                        "Live label {count}"
                    )))),
                )
                .unwrap();
        }
        definition.theme = Some(chart_core::theme::ThemeSpec::named(
            chart_core::theme::NamedTheme::Terminal,
        ));
        definition.revision = definition.revision.checked_next().unwrap();
        p.page = PageSize::points(400., 240.).unwrap();
        p.dpi = 600;
        p.precision = 0.001;
        resume_tx.send(()).unwrap();
    }
    let artifact = worker.join().unwrap().unwrap();
    assert_eq!(artifact.bytes, expected_bytes);
    assert_eq!(artifact.metadata.stamp.store, Revision::INITIAL);
    assert!(
        artifact
            .metadata
            .datasets
            .iter()
            .all(|(d, _)| d.revision == Revision::INITIAL)
    );
    assert_eq!(artifact.metadata.definition.revision, Revision::new(1));
    assert_eq!(artifact.metadata.profile.dpi, 300);
    assert_eq!(
        artifact
            .metadata
            .captured_state
            .annotations(&artifact.metadata.definition)[0]
            .text
            .lines[0][0]
            .text,
        "Captured label"
    );
    let source = store.snapshot();
    let s = source.get().unwrap();
    assert_eq!(s.revision(), Revision::new(80));
    for (d, y) in [(DATA, 1080.), (second, 2080.)] {
        assert_eq!(s.dataset(d).unwrap().version().revision, Revision::new(80));
        assert_eq!(
            s.dataset(d).unwrap().row(RowKey::new(1)).unwrap().value(Y),
            Some(ValueRef::Float64(y))
        );
    }
    empty(&queue);
    assert_eq!(queue.metrics().completed, 1);
}
#[test]
fn capacity_pending_cancel_drop_and_disposal_release_inputs_even_with_retained_controls() {
    let queue = ExportQueue::new(ExportLimits {
        max_jobs: 1,
        ..Default::default()
    })
    .unwrap();
    let (req, weak) = owned();
    let job = queue.submit(req, Format::Svg).unwrap();
    let cancel = job.cancellation();
    assert!(weak.upgrade().is_some());
    assert!(
        matches!(queue.submit(request(),Format::Svg),Err(e) if e.code==DiagnosticCode::ResourceLimit)
    );
    assert!(cancel.cancel());
    empty(&queue);
    assert!(weak.upgrade().is_none());
    assert!(!cancel.cancel());
    assert!(matches!(job.run(),Err(e) if e.code==DiagnosticCode::Cancelled));
    let (req, weak) = owned();
    let dropped = queue.submit(req, Format::Svg).unwrap();
    drop(dropped);
    assert!(weak.upgrade().is_none());
    empty(&queue);
    let (req, weak) = owned();
    let pending = queue.submit(req, Format::Svg).unwrap();
    queue.dispose();
    assert!(weak.upgrade().is_none());
    assert!(matches!(pending.run(),Err(e) if e.code==DiagnosticCode::Cancelled));
    empty(&queue);
    assert!(queue.metrics().disposed);
    assert_eq!(queue.metrics().cancelled, 3);
    let tiny = ExportQueue::new(ExportLimits {
        max_rows: 1,
        ..Default::default()
    })
    .unwrap();
    assert!(
        matches!(tiny.submit(request(),Format::Svg),Err(e) if e.code==DiagnosticCode::ResourceLimit)
    );
    empty(&tiny);
    let tiny = ExportQueue::new(ExportLimits {
        max_input_bytes: 1,
        ..Default::default()
    })
    .unwrap();
    assert!(
        matches!(tiny.submit(request(),Format::Svg),Err(e) if e.code==DiagnosticCode::ResourceLimit)
    );
    empty(&tiny);
}
#[test]
fn active_cancellation_and_errors_release_at_safe_boundaries_without_returning_partial_bytes() {
    let queue = ExportQueue::new(ExportLimits::default()).unwrap();
    let (req, weak) = owned();
    let job = queue.submit(req, Format::Png).unwrap();
    let cancel = job.cancellation();
    let (tx, rx) = mpsc::channel();
    let (go, wait) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        job.run_with_observer(|phase| {
            if phase == ExportPhase::Prepared {
                tx.send(()).unwrap();
                wait.recv().unwrap();
            }
            Ok(())
        })
    });
    rx.recv().unwrap();
    assert_eq!(queue.metrics().running, 1);
    assert!(cancel.cancel());
    assert!(weak.upgrade().is_some());
    go.send(()).unwrap();
    assert!(matches!(worker.join().unwrap(),Err(e) if e.code==DiagnosticCode::Cancelled));
    assert!(weak.upgrade().is_none());
    empty(&queue);
    let mut p = profile();
    p.max_raster_pixels = 1;
    let (bad, weak) = owned_profile(p);
    assert!(
        matches!(queue.submit(bad,Format::Png).unwrap().run(),Err(e) if e.code==DiagnosticCode::ResourceLimit)
    );
    assert!(weak.upgrade().is_none());
    empty(&queue);
    assert_eq!(queue.metrics().failed, 1);
    let (req, weak) = owned();
    let job = queue.submit(req, Format::Svg).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        job.run_with_observer(|_| panic!("host observer failed"))
    }));
    assert!(result.is_err());
    empty(&queue);
    assert!(weak.upgrade().is_none());
}
fn send(r: &mut ActionReducer, d: &ChartDefinition, a: ChartAction, stamp: SceneStamp) {
    let request = ActionRequest {
        definition_revision: d.revision,
        expected_state: r.state().revision(),
        origin: ActionOrigin::Control,
        scene: Some(stamp),
        action: a,
    };
    r.dispatch(d, request).unwrap();
}
#[test]
fn explicit_interaction_and_full_domain_policies_do_not_change_live_state_or_numeric_population() {
    let d = definition();
    let source = store().snapshot();
    let base = FigureSnapshot::capture(
        &d,
        source.clone(),
        &ChartState::default(),
        fonts(),
        profile(),
    )
    .unwrap();
    let stamp = base.scene().stamp();
    let mut reducer = ActionReducer::default();
    reducer.present(base.layout().clone());
    let target = MarkTarget {
        epoch: SourceEpoch::new(7),
        layer: LayerId::new(2),
        panel: None,
        identity: TargetIdentity::Source {
            dataset: DATA,
            key: RowKey::new(1),
        },
    };
    for a in [
        ChartAction::SetViewport(Viewport {
            x: Some((0.5, 1.5)),
            y: None,
        }),
        ChartAction::SetHover(vec![target.clone()]),
        ChartAction::SetFocus(Some(target.clone())),
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![target],
        },
        ChartAction::SetAnnotation(Box::new(annotation("Committed annotation"))),
        ChartAction::BeginGesture {
            id: Revision::new(1),
            kind: GestureKind::Annotation("live-label".into()),
        },
        ChartAction::PreviewGesture {
            id: Revision::new(1),
            preview: GesturePreview::Annotation(Box::new(annotation("Transient annotation"))),
        },
    ] {
        send(&mut reducer, &d, a, stamp);
    }
    let original = reducer.state().clone();
    let all = FigureRequest::new(
        d.clone(),
        source.clone(),
        original.clone(),
        fonts(),
        profile(),
        InteractionCapture::ALL,
    )
    .unwrap()
    .prepare()
    .unwrap();
    let clean = FigureRequest::new(
        d.clone(),
        source.clone(),
        original.clone(),
        fonts(),
        profile(),
        InteractionCapture::default(),
    )
    .unwrap()
    .prepare()
    .unwrap();
    let c = clean.layout().prepared().state();
    assert!(
        c.selection().is_empty()
            && c.hover().is_empty()
            && c.focus().is_none()
            && c.active_gesture().is_none()
    );
    assert_eq!(
        c.annotations(&d)[0].text.lines[0][0].text,
        "Committed annotation"
    );
    assert_eq!(
        all.layout().prepared().state().annotations(&d)[0]
            .text
            .lines[0][0]
            .text,
        "Transient annotation"
    );
    assert_eq!(
        all.layout().prepared().layers()[0].table().rows(),
        clean.layout().prepared().layers()[0].table().rows()
    );
    assert_eq!(reducer.state(), &original);
    let mut p = profile();
    p.view = ViewMode::FullDomain;
    let full = FigureRequest::new(
        d.clone(),
        source,
        original.clone(),
        fonts(),
        p,
        InteractionCapture::default(),
    )
    .unwrap()
    .prepare()
    .unwrap();
    assert_eq!(
        full.layout().prepared().state().viewport(),
        Viewport::default()
    );
    assert_eq!(full.metadata().captured_state, original);
    assert_eq!(
        full.metadata().manifest()["interaction_policy"]["preview"],
        false
    );
    let mut p = profile();
    p.view = ViewMode::FullDomain;
    let full_preview = FigureRequest::new(
        d.clone(),
        store().snapshot(),
        original.clone(),
        fonts(),
        p,
        InteractionCapture::ALL,
    )
    .unwrap()
    .prepare()
    .unwrap();
    assert_eq!(
        full_preview.layout().prepared().state().annotations(&d)[0]
            .text
            .lines[0][0]
            .text,
        "Transient annotation"
    );
    assert_eq!(
        full_preview.layout().prepared().state().viewport(),
        Viewport::default()
    );
    send(
        &mut reducer,
        &d,
        ChartAction::CancelGesture(CancelReason::Explicit),
        stamp,
    );
    send(
        &mut reducer,
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(2),
            kind: GestureKind::Viewport,
        },
        stamp,
    );
    send(
        &mut reducer,
        &d,
        ChartAction::PreviewGesture {
            id: Revision::new(2),
            preview: GesturePreview::Viewport(Viewport {
                x: Some((1., 2.)),
                y: None,
            }),
        },
        stamp,
    );
    let mut p = profile();
    p.view = ViewMode::FullDomain;
    let full = FigureRequest::new(
        d,
        store().snapshot(),
        reducer.state().clone(),
        fonts(),
        p,
        InteractionCapture::ALL,
    )
    .unwrap()
    .prepare()
    .unwrap();
    assert_eq!(
        full.layout().prepared().state().viewport(),
        Viewport::default()
    );
}
#[test]
fn out_of_order_exports_keep_their_own_snapshot_and_success_releases_font_ownership() {
    let queue = ExportQueue::new(ExportLimits::default()).unwrap();
    let (req, weak) = owned();
    let old = queue.submit(req, Format::Svg).unwrap();
    let control = old.cancellation();
    let mut data = store();
    let source = data.snapshot();
    let s = source.get().unwrap();
    assert!(matches!(
        data.apply(Transaction {
            id: TransactionId::new("later").unwrap(),
            epoch: s.epoch(),
            expected: vec![s.dataset(DATA).unwrap().version()],
            operations: vec![Operation {
                dataset: DATA,
                mutation: Mutation::UpsertByKey(batch(&[(1, 0., Some(999.))]))
            }]
        }),
        CommitOutcome::Applied(_)
    ));
    let newer = queue
        .submit(
            FigureRequest::new(
                definition(),
                data.snapshot(),
                ChartState::default(),
                fonts(),
                profile(),
                InteractionCapture::default(),
            )
            .unwrap(),
            Format::Svg,
        )
        .unwrap();
    let newer = newer.run().unwrap();
    let older = old.run().unwrap();
    assert_eq!(newer.metadata.stamp.store, Revision::new(1));
    assert_eq!(older.metadata.stamp.store, Revision::INITIAL);
    assert_ne!(newer.bytes, older.bytes);
    assert!(weak.upgrade().is_none());
    assert!(!control.cancel());
    empty(&queue);
    assert_eq!(queue.metrics().completed, 2);
}
