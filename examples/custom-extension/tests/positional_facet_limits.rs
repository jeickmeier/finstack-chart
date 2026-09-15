//! FIX-GG04: callbacks train across the full shared facet population.
use chart_core::{
    grammar::{EmptyPanels, GroupValue, PanelKey, PreparedGeometry, ScaleOob, StatField},
    prelude::*,
};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
#[test]
fn automatic_timestamp_facet_callbacks_match_reference() {
    let mut fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-facet-temporal-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array_mut().unwrap();
    cases.retain(|c| c["kind"] == "datetime");
    for c in cases {
        c["automatic_axis"] = true.into();
    }
    for (unit, ticks) in [
        (chart_core::data::TimeUnit::Milliseconds, 1000),
        (chart_core::data::TimeUnit::Microseconds, 1000000),
        (chart_core::data::TimeUnit::Nanoseconds, 1000000000),
    ] {
        let actual = check_fixture(&fixture, unit, ticks);
        assert_eq!(actual, (229, 191));
    }
}
#[test]
fn authored_facet_limits_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-facet-authored-limits.json"
    ))
    .unwrap();
    let actual = check_fixture(&fixture, chart_core::data::TimeUnit::Milliseconds, 1000);
    println!("authored facets: {actual:?}");
    assert_eq!(actual, (714, 6));
}
#[test]
fn facet_callbacks_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-facet-functions.json"
    ))
    .unwrap();
    assert_eq!(
        check_fixture(&fixture, chart_core::data::TimeUnit::Milliseconds, 1000),
        (597, 411)
    );
}
#[test]
fn temporal_facet_callbacks_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-facet-temporal-functions.json"
    ))
    .unwrap();
    for (unit, ticks) in [
        (chart_core::data::TimeUnit::Milliseconds, 1000),
        (chart_core::data::TimeUnit::Microseconds, 1000000),
        (chart_core::data::TimeUnit::Nanoseconds, 1000000000),
    ] {
        let actual = check_fixture(&fixture, unit, ticks);
        println!("temporal {unit:?}: {actual:?}");
        assert_eq!(actual, (717, 543));
    }
}
#[test]
fn binned_facet_callbacks_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-facet-bin-functions.json"
    ))
    .unwrap();
    assert_eq!(
        check_fixture(&fixture, chart_core::data::TimeUnit::Milliseconds, 1000),
        (626, 526)
    );
}
fn check_fixture(
    fixture: &serde_json::Value,
    unit: chart_core::data::TimeUnit,
    ticks: i64,
) -> (usize, usize) {
    let registry = chart_extension_example::registry().unwrap();
    let mut successes = 0;
    let mut errors = 0;
    for c in fixture["cases"].as_array().unwrap().iter() {
        let summary_case = c["context"] == "summary";
        let temporal = matches!(c["kind"].as_str(), Some("date" | "datetime"));
        let date = c["kind"] == "date";
        let epoch = if date {
            19723.
        } else if temporal {
            1704067200.
        } else {
            0.
        };
        let factor = if temporal {
            ticks as f64 * if date { 86400. } else { 1. }
        } else {
            1.
        };
        let origin = 1704067200_i64 * ticks;
        let mapping = if temporal {
            Mapping::Timestamp {
                field: "v".into(),
                origin,
            }
        } else {
            Mapping::from("v")
        };
        let inputs: Vec<_> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64())
            .collect();
        let data = if temporal {
            Data::columns().column(
                "v",
                nullable_timestamps(
                    inputs
                        .iter()
                        .map(|v| v.map(|v| origin + ((v - epoch) * factor).round() as i64))
                        .collect::<Vec<_>>(),
                    unit,
                    "UTC",
                ),
            )
        } else {
            Data::columns().column("v", inputs)
        }
        .column("panel", categorical(["A", "A", "B", "B"]))
        .build()
        .unwrap();
        let mut axis = if summary_case { y_axis() } else { x_axis() };
        if c["automatic_axis"] != true {
            axis = axis.scale(
                match c["kind"].as_str().or(c["transform"].as_str()).unwrap() {
                    "binned" => scale_binned(chart_core::scales::GgplotBinnedPosition {
                        transform: match c["transform"].as_str().unwrap() {
                            "sqrt" => Some(chart_core::scales::ScaleTransform::Sqrt),
                            "reverse" => Some(chart_core::scales::ScaleTransform::Reverse),
                            _ => None,
                        },
                        ..Default::default()
                    }),
                    "date" => scale_date(),
                    "datetime" => scale_utc(),
                    "duration" => scale_duration(),
                    "sqrt" => scale_sqrt(),
                    "reverse" => scale_reverse(),
                    _ => scale_linear(),
                },
            );
        }
        axis = axis
            .range(100., 540.)
            .guide_geometry(Some(chart_core::layout::GuideGeometry {
                labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                ..Default::default()
            }))
            .oob(match c["oob"].as_str().unwrap() {
                "squish" => ScaleOob::Squish,
                "keep" => ScaleOob::Keep,
                _ => ScaleOob::Censor,
            });
        if c["authored"] == true {
            let values = c["limits"].as_array().unwrap();
            axis = axis.numeric_limits(Some(
                [0, 1].map(|i| values[i].as_f64().map(chart_core::interpolate::Number)),
            ));
        } else if c["control"] != "automatic" {
            axis = axis.limits_function(chart_extension_example::numeric_limits::operation(
                c["control"].as_str().unwrap(),
            ));
        }
        let b = if summary_case {
            plot(data)
                .aes(aes().x(1.).y(mapping.clone()))
                .layer(
                    points()
                        .stat(summary().x(mapping.clone()))
                        .after_stat(stat_aes().x(1.).y(StatField::Mean)),
                )
                .y_axis(axis)
        } else {
            plot(data)
                .aes(aes().x(mapping).y(1.))
                .layer(points())
                .x_axis(axis)
        };
        let order = c["levels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| PanelKey {
                values: vec![GroupValue::Text(s.as_str().unwrap().into())],
            })
            .collect();
        let prepared = b
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone())
            .facet(
                facet_wrap("panel")
                    .free_x(c["policy"] == "free" && !summary_case)
                    .free_y(c["policy"] == "free" && summary_case)
                    .columns(3)
                    .order(order)
                    .empty(EmptyPanels::Keep),
            )
            .build()
            .and_then(|p| p.chart()?.prepare());
        if c["result"].get("error").is_some() {
            assert!(
                prepared.and_then(frame).is_err(),
                "accepted reference rejection {c}"
            );
            errors += 1;
            continue;
        }
        let prepared = prepared.unwrap_or_else(|e| panic!("{e:?}: {c}"));
        assert_eq!(
            prepared.panels().len(),
            c["result"]["panels"].as_array().unwrap().len()
        );
        // Exercise the combined facet layout as well as each retained panel's axes.
        frame(prepared.clone()).unwrap_or_else(|e| panic!("{e:?}: {c}"));
        for (panel, expected) in prepared
            .panels()
            .iter()
            .zip(c["result"]["panels"].as_array().unwrap())
        {
            let chart = &panel.chart;
            let layout = frame(chart.clone()).unwrap_or_else(|e| panic!("{e:?}: {c}"));
            let axis = &layout.axes()[&chart_core::ScaleId::new(if summary_case { 1 } else { 0 })];
            let actual: Vec<_> = chart.layers()[0]
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
                    _ => panic!(),
                })
                .collect();
            let values: Vec<_> = expected["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(number)
                .filter(|v| v.is_finite())
                .map(|v| (v - epoch) * factor)
                .collect();
            assert_eq!(actual.len(), values.len(), "{actual:?} vs {values:?}: {c}");
            for (a, b) in actual.iter().zip(values) {
                assert!(
                    (a - b).abs()
                        <= if temporal {
                            2. * f64::EPSILON * epoch * factor
                        } else {
                            3e-12
                        },
                    "{a} != {b}: {c}"
                );
            }
            let range = axis.spec.range.unwrap();
            let space = if summary_case {
                &chart.layers()[0].domains().y_space
            } else {
                &chart.layers()[0].domains().x_space
            };
            let space = space
                .as_ref()
                .unwrap_or(&chart_core::grammar::ValueSpace::Data);
            for (a, b) in actual.iter().zip(
                expected["point_positions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .filter(|v| v.is_finite()),
            ) {
                let mapped = axis.map(*a, space).unwrap().unwrap();
                let n = (mapped - range.start()) / (range.end() - range.start());
                assert!((n - b).abs() < 3e-12, "point {n} != {b}: {c}");
            }
            let ticks: Vec<_> = expected["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(expected["labels"].as_array().unwrap())
                .filter(|(p, _)| number(p).is_finite())
                .collect();
            assert_eq!(axis.ticks.len(), ticks.len(), "{c}");
            for (t, (p, l)) in axis.ticks.iter().zip(ticks) {
                let n = (t.position - range.start()) / (range.end() - range.start());
                assert!((n - number(p)).abs() < 3e-12, "tick {n}: {c}");
                assert_eq!(t.label, l.as_str().unwrap(), "{c}");
            }
        }
        successes += 1;
    }
    println!("{successes} reference successes; {errors} reference errors");
    (successes, errors)
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
fn shared_callback_source_training_obeys_filters_and_panel_catalog() {
    let data = Data::columns()
        .column("v", vec![1., 3., 100., 120.])
        .column("panel", categorical(["A", "A", "B", "B"]))
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("v").y(1.))
        .layer(points().filter(filter("v").maximum(10.)))
        .x_axis(
            x_axis().limits_function(chart_extension_example::numeric_limits::operation(
                "identity",
            )),
        )
        .facet(facet_wrap("panel").order(vec![PanelKey {
            values: vec![GroupValue::Text("A".into())],
        }]))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let panel = &prepared.panels()[0].chart;
    let limits = panel
        .definition()
        .axes
        .iter()
        .find(|a| a.id == chart_core::ScaleId::new(0))
        .unwrap()
        .resolved_limits
        .as_ref()
        .unwrap();
    assert_eq!(limits.iter().map(|n| n.0).collect::<Vec<_>>(), vec![1., 3.]);
    assert_eq!(panel.layers()[0].marks().len(), 2);
}
