//! FIX-GG04 / GG2-03: independent R positional null/factor/limits corpus.
use chart_core::{DiagnosticCode, scales::*};
use serde_json::Value;
fn keys(value: &Value) -> Vec<ScaleKey> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v.as_str()
                .map_or(ScaleKey::Null, |v| ScaleKey::Text(v.into()))
        })
        .collect()
}
fn optional_keys(value: &Value) -> Option<Vec<ScaleKey>> {
    (!value.is_null()).then(|| keys(value))
}
fn cases() -> Vec<Value> {
    [
        include_str!("../../../fixtures/parity/ggplot2/position-null-categories.json"),
        include_str!("../../../fixtures/parity/ggplot2/position-null-continuous-limits.json"),
        include_str!("../../../fixtures/parity/ggplot2/position-null-minor-breaks.json"),
    ]
    .into_iter()
    .zip([576, 1152, 192])
    .flat_map(|(s, expected)| {
        let values = serde_json::from_str::<Value>(s).unwrap()["cases"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(values.len(), expected);
        values
    })
    .collect()
}
fn number_values(value: &Value) -> Option<Vec<chart_core::interpolate::Number>> {
    value.as_array().map(|values| {
        values
            .iter()
            .map(|v| {
                chart_core::interpolate::Number(match v.as_str() {
                    Some("NaN") => f64::NAN,
                    Some("Infinity") => f64::INFINITY,
                    Some("-Infinity") => f64::NEG_INFINITY,
                    _ => v.as_f64().unwrap(),
                })
            })
            .collect()
    })
}
fn check(actual: Option<f64>, expected: &Value, case: &Value) {
    if expected.is_null() {
        assert_eq!(actual, None, "{case}");
    } else {
        let expected = expected.as_f64().unwrap();
        assert!(
            actual.is_some_and(|a| (a - expected).abs() <= 2e-14),
            "actual={actual:?} expected={expected} {case}"
        );
    }
}
#[test]
fn ggplot_discrete_position_matches_nullable_reference() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/position-null-categories.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 576);
    let all_cases = cases();
    let cases = &all_cases;
    let mut rejected = 0;
    for case in cases {
        let values = keys(&case["inputs"]);
        let spec = GgplotDiscretePosition {
            limits: optional_keys(&case["limits"]),
            levels: optional_keys(&case["levels"]),
            drop: case["drop"].as_bool().unwrap(),
            na_translate: case["na_translate"].as_bool().unwrap(),
            guide: GgplotDiscreteGuide {
                breaks: optional_keys(&case["breaks"]),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut spec = spec;
        if !case["expansion"].is_null() {
            spec.expansion = serde_json::from_value(case["expansion"].clone()).unwrap();
        }
        let limits = number_values(&case["continuous_limits"]);
        let expected = &case["result"];
        let result = spec.train_with_continuous_limits(&values, limits.as_deref());
        if !expected["error"].is_null() {
            assert_eq!(
                result.unwrap_err().code,
                DiagnosticCode::NumericalDomain,
                "{case}"
            );
            rejected += 1;
            continue;
        }
        let scale = result.unwrap();
        // R exposes numeric fallback limits for an entirely untrained discrete scale;
        // the typed category catalog stays empty, while its expanded viewport agrees.
        if values.is_empty() && spec.limits.is_none() {
            assert!(scale.domain().is_empty());
        } else {
            assert_eq!(scale.domain(), keys(&expected["limits"]), "{case}");
        }
        assert_eq!(
            scale
                .entries()
                .iter()
                .map(|v| v.key.clone())
                .collect::<Vec<_>>(),
            keys(&expected["breaks"]),
            "{case}"
        );
        let labels: Vec<Option<String>> =
            serde_json::from_value(expected["labels"].clone()).unwrap();
        assert_eq!(
            scale
                .entries()
                .iter()
                .map(|v| v.label.clone())
                .collect::<Vec<_>>(),
            labels,
            "{case}"
        );
        for (actual, expected) in scale
            .viewport()
            .iter()
            .zip(expected["range"].as_array().unwrap())
        {
            match expected.as_str() {
                Some("-Inf") => assert_eq!(actual.0, f64::NEG_INFINITY, "{case}"),
                Some("Inf") => assert_eq!(actual.0, f64::INFINITY, "{case}"),
                _ => check(Some(actual.0), expected, case),
            }
        }
        let range = Bounds::new(0., 1.).unwrap();
        for (entry, expected) in scale
            .entries()
            .iter()
            .zip(expected["major_positions"].as_array().unwrap())
        {
            check(scale.project(&entry.key, range).unwrap(), expected, case);
        }
        let mut count = 0;
        for (i, key) in values.iter().enumerate() {
            check(scale.map(key), &expected["mapped"][i], case);
            let position = scale.project(key, range).unwrap();
            count += usize::from(position.is_some());
            check(position, &expected["point_positions"][i], case);
        }
        assert_eq!(
            count,
            expected["point_count"].as_u64().unwrap() as usize,
            "{case}"
        );
    }
    assert_eq!(rejected, 24);
}
#[test]
fn ggplot_discrete_position_retains_identity_and_training() {
    let spec = GgplotDiscretePosition::default();
    let values = vec![ScaleKey::Null, ScaleKey::Text("NA".into())];
    let old = spec.train(&values).unwrap();
    assert_eq!(old.map(&values[0]), Some(2.));
    assert_eq!(old.map(&values[1]), Some(1.));
    let next = spec.train(&values[1..]).unwrap();
    assert_eq!(next.map(&values[0]), None);
    assert_eq!(old.map(&values[0]), Some(2.));
    let reverse = Bounds::new(100., 0.).unwrap();
    assert!(old.project(&values[0], reverse).unwrap() < old.project(&values[1], reverse).unwrap());
    assert!(spec.train(&[ScaleKey::Integer(1)]).is_err());
    let restored: GgplotDiscretePosition =
        serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
    assert_eq!(restored.train(&values).unwrap(), old);
}

#[test]
fn primary_nullable_positions_keep_literal_na_and_null_separate() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*,
        prelude::*, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/position-null-categories.json"
    ))
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
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        c["level_name"] == "character"
            && c["limits_name"] == "auto"
            && c["drop"] == true
            && c["na_translate"] == true
            && c["breaks_name"] == "auto"
    }) {
        let input = case["inputs"].as_array().unwrap();
        for point in [false, true] {
            for reverse in [false, true] {
                let data = Data::columns()
                    .column(
                        "x",
                        categorical(input.iter().map(|v| v.as_str().unwrap_or("")))
                            .validity(input.iter().map(|v| !v.is_null()).collect()),
                    )
                    .column("y", vec![1.; input.len()])
                    .build()
                    .unwrap();
                let figure = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points())
                    .x_axis(
                        x_axis()
                            .scale(if point { scale_point() } else { scale_band() })
                            .range(
                                if reverse { 100. } else { 0. },
                                if reverse { 0. } else { 100. },
                            )
                            .guide_geometry(Some(GuideGeometry {
                                labels: Some(GuideLabelPolicy::Preserve),
                                ..Default::default()
                            })),
                    )
                    .build()
                    .unwrap();
                let figure = Plot::from_json(&figure.to_json().unwrap()).unwrap();
                let mut chart = figure.chart().unwrap();
                let frame = layout(chart.prepare().unwrap(), &request, &Metrics).unwrap();
                let axis = &frame.axes()[&ScaleId::new(0)];
                for (i, v) in input.iter().enumerate() {
                    let key = v.as_str().map_or(ScaleValue::MissingCategory, |v| {
                        ScaleValue::Category(v.into())
                    });
                    let position = axis
                        .map_value(&key)
                        .unwrap()
                        .map(|v| if reverse { 1. - v / 100. } else { v / 100. });
                    check(position, &case["result"]["point_positions"][i], case);
                }
                let snapshot = frame
                    .guide_snapshots()
                    .into_iter()
                    .find(|v| v.spec.side == AxisSide::Bottom)
                    .unwrap();
                let mut actual = snapshot
                    .ticks
                    .iter()
                    .map(|v| match &v.value {
                        ScaleValue::Category(s) => ScaleKey::Text(s.clone()),
                        ScaleValue::MissingCategory => ScaleKey::Null,
                        _ => panic!("non-category guide"),
                    })
                    .collect::<Vec<_>>();
                if reverse {
                    actual.reverse();
                }
                assert_eq!(actual, keys(&case["result"]["breaks"]), "{case}");
            }
        }
    }
}

#[test]
fn primary_discrete_policy_matches_all_nullable_reference_cases() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*,
        prelude::*, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/position-null-categories.json"
    ))
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
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 576);
    let all_cases = cases();
    let mut checked = 0;
    for case in &all_cases {
        let input = case["inputs"].as_array().unwrap();
        for point in [false, true] {
            let data = Data::columns()
                .column(
                    "x",
                    categorical(input.iter().map(|v| v.as_str().unwrap_or("")))
                        .validity(input.iter().map(|v| !v.is_null()).collect()),
                )
                .column("y", vec![1.; input.len()])
                .build()
                .unwrap();
            let policy = GgplotDiscretePosition {
                limits: optional_keys(&case["limits"]),
                levels: optional_keys(&case["levels"]),
                drop: case["drop"].as_bool().unwrap(),
                na_translate: case["na_translate"].as_bool().unwrap(),
                guide: GgplotDiscreteGuide {
                    breaks: optional_keys(&case["breaks"]),
                    ..Default::default()
                },
                ..Default::default()
            };
            let expansion: Option<GgplotExpansion> =
                serde_json::from_value(case["expansion"].clone()).unwrap();
            let limits = number_values(&case["continuous_limits"]);
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(if point { scale_point() } else { scale_band() })
                        .range(0., 100.)
                        .discrete_policy(Some(policy))
                        .expansion(expansion)
                        .continuous_limits(limits)
                        .minor_breaks(
                            number_values(&case["minor_breaks"]).map(MinorBreaks::Numeric),
                        )
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()
                .unwrap();
            assert_eq!(figure.definition().wire_version(), 22);
            let wire = figure.to_json().unwrap();
            let figure = Plot::from_json(&wire).unwrap();
            let mut stale: Value = serde_json::from_str(&wire).unwrap();
            stale["version"] = 21.into();
            assert!(Plot::from_json(&stale.to_string()).is_err());
            let mut chart = figure.chart().unwrap();
            let result = layout(chart.prepare().unwrap(), &request, &Metrics);
            checked += 1;
            if !case["result"]["error"].is_null() {
                assert_eq!(result.unwrap_err().code, DiagnosticCode::NumericalDomain);
                continue;
            }
            let frame = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
            let axis = &frame.axes()[&ScaleId::new(0)];
            for (i, v) in input.iter().enumerate() {
                let key = v.as_str().map_or(ScaleValue::MissingCategory, |v| {
                    ScaleValue::Category(v.into())
                });
                check(
                    axis.map_value(&key).unwrap().map(|v| v / 100.),
                    &case["result"]["point_positions"][i],
                    case,
                );
            }
            let snapshot = frame
                .guide_snapshots()
                .into_iter()
                .find(|v| v.spec.side == AxisSide::Bottom)
                .unwrap();
            let expected_breaks = case["result"]["breaks"].as_array().unwrap();
            let mut visible = 0;
            for (i, key) in expected_breaks.iter().enumerate() {
                let value = key.as_str().map_or(ScaleValue::MissingCategory, |v| {
                    ScaleValue::Category(v.into())
                });
                let tick = snapshot.ticks.iter().find(|tick| tick.value == value);
                let position = &case["result"]["major_positions"][i];
                if position.is_null() {
                    assert!(tick.is_none(), "{case}");
                } else {
                    visible += 1;
                    let tick = tick.unwrap_or_else(|| panic!("missing tick: {case}"));
                    check(Some(tick.position / 100.), position, case);
                    assert_eq!(
                        tick.label,
                        case["result"]["labels"][i].as_str().unwrap_or("NA")
                    );
                }
            }
            assert_eq!(snapshot.ticks.len(), visible, "{case}");
            if let Some(values) = case["result"]["minor_values"].as_array() {
                assert_eq!(snapshot.minor_ticks.len(), values.len(), "{case}");
                for ((tick, value), position) in snapshot
                    .minor_ticks
                    .iter()
                    .zip(values)
                    .zip(case["result"]["minor_positions"].as_array().unwrap())
                {
                    assert_eq!(
                        tick.value,
                        Some(ScaleValue::Number(value.as_f64().unwrap())),
                        "{case}"
                    );
                    check(Some(tick.position / 100.), position, case);
                }
            }
        }
    }
    assert_eq!(checked, all_cases.len() * 2);
}

#[test]
fn nullable_category_replacements_match_fresh_frames() {
    use chart_core::{ChartResult, Rect, ResourceId, Revision, layout::*, prelude::*, services::*};
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
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
    let data = |values: &[Option<&str>]| {
        Data::columns()
            .column(
                "x",
                categorical(values.iter().map(|v| v.unwrap_or("")))
                    .validity(values.iter().map(Option::is_some).collect()),
            )
            .column("y", vec![1.; values.len()])
            .build()
            .unwrap()
    };
    for translate in [false, true] {
        for explicit in [false, true] {
            let point_layer = points();
            let build = |data: Data| {
                plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(point_layer.clone())
                    .x_axis(x_axis().discrete_policy(Some(GgplotDiscretePosition {
                        limits: explicit.then(|| {
                            vec![
                                ScaleKey::Null,
                                ScaleKey::Text("NA".into()),
                                ScaleKey::Text("a".into()),
                            ]
                        }),
                        na_translate: translate,
                        ..Default::default()
                    })))
                    .build()
                    .unwrap()
            };
            let original_data = data(&[None, Some("NA"), Some("a")]);
            let original = build(original_data.clone());
            let wire = original.to_json().unwrap();
            let mut updated = original.chart().unwrap();
            let held = updated.prepare().unwrap();
            let initial = layout(held.clone(), &request, &Metrics)
                .unwrap()
                .guide_snapshots();
            for values in [
                &[Some("NA"), Some("a")][..],
                &[None, None][..],
                &[][..],
                &[None, Some("NA"), Some("a")][..],
            ] {
                let transaction = updated
                    .transaction()
                    .unwrap()
                    .replace(&original_data, data(values))
                    .build()
                    .unwrap();
                updated.apply_transaction(transaction).unwrap();
                let revised = layout(updated.prepare().unwrap(), &request, &Metrics).unwrap();
                let fresh = layout(
                    build(data(values)).chart().unwrap().prepare().unwrap(),
                    &request,
                    &Metrics,
                )
                .unwrap();
                assert_eq!(revised.guide_snapshots(), fresh.guide_snapshots());
                assert_eq!(revised.scene().items(), fresh.scene().items());
                assert_eq!(
                    layout(held.clone(), &request, &Metrics)
                        .unwrap()
                        .guide_snapshots(),
                    initial
                );
                assert_eq!(original.to_json().unwrap(), wire);
            }
        }
    }
}

#[test]
fn explicit_d3_axes_keep_null_omission_in_reference_charts() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*,
        prelude::*, services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
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
    for point in [false, true] {
        let data = Data::columns()
            .column(
                "x",
                categorical(["", "NA", "a"]).validity(vec![false, true, true]),
            )
            .column("y", [1., 1., 1.])
            .build()
            .unwrap();
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(x_axis().range(0., 100.).scale(if point {
                scale_point_d3(Default::default())
            } else {
                scale_band_d3(Default::default())
            }))
            .build()
            .unwrap();
        let frame = layout(
            figure.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap();
        let axis = &frame.axes()[&ScaleId::new(0)];
        assert_eq!(axis.map_value(&ScaleValue::MissingCategory).unwrap(), None);
        assert_eq!(
            axis.map_value(&ScaleValue::Category("NA".into())).unwrap(),
            Some(if point { 0. } else { 25. })
        );
        assert_eq!(
            axis.map_value(&ScaleValue::Category("a".into())).unwrap(),
            Some(if point { 100. } else { 75. })
        );
        assert_eq!(
            frame
                .scene()
                .items()
                .iter()
                .filter(|item| item.layer.is_some())
                .count(),
            2
        );
    }
}
