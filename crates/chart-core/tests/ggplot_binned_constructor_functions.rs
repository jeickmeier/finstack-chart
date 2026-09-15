//! FIX-GG04: a registered adapter lowers the reference's count callback to bin vectors.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
struct CountPalette(Arc<Mutex<Vec<usize>>>);
impl CustomScalePalette for CountPalette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.binned_count", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Normalized(samples) = input.domain else {
            panic!("binned vector adapter")
        };
        let n = samples.len();
        self.0.lock().unwrap().push(n);
        let mode = input.parameters["mode"].as_str().unwrap();
        let mut values = (1..=n)
            .map(|i| {
                Value::Text(
                    if (i as f64) / (n as f64) < 0.5 {
                        "#ff0000"
                    } else {
                        "#0000ff"
                    }
                    .into(),
                )
            })
            .collect::<Vec<_>>();
        match mode {
            "short" => values.truncate(1),
            "empty" => values.clear(),
            "missing" if n > 1 => values[1] = Value::Missing,
            _ => {}
        }
        let names = (mode == "named").then(|| {
            (0..values.len())
                .map(|i| ["c", "b", "a", "d"][i % 4].into())
                .collect()
        });
        Ok(ScalePaletteOutput {
            values: (mode != "null").then_some(values),
            names,
        })
    }
}
struct Squish;
impl CustomScaleVector for Squish {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.count_squish", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
        Ok(Some(
            input
                .values
                .iter()
                .map(|v| {
                    Number(if v.0.is_finite() {
                        v.0.clamp(input.limits[0].0, input.limits[1].0)
                    } else {
                        v.0
                    })
                })
                .collect(),
        ))
    }
}
#[test]
fn count_callback_adapter_matches_108_public_constructor_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-constructor-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 108);
    for t in cases {
        for pipeline in [false, true] {
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::new();
            registry
                .register_scale_palette(Arc::new(CountPalette(calls.clone())))
                .unwrap();
            registry.register_scale_vector(Arc::new(Squish)).unwrap();
            let registry = Arc::new(registry);
            let inputs = t["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    inputs
                        .iter()
                        .map(|v| {
                            v.as_f64()
                                .or_else(|| v.as_str().map(|v| v.parse::<f64>().unwrap()))
                        })
                        .collect::<Vec<_>>(),
                )
                .build()
                .unwrap();
            let mut scale = MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(
                InterpolatedScaleSpec::sequential(NumericFamily::Linear),
            ))
            .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                breaks: GgplotBreaks::Nice(t["count"].as_f64().unwrap()),
                ..Default::default()
            })))
            .unwrap()
            .with_guide(GgplotScaleGuide::Hidden)
            .unwrap()
            .with_palette_function(ScalePaletteOperation {
                operation: OperationRef {
                    id: "test.binned_count".into(),
                    version: Revision::new(1),
                },
                parameters: json!({"mode":t["mode"]}),
            })
            .unwrap();
            if pipeline {
                scale = scale
                    .with_oob_function(ScaleVectorOperation {
                        operation: OperationRef {
                            id: "test.count_squish".into(),
                            version: Revision::new(1),
                        },
                        parameters: json!({}),
                    })
                    .unwrap();
            }
            let mapping = if t["channel"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("v")
            } else {
                aes().x("x").y(1.).color("v").color_scale("v")
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry.clone())
                .aes(mapping)
                .scale(color_mapped("v", scale))
                .layer(points().aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.))))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            assert!(calls.lock().unwrap().is_empty());
            let prepared = restored.chart().unwrap().prepare().unwrap();
            assert_eq!(json!(*calls.lock().unwrap()), t["calls"], "{t}");
            let marks = prepared.layers()[0].marks();
            assert_eq!(
                marks.len(),
                t["result"]["point_count"].as_u64().unwrap() as usize,
                "{t}"
            );
            for (mark, v) in marks
                .iter()
                .zip(t["result"]["point_colours"].as_array().unwrap())
            {
                let actual = if t["channel"] == "fill" {
                    mark.style.fill.unwrap()
                } else {
                    mark.style.color
                };
                if v.is_null() {
                    assert_eq!(actual.alpha, 0, "{t}");
                    continue;
                }
                assert_eq!(
                    actual,
                    chart_core::color::parse_r(v.as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "{t}"
                );
            }
        }
    }
}
