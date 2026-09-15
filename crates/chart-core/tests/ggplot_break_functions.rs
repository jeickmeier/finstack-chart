//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::grammar::{CustomScaleBreaks, ScaleBreaksInput, ScaleBreaksOutput};
use chart_core::{
    ChartResult, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
    interpolate::Number,
    scales::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
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
fn numeric_break_functions_match_2880_reference_builds() {
    assert_numeric_break_functions(false, false, false);
}
#[test]
fn binned_break_functions_match_2880_reference_builds() {
    assert_numeric_break_functions(true, false, false);
}
#[test]
fn numeric_named_breaks_match_default_labels() {
    assert_numeric_break_functions(false, true, false);
}
#[test]
fn binned_named_breaks_match_default_labels() {
    assert_numeric_break_functions(true, true, false);
}
#[test]
fn binned_constructor_guides_compose_with_registered_breaks() {
    assert_numeric_break_functions(true, false, true);
    assert_numeric_break_functions(true, true, true);
}
fn assert_numeric_break_functions(binned: bool, automatic: bool, constructor_guides: bool) {
    use chart_core::prelude::*;
    let fixture: Value = serde_json::from_str(match (binned, automatic) {
        (false, false) => {
            include_str!("../../../fixtures/parity/ggplot2/numeric-break-functions.json")
        }
        (true, false) => {
            include_str!("../../../fixtures/parity/ggplot2/binned-break-functions.json")
        }
        (false, true) => {
            include_str!("../../../fixtures/parity/ggplot2/numeric-break-default-names.json")
        }
        (true, true) => {
            include_str!("../../../fixtures/parity/ggplot2/binned-break-default-names.json")
        }
    })
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), if automatic { 80 } else { 2880 });
    let mut successes = 0;
    for case in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let labels = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_breaks(Arc::new(Breaks(
                calls.clone(),
                case["signature"].as_str().unwrap().into(),
                true,
            )))
            .unwrap();
        registry
            .register_guide_formatter(Arc::new(Formatter(labels.clone(), true)))
            .unwrap();
        let registry = Arc::new(registry);
        let family = match case["transform"].as_str().unwrap() {
            "sqrt" => json!({"Pow":{"exponent":0.5}}),
            "log10" => json!({"Log":{"base":10.}}),
            _ => json!("Linear"),
        };
        let mut spec: MappedScaleSpec=serde_json::from_value(json!({
            "training":"Eligible",
            "function":{"Interpolated":{
                "normalization":{"Ggplot":{"family":family,"domain":[0,1],"reverse":case["transform"]=="reverse","rescaler":"Range"}},
                "output":{"Interpolate":{"operation":"PowerRange","range":[1,6],"exponent":0.5,"absolute":false}},"unknown":{"kind":"Missing"}
            }},
            "ggplot":{"Continuous":{"limits":if case["limits"]=="full"{json!([1,10])}else{Value::Null},"oob":"Censor"}},
            "breaks_function":{"operation":{"id":"test.scale_breaks","version":"1"},"parameters":case["mode"]},
            "guide":{"Continuous":{"count":match case["count"].as_str().unwrap(){"three"=>json!(3),"zero"=>json!(0),_=>Value::Null},"labels":{"Registered":{"operation":{"id":"test.scale_labels","version":"1"},"parameters":"indexed"}}}}
        })).unwrap();
        if binned {
            spec.ggplot = Some(Box::new(GgplotScalePolicy::Binned(Box::new(
                GgplotBinnedPolicy {
                    limits: if case["limits"] == "full" {
                        Some([Some(Number(1.)), Some(Number(10.))])
                    } else {
                        None
                    },
                    breaks: GgplotBreaks::Equal(match case["count"].as_str().unwrap() {
                        "three" => 3.,
                        "zero" => 0.,
                        _ => 5.,
                    }),
                    ..Default::default()
                },
            ))));
            spec.guide = Some(Box::new(GgplotScaleGuide::Binned(
                match spec.guide.as_deref().unwrap() {
                    GgplotScaleGuide::Continuous(g) => g.labels.clone(),
                    _ => unreachable!(),
                },
            )));
        }
        if automatic {
            match spec.guide.as_deref_mut().unwrap() {
                GgplotScaleGuide::Continuous(g) => g.labels = GgplotGuideLabels::Automatic,
                GgplotScaleGuide::Binned(labels) => *labels = GgplotGuideLabels::Automatic,
                _ => unreachable!(),
            }
        }
        let colour = case["channel"] == "colour";
        if colour {
            let ColorScale::Mapped { scale: default, .. } = ggplot_color_default(true).unwrap()
            else {
                unreachable!()
            };
            let ScaleFunctionSpec::Interpolated(default) = default.function else {
                unreachable!()
            };
            let ScaleFunctionSpec::Interpolated(scale) = &mut spec.function else {
                unreachable!()
            };
            scale.output = default.output;
        }
        if constructor_guides {
            let Some(GgplotScaleGuide::Binned(labels)) = spec.guide.as_deref() else {
                unreachable!()
            };
            spec.guide = Some(Box::new(if colour {
                GgplotScaleGuide::BinnedSteps(labels.clone())
            } else {
                GgplotScaleGuide::BinnedBins(labels.clone())
            }));
        }
        let inputs = case["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", inputs.iter().map(Value::as_f64).collect::<Vec<_>>())
            .build()
            .unwrap();
        let result = (|| -> ChartResult<Value> {
            let draft = plot(data)
                .extensions(registry.clone())
                .profile(Profile::Ggplot2_4_0_3);
            let plot = if colour {
                draft
                    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                    .scale(color_mapped("v", spec))
                    .layer(points())
                    .build()?
            } else {
                draft
                    .aes(aes().x("x").y(1.))
                    .layer(points().numeric_scale(NumericAesthetic::Size, "v", spec))
                    .build()?
            };
            let wire = plot.to_json()?;
            assert_eq!(
                serde_json::from_str::<Value>(&wire).unwrap()["version"],
                if constructor_guides { 44 } else { 33 }
            );
            let restored = Plot::from_json_with_extensions(&wire, registry.clone())?;
            assert_eq!(restored.to_json()?, wire);
            assert!(calls.lock().unwrap().is_empty());
            assert!(labels.lock().unwrap().is_empty());
            let prepared = restored.chart()?.prepare()?;
            if binned {
                let trained = if colour {
                    prepared.layers()[0]
                        .color_legend()
                        .unwrap()
                        .mapping
                        .as_ref()
                        .unwrap()
                } else {
                    &prepared.layers()[0]
                        .numeric_scales()
                        .get(&NumericAesthetic::Size)
                        .unwrap()
                        .scale
                };
                let mapping = if colour {
                    MappedScale::for_colors_with_registry(trained.clone(), &registry)?
                } else {
                    MappedScale::new_with_registry(trained.clone(), &registry)?
                };
                let expected = case["result"]["mapped"].as_array().unwrap();
                assert_eq!(inputs.len(), expected.len(), "{case}");
                for (input, expected) in inputs.iter().zip(expected) {
                    if colour {
                        let actual = mapping.color(
                            input.as_f64(),
                            None,
                            chart_core::color::parse_r("grey50")?.resolve(),
                        )?;
                        assert_eq!(
                            actual,
                            chart_core::color::parse_r(expected.as_str().unwrap())?.resolve(),
                            "{case}"
                        );
                    } else {
                        let actual = mapping.numeric(input.as_f64())?;
                        let actual = match actual {
                            chart_core::interpolate::Value::Number(n) => encoded_number(n.0),
                            chart_core::interpolate::Value::Missing => Value::Null,
                            _ => panic!("numeric output"),
                        };
                        compare(&actual, expected);
                    }
                }
            }
            if colour {
                return Ok(
                    json!({"labels":prepared.layers()[0].color_legend().map(|l|if binned { l.numeric_breaks.iter().filter(|e|e.visible).map(|e|e.label.clone().unwrap_or_else(||"NA".into())).collect::<Vec<_>>() } else {l.entries.iter().map(|e|e.0.clone()).collect::<Vec<_>>()}).unwrap_or_default()}),
                );
            }
            let entries = prepared.layers()[0]
                .numeric_value_guides()
                .values()
                .flatten()
                .filter(|e| e.visible)
                .collect::<Vec<_>>();
            Ok(
                json!({"values":entries.iter().map(|e|encoded_number(e.transformed.0)).collect::<Vec<_>>(),"labels":entries.iter().map(|e|e.label.clone()).collect::<Vec<_>>()}),
            )
        })();
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}: {result:?}");
        } else {
            successes += 1;
            let actual = result.unwrap_or_else(|e| panic!("{case}: {e:?}"));
            if colour {
                let expected = case["result"]["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|k| k["labels"].as_array().unwrap().iter().cloned())
                    .collect::<Vec<_>>();
                compare(&actual["labels"], &json!(expected));
            } else {
                let mut expected = case["result"]["keys"]
                    .as_array()
                    .unwrap()
                    .first()
                    .cloned()
                    .unwrap_or(json!({"values":[],"labels":[]}));
                if binned {
                    expected["values"] = case["result"]["boundaries"].clone();
                    if expected["values"].as_array().unwrap().len() == 1 {
                        let value = expected["values"][0].clone();
                        expected["values"] = json!([value, value]);
                    }
                }
                compare(&actual, &expected);
            }
        }
        compare(&json!(*calls.lock().unwrap()), &case["calls"]);
        compare(&json!(*labels.lock().unwrap()), &case["label_calls"]);
    }
    assert_eq!(
        successes,
        if automatic {
            if binned { 36 } else { 80 }
        } else if binned {
            1800
        } else {
            2448
        }
    );
}
fn compare(a: &Value, b: &Value) {
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        assert_eq!(a.len(), b.len(), "{a:?} != {b:?}");
        for (a, b) in a.iter().zip(b) {
            compare(a, b);
        }
    } else if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        assert_eq!(a.len(), b.len());
        for (k, v) in a {
            compare(v, &b[k]);
        }
    } else {
        equal(a, b);
    }
}

#[test]
fn break_registration_portability_and_wire_version_precede_evaluation() {
    use chart_core::grammar::{OperationRef, ScaleBreaksOperation};
    use chart_core::prelude::*;
    for portable in [true, false] {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_breaks(Arc::new(Breaks(calls.clone(), "n".into(), portable)))
            .unwrap();
        assert!(
            registry
                .register_scale_breaks(Arc::new(Breaks(calls.clone(), "n".into(), portable)))
                .is_err()
        );
        let registry = Arc::new(registry);
        let ColorScale::Mapped { scale, .. } = ggplot_color_default(true).unwrap() else {
            unreachable!()
        };
        let scale = scale
            .with_breaks_function(ScaleBreaksOperation {
                operation: OperationRef::new("test.scale_breaks", Revision::new(1)),
                parameters: json!("domain"),
            })
            .unwrap();
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
        if portable {
            let wire = plot.to_json().unwrap();
            assert!(Plot::from_json(&wire).is_err());
            let mut wrong: Value = serde_json::from_str(&wire).unwrap();
            wrong["version"] = 32.into();
            assert!(Plot::from_json_with_extensions(&wrong.to_string(), registry).is_err());
        } else {
            assert_eq!(
                plot.to_json().unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
        }
        assert!(make(Arc::new(ExtensionRegistry::new())).is_err());
        assert!(calls.lock().unwrap().is_empty());
    }
}

struct DiscreteBreaks(Arc<Mutex<Vec<Value>>>);
impl CustomScaleBreaks for DiscreteBreaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.discrete_breaks", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        let values = input
            .domain
            .iter()
            .map(|k| match k {
                ScaleKey::Text(s) => json!(s),
                ScaleKey::Null => Value::Null,
                _ => panic!("text domain"),
            })
            .collect::<Vec<_>>();
        assert!(input.count.is_none());
        self.0
            .lock()
            .unwrap()
            .push(json!({"values":values,"names":[]}));
        let (values, names) = match input.parameters.as_str().unwrap() {
            "domain" => (Some(input.domain.to_vec()), None),
            "mixed" => (
                Some(
                    [Some("z"), Some("b"), Some("b"), None, Some("a")]
                        .map(|s| s.map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into())))
                        .to_vec(),
                ),
                Some(["Z", "B", "B2", "M", "A"].map(String::from).to_vec()),
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
#[test]
fn discrete_break_functions_match_600_reference_builds() {
    use chart_core::grammar::ValueAesthetic;
    use chart_core::prelude::*;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-break-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 600);
    for case in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let labels = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_breaks(Arc::new(DiscreteBreaks(calls.clone())))
            .unwrap();
        registry
            .register_guide_formatter(Arc::new(Formatter(labels.clone(), true)))
            .unwrap();
        let registry = Arc::new(registry);
        let channel = case["channel"].as_str().unwrap();
        let palette = match channel {
            "colour" => {
                json!({"Hue":{"h":[15,375],"chroma":100,"luminance":65,"start":0,"reverse":false}})
            }
            "shape" => json!({"Shape":{"solid":true}}),
            "linetype" => json!("LineType"),
            _ => {
                json!({"NumericRange":{"range":if channel=="alpha"{[0.1,1.]}else{[2.,6.]},"area":channel=="size"}})
            }
        };
        let spec:MappedScaleSpec=serde_json::from_value(json!({
            "training":"Eligible","function":{"Ordinal":{"domain":[],"range":[],"unknown":{"Explicit":null}}},
            "ggplot":{"Discrete":{"limits":if case["limits"]=="explicit"{json!([{"Text":"c"},{"Text":"b"},{"Text":"a"},"Null"])}else{Value::Null},"levels":null,"drop":true,"na_translate":true,"palette":palette}},
            "breaks_function":{"operation":{"id":"test.discrete_breaks","version":"1"},"parameters":case["mode"]},
            "guide":{"Discrete":{"labels":if case["label_mode"]=="default"{json!("Automatic")}else{json!({"Registered":{"operation":{"id":"test.scale_labels","version":"1"},"parameters":"indexed"}})}}}
        })).unwrap();
        let inputs = case["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", inputs.iter().map(Value::as_str).collect::<Vec<_>>())
            .build()
            .unwrap();
        let draft = plot(data)
            .extensions(registry.clone())
            .profile(Profile::Ggplot2_4_0_3);
        let plot = if channel == "colour" {
            draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", spec))
                .layer(points())
                .build()
        } else {
            let layer = match channel {
                "size" => points().numeric_scale(NumericAesthetic::Size, "v", spec),
                "alpha" => points().numeric_scale(NumericAesthetic::Alpha, "v", spec),
                "linewidth" => line().numeric_scale(NumericAesthetic::StrokeWidth, "v", spec),
                "shape" => points().value_scale(ValueAesthetic::Shape, "v", spec),
                "linetype" => line().value_scale(ValueAesthetic::LineType, "v", spec),
                _ => unreachable!(),
            };
            draft.aes(aes().x("x").y(1.)).layer(layer).build()
        }
        .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let wire = plot.to_json().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 33);
        let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
        assert_eq!(wire, restored.to_json().unwrap());
        assert!(calls.lock().unwrap().is_empty());
        assert!(labels.lock().unwrap().is_empty());
        let prepared = restored
            .chart()
            .unwrap()
            .prepare()
            .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let expected = case["result"]["keys"]
            .as_array()
            .unwrap()
            .first()
            .cloned()
            .unwrap_or(json!({"values":[],"labels":[]}));
        if channel == "colour" {
            let actual = prepared.layers()[0]
                .color_legend()
                .map(|l| l.entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>())
                .unwrap_or_default();
            let expected = expected["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap_or("NA"))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{case}");
        } else {
            let entries = prepared.layers()[0]
                .discrete_value_guides()
                .values()
                .flatten()
                .collect::<Vec<_>>();
            let actual = json!({"values":entries.iter().map(|e|match &e.key {ScaleKey::Text(s)=>json!(s),ScaleKey::Null=>Value::Null,_=>panic!("text key")}).collect::<Vec<_>>(),"labels":entries.iter().map(|e|e.label.clone()).collect::<Vec<_>>()});
            assert_eq!(actual, expected, "{case}");
        }
        assert_eq!(json!(*calls.lock().unwrap()), case["calls"], "{case}");
        assert_eq!(
            json!(*labels.lock().unwrap()),
            case["label_calls"],
            "{case}"
        );
    }
}

#[test]
fn binned_count_prefers_n_breaks_and_consumes_one_result() {
    struct Both(Arc<Mutex<usize>>);
    impl CustomScaleBreaks for Both {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("test.both_counts", Revision::new(1), true)
        }
        fn accepts_n(&self) -> bool {
            true
        }
        fn accepts_n_breaks(&self) -> bool {
            true
        }
        fn validate(&self, _: &Value) -> ChartResult<()> {
            Ok(())
        }
        fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
            assert_eq!(input.count_argument, Some("n.breaks"));
            assert_eq!(input.count, Some(3.));
            *self.0.lock().unwrap() += 1;
            Ok(ScaleBreaksOutput {
                temporal: None,
                values: Some(input.domain.to_vec()),
                names: None,
            })
        }
    }
    let calls = Arc::new(Mutex::new(0));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_breaks(Arc::new(Both(calls.clone())))
        .unwrap();
    let source: MappedScaleSpec = serde_json::from_value(json!({
        "training":"Eligible",
        "function":{"Interpolated":{
            "normalization":{"Ggplot":{"family":"Linear","domain":[0,1],"reverse":false,"rescaler":"Range"}},
            "output":{"Interpolate":{"operation":"PowerRange","range":[1,6],"exponent":0.5,"absolute":false}},"unknown":{"kind":"Missing"}
        }},
        "ggplot":{"Binned":{"limits":null,"oob":"Squish","breaks":{"Equal":3},"right":true}},
        "breaks_function":{"operation":{"id":"test.both_counts","version":"1"},"parameters":null}
    })).unwrap();
    let original = serde_json::to_string(&source).unwrap();
    let trained = source
        .trained_with_registry(&[Some(Number(1.)), Some(Number(10.))], &registry)
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), 1);
    let mapping = MappedScale::new_with_registry(trained, &registry).unwrap();
    for _ in 0..2 {
        assert_eq!(
            mapping.numeric(Some(4.)).unwrap(),
            chart_core::interpolate::Value::Number(Number(1. + 5. * 0.5f64.sqrt()))
        );
        mapping.binned_value_guide_entries(4096, 1_048_576).unwrap();
    }
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(serde_json::to_string(&source).unwrap(), original);
    let mut invalid = source;
    if let Some(GgplotScalePolicy::Binned(p)) = invalid.ggplot.as_deref_mut() {
        p.breaks = GgplotBreaks::Explicit(vec![Number(2.)]);
    }
    assert!(invalid.trained_with_registry(&[], &registry).is_err());
    assert_eq!(*calls.lock().unwrap(), 1);
}
