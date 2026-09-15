//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::{
    ChartResult, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
    interpolate::Number,
    prelude::*,
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
#[test]
fn binned_label_callbacks_match_reference_inputs_and_primary_keys() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-label-functions.json"
    ))
    .unwrap();
    let mut passed = 0;
    let mut rejected = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::default();
        registry
            .register_guide_formatter(Arc::new(Formatter(calls.clone(), true)))
            .unwrap();
        let inputs = c["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "v",
                chart_core::plot::column(
                    inputs
                        .iter()
                        .map(|v| v.as_f64().unwrap_or(0.))
                        .collect::<Vec<_>>(),
                )
                .validity(inputs.iter().map(|v| !v.is_null()).collect()),
            )
            .build()
            .unwrap();
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
            unreachable!()
        };
        let ScaleFunctionSpec::Interpolated(mapping) = &mut scale.function else {
            unreachable!()
        };
        mapping.normalization = NormalizationSpec::Ggplot {
            family: match c["transform"].as_str().unwrap() {
                "sqrt" => NumericFamily::Pow { exponent: 0.5 },
                "log10" => NumericFamily::Log { base: 10. },
                _ => NumericFamily::Linear,
            },
            domain: [Number(0.), Number(1.)],
            reverse: c["transform"] == "reverse",
            rescaler: GgplotRescaler::Range,
            timestamp: None,
        };
        scale.ggplot = Some(Box::new(GgplotScalePolicy::Binned(Box::new(
            GgplotBinnedPolicy {
                limits: (c["limits"] == "full").then_some([Some(Number(1.)), Some(Number(10.))]),
                breaks: match c["break_mode"].as_str().unwrap() {
                    "explicit" => GgplotBreaks::Explicit(
                        [-1., 0., 1., 1., 3., 20., f64::INFINITY, f64::NAN]
                            .map(Number)
                            .to_vec(),
                    ),
                    "empty" => GgplotBreaks::Explicit(vec![]),
                    "equal" => GgplotBreaks::Equal(5.),
                    _ => GgplotBreaks::Nice(5.),
                },
                ..Default::default()
            },
        ))));
        scale.guide = Some(Box::new(GgplotScaleGuide::Binned(
            GgplotGuideLabels::Registered {
                operation: chart_core::grammar::OperationRef::new(
                    "test.scale_labels",
                    Revision::new(1),
                ),
                parameters: c["label_mode"].clone(),
            },
        )));
        let direct = scale
            .trained_with_registry(
                &inputs
                    .iter()
                    .map(|v| v.as_f64().map(Number))
                    .collect::<Vec<_>>(),
                &registry,
            )
            .and_then(|scale| MappedScale::new_with_registry(scale, &registry))
            .and_then(|scale| scale.binned_guide_entries(4096, 1_048_576));
        if c["direct"]["error"].is_string() {
            assert!(direct.is_err(), "direct should reject: {c}");
        } else {
            let direct = direct
                .unwrap_or_else(|e| panic!("direct {c}: {e:?}"))
                .unwrap();
            let expected = c["direct"]["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>();
            assert_eq!(
                direct.iter().map(|e| e.label.clone()).collect::<Vec<_>>(),
                expected,
                "direct {c}"
            );
        }
        {
            let recorded = calls.lock().unwrap();
            let expected = c["direct_calls"].as_array().unwrap();
            assert_eq!(recorded.len(), expected.len(), "direct callback count {c}");
            for (a, b) in recorded.iter().zip(expected) {
                assert_eq!(a["names"], b["names"], "direct {c}");
                let av = a["values"].as_array().unwrap();
                let bv = b["values"].as_array().unwrap();
                assert_eq!(av.len(), bv.len(), "direct {c}");
                for (a, b) in av.iter().zip(bv) {
                    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                        assert!(
                            (a - b).abs() <= 3e-12 * b.abs().max(1.),
                            "direct {c}: {a} {b}"
                        );
                    } else {
                        assert_eq!(a, b, "direct {c}");
                    }
                }
            }
        }
        calls.lock().unwrap().clear();
        let result = (|| {
            let p = plot(data)
                .extensions(Arc::new(registry))
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points())
                .build()?;
            let prepared = p.chart()?.prepare()?;
            Ok::<_, chart_core::Diagnostic>(
                prepared.layers()[0]
                    .color_legend()
                    .map(|g| {
                        g.numeric_breaks
                            .iter()
                            .filter(|e| e.visible)
                            .map(|e| e.label.clone().unwrap_or_else(|| "NA".into()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            )
        })();
        let recorded = calls.lock().unwrap();
        let expected_calls = c["calls"].as_array().unwrap();
        if expected_calls.is_empty() {
            assert!(recorded.is_empty(), "{c} {recorded:?}");
        } else {
            assert!(!recorded.is_empty(), "missing call {c}: {result:?}");
            for call in recorded.iter() {
                let expected = &expected_calls[0];
                assert_eq!(call["names"], expected["names"], "{c}");
                let a = call["values"].as_array().unwrap();
                let b = expected["values"].as_array().unwrap();
                assert_eq!(a.len(), b.len(), "{c}");
                for (a, b) in a.iter().zip(b) {
                    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                        assert!((a - b).abs() < 3e-12 * b.abs().max(1.), "{c}: {a} {b}");
                    } else {
                        assert_eq!(a, b, "{c}");
                    }
                }
            }
        }
        if c["result"]["error"].is_string() {
            assert!(result.is_err(), "expected rejection {c}");
            rejected += 1;
        } else {
            let expected = c["result"]["keys"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|k| {
                    k["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap_or("NA").to_owned())
                })
                .collect::<Vec<_>>();
            assert_eq!(
                result.unwrap_or_else(|e| panic!("{c}: {e:?}")),
                expected,
                "{c}"
            );
            passed += 1;
        }
    }
    assert_eq!((passed, rejected), (378, 262));
}
