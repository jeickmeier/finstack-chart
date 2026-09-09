//! Registered protocols share grammar, projection, guide geometry and portable ownership.
use chart_core::{
    ChartResult, DiagnosticCode, Rect, ResourceId, Revision,
    grammar::{
        Compiler, GroupValue, NumericAesthetic as A, OperationRef, PreparedGeometry, ShapeFamily,
        ShapeOperation,
    },
    layout::{AxisScale, LayoutRequest, layout},
    path::Precision,
    plot::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use serde_json::json;
use std::sync::Arc;
fn op(name: &str, parameters: serde_json::Value) -> ShapeOperation {
    ShapeOperation {
        operation: OperationRef::new(format!("example.{name}"), Revision::new(1)),
        parameters,
    }
}
fn prepare(p: &Plot) -> chart_core::grammar::PreparedChart {
    Compiler::with_extensions(p.extensions().clone())
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap()
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(prepared: chart_core::grammar::PreparedChart) -> chart_core::layout::LaidOutChart {
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.padding = 0.;
    for axis in &mut request.axes {
        axis.visible = false;
        axis.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 4.).unwrap()));
    }
    layout(Arc::new(prepared), &request, &Metrics).unwrap()
}
fn svg(geometry: &chart_core::path::PathGeometry) -> String {
    geometry
        .to_svg(Precision::from_digits(3.).unwrap(), 10000)
        .unwrap()
}
#[test]
fn custom_curve_survives_registry_drop_wire_roundtrip_and_projected_layout() {
    let registry = chart_extension_example::registry().unwrap();
    let data = Data::columns()
        .column("x", [1., 3.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let p = plot(data.clone())
        .extensions(registry.clone())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_line().shape_protocol(ShapeFamily::Curve, op("shift_curve", json!({"amount":5}))),
        )
        .build()
        .unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 9);
    assert!(Plot::from_json(&wire).is_err());
    let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    let prepared = prepare(&restored);
    drop(restored);
    drop(p);
    drop(registry);
    let f = frame(prepared);
    let paths: Vec<_> = f
        .scene()
        .items()
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::ShapePath { geometry, .. } => Some(svg(geometry)),
            _ => None,
        })
        .collect();
    assert_eq!(paths, ["M100,155L300,105"]);
    let native = plot(data)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_line()
                .shape_protocol(ShapeFamily::Curve, op("native_curve", json!({"amount":5}))),
        )
        .build()
        .unwrap();
    assert_eq!(
        native.to_json().unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    assert_eq!(prepare(&native).layers()[0].marks().len(), 1);
}
#[test]
fn custom_symbol_size_guides_use_the_same_draw_protocol() {
    let data = Data::columns()
        .column("x", [1., 3.])
        .column("y", [2., 2.])
        .column("area", [16., 64.])
        .build()
        .unwrap();
    let p = plot(data)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_symbol()
                .shape_value(A::AreaSize, "area")
                .symbol_size_guide("Area", vec![16., 64.])
                .shape_protocol(
                    ShapeFamily::Symbol,
                    op("rectangle_symbol", json!({"amount":4})),
                ),
        )
        .build()
        .unwrap();
    let pre = prepare(&p);
    let layer = &pre.layers()[0];
    let expected = ["M-4,-1h8v2h-8Z", "M-8,-2h16v4h-16Z"];
    for (mark, expected) in layer.marks().iter().zip(expected) {
        let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
            panic!("symbol")
        };
        assert_eq!(svg(geometry), expected);
    }
    for (entry, expected) in layer.symbol_legends()[0].entries.iter().zip(expected) {
        assert_eq!(svg(entry.geometry.as_ref().unwrap()), expected);
    }
    assert!(frame(pre).scene().items().len() > 2);
}
#[test]
fn registered_pie_order_and_stack_offset_keep_source_identity_and_domains() {
    let data = Data::columns()
        .column("x", [2., 2., 2.])
        .column("y", [2., 2., 2.])
        .column("value", [1., 2., 3.])
        .keys([9007199254740995, 9007199254740993, 9007199254740994])
        .build()
        .unwrap();
    let p = plot(data)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_pie()
                .shape_value(A::PieValue, "value")
                .pie_angles(chart_core::shape::PieAngles {
                    end_angle: 6.,
                    ..Default::default()
                })
                .shape_protocol(
                    ShapeFamily::PieComparator,
                    op("field_comparator", json!({"field":"key"})),
                ),
        )
        .build()
        .unwrap();
    let pre = prepare(&p);
    for (mark, angles) in pre.layers()[0]
        .marks()
        .iter()
        .zip([[5., 6.], [0., 2.], [2., 5.]])
    {
        let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
            panic!("pie")
        };
        let expected = chart_core::shape::Arc::new()
            .generate(chart_core::shape::ArcDatum {
                inner_radius: 0.,
                outer_radius: 40.,
                start_angle: angles[0],
                end_angle: angles[1],
                ..Default::default()
            })
            .unwrap()
            .geometry();
        assert_eq!(geometry, &expected);
        assert_eq!(mark.targets.len(), 1);
    }
    let data = Data::columns()
        .column("x", [0., 0., 1., 1.])
        .column("y", [3., 1., 4., 2.])
        .column("group", [0_i64, 1, 0, 1])
        .build()
        .unwrap();
    let p = plot(data)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("x").x2("x").y("y").y2(0.).group("group"))
        .layer(
            bars()
                .position(shape_stack(vec![GroupValue::Int(0), GroupValue::Int(1)]))
                .shape_protocol(ShapeFamily::StackOrder, op("first_value_order", json!({})))
                .shape_protocol(
                    ShapeFamily::StackOffset,
                    op("shift_offset", json!({"amount":10})),
                ),
        )
        .build()
        .unwrap();
    let pre = prepare(&p);
    let y = pre.layers()[0].domains().y.unwrap();
    assert_eq!([y.minimum, y.maximum], [10., 16.]);
    let endpoints: Vec<_> = pre.layers()[0]
        .marks()
        .iter()
        .map(|m| match &m.geometry {
            PreparedGeometry::Bar { from, to, .. } => [from.y(), to.y()],
            _ => panic!("bar"),
        })
        .collect();
    assert_eq!(endpoints, [[14., 11.], [11., 10.], [16., 12.], [12., 10.]]);
}

#[test]
fn registered_curve_budget_rejects_before_text_callbacks() {
    struct Count(std::sync::atomic::AtomicUsize);
    impl TextMeasurer for Count {
        fn measure(&self, _: TextRequest<'_>) -> ChartResult<TextMetrics> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            TextMetrics::new(5., 8., 2.)
        }
    }
    let d = Data::columns()
        .column("x", [1., 3.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let p = plot(d)
        .extensions(chart_extension_example::registry().unwrap())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_line().shape_protocol(ShapeFamily::Curve, op("shift_curve", json!({"amount":5}))),
        )
        .build()
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.limits.max_path_commands = 3;
    let metrics = Count(std::sync::atomic::AtomicUsize::new(0));
    assert_eq!(
        layout(Arc::new(prepare(&p)), &request, &metrics)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(metrics.0.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn builtin_setters_replace_registered_protocols_without_stale_selection() {
    let d = Data::columns()
        .column("x", [1., 3.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let layer = shape_line()
        .shape_protocol(ShapeFamily::Curve, op("shift_curve", json!({"amount":5})))
        .curve(chart_core::shape::CurveSpec::Step);
    let p = plot(d)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .build()
        .unwrap();
    assert!(p.definition().layers[0].shape_protocols.is_empty());
    assert_eq!(p.definition().wire_version(), 7);
    assert!(p.to_json().is_ok());
}
