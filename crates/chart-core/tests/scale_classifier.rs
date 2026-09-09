//! SP-04 / FIX-20 pinned classifier lookup and inverse-extent contracts.
use chart_core::{interpolate::Number, scales::*};
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
fn bound(actual: Option<Number>, expected: &Json) {
    let Some(Number(a)) = actual else {
        assert_eq!(expected["kind"], "Undefined");
        return;
    };
    let e = number(expected);
    assert!(
        a == e || a.is_nan() && e.is_nan() || (a - e).abs() <= 1e-12 + 1e-12 * e.abs(),
        "{a} != {e}"
    );
}
fn output<'a>(v: Option<&'a Json>, expected: &'a Json) {
    assert_eq!(v, (expected["kind"] != "Undefined").then_some(expected));
}
#[test]
fn all_pinned_classifier_cases_match_mapping_extents_and_cuts() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut counts = [0; 3];
    let mut errors = 0;
    for c in corpus["cases"].as_array().unwrap() {
        let family = c["factory"].as_str().unwrap();
        let lane = match family {
            "scaleQuantile" => 0,
            "scaleQuantize" => 1,
            "scaleThreshold" => 2,
            _ => continue,
        };
        counts[lane] += 1;
        let cfg = &c["config"];
        let range = cfg["range"][0].as_array().cloned().unwrap_or_else(|| {
            if lane == 0 {
                vec![]
            } else {
                vec![Json::from(0), Json::from(1)]
            }
        });
        let unknown = cfg.get("unknown").map(|v| v[0].clone());
        let domain = cfg["domain"][0]
            .as_array()
            .cloned()
            .unwrap_or_else(|| match lane {
                0 => vec![],
                1 => vec![Json::from(0), Json::from(1)],
                _ => vec![Json::from(0.5)],
            });
        if c["id"] == "threshold-text" {
            let s = ThresholdScale::new(ThresholdSpec {
                domain: domain
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect(),
                range,
                unknown,
            })
            .unwrap();
            for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
                output(
                    s.map(v.as_str().map(str::to_owned).as_ref()),
                    &c["output"][i]["value"],
                );
            }
            for q in c["queries"]["invertExtent"].as_array().unwrap() {
                let e = s.invert_extent(&q["input"]);
                for (a, b) in [e.lower, e.upper]
                    .iter()
                    .zip(q["value"].as_array().unwrap())
                {
                    assert_eq!(a.as_deref(), b.as_str());
                }
            }
        } else if lane == 2 {
            let s = ThresholdScale::new(ThresholdSpec {
                domain: domain.iter().map(number).collect(),
                range,
                unknown,
            })
            .unwrap();
            for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
                output(
                    s.map((!v.is_null()).then(|| number(v)).as_ref()),
                    &c["output"][i]["value"],
                );
            }
            for q in c["queries"]["invertExtent"].as_array().unwrap() {
                let e = s.invert_extent(&q["input"]);
                bound(e.lower.map(Number), &q["value"][0]);
                bound(e.upper.map(Number), &q["value"][1]);
            }
            let copy = s.clone();
            assert_eq!(copy.spec(), s.spec());
        } else {
            let domain = if lane == 0 {
                ClassifierDomain::Quantile(
                    domain
                        .iter()
                        .map(|v| (!v.is_null()).then(|| Number(number(v))))
                        .collect(),
                )
            } else {
                ClassifierDomain::Quantize([
                    Number(domain.first().map_or(f64::NAN, number)),
                    Number(domain.get(1).map_or(f64::NAN, number)),
                ])
            };
            let s = ClassifierScale::new(ClassifierSpec {
                domain,
                range,
                unknown,
            });
            if c["setup"].get("error").is_some() {
                assert!(s.is_err());
                errors += 1;
                continue;
            }
            let s = s.unwrap();
            for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
                output(
                    s.map((!v.is_null()).then(|| number(v))),
                    &c["output"][i]["value"],
                );
            }
            for q in c["queries"]["invertExtent"].as_array().unwrap() {
                let e = s.invert_extent(&q["input"]);
                bound(e.lower, &q["value"][0]);
                bound(e.upper, &q["value"][1]);
            }
            let cuts = &c["queries"][if lane == 0 { "quantiles" } else { "thresholds" }]["value"];
            assert_eq!(s.thresholds().len(), cuts.as_array().unwrap().len());
            for (a, e) in s.thresholds().iter().zip(cuts.as_array().unwrap()) {
                bound(*a, e);
            }
            for (a, e) in s
                .domain()
                .iter()
                .zip(c["getters"]["domain"]["value"].as_array().unwrap())
            {
                bound(Some(*a), e);
            }
            let copy = s.clone();
            assert_eq!(copy.spec(), s.spec());
        }
    }
    assert_eq!(counts, [19, 19, 20]);
    assert_eq!(errors, 6);
}
#[test]
fn generic_outputs_large_keys_and_population_corrections_are_independent() {
    let s = ThresholdScale::new(ThresholdSpec {
        domain: vec![u64::MAX - 2, u64::MAX - 1],
        range: vec![vec![1, 2], vec![3], vec![4, 5]],
        unknown: None,
    })
    .unwrap();
    assert_eq!(s.map(Some(&(u64::MAX - 1))), Some(&vec![4, 5]));
    assert_eq!(s.invert_extent(&vec![3]).lower, Some(u64::MAX - 2));
    let samples = [
        Some(Number(10.)),
        None,
        Some(Number(f64::NAN)),
        Some(Number(0.)),
        Some(Number(0.)),
        Some(Number(2.)),
    ];
    let spec = ClassifierSpec {
        domain: ClassifierDomain::Quantile(samples.to_vec()),
        range: vec![false, true],
        unknown: None,
    };
    let s = ClassifierScale::new(spec.clone()).unwrap();
    assert_eq!(s.thresholds(), &[Some(Number(1.))]);
    let corrected = ClassifierScale::new(ClassifierSpec {
        domain: ClassifierDomain::Quantile(vec![
            Some(Number(10.)),
            Some(Number(0.)),
            Some(Number(100.)),
            Some(Number(2.)),
        ]),
        ..spec
    })
    .unwrap();
    assert_eq!(corrected.thresholds(), &[Some(Number(6.))]);
    assert_eq!(s.map(Some(2.)), Some(&true));
    assert_eq!(corrected.map(Some(2.)), Some(&false));
    let json = serde_json::to_string(corrected.spec()).unwrap();
    assert_eq!(
        ClassifierScale::<bool>::new(serde_json::from_str(&json).unwrap()).unwrap(),
        corrected
    );
}
#[test]
fn reference_constructor_defaults_are_usable_without_manual_configuration() {
    use chart_core::interpolate::Value;
    assert_eq!(
        ClassifierScale::new(ClassifierSpec::quantile())
            .unwrap()
            .map(Some(0.)),
        None
    );
    assert_eq!(
        ClassifierScale::new(ClassifierSpec::quantize())
            .unwrap()
            .map(Some(0.5)),
        Some(&Value::number(1.))
    );
    let threshold = ThresholdScale::new(ThresholdSpec::d3()).unwrap();
    assert_eq!(
        threshold.map(Some(&ScaleKey::Number(Number(0.5)))),
        Some(&Value::number(1.))
    );
    let sequential =
        InterpolatedScale::new(InterpolatedScaleSpec::sequential(NumericFamily::Log {
            base: 10.,
        }))
        .unwrap();
    assert_eq!(sequential.map(Some(1.)).unwrap(), Value::number(0.));
    let diverging =
        InterpolatedScale::new(InterpolatedScaleSpec::diverging(NumericFamily::Linear)).unwrap();
    assert_eq!(diverging.map(Some(0.5)).unwrap(), Value::number(0.5));
    let rank = InterpolatedScale::new(InterpolatedScaleSpec::quantile()).unwrap();
    let Value::Number(Number(v)) = rank.map(Some(0.)).unwrap() else {
        panic!("identity rank result");
    };
    assert_eq!(v.to_bits(), (-0.0f64).to_bits());
    let continuous = ContinuousScale::new(ContinuousScaleSpec::d3(NumericFamily::Linear)).unwrap();
    assert_eq!(continuous.map(Some(0.5)).unwrap(), Value::number(0.5));
}
