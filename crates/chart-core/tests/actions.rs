//! WP-15 independent state/action traces. All tests run without a window or worker thread.
use chart_core::{
    composition::*, data::*, grammar::*, layout::*, services::*, state::*, transaction::DataStore,
    typography::RichText, *,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
    // Explicit deterministic service double; actual font/paint integration is tested in export.
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
            metrics: TextMetrics::new(r.run.text.len() as f64 * 5., 8., 2.)?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
}
fn definition() -> ChartDefinition {
    ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        DatasetId::new(1),
        Geom::Point,
        SourceAes::new().x(FieldId::new(1)).y(FieldId::new(2)),
    ))
}
fn scene(d: &ChartDefinition, state: &ChartState, width: f64) -> Arc<LaidOutChart> {
    let rows = TypedRows::snapshot(
        DatasetId::new(1),
        Revision::new(1),
        vec![RowKey::new(101), RowKey::new(102), RowKey::new(103)],
        vec![(0., 2.), (1., 4.), (2., 6.)],
        10,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(FieldId::new(1), "x", |r| Some(r.0))
        .float(FieldId::new(2), "y", |r| Some(r.1))
        .finish(DataLimits::default())
        .unwrap();
    let source = DataStore::new(
        SourceEpoch::new(7),
        vec![(DatasetId::new(1), batch)],
        DataLimits::default(),
    )
    .unwrap();
    let p = Compiler::new()
        .prepare(d, &source.snapshot(), state, CompileLimits::default())
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., width, 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.revision = Revision::new(width as u64);
    Arc::new(layout(Arc::new(p), &request, &Metrics).unwrap())
}
fn mark(key: u64) -> MarkTarget {
    MarkTarget {
        epoch: SourceEpoch::new(7),
        layer: LayerId::new(1),
        panel: None,
        identity: TargetIdentity::Source {
            dataset: DatasetId::new(1),
            key: RowKey::new(key),
        },
    }
}
fn send(r: &mut ActionReducer, d: &ChartDefinition, action: ChartAction) -> DispatchOutcome {
    r.dispatch(d, r.request(d, action, ActionOrigin::Control))
        .unwrap()
}
fn annotation(x: f64) -> Annotation {
    Annotation {
        id: "threshold".into(),
        anchor: Anchor::Figure { x, y: 0.1 },
        text: RichText::plain("Threshold"),
        offset: [0., 0.],
        priority: 0,
        collision: Collision::Keep,
        callout: None,
        connector_origin: chart_core::composition::ConnectorOrigin::Label,
        overflow: false,
    }
}
fn view(x: f64) -> Viewport {
    Viewport {
        x: Some((x, x + 1.)),
        y: None,
    }
}
#[test]
fn effective_changes_separate_hover_focus_pin_selection_follow_and_visibility() {
    let d = definition();
    let mut r = ActionReducer::default();
    r.present(scene(&d, r.state(), 400.));
    let hover = r.request(
        &d,
        ChartAction::SetHover(vec![mark(101)]),
        ActionOrigin::Pointer,
    );
    let first = r.dispatch(&d, hover.clone()).unwrap();
    assert_eq!(first.outcome.revision, Revision::new(1));
    assert_eq!(first.event.as_ref().unwrap().origin, ActionOrigin::Pointer);
    assert!(!first.event.unwrap().durable);
    assert!(r.state().focus().is_none());
    assert_eq!(r.state().revisions().hover, Revision::new(1));
    assert!(
        send(&mut r, &d, ChartAction::SetHover(vec![mark(101)]))
            .event
            .is_none()
    );
    assert_eq!(
        r.dispatch(&d, hover).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    send(&mut r, &d, ChartAction::SetFocus(Some(mark(102))));
    send(&mut r, &d, ChartAction::SetPinned(Some(mark(103))));
    assert_eq!(r.state().hover().iter().next(), Some(&mark(101)));
    assert_eq!(r.state().focus(), Some(&mark(102)));
    assert_eq!(r.state().pinned(), Some(&mark(103)));
    send(
        &mut r,
        &d,
        ChartAction::Select {
            change: SelectionChange::Add,
            targets: vec![mark(101), mark(102)],
        },
    );
    send(
        &mut r,
        &d,
        ChartAction::Select {
            change: SelectionChange::Toggle,
            targets: vec![mark(102), mark(103)],
        },
    );
    assert_eq!(
        r.state().selection().iter().cloned().collect::<Vec<_>>(),
        [mark(101), mark(103)]
    );
    send(&mut r, &d, ChartAction::SetLegendVisible(false));
    assert!(r.state().is_visible(LayerId::new(1)));
    assert!(!r.state().legend_visible());
    send(&mut r, &d, ChartAction::SetViewport(view(0.25)));
    assert_eq!(r.state().follow(), FollowMode::InspectHistory);
    assert_eq!(r.state().viewport_revision(), Revision::new(1));
    send(&mut r, &d, ChartAction::ResumeLatest);
    assert_eq!(r.state().follow(), FollowMode::FollowLatest);
    assert_eq!(r.state().viewport(), view(0.25));
    assert_eq!(r.history_lengths(), (0, 0));
    let before = r.state().clone();
    let request = r.request(
        &d,
        ChartAction::SetFocus(Some(mark(999))),
        ActionOrigin::Keyboard,
    );
    assert_eq!(
        r.dispatch(&d, request).unwrap_err().code,
        DiagnosticCode::Validation
    );
    assert_eq!(r.state(), &before);
}
#[test]
fn gestures_pin_old_coordinates_cancel_without_committing_and_reject_competing_owners() {
    let d = definition();
    let mut r = ActionReducer::default();
    let first = scene(&d, r.state(), 400.);
    let weak = Arc::downgrade(&first);
    let stamp = first.scene().stamp();
    let plot = first.plot().unwrap();
    r.present(first.clone());
    drop(first);
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(1),
            kind: GestureKind::Viewport,
        },
    );
    let conflict = r.request(
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(2),
            kind: GestureKind::Selection,
        },
        ActionOrigin::Pointer,
    );
    assert_eq!(
        r.dispatch(&d, conflict).unwrap_err().code,
        DiagnosticCode::Validation
    );
    r.present(scene(&d, r.state(), 800.));
    assert_eq!(r.gesture_basis().unwrap().plot(), Some(plot));
    assert_eq!(r.gesture_basis().unwrap().scene().stamp(), stamp);
    let request = r.request(
        &d,
        ChartAction::PreviewGesture {
            id: Revision::new(1),
            preview: GesturePreview::Viewport(view(0.5)),
        },
        ActionOrigin::Pointer,
    );
    let mut wrong = request.clone();
    wrong.scene = r.presented().map(|p| p.scene().stamp());
    assert_eq!(
        r.dispatch(&d, wrong).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    let preview = r.dispatch(&d, request).unwrap();
    assert!(!preview.event.unwrap().durable);
    assert_eq!(r.state().viewport(), view(0.5));
    assert_eq!(r.state().committed_viewport(), Viewport::default());
    assert_eq!(r.state().revisions().durable, Revision::INITIAL);
    let cancel = send(
        &mut r,
        &d,
        ChartAction::CancelGesture(CancelReason::CaptureLost),
    );
    assert_eq!(
        cancel.event.unwrap().cancellation,
        Some(CancelReason::CaptureLost)
    );
    assert_eq!(r.state().viewport(), Viewport::default());
    assert!(r.gesture_basis().is_none());
    assert!(weak.upgrade().is_none());
    assert_eq!(r.history_lengths(), (0, 0));
    assert!(
        send(
            &mut r,
            &d,
            ChartAction::CancelGesture(CancelReason::Explicit)
        )
        .event
        .is_none()
    );
}
#[test]
fn annotation_preview_is_one_undo_command_and_cancellation_never_changes_source_or_history() {
    let mut d = definition();
    d.figure = Some(FigureComposition {
        annotations: vec![annotation(0.1)],
        ..Default::default()
    });
    let mut r = ActionReducer::default();
    r.present(scene(&d, r.state(), 400.));
    assert!(
        send(
            &mut r,
            &d,
            ChartAction::SetAnnotation(annotation(0.1).into())
        )
        .event
        .is_none()
    );
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(1),
            kind: GestureKind::Annotation("threshold".into()),
        },
    );
    for i in 1..=50 {
        let result = send(
            &mut r,
            &d,
            ChartAction::PreviewGesture {
                id: Revision::new(1),
                preview: GesturePreview::Annotation(annotation(0.1 + f64::from(i) / 100.).into()),
            },
        );
        assert!(!result.event.unwrap().durable);
        assert_eq!(r.history_lengths(), (0, 0));
    }
    assert_eq!(r.state().annotations(&d), [annotation(0.6)]);
    send(
        &mut r,
        &d,
        ChartAction::CommitGesture {
            id: Revision::new(1),
        },
    );
    assert_eq!(r.history_lengths(), (1, 0));
    assert_eq!(r.state().revisions().annotations, Revision::new(1));
    send(&mut r, &d, ChartAction::Undo);
    assert_eq!(r.state().annotations(&d), [annotation(0.1)]);
    assert_eq!(r.history_lengths(), (0, 1));
    send(&mut r, &d, ChartAction::Redo);
    assert_eq!(r.state().annotations(&d), [annotation(0.6)]);
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(2),
            kind: GestureKind::Annotation("threshold".into()),
        },
    );
    send(
        &mut r,
        &d,
        ChartAction::PreviewGesture {
            id: Revision::new(2),
            preview: GesturePreview::Annotation(annotation(0.9).into()),
        },
    );
    send(
        &mut r,
        &d,
        ChartAction::CancelGesture(CancelReason::FocusLost),
    );
    assert_eq!(r.state().annotations(&d), [annotation(0.6)]);
    assert_eq!(r.history_lengths(), (1, 0));
    let p = scene(&d, r.state(), 400.);
    let rows = p
        .prepared()
        .source()
        .get()
        .unwrap()
        .dataset(DatasetId::new(1))
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.row(RowKey::new(102)).unwrap().value(FieldId::new(2)),
        Some(ValueRef::Float64(4.))
    );
    assert_eq!(d.figure.unwrap().annotations, [annotation(0.1)]);
}
#[test]
fn controlled_acknowledgements_cannot_overwrite_newer_actions_or_reuse_components() {
    let d = definition();
    let mut r = ActionReducer::default();
    send(&mut r, &d, ChartAction::SetViewport(view(0.)));
    let old = r.state().clone();
    send(&mut r, &d, ChartAction::SetViewport(view(1.)));
    let current = r.state().clone();
    assert_eq!(
        r.accept_controlled(&d, old.revision(), old)
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(r.state(), &current);
    assert!(
        !r.accept_controlled(&d, current.revision(), current.clone())
            .unwrap()
    );
    let mut replacement = current.clone();
    replacement
        .apply(&d, ChartAction::SetViewport(view(2.)))
        .unwrap();
    assert!(
        r.accept_controlled(&d, current.revision(), replacement)
            .unwrap()
    );
    assert_eq!(r.state().viewport(), view(2.));
}
#[test]
fn linked_sequences_are_bounded_idempotent_and_preserve_origin_without_feedback_history() {
    let d = definition();
    let mut r = ActionReducer::default();
    r.present(scene(&d, r.state(), 400.));
    let linked = |r: &ActionReducer, sequence, v| {
        r.request(
            &d,
            ChartAction::Synchronize {
                revision: Revision::new(sequence),
                viewport: Some(view(v)),
                windows: None,
                selection: Some(vec![mark(101)]),
            },
            ActionOrigin::Linked("overview".into()),
        )
    };
    let result = r.dispatch(&d, linked(&r, 2, 0.)).unwrap();
    assert_eq!(
        result.event.unwrap().origin,
        ActionOrigin::Linked("overview".into())
    );
    assert!(r.dispatch(&d, linked(&r, 2, 0.)).unwrap().event.is_none());
    assert!(r.dispatch(&d, linked(&r, 1, 9.)).unwrap().event.is_none());
    assert_eq!(r.state().viewport(), view(0.));
    assert_eq!(r.history_lengths(), (0, 0));
    assert_eq!(
        r.dispatch(&d, linked(&r, 2, 2.)).unwrap_err().code,
        DiagnosticCode::TransactionReuse
    );
}
#[test]
fn freeze_resume_and_disposal_release_only_owned_snapshot_resources() {
    let d = definition();
    let mut r = ActionReducer::default();
    let first = scene(&d, r.state(), 400.);
    let weak = Arc::downgrade(&first);
    r.present(first);
    let stamp = r.presented().unwrap().scene().stamp();
    send(
        &mut r,
        &d,
        ChartAction::SetFollow(FollowMode::FreezePresentation),
    );
    r.present(scene(&d, r.state(), 800.));
    assert_eq!(r.presented().unwrap().scene().stamp(), stamp);
    assert!(weak.upgrade().is_some());
    send(&mut r, &d, ChartAction::ResumeLatest);
    assert!(weak.upgrade().is_none());
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id: Revision::new(1),
            kind: GestureKind::Selection,
        },
    );
    let result = r.dispose(&d).unwrap();
    assert_eq!(
        result.event.unwrap().cancellation,
        Some(CancelReason::Disposed)
    );
    assert!(r.presented().is_none());
    assert!(r.gesture_basis().is_none());
    let request = r.request(&d, ChartAction::Reset, ActionOrigin::Control);
    assert_eq!(
        r.dispatch(&d, request).unwrap_err().code,
        DiagnosticCode::DisposedHandle
    );
}
#[test]
fn invalid_previews_and_resource_caps_preserve_valid_state_and_bounded_history() {
    let d = definition();
    let mut r = ActionReducer::default();
    r.present(scene(&d, r.state(), 400.));
    send(
        &mut r,
        &d,
        ChartAction::Configure(InteractionConfig {
            max_targets: 2,
            history_capacity: 2,
            ..Default::default()
        }),
    );
    assert_eq!(r.state().revisions().configuration, Revision::new(1));
    assert_eq!(r.state().revisions().durable, Revision::new(1));
    let before = r.state().clone();
    let request = r.request(
        &d,
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![mark(101), mark(102), mark(103)],
        },
        ActionOrigin::Control,
    );
    assert_eq!(
        r.dispatch(&d, request).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(r.state(), &before);
    for x in [0.1, 0.2, 0.3] {
        send(&mut r, &d, ChartAction::SetAnnotation(annotation(x).into()));
    }
    assert_eq!(r.history_lengths(), (2, 0));
    send(&mut r, &d, ChartAction::Undo);
    send(&mut r, &d, ChartAction::Undo);
    assert_eq!(r.state().annotations(&d), [annotation(0.1)]);
    assert!(send(&mut r, &d, ChartAction::Undo).event.is_none());
    let before = r.state().clone();
    let request = r.request(
        &d,
        ChartAction::SetAnnotation(annotation(f64::NAN).into()),
        ActionOrigin::Control,
    );
    assert_eq!(
        r.dispatch(&d, request).unwrap_err().code,
        DiagnosticCode::Validation
    );
    assert_eq!(r.state(), &before);
}

#[test]
fn legacy_viewports_replace_primary_named_windows_without_losing_other_axes() {
    use chart_core::ScaleId;
    let mut d = definition();
    d.axes = vec![
        AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
        AxisSpec::new(ScaleId::new(1), AxisSide::Left),
        AxisSpec::new(ScaleId::new(7), AxisSide::Top),
    ];
    let mut r = ActionReducer::default();
    r.present(scene(&d, r.state(), 400.));
    let windows: AxisWindows = [
        (ScaleId::new(0), AxisWindow::Numeric(0.5, 1.5)),
        (ScaleId::new(7), AxisWindow::Numeric(10., 20.)),
    ]
    .into_iter()
    .collect();
    send(&mut r, &d, ChartAction::SetAxisWindows(windows.clone()));
    let begin = |r: &mut ActionReducer| {
        let id = r.next_gesture_id().unwrap();
        send(
            r,
            &d,
            ChartAction::BeginGesture {
                id,
                kind: GestureKind::Viewport,
            },
        );
        id
    };
    let id = begin(&mut r);
    send(
        &mut r,
        &d,
        ChartAction::PreviewGesture {
            id,
            preview: GesturePreview::Viewport(Viewport::default()),
        },
    );
    assert!(!r.state().axis_windows().contains_key(&ScaleId::new(0)));
    assert_eq!(
        r.state().axis_windows()[&ScaleId::new(7)],
        AxisWindow::Numeric(10., 20.)
    );
    assert_eq!(
        r.state().interaction_snapshot().windows,
        windows,
        "preview is not serialized as committed"
    );
    send(
        &mut r,
        &d,
        ChartAction::CancelGesture(CancelReason::Explicit),
    );
    assert_eq!(r.state().axis_windows().as_ref(), &windows);
    let id = begin(&mut r);
    send(
        &mut r,
        &d,
        ChartAction::PreviewGesture {
            id,
            preview: GesturePreview::Viewport(view(1.)),
        },
    );
    send(&mut r, &d, ChartAction::CommitGesture { id });
    assert_eq!(r.state().viewport(), view(1.));
    assert!(!r.state().axis_windows().contains_key(&ScaleId::new(0)));
    send(&mut r, &d, ChartAction::SetAxisWindows(windows));
    send(&mut r, &d, ChartAction::SetViewport(view(2.)));
    assert_eq!(r.state().viewport(), view(2.));
    assert_eq!(r.state().axis_windows().len(), 1);
}

#[test]
fn frozen_resize_advances_projection_without_admitting_later_semantics() {
    let d = definition();
    let mut r = ActionReducer::default();
    let original = scene(&d, r.state(), 400.);
    let weak = Arc::downgrade(&original);
    r.present(original.clone());
    send(
        &mut r,
        &d,
        ChartAction::SetFollow(FollowMode::FreezePresentation),
    );
    let state = r.state().clone();
    let make = |revision| {
        let mut request = LayoutRequest::new(
            Rect::new(0., 0., 700., 300.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        request.revision = Revision::new(revision);
        Arc::new(layout(original.prepared().clone(), &request, &Metrics).unwrap())
    };
    let resized = make(401);
    assert_ne!(resized.scene().bounds(), original.scene().bounds());
    r.present_frozen(resized.clone()).unwrap();
    assert_eq!(r.state(), &state);
    assert!(Arc::ptr_eq(r.presented().unwrap(), &resized));
    assert!(Arc::ptr_eq(r.frozen_scene().unwrap(), &resized));
    // A new preparation from another store may have identical public IDs/revisions.
    let unrelated = scene(&d, &ChartState::default(), 800.);
    assert_eq!(
        unrelated.scene().stamp().store,
        original.scene().stamp().store
    );
    assert_eq!(
        r.present_frozen(unrelated).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(
        r.present_frozen(make(400)).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(
        r.present_frozen(resized.clone()).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    // Subsequent actions use the newly painted basis. The old pixel stamp is stale.
    let request = r.request(
        &d,
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![mark(101)],
        },
        ActionOrigin::Pointer,
    );
    assert_eq!(request.scene, Some(resized.scene().stamp()));
    r.dispatch(&d, request).unwrap();
    drop(original);
    assert!(weak.upgrade().is_none());
    send(
        &mut r,
        &d,
        ChartAction::SetFollow(FollowMode::InspectHistory),
    );
    assert_eq!(
        r.present_frozen(resized).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
}
