//! FIX-GG04: explicit R date labels over the shared calendar renderer.
use chart_core::{
    data::TimeUnit,
    scales::{Calendar, CalendarZone, GgplotTimeFormat, TimeFormat},
};

#[test]
fn r_c_locale_time_patterns_match_independent_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-formats.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let spec = GgplotTimeFormat {
            pattern: case["pattern"].as_str().unwrap().into(),
            locale: None,
        };
        let formatter = spec
            .prepare(Calendar::new(CalendarZone::Utc).unwrap())
            .unwrap();
        assert_eq!(
            formatter
                .format(
                    case["microseconds"].as_str().unwrap().parse().unwrap(),
                    TimeUnit::Microseconds
                )
                .unwrap(),
            case["label"].as_str().unwrap(),
            "{case}"
        );
    }
}

#[test]
fn r_patterns_are_explicit_and_bounded() {
    let calendar = Calendar::new(CalendarZone::Utc).unwrap();
    let prepare = |pattern: &str| {
        GgplotTimeFormat {
            pattern: pattern.into(),
            locale: None,
        }
        .prepare(calendar.clone())
    };
    assert!(prepare("%s").is_err());
    assert!(prepare("a\0b").is_err());
    assert!(prepare(&"a".repeat(4097)).is_err());
    let r = prepare("%Z|%q|%-d|abc%").unwrap();
    assert_eq!(r.format(0, TimeUnit::Seconds).unwrap(), "UTC|q|-d|abc%");
    let d3 = TimeFormat {
        pattern: Some("%Z|%q|%-d|abc%".into()),
        ..TimeFormat::default()
    }
    .prepare(calendar)
    .unwrap();
    assert_eq!(d3.format(0, TimeUnit::Seconds).unwrap(), "+0000|1|1|abc");
}

#[test]
fn explicit_r_patterns_reach_primary_axes_and_versioned_json() {
    use chart_core::{
        Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*, prelude::*,
        scales::*, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let values = [1_704_164_645_100_000, 1_704_164_645_123_456];
    let ticks: Vec<_> = values
        .iter()
        .map(|value| ScaleValue::Timestamp {
            value: *value,
            unit: TimeUnit::Microseconds,
        })
        .collect();
    for side in [
        AxisSide::Bottom,
        AxisSide::Top,
        AxisSide::Left,
        AxisSide::Right,
    ] {
        let horizontal = matches!(side, AxisSide::Bottom | AxisSide::Top);
        let axis = (if horizontal { x_axis() } else { y_axis() })
            .side(side)
            .scale(scale_calendar(TimeScaleSpec {
                domain: values.to_vec(),
                unit: TimeUnit::Microseconds,
                ..Default::default()
            }))
            .tick_values(Some(ticks.clone()))
            .tick_format(Some(GuideFormatter::GgplotTime(Box::new(
                GgplotTimeFormat {
                    pattern: "%F\n%OS6 %Z".into(),
                    locale: None,
                },
            ))))
            .guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }));
        let builder = plot(
            Data::columns()
                .column(
                    "time",
                    timestamps(values.to_vec(), TimeUnit::Microseconds, "UTC"),
                )
                .column("value", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(
            aes()
                .x(if horizontal { "time" } else { "value" })
                .y(if horizontal { "value" } else { "time" }),
        )
        .layer(points());
        let p = (if horizontal {
            builder.x_axis(axis)
        } else {
            builder.y_axis(axis)
        })
        .build()
        .unwrap();
        assert_eq!(p.definition().wire_version(), 17);
        let wire = p.to_json().unwrap();
        let p = Plot::from_json(&wire).unwrap();
        assert_eq!(p.to_json().unwrap(), wire);
        let request = LayoutRequest::new(
            Rect::new(0., 0., 1000., 300.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap();
        assert!(frame.scene().items().iter().all(|item| !matches!(&item.primitive, chart_core::scene::Primitive::Text {text, ..} if text.contains('\n'))));
        let lines: Vec<_> = frame
            .scene()
            .items()
            .iter()
            .filter_map(|item| match &item.primitive {
                chart_core::scene::Primitive::Text { text, origin, .. }
                    if text == "2024-01-02" || text.ends_with(" UTC") =>
                {
                    Some((text, origin))
                }
                _ => None,
            })
            .collect();
        assert_eq!(lines.len(), 4);
        let bounds = frame.plot().unwrap();
        for (text, origin) in lines {
            assert!(
                match side {
                    AxisSide::Bottom => origin.y() > bounds.max_y(),
                    AxisSide::Top => origin.y() < bounds.origin().y(),
                    AxisSide::Left => origin.x() + text.len() as f64 * 5. < bounds.origin().x(),
                    AxisSide::Right => origin.x() > bounds.max_x(),
                },
                "{side:?}: {text} at {origin:?}; {bounds:?}"
            );
        }
        let actual = &frame.axes()[&ScaleId::new(if horizontal { 0 } else { 1 })].ticks;
        assert_eq!(
            actual.iter().map(|t| t.value.clone()).collect::<Vec<_>>(),
            ticks
        );
        assert_eq!(
            actual.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
            ["2024-01-02\n05.099999 UTC", "2024-01-02\n05.123456 UTC"]
        );
    }
}

#[test]
fn explicit_resources_control_r_locale_and_local_offsets() {
    use chart_core::{
        Revision,
        scales::{TimeBounds, TimeLocale, TimeZoneRules},
    };
    let calendar = Calendar::new(CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
        version: 1,
        zone: "supplied-fixed".into(),
        revision: Revision::INITIAL,
        tzdata: "fixture-offset-only".into(),
        coverage: TimeBounds {
            start: -86_400_000,
            end: 86_400_000,
        },
        initial_offset_seconds: -18000,
        transitions: vec![],
    })))
    .unwrap();
    let spec = |pattern: &str| GgplotTimeFormat {
        pattern: pattern.into(),
        locale: None,
    };
    assert!(spec("%Z").prepare(calendar.clone()).is_err());
    assert!(spec("%+").prepare(calendar.clone()).is_err());
    assert_eq!(
        spec("%F %T %z")
            .prepare(calendar)
            .unwrap()
            .format(0, TimeUnit::Seconds)
            .unwrap(),
        "1969-12-31 19:00:00 -0500"
    );
    let narrow = Calendar::new(CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
        version: 1,
        zone: "subsecond-window".into(),
        revision: Revision::INITIAL,
        tzdata: "supplied-window".into(),
        coverage: TimeBounds {
            start: 100,
            end: 999,
        },
        initial_offset_seconds: -18000,
        transitions: vec![chart_core::scales::TimeZoneTransition {
            at_millis: 500,
            offset_seconds: -14400,
        }],
    })))
    .unwrap();
    let formatter = spec("%H:%M:%S %z").prepare(narrow).unwrap();
    assert_eq!(
        formatter.format(499, TimeUnit::Milliseconds).unwrap(),
        "19:00:00 -0500"
    );
    assert_eq!(
        formatter.format(500, TimeUnit::Milliseconds).unwrap(),
        "20:00:00 -0400"
    );
    let mut locale = TimeLocale::default();
    locale.short_months[0] = "JanuaryCustom".into();
    let spec = GgplotTimeFormat {
        pattern: "%b".into(),
        locale: Some(locale),
    };
    let prepared = spec
        .prepare(Calendar::new(CalendarZone::Utc).unwrap())
        .unwrap();
    assert_eq!(prepared.ggplot_spec(), Some(&spec));
    assert_eq!(
        prepared.format(0, TimeUnit::Seconds).unwrap(),
        "JanuaryCustom"
    );
}

#[test]
fn r_duration_patterns_match_primary_hms_panels_without_integer_resampling() {
    use chart_core::{
        Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*, prelude::*,
        scales::GgplotExpansion, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/duration-formats.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 40);
    let request = LayoutRequest::new(
        Rect::new(0., 0., 1600., 300.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for case in fixture["cases"].as_array().unwrap() {
        let seconds: Vec<f64> = case["seconds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        let p = plot(
            Data::columns()
                .column("x", seconds.clone())
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_duration())
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .tick_values(Some(
                    seconds.iter().map(|v| ScaleValue::Number(*v)).collect(),
                ))
                .tick_format(Some(GuideFormatter::GgplotTime(Box::new(
                    GgplotTimeFormat {
                        pattern: case["pattern"].as_str().unwrap().into(),
                        locale: None,
                    },
                ))))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        let wire = p.to_json().unwrap();
        let restored = Plot::from_json(&wire).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        let frame = layout(
            restored.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap();
        let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
        assert_eq!(
            serde_json::json!(ticks.iter().map(|t| &t.label).collect::<Vec<_>>()),
            case["labels"],
            "{case}"
        );
        assert_eq!(
            ticks.iter().map(|t| t.value.clone()).collect::<Vec<_>>(),
            seconds
                .iter()
                .map(|v| ScaleValue::Number(*v))
                .collect::<Vec<_>>()
        );
    }
}
