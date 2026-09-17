//! External facet planning retains exact population identities and common validation.
use chart_core::{grammar::*, prelude::*, *};
use std::sync::Arc;
fn author(registry: Arc<ExtensionRegistry>, operation: &str) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("g", ["a", "b", "c"])
        .build()?;
    plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.))
        .layer(points())
        .facet(facet_wrap("g").registered(
            operation,
            Revision::new(1),
            serde_json::json!({"columns":1}),
        ))
        .build()
}
#[test]
fn custom_catalog_roundtrip_and_native_rejection() {
    let registry = chart_extension_example::registry().unwrap();
    let p = author(
        registry.clone(),
        chart_extension_example::facet_planner::REVERSE,
    )
    .unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 84);
    assert!(Plot::from_json(&wire).is_err());
    let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
    let mut chart = p.chart().unwrap();
    let a = chart.prepare().unwrap();
    let mut other = restored.chart().unwrap();
    let b = other.prepare().unwrap();
    assert_eq!(
        a.definition().facets.as_ref().unwrap().layout,
        FacetLayout::Wrap { columns: 1 }
    );
    assert_eq!(
        a.panels()
            .iter()
            .map(|p| (p.row, p.column))
            .collect::<Vec<_>>(),
        vec![(0, 0), (1, 0), (2, 0)]
    );
    let keys = a.panels().iter().map(|p| p.key.clone()).collect::<Vec<_>>();
    assert_eq!(
        keys,
        b.panels().iter().map(|p| p.key.clone()).collect::<Vec<_>>()
    );
    assert_eq!(
        keys.iter().map(|k| k.values[0].clone()).collect::<Vec<_>>(),
        vec![
            GroupValue::Text("c".into()),
            GroupValue::Text("b".into()),
            GroupValue::Text("a".into())
        ]
    );
    assert!(
        author(
            registry,
            chart_extension_example::facet_planner::NATIVE_REVERSE
        )
        .unwrap()
        .to_json()
        .is_err()
    );
}
struct Bad;
impl CustomFacet for Bad {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.bad_facet", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn plan(&self, input: FacetPlanInput<'_>) -> ChartResult<FacetSpec> {
        let mut plan = input.base.clone();
        plan.order.push(plan.order[0].clone());
        Ok(plan)
    }
}
#[test]
fn duplicate_keys_and_unknown_registrations_reject() {
    let mut registry = ExtensionRegistry::new();
    registry.register_facet(Arc::new(Bad)).unwrap();
    assert!(registry.register_facet(Arc::new(Bad)).is_err());
    assert!(author(Arc::new(registry), "example.bad_facet").is_err());
    assert!(author(Arc::new(ExtensionRegistry::new()), "missing").is_err());
}
