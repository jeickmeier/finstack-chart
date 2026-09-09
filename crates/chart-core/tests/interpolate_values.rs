//! FIX-I01-A/B/C/D: target dispatch, strings, dates, arrays, bounds and owned reuse.
use chart_core::{ChartResult, interpolate::*};
use serde_json::Value as Json;
fn compare(a: &Value, e: &Value, id: &str) {
    match (a, e) {
        (Value::Number(a), Value::Number(e)) => {
            if e.0.is_nan() {
                assert!(a.0.is_nan(), "{id}");
            } else if e.0.is_infinite() || e.0 == 0. {
                assert_eq!(a, e, "{id}");
            } else {
                assert!(
                    (a.0 - e.0).abs() <= 1e-12 + 1e-12 * e.0.abs(),
                    "{id}: {} != {}",
                    a.0,
                    e.0
                );
            }
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len(), "{id}");
            for (a, e) in a.iter().zip(e) {
                compare(a, e, id);
            }
        }
        (Value::Record(a), Value::Record(e)) => {
            assert_eq!(a.len(), e.len(), "{id}");
            for (k, e) in e {
                compare(&a[k], e, id);
            }
        }
        (Value::NumericArray(a), Value::NumericArray(e)) => {
            assert_eq!(a.element(), e.element(), "{id}");
            assert_eq!(a.values().len(), e.values().len(), "{id}");
            for (a, e) in a.values().iter().zip(e.values()) {
                if a.0.is_nan()
                    || e.0.is_nan()
                    || a.0.is_infinite()
                    || e.0.is_infinite()
                    || a.0 == 0.
                    || e.0 == 0.
                    || a.0 as f32 as f64 == a.0 && e.0 as f32 as f64 == e.0
                {
                    assert_eq!(a, e, "{id}");
                } else {
                    assert!((a.0 - e.0).abs() <= 1e-12 + 1e-12 * e.0.abs(), "{id}");
                }
            }
        }
        _ => assert_eq!(a, e, "{id}"),
    }
}
#[test]
fn every_structured_value_case_matches_offline_reference_and_adaptations() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-interpolate/cases.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["op"] == "piecewise" && c["config"]["factory"].is_null() {
            count += 1;
            let controls: Value = serde_json::from_value(c["args"][0].clone()).unwrap();
            let Value::Array(controls) = controls else {
                panic!()
            };
            let f = Piecewise::new(&controls, ValueInterpolator::new).unwrap();
            for (t, e) in c["times"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["expected"].as_array().unwrap())
            {
                let e: Value = serde_json::from_value(e.clone()).unwrap();
                compare(
                    &f.sample(t.as_f64().unwrap()).unwrap(),
                    &e,
                    "piecewise-default",
                );
            }
            continue;
        }
        let op = match c["op"].as_str().unwrap() {
            "interpolate" => ValueOperation::Value,
            "interpolateString" => ValueOperation::String,
            "interpolateDate" => ValueOperation::Date,
            "interpolateArray" => ValueOperation::Array,
            "interpolateNumberArray" => ValueOperation::NumberArray,
            "interpolateObject" => ValueOperation::Object,
            _ => continue,
        };
        count += 1;
        let id = c["id"].as_str().unwrap();
        let a: Value = serde_json::from_value(c["args"][0].clone()).unwrap();
        let b: Value = serde_json::from_value(c["args"][1].clone()).unwrap();
        let f = ValueInterpolator::with_operation(op, &a, &b);
        if !c["adaptation"].is_null() {
            assert!(f.is_err(), "{id}: adaptation {:?}", c["adaptation"]);
            continue;
        }
        let f = f.unwrap_or_else(|e| panic!("{id}: {e}"));
        let mut reused = Value::Missing;
        for (t, e) in c["times"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["expected"].as_array().unwrap())
        {
            let t = t.as_f64().unwrap();
            let expected: Value = serde_json::from_value(e.clone()).unwrap();
            let actual = f.sample(t).unwrap();
            compare(&actual, &expected, id);
            f.sample_into(t, &mut reused).unwrap();
            compare(&reused, &expected, id);
            assert_eq!(
                Value::from_json(&actual.to_json().unwrap()).unwrap(),
                actual,
                "{id}: owned wire round trip"
            );
        }
    }
    assert_eq!(count, 191);
}
#[test]
fn independent_target_shapes_dates_casts_and_reuse() {
    let a=Value::from_json(r#"{"kind":"Record","value":{"x":{"kind":"Number","value":2},"old":{"kind":"Number","value":9}}}"#).unwrap();
    let b=Value::from_json(r#"{"kind":"Record","value":{"x":{"kind":"Number","value":10},"new":{"kind":"Text","value":"fixed"}}}"#).unwrap();
    let f = ValueInterpolator::new(&a, &b).unwrap();
    let held = f.sample(0.5).unwrap();
    let Value::Record(fields) = &held else {
        panic!()
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields["x"], Value::number(6.));
    assert_eq!(fields["new"], Value::Text("fixed".into()));
    let samples = quantize(&f, 3).unwrap();
    assert_ne!(samples[0], samples[2]);
    assert_eq!(samples[1], held);
    let piece = Piecewise::new(&[a.clone(), b.clone(), a], ValueInterpolator::new).unwrap();
    assert_eq!(piece.sample(0.25).unwrap(), held);
    let f = ValueInterpolator::with_operation(
        ValueOperation::String,
        &Value::Text("x0 y1".into()),
        &Value::Text("q8 z5".into()),
    )
    .unwrap();
    assert_eq!(f.sample(0.5).unwrap(), Value::Text("q4 z3".into()));
    let mut out = Value::Text(String::with_capacity(256));
    let Value::Text(s) = &out else { panic!() };
    let pointer = s.as_ptr();
    for t in [0., 0.5, 1.] {
        f.sample_into(t, &mut out).unwrap();
        let Value::Text(s) = &out else { panic!() };
        assert_eq!(s.as_ptr(), pointer);
    }
    let before = out.clone();
    assert!(f.sample_into(f64::NAN, &mut out).is_err());
    assert_eq!(before, out);
    assert_eq!(Value::date(-0.75), Value::date(0.));
    assert_eq!(Value::date(-1.75), Value::date(-1.));
    assert_eq!(Value::date(8.64e15 + 1.), Value::date(f64::NAN));
    assert_eq!(NumericKind::Int8.cast(255.), -1.);
    assert_eq!(NumericKind::Uint8.cast(-1.5), 255.);
    assert_eq!(NumericKind::Uint16.cast(65537.), 1.);
    assert_eq!(NumericKind::Int32.cast(4294967295.), -1.);
    assert_eq!(NumericKind::Uint32.cast(-1.), 4294967295.);
    assert_eq!(NumericKind::Uint8Clamped.cast(1.5), 2.);
    assert_eq!(NumericKind::Uint8Clamped.cast(2.5), 2.);
    assert_eq!(NumericKind::Uint8Clamped.cast(f64::NAN), 0.);
    assert_eq!(NumericKind::Float32.cast(16777217.), 16777216.);
    let f = ValueInterpolator::new(
        &Value::Null,
        &Value::NumericArray(NumericArray::new(NumericKind::Float64, vec![1., 2.]).unwrap()),
    )
    .unwrap();
    let mut out = f.sample(0.).unwrap();
    let Value::NumericArray(v) = &out else {
        panic!()
    };
    let pointer = v.values().as_ptr();
    f.sample_into(1., &mut out).unwrap();
    let Value::NumericArray(v) = &out else {
        panic!()
    };
    assert_eq!(pointer, v.values().as_ptr());
}
#[test]
fn strict_value_decoding_and_aggregate_limits_reject_invalid_inputs() {
    for json in [
        r#"{"kind":"Number","value":null}"#,
        r#"{"kind":"Number","value":{"number":"bad"}}"#,
        r#"{"kind":"Number","value":1,"extra":1}"#,
        r#"{"kind":"Date","value":0.5}"#,
        r#"{"kind":"Date","value":{"number":"Infinity"}}"#,
        r#"{"kind":"Record","value":{"x":{"kind":"Null"},"x":{"kind":"Missing"}}}"#,
        r#"{"kind":"NumericArray","value":{"element":"BigInt64Array","values":[1]}}"#,
    ] {
        assert!(Value::from_json(json).is_err(), "{json}");
    }
    let mut nested = Value::Null;
    for _ in 0..MAX_VALUE_DEPTH + 2 {
        nested = Value::Array(vec![nested]);
    }
    assert!(nested.validate().is_err());
    assert!(
        Value::Array(vec![Value::Null; MAX_VALUES])
            .validate()
            .is_err()
    );
    assert!(
        Value::Text("x".repeat(MAX_VALUE_BYTES + 1))
            .validate()
            .is_err()
    );
    let input = Value::Text("1 ".repeat(MAX_VALUES + 1));
    assert!(ValueInterpolator::with_operation(ValueOperation::String, &input, &input).is_err());
    let a = Value::Text("1 ".repeat(170_000));
    let b = Value::Text("2 ".repeat(170_000));
    assert!(
        ValueInterpolator::with_operation(ValueOperation::String, &a, &b).is_err(),
        "potential sampled text budget"
    );
    for v in [0., -0., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let v = Value::number(v);
        assert_eq!(Value::from_json(&v.to_json().unwrap()).unwrap(), v);
    }
    let unsupported: ChartResult<_> =
        ValueInterpolator::new(&Value::Array(vec![]), &Value::number(1.));
    assert!(unsupported.is_err());
}
