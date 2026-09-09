//! CP-04/05: named output metadata, normalization, missing paint and retained preparation.
use chart_core::{
    grammar::Compiler,
    interpolate::{InterpolationSpec, Number, Value},
    prelude::*,
    scales::{chromatic::*, *},
    scene::Color,
    state::ChartState,
};
use std::sync::Arc;
fn ramp(domain: [f64; 3], reverse: bool) -> MappedScaleSpec {
    ScaleConstructor::Diverging
        .create(ScaleOptions {
            domain: Some(
                domain
                    .into_iter()
                    .map(|x| ScaleInput::Number(Number(x)))
                    .collect(),
            ),
            interpolator: Some(InterpolationSpec::Chromatic {
                spec: ChromaticSpec {
                    id: InterpolatorId::RdBu,
                    reverse,
                },
            }),
            ..Default::default()
        })
        .unwrap()
        .mapped(ScaleTraining::Authored)
        .unwrap()
}
fn plot_for(scale: MappedScaleSpec) -> Plot {
    plot(
        Data::columns()
            .column("x", [-10., -5., 0., 50., 100.])
            .column("y", [1., 2., 3., 2., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y").color("x").color_scale("ramp"))
    .layer(points())
    .scale(color_mapped("ramp", scale))
    .legend(legend().scale("ramp"))
    .build()
    .unwrap()
}
#[test]
fn asymmetric_normalization_reversal_missing_and_guide_identity() {
    let missing = Color {
        red: 11,
        green: 22,
        blue: 33,
        alpha: 44,
    };
    let reference = ChromaticRamp::new(ChromaticSpec {
        id: InterpolatorId::RdBu,
        reverse: false,
    })
    .unwrap();
    for descending in [false, true] {
        for reverse in [false, true] {
            let spec = ramp(
                if descending {
                    [100., 0., -10.]
                } else {
                    [-10., 0., 100.]
                },
                reverse,
            );
            let m = MappedScale::for_colors(spec.clone()).unwrap();
            for (x, t) in [(-10., 0.), (-5., 0.25), (0., 0.5), (50., 0.75), (100., 1.)] {
                let t = if descending { 1. - t } else { t };
                let t = if reverse { 1. - t } else { t };
                assert_eq!(
                    m.color(Some(x), None, missing).unwrap(),
                    reference.evaluate(t).unwrap()
                );
            }
            for v in [
                None,
                Some(f64::NAN),
                Some(f64::INFINITY),
                Some(f64::NEG_INFINITY),
            ] {
                assert_eq!(m.color(v, None, missing).unwrap(), missing);
            }
            let guide = m.legend(chart_core::ScaleId::new(1), missing).unwrap();
            assert_eq!(guide.mapping.as_ref(), Some(&spec));
            assert_eq!(guide.entries.len(), 3);
        }
    }
}
#[test]
fn palette_metadata_checked_and_v1_v6_ingestion_is_strict() {
    let s = ScaleConstructor::Quantile
        .create(ScaleOptions {
            range: Some(vec![Value::Text("red".into())]),
            ..Default::default()
        })
        .unwrap()
        .mapped(ScaleTraining::Eligible)
        .unwrap();
    let named = s
        .with_palette(SchemeSpec::new(SchemeId::Blues, Some(3), true))
        .unwrap();
    let trained = named
        .trained(&[Some(Number(0.)), Some(Number(2.)), Some(Number(4.))])
        .unwrap();
    assert_eq!(trained.catalog, named.catalog);
    let mut bad = named.clone();
    bad.catalog.as_mut().unwrap().reverse = false;
    assert!(MappedScale::new(bad).is_err());
    let mut bad = named;
    bad.catalog.as_mut().unwrap().version = 2;
    assert!(MappedScale::new(bad).is_err());
    let p = plot_for(ramp([-10., 0., 100.], false));
    let wire = p.to_json().unwrap();
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let mut value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(value["version"], 6);
    for version in [1, 5, 7] {
        value["version"] = version.into();
        assert!(Plot::from_json(&value.to_string()).is_err());
    }
    let old = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [1., 2.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .build()
    .unwrap()
    .to_json()
    .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&old).unwrap()["version"],
        1
    );
    assert_eq!(Plot::from_json(&old).unwrap().to_json().unwrap(), old);
}
#[test]
fn palette_reconfiguration_reuses_tables_and_matches_fresh_preparation() {
    let p = plot_for(ramp([-10., 0., 100.], false));
    let source = p.source();
    let state = ChartState::default();
    let mut compiler = Compiler::new();
    let first = compiler
        .prepare(p.definition(), &source, &state, p.compile_limits())
        .unwrap();
    let mut d = p.definition().clone();
    d.revision = chart_core::Revision::new(1);
    let ColorScale::Mapped { scale, .. } = &mut d.layers[0].color.as_mut().unwrap().scale else {
        panic!("mapped")
    };
    *scale = ramp([-10., 0., 100.], true);
    let next = compiler
        .prepare(&d, &source, &state, p.compile_limits())
        .unwrap();
    let fresh = Compiler::new()
        .prepare(&d, &source, &state, p.compile_limits())
        .unwrap();
    assert_eq!(next.metrics().evaluated_layers, 0);
    assert!(Arc::ptr_eq(
        first.layers()[0].table(),
        next.layers()[0].table()
    ));
    assert_eq!(next.domains(), first.domains());
    assert_eq!(next.layers()[0].marks(), fresh.layers()[0].marks());
    assert_eq!(
        next.layers()[0].color_legend(),
        fresh.layers()[0].color_legend()
    );
    assert_ne!(
        next.layers()[0].color_legend(),
        first.layers()[0].color_legend()
    );
    for (a, b) in first.layers()[0]
        .marks()
        .iter()
        .zip(next.layers()[0].marks())
    {
        assert_eq!(a.geometry, b.geometry);
        assert_eq!(a.targets, b.targets);
    }
}

#[test]
fn nonfinite_named_ordinal_keys_are_missing_and_do_not_shift_training() {
    let keys =
        [0., f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.].map(|x| ScaleKey::Number(Number(x)));
    let scale = ScaleConstructor::Ordinal
        .create(ScaleOptions {
            range: Some(vec![Value::Text("black".into())]),
            ..Default::default()
        })
        .unwrap()
        .mapped(ScaleTraining::Eligible)
        .unwrap()
        .with_palette(SchemeSpec::new(SchemeId::Category10, None, false))
        .unwrap()
        .trained_keys(&keys)
        .unwrap();
    let ScaleFunctionSpec::Ordinal(s) = &scale.function else {
        panic!("ordinal")
    };
    assert_eq!(s.domain, vec![keys[0].clone(), keys[4].clone()]);
    let missing = Color {
        red: 11,
        green: 22,
        blue: 33,
        alpha: 44,
    };
    let prepared = MappedScale::for_colors(scale).unwrap();
    for key in &keys[1..4] {
        assert_eq!(prepared.color(None, Some(key), missing).unwrap(), missing);
    }
    let colors = scheme(SchemeId::Category10, None, false).unwrap();
    assert_eq!(
        prepared.color(None, Some(&keys[4]), missing).unwrap(),
        colors[1]
    );
}

#[test]
fn every_named_scale_composition_matches_pinned_reference() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale-chromatic/composition.json"
    ))
    .unwrap();
    let missing = Color {
        red: 11,
        green: 22,
        blue: 33,
        alpha: 44,
    };
    let mut samples = 0;
    for case in corpus["cases"].as_array().unwrap() {
        let family: ScaleConstructor = serde_json::from_value(case["family"].clone()).unwrap();
        let ordinal = case["family"] == "ordinal";
        let mut options = case["options"].clone();
        options["domain"] = serde_json::Value::Array(
            options["domain"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    if ordinal {
                        serde_json::json!({"Key":{"Text":v}})
                    } else if case["family"] == "threshold" {
                        serde_json::json!({"Key":{"Number":v}})
                    } else {
                        serde_json::json!({"Number":v})
                    }
                })
                .collect(),
        );
        for field in ["range", "unknown"] {
            if let Some(v) = options.get_mut(field) {
                *v = if field == "range" {
                    serde_json::Value::Array(
                        v.as_array()
                            .unwrap()
                            .iter()
                            .map(|x| serde_json::json!({"kind":"Text","value":x}))
                            .collect(),
                    )
                } else {
                    serde_json::json!({"kind":"Text","value":v})
                };
            }
        }
        let selection = &case["selection"];
        if let Some(id) = selection.get("ramp") {
            options["interpolator"] = serde_json::json!({"operation":"Chromatic","spec":{"id":id,"reverse":selection["reverse"]}});
        }
        let mut spec = family
            .create(serde_json::from_value(options).unwrap())
            .unwrap()
            .mapped(ScaleTraining::Authored)
            .unwrap();
        if let Some(id) = selection.get("scheme") {
            spec = spec
                .with_palette(SchemeSpec::new(
                    id.as_str().unwrap().parse().unwrap(),
                    selection["size"].as_u64().map(|v| v as usize),
                    selection["reverse"].as_bool().unwrap(),
                ))
                .unwrap();
        }
        let prepared = MappedScale::for_colors(spec).unwrap();
        for sample in case["samples"].as_array().unwrap() {
            samples += 1;
            let value = &sample["input"];
            let key = value.as_str().map(|s| ScaleKey::Text(s.to_owned()));
            let numeric = if value.is_null() || ordinal {
                None
            } else {
                Some(serde_json::from_value::<Number>(value.clone()).unwrap().0)
            };
            let result = prepared.color(numeric, key.as_ref(), missing);
            if sample["expected"].is_null() {
                assert!(result.is_err(), "{} {value}", case["id"]);
                continue;
            }
            let c = result.unwrap();
            assert_eq!(
                serde_json::json!([c.red, c.green, c.blue, c.alpha]),
                sample["expected"],
                "{} {value}",
                case["id"]
            );
        }
    }
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 90);
    assert_eq!(samples, 1268);
}

#[test]
fn equal_endpoint_swatches_do_not_erase_reversed_ramp_identity() {
    let legend_for = |reverse| {
        let scale = ScaleConstructor::Sequential
            .create(ScaleOptions {
                interpolator: Some(InterpolationSpec::Chromatic {
                    spec: ChromaticSpec {
                        id: InterpolatorId::Rainbow,
                        reverse,
                    },
                }),
                ..Default::default()
            })
            .unwrap();
        MappedScale::for_colors(scale.mapped(ScaleTraining::Authored).unwrap())
            .unwrap()
            .legend(
                chart_core::ScaleId::new(99),
                Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
            )
            .unwrap()
    };
    let a = legend_for(false);
    let b = legend_for(true);
    assert_eq!(a.entries, b.entries);
    assert_ne!(
        a, b,
        "Guide collection compares the full mapping, not just identical endpoint swatches."
    );
}

#[test]
fn exceptional_normalized_parameters_preserve_valid_reference_colors() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale-chromatic/transformed.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 304);
    for case in cases {
        let family: ScaleConstructor = serde_json::from_value(case["family"].clone()).unwrap();
        let mut options = case["options"].clone();
        options["domain"] = serde_json::Value::Array(
            options["domain"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| serde_json::json!({"Number":v}))
                .collect(),
        );
        options["interpolator"] = serde_json::json!({"operation":"Chromatic","spec":{"id":case["ramp"],"reverse":case["reverse"]}});
        let scale = family
            .create(serde_json::from_value(options).unwrap())
            .unwrap();
        let mapped =
            MappedScale::for_colors(scale.mapped(ScaleTraining::Authored).unwrap()).unwrap();
        let x = case["input"].as_f64().unwrap();
        let standalone = scale.map(ScaleInput::Number(Number(x)));
        let paint = mapped.color(
            Some(x),
            None,
            Color {
                red: 11,
                green: 22,
                blue: 33,
                alpha: 44,
            },
        );
        if case["expected"].is_null() {
            assert!(standalone.is_err() && paint.is_err(), "{case}");
            continue;
        }
        let Value::Color(value) = standalone.unwrap() else {
            panic!("named color")
        };
        for c in [value.to_paint(), paint.unwrap()] {
            assert_eq!(
                serde_json::json!([c.red, c.green, c.blue, c.alpha]),
                case["expected"],
                "{case}"
            );
        }
    }
}
