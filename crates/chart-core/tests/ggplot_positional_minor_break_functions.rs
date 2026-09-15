//! FIX-GG04: numeric primary minor callback arity and transformed-domain contract.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    grammar::{
        CustomScaleBreaks, ExtensionDescriptor, ExtensionRegistry, OperationRef, ScaleBreaksInput,
        ScaleBreaksOperation, ScaleBreaksOutput,
    },
    interpolate::Number,
    layout::*,
    prelude::*,
    scales::*,
    services::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
fn encoded(v: f64) -> Value {
    if v.is_nan() {
        Value::Null
    } else if v.is_infinite() {
        json!(if v > 0. { "Infinity" } else { "-Infinity" })
    } else {
        json!(v)
    }
}
struct Breaks(Arc<Mutex<Vec<Value>>>, bool, bool);
impl CustomScaleBreaks for Breaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.minor_breaks", Revision::new(1), self.2)
    }
    fn accepts_major_breaks(&self) -> bool {
        self.1
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        assert_eq!(input.count, None);
        assert_eq!(input.count_argument, None);
        assert!(input.temporal.is_none());
        let numbers = |keys: &[ScaleKey]| {
            keys.iter()
                .map(|v| {
                    let ScaleKey::Number(n) = v else {
                        panic!("numeric callback inputs")
                    };
                    n.0
                })
                .collect::<Vec<_>>()
        };
        let d = numbers(input.domain);
        let b = input.major_breaks.map(numbers);
        self.0.lock().unwrap().push(json!({"limits":d.iter().map(|v|encoded(*v)).collect::<Vec<_>>(),"major":b.as_ref().map(|v|v.iter().map(|n|encoded(*n)).collect::<Vec<_>>())}));
        let values = match input.parameters.as_str().unwrap() {
            "domain" => Some(d),
            "majors" => b,
            "mixed" => Some(vec![
                d[1],
                (d[0] + d[1]) / 2.,
                d[0],
                d[0],
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]),
            "outside" => Some(vec![
                d[1],
                (d[0] + d[1]) / 2.,
                d[0],
                d[0],
                -100.,
                100.,
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]),
            "empty" => Some(vec![]),
            "null" => None,
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            values: values.map(|v| v.into_iter().map(|v| ScaleKey::Number(Number(v))).collect()),
            ..Default::default()
        })
    }
}
struct Major(Arc<Mutex<Vec<Value>>>);
impl CustomScaleBreaks for Major {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.joint_major", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        assert!(input.major_breaks.is_none());
        assert!(input.count.is_none());
        let d = input
            .domain
            .iter()
            .map(|v| {
                let ScaleKey::Number(n) = v else {
                    unreachable!()
                };
                n.0
            })
            .collect::<Vec<_>>();
        self.0
            .lock()
            .unwrap()
            .push(json!(d.iter().map(|v| encoded(*v)).collect::<Vec<_>>()));
        let values = if input.parameters == "function_fixed" {
            vec![10., 5., 1., 1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
        } else {
            d
        };
        Ok(ScaleBreaksOutput {
            values: Some(
                values
                    .into_iter()
                    .map(|n| ScaleKey::Number(Number(n)))
                    .collect(),
            ),
            ..Default::default()
        })
    }
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn same(a: &Value, b: &Value, path: &str) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a.as_f64().unwrap();
            let b = b.as_f64().unwrap();
            assert!(
                (a - b).abs() <= 3e-12 * b.abs().max(1.),
                "{path}: {a} != {b}"
            );
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: {a:?} != {b:?}");
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                same(a, b, &format!("{path}/{i}"));
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, a) in a {
                same(a, &b[k], &format!("{path}/{k}"));
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
#[test]
fn numeric_minor_callbacks_match_3200_source_builds() {
    assert_numeric_minor_callbacks(
        include_str!("../../../fixtures/parity/ggplot2/positional-minor-break-functions.json"),
        2516,
        false,
    );
}
#[test]
fn joint_major_minor_callbacks_match_1600_source_builds() {
    assert_numeric_minor_callbacks(
        include_str!(
            "../../../fixtures/parity/ggplot2/positional-minor-break-joint-functions.json"
        ),
        1258,
        true,
    );
}
fn assert_numeric_minor_callbacks(source: &str, expected_success: usize, joint: bool) {
    let fixture: Value = serde_json::from_str(source).unwrap();
    let mut success = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::default();
        let major_calls = Arc::new(Mutex::new(vec![]));
        if joint {
            registry
                .register_scale_breaks(Arc::new(Major(major_calls.clone())))
                .unwrap();
        }
        registry
            .register_scale_breaks(Arc::new(Breaks(
                calls.clone(),
                c["signature"] == "two",
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
        let (mut scale, transform) = match c["transform"].as_str().unwrap() {
            "identity" => (scale_linear(), None),
            "sqrt" => (scale_sqrt(), Some(ScaleTransform::Sqrt)),
            "log10" => (scale_log(10.), Some(ScaleTransform::Log { base: 10. })),
            "reverse" => (scale_reverse(), Some(ScaleTransform::Reverse)),
            _ => unreachable!(),
        };
        if c["limits"] == "full" {
            scale = scale.domain(1., 10.);
        }
        let mut axis = x_axis()
            .scale(scale)
            .range(100., 540.)
            .guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }))
            .minor_breaks(Some(MinorBreaks::Registered(ScaleBreaksOperation {
                operation: OperationRef::new("test.minor_breaks", Revision::new(1)),
                parameters: c["mode"].clone(),
            })));
        match c["major"].as_str().unwrap() {
            "explicit" => {
                axis = axis.tick_values(Some(
                    [10., 5., 1., 1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
                        .into_iter()
                        .map(ScaleValue::Number)
                        .collect(),
                ))
            }
            "empty" => axis = axis.tick_values(Some(vec![])),
            "null" => axis = axis.ticks(Vec::<(ScaleValue, String)>::new()),
            _ => (),
        }
        if joint {
            axis = axis.breaks_function(Some(ScaleBreaksOperation {
                operation: OperationRef::new("test.joint_major", Revision::new(1)),
                parameters: c["major"].clone(),
            }));
        }
        if c["expand"] == "zero" {
            axis = axis.expansion(Some(GgplotExpansion {
                mult: [0.; 2],
                add: [0.; 2],
            }));
        }
        let result = (|| -> ChartResult<_> {
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(axis)
                .extensions(Arc::new(registry))
                .build()?;
            let wire = p.to_json()?;
            assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 36);
            assert!(calls.lock().unwrap().is_empty());
            layout(
                p.chart()?.prepare()?,
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
            "case {index} {c}: {:?}",
            result.as_ref().err()
        );
        let actual = calls.lock().unwrap();
        let expected = c["calls"].as_array().unwrap();
        assert_eq!(
            actual.is_empty(),
            expected.is_empty(),
            "call presence {index}"
        );
        for call in actual.iter() {
            same(call, &expected[0], &format!("callback {index}"));
        }
        if joint {
            let actual = major_calls.lock().unwrap();
            let expected = c["major_calls"].as_array().unwrap();
            assert_eq!(
                actual.is_empty(),
                expected.is_empty(),
                "major presence {index}"
            );
            for call in actual.iter() {
                same(call, &expected[0], &format!("major callback {index}"));
            }
        }
        if let Ok(frame) = result {
            success += 1;
            let range = c["result"]["range"].as_array().unwrap();
            let finite = range.iter().all(Value::is_number);
            let wanted_labels = c["result"]["major"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .filter(|(v, _)| v.is_number() && finite)
                .map(|(_, label)| label.as_str().unwrap_or(""))
                .collect::<Vec<_>>();
            assert_eq!(
                frame.guides()[&GuideId::new(0)]
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>(),
                wanted_labels,
                "major labels {index}"
            );
            for (tick, wanted) in frame.guides()[&GuideId::new(0)].ticks.iter().zip(
                c["result"]["major"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(Value::as_f64)
                    .filter(|_| finite),
            ) {
                let a = range[0].as_f64().unwrap();
                let b = range[1].as_f64().unwrap();
                let position = if a == b {
                    320.
                } else {
                    100. + 440. * (wanted - a) / (b - a)
                };
                assert!(
                    (tick.position - position).abs() < 1e-8,
                    "major position {index}: {} != {position}",
                    tick.position
                );
            }
            let expected: Vec<_> = c["result"]["minor"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_f64)
                .filter(|_| finite)
                .collect();
            let got = &frame.guides()[&GuideId::new(0)].minor_ticks;
            assert_eq!(
                got.len(),
                expected.len(),
                "minor count {index}: {got:?} != {expected:?}"
            );
            for (tick, wanted) in got.iter().zip(expected) {
                let raw = match tick.value {
                    Some(ScaleValue::Number(n)) => n,
                    _ => panic!("numeric minor values"),
                };
                let value = match transform {
                    None => raw,
                    Some(ScaleTransform::Sqrt) => raw.sqrt(),
                    Some(ScaleTransform::Log { base }) => raw.log(base),
                    Some(ScaleTransform::Reverse) => -raw,
                    _ => unreachable!(),
                };
                same(&json!(value), &json!(wanted), &format!("minor {index}"));
                let a = range[0].as_f64().unwrap();
                let b = range[1].as_f64().unwrap();
                let position = 100. + 440. * (wanted - a) / (b - a);
                assert!(
                    (tick.position - position).abs() < 1e-8,
                    "position {index}: {} != {position}",
                    tick.position
                );
            }
        }
    }
    assert_eq!(success, expected_success);
}

#[test]
fn registered_minor_wire_portability_extra_guides_and_budget_are_explicit() {
    for portable in [true, false] {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::default();
        registry
            .register_scale_breaks(Arc::new(Breaks(calls.clone(), true, portable)))
            .unwrap();
        let registry = Arc::new(registry);
        let call = ScaleBreaksOperation {
            operation: OperationRef::new("test.minor_breaks", Revision::new(1)),
            parameters: json!("mixed"),
        };
        let data = Data::columns().column("x", [1., 2.]).build().unwrap();
        let make = |registry, axis| {
            plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(axis)
                .y_axis(y_axis().visible(false))
                .extensions(registry)
                .build()
        };
        let axis = x_axis()
            .ticks(Vec::<(ScaleValue, String)>::new())
            .minor_breaks(Some(MinorBreaks::Registered(call.clone())));
        let p = make(registry.clone(), axis.clone()).unwrap();
        if portable {
            let wire = p.to_json().unwrap();
            assert!(Plot::from_json(&wire).is_err());
            assert_eq!(
                Plot::from_json_with_extensions(&wire, registry.clone())
                    .unwrap()
                    .to_json()
                    .unwrap(),
                wire
            );
            let mut wrong: Value = serde_json::from_str(&wire).unwrap();
            wrong["version"] = 35.into();
            assert!(Plot::from_json_with_extensions(&wrong.to_string(), registry.clone()).is_err());
        } else {
            assert_eq!(
                p.to_json().unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
        }
        assert!(make(Arc::new(ExtensionRegistry::default()), axis).is_err());
        assert!(calls.lock().unwrap().is_empty());
        let mut request = LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        request.max_ticks = 2;
        request.target_ticks = 2;
        let failure =
            layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap_err();
        assert_eq!(
            failure.code,
            chart_core::DiagnosticCode::ResourceLimit,
            "{failure:?}"
        );
        assert!(!calls.lock().unwrap().is_empty());
        let extra = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(points())
            .guide(axis_guide("minor", "x").minor_breaks(Some(MinorBreaks::Registered(call))))
            .extensions(registry)
            .build()
            .unwrap();
        request.max_ticks = 4096;
        let frame = layout(
            extra.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap();
        let guide = frame
            .guides()
            .values()
            .find(|g| matches!(g.spec.style.minor_breaks, Some(MinorBreaks::Registered(_))))
            .unwrap();
        assert_eq!(guide.minor_ticks.len(), 4);
    }
}

#[test]
fn discrete_minor_callbacks_and_binned_exclusion_match_source_builds() {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-other-minor-break-functions.json"
    ))
    .unwrap();
    let mut success = 0;
    let mut rejected = 0;
    for (index, c) in source["cases"].as_array().unwrap().iter().enumerate() {
        for point in [false, true] {
            let discrete = c["family"] == "discrete";
            if !discrete && point {
                continue;
            }
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            registry
                .register_scale_breaks(Arc::new(Breaks(
                    calls.clone(),
                    c["signature"] == "two",
                    true,
                )))
                .unwrap();
            let inputs = c["inputs"].as_array().unwrap();
            let data = if discrete {
                Data::columns()
                    .column(
                        "x",
                        categorical(inputs.iter().map(|v| v.as_str().unwrap_or("")))
                            .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                    )
                    .build()
                    .unwrap()
            } else {
                Data::columns()
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
                    .build()
                    .unwrap()
            };
            let mut axis = if discrete {
                (if point {
                    x_axis().scale(scale_point())
                } else {
                    x_axis().scale(scale_band())
                })
                .discrete_policy(Some(GgplotDiscretePosition::default()))
            } else {
                x_axis().scale(scale_binned(GgplotBinnedPosition::default()))
            };
            axis = axis
                .range(100., 540.)
                .minor_breaks(Some(MinorBreaks::Registered(ScaleBreaksOperation {
                    operation: OperationRef::new("test.minor_breaks", Revision::new(1)),
                    parameters: if c["mode"] == "mixed" {
                        json!("outside")
                    } else {
                        c["mode"].clone()
                    },
                })));
            if c["major"] == "empty" {
                axis = axis.tick_values(Some(vec![]));
            } else if c["major"] == "null" {
                axis = axis.ticks([]);
            }
            if c["expand"] == "zero" {
                axis = axis.expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }));
            }
            let result = (|| -> ChartResult<_> {
                let p = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y(1.))
                    .layer(points())
                    .x_axis(axis)
                    .y_axis(y_axis().visible(false))
                    .extensions(Arc::new(registry))
                    .build()?;
                assert_eq!(
                    serde_json::from_str::<Value>(&p.to_json()?).unwrap()["version"],
                    36
                );
                layout(
                    p.chart()?.prepare()?,
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
            if !discrete {
                assert!(result.is_err(), "binned {index}");
                assert!(calls.lock().unwrap().is_empty());
                rejected += 1;
                continue;
            }
            let frame = result.unwrap_or_else(|e| panic!("case {index} point {point}: {e:?}"));
            let actual = calls.lock().unwrap();
            assert!(!actual.is_empty(), "callback {index}");
            for call in actual.iter() {
                same(
                    call,
                    &c["calls"][0],
                    &format!("discrete {index} point {point}"),
                );
            }
            let guide = &frame.guides()[&GuideId::new(0)];
            let expected = c["result"]["minor"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_f64)
                .collect::<Vec<_>>();
            assert_eq!(
                guide.minor_ticks.len(),
                expected.len(),
                "minor count {index} point {point}"
            );
            let a = c["result"]["range"][0].as_f64().unwrap();
            let b = c["result"]["range"][1].as_f64().unwrap();
            for (tick, value) in guide.minor_ticks.iter().zip(expected) {
                assert_eq!(tick.value, Some(ScaleValue::Number(value)));
                let position = if a == b {
                    320.
                } else {
                    100. + 440. * (value - a) / (b - a)
                };
                assert!((tick.position - position).abs() < 1e-8, "position {index}");
            }
            success += 1;
        }
    }
    assert_eq!((success, rejected), (600, 300));
}
