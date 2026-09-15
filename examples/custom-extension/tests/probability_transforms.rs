//! FIX-GG04 quantile/CDF registration, chart training and guide visibility.
use chart_core::grammar::{
    ExtensionRegistry, OperationRef, TransformOperation, TransformSelection,
};
use serde_json::Value;
use std::sync::Arc;
fn registry() -> Arc<ExtensionRegistry> {
    chart_extension_example::registry().unwrap()
}
fn transform(c: &Value) -> GgplotTransform {
    GgplotTransform::Registered {
        selection: TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "example.probability_transform".into(),
                version: Revision::new(1),
            },
            parameters: if c["family"] == "unif" { serde_json::json!({"distribution":"Uniform","min":if c["custom"] == true { -2. } else { 0. },"max":if c["custom"] == true { 3. } else { 1. }}) } else { serde_json::json!({"distribution":"Exponential","rate":if c["custom"] == true { 2. } else { 0.5 }}) },
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
        "../../../fixtures/parity/ggplot2/probability-transforms.json"
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
        "../../../fixtures/parity/ggplot2/probability-transforms.json"
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
                    if c["route"] == "binned_paint" {
                        scale
                    } else {
                        scale.with_guide(GgplotScaleGuide::Hidden)?
                    },
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
fn guide_candidates_match_reference_with_explicit_visibility_adaptation() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/probability-transforms.json"
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
                    .enumerate()
                    .map(|(index, v)| {
                        let cut = number(&c["result"]["breaks"][index]);
                        let bounds = c["result"]["limits"].as_array().unwrap();
                        if !binned && (cut < number(&bounds[0]) || cut > number(&bounds[1])) {
                            None
                        } else {
                            v.as_str()
                        }
                    })
                    .collect::<Vec<_>>(),
                "query {i}: {c}"
            );
        }
    }
}

#[test]
fn probability_quantiles_and_cdfs_match_reference_boundaries() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/probability-transforms.json"
    ))
    .unwrap();
    for c in fixture["configurations"].as_array().unwrap() {
        let t = registry().resolve_transform(&transform(c)).unwrap();
        assert_eq!(t.domain(), [0., 1.]);
        for (i, x) in c["inputs"].as_array().unwrap().iter().enumerate() {
            close(t.forward(number(x)), number(&c["quantiles"][i]), "quantile");
            close(t.inverse(number(x)), number(&c["probabilities"][i]), "cdf");
        }
    }
}
