//! FIX-GG04 independent scales 1.4.0 transform arithmetic and population errors.
use chart_core::scales::GgplotTransform;
use serde_json::Value;
fn number(v: &Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| match v["number"].as_str().unwrap() {
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            "NA" | "NaN" => f64::NAN,
            s => panic!("Unknown number {s}"),
        })
}
fn transform(c: &Value) -> GgplotTransform {
    use GgplotTransform as T;
    if let Some(parts) = c.get("transforms").and_then(Value::as_array) {
        return T::Compose {
            transforms: parts
                .iter()
                .map(|part| {
                    transform(&serde_json::json!({"constructor":part["name"],"args":part["args"]}))
                })
                .collect(),
        };
    }
    let arg = |name: &str, default: f64| c["args"][name].as_f64().unwrap_or(default);
    match c["constructor"].as_str().unwrap() {
        "asinh" => T::Asinh,
        "asn" => T::Asn,
        "atanh" => T::Atanh,
        "boxcox" => T::BoxCox {
            p: arg("p", 1.),
            offset: arg("offset", 0.),
        },
        "modulus" => T::Modulus {
            p: arg("p", 1.),
            offset: arg("offset", 1.),
        },
        "exp" => T::Exp {
            base: arg("base", std::f64::consts::E),
        },
        "identity" => T::Identity,
        "log" => T::Log {
            base: arg("base", std::f64::consts::E),
        },
        "log10" => T::Log { base: 10. },
        "log2" => T::Log { base: 2. },
        "log1p" => T::Log1p,
        "logit" => T::Logistic {
            location: 0.,
            scale: 1.,
        },
        "probit" => T::Normal { mean: 0., sd: 1. },
        "pseudo_log" => T::PseudoLog {
            sigma: arg("sigma", 1.),
            base: arg("base", std::f64::consts::E),
        },
        "reciprocal" => T::Reciprocal,
        "reverse" => T::Reverse,
        "sqrt" => T::Sqrt,
        "yj" => T::YeoJohnson { p: arg("p", 1.) },
        "probability" => {
            if c["args"]["distribution"] == "norm" {
                T::Normal {
                    mean: arg("mean", 0.),
                    sd: arg("sd", 1.),
                }
            } else {
                T::Logistic {
                    location: arg("location", 0.),
                    scale: arg("scale", 1.),
                }
            }
        }
        s => panic!("Unknown transform {s}"),
    }
}
fn close(actual: f64, expected: f64, context: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{context}: {actual} expected NaN");
    } else if expected.is_infinite() {
        assert_eq!(actual, expected, "{context}");
    } else {
        assert!(
            (actual - expected).abs() <= 4e-14 * expected.abs().max(1.),
            "{context}: actual={actual:.17e}, expected={expected:.17e}"
        );
    }
}
#[test]
fn reference_transform_scalar_contracts() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-transform-contracts.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 29);
    for (case, c) in cases.iter().enumerate() {
        let t = transform(c);
        t.validate().unwrap();
        let restored: GgplotTransform =
            serde_json::from_value(serde_json::to_value(&t).unwrap()).unwrap();
        assert_eq!(restored, t);
        for (a, e) in t.domain().iter().zip(c["domain"].as_array().unwrap()) {
            close(*a, number(e), "domain");
        }
        let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
        for op in ["forward", "inverse"] {
            if c[op].get("error").is_some() {
                assert!(
                    t.forward_population(&input).is_err(),
                    "{case}: expected population rejection"
                );
                continue;
            }
            for (i, (x, e)) in input
                .iter()
                .zip(c[op]["values"].as_array().unwrap())
                .enumerate()
            {
                close(
                    if op == "forward" {
                        t.forward(*x)
                    } else {
                        t.inverse(*x)
                    },
                    number(e),
                    &format!("{case} {t:?} {op} input{i}={x}"),
                );
            }
        }
    }
}

use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision, layout::*, prelude::*, scales::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn positional_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-transform-contracts.json"
    ))
    .unwrap();
    check_positional_transform_reference_plots(&fixture);
}
fn check_positional_transform_reference_plots(fixture: &Value) {
    for (index, c) in fixture["plots"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["route"] == "position")
    {
        let t = transform(&fixture["cases"][c["configuration"].as_u64().unwrap() as usize]);
        let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
        let data = Data::columns()
            .column("x", input)
            .column("y", vec![1.; c["inputs"].as_array().unwrap().len()])
            .build()
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                            transform: t.clone(),
                        }))
                        .range(100., 540.)
                        .tick_arguments(c["count"].as_f64().map(|count| GuideTickArguments {
                            count: Some(count),
                            ..Default::default()
                        }))
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()?;
            let wire = p.to_json()?;
            assert_eq!(
                serde_json::from_str::<Value>(&wire).unwrap()["version"],
                if matches!(t, GgplotTransform::Compose { .. }) {
                    54
                } else {
                    53
                }
            );
            let restored = chart_core::plot::Plot::from_json(&wire)?;
            assert_eq!(restored.to_json()?, wire);
            let prepared = restored.chart()?.prepare()?;
            if let Some(mapped) = c["result"]["mapped"].as_array() {
                let expected: Vec<_> = mapped.iter().map(number).filter(|v| !v.is_nan()).collect();
                let actual: Vec<_> = prepared.layers()[0]
                    .marks()
                    .iter()
                    .map(|m| match m.geometry {
                        chart_core::grammar::PreparedGeometry::Point(p) => p.x(),
                        chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => p[0].0,
                        _ => panic!("Expected point"),
                    })
                    .collect();
                assert_eq!(
                    actual.len(),
                    expected.len(),
                    "plot{index} {t:?} retained population"
                );
                for (a, e) in actual.iter().zip(expected) {
                    close(*a, e, &format!("plot{index} {t:?} mapped"));
                }
            }
            layout(
                prepared,
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.)?,
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                ),
                &Metrics,
            )
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "plot{index} {t:?}: {c}, {:?}",
            result.as_ref().err()
        );
        if let Ok(frame) = result {
            let mut ticks: Vec<_> = frame.guides()[&GuideId::new(0)].ticks.iter().collect();
            ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut expected: Vec<_> = c["result"]["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .filter(|(v, _)| v.is_number())
                .collect();
            expected.sort_by(|a, b| number(a.0).total_cmp(&number(b.0)));
            assert_eq!(ticks.len(), expected.len(), "plot{index} {t:?}: {c}");
            for (tick, (pos, label)) in ticks.iter().zip(expected) {
                close(
                    (tick.position - 100.) / 440.,
                    number(pos),
                    &format!("plot{index} {t:?} position"),
                );
                assert_eq!(
                    tick.label,
                    label.as_str().unwrap(),
                    "plot{index} {t:?} label"
                );
            }
        }
    }
}

#[test]
fn paint_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-transform-contracts.json"
    ))
    .unwrap();
    check_paint_transform_reference_plots(&fixture);
}
fn check_paint_transform_reference_plots(fixture: &Value) {
    for (index, c) in fixture["plots"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["route"] == "paint" || c["route"] == "binned_paint")
    {
        let t = transform(&fixture["cases"][c["configuration"].as_u64().unwrap() as usize]);
        let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
        let data = Data::columns()
            .column("x", (0..input.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("value", input)
            .build()
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true)? else {
                unreachable!()
            };
            scale.palette_theme_aesthetics.clear();
            scale.training = ScaleTraining::Eligible;
            if c["route"] == "binned_paint" {
                scale = scale.with_ggplot(GgplotScalePolicy::Binned(Box::default()))?;
            }
            let ScaleFunctionSpec::Interpolated(spec) = &mut scale.function else {
                unreachable!()
            };
            spec.normalization = NormalizationSpec::Ggplot {
                family: NumericFamily::Ggplot {
                    transform: t.clone(),
                },
                domain: [0., 1.].map(chart_core::interpolate::Number),
                reverse: false,
                rescaler: GgplotRescaler::Range,
                timestamp: None,
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("value").color_scale("paint"))
                .scale(color_mapped(
                    "paint",
                    scale.with_guide(GgplotScaleGuide::Hidden)?,
                ))
                .layer(points())
                .build()?;
            let wire = p.to_json()?;
            assert_eq!(
                serde_json::from_str::<Value>(&wire).unwrap()["version"],
                if matches!(t, GgplotTransform::Compose { .. }) {
                    54
                } else {
                    53
                }
            );
            let restored = chart_core::plot::Plot::from_json(&wire)?;
            assert_eq!(restored.to_json()?, wire);
            restored.chart()?.prepare()
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "paint{index} {t:?}: {c}, {:?}",
            result.as_ref().err()
        );
        if let Ok(prepared) = result {
            let marks = prepared.layers()[0].marks();
            let wanted = c["result"]["mapped"].as_array().unwrap();
            assert_eq!(marks.len(), wanted.len(), "paint{index}");
            for (mark, expected) in marks.iter().zip(wanted) {
                assert_eq!(
                    mark.style.color,
                    chart_core::color::parse_r(expected.as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "paint{index} {t:?}: {c}"
                );
            }
        }
    }
}

struct FixedTwo;
impl chart_core::grammar::CustomGuideFormatter for FixedTwo {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::ExtensionDescriptor::batch("test.fixed_two", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn format_labels(
        &self,
        input: chart_core::grammar::GuideLabelsInput<'_>,
    ) -> ChartResult<Vec<Option<String>>> {
        Ok(input
            .values
            .iter()
            .map(|v| {
                let chart_core::composition::ScaleValue::Number(v) = v else {
                    panic!("numeric callback")
                };
                v.is_finite().then(|| {
                    let rounded = (v * 100.).round_ties_even() / 100.;
                    format!("{:.2}", if rounded == 0. { 0. } else { rounded })
                })
            })
            .collect())
    }
}
#[test]
fn transform_guide_count_and_registered_labels_reference_plots() {
    let transforms: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-transform-contracts.json"
    ))
    .unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transform-guide-controls.json"
    ))
    .unwrap();
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let t = transform(&transforms["cases"][c["configuration"].as_u64().unwrap() as usize]);
        let result = (|| -> ChartResult<_> {
            let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
            let data = Data::columns().column("x", input).build()?;
            let mut axis = x_axis()
                .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                    transform: t.clone(),
                }))
                .range(100., 540.)
                .tick_arguments(Some(GuideTickArguments {
                    count: c["count"].as_f64(),
                    ..Default::default()
                }))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                }));

            let mut registry = chart_core::grammar::ExtensionRegistry::default();
            registry.register_guide_formatter(std::sync::Arc::new(FixedTwo))?;
            if c["format"] == "two_digits" {
                axis = axis.tick_format(Some(GuideFormatter::Registered {
                    operation: chart_core::grammar::OperationRef::new(
                        "test.fixed_two",
                        Revision::new(1),
                    ),
                    parameters: Value::Null,
                }));
            }
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(axis)
                .extensions(std::sync::Arc::new(registry))
                .build()?;
            layout(
                p.chart()?.prepare()?,
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.)?,
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                ),
                &Metrics,
            )
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "case{index} {t:?}: {:?}",
            result.as_ref().err()
        );
        if let Ok(frame) = result {
            let mut ticks: Vec<_> = frame.guides()[&GuideId::new(0)].ticks.iter().collect();
            ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut expected: Vec<_> = c["result"]["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .filter(|(v, _)| v.is_number())
                .collect();
            expected.sort_by(|a, b| number(a.0).total_cmp(&number(b.0)));
            assert_eq!(ticks.len(), expected.len(), "case{index} {t:?}: {c}");
            for (tick, (position, label)) in ticks.iter().zip(expected) {
                close(
                    (tick.position - 100.) / 440.,
                    number(position),
                    &format!("case{index} {t:?}"),
                );
                assert_eq!(tick.label, label.as_str().unwrap(), "case{index} {t:?}");
            }
        }
    }
}

#[test]
fn builtin_transform_secondary_reference_plots() {
    let transforms: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-transform-contracts.json"
    ))
    .unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transform-secondary.json"
    ))
    .unwrap();
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let t = transform(&transforms["cases"][c["configuration"].as_u64().unwrap() as usize]);
        let result = (|| -> ChartResult<_> {
            let data = Data::columns()
                .column(
                    "x",
                    c["inputs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(number)
                        .collect::<Vec<_>>(),
                )
                .build()?;
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                            transform: t.clone(),
                        }))
                        .range(100., 540.)
                        .expansion(Some(GgplotExpansion {
                            mult: [0.; 2],
                            add: [0.; 2],
                        }))
                        .tick_arguments(Some(GuideTickArguments {
                            count: Some(3.),
                            ..Default::default()
                        })),
                )
                .axis(
                    x_axis()
                        .name("second")
                        .side(AxisSide::Top)
                        .secondary("x", 2., 1.)
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()?;
            let id = p.axis("second")?.id();
            let frame = layout(
                p.chart()?.prepare()?,
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.)?,
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                ),
                &Metrics,
            )?;
            Ok((frame, id))
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "case{index} {t:?}: {:?}",
            result.as_ref().err()
        );
        if let Ok((frame, id)) = result {
            let axis = &frame.axes()[&id];
            assert!(!axis.capabilities().numeric_inverse);
            let mut ticks: Vec<_> = axis.ticks.iter().collect();
            ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut expected: Vec<_> = c["result"]["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .collect();
            expected.sort_by(|a, b| number(a.0).total_cmp(&number(b.0)));
            assert_eq!(ticks.len(), expected.len(), "case{index} {t:?}: {c}");
            for (tick, (position, label)) in ticks.iter().zip(expected) {
                assert!(
                    ((tick.position - 100.) / 440. - number(position)).abs() <= 2e-12,
                    "case{index} {t:?}: {} != {}",
                    (tick.position - 100.) / 440.,
                    number(position)
                );
                assert_eq!(tick.label, label.as_str().unwrap(), "case{index} {t:?}");
            }
        }
    }
}

#[test]
fn builtin_standalone_envelopes_reject_downgrades() {
    standalone_envelopes(GgplotTransform::Asinh, 8);
}
fn standalone_envelopes(transform: GgplotTransform, version: u32) {
    let family = NumericFamily::Ggplot { transform };
    let mut interpolated = InterpolatedScaleSpec::sequential(family.clone());
    let mut specs = vec![
        StandaloneScaleSpec::Numeric(NumericScaleSpec::d3(family.clone())),
        StandaloneScaleSpec::Continuous(ContinuousScaleSpec::d3(family.clone())),
        StandaloneScaleSpec::Interpolated(interpolated.clone()),
        StandaloneScaleSpec::Interpolated(InterpolatedScaleSpec::diverging(family.clone())),
    ];
    interpolated.normalization = NormalizationSpec::Ggplot {
        family,
        domain: [0.0.into(), 1.0.into()],
        reverse: false,
        rescaler: GgplotRescaler::Range,
        timestamp: None,
    };
    specs.push(StandaloneScaleSpec::Interpolated(interpolated));
    for spec in specs {
        let scale = StandaloneScale::new(spec).unwrap();
        let wire = scale.to_json().unwrap();
        let mut json: Value = serde_json::from_str(&wire).unwrap();
        assert_eq!(json["version"], version);
        assert_eq!(
            StandaloneScale::from_json(&wire)
                .unwrap()
                .to_json()
                .unwrap(),
            wire
        );
        for downgrade in 1..version {
            json["version"] = downgrade.into();
            assert!(StandaloneScale::from_json(&json.to_string()).is_err());
        }
    }
    assert_eq!(
        StandaloneScaleSpec::Numeric(NumericScaleSpec::d3(NumericFamily::Linear)).wire_version(),
        1
    );
}

#[test]
fn builtin_numeric_envelope_rejects_legacy_version() {
    let scale = NumericScale::new(NumericScaleSpec::d3(NumericFamily::Ggplot {
        transform: GgplotTransform::Asinh,
    }))
    .unwrap();
    let wire = scale.to_json().unwrap();
    let mut json: Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(json["version"], 2);
    assert_eq!(
        NumericScale::from_json(&wire).unwrap().to_json().unwrap(),
        wire
    );
    json["version"] = 1.into();
    assert!(NumericScale::from_json(&json.to_string()).is_err());
    let legacy = NumericScale::new(NumericScaleSpec::d3(NumericFamily::Linear)).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&legacy.to_json().unwrap()).unwrap()["version"],
        1
    );
}

#[test]
fn composed_transform_scalar_contracts() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transform-compositions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 13);
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let t = transform(case);
        assert_eq!(
            t.validate().is_err(),
            case["result"].get("error").is_some(),
            "case{index}: {t:?}"
        );
        if t.validate().is_err() {
            continue;
        }
        for (actual, expected) in t
            .domain()
            .iter()
            .zip(case["result"]["domain"].as_array().unwrap())
        {
            close(*actual, number(expected), "composition domain");
        }
        let input: Vec<_> = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        if case["result"]["forward"].get("error").is_some() {
            assert!(t.forward_population(&input).is_err());
        } else {
            let actual = t.forward_population(&input).unwrap();
            for (actual, expected) in actual
                .iter()
                .zip(case["result"]["forward"]["values"].as_array().unwrap())
            {
                close(*actual, number(expected), &format!("case{index} forward"));
            }
        }
        for (input, expected) in input
            .iter()
            .zip(case["result"]["inverse"]["values"].as_array().unwrap())
        {
            close(
                t.inverse(*input),
                number(expected),
                &format!("case{index} inverse"),
            );
        }
    }
}
#[test]
fn composed_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/transform-compositions.json"
    ))
    .unwrap();
    assert_eq!(fixture["plots"].as_array().unwrap().len(), 195);
    check_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}

#[test]
fn composed_transform_envelopes_and_budgets() {
    let composed = GgplotTransform::Compose {
        transforms: vec![GgplotTransform::Asinh, GgplotTransform::Reverse],
    };
    standalone_envelopes(composed.clone(), 9);
    let numeric = NumericScale::new(NumericScaleSpec::d3(NumericFamily::Ggplot {
        transform: composed,
    }))
    .unwrap();
    let wire = numeric.to_json().unwrap();
    let mut json: Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(json["version"], 3);
    assert_eq!(
        NumericScale::from_json(&wire).unwrap().to_json().unwrap(),
        wire
    );
    for downgrade in [1, 2] {
        json["version"] = downgrade.into();
        assert!(NumericScale::from_json(&json.to_string()).is_err());
    }
    let empty = GgplotTransform::Compose { transforms: vec![] };
    assert!(empty.validate().is_err());
    let wide = GgplotTransform::Compose {
        transforms: vec![GgplotTransform::Identity; 4096],
    };
    assert_eq!(
        wide.validate().unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
    let mut deep = GgplotTransform::Identity;
    for _ in 0..34 {
        deep = GgplotTransform::Compose {
            transforms: vec![deep],
        };
    }
    assert_eq!(
        deep.validate().unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
    let nested = GgplotTransform::Compose {
        transforms: vec![
            GgplotTransform::Reverse,
            GgplotTransform::Compose {
                transforms: vec![GgplotTransform::Identity, GgplotTransform::Reverse],
            },
        ],
    };
    assert_eq!(
        nested.forward_population(&[-2., 0., 3.]).unwrap(),
        [-2., 0., 3.]
    );
}

#[test]
fn identity_transform_guides_share_the_mapping_transform() {
    use chart_core::{interpolate::Number, scales::*};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-transform-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 232);
    let mut label_mismatches = Vec::new();
    for c in cases {
        let context = c.to_string();
        let t =
            transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize]);
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .map(Number)
            .collect::<Vec<_>>();
        let mut spec = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                transform: Some(ScaleTransform::Ggplot { transform: t }),
                guide: c["guide"] == "legend",
                ..Default::default()
            },
        ));
        spec.training = ScaleTraining::Eligible;
        spec.guide = Some(Box::new(GgplotScaleGuide::Continuous(
            GgplotContinuousGuide {
                breaks: (c["break_mode"] == "explicit").then(|| inputs.clone()),
                ..Default::default()
            },
        )));
        let result = spec
            .trained(&inputs.iter().copied().map(Some).collect::<Vec<_>>())
            .and_then(MappedScale::for_numbers)
            .and_then(|scale| {
                let mapped = inputs
                    .iter()
                    .map(|v| scale.numeric(Some(v.0)))
                    .collect::<chart_core::ChartResult<Vec<_>>>()?;
                let entries = scale.continuous_guide_entries(4096, 16384)?.unwrap();
                Ok((mapped, entries))
            });
        if c["result"].get("error").is_some() {
            assert!(result.is_err(), "{context}: expected source rejection");
            continue;
        }
        let (mapped, entries) = result.unwrap_or_else(|e| panic!("{context}: {e:?}"));
        let expected = c["result"]["mapped"].as_array().unwrap();
        assert_eq!(mapped.len(), expected.len(), "{context}");
        for (actual, expected) in mapped.iter().zip(expected) {
            let chart_core::interpolate::Value::Number(actual) = actual else {
                panic!("{context}: {actual:?}")
            };
            close(actual.0, number(expected), &context);
        }
        let visible = entries.iter().filter(|v| v.visible).collect::<Vec<_>>();
        let keys = c["result"]["keys"].as_array().unwrap();
        if keys.is_empty() {
            assert!(visible.is_empty(), "{context}: {visible:?}");
            continue;
        }
        assert_eq!(keys.len(), 1, "{context}");
        let expected = keys[0]["values"].as_array().unwrap();
        assert_eq!(visible.len(), expected.len(), "{context}: {visible:?}");
        for (index, (actual, expected)) in visible.iter().zip(expected).enumerate() {
            close(actual.transformed.0, number(expected), &context);
            if actual.label.as_deref() != keys[0]["labels"][index].as_str() {
                label_mismatches.push(format!(
                    "config={} mode={} actual={:?} expected={:?}",
                    c["configuration"], c["break_mode"], actual.label, keys[0]["labels"][index]
                ));
            }
        }
    }
    assert!(label_mismatches.is_empty(), "{label_mismatches:#?}");
}
