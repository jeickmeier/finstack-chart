//! FIX-GG04 exact population arithmetic, distinct from chart-stage qualification.
use chart_core::{
    Revision,
    grammar::{OperationRef, TransformOperation, TransformSelection},
    scales::GgplotTransform,
};
use serde_json::Value;
fn number(v: &Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| match v["number"].as_str().unwrap() {
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            _ => f64::NAN,
        })
}
fn values(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(number).collect()
}
fn equal(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
        assert!(
            (a.is_nan() && b.is_nan()) || a == b,
            "{actual:?} != {expected:?}"
        );
    }
}
fn transform(family: &str) -> GgplotTransform {
    GgplotTransform::Registered {
        selection: Box::new(TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "example.scale_transform_vector".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!({"family": family}),
        })),
    }
}
#[test]
fn reference_batches_preserve_cardinality_nonfinite_and_empty_inputs() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-transforms.json"
    ))
    .unwrap();
    let registry = chart_extension_example::registry().unwrap();
    let cases = fixture["batches"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let t = registry
            .resolve_transform(&transform(case["family"].as_str().unwrap()))
            .unwrap();
        assert!(!t.is_pointwise());
        let input = values(&case["input"]);
        equal(
            &t.forward_population(&input).unwrap(),
            &values(&case["forward"]),
        );
        equal(
            &t.inverse_population(&input).unwrap(),
            &values(&case["inverse"]),
        );
    }
    let t = registry
        .resolve_transform(&transform("cardinality"))
        .unwrap();
    equal(
        &t.forward_population(&[1., 2., 4., 8., 16.]).unwrap(),
        &[6., 7., 9., 13., 21.],
    );
    assert_eq!(t.forward(1.), 2.);
}
#[test]
fn reference_composition_domain_and_batch_order_are_retained_after_json() {
    let registry = chart_extension_example::registry().unwrap();
    let selection = GgplotTransform::Compose {
        transforms: vec![transform("cardinality"), GgplotTransform::Reverse],
    };
    let decoded = serde_json::from_str(&serde_json::to_string(&selection).unwrap()).unwrap();
    let t = registry.resolve_transform(&decoded).unwrap();
    equal(&t.domain(), &[f64::NEG_INFINITY, f64::INFINITY]);
    equal(
        &t.forward_population(&[1., 2., 4.]).unwrap(),
        &[-4., -5., -7.],
    );
    equal(
        &t.inverse_population(&[-4., -5., -7.]).unwrap(),
        &[1., 2., 4.],
    );
    assert!(!t.is_pointwise());
    let invalid = GgplotTransform::Compose {
        transforms: vec![transform("center"), GgplotTransform::Reverse],
    };
    assert!(registry.resolve_transform(&invalid).is_err());
}

#[test]
fn invalid_batch_lengths_and_kernel_errors_are_not_silently_mapped() {
    use chart_core::{ChartResult, DiagnosticCode, grammar::*, interpolate::Number};
    use std::sync::Arc;
    struct Kernel(bool);
    impl PreparedTransform for Kernel {
        fn forward(&self, value: f64) -> f64 {
            value
        }
        fn inverse(&self, value: f64) -> f64 {
            value
        }
        fn is_pointwise(&self) -> bool {
            false
        }
        fn domain(&self) -> [Number; 2] {
            [Number(f64::NEG_INFINITY), Number(f64::INFINITY)]
        }
        fn monotone_on(&self, _: [f64; 2]) -> bool {
            true
        }
        fn forward_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
            if self.0 {
                return Err(chart_core::Diagnostic::error(
                    DiagnosticCode::Validation,
                    "Rejected batch.",
                    "Use an accepted batch.",
                ));
            }
            let mut result = values.to_vec();
            result.push(0.);
            Ok(result)
        }
        fn inverse_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
            self.forward_batch(values)
        }
    }
    struct Factory;
    impl CustomTransformFactory for Factory {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("test.invalid_batch", Revision::new(1), true)
        }
        fn validate(&self, _: &Value) -> ChartResult<()> {
            Ok(())
        }
        fn compile(&self, p: &Value) -> ChartResult<Arc<dyn PreparedTransform>> {
            Ok(Arc::new(Kernel(p.as_bool().unwrap())))
        }
    }
    let mut registry = ExtensionRegistry::new();
    registry.register_transform(Arc::new(Factory)).unwrap();
    for fail in [false, true] {
        let t = registry
            .resolve_transform(&GgplotTransform::Registered {
                selection: Box::new(TransformSelection::new(TransformOperation {
                    operation: OperationRef::new("test.invalid_batch", Revision::new(1)),
                    parameters: Value::Bool(fail),
                })),
            })
            .unwrap();
        assert!(!t.is_pointwise());
        for input in [vec![], vec![1., f64::NAN, f64::INFINITY]] {
            assert!(t.forward_population(&input).is_err());
            assert!(t.inverse_population(&input).is_err());
        }
    }
}
