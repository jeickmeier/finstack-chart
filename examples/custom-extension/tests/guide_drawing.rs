//! Registered guide drawing preserves training and uses one destination renderer.
use chart_core::{grammar::*, layout::*, prelude::*, scene::*, services::*, *};
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
        .column("x", [1., 2.])
        .column("g", ["a", "b"])
        .build()?;
    plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.).color("g").color_scale("groups"))
        .scale(color_discrete("groups"))
        .layer(points())
        .legend(legend().scale("groups").registered(
            operation,
            Revision::new(1),
            serde_json::json!({}),
        ))
        .build()
}
#[test]
fn drawing_roundtrip_native_rejection_and_replay() {
    let registry = chart_extension_example::registry().unwrap();
    let p = author(
        registry.clone(),
        chart_extension_example::guide_drawing::STRIP,
    )
    .unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 83);
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
    assert!(
        a.scene()
            .items()
            .iter()
            .any(|i| matches!(i.primitive, Primitive::VectorPath { .. }))
    );
    assert!(
        author(
            registry,
            chart_extension_example::guide_drawing::NATIVE_STRIP
        )
        .unwrap()
        .to_json()
        .is_err()
    );
}
struct Bad;
impl CustomGuideDrawing for Bad {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.bad_guide", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn draw(&self, input: GuideDrawingInput<'_>) -> ChartResult<KeyGlyphOutput> {
        assert_eq!(input.labels, ["a", "b"]);
        assert_eq!(input.values.len(), 2);
        assert_eq!(input.colors.len(), 2);
        assert_eq!(input.units, Units::Points);
        Ok(KeyGlyphOutput {
            size: [f64::NAN, 1.],
            paths: vec![],
        })
    }
}
#[test]
fn malformed_output_is_rejected_with_trained_context() {
    let mut registry = ExtensionRegistry::new();
    registry.register_guide_drawing(Arc::new(Bad)).unwrap();
    assert!(registry.register_guide_drawing(Arc::new(Bad)).is_err());
    let p = author(Arc::new(registry), "example.bad_guide").unwrap();
    assert!(layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).is_err());
    assert!(author(Arc::new(ExtensionRegistry::new()), "missing").is_err());
}
