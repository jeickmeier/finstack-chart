//! FIX-GG04: continuous/binned constructor palette sampling and argument forwarding.
use chart_core::{
    ChartResult,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::Value as Json;

#[test]
fn continuous_paint_constructors_match_200_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-paint-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 200);
    check_cases(cases);
}

#[test]
fn gradient_remapping_lengths_match_96_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/gradient-remap-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 96);
    check_cases(cases);
}

#[test]
fn nonfinite_gradient_remapping_matches_156_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/gradient-nonfinite-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 156);
    check_cases(cases);
}

#[test]
fn invalid_gradient_positions_match_200_reference_outcomes() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/gradient-invalid-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 200);
    check_cases(cases);
}

fn check_cases(cases: &[Json]) {
    for t in cases {
        let result = (|| -> ChartResult<_> {
            let inputs = t["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("v", inputs.iter().map(Json::as_f64).collect::<Vec<_>>())
                .build()?;
            let scale = scale_for(t)?;
            let authored = color_mapped("v", scale);
            let mapping = if t["channel"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("v")
            } else {
                aes().x("x").y(1.).color("v").color_scale("v")
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .scale(authored)
                .layer(points().aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.))))
                .build()?;
            let wire = p.to_json()?;
            let special = t["args"]["values"]
                .as_array()
                .is_some_and(|v| v.iter().any(Json::is_object));
            if special || matches!(t["constructor"].as_str(), Some("distiller" | "viridis_c")) {
                let mut old: Json = serde_json::from_str(&wire).unwrap();
                assert_eq!(old["version"], if special { 52 } else { 51 });
                old["version"] = serde_json::json!(if special { 51 } else { 50 });
                assert_eq!(
                    Plot::from_json(&old.to_string()).unwrap_err().code,
                    chart_core::DiagnosticCode::UnsupportedCapability
                );
            }
            let restored = Plot::from_json(&wire)?;
            let prepared = restored.chart()?.prepare()?;
            assert_eq!(p.to_json()?, wire);
            Ok(prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    if t["channel"] == "fill" {
                        m.style.fill
                    } else {
                        Some(m.style.color)
                    }
                })
                .collect::<Vec<_>>())
        })();
        if t["result"].get("error").is_some() {
            assert!(result.is_err(), "{t}: {result:?}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{t}: {e}"));
        let expected = t["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| t["channel"] == "fill" || !v.is_null())
            .map(|v| {
                Some(v.as_str().map_or(
                    chart_core::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    },
                    |v| chart_core::color::parse_r(v).unwrap().resolve(),
                ))
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{t}");
        assert_eq!(
            actual.len(),
            t["result"]["point_count"].as_u64().unwrap() as usize
        );
    }
}

fn scale_for(t: &Json) -> ChartResult<MappedScaleSpec> {
    use chromatic::ggplot::PaletteSpec;
    let name = t["constructor"].as_str().unwrap();
    let custom = t["configuration"] == "custom";
    let binned = name.starts_with("steps") || matches!(name, "fermenter" | "viridis_b");
    let mut count_palette = None;
    let mut colors: Vec<&str> = if matches!(name, "gradient" | "steps") {
        if custom {
            vec!["red", "blue"]
        } else {
            vec!["#132B43", "#56B1F7"]
        }
    } else if matches!(name, "gradient2" | "steps2") {
        if custom {
            vec!["red", "white", "blue"]
        } else {
            vec!["#832424", "white", "#3A3A98"]
        }
    } else if custom {
        vec!["#ff000040", "#ffffff80", "#0000ffcc"]
    } else {
        vec!["red", "white", "blue"]
    };
    if let Some(v) = t["args"].get("colours") {
        colors = if let Some(s) = v.as_str() {
            vec![s]
        } else {
            v.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect()
        };
    }
    let values = t["args"]
        .get("values")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
    let palette = if matches!(name, "distiller" | "fermenter" | "viridis_c" | "viridis_b") {
        let palette = if name.starts_with("viridis") {
            GgplotDiscretePalette::Viridis {
                option: if custom {
                    chromatic::ggplot::ViridisOption::Magma
                } else {
                    chromatic::ggplot::ViridisOption::Viridis
                },
                alpha: if custom { 0.5 } else { 1. },
                begin: if custom { 0.2 } else { 0. },
                end: if custom { 0.8 } else { 1. },
                reverse: custom,
            }
        } else {
            GgplotDiscretePalette::Brewer {
                id: if custom {
                    chromatic::SchemeId::RdBu
                } else {
                    chromatic::SchemeId::Blues
                },
                reverse: !custom,
            }
        };
        if binned {
            count_palette = Some(GgplotBinnedPalette::Discrete(Box::new(palette)));
            None
        } else {
            Some(PaletteSpec::CountGradient {
                palette: Box::new(palette),
                count: if name == "distiller" { 7 } else { 6 },
                values,
            })
        }
    } else {
        Some(PaletteSpec::Gradient {
            colors: colors
                .into_iter()
                .map(chart_core::color::parse_r)
                .collect::<ChartResult<Vec<_>>>()?,
            values,
        })
    };
    let output = palette.map_or(ScaleRangeFunction::Identity, |spec| {
        ScaleRangeFunction::Interpolate(chart_core::interpolate::InterpolationSpec::GgplotPalette {
            spec,
        })
    });
    let scale = MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Ggplot {
            family: NumericFamily::Linear,
            domain: [Number(0.), Number(1.)],
            reverse: false,
            rescaler: if matches!(name, "gradient2" | "steps2") {
                GgplotRescaler::Midpoint(Number(if custom { 1. } else { 0. }))
            } else {
                GgplotRescaler::Range
            },
            timestamp: None,
        },
        output,
        unknown: Value::Missing,
    }))
    .with_ggplot(if binned {
        GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
            palette: count_palette,
            ..Default::default()
        }))
    } else {
        GgplotScalePolicy::Continuous {
            empty_population: false,
            nonfinite_population: false,
            limits: None,
            oob: GgplotOob::Censor,
        }
    })?;
    scale.with_guide(GgplotScaleGuide::Hidden)
}

#[test]
fn count_gradient_standalone_versions_and_resource_limits_are_checked() {
    use chart_core::interpolate::{InterpolationSpec, Interpolator};
    let recipe = InterpolationSpec::GgplotPalette {
        spec: chromatic::ggplot::PaletteSpec::CountGradient {
            palette: Box::new(GgplotDiscretePalette::Brewer {
                id: chromatic::SchemeId::Blues,
                reverse: true,
            }),
            count: 7,
            values: None,
        },
    };
    let p = Interpolator::new(recipe.clone()).unwrap();
    let wire = p.descriptor_json().unwrap();
    let mut old: Json = serde_json::from_str(&wire).unwrap();
    assert_eq!(old["version"], 4);
    old["version"] = serde_json::json!(3);
    assert_eq!(
        Interpolator::from_json(&old.to_string()).unwrap_err().code,
        chart_core::DiagnosticCode::UnsupportedCapability
    );
    assert_eq!(
        Interpolator::from_json(&wire)
            .unwrap()
            .descriptor_json()
            .unwrap(),
        wire
    );
    let spec = StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::sequential(NumericFamily::Linear),
        output: ScaleRangeFunction::Interpolate(recipe),
        unknown: Value::Missing,
    });
    assert_eq!(spec.wire_version(), 6);
    let scale = StandaloneScale::new(spec).unwrap();
    let wire = scale.to_json().unwrap();
    assert_eq!(
        StandaloneScale::from_json(&wire)
            .unwrap()
            .to_json()
            .unwrap(),
        wire
    );
    for count in [0, 65_537] {
        let recipe = InterpolationSpec::GgplotPalette {
            spec: chromatic::ggplot::PaletteSpec::CountGradient {
                palette: Box::new(GgplotDiscretePalette::Viridis {
                    option: chromatic::ggplot::ViridisOption::Viridis,
                    begin: 0.,
                    end: 1.,
                    reverse: false,
                    alpha: 1.,
                }),
                count,
                values: None,
            },
        };
        assert!(Interpolator::new(recipe).is_err());
    }
}

#[test]
fn special_gradient_positions_retain_identity_and_versioned_envelopes() {
    use chart_core::interpolate::{InterpolationSpec, Interpolator};
    for special in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0] {
        for counted in [false, true] {
            let values = Some(vec![Number(0.), Number(special), Number(1.)]);
            let spec = if counted {
                chromatic::ggplot::PaletteSpec::CountGradient {
                    palette: Box::new(GgplotDiscretePalette::Brewer {
                        id: chromatic::SchemeId::Blues,
                        reverse: true,
                    }),
                    count: 7,
                    values,
                }
            } else {
                chromatic::ggplot::PaletteSpec::Gradient {
                    colors: ["red", "white", "blue"]
                        .map(|s| chart_core::color::parse_r(s).unwrap())
                        .to_vec(),
                    values,
                }
            };
            let recipe = InterpolationSpec::GgplotPalette { spec };
            let restored: InterpolationSpec =
                serde_json::from_str(&serde_json::to_string(&recipe).unwrap()).unwrap();
            assert_eq!(restored, recipe);
            assert_eq!(recipe.wire_version(), 5);
            let interpolation = Interpolator::new(recipe.clone()).unwrap();
            let wire = interpolation.descriptor_json().unwrap();
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            old["version"] = serde_json::json!(4);
            assert_eq!(
                Interpolator::from_json(&old.to_string()).unwrap_err().code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
            assert_eq!(
                Interpolator::from_json(&wire)
                    .unwrap()
                    .descriptor_json()
                    .unwrap(),
                wire
            );
            let scale =
                StandaloneScale::new(StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec {
                    normalization: NormalizationSpec::sequential(NumericFamily::Linear),
                    output: ScaleRangeFunction::Interpolate(recipe),
                    unknown: Value::Missing,
                }))
                .unwrap();
            let wire = scale.to_json().unwrap();
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(old["version"], 7);
            old["version"] = serde_json::json!(6);
            assert_eq!(
                StandaloneScale::from_json(&old.to_string())
                    .unwrap_err()
                    .code,
                chart_core::DiagnosticCode::UnsupportedCapability
            );
            assert_eq!(
                StandaloneScale::from_json(&wire)
                    .unwrap()
                    .to_json()
                    .unwrap(),
                wire
            );
        }
    }
}
