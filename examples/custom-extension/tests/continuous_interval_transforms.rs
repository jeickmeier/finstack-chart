//! FIX-GG04: guide sampling uses the trained scale-space batch, including vector transforms.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
fn number(value: f64) -> Json {
    if value.is_nan() {
        Json::Null
    } else if value.is_infinite() {
        json!(if value > 0. { "Infinity" } else { "-Infinity" })
    } else {
        json!(value)
    }
}
fn equivalent(a: &Json, b: &Json) -> bool {
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        (a - b).abs() <= 4e-14 * a.abs().max(b.abs()).max(1.)
    } else if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
    } else {
        a == b
    }
}
struct Palette(Arc<Mutex<Vec<Json>>>);
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.interval", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Normalized(values) = input.domain else {
            unreachable!()
        };
        self.0.lock().unwrap().push(json!(
            values.iter().map(|v| number(v.0)).collect::<Vec<_>>()
        ));
        Ok(ScalePaletteOutput {
            names: None,
            values: Some(
                values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let v = Number(if input.parameters["mode"] == "index" {
                            (i + 1) as f64 / values.len() as f64
                        } else {
                            v.0
                        });
                        if v.0.is_nan() {
                            Value::Missing
                        } else if input.parameters["channel"] == "colour" {
                            Value::Text(if v.0 < 0.5 { "#ff0000" } else { "#0000ff" }.into())
                        } else {
                            Value::Number(Number(if input.parameters["channel"] == "size" {
                                1. + 4. * v.0
                            } else {
                                v.0
                            }))
                        }
                    })
                    .collect(),
            ),
        })
    }
}
#[test]
fn transformed_interval_keys_match_108_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-interval-transforms.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 108);
    for case in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = chart_extension_example::registry().unwrap();
        Arc::get_mut(&mut registry)
            .unwrap()
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let channel = case["channel"].as_str().unwrap();
        let transform = if case["transform"] == "log10" {
            json!({"Log":{"base":10}})
        } else {
            json!({"Registered":{"selection":{"call":{"operation":{"id":"example.scale_transform_vector","version":"1"},"parameters":{"family":case["transform"]}}}}})
        };
        let guide = if case["guide"] == "bins" {
            "ContinuousBins"
        } else {
            "ContinuousSteps"
        };
        let spec:MappedScaleSpec=serde_json::from_value(json!({"training":"Eligible","missing_paint_is_na":true,
          "guide":{guide:{"breaks":if case["breaks"]=="uneven"{json!([1,2,8,16])}else{Json::Null}}},
          "function":{"Interpolated":{"normalization":{"Ggplot":{"family":{"Ggplot":{"transform":transform}},"domain":[1,16],"reverse":false,"rescaler":"Range"}},"output":{"Interpolate":{"operation":"PowerRange","range":[0,1],"exponent":1,"absolute":false}},"unknown":{"kind":"Missing"}}},
          "ggplot":{"Continuous":{"limits":[1,16],"oob":"Censor"}},"palette_function":{"operation":{"id":"test.interval","version":"1"},"parameters":{"channel":channel,"mode":case["palette_mode"]}}})).unwrap();
        let values = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| match v.as_str() {
                Some("Infinity") => Some(f64::INFINITY),
                Some("-Infinity") => Some(f64::NEG_INFINITY),
                _ => v.as_f64(),
            })
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", values)
            .build()
            .unwrap();
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry);
        let draft = if channel == "colour" {
            draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", spec))
                .layer(points())
        } else {
            draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
                if channel == "size" {
                    NumericAesthetic::Size
                } else {
                    NumericAesthetic::Alpha
                },
                "v",
                spec,
            ))
        };
        let prepared = draft
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let recorded = calls.lock().unwrap().clone();
        assert!(
            equivalent(&json!(recorded), &case["calls"]),
            "calls {recorded:?}: {case}"
        );
        let layer = &prepared.layers()[0];
        if case["palette_mode"] == "index" {
            let expected = case["result"]["mapped"].as_array().unwrap();
            assert_eq!(layer.marks().len(), expected.len());
            for (mark, expected) in layer.marks().iter().zip(expected) {
                match channel {
                    "colour" => assert_eq!(
                        mark.style.color,
                        chart_core::color::parse_r(expected.as_str().unwrap())
                            .unwrap()
                            .resolve()
                    ),
                    "size" => assert!(
                        equivalent(&number(mark.style.radius), expected),
                        "indexed size: {case}"
                    ),
                    _ => assert_eq!(
                        mark.style.color.alpha,
                        (expected.as_f64().unwrap() * 255.).round() as u8
                    ),
                }
            }
        }
        let entries = if channel == "colour" {
            layer
                .color_legend()
                .map(|g| g.numeric_breaks.clone())
                .unwrap_or_default()
        } else {
            layer
                .numeric_value_guides()
                .values()
                .flatten()
                .cloned()
                .collect()
        };
        let entries = entries.iter().filter(|e| e.visible).collect::<Vec<_>>();
        let guides = case["result"]["guides"].as_array().unwrap();
        if guides.is_empty() {
            assert!(entries.is_empty(), "{case}");
            continue;
        }
        assert!(
            equivalent(
                &json!(
                    entries
                        .iter()
                        .map(|e| number(e.transformed.0))
                        .collect::<Vec<_>>()
                ),
                &guides[0]["scale_values"]
            ),
            "scale values {:?}: {case}",
            entries
        );
        assert_eq!(
            json!(entries.iter().map(|e| e.label.clone()).collect::<Vec<_>>()),
            guides[0]["labels"],
            "{case}"
        );
        let mapped = entries
            .iter()
            .map(|e| match e.mapped.as_ref().unwrap() {
                Value::Missing | Value::Null => Json::Null,
                Value::Number(v) => number(v.0),
                Value::Text(v) => json!(v),
                v => panic!("{v:?}"),
            })
            .collect::<Vec<_>>();
        assert!(
            equivalent(&json!(mapped), &guides[0]["mapped"]),
            "mapped keys: {case}"
        );
    }
}
