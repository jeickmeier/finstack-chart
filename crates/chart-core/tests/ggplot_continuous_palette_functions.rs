//! FIX-GG04: vector palettes against independently captured public constructor builds.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
struct Palette(Arc<Mutex<Vec<Vec<Number>>>>);
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Normalized(samples) = input.domain else {
            panic!("normalized input expected")
        };
        self.0.lock().unwrap().push(samples.to_vec());
        let mode = input.parameters["mode"].as_str().unwrap();
        let channel = input.parameters["channel"].as_str().unwrap();
        let mut values = samples
            .iter()
            .enumerate()
            .map(|(i, Number(sample))| {
                let t = match mode {
                    "index" => (i + 1) as f64 / samples.len() as f64,
                    "length" => samples.len() as f64 / 10.,
                    "first" => samples[0].0,
                    _ => *sample,
                };
                if t.is_nan() {
                    return Value::Missing;
                }
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
    let policy = GgplotScalePolicy::Continuous {
        empty_population: false,
        nonfinite_population: false,
        limits: (c["limits"] == "full" || c["inputs"][0].is_array())
            .then_some([Some(Number(1.)), Some(Number(10.))]),
        oob: GgplotOob::Censor,
    };
    let mut result =
        MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
            normalization: NormalizationSpec::Ggplot {
                timestamp: None,
                family: NumericFamily::Linear,
                domain: [Number(0.), Number(1.)],
                reverse: false,
                rescaler: GgplotRescaler::Range,
            },
            output: ScaleRangeFunction::Identity,
            unknown: Value::Missing,
        }))
        .with_ggplot(policy)
        .unwrap()
        .with_guide(if c["guide_mode"] == "hidden" {
            GgplotScaleGuide::Hidden
        } else {
            GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                breaks: match c["guide_mode"].as_str() {
                    Some("first") => Some(vec![Number(1.)]),
                    Some("restricted") => Some([10., 1., 4.].map(Number).to_vec()),
                    _ => None,
                },
                ..Default::default()
            })
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
    result.missing_paint_is_na = c["channel"] == "colour";
    result
}
fn encode(v: Value) -> Json {
    match v {
        Value::Missing => Json::Null,
        Value::Text(s) => json!(s),
        Value::Number(n) => json!(n.0),
        _ => panic!("palette primitive"),
    }
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
fn continuous_vector_palettes_match_540_reference_builds() {
    let mut count = 0;
    for source in [
        include_str!("../../../fixtures/parity/ggplot2/palette-functions.json"),
        include_str!("../../../fixtures/parity/ggplot2/vector-palette-lookup.json"),
    ] {
        let fixture: Json = serde_json::from_str(source).unwrap();
        for c in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["family"] == "continuous")
        {
            count += 1;
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::new();
            registry
                .register_scale_palette(Arc::new(Palette(calls.clone())))
                .unwrap();
            let inputs = c["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().map(Number))
                .collect::<Vec<_>>();
            let result = (|| -> ChartResult<Json> {
                let trained = spec(c).trained_with_registry(&inputs, &registry)?;
                let scale = MappedScale::new_with_registry(trained, &registry)?;
                let entries = scale
                    .continuous_guide_entries(4096, 65536)?
                    .unwrap_or_default();
                let has_candidates = !entries.is_empty();
                let mut values = vec![];
                let mut labels = vec![];
                let mut colors = vec![];
                for e in entries.into_iter().filter(|e| e.visible) {
                    values.push(e.value.0);
                    labels.push(e.label);
                    colors.push(encode(e.mapped.unwrap()));
                }
                let mapped = scale
                    .numeric_batch(&inputs.iter().map(|v| v.map(|v| v.0)).collect::<Vec<_>>())?
                    .map(|values| values.into_iter().map(encode).collect::<Vec<_>>())
                    .unwrap_or_else(|| {
                        vec![
                            match c["channel"].as_str().unwrap() {
                                "colour" => json!("black"),
                                "size" => json!(1.5),
                                _ => Json::Null,
                            };
                            inputs.len()
                        ]
                    });
                Ok(
                    json!({"mapped":mapped,"keys":if !has_candidates{json!([])}else{json!([{"values":values,"labels":labels,"mapped":colors}])}}),
                )
            })();
            if c["result"].get("error").is_some() {
                assert!(result.is_err(), "{c}: {result:?}");
            } else {
                let actual = result.unwrap_or_else(|e| panic!("{c}: {e:?}"));
                compare(&actual, &c["result"]);
            }
            let actual:Vec<_>=calls.lock().unwrap().iter().map(|v|json!({"values":v.iter().map(|v|json!(v.0)).collect::<Vec<_>>(),"names":[]})).collect();
            compare(&json!(actual), &c["calls"]);
        }
    }
    assert_eq!(count, 540);
}

#[test]
fn primary_continuous_palette_roundtrip_matches_540_reference_outcomes() {
    use chart_core::prelude::*;
    for source in [
        include_str!("../../../fixtures/parity/ggplot2/palette-functions.json"),
        include_str!("../../../fixtures/parity/ggplot2/vector-palette-lookup.json"),
    ] {
        let fixture: Json = serde_json::from_str(source).unwrap();
        for c in fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["family"] == "continuous")
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
                .column("v", inputs.iter().map(|v| v.as_f64()).collect::<Vec<_>>())
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
            if c["channel"] != "colour"
                && let Ok(prepared) = &result
            {
                let marks = prepared.layers()[0].marks();
                let expected = c["result"]["mapped"].as_array().unwrap();
                if c["channel"] == "size" {
                    compare(
                        &json!(marks.iter().map(|m| m.style.radius).collect::<Vec<_>>()),
                        &json!(expected.iter().filter(|v| !v.is_null()).collect::<Vec<_>>()),
                    );
                } else {
                    assert_eq!(
                        marks
                            .iter()
                            .map(|m| m.style.color.alpha)
                            .collect::<Vec<_>>(),
                        expected
                            .iter()
                            .map(|v| v
                                .as_f64()
                                .map_or(255, |v| (v * 255.).round_ties_even() as u8))
                            .collect::<Vec<_>>(),
                        "{c}"
                    );
                }
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
fn continuous_palette_missing_replacements_match_54_source_draws() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-palette-missing-paint.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["family"] == "continuous")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 54);
    for c in cases {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(Arc::new(Mutex::new(vec![])))))
            .unwrap();
        let inputs = c["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", inputs.iter().map(|v| v.as_f64()).collect::<Vec<_>>())
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
    }
}
#[test]
fn continuous_palette_batches_match_81_reference_builds() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-palette-batches.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["family"] == "continuous")
    {
        count += 1;
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let layers = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|layer| {
                layer
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(Json::as_f64)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let population = layers
            .iter()
            .flatten()
            .map(|v| v.map(Number))
            .collect::<Vec<_>>();
        let scale = MappedScale::new_with_registry(
            spec(c)
                .trained_with_registry(&population, &registry)
                .unwrap(),
            &registry,
        )
        .unwrap();
        let entries = scale
            .continuous_guide_entries(4096, 65536)
            .unwrap()
            .unwrap();
        let has_candidates = !entries.is_empty();
        let mut keys = vec![];
        let mut labels = vec![];
        let mut values = vec![];
        for e in entries.into_iter().filter(|e| e.visible) {
            keys.push(e.value.0);
            labels.push(e.label);
            values.push(encode(e.mapped.unwrap()));
        }
        compare(
            &if !has_candidates {
                json!([])
            } else {
                json!([{"values":keys,"labels":labels,"mapped":values}])
            },
            &c["result"]["keys"],
        );
        let mapped = layers
            .iter()
            .map(|layer| {
                scale
                    .numeric_batch(layer)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(encode)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        compare(&json!(mapped), &c["result"]["mapped"]);
        let actual: Vec<_> = calls
            .lock()
            .unwrap()
            .iter()
            .map(|v| json!({"values":v.iter().map(|v|json!(v.0)).collect::<Vec<_>>(),"names":[]}))
            .collect();
        compare(&json!(actual), &c["calls"]);
        let data = |name: &str, v: &Vec<Option<f64>>| {
            Data::columns()
                .name(name)
                .column("x", (0..v.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("v", v.clone())
                .build()
                .unwrap()
        };
        let draft = plot(data("first", &layers[0]))
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry));
        let p = if c["channel"] == "colour" {
            draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", spec(c)))
                .layer(points())
                .layer(points().data(data("second", &layers[1])))
                .build()
                .unwrap()
        } else {
            let channel = if c["channel"] == "size" {
                NumericAesthetic::Size
            } else {
                NumericAesthetic::Alpha
            };
            draft
                .aes(aes().x("x").y(1.))
                .layer(points().numeric_scale(channel, "v", spec(c)))
                .layer(points().data(data("second", &layers[1])).numeric_scale(
                    channel,
                    "v",
                    spec(c),
                ))
                .build()
                .unwrap()
        };
        let prepared = p.chart().unwrap().prepare().unwrap();
        for (i, layer) in prepared.layers().iter().enumerate() {
            let expected = c["result"]["mapped"][i].as_array().unwrap();
            let expected = expected
                .iter()
                .filter(|v| c["channel"] == "alpha" || !v.is_null())
                .collect::<Vec<_>>();
            assert_eq!(layer.marks().len(), expected.len(), "{c}");
            for (mark, value) in layer.marks().iter().zip(expected) {
                match c["channel"].as_str().unwrap() {
                    "colour" => assert_eq!(
                        mark.style.color,
                        chart_core::color::parse_r(value.as_str().unwrap())
                            .unwrap()
                            .resolve(),
                        "{c}"
                    ),
                    "size" => assert!(
                        (mark.style.radius - value.as_f64().unwrap()).abs() < 5e-14,
                        "{c}: {:?}",
                        mark.style
                    ),
                    _ => assert_eq!(
                        mark.style.color.alpha,
                        value
                            .as_f64()
                            .map_or(255, |v| (v * 255.).round_ties_even() as u8),
                        "{c}"
                    ),
                }
            }
        }
    }
    assert_eq!(count, 81);
}
