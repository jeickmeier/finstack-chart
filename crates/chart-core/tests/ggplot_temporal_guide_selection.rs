//! FIX-GG04: temporal guide selection reuses temporal candidates and ramp sampling.
use chart_core::{
    ChartResult, Revision,
    composition::ScaleValue,
    grammar::{
        CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput,
        OperationRef,
    },
};
use chart_core::{data::TimeUnit, grammar::NumericAesthetic, prelude::*, scales::*};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
fn spec(t: &Json, origin: i64, factor: f64, unit: TimeUnit) -> MappedScaleSpec {
    let channel = t["channel"].as_str().unwrap();
    let output = if channel == "colour" {
        json!({"Interpolate":{"operation":"GgplotPalette","spec":{"Gradient":{"colors":[{"red":19,"green":43,"blue":67,"alpha":255},{"red":86,"green":177,"blue":247,"alpha":255}],"values":null}}}})
    } else {
        json!({"Interpolate":{"operation":"PowerRange","range":if channel=="alpha"{[0.1,1.]}else{[1.,6.]},"exponent":if channel=="size"{0.5}else{1.},"absolute":false}})
    };
    let mut spec:MappedScaleSpec=serde_json::from_value(json!({"training":"Eligible","function":{"Interpolated":{"normalization":{"Ggplot":{"family":"Linear","domain":[0,10. * factor],"reverse":false,"rescaler":"Range"}},"output":output,"unknown":{"kind":"Missing"}}},"ggplot":{"Continuous":{"limits":if t["limits"]=="full"{json!([0,10. * factor])}else{Json::Null},"oob":"Censor"}}})).unwrap();
    spec = spec
        .with_timestamp_normalization(GgplotTimestampNormalization {
            origin,
            unit,
            date: t["kind"] == "date",
        })
        .unwrap();
    let g = GgplotTemporalGuide {
        origin,
        unit,
        zone: CalendarZone::Utc,
        arguments: GgplotTemporalGuideArguments {
            date: t["kind"] == "date",
            breaks: match t["breaks"].as_str().unwrap() {
                "null" => GgplotTemporalBreaks::None,
                "empty" => GgplotTemporalBreaks::Explicit(vec![]),
                "explicit" => GgplotTemporalBreaks::Explicit(
                    [-1., 0., 1., 1., 3., 20., f64::NAN]
                        .map(|v| chart_core::interpolate::Number(v * factor))
                        .to_vec(),
                ),
                _ => GgplotTemporalBreaks::Automatic,
            },
            labels: if t["label_mode"].is_string() {
                GgplotGuideLabels::Registered {
                    operation: OperationRef::new("test.temporal_interval_labels", Revision::new(1)),
                    parameters: t["label_mode"].clone(),
                }
            } else {
                GgplotGuideLabels::Automatic
            },
            format: (t["format_mode"] == "explicit").then(|| {
                Box::new(GgplotTimeFormat {
                    pattern: if t["kind"] == "date" {
                        "%d/%m"
                    } else {
                        "%Hh%M"
                    }
                    .into(),
                    locale: None,
                })
            }),
            ..Default::default()
        },
    };
    spec.with_guide(if t["guide"] == "none" {
        GgplotScaleGuide::Hidden
    } else if t["guide"] == "colourbar" || (t["guide"] == "default" && channel == "colour") {
        GgplotScaleGuide::TemporalColorbar(g)
    } else if t["guide"] == "bins" {
        GgplotScaleGuide::TemporalBins(g)
    } else if t["guide"] == "coloursteps" {
        GgplotScaleGuide::TemporalSteps(g)
    } else {
        GgplotScaleGuide::Temporal(g)
    })
    .unwrap()
}
fn build_case(t: &Json, unit: TimeUnit, multiplier: i64, registry: Arc<ExtensionRegistry>) -> Plot {
    let origin = 1_577_836_800 * multiplier;
    let factor = (if t["kind"] == "date" { 86_400 } else { 3600 }) as f64 * multiplier as f64;
    let inputs = t["inputs"].as_array().unwrap();
    let channel = t["channel"].as_str().unwrap();
    let data = Data::columns()
        .column("x", (0..inputs.len()).map(|v| v as f64).collect::<Vec<_>>())
        .column(
            "v",
            timestamps(
                inputs
                    .iter()
                    .map(|v| origin + (v.as_f64().unwrap() * factor) as i64)
                    .collect::<Vec<_>>(),
                unit,
                "UTC",
            ),
        )
        .build()
        .unwrap();
    let mapping = Mapping::Timestamp {
        field: "v".into(),
        origin,
    };
    let scale = spec(t, origin, factor, unit);
    let draft = plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3);
    if channel == "colour" {
        draft
            .aes(aes().x("x").y(1.).color(mapping).color_scale("v"))
            .scale(color_mapped("v", scale))
            .layer(points())
    } else {
        draft.aes(aes().x("x").y(1.)).layer(points().numeric_scale(
            if channel == "size" {
                NumericAesthetic::Size
            } else {
                NumericAesthetic::Alpha
            },
            mapping,
            scale,
        ))
    }
    .build()
    .unwrap()
}
#[test]
fn temporal_guides_match_648_reference_draws_in_four_units() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-guide-selection.json"
    ))
    .unwrap();
    let mut compared = 0;
    for t in fixture["cases"].as_array().unwrap().iter() {
        let interval = matches!(t["guide"].as_str(), Some("bins" | "coloursteps"));
        for (unit, multiplier) in [
            (TimeUnit::Seconds, 1_i64),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1_000_000),
            (TimeUnit::Nanoseconds, 1_000_000_000),
        ] {
            let date = t["kind"] == "date";
            let factor = (if date { 86_400 } else { 3600 }) as f64 * multiplier as f64;
            let channel = t["channel"].as_str().unwrap();
            let p = build_case(t, unit, multiplier, Arc::new(ExtensionRegistry::default()));
            let wire = p.to_json().unwrap();
            if t["guide"] == "colourbar" || (t["guide"] == "default" && channel == "colour") {
                let mut old: Json = serde_json::from_str(&wire).unwrap();
                assert_eq!(old["version"], 59);
                old["version"] = json!(58);
                assert!(Plot::from_json(&old.to_string()).is_err());
            }
            if interval {
                let mut old: Json = serde_json::from_str(&wire).unwrap();
                assert_eq!(old["version"], 60);
                old["version"] = json!(59);
                assert!(Plot::from_json(&old.to_string()).is_err());
            }
            let restored = Plot::from_json(&wire).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            let result = restored.chart().and_then(|mut chart| chart.prepare());
            if !t["result"]["error"].is_null() {
                assert!(result.is_err(), "reference rejected {t}");
                compared += 1;
                continue;
            }
            let prepared = result.unwrap_or_else(|e| panic!("{t}: {e}"));
            let layer = &prepared.layers()[0];
            let mapped = t["result"]["mapped"].as_array().unwrap();
            assert_eq!(layer.marks().len(), mapped.len(), "{t}");
            for (mark, value) in layer.marks().iter().zip(mapped) {
                if channel == "colour" {
                    assert_eq!(
                        mark.style.color,
                        chart_core::color::parse_r(value.as_str().unwrap())
                            .unwrap()
                            .resolve(),
                        "{t}"
                    );
                } else if channel == "size" {
                    assert!(
                        (mark.style.radius - value.as_f64().unwrap()).abs() < 2e-12,
                        "{t}"
                    );
                } else {
                    assert_eq!(
                        mark.style.color.alpha,
                        (value.as_f64().unwrap() * 255.).round_ties_even() as u8,
                        "{t}"
                    );
                }
            }
            let entries = if channel == "colour" {
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
                    .collect()
            };
            let actual = entries.iter().filter(|e| e.visible).collect::<Vec<_>>();
            let wanted = t["result"]["guides"].as_array().unwrap();
            let raw = if wanted.is_empty() {
                vec![]
            } else if interval {
                wanted[0]["source_values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .take(wanted[0]["values"].as_array().unwrap().len())
                    .cloned()
                    .collect()
            } else {
                t["result"]["raw_breaks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| {
                        v.as_f64().is_some_and(|v| {
                            v >= t["result"]["limits"][0].as_f64().unwrap()
                                && v <= t["result"]["limits"][1].as_f64().unwrap()
                        })
                    })
                    .cloned()
                    .collect()
            };
            assert_eq!(
                raw.len(),
                wanted
                    .first()
                    .and_then(|g| g["values"].as_array())
                    .map_or(0, Vec::len)
            );
            assert_eq!(actual.len(), raw.len(), "{t} {unit:?}");
            for (i, (entry, expected)) in actual.iter().zip(&raw).enumerate() {
                let absolute = if date {
                    18262. + entry.transformed.0 / factor
                } else {
                    1_577_836_800. + entry.transformed.0 / multiplier as f64
                };
                assert_eq!(absolute, expected.as_f64().unwrap(), "{t} {unit:?}");
                assert_eq!(
                    entry.label.as_deref(),
                    wanted[0]["labels"][i].as_str(),
                    "{t}"
                );
            }
            if interval && !wanted.is_empty() {
                for (entry, value) in actual.iter().zip(wanted[0]["mapped"].as_array().unwrap()) {
                    use chart_core::interpolate::Value;
                    match entry.mapped.as_ref().expect("interval key mapping") {
                        Value::Missing | Value::Null => assert!(value.is_null(), "{t}"),
                        Value::Color(color) => assert_eq!(
                            color.to_paint(),
                            chart_core::color::parse_r(value.as_str().unwrap())
                                .unwrap()
                                .resolve(),
                            "{t}"
                        ),
                        Value::Number(n) => assert!(
                            (n.0 - value.as_f64().unwrap()).abs() <= 3e-12 * n.0.abs().max(1.),
                            "{t}"
                        ),
                        mapped => panic!("unexpected interval key {mapped:?}: {t}"),
                    }
                }
            }
            let ramp = layer
                .color_legend()
                .map(|g| g.colorbar.as_slice())
                .unwrap_or_default();
            let expected_ramp = wanted
                .first()
                .and_then(|g| g["decor_values"].as_array())
                .map(Vec::as_slice)
                .unwrap_or_default();
            assert_eq!(
                ramp.len(),
                expected_ramp.len(),
                "temporal ramp: {t} {unit:?}"
            );
            for (i, (sample, wanted_value)) in ramp.iter().zip(expected_ramp).enumerate() {
                let absolute = sample.value.0;
                assert!(
                    (absolute - wanted_value.as_f64().unwrap()).abs()
                        <= 4. * f64::EPSILON * absolute.abs().max(1.),
                    "{t}: {absolute}"
                );
                assert_eq!(
                    sample.color,
                    chart_core::color::parse_r(wanted[0]["decor_colors"][i].as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "ramp color: {t} {i}"
                );
            }
            compared += 1;
        }
    }
    assert_eq!(compared, 2592);
}

struct Formatter(Arc<Mutex<Vec<Json>>>);
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.temporal_interval_labels", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        let context = input.temporal.expect("temporal context retained");
        let n = context.normalization;
        let multiplier = match n.unit {
            TimeUnit::Seconds => 1.,
            TimeUnit::Milliseconds => 1000.,
            TimeUnit::Microseconds => 1000000.,
            TimeUnit::Nanoseconds => 1000000000.,
        };
        let factor = multiplier * if n.date { 86400. } else { 1. };
        let values = input
            .values
            .iter()
            .map(|v| {
                let ScaleValue::Number(v) = v else {
                    panic!("expected origin-relative number")
                };
                json!(n.origin as f64 / factor + v / factor)
            })
            .collect::<Vec<_>>();
        let zone = match context.zone {
            CalendarZone::Utc => "UTC",
            CalendarZone::Local(rules) => &rules.zone,
        };
        self.0.lock().unwrap().push(json!({"values":values,"class":if n.date {vec!["Date"]}else{vec!["POSIXct","POSIXt"]},"zone":if n.date {vec![]}else{vec![zone]},"names":input.names.unwrap_or(&[])}));
        let mut labels = (1..=input.values.len())
            .map(|i| Some(format!("{i}/{}", input.values.len())))
            .collect::<Vec<_>>();
        match input.parameters.as_str().unwrap() {
            "missing" => {
                for (i, label) in labels.iter_mut().enumerate() {
                    if i % 2 == 1 {
                        *label = None;
                    }
                }
            }
            "short" => labels.truncate(1),
            "empty" => labels.clear(),
            "indexed" | "named" => (),
            _ => unreachable!(),
        }
        Ok(labels)
    }
}

#[test]
fn temporal_interval_label_callbacks_and_formats_match_reference() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-interval-label-functions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 2160);
    let mut checked = 0;
    for t in fixture["cases"].as_array().unwrap() {
        for (unit, multiplier) in [
            (TimeUnit::Seconds, 1),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1_000_000),
            (TimeUnit::Nanoseconds, 1_000_000_000),
        ] {
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            registry
                .register_guide_formatter(Arc::new(Formatter(calls.clone())))
                .unwrap();
            let registry = Arc::new(registry);
            let p = build_case(t, unit, multiplier, registry.clone());
            let wire = p.to_json().unwrap();
            let restored = Plot::from_json_with_extensions(&wire, registry).unwrap();
            assert_eq!(restored.to_json().unwrap(), wire);
            assert!(calls.lock().unwrap().is_empty());
            let result = restored.chart().and_then(|mut chart| chart.prepare());
            let recorded = calls.lock().unwrap();
            let expected_calls = t["label_calls"].as_array().unwrap();
            assert_eq!(
                recorded.len(),
                expected_calls.len(),
                "callback count {t} {unit:?}: {:?}",
                result.as_ref().err()
            );
            for (actual, expected) in recorded.iter().zip(expected_calls) {
                for key in ["class", "zone", "names"] {
                    assert_eq!(actual[key], expected[key], "callback {key}: {t} {unit:?}");
                }
                let a = actual["values"].as_array().unwrap();
                let b = expected["values"].as_array().unwrap();
                assert_eq!(a.len(), b.len(), "callback values: {t}");
                for (a, b) in a.iter().zip(b) {
                    assert_eq!(a.as_f64(), b.as_f64(), "callback values: {t} {unit:?}");
                }
            }
            drop(recorded);
            if t["result"]["error"].is_string() {
                assert!(result.is_err(), "reference rejected {t}");
            } else {
                let prepared = result.unwrap_or_else(|e| panic!("{t} {unit:?}: {e}"));
                let layer = &prepared.layers()[0];
                let entries = if t["channel"] == "colour" {
                    layer
                        .color_legend()
                        .map(|g| {
                            g.numeric_breaks
                                .iter()
                                .filter(|e| e.visible)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                } else {
                    layer
                        .numeric_value_guides()
                        .values()
                        .flatten()
                        .filter(|e| e.visible)
                        .collect::<Vec<_>>()
                };
                let wanted = t["result"]["guides"].as_array().unwrap();
                let n = wanted
                    .first()
                    .map_or(0, |g| g["values"].as_array().unwrap().len());
                assert_eq!(entries.len(), n, "{t} {unit:?}");
                let date = t["kind"] == "date";
                let factor = multiplier as f64 * if date { 86400. } else { 1. };
                for (i, e) in entries.iter().enumerate() {
                    assert_eq!(
                        e.label.as_deref(),
                        wanted[0]["labels"][i].as_str(),
                        "{t} {unit:?}"
                    );
                    assert_eq!(
                        1_577_836_800. / if date { 86400. } else { 1. } + e.transformed.0 / factor,
                        wanted[0]["source_values"][i].as_f64().unwrap(),
                        "{t} {unit:?}"
                    );
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 8640);
}
