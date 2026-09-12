//! AXIS-06 publication samples use the core plan and preserve exact captured labels.
#[path = "../../../examples/common/axis_transition_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn output() -> Output {
    Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap()
}
fn options() -> ExportOptions {
    export_options(PageSize::points(500., 300.).unwrap()).dpi(72)
}
#[test]
fn sampled_text_outline_pdf_png_and_displayed_capture_are_coherent() {
    let output = output();
    let [a, b, c] = fixtures::sequence().unwrap();
    for mode in [TextMode::Preserve, TextMode::Outline] {
        let capture = |plot| {
            output
                .request(plot, options().text(mode))
                .unwrap()
                .prepare()
                .unwrap()
        };
        let a = capture(&a);
        let b = capture(&b);
        let c = capture(&c);
        let plan = b.guide_transition(&a).unwrap();
        let start = plan.sample(0.).unwrap();
        let mid = plan.sample(0.5).unwrap();
        let end = plan.sample(1.).unwrap();
        let expected = ["ZERO", "half", "ONE", "TWO"];
        assert_eq!(
            mid.layout().guide_presentation()[0]
                .frame
                .ticks
                .iter()
                .map(|t| t.label.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(end.scene().items(), b.scene().items());
        for (frame, alpha) in [(&start, 0.000001), (&mid, 0.5000005)] {
            let svg = frame.export(Format::Svg).unwrap();
            let svg = std::str::from_utf8(&svg.bytes).unwrap();
            let doc = usvg::roxmltree::Document::parse(svg).unwrap();
            let groups: Vec<_> = doc
                .descendants()
                .filter(|n| n.attribute("class") == Some("tick"))
                .collect();
            assert_eq!(groups.len(), 4);
            assert_eq!(
                groups[3]
                    .attribute("opacity")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                alpha
            );
            assert_eq!(groups[1].attribute("data-label"), Some("half"));
            assert_eq!(
                doc.descendants().filter(|n| n.has_tag_name("text")).count(),
                if mode == TextMode::Preserve { 4 } else { 0 }
            );
            assert!(
                frame
                    .export(Format::Pdf)
                    .unwrap()
                    .bytes
                    .starts_with(b"%PDF-")
            );
            assert!(
                frame
                    .export(Format::Png)
                    .unwrap()
                    .bytes
                    .starts_with(b"\x89PNG")
            );
            assert!(frame.metadata().manifest()["displayed"]["guides"].is_array());
        }
        let interrupted = c.guide_transition(&mid).unwrap().sample(0.).unwrap();
        assert_eq!(
            interrupted.layout().guide_presentation()[0]
                .frame
                .ticks
                .len(),
            5
        );
        let enter = interrupted.layout().guide_presentation()[0].frame.ticks[4].position;
        assert!((enter - 850.5).abs() < 1e-9);
        assert!(plan.sample(f64::INFINITY).is_err());
        // Prior sample remains immutable after further plans/exports.
        assert_eq!(mid.layout().guide_presentation()[0].frame.ticks.len(), 4);
    }
}
#[test]
fn displayed_live_capture_preserves_sample_and_rejects_reflow_and_stale_frames() {
    let output = output();
    let [a, b, _] = fixtures::sequence().unwrap();
    let mut runtime = chart_export::host::Runtime::new(&a).unwrap();
    let opts = chart_export::host::Options::new(500., 300., "pt").unwrap();
    let initial = runtime.present(&output, &opts).unwrap();
    assert!(
        runtime
            .apply_plot(&b, a.definition().revision.get())
            .unwrap()
    );
    let target = runtime.present(&output, &opts).unwrap();
    let mid = target
        .guide_transition(&initial)
        .unwrap()
        .sample(0.5)
        .unwrap();
    runtime.acknowledge_frame(&mid).unwrap();
    let displayed = opts.set("basis", r#"["Displayed"]"#).unwrap();
    let copy = runtime
        .request(&output, &displayed)
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(copy.scene().items(), mid.scene().items());
    assert_eq!(copy.guides_json().unwrap(), mid.guides_json().unwrap());
    assert!(runtime.acknowledge_frame(&initial).is_err());
    let resized = displayed.set("page", r#"[600,300,"pt"]"#).unwrap();
    assert!(runtime.request(&output, &resized).is_err());
}

#[test]
fn facet_and_inset_guides_keep_independent_join_scopes_and_targets() {
    use chart_core::{
        layout::{GuideFormatter, GuideProfile},
        prelude::*,
        scene::GuideRole,
    };
    let output = output();
    let axis = |end, values: Vec<f64>| {
        x_axis()
            .scale(scale_linear().domain(0., end))
            .guide_profile(GuideProfile::D3_3_0_0)
            .tick_values(Some(values.iter().copied().map(Into::into).collect()))
            .tick_format(Some(GuideFormatter::Labels(
                values.iter().map(|n| format!("v{n}")).collect(),
            )))
    };
    for facets in [false, true] {
        let layer = points();
        let handle = layer.handle().unwrap();
        let builder = plot(
            Data::columns()
                .column("x", [0., 1., 0., 1.])
                .column("y", [0., 1., 1., 0.])
                .column("group", categorical(["a", "a", "b", "b"]))
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .x_axis(axis(1., vec![0., 0.5, 1.]))
        .y_axis(y_axis().visible(false));
        let a = if facets {
            builder.facet(facet_wrap("group").columns(2))
        } else {
            builder.inset(
                inset()
                    .id("zoom")
                    .layer(handle)
                    .rectangle(0.55, 0.05, 0.4, 0.45),
            )
        }
        .build()
        .unwrap();
        let b = a.edit().x_axis(axis(2., vec![0., 1., 2.])).build().unwrap();
        let capture = |p| {
            output
                .request(
                    p,
                    export_options(PageSize::points(600., 400.).unwrap()).dpi(72),
                )
                .unwrap()
                .prepare()
                .unwrap()
        };
        let a = capture(&a);
        let b = capture(&b);
        let mid = b.guide_transition(&a).unwrap().sample(0.5).unwrap();
        let frames = mid.layout().guide_presentation();
        assert_eq!(frames.len(), 2);
        assert_ne!(frames[0].scope, frames[1].scope);
        for guide in &frames {
            assert_eq!(guide.frame.ticks.len(), 4);
            assert_eq!(
                guide
                    .frame
                    .ticks
                    .iter()
                    .map(|t| t.identity)
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3]
            );
        }
        let mut scopes = std::collections::BTreeMap::new();
        for (index, item) in mid.scene().items().iter().enumerate() {
            if let Some(g) = &item.guide
                && g.role == GuideRole::Line
            {
                scopes
                    .entry(g.scope.clone())
                    .or_insert_with(Vec::new)
                    .push(g.animation.unwrap().identity);
                let scene: serde_json::Value =
                    serde_json::from_str(&mid.scene_json().unwrap()).unwrap();
                assert!(scene["targets"][index].as_array().unwrap().is_empty());
            }
        }
        assert_eq!(scopes.len(), 2);
        assert!(scopes.values().all(|ids| ids == &[0, 1, 2, 3]));
        assert!(mid.export(Format::Svg).is_ok());
    }
}
