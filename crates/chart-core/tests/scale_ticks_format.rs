//! FIX-20 / SCL-07: exact independent reference tick arrays and label strings.
use chart_core::{interpolate::Number, scales::*, typography::*};
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
fn same(a: f64, e: f64) -> bool {
    a.to_bits() == e.to_bits() || a.is_nan() && e.is_nan()
}
#[test]
fn all_pinned_numeric_ticks_and_default_labels() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut errors = Vec::new();
    let mut cases = 0;
    let mut queries = 0;
    for c in corpus["cases"].as_array().unwrap() {
        let name = c["factory"].as_str().unwrap();
        if !c["queries"]["ticks"].is_array() || name.contains("Time") || name.contains("Utc") {
            continue;
        }
        cases += 1;
        let family = if name.contains("Log") {
            NumericFamily::Log {
                base: number(&c["getters"]["base"]["value"]),
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
        for row in c["queries"]["ticks"].as_array().unwrap() {
            queries += 1;
            let actual = family.ticks(&domain, number(&row["count"]), 10000);
            let pass = match &actual {
                Ok(a) => row["value"].as_array().is_some_and(|e| {
                    a.len() == e.len() && a.iter().zip(e).all(|(a, e)| same(*a, number(e)))
                }),
                Err(_) => row.get("error").is_some(),
            };
            if !pass {
                errors.push(format!(
                    "{} ticks {}: {actual:?} != {row}",
                    c["id"], row["count"]
                ));
            }
        }
        for row in c["queries"]["labels"].as_array().unwrap() {
            let count = number(&row["count"]);
            let actual = family.ticks(&domain, count, 10000).and_then(|ticks| {
                family
                    .tick_format(&domain, count, None, NumericLocale::default())
                    .map(|f| {
                        ticks
                            .into_iter()
                            .map(|x| (x, f.format(x)))
                            .collect::<Vec<_>>()
                    })
            });
            let pass = match &actual {
                Ok(a) => row["value"].as_array().is_some_and(|e| {
                    a.len() == e.len()
                        && a.iter().zip(e).all(|((x, a), e)| {
                            same(*x, number(&e[0])) && Some(a.as_str()) == e[1].as_str()
                        })
                }),
                Err(_) => row.get("error").is_some(),
            };
            if !pass {
                errors.push(format!(
                    "{} labels {}: {actual:?} != {row}",
                    c["id"], row["count"]
                ));
            }
        }
    }
    println!(
        "Checked {cases} scale configurations and {queries} tick plus {queries} label queries."
    );
    assert!(cases > 200);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
#[test]
fn all_pinned_standalone_tick_formats() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut errors = Vec::new();
    let rows = corpus["format_cases"].as_array().unwrap();
    assert_eq!(rows.len(), 200);
    for c in rows {
        let (a, b, n) = (number(&c["start"]), number(&c["stop"]), number(&c["count"]));
        let actual = tick_format(a, b, n, c["specifier"].as_str(), NumericLocale::default())
            .map(|f| [a, (a + b) / 2., b, -0.].map(|x| f.format(x)));
        let pass = match &actual {
            Ok(v) => serde_json::to_value(v).unwrap() == c["result"]["value"],
            Err(_) => c["result"].get("error").is_some(),
        };
        if !pass {
            errors.push(format!("{c}: {actual:?}"));
        }
    }
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
#[test]
fn independent_resource_sign_rounding_and_copy_contracts() {
    assert_eq!(
        tick_candidates(0., 1., 4., 100).unwrap(),
        vec![0., 0.2, 0.4, 0.6, 0.8, 1.]
    );
    assert!(tick_candidates(0., 1., 4., 5).is_err());
    assert_eq!(
        tick_candidates(1., 0., 4., 100).unwrap(),
        vec![1., 0.8, 0.6, 0.4, 0.2, 0.]
    );
    assert!(tick_candidates(0., 1., f64::NAN, 100).unwrap().is_empty());
    assert_eq!(tick_candidates(0., 0., f64::INFINITY, 1).unwrap(), vec![0.]);
    assert!(tick_candidates(0., 0., 1., 0).is_err());
    let f = |s: &str, x| {
        NumericFormat {
            specifier: s.into(),
            locale: Default::default(),
        }
        .prepare()
        .unwrap()
        .format(x)
    };
    assert_eq!(f(".1f", 1.25), "1.3");
    assert_eq!(f(".2f", 2.675), "2.67");
    assert_eq!(f(".2g", 1.25), "1.3");
    assert_eq!(f("+.1f", -0.), "−0.0");
    assert_eq!(f(".1f", -0.), "0.0");
    assert_eq!(f("#x", 255.), "0xff");
    assert_eq!(f(".2e", 1.25), "1.25e+0");
    assert!(NumericSpecifier::parse(".f").is_err());
    assert!(NumericSpecifier::parse("99999999999f").is_err());
    let s = NumericScale::linear().with_domain([0., 1.]).unwrap();
    let original = s.clone();
    s.ticks(4., 100).unwrap();
    s.tick_format(4., None, Default::default()).unwrap();
    assert_eq!(s, original);
}
#[test]
fn expanded_pinned_specifiers_locales_and_tick_boundaries() {
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/numeric-format/cases.json"
    ))
    .unwrap();
    let locales: std::collections::BTreeMap<_, NumericLocale> = corpus["locales"]
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
    let mut errors = Vec::new();
    for lane in ["formats", "prefix"] {
        for c in corpus[lane].as_array().unwrap() {
            let format = NumericFormat {
                specifier: c["specifier"].as_str().unwrap().into(),
                locale: locales[c["locale"].as_str().unwrap()].clone(),
            };
            let f = if lane == "prefix" {
                format.prepare_prefix(number(&c["reference"]))
            } else {
                format.prepare()
            }
            .unwrap();
            let actual: Vec<_> = c["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| f.format(number(v)))
                .collect();
            if serde_json::to_value(&actual).unwrap() != c["result"]["value"] {
                errors.push(format!(
                    "{lane} {} {} ref {}: {actual:?} != {}",
                    c["locale"], c["specifier"], c["reference"], c["result"]
                ));
            }
            let wire = serde_json::to_string(&format).unwrap();
            assert_eq!(
                serde_json::from_str::<NumericFormat>(&wire).unwrap(),
                format
            );
        }
    }
    for c in corpus["parse"].as_array().unwrap() {
        assert!(c["result"].get("error").is_some());
        assert!(NumericSpecifier::parse(c["specifier"].as_str().unwrap()).is_err());
    }
    for c in corpus["text"].as_array().unwrap() {
        let f = NumericFormat {
            specifier: c["specifier"].as_str().unwrap().into(),
            locale: locales[c["locale"].as_str().unwrap()].clone(),
        }
        .prepare();
        let actual = f.and_then(|f| f.format_text(c["value"].as_str().unwrap()));
        let pass = match &actual {
            Ok(v) => c["result"]["value"].as_str() == Some(v),
            Err(_) => c["result"].get("error").is_some(),
        };
        if !pass {
            errors.push(format!("text {c}: {actual:?}"));
        }
    }
    for c in corpus["tick"].as_array().unwrap() {
        let (a, b, n) = (number(&c["start"]), number(&c["stop"]), number(&c["count"]));
        let step = tick_step(a, b, n);
        if !same(step, number(&c["step"])) {
            errors.push(format!("step {c}: {step}"));
        }
        let actual = tick_candidates(a, b, n, 10000);
        let pass = match &actual {
            Ok(a) => c["result"]["value"].as_array().is_some_and(|e| {
                a.len() == e.len() && a.iter().zip(e).all(|(a, e)| same(*a, number(e)))
            }),
            Err(_) => c["result"].get("error").is_some(),
        };
        if !pass {
            errors.push(format!("ticks {c}: {actual:?}"));
        }
    }
    for c in corpus["log"].as_array().unwrap() {
        let actual = log_tick_format(
            number(&c["domain"][0]),
            number(&c["domain"][1]),
            number(&c["base"]),
            number(&c["count"]),
            c["specifier"].as_str(),
            Default::default(),
        )
        .map(|f| {
            c["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| f.format(number(v)))
                .collect::<Vec<_>>()
        });
        let pass = match &actual {
            Ok(v) => serde_json::to_value(v).unwrap() == c["result"]["value"],
            Err(_) => c["result"].get("error").is_some(),
        };
        if !pass {
            errors.push(format!("log {c}: {actual:?}"));
        }
    }
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
#[test]
fn distribution_nice_preserves_midpoints_and_rebuilds_thresholds() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut errors = Vec::new();
    let mut checked = 0;
    for c in corpus["cases"].as_array().unwrap() {
        let name = c["factory"].as_str().unwrap();
        if !(name.starts_with("scaleSequential")
            || name.starts_with("scaleDiverging")
            || name == "scaleQuantize")
            || !c["queries"]["nice"].is_array()
        {
            continue;
        }
        let family = if name.contains("Log") {
            NumericFamily::Log {
                base: number(&c["getters"]["base"]["value"]),
            }
        } else {
            NumericFamily::Linear
        };
        let d: Vec<Number> = c["getters"]["domain"]["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Number(number(v)))
            .collect();
        for row in c["queries"]["nice"].as_array().unwrap() {
            checked += 1;
            let count = number(&row["count"]);
            let result = if name == "scaleQuantize" {
                let scale = ClassifierScale::new(ClassifierSpec {
                    domain: ClassifierDomain::Quantize(d.clone().try_into().unwrap()),
                    range: vec![0, 1, 2],
                    unknown: None,
                })
                .unwrap();
                scale.nice(count).map(|s| s.domain().to_vec())
            } else {
                let spec = if name.contains("Diverging") {
                    NormalizationSpec::Diverging {
                        family,
                        domain: d.clone().try_into().unwrap(),
                        clamp: false,
                    }
                } else {
                    NormalizationSpec::Sequential {
                        family,
                        domain: d.clone().try_into().unwrap(),
                        clamp: false,
                    }
                };
                let scale = ScaleNormalizer::new(spec).unwrap();
                let before = scale.clone();
                let result = scale.nice(count).map(|s| s.domain().to_vec());
                assert_eq!(scale.spec(), before.spec());
                result
            };
            let pass = match &result {
                Ok(a) => row["value"].as_array().is_some_and(|e| {
                    a.len() == e.len() && a.iter().zip(e).all(|(a, e)| same(a.0, number(e)))
                }),
                Err(_) => row.get("error").is_some(),
            };
            if !pass {
                errors.push(format!("{} nice {row}: {result:?}", c["id"]));
            }
        }
    }
    println!("Checked {checked} distribution nice queries.");
    assert!(checked > 400);
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
    let s = ScaleNormalizer::new(NormalizationSpec::Diverging {
        family: NumericFamily::Pow { exponent: 2. },
        domain: [Number(0.1), Number(0.37), Number(9.8)],
        clamp: false,
    })
    .unwrap();
    assert_eq!(
        s.nice(5.).unwrap().domain(),
        &[Number(0.), Number(0.37), Number(10.)]
    );
    let rank = InterpolatedScale::new(InterpolatedScaleSpec::quantile()).unwrap();
    assert!(rank.ticks(10., 100).is_err());
    assert!(rank.nice(10.).is_err());
    let q = ClassifierScale::new(ClassifierSpec {
        domain: ClassifierDomain::Quantize([Number(0.1), Number(9.8)]),
        range: vec!["a", "b"],
        unknown: None,
    })
    .unwrap();
    assert_eq!(q.nice(5.).unwrap().thresholds(), &[Some(Number(5.))]);
    assert_eq!(
        q.tick_format(5., None, Default::default())
            .unwrap()
            .format(2.),
        "2"
    );
    assert_eq!(
        NumericScale::linear().nice(f64::NAN).unwrap(),
        NumericScale::linear()
    );
}
