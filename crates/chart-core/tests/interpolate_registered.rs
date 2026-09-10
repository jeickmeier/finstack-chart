//! ITP-07: explicit registration identity, preparation, budgets and immutable consumers.
use chart_core::{ChartResult, DiagnosticCode, Revision, grammar::*, interpolate::*, scales::*};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct Factory {
    calls: Arc<AtomicUsize>,
    portable: bool,
    huge: bool,
}
impl CustomInterpolationFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            "test.interpolation",
            Revision::new(9007199254740993),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        if p == &serde_json::json!({}) {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                DiagnosticCode::Validation,
                "Unexpected parameter.",
                "Use an empty object.",
            ))
        }
    }
    fn compile(
        &self,
        input: InterpolationInput<'_>,
    ) -> ChartResult<Arc<dyn Sample<Value> + Send + Sync>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (Value::Number(a), Value::Number(b)) = (input.source, input.target) else {
            panic!("test endpoints")
        };
        Ok(Arc::new(Sampler {
            kernel: ScalarInterpolator::number(a.0, b.0),
            huge: self.huge,
        }))
    }
}
struct Sampler {
    kernel: ScalarInterpolator,
    huge: bool,
}
impl Sample<Value> for Sampler {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        if self.huge {
            Ok(Value::Array(vec![Value::Null; 200_001]))
        } else {
            self.kernel.sample(t * t).map(Value::number)
        }
    }
}
fn setup(
    portable: bool,
    huge: bool,
) -> (ExtensionRegistry, InterpolationFactory, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_interpolation(Arc::new(Factory {
            calls: calls.clone(),
            portable,
            huge,
        }))
        .unwrap();
    let factory = registry
        .interpolation_factory(
            OperationRef::new("test.interpolation", Revision::new(9007199254740993)),
            serde_json::json!({}),
        )
        .unwrap();
    (registry, factory, calls)
}
#[test]
fn factory_prepares_once_and_retains_owned_samples_and_exact_identity() {
    let (registry, factory, calls) = setup(true, false);
    let f = factory
        .between_with_registry(Value::number(0.), Value::number(100.), &registry)
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let held = f.sample(0.5).unwrap();
    for t in [-1., 0., 0.25, 0.5, 1., 2.] {
        assert_eq!(f.sample(t).unwrap(), Value::number(100. * t * t));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let wire = f.to_json().unwrap();
    assert!(wire.contains("9007199254740993"));
    assert!(wire.contains("\"version\":2"));
    assert_eq!(
        Interpolator::from_json(&wire).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    let restored = Interpolator::from_json_with_registry(&wire, &registry).unwrap();
    assert_eq!(restored.sample(0.5).unwrap(), held);
    assert!(
        Interpolator::from_json_with_registry(
            &wire.replacen("\"version\":2", "\"version\":1", 1),
            &registry
        )
        .is_err()
    );
    drop(registry);
    drop(f);
    assert_eq!(restored.sample(0.5).unwrap(), held);
    assert!(restored.sample(f64::NAN).is_err());
}
#[test]
fn registry_conflicts_native_policy_and_budgets_are_checked() {
    let (mut registry, factory, calls) = setup(false, false);
    assert!(
        registry
            .register_interpolation(Arc::new(Factory {
                calls: calls.clone(),
                portable: true,
                huge: false
            }))
            .is_err()
    );
    let f = factory
        .between_with_registry(Value::number(0.), Value::number(1.), &registry)
        .unwrap();
    assert!(f.to_json().is_err());
    assert!(
        Interpolator::from_json_with_registry(&f.descriptor_json().unwrap(), &registry).is_err()
    );
    let mut invalid = factory.clone();
    invalid.gamma = Some(Number(2.));
    assert!(
        invalid
            .between_with_registry(Value::number(0.), Value::number(1.), &registry)
            .is_err()
    );
    let mut invalid = factory.clone();
    invalid.registration.as_mut().unwrap().parameters = serde_json::json!({"wrong":true});
    assert!(
        invalid
            .between_with_registry(Value::number(0.), Value::number(1.), &registry)
            .is_err()
    );
    let before = calls.load(Ordering::SeqCst);
    assert!(
        factory
            .between_with_registry(
                Value::Text("x".repeat(MAX_VALUE_BYTES + 1)),
                Value::number(0.),
                &registry
            )
            .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), before);
    let (registry, factory, _) = setup(true, true);
    let f = factory
        .between_with_registry(Value::number(0.), Value::number(1.), &registry)
        .unwrap();
    assert_eq!(
        f.sample(0.5).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
}
#[test]
fn continuous_time_and_piecewise_scales_keep_snapshot_on_changes() {
    let (registry, factory, _) = setup(true, false);
    let options = ScaleOptions {
        domain: Some(vec![
            ScaleInput::Number(Number(0.)),
            ScaleInput::Number(Number(10.)),
        ]),
        range: Some(vec![Value::number(0.), Value::number(100.)]),
        factory: Some(factory.clone()),
        ..Default::default()
    };
    let scale = ScaleConstructor::Linear
        .create_with_registry(options, &registry)
        .unwrap();
    assert_eq!(
        scale.map(ScaleInput::Number(Number(5.))).unwrap(),
        Value::number(25.)
    );
    let wire = scale.to_json().unwrap();
    assert!(StandaloneScale::from_json(&wire).is_err());
    let restored = StandaloneScale::from_json_with_registry(&wire, &registry).unwrap();
    let nice = restored.nice(5.).unwrap();
    let time = ScaleConstructor::Utc
        .create_with_registry(
            ScaleOptions {
                domain: Some(vec![
                    ScaleInput::Time(9007199254740993),
                    ScaleInput::Time(9007199254741993),
                ]),
                unit: Some(chart_core::data::TimeUnit::Nanoseconds),
                range: Some(vec![Value::number(0.), Value::number(100.)]),
                factory: Some(factory.clone()),
                ..Default::default()
            },
            &registry,
        )
        .unwrap();
    let f = Interpolator::new_with_registry(
        InterpolationSpec::Piecewise {
            factory,
            values: vec![Value::number(0.), Value::number(100.), Value::number(200.)],
        },
        &registry,
    )
    .unwrap();
    drop(registry);
    assert_eq!(
        nice.map(ScaleInput::Number(Number(5.))).unwrap(),
        Value::number(25.)
    );
    assert_eq!(
        time.map(ScaleInput::Time(9007199254741493)).unwrap(),
        Value::number(25.)
    );
    assert_eq!(
        f.quantize(5).unwrap(),
        [0., 25., 100., 125., 200.].map(Value::number)
    );
}

#[test]
fn registered_normalized_ranges_share_unknown_clamp_and_composition() {
    let (registry, factory, _) = setup(true, false);
    let spec = factory
        .between_with_registry(Value::number(0.), Value::number(100.), &registry)
        .unwrap()
        .spec()
        .clone();
    for (family, domain, input, expected) in [
        (ScaleConstructor::Sequential, vec![0., 10.], 5., 25.),
        (
            ScaleConstructor::Diverging,
            vec![-10., 0., 100.],
            50.,
            56.25,
        ),
        (
            ScaleConstructor::SequentialQuantile,
            vec![0., 1., 2., 3., 4.],
            2.,
            25.,
        ),
    ] {
        let scale = family
            .create_with_registry(
                ScaleOptions {
                    domain: Some(
                        domain
                            .into_iter()
                            .map(|n| ScaleInput::Number(Number(n)))
                            .collect(),
                    ),
                    interpolator: Some(spec.clone()),
                    unknown: (family != ScaleConstructor::SequentialQuantile)
                        .then_some(Value::Null),
                    clamp: (family != ScaleConstructor::SequentialQuantile).then_some(true),
                    ..Default::default()
                },
                &registry,
            )
            .unwrap();
        assert_eq!(
            scale.map(ScaleInput::Number(Number(input))).unwrap(),
            Value::number(expected)
        );
        assert_eq!(
            scale.map(ScaleInput::Missing).unwrap(),
            if family == ScaleConstructor::SequentialQuantile {
                Value::Missing
            } else {
                Value::Null
            }
        );
        assert_eq!(
            scale.map(ScaleInput::Number(Number(200.))).unwrap(),
            Value::number(100.)
        );
        let copy =
            StandaloneScale::from_json_with_registry(&scale.to_json().unwrap(), &registry).unwrap();
        assert_eq!(
            copy.map(ScaleInput::Number(Number(input))).unwrap(),
            Value::number(expected)
        );
    }
}

#[test]
fn chart_prepares_factories_independently_of_mark_count() {
    use chart_core::{prelude::*, state::ChartState};
    fn prepare(rows: usize) -> usize {
        let (registry, factory, calls) = setup(true, false);
        let scale = ScaleConstructor::Linear
            .create_with_registry(
                ScaleOptions {
                    factory: Some(factory),
                    range: Some([2., 10.].map(Value::number).to_vec()),
                    ..Default::default()
                },
                &registry,
            )
            .unwrap();
        let plot = plot(
            Data::columns()
                .column(
                    "x",
                    (0..rows)
                        .map(|i| i as f64 / (rows - 1) as f64)
                        .collect::<Vec<_>>(),
                )
                .build()
                .unwrap(),
        )
        .extensions(Arc::new(registry))
        .aes(aes().x("x").y(1.))
        .layer(points().numeric_scale(
            NumericAesthetic::Size,
            "x",
            scale.mapped(ScaleTraining::Authored).unwrap(),
        ))
        .build()
        .unwrap();
        calls.store(0, Ordering::SeqCst);
        let mut compiler = Compiler::with_extensions(plot.extensions().clone());
        let prepared = compiler
            .prepare(
                plot.definition(),
                &plot.source(),
                &ChartState::default(),
                plot.compile_limits(),
            )
            .unwrap();
        assert_eq!(prepared.layers()[0].table().rows().len(), rows);
        calls.load(Ordering::SeqCst)
    }
    let few = prepare(5);
    let many = prepare(500);
    assert!(few > 0);
    assert_eq!(
        few, many,
        "factory compilation must not scale with mark count"
    );
}
