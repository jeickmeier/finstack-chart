//! FIX-GG04: Date policies reuse exact timestamp projection with day units.
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
#[test]
fn date_expansion_breaks_labels_and_positions_match_r() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/date-scales.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 81);
    for case in cases {
        let pair = |v: &serde_json::Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
        let days = pair(&case["limits_days"]);
        let expansion = GgplotExpansion {
            mult: pair(&case["policy"]["mult"]),
            add: pair(&case["policy"]["add"]),
        };
        for (unit, factor) in [
            (TimeUnit::Seconds, 86400.),
            (TimeUnit::Milliseconds, 86_400_000.),
        ] {
            let domain = days.map(|v| (v * factor).round() as i64);
            let p = plot(
                Data::columns()
                    .column("x", timestamps(domain.to_vec(), unit, "UTC"))
                    .column("y", [0., 1.])
                    .build()
                    .unwrap(),
            )
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(scale_date().time_domain(domain[0], domain[1]))
                    .expansion(Some(expansion))
                    .tick_arguments(Some(GuideTickArguments {
                        count: case["n"].as_f64(),
                        ..Default::default()
                    }))
                    .guide_geometry(Some(GuideGeometry {
                        labels: Some(GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            )
            .build()
            .unwrap();
            assert_eq!(p.definition().wire_version(), 17);
            let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
            let frame =
                layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
            let axis = &frame.axes()[&ScaleId::new(0)];
            let ResolvedScale::Calendar(scale) = &axis.scale else {
                panic!()
            };
            let actual_view = [
                scale.relative_viewport().start(),
                scale.relative_viewport().end(),
            ];
            for (actual, expected) in actual_view.into_iter().zip(pair(&case["range"])) {
                assert!(
                    (actual / factor + days[0] - expected).abs() < 1e-10,
                    "{case}: {actual}"
                );
            }
            let expected = case["breaks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (v.as_f64().unwrap() * factor).round() as i64)
                .collect::<Vec<_>>();
            assert_eq!(
                axis.ticks
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
                axis.ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>(),
                case["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>(),
                "{case}"
            );
            for (value, expected) in domain.into_iter().zip(pair(&case["position"])) {
                let p = axis
                    .map_value(&ScaleValue::Timestamp { value, unit })
                    .unwrap()
                    .unwrap();
                let actual =
                    (p - scale.range().start()) / (scale.range().end() - scale.range().start());
                assert!((actual - expected).abs() < 1e-10, "{case}: {actual}");
            }
        }
    }
}

#[test]
fn date_factory_preserves_source_and_uses_date_labels_for_custom_ticks() {
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(vec![21_600, 237_600], TimeUnit::Seconds, "UTC"),
            )
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_date())
            .expansion(Some(GgplotExpansion {
                mult: [0.; 2],
                add: [0.; 2],
            }))
            .tick_values(Some(vec![ScaleValue::Timestamp {
                value: 21_600,
                unit: TimeUnit::Seconds,
            }]))
            .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                pattern: Some("%Y-%m-%d %H:%M".into()),
                ..Default::default()
            })))),
    )
    .build()
    .unwrap();
    assert_eq!(p.definition().wire_version(), 17);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 17,
        definition: p.definition().clone(),
    };
    envelope.validate().unwrap();
    envelope.version = 16;
    assert!(envelope.validate().is_err());
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let original = &frame.axes()[&ScaleId::new(0)].ticks;
    assert_eq!(
        original[0].value,
        ScaleValue::Timestamp {
            value: 21_600,
            unit: TimeUnit::Seconds
        }
    );
    assert_eq!(original[0].label, "1970-01-01 00:00");
    let width = p
        .edit()
        .x_axis(
            x_axis()
                .scale(scale_date())
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .tick_arguments(Some(GuideTickArguments {
                    time_width: Some(CalendarInterval {
                        unit: CalendarUnit::Day,
                        step: 2,
                    }),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
    let frame = layout(
        width.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(frame.axes()[&ScaleId::new(0)].ticks.len(), 1);
    assert_eq!(
        frame.axes()[&ScaleId::new(0)].ticks[0].value,
        ScaleValue::Timestamp {
            value: 172_800,
            unit: TimeUnit::Seconds
        }
    );
    assert_eq!(frame.axes()[&ScaleId::new(0)].ticks[0].label, "1970-01-03");
    assert_eq!(
        &layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics)
            .unwrap()
            .axes()[&ScaleId::new(0)]
            .ticks,
        original
    );
    for arguments in [
        GuideTickArguments {
            seconds: Some(1.),
            ..Default::default()
        },
        GuideTickArguments {
            time_width: Some(CalendarInterval::new(CalendarUnit::Hour)),
            ..Default::default()
        },
    ] {
        let invalid = p
            .edit()
            .x_axis(
                x_axis()
                    .scale(scale_date())
                    .tick_arguments(Some(arguments))
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
    let numeric = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().scale(scale_date()))
    .build()
    .unwrap();
    assert_eq!(
        layout(
            numeric.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::SchemaConflict
    );
}

#[test]
fn date_widths_match_r_and_keep_default_date_formatting() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/date-widths.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let domain = case["limits_days"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v.as_f64().unwrap() * 86400.).round() as i64)
            .collect::<Vec<_>>();
        let unit = match case["width"].as_str().unwrap() {
            "2 days" => CalendarUnit::Day,
            "2 weeks" => CalendarUnit::Week(WeekStart::Monday),
            "2 months" => CalendarUnit::Month,
            "2 years" => CalendarUnit::Year,
            _ => panic!(),
        };
        let p = plot(
            Data::columns()
                .column("x", timestamps(domain, TimeUnit::Seconds, "UTC"))
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_date())
                .expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }))
                .tick_arguments(Some(GuideTickArguments {
                    time_width: Some(CalendarInterval { unit, step: 2 }),
                    ..Default::default()
                }))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
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
            case["breaks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (v.as_f64().unwrap() * 86400.).round() as i64)
                .collect::<Vec<_>>(),
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
