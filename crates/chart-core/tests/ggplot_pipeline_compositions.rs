//! FIX-GG04: callback limits retain their transformed order and full arity.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};

struct Limits;
impl CustomScaleLimits for Limits {
    fn requires_domain(&self, _: &Json) -> bool {
        false
    }
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("composition.limits", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
        Ok(match i.parameters.as_str().unwrap() {
            "singleton" => Some(vec![ScaleKey::Number(Number(4.))]),
            "empty" => Some(vec![]),
            "null" => None,
            "missing" => Some(vec![ScaleKey::Null, ScaleKey::Number(Number(10.))]),
            _ => unreachable!(),
        })
    }
}
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
fn equivalent(a: &Json, b: &Json) -> bool {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => {
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 5e-14
        }
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
        }
        (Json::Object(a), Json::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, a)| b.get(k).is_some_and(|b| equivalent(a, b)))
        }
        _ => a == b,
    }
}
struct Vector(Arc<Mutex<Vec<Json>>>);
impl CustomScaleVector for Vector {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("composition.vector", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
        self.0.lock().unwrap().push(json!({
            "operation": if i.stage == ScaleVectorStage::OutOfBounds { "oob" } else { "rescaler" },
            "values": i.values.iter().map(|v|encode(v.0)).collect::<Vec<_>>(),
            "range": i.limits.iter().map(|v|encode(v.0)).collect::<Vec<_>>(), "names": []
        }));
        Ok(Some(if i.stage == ScaleVectorStage::Rescale {
            if matches!(i.parameters.as_str(), Some("rescale_index" | "both_index")) {
                (1..=i.values.len())
                    .map(|v| Number(v as f64 / i.values.len() as f64))
                    .collect()
            } else {
                let a = i.limits.first().map_or(f64::NAN, |v| v.0);
                let b = i.limits.get(1).map_or(a, |v| v.0);
                if i.limits.is_empty() || (i.limits.len() == 2 && (a.is_nan() || b.is_nan())) {
                    return Err(chart_core::Diagnostic::error(
                        chart_core::DiagnosticCode::NumericalDomain,
                        "Reference callback requires comparable limits.",
                        "Use valid limits.",
                    ));
                }
                let constant = i.limits.len() == 1
                    || a == b
                    || (a != 0.
                        && b != 0.
                        && (a - b).abs() / a.abs().min(b.abs()) < 1000. * f64::EPSILON);
                i.values
                    .iter()
                    .map(|v| {
                        Number(if v.0.is_nan() {
                            f64::NAN
                        } else if constant {
                            0.5
                        } else {
                            (v.0 - a) / (b - a)
                        })
                    })
                    .collect()
            }
        } else if matches!(i.parameters.as_str(), Some("oob_index" | "both_index")) {
            (1..=i.values.len()).map(|v| Number(v as f64)).collect()
        } else {
            let a = i.limits.first().map_or(f64::NAN, |v| v.0);
            let b = i.limits.get(1).map_or(f64::NAN, |v| v.0);
            i.values
                .iter()
                .map(|v| {
                    Number(if v.0.is_finite() && (v.0 < a || v.0 > b) {
                        f64::NAN
                    } else {
                        v.0
                    })
                })
                .collect()
        }))
    }
}
struct Palette;
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("composition.palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, i: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let ScalePaletteDomain::Normalized(values) = i.domain else {
            unreachable!()
        };
        Ok(ScalePaletteOutput {
            names: None,
            values: Some(
                values
                    .iter()
                    .map(|v| {
                        if v.0.is_nan() {
                            Value::Missing
                        } else {
                            match i.parameters.as_str().unwrap() {
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
fn operation(id: &str) -> OperationRef {
    OperationRef {
        id: format!("composition.{id}"),
        version: Revision::new(1),
    }
}

fn spec(c: &Json) -> MappedScaleSpec {
    let limits = match c["limit_mode"].as_str().unwrap() {
        "automatic" => None,
        "full" => Some([Some(Number(1.)), Some(Number(10.))]),
        "reversed" => Some([Some(Number(10.)), Some(Number(1.))]),
        "constant" => Some([Some(Number(4.)), Some(Number(4.))]),
        _ => None,
    };
    let mut spec =
        MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
            normalization: NormalizationSpec::Ggplot {
                timestamp: None,
                family: if c["transform"] == "log10" {
                    NumericFamily::Log { base: 10. }
                } else {
                    NumericFamily::Linear
                },
                domain: [Number(1.), Number(10.)],
                reverse: c["transform"] == "reverse",
                rescaler: GgplotRescaler::Range,
            },
            output: ScaleRangeFunction::Identity,
            unknown: Value::Missing,
        }))
        .with_ggplot(GgplotScalePolicy::Continuous {
            limits,
            oob: GgplotOob::Censor,
            empty_population: false,
            nonfinite_population: false,
        })
        .unwrap()
        .with_guide(if c["guide_mode"] == "hidden" {
            GgplotScaleGuide::Hidden
        } else {
            GgplotScaleGuide::Continuous(Default::default())
        })
        .unwrap()
        .with_oob_function(ScaleVectorOperation {
            operation: operation("vector"),
            parameters: c["mode"].clone(),
        })
        .unwrap()
        .with_rescaler_function(ScaleVectorOperation {
            operation: operation("vector"),
            parameters: c["mode"].clone(),
        })
        .unwrap()
        .with_palette_function(ScalePaletteOperation {
            operation: operation("palette"),
            parameters: c["channel"].clone(),
        })
        .unwrap();
    if matches!(
        c["limit_mode"].as_str(),
        Some("singleton" | "empty" | "null" | "missing")
    ) {
        spec = spec
            .with_limits_function(ScaleLimitsOperation {
                operation: operation("limits"),
                parameters: c["limit_mode"].clone(),
            })
            .unwrap();
    }
    spec
}

#[test]
fn index_rescalers_can_ignore_empty_singleton_and_missing_limits() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-pipeline-compositions.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| {
            c["population"] == "ordinary"
                && c["guide_mode"] == "hidden"
                && matches!(c["mode"].as_str(), Some("rescale_index" | "both_index"))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 144);
    let mut failures = vec![];
    for c in cases {
        let calls = Arc::new(Mutex::new(vec![]));
        let mut registry = ExtensionRegistry::new();
        registry.register_scale_limits(Arc::new(Limits)).unwrap();
        registry
            .register_scale_vector(Arc::new(Vector(calls.clone())))
            .unwrap();
        registry.register_scale_palette(Arc::new(Palette)).unwrap();
        let spec = spec(c);
        let inputs = [Some(1.), Some(4.), Some(10.)];
        let result = spec
            .trained_with_registry(&inputs.map(|v| v.map(Number)), &registry)
            .and_then(|s| MappedScale::new_with_registry(s, &registry))
            .and_then(|s| s.numeric_batch(&inputs));
        let actual = result.map(|v| {
            json!(
                v.unwrap()
                    .iter()
                    .map(|v| match v {
                        Value::Number(v) => encode(v.0),
                        Value::Text(v) => json!(v),
                        Value::Missing => Json::Null,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>()
            )
        });
        let expected_calls = c["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["operation"] != "palette")
            .cloned()
            .collect::<Vec<_>>();
        let mapped_ok = if c["result"].get("error").is_some() {
            actual.is_err()
        } else {
            actual
                .as_ref()
                .is_ok_and(|a| equivalent(a, &c["result"]["mapped"]))
        };
        if !mapped_ok || !equivalent(&json!(*calls.lock().unwrap()), &json!(expected_calls)) {
            failures.push(format!(
                "{}/{}/{}/{}: {actual:?}; calls {:?}, expected {expected_calls:?}",
                c["transform"],
                c["limit_mode"],
                c["channel"],
                c["mode"],
                calls.lock().unwrap()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} composition failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_compositions_preserve_primary_marks_and_guide_keys() {
    use chart_core::prelude::*;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/continuous-pipeline-compositions.json"
    ))
    .unwrap();
    let cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 1728);
    let mut failures = vec![];
    for c in cases {
        let mut registry = ExtensionRegistry::new();
        registry.register_scale_limits(Arc::new(Limits)).unwrap();
        registry
            .register_scale_vector(Arc::new(Vector(Arc::new(Mutex::new(vec![])))))
            .unwrap();
        registry.register_scale_palette(Arc::new(Palette)).unwrap();
        let inputs = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| match v.as_str() {
                Some("Infinity") => Some(f64::INFINITY),
                Some("-Infinity") => Some(f64::NEG_INFINITY),
                _ => v.as_f64(),
            })
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|v| v as f64).collect::<Vec<_>>())
            .column("v", inputs)
            .build()
            .unwrap();
        let mut scale = spec(c);
        scale.missing_paint_is_na = c["channel"] == "colour";
        let draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry));
        let plot = if c["channel"] == "colour" {
            draft
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", scale))
                .layer(points())
                .build()
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
        }
        .unwrap();
        let result = plot.chart().unwrap().prepare();
        let matches = if c["result"].get("error").is_some() {
            result.is_err()
        } else if let Ok(prepared) = &result {
            let layer = &prepared.layers()[0];
            let entries = if c["channel"] == "colour" {
                layer
                    .color_legend()
                    .map(|v| v.numeric_breaks.clone())
                    .unwrap_or_default()
            } else {
                layer
                    .numeric_value_guides()
                    .values()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
            };
            let visible = entries.iter().filter(|v| v.visible).collect::<Vec<_>>();
            let keys = if visible.is_empty() {
                json!([])
            } else {
                json!([{"values":visible.iter().map(|v|encode(if c["transform"] == "reverse" { -v.value.0 } else if c["transform"] == "log10" { v.value.0.log10() } else { v.value.0 })).collect::<Vec<_>>(),"labels":visible.iter().map(|v|v.label.clone()).collect::<Vec<_>>(),"mapped":visible.iter().map(|v|match v.mapped.as_ref().unwrap(){Value::Number(v)=>encode(v.0),Value::Text(v)=>json!(v),Value::Missing=>Json::Null,_=>unreachable!()}).collect::<Vec<_>>()}])
            };
            let wanted = c["result"]["mapped"].as_array().unwrap();
            let marks_match = if c["channel"] == "size" {
                equivalent(
                    &json!(
                        layer
                            .marks()
                            .iter()
                            .map(|m| encode(m.style.radius))
                            .collect::<Vec<_>>()
                    ),
                    &json!(wanted.iter().filter(|v| !v.is_null()).collect::<Vec<_>>()),
                )
            } else if c["channel"] == "alpha" {
                layer.marks().len() == wanted.len()
                    && layer.marks().iter().zip(wanted).all(|(m, v)| {
                        m.style.color.alpha
                            == if matches!(v.as_str(), Some("Infinity" | "-Infinity")) {
                                0
                            } else {
                                (v.as_f64().unwrap_or(1.) * 255.).round_ties_even() as u8
                            }
                    })
            } else {
                let wanted = wanted.iter().filter(|v| !v.is_null()).collect::<Vec<_>>();
                layer.marks().len() == wanted.len()
                    && layer.marks().iter().zip(wanted).all(|(m, v)| {
                        m.style.color
                            == chart_core::color::parse_r(v.as_str().unwrap())
                                .unwrap()
                                .resolve()
                    })
            };
            if !equivalent(&keys, &c["result"]["keys"]) || !marks_match {
                failures.push(format!(
                    "{}/{}/{}/{}/{}: keys {keys}, expected {}; marks_match={marks_match}",
                    c["transform"],
                    c["limit_mode"],
                    c["channel"],
                    c["population"],
                    c["guide_mode"],
                    c["result"]["keys"]
                ));
            }
            true
        } else {
            false
        };
        if !matches {
            failures.push(format!(
                "{}/{}/{}/{}/{}: actual error {:?}, expected {}",
                c["transform"],
                c["limit_mode"],
                c["channel"],
                c["population"],
                c["guide_mode"],
                result.err(),
                c["result"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} primary composition failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
