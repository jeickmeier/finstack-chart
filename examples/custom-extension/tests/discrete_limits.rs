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

#[test]
fn primary_identity_callbacks_preserve_demand_names_and_null_domains() {
    use chart_core::{ChartResult, prelude::*};
    use std::sync::{Arc, Mutex};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-identity-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 192);
    for c in cases {
        let limits = Arc::new(Mutex::new(vec![]));
        let breaks = Arc::new(Mutex::new(vec![]));
        let mut extensions = chart_extension_example::registry().unwrap();
        let registry = Arc::get_mut(&mut extensions).unwrap();
        registry
            .register_scale_limits(Arc::new(IdentityLimits(limits.clone())))
            .unwrap();
        registry
            .register_scale_breaks(Arc::new(IdentityBreaks(breaks.clone())))
            .unwrap();
        let source = || {
            let mut s = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotDiscreteIdentity(
                GgplotDiscreteIdentity {
                    guide: c["guide"] == "legend",
                    levels: c["levels"].as_array().map(|v| v.iter().map(key).collect()),
                    drop: false,
                    ..Default::default()
                },
            ));
            s.training = ScaleTraining::Eligible;
            if c["limit_mode"] != "none" {
                let mut call = operation(c["limit_mode"].as_str().unwrap());
                call.operation.id = "test.identity_limits".into();
                s.limits_function = Some(Box::new(call));
            }
            if c["break_mode"] != "auto" {
                s.breaks_function = Some(Box::new(ScaleBreaksOperation {
                    operation: OperationRef {
                        id: "test.identity_breaks".into(),
                        version: chart_core::Revision::new(1),
                    },
                    parameters: c["break_mode"].clone(),
                }));
            }
            s
        };
        let result = (|| -> ChartResult<_> {
            let inputs = c["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    inputs
                        .iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Vec<_>>(),
                )
                .build()?;
            let mapping = if c["aesthetic"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("identity")
            } else {
                aes().x("x").y(1.).color("v").color_scale("identity")
            };
            let layer = || {
                points()
                    .name("marks")
                    .aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.)))
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(extensions.clone())
                .aes(mapping)
                .scale(color_mapped("identity", source()))
                .layer(layer())
                .build()?;
            let wire = p.to_json()?;
            let restored = Plot::from_json_with_extensions(&wire, extensions.clone())?;
            assert_eq!(restored.to_json()?, wire);
            let mut prepared = vec![restored.chart()?.prepare()?];
            prepared.push(
                restored
                    .edit()
                    .layer("marks", layer())
                    .build()?
                    .chart()?
                    .prepare()?,
            );
            prepared.push(restored.edit().theme(theme()).build()?.chart()?.prepare()?);
            Ok(prepared)
        })();
        assert_eq!(
            result.is_ok(),
            c["result"].get("error").is_none(),
            "{c}: {:?}",
            result.as_ref().err()
        );
        for (actual, expected) in [(&limits, &c["limit_calls"]), (&breaks, &c["break_calls"])] {
            let actual = actual.lock().unwrap();
            let expected = expected.as_array().unwrap();
            assert_eq!(actual.is_empty(), expected.is_empty(), "{c}: {actual:?}");
            for (domain, null) in actual.iter() {
                assert!(
                    expected
                        .iter()
                        .any(|call| call["is_null"] == *null && keys(&call["values"]) == *domain),
                    "{c}: unexpected callback {domain:?}, null={null}"
                );
            }
        }
        let Ok(prepared) = result else { continue };
        for prepared in prepared {
            let layer = &prepared.layers()[0];
            let expected = c["result"]["keys"].as_array().unwrap();
            let legend = if c["aesthetic"] == "fill" {
                layer.paint_legends().get(&PaintAesthetic::Fill)
            } else {
                layer.color_legend()
            };
            let entries = legend.map_or(&[][..], |g| g.entries.as_slice());
            if expected.is_empty() {
                assert!(entries.is_empty(), "{c}: {entries:?}");
            } else {
                let wanted = expected[0]["mapped"].as_array().unwrap();
                assert_eq!(entries.len(), wanted.len(), "{c}");
                for ((label, paint), (value, text)) in entries
                    .iter()
                    .zip(wanted.iter().zip(expected[0]["labels"].as_array().unwrap()))
                {
                    assert_eq!(label, text.as_str().unwrap_or("NA"), "{c}");
                    assert_identity_paint(*paint, value, &fixture["paints"]);
                }
            }
            let actual = layer
                .marks()
                .iter()
                .map(|m| {
                    if c["aesthetic"] == "fill" {
                        m.style.fill.unwrap()
                    } else {
                        m.style.color
                    }
                })
                .collect::<Vec<_>>();
            let expected = c["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| c["aesthetic"] == "fill" || !v.is_null())
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), expected.len(), "{c}");
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert_identity_paint(actual, expected, &fixture["paints"]);
            }
            if let Some(spec) = legend.and_then(|g| g.mapping.as_ref()) {
                let round: MappedScaleSpec =
                    serde_json::from_str(&serde_json::to_string(spec).unwrap()).unwrap();
                assert_eq!(&round, spec);
                let mapped = MappedScale::new_with_registry(round, &extensions).unwrap();
                assert_eq!(
                    mapped.discrete_guide_entries().unwrap().unwrap().len(),
                    entries.len(),
                    "{c}"
                );
            }
        }
    }
}
fn assert_identity_paint(
    actual: chart_core::scene::Color,
    value: &serde_json::Value,
    paints: &serde_json::Value,
) {
    if let Some(text) = value.as_str() {
        let expected: chart_core::scene::Color =
            serde_json::from_value(paints[text].clone()).unwrap();
        assert_eq!(actual, expected);
    } else {
        // Source NA carries no RGB components; its painted output is invisible.
        assert_eq!(actual.alpha, 0);
    }
}
type IdentityCalls = std::sync::Arc<std::sync::Mutex<Vec<(Vec<ScaleKey>, bool)>>>;
struct IdentityLimits(IdentityCalls);
impl CustomScaleLimits for IdentityLimits {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.identity_limits", chart_core::Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> chart_core::ChartResult<()> {
        chart_extension_example::discrete_limits::Limits { portable: true }.validate(p)
    }
    fn evaluate(
        &self,
        input: ScaleLimitsInput<'_>,
    ) -> chart_core::ChartResult<Option<Vec<ScaleKey>>> {
        self.0.lock().unwrap().push((
            input.domain.unwrap_or_default().to_vec(),
            input.domain.is_none(),
        ));
        chart_extension_example::discrete_limits::Limits { portable: true }.evaluate(input)
    }
}
struct IdentityBreaks(IdentityCalls);
impl CustomScaleBreaks for IdentityBreaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.identity_breaks", chart_core::Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> chart_core::ChartResult<()> {
        chart_extension_example::scale_breaks::DiscreteBreaks(false).validate(p)
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> chart_core::ChartResult<ScaleBreaksOutput> {
        self.0
            .lock()
            .unwrap()
            .push((input.domain.to_vec(), input.domain_is_null));
        chart_extension_example::scale_breaks::DiscreteBreaks(false).evaluate(input)
    }
}

#[test]
fn retained_null_identity_limits_require_their_wire_capability() {
    use chart_core::prelude::*;
    let registry = chart_extension_example::registry().unwrap();
    let source = spec(true, "null")
        .trained_keys_with_registry(
            &[ScaleKey::Text("red".into()), ScaleKey::Text("blue".into())],
            &registry,
        )
        .unwrap();
    assert!(source.resolved_discrete_limits_null);
    let p = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("v", ["red", "blue"])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .extensions(registry.clone())
    .aes(aes().x("x").y(1.).color("v").color_scale("identity"))
    .scale(color_mapped("identity", source.clone()))
    .layer(points())
    .build()
    .unwrap();
    let wire = p.to_json().unwrap();
    let mut encoded: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(encoded["version"], 63);
    assert_eq!(
        Plot::from_json_with_extensions(&wire, registry.clone())
            .unwrap()
            .to_json()
            .unwrap(),
        wire
    );
    encoded["version"] = 62.into();
    assert!(Plot::from_json_with_extensions(&encoded.to_string(), registry.clone()).is_err());
    assert!(Plot::from_json(&wire).is_err());
    let mut invalid = source.clone();
    invalid.limits_function = None;
    assert!(MappedScale::new_with_registry(invalid, &registry).is_err());
    let mut replacement = source;
    replacement.limits_function = Some(Box::new(operation("empty")));
    let replacement = replacement
        .trained_keys_with_registry(&[], &registry)
        .unwrap();
    assert!(!replacement.resolved_discrete_limits_null);
    assert_eq!(domain(&replacement), vec![]);
}
