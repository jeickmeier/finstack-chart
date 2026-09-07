//! Binding session lifetime and publication consistency beyond host wrapper compilation.
use chart_core::{
    DiagnosticCode,
    portable::{decode, encode},
};
use chart_export::{
    Format,
    portable::{PortableChart, ProfileEnvelope},
};
const CHART: &str = include_str!("../../../fixtures/bindings/chart.json");
const DATA: &str = include_str!("../../../fixtures/bindings/data.json");
const PROFILE: &str = include_str!("../../../fixtures/bindings/profile.json");
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
#[test]
fn captured_output_owns_resources_across_updates_and_disposal() {
    let mut chart = PortableChart::new(CHART, DATA, PROFILE, FONT.to_vec()).unwrap();
    let retained = chart.capture().unwrap();
    let weak = std::sync::Arc::downgrade(retained.layout());
    let original = retained.export(Format::Svg).unwrap().bytes;
    chart
        .transaction(include_str!("../../../fixtures/bindings/correction.json"))
        .unwrap();
    assert_ne!(chart.export("svg").unwrap(), original);
    chart.dispose();
    chart.dispose();
    assert_eq!(
        chart.semantics().unwrap_err().code,
        DiagnosticCode::DisposedHandle
    );
    assert_eq!(retained.export(Format::Svg).unwrap().bytes, original);
    drop(retained);
    assert!(weak.upgrade().is_none());
}
#[test]
fn binding_profile_is_explicit_and_never_falls_back_on_native_resources() {
    let profile: ProfileEnvelope = decode(PROFILE).unwrap();
    let canonical = encode(&profile).unwrap();
    assert!(PortableChart::new(CHART, DATA, &canonical, FONT.to_vec()).is_ok());
    let mut profile = profile;
    profile.version = 99;
    assert_eq!(
        PortableChart::new(CHART, DATA, &encode(&profile).unwrap(), FONT.to_vec())
            .err()
            .unwrap()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    let invalid = PROFILE.replace("\"Font\"", "\"NativeWidget\"");
    assert_eq!(
        PortableChart::new(CHART, DATA, &invalid, FONT.to_vec())
            .err()
            .unwrap()
            .code,
        DiagnosticCode::Validation
    );
    assert!(PortableChart::new(CHART, DATA, PROFILE, vec![]).is_err());
}

#[test]
fn publication_scene_targets_align_with_background_and_mark_items() {
    let chart = PortableChart::new(CHART, DATA, PROFILE, FONT.to_vec()).unwrap();
    let scene: serde_json::Value = serde_json::from_str(&chart.scene().unwrap()).unwrap();
    let items = scene["items"].as_array().unwrap();
    let targets = scene["targets"].as_array().unwrap();
    assert_eq!(items.len(), targets.len());
    assert_eq!(targets[0], serde_json::json!([]));
    assert!(items[0]["layer"].is_null());
    for (item, targets) in items.iter().zip(targets) {
        if item["layer"].is_null() {
            assert_eq!(targets, &serde_json::json!([]));
        } else {
            assert!(!targets.as_array().unwrap().is_empty());
        }
    }
    assert_eq!(items[1]["layer"], "9007199254742001");
    assert_eq!(
        targets[1][0]["Aggregate"]["members"],
        serde_json::json!(["9007199254743001", "9007199254743002"])
    );
}
