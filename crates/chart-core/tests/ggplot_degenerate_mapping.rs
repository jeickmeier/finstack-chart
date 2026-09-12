//! FIX-GG04: reference break metadata and sampling on zero/constant domains.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
fn authored(case: &serde_json::Value) -> chart_core::ChartResult<MappedScaleSpec> {
    let limits = (!case["limits"].is_null()).then(|| {
        [
            (!case["limits"][0].is_null()).then(|| Number(number(&case["limits"][0]))),
            (!case["limits"][1].is_null()).then(|| Number(number(&case["limits"][1]))),
        ]
    });
    let (family, transform, reverse) = match case["transform"].as_str().unwrap() {
        "sqrt" => (
            NumericFamily::Pow { exponent: 0.5 },
            Some(ScaleTransform::Sqrt),
            false,
        ),
        "log10" => (
            NumericFamily::Log { base: 10. },
            Some(ScaleTransform::Log { base: 10. }),
            false,
        ),
        "reverse" => (NumericFamily::Linear, Some(ScaleTransform::Reverse), true),
        _ => (NumericFamily::Linear, None, false),
    };
    let breaks = match case["mode"].as_str().unwrap() {
        "explicit" => Some(
            case.get("cuts")
                .and_then(serde_json::Value::as_array)
                .map_or_else(
                    || [-1., 0., 1., 2., 4., 4., 10., 20.].map(Number).to_vec(),
                    |v| v.iter().map(|v| Number(number(v))).collect(),
                ),
        ),
        "single" => Some([4., 4.].map(Number).to_vec()),
        "empty" => Some(vec![]),
        _ => None,
    };
    let mut scale = if case["kind"] == "identity" {
        MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                guide: true,
                limits,
                transform,
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
            reverse,
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
    scale.training = ScaleTraining::Eligible;
    Ok(scale)
}
fn population(case: &serde_json::Value) -> Vec<Option<Number>> {
    match case["population"].as_str() {
        Some("empty") => vec![],
        Some("missing") => vec![None, None],
        Some("infinite") => vec![Some(Number(f64::INFINITY)), Some(Number(f64::NEG_INFINITY))],
        _ => vec![Some(Number(1.)), Some(Number(10.))],
    }
}
fn specification(case: &serde_json::Value) -> chart_core::ChartResult<MappedScaleSpec> {
    authored(case)?.trained(&population(case))
}

fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn equal(a: f64, b: f64, case: &serde_json::Value) {
    assert!(
        a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.),
        "{a} != {b}: {case}"
    );
}
#[test]
fn degenerate_guide_and_mapping_match_256_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/degenerate-bin-mapping.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 256);
    compare_reference_cases(cases);
}
#[test]
fn authored_limits_match_2688_reference_population_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/authored-limit-populations.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2688);
    compare_reference_cases(cases);
}
fn compare_reference_cases(cases: &[serde_json::Value]) {
    let missing = chart_core::scene::Color {
        red: 127,
        green: 127,
        blue: 127,
        alpha: 255,
    };
    for case in cases {
        let definition = specification(case);
        let guide = definition.clone().and_then(MappedScale::new).and_then(|s| {
            if case["kind"].as_str().unwrap().starts_with("binned") {
                s.binned_guide_entries(100, 4096)
            } else {
                s.continuous_guide_entries(100, 4096)
            }
        });
        let expected = &case["guide"];
        if expected.get("error").is_some() {
            assert!(guide.is_err(), "{case}");
        } else {
            let actual = guide.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
            let breaks = expected["breaks"].as_array().unwrap();
            assert_eq!(actual.len(), breaks.len(), "{case}");
            for (a, b) in actual.iter().zip(breaks) {
                equal(a.transformed.0, number(b), case);
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
        let expected = &case["mapping"];
        let result = definition.and_then(|s| {
            let restored: MappedScaleSpec =
                serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
            assert_eq!(s, restored);
            let identity = case["kind"] == "identity";
            let mapping = if identity {
                MappedScale::new(restored)?
            } else {
                MappedScale::for_colors(restored)?
            };
            case.get("inputs")
                .and_then(serde_json::Value::as_array)
                .map_or_else(
                    || vec![Some(-1.), Some(0.), Some(1.), Some(4.), Some(10.), None],
                    |v| {
                        v.iter()
                            .map(|v| (!v.is_null()).then(|| number(v)))
                            .collect::<Vec<_>>()
                    },
                )
                .into_iter()
                .map(|v| {
                    if identity {
                        mapping.numeric(v)
                    } else {
                        mapping
                            .color(v, None, missing)
                            .map(|c| Value::Color(chart_core::color::Paint::Bytes(c).value()))
                    }
                })
                .collect::<chart_core::ChartResult<Vec<_>>>()
        });
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let values = expected["values"].as_array().unwrap();
        assert!(values.len() == 1 || values.len() == actual.len());
        for (i, a) in actual.iter().enumerate() {
            // R's constant binned mapping returns one scalar, recycled by a layer.
            let b = &values[if values.len() == 1 { 0 } else { i }];
            if case["kind"] == "identity" {
                if b.is_null() {
                    assert!(matches!(a, Value::Missing), "{a:?}: {case}");
                } else {
                    let Value::Number(v) = a else {
                        panic!("{a:?}: {case}")
                    };
                    equal(v.0, number(b), case);
                }
            } else {
                let Value::Color(c) = a else { unreachable!() };
                assert_eq!(
                    c.to_paint(),
                    chart_core::color::parse_r(b.as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "input {i}: {case}"
                );
            }
        }
    }
}

#[test]
fn primary_reference_missing_color_matches_r_and_keeps_explicit_overrides() {
    use chart_core::prelude::*;
    let ColorScale::Mapped {
        scale: reference,
        missing,
    } = ggplot_color_default(true).unwrap()
    else {
        unreachable!()
    };
    // Independent pinned R col2rgb('grey50') result; both reference authoring paths agree.
    assert_eq!(
        missing.resolve(),
        chart_core::scene::Color {
            red: 127,
            green: 127,
            blue: 127,
            alpha: 255
        }
    );
    let mut palette_only = reference.clone();
    palette_only.ggplot = None;
    palette_only.training = ScaleTraining::Authored;
    let ScaleFunctionSpec::Interpolated(s) = &mut palette_only.function else {
        unreachable!()
    };
    s.normalization = NormalizationSpec::sequential(NumericFamily::Linear);
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("v", [None::<f64>, None])
        .build()
        .unwrap();
    for (scale, default) in [(reference, 127), (palette_only, 128)] {
        for override_color in [None, Some(17)] {
            let mut builder = color_mapped("v", scale.clone());
            if let Some(value) = override_color {
                builder = builder.missing(rgb(value, value, value));
            }
            let p = plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(builder)
                .layer(points())
                .build()
                .unwrap();
            let prepared = Plot::from_json(&p.to_json().unwrap())
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            assert_eq!(prepared.layers()[0].marks().len(), 2);
            let value = override_color.unwrap_or(default);
            assert!(prepared.layers()[0].marks().iter().all(|m| m.style.color
                == chart_core::scene::Color {
                    red: value,
                    green: value,
                    blue: value,
                    alpha: 255
                }));
        }
    }
}

#[test]
fn primary_degenerate_color_scales_preserve_reference_paints_and_rejections() {
    use chart_core::prelude::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/degenerate-bin-mapping.json"
    ))
    .unwrap();
    let data = Data::columns()
        .column("x", [1., 2., 3., 4., 5., 6.])
        .column(
            "v",
            [Some(-1.), Some(0.), Some(1.), Some(4.), Some(10.), None],
        )
        .build()
        .unwrap();
    let mut count = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] != "identity")
    {
        count += 1;
        let result = specification(case).and_then(|s| {
            let p = plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", s))
                .layer(points())
                .build()?;
            Plot::from_json(&p.to_json()?)?.chart()?.prepare()
        });
        if case["guide"].get("error").is_some() || case["mapping"].get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let values = case["mapping"]["values"].as_array().unwrap();
        assert_eq!(prepared.layers()[0].marks().len(), 6, "{case}");
        for (i, mark) in prepared.layers()[0].marks().iter().enumerate() {
            let value = &values[if values.len() == 1 { 0 } else { i }];
            assert_eq!(
                mark.style.color,
                chart_core::color::parse_r(value.as_str().unwrap())
                    .unwrap()
                    .resolve(),
                "{case}"
            );
        }
    }
    assert_eq!(count, 192);
}

#[test]
fn primary_authored_limits_match_reference_chart_builds() {
    use chart_core::prelude::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/authored-limit-populations.json"
    ))
    .unwrap();
    let cases = fixture["primary"].as_array().unwrap();
    assert_eq!(cases.len(), 2016);
    for case in cases {
        let values = population(case)
            .into_iter()
            .map(|v| v.map(|v| v.0))
            .collect::<Vec<_>>();
        let count = values.len();
        let result = authored(case).and_then(|s| {
            let data = Data::columns()
                .column("x", (0..count).map(|i| i as f64).collect::<Vec<_>>())
                .column("v", values)
                .build()?;
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", s.with_guide(GgplotScaleGuide::Hidden)?))
                .layer(points())
                .build()?;
            Plot::from_json(&p.to_json()?)?.chart()?.prepare()
        });
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), count, "{case}");
        let colors = expected["colors"].as_array().unwrap();
        assert_eq!(colors.len(), count, "{case}");
        for (mark, color) in marks.iter().zip(colors) {
            assert_eq!(
                mark.style.color,
                chart_core::color::parse_r(color.as_str().unwrap())
                    .unwrap()
                    .resolve(),
                "{case}"
            );
        }
        assert!(prepared.layers()[0].color_legend().is_none(), "{case}");
        // The fixture also retains default colorbar build outcomes for GG-05.
    }
}

#[test]
fn authored_limit_retraining_replaces_population_state() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/authored-limit-populations.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let Ok(trained) = specification(case) else {
            continue;
        };
        let original = serde_json::to_string(&trained).unwrap();
        for next in ["finite", "empty", "missing", "infinite"] {
            let mut target = case.clone();
            target["population"] = serde_json::json!(next);
            let batch = specification(&target);
            let updated = trained.trained(&population(&target));
            match (batch, updated) {
                (Ok(batch), Ok(updated)) => assert_eq!(batch, updated, "{target}"),
                (Err(batch), Err(updated)) => assert_eq!(batch.code, updated.code, "{target}"),
                (batch, updated) => panic!("batch {batch:?}; updated {updated:?}: {target}"),
            }
            checked += 1;
        }
        assert_eq!(original, serde_json::to_string(&trained).unwrap());
    }
    assert!(checked > 9000, "{checked}");
}
