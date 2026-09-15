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
        if input.domain_is_null && mode == "mixed" {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::NumericalDomain,
                "Named mixed NULL-domain break vector has incompatible names.",
                "Supply a compatible named vector.",
            ));
        }
        let (values, names) = match mode {
            "domain" => ((!input.domain_is_null).then_some(d), None),
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
struct Limits(Arc<Mutex<Vec<Value>>>);
impl chart_core::grammar::CustomScaleLimits for Limits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.scale_limits", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(
        &self,
        input: chart_core::grammar::ScaleLimitsInput<'_>,
    ) -> ChartResult<Option<Vec<ScaleKey>>> {
        self.0.lock().unwrap().push(json!(
            input
                .domain
                .unwrap_or(&[])
                .iter()
                .map(|v| match v {
                    ScaleKey::Number(n) => encoded_number(n.0),
                    ScaleKey::Null => Value::Null,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
        ));
        let number = |n| ScaleKey::Number(Number(n));
        Ok(match input.parameters.as_str().unwrap() {
            "identity" => input.domain.map(<[ScaleKey]>::to_vec),
            "reverse" => input.domain.map(|v| v.iter().rev().cloned().collect()),
            "fixed" => Some(vec![number(1.), number(10.)]),
            "lower_zero" | "missing_lower" => {
                let first = if input.parameters == "lower_zero" {
                    number(0.)
                } else {
                    ScaleKey::Null
                };
                Some(if let Some(domain) = input.domain {
                    vec![first, domain.get(1).cloned().unwrap_or(ScaleKey::Null)]
                } else {
                    vec![first]
                })
            }
            "empty" => Some(vec![]),
            "single" => Some(vec![number(5.)]),
            _ => unreachable!(),
        })
    }
}
#[test]
fn positional_binned_break_functions_match_5760_reference_builds() {
    assert_binned_breaks(false);
}
#[test]
fn positional_binned_joint_functions_match_1120_reference_builds() {
    assert_binned_breaks(true);
}
fn assert_binned_breaks(joint: bool) {
    let fixture: Value = serde_json::from_str(if joint {
        include_str!("../../../fixtures/parity/ggplot2/positional-binned-joint-dimensions.json")
    } else {
        include_str!("../../../fixtures/parity/ggplot2/positional-binned-break-functions.json")
    })
    .unwrap();
    let mut passed = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        for policy_labels in if c["label_mode"] == "indexed" {
            vec![false, true]
        } else {
            vec![false]
        } {
            let calls = Arc::new(Mutex::new(vec![]));
            let break_calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            let limit_calls = Arc::new(Mutex::new(vec![]));
            if joint {
                registry
                    .register_scale_limits(Arc::new(Limits(limit_calls.clone())))
                    .unwrap();
            }
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
            let mut bins = chart_core::scales::GgplotBinnedPosition {
                breaks_function: Some(Box::new(chart_core::grammar::ScaleBreaksOperation {
                    operation: chart_core::grammar::OperationRef::new(
                        "test.scale_breaks",
                        Revision::new(1),
                    ),
                    parameters: c["mode"].clone(),
                })),
                bins: chart_core::scales::GgplotBinnedPolicy {
                    limits: (c["limits"] == "full")
                        .then_some([Some(1.0.into()), Some(10.0.into())]),
                    breaks: GgplotBreaks::Nice(match c["count"].as_str().unwrap() {
                        "three" => 3.,
                        "zero" => 0.,
                        _ => 10.,
                    }),
                    ..Default::default()
                },
                transform: match c["transform"].as_str().unwrap() {
                    "identity" => None,
                    "sqrt" => Some(chart_core::scales::ScaleTransform::Sqrt),
                    "log10" => Some(chart_core::scales::ScaleTransform::Log { base: 10. }),
                    "reverse" => Some(chart_core::scales::ScaleTransform::Reverse),
                    _ => unreachable!(),
                },
                show_limits: c["show_limits"].as_bool().unwrap(),
                ..Default::default()
            };
            if policy_labels && c["label_mode"] != "automatic" {
                bins.labels = chart_core::scales::GgplotGuideLabels::Registered {
                    operation: chart_core::grammar::OperationRef::new(
                        "test.scale_labels",
                        Revision::new(1),
                    ),
                    parameters: c["label_mode"].clone(),
                };
            }
            let scale = scale_binned(bins);
            let mut axis = x_axis()
                .scale(scale)
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
            if joint {
                axis = axis.limits_function(chart_core::grammar::ScaleLimitsOperation {
                    operation: chart_core::grammar::OperationRef::new(
                        "test.scale_limits",
                        Revision::new(1),
                    ),
                    parameters: c["control"].clone(),
                });
            }
            let result = (|| -> ChartResult<_> {
                let plot = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points())
                    .x_axis(axis)
                    .extensions(Arc::new(registry))
                    .build()?;
                let validated = break_calls.lock().unwrap().clone();
                for call in &validated {
                    compare(call, &c["calls"][0], &c.to_string());
                }
                let wire = plot.to_json()?;
                assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 35);
                assert_eq!(
                    *break_calls.lock().unwrap(),
                    validated,
                    "serialization does not train bins"
                );
                let mut chart = plot.chart()?;
                for call in break_calls.lock().unwrap().iter() {
                    compare(call, &c["calls"][0], &c.to_string());
                }
                break_calls.lock().unwrap().clear();
                let prepared = chart.prepare()?;
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
                c["result"].get("error").is_none()
                    && c["result"]["positions"].get("error").is_none(),
                "case {index}: {c} got {:?}",
                result.as_ref().err()
            );
            let actual_breaks = break_calls.lock().unwrap();
            assert_eq!(
                actual_breaks.is_empty(),
                c["calls"].as_array().unwrap().is_empty(),
                "break skipped {c}: {result:?}"
            );
            if c["population"] != "empty" && !actual_breaks.is_empty() {
                assert_eq!(
                    actual_breaks.len(),
                    1,
                    "training must share one cut result {c}"
                );
            }
            for call in actual_breaks.iter() {
                compare(call, &c["calls"][0], &c.to_string());
            }
            if joint {
                let expected = c["limit_calls"].as_array().unwrap();
                for actual in limit_calls.lock().unwrap().iter() {
                    assert!(
                        expected.iter().any(|e| same_numbers(actual, e)),
                        "case {index}: limit input {actual} absent from {expected:?}"
                    );
                }
            }
            let actual = calls.lock().unwrap();
            // Invalid joint limit vectors can fail axis preflight before pure label
            // formatting. Failure outcomes are checked above; callback scheduling on
            // rejected frames is not part of the supported contract.
            if !joint || result.is_ok() {
                assert_eq!(
                    actual.is_empty(),
                    c["label_calls"].as_array().unwrap().is_empty(),
                    "case {index}: {c} {actual:?}"
                );
            }
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
                let axis = &frame.axes()[&chart_core::ScaleId::new(0)];
                let chart_core::grammar::ValueSpace::Scaled { scale, .. } = &axis.space else {
                    panic!("missing scaled position space");
                };
                let bins = scale.binned.as_ref().expect("prepared position bins");
                let population = inputs
                    .iter()
                    .map(|v| v.as_f64().map(Number))
                    .collect::<Vec<_>>();
                // A zero-row compiled layer does not invoke the bin classifier.
                let indices = if population.is_empty() {
                    vec![]
                } else {
                    bins.map_before_statistics(&population)
                        .unwrap_or_else(|e| panic!("case {index}: {c} {e:?}"))
                };
                let mapped = bins
                    .map_after_statistics(&indices)
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map_or(Value::Null, |v| encoded_number(v.0)))
                    .collect::<Vec<_>>();
                compare(&json!(mapped), &c["result"]["mapped"], &c.to_string());
                if c["result"]["range"].as_array().unwrap().is_empty() {
                    let state = serde_json::to_value(bins).unwrap();
                    assert!(
                        state["function_limits"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(|v| v.get("number").is_some_and(|n| n == "NaN"))
                    );
                    let ResolvedScale::Unbounded(scale) = &axis.scale else {
                        panic!("empty extent must retain its sentinel");
                    };
                    assert_eq!(
                        scale.viewport(),
                        [Number(f64::INFINITY), Number(f64::NEG_INFINITY)]
                    );
                    assert!(frame.guides()[&GuideId::new(0)].ticks.is_empty());
                }
                let finite_range = c["result"]["range"].as_array().unwrap().len() == 2
                    && c["result"]["range"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(Value::is_number);
                if finite_range {
                    let axis = &frame.axes()[&chart_core::ScaleId::new(0)];
                    let (view, range) = match &axis.scale {
                        ResolvedScale::Linear(scale) => (scale.viewport(), scale.range()),
                        ResolvedScale::Nonlinear(scale) => {
                            (scale.transformed_viewport(), scale.range())
                        }
                        _ => panic!("case {index}: finite source panel has {:?}", axis.scale),
                    };
                    let expected_view = c["result"]["range"].as_array().unwrap();
                    for (actual, expected) in [view.minimum(), view.maximum()]
                        .into_iter()
                        .zip(expected_view)
                    {
                        let expected = expected.as_f64().unwrap();
                        assert!(
                            (actual - expected).abs() <= 3e-12 * expected.abs().max(1.),
                            "case {index}: panel {actual} != {expected}"
                        );
                    }
                    let positions = c["result"]["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(Value::as_f64)
                        .map(|v| {
                            (v - expected_view[0].as_f64().unwrap())
                                / (expected_view[1].as_f64().unwrap()
                                    - expected_view[0].as_f64().unwrap())
                        });
                    for (tick, expected) in
                        frame.guides()[&GuideId::new(0)].ticks.iter().zip(positions)
                    {
                        let actual =
                            (tick.position - range.start()) / (range.end() - range.start());
                        assert!(
                            (actual - expected).abs() <= 3e-12,
                            "case {index}: tick {actual} != {expected}"
                        );
                    }
                    let labels = c["result"]["values"]
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
                }
            }
        }
    }
    assert_eq!(passed, if joint { 1026 } else { 5940 });
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
fn positional_binned_break_registration_wire_and_fixed_cut_conflicts() {
    use chart_core::grammar::{OperationRef, ScaleBreaksOperation};
    for portable in [true, false] {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_breaks(Arc::new(Breaks(calls.clone(), "n".into(), portable)))
            .unwrap();
        let registry = Arc::new(registry);
        let mut spec = GgplotBinnedPosition {
            breaks_function: Some(Box::new(ScaleBreaksOperation {
                operation: OperationRef::new("test.scale_breaks", Revision::new(1)),
                parameters: json!("mixed"),
            })),
            ..Default::default()
        };
        let make = |registry, spec| {
            plot(Data::columns().column("x", [1., 4., 10.]).build().unwrap())
                .extensions(registry)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(x_axis().scale(scale_binned(spec)))
                .build()
        };
        let p = make(registry.clone(), spec.clone()).unwrap();
        let count = calls.lock().unwrap().len();
        if portable {
            let wire = p.to_json().unwrap();
            assert_eq!(calls.lock().unwrap().len(), count);
            assert!(Plot::from_json(&wire).is_err());
            let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            let mut wrong: Value = serde_json::from_str(&wire).unwrap();
            wrong["version"] = 34.into();
            assert!(Plot::from_json_with_extensions(&wrong.to_string(), registry.clone()).is_err());
        } else {
            assert_eq!(
                p.to_json().unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
        }
        assert!(make(Arc::new(ExtensionRegistry::new()), spec.clone()).is_err());
        spec.bins.breaks = GgplotBreaks::Explicit(vec![Number(3.)]);
        assert!(make(registry.clone(), spec.clone()).is_err());
        spec.breaks_function = None;
        let p = make(Arc::new(ExtensionRegistry::new()), spec).unwrap();
        assert!(
            serde_json::from_str::<Value>(&p.to_json().unwrap()).unwrap()["version"]
                .as_u64()
                .unwrap()
                < 35
        );
    }
}

fn same_numbers(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_numbers(a, b))
    } else if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        (a - b).abs() <= 3e-12 * b.abs().max(1.)
    } else {
        a == b
    }
}
