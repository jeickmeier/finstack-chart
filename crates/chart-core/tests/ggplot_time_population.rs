//! FIX-GG04: integer timestamp limits participate in the shared statistics stage.
use chart_core::{
    DiagnosticCode,
    data::TimeUnit,
    grammar::{PreparedGeometry, ScaleOob, StatField, ValueSpace},
    prelude::*,
    scales::*,
};
#[test]
fn timestamp_population_policies_precede_summary_and_coordinate_limits_do_not() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/time-population.json"
    ))
    .unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let factor = if case["kind"] == "date" {
            86_400_000.
        } else {
            1000.
        };
        let data = Data::columns()
            .column("x", [1., 1., 1., 1., 1.])
            .column(
                "y",
                nullable_timestamps(
                    case["input"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().map(|v| (v * factor).round() as i64))
                        .collect(),
                    TimeUnit::Milliseconds,
                    "UTC",
                ),
            )
            .build()
            .unwrap();
        let scale = if case["kind"] == "date" {
            scale_date()
        } else {
            scale_utc()
        }
        .time_domain(0, factor as i64);
        let policy = match case["policy"].as_str().unwrap() {
            "censor" | "coordinate" => ScaleOob::Censor,
            "squish" => ScaleOob::Squish,
            "keep" => ScaleOob::Keep,
            _ => panic!(),
        };
        let axis = if case["policy"] == "coordinate" {
            y_axis().coordinate_scale(scale)
        } else {
            y_axis().scale(scale).oob(policy)
        };
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .stat(summary().x("y"))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
            .y_axis(axis)
            .build()
            .unwrap();
        let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let layer = &prepared.layers()[0];
        assert_eq!(layer.marks().len(), 1, "{case}");
        let PreparedGeometry::Point(point) = layer.marks()[0].geometry else {
            panic!()
        };
        let Some(ValueSpace::Timestamp {
            origin,
            representation,
        }) = &layer.domains().y_space
        else {
            panic!("timestamp summary lost its representation")
        };
        assert_eq!(representation.unit, TimeUnit::Milliseconds);
        let mean = (point.y() + *origin as f64) / factor;
        assert!(
            (mean - case["mean"].as_f64().unwrap()).abs() < 1e-12,
            "{case}: {mean}"
        );
        if case["policy"] == "censor" {
            let scale = if case["kind"] == "date" {
                scale_date()
            } else {
                scale_utc()
            }
            .time_domain(0, factor as i64);
            let updated = p
                .edit()
                .y_axis(y_axis().scale(scale).oob(ScaleOob::Squish))
                .build()
                .unwrap();
            let revised = updated.chart().unwrap().prepare().unwrap();
            let PreparedGeometry::Point(point) = revised.layers()[0].marks()[0].geometry else {
                panic!()
            };
            assert!(((point.y() + *origin as f64) / factor - 0.375).abs() < 1e-12);
            let unchanged = p.chart().unwrap().prepare().unwrap();
            let PreparedGeometry::Point(point) = unchanged.layers()[0].marks()[0].geometry else {
                panic!()
            };
            assert!(((point.y() + *origin as f64) / factor - 0.25).abs() < 1e-12);
        }
    }
}

#[test]
fn timestamp_censor_and_squish_compare_integers_before_projection() {
    let origin = 9_007_199_254_740_993;
    let data = Data::columns()
        .column(
            "x",
            timestamps(
                vec![origin, i64::MIN, i64::MAX],
                TimeUnit::Nanoseconds,
                "UTC",
            ),
        )
        .column("y", [0., 1., 2.])
        .build()
        .unwrap();
    let base = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points());
    let p = base
        .clone()
        .x_axis(
            x_axis()
                .scale(scale_utc().time_domain(origin, origin + 10))
                .oob(ScaleOob::Squish),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let layer = &prepared.layers()[0];
    assert_eq!(layer.marks().len(), 3);
    assert_eq!(
        layer
            .marks()
            .iter()
            .map(|m| match m.geometry {
                PreparedGeometry::Point(p) => p.x(),
                _ => panic!(),
            })
            .collect::<Vec<_>>(),
        [0., 0., 10.]
    );
    assert!(
        matches!(layer.domains().x_space,Some(ValueSpace::Timestamp{origin:o,..}) if o==origin)
    );
    let censored = base
        .clone()
        .x_axis(x_axis().scale(scale_utc().time_domain(origin, origin + 10)))
        .build()
        .unwrap();
    assert_eq!(
        censored.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .len(),
        1
    );
    let kept = base
        .x_axis(
            x_axis()
                .scale(scale_utc().time_domain(origin, origin + 10))
                .oob(ScaleOob::Keep),
        )
        .build()
        .unwrap_err();
    assert_eq!(kept.code, DiagnosticCode::PrecisionLoss);
    assert_eq!(
        p.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .len(),
        3
    );
}

#[test]
fn timestamp_projection_wire_shape_and_units_are_checked_independently_of_profile() {
    use chart_core::{
        ScaleId,
        grammar::{ScaleProjection, TimestampProjection},
    };
    let policy = ScaleProjection {
        binned: None,
        id: ScaleId::new(0),
        timestamp: Some(TimestampProjection {
            origin: 0,
            limits: TimeBounds { start: 0, end: 10 },
            unit: Some(TimeUnit::Seconds),
        }),
        transform: None,
        limits: None,
        function_limits: None,
        missing: None,
        outside: ScaleOob::Squish,
    };
    let p = plot(
        Data::columns()
            .column("x", timestamps(vec![0, 20], TimeUnit::Seconds, "UTC"))
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(
        aes()
            .x(Mapping::Scaled {
                input: Box::new("x".into()),
                scale: policy.clone(),
            })
            .y("y"),
    )
    .layer(points())
    .build()
    .unwrap();
    assert_eq!(p.definition().wire_version(), 17);
    let wire = p.to_json().unwrap();
    let p = Plot::from_json(&wire).unwrap();
    assert_eq!(p.to_json().unwrap(), wire);
    assert!(
        chart_core::portable::ChartEnvelope {
            version: 16,
            definition: p.definition().clone()
        }
        .validate()
        .is_err()
    );
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert_eq!(
        prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| match m.geometry {
                PreparedGeometry::Point(p) => p.x(),
                _ => panic!(),
            })
            .collect::<Vec<_>>(),
        [0., 10.]
    );
    let mut invalid = policy.clone();
    invalid.transform = Some(ScaleTransform::Reverse);
    assert_eq!(
        invalid.validate().unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    let mut invalid = policy;
    invalid.timestamp.as_mut().unwrap().limits.end = i64::MAX;
    assert_eq!(
        invalid.validate().unwrap_err().code,
        DiagnosticCode::PrecisionLoss
    );
    let invalid = plot(
        Data::columns()
            .column("x", timestamps(vec![0, 10], TimeUnit::Milliseconds, "UTC"))
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().scale(scale_calendar(TimeScaleSpec {
        domain: vec![0, 10],
        unit: TimeUnit::Seconds,
        ..Default::default()
    })))
    .build()
    .unwrap_err();
    assert_eq!(invalid.code, DiagnosticCode::SchemaConflict);
}
