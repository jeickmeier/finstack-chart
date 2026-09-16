//! FIX-GG06 independent Rust authors from pinned bin input/control cases.
use chart_core::{
    grammar::{BinClosure, GgplotBinOptions, Profile},
    prelude::*,
};
pub fn cases() -> Vec<serde_json::Value> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/parity/ggplot2/bin-stat-controls.json"
    ))
    .expect("committed bin fixture");
    fixture["cases"].as_array().expect("cases").clone()
}
pub fn selected(t: &serde_json::Value) -> bool {
    t["closed"] == "right"
        && (t["population"] == "ordinary" && t["pad"] == false
            || t["mode"] == "explicit"
                && (t["population"] == "signed" || t["population"] == "zero")
                && t["pad"] == true)
        || t["closed"] == "left"
            && t["population"] == "boundary"
            && t["mode"] == "explicit"
            && t["pad"] == true
}
pub fn name(t: &serde_json::Value) -> String {
    format!(
        "{} / {} / {}{}",
        t["population"].as_str().unwrap(),
        t["mode"].as_str().unwrap(),
        t["closed"].as_str().unwrap(),
        if t["pad"] == true { " / padded" } else { "" }
    )
}
pub fn author(t: &serde_json::Value) -> ChartResult<Plot> {
    let values = |field: &str| {
        t[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect::<Vec<_>>()
    };
    let data = Data::columns()
        .column("x", values("input"))
        .column("w", values("weight"))
        .build()?;
    let mode = t["mode"].as_str().unwrap();
    let options = GgplotBinOptions {
        closed: if t["closed"] == "right" {
            BinClosure::Right
        } else {
            BinClosure::Left
        },
        pad: t["pad"].as_bool().unwrap(),
        binwidth: (!matches!(mode, "explicit" | "bins")).then_some(0.75),
        center: (mode == "center").then_some(0.25),
        boundary: (mode == "boundary").then_some(0.25),
        ..Default::default()
    };
    let mut stat = bin().bins(3).ggplot_bin(options).bin_weight("w");
    if mode == "explicit" {
        stat = stat.breaks(vec![0., 1., 2., 4.]);
    }
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram().stat(stat))
        .build()
}
