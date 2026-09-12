//! FIX-GG04: alternate-unit selection preserves the primary positional mapping.
use chart_core::{
    DiagnosticCode, Rect, ResourceId, Revision, composition::ScaleValue, layout::*, prelude::*,
    scales::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 700., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 2e-12 * b.abs().max(1.), "{a} != {b}");
}
#[test]
fn affine_secondary_breaks_labels_and_projection_match_36_r_panels() {
    check(
        include_str!("../../../fixtures/parity/ggplot2/secondary-affine.json"),
        36,
    );
}
#[test]
fn transformed_secondary_guides_match_16_r_panels_and_exact_inverse_formulas() {
    check(
        include_str!("../../../fixtures/parity/ggplot2/secondary-transform.json"),
        16,
    );
}
fn transform(kind: &str) -> NumericScaleSpec {
    match kind {
        "cube" | "square" => NumericScaleSpec::d3(NumericFamily::Pow {
            exponent: if kind == "cube" { 3. } else { 2. },
        }),
        "piecewise" | "decreasing" => NumericScaleSpec {
            domain: [0., 5., 20.].map(Into::into).to_vec(),
            range: if kind == "piecewise" {
                [0., 2., 10.]
            } else {
                [10., 2., 0.]
            }
            .map(Into::into)
            .to_vec(),
            ..NumericScaleSpec::d3(NumericFamily::Linear)
        },
        _ => panic!(),
    }
}
fn check(input: &str, count: usize) {
    let f: serde_json::Value = serde_json::from_str(input).unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), count);
    for case in cases {
        let list = |name: &str| {
            case[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>()
        };
        let limits = list("limits");
        let kind = case["kind"].as_str();
        let factor = case["factor"].as_f64().unwrap_or(1.);
        let offset = case["offset"].as_f64().unwrap_or(0.);
        let scale = match case["family"].as_str().unwrap() {
            "linear" => scale_linear(),
            "log" => scale_log(10.),
            "reverse" => scale_reverse(),
            "sqrt" => scale_sqrt(),
            _ => panic!(),
        }
        .domain(limits[0], limits[1]);
        let primary = x_axis()
            .scale(scale)
            .expansion(kind.map(|_| GgplotExpansion {
                mult: [0.; 2],
                add: [0.; 2],
            }));
        let secondary = x_axis().name("second").side(AxisSide::Top);
        let secondary = if let Some(kind) = kind {
            secondary.secondary_transform("x", transform(kind))
        } else {
            secondary.secondary("x", factor, offset)
        };
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
        .x_axis(primary)
        .axis(secondary.guide_geometry(Some(GuideGeometry {
            labels: Some(GuideLabelPolicy::Preserve),
            ..Default::default()
        })))
        .build()
        .unwrap();
        let wire = p.to_json().unwrap();
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
        let id = p.axis("second").unwrap().id();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let frame = layout(prepared, &request(), &Metrics);
        if case.get("error").is_some() {
            assert_eq!(
                frame.unwrap_err().code,
                DiagnosticCode::NumericalDomain,
                "{case}"
            );
            continue;
        }
        let frame = frame.unwrap();
        let axis = &frame.axes()[&id];
        let ResolvedScale::Secondary { view, primary, .. } = &axis.scale else {
            panic!()
        };
        near(view.minimum(), list("range")[0]);
        near(view.maximum(), list("range")[1]);
        assert_eq!(axis.ticks.len(), list("values").len(), "{case}");
        let mut ticks: Vec<_> = axis.ticks.iter().collect();
        ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
        for (i, tick) in ticks.into_iter().enumerate() {
            let ScaleValue::Number(value) = tick.value else {
                panic!()
            };
            near(value, list("values")[i]);
            assert_eq!(tick.label, case["labels"][i].as_str().unwrap(), "{case}");
            let original = match kind {
                Some("cube") => value.cbrt(),
                Some("square") => value.sqrt(),
                Some("piecewise") => {
                    if value <= 2. {
                        value / 0.4
                    } else {
                        5. + (value - 2.) * 15. / 8.
                    }
                }
                Some("decreasing") => {
                    if value >= 2. {
                        (10. - value) / 1.6
                    } else {
                        5. + (2. - value) * 7.5
                    }
                }
                None => (value - offset) / factor,
                _ => panic!(),
            };
            near(
                axis.map_value(&tick.value).unwrap().unwrap(),
                primary
                    .map_value(&ScaleValue::Number(original))
                    .unwrap()
                    .unwrap(),
            );
            near(
                tick.position,
                axis.guide_value_position(&tick.value).unwrap().unwrap(),
            );
            let normalized = (tick.position - frame.plot().unwrap().origin().x())
                / frame.plot().unwrap().width();
            near(normalized, list("positions")[i]);
        }
        assert!(!axis.capabilities().numeric_inverse);
        assert!(axis.invert_value(10.).is_err());
    }
}

fn custom_plot(spec: NumericScaleSpec) -> chart_core::ChartResult<Plot> {
    plot(
        Data::columns()
            .column("x", [2., 4.])
            .column("y", [0., 1.])
            .build()?,
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().scale(scale_linear().domain(2., 4.)))
    .axis(
        x_axis()
            .name("second")
            .side(AxisSide::Top)
            .secondary_transform("x", spec),
    )
    .build()
}
#[test]
fn secondary_transform_validation_preserves_invertibility_and_source_precision() {
    let valid = transform("cube");
    let mut invalid = Vec::new();
    for field in 0..8 {
        let mut spec = valid.clone();
        match field {
            0 => spec.clamp = true,
            1 => spec.round = true,
            2 => spec.domain[1] = spec.domain[0],
            3 => spec.range[1] = spec.range[0],
            4 => spec.range.push(2.0.into()),
            5 => spec.family = NumericFamily::Pow { exponent: 0. },
            6 => spec.family = NumericFamily::Pow { exponent: -1. },
            7 => spec.range[0] = f64::INFINITY.into(),
            _ => unreachable!(),
        }
        invalid.push(spec);
    }
    invalid.push(NumericScaleSpec {
        domain: [0., 1., 2.].map(Into::into).to_vec(),
        range: [0., 2., 1.].map(Into::into).to_vec(),
        ..NumericScaleSpec::d3(NumericFamily::Linear)
    });
    for spec in invalid {
        let result =
            custom_plot(spec).and_then(|p| layout(p.chart()?.prepare()?, &request(), &Metrics));
        assert_eq!(result.unwrap_err().code, DiagnosticCode::Validation);
    }
    let p = custom_plot(valid).unwrap();
    let tiny = p
        .edit()
        .x_axis(x_axis().scale(scale_linear().domain(1e-200, 2e-200)))
        .build()
        .unwrap();
    assert_eq!(
        layout(
            tiny.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .unwrap_err()
        .code,
        DiagnosticCode::PrecisionLoss
    );
    let id = p.axis("second").unwrap().id();
    let mut definition = p.definition().clone();
    // Neither profile provenance nor the primary scales should be the reason for v17.
    definition.semantics = None;
    assert_eq!(definition.wire_version(), 17);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 17,
        definition,
    };
    envelope.validate().unwrap();
    envelope.version = 11;
    assert!(envelope.validate().is_err());
    let edited = p
        .edit()
        .axis(
            x_axis()
                .name("second")
                .side(AxisSide::Top)
                .secondary_transform("x", transform("cube"))
                .tick_values(Some(vec![8.0.into(), 27.0.into(), 999.0.into()]))
                .tick_format(Some(GuideFormatter::Labels(vec![
                    "eight".into(),
                    "twenty-seven".into(),
                    "outside".into(),
                ]))),
        )
        .build()
        .unwrap();
    let frame = layout(
        edited.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    let axis = &frame.axes()[&id];
    assert_eq!(
        axis.ticks
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        ["eight", "twenty-seven"]
    );
    let reset = edited
        .edit()
        .axis(
            x_axis()
                .name("second")
                .side(AxisSide::Top)
                .secondary("x", 2., 0.),
        )
        .build()
        .unwrap();
    assert!(matches!(
        reset
            .definition()
            .axes
            .iter()
            .find(|a| a.id == id)
            .unwrap()
            .scale,
        AxisScale::Secondary {
            transform: None,
            ..
        }
    ));
    // Retained original remains independently usable after replacement/reset.
    let old = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    near(
        old.axes()[&id].map_value(&8.0.into()).unwrap().unwrap(),
        old.axes()[&chart_core::ScaleId::new(0)]
            .map_value(&2.0.into())
            .unwrap()
            .unwrap(),
    );
}

#[test]
fn secondary_time_breaks_labels_and_positions_match_54_r_panels() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/secondary-time.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 54);
    for case in cases {
        let date = case["kind"] == "date";
        let factor = if date { 86_400_000. } else { 1_000. };
        let limits = [0, 1].map(|i| (case["limits"][i].as_f64().unwrap() * factor) as i64);
        let zone = case["zone"].as_str().unwrap();
        let scale = if zone == "America/New_York" {
            scale_calendar(TimeScaleSpec {
                domain: limits.to_vec(),
                unit: TimeUnit::Milliseconds,
                zone: CalendarZone::Local(std::sync::Arc::new(TimeZoneRules {
                    version: 1,
                    zone: zone.into(),
                    revision: Revision::INITIAL,
                    tzdata: "explicit-2024-US-transitions".into(),
                    coverage: TimeBounds {
                        start: 1_704_067_200_000,
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
                })),
                ..Default::default()
            })
        } else {
            (if date { scale_date() } else { scale_utc() }).time_domain(limits[0], limits[1])
        };
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    timestamps(limits.to_vec(), TimeUnit::Milliseconds, "UTC"),
                )
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(x_axis().scale(scale).expansion(Some(GgplotExpansion {
            mult: [if case["expand"].as_bool().unwrap() {
                0.05
            } else {
                0.
            }; 2],
            add: [0.; 2],
        })))
        .axis(
            x_axis()
                .name("secondary")
                .side(AxisSide::Top)
                .secondary("x", 1., case["offset"].as_f64().unwrap())
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        let wire = p.to_json().unwrap();
        let p = Plot::from_json(&wire).unwrap();
        assert_eq!(p.to_json().unwrap(), wire);
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics)
            .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let secondary = &frame.axes()[&p.axis("secondary").unwrap().id()];
        let ResolvedScale::SecondaryTime { axis, .. } = &secondary.scale else {
            panic!()
        };
        let ResolvedScale::Calendar(scale) = &axis.scale else {
            panic!()
        };
        let view = scale.relative_viewport();
        for (actual, index) in [(view.start(), 0), (view.end(), 1)] {
            near(
                (actual + scale.origin() as f64) / factor,
                case["range"][index].as_f64().unwrap(),
            );
        }
        assert_eq!(
            secondary.ticks.len(),
            case["values"].as_array().unwrap().len(),
            "{case}"
        );
        for (i, tick) in secondary.ticks.iter().enumerate() {
            let ScaleValue::Timestamp { value, unit } = tick.value else {
                panic!()
            };
            assert_eq!(unit, TimeUnit::Milliseconds);
            near(value as f64 / factor, case["values"][i].as_f64().unwrap());
            assert_eq!(tick.label, case["labels"][i].as_str().unwrap(), "{case}");
            near(
                (tick.position - scale.range().start())
                    / (scale.range().end() - scale.range().start()),
                case["positions"][i].as_f64().unwrap(),
            );
            near(
                secondary.map_value(&tick.value).unwrap().unwrap(),
                tick.position,
            );
        }
        assert!(!secondary.capabilities().numeric_inverse);
        assert_eq!(
            secondary.invert_value(0.).unwrap_err().code,
            DiagnosticCode::UnsupportedCapability
        );
    }
}

#[test]
fn secondary_time_validation_edits_and_legacy_wire_are_explicit() {
    let make = |unit, factor, offset| {
        plot(
            Data::columns()
                .column("x", timestamps(vec![0, 120], unit, "UTC"))
                .column("y", [0., 1.])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(x_axis().scale(scale_utc()))
        .axis(
            x_axis()
                .name("secondary")
                .side(AxisSide::Top)
                .secondary("x", factor, offset),
        )
        .build()
        .unwrap()
    };
    let p = make(TimeUnit::Seconds, 1., 60.);
    assert_eq!(p.definition().wire_version(), 17);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 17,
        definition: p.definition().clone(),
    };
    envelope.validate().unwrap();
    envelope.version = 16;
    assert!(envelope.validate().is_err());
    let original = p.to_json().unwrap();
    let edited = p
        .edit()
        .axis(
            x_axis()
                .name("secondary")
                .side(AxisSide::Top)
                .secondary("x", 1., -60.),
        )
        .build()
        .unwrap();
    let a = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let b = layout(
        edited.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    let id = p.axis("secondary").unwrap().id();
    let first = |frame: &LaidOutChart| match frame.axes()[&id].ticks[0].value {
        ScaleValue::Timestamp { value, .. } => value,
        _ => panic!(),
    };
    assert_eq!(first(&a) - first(&b), 120);
    assert_eq!(p.to_json().unwrap(), original);
    assert!(
        Cartesian::new(
            &a.axes()[&id],
            &a.axes()[&p.axis("y").unwrap().id()],
            a.plot().unwrap()
        )
        .is_err()
    );
    for (factor, offset, code) in [
        (2., 0., DiagnosticCode::UnsupportedCapability),
        (1., 0.5, DiagnosticCode::PrecisionLoss),
        (1., 1e30, DiagnosticCode::PrecisionLoss),
    ] {
        let invalid = make(TimeUnit::Seconds, factor, offset);
        assert_eq!(
            layout(
                invalid.chart().unwrap().prepare().unwrap(),
                &request(),
                &Metrics
            )
            .unwrap_err()
            .code,
            code
        );
    }
    let fine = make(TimeUnit::Milliseconds, 1., 0.5);
    assert!(
        layout(
            fine.chart().unwrap().prepare().unwrap(),
            &request(),
            &Metrics
        )
        .is_ok()
    );
}
