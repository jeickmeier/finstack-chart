//! Coupled coordinates share projection, inverse, replay and malformed-output rejection.
use chart_core::{grammar::*, layout::*, prelude::*, services::*, *};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 6., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn author(registry: Arc<ExtensionRegistry>, operation: &str) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [0., 0.25, 0.5, 0.75, 1.])
        .column("y", [0., 0.5, 0.5, 0.5, 1.])
        .build()?;
    plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(line())
        .coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
            expand: [false; 4],
            ..Default::default()
        }))
        .registered_coordinate(
            operation,
            Revision::new(1),
            serde_json::json!({"amplitude":0.1}),
        )
        .build()
}
#[test]
fn paired_projection_inverse_and_roundtrip() {
    let registry = chart_extension_example::registry().unwrap();
    let p = author(registry.clone(), chart_extension_example::coordinates::WAVE).unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 85);
    assert!(Plot::from_json(&wire).is_err());
    let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
    let a = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let b = layout(
        restored.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(a.scene().items(), b.scene().items());
    let x = a
        .axes()
        .values()
        .find(|a| a.spec.side.horizontal())
        .unwrap();
    let y = a
        .axes()
        .values()
        .find(|a| !a.spec.side.horizontal())
        .unwrap();
    let bounds = a.plot().unwrap();
    let map = Coordinates::for_chart(a.prepared(), x, y, bounds).unwrap();
    let point = map
        .project(&ScaleValue::Number(0.25), &ScaleValue::Number(0.5))
        .unwrap()
        .unwrap();
    assert!((point.x() - (bounds.origin().x() + 0.25 * bounds.width())).abs() < 1e-10);
    assert!((point.y() - (bounds.max_y() - 0.6 * bounds.height())).abs() < 1e-10);
    let CoordinateInverse::Values(values) = map.inverse(point, 8).unwrap() else {
        panic!("inverse")
    };
    let (ScaleValue::Number(x), ScaleValue::Number(y)) = values[0] else {
        panic!("numeric")
    };
    assert!((x - 0.25).abs() < 1e-12 && (y - 0.5).abs() < 1e-12);
    assert!(
        author(registry, chart_extension_example::coordinates::NATIVE_WAVE)
            .unwrap()
            .to_json()
            .is_err()
    );
}
#[derive(Debug)]
struct Bad;
impl TrainedCoordinate for Bad {
    fn forward(&self, _: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        Ok(Some([f64::NAN, 0.]))
    }
    fn bounds(&self, _: [[f64; 2]; 2]) -> ChartResult<Option<[[f64; 2]; 2]>> {
        Ok(Some([[0., 1.], [0., 1.]]))
    }
    fn has_inverse(&self) -> bool {
        false
    }
    fn inverse(&self, _: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        panic!("undeclared inverse must not execute")
    }
}
impl CustomCoordinate for Bad {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.bad_coordinate", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn train(&self, input: CoordinateTrainInput<'_>) -> ChartResult<Arc<dyn TrainedCoordinate>> {
        assert_eq!(input.views, [[0., 1.], [0., 1.]]);
        Ok(Arc::new(Bad))
    }
}
#[test]
fn malformed_coordinate_and_missing_identity_reject() {
    let mut registry = ExtensionRegistry::new();
    registry.register_coordinate(Arc::new(Bad)).unwrap();
    assert!(registry.register_coordinate(Arc::new(Bad)).is_err());
    let p = author(Arc::new(registry), "example.bad_coordinate").unwrap();
    assert!(layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).is_err());
    assert!(author(Arc::new(ExtensionRegistry::new()), "missing").is_err());
}
#[test]
fn nonlinear_segment_enclosure_preserves_unsampled_extrema() {
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0.5, 0.5])
        .build()
        .unwrap();
    let p = plot(data)
        .extensions(chart_extension_example::registry().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
            expand: [false; 4],
            ylim: Some([Some(ScaleValue::Number(0.)), Some(ScaleValue::Number(1.))]),
            ..Default::default()
        }))
        .registered_coordinate(
            chart_extension_example::coordinates::WAVE,
            Revision::new(1),
            serde_json::json!({"amplitude":0.1}),
        )
        .build()
        .unwrap();
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let panel = frame.plot().unwrap();
    let geometry = frame
        .scene()
        .items()
        .iter()
        .filter(|i| i.layer.is_some())
        .find_map(|i| match &i.primitive {
            scene::Primitive::ShapePath { geometry, .. } => Some(geometry),
            _ => None,
        })
        .unwrap();
    assert!(
        geometry.commands().len() > 16,
        "A horizontal chord must subdivide around the hidden wave extrema"
    );
    let bounds = geometry.bounds(0.01, 100_000).unwrap().unwrap();
    assert!((bounds.origin().y() - (panel.max_y() - 0.6 * panel.height())).abs() < 0.12);
    assert!((bounds.max_y() - (panel.max_y() - 0.4 * panel.height())).abs() < 0.12);
}

#[derive(Debug)]
struct ForwardOnly;
impl TrainedCoordinate for ForwardOnly {
    fn forward(&self, point: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        Ok(Some(point))
    }
    fn bounds(&self, input: [[f64; 2]; 2]) -> ChartResult<Option<[[f64; 2]; 2]>> {
        Ok(Some(input))
    }
    fn has_inverse(&self) -> bool {
        false
    }
    fn inverse(&self, _: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        panic!("Undeclared inverse cannot execute")
    }
}
impl CustomCoordinate for ForwardOnly {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.forward_only", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn train(&self, _: CoordinateTrainInput<'_>) -> ChartResult<Arc<dyn TrainedCoordinate>> {
        Ok(Arc::new(ForwardOnly))
    }
}
#[test]
fn forward_only_map_renders_and_explicitly_withholds_inverse() {
    let mut registry = ExtensionRegistry::new();
    registry.register_coordinate(Arc::new(ForwardOnly)).unwrap();
    let p = author(Arc::new(registry), "example.forward_only").unwrap();
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let x = frame
        .axes()
        .values()
        .find(|a| a.spec.side.horizontal())
        .unwrap();
    let y = frame
        .axes()
        .values()
        .find(|a| !a.spec.side.horizontal())
        .unwrap();
    let map = Coordinates::for_chart(frame.prepared(), x, y, frame.plot().unwrap()).unwrap();
    let point = map
        .project(&ScaleValue::Number(0.5), &ScaleValue::Number(0.5))
        .unwrap()
        .unwrap();
    assert!(matches!(
        map.inverse(point, 8).unwrap(),
        CoordinateInverse::Unavailable
    ));
}
