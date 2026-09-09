//! SP-02 / FIX-20 numeric mapping and inverse observations; ticks remain SP-05.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::Value as Json;
fn number(v: &Json) -> f64 {
    match v.get("number").and_then(Json::as_str) {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        Some("-0") => -0.,
        _ => v.as_f64().unwrap(),
    }
}
fn equivalent(actual: &Value, expected: &Json) -> bool {
    match actual {
        Value::Missing => expected["kind"] == "Undefined",
        Value::Text(s) => expected == s,
        Value::Number(Number(a)) if expected.is_number() || expected.get("number").is_some() => {
            let e = number(expected);
            if !e.is_finite() {
                a.to_bits() == e.to_bits() || a.is_nan() && e.is_nan()
            } else {
                a.is_finite() && (*a - e).abs() <= 1e-12 + 1e-12 * e.abs()
            }
        }
        _ => false,
    }
}
#[test]
fn pinned_numeric_mapping_and_inverse_families_match() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut count = 0;
    let mut failures = Vec::new();
    for c in corpus["cases"].as_array().unwrap() {
        let family = match c["factory"].as_str().unwrap() {
            "scaleLinear" => NumericFamily::Linear,
            "scalePow" | "scaleSqrt" => NumericFamily::Pow {
                exponent: number(&c["getters"]["exponent"]["value"]),
            },
            "scaleLog" => NumericFamily::Log {
                base: number(&c["getters"]["base"]["value"]),
            },
            "scaleSymlog" => NumericFamily::Symlog {
                constant: number(&c["getters"]["constant"]["value"]),
            },
            "scaleIdentity" => NumericFamily::Identity,
            "scaleRadial" => NumericFamily::Radial,
            _ => continue,
        };
        let range = c["getters"]["range"]["value"].as_array().unwrap();
        if range
            .iter()
            .any(|v| !v.is_number() && v.get("number").is_none())
        {
            continue;
        } // Typed/color ranges are SP-04, not this numeric fixture scope.
        count += 1;
        let domain = c["getters"]["domain"]["value"].as_array().unwrap();
        let mut spec = NumericScaleSpec::d3(family);
        spec.domain = domain.iter().map(|v| Number(number(v))).collect();
        spec.range = range.iter().map(|v| Number(number(v))).collect();
        spec.clamp = c["getters"]["clamp"]["value"] == true;
        spec.round = c["getters"]["round"]["value"] == true
            || c["config"].get("rangeRound").is_some()
            || c["config"]["interpolate"][0]["name"] == "interpolateRound";
        if let Some(s) = c["getters"]["unknown"]["value"].as_str() {
            spec.unknown = Value::Text(s.into());
        }
        let s = NumericScale::new(spec).unwrap_or_else(|e| panic!("{}: {e}", c["id"]));
        assert_eq!(NumericScale::from_json(&s.to_json().unwrap()).unwrap(), s);
        for (i, input) in c["inputs"].as_array().unwrap().iter().enumerate() {
            let expected = &c["output"][i]["value"];
            let actual = s.map(if input.is_null() {
                None
            } else {
                Some(number(input))
            });
            let pass = match &actual {
                Ok(v) => equivalent(v, expected),
                Err(_) => expected.get("number").is_some_and(|v| v != "-0"),
            };
            if !pass {
                failures.push(format!(
                    "{} map({input}): {actual:?} != {expected}",
                    c["id"]
                ));
            }
        }
        for row in c["queries"]["nice"].as_array().unwrap() {
            let result = s.nice(number(&row["count"]));
            let expected = &row["value"];
            let matches = match &result {
                Ok(scale) => {
                    let actual = &scale.spec().domain;
                    let expected = expected.as_array().unwrap();
                    actual.len() == expected.len()
                        && actual
                            .iter()
                            .zip(expected)
                            .all(|(a, e)| equivalent(&Value::Number(*a), e))
                }
                Err(_) => expected
                    .as_array()
                    .is_some_and(|v| v.iter().any(|x| x.get("number").is_some_and(|n| n != "-0"))),
            };
            if !matches {
                failures.push(format!(
                    "{} nice({}): {result:?} != {expected}",
                    c["id"], row["count"]
                ));
            }
        }
        for row in c["queries"]["invert"].as_array().unwrap() {
            if row["input"].is_null() {
                continue;
            } // Implicit JS null-to-zero coercion is outside the typed inverse.
            let x = number(&row["input"]);
            let expected = &row["value"];
            let actual = s.invert_output(Some(x));
            let pass = match &actual {
                Ok(v) => equivalent(v, expected),
                Err(_) => expected.get("number").is_some_and(|v| v != "-0"),
            };
            if !pass {
                failures.push(format!("{} invert({x}): {actual:?} != {expected}", c["id"]));
            }
        }
    }
    assert_eq!(
        count, 161,
        "Every currently authored numeric-range case is required"
    );
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
#[test]
fn independent_piecewise_radial_clamp_and_copy_contracts() {
    let original = NumericScale::linear();
    let changed = original
        .with_domain([0., 10., 100.])
        .unwrap()
        .with_range([0., 50., 100.])
        .unwrap();
    assert_eq!(changed.map(Some(10.)).unwrap(), Value::number(50.));
    assert_eq!(changed.invert(50.).unwrap(), 10.);
    assert_eq!(original.map(Some(10.)).unwrap(), Value::number(10.));
    let mut spec = NumericScaleSpec::d3(NumericFamily::Radial);
    spec.range = vec![Number(0.), Number(10.)];
    let radial = NumericScale::new(spec).unwrap();
    assert_eq!(radial.map(Some(0.25)).unwrap(), Value::number(5.));
    assert_eq!(radial.invert(5.).unwrap(), 0.25);
    let mut spec = (*changed.spec()).clone();
    spec.clamp = true;
    let clamped = NumericScale::new(spec).unwrap();
    assert_eq!(clamped.invert(200.).unwrap(), 100.);
    assert_eq!(clamped.invert_unbounded(200.).unwrap(), 280.);
    assert!(changed.with_domain([0., 2., 1.]).is_err());
    assert_eq!(changed.invert(50.).unwrap(), 10.);
    let mut wire: Json = serde_json::from_str(&changed.to_json().unwrap()).unwrap();
    wire["version"] = 2.into();
    assert!(NumericScale::from_json(&wire.to_string()).is_err());
}

#[test]
fn chart_clamp_uses_effective_mapping_domain() {
    use chart_core::{
        data::TimeUnit,
        scales::{
            Bounds, NumericAxisScale, OutsidePolicy, TimeAxisScale, TimeScale, TimeScaleSpec,
        },
    };
    for reverse in [false, true] {
        let mut spec = NumericScaleSpec::d3(NumericFamily::Linear);
        spec.domain = if reverse {
            [100., 10., 0.]
        } else {
            [0., 10., 100.]
        }
        .map(Number)
        .to_vec();
        spec.range = [0., 100.].map(Number).to_vec();
        spec.clamp = true;
        let standalone = NumericScale::new(spec.clone()).unwrap();
        let axis = NumericAxisScale::resolve(
            spec.clone(),
            Bounds::new(0., 100.).unwrap(),
            None,
            OutsidePolicy::Extend,
        )
        .unwrap();
        for x in [-50., 0., 10., 50., 100., 150.] {
            assert_eq!(axis.map(x).unwrap(), standalone.map_finite(x).unwrap());
            assert_eq!(axis.invert(x).unwrap(), standalone.invert(x).unwrap());
        }
        let time_spec = TimeScaleSpec {
            domain: spec.domain.iter().map(|n| n.0 as i64).collect(),
            unit: TimeUnit::Nanoseconds,
            range: spec.range.iter().copied().map(Value::Number).collect(),
            clamp: true,
            ..Default::default()
        };
        let standalone = TimeScale::new(time_spec.clone()).unwrap();
        let axis = TimeAxisScale::resolve(
            time_spec,
            Bounds::new(0., 100.).unwrap(),
            None,
            OutsidePolicy::Extend,
        )
        .unwrap();
        for x in [-50, 0, 10, 50, 100, 150] {
            assert_eq!(
                axis.map(x).unwrap().map(Value::number).unwrap(),
                standalone.map(Some(x)).unwrap()
            );
            assert_eq!(
                axis.invert(x as f64).unwrap(),
                standalone.invert(x as f64).unwrap()
            );
        }
    }
}
