//! FIX-GG04: reference constructor/theme palette selection before population training.
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
    } else if value == f64::INFINITY {
        json!("Infinity")
    } else if value == f64::NEG_INFINITY {
        json!("-Infinity")
    } else {
        json!(value)
    }
}
struct Palette(Arc<Mutex<Vec<Json>>>);
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.selection", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let mode = input.parameters["mode"].as_str().unwrap();
        let channel = input.parameters["channel"].as_str().unwrap();
        let (arguments, samples) = match input.domain {
            ScalePaletteDomain::Count(n) => (
                vec![json!(n)],
                (0..n)
                    .map(|i| {
                        if mode == "generic" {
                            i as f64 / (n.saturating_sub(1).max(1)) as f64
                        } else {
                            (i + 1) as f64 / n as f64
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            ScalePaletteDomain::Normalized(values) => (
                values.iter().map(|v| number(v.0)).collect(),
                values.iter().map(|v| v.0).collect(),
            ),
        };
        self.0
            .lock()
            .unwrap()
            .push(json!({"mode":mode,"values":arguments}));
        let values = samples
            .iter()
            .enumerate()
            .map(|(i, sample)| {
                let t = match mode {
                    "index" => (i + 1) as f64 / samples.len() as f64,
                    "first" => samples[0],
                    _ => *sample,
                };
                if t.is_nan() {
                    Value::Missing
                } else if channel == "colour" {
                    Value::Text(if t < 0.5 { "#ff0000" } else { "#0000ff" }.into())
                } else {
                    Value::Number(Number(if channel == "size" { 1. + 4. * t } else { t }))
                }
            })
            .collect();
        Ok(ScalePaletteOutput {
            values: Some(values),
            names: None,
        })
    }
}
fn operation(channel: &str, mode: &str) -> ScalePaletteOperation {
    ScalePaletteOperation {
        operation: OperationRef {
            id: "test.selection".into(),
            version: Revision::new(1),
        },
        parameters: json!({"channel":channel,"mode":mode}),
    }
}
fn scale(case: &Json) -> MappedScaleSpec {
    let channel = case["channel"].as_str().unwrap();
    let discrete = case["family"] == "discrete";
    let mut scale = if channel == "colour" {
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(!discrete).unwrap() else {
            unreachable!()
        };
        scale.missing_paint_is_na = true;
        scale
    } else {
        let kind = if channel == "size" {
            GgplotNumericPalette::Size
        } else {
            GgplotNumericPalette::Alpha
        };
        if discrete {
            ggplot_numeric_ordinal(kind).unwrap()
        } else {
            ggplot_numeric_default(kind).unwrap()
        }
    };
    if case["family"] == "binned" {
        scale = scale
            .with_ggplot(GgplotScalePolicy::Binned(Box::default()))
            .unwrap();
    }
    scale = scale.with_guide(GgplotScaleGuide::Hidden).unwrap();
    if case["source"] == "explicit" || case["source"] == "fallback" {
        scale = scale.with_theme_palette(vec![channel.into()]).unwrap();
        scale = scale
            .with_palette_function(operation(
                channel,
                if case["source"] == "explicit" {
                    "full"
                } else {
                    "index"
                },
            ))
            .unwrap();
    }
    if case["source"] != "explicit" {
        let aesthetics = case.get("lookup_aesthetics").map_or_else(
            || vec![channel.into()],
            |values| {
                values
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect()
            },
        );
        scale = scale.with_theme_palette(aesthetics).unwrap();
    }
    scale
}
fn data(case: &Json) -> Data {
    let values = case["inputs"].as_array().unwrap();
    let data = Data::columns().column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>());
    if case["family"] == "discrete" {
        data.column(
            "v",
            values
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>(),
        )
        .build()
        .unwrap()
    } else {
        data.column(
            "v",
            values
                .iter()
                .map(|v| match v.as_str() {
                    Some("Infinity") => Some(f64::INFINITY),
                    Some("-Infinity") => Some(f64::NEG_INFINITY),
                    _ => v.as_f64(),
                })
                .collect::<Vec<_>>(),
        )
        .build()
        .unwrap()
    }
}
#[test]
fn constructor_theme_precedence_matches_324_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-palette-selection.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 324);
    for case in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let registry = Arc::new(registry);
        let channel = case["channel"].as_str().unwrap();
        let mut draft = plot(data(case))
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone());
        let scale = scale(case);
        if channel == "colour" {
            draft = draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points());
        } else {
            draft = draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
                if channel == "size" {
                    NumericAesthetic::Size
                } else {
                    NumericAesthetic::Alpha
                },
                "v",
                scale,
            ));
        }
        if case["theme_mode"] == "supplied" {
            let family = if case["family"] == "discrete" {
                "discrete"
            } else {
                "continuous"
            };
            let palettes = case.get("theme_palettes").map_or_else(
                || {
                    [(
                        format!("palette.{channel}.{family}"),
                        operation(channel, "first"),
                    )]
                    .into()
                },
                |palettes| {
                    palettes
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(key, mode)| {
                            (key.clone(), operation(channel, mode.as_str().unwrap()))
                        })
                        .collect()
                },
            );
            draft = draft.theme(theme().scale_palettes(palettes));
        }
        let plot = draft.build().unwrap();
        let wire = plot.to_json().unwrap();
        if case["source"] != "explicit" || case["theme_mode"] == "supplied" {
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(old["version"], 45);
            old["version"] = json!(44);
            assert!(Plot::from_json_with_extensions(&old.to_string(), registry.clone()).is_err());
        }
        let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        assert!(calls.lock().unwrap().is_empty());
        let prepared = restored.chart().unwrap().prepare().unwrap();
        let recorded = calls.lock().unwrap();
        let expected_calls = case["calls"].as_array().unwrap();
        assert_eq!(recorded.len(), expected_calls.len(), "{case}");
        for (actual, expected) in recorded.iter().zip(expected_calls) {
            assert_eq!(actual["mode"], expected["mode"], "{case}");
            let actual = actual["values"].as_array().unwrap();
            let expected = expected["values"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{case}");
            for (actual, expected) in actual.iter().zip(expected) {
                if let (Some(actual), Some(expected)) = (actual.as_f64(), expected.as_f64()) {
                    assert!((actual - expected).abs() < 5e-14, "{case}");
                } else {
                    assert_eq!(actual, expected, "{case}");
                }
            }
        }
        drop(recorded);
        let marks = prepared.layers()[0].marks();
        assert_eq!(
            marks.len(),
            case["result"]["point_count"].as_u64().unwrap() as usize,
            "{case}"
        );
        let colors = case["result"]["point_colours"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                chart_core::color::parse_r(v.as_str().unwrap())
                    .unwrap()
                    .resolve()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            marks.iter().map(|m| m.style.color).collect::<Vec<_>>(),
            colors,
            "{case}"
        );
        if channel == "size" {
            let wanted = case["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| match v.as_str() {
                    Some("Infinity") => Some(f64::INFINITY),
                    Some("-Infinity") => Some(f64::NEG_INFINITY),
                    _ => v.as_f64(),
                })
                .collect::<Vec<_>>();
            assert_eq!(marks.len(), wanted.len(), "{case}");
            for (mark, wanted) in marks.iter().zip(wanted) {
                assert!(
                    mark.style.radius == wanted || (mark.style.radius - wanted).abs() < 2e-12,
                    "{case}"
                );
            }
        }
    }
}

#[test]
fn theme_color_vectors_match_180_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-palette-values.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 180);
    let mut successes = 0;
    for case in cases {
        let result = (|| -> ChartResult<_> {
            let palette: chart_core::theme::ThemeScalePalette =
                serde_json::from_value(case["palette_values"].clone()).map_err(|e| {
                    chart_core::Diagnostic::error(
                        chart_core::DiagnosticCode::Validation,
                        e.to_string(),
                        "Supply a valid theme palette.",
                    )
                })?;
            let mut authored = case.clone();
            authored["channel"] = json!("colour");
            authored["source"] = json!("builtin");
            let scale = scale(&authored);
            let mut color = color_mapped("v", scale);
            if case["na_mode"] == "grey50" {
                color = color.missing(chart_core::color::parse_r("grey50")?);
            }
            let family = if case["family"] == "discrete" {
                "discrete"
            } else {
                "continuous"
            };
            let plot = plot(data(case))
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color)
                .layer(points())
                .theme(
                    theme().scale_palettes([(format!("palette.colour.{family}"), palette)].into()),
                )
                .build()?;
            let wire = plot.to_json()?;
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(
                old["version"],
                if case["palette_values"].as_array().unwrap().len() == 1 {
                    47
                } else {
                    46
                }
            );
            old["version"] = json!(45);
            assert!(Plot::from_json(&old.to_string()).is_err());
            let restored = Plot::from_json(&wire)?;
            assert_eq!(restored.to_json()?, wire);
            restored.chart()?.prepare()
        })();
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let prepared = result.unwrap_or_else(|error| panic!("{error:?}: {case}"));
        successes += 1;
        let marks = prepared.layers()[0].marks();
        assert_eq!(
            marks.len(),
            case["result"]["point_count"].as_u64().unwrap() as usize,
            "{case}"
        );
        let colors = case["result"]["point_colours"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                chart_core::color::parse_r(v.as_str().unwrap())
                    .unwrap()
                    .resolve()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            marks.iter().map(|m| m.style.color).collect::<Vec<_>>(),
            colors,
            "{case}"
        );
    }
    assert_eq!(successes, 110);
}

#[test]
fn overridden_paint_scale_does_not_redirect_the_next_theme_selection() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-palette-selection.json"
    ))
    .unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["family"] == "continuous"
                && case["channel"] == "colour"
                && case["source"] == "fallback"
                && case["theme_mode"] == "supplied"
                && case["population"] == "ordinary"
        })
        .unwrap();
    let calls = Arc::new(Mutex::new(vec![]));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_palette(Arc::new(Palette(calls)))
        .unwrap();
    let registry = Arc::new(registry);
    let plot = plot(data(case))
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(registry.clone())
        .aes(aes().x("x").y(1.).color("v").color_scale("v"))
        .scale(color_mapped("v", scale(case)))
        .layer(points())
        .theme(
            theme().scale_palettes(
                [(
                    "palette.colour.continuous".into(),
                    operation("colour", "first"),
                )]
                .into(),
            ),
        )
        .build()
        .unwrap();
    let mut definition = plot.definition().clone();
    let mut overridden = definition.layers[0].clone();
    overridden.id = chart_core::LayerId::new(900);
    overridden
        .paint_scales
        .insert(PaintAesthetic::Fill, overridden.color.take().unwrap());
    overridden.style.fill = Some(chart_core::color::parse_r("black").unwrap());
    definition.layers.insert(0, overridden);
    let mut compiler = Compiler::with_extensions(registry);
    let prepared = compiler
        .prepare(
            &definition,
            &plot.source(),
            &chart_core::state::ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    let expected = case["result"]["point_colours"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            chart_core::color::parse_r(v.as_str().unwrap())
                .unwrap()
                .resolve()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        prepared.layers()[1]
            .marks()
            .iter()
            .map(|m| m.style.color)
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn continuous_guide_selection_matches_189_primary_reference_draws() {
    fn equivalent(actual: &Json, expected: &Json) -> bool {
        if let (Some(a), Some(b)) = (actual.as_f64(), expected.as_f64()) {
            a == b
        } else if let (Some(a), Some(b)) = (actual.as_array(), expected.as_array()) {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
        } else {
            actual == expected
        }
    }
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-guide-selection.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["guide"].as_str(),
            Some("default" | "none" | "legend" | "colourbar" | "bins" | "coloursteps")
        )
    }) {
        let channel = case["channel"].as_str().unwrap();
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let registry = Arc::new(registry);
        let mut adapted = case.clone();
        adapted["family"] = json!("continuous");
        adapted["source"] = json!("explicit");
        let mut scale = scale(&adapted);
        if let Some(GgplotScalePolicy::Continuous { limits, .. }) = scale.ggplot.as_deref_mut() {
            *limits = Some([Some(Number(0.)), Some(Number(4.))]);
        }
        scale = scale
            .with_guide(if case["breaks"] == "null" || case["guide"] == "none" {
                GgplotScaleGuide::Hidden
            } else {
                let arguments = GgplotContinuousGuide {
                    breaks: if case["breaks"] == "uneven" {
                        Some(vec![Number(0.), Number(0.5), Number(3.), Number(4.)])
                    } else if case["breaks"] == "outside" {
                        Some(vec![Number(-2.), Number(6.)])
                    } else {
                        (case["breaks"] == "empty").then(Vec::new)
                    },
                    ..Default::default()
                };
                if case["guide"] == "bins" {
                    GgplotScaleGuide::ContinuousBins(arguments)
                } else if case["guide"] == "coloursteps" {
                    GgplotScaleGuide::ContinuousSteps(arguments)
                } else if case["guide"] == "colourbar" {
                    GgplotScaleGuide::Colorbar(arguments)
                } else {
                    GgplotScaleGuide::Continuous(arguments)
                }
            })
            .unwrap();
        let mut draft = plot(data(&adapted))
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone());
        if channel == "colour" {
            draft = draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points());
        } else {
            draft = draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
                if channel == "size" {
                    NumericAesthetic::Size
                } else {
                    NumericAesthetic::Alpha
                },
                "v",
                scale,
            ));
        }
        let authored = draft.build().unwrap();
        let wire = authored.to_json().unwrap();
        if case["guide"] == "colourbar" && case["breaks"] != "null" {
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(old["version"], 57);
            old["version"] = json!(56);
            assert!(Plot::from_json_with_extensions(&old.to_string(), registry.clone()).is_err());
        }

        if (case["guide"] == "bins" || case["guide"] == "coloursteps") && case["breaks"] != "null" {
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(old["version"], 58);
            old["version"] = json!(57);
            assert!(Plot::from_json_with_extensions(&old.to_string(), registry.clone()).is_err());
        }
        let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        assert!(calls.lock().unwrap().is_empty());
        let prepared = restored.chart().unwrap().prepare().unwrap();
        let recorded = calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call["values"].clone())
            .collect::<Vec<_>>();
        let expected_calls = case["calls"].as_array().unwrap();
        assert_eq!(
            recorded.len(),
            expected_calls.len(),
            "callback count: {case}"
        );
        for (actual, expected) in recorded.iter().zip(expected_calls) {
            if actual.as_array().unwrap().len() == 300 {
                assert_eq!(expected.as_array().unwrap().len(), 300);
                // The captured decimal ramp is compared in normalized units.
                for (a, b) in actual
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(expected.as_array().unwrap())
                {
                    assert!(
                        (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 2e-15,
                        "ramp sample {a} != {b}"
                    );
                }
            } else {
                assert!(
                    equivalent(actual, expected),
                    "callback batch {actual}: {case}"
                );
            }
        }
        let layer = &prepared.layers()[0];
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
        let actual = entries.iter().filter(|e| e.visible).collect::<Vec<_>>();
        let expected = case["result"]["guides"].as_array().unwrap();
        if expected.is_empty() {
            assert!(actual.is_empty(), "suppressed guide: {case}");
        } else {
            assert_eq!(expected.len(), 1);
            assert!(
                equivalent(
                    &json!(actual.iter().map(|e| number(e.value.0)).collect::<Vec<_>>()),
                    if case["guide"] == "bins" || case["guide"] == "coloursteps" {
                        &expected[0]["source_values"]
                    } else if case["guide"] == "colourbar" {
                        &case["result"]["raw_breaks"]
                    } else {
                        &expected[0]["values"]
                    }
                ),
                "keys: {case}"
            );
            assert_eq!(
                json!(actual.iter().map(|e| e.label.clone()).collect::<Vec<_>>()),
                expected[0]["labels"],
                "labels: {case}"
            );
        }
        if matches!(case["guide"].as_str(), Some("bins" | "coloursteps")) && !expected.is_empty() {
            let mapped = actual
                .iter()
                .map(
                    |entry| match entry.mapped.as_ref().expect("interval mapping") {
                        Value::Missing | Value::Null => Json::Null,
                        Value::Number(value) => number(value.0),
                        Value::Text(value) => json!(value),
                        value => panic!("unexpected interval value: {value:?}"),
                    },
                )
                .collect::<Vec<_>>();
            assert!(
                equivalent(&json!(mapped), &expected[0]["mapped"]),
                "mapped interval keys: {case}"
            );
        }
        let ramp = layer
            .color_legend()
            .map(|g| g.colorbar.as_slice())
            .unwrap_or_default();
        if case["guide"] == "colourbar" && channel == "colour" && !expected.is_empty() {
            assert_eq!(ramp.len(), 300);
            for (i, sample) in ramp.iter().enumerate() {
                let value = expected[0]["decor_values"][i].as_f64().unwrap();
                assert!((sample.value.0 - value).abs() <= 8e-15);
                assert_eq!(
                    sample.color,
                    chart_core::color::parse_r(expected[0]["decor_colors"][i].as_str().unwrap())
                        .unwrap()
                        .resolve()
                );
            }
        } else {
            assert!(ramp.is_empty());
        }
        checked += 1;
    }
    assert_eq!(checked, 189);
}

#[test]
fn generic_guide_suppression_matches_324_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/generic-guide-suppression.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 324);
    for case in cases {
        let channel = case["channel"].as_str().unwrap();
        let discrete = case["family"] == "discrete";
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let registry = Arc::new(registry);
        let mut adapted = case.clone();
        adapted["source"] = json!("explicit");
        let mut scale = scale(&adapted)
            .with_palette_function(operation(channel, "generic"))
            .unwrap();
        match scale.ggplot.as_deref_mut().unwrap() {
            GgplotScalePolicy::Discrete { limits, .. } => {
                if case["limit_mode"] == "fixed" {
                    *limits = Some(["a", "b", "c"].map(|v| ScaleKey::Text(v.into())).to_vec());
                }
            }
            GgplotScalePolicy::Binned(policy) => {
                if case["limit_mode"] == "fixed" {
                    policy.limits = Some([Some(Number(0.)), Some(Number(4.))]);
                }
                if case["breaks"] != "default" {
                    policy.breaks = GgplotBreaks::Explicit(vec![]);
                }
            }
            _ => unreachable!(),
        }
        let hidden = case["breaks"] == "null" || case["guide"] == "none";
        scale = scale
            .with_guide(if hidden {
                GgplotScaleGuide::Hidden
            } else if discrete {
                GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
                    breaks: (case["breaks"] == "empty").then(Vec::new),
                    ..Default::default()
                })
            } else if case["guide"] == "legend" {
                GgplotScaleGuide::BinnedLegend(GgplotGuideLabels::Automatic)
            } else {
                GgplotScaleGuide::BinnedBins(GgplotGuideLabels::Automatic)
            })
            .unwrap();
        let mut draft = plot(data(&adapted))
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone());
        if channel == "colour" {
            draft = draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points());
        } else {
            draft = draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
                if channel == "size" {
                    NumericAesthetic::Size
                } else {
                    NumericAesthetic::Alpha
                },
                "v",
                scale,
            ));
        }
        let authored = draft.build().unwrap();
        let wire = authored.to_json().unwrap();
        let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        assert!(calls.lock().unwrap().is_empty());
        let prepared = restored.chart().unwrap().prepare().unwrap();
        let recorded = calls.lock().unwrap();
        let expected = case["calls"].as_array().unwrap();
        assert_eq!(recorded.len(), expected.len(), "callback count: {case}");
        for (a, b) in recorded.iter().zip(expected) {
            let a = a["values"].as_array().unwrap();
            let b = b.as_array().unwrap();
            assert_eq!(a.len(), b.len(), "batch length: {case}");
            for (a, b) in a.iter().zip(b) {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    assert!((a - b).abs() <= 2e-15, "batch values: {case}");
                } else {
                    assert_eq!(a, b, "batch values: {case}");
                }
            }
        }
        if case["result"]["guides"].as_array().unwrap().is_empty() {
            let layer = &prepared.layers()[0];
            if channel == "colour" {
                assert!(
                    layer.color_legend().is_none_or(|g| g.entries.is_empty()),
                    "suppressed color guide: {case}"
                );
            } else {
                assert!(
                    layer
                        .numeric_value_guides()
                        .values()
                        .flatten()
                        .all(|e| !e.visible),
                    "suppressed value guide: {case}"
                );
            }
        }
    }
}

#[test]
fn builtin_interval_guides_sample_keys_and_preserve_continuous_marks() {
    for channel in ["colour", "size", "alpha"] {
        let case = json!({"family":"continuous", "source":"explicit", "channel":channel, "inputs":[0,1,2,3,4]});
        let mut mapped = scale(&case);
        mapped.palette_function = None;
        mapped.palette_theme_aesthetics.clear();
        if let Some(GgplotScalePolicy::Continuous { limits, .. }) = mapped.ggplot.as_deref_mut() {
            *limits = Some([Some(Number(0.)), Some(Number(4.))]);
        }
        if channel != "colour" {
            let ScaleFunctionSpec::Interpolated(scale) = &mut mapped.function else {
                unreachable!()
            };
            scale.output = serde_json::from_value(json!({"Interpolate":{"operation":"PowerRange", "range": if channel == "size" { [1.,5.] } else { [0.,1.] }, "exponent":1, "absolute":false}})).unwrap();
        }
        let mut baseline = None;
        for guide in [
            GgplotScaleGuide::Continuous(Default::default()),
            GgplotScaleGuide::ContinuousBins(Default::default()),
            GgplotScaleGuide::ContinuousSteps(Default::default()),
        ] {
            let bins = matches!(guide, GgplotScaleGuide::ContinuousBins(_));
            let steps = matches!(guide, GgplotScaleGuide::ContinuousSteps(_));
            let scale = mapped.clone().with_guide(guide).unwrap();
            let draft = plot(data(&case)).profile(Profile::Ggplot2_4_0_3);
            let draft = if channel == "colour" {
                draft
                    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                    .scale(color_mapped("v", scale))
                    .layer(points())
            } else {
                draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
                    if channel == "size" {
                        NumericAesthetic::Size
                    } else {
                        NumericAesthetic::Alpha
                    },
                    "v",
                    scale,
                ))
            };
            let prepared = draft.build().unwrap().chart().unwrap().prepare().unwrap();
            let layer = &prepared.layers()[0];
            let styles = layer.marks().iter().map(|m| m.style).collect::<Vec<_>>();
            assert_eq!(styles.len(), 5);
            if let Some(baseline) = &baseline {
                assert_eq!(&styles, baseline);
            } else {
                baseline = Some(styles);
            }
            if !bins && !steps {
                continue;
            }
            let entries = if channel == "colour" {
                layer.color_legend().unwrap().numeric_breaks.clone()
            } else {
                layer
                    .numeric_value_guides()
                    .values()
                    .flatten()
                    .cloned()
                    .collect()
            };
            if steps && channel != "colour" {
                assert!(entries.is_empty());
                continue;
            }
            assert_eq!(entries.len(), if bins { 5 } else { 3 });
            assert!(entries.iter().all(|e| e.mapped.is_some()));
            if bins {
                assert_eq!(entries[4].mapped, Some(Value::Missing));
                if channel != "colour" {
                    for (i, entry) in entries[..4].iter().enumerate() {
                        let midpoint = (i as f64 + 0.5) / 4.;
                        assert_eq!(
                            entry.mapped,
                            Some(Value::Number(Number(if channel == "size" {
                                1. + 4. * midpoint
                            } else {
                                midpoint
                            })))
                        );
                    }
                }
            }
        }
    }
}
