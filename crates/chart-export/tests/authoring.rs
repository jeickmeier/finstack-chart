//! FIX-AUTH02/06: actual primary authoring publication and independent capture choices.
use chart_core::prelude::*;
use chart_export::*;
fn output() -> Output {
    Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap()
}
fn plot_fixture() -> Plot {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [10., 12., 11.])
        .build()
        .unwrap();
    plot(data)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points())
        .title(title("Prices"))
        .subtitle(subtitle("Daily observations"))
        .x_axis(x_axis().label("Time"))
        .y_axis(y_axis().label("USD"))
        .caption(caption("Example caption"))
        .source_note(source_note("Example source"))
        .footnote(footnote("Example footnote"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()
        .unwrap()
}
#[test]
fn real_svg_pdf_png_preserve_component_text_geometry_and_physical_size() {
    let plot = plot_fixture();
    let output = output();
    let page = PageSize::millimeters(180., 120.).unwrap();
    let snapshot = output
        .request(&plot, export_options(page))
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(snapshot.layout().prepared().layers().len(), 2);
    let svg = String::from_utf8(snapshot.export(Format::Svg).unwrap().bytes).unwrap();
    for text in [
        "Prices",
        "Daily observations",
        "Time",
        "USD",
        "Example caption",
        "Example source",
        "Example footnote",
    ] {
        assert!(svg.contains(text), "missing {text}");
    }
    assert!(
        snapshot
            .export(Format::Pdf)
            .unwrap()
            .bytes
            .starts_with(b"%PDF-")
    );
    let png = snapshot.export(Format::Png).unwrap().bytes;
    let reader = png::Decoder::new(png.as_slice()).read_info().unwrap();
    assert_eq!((reader.info().width, reader.info().height), (2126, 1417));
    assert!((snapshot.scene().bounds().width() - 510.23622047244095).abs() < 1e-9);
}
#[test]
fn live_capture_requires_presentation_only_for_presented_basis() {
    let plot = plot_fixture();
    let chart = plot.chart().unwrap();
    let output = output();
    let options = export_options(PageSize::points(300., 200.).unwrap());
    assert_eq!(
        output
            .live_request(&chart, options.clone())
            .err()
            .unwrap()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    let request = output
        .live_request(&chart, options.basis(CaptureBasis::Current))
        .unwrap();
    assert_eq!(
        request.source().get().unwrap().revision(),
        chart.source().get().unwrap().revision()
    );
    assert!(request.manifest().unwrap()["origin_scene"].is_null());
    assert!(request.prepare().unwrap().export(Format::Svg).is_ok());
}

#[test]
fn presented_current_projection_and_interaction_captures_remain_independent_after_disposal() {
    use chart_core::{
        inspection::Inspector,
        state::{ChartAction, InteractionCapture, MarkTarget, SelectionChange, Viewport},
        transaction::CommitOutcome,
    };
    let original = plot_fixture();
    let mut chart = original.chart().unwrap();
    let output = output();
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
        inspector.semantic_targets().next().unwrap(),
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
        .column("x", [4.])
        .column("y", [20.])
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
        assert_eq!(
            snapshot.layout().prepared().layers()[1].marks().len(),
            if painted { 3 } else { 4 }
        );
        let svg = String::from_utf8(snapshot.export(Format::Svg).unwrap().bytes).unwrap();
        assert_eq!(svg.contains("Pending title"), !painted);
    }
}

#[test]
fn ordinary_and_faceted_legends_share_exported_labels_and_source_provenance() {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., 4., 6.])
        .column("series", ["Alpha", "Beta", "Alpha"])
        .column("panel", ["One", "One", "One"])
        .build()
        .unwrap();
    let output = output();
    for faceted in [false, true] {
        let mut label = labels().at(2., 4.).text("Peak");
        if faceted {
            label = label.panel(chart_core::grammar::PanelKey {
                values: vec![chart_core::grammar::GroupValue::Text("One".into())],
            });
        }
        let mut builder = plot(data.clone())
            .aes(aes().x("x").y("y").color("series"))
            .layer(points())
            .legend(legend().scale("series").title("Series"))
            .layer(label);
        if faceted {
            builder = builder.facet(facet_wrap("panel"));
        }
        let plot = builder.build().unwrap();
        let snapshot = output
            .request(&plot, export_options(PageSize::points(500., 300.).unwrap()))
            .unwrap()
            .prepare()
            .unwrap();
        let svg = String::from_utf8(snapshot.export(Format::Svg).unwrap().bytes).unwrap();
        for label in ["Series", "Alpha", "Beta", "Peak"] {
            assert!(svg.contains(label), "{faceted}: missing {label}");
        }
        assert_eq!(snapshot.scene().bounds().width(), 500.);
        let prepared = snapshot.layout().prepared();
        let layers = if faceted {
            prepared.panels()[0].chart.layers()
        } else {
            prepared.layers()
        };
        assert_eq!(layers[0].marks().len(), 3);
    }
}

#[test]
fn repeated_frozen_paints_preserve_scene_and_capture_policy_after_live_commit() {
    use chart_core::{
        Revision,
        state::{ChartAction, FollowMode},
    };
    use std::sync::Arc;
    let mut chart = plot_fixture().chart().unwrap();
    let output = output();
    let options = export_options(PageSize::points(400., 300.).unwrap());
    let initial = output
        .live_request(&chart, options.clone().basis(CaptureBasis::Current))
        .unwrap()
        .prepare()
        .unwrap();
    let scene = initial.layout().clone();
    let policy = initial.metadata().profile.layout.clone();
    chart
        .acknowledge_paint_with_layout(scene.clone(), &chart.state().clone(), &policy)
        .unwrap();
    chart
        .act(ChartAction::SetFollow(FollowMode::FreezePresentation))
        .unwrap();
    let tx = chart
        .transaction()
        .unwrap()
        .append(
            "data",
            Data::columns()
                .column("x", [4.])
                .column("y", [20.])
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    chart.apply_transaction(tx).unwrap();
    // Native paints can repeat after a button notification without resizing or compiling.
    for _ in 0..3 {
        chart
            .acknowledge_paint_with_layout(scene.clone(), &chart.state().clone(), &policy)
            .unwrap();
        assert!(Arc::ptr_eq(chart.reducer().frozen_scene().unwrap(), &scene));
        assert_eq!(
            serde_json::to_value(chart.painted_layout()).unwrap(),
            serde_json::to_value(&policy).unwrap()
        );
        let capture = output.live_request(&chart, options.clone()).unwrap();
        assert_eq!(
            capture.source().get().unwrap().revision(),
            Revision::INITIAL
        );
        assert_eq!(chart.source().get().unwrap().revision(), Revision::new(1));
    }
    let pending = output
        .live_request(&chart, options.basis(CaptureBasis::Current))
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(
        chart
            .acknowledge_paint(pending.layout().clone(), &chart.state().clone())
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    assert!(Arc::ptr_eq(chart.reducer().frozen_scene().unwrap(), &scene));
    assert_eq!(
        serde_json::to_value(chart.painted_layout()).unwrap(),
        serde_json::to_value(&policy).unwrap()
    );
}
