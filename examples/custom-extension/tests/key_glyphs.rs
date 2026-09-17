//! External key protocol: context, immutable replay, missing/native capability and output validation.
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
        .aes(aes().x("x").y(1.).color("g"))
        .layer(points().key_glyph(
            operation,
            Revision::new(1),
            serde_json::json!({"padding":0.1}),
        ))
        .build()
}
#[test]
fn portable_key_roundtrip_and_native_rejection() {
    let registry = chart_extension_example::registry().unwrap();
    let p = author(
        registry.clone(),
        chart_extension_example::key_glyphs::DIAMOND,
    )
    .unwrap();
    let json = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 82);
    assert!(Plot::from_json(&json).is_err());
    let restored = Plot::from_json_with_extensions(&json, registry.clone()).unwrap();
    let a = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let b = layout(
        restored.chart().unwrap().prepare().unwrap(),
        &request(),
        &Metrics,
    )
    .unwrap();
    assert_eq!(a.scene().items(), b.scene().items());
    let keys: Vec<_> = a
        .scene()
        .items()
        .iter()
        .filter(|i| {
            i.guide
                .as_ref()
                .is_some_and(|g| g.role == GuideRole::LegendKey)
        })
        .collect();
    assert_eq!(keys.len(), 2);
    assert!(
        keys.iter()
            .all(|i| matches!(i.primitive, Primitive::VectorPath { .. }))
    );
    let native = author(
        registry,
        chart_extension_example::key_glyphs::NATIVE_DIAMOND,
    )
    .unwrap();
    assert!(native.to_json().is_err());
}
struct Bad;
impl CustomKeyGlyph for Bad {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.bad_key", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn draw(&self, input: KeyGlyphInput<'_>) -> ChartResult<KeyGlyphOutput> {
        assert!(matches!(input.value, composition::ScaleValue::Category(_)));
        assert!(input.index < 2);
        Ok(KeyGlyphOutput {
            size: [f64::NAN, 1.],
            paths: vec![],
        })
    }
}
#[test]
fn malformed_output_and_registry_conflict_reject() {
    let mut registry = ExtensionRegistry::new();
    registry.register_key_glyph(Arc::new(Bad)).unwrap();
    assert!(registry.register_key_glyph(Arc::new(Bad)).is_err());
    let p = author(Arc::new(registry), "example.bad_key").unwrap();
    assert!(layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).is_err());
    assert!(author(Arc::new(ExtensionRegistry::new()), "missing.key").is_err());
}
