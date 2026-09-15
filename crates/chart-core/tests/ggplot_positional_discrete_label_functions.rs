//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
    layout::*,
    prelude::*,
    services::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
struct Formatter(Arc<Mutex<Vec<Value>>>, bool);
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.scale_labels", Revision::new(1), self.1)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        let values = input
            .values
            .iter()
            .map(|v| match v {
                ScaleValue::Number(v) if v.is_nan() => Value::Null,
                ScaleValue::Number(v) if v.is_infinite() => {
                    json!(if *v > 0. { "Infinity" } else { "-Infinity" })
                }
                ScaleValue::Number(v) => json!(v),
                ScaleValue::Category(v) => json!(v),
                ScaleValue::MissingCategory => Value::Null,
                _ => panic!("unexpected timestamp"),
            })
            .collect::<Vec<_>>();
        self.0
            .lock()
            .unwrap()
            .push(json!({"values":values,"names":input.names.unwrap_or(&[])}));
        let mut labels = if input.values.is_empty() {
            vec![Some("/0".into())]
        } else {
            (1..=input.values.len())
                .map(|i| Some(format!("{i}/{}", input.values.len())))
                .collect::<Vec<_>>()
        };
        match input.parameters.as_str().unwrap() {
            "missing" => {
                for (i, label) in labels.iter_mut().enumerate() {
                    if i % 2 == 1 {
                        *label = None;
                    }
                }
            }
            "short" => labels.truncate(1),
            "empty" => labels.clear(),
            "indexed" | "named" => {}
            _ => panic!("unexpected mode"),
        }
        Ok(labels)
    }
}

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn positional_discrete_labels_match_reference_selection_and_recycling() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-discrete-label-functions.json"
    ))
    .unwrap();
    let mut passed = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        for point in [false, true] {
            for policy_labels in [false, true] {
                let calls = Arc::new(Mutex::new(vec![]));
                let mut registry = ExtensionRegistry::default();
                registry
                    .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
                    .unwrap();
                let inputs = c["inputs"].as_array().unwrap();
                let data = Data::columns()
                    .column(
                        "x",
                        categorical(inputs.iter().map(|v| v.as_str().unwrap_or("")))
                            .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                    )
                    .column("y", vec![1.; inputs.len()])
                    .build()
                    .unwrap();
                let scale = if point { scale_point() } else { scale_band() };
                let mut policy = chart_core::scales::GgplotDiscretePosition {
                    levels: Some(
                        ["b", "a", "c", "unused"]
                            .map(|v| chart_core::scales::ScaleKey::Text(v.into()))
                            .to_vec(),
                    ),
                    limits: (c["limits"] == "full").then(|| {
                        vec![
                            chart_core::scales::ScaleKey::Text("c".into()),
                            chart_core::scales::ScaleKey::Text("b".into()),
                            chart_core::scales::ScaleKey::Text("a".into()),
                            chart_core::scales::ScaleKey::Null,
                        ]
                    }),
                    drop: c["drop"].as_bool().unwrap(),
                    na_translate: c["translate"].as_bool().unwrap(),
                    ..Default::default()
                };
                if c["break_mode"] == "named" {
                    policy.guide.breaks = Some(
                        [Some("outside"), Some("a"), Some("a"), None, Some("c")]
                            .map(|v| {
                                v.map_or(chart_core::scales::ScaleKey::Null, |v| {
                                    chart_core::scales::ScaleKey::Text(v.into())
                                })
                            })
                            .to_vec(),
                    );
                    policy.guide.break_names = Some(
                        ["off", "first", "duplicate", "missing", "last"]
                            .map(String::from)
                            .to_vec(),
                    );
                }
                if policy_labels {
                    policy.guide.labels = chart_core::scales::GgplotGuideLabels::Registered {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.scale_labels",
                            Revision::new(1),
                        ),
                        parameters: c["label_mode"].clone(),
                    };
                }
                let mut axis = x_axis()
                    .scale(scale)
                    .discrete_policy(Some(policy))
                    .tick_format(Some(GuideFormatter::Registered {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.scale_labels",
                            Revision::new(1),
                        ),
                        parameters: c["label_mode"].clone(),
                    }))
                    .guide_geometry(Some(GuideGeometry {
                        labels: Some(GuideLabelPolicy::Preserve),
                        ..Default::default()
                    }));
                if c["break_mode"] == "explicit" {
                    axis = axis.tick_values(Some(vec![
                        ScaleValue::Category("outside".into()),
                        ScaleValue::Category("a".into()),
                        ScaleValue::Category("a".into()),
                        ScaleValue::MissingCategory,
                        ScaleValue::Category("c".into()),
                    ]));
                } else if c["break_mode"] == "empty" {
                    axis = axis.tick_values(Some(vec![]));
                }
                if policy_labels {
                    axis = axis.tick_format(None);
                }
                let result = (|| -> ChartResult<_> {
                    let plot = plot(data)
                        .profile(Profile::Ggplot2_4_0_3)
                        .aes(aes().x("x").y("y"))
                        .layer(points())
                        .x_axis(axis)
                        .extensions(Arc::new(registry))
                        .build()?;
                    let prepared = plot.chart()?.prepare()?;
                    layout(
                        prepared,
                        &LayoutRequest::new(
                            Rect::new(0., 0., 640., 360.)?,
                            Units::LogicalPixels,
                            ResourceDescriptor {
                                id: ResourceId::new(1),
                                revision: Revision::INITIAL,
                                kind: ResourceKind::Font,
                                byte_len: 1,
                            },
                        ),
                        &Metrics,
                    )
                })();
                assert_eq!(
                    result.is_ok(),
                    c["result"].get("error").is_none(),
                    "case {index}: {c} got {:?}",
                    result.as_ref().err()
                );
                let actual = calls.lock().unwrap();
                assert_eq!(
                    actual.is_empty(),
                    c["calls"].as_array().unwrap().is_empty(),
                    "case {index}: {c} {actual:?}"
                );
                for call in actual.iter() {
                    let expected = &c["calls"][0];
                    assert_eq!(call["names"], expected["names"], "case {index}");
                    let a = call["values"].as_array().unwrap();
                    let b = expected["values"].as_array().unwrap();
                    assert_eq!(a.len(), b.len(), "case {index}: {a:?} != {b:?}");
                    for (a, b) in a.iter().zip(b) {
                        if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                            assert!(
                                (a - b).abs() <= 3e-12 * b.abs().max(1.),
                                "case {index}: {a} != {b}"
                            );
                        } else {
                            assert_eq!(a, b, "case {index}");
                        }
                    }
                }
                if let Ok(frame) = result {
                    passed += 1;
                    let finite_range = c["result"]["range"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(Value::is_number);
                    if finite_range {
                        let raw = c["result"]["labels"].as_array().unwrap();
                        let labels = (0..c["result"]["values"].as_array().unwrap().len())
                            .map(|i| raw[i % raw.len()].as_str().unwrap_or(""))
                            .collect::<Vec<_>>();
                        let got = frame.guides()[&GuideId::new(0)]
                            .ticks
                            .iter()
                            .map(|t| t.label.as_str())
                            .collect::<Vec<_>>();
                        assert_eq!(got, labels, "case {index}");
                    }
                }
            }
        }
    }
    assert_eq!(passed, 2140);
}
