//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::{
    ChartResult, Limits, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
    interpolate::Number,
    scales::*,
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
                ScaleValue::Number(v) => encoded_number(*v),
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
fn number(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn encoded_number(v: f64) -> Value {
    if v.is_nan() {
        Value::Null
    } else if v.is_infinite() {
        json!(if v > 0. { "Infinity" } else { "-Infinity" })
    } else {
        json!(v)
    }
}
fn equal(a: &Value, b: &Value) {
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        assert!((a - b).abs() <= 3e-12 * b.abs().max(1.), "{a} != {b}");
    } else {
        assert_eq!(a, b);
    }
}
#[test]
fn registered_scale_labels_match_180_reference_vectors_and_key_rules() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/guide-label-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 180);
    for c in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
            .unwrap();
        let labels = GgplotGuideLabels::Registered {
            operation: chart_core::grammar::OperationRef::new(
                "test.scale_labels",
                Revision::new(1),
            ),
            parameters: c["label_mode"].clone(),
        };
        let (actual, expected) = if c["kind"] == "continuous" {
            let family = match c["transform"].as_str().unwrap() {
                "sqrt" => NumericFamily::Pow { exponent: 0.5 },
                "log10" => NumericFamily::Log { base: 10. },
                _ => NumericFamily::Linear,
            };
            let guide = GgplotContinuousGuide {
                breaks: match c["break_mode"].as_str().unwrap() {
                    "auto" => None,
                    "empty" => Some(vec![]),
                    _ => Some(
                        [
                            -1.,
                            0.,
                            0.1,
                            1.,
                            5.,
                            10.,
                            20.,
                            f64::INFINITY,
                            f64::NAN,
                            f64::NAN,
                        ]
                        .map(Number)
                        .to_vec(),
                    ),
                },
                count: None,
                labels,
            };
            let result = guide.resolve_with_registry(
                [
                    Number(number(&c["limits"][0])),
                    Number(number(&c["limits"][1])),
                ],
                family,
                c["transform"] == "reverse",
                Limits::default(),
                &registry,
            );
            (
                result.map(|entries| entries.into_iter().map(|e| e.label).collect::<Vec<_>>()),
                c["result"].clone(),
            )
        } else {
            let domain = match c["population"].as_str().unwrap() {
                "empty" => vec![],
                "nullable" => vec![
                    ScaleKey::Text("a".into()),
                    ScaleKey::Text("b".into()),
                    ScaleKey::Null,
                ],
                _ => ["a", "b", "c"].map(|v| ScaleKey::Text(v.into())).to_vec(),
            };
            let explicit = [Some("z"), Some("b"), Some("b"), None, Some("a")]
                .map(|v| v.map_or(ScaleKey::Null, |v| ScaleKey::Text(v.into())))
                .to_vec();
            let guide = GgplotDiscreteGuide {
                breaks: match c["break_mode"].as_str().unwrap() {
                    "auto" => None,
                    "empty" => Some(vec![]),
                    _ => Some(explicit),
                },
                break_names: (c["break_mode"] == "named")
                    .then(|| ["Z", "B", "B2", "M", "A"].map(String::from).to_vec()),
                labels,
            };
            let result = guide
                .resolve_with_registry(&domain, &registry)
                .map(|entries| entries.into_iter().map(|e| e.label).collect::<Vec<_>>());
            let mut expected = c["build"].clone();
            if expected.get("error").is_none() {
                expected["labels"] = expected["keys"]
                    .as_array()
                    .unwrap()
                    .first()
                    .map_or(json!([]), |key| key["labels"].clone());
            }
            (result, expected)
        };
        if expected.get("error").is_some() {
            assert!(actual.is_err(), "{c}: {actual:?}");
        } else {
            assert_eq!(
                serde_json::to_value(actual.unwrap_or_else(|e| panic!("{c}: {e:?}"))).unwrap(),
                expected["labels"],
                "{c}"
            );
        }
        let calls = calls.lock().unwrap();
        let expected_calls = c["calls"].as_array().unwrap();
        assert_eq!(calls.len(), expected_calls.len(), "{c}");
        for (actual, expected) in calls.iter().zip(expected_calls) {
            assert_eq!(actual["names"], expected["names"], "{c}");
            let a = actual["values"].as_array().unwrap();
            let b = expected["values"].as_array().unwrap();
            assert_eq!(a.len(), b.len(), "{c}");
            for (a, b) in a.iter().zip(b) {
                equal(a, b);
            }
        }
    }
}

#[test]
fn primary_label_registry_portability_and_missing_installation_precede_execution() {
    use chart_core::prelude::*;
    for portable in [true, false] {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), portable)))
            .unwrap();
        let registry = Arc::new(registry);
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
            unreachable!()
        };
        scale.guide = Some(Box::new(GgplotScaleGuide::Continuous(
            GgplotContinuousGuide {
                labels: GgplotGuideLabels::Registered {
                    operation: chart_core::grammar::OperationRef::new(
                        "test.scale_labels",
                        Revision::new(1),
                    ),
                    parameters: json!("indexed"),
                },
                ..Default::default()
            },
        )));
        let data = Data::columns()
            .column("x", [1., 2.])
            .column("v", [1., 2.])
            .build()
            .unwrap();
        let make = |registry| {
            plot(data.clone())
                .extensions(registry)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale.clone()))
                .layer(points())
                .build()
        };
        let plot = make(registry.clone()).unwrap();
        calls.lock().unwrap().clear();
        let wire = plot.to_json();
        if portable {
            let wire = wire.unwrap();
            assert!(Plot::from_json(&wire).is_err());
            let mut wrong: Value = serde_json::from_str(&wire).unwrap();
            wrong["version"] = 31.into();
            assert!(Plot::from_json_with_extensions(&wrong.to_string(), registry).is_err());
        } else {
            assert_eq!(
                wire.unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
        }
        assert!(make(Arc::new(ExtensionRegistry::new())).is_err());
        assert!(calls.lock().unwrap().is_empty());
    }
}

#[test]
fn primary_noncolor_discrete_labels_match_800_reference_builds() {
    assert_noncolor_discrete_labels(
        include_str!("../../../fixtures/parity/ggplot2/noncolor-discrete-label-functions.json"),
        800,
        695,
    );
}

#[test]
fn factor_discrete_labels_preserve_drop_and_missing_policies() {
    let source =
        include_str!("../../../fixtures/parity/ggplot2/factor-discrete-label-functions.json");
    assert_noncolor_discrete_labels(source, 640, 640);
}

fn assert_noncolor_discrete_labels(source: &str, case_count: usize, successes: usize) {
    use chart_core::grammar::ValueAesthetic;
    use chart_core::prelude::*;
    let fixture: Value = serde_json::from_str(source).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), case_count);
    let mut successful = 0;
    for case in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
            .unwrap();
        let registry = Arc::new(registry);
        let palette = match case["channel"].as_str().unwrap() {
            "shape" => json!({"Shape":{"solid":true}}),
            "linetype" => json!("LineType"),
            channel => json!({"NumericRange":{"range":match channel {
                "alpha" => [0.1,1.], "linewidth" => [2.,6.], _ => [2.,6.]
            },"area":channel=="size"}}),
        };
        let key = |v: &Value| {
            if v.is_null() {
                json!("Null")
            } else {
                json!({"Text":v})
            }
        };
        let spec: MappedScaleSpec = serde_json::from_value(json!({
            "training":"Eligible",
            "function":{"Ordinal":{"domain":[],"range":[],"unknown":{"Explicit":null}}},
            "ggplot":{"Discrete":{"limits":if case["limit_mode"]=="explicit" {
                json!([{"Text":"c"},{"Text":"b"},{"Text":"a"},"Null"])
            } else { Value::Null },"levels":case["levels"].as_array().map(|v|v.iter().map(key).collect::<Vec<_>>()),"drop":case["drop"].as_bool().unwrap_or(true),"na_translate":case["na_translate"].as_bool().unwrap_or(true),"palette":palette}},
            "guide":{"Discrete":{
                "breaks":match case["break_mode"].as_str().unwrap() {
                    "auto" => Value::Null, "empty" => json!([]),
                    _ => Value::Array(json!(["z","b","b",null,"a"]).as_array().unwrap().iter().map(key).collect())
                },
                "break_names":if case["break_mode"]=="named" { json!(["Z","B","B2","M","A"]) } else { Value::Null },
                "labels":{"Registered":{"operation":{"id":"test.scale_labels","version":"1"},"parameters":case["label_mode"]}}
            }}
        })).unwrap();
        let inputs = case["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", inputs.iter().map(Value::as_str).collect::<Vec<_>>())
            .build()
            .unwrap();
        let layer = match case["channel"].as_str().unwrap() {
            "size" => points().numeric_scale(NumericAesthetic::Size, "v", spec),
            "alpha" => points().numeric_scale(NumericAesthetic::Alpha, "v", spec),
            "linewidth" => line().numeric_scale(NumericAesthetic::StrokeWidth, "v", spec),
            "shape" => points().value_scale(ValueAesthetic::Shape, "v", spec),
            "linetype" => line().value_scale(ValueAesthetic::LineType, "v", spec),
            _ => unreachable!(),
        };
        let result = (|| -> ChartResult<Value> {
            let plot = plot(data)
                .extensions(registry.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(layer)
                .build()?;
            let wire = plot.to_json()?;
            assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 32);
            let restored = Plot::from_json_with_extensions(&wire, registry)?;
            assert_eq!(restored.to_json()?, wire);
            assert!(
                calls.lock().unwrap().is_empty(),
                "preparation must own callback execution"
            );
            let prepared = restored.chart()?.prepare()?;
            let entries = prepared.layers()[0]
                .discrete_value_guides()
                .values()
                .flatten()
                .collect::<Vec<_>>();
            Ok(json!({
                "values":entries.iter().map(|entry| match &entry.key {
                    ScaleKey::Text(s) => json!(s), ScaleKey::Null => Value::Null, _ => panic!("unexpected key")
                }).collect::<Vec<_>>(),
                "labels":entries.iter().map(|entry| &entry.label).collect::<Vec<_>>()
            }))
        })();
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}: {result:?}");
        } else {
            successful += 1;
            let keys = case["result"]["keys"].as_array().unwrap();
            let expected = keys
                .first()
                .cloned()
                .unwrap_or_else(|| json!({"values":[],"labels":[]}));
            assert_eq!(
                result.unwrap_or_else(|e| panic!("{case}: {e:?}")),
                expected,
                "{case}"
            );
        }
        assert_eq!(
            *calls.lock().unwrap(),
            *case["calls"].as_array().unwrap(),
            "{case}"
        );
    }
    assert_eq!(successful, successes);
}

#[test]
fn primary_noncolor_continuous_labels_match_1440_reference_builds() {
    assert_noncolor_numeric_labels("continuous", 1440, 1044);
}
#[test]
fn primary_noncolor_binned_labels_match_1800_reference_builds() {
    assert_noncolor_numeric_labels("binned", 1800, 939);
}
fn assert_noncolor_numeric_labels(kind: &str, case_count: usize, successes: usize) {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/noncolor-numeric-label-functions.json"
    ))
    .unwrap();
    assert_numeric_labels(&fixture, kind, case_count, successes);
}
#[test]
fn continuous_interval_labels_match_1080_reference_builds() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-interval-label-functions.json"
    ))
    .unwrap();
    assert_numeric_labels(&fixture, "interval", 1080, 820);
}
fn assert_numeric_labels(fixture: &Value, kind: &str, case_count: usize, successes: usize) {
    use chart_core::prelude::*;
    let mut compared = 0;
    let mut successful = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == kind)
    {
        compared += 1;
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
            .unwrap();
        let registry = Arc::new(registry);
        let channel = case["channel"].as_str().unwrap();
        let family = match case["transform"].as_str().unwrap() {
            "sqrt" => json!({"Pow":{"exponent":0.5}}),
            "log10" => json!({"Log":{"base":10.}}),
            _ => json!("Linear"),
        };
        let labels = if case["label_mode"] == "default" {
            json!("Automatic")
        } else {
            json!({"Registered":{"operation":{"id":"test.scale_labels","version":"1"},"parameters":case["label_mode"]}})
        };
        let breaks = match case["break_mode"].as_str().unwrap() {
            "auto" => Value::Null,
            "empty" => json!([]),
            _ => json!([-1,0,1,1,3,20,{"number":"Infinity"},{"number":"NaN"}]),
        };
        let limits = if case["limits"] == "full" {
            json!([1, 10])
        } else {
            Value::Null
        };
        let policy = if kind == "binned" {
            json!({"Binned":{"limits":limits,"oob":"Squish","right":true,"breaks":if breaks.is_null(){json!({"Nice":5})}else{json!({"Explicit":breaks})}}})
        } else {
            json!({"Continuous":{"limits":limits,"oob":"Censor"}})
        };
        let guide = if kind == "binned" {
            json!({"Binned":labels})
        } else if kind == "interval" {
            let selection = if case["guide"] == "bins" {
                "ContinuousBins"
            } else {
                "ContinuousSteps"
            };
            json!({selection:{"breaks":breaks,"labels":labels}})
        } else {
            json!({"Continuous":{"breaks":breaks,"labels":labels}})
        };
        let output = if channel == "colour" {
            json!({"Interpolate":{"operation":"GgplotPalette","spec":{"Gradient":{"colors":[{"red":19,"green":43,"blue":67,"alpha":255},{"red":86,"green":177,"blue":247,"alpha":255}],"values":null}}}})
        } else {
            json!({"Interpolate":{"operation":"PowerRange","range":if channel=="alpha" {[0.1,1.]}else{[1.,6.]},"exponent":if channel=="size"{0.5}else{1.},"absolute":false}})
        };
        let spec: MappedScaleSpec = serde_json::from_value(json!({
            "training":"Eligible",
            "function":{"Interpolated":{
                "normalization":{"Ggplot":{"family":family,"domain":[0,1],"reverse":case["transform"]=="reverse","rescaler":"Range"}},
                "output":output,
                "unknown":{"kind":"Missing"}
            }},
            "ggplot":policy,"guide":guide
        })).unwrap();
        let inputs = case["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", inputs.iter().map(Value::as_f64).collect::<Vec<_>>())
            .build()
            .unwrap();
        let result = (|| -> ChartResult<Value> {
            let draft = plot(data)
                .extensions(registry.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.));
            let draft = if channel == "colour" {
                draft
                    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                    .scale(color_mapped("v", spec))
                    .layer(points())
            } else {
                let layer = match channel {
                    "size" => points().numeric_scale(NumericAesthetic::Size, "v", spec),
                    "alpha" => points().numeric_scale(NumericAesthetic::Alpha, "v", spec),
                    "linewidth" => line().numeric_scale(NumericAesthetic::StrokeWidth, "v", spec),
                    _ => unreachable!(),
                };
                draft.layer(layer)
            };
            let plot = draft.build()?;
            let wire = plot.to_json()?;
            let restored = Plot::from_json_with_extensions(&wire, registry)?;
            assert_eq!(restored.to_json()?, wire);
            assert!(calls.lock().unwrap().is_empty());
            let prepared = restored.chart()?.prepare()?;
            let entries = if channel == "colour" {
                prepared.layers()[0]
                    .color_legend()
                    .map(|g| g.numeric_breaks.clone())
                    .unwrap_or_default()
            } else {
                prepared.layers()[0]
                    .numeric_value_guides()
                    .values()
                    .flatten()
                    .cloned()
                    .collect()
            };
            let entries = entries
                .iter()
                .filter(|entry| entry.visible)
                .collect::<Vec<_>>();
            Ok(
                json!({"values":entries.iter().map(|entry|encoded_number(entry.transformed.0)).collect::<Vec<_>>(),"labels":entries.iter().map(|entry|&entry.label).collect::<Vec<_>>()}),
            )
        })();
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}: {result:?}");
            assert_eq!(
                calls.lock().unwrap().len(),
                case["calls"].as_array().unwrap().len(),
                "{case}: {result:?}"
            );
        } else {
            successful += 1;
            let actual = result.unwrap_or_else(|e| panic!("{case}: {e:?}"));
            let mut expected = case["result"]["keys"]
                .as_array()
                .unwrap()
                .first()
                .cloned()
                .unwrap_or_else(|| json!({"values":[],"labels":[]}));
            if kind == "binned" {
                expected["values"] = case["result"]["boundaries"].clone();
            } else if kind == "interval" {
                // The capture retains all parsed boundaries separately from visible keys.
                // Default forward interval guides censor an overflowing ordinal suffix.
                let visible = expected["values"].as_array().unwrap().len();
                expected["values"] = json!(
                    case["result"]["boundaries"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .take(visible)
                        .collect::<Vec<_>>()
                );
            }
            assert_eq!(actual["labels"], expected["labels"], "{case}");
            let values = actual["values"].as_array().unwrap();
            let expected = expected["values"].as_array().unwrap();
            assert_eq!(values.len(), expected.len(), "{case}");
            for (a, b) in values.iter().zip(expected) {
                equal(a, b);
            }
        }
        let calls = calls.lock().unwrap();
        let expected = case["calls"].as_array().unwrap();
        assert_eq!(calls.len(), expected.len(), "{case}");
        for (a, b) in calls.iter().zip(expected) {
            assert_eq!(a["names"], b["names"], "{case}");
            let av = a["values"].as_array().unwrap();
            let bv = b["values"].as_array().unwrap();
            assert_eq!(av.len(), bv.len(), "{case}");
            for (a, b) in av.iter().zip(bv) {
                equal(a, b);
            }
        }
    }
    assert_eq!((compared, successful), (case_count, successes));
}
