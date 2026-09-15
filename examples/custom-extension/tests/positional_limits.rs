//! FIX-GG04 primary numeric positional limit functions, independently captured in R.
use chart_core::{interpolate::Number, prelude::*};
use chart_extension_example::numeric_limits::operation;
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn equal(a: f64, b: f64) -> bool {
    a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.)
}
#[test]
fn primary_continuous_function_population_and_final_limits_match_reference() {
    primary_functions("continuous", false, 81, 45);
}
#[test]
fn primary_binned_function_population_and_final_limits_match_reference() {
    primary_functions("binned", false, 74, 52);
}
#[test]
fn primary_continuous_callback_arguments_match_reference() {
    primary_functions("continuous", true, 219, 159);
}
#[test]
fn primary_binned_callback_arguments_match_reference() {
    primary_functions("binned", true, 177, 201);
}
fn primary_functions(kind: &str, arguments: bool, successes: usize, errors: usize) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-limit-functions.json"
    ))
    .unwrap();
    let fixture = if arguments {
        serde_json::from_str(include_str!(
            "../../../fixtures/parity/ggplot2/positional-limit-function-arguments.json"
        ))
        .unwrap()
    } else {
        fixture
    };
    let registry = chart_extension_example::registry().unwrap();
    let mut checked = 0;
    let mut rejected = 0;
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == kind)
    {
        let inputs: Vec<Option<f64>> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (!v.is_null()).then(|| number(v)))
            .collect();
        let scale = match c["transform"].as_str().unwrap() {
            "sqrt" => scale_sqrt(),
            "log10" => scale_log(10.),
            "reverse" => scale_reverse(),
            _ => scale_linear(),
        };
        let scale = if kind == "binned" {
            scale_binned(chart_core::scales::GgplotBinnedPosition {
                transform: match c["transform"].as_str().unwrap() {
                    "sqrt" => Some(chart_core::scales::ScaleTransform::Sqrt),
                    "reverse" => Some(chart_core::scales::ScaleTransform::Reverse),
                    "log10" => Some(chart_core::scales::ScaleTransform::Log { base: 10. }),
                    _ => None,
                },
                ..Default::default()
            })
        } else {
            scale
        };
        let data = Data::columns().column("x", inputs).build().unwrap();
        let p = plot(data)
            .extensions(registry.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(points())
            .x_axis(
                x_axis()
                    .guide_geometry(Some(chart_core::layout::GuideGeometry {
                        labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                        ..Default::default()
                    }))
                    .scale(scale)
                    .oob(match c["oob"].as_str() {
                        Some("squish") => chart_core::grammar::ScaleOob::Squish,
                        Some("keep") => chart_core::grammar::ScaleOob::Keep,
                        Some("censor") => chart_core::grammar::ScaleOob::Censor,
                        _ if kind == "binned" => chart_core::grammar::ScaleOob::Squish,
                        _ => chart_core::grammar::ScaleOob::Censor,
                    })
                    .limits_function(operation(c["control"].as_str().unwrap())),
            )
            .build();
        if c["result"].get("error").is_some() || c["result"]["positions"].get("error").is_some() {
            let result = p.and_then(|p| p.chart()?.prepare()).and_then(frame);
            assert!(result.is_err(), "Reference rejection was accepted: {c}");
            rejected += 1;
            continue;
        }
        let p = p.unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let prepared = p
            .chart()
            .unwrap()
            .prepare()
            .unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let expected: Vec<_> = c["result"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| !v.is_null() && !number(v).is_nan())
            .map(number)
            .collect();
        let actual: Vec<_> = prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| match &m.geometry {
                chart_core::grammar::PreparedGeometry::Point(center) => center.x(),
                chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => p[0].0,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(actual.len(), expected.len(), "{c}");
        assert!(
            actual.iter().zip(expected).all(|(a, b)| equal(*a, b)),
            "{actual:?}: {c}"
        );
        let limits = &prepared.positional_limits()[&chart_core::ScaleId::new(0)];
        let expected = c["result"]["limits"].as_array().unwrap();
        assert_eq!(limits.len(), expected.len(), "{c}");
        assert!(
            limits
                .iter()
                .zip(expected)
                .all(|(Number(a), b)| equal(*a, number(b))),
            "{limits:?}: {c}"
        );
        let wire = p.to_json().unwrap();
        assert_eq!(p.definition().wire_version(), 29);
        assert_eq!(
            Plot::from_json_with_extensions(&wire, registry.clone())
                .unwrap()
                .to_json()
                .unwrap(),
            wire
        );
        let frame = frame(prepared).unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let axis = &frame.axes()[&chart_core::ScaleId::new(0)];
        let range: Vec<f64> = match &axis.scale {
            chart_core::layout::ResolvedScale::Linear(s) => {
                vec![s.viewport().start(), s.viewport().end()]
            }
            chart_core::layout::ResolvedScale::Nonlinear(s) => vec![
                s.transformed_viewport().start(),
                s.transformed_viewport().end(),
            ],
            chart_core::layout::ResolvedScale::Unbounded(s) => {
                s.viewport().iter().map(|v| v.0).collect()
            }
            _ => panic!("unexpected family"),
        };
        let expected = c["result"]["range"].as_array().unwrap();
        if expected.len() == 2 {
            assert!(
                range
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| equal(*a, number(b))),
                "range {range:?}: {c}"
            );
        }
        if axis.ticks.iter().any(|t| matches!(t.value, chart_core::composition::ScaleValue::Number(v) if v.is_infinite())) {
            assert_eq!(frame.scene().wire_version(), 16);
            let json = serde_json::to_value(&axis.ticks).unwrap();
            assert!(json.as_array().unwrap().iter().all(|t| t["value"]["Number"].get("number").is_some()));
        }
        let plot = frame.plot().unwrap();
        let expected_ticks: Vec<_> = c["result"]["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["result"]["labels"].as_array().unwrap())
            .zip(c["result"]["positions"].as_array().unwrap())
            .filter(|((v, _), position)| {
                !v.is_null() && !position.is_null() && number(position).is_finite()
            })
            .map(|((v, l), position)| {
                (
                    number(v),
                    l.as_str().unwrap_or_default().to_owned(),
                    number(position),
                )
            })
            .collect();
        let actual_ticks: Vec<_> = axis
            .ticks
            .iter()
            .filter_map(|t| {
                if let chart_core::composition::ScaleValue::Number(v) = t.value {
                    Some((
                        match c["transform"].as_str().unwrap() {
                            "sqrt" => v.sqrt(),
                            "reverse" => -v,
                            "log10" => v.log10(),
                            _ => v,
                        },
                        t.label.clone(),
                        (t.position - plot.origin().x()) / plot.width(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            actual_ticks.len(),
            expected_ticks.len(),
            "ticks {actual_ticks:?}: {c}"
        );
        for (value, label, position) in &expected_ticks {
            assert!(
                actual_ticks
                    .iter()
                    .any(|(v, l, p)| equal(*v, *value) && l == label && equal(*p, *position)),
                "ticks {actual_ticks:?}: {c}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, successes);
    assert_eq!(rejected, errors);
}

struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(
    prepared: std::sync::Arc<chart_core::grammar::PreparedChart>,
) -> chart_core::ChartResult<chart_core::layout::LaidOutChart> {
    use chart_core::{services::*, *};
    let mut request = layout::LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.axes = prepared.definition().axes.clone();
    layout::layout(prepared, &request, &Metrics)
}

#[test]
fn statistics_and_shared_layers_retrain_the_same_positional_scale() {
    use chart_core::grammar::{PreparedGeometry, StatField};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-limit-functions.json"
    ))
    .unwrap();
    let registry = chart_extension_example::registry().unwrap();
    let cases = fixture["stage_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 48);
    for c in cases {
        let scale = match c["transform"].as_str().unwrap() {
            "sqrt" => scale_sqrt(),
            "reverse" => scale_reverse(),
            _ => scale_linear(),
        };
        let scale = if c["kind"] == "binned" {
            scale_binned(chart_core::scales::GgplotBinnedPosition {
                transform: match c["transform"].as_str().unwrap() {
                    "sqrt" => Some(chart_core::scales::ScaleTransform::Sqrt),
                    "reverse" => Some(chart_core::scales::ScaleTransform::Reverse),
                    _ => None,
                },
                ..Default::default()
            })
        } else {
            scale
        };
        let function = operation(c["control"].as_str().unwrap());
        let summary_case = c["context"] == "summary";
        let builder = if summary_case {
            plot(
                Data::columns()
                    .column("x", [1., 1., 1.])
                    .column("y", [1., 3., 9.])
                    .build()
                    .unwrap(),
            )
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .stat(summary().x("y"))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
            .y_axis(y_axis().scale(scale).limits_function(function))
        } else {
            plot(Data::columns().column("x", [1., 3.]).build().unwrap())
                .aes(aes().x("x").y(1.))
                .layer(points())
                .layer(
                    points().data(
                        Data::columns()
                            .name("second")
                            .column("x", [9.])
                            .build()
                            .unwrap(),
                    ),
                )
                .x_axis(x_axis().scale(scale).limits_function(function))
        };
        let prepared = builder
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone())
            .build()
            .and_then(|p| p.chart()?.prepare());
        if c["result"].get("error").is_some() {
            assert!(prepared.and_then(frame).is_err(), "{c}");
            continue;
        }
        let prepared = prepared.unwrap_or_else(|e| panic!("{e:?}: {c}"));
        for (layer, expected) in prepared
            .layers()
            .iter()
            .zip(c["result"]["values"].as_array().unwrap())
        {
            let actual: Vec<_> = layer
                .marks()
                .iter()
                .map(|m| match m.geometry {
                    PreparedGeometry::Point(p) => {
                        if summary_case {
                            p.y()
                        } else {
                            p.x()
                        }
                    }
                    _ => panic!("point"),
                })
                .collect();
            let expected: Vec<_> = expected
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null() && !number(v).is_nan())
                .map(number)
                .collect();
            assert_eq!(actual.len(), expected.len(), "{c}");
            assert!(
                actual.iter().zip(expected).all(|(a, b)| equal(*a, b)),
                "{actual:?}: {c}"
            );
        }
        let limits = &prepared.positional_limits()
            [&chart_core::ScaleId::new(if summary_case { 1 } else { 0 })];
        let expected = c["result"]["limits"].as_array().unwrap();
        assert_eq!(limits.len(), expected.len(), "{c}");
        assert!(
            limits
                .iter()
                .zip(expected)
                .all(|(a, b)| equal(a.0, number(b))),
            "{limits:?}: {c}"
        );
    }
}
