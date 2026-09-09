//! Independent protocol, registry and resource-boundary acceptance fixtures.
use chart_core::{DiagnosticCode, Revision, grammar::*, path::PathLimits, shape::*};
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn op(name: &str, parameters: serde_json::Value) -> ShapeOperation {
    ShapeOperation {
        operation: OperationRef::new(format!("example.{name}"), Revision::new(1)),
        parameters,
    }
}
#[test]
fn registered_curve_lifecycle_gaps_radial_links_and_owned_reuse() {
    let registry = chart_extension_example::registry().unwrap();
    let shift = op("shift_curve", json!({"amount": 2}));
    let rows = [[0., 1.], [2., 3.], [9., 9.], [4., 2.], [5., 1.]];
    let mask = vec![true, true, false, true, true];
    let line = Line::new().defined(mask.clone());
    let area = Area::new().defined(mask);
    let retained = line.generate_registered(&rows, &registry, &shift).unwrap();
    assert_eq!(retained.to_svg().unwrap(), "M0,3L2,5M4,4L5,3");
    for _ in 0..3 {
        assert_eq!(
            area.generate_registered(&rows, &registry, &shift)
                .unwrap()
                .to_svg()
                .unwrap(),
            "M0,3L2,5L2,2L0,2ZM4,4L5,3L5,2L4,2Z"
        );
    }
    assert_eq!(retained.to_svg().unwrap(), "M0,3L2,5M4,4L5,3");
    assert_eq!(
        LineRadial::new()
            .generate_registered(
                &[[0., 2.], [std::f64::consts::FRAC_PI_2, 2.]],
                &registry,
                &shift
            )
            .unwrap()
            .to_svg()
            .unwrap(),
        "M0,0L2,2"
    );
    assert_eq!(
        Link::new(CurveSpec::Linear)
            .unwrap()
            .generate_registered(
                &LinkDatum {
                    source: vec![0., 1.],
                    target: vec![2., 3.]
                },
                &registry,
                &shift
            )
            .unwrap()
            .to_svg()
            .unwrap(),
        "M0,3L2,5"
    );
    let limits = ShapeLimits {
        max_points: 5,
        path: PathLimits {
            max_commands: 2,
            ..Default::default()
        },
    };
    assert_eq!(
        area.clone()
            .limits(limits)
            .generate_registered(&rows, &registry, &shift)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(
        Line::new()
            .generate_registered(
                &[[0., f64::MAX]],
                &registry,
                &op("shift_curve", json!({"amount": f64::MAX}))
            )
            .unwrap_err()
            .code,
        DiagnosticCode::NumericalDomain
    );
}
#[test]
fn registered_symbol_pie_and_stack_have_independent_expected_values() {
    let registry = chart_extension_example::registry().unwrap();
    let symbol = Symbol::new().size(16.);
    let rectangle = op("rectangle_symbol", json!({"amount": 4}));
    assert_eq!(
        symbol
            .generate_registered(&registry, &rectangle)
            .unwrap()
            .to_svg()
            .unwrap(),
        "M-4,-1h8v2h-8Z"
    );
    let data = vec![
        json!({"rank":"9007199254740993", "label":"a"}),
        json!({"rank":"9007199254740992", "label":"b"}),
        json!({"rank":"9007199254740993", "label":"c"}),
    ];
    let compare = op("field_comparator", json!({"field":"rank"}));
    let pie = Pie::new()
        .end_angle(6.)
        .layout_registered(&data, &[1., 2., 3.], &registry, &compare)
        .unwrap();
    assert_eq!(pie.iter().map(|v| v.index).collect::<Vec<_>>(), [1, 0, 2]);
    assert_eq!(
        pie.iter()
            .map(|v| [v.start_angle, v.end_angle])
            .collect::<Vec<_>>(),
        [[2., 3.], [0., 2.], [3., 6.]]
    );
    assert_eq!(
        pie.iter().map(|v| &v.data).collect::<Vec<_>>(),
        data.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        Pie::new()
            .layout_registered(&[json!({}), json!({})], &[1., 1.], &registry, &compare)
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    let stack = Stack::new().keys(vec!["a".into(), "b".into()]);
    let order = op("first_value_order", json!({}));
    let offset = op("shift_offset", json!({"amount": 10}));
    let result = stack
        .layout_registered(
            &data[..2],
            &[vec![Some(3.), Some(1.)], vec![Some(4.), Some(2.)]],
            &registry,
            Some(&order),
            Some(&offset),
        )
        .unwrap();
    assert_eq!(result.iter().map(|s| s.index).collect::<Vec<_>>(), [1, 0]);
    let endpoints = serde_json::to_value(&result).unwrap();
    assert_eq!(endpoints[0]["points"][0]["y0"], 11.);
    assert_eq!(endpoints[0]["points"][0]["y1"], 14.);
    assert_eq!(endpoints[1]["points"][1]["y0"], 10.);
    assert_eq!(endpoints[1]["points"][1]["y1"], 12.);
    assert_eq!(
        stack
            .limits(StackLimits {
                max_work: 7,
                ..Default::default()
            })
            .layout_registered(
                &data[..2],
                &[vec![Some(3.), Some(1.)], vec![Some(4.), Some(2.)]],
                &registry,
                Some(&order),
                Some(&offset)
            )
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
}
struct WrongProtocol(Arc<AtomicUsize>);
impl CustomShape for WrongProtocol {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.wrong", Revision::new(1), true)
    }
    fn family(&self) -> ShapeFamily {
        ShapeFamily::Curve
    }
    fn resolve(&self, _: &serde_json::Value) -> chart_core::ChartResult<ShapeProtocol> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(ShapeProtocol::Symbol(Box::new(SymbolKind::Circle)))
    }
}
#[test]
fn registry_versions_families_parameters_and_native_only_boundaries() {
    let mut registry = (*chart_extension_example::registry().unwrap()).clone();
    let shift = op("shift_curve", json!({"amount": 2}));
    let wire = registry.shape_to_json(&shift, ShapeFamily::Curve).unwrap();
    assert_eq!(
        serde_json::from_str::<ShapeOperation>(&wire).unwrap(),
        shift
    );
    let native = op("native_curve", json!({"amount": 2}));
    assert!(registry.resolve_shape(&native, ShapeFamily::Curve).is_ok());
    assert_eq!(
        registry
            .shape_to_json(&native, ShapeFamily::Curve)
            .unwrap_err()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    let mut unknown = shift.clone();
    unknown.operation.version = Revision::new(2);
    assert!(
        registry
            .resolve_shape(&unknown, ShapeFamily::Curve)
            .is_err()
    );
    assert!(registry.resolve_shape(&shift, ShapeFamily::Symbol).is_err());
    assert!(
        registry
            .resolve_shape(
                &op("shift_curve", json!({"amount":2,"unexpected":true})),
                ShapeFamily::Curve
            )
            .is_err()
    );
    let calls = Arc::new(AtomicUsize::new(0));
    registry
        .register_shape(Arc::new(WrongProtocol(calls.clone())))
        .unwrap();
    assert!(
        registry
            .register_shape(Arc::new(WrongProtocol(calls.clone())))
            .is_err()
    );
    assert!(
        registry
            .resolve_shape(&op("wrong", json!("x".repeat(65537))), ShapeFamily::Curve)
            .is_err()
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(
        registry
            .resolve_shape(&op("wrong", json!({})), ShapeFamily::Symbol)
            .is_err()
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(
        registry
            .resolve_shape(&op("wrong", json!({})), ShapeFamily::Curve)
            .is_err()
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

struct BadSymbol;
impl SymbolDraw for BadSymbol {
    fn draw(&self, path: &mut chart_core::path::Path, _: f64) -> chart_core::ChartResult<()> {
        path.move_to(f64::NAN, 0.)
    }
}
struct BadOrder;
impl StackOrdering for BadOrder {
    fn order(&self, series: &[Vec<[f64; 2]>]) -> chart_core::ChartResult<Vec<usize>> {
        Ok(vec![0; series.len()])
    }
}
struct BadOffset;
impl StackOffsetting for BadOffset {
    fn offset(&self, series: &mut [Vec<[f64; 2]>], _: &[usize]) -> chart_core::ChartResult<()> {
        series[0].push([0., 0.]);
        Ok(())
    }
}
struct ShortBound;
impl CurveFactory for ShortBound {
    fn command_bound(&self, points: usize) -> Option<usize> {
        Some(points)
    }
    fn supports_area(&self) -> bool {
        true
    }
    fn create<'a>(
        &'a self,
        path: &'a mut chart_core::path::Path,
        points: usize,
    ) -> chart_core::ChartResult<Box<dyn CurveProtocol + 'a>> {
        CurveSpec::Step.create(path, points)
    }
}
struct InvalidOutput(ShapeFamily);
impl CustomShape for InvalidOutput {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(format!("invalid.{:?}", self.0), Revision::new(1), true)
    }
    fn family(&self) -> ShapeFamily {
        self.0
    }
    fn resolve(&self, _: &serde_json::Value) -> chart_core::ChartResult<ShapeProtocol> {
        Ok(match self.0 {
            ShapeFamily::Curve => ShapeProtocol::Curve(Box::new(ShortBound)),
            ShapeFamily::Symbol => ShapeProtocol::Symbol(Box::new(BadSymbol)),
            ShapeFamily::StackOrder => ShapeProtocol::StackOrder(Box::new(BadOrder)),
            ShapeFamily::StackOffset => ShapeProtocol::StackOffset(Box::new(BadOffset)),
            _ => unreachable!(),
        })
    }
}
#[test]
fn registered_invalid_outputs_cannot_escape_native_shape_checks() {
    let mut registry = ExtensionRegistry::new();
    for family in [
        ShapeFamily::Curve,
        ShapeFamily::Symbol,
        ShapeFamily::StackOrder,
        ShapeFamily::StackOffset,
    ] {
        registry
            .register_shape(Arc::new(InvalidOutput(family)))
            .unwrap();
    }
    let selection = |family| ShapeOperation {
        operation: OperationRef::new(format!("invalid.{family:?}"), Revision::new(1)),
        parameters: json!({}),
    };
    assert_eq!(
        Symbol::new()
            .generate_registered(&registry, &selection(ShapeFamily::Symbol))
            .unwrap_err()
            .code,
        DiagnosticCode::NumericalDomain
    );
    let stack = Stack::new().keys(vec!["a".into(), "b".into()]);
    assert_eq!(
        stack
            .layout_registered(
                &[0],
                &[vec![Some(1.), Some(2.)]],
                &registry,
                Some(&selection(ShapeFamily::StackOrder)),
                None
            )
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    assert_eq!(
        stack
            .layout_registered(
                &[0],
                &[vec![Some(1.), Some(2.)]],
                &registry,
                None,
                Some(&selection(ShapeFamily::StackOffset))
            )
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    assert_eq!(
        Line::new()
            .generate_registered(
                &[[0., 0.], [1., 1.]],
                &registry,
                &selection(ShapeFamily::Curve)
            )
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
}
struct CatalogEntry(u64);
impl CustomShape for CatalogEntry {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.catalog", Revision::new(self.0), true)
    }
    fn family(&self) -> ShapeFamily {
        ShapeFamily::Curve
    }
    fn resolve(&self, _: &serde_json::Value) -> chart_core::ChartResult<ShapeProtocol> {
        Ok(ShapeProtocol::Curve(Box::new(CurveSpec::Linear)))
    }
}
#[test]
fn shape_catalog_has_exact_version_and_registration_bounds() {
    let mut registry = ExtensionRegistry::new();
    assert_eq!(
        registry
            .register_shape(Arc::new(CatalogEntry(0)))
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    for version in 1..=64 {
        registry
            .register_shape(Arc::new(CatalogEntry(version)))
            .unwrap();
    }
    assert_eq!(
        registry
            .register_shape(Arc::new(CatalogEntry(65)))
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(
        registry
            .shape_descriptor(&OperationRef::new("test.catalog", Revision::new(64)))
            .unwrap()
            .operation
            .version,
        Revision::new(64)
    );
}
