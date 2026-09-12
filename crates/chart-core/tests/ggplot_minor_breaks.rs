//! FIX-GG04: minor candidates share transformed numeric space and reference tails.
use chart_core::{interpolate::Number, scales::ggplot_minor_breaks};
#[test]
fn automatic_minor_candidates_match_reference_panels() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/minor-breaks.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["minor"] == "auto")
    {
        let result = &case["result"];
        assert!(result.get("error").is_none(), "{case}");
        let numbers = |field: &str| {
            result[field]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Number(v.as_f64().unwrap()))
                .collect::<Vec<_>>()
        };
        let major = numbers("major");
        let limits = numbers("range");
        let actual = ggplot_minor_breaks(
            &major,
            [limits[0], limits[1]],
            case["transform"] == "reverse",
            4096,
        )
        .unwrap();
        let expected = numbers("minor");
        assert_eq!(actual.len(), expected.len(), "{case}");
        for (a, b) in actual.iter().zip(expected) {
            assert!(
                (a.0 - b.0).abs() < 1e-12 * b.0.abs().max(1.),
                "{a:?} != {b:?}: {case}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 112);
}

#[test]
fn primary_numeric_minor_values_and_positions_match_reference() {
    use chart_core::{
        Rect, ResourceId, Revision,
        layout::{LayoutRequest, MinorBreaks, layout},
        prelude::*,
        services::{
            ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
        },
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/minor-breaks.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 448);
    for case in cases {
        let inputs = match case["population"].as_str().unwrap() {
            "constant" => [4., 4.],
            "negative" => [-4., 10.],
            "zero_wide" => [0., 10.],
            _ => [1., 10.],
        };
        let scale = match case["transform"].as_str().unwrap() {
            "sqrt" => scale_sqrt(),
            "log10" => scale_log(10.),
            "reverse" => scale_reverse(),
            _ => scale_linear(),
        };
        let mut axis = x_axis().scale(scale);
        if case["population"] == "zero_wide" {
            axis = axis.expansion(Some(chart_core::scales::GgplotExpansion {
                mult: [1.; 2],
                add: [0.; 2],
            }));
        }
        let major = match case["major"].as_str().unwrap() {
            "regular" => Some(vec![1., 2., 4., 10.]),
            "descending" => Some(vec![10., 4., 2., 1.]),
            "outside" => Some(vec![-5., 0., 2., 20.]),
            "one" => Some(vec![4.]),
            "empty" | "none" => Some(vec![]),
            _ => None,
        };
        if let Some(major) = major {
            axis = axis.tick_values(Some(
                major
                    .into_iter()
                    .map(chart_core::composition::ScaleValue::Number)
                    .collect(),
            ));
        }
        let minor = match case["minor"].as_str().unwrap() {
            "none" => MinorBreaks::Hidden,
            "empty" => MinorBreaks::Numeric(vec![]),
            "explicit" => MinorBreaks::Numeric(
                [
                    f64::NEG_INFINITY,
                    -2.,
                    0.,
                    0.5,
                    1.,
                    3.,
                    6.,
                    10.,
                    12.,
                    f64::INFINITY,
                    f64::NAN,
                ]
                .map(Number)
                .to_vec(),
            ),
            _ => MinorBreaks::Automatic,
        };
        axis = axis.minor_breaks(Some(minor));
        let data = Data::columns()
            .column("x", inputs)
            .column("y", [1., 2.])
            .build()
            .unwrap();
        let plot = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(axis)
            .build()
            .unwrap();
        let wire = plot.to_json().unwrap();
        assert_eq!(plot.definition().wire_version(), 19);
        let restored = Plot::from_json(&wire).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        let request = LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(
            restored.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let guide = frame
            .guide_snapshots()
            .into_iter()
            .find(|g| g.spec.side == chart_core::layout::AxisSide::Bottom)
            .unwrap();
        let expected = &case["result"];
        assert!(expected.get("error").is_none(), "{case}");
        let values = expected["minor_values"].as_array().unwrap();
        assert_eq!(guide.minor_ticks.len(), values.len(), "{case}");
        for (i, tick) in guide.minor_ticks.iter().enumerate() {
            if values[i].is_null() {
                assert!(tick.value.is_none());
            } else {
                let Some(chart_core::composition::ScaleValue::Number(actual)) = tick.value else {
                    panic!()
                };
                let wanted = values[i].as_f64().unwrap();
                assert!(
                    (actual - wanted).abs() < 1e-11 * wanted.abs().max(1.),
                    "{actual} != {wanted}: {case}"
                );
            }
            let normalized = (tick.position - frame.plot().unwrap().origin().x())
                / frame.plot().unwrap().width();
            let wanted = expected["minor_positions"][i].as_f64().unwrap();
            assert!(
                (normalized - wanted).abs() < 1e-11,
                "{normalized} != {wanted}: {case}"
            );
        }
    }
}

#[test]
fn minor_selection_is_bounded_and_keeps_explicit_order() {
    let major = [Number(0.), Number(1.)];
    assert!(ggplot_minor_breaks(&major, [Number(0.), Number(1.)], false, 2).is_err());
    assert_eq!(
        ggplot_minor_breaks(&major, [Number(0.), Number(1.)], false, 3).unwrap(),
        [Number(0.), Number(0.5), Number(1.)]
    );
    assert!(ggplot_minor_breaks(&major, [Number(f64::NAN), Number(1.)], false, 3).is_err());
    assert!(
        ggplot_minor_breaks(&[Number(0.); 4097], [Number(0.), Number(1.)], false, 4096).is_err()
    );
}

#[test]
fn temporal_minor_policies_match_reference_calendar_and_duration_alignment() {
    use chart_core::{
        Rect, ResourceId, Revision,
        data::TimeUnit,
        layout::{LayoutRequest, MinorBreaks, layout},
        prelude::*,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for (mode, source) in [
        (
            "width",
            include_str!("../../../fixtures/parity/ggplot2/minor-time-widths.json"),
        ),
        (
            "auto",
            include_str!("../../../fixtures/parity/ggplot2/minor-time-auto.json"),
        ),
        (
            "values",
            include_str!("../../../fixtures/parity/ggplot2/minor-time-values.json"),
        ),
    ] {
        let automatic = mode == "auto";
        let fixture: serde_json::Value = serde_json::from_str(source).unwrap();
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(
            cases.len(),
            match mode {
                "auto" => 35,
                "width" => 30,
                _ => 25,
            }
        );
        for case in cases {
            let input = case["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            let kind = case["kind"].as_str().unwrap();
            let (data, scale) = if kind == "duration" {
                (
                    Data::columns()
                        .column("x", input.clone())
                        .column("y", [1., 2.])
                        .build()
                        .unwrap(),
                    scale_duration(),
                )
            } else {
                let multiplier = if kind == "date" {
                    86_400_000_000.
                } else {
                    1_000_000.
                };
                let ticks = input
                    .iter()
                    .map(|v| (v * multiplier).round() as i64)
                    .collect::<Vec<_>>();
                (
                    Data::columns()
                        .column("x", timestamps(ticks, TimeUnit::Microseconds, "UTC"))
                        .column("y", [1., 2.])
                        .build()
                        .unwrap(),
                    if kind == "date" {
                        scale_date()
                    } else {
                        scale_utc()
                    },
                )
            };
            let policy = if automatic {
                MinorBreaks::Automatic
            } else if mode == "values" {
                MinorBreaks::Timestamps(
                    case["minor_values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| {
                            v.as_f64()
                                .map(|v| chart_core::composition::ScaleValue::Timestamp {
                                    value: (v * if kind == "date" {
                                        86_400_000_000.
                                    } else {
                                        1_000_000.
                                    })
                                    .round() as i64,
                                    unit: TimeUnit::Microseconds,
                                })
                        })
                        .collect(),
                )
            } else {
                MinorBreaks::TimeWidth(case["width"].as_str().unwrap().into())
            };
            let mut axis = x_axis()
                .scale(scale)
                .range(0., 100.)
                .minor_breaks(Some(policy));
            if automatic {
                let ratios: Option<&[f64]> = match case["major"].as_str().unwrap() {
                    "auto" => None,
                    "regular" => Some(&[0., 0.25, 0.75, 1.]),
                    "descending" => Some(&[1., 0.75, 0.25, 0.]),
                    "one" => Some(&[0.]),
                    "empty" => Some(&[]),
                    _ => unreachable!(),
                };
                if let Some(ratios) = ratios {
                    axis = axis.tick_values(Some(
                        ratios
                            .iter()
                            .map(|r| {
                                let value = input[0] + r * (input[1] - input[0]);
                                if kind == "duration" {
                                    chart_core::composition::ScaleValue::Number(value)
                                } else {
                                    chart_core::composition::ScaleValue::Timestamp {
                                        value: (value
                                            * if kind == "date" {
                                                86_400_000_000.
                                            } else {
                                                1_000_000.
                                            })
                                        .round()
                                            as i64,
                                        unit: TimeUnit::Microseconds,
                                    }
                                }
                            })
                            .collect(),
                    ));
                }
            }
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(axis)
                .build()
                .unwrap();
            let mut request = LayoutRequest::new(
                Rect::new(0., 0., 640., 360.).unwrap(),
                Units::LogicalPixels,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            request.max_ticks = 4096;
            let frame = layout(
                figure.chart().unwrap().prepare().unwrap(),
                &request,
                &Metrics,
            );
            if case["result"].get("error").is_some() {
                assert!(frame.is_err(), "{case}");
                continue;
            }
            let frame = frame.unwrap_or_else(|e| panic!("{e:?}: {case}"));
            let guide = frame
                .guide_snapshots()
                .into_iter()
                .find(|g| g.spec.side == chart_core::layout::AxisSide::Bottom)
                .unwrap();
            let expected = &case["result"];
            assert!(expected.get("error").is_none(), "{case}");
            let values = expected["values"].as_array().unwrap();
            assert_eq!(guide.minor_ticks.len(), values.len(), "{case}");
            for (i, tick) in guide.minor_ticks.iter().enumerate() {
                let expected = values[i].as_f64().unwrap();
                match tick
                    .value
                    .as_ref()
                    .unwrap_or_else(|| panic!("missing raw: {tick:?} {case}"))
                {
                    chart_core::composition::ScaleValue::Number(actual) => {
                        assert!((*actual - expected).abs() < 1e-10, "{case}")
                    }
                    chart_core::composition::ScaleValue::Timestamp { value, unit } => {
                        assert_eq!(*unit, TimeUnit::Microseconds);
                        assert_eq!(
                            *value,
                            (expected
                                * if kind == "date" {
                                    86_400_000_000.
                                } else {
                                    1_000_000.
                                })
                            .round() as i64,
                            "{case}"
                        );
                    }
                    _ => panic!(),
                }
                let expected = case["result"]["positions"][i].as_f64().unwrap();
                assert!(
                    (tick.position / 100. - expected).abs() < 1e-12,
                    "{} != {expected}: {case}",
                    tick.position / 100.
                );
            }
        }
    }
}

#[test]
fn temporal_minor_midpoints_preserve_exact_origins_and_promote_resolution() {
    use chart_core::{
        Rect, ResourceId, Revision, composition::ScaleValue, data::TimeUnit, layout::*, prelude::*,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for (unit, origin, promoted) in [
        (
            TimeUnit::Seconds,
            1_700_000_000_i64,
            Some((1_700_000_000_500_i64, TimeUnit::Milliseconds)),
        ),
        (
            TimeUnit::Milliseconds,
            1_700_000_000_000_i64,
            Some((1_700_000_000_000_500_i64, TimeUnit::Microseconds)),
        ),
        (
            TimeUnit::Microseconds,
            1_700_000_000_000_000_i64,
            Some((1_700_000_000_000_000_500_i64, TimeUnit::Nanoseconds)),
        ),
        (TimeUnit::Nanoseconds, 1_700_000_000_000_000_000_i64, None),
    ] {
        let data = Data::columns()
            .column("x", timestamps(vec![origin, origin + 1], unit, "UTC"))
            .column("y", [1., 2.])
            .build()
            .unwrap();
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(scale_utc())
                    .range(0., 100.)
                    .expansion(Some(chart_core::scales::GgplotExpansion {
                        mult: [0., 0.],
                        add: [0., 0.],
                    }))
                    .tick_values(Some(
                        [origin, origin + 1]
                            .into_iter()
                            .map(|value| ScaleValue::Timestamp { value, unit })
                            .collect(),
                    ))
                    .tick_format(Some(GuideFormatter::GgplotTime(Box::new(
                        chart_core::scales::GgplotTimeFormat {
                            pattern: "%Y".into(),
                            locale: None,
                        },
                    )))),
            )
            .build()
            .unwrap();
        let request = LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(
            figure.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap();
        let guide = frame
            .guide_snapshots()
            .into_iter()
            .find(|g| g.spec.side == AxisSide::Bottom)
            .unwrap();
        assert_eq!(guide.minor_ticks.len(), 3);
        let mid = &guide.minor_ticks[1];
        assert_eq!(mid.position, 50.);
        assert_eq!(
            mid.value,
            promoted.map(|(value, unit)| ScaleValue::Timestamp { value, unit })
        );
    }
}

#[test]
fn discrete_numeric_minors_use_reference_category_coordinates() {
    use chart_core::{
        Rect, ResourceId, Revision, composition::ScaleValue, layout::*, prelude::*,
        scales::GgplotExpansion, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/minor-discrete.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 36);
    for case in cases {
        for family in [0, 1, 2] {
            for reversed in [false, true] {
                let n = case["count"].as_u64().unwrap() as usize;
                let labels: Vec<String> = (0..n)
                    .map(|i| char::from(b'a' + i as u8).to_string())
                    .collect();
                let data = Data::columns()
                    .column("x", categorical(labels))
                    .column("y", vec![1.; n])
                    .build()
                    .unwrap();
                let mut axis = x_axis().range(
                    if reversed { 100. } else { 0. },
                    if reversed { 0. } else { 100. },
                );
                if family == 1 {
                    axis = axis.scale(scale_band());
                } else if family == 2 {
                    axis = axis.scale(scale_point());
                }
                axis = axis
                    .expansion(match case["expansion"].as_str().unwrap() {
                        "default" => None,
                        "none" => Some(GgplotExpansion {
                            mult: [0.; 2],
                            add: [0.; 2],
                        }),
                        "wide" => Some(GgplotExpansion {
                            mult: [0.2, 0.5],
                            add: [1., 2.],
                        }),
                        _ => unreachable!(),
                    })
                    .minor_breaks(Some(match case["minor"].as_str().unwrap() {
                        "auto" => MinorBreaks::Automatic,
                        "hidden" => MinorBreaks::Hidden,
                        "empty" => MinorBreaks::Numeric(vec![]),
                        "explicit" => MinorBreaks::Numeric(
                            [
                                f64::NEG_INFINITY,
                                -1.,
                                0.,
                                0.5,
                                1.,
                                1.5,
                                2.5,
                                3.5,
                                4.,
                                f64::INFINITY,
                                f64::NAN,
                            ]
                            .into_iter()
                            .map(Number)
                            .collect(),
                        ),
                        _ => unreachable!(),
                    }));
                let figure = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points())
                    .x_axis(axis)
                    .build()
                    .unwrap();
                let wire = figure.to_json().unwrap();
                assert_eq!(figure.definition().wire_version(), 19);
                let restored = Plot::from_json(&wire).unwrap();
                assert_eq!(restored.to_json().unwrap(), wire);
                let request = LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.).unwrap(),
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                );
                let frame = layout(
                    restored.chart().unwrap().prepare().unwrap(),
                    &request,
                    &Metrics,
                )
                .unwrap_or_else(|e| panic!("{e:?} {case}"));
                let guide = frame
                    .guide_snapshots()
                    .into_iter()
                    .find(|g| g.spec.side == AxisSide::Bottom)
                    .unwrap();
                let expected = &case["result"];
                let values = expected["values"].as_array().unwrap();
                assert_eq!(guide.minor_ticks.len(), values.len(), "{case}");
                for (i, tick) in guide.minor_ticks.iter().enumerate() {
                    assert_eq!(
                        tick.value,
                        Some(ScaleValue::Number(values[i].as_f64().unwrap()))
                    );
                    let position = expected["positions"][i].as_f64().unwrap();
                    let position = if reversed { 1. - position } else { position };
                    assert!(
                        (tick.position / 100. - position).abs() < 1e-12,
                        "{tick:?} {case}"
                    );
                }
            }
        }
    }
}

#[test]
fn timestamp_minor_inputs_enforce_types_units_and_budgets() {
    use chart_core::{
        Rect, ResourceId, Revision, composition::ScaleValue, data::TimeUnit, layout::*, prelude::*,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for values in [
        vec![Some(ScaleValue::Number(0.))],
        vec![Some(ScaleValue::Timestamp {
            value: 0,
            unit: TimeUnit::Milliseconds,
        })],
        vec![None; 257],
    ] {
        let data = Data::columns()
            .column("x", timestamps(vec![0, 86400], TimeUnit::Seconds, "UTC"))
            .column("y", [1., 2.])
            .build()
            .unwrap();
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(scale_utc())
                    .minor_breaks(Some(MinorBreaks::Timestamps(values))),
            )
            .build()
            .unwrap();
        let request = LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        assert!(
            layout(
                figure.chart().unwrap().prepare().unwrap(),
                &request,
                &Metrics
            )
            .is_err()
        );
    }
}
