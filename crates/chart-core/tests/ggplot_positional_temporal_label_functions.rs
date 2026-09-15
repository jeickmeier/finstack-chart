//! FIX-GG04: temporal label callback values, metadata, precedence and primary guides.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput,
        OperationRef,
    },
    layout::*,
    prelude::*,
    scales::*,
    services::*,
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
        let mut labels = if input.values.is_empty() {
            vec![Some("/0".into())]
        } else {
            (1..=input.values.len())
                .map(|i| Some(format!("{i}/{}", input.values.len())))
                .collect::<Vec<_>>()
        };
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

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn positional_temporal_labels_match_reference_metadata_and_visible_ticks() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-temporal-label-functions.json"
    ))
    .unwrap();
    let mut passed = 0;
    for (index, c) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        for (unit, multiplier) in [
            (TimeUnit::Seconds, 1_i64),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1000000),
            (TimeUnit::Nanoseconds, 1000000000),
        ] {
            let date = c["kind"] == "date";
            let origin = c["epoch"].as_i64().unwrap() * multiplier;
            let factor = multiplier * if date { 86400 } else { 3600 };
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            registry
                .register_guide_formatter(Arc::new(Formatter(calls.clone())))
                .unwrap();
            let inputs = c["inputs"].as_array().unwrap();
            let stamps = inputs
                .iter()
                .map(|v| {
                    (v.as_f64().unwrap_or(0.) * multiplier as f64 * if date { 86400. } else { 1. })
                        as i64
                })
                .collect::<Vec<_>>();
            let data = Data::columns()
                .column(
                    "x",
                    timestamps(stamps, unit, c["zone"].as_str().unwrap())
                        .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                )
                .column("y", vec![1.; inputs.len()])
                .build()
                .unwrap();
            let scale = if date {
                scale_date()
            } else if c["zone"] == "UTC" {
                scale_utc()
            } else {
                scale_calendar(TimeScaleSpec {
                    domain: vec![],
                    unit,
                    zone: zone(c["zone"].as_str().unwrap()),
                    ..Default::default()
                })
            };
            let mut axis = x_axis().scale(scale).guide_geometry(Some(GuideGeometry {
                labels: Some(GuideLabelPolicy::Preserve),
                ..Default::default()
            }));
            axis = axis.tick_format(Some(if c["control"] == "format" {
                GuideFormatter::GgplotTime(Box::new(GgplotTimeFormat {
                    pattern: if date { "%Y-%m-%d" } else { "%H:%M" }.into(),
                    locale: None,
                }))
            } else {
                GuideFormatter::Registered {
                    operation: OperationRef::new("test.temporal_labels", Revision::new(1)),
                    parameters: c["label_mode"].clone(),
                }
            }));
            if c["control"] == "explicit" {
                let mut values = [-1, 0, 1, 3, 4]
                    .map(|v| ScaleValue::Timestamp {
                        value: origin + v * factor,
                        unit,
                    })
                    .to_vec();
                values.push(ScaleValue::Number(f64::NAN));
                axis = axis.tick_values(Some(values));
            } else if c["control"] == "empty" {
                axis = axis.tick_values(Some(vec![]));
            }
            let result = (|| -> ChartResult<_> {
                let p = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points())
                    .x_axis(axis)
                    .extensions(Arc::new(registry))
                    .build()?;
                layout(
                    p.chart()?.prepare()?,
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
                "case {index} {unit:?} {c}: {:?}",
                result.as_ref().err()
            );
            let actual = calls.lock().unwrap();
            let expected = c["calls"].as_array().unwrap();
            assert_eq!(
                actual.is_empty(),
                expected.is_empty(),
                "case {index}: {actual:?} != {expected:?}"
            );
            for call in actual.iter() {
                let expected = &expected[0];
                for key in ["class", "zone", "names"] {
                    assert_eq!(call[key], expected[key], "case {index} {key}");
                }
                let a = call["values"].as_array().unwrap();
                let b = expected["values"].as_array().unwrap();
                assert_eq!(a.len(), b.len(), "case {index}: {a:?} != {b:?}");
                for (a, b) in a.iter().zip(b) {
                    assert_eq!(a.as_f64(), b.as_f64(), "case {index}: {a} != {b}");
                }
            }
            if let Ok(frame) = result {
                passed += 1;
                if c["result"]["range"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(Value::is_number)
                {
                    let wanted = c["result"]["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(c["result"]["labels"].as_array().unwrap())
                        .filter(|(v, _)| v.is_number())
                        .map(|(_, v)| v.as_str().unwrap_or(""))
                        .collect::<Vec<_>>();
                    let got = frame.guides()[&GuideId::new(0)]
                        .ticks
                        .iter()
                        .map(|t| t.label.as_str())
                        .collect::<Vec<_>>();
                    assert_eq!(got, wanted, "case {index}");
                }
            }
        }
    }
    assert_eq!(passed, 740);
}
