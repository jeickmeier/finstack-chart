//! Exact strip context, malformed output, budgets, portable boundary and replay.
use chart_core::{grammar::*, layout::*, prelude::*, scene::*, services::*, *};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 4., 8., 2.)
    }
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::typography::ShapedRun> {
        Ok(chart_core::typography::ShapedRun {
            text: r.run.text.clone(),
            font: *r.default_font,
            font_size: r.font_size * r.run.size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: chart_core::services::TextMetrics::new(
                r.run.text.chars().count() as f64 * 6.,
                9.,
                3.,
            )?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
}
fn request(units: Units) -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 900., 600.).unwrap(),
        units,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn author(operation: &str, mode: &str) -> Plot {
    plot(
        Data::columns()
            .column("x", [1., 2., 3.])
            .column("r", [Some("a"), None, Some("a")])
            .column("c", [1_i64, 2, 2])
            .build()
            .unwrap(),
    )
    .extensions(chart_extension_example::registry().unwrap())
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y(1.))
    .layer(points())
    .facet(facet_grid("r", "c").reference(FacetPolicy {
        margins: vec![0, 1],
        labeller: FacetLabeller {
            registered: Some(FacetLabelOperation {
                operation: OperationRef::new(operation, Revision::new(1)),
                parameters: serde_json::json!(mode),
            }),
            ..Default::default()
        },
        ..Default::default()
    }))
    .build()
    .unwrap()
}
#[test]
fn exact_context_replay_and_native_only() {
    let p = author(chart_extension_example::facet_labels::LABELS, "context");
    let wire = p.to_json().unwrap();
    assert!(Plot::from_json(&wire).is_err());
    let replay =
        Plot::from_json_with_extensions(&wire, chart_extension_example::registry().unwrap())
            .unwrap();
    let a = layout(
        p.chart().unwrap().prepare().unwrap(),
        &request(Units::Points),
        &Metrics,
    )
    .unwrap();
    let b = layout(
        replay.chart().unwrap().prepare().unwrap(),
        &request(Units::Points),
        &Metrics,
    )
    .unwrap();
    assert_eq!(a.scene().items(), b.scene().items());
    let labels = a
        .scene()
        .items()
        .iter()
        .filter_map(|i| match &i.primitive {
            Primitive::Text { text, .. } => Some(text.as_str()),
            Primitive::GlyphRun { run, .. } => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for expected in [
        "missing:NA",
        "margin:All",
        "int:1",
        "text:a",
        "Top grid",
        "Right grid",
    ] {
        assert!(
            labels.iter().any(|s| s.contains(expected)),
            "{expected}: {labels:?}"
        );
    }
    let native = author(
        chart_extension_example::facet_labels::NATIVE_LABELS,
        "context",
    );
    assert!(native.to_json().is_err());
    assert!(
        layout(
            native.chart().unwrap().prepare().unwrap(),
            &request(Units::Points),
            &Metrics
        )
        .is_err()
    );
    assert!(
        layout(
            native.chart().unwrap().prepare().unwrap(),
            &request(Units::LogicalPixels),
            &Metrics
        )
        .is_ok()
    );
}
#[test]
fn malformed_and_budget_fail_before_strip_paint() {
    for mode in ["short", "oversize"] {
        let p = author(chart_extension_example::facet_labels::LABELS, mode);
        let mut r = request(Units::Points);
        r.limits.max_text_bytes = 4096;
        assert!(
            layout(p.chart().unwrap().prepare().unwrap(), &r, &Metrics).is_err(),
            "{mode}"
        );
    }
    let p = author(chart_extension_example::facet_labels::LABELS, "missing");
    assert!(
        layout(
            p.chart().unwrap().prepare().unwrap(),
            &request(Units::Points),
            &Metrics
        )
        .is_ok()
    );
}
