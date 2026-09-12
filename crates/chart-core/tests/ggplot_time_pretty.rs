//! FIX-GG04: independently generated automatic datetime candidates and labels.
use chart_core::{Revision, data::TimeUnit, scales::*};
fn calendar(name: &str) -> Calendar {
    let zone = if name == "UTC" {
        CalendarZone::Utc
    } else {
        CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
            version: 1,
            zone: name.into(),
            revision: Revision::INITIAL,
            tzdata: "explicit-US-2024-2025".into(),
            coverage: TimeBounds {
                start: 1_704_067_200_000,
                end: 1_767_225_600_000,
            },
            initial_offset_seconds: -18000,
            transitions: vec![
                (1_710_054_000_000, -14400),
                (1_730_613_600_000, -18000),
                (1_741_503_600_000, -14400),
                (1_762_063_200_000, -18000),
            ]
            .into_iter()
            .map(|(at_millis, offset_seconds)| TimeZoneTransition {
                at_millis,
                offset_seconds,
            })
            .collect(),
        }))
    };
    Calendar::new(zone).unwrap()
}
#[test]
fn automatic_datetime_candidates_and_default_labels_match_r() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-pretty.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 504);
    let mut errors = Vec::new();
    for case in cases {
        let millis = |v: &serde_json::Value| (v.as_f64().unwrap() * 1000.).round() as i64;
        let start = millis(&case["limits"][0]);
        let end = millis(&case["limits"][1]);
        let actual = ggplot_breaks_pretty_time(
            start,
            Bounds::new(0., (end - start) as f64).unwrap(),
            TimeUnit::Milliseconds,
            &calendar(case["zone"].as_str().unwrap()),
            case["n"].as_f64().unwrap(),
            100,
        )
        .unwrap();
        let expected: Vec<_> = case["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(millis)
            .collect();
        let labels: Vec<_> = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        if actual.values != expected || actual.labels != labels {
            errors.push(format!("{case}\nactual {actual:?}"));
        }
    }
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

use chart_core::{
    Rect, ResourceId, ScaleId, composition::ScaleValue, layout::*, prelude::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 1000., 300.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
#[test]
fn automatic_time_policy_reaches_primary_authoring_and_default_selection() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-pretty.json"
    ))
    .unwrap();
    for case in f["cases"].as_array().unwrap() {
        let millis = |v: &serde_json::Value| (v.as_f64().unwrap() * 1000.).round() as i64;
        let domain = vec![millis(&case["limits"][0]), millis(&case["limits"][1])];
        let zone = case["zone"].as_str().unwrap();
        let axis = x_axis()
            .scale(scale_calendar(TimeScaleSpec {
                domain: domain.clone(),
                zone: calendar(zone).zone().clone(),
                ..Default::default()
            }))
            .expansion(Some(GgplotExpansion {
                mult: [0.; 2],
                add: [0.; 2],
            }))
            .guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }));
        let p = plot(
            Data::columns()
                .column("x", timestamps(domain, TimeUnit::Milliseconds, zone))
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(axis.clone().tick_arguments(Some(GuideTickArguments {
            count: case["n"].as_f64(),
            ..Default::default()
        })))
        .build()
        .unwrap();
        let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
        let expected: Vec<_> = case["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(millis)
            .collect();
        let labels: Vec<_> = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            ticks
                .iter()
                .map(|t| match t.value {
                    ScaleValue::Timestamp { value, .. } => value,
                    _ => panic!(),
                })
                .collect::<Vec<_>>(),
            expected,
            "{case}"
        );
        assert_eq!(
            ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
            labels,
            "{case}"
        );
        if case["n"] == 5 {
            let default = p.edit().x_axis(axis).build().unwrap();
            let frame = layout(
                default.chart().unwrap().prepare().unwrap(),
                &request(),
                &Metrics,
            )
            .unwrap();
            assert_eq!(
                &frame.axes()[&ScaleId::new(0)].ticks,
                ticks,
                "default: {case}"
            );
        }
    }
}

#[test]
fn automatic_time_format_overrides_and_resource_boundaries() {
    let calendar = calendar("UTC");
    let view = Bounds::new(0., 10_000.).unwrap();
    for count in [f64::NAN, f64::INFINITY, 0., -1., 129.] {
        assert!(
            ggplot_breaks_pretty_time(0, view, TimeUnit::Milliseconds, &calendar, count, 100)
                .is_err()
        );
    }
    assert_eq!(
        ggplot_breaks_pretty_time(0, view, TimeUnit::Milliseconds, &calendar, 5., 1)
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::ResourceLimit
    );
    let axis = x_axis().scale(scale_utc()).expansion(Some(GgplotExpansion {
        mult: [0.; 2],
        add: [0.; 2],
    }));
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(vec![0, 10_000], TimeUnit::Milliseconds, "UTC"),
            )
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(axis.clone())
    .build()
    .unwrap();
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let original = &frame.axes()[&ScaleId::new(0)].ticks;
    assert_eq!(
        original
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        ["00", "02", "04", "06", "08", "10"]
    );
    let formatted = p
        .edit()
        .x_axis(
            axis.clone()
                .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                    pattern: Some("%H:%M:%S".into()),
                    ..Default::default()
                })))),
        )
        .build()
        .unwrap();
    let frame = layout(
        formatted.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(
        frame.axes()[&ScaleId::new(0)]
            .ticks
            .iter()
            .map(|t| t.value.clone())
            .collect::<Vec<_>>(),
        original.iter().map(|t| t.value.clone()).collect::<Vec<_>>()
    );
    assert_eq!(frame.axes()[&ScaleId::new(0)].ticks[0].label, "00:00:00");
    let empty = p
        .edit()
        .x_axis(
            axis.tick_arguments(Some(GuideTickArguments {
                count: Some(1000.),
                ..Default::default()
            }))
            .tick_values(Some(vec![])),
        )
        .build()
        .unwrap();
    assert!(
        layout(
            empty.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap()
        .axes()[&ScaleId::new(0)]
            .ticks
            .is_empty()
    );
    assert_eq!(
        &layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics)
            .unwrap()
            .axes()[&ScaleId::new(0)]
            .ticks,
        original
    );
}

#[test]
fn expanded_automatic_datetime_guides_match_r() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-pretty-expansion.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    for case in cases {
        let pair = |v: &serde_json::Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
        let domain = case["limits_ms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<_>>();
        let policy = GgplotExpansion {
            mult: pair(&case["policy"]["mult"]),
            add: pair(&case["policy"]["add"]),
        };
        let zone = case["zone"].as_str().unwrap();
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    timestamps(domain.clone(), TimeUnit::Milliseconds, zone),
                )
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_calendar(TimeScaleSpec {
                    domain,
                    zone: calendar(zone).zone().clone(),
                    ..Default::default()
                }))
                .expansion((policy != GgplotExpansion::default()).then_some(policy))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
        let expected = case["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v.as_f64().unwrap() * 1000.).round() as i64)
            .collect::<Vec<_>>();
        assert_eq!(
            ticks
                .iter()
                .map(|t| match t.value {
                    ScaleValue::Timestamp { value, .. } => value,
                    _ => panic!(),
                })
                .collect::<Vec<_>>(),
            expected,
            "{case}"
        );
        assert_eq!(
            ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
            case["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            "{case}"
        );
    }
}

#[test]
fn formatter_changes_keep_authored_time_intervals_and_unrepresentable_ticks_reject() {
    for calendar_axis in [false, true] {
        let scale = if calendar_axis {
            scale_calendar(TimeScaleSpec {
                domain: vec![0, 10_000],
                ..Default::default()
            })
            .calendar_interval(CalendarInterval {
                unit: CalendarUnit::Second,
                step: 3,
            })
        } else {
            scale_utc().interval(UtcInterval::Seconds(3))
        };
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    timestamps(vec![0, 10_000], TimeUnit::Milliseconds, "UTC"),
                )
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale)
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                    pattern: Some("%S".into()),
                    ..Default::default()
                })))),
        )
        .build()
        .unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        assert_eq!(
            frame.axes()[&ScaleId::new(0)]
                .ticks
                .iter()
                .map(|t| t.label.as_str())
                .collect::<Vec<_>>(),
            ["00", "03", "06", "09"]
        );
    }
    let error = ggplot_breaks_pretty_time(
        1_700_000_000_000_000_000,
        Bounds::new(0.1, 1.1).unwrap(),
        TimeUnit::Nanoseconds,
        &calendar("UTC"),
        5.,
        100,
    )
    .unwrap_err();
    assert_eq!(error.code, chart_core::DiagnosticCode::PrecisionLoss);
}
