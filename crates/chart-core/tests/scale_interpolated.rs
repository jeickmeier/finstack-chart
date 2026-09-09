//! SP-04 / FIX-20 sequential, diverging, sample-rank and shared interpolation contracts.
use chart_core::{interpolate::*, scales::*};
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
fn value(v: &Json) -> Value {
    if let Some(s) = v.as_str() {
        Value::Text(s.to_owned())
    } else if v["kind"] == "Undefined" {
        Value::Missing
    } else {
        Value::number(number(v))
    }
}
fn close(a: &Value, e: &Json) -> bool {
    match a {
        Value::Missing => e["kind"] == "Undefined",
        Value::Text(s) => e == s,
        Value::Number(Number(a)) if e.is_number() || e.get("number").is_some() => {
            let e = number(e);
            a.to_bits() == e.to_bits()
                || a.is_nan() && e.is_nan()
                || e != 0. && (*a - e).abs() <= 1e-12 + 1e-12 * e.abs()
        }
        _ => false,
    }
}
#[test]
fn all_pinned_sequential_diverging_and_rank_cases_match() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut counts = [0; 3];
    let mut failures = vec![];
    for c in corpus["cases"].as_array().unwrap() {
        let name = c["factory"].as_str().unwrap();
        if !name.starts_with("scaleSequential") && !name.starts_with("scaleDiverging") {
            continue;
        }
        let rank = name == "scaleSequentialQuantile";
        let diverging = name.starts_with("scaleDiverging");
        counts[if rank { 2 } else { usize::from(diverging) }] += 1;
        let family = if name.ends_with("Log") {
            NumericFamily::Log {
                base: number(&c["getters"]["base"]["value"]),
            }
        } else if name.ends_with("Symlog") {
            NumericFamily::Symlog {
                constant: number(&c["getters"]["constant"]["value"]),
            }
        } else if name.ends_with("Pow") || name.ends_with("Sqrt") {
            NumericFamily::Pow {
                exponent: number(&c["getters"]["exponent"]["value"]),
            }
        } else {
            NumericFamily::Linear
        };
        let domain: Vec<_> = c["getters"]["domain"]["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Number(number(v)))
            .collect();
        let clamp = c["getters"]["clamp"]["value"] == true;
        let normalization = if rank {
            NormalizationSpec::Quantile {
                samples: domain.into_iter().map(Some).collect(),
            }
        } else if diverging {
            NormalizationSpec::Diverging {
                family,
                domain: domain.try_into().unwrap(),
                clamp,
            }
        } else {
            NormalizationSpec::Sequential {
                family,
                domain: domain.try_into().unwrap(),
                clamp,
            }
        };
        let output = if c["config"].get("interpolator").is_some() {
            ScaleRangeFunction::Interpolate(InterpolationSpec::Between {
                factory: InterpolationFactory::new(FactoryKind::Rgb)
                    .with_gamma(2.)
                    .unwrap(),
                a: Value::Text("red".into()),
                b: Value::Text("blue".into()),
            })
        } else if let Some(range) = c["config"]["range"][0].as_array() {
            let values: Vec<_> = range.iter().map(value).collect();
            let factory = InterpolationFactory::new(FactoryKind::Value);
            ScaleRangeFunction::Interpolate(if diverging {
                InterpolationSpec::Piecewise { factory, values }
            } else {
                InterpolationSpec::Between {
                    factory,
                    a: values[0].clone(),
                    b: values[1].clone(),
                }
            })
        } else {
            ScaleRangeFunction::Identity
        };
        let spec = InterpolatedScaleSpec {
            normalization,
            output,
            unknown: c["config"]
                .get("unknown")
                .map_or(Value::Missing, |v| value(&v[0])),
        };
        let s = InterpolatedScale::new(spec).unwrap();
        for (i, input) in c["inputs"].as_array().unwrap().iter().enumerate() {
            // D3 diverging coerces null to 0; typed None remains missing. Compare its
            // explicitly equivalent numeric input and independently test None below.
            let x = if input.is_null() {
                diverging.then_some(0.)
            } else {
                Some(number(input))
            };
            let actual = s.map(x);
            let expected = &c["output"][i];
            if expected.get("error").is_some() {
                if actual.is_ok() {
                    failures.push(format!("{}[{i}] expected error got {actual:?}", c["id"]));
                }
            } else if !actual.as_ref().is_ok_and(|a| close(a, &expected["value"])) {
                failures.push(format!(
                    "{}[{i}] got {actual:?} expected {expected}",
                    c["id"]
                ));
            }
        }
        let expected_range = &c["getters"]["range"];
        let range = s.range();
        if expected_range.get("error").is_some() {
            assert!(range.is_err());
        } else {
            let a = range.unwrap();
            let e = expected_range["value"].as_array().unwrap();
            assert_eq!(a.len(), e.len());
            for (a, e) in a.iter().zip(e) {
                assert!(close(a, e), "{} range {a:?} != {e}", c["id"]);
            }
        }
        if rank {
            for q in c["queries"]["quantiles"].as_array().unwrap() {
                let quantiles = s.normalizer().quantiles(number(&q["count"])).unwrap();
                assert_eq!(quantiles.len(), q["value"].as_array().unwrap().len());
                for (a, e) in quantiles.iter().zip(q["value"].as_array().unwrap()) {
                    assert!(close(&a.map_or(Value::Missing, Value::Number), e));
                }
            }
        }
        assert_eq!(s.map(None).unwrap(), s.spec().unknown);
        let json = serde_json::to_string(s.spec()).unwrap();
        let restored = InterpolatedScale::new(serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(restored.spec(), s.spec());
    }
    assert_eq!(counts, [40, 40, 7]);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn custom_typed_interpolation_midpoint_rounding_and_rank_degeneracies() {
    let spec = NormalizationSpec::Diverging {
        family: NumericFamily::Linear,
        domain: [Number(-10.), Number(0.), Number(100.)],
        clamp: false,
    };
    let n = ScaleNormalizer::new(spec.clone()).unwrap();
    let callback = |t| Ok(vec![t, 1. - t]);
    assert_eq!(
        n.map_with(Some(0.), &callback).unwrap(),
        Some(vec![0.5, 0.5])
    );
    assert_eq!(
        n.map_with(Some(50.), &callback).unwrap(),
        Some(vec![0.75, 0.25])
    );
    let s = InterpolatedScale::new(InterpolatedScaleSpec {
        normalization: spec,
        output: ScaleRangeFunction::Interpolate(InterpolationSpec::Piecewise {
            factory: InterpolationFactory::new(FactoryKind::Round),
            values: vec![Value::number(0.), Value::number(3.), Value::number(10.)],
        }),
        unknown: Value::Missing,
    })
    .unwrap();
    assert_eq!(s.map(Some(-5.)).unwrap(), Value::number(2.));
    assert_eq!(s.map(Some(50.)).unwrap(), Value::number(7.));
    let singleton = ScaleNormalizer::new(NormalizationSpec::Quantile {
        samples: vec![Some(Number(7.))],
    })
    .unwrap();
    assert!(singleton.parameter(Some(7.)).unwrap().is_nan());
    let empty = ScaleNormalizer::new(NormalizationSpec::Quantile { samples: vec![] }).unwrap();
    assert!(empty.quantiles(f64::NAN).unwrap().is_empty());
    assert!(empty.quantiles(f64::NEG_INFINITY).unwrap().is_empty());
    assert!(empty.quantiles(f64::INFINITY).is_err());
    assert_eq!(
        empty.parameter(Some(7.)).unwrap().to_bits(),
        (-0.0f64).to_bits()
    );
    // Exceptional scale sampling must not alter the public interpolation parameter contract.
    let interpolation = InterpolationFactory::new(FactoryKind::Value)
        .between(Value::number(0.), Value::number(1.))
        .unwrap();
    assert!(interpolation.sample(f64::NAN).is_err());
}
#[test]
fn continuous_typed_ranges_use_shared_numeric_knots() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut count = 0;
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
            _ => continue,
        };
        let range = c["getters"]["range"]["value"].as_array().unwrap();
        if range
            .iter()
            .all(|v| v.is_number() || v.get("number").is_some())
        {
            continue;
        }
        let s = ContinuousScale::new(ContinuousScaleSpec {
            family,
            domain: c["getters"]["domain"]["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Number(number(v)))
                .collect(),
            range: range.iter().map(value).collect(),
            factory: InterpolationFactory::new(
                c["config"]["interpolate"][0]["name"]
                    .as_str()
                    .map_or(FactoryKind::Value, |name| {
                        FactoryKind::from_name(name).unwrap()
                    }),
            ),
            clamp: c["getters"]["clamp"]["value"] == true,
            unknown: c["config"]
                .get("unknown")
                .map_or(Value::Missing, |v| value(&v[0])),
        })
        .unwrap();
        for (i, input) in c["inputs"].as_array().unwrap().iter().enumerate() {
            let a = s.map((!input.is_null()).then(|| number(input))).unwrap();
            assert!(
                close(&a, &c["output"][i]["value"]),
                "{}[{i}] {a:?} != {}",
                c["id"],
                c["output"][i]
            );
        }
        count += 1;
    }
    assert_eq!(count, 15);
    let s = ContinuousScale::new(ContinuousScaleSpec {
        family: NumericFamily::Linear,
        domain: vec![Number(0.), Number(10.), Number(100.)],
        range: vec![
            Value::Array(vec![Value::number(0.), Value::Boolean(true)]),
            Value::Array(vec![Value::number(50.), Value::Boolean(false)]),
            Value::Array(vec![Value::number(100.), Value::Boolean(true)]),
        ],
        factory: InterpolationFactory::new(FactoryKind::Value),
        clamp: true,
        unknown: Value::Null,
    })
    .unwrap();
    assert_eq!(
        s.map(Some(5.)).unwrap(),
        Value::Array(vec![Value::number(25.), Value::Boolean(false)])
    );
    assert_eq!(
        s.map(Some(55.)).unwrap(),
        Value::Array(vec![Value::number(75.), Value::Boolean(true)])
    );
    assert_eq!(s.map(None).unwrap(), Value::Null);
}
