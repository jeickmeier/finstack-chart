//! FIX-GG04: authored numeric limits preserve category populations and reference positions.
use chart_core::{
    ChartResult, Diagnostic, DiagnosticCode, Rect, ResourceId, Revision, ScaleId,
    composition::ScaleValue, interpolate::Number, layout::*, prelude::*, scales::*, services::*,
};
use serde_json::Value;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 640., 360.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn number(v: &Value) -> ChartResult<Number> {
    if let Some(v) = v.as_f64() {
        return Ok(Number(v));
    }
    if let Some(v) = v.as_bool() {
        return Ok(Number(if v { 1. } else { 0. }));
    }
    match v.as_str() {
        Some("Infinity") => Ok(Number(f64::INFINITY)),
        Some("-Infinity") => Ok(Number(f64::NEG_INFINITY)),
        Some("NaN") => Ok(Number(f64::NAN)),
        _ if v.is_null() => Ok(Number(f64::NAN)),
        _ => Err(Diagnostic::error(
            DiagnosticCode::Validation,
            "Non-numeric continuous limit.",
            "Use numeric limits.",
        )),
    }
}
fn build(case: &Value, family: &str, reversed: bool) -> ChartResult<Plot> {
    let input = case["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let data = Data::columns()
        .column("x", categorical(input.clone()))
        .column("y", vec![1.; input.len()])
        .build()?;
    let mut axis = x_axis()
        .range(
            if reversed { 100. } else { 0. },
            if reversed { 0. } else { 100. },
        )
        .guide_geometry(Some(GuideGeometry {
            labels: Some(GuideLabelPolicy::Preserve),
            ..Default::default()
        }));
    if family != "auto" {
        let mut scale = if family == "band" {
            scale_band()
        } else {
            scale_point()
        };
        if let Some(levels) = case["levels"].as_array() {
            scale = scale.categories(levels.iter().map(|v| v.as_str().unwrap()));
        }
        axis = axis.scale(scale);
    }
    let expansion = match case["expansion"].as_str().unwrap() {
        "default" => None,
        "none" => Some(GgplotExpansion {
            mult: [0.; 2],
            add: [0.; 2],
        }),
        "asymmetric" => Some(GgplotExpansion {
            mult: [0.1, 0.2],
            add: [0.3, 0.7],
        }),
        "contract" => Some(GgplotExpansion {
            mult: [-0.5, -0.25],
            add: [-0.2, -0.1],
        }),
        _ => unreachable!(),
    };
    let limits = case["limits"]
        .as_array()
        .map(|v| v.iter().map(number).collect::<ChartResult<Vec<_>>>())
        .transpose()?;
    axis = axis
        .expansion(expansion)
        .continuous_limits(limits)
        .minor_breaks(Some(MinorBreaks::Numeric(
            [-2., 0.5, 1., 1.5, 2., 3., 4.]
                .into_iter()
                .map(Number)
                .collect(),
        )));
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(axis)
        .build()
}
#[test]
fn primary_category_limits_preserve_reference_points_guides_and_wire_versions() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-continuous-limits.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 756);
    let mut checked = 0;
    for case in cases {
        let families: &[&str] = if case["levels"].is_null() {
            &["auto", "band", "point"]
        } else {
            &["band", "point"]
        };
        for family in families {
            for reversed in [false, true] {
                checked += 1;
                let figure = build(case, family, reversed);
                if case["result"].get("error").is_some() {
                    let error = figure.unwrap_err();
                    assert_eq!(
                        error.code,
                        if case["limits_name"] == "invalid_type" {
                            DiagnosticCode::Validation
                        } else {
                            DiagnosticCode::SchemaConflict
                        }
                    );
                    continue;
                }
                let figure = figure.unwrap_or_else(|e| panic!("{e:?}: {case}"));
                let expected_version = if case["limits"].is_null() { 19 } else { 20 };
                assert_eq!(figure.definition().wire_version(), expected_version);
                let wire = figure.to_json().unwrap();
                let restored = Plot::from_json(&wire).unwrap();
                assert_eq!(restored.to_json().unwrap(), wire);
                if expected_version == 20 {
                    let mut old: Value = serde_json::from_str(&wire).unwrap();
                    old["version"] = 19.into();
                    assert!(Plot::from_json(&old.to_string()).is_err());
                }
                let frame = layout(
                    restored.chart().unwrap().prepare().unwrap(),
                    &request(),
                    &Metrics,
                )
                .unwrap_or_else(|e| panic!("{e:?}: {case}"));
                let axis = &frame.axes()[&ScaleId::new(0)];
                let expected = &case["result"];
                for (i, input) in case["inputs"].as_array().unwrap().iter().enumerate() {
                    let actual = axis
                        .map_value(&ScaleValue::Category(input.as_str().unwrap().into()))
                        .unwrap();
                    let target = number(&expected["point_positions"][i]).unwrap().0;
                    if !target.is_finite() {
                        assert!(actual.is_none(), "{case}");
                    } else {
                        let target = if reversed { 1. - target } else { target };
                        assert!(
                            (actual.unwrap() / 100. - target).abs() < 1e-12,
                            "{actual:?} != {target}: {case}"
                        );
                    }
                }
                let guide = frame
                    .guide_snapshots()
                    .into_iter()
                    .find(|g| g.spec.side == AxisSide::Bottom)
                    .unwrap();
                let labels = expected["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>();
                let mut actual_labels = guide
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>();
                actual_labels.sort_unstable();
                let mut sorted_labels = labels.clone();
                sorted_labels.sort_unstable();
                assert_eq!(actual_labels, sorted_labels, "{case}");
                for (i, label) in labels.iter().enumerate() {
                    let tick = guide
                        .ticks
                        .iter()
                        .find(|tick| tick.label == *label)
                        .unwrap();
                    let target = expected["major_positions"][i].as_f64().unwrap();
                    let target = if reversed { 1. - target } else { target };
                    assert!(
                        (tick.position / 100. - target).abs() < 1e-12,
                        "{tick:?} {case}"
                    );
                }
                assert_eq!(
                    guide.minor_ticks.len(),
                    expected["minor_values"].as_array().unwrap().len(),
                    "{case}"
                );
                for (i, tick) in guide.minor_ticks.iter().enumerate() {
                    assert_eq!(
                        tick.value,
                        Some(ScaleValue::Number(
                            expected["minor_values"][i].as_f64().unwrap()
                        ))
                    );
                    let target = expected["minor_positions"][i].as_f64().unwrap();
                    let target = if reversed { 1. - target } else { target };
                    assert!(
                        (tick.position / 100. - target).abs() < 1e-12,
                        "{tick:?} {case}"
                    );
                }
            }
        }
    }
    assert_eq!(checked, 3528);
}

#[test]
fn category_limit_population_updates_match_fresh_batches_and_keep_captures() {
    let data = |labels: &[&str]| {
        Data::columns()
            .column(
                "x",
                categorical(labels.iter().map(|v| (*v).to_owned()).collect::<Vec<_>>()),
            )
            .column("y", vec![1.; labels.len()])
            .build()
            .unwrap()
    };
    for point in [false, true] {
        for limits in [
            vec![Number(1.5), Number(2.5)],
            vec![Number(0.), Number(f64::INFINITY)],
            vec![Number(f64::NEG_INFINITY), Number(f64::INFINITY)],
        ] {
            let build = |data| {
                plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points())
                    .x_axis(
                        x_axis()
                            .scale(
                                (if point { scale_point() } else { scale_band() })
                                    .categories(["a", "b", "c", "d"]),
                            )
                            .continuous_limits(Some(limits.clone()))
                            .guide_geometry(Some(GuideGeometry {
                                labels: Some(GuideLabelPolicy::Preserve),
                                ..Default::default()
                            })),
                    )
                    .build()
                    .unwrap()
            };
            let original_data = data(&["a", "c"]);
            let original = build(original_data.clone());
            let wire = original.to_json().unwrap();
            let captured = original.chart().unwrap().prepare().unwrap();
            let initial = layout(captured.clone(), &request(), &Metrics)
                .unwrap()
                .guide_snapshots();
            let mut updated = original.chart().unwrap();
            for labels in [&["d"][..], &["z"][..], &[][..], &["a", "c"][..]] {
                let transaction = updated
                    .transaction()
                    .unwrap()
                    .replace(&original_data, data(labels))
                    .build()
                    .unwrap();
                updated.apply_transaction(transaction).unwrap();
                let revised = layout(updated.prepare().unwrap(), &request(), &Metrics).unwrap();
                let fresh = layout(
                    build(data(labels)).chart().unwrap().prepare().unwrap(),
                    &request(),
                    &Metrics,
                )
                .unwrap();
                assert_eq!(revised.guide_snapshots(), fresh.guide_snapshots());
                for label in ["a", "b", "c", "d", "z"] {
                    let value = ScaleValue::Category(label.into());
                    assert_eq!(
                        revised.axes()[&ScaleId::new(0)].map_value(&value).unwrap(),
                        fresh.axes()[&ScaleId::new(0)].map_value(&value).unwrap()
                    );
                }
                assert_eq!(original.to_json().unwrap(), wire);
                assert_eq!(
                    layout(captured.clone(), &request(), &Metrics)
                        .unwrap()
                        .guide_snapshots(),
                    initial
                );
            }
        }
    }
}

#[test]
fn category_continuous_limit_reset_and_axis_rejections_are_explicit() {
    let data = Data::columns()
        .column("x", categorical(vec!["a".to_owned(), "b".to_owned()]))
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let axis = || {
        x_axis()
            .scale(scale_band())
            .continuous_limits(Some(vec![Number(0.), Number(5.)]))
    };
    let original = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(axis())
        .build()
        .unwrap();
    let wire = original.to_json().unwrap();
    assert_eq!(original.definition().wire_version(), 20);
    let reset = original
        .edit()
        .x_axis(axis().continuous_limits(None))
        .build()
        .unwrap();
    let fresh = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(x_axis().scale(scale_band()))
        .build()
        .unwrap();
    let guides = |p: &Plot| {
        layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics)
            .unwrap()
            .guide_snapshots()
    };
    assert_eq!(guides(&reset), guides(&fresh));
    assert!(reset.definition().wire_version() < 20);
    assert_eq!(original.to_json().unwrap(), wire);
    let numeric = Data::columns()
        .column("x", [1., 2.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let invalid = plot(numeric)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_linear())
                .continuous_limits(Some(vec![Number(0.), Number(1.)])),
        )
        .build();
    assert_eq!(
        invalid.unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    let oversized = original
        .edit()
        .x_axis(axis().continuous_limits(Some(vec![Number(0.); 100_001])))
        .build();
    assert_eq!(oversized.unwrap_err().code, DiagnosticCode::ResourceLimit);
}

#[test]
fn reference_character_domains_sort_without_changing_explicit_order() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-order.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        for family in ["band", "point"] {
            for reversed in [false, true] {
                let plot = build(case, family, reversed).unwrap();
                let frame = layout(
                    plot.chart().unwrap().prepare().unwrap(),
                    &request(),
                    &Metrics,
                )
                .unwrap();
                let axis = &frame.axes()[&ScaleId::new(0)];
                for (i, input) in case["inputs"].as_array().unwrap().iter().enumerate() {
                    let actual = axis
                        .map_value(&ScaleValue::Category(input.as_str().unwrap().into()))
                        .unwrap()
                        .unwrap()
                        / 100.;
                    let target = case["result"]["point_positions"][i].as_f64().unwrap();
                    let target = if reversed { 1. - target } else { target };
                    assert!(
                        (actual - target).abs() < 1e-12,
                        "{actual} != {target}: {case}"
                    );
                }
            }
        }
    }
}
