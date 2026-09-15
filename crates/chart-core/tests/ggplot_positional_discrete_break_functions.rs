//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    grammar::{
        CustomGuideFormatter, CustomScaleBreaks, ExtensionDescriptor, ExtensionRegistry,
        GuideLabelsInput, ScaleBreaksInput, ScaleBreaksOutput,
    },
    interpolate::Number,
    layout::*,
    prelude::*,
    scales::*,
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

struct Breaks(Arc<Mutex<Vec<Value>>>);
impl CustomScaleBreaks for Breaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.discrete_breaks", Revision::new(1), true)
    }
    fn accepts_n(&self) -> bool {
        true
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        assert!(
            input.count.is_none() && input.count_argument.is_none() && input.temporal.is_none()
        );
        let encoded = input
            .domain
            .iter()
            .map(|v| match v {
                ScaleKey::Text(v) => json!(v),
                ScaleKey::Null => Value::Null,
                _ => panic!("discrete key"),
            })
            .collect::<Vec<_>>();
        self.0
            .lock()
            .unwrap()
            .push(json!({"values":encoded,"names":[]}));
        let (values, names) = match input.parameters.as_str().unwrap() {
            "domain" => (Some(input.domain.to_vec()), None),
            "mixed" => (
                Some(vec![
                    ScaleKey::Text("outside".into()),
                    ScaleKey::Text("a".into()),
                    ScaleKey::Text("a".into()),
                    ScaleKey::Null,
                    ScaleKey::Text("c".into()),
                ]),
                Some(
                    ["off", "first", "duplicate", "missing", "last"]
                        .map(String::from)
                        .to_vec(),
                ),
            ),
            "numeric" => (
                Some(vec![
                    ScaleKey::Number(Number(1.)),
                    ScaleKey::Null,
                    ScaleKey::Number(Number(1.)),
                ]),
                Some(["one", "missing", "again"].map(String::from).to_vec()),
            ),
            "empty" => (Some(vec![]), None),
            "null" => (None, None),
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            values,
            names,
            temporal: None,
        })
    }
}

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn positional_discrete_break_functions_match_400_reference_builds() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-discrete-break-functions.json"
    ))
    .unwrap();
    let mut passed = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        for family in ["auto", "band", "point"] {
            for policy_labels in if c["label_mode"] == "indexed" {
                vec![false, true]
            } else {
                vec![false]
            } {
                let calls = Arc::new(Mutex::new(vec![]));
                let break_calls = Arc::new(Mutex::new(vec![]));
                let mut registry = ExtensionRegistry::default();
                registry
                    .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
                    .unwrap();
                registry
                    .register_scale_breaks(Arc::new(Breaks(break_calls.clone())))
                    .unwrap();
                let inputs = c["inputs"].as_array().unwrap();
                let data = Data::columns()
                    .column(
                        "x",
                        categorical(inputs.iter().map(|v| {
                            v.as_str().unwrap_or(if c["label_mode"] == "automatic" {
                                "NA"
                            } else {
                                ""
                            })
                        }))
                        .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                    )
                    .column("y", vec![1.; inputs.len()])
                    .build()
                    .unwrap();
                let axis = match family {
                    "band" => x_axis().scale(scale_band()),
                    "point" => x_axis().scale(scale_point()),
                    _ => x_axis(),
                };
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
                if policy_labels {
                    policy.guide.labels = chart_core::scales::GgplotGuideLabels::Registered {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.scale_labels",
                            Revision::new(1),
                        ),
                        parameters: c["label_mode"].clone(),
                    };
                }
                let mut axis = axis
                    .breaks_function(Some(chart_core::grammar::ScaleBreaksOperation {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.discrete_breaks",
                            Revision::new(1),
                        ),
                        parameters: c["mode"].clone(),
                    }))
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
                if policy_labels || c["label_mode"] == "automatic" {
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
                    let wire = plot.to_json()?;
                    assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 34);
                    assert!(break_calls.lock().unwrap().is_empty());
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
                let actual_breaks = break_calls.lock().unwrap();
                assert_eq!(
                    actual_breaks.is_empty(),
                    c["calls"].as_array().unwrap().is_empty(),
                    "break bypass {c}: {result:?}"
                );
                for call in actual_breaks.iter() {
                    assert_eq!(call, &c["calls"][0], "{c}");
                }
                let actual = calls.lock().unwrap();
                assert_eq!(
                    actual.is_empty(),
                    c["label_calls"].as_array().unwrap().is_empty(),
                    "case {index}: {c} {actual:?}"
                );
                for call in actual.iter() {
                    let expected = &c["label_calls"][0];
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
                            .map(|i| {
                                raw[i % raw.len()].as_str().unwrap_or(
                                    if c["label_mode"] == "automatic" {
                                        "NA"
                                    } else {
                                        ""
                                    },
                                )
                            })
                            .collect::<Vec<_>>();
                        let got = frame.guides()[&GuideId::new(0)]
                            .ticks
                            .iter()
                            .map(|t| t.label.as_str())
                            .collect::<Vec<_>>();
                        assert_eq!(got, labels, "case {index}");
                        let values = frame.guides()[&GuideId::new(0)]
                            .ticks
                            .iter()
                            .map(|tick| match &tick.value {
                                ScaleValue::Category(value) => json!(value),
                                ScaleValue::MissingCategory => Value::Null,
                                _ => panic!("category tick"),
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(json!(values), c["result"]["values"], "case {index}");
                    }
                }
            }
        }
    }
    assert_eq!(passed, 1800);
}
