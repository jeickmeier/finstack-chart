//! FIX-GG18 integrated prepared/model/capture boundaries.
#[path = "../../../examples/common/ggplot_integration_controls.rs"]
mod fixtures;
use chart_core::{
    grammar::*,
    prelude::*,
    state::{ChartAction, Viewport},
};
use chart_export::*;
fn model_values(
    chart: &PreparedChart,
) -> Vec<(
    Vec<chart_core::grammar::StatValue>,
    u64,
    Vec<chart_core::RowKey>,
)> {
    chart
        .panels()
        .iter()
        .flat_map(|p| p.chart.layers().iter())
        .filter_map(|l| match l.table().rows() {
            PreparedRows::Statistical(rows) => Some(rows.iter()),
            _ => None,
        })
        .flatten()
        .map(|r| (r.values.clone(), r.count, r.members.to_vec()))
        .collect()
}
#[test]
fn source_filter_changes_model_population_while_coordinate_and_state_zoom_do_not() {
    assert_eq!(fixtures::CASES, 4);
    let p = fixtures::author(0).unwrap();
    let mut chart = p.chart().unwrap();
    let original = chart.prepare().unwrap();
    let values = model_values(&original);
    assert!(!values.is_empty());
    chart
        .act(ChartAction::SetViewport(Viewport {
            x: Some((4., 11.)),
            y: None,
        }))
        .unwrap();
    assert_eq!(values, model_values(&chart.prepare().unwrap()));
    assert_eq!(
        values,
        model_values(
            &fixtures::author(2)
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap()
        )
    );
    let filtered = model_values(
        &fixtures::author(1)
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap(),
    );
    assert_ne!(values, filtered);
    assert!(filtered.iter().all(|r| r.1 == 8));
    assert!(values.iter().all(|r| r.1 == 12));
}
#[test]
fn integrated_presented_current_view_interaction_matrix_survives_disposal() {
    use chart_core::{
        inspection::Inspector,
        state::{ChartAction, InteractionCapture, MarkTarget, SelectionChange, Viewport},
        transaction::CommitOutcome,
    };
    let original = fixtures::author(0).unwrap();
    let mut chart = original.chart().unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let page = PageSize::points(400., 300.).unwrap();
    chart
        .act(ChartAction::SetViewport(Viewport {
            x: Some((1., 2.5)),
            y: None,
        }))
        .unwrap();
    let initial = output
        .live_request(
            &chart,
            export_options(page).basis(CaptureBasis::Current).layout(
                layout_options().host_style(style().foreground(chart_core::theme::rgb(20, 40, 60))),
            ),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let laid_out = initial.layout().clone();
    let policy = initial.metadata().profile.layout.clone();
    chart
        .acknowledge_paint_with_layout(laid_out.clone(), &chart.state().clone(), &policy)
        .unwrap();
    let inspector = Inspector::new(laid_out.clone(), 10., 32).unwrap();
    let target = MarkTarget::from_inspected(
        inspector
            .semantic_targets()
            .find(|t| matches!(t.target, chart_core::provenance::Target::Source(_)))
            .unwrap(),
        chart.source().get().unwrap().epoch(),
    );
    chart
        .act(ChartAction::Select {
            change: SelectionChange::Replace,
            targets: vec![target.clone()],
        })
        .unwrap();
    chart.act(ChartAction::SetHover(vec![target])).unwrap();
    chart
        .acknowledge_paint_with_layout(laid_out.clone(), &chart.state().clone(), &policy)
        .unwrap();
    let painted_revision = chart.source().get().unwrap().revision();
    let incoming = Data::columns()
        .column("x", [12.])
        .column("y", [20.])
        .column("g", ["a"])
        .keys([9_007_199_254_741_017])
        .build()
        .unwrap();
    let tx = chart
        .transaction()
        .unwrap()
        .append("data", incoming)
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    let edited = original
        .edit()
        .title(title("Pending title"))
        .build()
        .unwrap();
    chart
        .apply_plot(&edited, chart.definition().revision)
        .unwrap();
    chart
        .act(ChartAction::SetViewport(Viewport {
            x: Some((2., 4.)),
            y: None,
        }))
        .unwrap();
    let current_revision = chart.source().get().unwrap().revision();
    assert!(current_revision > painted_revision);
    let mut requests = vec![];
    for basis in [CaptureBasis::Presented, CaptureBasis::Current] {
        for view in [ViewMode::VisibleView, ViewMode::FullDomain] {
            for include in [false, true] {
                let interaction = if include {
                    InteractionCapture::ALL
                } else {
                    InteractionCapture::default()
                };
                let request = output
                    .live_request(
                        &chart,
                        export_options(page)
                            .basis(basis)
                            .view(view)
                            .interaction(interaction),
                    )
                    .unwrap();
                let painted = basis == CaptureBasis::Presented;
                assert_eq!(
                    request.source().get().unwrap().revision(),
                    if painted {
                        painted_revision
                    } else {
                        current_revision
                    }
                );
                assert_eq!(
                    request.manifest().unwrap()["origin_scene"].is_null(),
                    !painted
                );
                assert_eq!(
                    request.manifest().unwrap()["origin_layout"].is_null(),
                    !painted
                );
                if painted {
                    assert_eq!(request.profile().layout.host_theme, policy.host_theme);
                }
                requests.push((basis, view, include, request));
            }
        }
    }
    chart.dispose().unwrap();
    for (basis, view, include, request) in requests {
        let snapshot = request.prepare().unwrap();
        let captured = &snapshot.metadata().captured_state;
        let effective = &snapshot.metadata().effective_state;
        assert_eq!(captured.selection().len(), 1);
        assert_eq!(effective.selection().len(), usize::from(include));
        assert_eq!(effective.hover().len(), usize::from(include));
        let painted = basis == CaptureBasis::Presented;
        let expected_view = if painted { (1., 2.5) } else { (2., 4.) };
        assert_eq!(captured.viewport().x, Some(expected_view));
        assert_eq!(
            effective.viewport().x,
            if view == ViewMode::VisibleView {
                Some(expected_view)
            } else {
                None
            }
        );
        assert_eq!(snapshot.layout().prepared().panels().len(), 2);
        assert!(
            snapshot
                .layout()
                .prepared()
                .semantic_targets()
                .any(|t| matches!(t.target, chart_core::provenance::Target::Derived { .. }))
        );
        let svg = String::from_utf8(snapshot.export(Format::Svg).unwrap().bytes).unwrap();
        assert_eq!(svg.contains("Pending title"), !painted);
    }
}

type Row = (u64, f64, f64, &'static str);
fn rows_data(rows: &[Row]) -> Data {
    Data::columns()
        .column("x", rows.iter().map(|r| r.1).collect::<Vec<_>>())
        .column("y", rows.iter().map(|r| r.2).collect::<Vec<_>>())
        .column("g", rows.iter().map(|r| r.3).collect::<Vec<_>>())
        .keys(rows.iter().map(|r| r.0))
        .build()
        .unwrap()
}
#[test]
fn global_models_recompute_exactly_through_updates_and_retained_capture() {
    use chart_core::transaction::CommitOutcome;
    let first = 9_007_199_254_740_993;
    let mut rows = (0..24)
        .map(|i| {
            (
                first + i,
                (i / 2) as f64,
                2. + 0.3 * (i / 2) as f64 + ((i / 2) % 3) as f64 + (i % 2) as f64,
                if i % 2 == 0 { "a" } else { "b" },
            )
        })
        .collect::<Vec<_>>();
    let data = rows_data(&rows);
    let p = fixtures::from_data(data.clone(), 0).unwrap();
    let mut chart = p.chart().unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let options =
        export_options(PageSize::points(600., 360.).unwrap()).basis(CaptureBasis::Current);
    let old = output.live_request(&chart, options.clone()).unwrap();
    let old_bytes = old.prepare().unwrap().export(Format::Svg).unwrap().bytes;
    let compare = |chart: &mut chart_core::runtime::Chart, rows: &[Row]| {
        let batch = fixtures::from_data(rows_data(rows), 0).unwrap();
        let expected = batch.chart().unwrap().prepare().unwrap();
        assert_eq!(
            model_values(&chart.prepare().unwrap()),
            model_values(&expected)
        );
        let a = output
            .live_request(chart, options.clone())
            .unwrap()
            .prepare()
            .unwrap();
        let b = output
            .request(&batch, options.clone())
            .unwrap()
            .prepare()
            .unwrap();
        assert_eq!(
            a.scene()
                .items()
                .iter()
                .map(|i| &i.primitive)
                .collect::<Vec<_>>(),
            b.scene()
                .items()
                .iter()
                .map(|i| &i.primitive)
                .collect::<Vec<_>>()
        );
    };
    let new = [(first + 24, 12., 9., "a"), (first + 25, 12., 10., "b")];
    let tx = chart
        .transaction()
        .unwrap()
        .append(&data, rows_data(&new))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    rows.extend(new);
    compare(&mut chart, &rows);
    rows[0].2 = 20.;
    let tx = chart
        .transaction()
        .unwrap()
        .upsert(&data, rows_data(&rows[..1]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    compare(&mut chart, &rows);
    let tx = chart
        .transaction()
        .unwrap()
        .remove(&data, [first + 1])
        .retain_count(&data, Some(20))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    rows.retain(|r| r.0 != first + 1);
    rows.drain(..rows.len() - 20);
    compare(&mut chart, &rows);
    let before = model_values(&chart.prepare().unwrap());
    let tx = chart
        .transaction()
        .unwrap()
        .append(&data, rows_data(&rows[..1]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Rejected(_)
    ));
    assert_eq!(before, model_values(&chart.prepare().unwrap()));
    chart.dispose().unwrap();
    assert_eq!(
        old_bytes,
        old.prepare().unwrap().export(Format::Svg).unwrap().bytes
    );
}

#[test]
fn aggregate_and_model_selection_resolve_exact_presented_provenance() {
    use chart_core::{
        inspection::Inspector,
        provenance::{ResolvedTarget, Target},
        state::{InteractionCapture, MarkTarget, SelectionChange},
    };
    let plot = fixtures::author(0)
        .unwrap()
        .edit()
        .layer(
            "summary",
            points().stat(
                summary()
                    .x("x")
                    .y("y")
                    .group("g")
                    .summary_helper(SummaryHelper::default()),
            ),
        )
        .build()
        .unwrap();
    let mut chart = plot.chart().unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let options = export_options(PageSize::points(600., 360.).unwrap())
        .basis(CaptureBasis::Current)
        .interaction(InteractionCapture::ALL);
    let initial = output
        .live_request(&chart, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let layout = initial.layout().clone();
    chart
        .acknowledge_paint_with_layout(
            layout.clone(),
            &chart.state().clone(),
            &initial.metadata().profile.layout,
        )
        .unwrap();
    let inspector = Inspector::new(layout.clone(), 10., 32).unwrap();
    let aggregate = inspector
        .semantic_targets()
        .find(|t| matches!(t.target, Target::Aggregate { .. }))
        .expect("summary exposes aggregate selection");
    let derived = inspector
        .semantic_targets()
        .find(|t| matches!(t.target, Target::Derived { .. }))
        .expect("model exposes derived path selection");
    let source = chart.source();
    let source = source.get().unwrap();
    match aggregate.target.resolve(source).unwrap() {
        ResolvedTarget::Aggregate { members, .. } => {
            assert_eq!(members.len(), 1);
        }
        other => panic!("unexpected {other:?}"),
    };
    match derived.target.resolve(source).unwrap() {
        ResolvedTarget::Derived { inputs, .. } => {
            assert_eq!(inputs.len(), 1);
            assert_eq!(inputs[0].len(), 24);
        }
        other => panic!("unexpected {other:?}"),
    };
    let targets = vec![
        MarkTarget::from_inspected(aggregate, source.epoch()),
        MarkTarget::from_inspected(derived, source.epoch()),
    ];
    assert!(
        targets
            .iter()
            .all(|t| inspector.target(t).unwrap().is_some())
    );
    assert!(!inspector.highlights(&targets).unwrap().is_empty());
    chart
        .act(ChartAction::Select {
            change: SelectionChange::Replace,
            targets: targets.clone(),
        })
        .unwrap();
    chart.act(ChartAction::SetHover(targets.clone())).unwrap();
    let selected = output.live_request(&chart, options).unwrap();
    chart.dispose().unwrap();
    let selected = selected.prepare().unwrap();
    assert_eq!(selected.metadata().effective_state.selection().len(), 2);
    assert_eq!(selected.metadata().effective_state.hover().len(), 2);
    assert!(!selected.export(Format::Svg).unwrap().bytes.is_empty());
    assert_eq!(
        model_values(layout.prepared()),
        model_values(selected.layout().prepared())
    );
}

#[test]
fn alternate_profiles_and_registered_geographic_models_retain_replay_identities() {
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    for profile in [Profile::LibraryV1, Profile::Ggplot2_4_0_3] {
        for mode in [0, 3] {
            let original =
                fixtures::from_data_profile(fixtures::data().unwrap(), mode, profile).unwrap();
            let restored = Plot::from_json_with_extensions(
                &original.to_json().unwrap(),
                chart_extension_example::registry().unwrap(),
            )
            .unwrap();
            assert_eq!(restored.profile(), profile);
            assert_eq!(original.definition(), restored.definition());
            let options = export_options(PageSize::points(600., 360.).unwrap());
            let a = output
                .request(&original, options.clone())
                .unwrap()
                .prepare()
                .unwrap();
            let b = output
                .request(&restored, options)
                .unwrap()
                .prepare()
                .unwrap();
            assert_eq!(a.scene_json().unwrap(), b.scene_json().unwrap());
            assert_eq!(
                model_values(a.layout().prepared()),
                model_values(b.layout().prepared())
            );
            assert_eq!(a.layout().prepared().panels().len(), 2);
            if mode == 3 {
                assert!(a.layout().prepared().semantic_targets().any(|t|matches!(&t.target,chart_core::provenance::Target::Derived{model,..} if model.starts_with("chart.model/"))));
            }
        }
    }
}
