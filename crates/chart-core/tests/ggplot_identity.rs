//! FIX-GG04: raw identity values, separate guide extent, and primary integration.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};

fn number(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().or_else(|| match v["number"].as_str().unwrap() {
        "NA" => None,
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => unreachable!(),
    })
}
fn same(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(b)) if a.is_nan() && b.is_nan() || a == b => {}
        (Some(a), Some(b)) => assert!((a - b).abs() <= 2e-13 * b.abs().max(1.), "{a} != {b}"),
        _ => panic!("{actual:?} != {expected:?}"),
    }
}
fn descriptor(identity: GgplotNumericIdentity) -> MappedScaleSpec {
    let mut spec = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(identity));
    spec.training = ScaleTraining::Eligible;
    spec
}
#[test]
fn numeric_identity_matches_twenty_pinned_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-scales.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["kind"].as_str(),
            Some("size" | "alpha" | "linewidth" | "shape")
        )
    }) {
        let transform = match case["transform"].as_str() {
            Some("sqrt") => Some(ScaleTransform::Sqrt),
            Some("log10") => Some(ScaleTransform::Log { base: 10. }),
            Some("reverse") => Some(ScaleTransform::Reverse),
            _ => None,
        };
        let limits = case["limits"].as_array().map(|v| {
            [
                Some(Number(v[0].as_f64().unwrap())),
                Some(Number(v[1].as_f64().unwrap())),
            ]
        });
        let spec = descriptor(GgplotNumericIdentity {
            transform,
            limits,
            guide: case["guide"] == "legend",
            trained: None,
            has_population: false,
        })
        .trained(
            &case["train"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| number(v).map(Number))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &spec.function else {
            unreachable!()
        };
        for (a, b) in identity
            .domain()
            .unwrap()
            .iter()
            .zip(case["result"]["domain"].as_array().unwrap())
        {
            same(Some(a.0), number(b));
        }
        let wire = serde_json::to_string(&spec).unwrap();
        assert_eq!(
            serde_json::from_str::<MappedScaleSpec>(&wire).unwrap(),
            spec
        );
        let scale = MappedScale::for_numbers(spec).unwrap();
        for (input, expected) in case["input"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["result"]["mapped"].as_array().unwrap())
        {
            let actual = match scale.numeric(number(input)).unwrap() {
                Value::Missing => None,
                Value::Number(n) => Some(n.0),
                _ => unreachable!(),
            };
            same(actual, number(expected));
        }
        count += 1;
    }
    assert_eq!(count, 20);
}

#[test]
fn identity_linewidth_reaches_primary_styles_and_keeps_limits_out_of_mapping() {
    use chart_core::{
        grammar::{Compiler, NumericAesthetic},
        prelude::*,
        state::ChartState,
    };
    let scale = descriptor(GgplotNumericIdentity {
        limits: Some([Some(Number(1.)), Some(Number(3.))]),
        ..Default::default()
    });
    let data = Data::columns().column("x", [0.5, 1., 4.]).build().unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(0.).x2("x").y2(1.))
        .layer(
            rule()
                .name("rules")
                .numeric_scale(NumericAesthetic::StrokeWidth, "x", scale),
        )
        .build()
        .unwrap();
    let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let prepared = Compiler::new()
        .prepare(
            restored.definition(),
            &restored.source(),
            &ChartState::default(),
            restored.compile_limits(),
        )
        .unwrap();
    assert_eq!(
        prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| m.style.stroke_width)
            .collect::<Vec<_>>(),
        [0.5, 1., 4.]
    );
    assert!(MappedScale::for_colors(descriptor(Default::default())).is_err());
    let invalid = descriptor(GgplotNumericIdentity {
        transform: Some(ScaleTransform::Log { base: 1. }),
        ..Default::default()
    });
    assert!(MappedScale::for_numbers(invalid).is_err());
}

fn key(v: &serde_json::Value) -> ScaleKey {
    if let Some(s) = v.as_str() {
        ScaleKey::Text(s.into())
    } else {
        number(v).map_or(ScaleKey::Null, |n| ScaleKey::Number(Number(n)))
    }
}
fn discrete(identity: GgplotDiscreteIdentity) -> MappedScaleSpec {
    let mut spec = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotDiscreteIdentity(identity));
    spec.training = ScaleTraining::Eligible;
    spec
}
#[test]
fn discrete_identity_matches_six_reference_records_and_unseen_colours() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-scales.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["kind"].as_str(), Some("colour" | "fill" | "linetype")))
    {
        let spec = discrete(GgplotDiscreteIdentity {
            guide: case["guide"] == "legend",
            ..Default::default()
        })
        .trained_keys(
            &case["train"]
                .as_array()
                .unwrap()
                .iter()
                .map(key)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let ScaleFunctionSpec::GgplotDiscreteIdentity(s) = &spec.function else {
            unreachable!()
        };
        assert_eq!(
            s.domain().unwrap(),
            case["result"]["domain"]
                .as_array()
                .unwrap()
                .iter()
                .map(key)
                .collect::<Vec<_>>()
        );
        let wire = serde_json::to_string(&spec).unwrap();
        assert_eq!(
            serde_json::from_str::<MappedScaleSpec>(&wire).unwrap(),
            spec
        );
        let scale = MappedScale::new(spec.clone()).unwrap();
        for (input, expected) in case["input"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["result"]["mapped"].as_array().unwrap())
        {
            assert_eq!(
                scale.category(Some(&key(input))).unwrap(),
                GgplotDiscreteIdentity::map(Some(&key(expected))).unwrap()
            );
        }
        if case["kind"] != "linetype" {
            let colours = MappedScale::for_colors(spec).unwrap();
            let blue = colours
                .color(
                    None,
                    Some(&ScaleKey::Text("blue".into())),
                    chart_core::scene::Color {
                        red: 99,
                        green: 99,
                        blue: 99,
                        alpha: 255,
                    },
                )
                .unwrap();
            assert_eq!(
                [blue.red, blue.green, blue.blue, blue.alpha],
                [0, 0, 255, 255]
            );
        }
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn identity_colours_reach_primary_marks_without_default_guides() {
    use chart_core::prelude::*;
    let spec = discrete(GgplotDiscreteIdentity {
        limits: Some(vec![ScaleKey::Text("red4".into())]),
        ..Default::default()
    });
    let p = plot(
        Data::columns()
            .column("x", [1., 2., 3.])
            .column("paint", ["red4", "blue", "#ABC0"])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y(1.).color("paint").color_scale("identity"))
    .scale(color_mapped("identity", spec))
    .layer(points())
    .build()
    .unwrap();
    let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let chart = p.chart().unwrap().prepare().unwrap();
    let layer = &chart.layers()[0];
    assert!(layer.color_legend().is_none());
    assert_eq!(layer.marks().len(), 3);
    for (mark, expected) in
        layer
            .marks()
            .iter()
            .zip([[139, 0, 0, 255], [0, 0, 255, 255], [170, 187, 204, 0]])
    {
        let c = mark.style.color;
        assert_eq!([c.red, c.green, c.blue, c.alpha], expected);
    }
}

#[test]
fn identity_limits_factors_and_missing_controls_match_ninety_six_reference_cases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-controls.json"
    ))
    .unwrap();
    assert_eq!(fixture["numeric"].as_array().unwrap().len(), 48);
    assert_eq!(fixture["discrete"].as_array().unwrap().len(), 48);
    for case in fixture["numeric"].as_array().unwrap() {
        let transform = match case["transform"].as_str().unwrap() {
            "sqrt" => Some(ScaleTransform::Sqrt),
            "log10" => Some(ScaleTransform::Log { base: 10. }),
            "reverse" => Some(ScaleTransform::Reverse),
            _ => None,
        };
        let limits = case["limits"]
            .as_array()
            .map(|v| [number(&v[0]).map(Number), number(&v[1]).map(Number)]);
        let spec = descriptor(GgplotNumericIdentity {
            transform,
            limits,
            guide: case["guide"] == "legend",
            trained: None,
            has_population: false,
        })
        .trained(
            &case["train"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| number(v).map(Number))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let ScaleFunctionSpec::GgplotNumericIdentity(s) = &spec.function else {
            unreachable!()
        };
        for (actual, expected) in s
            .domain()
            .unwrap()
            .iter()
            .zip(case["domain"].as_array().unwrap())
        {
            same(Some(actual.0), number(expected));
        }
        let scale = MappedScale::for_numbers(spec).unwrap();
        for (input, expected) in case["train"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["mapped"].as_array().unwrap())
        {
            let actual = match scale.numeric(number(input)).unwrap() {
                Value::Missing => None,
                Value::Number(v) => Some(v.0),
                _ => unreachable!(),
            };
            same(actual, number(expected));
        }
    }
    for case in fixture["discrete"].as_array().unwrap() {
        let spec = discrete(GgplotDiscreteIdentity {
            guide: case["guide"] == "legend",
            limits: case["limits"]
                .as_array()
                .map(|v| v.iter().map(key).collect()),
            levels: Some(case["levels"].as_array().unwrap().iter().map(key).collect()),
            drop: case["drop"].as_bool().unwrap(),
            na_translate: case["na_translate"].as_bool().unwrap(),
            observed: vec![],
        })
        .trained_keys(
            &case["train"]
                .as_array()
                .unwrap()
                .iter()
                .map(key)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let ScaleFunctionSpec::GgplotDiscreteIdentity(s) = &spec.function else {
            unreachable!()
        };
        assert_eq!(
            s.domain().unwrap(),
            case["domain"]
                .as_array()
                .unwrap()
                .iter()
                .map(key)
                .collect::<Vec<_>>(),
            "{case}"
        );
        let scale = MappedScale::new(spec).unwrap();
        for (input, expected) in case["input"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["mapped"].as_array().unwrap())
        {
            assert_eq!(
                scale.category(Some(&key(input))).unwrap(),
                GgplotDiscreteIdentity::map(Some(&key(expected))).unwrap()
            );
        }
    }
}

#[test]
fn identity_alpha_lowering_matches_reference_point_grobs() {
    use chart_core::{
        grammar::{AfterScaleAesthetic, NumericAesthetic},
        prelude::*,
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-alpha.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let input = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", (0..input.len()).map(|v| v as f64).collect::<Vec<_>>())
            .column("alpha", input)
            .build()
            .unwrap();
        let mut layer = points()
            .color(chart_core::color::Paint::from_css("#12345678").unwrap())
            .numeric_scale(
                NumericAesthetic::Alpha,
                "alpha",
                descriptor(Default::default()),
            );
        if case["after_scale_double"] == true {
            layer = layer
                .after_scale(scale_aes().alpha(after_scale_expr(AfterScaleAesthetic::Alpha) * 2.));
        }
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(layer)
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), 12);
        for (i, (mark, expected)) in marks
            .iter()
            .zip(case["rgba"].as_array().unwrap())
            .enumerate()
        {
            let c = mark.style.color;
            assert_eq!(
                serde_json::json!([c.red, c.green, c.blue, c.alpha]),
                *expected,
                "input {i}, doubled {}",
                case["after_scale_double"]
            );
        }
    }
    let numeric_guide = discrete(GgplotDiscreteIdentity {
        guide: true,
        ..Default::default()
    })
    .trained_keys(&[ScaleKey::Number(Number(1.))]);
    assert!(numeric_guide.is_err());
}

#[test]
fn reference_after_scale_arithmetic_matches_168_point_paints() {
    use chart_core::{
        grammar::{
            AfterScaleAesthetic, Expression, ExpressionBinary as B, ExpressionReduce as R,
            ExpressionUnary as U, NumericAesthetic,
        },
        prelude::*,
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-alpha-expressions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 14);
    for case in fixture["cases"].as_array().unwrap() {
        let input = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", (0..input.len()).map(|v| v as f64).collect::<Vec<_>>())
            .column("alpha", input)
            .build()
            .unwrap();
        let alpha = after_scale_expr(AfterScaleAesthetic::Alpha);
        let expression = match case["name"].as_str().unwrap() {
            "negate" => alpha.unary(U::Negate),
            "abs" => alpha.unary(U::Abs),
            "sqrt" => alpha.unary(U::Sqrt),
            "log" => alpha.unary(U::Log),
            "exp" => alpha.unary(U::Exp),
            "reciprocal" => Expression::constant(1.) / alpha,
            "subtract" => alpha.clone() - alpha,
            "power" => alpha.binary(B::Power, Expression::constant(0.)),
            "sum" => alpha.reduce(R::Sum, true),
            "mean" => alpha.reduce(R::Mean, true),
            "min" => alpha.reduce(R::Min, true),
            "max" => alpha.reduce(R::Max, true),
            "abs_sum" => alpha.unary(U::Abs).reduce(R::Sum, true),
            "missing_sum" => alpha.reduce(R::Sum, false),
            _ => unreachable!(),
        };
        let layer = points()
            .color(chart_core::color::Paint::from_css("#12345678").unwrap())
            .numeric_scale(
                NumericAesthetic::Alpha,
                "alpha",
                descriptor(Default::default()),
            )
            .after_scale(scale_aes().alpha(expression));
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(layer)
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), 12);
        for (i, (mark, expected)) in marks
            .iter()
            .zip(case["rgba"].as_array().unwrap())
            .enumerate()
        {
            let c = mark.style.color;
            assert_eq!(
                serde_json::json!([c.red, c.green, c.blue, c.alpha]),
                *expected,
                "{} input {i}",
                case["name"]
            );
        }
    }
}
