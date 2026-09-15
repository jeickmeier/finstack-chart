//! GG2-03/FIX-GG04: exact authored temporal endpoints and missing-side training.
use chart_core::{grammar::PreparedGeometry, prelude::*};
use serde_json::Value;

#[test]
fn date_and_datetime_helpers_match_reference_population_limits() {
    date_and_datetime_helpers_match_reference_population_limits_for(include_str!(
        "../../../fixtures/parity/ggplot2/scale-limit-helpers.json"
    ));
}

#[test]
fn named_date_and_datetime_helpers_match_reference_population_limits() {
    date_and_datetime_helpers_match_reference_population_limits_for(include_str!(
        "../../../fixtures/parity/ggplot2/named-limit-dispatch.json"
    ));
}

fn date_and_datetime_helpers_match_reference_population_limits_for(source: &str) {
    let fixture: Value = serde_json::from_str(source).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if !["date", "datetime"]
            .iter()
            .any(|name| case["family"] == *name)
            || case["result"]["error"].is_string()
        {
            continue;
        }
        for origin in [0, 123456789] {
            let data = Data::columns()
                .column(
                    "t",
                    timestamps(
                        [0, 1, 2, 4, 5].map(|v| v * 86400000).to_vec(),
                        TimeUnit::Milliseconds,
                        "UTC",
                    ),
                )
                .column("i", [1., 2., 3., 4., 5.])
                .keys([1, 2, 3, 4, 5])
                .build()
                .unwrap();
            let mapping = Mapping::Timestamp {
                field: "t".into(),
                origin,
            };
            let horizontal = case["axis"] == "x";
            let values = case["authored"].as_array().unwrap();
            let limits = values
                .iter()
                .map(|v| v.as_i64().map(|v| time_value(v * 86400, TimeUnit::Seconds)))
                .collect();
            let limits = if case["family"] == "date" {
                PositionalLimits::Date(limits)
            } else {
                PositionalLimits::Datetime(limits)
            };
            let axis = if horizontal {
                xlim(limits)
            } else {
                ylim(limits)
            };
            let builder = plot(data).profile(Profile::Ggplot2_4_0_3).layer(points());
            let p = if horizontal {
                builder.aes(aes().x(mapping).y("i")).x_axis(axis)
            } else {
                builder.aes(aes().x("i").y(mapping)).y_axis(axis)
            }
            .build()
            .unwrap();
            assert_eq!(p.definition().wire_version(), 43);
            let mut chart = p.chart().unwrap();
            let prepared = chart.prepare().unwrap();
            let layer = &prepared.layers()[0];
            let expected = case["result"]["mapped"].as_array().unwrap();
            let expected_values = expected
                .iter()
                .filter_map(Value::as_f64)
                .collect::<Vec<_>>();
            let factor = if case["family"] == "date" {
                86400000.
            } else {
                1000.
            };
            let actual = layer
                .marks()
                .iter()
                .map(|mark| {
                    let PreparedGeometry::Point(point) = mark.geometry else {
                        panic!("point");
                    };
                    ((if horizontal { point.x() } else { point.y() }) + origin as f64) / factor
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected_values, "{case}, origin={origin}");
            let id = p.axis(if horizontal { "x" } else { "y" }).unwrap().id();
            let actual = prepared.positional_limits()[&id]
                .iter()
                .map(|v| (v.0 + origin as f64) / factor)
                .collect::<Vec<_>>();
            assert_eq!(
                actual,
                case["result"]["limits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect::<Vec<_>>(),
                "{case}"
            );
            let frame = frame(prepared.clone());
            let side = if horizontal {
                chart_core::layout::AxisSide::Bottom
            } else {
                chart_core::layout::AxisSide::Left
            };
            let guide = frame
                .guides()
                .values()
                .find(|g| g.spec.side == side)
                .unwrap();
            let expected_labels = case["result"]["breaks"]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["result"]["labels"].as_array().unwrap())
                .filter(|(v, _)| !v.is_null())
                .map(|(_, label)| label.as_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                guide
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>(),
                expected_labels,
                "{case}"
            );
            let wire = p.to_json().unwrap();
            assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
            count += 1;
        }
    }
    assert_eq!(count, 32);
}

struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(
    prepared: std::sync::Arc<chart_core::grammar::PreparedChart>,
) -> chart_core::layout::LaidOutChart {
    use chart_core::{services::*, *};
    let mut request = layout::LayoutRequest::new(
        Rect::new(0., 0., 640., 640.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.axes = prepared.definition().axes.clone();
    layout::layout(prepared, &request, &Metrics).unwrap()
}

#[test]
fn partial_limits_retrain_without_losing_nanosecond_origins() {
    let origin = 9_007_199_254_740_991;
    let data = |values: &[i64]| {
        Data::columns()
            .column(
                "t",
                timestamps(
                    values.iter().map(|v| origin + v).collect(),
                    TimeUnit::Nanoseconds,
                    "UTC",
                ),
            )
            .build()
            .unwrap()
    };
    let original = data(&[-2, 0, 2, 4]);
    let build = |data: Data| {
        plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "t".into(),
                        origin,
                    })
                    .y(1.),
            )
            .layer(points())
            .x_axis(x_axis().scale(scale_utc()).temporal_limits(Some([
                None,
                Some(time_value(origin + 2, TimeUnit::Nanoseconds)),
            ])))
            .build()
            .unwrap()
    };
    let p = build(original.clone());
    let mut chart = p.chart().unwrap();
    let before = chart.prepare().unwrap();
    let id = p.axis("x").unwrap().id();
    assert_eq!(
        before.positional_limits()[&id]
            .iter()
            .map(|v| v.0)
            .collect::<Vec<_>>(),
        [-2., 2.]
    );
    let replacement = data(&[-4, 0, 4]);
    let tx = chart
        .transaction()
        .unwrap()
        .replace(&original, &replacement)
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        chart_core::transaction::CommitOutcome::Applied(_)
    ));
    let after = chart.prepare().unwrap();
    assert_eq!(
        after.positional_limits()[&id]
            .iter()
            .map(|v| v.0)
            .collect::<Vec<_>>(),
        [-4., 2.]
    );
    let fresh = build(replacement).chart().unwrap().prepare().unwrap();
    let coordinates = |p: &chart_core::grammar::PreparedChart| {
        p.layers()[0]
            .marks()
            .iter()
            .map(|m| {
                let PreparedGeometry::Point(point) = m.geometry else {
                    panic!("point");
                };
                point.x()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(coordinates(&before), [-2., 0., 2.]);
    assert_eq!(coordinates(&after), [-4., 0.]);
    assert_eq!(coordinates(&after), coordinates(&fresh));
    assert_eq!(coordinates(&before), [-2., 0., 2.]);
}

#[test]
fn temporal_limit_precision_and_incompatible_definitions_reject() {
    let data = Data::columns()
        .column("t", timestamps(vec![0, 1], TimeUnit::Nanoseconds, "UTC"))
        .build()
        .unwrap();
    for endpoint in [(1_i64 << 53) + 1, -(1_i64 << 53) - 1] {
        let result = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "t".into(),
                        origin: 0,
                    })
                    .y(1.),
            )
            .layer(points())
            .x_axis(x_axis().scale(scale_utc()).temporal_limits(Some([
                None,
                Some(time_value(endpoint, TimeUnit::Nanoseconds)),
            ])))
            .build()
            .and_then(|p| p.chart()?.prepare());
        assert_eq!(
            result.unwrap_err().code,
            chart_core::DiagnosticCode::PrecisionLoss
        );
    }
    for axis in [
        x_axis().temporal_limits(Some([None, None])),
        x_axis().scale(scale_utc()).temporal_limits(Some([
            None,
            Some(chart_core::composition::ScaleValue::Number(1.)),
        ])),
        x_axis()
            .scale(scale_utc())
            .numeric_limits(Some([None, None]))
            .temporal_limits(Some([None, None])),
    ] {
        assert!(
            plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("t").y(1.))
                .layer(points())
                .x_axis(axis)
                .build()
                .is_err()
        );
    }
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("t").y(1.))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_utc())
                .temporal_limits(Some([None, None])),
        )
        .build()
        .unwrap();
    for version in [42, 44] {
        let mut wire: Value = serde_json::from_str(&p.to_json().unwrap()).unwrap();
        wire["version"] = version.into();
        assert!(Plot::from_json(&wire.to_string()).is_err());
        assert!(
            chart_core::portable::ChartEnvelope {
                version,
                definition: p.definition().clone()
            }
            .validate()
            .is_err()
        );
    }
}

#[test]
fn numeric_and_character_limit_helpers_reuse_existing_scale_policies() {
    numeric_and_character_limit_helpers_reuse_existing_scale_policies_for(include_str!(
        "../../../fixtures/parity/ggplot2/scale-limit-helpers.json"
    ));
}

#[test]
fn named_numeric_and_character_limit_helpers_reuse_existing_scale_policies() {
    numeric_and_character_limit_helpers_reuse_existing_scale_policies_for(include_str!(
        "../../../fixtures/parity/ggplot2/named-limit-dispatch.json"
    ));
}

fn numeric_and_character_limit_helpers_reuse_existing_scale_policies_for(source: &str) {
    use chart_core::{interpolate::Number, scales::ScaleKey};
    let fixture: Value = serde_json::from_str(source).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if !["numeric", "character"]
            .iter()
            .any(|name| case["family"] == *name)
        {
            continue;
        }
        let values = case["authored"].as_array().unwrap();
        if case["result"]["error"].is_string() {
            assert!(
                serde_json::from_value::<[Option<Number>; 2]>(case["authored"].clone()).is_err()
            );
            count += 1;
            continue;
        }
        let category = case["family"] == "character";
        let data = Data::columns()
            .column(
                "v",
                if category {
                    categorical(["0", "1", "2", "4", "5"])
                } else {
                    vec![0., 1., 2., 4., 5.].into()
                },
            )
            .column("i", [1., 2., 3., 4., 5.])
            .build()
            .unwrap();
        let horizontal = case["axis"] == "x";
        let limits = if category {
            PositionalLimits::Discrete(
                values
                    .iter()
                    .map(|v| {
                        v.as_i64()
                            .map_or(ScaleKey::Null, |v| ScaleKey::Text(v.to_string()))
                    })
                    .collect(),
            )
        } else {
            PositionalLimits::Numeric(serde_json::from_value(case["authored"].clone()).unwrap())
        };
        let axis = if horizontal {
            xlim(limits)
        } else {
            ylim(limits)
        }
        .range(100., 540.)
        .guide_geometry(Some(chart_core::layout::GuideGeometry {
            labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
            ..Default::default()
        }));
        let builder = plot(data).profile(Profile::Ggplot2_4_0_3).layer(points());
        let p = if horizontal {
            builder.aes(aes().x("v").y("i")).x_axis(axis)
        } else {
            builder.aes(aes().x("i").y("v")).y_axis(axis)
        }
        .build()
        .unwrap();
        let frame = frame(p.chart().unwrap().prepare().unwrap());
        let points = frame
            .scene()
            .items()
            .iter()
            .filter(|item| item.layer.is_some())
            .filter_map(|item| {
                if let chart_core::scene::Primitive::Point { center, .. } = item.primitive {
                    Some(((if horizontal { center.x() } else { center.y() }) - 100.) / 440.)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let expected = case["result"]["point_positions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_f64)
            .collect::<Vec<_>>();
        assert_eq!(points.len(), expected.len(), "{case}");
        assert!(
            points
                .iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < 1e-12),
            "{case}: {points:?}"
        );
        count += 1;
    }
    assert_eq!(count, 24);
}
