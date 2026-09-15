//! FIX-GG04: typed temporal minor callbacks, retained major metadata and width precedence.
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        CustomScaleBreaks, ExtensionDescriptor, ExtensionRegistry, OperationRef, ScaleBreaksInput,
        ScaleBreaksOutput,
    },
    interpolate::Number,
    layout::*,
    prelude::*,
    scales::*,
    services::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
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
fn same(actual: &Value, expected: &Value, tolerance: f64, context: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= tolerance,
            "{context}: {actual} != {expected}"
        ),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{context}");
            for (a, b) in a.iter().zip(b) {
                same(a, b, tolerance, context);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{context}"
            );
            for (k, a) in a {
                same(a, &b[k], tolerance, context);
            }
        }
        _ => assert_eq!(actual, expected, "{context}"),
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
        ExtensionDescriptor::batch("test.temporal_minor", Revision::new(1), true)
    }
    fn accepts_major_breaks(&self) -> bool {
        self.1 == "two"
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        let context = input.temporal.expect("typed temporal limits");
        let n = context.normalization;
        let read = |v: &ScaleKey| match v {
            ScaleKey::Number(n) => n.0,
            _ => panic!("numeric offsets"),
        };
        let d = input.domain.iter().map(read).collect::<Vec<_>>();
        let zone = match context.zone {
            CalendarZone::Utc => "UTC",
            CalendarZone::Local(r) => &r.zone,
        };
        let multiplier = match n.unit {
            TimeUnit::Seconds => 1.,
            TimeUnit::Milliseconds => 1e3,
            TimeUnit::Microseconds => 1e6,
            TimeUnit::Nanoseconds => 1e9,
        };
        let factor = multiplier * if n.date { 86400. } else { 1. };
        let metadata = |values: &[ScaleKey], names: Option<&[String]>| {
            json!({
                "values": values.iter().map(|v| encoded_number(n.origin as f64 / factor + read(v) / factor)).collect::<Vec<_>>(),
                "class": if n.date { vec!["Date"] } else { vec!["POSIXct", "POSIXt"] },
                "zone": if n.date { vec![] } else { vec![zone] }, "names": names.unwrap_or(&[])
            })
        };
        self.0
            .lock()
            .unwrap()
            .push(json!({"limits":metadata(input.domain, None),
            "major":input.major_breaks.map(|v|metadata(v,input.major_break_names))}));
        assert!(input.count.is_none());
        let mode = input.parameters.as_str().unwrap();
        let values = match mode {
            "domain" | "numeric" => Some(d),
            "mixed" => Some(vec![
                d[1],
                (d[0] + d[1]) / 2.,
                d[0],
                d[0],
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]),
            "majors" => input.major_breaks.map(|v| v.iter().map(read).collect()),
            "empty" => Some(vec![]),
            "null" => None,
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            values: values.map(|v| v.into_iter().map(|v| ScaleKey::Number(Number(v))).collect()),
            names: None,
            temporal: (mode != "numeric"
                && mode != "null"
                && !(mode == "majors" && input.major_breaks.is_none()))
            .then_some(n),
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
fn temporal_minor_callbacks_match_reference() {
    check(
        include_str!(
            "../../../fixtures/parity/ggplot2/positional-temporal-minor-break-functions.json"
        ),
        11320,
    );
}
#[test]
fn temporal_minor_width_overrides_callbacks() {
    check(
        include_str!(
            "../../../fixtures/parity/ggplot2/positional-temporal-minor-break-overrides.json"
        ),
        240,
    );
}
fn check(source: &str, expected_success: usize) {
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
            let breaks = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::default();
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
            axis = axis.minor_breaks(Some(MinorBreaks::Registered(
                chart_core::grammar::ScaleBreaksOperation {
                    operation: OperationRef::new("test.temporal_minor", Revision::new(1)),
                    parameters: c["mode"].clone(),
                },
            )));
            match c["major"].as_str().unwrap() {
                "explicit" => {
                    let mut values = [3, 1, 0, 0]
                        .map(|v| ScaleValue::Timestamp {
                            value: origin + v * factor,
                            unit,
                        })
                        .to_vec();
                    values.extend(
                        [f64::NAN, f64::INFINITY, f64::NEG_INFINITY].map(ScaleValue::Number),
                    );
                    axis = axis.tick_values(Some(values));
                }
                "empty" => axis = axis.tick_values(Some(vec![])),
                "null" => axis = axis.ticks([]),
                "automatic" => (),
                _ => unreachable!(),
            }
            if c["expand"] == "zero" {
                axis = axis.expansion(Some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                }));
            }
            if c["control"] == "width" {
                axis = axis.minor_breaks(Some(MinorBreaks::TimeWidth(
                    if date { "1 day" } else { "1 hour" }.into(),
                )));
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
                // Bounds near epoch zero lose at most a few ULPs when expressed
                // relative to the authored 2024 origin and converted back.
                let tolerance = 4.
                    * f64::EPSILON
                    * (c["epoch"].as_f64().unwrap() / if date { 86400. } else { 1. }).max(1.);
                same(
                    call,
                    &expected_breaks[0],
                    tolerance,
                    &format!("break call {index} {unit:?}"),
                );
            }
            if let Ok(frame) = result {
                passed += 1;
                let minor = &frame.guides()[&GuideId::new(0)].minor_ticks;
                let expected = c["result"]["minor"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(Value::as_f64)
                    .filter(|_| {
                        c["result"]["range"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(Value::is_number)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(minor.len(), expected.len(), "minor count {index} {unit:?}");
                for (tick, expected) in minor.iter().zip(expected) {
                    let factor = multiplier as f64 * if date { 86400. } else { 1. };
                    let actual = match tick.value.as_ref().unwrap() {
                        ScaleValue::Timestamp { value, .. } => *value as f64 / factor,
                        ScaleValue::Number(v) => origin as f64 / factor + v / factor,
                        _ => unreachable!(),
                    };
                    assert!(
                        (actual - expected).abs()
                            <= 4.
                                * f64::EPSILON
                                * (c["epoch"].as_f64().unwrap() / if date { 86400. } else { 1. })
                                    .max(expected.abs())
                                    .max(1.),
                        "minor value {index} {unit:?}: {actual} != {expected}"
                    );
                    let range = c["result"]["range"].as_array().unwrap();
                    if let (Some(a), Some(b)) = (range[0].as_f64(), range[1].as_f64()) {
                        let wanted = if a == b {
                            320.
                        } else {
                            100. + 440. * (expected - a) / (b - a)
                        };
                        assert!(
                            (tick.position - wanted).abs() < 1e-7,
                            "minor position {index} {unit:?}: {} != {wanted}",
                            tick.position
                        );
                    }
                }

                if c["result"]["range"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(Value::is_number)
                {
                    let wanted = c["result"]["major"]
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
                        c["result"]["major"]
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
