//! FIX-GG04 / GG2-03: fixed numeric positional palettes, independent R expectations.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, interpolate::Number,
    layout::*, prelude::*, scales::*, services::*,
};
use serde_json::Value;
fn cases() -> Vec<Value> {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-position-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap().clone();
    assert_eq!(cases.len(), 144);
    cases
}
fn key(v: &Value) -> ScaleKey {
    v.as_str()
        .map_or(ScaleKey::Null, |v| ScaleKey::Text(v.into()))
}
fn semantic(v: &Value) -> ScaleValue {
    v.as_str().map_or(ScaleValue::MissingCategory, |v| {
        ScaleValue::Category(v.into())
    })
}
fn number(v: &Value) -> f64 {
    match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => v.as_f64().unwrap_or(f64::NAN),
    }
}
fn near(actual: f64, expected: f64, case: &Value) {
    assert!(
        actual == expected
            || (actual.is_nan() && expected.is_nan())
            || (actual - expected).abs() < 2e-14,
        "{actual} vs {expected}: {case}"
    );
}
fn projected(actual: Option<f64>, expected: &Value, case: &Value) {
    if expected.is_null() {
        assert!(actual.is_none(), "{actual:?} {case}")
    } else {
        near(actual.unwrap(), number(expected), case)
    }
}
fn policy(case: &Value) -> GgplotDiscretePosition {
    GgplotDiscretePosition {
        limits: case["limits"]
            .as_array()
            .map(|v| v.iter().map(key).collect()),
        palette: Some(
            case["palette_values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Number(number(v)))
                .collect(),
        ),
        na_translate: case["na_translate"].as_bool().unwrap(),
        ..Default::default()
    }
}
#[test]
fn materialized_position_palette_matches_reference_mapping_and_range() {
    for case in cases() {
        let values = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(key)
            .collect::<Vec<_>>();
        let result = policy(&case).train(&values);
        let expected = &case["result"];
        if expected["error"].is_string() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let scale = result.unwrap_or_else(|e| panic!("{e:?} {case}"));
        for (a, e) in scale
            .viewport()
            .into_iter()
            .zip(expected["range"].as_array().unwrap())
        {
            near(a.0, number(e), &case)
        }
        for (i, key) in values.iter().enumerate() {
            projected(scale.map(key), &expected["mapped"][i], &case);
            projected(
                scale.project(key, Bounds::new(0., 1.).unwrap()).unwrap(),
                &expected["positions"][i],
                &case,
            );
        }
    }
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn primary_palette_and_secondary_guides_share_mapped_category_coordinates() {
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
    for case in cases() {
        for point in [false, true] {
            let values = case["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column(
                    "x",
                    categorical(values.iter().map(|v| v.as_str().unwrap_or("")))
                        .validity(values.iter().map(|v| !v.is_null()).collect()),
                )
                .column("y", vec![1.; values.len()])
                .build()
                .unwrap();
            let builder = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(if point { scale_point() } else { scale_band() })
                        .range(0., 100.)
                        .discrete_policy(Some(policy(&case)))
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                );
            let figure = builder.clone().build().unwrap();
            assert_eq!(figure.definition().wire_version(), 23);
            let wire = figure.to_json().unwrap();
            let figure = Plot::from_json(&wire).unwrap();
            let mut stale: Value = serde_json::from_str(&wire).unwrap();
            stale["version"] = 22.into();
            assert!(Plot::from_json(&stale.to_string()).is_err());
            let result = layout(
                figure.chart().unwrap().prepare().unwrap(),
                &request,
                &Metrics,
            );
            let expected = &case["result"];
            if expected["error"].is_string() {
                assert!(result.is_err(), "{case}");
                continue;
            }
            let frame = result.unwrap_or_else(|e| panic!("{e:?} {case}"));
            let axis = &frame.axes()[&ScaleId::new(0)];
            for (i, value) in values.iter().enumerate() {
                projected(
                    axis.map_value(&semantic(value)).unwrap().map(|p| p / 100.),
                    &expected["positions"][i],
                    &case,
                )
            }
            for (i, key) in expected["breaks"].as_array().unwrap().iter().enumerate() {
                let tick = axis.ticks.iter().find(|tick| tick.value == semantic(key));
                let position = &expected["major_positions"][i];
                if position.is_null() {
                    assert!(tick.is_none(), "{case}")
                } else {
                    let tick = tick.unwrap();
                    near(tick.position / 100., number(position), &case);
                    assert_eq!(
                        tick.label,
                        expected["labels"][i].as_str().unwrap_or("NA"),
                        "{case}"
                    )
                }
            }
            let secondary = builder
                .x_axis(
                    x_axis()
                        .name("secondary")
                        .side(AxisSide::Top)
                        .secondary("x", 1., 0.)
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()
                .unwrap();
            let id = secondary.axis("secondary").unwrap().id();
            let result = layout(
                secondary.chart().unwrap().prepare().unwrap(),
                &request,
                &Metrics,
            );
            let expected = &expected["secondary"];
            if expected["error"].is_string() {
                assert!(result.is_err(), "{case}");
                continue;
            }
            let frame = result.unwrap_or_else(|e| panic!("{e:?} {case}"));
            let axis = &frame.axes()[&id];
            assert_eq!(
                axis.ticks.len(),
                expected["breaks"].as_array().unwrap().len(),
                "{case}"
            );
            for (i, tick) in axis.ticks.iter().enumerate() {
                let ScaleValue::Number(value) = tick.value else {
                    panic!()
                };
                near(value, number(&expected["breaks"][i]), &case);
                near(
                    tick.position / 100.,
                    number(&expected["positions"][i]),
                    &case,
                );
                assert_eq!(
                    tick.label,
                    expected["labels"][i].as_str().unwrap_or("NA"),
                    "{case}"
                )
            }
        }
    }
}

#[test]
fn fixed_position_palettes_retrain_after_replacement_without_mutating_captures() {
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
    let layer = points();
    let secondary_axis = x_axis()
        .name("secondary")
        .side(AxisSide::Top)
        .secondary("x", 1., 0.);
    let build = |data: Data| {
        plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(layer.clone())
            .x_axis(x_axis().discrete_policy(Some(GgplotDiscretePosition {
                palette: Some(vec![8_f64.into(), 2_f64.into(), 5_f64.into(), 3_f64.into()]),
                ..Default::default()
            })))
            .x_axis(secondary_axis.clone())
            .build()
            .unwrap()
    };
    let original_data = data(&[Some("a"), None, Some("b")]);
    let original = build(original_data.clone());
    let wire = original.to_json().unwrap();
    let mut updated = original.chart().unwrap();
    let held = updated.prepare().unwrap();
    let initial = layout(held.clone(), &request, &Metrics)
        .unwrap()
        .guide_snapshots();
    for values in [
        &[Some("c"), None, Some("b"), Some("a")][..],
        &[Some("b"), None][..],
        &[][..],
        &[Some("a"), None, Some("b")][..],
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
