//! FIX-GG04 / GG2-03: primary discrete duplicate axes against the pinned R oracle.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, composition::ScaleValue, layout::*, prelude::*,
    scales::*, services::*,
};
use serde_json::Value;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn discrete_secondary_matches_reference_panels() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-secondary.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 176);
    let request = LayoutRequest::new(
        Rect::new(0., 0., 640., 360.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for case in cases {
        for point in [false, true] {
            let inputs = case["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column(
                    "x",
                    categorical(inputs.iter().map(|v| v.as_str().unwrap_or("")))
                        .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                )
                .column("y", vec![1.; inputs.len()])
                .build()
                .unwrap();
            let control = case["control"].as_str().unwrap();
            let mut secondary = x_axis()
                .name("secondary")
                .side(AxisSide::Top)
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                }))
                .secondary("x", if control == "transformed" { 2. } else { 1. }, 0.);
            let keys = |values: &[Option<&str>]| {
                values
                    .iter()
                    .map(|v| {
                        v.map_or(ScaleValue::MissingCategory, |s| {
                            ScaleValue::Category(s.into())
                        })
                    })
                    .collect()
            };
            secondary =
                match control {
                    "empty" => secondary.tick_values(Some(vec![])),
                    "numeric" => secondary.tick_values(Some(
                        [-1., 0., 0.5, 1., 1.5, 2., 3., 4., 5.]
                            .into_iter()
                            .map(ScaleValue::Number)
                            .collect(),
                    )),
                    "character" => secondary.tick_values(Some(keys(&[
                        Some("b"),
                        None,
                        Some("NA"),
                        Some("absent"),
                        Some("a"),
                    ]))),
                    "explicit" => secondary
                        .tick_values(Some(keys(&[Some("b"), None, Some("NA"), Some("a")])))
                        .tick_format(Some(GuideFormatter::Labels(
                            ["Bee", "Missing", "Literal", "A"].map(String::from).into(),
                        ))),
                    "bad_labels" => secondary
                        .tick_values(Some(keys(&[Some("b"), Some("a")])))
                        .tick_format(Some(GuideFormatter::Labels(vec!["wrong".into()]))),
                    "hidden" => secondary.tick_format(Some(GuideFormatter::Labels(vec![
                            String::new();
                            case["result"]["breaks"].as_array().map_or(0, Vec::len)
                        ]))),
                    _ => secondary,
                };
            let mut policy = GgplotDiscretePosition {
                limits: case["limits"].as_array().map(|v| {
                    v.iter()
                        .map(|k| {
                            k.as_str()
                                .map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into()))
                        })
                        .collect()
                }),
                na_translate: case["na_translate"].as_bool().unwrap(),
                ..Default::default()
            };
            if matches!(control, "primary_breaks" | "primary_labels") {
                policy.guide.breaks =
                    Some(vec![ScaleKey::Text("b".into()), ScaleKey::Text("a".into())]);
            }
            if control == "primary_labels" {
                policy.guide.labels =
                    GgplotGuideLabels::Explicit(vec![Some("Bee".into()), Some("Aye".into())]);
            }
            if control == "primary_hidden" {
                policy.guide.labels = GgplotGuideLabels::Hidden;
            }
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(if point { scale_point() } else { scale_band() })
                        .range(0., 100.)
                        .discrete_policy(Some(policy)),
                )
                .x_axis(secondary)
                .build()
                .unwrap();
            let figure = Plot::from_json(&figure.to_json().unwrap()).unwrap();
            let id = figure.axis("secondary").unwrap().id();
            let result = layout(
                figure.chart().unwrap().prepare().unwrap(),
                &request,
                &Metrics,
            );
            if !case["result"]["error"].is_null() {
                assert!(result.is_err(), "{case}");
                continue;
            }
            let frame = result.unwrap_or_else(|e| panic!("{e:?} {case}"));
            let axis = &frame.axes()[&id];
            let expected = &case["result"];
            assert_eq!(
                axis.ticks.len(),
                expected["breaks"].as_array().unwrap().len(),
                "{case}"
            );
            for (i, tick) in axis.ticks.iter().enumerate() {
                let ScaleValue::Number(value) = tick.value else {
                    panic!("numeric secondary index")
                };
                assert!(
                    (value - expected["breaks"][i].as_f64().unwrap()).abs() < 2e-14,
                    "{case}"
                );
                let label = if matches!(control, "hidden" | "primary_hidden") {
                    ""
                } else {
                    expected["labels"][i].as_str().unwrap_or("NA")
                };
                assert_eq!(tick.label, label, "{case}");
                assert!(
                    (tick.position / 100. - expected["positions"][i].as_f64().unwrap()).abs()
                        < 2e-14,
                    "position {} {case}",
                    tick.position / 100.
                );
            }
        }
    }
}

#[test]
fn discrete_secondary_uses_primary_spacing_in_both_orientations_and_directions() {
    let request = LayoutRequest::new(
        Rect::new(0., 0., 640., 360.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for horizontal in [true, false] {
        for reverse in [true, false] {
            for point in [true, false] {
                let data = Data::columns()
                    .column("category", categorical(["a", "b", "c"]))
                    .column("number", vec![1., 2., 3.])
                    .build()
                    .unwrap();
                let range = if reverse { [540., 100.] } else { [100., 540.] };
                let primary = if horizontal { x_axis() } else { y_axis() }
                    .scale(if point { scale_point() } else { scale_band() })
                    .range(range[0], range[1]);
                let secondary = if horizontal { x_axis() } else { y_axis() }
                    .name("secondary")
                    .side(if horizontal {
                        AxisSide::Top
                    } else {
                        AxisSide::Right
                    })
                    .secondary(if horizontal { "x" } else { "y" }, 1., 0.)
                    .tick_format(Some(GuideFormatter::Numeric(Box::new(
                        chart_core::typography::NumericFormat {
                            specifier: ".2f".into(),
                            locale: Default::default(),
                        },
                    ))))
                    .guide_geometry(Some(GuideGeometry {
                        labels: Some(GuideLabelPolicy::Preserve),
                        ..Default::default()
                    }));
                let builder = plot(data).profile(Profile::Ggplot2_4_0_3).layer(points());
                let figure = if horizontal {
                    builder
                        .aes(aes().x("category").y("number"))
                        .x_axis(primary)
                        .x_axis(secondary)
                } else {
                    builder
                        .aes(aes().x("number").y("category"))
                        .y_axis(primary)
                        .y_axis(secondary)
                }
                .build()
                .unwrap();
                let id = figure.axis("secondary").unwrap().id();
                let frame = layout(
                    figure.chart().unwrap().prepare().unwrap(),
                    &request,
                    &Metrics,
                )
                .unwrap();
                let axis = &frame.axes()[&id];
                assert_eq!(axis.ticks.len(), 3);
                for (i, expected) in [0.187, 0.5, 0.812].into_iter().enumerate() {
                    let tick = axis
                        .ticks
                        .iter()
                        .find(|t| t.value == ScaleValue::Number(1. + i as f64))
                        .unwrap();
                    assert!(
                        ((tick.position - range[0]) / (range[1] - range[0]) - expected).abs()
                            < 2e-14
                    );
                    assert_eq!(tick.label, format!("{:.2}", 1. + i as f64));
                    assert!(axis.invert_value(tick.position).is_err());
                }
            }
        }
    }
}
