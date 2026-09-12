//! FIX-GG04: nullable domain order, missing-palette behavior and shared identity training.
use chart_core::{color::parse_r, interpolate::Value, prelude::*, scales::*};
use serde_json::Value as Json;
fn keys(v: &Json) -> Vec<ScaleKey> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v.as_str()
                .map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into()))
        })
        .collect()
}
fn descriptor(case: &Json) -> MappedScaleSpec {
    let limits = case["limits"].as_array().map(|_| keys(&case["limits"]));
    let levels = case["levels"].as_array().map(|_| keys(&case["levels"]));
    if case["kind"] == "identity" {
        let mut spec = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotDiscreteIdentity(
            GgplotDiscreteIdentity {
                limits,
                levels,
                drop: case["drop"].as_bool().unwrap(),
                na_translate: case["na_translate"].as_bool().unwrap(),
                guide: case["guide"].as_bool().unwrap_or(true),
                observed: vec![],
            },
        ));
        spec.training = ScaleTraining::Eligible;
        spec
    } else {
        MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
            .with_ggplot(GgplotScalePolicy::Discrete {
                empty_population: false,
                limits,
                levels,
                drop: case["drop"].as_bool().unwrap(),
                na_translate: case["na_translate"].as_bool().unwrap(),
                palette: GgplotDiscretePalette::Hue(Default::default()),
            })
            .unwrap()
    }
}
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-null-domains.json"
    ))
    .unwrap()
}
fn paint(value: &Json) -> chart_core::scene::Color {
    // R NA denotes absent paint; RGB channels under zero alpha are not specified.
    value.as_str().map_or(
        chart_core::scene::Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        },
        |value| parse_r(value).unwrap().resolve(),
    )
}
#[test]
fn nullable_domains_match_reference_order_and_mapping() {
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 512);
    for case in fixture["cases"].as_array().unwrap() {
        assert!(case["result"].get("error").is_none(), "{case}");
        let trained = descriptor(case)
            .trained_keys(&keys(&case["inputs"]))
            .unwrap();
        let domain = match &trained.function {
            ScaleFunctionSpec::Ordinal(s) => s.domain.clone(),
            ScaleFunctionSpec::GgplotDiscreteIdentity(s) => s.domain().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(domain, keys(&case["result"]["breaks"]), "{case}");
        let wire = serde_json::to_string(&trained).unwrap();
        assert_eq!(
            serde_json::from_str::<MappedScaleSpec>(&wire).unwrap(),
            trained
        );
        if case["kind"] == "hue" {
            let mapped = MappedScale::for_colors(trained).unwrap();
            for (key, expected) in keys(&case["query"])
                .iter()
                .zip(case["result"]["mapped"].as_array().unwrap())
            {
                assert_eq!(
                    mapped
                        .color(None, Some(key), parse_r("grey50").unwrap().resolve())
                        .unwrap(),
                    paint(expected),
                    "{case}"
                );
            }
        } else {
            let mapped = MappedScale::new(trained).unwrap();
            for (key, expected) in keys(&case["query"])
                .iter()
                .zip(case["result"]["mapped"].as_array().unwrap())
            {
                assert_eq!(
                    mapped.category(Some(key)).unwrap(),
                    expected
                        .as_str()
                        .map_or(Value::Missing, |s| Value::Text(s.into())),
                    "{case}"
                );
            }
        }
    }
}
#[test]
fn primary_nullable_color_domains_preserve_labels_and_marks_through_json() {
    primary_nullable_paints(&fixture(), "hue");
}
#[test]
fn primary_identity_null_paints_preserve_raw_colors_through_json() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-null-paints.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 512);
    primary_nullable_paints(&fixture, "identity");
}
fn primary_nullable_paints(fixture: &Json, kind: &str) {
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == kind)
    {
        let values = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column("v", values)
            .build()
            .unwrap();
        let plot = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", descriptor(case)))
            .layer(points())
            .build()
            .unwrap();
        let wire = plot.to_json().unwrap();
        let restored = Plot::from_json(&wire).unwrap();
        assert_eq!(restored.to_json().unwrap(), wire);
        let prepared = restored.chart().unwrap().prepare().unwrap();
        let layer = &prepared.layers()[0];
        let actual = layer
            .marks()
            .iter()
            .map(|mark| mark.style.color)
            .collect::<Vec<_>>();
        let expected = case["result"]["training_paints"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|value| !value.is_null())
            .map(paint)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{case}");
        if let Some(count) = case["result"]["point_count"].as_u64() {
            assert_eq!(actual.len(), count as usize, "{case}");
        }
        let actual = layer
            .color_legend()
            .map(|legend| {
                legend
                    .entries
                    .iter()
                    .map(|e| e.0.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let expected = case["result"]["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap_or("NA").to_owned())
            .collect::<Vec<_>>();
        let expected = if kind == "identity" && case["guide"] == false {
            vec![]
        } else {
            expected
        };
        assert_eq!(actual, expected, "{case}");
    }
}

#[test]
fn missing_paint_preserves_both_numeric_and_category_position_domains() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision, ScaleId,
        composition::ScaleValue,
        layout::{AxisSide, LayoutRequest, layout},
        services::*,
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
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/missing-paint-positions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 72);
    for case in fixture["cases"].as_array().unwrap() {
        let mut data = Data::columns();
        for (axis, kind) in [("x", "x_kind"), ("y", "y_kind")] {
            let values = case[axis].as_array().unwrap();
            data = if case[kind] == "category" {
                data.column(
                    axis,
                    categorical(
                        values
                            .iter()
                            .map(|v| v.as_str().unwrap().to_owned())
                            .collect::<Vec<_>>(),
                    ),
                )
            } else {
                data.column(
                    axis,
                    values
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect::<Vec<_>>(),
                )
            };
        }
        let data = data
            .column(
                "v",
                case["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let mut policy_case = case.clone();
        policy_case["kind"] = "hue".into();
        policy_case["drop"] = true.into();
        policy_case["levels"] = Json::Null;
        let mut scale = descriptor(&policy_case);
        scale.guide = Some(Box::new(GgplotScaleGuide::Hidden));
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .x_axis(x_axis().range(100., 300.))
            .y_axis(y_axis().range(100., 300.))
            .build()
            .unwrap();
        let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        assert_eq!(
            prepared.layers()[0].marks().len(),
            case["result"]["colors"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null())
                .count(),
            "{case}"
        );
        let frame = layout(prepared, &request, &Metrics).unwrap();
        for (axis, kind, id, positions, labels, side) in [
            (
                "x",
                "x_kind",
                0,
                "x_positions",
                "x_labels",
                AxisSide::Bottom,
            ),
            ("y", "y_kind", 1, "y_positions", "y_labels", AxisSide::Left),
        ] {
            let resolved = &frame.axes()[&ScaleId::new(id)];
            for (input, expected) in case[axis]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["result"][positions].as_array().unwrap())
            {
                let value = if case[kind] == "category" {
                    ScaleValue::Category(input.as_str().unwrap().into())
                } else {
                    ScaleValue::Number(input.as_f64().unwrap())
                };
                let actual = (resolved.map_value(&value).unwrap().unwrap() - 100.) / 200.;
                assert!(
                    (actual - expected.as_f64().unwrap()).abs() < 1e-12,
                    "{actual} != {expected}: {case}"
                );
            }
            let snapshots = frame.guide_snapshots();
            let guide = snapshots.iter().find(|g| g.spec.side == side).unwrap();
            assert_eq!(
                guide
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>(),
                case["result"][labels]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>(),
                "{case}"
            );
        }
    }
}

#[test]
fn retained_empty_population_requires_version_21_and_ordinary_policies_stay_17() {
    let fixture = fixture();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["kind"] == "hue"
                && case["population"] == "empty"
                && case["limits_name"] == "auto"
                && case["levels"].is_null()
        })
        .unwrap();
    for retained in [false, true] {
        let scale = if retained {
            descriptor(case).trained_keys(&[]).unwrap()
        } else {
            descriptor(case)
        };
        let data = Data::columns()
            .column("x", Vec::<f64>::new())
            .column("v", Vec::<Option<String>>::new())
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .build()
            .unwrap();
        let wire = p.to_json().unwrap();
        assert_eq!(
            p.definition().wire_version(),
            if retained { 21 } else { 17 }
        );
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
        if retained {
            let mut downgraded: Json = serde_json::from_str(&wire).unwrap();
            downgraded["version"] = 20.into();
            assert!(Plot::from_json(&downgraded.to_string()).is_err());
            assert!(
                chart_core::portable::ChartEnvelope {
                    version: 20,
                    definition: p.definition().clone()
                }
                .validate()
                .is_err()
            );
        } else {
            assert!(!wire.contains("empty_population"));
        }
    }
}
