//! FIX-GG04: default numeric labels compared as whole vectors to pinned R.
#[test]
fn default_numeric_labels_match_r_vectors() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/labels.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 499);
    for case in cases {
        let values: Vec<_> = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let labels = chart_core::typography::ggplot_numeric_labels(&values, 100_000).unwrap();
        let expected: Vec<_> = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(labels, expected, "{case}");
    }
    assert!(chart_core::typography::ggplot_numeric_labels(&[1., 2.], 1).is_err());
    assert!(chart_core::typography::ggplot_numeric_labels(&[f64::NAN], 100).is_err());
}
