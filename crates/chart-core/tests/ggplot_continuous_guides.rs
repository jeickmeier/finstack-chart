//! FIX-GG04: continuous candidate and label semantics from the pinned R scale.
use chart_core::{interpolate::Number, scales::*};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn equal(a: f64, b: f64) {
    assert!(
        a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.),
        "{a} != {b}"
    );
}
fn guide(bm: &str, lm: &str) -> GgplotContinuousGuide {
    GgplotContinuousGuide {
        breaks: match bm {
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
        labels: match lm {
            "auto" => GgplotGuideLabels::Automatic,
            "hidden" => GgplotGuideLabels::Hidden,
            "short" => GgplotGuideLabels::Explicit(vec![Some("a".into()), Some("b".into())]),
            _ => GgplotGuideLabels::Explicit(
                (1..=if bm == "explicit" {
                    10
                } else if bm == "empty" {
                    0
                } else {
                    5
                })
                    .map(|i| Some(format!("L{i}")))
                    .collect(),
            ),
        },
        count: None,
    }
}
#[test]
fn one_hundred_sixty_pinned_continuous_guide_cases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 160);
    for case in cases {
        let mut guide = guide(
            case["break_mode"].as_str().unwrap(),
            case["label_mode"].as_str().unwrap(),
        );
        guide.count = case["count"].as_f64();
        let family = match case["transform"].as_str().unwrap() {
            "sqrt" => NumericFamily::Pow { exponent: 0.5 },
            "log10" => NumericFamily::Log { base: 10. },
            _ => NumericFamily::Linear,
        };
        let domain = [
            Number(number(&case["limits"][0])),
            Number(number(&case["limits"][1])),
        ];
        let result = guide.resolve(domain, family, case["transform"] == "reverse", 1000, 65536);
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let expected = &case["result"];
        let values = expected["breaks"].as_array().unwrap();
        assert_eq!(actual.len(), values.len(), "{case}");
        for (a, b) in actual.iter().zip(values) {
            equal(a.transformed.0, number(b));
        }
        let labels = actual.iter().map(|v| v.label.clone()).collect::<Vec<_>>();
        if expected["hidden"] == true {
            assert!(labels.iter().all(Option::is_none));
        } else {
            assert_eq!(
                labels,
                expected["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>(),
                "{case}"
            );
        }
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
fn primary_continuous_guide_matches_reference_labels_and_mark_colors() {
    use chart_core::prelude::*;
    let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
        unreachable!()
    };
    let ScaleFunctionSpec::Interpolated(mapping) = &mut scale.function else {
        unreachable!()
    };
    mapping.normalization = NormalizationSpec::sequential(NumericFamily::Log { base: 10. });
    scale = scale
        .with_ggplot(GgplotScalePolicy::Continuous {
            empty_population: false,
            nonfinite_population: false,
            limits: Some([Some(Number(0.8)), Some(Number(8.4))]),
            oob: GgplotOob::Censor,
        })
        .unwrap();
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("v", [1., 3., 5.])
        .build()
        .unwrap();
    let make = |scale| {
        plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .build()
            .unwrap()
    };
    let base = make(scale.clone());
    let round = Plot::from_json(&base.to_json().unwrap()).unwrap();
    let prepared = round.chart().unwrap().prepare().unwrap();
    let layer = &prepared.layers()[0];
    let legend = layer.color_legend().unwrap();
    assert_eq!(
        legend
            .entries
            .iter()
            .map(|e| e.0.as_str())
            .collect::<Vec<_>>(),
        ["1.0", "3.0", "5.0"]
    );
    for ((_, color), mark) in legend.entries.iter().zip(layer.marks()) {
        assert_eq!(*color, mark.style.color);
    }
    let selected = scale
        .clone()
        .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
            breaks: Some([-1., 1., 20.].map(Number).to_vec()),
            labels: GgplotGuideLabels::Explicit(
                ["Outside", "One", "Outside"]
                    .map(|s| Some(s.into()))
                    .to_vec(),
            ),
            count: Some(-1.),
        }))
        .unwrap();
    let edited = make(selected);
    let edited = Plot::from_json(&edited.to_json().unwrap()).unwrap();
    let chart = edited.chart().unwrap().prepare().unwrap();
    assert_eq!(
        chart.layers()[0].color_legend().unwrap().entries,
        vec![("One".into(), layer.marks()[0].style.color)]
    );
    assert_eq!(chart.layers()[0].marks(), layer.marks());
    let no_guide = make(scale.clone().with_guide(GgplotScaleGuide::Hidden).unwrap())
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    assert!(no_guide.layers()[0].color_legend().is_none());
    assert_eq!(no_guide.layers()[0].marks(), layer.marks());
    let hidden = scale
        .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
            labels: GgplotGuideLabels::Hidden,
            ..Default::default()
        }))
        .unwrap();
    let hidden = make(hidden).chart().unwrap().prepare().unwrap();
    assert!(
        hidden.layers()[0]
            .color_legend()
            .unwrap()
            .entries
            .iter()
            .all(|(label, _)| label.is_empty())
    );
    assert_eq!(hidden.layers()[0].marks(), layer.marks());
}
#[test]
fn identity_guide_shares_transforms_and_limits_without_rescaling_values() {
    let mut scale = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
        GgplotNumericIdentity {
            transform: Some(ScaleTransform::Log { base: 10. }),
            limits: Some([Some(Number(0.8)), Some(Number(8.4))]),
            guide: true,
            trained: None,
            has_population: false,
        },
    ));
    scale.training = ScaleTraining::Eligible;
    let scale = MappedScale::new(
        scale
            .trained(&[Some(Number(1.)), Some(Number(5.))])
            .unwrap(),
    )
    .unwrap();
    let entries = scale.continuous_guide_entries(100, 4096).unwrap().unwrap();
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.visible)
            .map(|e| e.label.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["1.0", "3.0", "5.0"]
    );
    assert_eq!(
        scale.numeric(Some(100.)).unwrap(),
        chart_core::interpolate::Value::Number(Number(2.))
    );
    let guide = GgplotContinuousGuide {
        breaks: Some(vec![Number(1.), Number(2.)]),
        ..Default::default()
    };
    assert!(
        guide
            .resolve(
                [Number(1.), Number(3.)],
                NumericFamily::Linear,
                false,
                1,
                4096
            )
            .is_err()
    );
    assert!(
        guide
            .resolve([Number(1.), Number(3.)], NumericFamily::Linear, false, 2, 0)
            .is_err()
    );
    assert!(
        guide
            .resolve(
                [Number(1.), Number(1.)],
                NumericFamily::Linear,
                false,
                0,
                4096
            )
            .is_err()
    );
    assert!(
        guide
            .resolve(
                [Number(1.), Number(3.)],
                NumericFamily::Pow { exponent: 0. },
                false,
                100,
                4096
            )
            .is_err()
    );
}

#[test]
fn zero_row_training_does_not_turn_fallback_domains_into_guides() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/empty-continuous-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 18);
    for case in cases {
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
            unreachable!()
        };
        let limits = match case["limits"].as_str().unwrap() {
            "partial" => Some([None, Some(Number(10.))]),
            "full" => Some([Some(Number(1.)), Some(Number(10.))]),
            _ => None,
        };
        scale = if case["family"] == "identity" {
            MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
                GgplotNumericIdentity {
                    guide: true,
                    limits,
                    ..Default::default()
                },
            ))
        } else {
            scale
                .with_ggplot(GgplotScalePolicy::Continuous {
                    empty_population: false,
                    nonfinite_population: false,
                    limits,
                    oob: GgplotOob::Censor,
                })
                .unwrap()
        };
        scale = scale
            .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                breaks: match case["breaks"].as_str().unwrap() {
                    "explicit" => Some(vec![Number(1.), Number(2.)]),
                    "empty" => Some(vec![]),
                    _ => None,
                },
                ..Default::default()
            }))
            .unwrap();
        scale.training = ScaleTraining::Eligible;
        // Replacing a populated batch by zero rows must replace its guide state too.
        let populated = scale
            .trained(&[Some(Number(1.)), Some(Number(2.))])
            .unwrap();
        let empty = populated.trained(&[]).unwrap();
        let restored: MappedScaleSpec =
            serde_json::from_str(&serde_json::to_string(&empty).unwrap()).unwrap();
        let entries = MappedScale::new(restored)
            .unwrap()
            .continuous_guide_entries(100, 4096)
            .unwrap()
            .unwrap();
        let expected = case["values"].as_array().unwrap();
        assert_eq!(entries.len(), expected.len(), "{case}");
        for (entry, value) in entries.iter().zip(expected) {
            equal(entry.transformed.0, value.as_f64().unwrap());
        }
        assert_eq!(
            entries
                .iter()
                .map(|e| e.label.as_deref())
                .collect::<Vec<_>>(),
            case["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            "{case}"
        );
        assert_eq!(
            empty
                .trained(&[Some(Number(1.)), Some(Number(2.))])
                .unwrap(),
            populated
        );
    }
}

#[test]
fn primary_empty_continuous_color_has_no_fallback_legend_entries() {
    use chart_core::prelude::*;
    let ColorScale::Mapped { scale, .. } = ggplot_color_default(true).unwrap() else {
        unreachable!()
    };
    let data = Data::columns()
        .column("x", Vec::<f64>::new())
        .column("v", Vec::<f64>::new())
        .build()
        .unwrap();
    let plot = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.).color("v").color_scale("v"))
        .scale(color_mapped("v", scale))
        .layer(points())
        .build()
        .unwrap();
    let round = Plot::from_json(&plot.to_json().unwrap()).unwrap();
    let prepared = round.chart().unwrap().prepare().unwrap();
    assert!(prepared.layers()[0].marks().is_empty());
    assert!(
        prepared.layers()[0]
            .color_legend()
            .is_none_or(|legend| legend.entries.is_empty())
    );
}

#[test]
fn nonfinite_populations_match_192_reference_candidate_label_and_censor_cases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/nonfinite-continuous-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 192);
    for case in cases {
        let limits = match case["limits"].as_str().unwrap() {
            "lower" => Some([Some(Number(1.)), None]),
            "upper" => Some([None, Some(Number(10.))]),
            "full" => Some([Some(Number(1.)), Some(Number(10.))]),
            _ => None,
        };
        let family = match case["transform"].as_str().unwrap() {
            "sqrt" => NumericFamily::Pow { exponent: 0.5 },
            "log10" => NumericFamily::Log { base: 10. },
            _ => NumericFamily::Linear,
        };
        let reverse = case["transform"] == "reverse";
        let mut scale = if case["family"] == "identity" {
            MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
                GgplotNumericIdentity {
                    guide: true,
                    limits,
                    transform: match case["transform"].as_str().unwrap() {
                        "sqrt" => Some(ScaleTransform::Sqrt),
                        "log10" => Some(ScaleTransform::Log { base: 10. }),
                        "reverse" => Some(ScaleTransform::Reverse),
                        _ => None,
                    },
                    ..Default::default()
                },
            ))
        } else {
            let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
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
            scale
                .with_ggplot(GgplotScalePolicy::Continuous {
                    empty_population: false,
                    nonfinite_population: false,
                    limits,
                    oob: GgplotOob::Censor,
                })
                .unwrap()
        };
        scale = scale
            .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                breaks: match case["breaks"].as_str().unwrap() {
                    "explicit" => Some([0., 1., 2., 20.].map(Number).to_vec()),
                    "empty" => Some(vec![]),
                    _ => None,
                },
                ..Default::default()
            }))
            .unwrap();
        scale.training = ScaleTraining::Eligible;
        let population = if case["population"] == "missing" {
            [None, None]
        } else {
            [Some(Number(f64::INFINITY)), Some(Number(f64::NEG_INFINITY))]
        };
        let populated = scale
            .trained(&[Some(Number(1.)), Some(Number(2.))])
            .unwrap();
        let trained = populated.trained(&population).unwrap();
        assert_eq!(
            trained
                .trained(&[Some(Number(1.)), Some(Number(2.))])
                .unwrap(),
            populated
        );
        let restored: MappedScaleSpec =
            serde_json::from_str(&serde_json::to_string(&trained).unwrap()).unwrap();
        let prepared = MappedScale::new(restored).unwrap();
        let actual = prepared.continuous_guide_entries(100, 4096);
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(actual.is_err(), "{case}");
            continue;
        }
        let actual = actual.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
        let breaks = expected["breaks"].as_array().unwrap();
        if !breaks.is_empty() {
            assert!(
                prepared.continuous_guide_entries(0, 4096).is_err(),
                "{case}"
            );
            assert!(prepared.continuous_guide_entries(100, 0).is_err(), "{case}");
        }
        assert_eq!(actual.len(), breaks.len(), "{case}");
        for (entry, value) in actual.iter().zip(breaks) {
            equal(entry.transformed.0, number(value));
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
fn primary_all_missing_color_suppresses_guides_without_removing_marks() {
    use chart_core::prelude::*;
    let ColorScale::Mapped { scale, .. } = ggplot_color_default(true).unwrap() else {
        unreachable!()
    };
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("v", [None::<f64>, None])
        .build()
        .unwrap();
    let make = |scale| {
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .build()
            .unwrap();
        Plot::from_json(&p.to_json().unwrap())
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap()
    };
    let automatic = make(scale.clone());
    let explicit = make(
        scale
            .with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                breaks: Some([0., 1., 2., 20.].map(Number).to_vec()),
                ..Default::default()
            }))
            .unwrap(),
    );
    for prepared in [&automatic, &explicit] {
        assert_eq!(prepared.layers()[0].marks().len(), 2);
        assert!(
            prepared.layers()[0]
                .color_legend()
                .is_none_or(|g| g.entries.is_empty())
        );
    }
    assert_eq!(automatic.layers()[0].marks(), explicit.layers()[0].marks());
}
