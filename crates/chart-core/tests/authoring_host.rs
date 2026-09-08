//! FIX-AUTH07: the shared host adapter immediately uses the canonical typed builders.
use chart_core::{
    grammar::PreparedRows,
    plot::host::{Component, Draft},
    prelude::*,
};
#[test]
fn host_composition_matches_independent_bins_and_rejects_misplaced_components() {
    let data = Data::columns().column("x", [0., 1., 2.]).build().unwrap();
    let mappings = Component::new("aes", "[]")
        .unwrap()
        .set("x", r#"["x"]"#)
        .unwrap();
    let histogram = Component::new("histogram", "[]")
        .unwrap()
        .set("breaks", "[[0,1,2]]")
        .unwrap()
        .set("name", r#"["counts"]"#)
        .unwrap();
    let title = Component::new("title", r#"["Distribution"]"#).unwrap();
    let draft = Draft::new(&data)
        .with("aes", &mappings)
        .unwrap()
        .with("layer", &histogram)
        .unwrap()
        .with("title", &title)
        .unwrap();
    let plot = draft.build().unwrap();
    assert_eq!(
        plot.layer("counts").unwrap(),
        histogram.layer_handle().unwrap()
    );
    let prepared = plot.chart().unwrap().prepare().unwrap();
    let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
        panic!("bins")
    };
    assert_eq!(rows.iter().map(|r| r.count).collect::<Vec<_>>(), [1, 2]);
    assert!(draft.with("layer", &title).is_err());
    assert!(mappings.set("typo", "[1]").is_err());
    assert!(histogram.set("size", "[1,2]").is_err());
    assert!(
        histogram
            .with("aes", &Component::new("stat_aes", "[]").unwrap())
            .is_err()
    );
    let edited = Draft::edit(&plot)
        .with("title", &Component::new("title", r#"["Changed"]"#).unwrap())
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(
        edited.layer("counts").unwrap(),
        plot.layer("counts").unwrap()
    );
    assert!(edited.definition().revision > plot.definition().revision);
}
#[test]
fn primary_envelope_roundtrip_preserves_exact_data_names_and_rejects_malformed_registry() {
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [2., 3.])
        .column("exact", [9_007_199_254_740_993_u64, u64::MAX])
        .keys([u64::MAX - 1, u64::MAX])
        .build()
        .unwrap();
    let original = plot(data.clone())
        .aes(aes().x("x").y("y"))
        .layer(points().name("dots"))
        .x_axis(x_axis().label("X"))
        .title(title("Original"))
        .build()
        .unwrap();
    let json = original.to_json().unwrap();
    let loaded = Plot::from_json(&json).unwrap();
    assert_eq!(loaded.to_json().unwrap(), json);
    assert_eq!(
        loaded.data("data").unwrap().batch().keys(),
        data.batch().keys()
    );
    assert_eq!(
        loaded.data("data").unwrap().batch().columns(),
        data.batch().columns()
    );
    assert_eq!(loaded.definition(), original.definition());
    assert_eq!(
        loaded.layer("dots").unwrap(),
        original.layer("dots").unwrap()
    );
    let mut runtime = loaded.chart().unwrap();
    assert!(
        runtime
            .apply_plot(&original, runtime.definition().revision)
            .is_err()
    );
    let mut malformed: serde_json::Value = serde_json::from_str(&json).unwrap();
    malformed["layers"]["dots"] = serde_json::json!("999999");
    assert!(Plot::from_json(&malformed.to_string()).is_err());
    malformed = serde_json::from_str(&json).unwrap();
    malformed["version"] = serde_json::json!(2);
    assert!(Plot::from_json(&malformed.to_string()).is_err());
}
