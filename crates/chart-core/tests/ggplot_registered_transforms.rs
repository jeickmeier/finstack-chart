//! FIX-GG04 registered pointwise ownership and transport contract.
use chart_core::{
    ChartResult, Revision,
    grammar::{
        CustomTransformFactory, ExtensionDescriptor, ExtensionRegistry, OperationRef,
        PointwiseTransform, TransformOperation, TransformSelection,
    },
    interpolate::{Number, Value},
    scales::{
        GgplotTransform, NumericFamily, NumericScale, NumericScaleSpec, ScaleInput,
        StandaloneScale, StandaloneScaleSpec,
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct Affine {
    slope: f64,
}
impl PointwiseTransform for Affine {
    fn forward(&self, x: f64) -> f64 {
        self.slope * x + 3.
    }
    fn inverse(&self, x: f64) -> f64 {
        (x - 3.) / self.slope
    }
    fn domain(&self) -> [Number; 2] {
        [Number(f64::NEG_INFINITY), Number(f64::INFINITY)]
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        true
    }
}
struct Factory {
    portable: bool,
    calls: Arc<AtomicUsize>,
}
impl CustomTransformFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.affine", Revision::new(1), self.portable)
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        if p.as_f64().is_some_and(|v| v.is_finite() && v != 0.) {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Invalid affine slope.",
                "Use a finite nonzero slope.",
            ))
        }
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(Affine {
            slope: p.as_f64().unwrap(),
        }))
    }
}
fn selection() -> GgplotTransform {
    GgplotTransform::Registered {
        selection: TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "test.affine".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!(2.),
        })
        .into(),
    }
}
fn spec(transform: GgplotTransform) -> StandaloneScaleSpec {
    let mut s = NumericScaleSpec::d3(NumericFamily::Ggplot { transform });
    s.domain = vec![Number(-2.), Number(2.)];
    s.range = vec![Number(0.), Number(100.)];
    StandaloneScaleSpec::Numeric(s)
}
fn registry(portable: bool) -> (ExtensionRegistry, Arc<AtomicUsize>) {
    let mut r = ExtensionRegistry::new();
    let calls = Arc::new(AtomicUsize::new(0));
    r.register_transform(Arc::new(Factory {
        portable,
        calls: calls.clone(),
    }))
    .unwrap();
    (r, calls)
}
fn sample(s: &StandaloneScale, x: f64) -> f64 {
    let Value::Number(v) = s.map(ScaleInput::Number(Number(x))).unwrap() else {
        panic!("numeric")
    };
    v.0
}
#[test]
fn standalone_retains_registry_and_requires_exact_portable_version() {
    let (r, calls) = registry(true);
    assert!(StandaloneScale::new(spec(selection())).is_err());
    let s = StandaloneScale::new_with_registry(spec(selection()), &r).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(sample(&s, -1.), 25.);
    let wire = s.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
        10
    );
    assert!(StandaloneScale::from_json(&wire).is_err());
    let decoded = StandaloneScale::from_json_with_registry(&wire, &r).unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "one preparation per decode"
    );
    assert_eq!(sample(&decoded, 1.), 75.);
    assert!(
        StandaloneScale::from_json_with_registry(&wire.replace("test.affine", "test.missing"), &r)
            .is_err()
    );
    assert!(
        StandaloneScale::from_json_with_registry(
            &wire.replacen("\"version\":10", "\"version\":9", 1),
            &r
        )
        .is_err()
    );
    drop(r);
    assert_eq!(sample(&s.clone(), 0.), 50.);
    assert_eq!(sample(&s.reconfigure(spec(selection())).unwrap(), 1.), 75.);
}
#[test]
fn resolved_scalar_and_numeric_wire_follow_captured_functions() {
    let (r, _) = registry(true);
    assert!(selection().validate().is_err());
    let t = r.resolve_transform(&selection()).unwrap();
    assert_eq!(
        t.forward_population(&[-2., -1., 0., 1., 2.]).unwrap(),
        [-1., 1., 3., 5., 7.]
    );
    for (y, x) in [(-1., -2.), (1., -1.), (3., 0.), (5., 1.), (7., 2.)] {
        assert_eq!(t.inverse(y), x);
    }
    assert!(t.forward(f64::NAN).is_nan());
    assert_eq!(t.forward(f64::INFINITY), f64::INFINITY);
    let StandaloneScaleSpec::Numeric(s) = spec(t) else {
        unreachable!()
    };
    let n = NumericScale::new(s).unwrap();
    let wire = n.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
        4
    );
    assert!(NumericScale::from_json(&wire).is_err());
    let n = NumericScale::from_json_with_registry(&wire, &r).unwrap();
    assert_eq!(n.invert(25.).unwrap(), -1.);
}
#[test]
fn registration_snapshot_and_native_transport_are_explicit() {
    let mut r = ExtensionRegistry::new();
    let before = r.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    r.register_transform(Arc::new(Factory {
        portable: false,
        calls: calls.clone(),
    }))
    .unwrap();
    assert!(before.resolve_transform(&selection()).is_err());
    assert!(
        r.register_transform(Arc::new(Factory {
            portable: false,
            calls
        }))
        .is_err()
    );
    let s = StandaloneScale::new_with_registry(spec(selection()), &r).unwrap();
    assert_eq!(sample(&s, 1.), 75.);
    assert!(s.to_json().is_err());
    let (portable, _) = registry(true);
    let wire = StandaloneScale::new_with_registry(spec(selection()), &portable)
        .unwrap()
        .to_json()
        .unwrap();
    assert!(StandaloneScale::from_json_with_registry(&wire, &r).is_err());
    let compose = GgplotTransform::Compose {
        transforms: vec![selection(), GgplotTransform::Reverse],
    };
    let s = StandaloneScale::new_with_registry(spec(compose), &portable).unwrap();
    assert_eq!(sample(&s, 1.), 75.);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&s.to_json().unwrap()).unwrap()["version"],
        10
    );
}

struct Square {
    invalid_domain: bool,
}
impl PointwiseTransform for Square {
    fn forward(&self, x: f64) -> f64 {
        x * x
    }
    fn inverse(&self, x: f64) -> f64 {
        x.sqrt()
    }
    fn domain(&self) -> [Number; 2] {
        if self.invalid_domain {
            [Number(1.), Number(0.)]
        } else {
            [Number(0.), Number(f64::INFINITY)]
        }
    }
    fn monotone_on(&self, bounds: [f64; 2]) -> bool {
        bounds[0] >= 0.
    }
    fn validate_population(&self, values: &[f64]) -> ChartResult<()> {
        if values.iter().any(|v| *v < 0.) {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::NumericalDomain,
                "Negative population.",
                "Use nonnegative inputs.",
            ))
        } else {
            Ok(())
        }
    }
}
struct SquareFactory;
impl CustomTransformFactory for SquareFactory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.square", Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        if p.is_boolean() {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Expected domain flag.",
                "Use a boolean.",
            ))
        }
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(Square {
            invalid_domain: p.as_bool().unwrap(),
        }))
    }
}
#[test]
fn domain_population_and_inverse_branch_are_checked() {
    let mut r = ExtensionRegistry::new();
    r.register_transform(Arc::new(SquareFactory)).unwrap();
    let t = |invalid| GgplotTransform::Registered {
        selection: TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "test.square".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!(invalid),
        })
        .into(),
    };
    assert_eq!(
        r.resolve_transform(&t(true)).unwrap_err().code,
        chart_core::DiagnosticCode::NumericalDomain
    );
    let t = r.resolve_transform(&t(false)).unwrap();
    assert_eq!(
        t.forward_population(&[-1., f64::NAN]).unwrap_err().code,
        chart_core::DiagnosticCode::NumericalDomain
    );
    assert_eq!(t.forward_population(&[0., 1., 2.]).unwrap(), [0., 1., 4.]);
    let StandaloneScaleSpec::Numeric(mut s) = spec(t) else {
        unreachable!()
    };
    assert_eq!(
        NumericScale::new(s.clone())
            .unwrap()
            .invert(50.)
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::UnsupportedCapability
    );
    s.domain = [1., 2.].map(Number).to_vec();
    let n = NumericScale::new(s).unwrap();
    assert_eq!(n.invert(0.).unwrap(), 1.);
    assert_eq!(n.invert(100.).unwrap(), 2.);
}
#[test]
fn invalid_configuration_and_depth_reject_before_factory_execution() {
    let (r, calls) = registry(true);
    let mut descriptor = serde_json::to_value(selection()).unwrap();
    descriptor["Registered"]["selection"]["call"]["parameters"] = serde_json::json!(0.);
    let invalid = serde_json::from_value(descriptor).unwrap();
    assert!(r.resolve_transform(&invalid).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut deep = selection();
    for _ in 0..40 {
        deep = GgplotTransform::Compose {
            transforms: vec![deep],
        };
    }
    assert_eq!(
        r.resolve_transform(&deep).unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn retained_binned_transform_participates_in_wire_and_portability_checks() {
    use chart_core::{
        ScaleId,
        grammar::ChartDefinition,
        layout::{AxisScale, AxisSide, AxisSpec},
        scales::{GgplotBinnedPosition, ScaleTransform},
    };
    let (r, _) = registry(false);
    let bins = GgplotBinnedPosition {
        transform: Some(ScaleTransform::Ggplot {
            transform: r.resolve_transform(&selection()).unwrap(),
        }),
        ..Default::default()
    };
    let prepared = bins
        .train_with_registry(&[Some(Number(-2.)), Some(Number(2.))], &r)
        .unwrap();
    let mut axis = AxisSpec::new(ScaleId::new(0), AxisSide::Bottom);
    axis.scale = AxisScale::Binned {
        spec: Box::default(),
        prepared: Some(Arc::new(prepared)),
    };
    let mut definition = ChartDefinition::new(Revision::new(1));
    definition.axes.push(axis);
    assert_eq!(definition.wire_version(), 55);
    assert_eq!(
        r.validate_portable_interpolations(&definition)
            .unwrap_err()
            .code,
        chart_core::DiagnosticCode::UnsupportedCapability
    );
}
