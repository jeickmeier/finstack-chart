//! FIX-GG04 elapsed-time break and label oracle.
use chart_core::scales::ggplot_breaks_duration;

#[test]
fn elapsed_breaks_match_pinned_hms_across_unit_boundaries() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/durations.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 45);
    for case in cases {
        let domain = [
            case["limits"][0].as_f64().unwrap(),
            case["limits"][1].as_f64().unwrap(),
        ];
        let actual = ggplot_breaks_duration(domain, case["count"].as_f64().unwrap(), 100).unwrap();
        let expected = case["breaks"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{case}");
        for (actual, expected) in actual.into_iter().zip(expected) {
            let expected = expected.as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{case}: {actual} != {expected}"
            );
        }
    }
    assert!(ggplot_breaks_duration([f64::NAN, 1.], 5., 100).is_err());
    assert!(ggplot_breaks_duration([0., 10.], 5., 2).is_err());
}

#[test]
fn elapsed_labels_match_hms_vector_padding_and_fractional_precision() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/durations.json"
    ))
    .unwrap();
    for case in fixture["labels"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["cases"].as_array().unwrap())
    {
        let values = case["exact"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        let labels = chart_core::typography::ggplot_duration_labels(&values, 10000).unwrap();
        assert_eq!(serde_json::json!(labels), case["labels"], "{case}");
    }
    assert!(chart_core::typography::ggplot_duration_labels(&[0., 1.], 1).is_err());
    assert!(chart_core::typography::ggplot_duration_labels(&[f64::NAN], 100).is_err());
}

#[test]
fn elapsed_axes_match_reference_panels_through_primary_authoring() {
    use chart_core::{
        Rect, ResourceId, Revision, ScaleId, layout::*, prelude::*, scales::GgplotExpansion,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/durations.json"
    ))
    .unwrap();
    let panels = fixture["panels"].as_array().unwrap();
    assert_eq!(panels.len(), 30);
    assert_eq!(fixture["widths"].as_array().unwrap().len(), 12);
    assert_eq!(fixture["named"].as_array().unwrap().len(), 12);
    assert_eq!(fixture["patterns"].as_array().unwrap().len(), 12);
    for case in panels
        .iter()
        .chain(fixture["widths"].as_array().unwrap())
        .chain(fixture["named"].as_array().unwrap())
        .chain(fixture["patterns"].as_array().unwrap())
    {
        let domain = [
            case["limits"][0].as_f64().unwrap(),
            case["limits"][1].as_f64().unwrap(),
        ];
        let p = plot(
            Data::columns()
                .column("x", domain)
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
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                }))
                .tick_format(case["pattern"].as_str().map(|pattern| {
                    GuideFormatter::Time(Box::new(chart_core::scales::TimeFormat {
                        pattern: Some(pattern.into()),
                        ..Default::default()
                    }))
                }))
                .tick_arguments(
                    case["seconds"]
                        .as_f64()
                        .map(|seconds| chart_core::scales::GuideTickArguments {
                            seconds: Some(seconds),
                            ..Default::default()
                        })
                        .or_else(|| {
                            case["unit"].as_str().map(|unit| {
                                use chart_core::scales::{
                                    CalendarInterval, CalendarUnit, GuideTickArguments, WeekStart,
                                };
                                let unit = match unit {
                                    "min" => CalendarUnit::Minute,
                                    "hour" => CalendarUnit::Hour,
                                    "day" => CalendarUnit::Day,
                                    "week" => CalendarUnit::Week(WeekStart::Monday),
                                    "month" => CalendarUnit::Month,
                                    "year" => CalendarUnit::Year,
                                    _ => panic!(),
                                };
                                GuideTickArguments {
                                    time_width: Some(CalendarInterval {
                                        unit,
                                        step: case["step"].as_u64().unwrap() as u32,
                                    }),
                                    ..Default::default()
                                }
                            })
                        }),
                )
                .expansion(Some(GgplotExpansion {
                    mult: [if case["expand"] == true { 0.05 } else { 0. }; 2],
                    add: [0.; 2],
                })),
        )
        .build()
        .unwrap();
        let json = p.to_json().unwrap();
        let restored = Plot::from_json(&json).unwrap();
        assert_eq!(restored.to_json().unwrap(), json);
        assert_eq!(p.definition().wire_version(), 17);
        let prepared = p.chart().unwrap().prepare().unwrap();
        let request = LayoutRequest::new(
            Rect::new(0., 0., 600., 300.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(prepared, &request, &Metrics).unwrap();
        let axis = &frame.axes()[&ScaleId::new(0)];
        let ResolvedScale::Linear(scale) = &axis.scale else {
            panic!()
        };
        for (actual, expected) in [scale.viewport().start(), scale.viewport().end()]
            .into_iter()
            .zip(
                case.get("viewport")
                    .unwrap_or(&case["limits"])
                    .as_array()
                    .unwrap(),
            )
        {
            let expected = expected.as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{case}: viewport {actual} != {expected}"
            );
        }
        assert_eq!(
            axis.ticks.len(),
            case["breaks"].as_array().unwrap().len(),
            "{case}"
        );
        for (i, tick) in axis.ticks.iter().enumerate() {
            let chart_core::composition::ScaleValue::Number(actual) = tick.value else {
                panic!()
            };
            let expected = case["breaks"][i].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{case}: {actual} != {expected}"
            );
            assert_eq!(tick.label, case["labels"][i].as_str().unwrap(), "{case}");
        }
    }
    let secondaries = fixture["secondary"].as_array().unwrap();
    assert_eq!(secondaries.len(), 9);
    for case in secondaries {
        let limits = [
            case["limits"][0].as_f64().unwrap(),
            case["limits"][1].as_f64().unwrap(),
        ];
        let p = plot(
            Data::columns()
                .column("x", limits)
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(x_axis().scale(scale_duration()))
        .axis(
            x_axis()
                .name("second")
                .side(AxisSide::Top)
                .secondary(
                    "x",
                    case["factor"].as_f64().unwrap(),
                    case["offset"].as_f64().unwrap(),
                )
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        let id = p.axis("second").unwrap().id();
        let frame = layout(
            p.chart().unwrap().prepare().unwrap(),
            &LayoutRequest::new(
                Rect::new(0., 0., 600., 300.).unwrap(),
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
        .unwrap();
        let axis = &frame.axes()[&id];
        assert_eq!(
            axis.ticks.len(),
            case["values"].as_array().unwrap().len(),
            "{case}"
        );
        let mut ticks = axis.ticks.iter().collect::<Vec<_>>();
        ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
        let ResolvedScale::Secondary { primary, .. } = &axis.scale else {
            panic!()
        };
        let ResolvedScale::Linear(scale) = &primary.scale else {
            panic!()
        };
        for (i, tick) in ticks.into_iter().enumerate() {
            let chart_core::composition::ScaleValue::Number(value) = tick.value else {
                panic!()
            };
            let expected = case["values"][i].as_f64().unwrap();
            assert!(
                (value - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{case}: {value} != {expected}"
            );
            assert_eq!(tick.label, case["labels"][i].as_str().unwrap(), "{case}");
            let position = (tick.position - scale.range().start())
                / (scale.range().end() - scale.range().start());
            assert!(
                (position - case["positions"][i].as_f64().unwrap()).abs() <= 1e-12,
                "{case}: {position}"
            );
        }
    }
}
