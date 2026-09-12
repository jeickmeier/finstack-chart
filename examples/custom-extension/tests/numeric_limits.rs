//! FIX-GG04 numeric function limits against the pinned independent R matrix.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
fn same(a: f64, b: f64) -> bool {
    a == b || a.is_nan() && b.is_nan() || (a - b).abs() <= 3e-12 * b.abs().max(1.)
}
fn temporal(case: &serde_json::Value) -> Option<(GgplotTimestampNormalization, f64)> {
    use chart_core::data::TimeUnit;
    if !matches!(case["kind"].as_str(), Some("date" | "datetime")) {
        return None;
    }
    let units = case["units"].as_i64().unwrap_or(1);
    let date = case["kind"] == "date";
    Some((
        GgplotTimestampNormalization {
            origin: 1_704_067_200 * units,
            unit: match units {
                1 => TimeUnit::Seconds,
                1000 => TimeUnit::Milliseconds,
                1000000 => TimeUnit::Microseconds,
                _ => TimeUnit::Nanoseconds,
            },
            date,
        },
        units as f64 * if date { 86400. } else { 1. },
    ))
}
fn spec(case: &serde_json::Value) -> MappedScaleSpec {
    let transform = match case["transform"].as_str().unwrap_or("identity") {
        "sqrt" => Some(ScaleTransform::Sqrt),
        "reverse" => Some(ScaleTransform::Reverse),
        _ => None,
    };
    let mut s = if case["kind"] == "identity" {
        let mut s = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
            GgplotNumericIdentity {
                transform,
                guide: true,
                ..Default::default()
            },
        ));
        s.training = ScaleTraining::Eligible;
        s
    } else {
        let mut s = ggplot_numeric_default(GgplotNumericPalette::Size).unwrap();
        if case["kind"] == "binned" {
            s = s
                .with_ggplot(GgplotScalePolicy::Binned(Box::default()))
                .unwrap();
        }
        let ScaleFunctionSpec::Interpolated(f) = &mut s.function else {
            unreachable!()
        };
        f.normalization = NormalizationSpec::Ggplot {
            family: if transform == Some(ScaleTransform::Sqrt) {
                NumericFamily::Pow { exponent: 0.5 }
            } else {
                NumericFamily::Linear
            },
            domain: [Number(0.), Number(1.)],
            reverse: transform == Some(ScaleTransform::Reverse),
            rescaler: match case["rescaler"].as_str() {
                Some("maximum") => GgplotRescaler::Maximum,
                Some("midpoint") => GgplotRescaler::Midpoint(Number(2.)),
                _ => GgplotRescaler::Range,
            },
            timestamp: None,
        };
        if case.get("rescaler").is_some() {
            f.output = ScaleRangeFunction::Identity;
        }
        s
    };
    if let Some((t, _)) = temporal(case) {
        s = s
            .with_timestamp_normalization(t)
            .unwrap()
            .with_guide(GgplotScaleGuide::Temporal(GgplotTemporalGuide {
                origin: t.origin,
                unit: t.unit,
                zone: CalendarZone::Utc,
                arguments: GgplotTemporalGuideArguments {
                    date: t.date,
                    ..Default::default()
                },
            }))
            .unwrap();
    }
    s = s
        .with_limits_function(chart_extension_example::numeric_limits::operation(
            case["control"].as_str().unwrap(),
        ))
        .unwrap();
    s
}
#[test]
fn reference_numeric_limit_functions() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/limit-functions.json"
    ))
    .unwrap();
    let cases: Vec<_> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c.get("transform").is_some())
        .cloned()
        .collect();
    assert_eq!(cases.len(), 378);
    check_cases(&cases);
}
#[test]
fn reference_numeric_limit_rescalers() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/limit-rescalers.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 126);
    check_cases(cases);
}
#[test]
fn reference_temporal_limit_functions() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-limit-functions.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 84);
    let cases: Vec<_> = [1, 1000, 1000000, 1000000000]
        .into_iter()
        .flat_map(|units| {
            fixture["cases"]
                .as_array()
                .unwrap()
                .iter()
                .cloned()
                .map(move |mut c| {
                    c["units"] = serde_json::json!(units);
                    c
                })
        })
        .collect();
    check_cases(&cases);
}
fn check_cases(cases: &[serde_json::Value]) {
    let registry = chart_extension_example::registry().unwrap();
    let mut errors = vec![];
    let mut count = 0;
    for c in cases {
        let time = temporal(c);
        let to_offset =
            |v: f64| time.map_or(v, |(t, factor)| (v - t.origin as f64 / factor) * factor);
        let absolute = |v: f64| time.map_or(v, |(t, factor)| t.origin as f64 / factor + v / factor);
        count += 1;
        let id = format!(
            "{}/{}/{}/{}/{}",
            c["kind"], c["transform"], c["population"], c["control"], c["rescaler"]
        );
        let inputs: Vec<_> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                if v.is_null() {
                    None
                } else {
                    Some(Number(to_offset(number(v))))
                }
            })
            .collect();
        let source = spec(c);
        let trained = source.trained_with_registry(&inputs, &registry);
        if let Ok(trained) = &trained {
            let round: MappedScaleSpec =
                serde_json::from_str(&serde_json::to_string(trained).unwrap()).unwrap();
            assert_eq!(&round, trained, "{id}: serde");
            let replacement = [Some(Number(2.)), Some(Number(7.)), None];
            assert_eq!(
                trained
                    .trained_with_registry(&replacement, &registry)
                    .unwrap(),
                source
                    .trained_with_registry(&replacement, &registry)
                    .unwrap(),
                "{id}: replacement"
            );
            if let Some(limits) = c["result"]["limits"].as_array() {
                let actual = trained.resolved_numeric_limits.as_ref().unwrap();
                assert_eq!(actual.len(), limits.len(), "{id}: limits length");
                for (a, e) in actual.iter().zip(limits) {
                    let transformed = match c["transform"].as_str().unwrap_or("identity") {
                        "sqrt" => a.0.sqrt(),
                        "reverse" => -a.0,
                        _ => absolute(a.0),
                    };
                    assert!(
                        same(transformed, number(e)),
                        "{id}: limit {transformed} != {}",
                        number(e)
                    );
                }
            }
        }
        let result = trained
            .and_then(|trained| MappedScale::new_with_registry(trained, &registry))
            .and_then(|m| {
                let mut values = vec![];
                // Check empty-vector mapping validity as well as each observation.
                if inputs.is_empty() && c["kind"] != "identity" {
                    m.numeric(None)?;
                }
                for v in &inputs {
                    values.push(m.numeric(v.map(|n| n.0))?);
                }
                Ok((m, values))
            });
        if c["result"].get("error").is_some() {
            if result.is_ok() {
                errors.push(format!("{id}: expected map error"));
            }
            continue;
        }
        let (mapped, values) = match result {
            Ok(x) => x,
            Err(e) => {
                errors.push(format!("{id}: unexpected {e:?}"));
                continue;
            }
        };
        let expected = c["result"]["values"].as_array().unwrap();
        for (i, v) in values.iter().enumerate() {
            let actual = match v {
                Value::Number(n) => n.0,
                Value::Missing => f64::NAN,
                _ => panic!("numeric output"),
            };
            let wanted = number(&expected[if expected.len() == 1 { 0 } else { i }]);
            if !same(actual, wanted) {
                errors.push(format!("{id}: value{i} {actual} != {wanted}"));
            }
        }
        let guide = if c["kind"] == "binned" {
            mapped.binned_guide_entries(4096, 1048576)
        } else {
            mapped.continuous_guide_entries(4096, 1048576)
        };
        if c["result"]["guide"].get("error").is_some() {
            if guide.is_ok() {
                errors.push(format!("{id}: expected guide error"));
            }
            continue;
        }
        let guide = match guide {
            Ok(Some(g)) => g,
            other => {
                errors.push(format!("{id}: unexpected guide {other:?}"));
                continue;
            }
        };
        let expected = c["result"]["guide"]["breaks"].as_array().unwrap();
        if guide.len() != expected.len() {
            errors.push(format!(
                "{id}: guide length {} != {}",
                guide.len(),
                expected.len()
            ));
            continue;
        }
        for (g, e) in guide.iter().zip(expected) {
            if !same(absolute(g.transformed.0), number(e)) {
                errors.push(format!(
                    "{id}: break {} != {}",
                    absolute(g.transformed.0),
                    number(e)
                ));
            }
        }
        let labels: Vec<_> = guide.iter().map(|g| g.label.clone()).collect();
        let expected_labels: Vec<_> = c["result"]["guide"]["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map(str::to_owned))
            .collect();
        if labels != expected_labels {
            errors.push(format!("{id}: labels {labels:?} != {expected_labels:?}"));
        }
    }
    assert_eq!(count, cases.len());
    assert!(
        errors.is_empty(),
        "{} differences:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn callback_receives_inverse_transformed_training_and_rejects_wrong_key_kinds() {
    use chart_core::{ChartResult, grammar::*};
    use std::sync::{Arc, Mutex};
    type Captured = (Option<Vec<ScaleKey>>, Option<GgplotTimestampNormalization>);
    #[derive(Clone)]
    struct Recorded(Arc<Mutex<Vec<Captured>>>);
    impl CustomScaleLimits for Recorded {
        fn descriptor(&self) -> ExtensionDescriptor {
            chart_extension_example::numeric_limits::Limits.descriptor()
        }
        fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
            chart_extension_example::numeric_limits::Limits.validate(p)
        }
        fn evaluate(&self, input: ScaleLimitsInput<'_>) -> ChartResult<Option<Vec<ScaleKey>>> {
            self.0
                .lock()
                .unwrap()
                .push((input.domain.map(<[ScaleKey]>::to_vec), input.temporal));
            chart_extension_example::numeric_limits::Limits.evaluate(input)
        }
    }
    let seen = Arc::new(Mutex::new(vec![]));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_limits(Arc::new(Recorded(seen.clone())))
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/limit-functions.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c.get("transform").is_some())
    {
        seen.lock().unwrap().clear();
        let inputs: Vec<_> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (!v.is_null()).then(|| Number(number(v))))
            .collect();
        let _ = spec(c).trained_with_registry(&inputs, &registry);
        let calls = seen.lock().unwrap();
        let expected = c["seen"].as_array().unwrap();
        if expected.is_empty() {
            assert!(calls.is_empty(), "{c}");
            continue;
        }
        assert!(calls[0].1.is_none());
        let actual = calls[0].0.as_deref().unwrap_or_default();
        let expected = expected[0].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{c}");
        for (a, b) in actual.iter().zip(expected) {
            let ScaleKey::Number(a) = a else {
                panic!("numeric trained endpoint")
            };
            assert!(same(a.0, number(b)), "{c}: {} != {}", a.0, number(b));
        }
        count += 1;
    }
    assert_eq!(count, 357); // Reverse inverse(NULL) fails before the callback.
    // Primary position scales use this same first-stage callback domain. Later
    // stages are deliberately not inferred from aesthetic-scale qualification.
    let positional_fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/positional-limit-functions.json"
    ))
    .unwrap();
    let mut positional_calls = 0;
    for c in positional_fixture["cases"].as_array().unwrap() {
        seen.lock().unwrap().clear();
        let inputs: Vec<_> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (!v.is_null()).then(|| Number(number(v))))
            .collect();
        let result = spec(c).trained_with_registry(&inputs, &registry);
        let calls = seen.lock().unwrap();
        let expected = c["seen"].as_array().unwrap();
        if expected.is_empty() {
            assert!(calls.is_empty(), "{c}");
            assert!(result.is_err(), "{c}");
            continue;
        }
        assert_eq!(calls.len(), 1, "{c}");
        assert!(calls[0].1.is_none());
        let actual = calls[0].0.as_deref().unwrap_or_default();
        let expected = expected[0].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{c}");
        for (a, b) in actual.iter().zip(expected) {
            let ScaleKey::Number(a) = a else {
                panic!("numeric trained endpoint")
            };
            assert!(same(a.0, number(b)), "{c}: {} != {}", a.0, number(b));
        }
        positional_calls += 1;
    }
    assert_eq!(positional_calls, 238);
    let temporal_fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-limit-functions.json"
    ))
    .unwrap();
    let mut temporal_calls = 0;
    for original in temporal_fixture["cases"].as_array().unwrap() {
        let mut c = original.clone();
        c["units"] = serde_json::json!(1_000_000_000);
        let (t, factor) = temporal(&c).unwrap();
        seen.lock().unwrap().clear();
        let inputs: Vec<_> = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                (!v.is_null()).then(|| Number((number(v) - t.origin as f64 / factor) * factor))
            })
            .collect();
        let _ = spec(&c).trained_with_registry(&inputs, &registry);
        let calls = seen.lock().unwrap();
        let expected = c["seen"].as_array().unwrap();
        if expected.is_empty() {
            assert!(calls.is_empty(), "{c}");
            continue;
        }
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, Some(t));
        let class = expected[0]["class"].as_array().unwrap();
        assert_eq!(class[0], if t.date { "Date" } else { "POSIXct" });
        let values = expected[0]["values"].as_array().unwrap();
        let domain = calls[0].0.as_ref().unwrap();
        assert_eq!(domain.len(), values.len());
        for (a, b) in domain.iter().zip(values) {
            let ScaleKey::Number(a) = a else {
                panic!("temporal offset")
            };
            assert!(
                same(t.origin as f64 / factor + a.0 / factor, number(b)),
                "{c}"
            );
        }
        temporal_calls += 1;
    }
    assert_eq!(temporal_calls, 70);
    let c = &fixture["cases"][0];
    let registry = chart_extension_example::registry().unwrap();
    let s = spec(c)
        .with_limits_function(chart_extension_example::discrete_limits::operation("fixed"))
        .unwrap();
    assert_eq!(
        s.trained_with_registry(&[Some(Number(1.))], &registry)
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::SchemaConflict
    );
}

#[test]
fn primary_reference_nan_output_is_missing_and_legacy_rejects_it() {
    use chart_core::prelude::*;
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("v", [1., 3., 9.])
        .build()
        .unwrap();
    let descriptor = spec(
        &serde_json::json!({"kind":"continuous","transform":"identity","rescaler":"maximum","control":"empty"}),
    );
    let registry = chart_extension_example::registry().unwrap();
    let build = |profile| {
        plot(data.clone())
            .extensions(registry.clone())
            .profile(profile)
            .aes(aes().x("x").y(1.))
            .layer(points().numeric_scale(NumericAesthetic::Size, "v", descriptor.clone()))
            .build()
            .unwrap()
    };
    let reference = build(Profile::Ggplot2_4_0_3);
    let wire = reference.to_json().unwrap();
    let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
    assert!(
        restored.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .is_empty()
    );
    assert_eq!(
        build(Profile::LibraryV1)
            .chart()
            .unwrap()
            .prepare()
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::NumericalDomain
    );
}
