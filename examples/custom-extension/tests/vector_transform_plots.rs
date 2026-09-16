//! FIX-GG04 vector transform positional chart oracle.
use chart_core::grammar::{OperationRef, TransformOperation, TransformSelection};
use serde_json::Value;
fn registry() -> std::sync::Arc<chart_core::grammar::ExtensionRegistry> {
    chart_extension_example::registry().unwrap()
}
fn transform(c: &Value) -> GgplotTransform {
    let t = GgplotTransform::Registered {
        selection: Box::new(TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "example.scale_transform_vector".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!({"family": c["family"]}),
        })),
    };
    if c["composed"] == true {
        GgplotTransform::Compose {
            transforms: vec![t, GgplotTransform::Reverse],
        }
    } else {
        t
    }
}
fn limits(c: &Value) -> Option<[Option<chart_core::interpolate::Number>; 2]> {
    c["limits"].as_array().map(|v| {
        std::array::from_fn(|i| {
            let value = number(&v[i]);
            (!value.is_nan()).then_some(chart_core::interpolate::Number(value))
        })
    })
}
fn number(v: &Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| match v["number"].as_str().unwrap() {
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            _ => f64::NAN,
        })
}
fn close(actual: f64, expected: f64, context: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{context}: {actual} expected NaN");
    } else if expected.is_infinite() {
        assert_eq!(actual, expected, "{context}");
    } else {
        assert!(
            (actual - expected).abs() <= 4e-14 * expected.abs().max(1.),
            "{context}: actual={actual:.17e}, expected={expected:.17e}"
        );
    }
}
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision, layout::*, prelude::*, scales::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn vector_positional_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transforms.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
}
#[test]
fn vector_transform_authored_limits_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-limits.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}
#[test]
fn vector_transform_layer_populations_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-layers.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}
#[test]
fn vector_transform_layer_limits_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-layer-limits.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}
#[test]
fn vector_transform_facet_populations_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-facets.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}
#[test]
fn vector_transform_facet_limits_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-facet-limits.json"
    ))
    .unwrap();
    check_vector_positional_transform_reference_plots(&fixture);
    check_paint_transform_reference_plots(&fixture);
}
fn facet(c: &Value) -> Option<chart_core::plot::FacetBuilder> {
    c["facet"].as_str().filter(|v| *v != "none").map(|policy| {
        facet_wrap("panel")
            .free_x(policy == "free_x")
            .columns(2)
            .order(vec![
                PanelKey {
                    values: vec![GroupValue::Text("a".into())],
                },
                PanelKey {
                    values: vec![GroupValue::Text("b".into())],
                },
            ])
            .empty(EmptyPanels::Keep)
    })
}
fn check_vector_positional_transform_reference_plots(fixture: &Value) {
    for (index, c) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["route"] == "position")
    {
        let t =
            transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize]);
        let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
        let data = Data::columns()
            .column(
                "panel",
                (0..input.len())
                    .map(|i| if i % 2 == 0 { "a" } else { "b" })
                    .collect::<Vec<_>>(),
            )
            .column("x", input)
            .column("y", vec![1.; c["inputs"].as_array().unwrap().len()])
            .build()
            .unwrap();
        let break_calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let minor_calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let mut extensions = registry();
        if c.get("minor_mode").is_some() {
            std::sync::Arc::get_mut(&mut extensions)
                .unwrap()
                .register_scale_breaks(std::sync::Arc::new(VectorLoggedMinors(minor_calls.clone())))
                .unwrap();
        }
        if c.get("break_mode").is_some() {
            std::sync::Arc::get_mut(&mut extensions)
                .unwrap()
                .register_scale_breaks(std::sync::Arc::new(IdentityLoggedBreaks(
                    break_calls.clone(),
                )))
                .unwrap();
        }
        let result = (|| -> ChartResult<_> {
            let mut axis = x_axis()
                .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                    transform: t.clone(),
                }))
                .numeric_limits(limits(c))
                .breaks_function(c.get("break_mode").map(|mode| {
                    chart_core::grammar::ScaleBreaksOperation {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.identity_breaks",
                            Revision::new(1),
                        ),
                        parameters: mode.clone(),
                    }
                }))
                .range(100., 540.)
                .tick_arguments(c["count"].as_f64().map(|count| GuideTickArguments {
                    count: Some(count),
                    ..Default::default()
                }))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                }));
            if let Some(values) = c["break_values"].as_array() {
                let values: Vec<_> = values
                    .iter()
                    .map(|v| ScaleValue::Number(number(v)))
                    .collect();
                axis = if let Some(labels) = c["explicit_labels"].as_array() {
                    axis.ticks(
                        values
                            .into_iter()
                            .zip(labels.iter().map(|v| v.as_str().unwrap().to_owned())),
                    )
                } else {
                    axis.tick_values(Some(values))
                };
            }
            if let Some(mode) = c["minor_mode"].as_str() {
                axis = axis.minor_breaks(Some(if mode == "explicit" {
                    MinorBreaks::Numeric(vec![2.0.into(), 5.0.into()])
                } else {
                    MinorBreaks::Registered(chart_core::grammar::ScaleBreaksOperation {
                        operation: chart_core::grammar::OperationRef::new(
                            "test.vector_minors",
                            Revision::new(1),
                        ),
                        parameters: mode.into(),
                    })
                }));
            }
            let mut builder = plot(data)
                .extensions(extensions.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(axis);
            if let Some(values) = c["second"].as_array() {
                builder = builder.layer(
                    points().data(
                        Data::columns()
                            .name("second")
                            .column("x", values.iter().map(number).collect::<Vec<_>>())
                            .column("y", vec![1.; values.len()])
                            .build()?,
                    ),
                );
            }
            if let Some(facet) = facet(c) {
                builder = builder.facet(facet);
            }
            let p = builder.build()?;
            let wire = p.to_json()?;
            assert_eq!(
                serde_json::from_str::<Value>(&wire).unwrap()["version"],
                if p.definition().facets.is_some() {
                    76
                } else {
                    55
                }
            );
            let restored =
                chart_core::plot::Plot::from_json_with_extensions(&wire, extensions.clone())?;
            assert_eq!(restored.to_json()?, wire);
            let prepared = restored.chart()?.prepare()?;
            if let Some(panels) = c["result"]["panels"].as_array() {
                assert_eq!(prepared.panels().len(), panels.len());
                for (panel, expected) in prepared.panels().iter().zip(panels) {
                    let wanted = expected["mapped"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(number)
                        .filter(|v| !v.is_nan())
                        .collect::<Vec<_>>();
                    let marks = panel.chart.layers()[0].marks();
                    assert_eq!(marks.len(), wanted.len(), "facet{index}: {c}");
                    for (mark, wanted) in marks.iter().zip(wanted) {
                        let actual = match mark.geometry {
                            chart_core::grammar::PreparedGeometry::Point(p) => p.x(),
                            chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => p[0].0,
                            _ => panic!("point expected"),
                        };
                        close(actual, wanted, &format!("facet{index}: {c}"));
                    }
                }
            }
            for (layer, field) in [(0, "mapped"), (1, "mapped_second")] {
                if let Some(mapped) = c["result"][field].as_array() {
                    let expected: Vec<_> =
                        mapped.iter().map(number).filter(|v| !v.is_nan()).collect();
                    let actual: Vec<_> = prepared.layers()[layer]
                        .marks()
                        .iter()
                        .map(|m| match m.geometry {
                            chart_core::grammar::PreparedGeometry::Point(p) => p.x(),
                            chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => p[0].0,
                            _ => panic!("Expected point"),
                        })
                        .collect();
                    assert_eq!(
                        actual.len(),
                        expected.len(),
                        "plot{index} {t:?} retained population"
                    );
                    for (a, e) in actual.iter().zip(expected) {
                        close(*a, e, &format!("plot{index} {t:?} mapped"));
                    }
                }
            }
            layout(
                prepared,
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.)?,
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
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "plot{index} {t:?}: {c}, {:?}",
            result.as_ref().err()
        );
        if let Some(expected) = c["minor_calls"].as_array() {
            let actual = minor_calls.lock().unwrap();
            assert_eq!(
                actual.is_empty(),
                expected.is_empty(),
                "plot{index} minor demand: {c}; {actual:?}"
            );
            for (domain, major) in actual.iter() {
                assert!(
                    expected
                        .iter()
                        .any(|call| [(domain, "domain"), (major, "major")].iter().all(
                            |(values, field)| {
                                let wanted = call[*field].as_array().unwrap();
                                values.len() == wanted.len()
                                    && values.iter().zip(wanted).all(|(a, b)| {
                                        let b = number(b);
                                        (a.is_nan() && b.is_nan())
                                            || *a == b
                                            || (a - b).abs() <= 4e-14 * b.abs().max(1.0)
                                    })
                            }
                        )),
                    "plot{index} minor input: {domain:?}, {major:?}; {c}"
                );
            }
        }
        if let Some(expected) = c["break_calls"].as_array() {
            let actual = break_calls.lock().unwrap();
            assert_eq!(
                actual.is_empty(),
                expected.is_empty(),
                "plot{index}: {c}; {actual:?}"
            );
            for (values, is_null) in actual.iter() {
                assert!(
                    expected.iter().any(|call| call["is_null"] == *is_null
                        && call["values"].as_array().unwrap().len() == values.len()
                        && values
                            .iter()
                            .zip(call["values"].as_array().unwrap())
                            .all(|(a, b)| {
                                let b = number(b);
                                (a.is_nan() && b.is_nan())
                                    || *a == b
                                    || (a - b).abs() <= 4e-14 * b.abs().max(1.)
                            })),
                    "plot{index}: unexpected callback {values:?}, null={is_null}; {c}"
                );
            }
        }
        if let Ok(frame) = result {
            if let Some(panels) = c["result"]["panels"].as_array() {
                for (panel, expected) in frame.panels().iter().zip(panels) {
                    check_position_guides(
                        &panel.chart,
                        &serde_json::json!({"result": expected}),
                        index,
                        &t,
                    );
                }
                continue;
            }
            check_position_guides(&frame, c, index, &t);
        }
    }
}

fn check_position_guides(frame: &LaidOutChart, c: &Value, index: usize, t: &GgplotTransform) {
    let bounds = c["result"]["range"].as_array().unwrap();
    let lo = number(&bounds[0]);
    let hi = number(&bounds[1]);
    let expected_minor = c["result"]["minor"]
        .as_array()
        .unwrap()
        .iter()
        .map(number)
        .filter(|v| v.is_finite() && ((v - lo) / (hi - lo)).is_finite())
        .collect::<Vec<_>>();
    let actual_minor = &frame.guides()[&GuideId::new(0)].minor_ticks;
    assert_eq!(
        actual_minor.len(),
        expected_minor.len(),
        "plot{index} minor length: {c}; actual={actual_minor:?}; axes={:?}",
        frame.axes()
    );
    for (tick, expected) in actual_minor.iter().zip(expected_minor) {
        close(
            (tick.position - 100.) / 440.,
            (expected - lo) / (hi - lo),
            &format!("plot{index} minor position"),
        );
    }
    let mut ticks: Vec<_> = frame.guides()[&GuideId::new(0)].ticks.iter().collect();
    ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
    let mut expected: Vec<_> = c["result"]["positions"]
        .as_array()
        .unwrap()
        .iter()
        .zip(
            c["result"]["labels"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        )
        .filter(|(v, _)| v.is_number())
        .collect();
    expected.sort_by(|a, b| number(a.0).total_cmp(&number(b.0)));
    assert_eq!(ticks.len(), expected.len(), "plot{index} {t:?}: {c}");
    for (tick, (pos, label)) in ticks.iter().zip(expected) {
        close(
            (tick.position - 100.) / 440.,
            number(pos),
            &format!("plot{index} {t:?} position"),
        );
        assert_eq!(
            tick.label,
            label.as_str().unwrap(),
            "plot{index} {t:?} label"
        );
    }
}

#[test]
fn vector_paint_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transforms.json"
    ))
    .unwrap();
    check_paint_transform_reference_plots(&fixture);
}
fn check_paint_transform_reference_plots(fixture: &Value) {
    for (index, c) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["route"] == "paint" || c["route"] == "binned_paint")
    {
        let t =
            transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize]);
        let input: Vec<_> = c["inputs"].as_array().unwrap().iter().map(number).collect();
        let data = Data::columns()
            .column("x", (0..input.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "panel",
                (0..input.len())
                    .map(|i| if i % 2 == 0 { "a" } else { "b" })
                    .collect::<Vec<_>>(),
            )
            .column("value", input.clone())
            .build()
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true)? else {
                unreachable!()
            };
            scale.palette_theme_aesthetics.clear();
            scale.training = ScaleTraining::Eligible;
            if let Some(GgplotScalePolicy::Continuous {
                limits: authored, ..
            }) = scale.ggplot.as_deref_mut()
            {
                *authored = limits(c);
            }
            if c["route"] == "binned_paint" {
                scale =
                    scale.with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                        breaks: GgplotBreaks::Nice(c["count"].as_f64().unwrap_or(5.)),
                        limits: limits(c),
                        ..Default::default()
                    })))?;
            }
            let ScaleFunctionSpec::Interpolated(spec) = &mut scale.function else {
                unreachable!()
            };
            spec.normalization = NormalizationSpec::Ggplot {
                family: NumericFamily::Ggplot {
                    transform: t.clone(),
                },
                domain: [0., 1.].map(chart_core::interpolate::Number),
                reverse: false,
                rescaler: GgplotRescaler::Range,
                timestamp: None,
            };
            if c["route"] == "paint" {
                scale = scale.with_guide(GgplotScaleGuide::Continuous(GgplotContinuousGuide {
                    count: c["count"].as_f64(),
                    ..Default::default()
                }))?;
            }
            if c["second"].is_null() && facet(c).is_none() {
                let trained = scale.trained_with_registry(
                    &input
                        .iter()
                        .map(|v| Some(chart_core::interpolate::Number(*v)))
                        .collect::<Vec<_>>(),
                    &registry(),
                )?;
                let retained = plot(data.clone())
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y(1.).color("value").color_scale("paint"))
                    .scale(color_mapped("paint", trained.clone()))
                    .extensions(registry())
                    .layer(points())
                    .build()?;
                let retained_wire = retained.to_json()?;
                let mut old: Value = serde_json::from_str(&retained_wire).unwrap();
                assert_eq!(old["version"], 56);
                assert_eq!(
                    chart_core::plot::Plot::from_json_with_extensions(&retained_wire, registry())?
                        .to_json()?,
                    retained_wire
                );
                old["version"] = Value::from(55);
                assert!(
                    chart_core::plot::Plot::from_json_with_extensions(&old.to_string(), registry())
                        .is_err()
                );
                let mapped = MappedScale::for_colors_with_registry(trained, &registry())?;
                let entries = if c["route"] == "paint" {
                    mapped.continuous_guide_entries(4096, 4096)?
                } else {
                    mapped.binned_guide_entries(4096, 4096)?
                }
                .unwrap_or_default();
                if c["second"].is_null()
                    && let Some(expected) = c["result"]["labels"].as_array()
                {
                    assert_eq!(
                        entries.len(),
                        expected.len(),
                        "paint{index} guide candidate count"
                    );
                    // Reference records expose scale labels before guide censoring.
                    // This API retains candidates but clears labels outside the viewport.
                    for (entry, wanted) in entries.iter().zip(expected) {
                        if entry.visible {
                            assert_eq!(
                                entry.label.as_deref(),
                                wanted.as_str(),
                                "paint{index} visible guide label"
                            );
                        }
                    }
                }
            }
            let mut builder = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("value").color_scale("paint"))
                .scale(color_mapped("paint", scale))
                .extensions(registry())
                .layer(points());
            if let Some(values) = c["second"].as_array() {
                builder = builder.layer(
                    points().data(
                        Data::columns()
                            .name("second")
                            .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                            .column("value", values.iter().map(number).collect::<Vec<_>>())
                            .build()?,
                    ),
                );
            }
            if let Some(facet) = facet(c) {
                builder = builder.facet(facet);
            }
            let p = builder.build()?;
            let wire = p.to_json()?;
            let restored = chart_core::plot::Plot::from_json_with_extensions(&wire, registry())?;
            assert_eq!(restored.to_json()?, wire);
            restored.chart()?.prepare()
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "paint{index} {t:?}: {c}, {:?}",
            result.as_ref().err()
        );
        if let Ok(prepared) = result {
            if let Some(panels) = c["result"]["panels"].as_array() {
                assert_eq!(prepared.panels().len(), panels.len());
                for (panel, expected) in prepared.panels().iter().zip(panels) {
                    let wanted = expected["mapped"].as_array().unwrap();
                    let marks = panel.chart.layers()[0].marks();
                    assert_eq!(marks.len(), wanted.len(), "paintfacet{index}: {c}");
                    for (mark, wanted) in marks.iter().zip(wanted) {
                        assert_eq!(
                            mark.style.color,
                            chart_core::color::parse_r(wanted.as_str().unwrap())
                                .unwrap()
                                .resolve(),
                            "paintfacet{index}: {c}"
                        );
                    }
                }
                continue;
            }
            for (layer, field) in [(0, "mapped"), (1, "mapped_second")] {
                let Some(wanted) = c["result"][field].as_array() else {
                    continue;
                };
                let marks = prepared.layers()[layer].marks();
                assert_eq!(marks.len(), wanted.len(), "paint{index}");
                for (mark, expected) in marks.iter().zip(wanted) {
                    assert_eq!(
                        mark.style.color,
                        chart_core::color::parse_r(expected.as_str().unwrap())
                            .unwrap()
                            .resolve(),
                        "paint{index} {t:?}: {c}"
                    );
                }
            }
        }
    }
}

#[test]
fn retained_vector_bounds_require_a_registered_reference_scale() {
    let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true).unwrap() else {
        unreachable!()
    };
    scale.palette_theme_aesthetics.clear();
    scale.trained_transformed_bounds = Some([2., 22.].map(chart_core::interpolate::Number));
    assert!(MappedScale::for_colors_with_registry(scale.clone(), &registry()).is_err());
    let ScaleFunctionSpec::Interpolated(spec) = &mut scale.function else {
        unreachable!()
    };
    spec.normalization = NormalizationSpec::Ggplot {
        family: NumericFamily::Ggplot {
            transform: transform(&serde_json::json!({"family":"cardinality","composed":false})),
        },
        domain: [0., 20.].map(chart_core::interpolate::Number),
        reverse: false,
        rescaler: GgplotRescaler::Range,
        timestamp: None,
    };
    assert!(MappedScale::for_colors_with_registry(scale.clone(), &registry()).is_ok());
    scale.ggplot = None;
    assert!(MappedScale::for_colors_with_registry(scale, &registry()).is_err());
}

#[test]
fn vector_transform_size_facet_populations_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-size-facets.json"
    ))
    .unwrap();
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let t =
            transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize]);
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect::<Vec<_>>();
        let result = (|| -> ChartResult<_> {
            let mut scale = ggplot_numeric_default(GgplotNumericPalette::Size)?;
            scale.palette_theme_aesthetics.clear();
            scale.training = ScaleTraining::Eligible;
            let ScaleFunctionSpec::Interpolated(spec) = &mut scale.function else {
                unreachable!()
            };
            let NormalizationSpec::Ggplot { family, .. } = &mut spec.normalization else {
                unreachable!()
            };
            *family = NumericFamily::Ggplot {
                transform: t.clone(),
            };
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "panel",
                    (0..inputs.len())
                        .map(|i| if i % 2 == 0 { "a" } else { "b" })
                        .collect::<Vec<_>>(),
                )
                .column("value", inputs)
                .build()?;
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .extensions(registry())
                .layer(points().numeric_scale(
                    chart_core::grammar::NumericAesthetic::Size,
                    "value",
                    scale,
                ))
                .facet(facet(c).unwrap())
                .build()?;
            let wire = p.to_json()?;
            chart_core::plot::Plot::from_json_with_extensions(&wire, registry())?
                .chart()?
                .prepare()
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "size{index}: {c}; {:?}",
            result.as_ref().err()
        );
        if let Ok(prepared) = result {
            let panels = c["result"]["panels"].as_array().unwrap();
            assert_eq!(prepared.panels().len(), panels.len());
            for (panel, expected) in prepared.panels().iter().zip(panels) {
                let wanted = expected["mapped"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .filter(|v| !v.is_nan())
                    .collect::<Vec<_>>();
                let marks = panel.chart.layers()[0].marks();
                assert_eq!(marks.len(), wanted.len(), "size{index}: {c}");
                for (mark, value) in marks.iter().zip(wanted) {
                    close(mark.style.radius, value, &format!("size{index}: {c}"));
                }
            }
        }
    }
}

#[test]
fn vector_transform_generated_count_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-statistics.json"
    ))
    .unwrap();
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let result = (|| -> ChartResult<_> {
            let data = Data::columns()
                .column("x", categorical(["a", "a", "b", "c", "c", "c"]))
                .column("panel", ["a", "b", "a", "a", "b", "b"])
                .build()?;
            let mut builder = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry())
                .aes(aes().x("x"))
                .layer(points().stat(count().group("x")))
                .y_axis(y_axis().scale(chart_core::plot::scale_transform(
                    ScaleTransform::Ggplot {
                        transform: transform(c),
                    },
                )));
            if c["faceted"] == true {
                builder = builder.facet(facet_wrap("panel").columns(2));
            }
            let p = builder.build()?;
            let wire = p.to_json()?;
            chart_core::plot::Plot::from_json_with_extensions(&wire, registry())?
                .chart()?
                .prepare()
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "generated{index}: {c}; {:?}",
            result.as_ref().err()
        );
        if let Ok(prepared) = result {
            let charts = if c["faceted"] == true {
                prepared
                    .panels()
                    .iter()
                    .map(|p| &p.chart)
                    .collect::<Vec<_>>()
            } else {
                vec![&prepared]
            };
            let panels = c["result"]["panels"].as_object().unwrap();
            assert_eq!(charts.len(), panels.len());
            for (chart, expected) in charts.iter().zip(panels.values()) {
                let wanted = expected["y"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .filter(|v| !v.is_nan())
                    .collect::<Vec<_>>();
                let marks = chart.layers()[0].marks();
                assert_eq!(marks.len(), wanted.len(), "generated{index}: {c}");
                for (mark, value) in marks.iter().zip(wanted) {
                    let actual = match mark.geometry {
                        chart_core::grammar::PreparedGeometry::Point(p) => p.y(),
                        chart_core::grammar::PreparedGeometry::UnboundedPoint(p) => p[1].0,
                        _ => panic!("Expected point"),
                    };
                    close(actual, value, &format!("generated{index}: {c}"));
                }
            }
        }
    }
}

#[test]
fn vector_transform_generated_style_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transform-statistic-styles.json"
    ))
    .unwrap();
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let result = (|| -> ChartResult<_> {
            let mut scale = if c["route"] == "size" {
                ggplot_numeric_default(GgplotNumericPalette::Size)?
            } else {
                let ColorScale::Mapped { scale, .. } = ggplot_color_default(true)? else {
                    unreachable!()
                };
                scale
            };
            scale.palette_theme_aesthetics.clear();
            scale.training = ScaleTraining::Eligible;
            let ScaleFunctionSpec::Interpolated(spec) = &mut scale.function else {
                unreachable!()
            };
            spec.normalization = NormalizationSpec::Ggplot {
                family: NumericFamily::Ggplot {
                    transform: transform(c),
                },
                domain: [0., 1.].map(chart_core::interpolate::Number),
                reverse: false,
                rescaler: GgplotRescaler::Range,
                timestamp: None,
            };
            let data = Data::columns()
                .column("x", categorical(["a", "a", "b", "c", "c", "c"]))
                .column("panel", ["a", "b", "a", "a", "b", "b"])
                .build()?;
            let mut layer = points().stat(count().group("x"));
            let mut builder = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(registry())
                .aes(aes().x("x"));
            if c["route"] == "size" {
                layer = layer.numeric_scale(
                    chart_core::grammar::NumericAesthetic::Size,
                    StatField::Count,
                    scale,
                );
            } else {
                layer = layer.after_stat(
                    stat_aes()
                        .x(StatField::Group)
                        .y(StatField::Count)
                        .color(StatField::Count)
                        .color_scale("paint"),
                );
                builder = builder.scale(color_mapped("paint", scale.clone()));
            }
            builder = builder.layer(layer);
            if c["faceted"] == true {
                builder = builder.facet(facet_wrap("panel").columns(2));
            }
            let p = builder.build()?;
            chart_core::plot::Plot::from_json_with_extensions(&p.to_json()?, registry())?
                .chart()?
                .prepare()
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "generated style{index}: {c}; {:?}",
            result.as_ref().err()
        );
        if let Ok(prepared) = result {
            let charts = if c["faceted"] == true {
                prepared
                    .panels()
                    .iter()
                    .map(|p| &p.chart)
                    .collect::<Vec<_>>()
            } else {
                vec![&prepared]
            };
            let panels = c["result"]["panels"].as_object().unwrap();
            assert_eq!(charts.len(), panels.len());
            for (chart, expected) in charts.iter().zip(panels.values()) {
                let wanted = expected["mapped"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| c["route"] == "paint" || !number(v).is_nan())
                    .collect::<Vec<_>>();
                let marks = chart.layers()[0].marks();
                assert_eq!(marks.len(), wanted.len(), "generated style{index}: {c}");
                for (mark, value) in marks.iter().zip(wanted) {
                    if c["route"] == "size" {
                        close(
                            mark.style.radius,
                            number(value),
                            &format!("generated style{index}: {c}"),
                        );
                    } else {
                        assert_eq!(
                            mark.style.color,
                            chart_core::color::parse_r(value.as_str().unwrap())
                                .unwrap()
                                .resolve(),
                            "generated style{index}: {c}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn identity_vector_transform_mapping_and_guides_preserve_layer_batches() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-vector-transforms.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 96);
    check_identity_vector_reference(&fixture);
}

#[test]
fn identity_vector_callbacks_preserve_vectors_and_hidden_demand() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/identity-vector-functions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 288);
    check_identity_vector_reference(&fixture);
}

fn check_identity_vector_reference(fixture: &Value) {
    use chart_core::{grammar::NumericAesthetic, interpolate::Number};
    let functions = fixture["cases"][0].get("limit_mode").is_some();
    for c in fixture["cases"].as_array().unwrap() {
        let limit_calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let break_calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let mut extensions = registry();
        let mutable = std::sync::Arc::get_mut(&mut extensions).unwrap();
        mutable
            .register_scale_limits(std::sync::Arc::new(IdentityLoggedLimits(
                limit_calls.clone(),
            )))
            .unwrap();
        mutable
            .register_scale_breaks(std::sync::Arc::new(IdentityLoggedBreaks(
                break_calls.clone(),
            )))
            .unwrap();
        let context = c.to_string();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect::<Vec<_>>();
        let result = (|| -> ChartResult<_> {
            let mut scale = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
                GgplotNumericIdentity {
                    transform: Some(ScaleTransform::Ggplot {
                        transform: transform(
                            &fixture["configurations"]
                                [c["configuration"].as_u64().unwrap() as usize],
                        ),
                    }),
                    limits: (c["limits"] == "fixed")
                        .then_some([Some(Number(0.)), Some(Number(10.))]),
                    guide: c["guide"] == "legend",
                    ..Default::default()
                },
            ));
            scale.training = ScaleTraining::Eligible;
            if c["limit_mode"] == "reverse" {
                let mut call = chart_extension_example::numeric_limits::operation("reverse");
                call.operation.id = "test.identity_limits".into();
                scale.limits_function = Some(Box::new(call));
            }
            if functions && c["break_mode"] != "auto" {
                scale.breaks_function = Some(Box::new(chart_core::grammar::ScaleBreaksOperation {
                    operation: OperationRef {
                        id: "test.identity_breaks".into(),
                        version: Revision::new(1),
                    },
                    parameters: c["break_mode"].clone(),
                }));
            }
            let data = |name: &str, values: Vec<f64>| {
                Data::columns()
                    .name(name)
                    .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                    .column("v", values)
                    .build()
            };
            let layer = || points().numeric_scale(NumericAesthetic::Size, "v", scale.clone());
            let mut builder = plot(data("first", inputs.clone())?)
                .extensions(extensions.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(layer().name("first"));
            if c["layers"] == "two" {
                builder = builder.layer(layer().name("second").data(data(
                    "second",
                    c["second"].as_array().unwrap().iter().map(number).collect(),
                )?));
            }
            let p = builder.build()?;
            let mut wire: Value = serde_json::from_str(&p.to_json()?).unwrap();
            assert_eq!(wire["version"], 55);
            if c["layers"] == "two" {
                // The portable definition explicitly shares one numeric scale identity.
                wire["definition"]["layers"][1]["numeric_scales"]["Size"]["id"] =
                    wire["definition"]["layers"][0]["numeric_scales"]["Size"]["id"].clone();
            }
            let restored = chart_core::plot::Plot::from_json_with_extensions(
                &wire.to_string(),
                extensions.clone(),
            )?;
            let original = restored.chart()?.prepare()?;
            let edited = restored
                .edit()
                .layer("first", layer())
                .build()?
                .chart()?
                .prepare()?;
            for (before, after) in original.layers().iter().zip(edited.layers()) {
                assert_eq!(
                    before.numeric_scales()[&NumericAesthetic::Size].id,
                    after.numeric_scales()[&NumericAesthetic::Size].id
                );
            }
            for layer in edited.layers() {
                let scale = MappedScale::for_numbers_with_registry(
                    layer.numeric_scales()[&NumericAesthetic::Size]
                        .scale
                        .clone(),
                    &extensions,
                )?;
                scale.continuous_guide_entries(4096, 16384)?;
            }
            Ok(edited)
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "{context}: {:?}",
            result.as_ref().err()
        );
        if functions {
            for (actual, expected) in [
                (&limit_calls, &c["limit_calls"]),
                (&break_calls, &c["break_calls"]),
            ] {
                let actual = actual.lock().unwrap();
                let expected = expected.as_array().unwrap();
                assert_eq!(
                    actual.is_empty(),
                    expected.is_empty(),
                    "{context}: {actual:?}"
                );
                for (values, is_null) in actual.iter() {
                    assert!(
                        expected.iter().any(|call| {
                            call["is_null"] == *is_null
                                && call["values"].as_array().unwrap().len() == values.len()
                                && values.iter().zip(call["values"].as_array().unwrap()).all(
                                    |(a, b)| {
                                        let b = number(b);
                                        (a.is_nan() && b.is_nan())
                                            || *a == b
                                            || (a - b).abs() <= 4e-14 * b.abs().max(1.)
                                    },
                                )
                        }),
                        "{context}: unexpected callback {values:?}, null={is_null}"
                    );
                }
            }
        }
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "{context}: {:?}",
            result.as_ref().err()
        );
        let Ok(prepared) = result else { continue };
        for (index, layer) in prepared.layers().iter().enumerate() {
            let spec = &layer.numeric_scales()[&NumericAesthetic::Size].scale;
            let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &spec.function else {
                panic!("identity")
            };
            let actual = if functions {
                identity.trained.into_iter().flatten().collect::<Vec<_>>()
            } else {
                identity.domain().unwrap().to_vec()
            };
            let expected = c["result"][if functions { "trained" } else { "limits" }]
                .as_array()
                .unwrap();
            assert_eq!(actual.len(), expected.len(), "{context}");
            for (a, b) in actual.iter().zip(expected) {
                close(a.0, number(b), &context);
            }
            let scale = MappedScale::for_numbers_with_registry(spec.clone(), &extensions).unwrap();
            let raw = if index == 0 {
                &c["inputs"]
            } else {
                &c["second"]
            };
            let mapped = scale
                .numeric_batch(
                    &raw.as_array()
                        .unwrap()
                        .iter()
                        .map(number)
                        .map(Some)
                        .collect::<Vec<_>>(),
                )
                .unwrap()
                .unwrap();
            assert_eq!(
                mapped.len(),
                c["result"]["mapped"][index].as_array().unwrap().len(),
                "{context}"
            );
            for (actual, expected) in mapped
                .iter()
                .zip(c["result"]["mapped"][index].as_array().unwrap())
            {
                let actual = match actual {
                    chart_core::interpolate::Value::Number(v) => v.0,
                    chart_core::interpolate::Value::Missing => f64::NAN,
                    _ => panic!("number"),
                };
                close(actual, number(expected), &context);
            }
            let guides = scale
                .continuous_guide_entries(4096, 16384)
                .unwrap()
                .unwrap();
            let visible = guides.iter().filter(|g| g.visible).collect::<Vec<_>>();
            let keys = c["result"]["keys"].as_array().unwrap();
            if keys.is_empty() {
                assert!(visible.is_empty(), "{context}: {visible:?}");
                continue;
            }
            let expected = keys[0]["values"].as_array().unwrap();
            assert_eq!(visible.len(), expected.len(), "{context}: {visible:?}");
            for (index, (actual, expected)) in visible.iter().zip(expected).enumerate() {
                close(actual.transformed.0, number(expected), &context);
                assert_eq!(
                    actual.mapped,
                    Some(chart_core::interpolate::Value::Number(Number(number(
                        &keys[0]["mapped"][index]
                    )))),
                    "{context}"
                );
                assert_eq!(
                    actual.label.as_deref(),
                    keys[0]["labels"][index].as_str(),
                    "{context}"
                );
            }
        }
    }
}

type IdentityCalls = std::sync::Arc<std::sync::Mutex<Vec<(Vec<f64>, bool)>>>;
struct IdentityLoggedLimits(IdentityCalls);
impl chart_core::grammar::CustomScaleLimits for IdentityLoggedLimits {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::ExtensionDescriptor::batch(
            "test.identity_limits",
            Revision::new(1),
            true,
        )
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        chart_extension_example::numeric_limits::Limits.validate(p)
    }
    fn evaluate(
        &self,
        input: chart_core::grammar::ScaleLimitsInput<'_>,
    ) -> ChartResult<Option<Vec<ScaleKey>>> {
        self.0.lock().unwrap().push((
            input
                .domain
                .unwrap_or_default()
                .iter()
                .map(|v| match v {
                    ScaleKey::Number(v) => v.0,
                    _ => panic!("number"),
                })
                .collect(),
            input.domain.is_none(),
        ));
        chart_extension_example::numeric_limits::Limits.evaluate(input)
    }
}
struct IdentityLoggedBreaks(IdentityCalls);
impl chart_core::grammar::CustomScaleBreaks for IdentityLoggedBreaks {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::ExtensionDescriptor::batch(
            "test.identity_breaks",
            Revision::new(1),
            true,
        )
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        chart_extension_example::scale_breaks::Breaks("limits").validate(p)
    }
    fn evaluate(
        &self,
        input: chart_core::grammar::ScaleBreaksInput<'_>,
    ) -> ChartResult<chart_core::grammar::ScaleBreaksOutput> {
        self.0.lock().unwrap().push((
            input
                .domain
                .iter()
                .map(|v| match v {
                    ScaleKey::Number(v) => v.0,
                    _ => panic!("number"),
                })
                .collect(),
            input.domain_is_null,
        ));
        chart_extension_example::scale_breaks::Breaks("limits").evaluate(input)
    }
}

#[test]
fn vector_scale_callbacks_preserve_layer_populations_and_guide_vectors() {
    check_vector_scale_callbacks(
        include_str!("../../../fixtures/parity/ggplot2/vector-scale-functions.json"),
        1152,
    );
}

#[test]
fn vector_null_breaks_preserve_transform_dispatch() {
    check_vector_scale_callbacks(
        include_str!("../../../fixtures/parity/ggplot2/vector-null-breaks.json"),
        384,
    );
}

fn check_vector_scale_callbacks(source: &str, expected_count: usize) {
    use chart_core::{grammar::NumericAesthetic, interpolate::Number};
    use std::sync::{Arc, Mutex};
    let fixture: Value = serde_json::from_str(source).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), expected_count);
    for (case_index, c) in cases.iter().enumerate() {
        let context = format!("case{case_index}: {c}");
        let limit_calls = Arc::new(Mutex::new(vec![]));
        let break_calls = Arc::new(Mutex::new(vec![]));
        let mut extensions = registry();
        let mutable = Arc::get_mut(&mut extensions).unwrap();
        mutable
            .register_scale_limits(Arc::new(IdentityLoggedLimits(limit_calls.clone())))
            .unwrap();
        mutable
            .register_scale_breaks(Arc::new(IdentityLoggedBreaks(break_calls.clone())))
            .unwrap();
        let paint = c["route"] == "paint";
        let result = (|| -> ChartResult<_> {
            let output = if paint {
                serde_json::json!({"Interpolate":{"operation":"GgplotPalette","spec":{"Gradient":{"colors":[{"red":19,"green":43,"blue":67,"alpha":255},{"red":86,"green":177,"blue":247,"alpha":255}],"values":null}}}})
            } else {
                serde_json::json!({"Interpolate":{"operation":"PowerRange","range":[1,6],"exponent":0.5,"absolute":false}})
            };
            let policy = if c["kind"] == "binned" {
                serde_json::json!({"Binned":{"limits":null,"oob":"Squish","breaks":{"Nice":5},"right":true}})
            } else {
                serde_json::json!({"Continuous":{"limits":null,"oob":"Censor"}})
            };
            let guide = if c["guide"] == "none" {
                serde_json::json!("Hidden")
            } else if c["kind"] == "binned" {
                serde_json::json!({"BinnedLegend":"Automatic"})
            } else {
                serde_json::json!({"Continuous":{"breaks":null,"labels":"Automatic"}})
            };
            let mut scale: MappedScaleSpec = serde_json::from_value(serde_json::json!({
                "training":"Eligible","function":{"Interpolated":{"normalization":{"Ggplot":{"family":{"Ggplot":{"transform":transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize])}},"domain":[0,1],"reverse":false,"rescaler":"Range"}},"output":output,"unknown":{"kind":"Missing"}}},"ggplot":policy,"guide":guide
            })).unwrap();
            if c["limit_mode"] == "reverse" {
                let mut call = chart_extension_example::numeric_limits::operation("reverse");
                call.operation.id = "test.identity_limits".into();
                scale.limits_function = Some(Box::new(call));
            }
            if c["break_mode"] != "auto" {
                scale.breaks_function = Some(Box::new(chart_core::grammar::ScaleBreaksOperation {
                    operation: OperationRef {
                        id: "test.identity_breaks".into(),
                        version: Revision::new(1),
                    },
                    parameters: c["break_mode"].clone(),
                }));
            }
            let data = |name: &str, values: &Value| {
                let values = values
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .collect::<Vec<_>>();
                Data::columns()
                    .name(name)
                    .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                    .column("v", values)
                    .build()
            };
            let layer = || {
                if paint {
                    points()
                } else {
                    points().numeric_scale(NumericAesthetic::Size, "v", scale.clone())
                }
            };
            let mut builder = plot(data("first", &c["inputs"])?)
                .extensions(extensions.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(if paint {
                    aes().x("x").y(1.).color("v").color_scale("paint")
                } else {
                    aes().x("x").y(1.)
                })
                .layer(layer().name("first"));
            if paint {
                builder = builder.scale(color_mapped("paint", scale.clone()));
            }
            if c["layers"] == "two" {
                builder = builder.layer(layer().name("second").data(data("second", &c["second"])?));
            }
            let p = builder.build()?;
            let mut wire: Value = serde_json::from_str(&p.to_json()?).unwrap();
            assert_eq!(wire["version"], 55);
            if !paint && c["layers"] == "two" {
                wire["definition"]["layers"][1]["numeric_scales"]["Size"]["id"] =
                    wire["definition"]["layers"][0]["numeric_scales"]["Size"]["id"].clone();
            }
            let restored = chart_core::plot::Plot::from_json_with_extensions(
                &wire.to_string(),
                extensions.clone(),
            )?;
            let prepared = restored.chart()?.prepare()?;
            let mut guides = vec![];
            for layer in prepared.layers() {
                if paint {
                    guides.push(
                        layer
                            .color_legend()
                            .map_or(vec![], |g| g.numeric_breaks.clone()),
                    );
                } else {
                    let mapped = MappedScale::for_numbers_with_registry(
                        layer.numeric_scales()[&NumericAesthetic::Size]
                            .scale
                            .clone(),
                        &extensions,
                    )?;
                    guides.push(
                        if c["kind"] == "binned" {
                            mapped.binned_guide_entries(4096, 16384)?
                        } else {
                            mapped.continuous_guide_entries(4096, 16384)?
                        }
                        .unwrap_or_default(),
                    );
                }
            }
            Ok((prepared, guides))
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "{context}: {:?}",
            result.as_ref().err()
        );
        for (actual, expected) in [
            (&limit_calls, &c["limit_calls"]),
            (&break_calls, &c["break_calls"]),
        ] {
            let actual = actual.lock().unwrap();
            let expected = expected.as_array().unwrap();
            assert_eq!(
                actual.is_empty(),
                expected.is_empty(),
                "{context}: {actual:?}"
            );
            for (values, is_null) in actual.iter() {
                assert!(
                    expected.iter().any(|call| call["is_null"] == *is_null
                        && call["values"].as_array().unwrap().len() == values.len()
                        && values
                            .iter()
                            .zip(call["values"].as_array().unwrap())
                            .all(|(a, b)| {
                                let b = number(b);
                                (a.is_nan() && b.is_nan())
                                    || *a == b
                                    || (a - b).abs() <= 4e-14 * b.abs().max(1.)
                            })),
                    "{context}: unexpected callback {values:?}, null={is_null}"
                );
            }
        }
        let Ok((prepared, guides)) = result else {
            continue;
        };
        for (index, layer) in prepared.layers().iter().enumerate() {
            let expected = c["result"]["mapped"][index]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| paint || !number(v).is_nan())
                .collect::<Vec<_>>();
            assert_eq!(layer.marks().len(), expected.len(), "{context}");
            for (mark, expected) in layer.marks().iter().zip(expected) {
                if paint {
                    assert_eq!(
                        mark.style.color,
                        chart_core::color::parse_r(expected.as_str().unwrap())
                            .unwrap()
                            .resolve(),
                        "{context}"
                    );
                } else {
                    close(mark.style.radius, number(expected), &context);
                }
            }
            let visible = guides[index]
                .iter()
                .filter(|g| g.visible)
                .collect::<Vec<_>>();
            let expected = c["result"]["keys"].as_array().unwrap().first();
            let values = expected.map_or(&[][..], |e| e["values"].as_array().unwrap().as_slice());
            assert_eq!(visible.len(), values.len(), "{context}: {visible:?}");
            for (i, (actual, value)) in visible.iter().zip(values).enumerate() {
                close(actual.transformed.0, number(value), &context);
                assert_eq!(
                    actual.label.as_deref(),
                    expected.unwrap()["labels"][i].as_str(),
                    "{context}"
                );
                if paint {
                    let Some(chart_core::interpolate::Value::Color(actual)) = &actual.mapped else {
                        panic!("{context}: guide paint")
                    };
                    assert_eq!(
                        chart_core::color::Paint::from(*actual).resolve(),
                        chart_core::color::parse_r(
                            expected.unwrap()["mapped"][i].as_str().unwrap()
                        )
                        .unwrap()
                        .resolve(),
                        "{context}"
                    );
                } else {
                    let Some(chart_core::interpolate::Value::Number(Number(actual))) =
                        &actual.mapped
                    else {
                        panic!("{context}: guide number")
                    };
                    close(*actual, number(&expected.unwrap()["mapped"][i]), &context);
                }
            }
        }
    }
}

#[test]
fn vector_position_callbacks_match_primary_vectors_and_guides() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-position-callbacks.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 72);
    check_vector_positional_transform_reference_plots(&fixture);
}

#[test]
fn vector_position_explicit_breaks_match_primary_vectors_and_guides() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-position-explicit.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 144);
    check_vector_positional_transform_reference_plots(&fixture);
}

type VectorMinorCalls = std::sync::Arc<std::sync::Mutex<Vec<(Vec<f64>, Vec<f64>)>>>;
struct VectorLoggedMinors(VectorMinorCalls);
impl chart_core::grammar::CustomScaleBreaks for VectorLoggedMinors {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::ExtensionDescriptor::batch(
            "test.vector_minors",
            Revision::new(1),
            true,
        )
    }
    fn accepts_major_breaks(&self) -> bool {
        true
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        chart_extension_example::scale_breaks::Breaks("minor_two").validate(p)
    }
    fn evaluate(
        &self,
        input: chart_core::grammar::ScaleBreaksInput<'_>,
    ) -> ChartResult<chart_core::grammar::ScaleBreaksOutput> {
        let numbers = |values: &[ScaleKey]| {
            values
                .iter()
                .map(|v| {
                    let ScaleKey::Number(n) = v else {
                        panic!("numeric minor input")
                    };
                    n.0
                })
                .collect()
        };
        self.0
            .lock()
            .unwrap()
            .push((numbers(input.domain), numbers(input.major_breaks.unwrap())));
        chart_extension_example::scale_breaks::Breaks("minor_two").evaluate(input)
    }
}
#[test]
fn vector_position_minor_breaks_match_primary_vectors_and_guides() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-position-minors.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 240);
    check_vector_positional_transform_reference_plots(&fixture);
}
