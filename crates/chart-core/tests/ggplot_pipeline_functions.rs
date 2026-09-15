//! FIX-GG04: complete OOB/rescale vector calls from public reference builds.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
fn encode_number(v: f64) -> Json {
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
fn numbers(v: &[Number]) -> Json {
    json!(v.iter().map(|v| encode_number(v.0)).collect::<Vec<_>>())
}
fn input(v: &Json) -> Option<f64> {
    match v.as_str() {
        Some("Infinity") => Some(f64::INFINITY),
        Some("-Infinity") => Some(f64::NEG_INFINITY),
        _ => v.as_f64(),
    }
}
struct Vector(Arc<Mutex<Vec<Json>>>);
impl CustomScaleVector for Vector {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.vector", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
        let operation = match i.stage {
            ScaleVectorStage::OutOfBounds => "oob",
            ScaleVectorStage::Rescale => "rescaler",
        };
        self.0.lock().unwrap().push(json!({"operation":operation,"values":numbers(i.values),"range":numbers(i.limits),"names":[]}));
        let mode = i.parameters["mode"].as_str().unwrap();
        let mut values = if i.stage == ScaleVectorStage::OutOfBounds {
            match mode {
                "oob_reverse" => i.values.iter().rev().copied().collect(),
                "oob_index" => (1..=i.values.len()).map(|v| Number(v as f64)).collect(),
                _ => i
                    .values
                    .iter()
                    .map(|v| {
                        Number(if i.parameters["family"] == "binned" && v.0.is_finite() {
                            v.0.clamp(i.limits[0].0, i.limits[1].0)
                        } else if v.0.is_finite() && (v.0 < i.limits[0].0 || v.0 > i.limits[1].0) {
                            f64::NAN
                        } else {
                            v.0
                        })
                    })
                    .collect(),
            }
        } else {
            if mode == "rescale_null" {
                return Ok(None);
            }
            if mode == "rescale_empty" {
                return Ok(Some(vec![]));
            }
            if mode == "rescale_index" {
                (1..=i.values.len())
                    .map(|v| Number(v as f64 / i.values.len() as f64))
                    .collect()
            } else {
                i.values
                    .iter()
                    .map(|v| Number((v.0 - i.limits[0].0) / (i.limits[1].0 - i.limits[0].0)))
                    .collect::<Vec<_>>()
            }
        };
        if i.stage == ScaleVectorStage::Rescale {
            if mode == "rescale_reverse" {
                values.reverse();
            }
            if mode == "rescale_short" {
                values.truncate(1);
            }
        }
        Ok(Some(values))
    }
}
struct Palette(Arc<Mutex<Vec<Json>>>);
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Normalized(v) = i.domain else {
            panic!()
        };
        self.0
            .lock()
            .unwrap()
            .push(json!({"operation":"palette","values":numbers(v),"range":[],"names":[]}));
        Ok(ScalePaletteOutput {
            names: None,
            values: Some(
                v.iter()
                    .map(|v| {
                        if v.0.is_nan() {
                            Value::Missing
                        } else {
                            match i.parameters["channel"].as_str().unwrap() {
                                "colour" => Value::Text(
                                    if v.0 < 0.5 { "#ff0000" } else { "#0000ff" }.into(),
                                ),
                                "size" => Value::number(1. + 4. * v.0),
                                _ => Value::number(v.0),
                            }
                        }
                    })
                    .collect(),
            ),
        })
    }
}
struct Labels(Arc<Mutex<Vec<Json>>>);
impl CustomGuideFormatter for Labels {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.pipeline_labels", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn format_labels(&self, i: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        let values = i
            .values
            .iter()
            .map(|v| match v {
                chart_core::composition::ScaleValue::Number(v) => encode_number(*v),
                _ => panic!("numeric labels"),
            })
            .collect::<Vec<_>>();
        self.0
            .lock()
            .unwrap()
            .push(json!({"operation":"labels", "values":values, "range":[], "names":[]}));
        let mut labels = if values.is_empty() {
            vec![Some("/0".into())]
        } else {
            (1..=values.len())
                .map(|n| Some(format!("{n}/{}", values.len())))
                .collect::<Vec<_>>()
        };
        if i.parameters == "short" {
            labels.truncate(1);
        }
        if i.parameters == "missing" {
            for (n, label) in labels.iter_mut().enumerate() {
                if n % 2 == 1 {
                    *label = None;
                }
            }
        }
        Ok(labels)
    }
}
fn spec(c: &Json) -> MappedScaleSpec {
    let labels = if c["label_mode"].is_null() {
        GgplotGuideLabels::Automatic
    } else {
        GgplotGuideLabels::Registered {
            operation: OperationRef::new("test.pipeline_labels", Revision::new(1)),
            parameters: c["label_mode"].clone(),
        }
    };
    let call = ScaleVectorOperation {
        operation: OperationRef {
            id: "test.vector".into(),
            version: Revision::new(1),
        },
        parameters: json!({"mode":c["mode"], "family":c["family"]}),
    };
    MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Ggplot {
            timestamp: None,
            family: NumericFamily::Linear,
            domain: [Number(1.), Number(10.)],
            reverse: false,
            rescaler: GgplotRescaler::Range,
        },
        output: ScaleRangeFunction::Identity,
        unknown: Value::Missing,
    }))
    .with_ggplot(if c["family"] == "binned" {
        GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
            limits: Some([Some(Number(1.)), Some(Number(10.))]),
            ..Default::default()
        }))
    } else {
        GgplotScalePolicy::Continuous {
            empty_population: false,
            nonfinite_population: false,
            limits: Some([Some(Number(1.)), Some(Number(10.))]),
            oob: GgplotOob::Censor,
        }
    })
    .unwrap()
    .with_guide(if c["guide_mode"] == "hidden" {
        GgplotScaleGuide::Hidden
    } else if c["guide_mode"] == "bins" {
        GgplotScaleGuide::BinnedBins(labels.clone())
    } else if c["guide_mode"] == "coloursteps" {
        GgplotScaleGuide::BinnedSteps(labels)
    } else if c["family"] == "binned" {
        GgplotScaleGuide::BinnedLegend(GgplotGuideLabels::Automatic)
    } else {
        GgplotScaleGuide::Continuous(Default::default())
    })
    .unwrap()
    .with_oob_function(call.clone())
    .unwrap()
    .with_rescaler_function(call)
    .unwrap()
    .with_palette_function(ScalePaletteOperation {
        operation: OperationRef {
            id: "test.palette".into(),
            version: Revision::new(1),
        },
        parameters: json!({"channel":c["channel"]}),
    })
    .unwrap()
}
fn encode(v: Value) -> Json {
    match v {
        Value::Missing => Json::Null,
        Value::Number(v) => encode_number(v.0),
        Value::Text(v) => json!(v),
        _ => panic!(),
    }
}
fn compare(actual: &Json, expected: &Json) {
    match (actual, expected) {
        (Json::Number(a), Json::Number(b)) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 5e-14,
            "{actual} != {expected}"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                compare(a, b);
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                compare(v, &b[k]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn pipelines_match_288_public_builds_and_complete_call_sequences() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-pipeline-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 288);
    for c in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        registry
            .register_scale_palette(Arc::new(Palette(calls.clone())))
            .unwrap();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(input)
            .collect::<Vec<_>>();
        let result = (|| -> ChartResult<Json> {
            let trained = spec(c).trained_with_registry(
                &inputs.iter().map(|v| v.map(Number)).collect::<Vec<_>>(),
                &registry,
            )?;
            let scale = MappedScale::new_with_registry(trained, &registry)?;
            let entries = if c["family"] == "binned" {
                scale.binned_guide_entries(4096, 65536)?
            } else {
                scale.continuous_guide_entries(4096, 65536)?
            }
            .unwrap_or_default();
            let keys = if entries.is_empty() {
                json!([])
            } else {
                let visible = entries.iter().filter(|e| e.visible).collect::<Vec<_>>();
                json!([{"values":visible.iter().map(|e|encode_number(e.value.0)).collect::<Vec<_>>(),"labels":visible.iter().map(|e|e.label.clone()).collect::<Vec<_>>(),"mapped":visible.iter().map(|e|encode(e.mapped.clone().unwrap())).collect::<Vec<_>>()}])
            };
            let mut mapped = scale
                .numeric_batch(&inputs)?
                .unwrap_or_default()
                .into_iter()
                .map(encode)
                .collect::<Vec<_>>();
            if mapped.is_empty() {
                mapped = vec![Json::Null; inputs.len()];
            } else if mapped.len() == 1 {
                mapped.resize(inputs.len(), mapped[0].clone());
            }
            Ok(json!({"mapped":mapped,"keys":keys}))
        })();
        if c["result"].get("error").is_some() {
            assert!(result.is_err(), "{c}: {result:?}");
        } else {
            compare(
                &result.unwrap_or_else(|e| panic!("{c}: {e:?}")),
                &c["result"],
            );
        }
        compare(&json!(*calls.lock().unwrap()), &c["calls"]);
    }
}

#[test]
fn primary_pipelines_preserve_all_288_draw_outcomes_and_special_sizes() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-pipeline-draws.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 288);
    for c in cases {
        check_primary_pipeline(c);
    }
}

struct IdentityLimits;
impl CustomScaleLimits for IdentityLimits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.identity_limits", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
        Ok(i.domain.map(<[ScaleKey]>::to_vec))
    }
}

#[test]
fn binned_hidden_and_legend_constructors_match_216_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guide-selection.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["guide_kind"] != "none" && c["guide_kind"] != "legend" {
            continue;
        }
        let mut c = c.clone();
        c["guide_mode"] = json!(if c["guide_kind"] == "none" {
            "hidden"
        } else {
            "legend"
        });
        check_primary_pipeline(&c);
        count += 1;
    }
    assert_eq!(count, 216);
}

#[test]
fn binned_bins_constructors_match_108_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guide-selection.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["guide_kind"] != "bins" {
            continue;
        }
        let mut c = c.clone();
        c["guide_mode"] = c["guide_kind"].clone();
        check_primary_pipeline(&c);
        count += 1;
    }
    assert_eq!(count, 108);
}

#[test]
fn binned_default_constructors_match_108_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guide-selection.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["guide_kind"] != "default" {
            continue;
        }
        let mut c = c.clone();
        c["guide_mode"] = json!("bins");
        check_primary_pipeline(&c);
        count += 1;
    }
    assert_eq!(count, 108);
}

#[test]
fn binned_coloursteps_constructors_match_108_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guide-selection.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["guide_kind"] != "coloursteps" {
            continue;
        }
        let mut c = c.clone();
        c["guide_mode"] = c["guide_kind"].clone();
        check_primary_pipeline(&c);
        count += 1;
    }
    assert_eq!(count, 108);
}

#[test]
fn binned_registered_labels_match_108_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-guide-label-pipelines.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let mut c = c.clone();
        c["guide_mode"] = c["guide_kind"].clone();
        check_primary_pipeline(&c);
        count += 1;
    }
    assert_eq!(count, 108);
}

fn check_primary_pipeline(c: &Json) {
    use chart_core::prelude::*;
    let calls = Arc::new(Mutex::new(vec![]));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_guide_formatter(Arc::new(Labels(calls.clone())))
        .unwrap();
    registry
        .register_scale_vector(Arc::new(Vector(calls.clone())))
        .unwrap();
    registry
        .register_scale_palette(Arc::new(Palette(calls.clone())))
        .unwrap();
    registry
        .register_scale_limits(Arc::new(IdentityLimits))
        .unwrap();
    let registry = Arc::new(registry);
    let inputs = c["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(input)
        .collect::<Vec<_>>();
    let data = Data::columns()
        .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
        .column("v", inputs)
        .build()
        .unwrap();
    let draft = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(registry.clone());
    let mut scale = spec(c);
    if c["limit_mode"] == "automatic" || c["limit_mode"] == "callback" {
        let Some(GgplotScalePolicy::Binned(policy)) = scale.ggplot.as_deref_mut() else {
            panic!("binned constructor fixture");
        };
        policy.limits = None;
    }
    if c["limit_mode"] == "callback" {
        scale = scale
            .with_limits_function(ScaleLimitsOperation {
                operation: OperationRef {
                    id: "test.identity_limits".into(),
                    version: Revision::new(1),
                },
                parameters: json!({}),
            })
            .unwrap();
    }
    scale.missing_paint_is_na = c["channel"] == "colour";
    let p = if c["channel"] == "colour" {
        draft
            .aes(aes().x("x").y(1.).color("v").color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
            .build()
            .unwrap()
    } else {
        draft
            .aes(aes().x("x").y(1.))
            .layer(points().numeric_scale(
                if c["channel"] == "size" {
                    NumericAesthetic::Size
                } else {
                    NumericAesthetic::Alpha
                },
                "v",
                scale,
            ))
            .build()
            .unwrap()
    };
    let wire = p.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<Json>(&wire).unwrap()["version"],
        if c["guide_mode"] == "bins" || c["guide_mode"] == "coloursteps" {
            44
        } else if c["family"] == "binned" && c["guide_mode"] != "hidden" {
            39
        } else {
            38
        }
    );
    if c["family"] == "binned" && c["guide_mode"] != "hidden" {
        let mut old: Json = serde_json::from_str(&wire).unwrap();
        old["version"] = json!(
            if c["guide_mode"] == "bins" || c["guide_mode"] == "coloursteps" {
                43
            } else {
                38
            }
        );
        assert!(Plot::from_json_with_extensions(&old.to_string(), registry.clone()).is_err());
    }
    assert!(calls.lock().unwrap().is_empty());
    assert!(Plot::from_json(&wire).is_err());
    let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    let result = restored.chart().unwrap().prepare();
    if c["result"].get("error").is_some() {
        assert!(result.is_err(), "{c}");
        return;
    }
    let prepared = result.unwrap_or_else(|e| panic!("{c}: {e:?}"));
    if !c["guide_kind"].is_null() {
        compare(&json!(*calls.lock().unwrap()), &c["calls"]);
    }
    let layer = &prepared.layers()[0];
    let entries = if c["channel"] == "colour" {
        layer
            .color_legend()
            .map(|g| g.numeric_breaks.clone())
            .unwrap_or_default()
    } else {
        layer
            .numeric_value_guides()
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
    };
    let visible = entries.iter().filter(|e| e.visible).collect::<Vec<_>>();
    let mut boundaries = visible.iter().map(|e| e.transformed.0).collect::<Vec<_>>();
    if c["guide_mode"] == "coloursteps" && !visible.is_empty() {
        let calls = calls.lock().unwrap();
        boundaries.extend(
            calls[0]["range"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| input(v).unwrap()),
        );
        boundaries.sort_by(f64::total_cmp);
        boundaries.dedup_by(|a, b| *a == *b);
    }
    let keys = if visible.is_empty() {
        json!([])
    } else {
        json!([{"values":visible.iter().enumerate().map(|(i,e)|encode_number(if c["guide_mode"] == "bins" { i as f64 / (visible.len()-1) as f64 } else if c["guide_mode"] == "coloursteps" { boundaries.iter().position(|v| *v == e.transformed.0).unwrap() as f64 / (boundaries.len()-1) as f64 } else { e.value.0 })).collect::<Vec<_>>(),"labels":visible.iter().map(|e|e.label.clone()).collect::<Vec<_>>(),"mapped":visible.iter().map(|e|encode(e.mapped.clone().expect("retained guide batch"))).collect::<Vec<_>>()}])
    };
    compare(&keys, &c["result"]["keys"]);
    let marks = prepared.layers()[0].marks();
    assert_eq!(
        marks.len(),
        c["result"]["point_count"].as_u64().unwrap() as usize,
        "{c}"
    );
    let colors = c["result"]["point_colours"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            chart_core::color::parse_r(v.as_str().unwrap())
                .unwrap()
                .resolve()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        marks.iter().map(|m| m.style.color).collect::<Vec<_>>(),
        colors,
        "{c}"
    );
    if c["channel"] == "size" {
        compare(
            &json!(
                marks
                    .iter()
                    .map(|m| encode_number(m.style.radius))
                    .collect::<Vec<_>>()
            ),
            &json!(
                c["result"]["mapped"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| !v.is_null())
                    .collect::<Vec<_>>()
            ),
        );
        for mark in marks.iter().filter(|m| m.style.radius.is_infinite()) {
            let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
                panic!("infinite size must have a bounded glyph")
            };
            assert!(geometry.commands().is_empty());
            let value = serde_json::to_value(mark.style).unwrap();
            assert!(value["radius"]["number"].is_string());
        }
    }
}

#[test]
fn nonfinite_sizes_preserve_rows_but_paint_no_reference_glyphs() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/point-nonfinite-sizes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 78);
    for c in cases {
        let size = input(&c["size"]).unwrap();
        let mut identity = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                transform: None,
                limits: None,
                guide: false,
                trained: None,
                has_population: false,
            },
        ));
        identity.training = ScaleTraining::Eligible;
        let p = plot(
            Data::columns()
                .column("x", vec![1.])
                .column("size", vec![size])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.))
        .layer(
            points()
                .numeric_scale(NumericAesthetic::Size, "size", identity)
                .aesthetic_value(
                    ValueAesthetic::Shape,
                    Value::number(c["shape"].as_f64().unwrap()),
                ),
        )
        .build()
        .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), c["result"]["rows"].as_u64().unwrap() as usize);
        assert_eq!(marks[0].style.radius, size);
        let PreparedGeometry::ShapePath { geometry, .. } = &marks[0].geometry else {
            panic!()
        };
        assert_eq!(
            geometry.commands().is_empty(),
            c["result"]["ink_pixels"] == 0,
            "{c}"
        );
        let serialized = serde_json::to_value(marks[0].style).unwrap();
        let expected = if size.is_finite() {
            c["size"].clone()
        } else {
            json!({"number":c["size"]})
        };
        compare(&serialized["radius"], &expected);
    }
}

#[test]
fn vector_results_are_bounded_and_duplicate_registration_does_not_replace_them() {
    struct Oversized;
    impl CustomScaleVector for Oversized {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("test.vector", Revision::new(1), true)
        }
        fn validate(&self, _: &Json) -> ChartResult<()> {
            Ok(())
        }
        fn evaluate(&self, _: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
            Ok(Some(vec![Number(0.); 200_001]))
        }
    }
    let mut registry = ExtensionRegistry::new();
    registry.register_scale_vector(Arc::new(Oversized)).unwrap();
    assert_eq!(
        registry
            .register_scale_vector(Arc::new(Vector(Arc::new(Mutex::new(vec![])))))
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::SchemaConflict
    );
    registry
        .register_scale_palette(Arc::new(Palette(Arc::new(Mutex::new(vec![])))))
        .unwrap();
    let c = json!({"mode":"default","channel":"alpha","guide_mode":"hidden"});
    let trained = spec(&c)
        .trained_with_registry(&[Some(Number(1.))], &registry)
        .unwrap();
    let scale = MappedScale::new_with_registry(trained, &registry).unwrap();
    assert_eq!(
        scale.numeric_batch(&[Some(1.)]).unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
}

#[test]
fn reference_alpha_matches_all_765_byte_boundary_cases() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/alpha-byte-rounding.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 255);
    for c in cases {
        let identity = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                transform: None,
                limits: None,
                guide: false,
                trained: None,
                has_population: false,
            },
        ));
        let p = plot(
            Data::columns()
                .column("x", vec![0., 1., 2.])
                .column(
                    "alpha",
                    c["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().parse::<f64>().unwrap())
                        .collect::<Vec<_>>(),
                )
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.))
        .layer(points().numeric_scale(NumericAesthetic::Alpha, "alpha", identity))
        .build()
        .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let actual = prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| m.style.color.alpha)
            .collect::<Vec<_>>();
        let expected = c["colours"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                chart_core::color::parse_r(v.as_str().unwrap())
                    .unwrap()
                    .resolve()
                    .alpha
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{c}");
    }
}

#[test]
fn binned_prepared_clones_share_palette_cache_and_empty_batches_skip_callbacks() {
    let c = json!({"family":"binned", "channel":"size", "mode":"default", "guide_mode":"hidden"});
    let calls = Arc::new(Mutex::new(vec![]));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_vector(Arc::new(Vector(calls.clone())))
        .unwrap();
    registry
        .register_scale_palette(Arc::new(Palette(calls.clone())))
        .unwrap();
    let trained = spec(&c)
        .trained_with_registry(&[Some(Number(1.)), Some(Number(10.))], &registry)
        .unwrap();
    let scale = MappedScale::new_with_registry(trained, &registry).unwrap();
    let cloned = scale.clone();
    assert!(scale.numeric_batch(&[]).unwrap().unwrap().is_empty());
    assert!(calls.lock().unwrap().is_empty());
    let expected = scale.numeric_batch(&[Some(1.), Some(10.)]).unwrap();
    assert_eq!(
        cloned.numeric_batch(&[Some(1.), Some(10.)]).unwrap(),
        expected
    );
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls.iter().filter(|c| c["operation"] == "palette").count(),
        1
    );
    assert_eq!(calls.iter().filter(|c| c["operation"] == "oob").count(), 2);
    assert_eq!(
        calls
            .iter()
            .filter(|c| c["operation"] == "rescaler")
            .count(),
        4
    );
}

#[test]
fn scalar_pipeline_sampling_uses_selected_bins_and_single_row_assignment() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-pipeline-singletons.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 96);
    for c in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        registry
            .register_scale_palette(Arc::new(Palette(calls)))
            .unwrap();
        let value = input(&c["inputs"][0]);
        let trained = spec(c)
            .trained_with_registry(&[value.map(Number)], &registry)
            .unwrap();
        let scale = MappedScale::new_with_registry(trained, &registry).unwrap();
        let result = scale.numeric(value);
        if c["result"].get("error").is_some() {
            assert!(result.is_err(), "{c}");
            continue;
        }
        compare(&encode(result.unwrap()), &c["result"]["mapped"][0]);
        if c["channel"] == "colour" {
            let missing = chart_core::color::parse_r("NA").unwrap();
            let expected = c["result"]["mapped"][0].as_str().unwrap_or("NA");
            assert_eq!(
                scale.paint(value, None, missing).unwrap().resolve(),
                chart_core::color::parse_r(expected).unwrap().resolve(),
                "{c}"
            );
        }
    }
}
