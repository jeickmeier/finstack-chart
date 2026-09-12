//! FIX-GG04: transformed missing limits across finite, empty and nonfinite populations.
use chart_core::{interpolate::Number, scales::*};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn specification(case: &serde_json::Value) -> chart_core::ChartResult<MappedScaleSpec> {
    let limits = Some([
        case["limits"][0].as_f64().map(Number),
        case["limits"][1].as_f64().map(Number),
    ]);
    let family = if case["transform"] == "sqrt" {
        NumericFamily::Pow { exponent: 0.5 }
    } else {
        NumericFamily::Log { base: 10. }
    };
    let breaks =
        (case["explicit"] == true).then(|| [-1., 0., 1., 2., 10., 20.].map(Number).to_vec());
    let mut scale = if case["kind"] == "identity" {
        MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                guide: true,
                limits,
                transform: Some(if case["transform"] == "sqrt" {
                    ScaleTransform::Sqrt
                } else {
                    ScaleTransform::Log { base: 10. }
                }),
                ..Default::default()
            },
        ))
        .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
            breaks,
            ..Default::default()
        }))?
    } else {
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true)? else {
            unreachable!()
        };
        let ScaleFunctionSpec::Interpolated(s) = &mut scale.function else {
            unreachable!()
        };
        s.normalization = NormalizationSpec::Ggplot {
            timestamp: None,
            family,
            domain: [Number(1.), Number(10.)],
            reverse: false,
            rescaler: GgplotRescaler::Range,
        };
        if case["kind"] == "continuous" {
            scale
                .with_ggplot(GgplotScalePolicy::Continuous {
                    empty_population: false,
                    nonfinite_population: false,
                    limits,
                    oob: GgplotOob::Censor,
                })?
                .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                    breaks,
                    ..Default::default()
                }))?
        } else {
            scale
                .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                    limits,
                    breaks: breaks.map_or_else(
                        || {
                            if case["kind"] == "binned_equal" {
                                GgplotBreaks::Equal(5.)
                            } else {
                                GgplotBreaks::Nice(5.)
                            }
                        },
                        GgplotBreaks::Explicit,
                    ),
                    ..Default::default()
                })))?
                .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))?
        }
    };
    let values = match case["population"].as_str().unwrap() {
        "finite" => vec![Some(Number(1.)), Some(Number(10.))],
        "empty" => vec![],
        "missing" => vec![None, None],
        _ => vec![Some(Number(f64::INFINITY)), Some(Number(f64::NEG_INFINITY))],
    };
    scale.training = ScaleTraining::Eligible;
    scale.trained(&values)
}
#[test]
fn transformed_missing_limits_match_320_reference_population_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transformed-missing-limits.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 320);
    for case in cases {
        let result = specification(case)
            .and_then(|s| {
                let restored: MappedScaleSpec =
                    serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
                assert_eq!(s, restored);
                MappedScale::new(restored)
            })
            .and_then(|s| {
                if case["kind"].as_str().unwrap().starts_with("binned") {
                    s.binned_guide_entries(100, 4096)
                } else {
                    s.continuous_guide_entries(100, 4096)
                }
            });
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
        let values = expected["breaks"].as_array().unwrap();
        assert_eq!(actual.len(), values.len(), "{case}");
        for (entry, value) in actual.iter().zip(values) {
            let a = entry.transformed.0;
            let b = number(value);
            assert!(
                a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.),
                "{a} != {b}: {case}"
            );
        }
        assert_eq!(
            actual
                .iter()
                .map(|v| v.label.as_deref())
                .collect::<Vec<_>>(),
            expected["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            "{case}"
        );
        assert_eq!(
            actual.iter().map(|v| v.visible).collect::<Vec<_>>(),
            expected["visible"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_bool().unwrap())
                .collect::<Vec<_>>(),
            "{case}"
        );
    }
}

#[test]
fn primary_missing_limit_populations_keep_missing_color_marks_and_empty_guide_rules() {
    use chart_core::prelude::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transformed-missing-limits.json"
    ))
    .unwrap();
    let reference = fixture["primary"].as_array().unwrap();
    assert_eq!(reference.len(), 24);
    for source in reference {
        let mut case = source.clone();
        case["explicit"] = serde_json::json!(false);
        case["population"] = serde_json::json!("finite");
        let recipe = specification(&case).unwrap();
        let population = source["population"].as_str().unwrap();
        let values = match population {
            "empty" => vec![],
            "missing" => vec![None, None],
            _ => vec![Some(f64::INFINITY), Some(f64::NEG_INFINITY)],
        };
        let count = values.len();
        let data = Data::columns()
            .column("x", (0..count).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", values)
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", recipe.clone()))
            .layer(points())
            .build()
            .unwrap();
        let result = Plot::from_json(&p.to_json().unwrap())
            .unwrap()
            .chart()
            .unwrap()
            .prepare();
        let expected = &source["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{source}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{e:?}: {source}"));
        assert_eq!(prepared.layers()[0].marks().len(), count);
        let has_guide = prepared.layers()[0]
            .color_legend()
            .is_some_and(|g| !g.entries.is_empty());
        assert_eq!(
            usize::from(has_guide),
            expected["guides"].as_u64().unwrap() as usize,
            "{source}"
        );
        assert_eq!(expected["colors"].as_array().unwrap().len(), count);
        if count != 0 {
            assert!(
                expected["colors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|v| v == "grey50")
            );
            assert!(prepared.layers()[0].marks().iter().all(|m| m.style.color
                == chart_core::scene::Color {
                    red: 127,
                    green: 127,
                    blue: 127,
                    alpha: 255
                }));
        }
        case["population"] = source["population"].clone();
        let trained = specification(&case).unwrap();
        let restored = trained
            .trained(&[Some(Number(1.)), Some(Number(10.))])
            .unwrap();
        assert_eq!(restored, recipe);
    }
}
