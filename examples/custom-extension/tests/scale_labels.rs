//! FIX-GG04: label functions through population training, primary wire and guides.
use chart_core::{interpolate::Number, prelude::*, scales::*};
use serde_json::Value;
fn key(v: Option<&str>) -> ScaleKey {
    v.map_or(ScaleKey::Null, |v| ScaleKey::Text(v.into()))
}
#[test]
fn primary_scale_label_functions_match_reference_guide_keys() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/guide-label-functions.json"
    ))
    .unwrap();
    let registry = chart_extension_example::registry().unwrap();
    let mut passed = 0;
    let mut rejected = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let continuous = c["kind"] == "continuous";
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(continuous).unwrap() else {
            unreachable!()
        };
        let labels = GgplotGuideLabels::Registered {
            operation: chart_core::grammar::OperationRef::new(
                "example.scale_labels",
                chart_core::Revision::new(1),
            ),
            parameters: c["label_mode"].clone(),
        };
        let data = if continuous {
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
            scale.ggplot = Some(Box::new(GgplotScalePolicy::Continuous {
                empty_population: false,
                nonfinite_population: false,
                limits: Some([
                    Some(Number(c["limits"][0].as_f64().unwrap())),
                    Some(Number(c["limits"][1].as_f64().unwrap())),
                ]),
                oob: GgplotOob::Censor,
            }));
            scale.guide = Some(Box::new(GgplotScaleGuide::Continuous(
                GgplotContinuousGuide {
                    breaks: match c["break_mode"].as_str().unwrap() {
                        "auto" => None,
                        "empty" => Some(vec![]),
                        _ => Some(
                            [
                                -1.,
                                0.,
                                0.1,
                                1.,
                                5.,
                                10.,
                                20.,
                                f64::INFINITY,
                                f64::NAN,
                                f64::NAN,
                            ]
                            .map(Number)
                            .to_vec(),
                        ),
                    },
                    count: None,
                    labels,
                },
            )));
            Data::columns()
                .column("x", [1., 2., 3.])
                .column("v", [1., 4., 10.])
                .build()
                .unwrap()
        } else {
            scale.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
                breaks: match c["break_mode"].as_str().unwrap() {
                    "auto" => None,
                    "empty" => Some(vec![]),
                    _ => Some(
                        [Some("z"), Some("b"), Some("b"), None, Some("a")]
                            .map(key)
                            .to_vec(),
                    ),
                },
                break_names: (c["break_mode"] == "named")
                    .then(|| ["Z", "B", "B2", "M", "A"].map(String::from).to_vec()),
                labels,
            })));
            let values = if c["population"] == "empty" {
                vec![]
            } else {
                vec![
                    "a",
                    "b",
                    if c["population"] == "nullable" {
                        ""
                    } else {
                        "c"
                    },
                ]
            };
            let validity = (0..values.len())
                .map(|i| c["population"] != "nullable" || i != 2)
                .collect();
            Data::columns()
                .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("v", chart_core::plot::column(values).validity(validity))
                .build()
                .unwrap()
        };
        let result = (|| {
            let p = plot(data)
                .extensions(registry.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points())
                .build()?;
            // Default colors carry theme-palette selection in both families.
            let expected_version = 45;
            assert_eq!(p.definition().wire_version(), expected_version);
            let wire = p.to_json()?;
            let restored = Plot::from_json_with_extensions(&wire, registry.clone())?;
            assert_eq!(restored.to_json()?, wire);
            let mut old: Value = serde_json::from_str(&wire).unwrap();
            old["version"] = (expected_version - 1).into();
            assert!(Plot::from_json_with_extensions(&old.to_string(), registry.clone()).is_err());
            let prepared = restored.chart()?.prepare()?;
            Ok::<_, chart_core::Diagnostic>(
                prepared.layers()[0]
                    .color_legend()
                    .map(|g| g.entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
        })();
        if c["build"].get("error").is_some() {
            assert!(result.is_err(), "{c}: {result:?}");
            rejected += 1;
        } else {
            let expected = c["build"]["keys"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|g| g["labels"].as_array().unwrap())
                .map(|l| l.as_str().unwrap_or("NA").to_string())
                .collect::<Vec<_>>();
            assert_eq!(
                result.unwrap_or_else(|e| panic!("{c}: {e:?}")),
                expected,
                "{c}"
            );
            passed += 1;
        }
    }
    assert_eq!(passed + rejected, 180);
    eprintln!("{passed} primary label builds, {rejected} reference rejections");
}
