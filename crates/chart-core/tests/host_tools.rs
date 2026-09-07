//! WP-17 independent linked-view, annotation and accessible-data expectations.
use chart_core::{
    composition::*, data::*, editing::*, grammar::*, inspection::Inspector, layout::*, linking::*,
    scales::*, services::*, state::*, transaction::DataStore, typography::*, *,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
    fn shape(&self, r: ShapeRequest<'_>) -> ChartResult<ShapedRun> {
        Ok(ShapedRun {
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
fn definition(layer: u64) -> ChartDefinition {
    let mut d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(layer),
        DatasetId::new(1),
        Geom::Point,
        SourceAes::new().x(FieldId::new(1)).y(FieldId::new(2)),
    ));
    d.figure = Some(FigureComposition {
        annotations: vec![Annotation {
            id: "threshold".into(),
            anchor: Anchor::Data {
                panel: None,
                scales: ScaleBindings::default(),
                x: ScaleValue::Number(1.),
                y: ScaleValue::Number(4.),
            },
            callout: Some(Anchor::Data {
                panel: None,
                scales: ScaleBindings::default(),
                x: ScaleValue::Number(8.),
                y: ScaleValue::Number(4.),
            }),
            text: RichText::plain("Threshold"),
            offset: [0., 0.],
            priority: 0,
            collision: Collision::Keep,
            connector_origin: ConnectorOrigin::Anchor,
            overflow: false,
        }],
        ..FigureComposition::default()
    });
    d
}
fn scene(d: &ChartDefinition, state: &ChartState, width: f64) -> Arc<LaidOutChart> {
    let rows = TypedRows::snapshot(
        DatasetId::new(1),
        Revision::INITIAL,
        vec![RowKey::new(9007199254741001), RowKey::new(9007199254741002)],
        vec![(2., 3.), (6., 7.)],
        10,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(FieldId::new(1), "Time", |r| Some(r.0))
        .float(FieldId::new(2), "Value", |r| Some(r.1))
        .finish(DataLimits::default())
        .unwrap();
    let store = DataStore::new(
        SourceEpoch::new(7),
        vec![(DatasetId::new(1), batch)],
        DataLimits::default(),
    )
    .unwrap();
    let p = Compiler::new()
        .prepare(d, &store.snapshot(), state, CompileLimits::default())
        .unwrap();
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., width, 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.padding = 0.;
    for a in &mut r.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 10.).unwrap()));
    }
    Arc::new(layout(Arc::new(p), &r, &Metrics).unwrap())
}
fn send(r: &mut ActionReducer, d: &ChartDefinition, a: ChartAction) -> DispatchOutcome {
    r.dispatch(d, r.request(d, a, ActionOrigin::Control))
        .unwrap()
}
fn inspector(d: &ChartDefinition, r: &mut ActionReducer) -> Inspector {
    let s = scene(d, r.state(), 400.);
    r.present(s.clone());
    Inspector::new(s, 4., 16).unwrap()
}
#[test]
fn semantic_links_remap_layer_ids_and_suppress_cycles_without_source_edits() {
    let a = definition(1);
    let b = definition(99);
    let mut ra = ActionReducer::default();
    let mut rb = ActionReducer::default();
    let ia = inspector(&a, &mut ra);
    let ib = inspector(&b, &mut rb);
    let mark =
        MarkTarget::from_inspected(ia.semantic_targets().next().unwrap(), SourceEpoch::new(7));
    let event = send(
        &mut ra,
        &a,
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![mark],
        },
    )
    .event
    .unwrap();
    let message = LinkMessage::from_event("left", &event, &ia, ra.state(), &[], None, true)
        .unwrap()
        .unwrap();
    let update = message
        .resolve(&ib, &[], None, MissingMatch::Reject)
        .unwrap();
    let out = rb
        .dispatch(
            &b,
            rb.request(&b, update.action.clone(), update.origin.clone()),
        )
        .unwrap();
    assert_eq!(rb.state().selection().len(), 1);
    let t = rb.state().selection().first().unwrap();
    assert_eq!(t.layer, LayerId::new(99));
    assert!(matches!(t.identity,TargetIdentity::Source {key,..} if key.get()==9007199254741001));
    assert!(
        LinkMessage::from_event(
            "right",
            out.event.as_ref().unwrap(),
            &ib,
            rb.state(),
            &[],
            None,
            true
        )
        .unwrap()
        .is_none()
    );
    assert!(
        !rb.dispatch(&b, rb.request(&b, update.action, update.origin))
            .unwrap()
            .outcome
            .changed
    );
    assert_eq!(
        ia.presented()
            .prepared()
            .source()
            .get()
            .unwrap()
            .dataset(DatasetId::new(1))
            .unwrap()
            .len(),
        2
    );
    let mut missing = message.clone();
    missing.selection.as_mut().unwrap()[0].epoch = SourceEpoch::new(9);
    assert!(
        missing
            .resolve(&ib, &[], None, MissingMatch::Reject)
            .is_err()
    );
    let omitted = missing
        .resolve(&ib, &[], None, MissingMatch::ReportAndOmit)
        .unwrap();
    assert_eq!(omitted.unmatched.len(), 1);
    assert!(
        matches!(omitted.action,ChartAction::Synchronize {selection:Some(v),..} if v.is_empty())
    );
}
#[test]
fn linked_windows_use_semantic_values_and_preserve_unrelated_axes() {
    let a = definition(1);
    let b = definition(2);
    let mut ra = ActionReducer::default();
    let mut rb = ActionReducer::default();
    let _ = inspector(&a, &mut ra);
    let windows = [(ScaleId::new(0), AxisWindow::Numeric(2., 8.))]
        .into_iter()
        .collect();
    let event = send(&mut ra, &a, ChartAction::SetAxisWindows(windows))
        .event
        .unwrap();
    let old = Inspector::new(scene(&a, &ChartState::default(), 400.), 4., 16).unwrap();
    assert!(
        LinkMessage::from_event(
            "left",
            &event,
            &old,
            ra.state(),
            &[ScaleId::new(0)],
            None,
            false
        )
        .is_err()
    );
    let ia = inspector(&a, &mut ra);
    let ib = inspector(&b, &mut rb);
    let m = LinkMessage::from_event(
        "left",
        &event,
        &ia,
        ra.state(),
        &[ScaleId::new(0)],
        None,
        false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(m.windows[0].window, AxisWindow::Numeric(2., 8.));
    send(
        &mut rb,
        &b,
        ChartAction::SetAxisWindows(
            [(ScaleId::new(1), AxisWindow::Numeric(1., 9.))]
                .into_iter()
                .collect(),
        ),
    );
    let u = m
        .resolve(
            &ib,
            &[AxisLink {
                source: ScaleId::new(0),
                destination: ScaleId::new(0),
            }],
            None,
            MissingMatch::Reject,
        )
        .unwrap();
    rb.dispatch(&b, rb.request(&b, u.action.clone(), u.origin.clone()))
        .unwrap();
    assert_eq!(
        rb.state().axis_windows()[&ScaleId::new(1)],
        AxisWindow::Numeric(1., 9.)
    );
    send(
        &mut rb,
        &b,
        ChartAction::SetAxisWindows(
            [
                (ScaleId::new(0), AxisWindow::Numeric(2., 8.)),
                (ScaleId::new(1), AxisWindow::Numeric(3., 7.)),
            ]
            .into_iter()
            .collect(),
        ),
    );
    assert!(
        !rb.dispatch(&b, rb.request(&b, u.action, u.origin))
            .unwrap()
            .outcome
            .changed
    );
    let mut bad = m.clone();
    bad.windows[0].meaning = AxisMeaning::Numeric("other units".into());
    assert_eq!(
        bad.resolve(
            &ib,
            &[AxisLink {
                source: ScaleId::new(0),
                destination: ScaleId::new(0)
            }],
            None,
            MissingMatch::Reject
        )
        .unwrap_err()
        .code,
        DiagnosticCode::SchemaConflict
    );
}
#[test]
fn threshold_translation_snaps_both_endpoints_and_commits_one_undo_command() {
    let d = definition(1);
    let mut r = ActionReducer::default();
    let i = inspector(&d, &mut r);
    let editor = AnnotationEditor::new(
        i.presented().clone(),
        "threshold",
        AnnotationPart::Translate,
        EditConstraints {
            horizontal: false,
            vertical: true,
            y: Some(ValueConstraint::Number {
                bounds: Some([1., 8.]),
                step: Some(1.),
                origin: 0.,
            }),
            ..EditConstraints::default()
        },
    )
    .unwrap();
    let start = i.presented().scene().stamp();
    let id = r.next_gesture_id().unwrap();
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id,
            kind: GestureKind::Annotation("threshold".into()),
        },
    );
    for _ in 0..20 {
        // y is reversed: -33 destination units = +1.65 data units, which snaps to six.
        let a = editor.preview(start, 120., -33.).unwrap();
        assert!(matches!(
            a.anchor,
            Anchor::Data {
                x: ScaleValue::Number(1.),
                y: ScaleValue::Number(6.),
                ..
            }
        ));
        assert!(matches!(
            a.callout,
            Some(Anchor::Data {
                x: ScaleValue::Number(8.),
                y: ScaleValue::Number(6.),
                ..
            })
        ));
        send(
            &mut r,
            &d,
            ChartAction::PreviewGesture {
                id,
                preview: GesturePreview::Annotation(Box::new(a)),
            },
        );
    }
    assert_eq!(r.history_lengths(), (0, 0));
    send(&mut r, &d, ChartAction::CommitGesture { id });
    assert_eq!(r.history_lengths(), (1, 0));
    send(&mut r, &d, ChartAction::Undo);
    assert_eq!(
        r.state().annotations(&d),
        d.figure.as_ref().unwrap().annotations
    );
    send(&mut r, &d, ChartAction::Redo);
    assert!(matches!(
        r.state().annotations(&d)[0].anchor,
        Anchor::Data {
            y: ScaleValue::Number(6.),
            ..
        }
    ));
    assert!(matches!(
        d.figure.as_ref().unwrap().annotations[0].anchor,
        Anchor::Data {
            y: ScaleValue::Number(4.),
            ..
        }
    ));
}
#[test]
fn range_handles_reject_crossing_and_pinned_edits_ignore_resize_then_cancel() {
    let d = definition(1);
    let mut r = ActionReducer::default();
    let i = inspector(&d, &mut r);
    let e = AnnotationEditor::new(
        i.presented().clone(),
        "threshold",
        AnnotationPart::Callout,
        EditConstraints {
            horizontal: true,
            vertical: false,
            preserve_order: Some(true),
            ..EditConstraints::default()
        },
    )
    .unwrap();
    assert!(e.preview(i.presented().scene().stamp(), -300., 0.).is_err());
    let id = r.next_gesture_id().unwrap();
    send(
        &mut r,
        &d,
        ChartAction::BeginGesture {
            id,
            kind: GestureKind::Annotation("threshold".into()),
        },
    );
    r.present(scene(&d, r.state(), 800.));
    let a = e.preview(i.presented().scene().stamp(), -40., 0.).unwrap();
    assert!(matches!(
        a.callout,
        Some(Anchor::Data {
            x: ScaleValue::Number(7.),
            ..
        })
    ));
    send(
        &mut r,
        &d,
        ChartAction::PreviewGesture {
            id,
            preview: GesturePreview::Annotation(Box::new(a)),
        },
    );
    send(
        &mut r,
        &d,
        ChartAction::CancelGesture(CancelReason::FocusLost),
    );
    assert_eq!(
        r.state().annotations(&d),
        d.figure.as_ref().unwrap().annotations
    );
    assert_eq!(r.history_lengths(), (0, 0));
    assert!(
        AnnotationEditor::new(
            i.presented().clone(),
            "absent",
            AnnotationPart::Anchor,
            EditConstraints::default()
        )
        .is_err()
    );
    assert!(
        AnnotationEditor::new(
            i.presented().clone(),
            "threshold",
            AnnotationPart::Anchor,
            EditConstraints {
                x: Some(ValueConstraint::Number {
                    bounds: None,
                    step: Some(f64::NAN),
                    origin: 0.
                }),
                ..EditConstraints::default()
            }
        )
        .is_err()
    );
}
#[test]
fn accessible_page_preserves_exact_keys_values_and_bounded_pagination() {
    let d = definition(1);
    let mut r = ActionReducer::default();
    let i = inspector(&d, &mut r);
    let p = i.accessible_page(r.state(), 0, 1).unwrap();
    assert_eq!(p.total, 2);
    assert_eq!(p.targets.len(), 1);
    assert!(p.targets[0].description.contains("9007199254741001"));
    assert_eq!(p.targets[0].cells[1].field, "Value");
    assert_eq!(p.targets[0].cells[1].value, "3");
    assert_eq!(p.targets[0].omitted_fields, 0);
    let target = p.targets[0].target.clone();
    send(&mut r, &d, ChartAction::SetFocus(Some(target.clone())));
    send(
        &mut r,
        &d,
        ChartAction::Select {
            change: SelectionChange::Add,
            targets: vec![target],
        },
    );
    let p = i.accessible_page(r.state(), 0, 1).unwrap();
    assert!(p.targets[0].selected && p.targets[0].focused);
    assert_eq!(
        i.accessible_page(r.state(), 1, 1).unwrap().targets[0].cells[1].value,
        "7"
    );
    assert!(i.accessible_page(r.state(), 3, 1).is_err());
    assert!(i.accessible_page(r.state(), 0, 257).is_err());
}
#[test]
fn time_constraints_roundtrip_as_decimal_strings_without_integer_narrowing() {
    let c = ValueConstraint::Timestamp {
        bounds: Some([9007199254741001, 9007199254741101]),
        step: Some(10),
        origin: 9007199254741001,
    };
    let wire = serde_json::to_value(&c).unwrap();
    assert_eq!(wire["Timestamp"]["origin"], "9007199254741001");
    assert_eq!(wire["Timestamp"]["bounds"][0], "9007199254741001");
    assert_eq!(wire["Timestamp"]["step"], "10");
    assert_eq!(serde_json::from_value::<ValueConstraint>(wire).unwrap(), c);
}
fn portable_scene(case: &serde_json::Value) -> Arc<LaidOutChart> {
    let mut session =
        chart_core::portable::Session::new(&case["chart"].to_string(), &case["data"].to_string())
            .unwrap();
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.padding = 0.;
    Arc::new(layout(session.prepare().unwrap(), &r, &Metrics).unwrap())
}
#[test]
fn exact_timestamp_and_category_editing_use_presented_typed_coordinates() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../fixtures/interaction/cases.json")).unwrap();
    let mut time = cases[4].clone();
    let origin = 9007199254741001i64;
    time["data"]["datasets"][0]["batch"]["fields"][0]["kind"]["Timestamp"]["unit"] =
        serde_json::json!("Nanoseconds");
    time["data"]["datasets"][0]["batch"]["columns"][0]["values"]["Timestamp"] = serde_json::json!(
        (0..4)
            .map(|i| (origin + i * 100).to_string())
            .collect::<Vec<_>>()
    );
    let mut annotation = definition(1).figure.unwrap().annotations.remove(0);
    annotation.callout = None;
    if let Anchor::Data { x, .. } = &mut annotation.anchor {
        *x = ScaleValue::Timestamp {
            value: origin + 100,
            unit: TimeUnit::Nanoseconds,
        };
    }
    time["chart"]["definition"]["figure"] = serde_json::to_value(FigureComposition {
        annotations: vec![annotation.clone()],
        ..FigureComposition::default()
    })
    .unwrap();
    let chart = portable_scene(&time);
    let stamp = chart.scene().stamp();
    let e = AnnotationEditor::new(
        chart,
        "threshold",
        AnnotationPart::Anchor,
        EditConstraints {
            horizontal: true,
            vertical: false,
            x: Some(ValueConstraint::Timestamp {
                bounds: Some([origin, origin + 300]),
                step: Some(20),
                origin,
            }),
            ..EditConstraints::default()
        },
    )
    .unwrap();
    for a in [
        e.preview(stamp, 20., 0.).unwrap(),
        e.nudge(stamp, true, true, 1).unwrap(),
    ] {
        assert!(
            matches!(a.anchor,Anchor::Data {x:ScaleValue::Timestamp {value,unit:TimeUnit::Nanoseconds},..} if value==origin+120)
        );
    }
    let mut category = cases[3].clone();
    if let Anchor::Data { x, y, .. } = &mut annotation.anchor {
        *x = ScaleValue::Category("Alpha".into());
        *y = ScaleValue::Number(2.);
    }
    category["chart"]["definition"]["figure"] = serde_json::to_value(FigureComposition {
        annotations: vec![annotation],
        ..FigureComposition::default()
    })
    .unwrap();
    let chart = portable_scene(&category);
    let stamp = chart.scene().stamp();
    let e = AnnotationEditor::new(
        chart,
        "threshold",
        AnnotationPart::Anchor,
        EditConstraints {
            horizontal: true,
            vertical: false,
            x: Some(ValueConstraint::Category(vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
            ])),
            ..EditConstraints::default()
        },
    )
    .unwrap();
    assert!(
        matches!(e.nudge(stamp,true,true,1).unwrap().anchor,Anchor::Data {x:ScaleValue::Category(v),..} if v=="Beta")
    );
}
#[test]
fn aggregate_links_and_descriptions_preserve_membership_revision() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../fixtures/statistics/portable-cases.json"
    ))
    .unwrap();
    let chart = portable_scene(&cases[0]);
    let i = Inspector::new(chart, 5., 16).unwrap();
    let hit = i
        .semantic_targets()
        .find(|h| matches!(h.target, chart_core::provenance::Target::Aggregate { .. }))
        .unwrap();
    let epoch = i.presented().prepared().source().get().unwrap().epoch();
    let target = LinkedTarget::from_hit(hit, epoch);
    assert_eq!(target.inputs.len(), 1);
    let description = i
        .describe_target(hit, i.presented().prepared().state())
        .unwrap();
    assert!(description.description.starts_with("Aggregate"));
    assert!(description.member_count.is_some());
    let mut message = LinkMessage {
        origin: "histogram".into(),
        revision: Revision::new(1),
        windows: vec![],
        selection: Some(vec![target]),
    };
    assert!(message.resolve(&i, &[], None, MissingMatch::Reject).is_ok());
    message.selection.as_mut().unwrap()[0].inputs[0].revision = Revision::new(99);
    assert!(
        message
            .resolve(&i, &[], None, MissingMatch::Reject)
            .is_err()
    );
    assert_eq!(
        message
            .resolve(&i, &[], None, MissingMatch::ReportAndOmit)
            .unwrap()
            .unmatched
            .len(),
        1
    );
}
#[test]
fn threshold_connector_uses_authored_endpoints_independent_of_label_offset() {
    let mut d = definition(1);
    d.figure.as_mut().unwrap().annotations[0].offset = [35., -40.];
    let chart = scene(&d, &ChartState::default(), 400.);
    let line = chart
        .scene()
        .items()
        .iter()
        .find_map(|item| match item.primitive {
            chart_core::scene::Primitive::Rule { from, to, .. }
                if from.x() == 40. && to.x() == 320. =>
            {
                Some((from, to))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(line.0.y(), 120.);
    assert_eq!(line.1.y(), 120.);
    let e = AnnotationEditor::new(
        chart.clone(),
        "threshold",
        AnnotationPart::Translate,
        EditConstraints {
            horizontal: false,
            vertical: true,
            y: Some(ValueConstraint::Number {
                bounds: Some([0., 10.]),
                step: Some(1.),
                origin: 0.,
            }),
            ..EditConstraints::default()
        },
    )
    .unwrap();
    assert!(matches!(
        e.nudge(chart.scene().stamp(), false, true, 1)
            .unwrap()
            .anchor,
        Anchor::Data {
            y: ScaleValue::Number(3.),
            ..
        }
    ));
    assert!(matches!(
        e.nudge(chart.scene().stamp(), false, false, 1)
            .unwrap()
            .anchor,
        Anchor::Data {
            y: ScaleValue::Number(5.),
            ..
        }
    ));
}
#[test]
fn linked_selection_survives_viewport_clipping_and_hidden_receiving_layers() {
    let d = definition(1);
    let other = definition(99);
    let mut a = ActionReducer::default();
    let mut b = ActionReducer::default();
    let original = inspector(&d, &mut a);
    let mark = MarkTarget::from_inspected(
        original.semantic_targets().next().unwrap(),
        SourceEpoch::new(7),
    );
    send(
        &mut a,
        &d,
        ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![mark],
        },
    );
    let event = send(
        &mut a,
        &d,
        ChartAction::SetAxisWindows(
            [(ScaleId::new(0), AxisWindow::Numeric(6., 10.))]
                .into_iter()
                .collect(),
        ),
    )
    .event
    .unwrap();
    let clipped = inspector(&d, &mut a);
    assert_eq!(clipped.semantic_targets().len(), 1);
    let message = LinkMessage::from_event(
        "left",
        &event,
        &clipped,
        a.state(),
        &[ScaleId::new(0)],
        None,
        true,
    )
    .unwrap()
    .unwrap();
    send(
        &mut b,
        &other,
        ChartAction::SetLayerVisible {
            layer: LayerId::new(99),
            visible: false,
        },
    );
    let hidden = inspector(&other, &mut b);
    assert_eq!(hidden.semantic_targets().len(), 0);
    let update = message
        .resolve(
            &hidden,
            &[AxisLink {
                source: ScaleId::new(0),
                destination: ScaleId::new(0),
            }],
            None,
            MissingMatch::Reject,
        )
        .unwrap();
    b.dispatch(&other, b.request(&other, update.action, update.origin))
        .unwrap();
    assert_eq!(b.state().selection().len(), 1);
    assert!(
        matches!(b.state().selection().first().unwrap().identity,TargetIdentity::Source {key,..} if key.get()==9007199254741001)
    );
}
