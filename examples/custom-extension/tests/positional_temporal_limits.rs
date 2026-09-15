//! FIX-GG04: temporal callbacks preserve exact origins through both training stages.
use chart_core::{
    data::TimeUnit,
    grammar::{PreparedGeometry, ScaleOob, StatField, ValueSpace},
    prelude::*,
};
fn number(value: &serde_json::Value) -> f64 {
    value.as_f64().unwrap_or_else(|| match value.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn calendar_zone(name: &str) -> chart_core::scales::CalendarZone {
    use chart_core::scales::*;
    if name == "UTC" {
        return CalendarZone::Utc;
    }
    CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
        version: 1,
        zone: name.into(),
        revision: chart_core::Revision::INITIAL,
        tzdata: "explicit-2024-US-transitions".into(),
        coverage: TimeBounds {
            start: 1_710_000_000_000,
            end: 1_740_000_000_000,
        },
        initial_offset_seconds: -18000,
        transitions: vec![
            TimeZoneTransition {
                at_millis: 1_710_054_000_000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1_730_613_600_000,
                offset_seconds: -18000,
            },
        ],
    }))
}
#[test]
fn calendar_callback_units_and_resource_coverage_are_checked() {
    use chart_core::scales::*;
    for wrong_unit in [false, true] {
        let unit = if wrong_unit {
            TimeUnit::Microseconds
        } else {
            TimeUnit::Milliseconds
        };
        let factor = if wrong_unit { 1000 } else { 1 };
        let origin = 1_730_606_400_000 * factor;
        let data = Data::columns()
            .column(
                "t",
                nullable_timestamps(
                    vec![Some(origin), Some(origin + 1000 * factor)],
                    unit,
                    "UTC",
                ),
            )
            .build()
            .unwrap();
        let mut zone = calendar_zone("America/New_York");
        if !wrong_unit && let CalendarZone::Local(rules) = &mut zone {
            let rules = std::sync::Arc::make_mut(rules);
            rules.coverage.end = 1_712_000_000_000;
            rules.transitions.truncate(1);
        }
        let result = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(chart_extension_example::registry().unwrap())
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "t".into(),
                        origin,
                    })
                    .y(1.),
            )
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(scale_calendar(TimeScaleSpec {
                        domain: vec![1_710_046_800_000, 1_710_133_200_000],
                        zone,
                        ..Default::default()
                    }))
                    .limits_function(chart_extension_example::numeric_limits::operation(
                        "identity",
                    )),
            )
            .build()
            .and_then(|p| p.chart())
            .and_then(|mut c| c.prepare())
            .and_then(frame);
        assert_eq!(
            result.unwrap_err().code,
            if wrong_unit {
                chart_core::DiagnosticCode::SchemaConflict
            } else {
                chart_core::DiagnosticCode::MissingResource
            }
        );
    }
}
#[test]
fn calendar_timestamp_callbacks_match_reference_builds() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-calendar-functions.json"
    ))
    .unwrap();
    check_fixture(&fixture, false, false, (588, 60));
}
#[test]
fn automatic_callback_population_type_is_consistent() {
    let registry = chart_extension_example::registry().unwrap();
    let origin = 1_704_067_200_000_i64;
    for reverse in [false, true] {
        for mixed in [false, true] {
            let data = Data::columns()
                .column("n", [1., 2.])
                .column(
                    "t",
                    nullable_timestamps(
                        vec![Some(origin), Some(origin + 1000)],
                        TimeUnit::Milliseconds,
                        "UTC",
                    ),
                )
                .build()
                .unwrap();
            let numeric = points().aes(aes().x("n").y(1.));
            let other = points().aes(
                aes()
                    .x(if mixed {
                        Mapping::Timestamp {
                            field: "t".into(),
                            origin,
                        }
                    } else {
                        Mapping::from("n")
                    })
                    .y(1.),
            );
            let layers = if reverse {
                [other, numeric]
            } else {
                [numeric, other]
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry.clone())
                .x_axis(x_axis().limits_function(
                    chart_extension_example::numeric_limits::operation("identity"),
                ))
                .layer(layers[0].clone())
                .layer(layers[1].clone())
                .build();
            let result = p.and_then(|p| p.chart()).and_then(|mut c| c.prepare());
            if mixed {
                assert_eq!(
                    result.unwrap_err().code,
                    chart_core::DiagnosticCode::SchemaConflict
                );
            } else {
                assert_eq!(result.unwrap().layers()[0].marks().len(), 2);
            }
        }
    }
}
#[test]
fn automatic_timestamp_callbacks_match_reference_builds() {
    let mut fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-temporal-functions.json"
    ))
    .unwrap();
    fixture["cases"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["kind"] == "datetime");
    check_fixture(&fixture, false, true, (363, 393));
    let mut fractional: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-temporal-fractional-statistics.json"
    ))
    .unwrap();
    fractional["cases"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["kind"] == "datetime");
    check_fixture(&fractional, false, true, (108, 0));
}
#[test]
fn temporal_callbacks_match_reference_builds() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-temporal-functions.json"
    ))
    .unwrap();
    check_fixture(&fixture, false, false, (726, 786));
}
#[test]
fn duration_callbacks_match_reference_builds() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-duration-functions.json"
    ))
    .unwrap();
    check_fixture(&fixture, true, false, (150, 102));
}
#[test]
fn fractional_temporal_statistics_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-temporal-fractional-statistics.json"
    ))
    .unwrap();
    check_fixture(&fixture, false, false, (216, 0));
}
fn check_fixture(
    fixture: &serde_json::Value,
    duration: bool,
    automatic: bool,
    expected_counts: (usize, usize),
) {
    let registry = chart_extension_example::registry().unwrap();
    let mut checked = 0;
    let mut rejected = 0;
    let units = if duration {
        vec![(TimeUnit::Seconds, 1_i64)]
    } else {
        vec![
            (TimeUnit::Milliseconds, 1000_i64),
            (TimeUnit::Microseconds, 1000000),
            (TimeUnit::Nanoseconds, 1000000000),
        ]
    };
    for (unit, ticks) in units {
        for c in fixture["cases"].as_array().unwrap() {
            let date = c["kind"] == "date";
            let is_summary = c["context"] == "summary";
            let factor = ticks as f64 * if date { 86400. } else { 1. };
            let epoch = if let Some(epoch) = c["epoch"].as_f64() {
                epoch
            } else if duration {
                0.
            } else if date {
                19723.
            } else {
                1704067200.
            };
            let origin = if c["epoch"].is_number() {
                epoch as i64 * ticks
            } else {
                1704067200_i64 * ticks
            };
            let inputs: Vec<_> = c["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    (!v.is_null()).then(|| origin + ((number(v) - epoch) * factor).round() as i64)
                })
                .collect();
            let mapping = if duration {
                Mapping::from("t")
            } else {
                Mapping::Timestamp {
                    field: "t".into(),
                    origin,
                }
            };
            let data = if duration {
                Data::columns()
                    .column(
                        "t",
                        c["inputs"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| (!v.is_null()).then(|| number(v)))
                            .collect::<Vec<Option<f64>>>(),
                    )
                    .build()
                    .unwrap()
            } else {
                Data::columns()
                    .column("t", nullable_timestamps(inputs, unit, "UTC"))
                    .build()
                    .unwrap()
            };
            let mut axis = if is_summary { y_axis() } else { x_axis() };
            if !automatic {
                axis = axis.scale(if c["calendar"] == true {
                    scale_calendar(chart_core::scales::TimeScaleSpec {
                        domain: vec![origin, origin + 86400 * ticks],
                        unit,
                        zone: calendar_zone(c["zone"].as_str().unwrap()),
                        ..Default::default()
                    })
                } else if duration {
                    scale_duration()
                } else if date {
                    scale_date()
                } else {
                    scale_utc()
                });
            }
            let axis = axis
                .range(100., 540.)
                .guide_geometry(Some(chart_core::layout::GuideGeometry {
                    labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                    ..Default::default()
                }))
                .limits_function(chart_extension_example::numeric_limits::operation(
                    c["control"].as_str().unwrap(),
                ))
                .oob(match c["oob"].as_str().unwrap() {
                    "squish" => ScaleOob::Squish,
                    "keep" => ScaleOob::Keep,
                    _ => ScaleOob::Censor,
                });
            let builder = if is_summary {
                plot(data)
                    .aes(aes().x(1.).y(mapping.clone()))
                    .layer(
                        points()
                            .stat(summary().x(mapping))
                            .after_stat(stat_aes().x(1.).y(StatField::Mean)),
                    )
                    .y_axis(axis)
            } else {
                plot(data)
                    .aes(aes().x(mapping).y(1.))
                    .layer(points())
                    .x_axis(axis)
            };
            let p = builder
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry.clone())
                .build();
            let prepared = p.and_then(|p| p.chart()?.prepare());
            if c["result"].get("error").is_some() {
                assert!(
                    prepared.and_then(frame).is_err(),
                    "Reference rejection accepted: {c}"
                );
                rejected += 1;
                continue;
            }
            let prepared = prepared.unwrap_or_else(|e| panic!("{e:?}: {c}"));
            let layer = &prepared.layers()[0];
            let space = if is_summary {
                &layer.domains().y_space
            } else {
                &layer.domains().x_space
            };
            assert!(
                duration
                    || matches!(space, Some(ValueSpace::Timestamp { origin: actual, .. }) if *actual == origin),
                "{space:?}: {c}"
            );
            let actual: Vec<_> = layer
                .marks()
                .iter()
                .map(|m| match m.geometry {
                    PreparedGeometry::Point(p) => {
                        if is_summary {
                            p.y()
                        } else {
                            p.x()
                        }
                    }
                    _ => panic!(),
                })
                .collect();
            let expected: Vec<_> = c["result"]["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(number)
                .filter(|v| v.is_finite())
                .map(|v| (v - epoch) * factor)
                .collect();
            assert_eq!(actual.len(), expected.len(), "{actual:?}: {c}");
            // R summaries round absolute Date/POSIXct doubles; the core retains offsets.
            let tolerance = if duration {
                3e-12
            } else {
                2. * f64::EPSILON * epoch * factor
            };
            assert!(
                actual
                    .iter()
                    .zip(expected.iter())
                    .all(|(a, b)| (a - b).abs() <= tolerance),
                "{actual:?} != {expected:?}: {c}"
            );
            let laid = frame(prepared.clone()).unwrap_or_else(|e| panic!("{e:?}: {c}"));
            let axis = &laid.axes()[&chart_core::ScaleId::new(if is_summary { 1 } else { 0 })];
            let range = match &axis.scale {
                chart_core::layout::ResolvedScale::Calendar(s) => vec![
                    epoch + s.relative_viewport().start() / factor,
                    epoch + s.relative_viewport().end() / factor,
                ],
                chart_core::layout::ResolvedScale::Unbounded(s) => {
                    s.viewport().iter().map(|v| v.0).collect()
                }
                chart_core::layout::ResolvedScale::Linear(s) if duration => {
                    vec![s.viewport().start(), s.viewport().end()]
                }
                _ => panic!("wrong temporal scale"),
            };
            assert!(
                range
                    .iter()
                    .zip(c["result"]["range"].as_array().unwrap())
                    .all(|(a, b)| *a == number(b)),
                "{range:?}: {c}"
            );
            let expected_positions: Vec<_> = c["result"]["point_positions"]
                .as_array()
                .unwrap()
                .iter()
                .map(number)
                .filter(|v| v.is_finite())
                .collect();
            let range = axis.spec.range.unwrap();
            for (value, expected) in actual.iter().zip(expected_positions) {
                let mapped = axis.map(*value, space.as_ref().unwrap()).unwrap().unwrap();
                let normalized = (mapped - range.start()) / (range.end() - range.start());
                assert!(
                    (normalized - expected).abs() < 3e-12,
                    "position {normalized} != {expected}: {c}"
                );
            }
            let expected_ticks: Vec<_> = c["result"]["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .filter(|(p, _)| number(p).is_finite())
                .collect();
            assert_eq!(
                axis.ticks.len(),
                expected_ticks.len(),
                "ticks {:?}: {c}",
                axis.ticks
            );
            for (tick, (position, label)) in axis.ticks.iter().zip(expected_ticks) {
                let normalized = (tick.position - range.start()) / (range.end() - range.start());
                assert!(
                    (normalized - number(position)).abs() < 3e-12,
                    "tick {normalized}: {c}"
                );
                assert_eq!(tick.label, label.as_str().unwrap(), "{c}");
            }
            checked += 1;
        }
    }
    assert_eq!((checked, rejected), expected_counts);
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
fn temporal_callbacks_reject_precision_loss_before_training() {
    for outside in [ScaleOob::Censor, ScaleOob::Squish, ScaleOob::Keep] {
        let data = Data::columns()
            .column(
                "t",
                nullable_timestamps(vec![Some((1_i64 << 53) + 1)], TimeUnit::Nanoseconds, "UTC"),
            )
            .build()
            .unwrap();
        let result = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(chart_extension_example::registry().unwrap())
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "t".into(),
                        origin: 0,
                    })
                    .y(1.),
            )
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(scale_utc())
                    .oob(outside)
                    .limits_function(chart_extension_example::numeric_limits::operation("fixed")),
            )
            .build()
            .and_then(|p| p.chart()?.prepare());
        assert_eq!(
            result.unwrap_err().code,
            chart_core::DiagnosticCode::PrecisionLoss
        );
    }
}
