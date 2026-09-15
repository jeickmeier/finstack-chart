//! FIX-GG04 / GG2-03: fixed numeric positional palettes, independent R expectations.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, ScaleId, composition::ScaleValue, layout::*,
    prelude::*, scales::*, services::*,
};
use serde_json::Value;
fn cases() -> Vec<Value> {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-palette-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap().clone();
    assert_eq!(cases.len(), 84);
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
    let mode = match case["route"].as_str().unwrap() {
        "reverse" => "reverse_count",
        "spread" => "square_count",
        "short" => "short_count",
        "named" => "named_square_count",
        "missing" => "missing_count",
        "character" => "reject",
        "null" => "null",
        _ => unreachable!(),
    };
    GgplotDiscretePosition {
        limits: match case["limit_mode"].as_str().unwrap() {
            "retained" => Some(
                ["c", "b", "a", "d"]
                    .into_iter()
                    .map(|s| ScaleKey::Text(s.into()))
                    .collect(),
            ),
            "empty" => Some(vec![]),
            _ => None,
        },
        palette_function: Some(Box::new(chart_core::grammar::ScalePaletteOperation {
            operation: chart_core::grammar::OperationRef::new(
                "example.scale_palette",
                Revision::new(1),
            ),
            parameters: serde_json::json!({"mode":mode,"channel":"size"}),
        })),
        ..Default::default()
    }
}
fn registry() -> std::sync::Arc<chart_core::grammar::ExtensionRegistry> {
    let mut r = chart_core::grammar::ExtensionRegistry::new();
    chart_extension_example::scale_palettes::register(&mut r).unwrap();
    std::sync::Arc::new(r)
}
struct Recorded(std::sync::Arc<std::sync::Mutex<Vec<usize>>>);
impl chart_core::grammar::CustomScalePalette for Recorded {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::CustomScalePalette::descriptor(
            &chart_extension_example::scale_palettes::Palette { portable: true },
        )
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        chart_core::grammar::CustomScalePalette::validate(
            &chart_extension_example::scale_palettes::Palette { portable: true },
            p,
        )
    }
    fn evaluate(
        &self,
        input: chart_core::grammar::ScalePaletteInput<'_>,
    ) -> ChartResult<chart_core::grammar::ScalePaletteOutput> {
        let chart_core::grammar::ScalePaletteDomain::Count(n) = input.domain else {
            panic!("count expected")
        };
        self.0.lock().unwrap().push(n);
        chart_core::grammar::CustomScalePalette::evaluate(
            &chart_extension_example::scale_palettes::Palette { portable: true },
            input,
        )
    }
}
#[test]
fn registered_position_palette_matches_reference_mapping_and_range() {
    for case in cases() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let mut registry = chart_core::grammar::ExtensionRegistry::new();
        registry
            .register_scale_palette(std::sync::Arc::new(Recorded(calls.clone())))
            .unwrap();
        let values = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(key)
            .collect::<Vec<_>>();
        let result = policy(&case).train_with_registry(&values, None, &registry);
        let expected_calls = case["calls"].as_array().unwrap();
        let actual = calls.lock().unwrap();
        assert_eq!(actual.is_empty(), expected_calls.is_empty(), "{case}");
        if let Some(n) = actual.first() {
            assert_eq!(
                actual.len(),
                1,
                "A pure palette is resolved once per training pass."
            );
            assert!(
                expected_calls.iter().all(|v| v.as_u64() == Some(*n as u64)),
                "{case}"
            );
        }
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
            near(a.0, number(e), &case);
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
fn primary_registered_position_palettes_match_reference_and_reject_downgrades() {
    let registry = registry();
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
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry.clone())
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
                )
                .build()
                .unwrap();
            assert_eq!(figure.definition().wire_version(), 61);
            let wire = figure.to_json().unwrap();
            assert!(Plot::from_json(&wire).is_err());
            let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            let mut stale: Value = serde_json::from_str(&wire).unwrap();
            stale["version"] = 60.into();
            assert!(Plot::from_json_with_extensions(&stale.to_string(), registry.clone()).is_err());
            let result = layout(
                restored.chart().unwrap().prepare().unwrap(),
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
                );
            }
            for (i, k) in expected["breaks"].as_array().unwrap().iter().enumerate() {
                let tick = axis.ticks.iter().find(|t| t.value == semantic(k));
                let position = &expected["major_positions"][i];
                if position.is_null() {
                    assert!(tick.is_none(), "{case}");
                } else {
                    let tick = tick.unwrap_or_else(|| panic!("{case}"));
                    near(tick.position / 100., number(position), &case);
                    assert_eq!(tick.label, expected["labels"][i].as_str().unwrap_or("NA"));
                }
            }
        }
    }
}
