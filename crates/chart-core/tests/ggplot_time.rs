//! FIX-GG04: exact elapsed break identities and supplied-zone labels through primary layout.
use chart_core::{
    DiagnosticCode, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, data::TimeUnit,
    layout::*, prelude::*, scales::*, services::*,
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
fn zone(name: &str, start: i64) -> CalendarZone {
    if name == "UTC" {
        return CalendarZone::Utc;
    }
    // Each resource is bounded to the fixture's season. It does not claim historical
    // timezone coverage beyond the supplied transition table.
    let (coverage, offset, transitions) = if start <= 0 {
        (
            TimeBounds {
                start: -86_400_000,
                end: 86_400_000,
            },
            -18000,
            vec![],
        )
    } else if start < 1_710_000_000_000 {
        (
            TimeBounds {
                start: 1_699_900_000_000,
                end: 1_700_100_000_000,
            },
            -18000,
            vec![],
        )
    } else {
        (
            TimeBounds {
                start: 1_710_000_000_000,
                end: 1_740_000_000_000,
            },
            -18000,
            vec![
                TimeZoneTransition {
                    at_millis: 1_710_054_000_000,
                    offset_seconds: -14400,
                },
                TimeZoneTransition {
                    at_millis: 1_730_613_600_000,
                    offset_seconds: -18000,
                },
            ],
        )
    };
    CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
        version: 1,
        zone: name.into(),
        revision: Revision::INITIAL,
        tzdata: "explicit-2024-US-transitions".into(),
        coverage,
        initial_offset_seconds: offset,
        transitions,
    }))
}
#[test]
fn reference_seconds_and_dst_labels_through_primary_calendar_axes() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-seconds.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    for case in cases {
        let millis = |v: &serde_json::Value| (v.as_f64().unwrap() * 1000.).round() as i64;
        let limits: Vec<_> = case["limits"]
            .as_array()
            .unwrap()
            .iter()
            .map(millis)
            .collect();
        let expected: Vec<_> = case["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(millis)
            .collect();
        let width = case["seconds"].as_f64().unwrap();
        let calendar = zone(case["zone"].as_str().unwrap(), limits[0]);
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    timestamps(
                        limits.clone(),
                        TimeUnit::Milliseconds,
                        case["zone"].as_str().unwrap(),
                    ),
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
                // The pinned reference plots explicitly disable expansion.
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .scale(scale_calendar(TimeScaleSpec {
                    domain: limits.clone(),
                    zone: calendar,
                    ..Default::default()
                }))
                .tick_arguments(Some(GuideTickArguments {
                    seconds: Some(width),
                    ..Default::default()
                }))
                .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                    // Shared formatter's %Z is the numeric offset emitted by R's %z.
                    pattern: Some("%Y-%m-%d %H:%M:%S %Z".into()),
                    ..Default::default()
                }))))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        assert_eq!(p.definition().wire_version(), 17);
        let wire = p.to_json().unwrap();
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let axis = &frame.axes()[&ScaleId::new(0)];
        let actual: Vec<_> = axis
            .ticks
            .iter()
            .map(|t| {
                let ScaleValue::Timestamp {
                    value,
                    unit: TimeUnit::Milliseconds,
                } = t.value
                else {
                    panic!()
                };
                value
            })
            .collect();
        assert_eq!(actual, expected, "{case}");
        let labels: Vec<_> = axis.ticks.iter().map(|t| t.label.as_str()).collect();
        let expected_labels: Vec<_> = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(labels, expected_labels, "{case}");
        for t in &axis.ticks {
            assert_eq!(Some(t.position), axis.map_value(&t.value).unwrap());
        }
    }
}
#[test]
fn seconds_use_exact_integer_epochs_and_bound_before_enumeration() {
    let base = 9_007_199_254_740_993;
    let b = TimeBounds {
        start: base,
        end: base + 20,
    };
    let expected = vec![base + 7, base + 17];
    assert_eq!(
        ggplot_breaks_seconds(b, TimeUnit::Nanoseconds, 1e-8, 2).unwrap(),
        expected
    );
    assert_eq!(
        ggplot_breaks_seconds(
            TimeBounds {
                start: b.end,
                end: b.start
            },
            TimeUnit::Nanoseconds,
            1e-8,
            2
        )
        .unwrap(),
        expected
    );
    assert_eq!(
        ggplot_breaks_seconds(b, TimeUnit::Nanoseconds, 1e-8, 1)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(
        ggplot_breaks_seconds(TimeBounds { start: -21, end: 1 }, TimeUnit::Seconds, 10., 3)
            .unwrap(),
        [-20, -10, 0]
    );
    for width in [0., -1., f64::NAN, f64::INFINITY] {
        assert_eq!(
            ggplot_breaks_seconds(b, TimeUnit::Seconds, width, 3)
                .unwrap_err()
                .code,
            DiagnosticCode::Validation
        );
    }
    assert_eq!(
        ggplot_breaks_seconds(b, TimeUnit::Seconds, 0.5, 3)
            .unwrap_err()
            .code,
        DiagnosticCode::PrecisionLoss
    );
    assert!(
        GuideTickArguments {
            count: Some(5.),
            seconds: Some(1.),
            ..Default::default()
        }
        .validate(100)
        .is_err()
    );
    assert!(
        GuideTickArguments {
            interval: Some(CalendarInterval::new(CalendarUnit::Day)),
            seconds: Some(1.),
            ..Default::default()
        }
        .validate(100)
        .is_err()
    );
}

#[test]
fn fixed_time_selection_is_versioned_and_explicit_values_bypass_generation() {
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
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_utc())
            .tick_arguments(Some(GuideTickArguments {
                seconds: Some(0.001),
                ..Default::default()
            }))
            .tick_values(Some(vec![])),
    )
    .build()
    .unwrap();
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 17,
        definition: p.definition().clone(),
    };
    envelope.validate().unwrap();
    envelope.version = 11;
    assert!(envelope.validate().is_err());
    let prepared = p.chart().unwrap().prepare().unwrap();
    let frame = layout(prepared.clone(), &request(), &Metrics).unwrap();
    assert!(frame.axes()[&ScaleId::new(0)].ticks.is_empty());
    let updated = p
        .edit()
        .x_axis(
            x_axis()
                .scale(scale_utc())
                .tick_arguments(Some(GuideTickArguments {
                    seconds: Some(0.001),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
    assert_eq!(
        layout(
            updated.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
    let updated = updated
        .edit()
        .x_axis(
            x_axis()
                .scale(scale_utc())
                .tick_arguments(Some(GuideTickArguments {
                    seconds: Some(5.),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
    let frame = layout(
        updated.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(frame.axes()[&ScaleId::new(0)].ticks.len(), 3);
    let updated = updated
        .edit()
        .y_axis(y_axis().tick_arguments(Some(GuideTickArguments {
            seconds: Some(5.),
            ..Default::default()
        })))
        .build()
        .unwrap();
    assert_eq!(
        layout(
            updated.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::UnsupportedCapability
    );
}

#[test]
fn reference_local_width_progression() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-widths.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for case in cases {
        let limits = case["limits"].as_array().unwrap();
        let bounds = TimeBounds {
            start: (limits[0].as_f64().unwrap() * 1000.) as i64,
            end: (limits[1].as_f64().unwrap() * 1000.) as i64,
        };
        let zone = if case["zone"] == "UTC" {
            CalendarZone::Utc
        } else {
            CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
                version: 1,
                zone: "America/New_York".into(),
                revision: Revision::INITIAL,
                tzdata: "explicit-US-2024-2026".into(),
                coverage: TimeBounds {
                    start: 1_704_067_200_000,
                    end: 1_799_000_000_000,
                },
                initial_offset_seconds: -18000,
                transitions: vec![
                    (1_710_054_000_000, -14400),
                    (1_730_613_600_000, -18000),
                    (1_741_503_600_000, -14400),
                    (1_762_063_200_000, -18000),
                    (1_772_953_200_000, -14400),
                    (1_793_512_800_000, -18000),
                ]
                .into_iter()
                .map(|(at_millis, offset_seconds)| TimeZoneTransition {
                    at_millis,
                    offset_seconds,
                })
                .collect(),
            }))
        };
        let calendar = Calendar::new(zone).unwrap();
        let width = match case["width"].as_str().unwrap() {
            "7 hours" => CalendarInterval {
                unit: CalendarUnit::Hour,
                step: 7,
            },
            "2 days" => CalendarInterval {
                unit: CalendarUnit::Day,
                step: 2,
            },
            "2 weeks" => CalendarInterval {
                unit: CalendarUnit::Week(WeekStart::Monday),
                step: 2,
            },
            "2 months" => CalendarInterval {
                unit: CalendarUnit::Month,
                step: 2,
            },
            "2 years" => CalendarInterval {
                unit: CalendarUnit::Year,
                step: 2,
            },
            _ => unreachable!(),
        };
        let actual =
            ggplot_breaks_width(bounds, TimeUnit::Milliseconds, &calendar, width, 100).unwrap();
        let expected: Vec<_> = case["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v.as_f64().unwrap() * 1000.) as i64)
            .collect();
        assert_eq!(actual, expected, "{case}");
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    timestamps(
                        vec![bounds.start, bounds.end],
                        TimeUnit::Milliseconds,
                        case["zone"].as_str().unwrap(),
                    ),
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
                // The pinned reference plots explicitly disable expansion.
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .scale(scale_calendar(TimeScaleSpec {
                    domain: vec![bounds.start, bounds.end],
                    zone: calendar.zone().clone(),
                    ..Default::default()
                }))
                .tick_arguments(Some(GuideTickArguments {
                    time_width: Some(width),
                    ..Default::default()
                }))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                }))
                .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                    pattern: Some("%Y-%m-%d %H:%M:%S %Z".into()),
                    ..Default::default()
                })))),
        )
        .build()
        .unwrap();
        assert_eq!(p.definition().wire_version(), 17);
        let wire = p.to_json().unwrap();
        let p = Plot::from_json(&wire).unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
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
fn width_validation_budgets_and_retained_edits() {
    let calendar = Calendar::new(CalendarZone::Utc).unwrap();
    let width = CalendarInterval {
        unit: CalendarUnit::Hour,
        step: 7,
    };
    let bounds = TimeBounds {
        start: -1,
        end: 40_000,
    };
    assert_eq!(
        ggplot_breaks_width(bounds, TimeUnit::Seconds, &calendar, width, 1).unwrap(),
        [21_600]
    );
    assert_eq!(
        ggplot_breaks_width(
            TimeBounds {
                start: bounds.end,
                end: bounds.start
            },
            TimeUnit::Seconds,
            &calendar,
            width,
            1
        )
        .unwrap(),
        [21_600]
    );
    assert_eq!(
        ggplot_breaks_width(bounds, TimeUnit::Seconds, &calendar, width, 0)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    for unit in [
        CalendarUnit::SourceTick,
        CalendarUnit::Millisecond,
        CalendarUnit::UnixDay,
        CalendarUnit::Week(WeekStart::Sunday),
    ] {
        assert_eq!(
            GuideTickArguments {
                time_width: Some(CalendarInterval::new(unit)),
                ..Default::default()
            }
            .validate(100)
            .unwrap_err()
            .code,
            DiagnosticCode::UnsupportedCapability
        );
    }
    assert!(
        GuideTickArguments {
            time_width: Some(width),
            seconds: Some(1.),
            ..Default::default()
        }
        .validate(100)
        .is_err()
    );
    assert!(
        GuideTickArguments {
            time_width: Some(CalendarInterval { step: 0, ..width }),
            ..Default::default()
        }
        .validate(100)
        .is_err()
    );
    let p = plot(
        Data::columns()
            .column("x", timestamps(vec![0, 100_000], TimeUnit::Seconds, "UTC"))
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_utc())
            .tick_arguments(Some(GuideTickArguments {
                time_width: Some(width),
                ..Default::default()
            })),
    )
    .build()
    .unwrap();
    assert_eq!(p.definition().wire_version(), 17);
    assert!(
        chart_core::portable::ChartEnvelope {
            version: 11,
            definition: p.definition().clone()
        }
        .validate()
        .is_err()
    );
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    assert_eq!(frame.axes()[&ScaleId::new(0)].ticks.len(), 4);
    let updated = p
        .edit()
        .x_axis(
            x_axis()
                .scale(scale_utc())
                .tick_arguments(Some(GuideTickArguments {
                    time_width: Some(CalendarInterval::new(CalendarUnit::Second)),
                    ..Default::default()
                }))
                .tick_values(Some(vec![])),
        )
        .build()
        .unwrap();
    assert!(
        layout(
            updated.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap()
        .axes()[&ScaleId::new(0)]
            .ticks
            .is_empty()
    );
    assert_eq!(
        layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics)
            .unwrap()
            .axes()[&ScaleId::new(0)]
            .ticks
            .len(),
        4
    );
    let invalid = p
        .edit()
        .y_axis(
            y_axis()
                .tick_arguments(Some(GuideTickArguments {
                    time_width: Some(width),
                    ..Default::default()
                }))
                .tick_values(Some(vec![])),
        )
        .build()
        .unwrap();
    assert_eq!(
        layout(
            invalid.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::UnsupportedCapability
    );
}

#[test]
fn reference_datetime_expansion_keeps_fractional_bounds() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-expansion.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for case in cases {
        let limits: Vec<_> = case["limits_ms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        let pair = |v: &serde_json::Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
        let expansion = GgplotExpansion {
            mult: pair(&case["policy"]["mult"]),
            add: pair(&case["policy"]["add"]),
        };
        for mode in 0..3 {
            let axis = match mode {
                0 => x_axis().scale(scale_calendar(TimeScaleSpec {
                    domain: limits.clone(),
                    ..Default::default()
                })),
                1 => x_axis().scale(scale_utc()),
                _ => x_axis(),
            }
            .expansion(Some(expansion))
            .tick_values(Some(vec![]));
            let p = plot(
                Data::columns()
                    .column(
                        "x",
                        timestamps(limits.clone(), TimeUnit::Milliseconds, "UTC"),
                    )
                    .column("y", [0., 1.])
                    .build()
                    .unwrap(),
            )
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(axis)
            .build()
            .unwrap();
            let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
            let frame =
                layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
            let axis = &frame.axes()[&ScaleId::new(0)];
            let ResolvedScale::Calendar(scale) = &axis.scale else {
                panic!()
            };
            assert_eq!(
                scale.domain(),
                TimeBounds {
                    start: limits[0],
                    end: limits[1]
                }
            );
            let view = scale.relative_viewport();
            // R stores absolute POSIX doubles. At a modern epoch its ULP is
            // 2^-22 seconds; allow two such ULPs when comparing its range.
            let epoch_tolerance = if limits[0].abs() > 1_000_000_000_000 {
                2_f64.powi(-21)
            } else {
                1e-12
            };
            for (actual, expected) in [view.start(), view.end()]
                .into_iter()
                .zip(pair(&case["relative_range"]))
            {
                assert!(
                    (actual / 1000. - expected).abs() <= epoch_tolerance,
                    "{case}: {actual}"
                );
            }
            for (value, expected) in limits.iter().zip(pair(&case["position"])) {
                let position = axis
                    .map_value(&ScaleValue::Timestamp {
                        value: *value,
                        unit: TimeUnit::Milliseconds,
                    })
                    .unwrap()
                    .unwrap();
                let normalized = (position - scale.range().start())
                    / (scale.range().end() - scale.range().start());
                let span_seconds = (view.end() - view.start()).abs() / 1000.;
                let tolerance = if span_seconds == 0. {
                    1e-12
                } else {
                    1e-12 + 2. * epoch_tolerance / span_seconds
                };
                assert!(
                    (normalized - expected).abs() <= tolerance,
                    "{case}: {normalized} != {expected}"
                );
            }
        }
    }
}

#[test]
fn continuous_panel_contraction_sorts_crossed_endpoints() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-expansion.json"
    ))
    .unwrap();
    for case in f["cases"].as_array().unwrap() {
        let pair = |v: &serde_json::Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
        let values = pair(&case["limits_ms"]).map(|v| v / 1000.);
        let expansion = GgplotExpansion {
            mult: pair(&case["policy"]["mult"]),
            add: pair(&case["policy"]["add"]),
        };
        let p = plot(
            Data::columns()
                .column("x", values)
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .expansion(Some(expansion))
                .tick_values(Some(vec![])),
        )
        .build()
        .unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let axis = &frame.axes()[&ScaleId::new(0)];
        let ResolvedScale::Linear(scale) = &axis.scale else {
            panic!()
        };
        let expected_range = pair(&case["numeric_range"]);
        for (actual, expected) in [scale.viewport().start(), scale.viewport().end()]
            .into_iter()
            .zip(expected_range)
        {
            assert!(
                (actual - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{case}"
            );
        }
        for (value, expected) in values.into_iter().zip(pair(&case["numeric_position"])) {
            let position = axis.map_value(&ScaleValue::Number(value)).unwrap().unwrap();
            let normalized =
                (position - scale.range().start()) / (scale.range().end() - scale.range().start());
            assert!(
                (normalized - expected).abs() < 1e-12,
                "{case}: {normalized} != {expected}"
            );
        }
    }
}

#[test]
fn fractional_time_views_preserve_source_quanta_and_overrides() {
    let spec = TimeScaleSpec {
        domain: vec![0, 1],
        ..Default::default()
    };
    let expansion = GgplotExpansion {
        mult: [-0.1; 2],
        add: [0.; 2],
    };
    for outside in [
        OutsidePolicy::Extend,
        OutsidePolicy::Omit,
        OutsidePolicy::Clamp,
    ] {
        let s = TimeAxisScale::resolve_reference(
            spec.clone(),
            Bounds::new(0., 100.).unwrap(),
            None,
            outside,
            Some(expansion),
        )
        .unwrap();
        assert_eq!(s.viewport(), TimeBounds { start: 0, end: 1 });
        assert_eq!(s.relative_viewport(), Bounds::new(0.1, 0.9).unwrap());
        assert_eq!(s.tick_bounds().unwrap(), None);
        assert!(
            s.ticks(
                CalendarTicks::Interval(CalendarInterval::new(CalendarUnit::SourceTick)),
                1
            )
            .unwrap()
            .is_empty()
        );
        match outside {
            OutsidePolicy::Extend => {
                assert_eq!(s.map(0).unwrap(), Some(-12.5));
                assert_eq!(s.map(1).unwrap(), Some(112.5));
            }
            OutsidePolicy::Omit => {
                assert_eq!(s.map(0).unwrap(), None);
                assert_eq!(s.map(1).unwrap(), None);
            }
            OutsidePolicy::Clamp => {
                assert_eq!(s.map(0).unwrap(), Some(0.));
                assert_eq!(s.map(1).unwrap(), Some(100.));
            }
        }
    }
    let origin = 9_007_199_254_740_993;
    let s = TimeAxisScale::resolve_reference(
        TimeScaleSpec {
            domain: vec![origin, origin + 10],
            unit: TimeUnit::Nanoseconds,
            ..Default::default()
        },
        Bounds::new(0., 100.).unwrap(),
        None,
        OutsidePolicy::Extend,
        Some(GgplotExpansion::default()),
    )
    .unwrap();
    assert_eq!(s.origin(), origin);
    assert!(s.tick_bounds().unwrap().unwrap().start < origin);
    assert!(s.tick_bounds().unwrap().unwrap().end > origin + 10);
    assert!((s.map(origin + 5).unwrap().unwrap() - 50.).abs() < 1e-12);
    assert_eq!(s.invert(50.).unwrap(), origin + 5);
    let overridden = TimeAxisScale::resolve_reference(
        TimeScaleSpec {
            domain: vec![0, 0],
            ..Default::default()
        },
        Bounds::new(0., 100.).unwrap(),
        Some(TimeBounds {
            start: -1000,
            end: 1000,
        }),
        OutsidePolicy::Extend,
        Some(GgplotExpansion::default()),
    )
    .unwrap();
    assert_eq!(
        overridden.relative_viewport(),
        Bounds::new(-1000., 1000.).unwrap()
    );
    assert_eq!(overridden.map(500).unwrap(), Some(75.));
    assert_eq!(overridden.invert(75.).unwrap(), 500);
    assert_eq!(overridden.domain(), TimeBounds { start: 0, end: 0 });
    assert!(
        TimeAxisScale::resolve_reference(
            TimeScaleSpec {
                domain: vec![i64::MAX, i64::MAX],
                unit: TimeUnit::Nanoseconds,
                ..Default::default()
            },
            Bounds::new(0., 100.).unwrap(),
            None,
            OutsidePolicy::Extend,
            Some(GgplotExpansion::default())
        )
        .is_err()
    );
}

#[test]
fn fractional_expansion_does_not_move_width_anchor_to_the_next_hour() {
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(vec![3_600_000, 7_200_000], TimeUnit::Milliseconds, "UTC"),
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
            .scale(scale_utc())
            .expansion(Some(GgplotExpansion {
                mult: [0.; 2],
                add: [0.0001, 0.],
            }))
            .tick_arguments(Some(GuideTickArguments {
                time_width: Some(CalendarInterval {
                    unit: CalendarUnit::Hour,
                    step: 7,
                }),
                ..Default::default()
            })),
    )
    .build()
    .unwrap();
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    // The true lower boundary is 00:59:59.9999. R anchors at 00:00;
    // the first seven-hour candidate is outside this two-hour view.
    assert!(frame.axes()[&ScaleId::new(0)].ticks.is_empty());
    let explicit = p
        .edit()
        .x_axis(
            x_axis()
                .scale(scale_utc().interval(UtcInterval::Seconds(1800)))
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.0001, 0.],
                })),
        )
        .build()
        .unwrap();
    let frame = layout(
        explicit.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(
        frame.axes()[&ScaleId::new(0)]
            .ticks
            .iter()
            .map(|t| match t.value {
                ScaleValue::Timestamp { value, .. } => value,
                _ => panic!(),
            })
            .collect::<Vec<_>>(),
        [3_600_000, 5_400_000, 7_200_000]
    );
}

#[test]
fn datetime_near_zero_policy_uses_epoch_magnitude_without_losing_source_identity() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-expansion.json"
    ))
    .unwrap();
    assert_eq!(f["fine_cases"].as_array().unwrap().len(), 3);
    for case in f["fine_cases"].as_array().unwrap() {
        let domain: Vec<i64> = case["limits_ns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        let s = TimeAxisScale::resolve_reference(
            TimeScaleSpec {
                domain: domain.clone(),
                unit: TimeUnit::Nanoseconds,
                ..Default::default()
            },
            Bounds::new(0., 100.).unwrap(),
            None,
            OutsidePolicy::Extend,
            Some(GgplotExpansion::default()),
        )
        .unwrap();
        assert_eq!(
            s.domain(),
            TimeBounds {
                start: domain[0],
                end: domain[1]
            }
        );
        assert!((s.relative_viewport().start() + 50_000_000.).abs() < 1e-8);
        assert!(
            (s.relative_viewport().end() - ((domain[1] - domain[0]) as f64 + 50_000_000.)).abs()
                < 1e-8
        );
        for (actual, expected) in [s.relative_viewport().start(), s.relative_viewport().end()]
            .into_iter()
            .zip(case["relative_range"].as_array().unwrap())
        {
            let expected = expected.as_f64().unwrap();
            assert!((actual / 1e9 - expected).abs() <= 2_f64.powi(-21), "{case}");
        }
        assert!(s.map(domain[1]).unwrap().unwrap() > s.map(domain[0]).unwrap().unwrap());
    }
}
