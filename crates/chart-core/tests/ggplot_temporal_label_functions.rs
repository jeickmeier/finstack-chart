//! FIX-GG04: temporal label callback values, metadata, precedence and primary guides.
use chart_core::{
    ChartResult, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput,
        OperationRef,
    },
    interpolate::Number,
    prelude::*,
    scales::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
struct Formatter(Arc<Mutex<Vec<Value>>>);
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.temporal_labels", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
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
            "indexed" => (),
            _ => unreachable!(),
        }
        Ok(labels)
    }
}
fn zone(name: &str) -> CalendarZone {
    if name == "UTC" {
        return CalendarZone::Utc;
    }
    CalendarZone::Local(Arc::new(TimeZoneRules {
        version: 1,
        zone: name.into(),
        revision: Revision::INITIAL,
        tzdata: "explicit-US-2024".into(),
        coverage: TimeBounds {
            start: 1704067200000,
            end: 1735689600000,
        },
        initial_offset_seconds: -18000,
        transitions: vec![
            TimeZoneTransition {
                at_millis: 1710054000000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1730613600000,
                offset_seconds: -18000,
            },
        ],
    }))
}
#[test]
fn temporal_label_callbacks_match_reference_inputs_and_primary_guide_labels() {
    assert_temporal_labels(
        include_str!("../../../fixtures/parity/ggplot2/temporal-label-functions.json"),
        1180,
        420,
    );
}
#[test]
fn noncolor_temporal_labels_match_1200_reference_builds_in_four_units() {
    assert_temporal_labels(
        include_str!("../../../fixtures/parity/ggplot2/noncolor-temporal-label-functions.json"),
        3540,
        1260,
    );
}
fn assert_temporal_labels(source: &str, successes: usize, errors: usize) {
    let fixture: Value = serde_json::from_str(source).unwrap();
    let mut passed = 0;
    let mut rejected = 0;
    for c in fixture["cases"].as_array().unwrap() {
        for (unit, multiplier) in [
            (TimeUnit::Seconds, 1),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1000000),
            (TimeUnit::Nanoseconds, 1000000000),
        ] {
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            registry
                .register_guide_formatter(Arc::new(Formatter(calls.clone())))
                .unwrap();
            let date = c["kind"] == "date";
            let origin = c["epoch"].as_i64().unwrap() * multiplier;
            let factor = multiplier * if date { 86400 } else { 3600 };
            let inputs = c["inputs"].as_array().unwrap();
            let stamps = inputs
                .iter()
                .map(|v| {
                    (v.as_f64().unwrap_or(0.) * multiplier as f64 * if date { 86400. } else { 1. })
                        as i64
                })
                .collect::<Vec<_>>();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    timestamps(stamps, unit, c["zone"].as_str().unwrap())
                        .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                )
                .build()
                .unwrap();
            let channel = c["channel"].as_str().unwrap_or("colour");
            let scale = if channel == "colour" {
                let ColorScale::Mapped { scale, .. } = ggplot_color_default(true).unwrap() else {
                    unreachable!()
                };
                scale
            } else {
                ggplot_numeric_default(match channel {
                    "size" => GgplotNumericPalette::Size,
                    "alpha" => GgplotNumericPalette::Alpha,
                    "linewidth" => GgplotNumericPalette::Linewidth,
                    _ => unreachable!(),
                })
                .unwrap()
            };
            let scale = scale
                .with_timestamp_normalization(GgplotTimestampNormalization { origin, unit, date })
                .unwrap()
                .with_guide(GgplotScaleGuide::Temporal(GgplotTemporalGuide {
                    origin,
                    unit,
                    zone: zone(c["zone"].as_str().unwrap()),
                    arguments: GgplotTemporalGuideArguments {
                        date,
                        breaks: match c["control"].as_str().unwrap() {
                            "explicit" => GgplotTemporalBreaks::Explicit(
                                [-1., 0., 1., 3., 4., f64::NAN]
                                    .map(|v| Number(v * factor as f64))
                                    .to_vec(),
                            ),
                            "empty" => GgplotTemporalBreaks::Explicit(vec![]),
                            _ => GgplotTemporalBreaks::Automatic,
                        },
                        labels: GgplotGuideLabels::Registered {
                            operation: OperationRef::new("test.temporal_labels", Revision::new(1)),
                            parameters: c["label_mode"].clone(),
                        },
                        format: (c["control"] == "format").then(|| {
                            Box::new(GgplotTimeFormat {
                                pattern: if date { "%Y-%m-%d" } else { "%H:%M" }.into(),
                                locale: None,
                            })
                        }),
                        count: None,
                    },
                }))
                .unwrap();
            let result = (|| {
                let builder = plot(data)
                    .extensions(Arc::new(registry.clone()))
                    .profile(Profile::Ggplot2_4_0_3);
                let mapping = Mapping::Timestamp {
                    field: "v".into(),
                    origin,
                };
                let p = if channel == "colour" {
                    builder
                        .aes(aes().x("x").y(1.).color(mapping).color_scale("v"))
                        .scale(color_mapped("v", scale))
                        .layer(points())
                        .build()?
                } else {
                    let layer = match channel {
                        "size" => points().numeric_scale(NumericAesthetic::Size, mapping, scale),
                        "alpha" => points().numeric_scale(NumericAesthetic::Alpha, mapping, scale),
                        "linewidth" => {
                            line().numeric_scale(NumericAesthetic::StrokeWidth, mapping, scale)
                        }
                        _ => unreachable!(),
                    };
                    builder.aes(aes().x("x").y(1.)).layer(layer).build()?
                };
                let wire = p.to_json()?;
                let restored = Plot::from_json_with_extensions(&wire, Arc::new(registry.clone()))?;
                assert_eq!(restored.to_json()?, wire);
                assert!(calls.lock().unwrap().is_empty());
                let prepared = restored.chart()?.prepare()?;
                let layer = &prepared.layers()[0];
                if channel == "colour" {
                    let actual = layer
                        .color_legend()
                        .map(|g| g.entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>())
                        .unwrap_or_default();
                    let expected = c["result"]["keys"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .flat_map(|key| {
                            key["labels"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|label| label.as_str().unwrap_or("NA").to_owned())
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(actual, expected, "{c} {unit:?}");
                }
                let factor = (multiplier * if date { 86400 } else { 1 }) as f64;
                let entries = if channel == "colour" {
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
                let values = entries
                    .iter()
                    .map(|e| origin as f64 / factor + e.transformed.0 / factor)
                    .collect::<Vec<_>>();
                let labels = entries.iter().map(|e| e.label.clone()).collect::<Vec<_>>();
                Ok::<_, chart_core::Diagnostic>(json!({"values":values,"labels":labels}))
            })();
            let recorded = calls.lock().unwrap();
            let expected_calls = c["calls"].as_array().unwrap();
            assert_eq!(
                recorded.len(),
                expected_calls.len(),
                "{c} {unit:?}: {result:?}"
            );
            if expected_calls.is_empty() {
                assert!(recorded.is_empty(), "{c} {recorded:?}");
            } else {
                assert!(!recorded.is_empty(), "missing call {c}: {result:?}");
                for call in recorded.iter() {
                    let expected = &expected_calls[0];
                    for key in ["class", "zone", "names"] {
                        assert_eq!(call[key], expected[key], "{c} {unit:?}");
                    }
                    let a = call["values"].as_array().unwrap();
                    let b = expected["values"].as_array().unwrap();
                    assert_eq!(a.len(), b.len(), "{c}");
                    for (a, b) in a.iter().zip(b) {
                        assert_eq!(a.as_f64(), b.as_f64(), "{c} {unit:?}");
                    }
                }
            }
            if c["result"]["error"].is_string() {
                assert!(result.is_err(), "expected rejection {c} {unit:?}");
                rejected += 1;
            } else {
                let expected = c["result"]["keys"]
                    .as_array()
                    .unwrap()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| json!({"values":[],"labels":[]}));
                let actual = result.unwrap_or_else(|e| panic!("{c} {unit:?}: {e:?}"));
                assert_eq!(actual["labels"], expected["labels"], "{c} {unit:?}");
                if channel != "colour" {
                    let actual = actual["values"].as_array().unwrap();
                    let expected = expected["values"].as_array().unwrap();
                    assert_eq!(actual.len(), expected.len(), "{c} {unit:?}");
                    for (actual, expected) in actual.iter().zip(expected) {
                        assert_eq!(actual.as_f64(), expected.as_f64(), "{c} {unit:?}");
                    }
                }
                passed += 1;
            }
        }
    }
    assert_eq!((passed, rejected), (successes, errors));
}
