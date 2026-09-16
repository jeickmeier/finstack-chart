//! FIX-GG04: complete positional OOB vectors at both primary population stages.
use chart_core::{ChartResult, Revision, grammar::*, interpolate::Number, prelude::*};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
fn encode(v: f64) -> Json {
    if v.is_nan() {
        Json::Null
    } else if v == f64::INFINITY {
        json!("Infinity")
    } else if v == f64::NEG_INFINITY {
        json!("-Infinity")
    } else {
        json!(v)
    }
}
fn input(v: &Json) -> Option<f64> {
    match v.as_str() {
        Some("Infinity") => Some(f64::INFINITY),
        Some("-Infinity") => Some(f64::NEG_INFINITY),
        _ => v.as_f64(),
    }
}
fn same(a: &Json, b: &Json) -> bool {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => {
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 5e-14
        }
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        (Json::Object(a), Json::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| same(a, b)))
        }
        _ => a == b,
    }
}
struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn guides(prepared: Arc<chart_core::grammar::PreparedChart>, c: &Json) -> bool {
    use chart_core::{GuideId, Rect, ResourceId, layout::*, services::*};
    let frame = layout(
        prepared,
        &LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        ),
        &Metrics,
    )
    .unwrap();
    let ticks = &frame.guides()[&GuideId::new(if c["axis"] == "x" { 0 } else { 1 })].ticks;
    let expected = c["result"]["breaks"]
        .as_array()
        .unwrap()
        .iter()
        .zip(c["result"]["labels"].as_array().unwrap())
        .filter(|(v, _)| v.is_number())
        .collect::<Vec<_>>();
    let labels = json!(ticks.iter().map(|v| v.label.as_str()).collect::<Vec<_>>());
    let wanted = json!(
        expected
            .iter()
            .map(|(_, label)| label.as_str().unwrap_or(""))
            .collect::<Vec<_>>()
    );
    let values = json!(
        ticks
            .iter()
            .map(|v| {
                let chart_core::composition::ScaleValue::Number(v) = v.value else {
                    panic!()
                };
                encode(match c["transform"].as_str().unwrap() {
                    "reverse" => -v,
                    "log10" => v.log10(),
                    _ => v,
                })
            })
            .collect::<Vec<_>>()
    );
    let matches = same(&labels, &wanted)
        && same(
            &values,
            &json!(expected.iter().map(|(v, _)| v).collect::<Vec<_>>()),
        );
    if !matches {
        eprintln!("actual labels {labels}, expected {wanted}; values {values}");
    }
    matches
}
struct Vector(Arc<Mutex<Vec<Json>>>);
impl CustomScaleVector for Vector {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.positional_vector", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
        assert_eq!(i.stage, ScaleVectorStage::OutOfBounds);
        self.0.lock().unwrap().push(json!({"values":i.values.iter().map(|v|encode(v.0)).collect::<Vec<_>>(),"range":i.limits.iter().map(|v|encode(v.0)).collect::<Vec<_>>(),"names":[]}));
        Ok(match i.parameters.as_str().unwrap() {
            "default" => Some(
                i.values
                    .iter()
                    .map(|v| {
                        Number(
                            if v.0.is_finite()
                                && (v.0 < i.limits.first().map_or(f64::NAN, |v| v.0)
                                    || v.0 > i.limits.get(1).map_or(f64::NAN, |v| v.0))
                            {
                                f64::NAN
                            } else {
                                v.0
                            },
                        )
                    })
                    .collect(),
            ),
            "reverse" => Some(i.values.iter().rev().copied().collect()),
            "mean" => Some(vec![Number(
                i.values.iter().map(|v| v.0).sum::<f64>() / i.values.len() as f64,
            )]),
            "index" => Some((1..=i.values.len()).map(|v| Number(v as f64)).collect()),
            "short" => Some(i.values.iter().take(1).copied().collect()),
            "empty" => Some(vec![]),
            "null" => None,
            _ => unreachable!(),
        })
    }
}
#[test]
fn primary_points_preserve_both_positional_vector_calls() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-functions.json"
    ))
    .unwrap();
    let domains: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-domains.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(domains["cases"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 324);
    let mut failures = vec![];
    for (index, c) in cases.iter().enumerate() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(input)
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("v", inputs.clone())
            .column(
                "i",
                (0..inputs.len()).map(|i| i as f64 + 1.).collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let mut scale = match c["transform"].as_str().unwrap() {
            "identity" => scale_linear(),
            "reverse" => scale_reverse(),
            "log10" => scale_log(10.),
            _ => unreachable!(),
        };
        match c["limit_mode"].as_str().unwrap_or("full") {
            "automatic" => {}
            "constant" => scale = scale.domain(4., 4.),
            _ => scale = scale.domain(1., 10.),
        }
        let call = ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: c["mode"].clone(),
        };
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .layer(points());
        let plot = if c["axis"] == "x" {
            draft.aes(aes().x("v").y("i")).x_axis(
                x_axis()
                    .scale(scale)
                    .oob_function(Some(call))
                    .guide_geometry(Some(chart_core::layout::GuideGeometry {
                        labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            )
        } else {
            draft.aes(aes().x("i").y("v")).y_axis(
                y_axis()
                    .scale(scale)
                    .oob_function(Some(call))
                    .guide_geometry(Some(chart_core::layout::GuideGeometry {
                        labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            )
        }
        .build()
        .unwrap();
        assert_eq!(
            serde_json::from_str::<Json>(&plot.to_json().unwrap()).unwrap()["version"],
            40
        );
        assert!(
            calls.lock().unwrap().is_empty(),
            "schema validation ran a vector callback"
        );
        let result = plot.chart().unwrap().prepare();
        if result.is_ok() != c["result"].get("error").is_none()
            || !same(&json!(*calls.lock().unwrap()), &c["calls"])
        {
            failures.push(format!(
                "{index}: {} calls {:?}; result {:?}",
                c,
                calls.lock().unwrap(),
                result.as_ref().err()
            ));
            continue;
        }
        if let Ok(prepared) = result {
            let actual = prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    encode(match m.geometry {
                        PreparedGeometry::Point(p) => {
                            if c["axis"] == "x" {
                                p.x()
                            } else {
                                p.y()
                            }
                        }
                        PreparedGeometry::UnboundedPoint(p) => p[usize::from(c["axis"] != "x")].0,
                        _ => panic!("unexpected point geometry"),
                    })
                })
                .collect::<Vec<_>>();
            let expected = c["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null())
                .cloned()
                .collect::<Vec<_>>();
            if !guides(prepared.clone(), c) {
                failures.push(format!("{index}: guide mismatch {c}"));
            }
            if !same(&json!(actual), &json!(expected)) {
                failures.push(format!("{index} marks {actual:?} expected {expected:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn mean_summary_maps_source_and_selected_generated_values() {
    check_mean_summary_vectors(false);
}

#[test]
fn facets_preserve_ordered_positional_populations() {
    check_facet_vectors(false, false, false);
}
#[test]
fn shared_facets_preserve_ordered_positional_populations() {
    check_facet_vectors(true, false, false);
}
#[test]
fn binned_facets_preserve_ordered_source_vectors() {
    check_facet_vectors(false, true, false);
}
#[test]
fn shared_binned_facets_preserve_ordered_source_vectors() {
    check_facet_vectors(true, true, false);
}
#[test]
fn broadcast_facets_preserve_distinct_source_results_per_panel() {
    check_facet_vectors(false, false, true);
}
#[test]
fn shared_broadcast_facets_preserve_distinct_source_results_per_panel() {
    check_facet_vectors(true, false, true);
}
#[test]
fn binned_broadcast_facets_preserve_distinct_source_results_per_panel() {
    check_facet_vectors(false, true, true);
}
#[test]
fn shared_binned_broadcast_facets_preserve_distinct_source_results_per_panel() {
    check_facet_vectors(true, true, true);
}
fn check_facet_vectors(shared: bool, binned: bool, broadcast: bool) {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-facets.json"
    ))
    .unwrap();
    let arities: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-facet-arities.json"
    ))
    .unwrap();
    let binned_fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-binned-facets.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(arities["cases"].as_array().unwrap())
        .filter(|c| !shared || c["kind"] == "mean")
        .collect::<Vec<_>>();
    let broadcast_fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-broadcast-facets.json"
    ))
    .unwrap();
    let cases = if binned {
        binned_fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| !shared || c["kind"] == "mean")
            .collect::<Vec<_>>()
    } else {
        cases
    };
    let broadcast_bins: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-binned-broadcast-facets.json"
    ))
    .unwrap();
    let broadcast_fixture = if binned {
        &broadcast_bins
    } else {
        &broadcast_fixture
    };
    let cases = if broadcast {
        broadcast_fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| !shared || c["kind"] == "mean")
            .collect::<Vec<_>>()
    } else {
        cases
    };
    assert_eq!(
        cases.len(),
        if broadcast {
            if binned {
                if shared { 216 } else { 432 }
            } else if shared {
                108
            } else {
                216
            }
        } else if binned {
            if shared { 216 } else { 432 }
        } else if shared {
            108
        } else {
            246
        }
    );
    let mut failures = vec![];
    for (index, c) in cases.iter().enumerate() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        let data = Data::columns()
            .column(
                "v",
                c["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(input)
                    .collect::<Vec<_>>(),
            )
            .column(
                "i",
                (1..=c["inputs"].as_array().unwrap().len())
                    .map(|v| v as f64)
                    .collect::<Vec<_>>(),
            )
            .column(
                "g",
                c["groups"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "f",
                c["facets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let scale = match c["transform"].as_str().unwrap() {
            "identity" => scale_linear(),
            "reverse" => scale_reverse(),
            "log10" => scale_log(10.),
            _ => unreachable!(),
        };
        let scale = if binned {
            use chart_core::scales::{
                GgplotBinnedPolicy, GgplotBinnedPosition, GgplotBreaks, ScaleTransform,
            };
            scale_binned(GgplotBinnedPosition {
                bins: GgplotBinnedPolicy {
                    breaks: GgplotBreaks::Nice(3.),
                    ..Default::default()
                },
                transform: match c["transform"].as_str().unwrap() {
                    "reverse" => Some(ScaleTransform::Reverse),
                    "log10" => Some(ScaleTransform::Log { base: 10. }),
                    _ => None,
                },
                ..Default::default()
            })
        } else {
            scale
        };
        let panel_names = if c["panels"] == "occupied" {
            vec!["A", "B"]
        } else {
            vec!["A", "B", "C"]
        };
        let call = ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: c["mode"].clone(),
        };
        let layer = if shared {
            points()
                .from_transform("mean")
                .after_stat(stat_aes().x(1.).y(StatField::Mean))
        } else if c["kind"] == "point" {
            points()
        } else {
            points()
                .stat(summary().x("v").group("g"))
                .after_stat(stat_aes().x(1.).y(StatField::Mean))
        };
        let layer = if broadcast {
            layer.facet_target(FacetTarget::Broadcast)
        } else {
            layer
        };
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .aes(aes().x("i").y("v"))
            .layer(layer)
            .facet(
                facet_wrap("f").free_y(c["scales"] == "free_y").order(
                    panel_names
                        .iter()
                        .copied()
                        .map(|s| PanelKey {
                            values: vec![GroupValue::Text(s.into())],
                        })
                        .collect(),
                ),
            )
            .y_axis(
                y_axis()
                    .scale(scale)
                    .oob_function(Some(call))
                    .guide_geometry(Some(chart_core::layout::GuideGeometry {
                        labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            );
        let draft = if shared {
            let target = if broadcast {
                FacetTarget::Broadcast
            } else {
                FacetTarget::Match
            };
            draft
                .transform(
                    transform("mean", summary().x("v").group("g")).facet_target(target.clone()),
                )
                .transform(
                    transform("copy", identity_stat())
                        .from_transform("mean")
                        .facet_target(target.clone()),
                )
                .layer(
                    points()
                        .from_transform("copy")
                        .facet_target(target)
                        .after_stat(stat_aes().x(2.).y(StatField::Mean)),
                )
        } else {
            draft
        };
        let plot = draft.build().unwrap();
        assert!(calls.lock().unwrap().is_empty());
        let result = plot.chart().unwrap().prepare();
        if result.is_ok() != c["result"].get("error").is_none() {
            failures.push(format!(
                "{index}: result {:?}, reference {c}",
                result.as_ref().err()
            ));
            continue;
        }
        // Compare source and selected generated Y calls. The mean reference
        // also maps unused all-missing ymin/ymax columns, omitted by this mapping.
        let actual = calls.lock().unwrap().clone();
        let reference = c["calls"].as_array().unwrap();
        let source_count = if c["scales"] == "fixed" {
            1
        } else {
            panel_names.len()
        };
        let mut expected: Vec<_> = reference
            .iter()
            .take(source_count * if binned { 1 } else { 2 })
            .cloned()
            .collect();
        if shared && !binned && result.is_ok() {
            expected.extend(
                reference
                    .iter()
                    .skip(source_count)
                    .take(source_count)
                    .cloned(),
            );
        }
        if !same(&json!(actual), &json!(expected)) {
            failures.push(format!(
                "{index}: callback inputs {actual:?}, expected {expected:?}"
            ));
        }
        if let Ok(prepared) = result {
            for (panel, expected) in prepared
                .panels()
                .iter()
                .zip(c["result"]["panels"].as_array().unwrap())
            {
                for layer in panel.chart.layers() {
                    let actual = layer
                        .marks()
                        .iter()
                        .map(|m| match m.geometry {
                            PreparedGeometry::Point(p) => encode(p.y()),
                            PreparedGeometry::UnboundedPoint(p) => encode(p[1].0),
                            _ => panic!(),
                        })
                        .collect::<Vec<_>>();
                    let wanted = expected["mapped"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|v| !v.is_null())
                        .cloned()
                        .collect::<Vec<_>>();
                    if !same(&json!(actual), &json!(wanted)) {
                        failures.push(format!(
                            "{index} {:?}: values {actual:?}, expected {wanted:?}",
                            panel.key
                        ));
                    }
                }
                let mut guide_case = (*c).clone();
                guide_case["axis"] = json!("y");
                guide_case["result"] = expected.clone();
                if !guides(panel.chart.clone(), &guide_case) {
                    failures.push(format!("{index}: guide mismatch {:?}", panel.key));
                }
            }
            assert_eq!(prepared.panels().len(), panel_names.len());
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn shared_summary_maps_source_once_and_each_generated_consumer() {
    check_mean_summary_vectors(true);
}
fn check_mean_summary_vectors(shared: bool) {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-statistics.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 54);
    let mut failures = vec![];
    for (index, c) in cases.iter().enumerate() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(input)
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("v", inputs)
            .column(
                "g",
                c["groups"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let scale = match c["transform"].as_str().unwrap() {
            "identity" => scale_linear(),
            "reverse" => scale_reverse(),
            "log10" => scale_log(10.),
            _ => unreachable!(),
        }
        .domain(1., 10.);
        let call = ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: c["mode"].clone(),
        };
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .aes(aes().x("g").y("v"));
        let draft = if shared {
            draft
                .transform(transform("mean", summary().x("v").group("g")))
                .transform(transform("copy", identity_stat()).from_transform("mean"))
                .layer(
                    points()
                        .from_transform("mean")
                        .after_stat(stat_aes().x(1.).y(StatField::Mean)),
                )
                .layer(
                    points()
                        .from_transform("copy")
                        .after_stat(stat_aes().x(2.).y(StatField::Mean)),
                )
        } else {
            draft.layer(
                points()
                    .stat(summary().x("v").group("g"))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
        };
        let plot = draft
            .y_axis(
                y_axis()
                    .scale(scale)
                    .oob_function(Some(call))
                    .guide_geometry(Some(chart_core::layout::GuideGeometry {
                        labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            )
            .build()
            .unwrap();
        assert!(
            calls.lock().unwrap().is_empty(),
            "schema validation ran a vector callback"
        );
        let result = plot.chart().unwrap().prepare();
        // The typed mapping selects y only. The reference also maps its unused,
        // all-missing ymin/ymax columns; those are not authored in this comparison.
        let mut expected_calls = c["calls"]
            .as_array()
            .unwrap()
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>();
        if shared && c["result"].get("error").is_none() && expected_calls.len() == 2 {
            expected_calls.push(expected_calls[1].clone());
        }
        if result.is_ok() != c["result"].get("error").is_none()
            || !same(&json!(*calls.lock().unwrap()), &json!(expected_calls))
        {
            failures.push(format!(
                "{index}: {} calls {:?}; result {:?}",
                c,
                calls.lock().unwrap(),
                result.as_ref().err()
            ));
            continue;
        }
        if let Ok(prepared) = result {
            for layer in prepared.layers() {
                let actual = layer
                    .marks()
                    .iter()
                    .map(|m| {
                        let PreparedGeometry::Point(p) = m.geometry else {
                            panic!()
                        };
                        encode(p.y())
                    })
                    .collect::<Vec<_>>();
                let expected = &c["result"]["mapped"];
                if !same(&json!(actual), expected) {
                    failures.push(format!("{index} summary {actual:?} expected {expected:?}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn shared_vector_population_uses_node_filters_and_rejects_conflicting_consumers() {
    let calls = Arc::new(Mutex::new(vec![]));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_vector(Arc::new(Vector(calls.clone())))
        .unwrap();
    let data = Data::columns()
        .column("x", [1., 1., 1., 1.])
        .column("v", [999., 1., 10., 100.])
        .build()
        .unwrap();
    let axis = y_axis()
        .scale(scale_log(10.).domain(1., 100.))
        .oob_function(Some(ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: json!("index"),
        }));
    let base = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(Arc::new(registry))
        .aes(aes().x("x").y("v"))
        .transform(transform("copy", identity_stat()).from_transform("mean"))
        .transform(transform("mean", summary().x("v")).filter(filter("v").maximum(100.)))
        .y_axis(axis.clone())
        .layer(
            points()
                .from_transform("mean")
                .after_stat(stat_aes().x(1.).y(StatField::Mean)),
        );
    let figure = base
        .clone()
        .layer(
            points()
                .from_transform("copy")
                .after_stat(stat_aes().x(2.).y(StatField::Mean)),
        )
        .build()
        .unwrap();
    assert!(calls.lock().unwrap().is_empty());
    let prepared = figure.chart().unwrap().prepare().unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            json!({"values":[0.,1.,2.],"range":[0.,2.],"names":[]}),
            json!({"values":[2.],"range":[0.,2.],"names":[]}),
            json!({"values":[2.],"range":[0.,2.],"names":[]}),
        ]
    );
    for layer in prepared.layers() {
        let PreparedGeometry::Point(p) = layer.marks()[0].geometry else {
            panic!()
        };
        assert_eq!(p.y(), 1.);
    }
    calls.lock().unwrap().clear();
    let conflict = base
        .axis(axis.name("other").oob_function(Some(ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: json!("default"),
        })))
        .layer(
            points()
                .from_transform("copy")
                .axes("x", "other")
                .after_stat(stat_aes().x(2.).y(StatField::Mean)),
        )
        .build();
    assert!(
        conflict
            .err()
            .unwrap()
            .message
            .contains("incompatible positional scale")
    );
    assert!(calls.lock().unwrap().is_empty());
}

#[test]
fn temporal_vectors_preserve_reference_units_at_both_mapping_stages() {
    use chart_core::{
        GuideId, Rect, ResourceId, composition::ScaleValue, data::TimeUnit, layout::*, services::*,
    };
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-temporal.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 324);
    let mut failures = vec![];
    let configurations = cases
        .iter()
        .enumerate()
        .flat_map(|(index, c)| {
            [
                (TimeUnit::Seconds, 1.),
                (TimeUnit::Milliseconds, 1000.),
                (TimeUnit::Microseconds, 1e6),
                (TimeUnit::Nanoseconds, 1e9),
            ]
            .into_iter()
            .filter(move |(unit, _)| c["family"] != "duration" || *unit == TimeUnit::Seconds)
            .map(move |(unit, multiplier)| (format!("{index}/{unit:?}"), c, unit, multiplier))
        })
        .collect::<Vec<_>>();
    assert_eq!(configurations.len(), 972);
    for (index, c, unit, multiplier) in configurations {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        let duration = c["family"] == "duration";
        let date = c["family"] == "date";
        let factor = multiplier * if date { 86400. } else { 1. };
        let origin = if duration {
            0
        } else {
            (c["origin"].as_f64().unwrap() * factor) as i64
        };
        let absolute = |v: f64| {
            if duration {
                v
            } else {
                origin as f64 / factor + v / factor
            }
        };
        let values = c["inputs"].as_array().unwrap();
        let mut data = Data::columns();
        if duration {
            data = data.column("v", values.iter().map(input).collect::<Vec<_>>());
        } else {
            data = data.column(
                "v",
                timestamps(
                    values
                        .iter()
                        .map(|v| (v.as_f64().unwrap_or(0.) * factor) as i64)
                        .collect::<Vec<_>>(),
                    unit,
                    "UTC",
                )
                .validity(values.iter().map(|v| !v.is_null()).collect()),
            );
        }
        let summary = c["kind"] == "mean";
        let data = data
            .column(
                "i",
                if summary {
                    c["groups"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect::<Vec<_>>()
                } else {
                    (1..=values.len()).map(|v| v as f64).collect::<Vec<_>>()
                },
            )
            .build()
            .unwrap();
        let mut scale = if duration {
            scale_duration()
        } else if date {
            scale_date()
        } else {
            scale_utc()
        };
        if c["limit_mode"] == "explicit" {
            let base = c["origin"].as_f64().unwrap();
            scale = if duration {
                scale.domain(base + 1., base + 10.)
            } else {
                scale.time_domain(
                    ((base + 1.) * factor) as i64,
                    ((base + 10.) * factor) as i64,
                )
            };
        }
        let mapping = if duration {
            chart_core::plot::Mapping::from("v")
        } else {
            chart_core::plot::Mapping::Timestamp {
                field: "v".into(),
                origin,
            }
        };
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry));
        let horizontal = c["axis"] == "x";
        let draft = draft.aes(if horizontal {
            aes().x(mapping).y("i")
        } else {
            aes().x("i").y(mapping)
        });
        let draft = if summary {
            draft.layer(
                points()
                    .stat(summary_stat_for_temporal(duration, origin))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
        } else {
            draft.layer(points())
        };
        let call = Some(ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: c["mode"].clone(),
        });
        let guide = Some(GuideGeometry {
            labels: Some(GuideLabelPolicy::Preserve),
            ..Default::default()
        });
        let draft = if horizontal {
            draft.x_axis(
                x_axis()
                    .scale(scale)
                    .oob_function(call)
                    .guide_geometry(guide),
            )
        } else {
            draft.y_axis(
                y_axis()
                    .scale(scale)
                    .oob_function(call)
                    .guide_geometry(guide),
            )
        };
        let figure = match draft.build() {
            Ok(p) => p,
            Err(e) => {
                failures.push(format!("{index}: build {e:?}"));
                continue;
            }
        };
        assert!(calls.lock().unwrap().is_empty());
        let result = figure.chart().unwrap().prepare().and_then(|prepared| {
            let frame = layout(
                prepared.clone(),
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.).unwrap(),
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                ),
                &Metrics,
            )?;
            Ok((prepared, frame))
        });
        let expected_calls = c["calls"]
            .as_array()
            .unwrap()
            .iter()
            .take(2)
            .map(|call| {
                assert_eq!(call["value_class"], json!(["numeric"]));
                assert_eq!(call["range_class"], json!(["numeric"]));
                json!({"values":call["values"],"range":call["range"],"names":call["names"]})
            })
            .collect::<Vec<_>>();
        if !same(&json!(*calls.lock().unwrap()), &json!(expected_calls)) {
            failures.push(format!(
                "{index}: calls {:?}, expected {expected_calls:?}",
                calls.lock().unwrap()
            ));
        }
        if result.is_ok() != c["result"].get("error").is_none() {
            failures.push(format!(
                "{index}: result {:?}, reference {}",
                result.as_ref().err(),
                c["result"]
            ));
            continue;
        }
        if let Ok((prepared, frame)) = result {
            let limits = &prepared.positional_limits()
                [&chart_core::ScaleId::new(if horizontal { 0 } else { 1 })];
            let actual_limits = json!(
                limits
                    .iter()
                    .map(|v| encode(absolute(v.0)))
                    .collect::<Vec<_>>()
            );
            if !same(&actual_limits, &c["result"]["limits"]) {
                failures.push(format!(
                    "{index}: limits {actual_limits}, expected {}",
                    c["result"]["limits"]
                ));
            }
            let actual = prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    let PreparedGeometry::Point(p) = m.geometry else {
                        panic!()
                    };
                    encode(absolute(if horizontal { p.x() } else { p.y() }))
                })
                .collect::<Vec<_>>();
            let expected = c["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null())
                .cloned()
                .collect::<Vec<_>>();
            if !same(&json!(actual), &json!(expected)) {
                failures.push(format!(
                    "{index}: positions {actual:?}, expected {expected:?}"
                ));
            }
            let ticks = &frame.guides()[&GuideId::new(if horizontal { 0 } else { 1 })].ticks;
            let values = ticks
                .iter()
                .map(|t| {
                    encode(match t.value {
                        ScaleValue::Number(v) => absolute(v),
                        ScaleValue::Timestamp { value, .. } => value as f64 / factor,
                        _ => panic!(),
                    })
                })
                .collect::<Vec<_>>();
            let expected = c["result"]["breaks"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["result"]["labels"].as_array().unwrap())
                .filter(|(v, _)| v.is_number())
                .collect::<Vec<_>>();
            let wanted = json!(expected.iter().map(|(v, _)| v).collect::<Vec<_>>());
            let labels = json!(ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>());
            let wanted_labels = json!(
                expected
                    .iter()
                    .map(|(_, l)| l.as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
            );
            if !same(&json!(values), &wanted) || labels != wanted_labels {
                failures.push(format!(
                    "{index}: guides {values:?} {labels}, expected {wanted} {wanted_labels}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
fn summary_stat_for_temporal(duration: bool, origin: i64) -> chart_core::plot::StatBuilder {
    if duration {
        summary().x("v").group("i")
    } else {
        summary()
            .x(chart_core::plot::Mapping::Timestamp {
                field: "v".into(),
                origin,
            })
            .group("i")
    }
}
#[test]
fn binned_vectors_run_once_before_source_classification() {
    check_binned_vectors(false);
}
#[test]
fn binned_vectors_compose_with_limits_and_breaks() {
    check_binned_vectors(true);
}
fn check_binned_vectors(joint: bool) {
    use chart_core::scales::{
        GgplotBinnedPolicy, GgplotBinnedPosition, GgplotBreaks, ScaleTransform,
    };
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-binned-vectors.json"
    ))
    .unwrap();
    let compositions: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-pipeline-binned-compositions.json"
    ))
    .unwrap();
    let fixture = if joint { compositions } else { fixture };
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), if joint { 1728 } else { 648 });
    let mut failures = vec![];
    for (index, original) in cases.iter().enumerate() {
        let mut c = original.clone();
        c["transform"] = c["family"].clone();
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        registry.register_scale_limits(Arc::new(BinLimits)).unwrap();
        registry.register_scale_breaks(Arc::new(BinBreaks)).unwrap();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(input)
            .collect::<Vec<_>>();
        let groups = if c["kind"] == "mean" {
            c["groups"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>()
        } else {
            (1..=inputs.len()).map(|i| i as f64).collect()
        };
        let data = Data::columns()
            .column("v", inputs)
            .column("g", groups)
            .build()
            .unwrap();
        let bins = GgplotBinnedPosition {
            breaks_function: (joint && c["cuts"] != "nice").then(|| {
                Box::new(ScaleBreaksOperation {
                    operation: OperationRef::new("test.bin_breaks", Revision::new(1)),
                    parameters: c["cuts"].clone(),
                })
            }),
            bins: GgplotBinnedPolicy {
                limits: (c["limit_mode"] == "explicit")
                    .then_some([Some(Number(1.)), Some(Number(10.))]),
                breaks: if c["cuts"] == "fixed" {
                    GgplotBreaks::Explicit(vec![Number(1.), Number(4.), Number(10.)])
                } else {
                    GgplotBreaks::Nice(3.)
                },
                ..Default::default()
            },
            transform: match c["family"].as_str().unwrap() {
                "reverse" => Some(ScaleTransform::Reverse),
                "log10" => Some(ScaleTransform::Log { base: 10. }),
                _ => None,
            },
            ..Default::default()
        };
        let call = ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: c["mode"].clone(),
        };
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry));
        let draft = if c["kind"] == "mean" {
            draft.aes(aes().x("g").y("v")).layer(
                points()
                    .stat(summary().x("v").group("g"))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
        } else if c["axis"] == "x" {
            draft.aes(aes().x("v").y("g")).layer(points())
        } else {
            draft.aes(aes().x("g").y("v")).layer(points())
        };
        let geometry = Some(chart_core::layout::GuideGeometry {
            labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
            ..Default::default()
        });
        let axis = if c["axis"] == "x" { x_axis() } else { y_axis() };
        let axis = axis
            .scale(scale_binned(bins))
            .oob_function(Some(call))
            .guide_geometry(geometry);
        let axis = if joint {
            axis.limits_function(ScaleLimitsOperation {
                operation: OperationRef::new("test.bin_limits", Revision::new(1)),
                parameters: c["control"].clone(),
            })
        } else {
            axis
        };
        let draft = if c["axis"] == "x" {
            draft.x_axis(axis)
        } else {
            draft.y_axis(axis)
        };
        let plot = draft.build();
        assert!(calls.lock().unwrap().is_empty());
        let result = plot
            .and_then(|plot| plot.chart())
            .and_then(|mut chart| chart.prepare());
        let expected_calls = c["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| json!({"values":v["values"],"range":v["range"],"names":v["names"]}))
            .collect::<Vec<_>>();
        if result.is_ok() != c["result"].get("error").is_none()
            || !same(&json!(*calls.lock().unwrap()), &json!(expected_calls))
        {
            failures.push(format!(
                "{index} {c}: calls {:?}, error {:?}",
                calls.lock().unwrap(),
                result.as_ref().err()
            ));
            continue;
        }
        if let Ok(prepared) = result {
            let actual = prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    encode(match m.geometry {
                        PreparedGeometry::Point(p) => {
                            if c["axis"] == "x" {
                                p.x()
                            } else {
                                p.y()
                            }
                        }
                        PreparedGeometry::UnboundedPoint(p) => p[usize::from(c["axis"] != "x")].0,
                        _ => panic!("unexpected point"),
                    })
                })
                .collect::<Vec<_>>();
            let expected = c["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null())
                .cloned()
                .collect::<Vec<_>>();
            if !same(&json!(actual), &json!(expected)) {
                failures.push(format!("{index} actual {actual:?} expected {expected:?}"));
            }
            if !guides(prepared, &c) {
                failures.push(format!("{index} guide mismatch"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
struct BinLimits;
impl CustomScaleLimits for BinLimits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.bin_limits", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(
        &self,
        i: ScaleLimitsInput<'_>,
    ) -> ChartResult<Option<Vec<chart_core::scales::ScaleKey>>> {
        use chart_core::scales::ScaleKey;
        Ok(match i.parameters.as_str().unwrap() {
            "identity" => i.domain.map(<[ScaleKey]>::to_vec),
            "reverse" => i.domain.map(|v| v.iter().rev().cloned().collect()),
            "fixed" => Some(vec![
                ScaleKey::Number(Number(1.)),
                ScaleKey::Number(Number(10.)),
            ]),
            "single" => Some(vec![ScaleKey::Number(Number(5.))]),
            _ => unreachable!(),
        })
    }
}
struct BinBreaks;
impl CustomScaleBreaks for BinBreaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.bin_breaks", Revision::new(1), true)
    }
    fn accepts_n(&self) -> bool {
        true
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        use chart_core::scales::ScaleKey;
        let d = i
            .domain
            .iter()
            .map(|v| match v {
                ScaleKey::Number(v) => v.0,
                _ => f64::NAN,
            })
            .collect::<Vec<_>>();
        let values = if i.parameters == "domain" {
            d
        } else {
            vec![
                d.get(1).copied().unwrap_or(f64::NAN),
                d.iter().sum::<f64>() / d.len() as f64,
                d.first().copied().unwrap_or(f64::NAN),
                d.first().copied().unwrap_or(f64::NAN),
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]
        };
        Ok(ScaleBreaksOutput {
            values: (!(i.domain_is_null && i.parameters == "domain")).then(|| {
                values
                    .into_iter()
                    .map(|v| ScaleKey::Number(Number(v)))
                    .collect()
            }),
            names: None,
            temporal: None,
        })
    }
}

#[test]
fn explicit_panel_vectors_exclude_untargeted_population() {
    check_targeted_vectors(false, false, 0);
}
#[test]
fn chart_scope_vectors_use_complete_filtered_population() {
    check_targeted_vectors(true, false, 0);
}
#[test]
fn identity_source_chains_preserve_filters_and_callback_population() {
    check_targeted_vectors(false, true, 0);
    check_targeted_vectors(true, true, 0);
}
#[test]
fn mixed_identity_routes_preserve_the_intersection_of_source_populations() {
    for routes in 1..=5 {
        check_targeted_vectors(false, true, routes);
    }
}
fn check_targeted_vectors(chart_scope: bool, identity_chain: bool, routes: u8) {
    let key = |name: &str| PanelKey {
        values: vec![GroupValue::Text(name.into())],
    };
    for free in [false, true] {
        for shared in [false, true] {
            for binned in [false, true] {
                for mode in ["index", "reverse", "short", "empty", "null", "default"] {
                    let run = |special: bool| {
                        let calls = Arc::new(Mutex::new(vec![]));
                        let mut registry = ExtensionRegistry::new();
                        registry
                            .register_scale_vector(Arc::new(Vector(calls.clone())))
                            .unwrap();
                        let chain = identity_chain && special;
                        let data = if chain {
                            Data::columns()
                                .column("v", [999., 10., 20., -99.])
                                .column("i", [99., 1., 2., -99.])
                                .column("f", ["A", "A", "C", "C"])
                        } else {
                            Data::columns()
                                .column("v", [10., 20.])
                                .column("i", [1., 2.])
                                .column("f", ["A", "C"])
                        }
                        .build()
                        .unwrap();
                        let target = if routes == 3 {
                            if special {
                                FacetTarget::Broadcast
                            } else {
                                FacetTarget::Match
                            }
                        } else if routes == 4 {
                            FacetTarget::Match
                        } else if special {
                            if chart_scope {
                                FacetTarget::Match
                            } else {
                                FacetTarget::Panels(vec![key("C"), key("A")])
                            }
                        } else {
                            FacetTarget::Broadcast
                        };
                        let scope = if special && chart_scope {
                            StatScope::Chart
                        } else {
                            StatScope::Facet
                        };
                        let order = if (special && !chart_scope) || [3, 4].contains(&routes) {
                            vec![key("A"), key("B"), key("C")]
                        } else {
                            vec![key("A"), key("C")]
                        };
                        let layer = if shared {
                            points()
                                .from_transform("mean")
                                .after_stat(stat_aes().x(1.).y(StatField::Mean))
                        } else {
                            points()
                        };
                        let layer = if chain && !shared {
                            layer.from_transform("upper")
                        } else {
                            layer
                        };
                        let presentation = if shared && chart_scope {
                            FacetTarget::Broadcast
                        } else {
                            target.clone()
                        };
                        let scale = if binned {
                            scale_binned(chart_core::scales::GgplotBinnedPosition {
                                bins: chart_core::scales::GgplotBinnedPolicy {
                                    limits: Some([Some(Number(0.)), Some(Number(5.))]),
                                    breaks: chart_core::scales::GgplotBreaks::Nice(3.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                        } else {
                            scale_linear()
                        };
                        let draft = plot(data)
                            .profile(Profile::Ggplot2_4_0_3)
                            .extensions(Arc::new(registry))
                            .aes(aes().x("i").y("v"))
                            .layer(layer.facet_target(presentation.clone()).scope(scope))
                            .facet(facet_wrap("f").free_y(free).order(order))
                            .y_axis(y_axis().scale(scale).oob_function(Some(
                                ScaleVectorOperation {
                                    operation: OperationRef::new(
                                        "test.positional_vector",
                                        Revision::new(1),
                                    ),
                                    parameters: json!(mode),
                                },
                            )));
                        let ancestor = match routes {
                            1 | 4 | 5 => FacetTarget::Broadcast,
                            2 => FacetTarget::Panels(vec![key("A"), key("C")]),
                            3 => FacetTarget::Match,
                            _ => target.clone(),
                        };
                        let draft = if chain {
                            draft
                                .transform(
                                    transform("lower", identity_stat())
                                        .facet_target(ancestor.clone())
                                        .scope(if routes == 5 { StatScope::Chart } else { scope })
                                        .filter(filter("v").minimum(0.)),
                                )
                                .transform(
                                    transform("upper", identity_stat())
                                        .from_transform("lower")
                                        .facet_target(ancestor)
                                        .scope(scope)
                                        .filter(filter("v").maximum(100.)),
                                )
                        } else {
                            draft
                        };
                        let mean = transform("mean", summary().x("v"));
                        let mean = if chain {
                            mean.from_transform("upper")
                        } else {
                            mean
                        };
                        let draft = if shared {
                            draft
                                .transform(mean.facet_target(target.clone()).scope(scope))
                                .layer(
                                    points()
                                        .from_transform("mean")
                                        .facet_target(presentation)
                                        .scope(scope)
                                        .after_stat(stat_aes().x(2.).y(StatField::Mean)),
                                )
                        } else {
                            draft
                        };
                        let p = draft.build().unwrap();
                        let prepared = match p.chart().unwrap().prepare() {
                            Ok(prepared) => prepared,
                            Err(error) => return (Err(error.code), calls.lock().unwrap().clone()),
                        };
                        let values = prepared
                            .panels()
                            .iter()
                            .filter(|panel| panel.key != key("B"))
                            .map(|panel| {
                                panel
                                    .chart
                                    .layers()
                                    .iter()
                                    .flat_map(|layer| layer.marks())
                                    .map(|m| match m.geometry {
                                        PreparedGeometry::Point(p) => encode(p.y()),
                                        PreparedGeometry::UnboundedPoint(p) => encode(p[1].0),
                                        _ => panic!(),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>();
                        if special && !chart_scope {
                            assert!(
                                prepared.panels()[1]
                                    .chart
                                    .layers()
                                    .iter()
                                    .all(|l| l.marks().is_empty())
                            );
                        }
                        let calls = calls.lock().unwrap().clone();
                        (Ok(values), calls)
                    };
                    let (wanted, reference_calls) = run(false);
                    let (actual, actual_calls) = run(true);
                    if mode == "index" && !shared && !binned && routes < 3 {
                        assert_eq!(
                            actual,
                            Ok(vec![
                                vec![json!(1.), json!(2.)],
                                if free {
                                    vec![json!(1.), json!(2.)]
                                } else {
                                    vec![json!(3.), json!(4.)]
                                }
                            ])
                        );
                    }
                    assert_eq!(
                        actual, wanted,
                        "chart={chart_scope}, shared={shared}, bins={binned}, free={free}, mode={mode}"
                    );
                    // Empty free panels may participate in the later panel-axis pass;
                    // they must never add source observations or shift callback indices.
                    let populated = |calls: Vec<Json>| {
                        calls
                            .into_iter()
                            .filter(|c| !c["values"].as_array().unwrap().is_empty())
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        populated(actual_calls),
                        populated(reference_calls),
                        "chart={chart_scope}, shared={shared}, bins={binned}, free={free}, mode={mode}"
                    );
                }
            }
        }
    }
}

#[test]
fn grid_margin_vectors_match_pinned_shared_scale_occurrences() {
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/facet-vector-sharing.json"
    ))
    .unwrap();
    for case in source["cases"].as_array().unwrap() {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        let margins = case["margins"].as_bool().unwrap();
        let mode = match case["mode"].as_str().unwrap() {
            "identity" => "default",
            "scalar" => "mean",
            other => other,
        };
        let p = plot(
            Data::columns()
                .column("x", [1., 2., 3., 4.])
                .column("y", [2., 4., 6., 8.])
                .column("r", ["A", "A", "B", "B"])
                .column("c", ["L", "R", "L", "R"])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(Arc::new(registry))
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_grid("r", "c").free_y(true).reference(FacetPolicy {
            margins: if margins { vec![0, 1] } else { vec![] },
            ..Default::default()
        }))
        .y_axis(y_axis().oob_function(Some(ScaleVectorOperation {
            operation: OperationRef::new("test.positional_vector", Revision::new(1)),
            parameters: json!(mode),
        })))
        .build()
        .unwrap();
        let result = p.chart().unwrap().prepare();
        assert_eq!(
            result.is_ok(),
            case["result"]["ok"].as_bool().unwrap(),
            "{case}: {:?}",
            result.as_ref().err()
        );
        let n = if margins { 3 } else { 2 };
        let observed = calls.lock().unwrap();
        for (actual, expected) in observed
            .iter()
            .take(n)
            .zip(case["calls"].as_array().unwrap())
        {
            assert!(
                same(&actual["values"], &expected["values"]),
                "{case}: {observed:?}"
            );
            assert!(
                same(&actual["range"], &expected["range"]),
                "{case}: {observed:?}"
            );
        }
        assert!(observed.len() >= n);
        if let Ok(prepared) = result {
            let mut actual = prepared
                .panels()
                .iter()
                .enumerate()
                .flat_map(|(index, p)| {
                    p.chart.layers()[0].marks().iter().map(move |m| {
                        let PreparedGeometry::Point(v) = m.geometry else {
                            panic!()
                        };
                        (index + 1, v.y())
                    })
                })
                .collect::<Vec<_>>();
            let mut expected = case["result"]["data"][0]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    (
                        row["PANEL"].as_str().unwrap().parse::<usize>().unwrap(),
                        row["y"].as_f64().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            actual.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
            expected.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
            assert_eq!(actual, expected, "{case}");
        }
    }
}
