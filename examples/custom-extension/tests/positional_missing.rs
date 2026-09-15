//! FIX-GG04 positional missing-value reference builds.
use chart_core::{interpolate::Number, prelude::*};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn equal(a: f64, b: f64) -> bool {
    a == b || a.is_nan() && b.is_nan() || (a - b).abs() < 3e-12 * b.abs().max(1.)
}
#[test]
fn positional_missing_values_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-missing.json"
    ))
    .unwrap();
    check_fixture(&fixture, 1152);
}
#[test]
fn duration_missing_values_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-duration-missing.json"
    ))
    .unwrap();
    check_fixture(&fixture, 192);
}
fn check_fixture(fixture: &serde_json::Value, count: usize) {
    let registry = chart_extension_example::registry().unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), count);
    for c in cases {
        let input: Vec<Option<f64>> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (!v.is_null()).then(|| number(v)))
            .collect();
        let is_summary = c["context"] == "summary";
        let mut scale = match c["transform"].as_str().unwrap() {
            "sqrt" => scale_sqrt(),
            "reverse" => scale_reverse(),
            "log10" => scale_log(10.),
            "duration" => scale_duration(),
            _ => scale_linear(),
        };
        if c["limit_control"] == "fixed" {
            scale = scale.domain(1., 4.);
        }
        let mut axis = if is_summary { y_axis() } else { x_axis() }
            .scale(scale)
            .range(100., 540.)
            .guide_geometry(Some(chart_core::layout::GuideGeometry {
                labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                ..Default::default()
            }))
            .missing_value(match c["replacement"].as_str().unwrap() {
                "zero" => Some(Number(0.)),
                "five" => Some(Number(5.)),
                "negative" => Some(Number(-1.)),
                _ => None,
            })
            .oob(match c["oob"].as_str().unwrap() {
                "squish" => chart_core::grammar::ScaleOob::Squish,
                "keep" => chart_core::grammar::ScaleOob::Keep,
                _ => chart_core::grammar::ScaleOob::Censor,
            });
        if c["limit_control"] == "function" {
            axis = axis.limits_function(chart_extension_example::numeric_limits::operation(
                "identity",
            ));
        }
        let builder = if is_summary {
            plot(
                Data::columns()
                    .column("x", vec![1.; input.len()])
                    .column("y", input)
                    .build()
                    .unwrap(),
            )
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .stat(summary().x("y"))
                    .after_stat(stat_aes().x(1.).y(chart_core::grammar::StatField::Mean)),
            )
            .y_axis(axis)
        } else {
            plot(Data::columns().column("x", input).build().unwrap())
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(axis)
        };
        let p = builder
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone())
            .build();
        let prepared = p
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|p| p.chart()?.prepare());
        if c["result"].get("error").is_some() || c["result"]["positions"].get("error").is_some() {
            assert!(
                prepared.and_then(frame).is_err(),
                "Reference rejection accepted: {c}"
            );
            continue;
        }
        let prepared = prepared.unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let actual: Vec<_> = prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| match &m.geometry {
                chart_core::grammar::PreparedGeometry::Point(p) => {
                    if is_summary {
                        p.y()
                    } else {
                        p.x()
                    }
                }
                chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => {
                    p[usize::from(is_summary)].0
                }
                _ => panic!(),
            })
            .collect();
        let expected: Vec<_> = c["result"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .filter(|v| !v.is_nan())
            .collect();
        assert_eq!(actual.len(), expected.len(), "{actual:?}: {c}");
        assert!(
            actual.iter().zip(expected).all(|(a, b)| equal(*a, b)),
            "{actual:?}: {c}"
        );
        let laid = frame(prepared).unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let axis = &laid.axes()[&chart_core::ScaleId::new(if is_summary { 1 } else { 0 })];
        let range = match &axis.scale {
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
        assert!(
            range
                .iter()
                .zip(c["result"]["range"].as_array().unwrap())
                .all(|(a, b)| equal(*a, number(b))),
            "range {range:?}: {c}"
        );
        let expected_ticks: Vec<_> = c["result"]["positions"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["result"]["labels"].as_array().unwrap())
            .filter(|(p, _)| !p.is_null() && number(p).is_finite())
            .map(|(p, l)| (number(p), l.as_str().unwrap_or_default()))
            .collect();
        assert_eq!(
            axis.ticks.len(),
            expected_ticks.len(),
            "ticks {:?}: {c}",
            axis.ticks
        );
        for (position, label) in expected_ticks {
            assert!(
                axis.ticks
                    .iter()
                    .any(|t| t.label == label && equal((t.position - 100.) / 440., position)),
                "ticks {:?}: {c}",
                axis.ticks
            );
        }
        let wire = p.unwrap().to_json().unwrap();
        assert_eq!(
            Plot::from_json_with_extensions(&wire, registry.clone())
                .unwrap()
                .to_json()
                .unwrap(),
            wire
        );
    }
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
fn nullable_endpoints_share_the_same_scale_replacement() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-missing.json"
    ))
    .unwrap();
    for c in fixture["endpoint_cases"].as_array().unwrap() {
        let data = Data::columns()
            .column("x", vec![None, Some(-4.), Some(4.)])
            .column("xend", vec![Some(4.), None, Some(9.)])
            .build()
            .unwrap();
        let scale = if c["transform"] == "sqrt" {
            scale_sqrt()
        } else {
            scale_linear()
        };
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).x2("xend").y2(2.))
            .layer(rule())
            .x_axis(
                x_axis()
                    .scale(scale.domain(1., 4.))
                    .missing_value(Some(Number(5.))),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        assert_eq!(prepared.layers()[0].marks().len(), 3);
        for (i, m) in prepared.layers()[0].marks().iter().enumerate() {
            let chart_core::grammar::PreparedGeometry::Rule { from, to } = &m.geometry else {
                panic!("{:?}", m.geometry)
            };
            assert!(equal(from.x(), number(&c["x"][i])) && equal(to.x(), number(&c["xend"][i])));
        }
    }
}
#[test]
fn missing_replacement_requires_the_correct_stage_and_family() {
    for profile in [Profile::LibraryV1, Profile::Ggplot2_4_0_3] {
        let axis = x_axis()
            .coordinate_scale(scale_linear())
            .missing_value(Some(Number(0.)));
        let p = plot(Data::columns().column("x", [1.]).build().unwrap())
            .profile(profile)
            .aes(aes().x("x").y(1.))
            .layer(points())
            .x_axis(axis)
            .build();
        assert!(p.and_then(|p| p.chart()?.prepare()).is_err());
    }
    let projection = chart_core::grammar::ScaleProjection {
        id: chart_core::ScaleId::new(0),
        timestamp: None,
        binned: None,
        transform: Some(chart_core::scales::ScaleTransform::Sqrt),
        limits: Some(chart_core::scales::Bounds::new(1., 4.).unwrap()),
        function_limits: None,
        missing: Some(Number(5.)),
        outside: chart_core::grammar::ScaleOob::Censor,
    };
    assert_eq!(projection.project_optional(None), Some(5.));
    assert_eq!(projection.project(-1.), Some(5.));
    assert_eq!(projection.project(9.), Some(5.));
    assert_eq!(projection.project(4.), Some(2.));
    assert_eq!(projection.project(f64::INFINITY), Some(f64::INFINITY));
    assert_eq!(projection.project_transformed(5.), Some(5.));
}
