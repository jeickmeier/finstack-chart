//! External implementation of FIX-GG04 discrete function limits, pinned to R.
use chart_core::{
    grammar::*,
    interpolate::{Number, Value},
    scales::*,
};
use chart_extension_example::discrete_limits::operation;
fn key(v: &serde_json::Value) -> ScaleKey {
    if let Some(s) = v.as_str() {
        ScaleKey::Text(s.into())
    } else {
        v.as_f64()
            .map_or(ScaleKey::Null, |n| ScaleKey::Number(Number(n)))
    }
}
fn keys(v: &serde_json::Value) -> Vec<ScaleKey> {
    v.as_array().unwrap().iter().map(key).collect()
}
fn spec(identity: bool, mode: &str) -> MappedScaleSpec {
    let mut spec = if identity {
        let mut s = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotDiscreteIdentity(
            GgplotDiscreteIdentity {
                guide: true,
                ..Default::default()
            },
        ));
        s.training = ScaleTraining::Eligible;
        s
    } else {
        MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
            .with_ggplot(GgplotScalePolicy::Discrete {
                empty_population: false,
                limits: None,
                levels: None,
                drop: true,
                na_translate: true,
                palette: GgplotDiscretePalette::Shape { solid: true },
            })
            .unwrap()
    };
    spec = spec.with_limits_function(operation(mode)).unwrap();
    spec
}
fn domain(s: &MappedScaleSpec) -> Vec<ScaleKey> {
    match &s.function {
        ScaleFunctionSpec::Ordinal(s) => s.domain.clone(),
        ScaleFunctionSpec::GgplotDiscreteIdentity(s) => s.domain().unwrap(),
        _ => unreachable!(),
    }
}
#[test]
fn ninety_reference_function_limits_mapping_and_guides() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/limit-functions.json"
    ))
    .unwrap();
    let registry = chart_extension_example::registry().unwrap();
    let cases: Vec<_> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c.get("transform").is_none())
        .collect();
    assert_eq!(cases.len(), 90);
    for case in cases {
        let mut source = spec(
            case["kind"] == "identity",
            case["control"].as_str().unwrap(),
        );
        if let ScaleFunctionSpec::GgplotDiscreteIdentity(identity) = &mut source.function {
            identity.guide = case["guide"] == true;
        }
        let inputs = keys(&case["inputs"]);
        let result = source.trained_keys_with_registry(&inputs, &registry);
        if case["result"].get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let trained = result.unwrap();
        let actual_domain = domain(&trained);
        assert_eq!(actual_domain, keys(&case["result"]["limits"]), "{case}");
        let guide = GgplotDiscreteGuide::default()
            .resolve(&actual_domain)
            .unwrap();
        let labels: Vec<_> = guide.iter().map(|g| g.label.clone()).collect();
        let expected: Vec<_> = case["result"]["guide"]["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map(String::from))
            .collect();
        assert_eq!(labels, expected, "{case}");
        let round: MappedScaleSpec =
            serde_json::from_str(&serde_json::to_string(&trained).unwrap()).unwrap();
        let mapped = MappedScale::new_with_registry(round, &registry).unwrap();
        for (input, expected) in inputs
            .iter()
            .zip(case["result"]["values"].as_array().unwrap())
        {
            let expected = if expected.is_null() {
                Value::Missing
            } else {
                GgplotDiscreteIdentity::map(Some(&key(expected))).unwrap()
            };
            assert_eq!(mapped.category(Some(input)).unwrap(), expected, "{case}");
        }
        // Replacing data recomputes from the population, without accumulating old function results.
        let fresh = vec![ScaleKey::Text("z".into())];
        assert_eq!(
            trained
                .trained_keys_with_registry(&fresh, &registry)
                .unwrap(),
            source
                .trained_keys_with_registry(&fresh, &registry)
                .unwrap()
        );
    }
}
#[test]
fn missing_registration_version_and_parameters_are_rejected() {
    let registry = chart_extension_example::registry().unwrap();
    let mut source = spec(false, "reverse");
    assert!(source.trained_keys(&[]).is_err());
    source.limits_function.as_mut().unwrap().operation.version = chart_core::Revision::new(2);
    assert!(source.trained_keys_with_registry(&[], &registry).is_err());
    source.limits_function.as_mut().unwrap().operation.version = chart_core::Revision::new(1);
    source.limits_function.as_mut().unwrap().parameters = serde_json::json!({"mode":"bogus"});
    assert!(MappedScale::new_with_registry(source, &registry).is_err());
    let mut copied = registry.as_ref().clone();
    assert!(
        copied
            .register_scale_limits(std::sync::Arc::new(
                chart_extension_example::discrete_limits::Limits { portable: true }
            ))
            .is_err()
    );
    assert!(
        registry
            .scale_limits_descriptor(&operation("identity").operation)
            .unwrap()
            .portable
    );
    assert!(
        ExtensionRegistry::new()
            .scale_limits_descriptor(&operation("identity").operation)
            .is_err()
    );
}

#[test]
fn primary_wire_preserves_owned_registration_and_rejects_native_portability() {
    use chart_core::prelude::*;
    let registry = chart_extension_example::registry().unwrap();
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("v", ["b", "a", "c"])
        .build()
        .unwrap();
    let build = |s| {
        plot(data.clone())
            .extensions(registry.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(points().value_scale(ValueAesthetic::Shape, "v", s))
            .build()
    };
    let p = build(spec(false, "reverse")).unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 28);
    assert!(Plot::from_json(&wire).is_err());
    let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    assert_eq!(
        restored.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .len(),
        3
    );
    let mut native = spec(false, "reverse");
    native.limits_function.as_mut().unwrap().operation.id = "example.native_discrete_limits".into();
    let p = build(native).unwrap();
    assert!(p.chart().unwrap().prepare().is_ok());
    assert!(p.to_json().is_err());
    drop(registry);
    assert_eq!(restored.to_json().unwrap(), wire);
}

struct Inspect {
    expected: Option<Vec<ScaleKey>>,
    output: Vec<ScaleKey>,
}
impl CustomScaleLimits for Inspect {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.inspect", chart_core::Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> chart_core::ChartResult<()> {
        Ok(())
    }
    fn evaluate(
        &self,
        input: ScaleLimitsInput<'_>,
    ) -> chart_core::ChartResult<Option<Vec<ScaleKey>>> {
        assert_eq!(input.domain, self.expected.as_deref());
        Ok(Some(self.output.clone()))
    }
}
#[test]
fn factor_na_input_and_output_resource_boundaries() {
    let k = |s: &str| ScaleKey::Text(s.into());
    for drop in [false, true] {
        for translate in [false, true] {
            let mut expected = if drop {
                vec![k("b")]
            } else {
                vec![k("c"), k("b"), k("a")]
            };
            if translate {
                expected.push(ScaleKey::Null);
            }
            let mut registry = ExtensionRegistry::new();
            registry
                .register_scale_limits(std::sync::Arc::new(Inspect {
                    expected: Some(expected.clone()),
                    output: expected.iter().rev().cloned().collect(),
                }))
                .unwrap();
            let mut s = spec(false, "identity");
            s.limits_function.as_mut().unwrap().operation.id = "test.inspect".into();
            let GgplotScalePolicy::Discrete {
                levels,
                drop: d,
                na_translate,
                limits,
                ..
            } = s.ggplot.as_deref_mut().unwrap()
            else {
                unreachable!()
            };
            *levels = Some(vec![k("c"), k("b"), k("a")]);
            *d = drop;
            *na_translate = translate;
            *limits = Some(vec![k("ignored")]);
            let trained = s
                .trained_keys_with_registry(&[k("b"), ScaleKey::Null], &registry)
                .unwrap();
            assert_eq!(
                domain(&trained),
                expected.into_iter().rev().collect::<Vec<_>>()
            );
        }
    }
    for output in [
        vec![ScaleKey::Unsigned(u64::MAX)],
        vec![ScaleKey::Null; chart_core::interpolate::MAX_VALUES + 1],
        vec![k(&"x".repeat(chart_core::interpolate::MAX_VALUE_BYTES + 1))],
    ] {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_scale_limits(std::sync::Arc::new(Inspect {
                expected: None,
                output,
            }))
            .unwrap();
        let mut s = spec(false, "identity");
        s.limits_function.as_mut().unwrap().operation.id = "test.inspect".into();
        assert!(s.trained_keys_with_registry(&[], &registry).is_err());
    }
}
