//! FIX-S01: independently generated shape contexts consume the accepted path engine.
use chart_core::{
    ChartResult,
    path::{Command, PathLimits, PathRequest, PathSink, Precision},
};

#[derive(Default)]
struct ExternalSink(Vec<Command>);
impl PathSink for ExternalSink {
    fn command(&mut self, command: &Command) -> ChartResult<()> {
        self.0.push(*command);
        Ok(())
    }
}

#[test]
fn every_reference_shape_context_uses_one_checked_path_engine() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/cases.json")).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 333);
    for case in cases {
        let request = PathRequest {
            version: 1,
            digits: None,
            limits: PathLimits::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        };
        let path = request.clone().build().unwrap();
        let retained = path.geometry();
        let mut external = ExternalSink::default();
        path.replay(&mut external).unwrap();
        assert_eq!(external.0.as_slice(), retained.commands(), "{}", case["id"]);
        for digits in [0, 3, 12] {
            let expected = case["svg_digits"][digits.to_string()]
                .as_str()
                .unwrap_or("");
            assert_eq!(
                retained
                    .to_svg(Precision::Digits(digits), 1_000_000)
                    .unwrap(),
                expected,
                "{} digits={digits}",
                case["id"]
            );
        }
        assert_eq!(request.build().unwrap().geometry(), retained);
        if !external.0.is_empty() {
            let mut bounded = ExternalSink::default();
            assert!(retained.replay(&mut bounded, external.0.len() - 1).is_err());
            assert!(bounded.0.is_empty());
        }
    }
}

#[test]
fn shape_inventory_accounts_for_methods_aliases_and_protocols() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/inventory.json")).unwrap();
    let exports = inventory["exports"].as_array().unwrap();
    assert_eq!(exports.len(), 63);
    assert_eq!(exports.iter().filter(|v| v["kind"] == "curve").count(), 20);
    assert_eq!(
        exports.iter().filter(|v| !v["alias_of"].is_null()).count(),
        4
    );
    for name in ["lineRadial", "areaRadial"] {
        let entry = exports.iter().find(|v| v["name"] == name).unwrap();
        assert!(
            entry["methods"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("digits"))
        );
    }
    for entry in exports.iter().filter(|v| v["kind"] == "curve") {
        for method in ["lineStart", "lineEnd", "point"] {
            assert!(
                entry["methods"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(method))
            );
        }
    }
}
