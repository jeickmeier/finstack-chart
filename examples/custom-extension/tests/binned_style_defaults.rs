//! FIX-GG04: typed binned shape defaults and registered theme/fallback behavior.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
struct RecordedPalette(Arc<Mutex<Vec<Vec<f64>>>>);
impl CustomScalePalette for RecordedPalette {
    fn descriptor(&self) -> ExtensionDescriptor {
        chart_extension_example::scale_palettes::Palette { portable: true }.descriptor()
    }
    fn validate(&self, p: &Json) -> ChartResult<()> {
        chart_extension_example::scale_palettes::Palette { portable: true }.validate(p)
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        if input.parameters["mode"] != "reject" {
            let ScalePaletteDomain::Normalized(values) = input.domain else {
                panic!("vector callback expected")
            };
            self.0
                .lock()
                .unwrap()
                .push(values.iter().map(|v| v.0).collect());
        }
        chart_extension_example::scale_palettes::Palette { portable: true }.evaluate(input)
    }
}
fn operation(mode: &str) -> ScalePaletteOperation {
    ScalePaletteOperation {
        operation: OperationRef::new("example.scale_palette", Revision::new(1)),
        parameters: json!({"mode":mode,"channel":"size"}),
    }
}
#[test]
fn binned_shape_defaults_match_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-style-defaults.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 48);
    for t in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(RecordedPalette(calls.clone())))
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let values = t["inputs"].as_array().unwrap();
            let d = Data::columns()
                .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("v", values.iter().map(Json::as_f64).collect::<Vec<_>>())
                .build()?;
            let mut scale =
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
                .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                    palette: (t["route"] != "null").then_some(GgplotBinnedPalette::Shape {
                        solid: t["route"] != "hollow",
                    }),
                    ..Default::default()
                })))?
                .with_guide(GgplotScaleGuide::BinnedBins(GgplotGuideLabels::Automatic))?;
            if t["route"] == "null" {
                // R's absent-solid fallback is a failing function. Typed authoring
                // expresses that computation through the existing palette registry.
                scale = scale
                    .with_palette_function(operation("reject"))?
                    .with_theme_palette(vec!["shape".into()])?;
            }
            let mut p = plot(d)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(Arc::new(registry))
                .aes(aes().x("x").y(1.))
                .layer(points().value_scale(ValueAesthetic::Shape, "v", scale));
            if t["theme_mode"] != "absent" {
                p = p.theme(
                    theme().scale_palettes(
                        [(
                            "palette.shape.continuous".into(),
                            operation(if t["theme_mode"] == "vector" {
                                "constant"
                            } else {
                                "repeat_count"
                            }),
                        )]
                        .into(),
                    ),
                );
            }
            p.build()?.chart()?.prepare()
        })();
        let actual_calls = calls.lock().unwrap();
        let expected_calls = t["calls"].as_array().unwrap();
        assert_eq!(actual_calls.len(), expected_calls.len(), "{t}");
        for (actual, expected) in actual_calls.iter().zip(expected_calls) {
            let expected = expected.as_array().map_or_else(
                || vec![expected.as_f64().unwrap()],
                |v| v.iter().map(|v| v.as_f64().unwrap()).collect(),
            );
            assert_eq!(actual, &expected, "{t}");
        }
        if t["result"].get("error").is_some() {
            assert!(result.is_err(), "{t}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{t}: {e:?}"));
        let layer = &prepared.layers()[0];
        let expected = t["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Json::as_f64)
            .collect::<Vec<_>>();
        assert_eq!(layer.marks().len(), expected.len(), "{t}");
        for (mark, expected) in layer.marks().iter().zip(expected) {
            assert_eq!(
                mark.aesthetics.get(&ValueAesthetic::Shape),
                Some(&Value::Number(Number(expected))),
                "{t}"
            );
        }
        let entries = layer
            .numeric_value_guides()
            .values()
            .flatten()
            .filter(|e| e.visible)
            .collect::<Vec<_>>();
        let expected = t["result"]["guides"].as_array().unwrap();
        assert_eq!(
            entries.len(),
            expected
                .first()
                .map_or(0, |v| v["values"].as_array().unwrap().len()),
            "{t}"
        );
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(
                entry.label.as_deref(),
                expected[0]["labels"][i].as_str(),
                "{t}"
            );
            assert_eq!(
                entry.mapped,
                Some(
                    expected[0]["mapped"][i]
                        .as_f64()
                        .map_or(Value::Missing, |v| Value::Number(Number(v)))
                ),
                "{t}"
            );
        }
    }
}
