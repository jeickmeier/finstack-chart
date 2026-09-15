//! FIX-GG04 independent reference chart outcomes for registered pointwise transforms.
use chart_core::grammar::{
    CustomTransformFactory, ExtensionDescriptor, ExtensionRegistry, OperationRef,
    PointwiseTransform, TransformOperation, TransformSelection,
};
use serde_json::Value;
use std::sync::Arc;
struct Factory;
struct Kernel {
    cubic: bool,
    custom: bool,
}
impl PointwiseTransform for Kernel {
    fn forward(&self, x: f64) -> f64 {
        if self.cubic { x * x * x } else { 2. * x + 3. }
    }
    fn inverse(&self, x: f64) -> f64 {
        if self.cubic {
            if x == 0. {
                x
            } else {
                x.signum() * x.abs().powf(1. / 3.)
            }
        } else {
            (x - 3.) / 2.
        }
    }
    fn domain(&self) -> [chart_core::interpolate::Number; 2] {
        [f64::NEG_INFINITY, f64::INFINITY].map(chart_core::interpolate::Number)
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        true
    }
    fn breaks(
        &self,
        _: [f64; 2],
        _: f64,
    ) -> ChartResult<Option<Vec<chart_core::interpolate::Number>>> {
        Ok(self.custom.then(|| {
            [-2., -1., 0., 1., 2.]
                .map(chart_core::interpolate::Number)
                .to_vec()
        }))
    }
    fn labels(
        &self,
        values: &[chart_core::interpolate::Number],
    ) -> ChartResult<Option<Vec<Option<String>>>> {
        Ok(self.custom.then(|| {
            if values.is_empty() {
                vec![Some("value=".into())]
            } else {
                values
                    .iter()
                    .map(|v| {
                        Some(if v.0.is_nan() {
                            "value=NA".into()
                        } else {
                            format!("value={}", v.0)
                        })
                    })
                    .collect()
            }
        }))
    }
}
impl CustomTransformFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.reference_transform", Revision::new(1), true)
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        if matches!(p["family"].as_str(), Some("affine" | "cubic")) && p["custom"].is_boolean() {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Invalid reference transform.",
                "Use a captured configuration.",
            ))
        }
    }
    fn compile(&self, p: &Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(Kernel {
            cubic: p["family"] == "cubic",
            custom: p["custom"].as_bool().unwrap(),
        }))
    }
}
fn registry() -> Arc<ExtensionRegistry> {
    let mut r = ExtensionRegistry::new();
    r.register_transform(Arc::new(Factory)).unwrap();
    Arc::new(r)
}
fn transform(c: &Value) -> GgplotTransform {
    GgplotTransform::Registered {
        selection: TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "test.reference_transform".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!({"family":c["family"],"custom":c["custom"]}),
        })
        .into(),
    }
}
fn number(v: &Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| match v["number"].as_str().unwrap() {
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            "NA" | "NaN" => f64::NAN,
            s => panic!("Unknown number {s}"),
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
fn positional_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/registered-transforms.json"
    ))
    .unwrap();
    check_positional_transform_reference_plots(&fixture);
}
fn check_positional_transform_reference_plots(fixture: &Value) {
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
            .column("x", input)
            .column("y", vec![1.; c["inputs"].as_array().unwrap().len()])
            .build()
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let p = plot(data)
                .extensions(registry())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                            transform: t.clone(),
                        }))
                        .range(100., 540.)
                        .tick_arguments(c["count"].as_f64().map(|count| GuideTickArguments {
                            count: Some(count),
                            ..Default::default()
                        }))
                        .guide_geometry(Some(GuideGeometry {
                            labels: Some(GuideLabelPolicy::Preserve),
                            ..Default::default()
                        })),
                )
                .build()?;
            let wire = p.to_json()?;
            assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 55);
            let restored = chart_core::plot::Plot::from_json_with_extensions(&wire, registry())?;
            assert_eq!(restored.to_json()?, wire);
            let prepared = restored.chart()?.prepare()?;
            if let Some(mapped) = c["result"]["mapped"].as_array() {
                let expected: Vec<_> = mapped.iter().map(number).filter(|v| !v.is_nan()).collect();
                let actual: Vec<_> = prepared.layers()[0]
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
        if let Ok(frame) = result {
            let mut ticks: Vec<_> = frame.guides()[&GuideId::new(0)].ticks.iter().collect();
            ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut expected: Vec<_> = c["result"]["positions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(
                    c["result"]["panel_labels"]
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
    }
}

#[test]
fn paint_transform_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/registered-transforms.json"
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
            .column("value", input)
            .build()
            .unwrap();
        let result = (|| -> ChartResult<_> {
            let scale = paint_scale(t.clone(), c["route"] == "binned_paint")?;
            let p = plot(data)
                .extensions(registry())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("value").color_scale("paint"))
                .scale(color_mapped(
                    "paint",
                    scale.with_guide(GgplotScaleGuide::Hidden)?,
                ))
                .layer(points())
                .build()?;
            let wire = p.to_json()?;
            assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 55);
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
            let marks = prepared.layers()[0].marks();
            let wanted = c["result"]["mapped"].as_array().unwrap();
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

fn paint_scale(t: GgplotTransform, binned: bool) -> ChartResult<MappedScaleSpec> {
    let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(true)? else {
        unreachable!()
    };
    scale.palette_theme_aesthetics.clear();
    scale.training = ScaleTraining::Eligible;
    if binned {
        scale = scale.with_ggplot(GgplotScalePolicy::Binned(Box::default()))?;
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
    Ok(scale)
}

#[test]
fn direct_scale_query_outcomes_are_separate_from_chart_draws() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/registered-transforms.json"
    ))
    .unwrap();
    for (i, c) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["route"] != "position")
    {
        let t =
            transform(&fixture["configurations"][c["configuration"].as_u64().unwrap() as usize]);
        let values = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Some(chart_core::interpolate::Number(number(v))))
            .collect::<Vec<_>>();
        let r = registry();
        let binned = c["route"] == "binned_paint";
        let spec = paint_scale(t, binned)
            .unwrap()
            .trained_with_registry(&values, &r)
            .unwrap();
        let scale = MappedScale::new_with_registry(spec, &r).unwrap();
        let result = if binned {
            scale.binned_guide_entries(4096, 65536)
        } else {
            scale.continuous_guide_entries(4096, 65536)
        };
        assert_eq!(
            result.is_err(),
            c["result"]["labels"].get("error").is_some(),
            "query {i}: {result:?}, {c}"
        );
        if let Ok(Some(entries)) = result {
            let wanted = c["result"]["breaks"].as_array().unwrap();
            assert_eq!(entries.len(), wanted.len(), "query {i}: {c}");
            for (a, b) in entries.iter().zip(wanted) {
                close(a.transformed.0, number(b), &format!("query {i} break"));
            }
            assert_eq!(
                entries
                    .iter()
                    .map(|v| v.label.as_deref())
                    .collect::<Vec<_>>(),
                c["result"]["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(Value::as_str)
                    .collect::<Vec<_>>(),
                "query {i}: {c}"
            );
        }
    }
}
