//! FIX-GG04: reference width strings use the existing time and elapsed selectors.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, data::TimeUnit,
    layout::*, prelude::*, scales::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 1000., 300.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.max_ticks = 2048;
    r
}
fn frame(case: &serde_json::Value) -> ChartResult<LaidOutChart> {
    let kind = case["kind"].as_str().unwrap();
    let limits = [
        case["limits"][0].as_f64().unwrap(),
        case["limits"][1].as_f64().unwrap(),
    ];
    let mut axis = x_axis()
        .expansion(Some(GgplotExpansion {
            mult: [0.; 2],
            add: [0.; 2],
        }))
        .tick_arguments(Some(GuideTickArguments {
            width: Some(case["width"].as_str().unwrap().into()),
            ..Default::default()
        }))
        .guide_geometry(Some(GuideGeometry {
            labels: Some(GuideLabelPolicy::Preserve),
            ..Default::default()
        }));
    let data = if kind == "duration" {
        axis = axis.scale(scale_duration());
        Data::columns()
            .column("x", limits)
            .column("y", [0., 1.])
            .build()?
    } else {
        let factor = if kind == "date" { 86_400_000. } else { 1000. };
        let domain = limits.map(|v| (v * factor).round() as i64);
        axis = axis
            .scale(if kind == "date" {
                scale_date().time_domain(domain[0], domain[1])
            } else {
                scale_calendar(TimeScaleSpec {
                    domain: domain.to_vec(),
                    zone: CalendarZone::Utc,
                    ..Default::default()
                })
            })
            .tick_format(Some(GuideFormatter::Time(Box::new(TimeFormat {
                pattern: Some(
                    if kind == "date" {
                        "%Y-%m-%d"
                    } else {
                        "%Y-%m-%d %H:%M:%S"
                    }
                    .into(),
                ),
                ..Default::default()
            }))));
        Data::columns()
            .column(
                "x",
                timestamps(domain.to_vec(), TimeUnit::Milliseconds, "UTC"),
            )
            .column("y", [0., 1.])
            .build()?
    };
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(axis)
        .build()?;
    let wire = p.to_json()?;
    let p = Plot::from_json(&wire)?;
    assert_eq!(p.to_json()?, wire);
    layout(p.chart()?.prepare()?, &request(), &Metrics)
}
#[test]
fn width_strings_match_135_reference_panels_and_rejections() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-width-strings.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 135);
    for case in cases {
        let result = frame(case);
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let frame = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let axis = &frame.axes()[&ScaleId::new(0)];
        let values = &expected["breaks"].as_array().unwrap();
        assert_eq!(axis.ticks.len(), values.len(), "{case}");
        for (tick, expected) in axis.ticks.iter().zip(values.iter()) {
            let expected = expected.as_f64().unwrap();
            match tick.value {
                ScaleValue::Number(actual) => assert!(
                    (actual - expected).abs() <= 1e-10 * expected.abs().max(1.),
                    "{case}: {actual} != {expected}"
                ),
                ScaleValue::Timestamp {
                    value,
                    unit: TimeUnit::Milliseconds,
                } => assert_eq!(
                    value,
                    (expected
                        * if case["kind"] == "date" {
                            86_400_000.
                        } else {
                            1000.
                        })
                    .round() as i64,
                    "{case}"
                ),
                _ => panic!("wrong tick type: {case}"),
            }
        }
        assert_eq!(
            axis.ticks
                .iter()
                .map(|v| v.label.as_str())
                .collect::<Vec<_>>(),
            expected["labels"]
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
fn string_widths_preserve_nanosecond_epochs_budgets_and_explicit_selection() {
    use chart_core::DiagnosticCode;
    let base = 9_007_199_254_740_993_i64;
    let domain = vec![base, base + 240_000_000_000];
    let axis = |width: &str, empty: bool| {
        x_axis()
            .scale(scale_calendar(TimeScaleSpec {
                domain: domain.clone(),
                unit: TimeUnit::Nanoseconds,
                zone: CalendarZone::Utc,
                ..Default::default()
            }))
            .expansion(Some(GgplotExpansion {
                mult: [0.; 2],
                add: [0.; 2],
            }))
            .tick_arguments(Some(GuideTickArguments {
                width: Some(width.into()),
                ..Default::default()
            }))
            .tick_values(empty.then(Vec::new))
            .guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }))
    };
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(domain.clone(), TimeUnit::Nanoseconds, "UTC"),
            )
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(axis("1.5 min", false))
    .y_axis(y_axis().tick_values(Some(vec![])))
    .build()
    .unwrap();
    let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
    assert_eq!(p.definition().wire_version(), 17);
    let prepared = p.chart().unwrap().prepare().unwrap();
    let frame = layout(prepared.clone(), &request(), &Metrics).unwrap();
    let a = &frame.axes()[&ScaleId::new(0)];
    let ResolvedScale::Calendar(time) = &a.scale else {
        panic!()
    };
    assert_eq!(time.origin(), base);
    assert_eq!(
        a.ticks
            .iter()
            .map(|v| match v.value {
                ScaleValue::Timestamp {
                    value,
                    unit: TimeUnit::Nanoseconds,
                } => value,
                _ => panic!(),
            })
            .collect::<Vec<_>>(),
        [
            9_007_200_000_000_000,
            9_007_260_000_000_000,
            9_007_320_000_000_000,
            9_007_380_000_000_000
        ]
    );
    for tick in &a.ticks {
        assert_eq!(a.map_value(&tick.value).unwrap().unwrap(), tick.position);
    }
    let mut bounded = request();
    bounded.max_ticks = 2;
    bounded.target_ticks = 2;
    assert_eq!(
        layout(prepared.clone(), &bounded, &Metrics)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let too_fine = p
        .edit()
        .x_axis(axis("0.0000000001 sec", false))
        .build()
        .unwrap();
    assert_eq!(
        layout(
            too_fine.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::PrecisionLoss
    );
    let empty = p
        .edit()
        .x_axis(axis("0.0000000001 sec", true))
        .build()
        .unwrap();
    let empty_prepared = empty.chart().unwrap().prepare().unwrap();
    assert_eq!(
        empty_prepared.layers()[0].marks(),
        prepared.layers()[0].marks()
    );
    assert!(
        layout(empty_prepared, &request(), &Metrics).unwrap().axes()[&ScaleId::new(0)]
            .ticks
            .is_empty()
    );
    let args = GuideTickArguments {
        width: Some("1 day".into()),
        count: Some(5.),
        ..Default::default()
    };
    assert_eq!(
        args.validate(100).unwrap_err().code,
        DiagnosticCode::Validation
    );
    let args = GuideTickArguments {
        width: Some("1 day".into()),
        ..Default::default()
    };
    assert_eq!(
        args.validate(4).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
}
