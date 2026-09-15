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

struct Breaks(Arc<Mutex<Vec<Value>>>, String, bool);
impl CustomScaleBreaks for Breaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.scale_breaks", Revision::new(1), self.2)
    }
    fn accepts_n(&self) -> bool {
        self.1 == "n"
    }
    fn accepts_n_breaks(&self) -> bool {
        self.1 == "n.breaks"
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        assert_eq!(input.count_argument, input.count.map(|_| self.1.as_str()));
        let d = input
            .domain
            .iter()
            .map(|v| match v {
                ScaleKey::Number(n) => n.0,
                _ => panic!("numeric domain"),
            })
            .collect::<Vec<_>>();
        let effective = if self.1 == "n" {
            json!(input.count.unwrap_or(7.))
        } else if self.1 == "n.breaks" {
            json!(input.count.unwrap_or(9.))
        } else {
            Value::Null
        };
        self.0.lock().unwrap().push(json!({"limits":d.iter().map(|v|encoded_number(*v)).collect::<Vec<_>>(),"count":input.count,"effective":effective,"names":[]}));
        let mode = input.parameters.as_str().unwrap();
        let (values, names) = match mode {
            "domain" => (Some(d), None),
            "mixed" => (
                Some(vec![
                    d.get(1).copied().unwrap_or(f64::NAN),
                    d.iter().sum::<f64>() / d.len() as f64,
                    d.first().copied().unwrap_or(f64::NAN),
                    d.first().copied().unwrap_or(f64::NAN),
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ]),
                Some(
                    [
                        "last", "middle", "first", "again", "missing", "positive", "negative",
                    ]
                    .map(String::from)
                    .to_vec(),
                ),
            ),
            "empty" => (Some(vec![]), None),
            "null" => (None, None),
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            temporal: None,
            values: values.map(|v| v.into_iter().map(|n| ScaleKey::Number(Number(n))).collect()),
            names,
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
fn positional_numeric_break_functions_match_1440_reference_panels() {
    assert_positional_breaks(false);
}
#[test]
fn positional_named_breaks_match_default_labels() {
    assert_positional_breaks(true);
}
fn assert_positional_breaks(automatic: bool) {
    let fixture: Value = serde_json::from_str(if automatic {
        include_str!("../../../fixtures/parity/ggplot2/positional-break-default-names.json")
    } else {
        include_str!("../../../fixtures/parity/ggplot2/positional-break-functions.json")
    })
    .unwrap();
    let mut passed = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let calls = Arc::new(Mutex::new(vec![]));
        let break_calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::default();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
            .unwrap();
        registry
            .register_scale_breaks(Arc::new(Breaks(
                break_calls.clone(),
                c["signature"].as_str().unwrap().into(),
                true,
            )))
            .unwrap();
        let inputs = c["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column(
                "x",
                chart_core::plot::column(
                    inputs
                        .iter()
                        .map(|v| v.as_f64().unwrap_or(0.))
                        .collect::<Vec<_>>(),
                )
                .validity(inputs.iter().map(|v| !v.is_null()).collect()),
            )
            .column("y", vec![1.; inputs.len()])
            .build()
            .unwrap();
        let mut scale = match c["transform"].as_str().unwrap() {
            "identity" => scale_linear(),
            "sqrt" => scale_sqrt(),
            "log10" => scale_log(10.),
            "reverse" => scale_reverse(),
            _ => unreachable!(),
        };
        if c["limits"] == "full" {
            scale = scale.domain(1., 10.);
        }
        let axis = x_axis()
            .scale(scale)
            .breaks_function(Some(chart_core::grammar::ScaleBreaksOperation {
                operation: chart_core::grammar::OperationRef::new(
                    "test.scale_breaks",
                    Revision::new(1),
                ),
                parameters: c["mode"].clone(),
            }))
            .tick_arguments(Some(GuideTickArguments {
                count: match c["count"].as_str().unwrap() {
                    "three" => Some(3.),
                    "zero" => Some(0.),
                    _ => None,
                },
                ..Default::default()
            }))
            .tick_format(Some(GuideFormatter::Registered {
                operation: chart_core::grammar::OperationRef::new(
                    "test.scale_labels",
                    Revision::new(1),
                ),
                parameters: json!("indexed"),
            }))
            .guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }));
        let axis = if automatic {
            axis.tick_format(None)
        } else {
            axis
        };
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
            compare(call, &c["calls"][0], &c.to_string());
        }
        let actual = calls.lock().unwrap();
        assert_eq!(
            actual.is_empty(),
            c["label_calls"].as_array().unwrap().is_empty(),
            "label bypass {c}: {result:?}"
        );
        for call in actual.iter() {
            compare(call, &c["label_calls"][0], &c.to_string());
        }
        if let Ok(frame) = result {
            passed += 1;
            let finite_range = c["result"]["range"]
                .as_array()
                .unwrap()
                .iter()
                .all(Value::is_number);
            if finite_range {
                let labels = c["result"]["breaks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(c["result"]["labels"].as_array().unwrap())
                    .filter(|(v, _)| v.is_number())
                    .map(|(_, v)| v.as_str().unwrap_or(""))
                    .collect::<Vec<_>>();
                let got = frame.guides()[&GuideId::new(0)]
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(got, labels, "case {index}");
                let transformed = frame.guides()[&GuideId::new(0)]
                    .ticks
                    .iter()
                    .map(|tick| {
                        let ScaleValue::Number(value) = tick.value else {
                            panic!("numeric tick")
                        };
                        encoded_number(match c["transform"].as_str().unwrap() {
                            "sqrt" => value.sqrt(),
                            "log10" => value.log10(),
                            "reverse" => -value,
                            _ => value,
                        })
                    })
                    .collect::<Vec<_>>();
                let expected = c["result"]["breaks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| v.is_number())
                    .cloned()
                    .collect::<Vec<_>>();
                compare(&json!(transformed), &json!(expected), &c.to_string());
            }
        }
    }
    assert_eq!(passed, if automatic { 40 } else { 648 });
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
fn compare(a: &Value, b: &Value, case: &str) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            assert!(
                (a - b).abs() <= 3e-12 * b.abs().max(1.),
                "{case}: {a} != {b}"
            );
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{case}: {a:?} != {b:?}");
            for (a, b) in a.iter().zip(b) {
                compare(a, b, case)
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{case}");
            for (k, a) in a {
                compare(a, &b[k], case)
            }
        }
        _ => assert_eq!(a, b, "{case}"),
    }
}

#[test]
fn positional_break_wire_registration_and_replacement_are_checked_before_evaluation() {
    use chart_core::grammar::{OperationRef, ScaleBreaksOperation};
    for portable in [true, false] {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_breaks(Arc::new(Breaks(calls.clone(), "n".into(), portable)))
            .unwrap();
        let registry = Arc::new(registry);
        let call = ScaleBreaksOperation {
            operation: OperationRef::new("test.scale_breaks", Revision::new(1)),
            parameters: json!("domain"),
        };
        let data = Data::columns().column("x", [1., 2.]).build().unwrap();
        let make = |registry, axis| {
            plot(data.clone())
                .extensions(registry)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(axis)
                .build()
        };
        let axis = x_axis().breaks_function(Some(call.clone()));
        let p = make(registry.clone(), axis.clone()).unwrap();
        if portable {
            let wire = p.to_json().unwrap();
            assert!(Plot::from_json(&wire).is_err());
            let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            let mut wrong: Value = serde_json::from_str(&wire).unwrap();
            wrong["version"] = 33.into();
            assert!(Plot::from_json_with_extensions(&wrong.to_string(), registry.clone()).is_err());
        } else {
            assert_eq!(
                p.to_json().unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
        }
        assert!(make(Arc::new(ExtensionRegistry::new()), axis.clone()).is_err());
        let fixed = axis.tick_values(Some(vec![ScaleValue::Number(1.)]));
        let p = make(Arc::new(ExtensionRegistry::new()), fixed).unwrap();
        assert!(
            serde_json::from_str::<Value>(&p.to_json().unwrap()).unwrap()["version"]
                .as_u64()
                .unwrap()
                < 34
        );
        assert!(calls.lock().unwrap().is_empty());
    }
}
