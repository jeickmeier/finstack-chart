//! FIX-GG04: count palettes against independently captured public constructor builds.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
struct Palette(Arc<Mutex<Vec<usize>>>);
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Count(count) = input.domain else {
            panic!("discrete input expected")
        };
        self.0.lock().unwrap().push(count);
        let mode = input.parameters["mode"].as_str().unwrap();
        let channel = input.parameters["channel"].as_str().unwrap();
        let mut values = (1..=count)
            .map(|i| {
                let t = i as f64 / count.max(1) as f64;
                match channel {
                    "colour" => Value::Text(if t < 0.5 { "#ff0000" } else { "#0000ff" }.into()),
                    "size" => Value::Number(Number(1. + 4. * t)),
                    _ => Value::Number(Number(t)),
                }
            })
            .collect::<Vec<_>>();
        match mode {
            "short" => values.truncate(1),
            "empty" => values.clear(),
            "missing" if values.len() > 1 => values[1] = Value::Missing,
            _ => {}
        }
        let names = (mode == "named").then(|| {
            (0..values.len())
                .map(|i| ["c", "b", "a", "d"][i % 4].to_owned())
                .collect()
        });
        Ok(ScalePaletteOutput {
            values: (mode != "null").then_some(values),
            names,
        })
    }
}
fn spec(c: &Json) -> MappedScaleSpec {
    let mut result = MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
        .with_ggplot(GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: (c["limits"] == "full").then(|| {
                ["a", "b", "c", "d"]
                    .map(|s| ScaleKey::Text(s.into()))
                    .to_vec()
            }),
            levels: None,
            drop: true,
            na_translate: true,
            palette: GgplotDiscretePalette::Hue(Default::default()),
        })
        .unwrap()
        .with_palette_function(ScalePaletteOperation {
            operation: OperationRef {
                id: "test.palette".into(),
                version: Revision::new(1),
            },
            parameters: json!({"channel":c["channel"],"mode":c["mode"]}),
        })
        .unwrap();
    if c["guide_mode"] == "hidden" {
        result.guide = Some(Box::new(GgplotScaleGuide::Hidden));
    }
    if c["guide_mode"] == "first" {
        result.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
            breaks: Some(vec![ScaleKey::Text("a".into())]),
            ..Default::default()
        })));
    }
    result.missing_paint_is_na = c["channel"] == "colour";
    result
}
fn key(j: &Json) -> ScaleKey {
    j.as_str()
        .map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into()))
}
fn encode(v: Value) -> Json {
    match v {
        Value::Missing => Json::Null,
        Value::Text(s) => json!(s),
        Value::Number(n) => json!(n.0),
        _ => panic!("palette primitive"),
    }
}
#[test]
fn discrete_palette_functions_match_180_public_constructor_cases() {
    assert_cases(
        include_str!("../../../fixtures/parity/ggplot2/palette-functions.json"),
        180,
    );
}
#[test]
fn discrete_palette_lookup_matches_360_hidden_and_restricted_guide_cases() {
    assert_cases(
        include_str!("../../../fixtures/parity/ggplot2/discrete-palette-lookup.json"),
        360,
    );
}
fn assert_cases(source: &str, expected_count: usize) {
    let fixture: Json = serde_json::from_str(source).unwrap();
    let mut count = 0;
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["family"] == "discrete")
    {
        count += 1;
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let authored = spec(c);
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(key)
            .collect::<Vec<_>>();
        MappedScale::new_with_registry(authored.clone(), &registry).unwrap();
        assert!(calls.lock().unwrap().is_empty());
        let result = (|| -> ChartResult<Json> {
            let trained = authored.trained_keys_with_registry(&inputs, &registry)?;
            let mapping = MappedScale::new_with_registry(trained, &registry)?;
            let mapped = inputs
                .iter()
                .map(|k| mapping.category(Some(k)).map(encode))
                .collect::<ChartResult<Vec<_>>>()?;
            let entries = mapping.discrete_guide_entries()?.unwrap_or_default();
            let mut values = vec![];
            let mut labels = vec![];
            let mut colors = vec![];
            for e in entries {
                values.push(match &e.key {
                    ScaleKey::Null => Json::Null,
                    ScaleKey::Text(s) => json!(s),
                    _ => panic!("text key"),
                });
                labels.push(e.label);
                colors.push(encode(mapping.category(Some(&e.key))?));
            }
            Ok(
                json!({"mapped":mapped,"keys":if values.is_empty(){json!([])}else{json!([{"values":values,"labels":labels,"mapped":colors}])}}),
            )
        })();
        if c["result"].get("error").is_some() {
            assert!(result.is_err(), "{c}: unexpected preparation outcome");
        } else {
            compare(&result.unwrap(), &c["result"]);
        }
        let expected = c["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["values"][0].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        assert_eq!(*calls.lock().unwrap(), expected, "{c}");
    }
    assert_eq!(count, expected_count);
}

fn compare(actual: &Json, expected: &Json) {
    match (actual, expected) {
        (Json::Number(a), Json::Number(b)) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 5e-14,
            "{actual} != {expected}"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                compare(a, b);
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                compare(v, &b[k]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn primary_palette_roundtrip_prepares_all_540_reference_outcomes() {
    use chart_core::prelude::*;
    for source in [
        include_str!("../../../fixtures/parity/ggplot2/palette-functions.json"),
        include_str!("../../../fixtures/parity/ggplot2/discrete-palette-lookup.json"),
    ] {
        let fixture: Json = serde_json::from_str(source).unwrap();
        for c in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["family"] == "discrete")
        {
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::new();
            registry
                .register_scale_palette(Arc::new(Palette(calls.clone())))
                .unwrap();
            let registry = Arc::new(registry);
            let inputs = c["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    inputs
                        .iter()
                        .map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>(),
                )
                .build()
                .unwrap();
            let draft = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry.clone());
            let p = if c["channel"] == "colour" {
                draft
                    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                    .scale(color_mapped("v", spec(c)))
                    .layer(points())
                    .build()
                    .unwrap()
            } else {
                draft
                    .aes(aes().x("x").y(1.))
                    .layer(points().numeric_scale(
                        if c["channel"] == "size" {
                            NumericAesthetic::Size
                        } else {
                            NumericAesthetic::Alpha
                        },
                        "v",
                        spec(c),
                    ))
                    .build()
                    .unwrap()
            };
            let wire = p.to_json().unwrap();
            assert_eq!(serde_json::from_str::<Json>(&wire).unwrap()["version"], 37);
            assert!(calls.lock().unwrap().is_empty());
            assert!(Plot::from_json(&wire).is_err());
            let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            let result = restored.chart().unwrap().prepare();
            if c["channel"] == "colour"
                && let Ok(prepared) = &result
            {
                let actual = prepared.layers()[0]
                    .marks()
                    .iter()
                    .map(|m| m.style.color)
                    .collect::<Vec<_>>();
                let expected = c["result"]["mapped"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(Json::as_str)
                    .map(|v| chart_core::color::parse_r(v).unwrap().resolve())
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected, "{c}");
            }
            assert_eq!(
                result.is_err(),
                c["result"].get("error").is_some(),
                "{c}: unexpected preparation outcome"
            );
        }
    }
}

#[test]
fn palette_na_elements_and_missing_input_replacements_match_54_source_draws() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-palette-missing-paint.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 54);
    for c in cases {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(Arc::new(Mutex::new(vec![])))))
            .unwrap();
        let inputs = c["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "v",
                inputs
                    .iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let mut scale = color_mapped("v", spec(c));
        if c["na_mode"] != "NA" {
            scale = scale.missing(
                chart_core::color::parse_r(if c["na_mode"] == "green" {
                    "#00ff00"
                } else {
                    "#00000000"
                })
                .unwrap(),
            );
        }
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(scale)
            .layer(points())
            .build()
            .unwrap();
        let result = p.chart().unwrap().prepare();
        if c["result"].get("error").is_some() {
            assert!(result.is_err(), "{c}");
            continue;
        }
        let prepared = result.unwrap();
        let layer = &prepared.layers()[0];
        let marks = layer.marks();
        assert_eq!(
            marks.len(),
            c["result"]["point_count"].as_u64().unwrap() as usize,
            "{c}"
        );
        let expected = c["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Json::as_str)
            .map(|s| chart_core::color::parse_r(s).unwrap().resolve())
            .collect::<Vec<_>>();
        assert_eq!(
            marks.iter().map(|m| m.style.color).collect::<Vec<_>>(),
            expected,
            "{c}"
        );
        let entries = &layer.color_legend().unwrap().entries;
        let expected = c["result"]["keys"][0]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                v.as_str().map_or(
                    chart_core::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    },
                    |v| chart_core::color::parse_r(v).unwrap().resolve(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            entries.iter().map(|e| e.1).collect::<Vec<_>>(),
            expected,
            "{c}"
        );
    }
}

#[test]
fn malformed_palette_results_and_fallback_metadata_are_rejected() {
    struct Invalid(bool);
    impl CustomScalePalette for Invalid {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("test.palette", Revision::new(1), true)
        }
        fn validate(&self, _: &Json) -> ChartResult<()> {
            Ok(())
        }
        fn evaluate(&self, _: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
            Ok(if self.0 {
                ScalePaletteOutput {
                    values: Some(vec![Value::Missing]),
                    names: Some(vec![]),
                }
            } else {
                ScalePaletteOutput {
                    values: Some(vec![
                        Value::Missing;
                        chart_core::interpolate::MAX_VALUES + 1
                    ]),
                    names: None,
                }
            })
        }
    }
    let source = spec(&json!({"limits":"none","channel":"colour","mode":"full"}));
    for malformed_names in [true, false] {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Invalid(malformed_names)))
            .unwrap();
        let error = source
            .trained_keys_with_registry(&[ScaleKey::Text("a".into())], &registry)
            .unwrap_err();
        assert_eq!(
            error.code,
            if malformed_names {
                chart_core::DiagnosticCode::Validation
            } else {
                chart_core::DiagnosticCode::ResourceLimit
            }
        );
    }
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_palette(Arc::new(Palette(Arc::new(Mutex::new(vec![])))))
        .unwrap();
    let mut forged = source
        .trained_keys_with_registry(&[ScaleKey::Text("a".into())], &registry)
        .unwrap();
    forged.palette_fallback_indices = vec![1];
    assert!(MappedScale::new_with_registry(forged, &registry).is_err());
}
