//! FIX-GG05: shared multi-aesthetic key composition, placement and immutable controls.
use chart_core::{
    Rect, ResourceId, Revision,
    grammar::{KeyGlyph, LayerLegend, LegendAesthetic, LegendOptions, LegendPosition},
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::{GuideRole, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.chars().count() as f64 * 6., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for a in &mut r.axes {
        a.visible = false;
    }
    r
}
fn author(position: LegendPosition, facet: &str, reverse: bool, hide: bool) -> Plot {
    let data = Data::columns()
        .column("x", vec![1., 2., 3., 4.])
        .column("g", vec!["A", "B", "C", "D"])
        .column("f", vec!["one", "one", "two", "two"])
        .build()
        .unwrap();
    let options = LegendOptions {
        position: Some(position),
        reverse: Some(reverse),
        ncol: Some(2),
        ..Default::default()
    };
    let mut b = plot(data)
        .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.).color("g").shape("g"))
        .layer(points().legend(LayerLegend {
            show: Some(!hide),
            key_glyph: KeyGlyph::Point,
            ..Default::default()
        }))
        .legend(
            legend()
                .aesthetic(LegendAesthetic::Color)
                .options(options.clone()),
        )
        .legend(legend().aesthetic(LegendAesthetic::Shape).options(options));
    if facet != "single" {
        b = b.facet(facet_wrap("f").collect_guides(facet == "collected"));
    }
    b.build().unwrap()
}
#[test]
fn key_groups_place_inside_and_on_every_side_and_roundtrip() {
    for position in [
        LegendPosition::Right,
        LegendPosition::Left,
        LegendPosition::Top,
        LegendPosition::Bottom,
        LegendPosition::Inside { x: 0.5, y: 0.5 },
    ] {
        for facet in ["single", "collected", "local"] {
            for reverse in [false, true] {
                let p = author(position.clone(), facet, reverse, false);
                let wire = p.to_json().unwrap();
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
                    69
                );
                let restored = Plot::from_json(&wire).unwrap();
                let a =
                    layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
                let b = layout(
                    restored.chart().unwrap().prepare().unwrap(),
                    &request(),
                    &Metrics,
                )
                .unwrap();
                assert_eq!(a.scene().items(), b.scene().items());
                assert_eq!(a.scene().wire_version(), 19);
                let key_items: Vec<_> = a
                    .scene()
                    .items()
                    .iter()
                    .filter(|i| {
                        i.guide
                            .as_ref()
                            .is_some_and(|g| g.role == GuideRole::LegendKey)
                    })
                    .collect();
                assert_eq!(
                    key_items.len(),
                    if facet == "local" { 8 } else { 4 },
                    "{position:?} {facet}"
                );
                let title_count = a
                    .scene()
                    .items()
                    .iter()
                    .filter(|i| {
                        i.guide
                            .as_ref()
                            .is_some_and(|g| g.role == GuideRole::LegendTitle)
                    })
                    .count();
                assert_eq!(title_count, if facet == "local" { 2 } else { 1 });
                let labels: Vec<_> = a
                    .scene()
                    .items()
                    .iter()
                    .filter_map(|i| {
                        if i.guide
                            .as_ref()
                            .is_some_and(|g| g.role == GuideRole::LegendLabel)
                        {
                            if let Primitive::Text { text, .. } = &i.primitive {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                let expected = if reverse {
                    vec!["D", "C", "B", "A"]
                } else {
                    vec!["A", "B", "C", "D"]
                };
                assert_eq!(
                    labels,
                    if facet == "local" {
                        expected.repeat(2)
                    } else {
                        expected
                    },
                    "{position:?} {facet}"
                );
                if facet == "single" {
                    let plot = a.plot().unwrap();
                    let clip = key_items[0].clip.unwrap();
                    match position {
                        LegendPosition::Right => assert!(clip.origin().x() >= plot.max_x()),
                        LegendPosition::Left => assert!(clip.max_x() <= plot.origin().x()),
                        LegendPosition::Top => assert!(clip.max_y() <= plot.origin().y()),
                        LegendPosition::Bottom => assert!(clip.origin().y() >= plot.max_y()),
                        LegendPosition::Inside { .. } => assert!(
                            clip.origin().x() >= plot.origin().x()
                                && clip.max_x() <= plot.max_x()
                                && clip.origin().y() >= plot.origin().y()
                                && clip.max_y() <= plot.max_y()
                        ),
                    }
                }
            }
        }
    }
}
#[test]
fn layer_guide_suppression_preserves_marks() {
    let shown = author(LegendPosition::Right, "single", false, false);
    let hidden = author(LegendPosition::Right, "single", false, true);
    let a = shown.chart().unwrap().prepare().unwrap();
    let b = hidden.chart().unwrap().prepare().unwrap();
    assert_eq!(a.layers()[0].marks().len(), b.layers()[0].marks().len());
    let frame = layout(b, &request(), &Metrics).unwrap();
    assert!(
        !frame
            .scene()
            .items()
            .iter()
            .any(|i| i.guide.as_ref().is_some_and(|g| matches!(
                g.role,
                GuideRole::LegendKey | GuideRole::LegendLabel | GuideRole::LegendTitle
            )))
    );
}
#[test]
fn custom_vector_guide_roundtrip_and_validation() {
    use chart_core::grammar::{CustomGuidePath, CustomLegend};
    let mut path = chart_core::path::Path::new();
    path.move_to(0., 0.).unwrap();
    path.line_to(40., 0.).unwrap();
    path.line_to(20., 20.).unwrap();
    path.close_path().unwrap();
    let custom = CustomLegend {
        id: chart_core::ScaleId::new(999),
        bounds: [0., 0., 40., 20.],
        paths: vec![CustomGuidePath {
            geometry: path.geometry(),
            fill: Some(chart_core::color::parse_r("#123456").unwrap()),
            stroke: None,
        }],
        options: LegendOptions {
            title: Some("Custom".into()),
            position: Some(LegendPosition::Left),
            ..Default::default()
        },
    };
    let data = Data::columns().column("x", vec![1., 2.]).build().unwrap();
    let p = plot(data.clone())
        .aes(aes().x("x").y(1.))
        .layer(points())
        .legend(legend().custom(custom.clone()))
        .build()
        .unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
        71
    );
    let restored = Plot::from_json(&wire).unwrap();
    let envelope = chart_core::portable::ChartEnvelope {
        version: 71,
        definition: restored.chart().unwrap().definition().clone(),
    };
    envelope.validate().unwrap();
    let frame = layout(
        restored.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    let custom_items: Vec<_> = frame
        .scene()
        .items()
        .iter()
        .filter(|i| {
            i.guide
                .as_ref()
                .is_some_and(|g| g.scope == ["legend", "custom"])
        })
        .collect();
    assert_eq!(custom_items.len(), 2);
    assert!(
        custom_items
            .iter()
            .any(|i| matches!(i.primitive, Primitive::VectorPath { .. }))
    );
    // Reference Guides$merge maps unspecified order zero to 99.
    let mut earlier = custom.clone();
    earlier.id = chart_core::ScaleId::new(998);
    earlier.options.order = 1;
    earlier.options.title = Some("Earlier".into());
    let ordered = plot(data.clone())
        .aes(aes().x("x").y(1.))
        .layer(points())
        .legend(legend().custom(custom.clone()))
        .legend(legend().custom(earlier))
        .build()
        .unwrap();
    let ordered = layout(
        ordered.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    let titles: Vec<_> = ordered
        .scene()
        .items()
        .iter()
        .filter_map(|i| i.guide.as_ref())
        .filter(|g| g.role == GuideRole::LegendTitle)
        .map(|g| g.label.as_deref().unwrap())
        .collect();
    assert_eq!(titles, ["Earlier", "Custom"]);
    let mut bad = custom.clone();
    bad.bounds[2] = -1.;
    assert!(
        plot(data.clone())
            .aes(aes().x("x").y(1.))
            .layer(points())
            .legend(legend().custom(bad))
            .build()
            .is_err()
    );
    assert!(
        plot(data)
            .aes(aes().x("x").y(1.))
            .layer(points())
            .legend(legend().custom(custom.clone()))
            .legend(legend().custom(custom))
            .build()
            .is_err()
    );
}
