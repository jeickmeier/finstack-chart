//! SP-06 / FIX-20: supplied zones drive interval arithmetic, not labels alone.
use chart_core::{Revision, data::TimeUnit, scales::*};
use serde_json::Value as Json;
use std::sync::Arc;
fn calendar(zone: &Json) -> Calendar {
    let resource = &zone["resource"];
    let transitions = resource["transitions"].as_array().unwrap();
    Calendar::new(CalendarZone::Local(Arc::new(TimeZoneRules {
        version: 1,
        zone: zone["zone"].as_str().unwrap().into(),
        revision: Revision::new(7),
        tzdata: zone["tzdata"].as_str().unwrap().into(),
        coverage: TimeBounds {
            start: resource["start"].as_i64().unwrap(),
            end: resource["end"].as_i64().unwrap(),
        },
        initial_offset_seconds: transitions[0]["offset_seconds"].as_i64().unwrap() as i32,
        transitions: transitions
            .iter()
            .skip(1)
            .map(|t| TimeZoneTransition {
                at_millis: t["at"].as_i64().unwrap(),
                offset_seconds: t["offset_seconds"].as_i64().unwrap() as i32,
            })
            .collect(),
    })))
    .unwrap()
}
fn field(name: &str) -> CalendarUnit {
    match name {
        "second" => CalendarUnit::Second,
        "minute" => CalendarUnit::Minute,
        "hour" => CalendarUnit::Hour,
        "day" => CalendarUnit::Day,
        "week" => CalendarUnit::Week(WeekStart::Sunday),
        "month" => CalendarUnit::Month,
        "year" => CalendarUnit::Year,
        _ => panic!("unrecognized field"),
    }
}
fn expanded_field(name: &str) -> CalendarUnit {
    match name {
        "Millisecond" => CalendarUnit::Millisecond,
        "Second" => CalendarUnit::Second,
        "Minute" => CalendarUnit::Minute,
        "Hour" => CalendarUnit::Hour,
        "Day" => CalendarUnit::Day,
        "UnixDay" => CalendarUnit::UnixDay,
        "Sunday" => CalendarUnit::Week(WeekStart::Sunday),
        "Monday" => CalendarUnit::Week(WeekStart::Monday),
        "Month" => CalendarUnit::Month,
        "Year" => CalendarUnit::Year,
        _ => panic!("unrecognized field"),
    }
}
#[test]
fn every_fixed_zone_interval_matches_floor_ceil_offset_and_ticks() {
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/local-time.json"
    ))
    .unwrap();
    let mut errors = Vec::new();
    let mut count = 0;
    for zone in corpus["zones"].as_array().unwrap() {
        let calendar = calendar(zone);
        let wire = serde_json::to_string(calendar.zone()).unwrap();
        assert_eq!(
            &serde_json::from_str::<CalendarZone>(&wire).unwrap(),
            calendar.zone()
        );
        for record in zone["records"].as_array().unwrap() {
            let start = record["domain"][0].as_i64().unwrap();
            let stop = record["domain"][1].as_i64().unwrap();
            for query in record["intervals"].as_array().unwrap() {
                count += 1;
                let name = query["name"].as_str().unwrap();
                let interval = CalendarInterval::new(field(name));
                let every = query["every"].as_f64().unwrap();
                let filtered = CalendarInterval::every(field(name), every)
                    .unwrap()
                    .unwrap();
                for (method, actual) in [
                    (
                        "floor",
                        calendar.floor(start, TimeUnit::Milliseconds, interval),
                    ),
                    (
                        "ceil",
                        calendar.ceil(start, TimeUnit::Milliseconds, interval),
                    ),
                    (
                        "offset",
                        calendar.offset(start, TimeUnit::Milliseconds, interval, every),
                    ),
                ] {
                    if actual.as_ref().ok() != query[method].as_i64().as_ref() {
                        errors.push(format!(
                            "{} {} {name} {method}: {actual:?} != {}",
                            zone["zone"], record["id"], query[method]
                        ));
                    }
                }
                let nice = calendar
                    .floor(start, TimeUnit::Milliseconds, filtered)
                    .and_then(|a| {
                        calendar
                            .ceil(stop, TimeUnit::Milliseconds, filtered)
                            .map(|b| vec![a, b])
                    });
                if nice
                    .as_ref()
                    .ok()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .as_ref()
                    != Some(&query["nice"])
                {
                    errors.push(format!(
                        "{} {} {name} nice: {nice:?} != {}",
                        zone["zone"], record["id"], query["nice"]
                    ));
                }
                if query["ticks"].is_array() {
                    let ticks =
                        calendar.range(start, stop + 1, TimeUnit::Milliseconds, filtered, 10000);
                    if ticks
                        .as_ref()
                        .ok()
                        .map(|v| serde_json::to_value(v).unwrap())
                        .as_ref()
                        != Some(&query["ticks"])
                    {
                        errors.push(format!(
                            "{} {} {name} ticks: {ticks:?} != {}",
                            zone["zone"], record["id"], query["ticks"]
                        ));
                    }
                }
            }
        }
    }
    assert_eq!(count, 224);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
#[test]
fn automatic_fixed_zone_ticks_and_nice_match_all_reference_counts() {
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/local-time.json"
    ))
    .unwrap();
    let mut errors = Vec::new();
    let mut count = 0;
    for zone in corpus["zones"].as_array().unwrap() {
        let calendar = calendar(zone);
        for record in zone["records"].as_array().unwrap() {
            let domain = TimeBounds {
                start: record["domain"][0].as_i64().unwrap(),
                end: record["domain"][1].as_i64().unwrap(),
            };
            for query in record["automatic"].as_array().unwrap() {
                count += 1;
                let selection = CalendarTicks::Count(query["count"].as_f64().unwrap().into());
                let ticks = calendar.ticks(domain, TimeUnit::Milliseconds, selection, 10000);
                if ticks
                    .as_ref()
                    .ok()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .as_ref()
                    != Some(&query["ticks"])
                {
                    errors.push(format!(
                        "{} {} ticks {}: {ticks:?} != {}",
                        zone["zone"], record["id"], query["count"], query["ticks"]
                    ));
                }
                let nice = calendar
                    .nice(domain, TimeUnit::Milliseconds, selection)
                    .map(|d| vec![d.start, d.end]);
                if nice
                    .as_ref()
                    .ok()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .as_ref()
                    != Some(&query["nice"])
                {
                    errors.push(format!(
                        "{} {} nice {}: {nice:?} != {}",
                        zone["zone"], record["id"], query["count"], query["nice"]
                    ));
                }
                let formatter = TimeFormat::default().prepare(calendar.clone()).unwrap();
                let labels = query["ticks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| formatter.format(v.as_i64().unwrap(), TimeUnit::Milliseconds))
                    .collect::<chart_core::ChartResult<Vec<_>>>();
                if labels
                    .as_ref()
                    .ok()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .as_ref()
                    != Some(&query["labels"])
                {
                    errors.push(format!(
                        "{} {} labels {}: {labels:?} != {}",
                        zone["zone"], record["id"], query["count"], query["labels"]
                    ));
                }
            }
        }
    }
    assert_eq!(count, 128);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
#[test]
fn expanded_filtered_intervals_custom_formats_and_utc_day_selection_match() {
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/calendar/cases.json"
    ))
    .unwrap();
    let mut errors = Vec::new();
    let mut intervals = 0;
    let mut formats = 0;
    let mut automatic = 0;
    for zone in corpus["zones"].as_array().unwrap() {
        let calendar = if zone["mode"] == "Utc" {
            Calendar::new(CalendarZone::Utc).unwrap()
        } else {
            calendar(zone)
        };
        let locales: std::collections::BTreeMap<_, TimeLocale> = zone["locales"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| {
                (
                    l["id"].as_str().unwrap(),
                    serde_json::from_value(l["value"].clone()).unwrap(),
                )
            })
            .collect();
        for q in zone["intervals"].as_array().unwrap() {
            intervals += 1;
            let value = q["value"].as_i64().unwrap();
            let kind = expanded_field(q["name"].as_str().unwrap());
            let interval = CalendarInterval::every(kind, q["every"].as_f64().unwrap())
                .unwrap()
                .unwrap();
            for (method, actual) in [
                (
                    "floor",
                    calendar.floor(value, TimeUnit::Milliseconds, interval),
                ),
                (
                    "ceil",
                    calendar.ceil(value, TimeUnit::Milliseconds, interval),
                ),
                (
                    "round",
                    calendar.round(value, TimeUnit::Milliseconds, interval),
                ),
            ] {
                if actual.as_ref().ok() != q[method]["value"].as_i64().as_ref() {
                    errors.push(format!(
                        "{} {} {kind:?} {} at {value} {method}: {actual:?} != {}",
                        zone["mode"], zone["zone"], q["every"], q[method]
                    ));
                }
            }
            for row in q["offset"].as_array().unwrap() {
                let actual = calendar.offset(
                    value,
                    TimeUnit::Milliseconds,
                    interval,
                    row["step"].as_f64().unwrap(),
                );
                if actual.as_ref().ok() != row["result"]["value"].as_i64().as_ref() {
                    errors.push(format!(
                        "{} {} {kind:?} {} at {value} offset {row}: {actual:?}",
                        zone["mode"], zone["zone"], q["every"]
                    ));
                }
            }
            let span = match kind {
                CalendarUnit::Millisecond => 31,
                CalendarUnit::Second => 90000,
                CalendarUnit::Minute => 4 * 3600000,
                CalendarUnit::Hour => 4 * 86400000,
                _ => 35 * 86400000,
            };
            let range =
                calendar.range(value, value + span, TimeUnit::Milliseconds, interval, 10000);
            if range
                .as_ref()
                .ok()
                .map(|v| serde_json::to_value(v).unwrap())
                .as_ref()
                != Some(&q["range"]["value"])
            {
                errors.push(format!(
                    "{} {} {kind:?} {} at {value} range: {range:?} != {}",
                    zone["mode"], zone["zone"], q["every"], q["range"]
                ));
            }
        }
        for q in zone["formats"].as_array().unwrap() {
            formats += 1;
            let descriptor = TimeFormat {
                pattern: Some(q["pattern"].as_str().unwrap().into()),
                locale: locales[q["locale"].as_str().unwrap()].clone(),
            };
            let formatter = descriptor.prepare(calendar.clone()).unwrap();
            let actual = q["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| formatter.format(v.as_i64().unwrap(), TimeUnit::Milliseconds))
                .collect::<chart_core::ChartResult<Vec<_>>>();
            if actual
                .as_ref()
                .ok()
                .map(|v| serde_json::to_value(v).unwrap())
                .as_ref()
                != Some(&q["result"]["value"])
            {
                errors.push(format!(
                    "{} {} {} {} formats: {actual:?} != {}",
                    zone["mode"], zone["zone"], q["locale"], q["pattern"], q["result"]
                ));
            }
            assert_eq!(
                serde_json::from_str::<TimeFormat>(&serde_json::to_string(&descriptor).unwrap())
                    .unwrap(),
                descriptor
            );
        }
        for q in zone["automatic"].as_array().unwrap() {
            automatic += 1;
            let domain = TimeBounds {
                start: q["domain"][0].as_i64().unwrap(),
                end: q["domain"][1].as_i64().unwrap(),
            };
            let selection = CalendarTicks::Count(q["count"].as_f64().unwrap().into());
            let ticks = calendar.ticks(domain, TimeUnit::Milliseconds, selection, 10000);
            if ticks
                .as_ref()
                .ok()
                .map(|v| serde_json::to_value(v).unwrap())
                .as_ref()
                != Some(&q["ticks"]["value"])
            {
                errors.push(format!(
                    "{} {} {} ticks {}: {ticks:?} != {}",
                    zone["mode"], zone["zone"], q["id"], q["count"], q["ticks"]
                ));
            }
            let nice = calendar
                .nice(domain, TimeUnit::Milliseconds, selection)
                .map(|d| vec![d.start, d.end]);
            if nice
                .as_ref()
                .ok()
                .map(|v| serde_json::to_value(v).unwrap())
                .as_ref()
                != Some(&q["nice"]["value"])
            {
                errors.push(format!(
                    "{} {} {} nice {}: {nice:?} != {}",
                    zone["mode"], zone["zone"], q["id"], q["count"], q["nice"]
                ));
            }
            let formatter = TimeFormat::default().prepare(calendar.clone()).unwrap();
            let labels = q["ticks"]["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| formatter.format(v.as_i64().unwrap(), TimeUnit::Milliseconds))
                .collect::<chart_core::ChartResult<Vec<_>>>();
            if labels
                .as_ref()
                .ok()
                .map(|v| serde_json::to_value(v).unwrap())
                .as_ref()
                != Some(&q["labels"]["value"])
            {
                errors.push(format!(
                    "{} {} {} labels {}: {labels:?} != {}",
                    zone["mode"], zone["zone"], q["id"], q["count"], q["labels"]
                ));
            }
        }
    }
    println!(
        "Checked {intervals} filtered interval configurations, {formats} format configurations and {automatic} automatic queries."
    );
    assert_eq!(intervals, 2750);
    assert!(formats > 900);
    assert_eq!(automatic, 240);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn fixed_zone_time_mapping_and_inverse_match_reference() {
    use chart_core::interpolate::{FactoryKind, InterpolationFactory, Value};
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/calendar/cases.json"
    ))
    .unwrap();
    let mut errors = vec![];
    let mut count = 0;
    for zone in corpus["zones"].as_array().unwrap() {
        let calendar = if zone["mode"] == "Utc" {
            Calendar::new(CalendarZone::Utc).unwrap()
        } else {
            calendar(zone)
        };
        for q in zone["mapping"].as_array().unwrap() {
            count += 1;
            let spec = TimeScaleSpec {
                domain: q["domain"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect(),
                zone: calendar.zone().clone(),
                range: q["range"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| Value::number(v.as_f64().unwrap()))
                    .collect(),
                clamp: q["clamp"].as_bool().unwrap(),
                factory: InterpolationFactory::new(if q["round"] == true {
                    FactoryKind::Round
                } else {
                    FactoryKind::Value
                }),
                ..Default::default()
            };
            let scale = TimeScale::new(spec).unwrap();
            for (i, v) in q["values"].as_array().unwrap().iter().enumerate() {
                let Value::Number(actual) = scale.map(Some(v.as_i64().unwrap())).unwrap() else {
                    panic!("numeric range")
                };
                let expected = q["outputs"][i].as_f64().unwrap();
                if (actual.0 - expected).abs() > 1e-10_f64.max(expected.abs() * 1e-12) {
                    errors.push(format!("mapping {q} at {v}: {} != {expected}", actual.0));
                }
            }
            for (i, p) in q["positions"].as_array().unwrap().iter().enumerate() {
                let actual = scale.invert(p.as_f64().unwrap());
                let expected = q["inverse"][i].as_i64().unwrap();
                if actual.as_ref().ok() != Some(&expected) {
                    errors.push(format!(
                        "inverse {} {:?} at {p}: {actual:?} != {expected}",
                        q["id"],
                        scale.spec().domain
                    ));
                }
            }
        }
    }
    assert_eq!(count, 160);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn exact_time_units_integer_limits_and_typed_ranges() {
    use chart_core::{
        DiagnosticCode,
        interpolate::{FactoryKind, InterpolationFactory, Value},
    };
    for (unit, origin) in [
        (TimeUnit::Seconds, 9_007_199_254_740_993),
        (TimeUnit::Milliseconds, 9_007_199_254_740_993),
        (TimeUnit::Microseconds, 1_700_000_000_000_001),
        (TimeUnit::Nanoseconds, 1_700_000_000_000_000_001),
    ] {
        let scale = TimeScale::new(TimeScaleSpec {
            unit,
            domain: vec![origin, origin + 10],
            range: vec![Value::number(0.), Value::number(100.)],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(scale.map(Some(origin + 3)).unwrap(), Value::number(30.));
        assert_eq!(scale.invert(30.).unwrap(), origin + 3);
        assert_eq!(scale.invert(-1.).unwrap(), origin - 1);
        assert_eq!(scale.invert(1.).unwrap(), origin);
        assert_eq!(scale.map(None).unwrap(), Value::Missing);
        if matches!(unit, TimeUnit::Seconds | TimeUnit::Milliseconds) {
            assert!(
                scale
                    .tick_format(TimeFormat::default())
                    .unwrap()
                    .format(origin, unit)
                    .is_err()
            );
        }
    }
    for origin in [-100, 0, 100] {
        let scale = TimeScale::new(TimeScaleSpec {
            domain: vec![origin, origin + 10],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            scale.invert(0.15).unwrap(),
            (origin as f64 + 1.5).trunc() as i64
        );
        assert_eq!(
            scale.invert(-0.15).unwrap(),
            (origin as f64 - 1.5).trunc() as i64
        );
    }
    let bad = TimeScale::new(TimeScaleSpec {
        domain: vec![0, (1_i64 << 53) + 1],
        ..Default::default()
    });
    assert_eq!(bad.unwrap_err().code, DiagnosticCode::PrecisionLoss);
    let scale = TimeScale::new(TimeScaleSpec {
        domain: vec![0, 10],
        range: vec![Value::Text("0px".into()), Value::Text("20px".into())],
        factory: InterpolationFactory::new(FactoryKind::String),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(scale.map(Some(5)).unwrap(), Value::Text("10px".into()));
    assert_eq!(
        scale.invert(5.).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    for (unit, origin, step, label) in [
        (TimeUnit::Microseconds, 1_700_000_000_000_001, 1, ".000001"),
        (
            TimeUnit::Nanoseconds,
            1_700_000_000_000_000_001,
            1,
            ".000000001",
        ),
    ] {
        let scale = TimeScale::new(TimeScaleSpec {
            unit,
            domain: vec![origin, origin + 4],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            scale.ticks(CalendarTicks::Count(4.0.into()), 20).unwrap(),
            (0..5).map(|i| origin + i * step).collect::<Vec<_>>()
        );
        assert_eq!(
            scale
                .tick_format(TimeFormat::default())
                .unwrap()
                .format(origin, unit)
                .unwrap(),
            label
        );
    }
}

#[test]
fn resource_coverage_wall_resolution_and_invalid_requests_are_explicit() {
    use chart_core::DiagnosticCode;
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/local-time.json"
    ))
    .unwrap();
    let z = corpus["zones"]
        .as_array()
        .unwrap()
        .iter()
        .find(|z| z["zone"] == "America/New_York")
        .unwrap();
    let calendar = calendar(z);
    let CalendarZone::Local(rules) = calendar.zone() else {
        unreachable!()
    };
    let unit = TimeUnit::Milliseconds;
    assert_eq!(
        calendar
            .components(rules.coverage.start - 1, unit)
            .unwrap_err()
            .code,
        DiagnosticCode::MissingResource
    );
    assert_eq!(
        calendar
            .components(rules.coverage.end + 1, unit)
            .unwrap_err()
            .code,
        DiagnosticCode::MissingResource
    );
    assert!(calendar.components(rules.coverage.start, unit).is_ok());
    assert!(calendar.components(rules.coverage.end, unit).is_ok());
    let date = |month, day, hour, minute| CalendarDateTime {
        year: 2024,
        month,
        day,
        hour,
        minute,
        second: 0,
        nanosecond: 0,
        offset_seconds: 0,
    };
    assert_eq!(
        calendar.from_components(date(3, 10, 2, 30), unit).unwrap(),
        1_710_055_800_000
    ); // gap shifts to 03:30 EDT
    assert_eq!(
        calendar.from_components(date(11, 3, 1, 30), unit).unwrap(),
        1_730_611_800_000
    ); // earlier 01:30 EDT
    let mut bad = rules.as_ref().clone();
    bad.transitions.reverse();
    assert!(Calendar::new(CalendarZone::Local(Arc::new(bad))).is_err());
    let mut bad = rules.as_ref().clone();
    bad.transitions[0].at_millis = bad.coverage.start;
    assert!(bad.validate().is_err());
    let mut bad = rules.as_ref().clone();
    bad.initial_offset_seconds = 86401;
    assert!(bad.validate().is_err());
    let base = CalendarInterval::new(CalendarUnit::Hour);
    assert_eq!(
        calendar
            .ticks(
                TimeBounds {
                    start: 1_710_046_800_000,
                    end: 1_710_064_800_000
                },
                unit,
                CalendarTicks::Interval(base),
                2
            )
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert!(
        calendar
            .floor(
                1_710_046_800_000,
                unit,
                CalendarInterval {
                    unit: CalendarUnit::Hour,
                    step: 0
                }
            )
            .is_err()
    );
    assert!(
        CalendarInterval::every(CalendarUnit::Hour, 0.)
            .unwrap()
            .is_none()
    );
    assert!(
        CalendarInterval::every(CalendarUnit::Hour, f64::NAN)
            .unwrap()
            .is_none()
    );
    assert!(
        calendar
            .offset(1_710_046_800_000, unit, base, f64::INFINITY)
            .is_err()
    );
    let locale = TimeLocale {
        date_time: "%c".into(),
        ..Default::default()
    };
    assert!(
        TimeFormat {
            pattern: Some("%c".into()),
            locale
        }
        .prepare(calendar.clone())
        .is_err()
    );
    let utc = Calendar::new(CalendarZone::Utc).unwrap();
    let format = TimeFormat::default().prepare(utc.clone()).unwrap();
    assert_eq!(
        format.format(-8_640_000_000_000_000, unit).unwrap(),
        "-1821"
    );
    assert_eq!(
        format.format(8_640_000_000_000_000, unit).unwrap(),
        "Sat 13"
    );
    // Quantization preserves the declared coarse source resolution instead of rounding a boundary.
    assert_eq!(
        utc.floor(
            1,
            TimeUnit::Seconds,
            CalendarInterval {
                unit: CalendarUnit::Millisecond,
                step: 700
            }
        )
        .unwrap_err()
        .code,
        DiagnosticCode::PrecisionLoss
    );
    assert_eq!(
        TimeScaleSpec::local(CalendarZone::Utc).unwrap(),
        TimeScaleSpec::default()
    );
    assert_eq!(
        TimeScaleSpec::local(calendar.zone().clone())
            .unwrap_err()
            .code,
        DiagnosticCode::MissingResource
    );
    let spec = TimeScaleSpec {
        domain: vec![1_704_196_800_000, 1_706_529_600_000],
        zone: calendar.zone().clone(),
        ..Default::default()
    };
    let scale = TimeScale::new(spec).unwrap();
    let before = scale.to_json().unwrap();
    let copied = scale
        .nice(CalendarTicks::Interval(CalendarInterval::new(
            CalendarUnit::Month,
        )))
        .unwrap();
    assert_ne!(copied.spec().domain, scale.spec().domain);
    assert_eq!(scale.to_json().unwrap(), before);
    assert_eq!(TimeScale::from_json(&before).unwrap().spec(), scale.spec());
}

#[test]
fn original_time_scale_inventory_matches_declared_integer_date_overlap() {
    use chart_core::interpolate::Value;
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let zones: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/calendar/cases.json"
    ))
    .unwrap();
    let local = calendar(
        zones["zones"]
            .as_array()
            .unwrap()
            .iter()
            .find(|z| z["zone"] == "UTC" && z["mode"] == "Local")
            .unwrap(),
    );
    let mut count = 0;
    let mut fractional = 0;
    for c in corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["owner"] == "SP-06")
    {
        count += 1;
        let config = &c["config"];
        let zone = if c["factory"] == "scaleTime" {
            local.zone().clone()
        } else {
            CalendarZone::Utc
        };
        let mut spec = TimeScaleSpec {
            zone,
            ..Default::default()
        };
        if let Some(d) = config["domain"][0].as_array() {
            spec.domain = d.iter().map(|v| v["date"].as_i64().unwrap()).collect();
        }
        if let Some(r) = config["range"][0].as_array() {
            spec.range = r
                .iter()
                .map(|v| Value::number(v.as_f64().unwrap()))
                .collect();
        }
        let scale = TimeScale::new(spec).unwrap();
        for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
            let input = v["date"].as_i64().or_else(|| v.as_i64());
            let Some(input) = input else {
                fractional += 1;
                continue;
            }; // native timestamp API requires integer ticks in an explicit unit
            let Value::Number(actual) = scale.map(Some(input)).unwrap() else {
                panic!("numeric")
            };
            let expected = c["output"][i]["value"].as_f64().unwrap();
            assert!((actual.0 - expected).abs() < 1e-12 + expected.abs() * 1e-12);
        }
        let formatter = scale.tick_format(TimeFormat::default()).unwrap();
        for q in c["queries"]["invert"].as_array().unwrap() {
            let actual = scale.invert(
                q["input"]
                    .as_f64()
                    .or_else(|| q["input"]["date"].as_f64())
                    .unwrap(),
            );
            if let Some(expected) = q["value"]["date"].as_i64() {
                assert_eq!(actual.unwrap(), expected);
            } else {
                assert_eq!(q["value"]["date"]["number"], "NaN");
                assert_eq!(
                    actual.unwrap_err().code,
                    chart_core::DiagnosticCode::NumericalDomain
                );
            }
        }
        for q in c["queries"]["ticks"].as_array().unwrap() {
            assert_eq!(
                scale
                    .ticks(
                        CalendarTicks::Count(q["count"].as_f64().unwrap().into()),
                        10000
                    )
                    .unwrap(),
                q["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["date"].as_i64().unwrap())
                    .collect::<Vec<_>>()
            );
        }
        for q in c["queries"]["nice"].as_array().unwrap() {
            assert_eq!(
                scale
                    .nice(CalendarTicks::Count(q["count"].as_f64().unwrap().into()))
                    .unwrap()
                    .spec()
                    .domain,
                q["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["date"].as_i64().unwrap())
                    .collect::<Vec<_>>()
            );
        }
        for q in c["queries"]["labels"].as_array().unwrap() {
            for pair in q["value"].as_array().unwrap() {
                assert_eq!(
                    formatter
                        .format(pair[0]["date"].as_i64().unwrap(), TimeUnit::Milliseconds)
                        .unwrap(),
                    pair[1].as_str().unwrap()
                );
            }
        }
    }
    assert_eq!(count, 10);
    assert_eq!(fractional, 4);
}

#[test]
fn timestamp_clamp_and_omit_precede_integer_precision_conversion() {
    use chart_core::{DiagnosticCode, interpolate::Value};
    let spec = TimeScaleSpec {
        domain: vec![1_700_000_000_000_000_001, 1_700_000_000_000_000_011],
        unit: TimeUnit::Nanoseconds,
        clamp: true,
        ..Default::default()
    };
    let scale = TimeScale::new(spec.clone()).unwrap();
    assert_eq!(scale.map(Some(i64::MIN)).unwrap(), Value::number(0.));
    assert_eq!(scale.map(Some(i64::MAX)).unwrap(), Value::number(1.));
    let unclamped = TimeScaleSpec {
        clamp: false,
        ..spec.clone()
    };
    assert_eq!(
        TimeScale::new(unclamped.clone())
            .unwrap()
            .map(Some(i64::MIN))
            .unwrap_err()
            .code,
        DiagnosticCode::PrecisionLoss
    );
    for (outside, expected) in [
        (OutsidePolicy::Omit, None),
        (OutsidePolicy::Clamp, Some(0.)),
    ] {
        let axis = TimeAxisScale::resolve(
            unclamped.clone(),
            Bounds::new(0., 100.).unwrap(),
            None,
            outside,
        )
        .unwrap();
        assert_eq!(axis.map(i64::MIN).unwrap(), expected);
    }
    let axis = TimeAxisScale::resolve(
        spec,
        Bounds::new(0., 100.).unwrap(),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(axis.map(i64::MIN).unwrap(), Some(0.));
    let date = TimeScale::new(TimeScaleSpec {
        domain: vec![8_639_999_999_999_000, 8_640_000_000_000_000],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        date.invert(2.).unwrap_err().code,
        DiagnosticCode::NumericalDomain
    );
    let axis = TimeAxisScale::resolve(
        date.spec().clone(),
        Bounds::new(0., 100.).unwrap(),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(
        axis.invert(200.).unwrap_err().code,
        DiagnosticCode::NumericalDomain
    );
}
