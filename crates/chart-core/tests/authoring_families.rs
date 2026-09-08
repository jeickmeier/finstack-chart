//! FIX-AUTH03: all delivered statistic/position/geometry/scale fixture families.
#[path = "../../../examples/common/authoring_fixtures.rs"]
#[allow(dead_code)]
mod authors;
mod support {
    pub mod authoring_families;
}
#[test]
fn primary_builders_preserve_independent_family_fixtures() {
    assert_eq!(support::authoring_families::cases().len(), 31);
}
#[test]
fn primary_composition_preserves_independent_publication_fixtures() {
    use serde_json::Value;
    let fixtures: Vec<Value> = serde_json::from_str(include_str!(
        "../../../fixtures/composition/portable-cases.json"
    ))
    .unwrap();
    let figure = &fixtures[0]["chart"]["definition"]["figure"];
    let bold = serde_json::from_value(figure["title"]["lines"][0][0]["font"].clone()).unwrap();
    let arabic =
        serde_json::from_value(figure["subtitle"]["lines"][0][1]["fallback"][0].clone()).unwrap();
    assert_eq!(
        support::authoring_families::composition_cases(bold, arabic).len(),
        3
    );
}
