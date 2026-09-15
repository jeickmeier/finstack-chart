//! FIX-GG04: automatic shape/linetype fallback versus explicit shape palettes.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::Arc;
struct Constant;
impl CustomScalePalette for Constant {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.style_palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Count(n) = input.domain else {
            panic!("discrete palette")
        };
        let value = if input.parameters["channel"] == "shape" {
            Value::Number(Number(3.))
        } else {
            Value::Text("22".into())
        };
        Ok(ScalePaletteOutput {
            values: Some(vec![value; n]),
            names: None,
        })
    }
}
#[test]
fn default_style_palettes_match_36_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/default-style-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 36);
    for t in cases {
        assert!(t["result"].get("error").is_none(), "{t}");
        let shape = t["channel"] == "shape";
        let channel = t["channel"].as_str().unwrap();
        let values = t["inputs"].as_array().unwrap();
        let d = Data::columns()
            .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "end",
                (0..values.len())
                    .map(|i| i as f64 + 0.5)
                    .collect::<Vec<_>>(),
            )
            .column(
                "v",
                values
                    .iter()
                    .map(|v| v.as_str().map(str::to_owned))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let mut registry = ExtensionRegistry::new();
        registry.register_scale_palette(Arc::new(Constant)).unwrap();
        let mapping = if shape {
            aes().x("x").y(1.).shape("v")
        } else {
            aes().x("x").y(1.).x2("end").y2(1.).linetype("v")
        };
        let mut layer = if shape { points() } else { rule() };
        if t["route"] != "automatic" {
            let palette = if shape {
                GgplotDiscretePalette::Shape {
                    solid: t["route"] != "hollow",
                }
            } else {
                GgplotDiscretePalette::LineType
            };
            let mut scale =
                MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
                    .with_ggplot(GgplotScalePolicy::Discrete {
                        empty_population: false,
                        limits: None,
                        levels: None,
                        drop: true,
                        na_translate: true,
                        palette,
                    })
                    .unwrap();
            if t["route"] == "constructor" {
                scale = scale.with_theme_palette(vec![channel.into()]).unwrap();
            }
            layer = layer.value_scale(
                if shape {
                    ValueAesthetic::Shape
                } else {
                    ValueAesthetic::LineType
                },
                "v",
                scale,
            );
        }
        let mut draft = plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .aes(mapping)
            .layer(layer);
        if t["theme_mode"] == "supplied" {
            draft = draft.theme(
                theme().scale_palettes(
                    [(
                        format!("palette.{channel}.discrete"),
                        ScalePaletteOperation {
                            operation: OperationRef {
                                id: "test.style_palette".into(),
                                version: Revision::new(1),
                            },
                            parameters: json!({"channel":channel}),
                        },
                    )]
                    .into(),
                ),
            );
        }
        let p = draft.build().unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        let wanted = t["result"]["mapped"].as_array().unwrap();
        if shape {
            assert_eq!(
                marks.len(),
                wanted.iter().filter(|v| !v.is_null()).count(),
                "{t}"
            );
            for (m, v) in marks.iter().zip(wanted.iter().filter(|v| !v.is_null())) {
                assert_eq!(
                    m.aesthetics.get(&ValueAesthetic::Shape),
                    Some(&Value::Number(Number(v.as_f64().unwrap()))),
                    "{t}"
                );
            }
        } else {
            assert_eq!(marks.len(), wanted.len(), "{t}");
            for (m, v) in marks.iter().zip(wanted) {
                let expected = if let Some(v) = v.as_str() {
                    LineType::parse(v).unwrap()
                } else {
                    LineType::Blank
                };
                assert_eq!(m.style.line_type, Some(expected), "{t}");
            }
        }
    }
}
