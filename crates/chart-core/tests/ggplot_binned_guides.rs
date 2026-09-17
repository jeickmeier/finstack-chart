//! FIX-GG04: scale-owned binned candidates and labels, independent of guide layout.
use chart_core::{interpolate::Number, scales::*};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Inf" | "Infinity") => f64::INFINITY,
        Some("-Inf" | "-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn equal(a: f64, b: f64) {
    assert!(
        a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.),
        "{a} != {b}"
    );
}
fn spec(case: &serde_json::Value) -> chart_core::ChartResult<MappedScaleSpec> {
    let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true)? else {
        unreachable!()
    };
    let domain = [
        Number(number(&case["domain"][0])),
        Number(number(&case["domain"][1])),
    ];
    let ScaleFunctionSpec::Interpolated(s) = &mut scale.function else {
        unreachable!()
    };
    s.normalization = NormalizationSpec::Ggplot {
        timestamp: None,
        family: match case["transform"].as_str().unwrap() {
            "sqrt" => NumericFamily::Pow { exponent: 0.5 },
            "log10" => NumericFamily::Log { base: 10. },
            _ => NumericFamily::Linear,
        },
        domain,
        reverse: case["transform"] == "reverse",
        rescaler: GgplotRescaler::Range,
    };
    let breaks = match case["breaks"].as_str().unwrap() {
        "explicit" => GgplotBreaks::Explicit(case.get("cuts").map_or_else(
            || [0., 1., 1., 3., 20.].map(Number).to_vec(),
            |cuts| {
                cuts.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| Number(number(v)))
                    .collect()
            },
        )),
        "empty" => GgplotBreaks::Explicit(vec![]),
        "equal" => GgplotBreaks::Equal(case["count"].as_f64().unwrap_or(5.)),
        _ => GgplotBreaks::Nice(case["count"].as_f64().unwrap_or(5.)),
    };
    let labels = match case["labels"].as_str().unwrap() {
        "hidden" => GgplotGuideLabels::Hidden,
        "explicit" => GgplotGuideLabels::Explicit(if case["breaks"] == "empty" {
            vec![]
        } else {
            (1..=5).map(|i| Some(format!("L{i}"))).collect()
        }),
        _ => GgplotGuideLabels::Automatic,
    };
    scale
        .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
            breaks,
            limits: match case["limits"].as_str().unwrap_or("") {
                "full" => Some(domain.map(Some)),
                "lower" => Some([Some(Number(1.)), None]),
                "upper" => Some([None, Some(Number(10.))]),
                _ => case["limits"]
                    .as_array()
                    .map(|v| [Some(Number(number(&v[0]))), Some(Number(number(&v[1])))]),
            },
            ..Default::default()
        })))?
        .with_guide(GgplotScaleGuide::Binned(labels))?
        .trained(&domain.map(Some))
}
#[test]
fn binned_candidates_match_288_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 288);
    for case in cases {
        let result = spec(case)
            .and_then(MappedScale::new)
            .and_then(|s| s.binned_guide_entries(100, 4096));
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
        let breaks = expected["breaks"].as_array().unwrap();
        assert_eq!(actual.len(), breaks.len(), "{case}");
        for (entry, value) in actual.iter().zip(breaks) {
            equal(entry.transformed.0, number(value));
        }
        if expected["hidden"] == true {
            assert!(actual.iter().all(|v| v.label.is_none()), "{case}");
        } else {
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
fn primary_binned_metadata_keeps_cut_order_labels_and_mapping_through_json() {
    use chart_core::prelude::*;
    let case = serde_json::json!({"transform":"identity","domain":[1,10],"limits":"full","breaks":"explicit","labels":"auto"});
    let automatic = spec(&case).unwrap();
    let authored = automatic
        .clone()
        .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Explicit(
            (1..=5).map(|i| Some(format!("Cut {i}"))).collect(),
        )))
        .unwrap();
    let data = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("v", [1., 2., 3., 5.])
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
    let mut empty_case = case.clone();
    empty_case["breaks"] = serde_json::json!("empty");
    let empty = make(spec(&empty_case).unwrap());
    assert_eq!(empty.layers()[0].marks().len(), 4);
    assert!(
        empty.layers()[0]
            .color_legend()
            .is_none_or(|g| g.entries.is_empty())
    );
    let automatic = make(automatic);
    let authored = make(authored);
    assert_eq!(automatic.layers()[0].marks(), authored.layers()[0].marks());
    let legend = authored.layers()[0].color_legend().unwrap();
    assert_eq!(
        legend
            .numeric_breaks
            .iter()
            .map(|v| v.value.0)
            .collect::<Vec<_>>(),
        [0., 1., 1., 3., 20.]
    );
    assert_eq!(
        legend
            .numeric_breaks
            .iter()
            .map(|v| v.label.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("Cut 1"),
            Some("Cut 2"),
            Some("Cut 3"),
            Some("Cut 4"),
            Some("Cut 5")
        ]
    );
    assert_eq!(
        legend
            .numeric_breaks
            .iter()
            .map(|v| v.visible)
            .collect::<Vec<_>>(),
        [false, true, true, true, false]
    );
    let mapping = MappedScale::for_colors(legend.mapping.clone().unwrap()).unwrap();
    assert_eq!(
        mapping.binned_guide_entries(100, 4096).unwrap().unwrap(),
        legend.numeric_breaks
    );
    for (mark, value) in authored.layers()[0].marks().iter().zip([1., 2., 3., 5.]) {
        assert_eq!(
            mark.style.color,
            mapping.color(Some(value), None, legend.missing).unwrap()
        );
    }
    assert!(mapping.binned_guide_entries(4, 4096).is_err());
    assert!(mapping.binned_guide_entries(100, 0).is_err());
    assert_eq!(
        serde_json::to_value(legend).unwrap()["numeric_breaks"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn transformed_endpoint_bin_colors_match_independent_reference_samples() {
    use chart_core::color::Paint;
    // R 4.6.1 / ggplot2 4.0.3: train transform(x), get_breaks(), then map(transform(x)).
    // These cases independently exercise both newly exposed limit-order boundaries.
    let cases = [
        (
            "sqrt",
            [1., 10.],
            vec![1., 2.5, 5., 7.5, 10.],
            vec!["#214667", "#214667", "#35709F", "#4186BE", "#4A99D7"],
        ),
        (
            "reverse",
            [4., 4.],
            vec![4., 4.],
            vec!["#336A98", "#336A98"],
        ),
    ];
    let missing = Paint::from_css("#808080").unwrap().resolve();
    for (transform, domain, values, expected) in cases {
        let scale = MappedScale::for_colors(spec(&serde_json::json!({"transform":transform,"domain":domain,"limits":"none","breaks":"nice","labels":"auto"})).unwrap()).unwrap();
        for (value, expected) in values.into_iter().zip(expected) {
            assert_eq!(
                scale.color(Some(value), None, missing).unwrap(),
                Paint::from_css(expected).unwrap().resolve(),
                "{transform}: {value}"
            );
        }
    }
}

#[test]
fn binned_empty_and_nonfinite_populations_match_192_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-populations.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 192);
    for case in cases {
        let mut args = case.clone();
        args["domain"] = serde_json::json!([1, 10]);
        args["labels"] = serde_json::json!("auto");
        args["cuts"] = serde_json::json!([1, 2]);
        let populated = spec(&args).unwrap();
        let values = match case["population"].as_str().unwrap() {
            "empty" => vec![],
            "missing" => vec![None, None],
            _ => vec![Some(Number(f64::INFINITY)), Some(Number(f64::NEG_INFINITY))],
        };
        let trained = populated.trained(&values).unwrap();
        assert_eq!(
            trained
                .trained(&[Some(Number(1.)), Some(Number(10.))])
                .unwrap(),
            populated
        );
        let restored: MappedScaleSpec =
            serde_json::from_str(&serde_json::to_string(&trained).unwrap()).unwrap();
        let mapping = MappedScale::new(restored).unwrap();
        let result = mapping.binned_guide_entries(100, 4096);
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
        let breaks = expected["breaks"].as_array().unwrap();
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
        if !actual.is_empty() {
            assert!(mapping.binned_guide_entries(0, 4096).is_err(), "{case}");
            if actual.iter().any(|v| v.label.is_some()) {
                assert!(mapping.binned_guide_entries(100, 0).is_err(), "{case}");
            }
        }
    }
}

#[test]
fn primary_empty_binned_guides_preserve_empty_and_partial_limit_rules() {
    use chart_core::prelude::*;
    let data = Data::columns()
        .column("x", Vec::<f64>::new())
        .column("v", Vec::<f64>::new())
        .build()
        .unwrap();
    for limits in ["none", "lower"] {
        let scale = spec(&serde_json::json!({"transform":"identity","domain":[1,10],"limits":limits,"breaks":"nice","labels":"auto"})).unwrap();
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .build()
            .unwrap();
        let result = Plot::from_json(&p.to_json().unwrap())
            .unwrap()
            .chart()
            .unwrap()
            .prepare();
        if limits == "lower" {
            assert!(result.is_err());
        } else {
            let prepared = result.unwrap();
            assert!(prepared.layers()[0].marks().is_empty());
            assert!(
                prepared.layers()[0]
                    .color_legend()
                    .is_none_or(|g| g.entries.is_empty())
            );
        }
    }
}

#[test]
fn binned_counts_and_transformed_missing_limits_match_228_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-boundaries.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 228);
    compare_boundary_cases(cases);
}
#[test]
fn fractional_binned_counts_match_288_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-fractional-counts.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 288);
    compare_boundary_cases(cases);
}
fn compare_boundary_cases(cases: &[serde_json::Value]) {
    for source in cases {
        let mut case = source.clone();
        case["labels"] = serde_json::json!("auto");
        case["breaks"] = serde_json::json!(if case["nice"] == true {
            "nice"
        } else {
            "equal"
        });
        let result = spec(&case)
            .and_then(|s| {
                if let Some(bounds) = case["result"]["limits"].as_array() {
                    let ScaleFunctionSpec::Interpolated(scale) = &s.function else {
                        unreachable!()
                    };
                    let NormalizationSpec::Ggplot { domain, .. } = scale.normalization else {
                        unreachable!()
                    };
                    for (value, expected) in domain.iter().zip(bounds) {
                        let transformed = match case["transform"].as_str().unwrap() {
                            "sqrt" => value.0.sqrt(),
                            "log10" => value.0.log10(),
                            "reverse" => -value.0,
                            _ => value.0,
                        };
                        equal(transformed, number(expected));
                    }
                }
                let restored: MappedScaleSpec =
                    serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
                assert_eq!(s, restored);
                MappedScale::new(restored)
            })
            .and_then(|s| s.binned_guide_entries(100, 4096));
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{e:?}: {case}")).unwrap();
        let breaks = expected["breaks"].as_array().unwrap();
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
fn primary_boundary_limits_and_constant_count_preserve_training_and_source_definition() {
    use chart_core::prelude::*;
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("v", [1., 10.])
        .build()
        .unwrap();
    for transform in ["sqrt", "log10"] {
        let source = spec(&serde_json::json!({"transform":transform,"domain":[1,10],"limits":[-1,10],"breaks":"equal","labels":"auto"})).unwrap();
        let mut omitted = source.clone();
        let Some(GgplotScalePolicy::Binned(p)) = omitted.ggplot.as_deref_mut() else {
            unreachable!()
        };
        p.limits = Some([None, Some(Number(10.))]);
        let prepare = |scale| {
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
        let actual = prepare(source.clone());
        let expected = prepare(omitted);
        assert_eq!(actual.layers()[0].marks(), expected.layers()[0].marks());
        let a = actual.layers()[0].color_legend().unwrap();
        let b = expected.layers()[0].color_legend().unwrap();
        assert_eq!(a.entries, b.entries);
        assert_eq!(a.numeric_breaks, b.numeric_breaks);
        assert_eq!(a.intervals, b.intervals);
        let changed = source
            .trained(&[Some(Number(4.)), Some(Number(16.))])
            .unwrap();
        assert_eq!(
            changed
                .trained(&[Some(Number(1.)), Some(Number(10.))])
                .unwrap(),
            source
        );
        let Some(GgplotScalePolicy::Binned(p)) = changed.ggplot.as_deref() else {
            unreachable!()
        };
        assert_eq!(p.limits, Some([Some(Number(-1.)), Some(Number(10.))]));
    }
    let constant = spec(&serde_json::json!({"transform":"identity","domain":[4,4],"limits":null,"breaks":"nice","count":1,"labels":"auto"})).unwrap();
    let p = plot(
        Data::columns()
            .column("x", [1., 2.])
            .column("v", [4., 4.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
    .scale(color_mapped("v", constant.clone()))
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
    assert_eq!(
        prepared.layers()[0]
            .color_legend()
            .unwrap()
            .numeric_breaks
            .iter()
            .map(|v| v.value.0)
            .collect::<Vec<_>>(),
        [3.95, 4.05]
    );
    // A count of one is still invalid for an ordinary nonconstant extended search.
    assert!(
        constant
            .trained(&[Some(Number(1.)), Some(Number(10.))])
            .is_err()
    );
    for count in [0, 4097, usize::MAX] {
        assert!(spec(&serde_json::json!({"transform":"identity","domain":[4,4],"limits":null,"breaks":"nice","count":count,"labels":"auto"})).is_err());
    }
}

#[test]
fn fractional_counts_and_empty_hidden_sampling_keep_resource_boundaries() {
    use chart_core::DiagnosticCode;
    let case = serde_json::json!({"transform":"identity","domain":[1,10],"limits":"none","breaks":"equal","labels":"auto"});
    let source = spec(&case).unwrap();
    for breaks in [
        GgplotBreaks::Equal(-0.5),
        GgplotBreaks::Equal(f64::NAN),
        GgplotBreaks::Nice(0.5),
    ] {
        let mut invalid = source.clone();
        let Some(GgplotScalePolicy::Binned(p)) = invalid.ggplot.as_deref_mut() else {
            unreachable!()
        };
        p.breaks = breaks;
        assert_eq!(
            invalid.trained(&[]).unwrap_err().code,
            DiagnosticCode::Validation
        );
    }
    for breaks in [GgplotBreaks::Equal(4096.1), GgplotBreaks::Nice(4096.1)] {
        let mut oversized = source.clone();
        let Some(GgplotScalePolicy::Binned(p)) = oversized.ggplot.as_deref_mut() else {
            unreachable!()
        };
        p.breaks = breaks;
        assert_eq!(
            oversized.trained(&[]).unwrap_err().code,
            DiagnosticCode::ResourceLimit
        );
    }
    let mut empty = source.with_guide(GgplotScaleGuide::Hidden).unwrap();
    let Some(GgplotScalePolicy::Binned(p)) = empty.ggplot.as_deref_mut() else {
        unreachable!()
    };
    p.limits = Some([Some(Number(1.)), Some(Number(f64::INFINITY))]);
    let empty = empty.trained(&[]).unwrap();
    let empty: MappedScaleSpec =
        serde_json::from_str(&serde_json::to_string(&empty).unwrap()).unwrap();
    let mapping = MappedScale::for_colors(empty).unwrap();
    assert_eq!(
        mapping.numeric(Some(1.)).unwrap_err().code,
        DiagnosticCode::NumericalDomain
    );
}

#[test]
fn primary_break_selection_waits_for_the_actual_population() {
    use chart_core::prelude::*;
    let mut source = spec(&serde_json::json!({"transform":"identity","domain":[4,4],"limits":"none","breaks":"nice","count":1.1,"labels":"auto"})).unwrap();
    let ScaleFunctionSpec::Interpolated(s) = &mut source.function else {
        unreachable!()
    };
    s.normalization = NormalizationSpec::Ggplot {
        timestamp: None,
        family: NumericFamily::Linear,
        domain: [Number(1.), Number(10.)],
        reverse: false,
        rescaler: GgplotRescaler::Range,
    };
    let Some(GgplotScalePolicy::Binned(p)) = source.ggplot.as_deref_mut() else {
        unreachable!()
    };
    p.prepared_breaks = None;
    for values in [[4., 4.], [1., 10.]] {
        let data = Data::columns()
            .column("x", [0., 1.])
            .column("v", values)
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", source.clone()))
            .layer(points())
            .build()
            .unwrap();
        let result = Plot::from_json(&p.to_json().unwrap())
            .unwrap()
            .chart()
            .unwrap()
            .prepare();
        if values[0] != values[1] {
            assert!(result.is_err());
            continue;
        }
        let prepared = result.unwrap();
        assert!(prepared.layers()[0].marks().iter().all(|m| m.style.color
            == chart_core::scene::Color {
                red: 51,
                green: 106,
                blue: 152,
                alpha: 255
            }));
        let guide = prepared.layers()[0].color_legend().unwrap();
        assert_eq!(
            guide
                .numeric_breaks
                .iter()
                .map(|b| b.label.as_deref())
                .collect::<Vec<_>>(),
            [Some("3.95"), Some("4.05")]
        );
    }
}

#[test]
fn legacy_binned_candidates_keep_repeated_constant_breaks_through_primary_preparation() {
    use chart_core::prelude::*;
    let scale=spec(&serde_json::json!({"transform":"identity","domain":[4,4],"limits":[4,4],"breaks":"equal","count":1.1,"labels":"auto"})).unwrap();
    let p = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("v", [4., 4.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
    .scale(color_mapped("v", scale))
    .layer(points())
    .build()
    .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let legend = prepared.layers()[0].color_legend().unwrap();
    assert_eq!(
        legend
            .numeric_breaks
            .iter()
            .map(|b| (b.value.0, b.label.as_deref(), b.visible))
            .collect::<Vec<_>>(),
        vec![(4., Some("4"), true), (4., Some("4"), true)]
    );
    assert_eq!(prepared.layers()[0].marks().len(), 2);
}
