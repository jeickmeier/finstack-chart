//! FIX-GG04 scale policies compared with independent pinned R training/mapping.
use chart_core::{color::Paint, interpolate::Number, scales::*};
use serde_json::Value;
fn number(v: &Value) -> Option<Number> {
    v.as_f64()
        .map(Number)
        .or_else(|| match v["number"].as_str() {
            Some("Infinity") => Some(Number(f64::INFINITY)),
            Some("-Infinity") => Some(Number(f64::NEG_INFINITY)),
            _ => None,
        })
}
fn color(v: &Value) -> chart_core::scene::Color {
    let s = v.as_str().unwrap();
    Paint::from_css(if s == "grey50" { "#7f7f7f" } else { s })
        .unwrap()
        .resolve()
}
#[test]
fn ggplot_continuous_oob_and_discrete_population_match_reference() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/scales.json")).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let a = &case["args"];
        let ColorScale::Mapped { mut scale, missing } = ggplot_color_default(
            case["kind"] == "continuous"
                || case["kind"] == "transformed"
                || case["kind"] == "binned",
        )
        .unwrap() else {
            unreachable!()
        };
        let key = |v: &Value| {
            v.as_str()
                .map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into()))
        };
        match case["kind"].as_str().unwrap() {
            "continuous" => {
                let oob = match a["oob"].as_str().unwrap() {
                    "censor" => GgplotOob::Censor,
                    "censor_any" => GgplotOob::CensorAny,
                    "squish" => GgplotOob::Squish,
                    "squish_any" => GgplotOob::SquishAny,
                    "keep" => GgplotOob::Keep,
                    "squish_infinite" => GgplotOob::SquishInfinite,
                    _ => unreachable!(),
                };
                let limits = a["limits"]
                    .as_array()
                    .map(|p| [number(&p[0]), number(&p[1])]);
                scale = scale
                    .with_ggplot(GgplotScalePolicy::Continuous {
                        empty_population: false,
                        nonfinite_population: false,
                        limits,
                        oob,
                    })
                    .unwrap()
                    .trained(
                        &case["train"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(number)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
            }
            "discrete" | "manual" => {
                let manual = case["kind"] == "manual";
                let palette = if manual {
                    GgplotDiscretePalette::Manual {
                        values: ["red", "blue"]
                            .map(|s| {
                                chart_core::interpolate::Value::Color(
                                    Paint::from_css(s).unwrap().value(),
                                )
                            })
                            .to_vec(),
                        names: (a["named"] == true)
                            .then(|| vec![ScaleKey::Text("b".into()), ScaleKey::Text("a".into())]),
                    }
                } else {
                    GgplotDiscretePalette::Hue(Default::default())
                };
                scale = scale
                    .with_ggplot(GgplotScalePolicy::Discrete {
                        empty_population: false,
                        limits: a["limits"].as_array().map(|v| v.iter().map(key).collect()),
                        levels: a["levels"].as_array().map(|v| v.iter().map(key).collect()),
                        drop: manual || a["drop"] == true,
                        na_translate: manual || a["na_translate"] == true,
                        palette,
                    })
                    .unwrap();
                let trained = scale.trained_keys(
                    &case["train"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(key)
                        .collect::<Vec<_>>(),
                );
                if case["result"].get("error").is_some() {
                    assert!(trained.is_err(), "{}", case["id"]);
                    count += 1;
                    continue;
                }
                scale = trained.unwrap();
                let ScaleFunctionSpec::Ordinal(o) = &scale.function else {
                    unreachable!()
                };
                let expected = case["result"]["domain"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(key)
                    .collect::<Vec<_>>();
                assert_eq!(o.domain, expected, "{} domain", case["id"]);
            }
            "sizing" | "ordinal" => {
                let kind = match a["kind"].as_str().unwrap() {
                    "size" => GgplotNumericPalette::Size,
                    "size_area" => GgplotNumericPalette::Area,
                    "radius" => GgplotNumericPalette::Radius,
                    "alpha" => GgplotNumericPalette::Alpha,
                    "linewidth" => GgplotNumericPalette::Linewidth,
                    _ => unreachable!(),
                };
                let limits = a["limits"]
                    .as_array()
                    .map(|p| [number(&p[0]), number(&p[1])]);
                let ordinal = case["kind"] == "ordinal";
                let scale = if ordinal {
                    ggplot_numeric_ordinal(kind)
                        .unwrap()
                        .trained_keys(
                            &case["train"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(key)
                                .collect::<Vec<_>>(),
                        )
                        .unwrap()
                } else {
                    ggplot_numeric_default(kind)
                        .unwrap()
                        .with_ggplot(GgplotScalePolicy::Continuous {
                            empty_population: false,
                            nonfinite_population: false,
                            limits,
                            oob: GgplotOob::Censor,
                        })
                        .unwrap()
                        .trained(
                            &case["train"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(number)
                                .collect::<Vec<_>>(),
                        )
                        .unwrap()
                };
                let prepared = MappedScale::for_numbers(scale).unwrap();
                for (i, (input, want)) in case["input"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(case["result"]["mapped"].as_array().unwrap())
                    .enumerate()
                {
                    let got = if ordinal {
                        prepared.category(Some(&key(input))).unwrap()
                    } else {
                        prepared.numeric(number(input).map(|n| n.0)).unwrap()
                    };
                    if let Some(want) = want.as_f64() {
                        let chart_core::interpolate::Value::Number(Number(got)) = got else {
                            panic!("{} sample {i}: {got:?}", case["id"])
                        };
                        assert!(
                            (want - got).abs() <= 2e-12,
                            "{} sample {i}: {got} vs {want}",
                            case["id"]
                        );
                    } else {
                        assert_eq!(
                            got,
                            chart_core::interpolate::Value::Missing,
                            "{} sample {i}",
                            case["id"]
                        );
                    }
                }
                count += 1;
                continue;
            }
            "binned" => {
                let breaks = match a["mode"].as_str().unwrap() {
                    "nice" => GgplotBreaks::Nice(5.),
                    "equal" => GgplotBreaks::Equal(5.),
                    _ => {
                        GgplotBreaks::Explicit(a["breaks"].as_array().map_or_else(Vec::new, |v| {
                            v.iter().map(|v| number(v).unwrap()).collect()
                        }))
                    }
                };
                let limits = a["limits"]
                    .as_array()
                    .map(|v| [number(&v[0]), number(&v[1])]);
                scale = scale
                    .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                        limits,
                        breaks,
                        right: a["right"] == true,
                        ..Default::default()
                    })))
                    .unwrap()
                    .trained(
                        &case["train"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(number)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
                let Some(GgplotScalePolicy::Binned(policy)) = scale.ggplot.as_deref() else {
                    unreachable!()
                };
                let cuts = policy.prepared_breaks.as_ref().unwrap();
                let expected = case["result"]["breaks"].as_array().unwrap();
                assert_eq!(cuts.len(), expected.len());
                for (got, want) in cuts.iter().zip(expected) {
                    assert!(
                        (got.0 - want.as_f64().unwrap()).abs() < 2e-12,
                        "{} cut",
                        case["id"]
                    );
                }
            }
            "transformed" => {
                let (family, reverse) = match a["transform"].as_str().unwrap() {
                    "log10" => (NumericFamily::Log { base: 10. }, false),
                    "sqrt" => (NumericFamily::Pow { exponent: 0.5 }, false),
                    "reverse" => (NumericFamily::Linear, true),
                    _ => unreachable!(),
                };
                let ScaleFunctionSpec::Interpolated(ref mut s) = scale.function else {
                    unreachable!()
                };
                s.normalization = NormalizationSpec::Ggplot {
                    timestamp: None,
                    family,
                    domain: [Number(0.), Number(1.)],
                    reverse,
                    rescaler: GgplotRescaler::Range,
                };
                scale = scale
                    .trained(
                        &case["train"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(number)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
            }
            kind => panic!("unknown {kind}"),
        }
        let prepared = MappedScale::for_colors(scale).unwrap();
        let inputs = case["input"].as_array().unwrap();
        let expected = case["result"]["mapped"].as_array().unwrap();
        // ScaleBinned returns one scalar for a collapsed domain; ggplot's layer
        // assignment recycles that scalar across every row, including missing rows.
        let recycled = case["kind"] == "binned" && expected.len() == 1;
        assert!(
            recycled || inputs.len() == expected.len(),
            "{} output cardinality",
            case["id"]
        );
        for (i, input) in inputs.iter().enumerate() {
            let expected = &expected[if recycled { 0 } else { i }];
            if expected.is_object() {
                assert!(
                    matches!(
                        prepared
                            .category(input.as_str().map(|_| key(input)).as_ref())
                            .unwrap(),
                        chart_core::interpolate::Value::Missing
                    ),
                    "{} sample {i}",
                    case["id"]
                );
                continue;
            }
            let got = prepared
                .color(
                    number(input).map(|v| v.0),
                    input.as_str().map(|_| key(input)).as_ref(),
                    missing.resolve(),
                )
                .unwrap();
            assert_eq!(got, color(expected), "{} sample {i}", case["id"]);
        }
        count += 1;
    }
    assert_eq!(count, 93);
}

#[test]
fn ggplot_missing_paints_wait_for_after_scale_and_constants_override_mapping() {
    use chart_core::{
        grammar::{AfterScaleAesthetic as A, ExpressionBinary, ThemeRead},
        prelude::*,
    };
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [1., 2.])
        .column("group", ["a", "b"])
        .build()
        .unwrap();
    let ColorScale::Mapped { scale, .. } = ggplot_color_default(false).unwrap() else {
        unreachable!()
    };
    let scale = scale
        .with_ggplot(GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: Some(vec![ScaleKey::Text("a".into())]),
            levels: None,
            drop: true,
            na_translate: false,
            palette: GgplotDiscretePalette::Hue(Default::default()),
        })
        .unwrap();
    let base = || {
        plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").color("group").color_scale("paint"))
            .scale(color_mapped("paint", scale.clone()))
    };
    let plain = base().layer(points()).build().unwrap();
    let chart = plain.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 1);
    let recovered = base()
        .layer(
            points().after_scale(
                scale_aes().color(
                    after_scale_expr(A::Color)
                        .binary(ExpressionBinary::Coalesce, from_theme(ThemeRead::Accent)),
                ),
            ),
        )
        .build()
        .unwrap();
    let chart = recovered.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 2);
    assert_eq!(chart.layers()[0].marks()[1].style.color.alpha, 255);
    let constant = base().layer(points().color(rgb(1, 2, 3))).build().unwrap();
    let chart = constant.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 2);
    assert!(chart.layers()[0].color_legend().is_none());
    assert!(
        chart.layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.color == rgb(1, 2, 3))
    );
    let fill = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").fill("group").fill_scale("paint"))
        .scale(color_mapped("paint", scale))
        .layer(points().stroke(rgb(0, 0, 0)))
        .build()
        .unwrap();
    let chart = fill.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 2);
    assert_eq!(chart.layers()[0].marks()[1].style.fill.unwrap().alpha, 0);
}

#[test]
fn ggplot_trained_bins_obey_compile_category_budget() {
    use chart_core::{grammar::CompileLimits, prelude::*};
    let data = Data::columns()
        .column("x", [1.2, 3., 6.7])
        .column("y", [1., 2., 3.])
        .build()
        .unwrap();
    let ColorScale::Mapped { scale, .. } = ggplot_color_default(true).unwrap() else {
        unreachable!()
    };
    let scale = scale
        .with_ggplot(GgplotScalePolicy::Binned(Box::default()))
        .unwrap();
    let result = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("x").color_scale("bins"))
        .scale(color_mapped("bins", scale))
        .layer(points())
        .compile_limits(CompileLimits {
            max_groups: 2,
            ..Default::default()
        })
        .build();
    let result = result.unwrap().chart().unwrap().prepare();
    assert_eq!(
        result.unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
}

#[test]
fn ggplot_primary_point_size_scales_before_missing_removal_and_survives_edits() {
    use chart_core::{
        grammar::{AfterScaleAesthetic, ExpressionBinary, ThemeRead},
        prelude::*,
    };
    let data = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("y", [1., 2., 3., 4.])
        .column("size", [0., 1., 2., f64::NAN])
        .build()
        .unwrap();
    let base = || {
        plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").size("size"))
    };
    let p = base().layer(points().name("points")).build().unwrap();
    let expected = [1., 4.535533905932738, 6.];
    let check = |p: &Plot| {
        let chart = p.chart().unwrap().prepare().unwrap();
        let marks = chart.layers()[0].marks();
        assert_eq!(marks.len(), 3);
        for (m, expected) in marks.iter().zip(expected) {
            assert!((m.style.radius - expected).abs() < 2e-12);
            assert_eq!(
                m.style.stroke_width,
                chart_core::theme::GeometryTheme::<chart_core::scene::Color>::default().line_width
            );
        }
    };
    check(&p);
    let wire = p.to_json().unwrap();
    check(&Plot::from_json(&wire).unwrap());
    check(
        &p.edit()
            .layer("points", points().color(rgb(1, 2, 3)))
            .build()
            .unwrap(),
    );
    let fixed = base().layer(points().radius(3.)).build().unwrap();
    let chart = fixed.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 4);
    assert!(
        chart.layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.radius == 3.)
    );
    let recovered = base()
        .layer(
            points().after_scale(
                scale_aes().size(
                    after_scale_expr(AfterScaleAesthetic::Size)
                        .binary(ExpressionBinary::Coalesce, from_theme(ThemeRead::PointSize)),
                ),
            ),
        )
        .build()
        .unwrap();
    let chart = recovered.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks().len(), 4);
    assert_eq!(p.to_json().unwrap(), wire);
}

#[test]
fn ggplot_primary_discrete_size_trains_ordinal_outputs() {
    use chart_core::prelude::*;
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [1., 2., 3.])
        .column("size", ["c", "a", "b"])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").size("size"))
        .layer(points().name("points"))
        .build()
        .unwrap();
    for p in [
        p.clone(),
        p.edit()
            .layer("points", points().color(rgb(1, 2, 3)))
            .build()
            .unwrap(),
    ] {
        let chart = p.chart().unwrap().prepare().unwrap();
        let marks = chart.layers()[0].marks();
        assert_eq!(marks.len(), 3);
        for (m, expected) in marks.iter().zip([6., 2., 4.47213595499958]) {
            assert!((m.style.radius - expected).abs() < 2e-12);
        }
    }
}
