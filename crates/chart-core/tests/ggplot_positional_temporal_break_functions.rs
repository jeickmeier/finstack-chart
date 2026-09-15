//! FIX-GG04: temporal label callback values, metadata, precedence and primary guides.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        CustomGuideFormatter, CustomScaleBreaks, ExtensionDescriptor, ExtensionRegistry,
        GuideLabelsInput, OperationRef, ScaleBreaksInput, ScaleBreaksOutput,
    },
    interpolate::Number,
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
                encoded_number(n.origin as f64 / factor + v / factor)
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
        if input.values.is_empty() {
            labels.push(Some("/0".into()));
        }
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
fn normalize(v: &Value) -> Value {
    match v {
        Value::Number(n) => json!(n.as_f64().unwrap()),
        Value::Array(a) => Value::Array(a.iter().map(normalize).collect()),
        Value::Object(o) => {
            Value::Object(o.iter().map(|(k, v)| (k.clone(), normalize(v))).collect())
        }
        _ => v.clone(),
    }
}
fn encoded_number(v: f64) -> Value {
    if v.is_nan() {
        Value::Null
    } else if v == f64::INFINITY {
        json!("Infinity")
    } else if v == f64::NEG_INFINITY {
        json!("-Infinity")
    } else {
        json!(v)
    }
}
struct Breaks(Arc<Mutex<Vec<Value>>>, String);
impl CustomScaleBreaks for Breaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.temporal_breaks", Revision::new(1), true)
    }
    fn accepts_n(&self) -> bool {
        self.1 == "n"
    }
    fn accepts_n_breaks(&self) -> bool {
        self.1 == "n.breaks"
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        let context = input.temporal.expect("typed temporal limits");
        let n = context.normalization;
        let multiplier = match n.unit {
            TimeUnit::Seconds => 1.,
            TimeUnit::Milliseconds => 1e3,
            TimeUnit::Microseconds => 1e6,
            TimeUnit::Nanoseconds => 1e9,
        };
        let factor = multiplier * if n.date { 86400. } else { 1. };
        let d = input
            .domain
            .iter()
            .map(|v| match v {
                ScaleKey::Number(n) => n.0,
                _ => panic!("numeric offsets"),
            })
            .collect::<Vec<_>>();
        let zone = match context.zone {
            CalendarZone::Utc => "UTC",
            CalendarZone::Local(r) => &r.zone,
        };
        assert_eq!(input.count_argument, input.count.map(|_| "n"));
        self.0.lock().unwrap().push(json!({"values":d.iter().map(|v| encoded_number(n.origin as f64/factor+v/factor)).collect::<Vec<_>>(),
            "class":if n.date {vec!["Date"]}else{vec!["POSIXct","POSIXt"]}, "zone":if n.date {vec![]}else{vec![zone]}, "names":[], "count":input.count,
            "effective":match self.1.as_str() { "n"=>json!(input.count.unwrap_or(7.)), "n.breaks"=>json!(9.), _=>Value::Null }}));
        let mode = input.parameters.as_str().unwrap();
        let (values, names) = match mode {
            "domain" | "numeric" => (Some(d), None),
            "mixed" => (
                Some(vec![
                    d[1],
                    (d[0] + d[1]) / 2.,
                    d[0],
                    d[0],
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ]),
                Some(
                    [
                        "last", "middle", "first", "again", "missing", "positive", "negative",
                    ]
                    .map(String::from)
                    .to_vec(),
                ),
            ),
            "empty" => (Some(vec![]), None),
            "null" => (None, None),
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            values: values.map(|v| v.into_iter().map(|v| ScaleKey::Number(Number(v))).collect()),
            names,
            temporal: (mode != "numeric" && mode != "null").then_some(n),
        })
    }
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn positional_temporal_breaks_match_reference() {
    assert_positional_temporal_breaks(
        include_str!("../../../fixtures/parity/ggplot2/positional-temporal-break-functions.json"),
        9000,
    );
}
#[test]
fn positional_temporal_break_precedence_matches_reference() {
    assert_positional_temporal_breaks(
        include_str!("../../../fixtures/parity/ggplot2/positional-temporal-break-overrides.json"),
        560,
    );
}
#[test]
fn positional_temporal_zero_range_bypasses_break_callbacks() {
    assert_positional_temporal_breaks(
        include_str!("../../../fixtures/parity/ggplot2/positional-temporal-break-zero.json"),
        200,
    );
}
fn assert_positional_temporal_breaks(source: &str, expected_success: usize) {
    let fixture: Value = serde_json::from_str(source).unwrap();
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
            let breaks = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
            registry
                .register_guide_formatter(Arc::new(Formatter(calls.clone())))
                .unwrap();
            registry
                .register_scale_breaks(Arc::new(Breaks(
                    breaks.clone(),
                    c["signature"].as_str().unwrap().into(),
                )))
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
            let mut scale = if date {
                scale_date()
            } else if c["zone"] == "UTC" {
                scale_utc()
            } else {
                scale_calendar(TimeScaleSpec {
                    domain: if c["limits"] == "full" {
                        vec![origin, origin + 3 * factor]
                    } else {
                        vec![]
                    },
                    unit,
                    zone: zone(c["zone"].as_str().unwrap()),
                    ..Default::default()
                })
            };
            if c["limits"] == "full" && c["zone"] == "UTC" {
                scale = scale.time_domain(origin, origin + 3 * factor);
            }
            let mut axis =
                x_axis()
                    .scale(scale)
                    .range(100., 540.)
                    .guide_geometry(Some(GuideGeometry {
                        labels: Some(GuideLabelPolicy::Preserve),
                        ..Default::default()
                    }));
            axis = axis
                .breaks_function(Some(chart_core::grammar::ScaleBreaksOperation {
                    operation: OperationRef::new("test.temporal_breaks", Revision::new(1)),
                    parameters: c["mode"].clone(),
                }))
                .tick_arguments(Some(GuideTickArguments {
                    count: match c["count"].as_str().unwrap() {
                        "three" => Some(3.),
                        "zero" => Some(0.),
                        _ => None,
                    },
                    ..Default::default()
                }));
            if c["label_mode"] == "indexed" {
                axis = axis.tick_format(Some(GuideFormatter::Registered {
                    operation: OperationRef::new("test.temporal_labels", Revision::new(1)),
                    parameters: json!("indexed"),
                }));
            }
            if matches!(c["control"].as_str(), Some("width" | "both")) {
                axis = axis.tick_arguments(Some(GuideTickArguments {
                    count: None,
                    width: Some(if date { "1 day" } else { "1 hour" }.into()),
                    ..Default::default()
                }));
            }
            if matches!(c["control"].as_str(), Some("format" | "both")) {
                axis = axis.tick_format(Some(GuideFormatter::GgplotTime(Box::new(
                    GgplotTimeFormat {
                        pattern: if date { "%Y-%m-%d" } else { "%H:%M" }.into(),
                        locale: None,
                    },
                ))));
            }
            if c["control"] == "zero" {
                axis = axis.expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }));
            }
            let result = (|| -> ChartResult<_> {
                let p = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(
                        aes()
                            .x(Mapping::Timestamp {
                                field: "x".into(),
                                origin,
                            })
                            .y("y"),
                    )
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
            let observed = breaks.lock().unwrap();
            let expected_breaks = c["calls"].as_array().unwrap();
            assert_eq!(
                observed.is_empty(),
                expected_breaks.is_empty(),
                "break call presence {index}"
            );
            for call in observed.iter() {
                assert_eq!(
                    normalize(call),
                    normalize(&expected_breaks[0]),
                    "break call {index} {unit:?}"
                );
            }
            let actual = calls.lock().unwrap();
            let expected = c["label_calls"].as_array().unwrap();
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
                    assert_eq!(
                        normalize(a),
                        normalize(b),
                        "case {index} {unit:?}: {a} != {b}"
                    );
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
                    let wanted = c["result"]["breaks"]
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
                    let range = c["result"]["range"].as_array().unwrap();
                    let low = range[0].as_f64().unwrap();
                    let high = range[1].as_f64().unwrap();
                    for (tick, expected) in frame.guides()[&GuideId::new(0)].ticks.iter().zip(
                        c["result"]["breaks"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter_map(Value::as_f64),
                    ) {
                        let factor = multiplier as f64 * if date { 86400. } else { 1. };
                        let actual = match tick.value {
                            ScaleValue::Timestamp { value, .. } => value as f64 / factor,
                            ScaleValue::Number(n) => origin as f64 / factor + n / factor,
                            _ => unreachable!(),
                        };
                        assert_eq!(actual, expected, "tick value {index} {unit:?}");
                        let position = if low == high {
                            320.
                        } else {
                            100. + 440. * (expected - low) / (high - low)
                        };
                        assert!(
                            (tick.position - position).abs() < 1e-8,
                            "tick position {index} {unit:?}: {} != {position}",
                            tick.position
                        );
                    }
                }
            }
        }
    }
    assert_eq!(passed, expected_success);
}
