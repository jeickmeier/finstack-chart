//! FIX-GG04 positional bin inputs and post-statistic interval mapping from pinned R.
use chart_core::{interpolate::Number, scales::*};
fn number(value: &serde_json::Value) -> Number {
    Number(value.as_f64().unwrap_or_else(|| match value.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    }))
}
fn values(value: &serde_json::Value) -> Vec<Option<Number>> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| (!v.is_null()).then(|| number(v)))
        .collect()
}
fn equal(actual: f64, expected: f64, case: &serde_json::Value) {
    assert!(
        actual == expected
            || actual.is_nan() && expected.is_nan()
            || (actual - expected).abs() <= 3e-12 * expected.abs().max(1.),
        "{actual} != {expected}: {case}"
    );
}
fn compare(
    actual: chart_core::ChartResult<Vec<Option<Number>>>,
    expected: &serde_json::Value,
    case: &serde_json::Value,
) {
    if expected.get("error").is_some() {
        assert!(actual.is_err(), "{case}");
        return;
    }
    let actual = actual.unwrap_or_else(|e| panic!("{e:?}: {case}"));
    let expected = values(&expected["values"]);
    assert_eq!(actual.len(), expected.len(), "{case}");
    for (actual, expected) in actual.iter().zip(expected) {
        match (actual, expected) {
            (Some(a), Some(b)) => equal(a.0, b.0, case),
            (None, None) => {}
            _ => panic!("{actual:?} != {expected:?}: {case}"),
        }
    }
}
fn configuration(case: &serde_json::Value) -> GgplotBinnedPosition {
    GgplotBinnedPosition {
        bins: GgplotBinnedPolicy {
            limits: case["limits"].as_array().map(|v| {
                [
                    (!v[0].is_null()).then(|| number(&v[0])),
                    (!v[1].is_null()).then(|| number(&v[1])),
                ]
            }),
            breaks: match case["mode"].as_str().unwrap() {
                "equal" => GgplotBreaks::Equal(3.),
                "explicit" => {
                    GgplotBreaks::Explicit([-1., 1., 2., 4., 10., 20.].map(Number).to_vec())
                }
                "empty" => GgplotBreaks::Explicit(vec![]),
                _ => GgplotBreaks::Nice(3.),
            },
            right: case["right"].as_bool().unwrap(),
            ..Default::default()
        },
        transform: match case["transform"].as_str().unwrap() {
            "sqrt" => Some(ScaleTransform::Sqrt),
            "log10" => Some(ScaleTransform::Log { base: 10. }),
            "reverse" => Some(ScaleTransform::Reverse),
            _ => None,
        },
        show_limits: case["show_limits"].as_bool().unwrap(),
        ..Default::default()
    }
}
#[test]
fn positional_bins_match_2400_reference_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-bins.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2400);
    let before = values(&fixture["inputs"]);
    let after = values(&fixture["after"]);
    for case in cases {
        let expected = &case["result"];
        if case["mode"] == "none" {
            // R rejects NULL position breaks at construction; the typed policy rejects null too.
            assert!(expected.get("error").is_some());
            assert!(serde_json::from_value::<GgplotBinnedPosition>(serde_json::json!({"bins":{"limits":null,"breaks":null,"oob":"Squish","right":true}})).is_err());
            continue;
        }
        let spec = configuration(case);
        let population = match case["population"].as_str().unwrap() {
            "empty" => vec![],
            "missing" => vec![None, None],
            "infinite" => vec![Some(Number(f64::INFINITY)), Some(Number(f64::NEG_INFINITY))],
            "constant" => vec![Some(Number(4.)); 2],
            _ => vec![Some(Number(1.)), Some(Number(10.))],
        };
        let result = spec.train(&population);
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let trained = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        for (a, b) in trained
            .limits()
            .iter()
            .zip(expected["limits"].as_array().unwrap())
        {
            equal(a.0, number(b).0, case);
        }
        let entries = trained.guide_entries();
        assert_eq!(
            entries.len(),
            expected["breaks"].as_array().unwrap().len(),
            "{case}"
        );
        for (a, b) in entries.iter().zip(expected["breaks"].as_array().unwrap()) {
            equal(a.transformed.0, number(b).0, case);
        }
        assert_eq!(
            entries
                .iter()
                .map(|v| v.label.as_deref())
                .collect::<Vec<_>>(),
            expected["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            "{case}"
        );
        let restored: PreparedGgplotBinnedPosition =
            serde_json::from_str(&serde_json::to_string(&trained).unwrap()).unwrap();
        assert_eq!(trained, restored);
        compare(
            restored.map_before_statistics(&before),
            &expected["before"],
            case,
        );
        compare(
            restored.map_after_statistics(&after),
            &expected["after"],
            case,
        );
    }
}

#[test]
fn primary_bins_train_before_rows_and_preserve_reference_interval_mapping() {
    use chart_core::{grammar::PreparedGeometry, prelude::*};
    let data = Data::columns()
        .column("x", [1., 2., 4., 5., 8., 10.])
        .column("y", [1., 2., 3., 4., 5., 6.])
        .build()
        .unwrap();
    for (nice, expected) in [
        (true, [2.5, 2.5, 2.5, 2.5, 7.5, 7.5]),
        (false, [2.125, 2.125, 4.375, 4.375, 8.875, 8.875]),
    ] {
        let bins = GgplotBinnedPosition {
            bins: GgplotBinnedPolicy {
                breaks: if nice {
                    GgplotBreaks::Nice(3.)
                } else {
                    GgplotBreaks::Equal(3.)
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let figure = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(x_axis().scale(scale_binned(bins)))
            .build()
            .unwrap();
        let wire = figure.to_json().unwrap();
        assert_eq!(figure.definition().wire_version(), 18);
        let restored = Plot::from_json(&wire).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        let prepared = restored.chart().unwrap().prepare().unwrap();
        assert_eq!(prepared.layers()[0].marks().len(), 6);
        for (mark, value) in prepared.layers()[0].marks().iter().zip(expected) {
            let PreparedGeometry::Point(point) = mark.geometry else {
                panic!()
            };
            equal(point.x(), value, &serde_json::json!({"nice":nice}));
        }
    }
}

#[test]
fn primary_positional_bins_match_960_reference_panels() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-bin-panels.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 960);
    compare_primary_panels(cases);
}
#[test]
fn primary_finite_positional_bins_match_800_reference_panels() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-bin-panels.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["limits"] != serde_json::json!([1, "Infinity"]))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 800);
    compare_primary_panels(&cases);
}
fn compare_primary_panels(cases: &[serde_json::Value]) {
    use chart_core::{
        Rect, ResourceId, Revision, ScaleId, grammar::PreparedGeometry, layout::*, prelude::*,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for case in cases {
        let expected = &case["result"];
        if case["mode"] == "none" {
            assert!(expected.get("error").is_some());
            continue;
        }
        let source = if case["population"] == "constant" {
            [4., 4.]
        } else {
            [1., 10.]
        };
        let data = Data::columns()
            .column("x", source)
            .column("y", [1., 2.])
            .build()
            .unwrap();
        let result = (|| -> chart_core::ChartResult<_> {
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(scale_binned(configuration(case)))
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()?;
            let wire = figure.to_json()?;
            assert_eq!(figure.definition().wire_version(), 18);
            let restored = Plot::from_json(&wire)?;
            assert_eq!(restored.to_json()?, wire);
            let prepared = restored.chart()?.prepare()?;
            let request = LayoutRequest::new(
                Rect::new(0., 0., 640., 360.)?,
                Units::LogicalPixels,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            let frame = layout(prepared.clone(), &request, &Metrics)?;
            Ok((prepared, frame))
        })();
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let (prepared, frame) = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let wanted = expected["x"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_f64())
            .collect::<Vec<_>>();
        assert_eq!(prepared.layers()[0].marks().len(), wanted.len(), "{case}");
        for (mark, value) in prepared.layers()[0].marks().iter().zip(wanted) {
            let PreparedGeometry::Point(point) = mark.geometry else {
                panic!()
            };
            equal(point.x(), value, case);
        }
        let axis = &frame.axes()[&ScaleId::new(0)];
        if matches!(axis.scale, ResolvedScale::Unbounded(_)) {
            assert!(!axis.capabilities().numeric_inverse);
            assert!(axis.invert_value(100.).is_err());
            let drawable = expected["coordinate_x"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v.as_f64().is_some_and(f64::is_finite))
                .count();
            let points = frame
                .scene()
                .items()
                .iter()
                .filter(|item| {
                    item.layer.is_some()
                        && matches!(item.primitive, chart_core::scene::Primitive::Point { .. })
                })
                .count();
            assert_eq!(points, drawable, "{case}");
        }
        let range = match &axis.scale {
            ResolvedScale::Linear(s) => [s.viewport().start(), s.viewport().end()],
            ResolvedScale::Nonlinear(s) => [
                s.transformed_viewport().start(),
                s.transformed_viewport().end(),
            ],
            ResolvedScale::Unbounded(s) => s.viewport().map(|v| v.0),
            _ => panic!(),
        };
        for (a, b) in range.iter().zip(expected["range"].as_array().unwrap()) {
            equal(*a, number(b).0, case);
        }
        assert_eq!(
            axis.ticks
                .iter()
                .map(|v| v.label.as_str())
                .collect::<Vec<_>>(),
            expected["drawable_labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            "{case}"
        );
        assert_eq!(
            axis.ticks.len(),
            expected["drawable_break_values"].as_array().unwrap().len(),
            "{case}"
        );
        for (i, tick) in axis.ticks.iter().enumerate() {
            let chart_core::composition::ScaleValue::Number(value) = tick.value else {
                panic!()
            };
            equal(value, number(&expected["drawable_break_values"][i]).0, case);
            let normalized = (tick.position - frame.plot().unwrap().origin().x())
                / frame.plot().unwrap().width();
            equal(
                normalized,
                number(&expected["drawable_break_positions"][i]).0,
                case,
            );
        }
    }
}

#[test]
fn positional_bins_feed_grouped_statistics_and_filtered_populations() {
    use chart_core::{grammar::PreparedGeometry, prelude::*};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-bin-statistics.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 96);
    for case in cases {
        let mut configured = case.clone();
        configured["show_limits"] = false.into();
        let mut spec = configuration(&configured);
        if case["mode"] == "explicit" {
            spec.bins.breaks = GgplotBreaks::Explicit([1., 2., 4., 10.].map(Number).to_vec());
        }
        let data = Data::columns()
            .column("x", [1.; 6])
            .column("y", [1., 2., 4., 5., 8., 10.])
            .column("g", categorical(["A", "B", "A", "B", "A", "B"]))
            .build()
            .unwrap();
        let stat = if case["grouped"] == true {
            summary().x("y").group("g")
        } else {
            summary().x("y").group_all()
        };
        let mut layer = points()
            .stat(stat)
            .after_stat(stat_aes().x(1.).y(StatField::Mean));
        if case["filtered"] == true {
            layer = layer.filter(filter("y").minimum(4.));
        }
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .y_axis(y_axis().scale(scale_binned(spec)))
            .build()
            .unwrap();
        let prepared = figure
            .chart()
            .unwrap()
            .prepare()
            .unwrap_or_else(|e| panic!("{e:?}: {case}"));
        let expected = &case["result"];
        assert!(expected.get("error").is_none(), "{case}");
        let wanted = expected["y"].as_array().unwrap();
        assert_eq!(prepared.layers()[0].marks().len(), wanted.len(), "{case}");
        for (mark, expected) in prepared.layers()[0].marks().iter().zip(wanted) {
            let PreparedGeometry::Point(point) = mark.geometry else {
                panic!()
            };
            equal(point.y(), number(expected).0, case);
        }
    }
}

#[test]
fn positional_bin_population_replacement_matches_fresh_batch_without_mutating_original() {
    use chart_core::prelude::*;
    let data = |values: &[f64]| {
        Data::columns()
            .column("x", values.to_vec())
            .column("y", vec![1.; values.len()])
            .build()
            .unwrap()
    };
    for transform in [
        None,
        Some(ScaleTransform::Reverse),
        Some(ScaleTransform::Sqrt),
        Some(ScaleTransform::Log { base: 10. }),
    ] {
        let spec = GgplotBinnedPosition {
            transform,
            bins: GgplotBinnedPolicy {
                breaks: GgplotBreaks::Nice(3.),
                ..Default::default()
            },
            ..Default::default()
        };
        let build = |d| {
            plot(d)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(x_axis().scale(scale_binned(spec.clone())))
                .build()
                .unwrap()
        };
        let original_data = data(&[1., 2., 4., 5., 8., 10.]);
        let original = build(original_data.clone());
        let wire = original.to_json().unwrap();
        let initial = original.chart().unwrap().prepare().unwrap();
        let mut updated = original.chart().unwrap();
        for values in [
            &[1., 10.][..],
            &[4., 4.][..],
            &[][..],
            &[1., 2., 4., 5., 8., 10.][..],
        ] {
            let tx = updated
                .transaction()
                .unwrap()
                .replace(&original_data, data(values))
                .build()
                .unwrap();
            updated.apply_transaction(tx).unwrap();
            let revised = updated.prepare().unwrap();
            let fresh = build(data(values)).chart().unwrap().prepare().unwrap();
            let coordinates = |p: &chart_core::grammar::PreparedChart| {
                p.layers()[0]
                    .marks()
                    .iter()
                    .map(|m| m.geometry.clone())
                    .collect::<Vec<_>>()
            };
            assert_eq!(coordinates(&revised), coordinates(&fresh));
            assert_eq!(original.to_json().unwrap(), wire);
            assert_eq!(
                coordinates(&original.chart().unwrap().prepare().unwrap()),
                coordinates(&initial)
            );
        }
    }
}

#[test]
fn nested_binned_projections_require_v18_without_an_axis() {
    use chart_core::{
        Revision, ScaleId,
        grammar::{ChartDefinition, Numeric, ScaleOob, ScaleProjection},
    };
    let projection = |binned| ScaleProjection {
        id: ScaleId::new(0),
        timestamp: None,
        binned,
        transform: None,
        limits: None,
        outside: ScaleOob::Keep,
    };
    let binned = Numeric::Scaled {
        input: Box::new(Numeric::Literal(4.)),
        scale: Box::new(projection(Some(std::sync::Arc::new(
            GgplotBinnedPosition::default()
                .train(&[Some(Number(1.)), Some(Number(10.))])
                .unwrap(),
        )))),
    };
    let mut definition = ChartDefinition::new(Revision::INITIAL);
    definition.mappings.x = Some(Numeric::Scaled {
        input: Box::new(binned),
        scale: Box::new(projection(None)),
    });
    assert_eq!(definition.wire_version(), 18);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 18,
        definition,
    };
    envelope.validate().unwrap();
    let restored: chart_core::portable::ChartEnvelope =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    restored.validate().unwrap();
    envelope.version = 17;
    assert!(envelope.validate().is_err());
}
