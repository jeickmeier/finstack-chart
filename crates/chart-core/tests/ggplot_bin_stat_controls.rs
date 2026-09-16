//! FIX-GG06: source-backed weighted/fuzzy bins, alignment and generated normalization.
use chart_core::{
    grammar::{BinClosure, GgplotBinOptions, PreparedRows},
    prelude::*,
};
use serde_json::Value;

#[test]
fn closure_is_profile_owned_and_legacy_bins_keep_their_contract() {
    for (profile, expected) in [
        (Profile::LibraryV1, [1, 2]),
        (Profile::Ggplot2_4_0_3, [2, 1]),
    ] {
        let p = plot(Data::columns().column("x", [0., 1., 2.]).build().unwrap())
            .profile(profile)
            .aes(aes().x("x"))
            .layer(histogram().breaks(vec![0., 1., 2.]))
            .build()
            .unwrap();
        let chart = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Binned(rows) = chart.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(rows.iter().map(|r| r.count).collect::<Vec<_>>(), expected);
    }
}
#[test]
fn all_reference_bin_controls_match_generated_rows() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/bin-stat-controls.json"
    ))
    .unwrap();
    let close = |actual: f64, expected: &Value| {
        let expected = expected.as_f64().unwrap();
        assert!(
            (actual - expected).abs() < 3e-12 * expected.abs().max(1.),
            "{actual} != {expected}"
        );
    };
    for t in cases["cases"].as_array().unwrap() {
        let values = |key: &str| {
            t[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>()
        };
        let data = Data::columns()
            .column("x", values("input"))
            .column("w", values("weight"))
            .build()
            .unwrap();
        let mode = t["mode"].as_str().unwrap();
        let options = GgplotBinOptions {
            closed: if t["closed"] == "right" {
                BinClosure::Right
            } else {
                BinClosure::Left
            },
            pad: t["pad"].as_bool().unwrap(),
            binwidth: (mode != "explicit" && mode != "bins").then_some(0.75),
            center: (mode == "center").then_some(0.25),
            boundary: (mode == "boundary").then_some(0.25),
            ..Default::default()
        };
        let mut statistic = bin().bins(3).ggplot_bin(options).bin_weight("w");
        if mode == "explicit" {
            statistic = statistic.breaks(vec![0., 1., 2., 4.]);
        }
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x"))
            .layer(histogram().stat(statistic))
            .build()
            .unwrap();
        let wire = p.to_json().unwrap();
        let p = Plot::from_json(&wire).unwrap();
        assert_eq!(wire, p.to_json().unwrap());
        let chart = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Binned(rows) = chart.layers()[0].table().rows() else {
            panic!()
        };
        let expected = t["result"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len(), "{t}");
        for (r, e) in rows.iter().zip(expected) {
            close(r.start, &e["xmin"]);
            close(r.end, &e["xmax"]);
            let s = r.statistics.as_ref().unwrap();
            close(s.count, &e["count"]);
            for (actual, key) in [
                (s.density, "density"),
                (s.ncount, "ncount"),
                (s.ndensity, "ndensity"),
            ] {
                if e[key].is_null() {
                    assert!(actual.is_none(), "{t}");
                } else {
                    close(actual.unwrap(), &e[key]);
                }
            }
        }
    }
    assert_eq!(cases["cases"].as_array().unwrap().len(), 100);
}

#[test]
fn infinite_bin_weights_are_rejected_instead_of_counted_as_zero() {
    let p = plot(
        Data::columns()
            .column("x", [0.5])
            .column("w", [f64::INFINITY])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x"))
    .layer(
        histogram().stat(
            bin()
                .breaks(vec![0., 1.])
                .ggplot_bin(GgplotBinOptions::default())
                .bin_weight("w"),
        ),
    )
    .build()
    .unwrap();
    assert!(p.chart().unwrap().prepare().is_err());
}

#[test]
fn generated_bin_fields_are_versioned_and_checked_against_the_output_schema() {
    use chart_core::grammar::{BinField, Expression};
    for expr in [false, true] {
        let mapping = if expr {
            bin_aes().y(Expression::read(BinField::Density))
        } else {
            bin_aes().y(BinField::Density)
        };
        let p = plot(Data::columns().column("x", [0.5]).build().unwrap())
            .aes(aes().x("x"))
            .layer(histogram().breaks(vec![0., 1.]).after_bin(mapping))
            .build()
            .unwrap();
        assert_eq!(p.definition().wire_version(), 72);
        assert!(p.chart().unwrap().prepare().is_err());
    }
}

#[test]
fn automatic_bin_pretraining_keeps_original_validation() {
    let result = plot(Data::columns().column("x", [0., 1.]).build().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram().bins(0))
        .build();
    assert!(result.and_then(|p| p.chart()?.prepare()).is_err());
}
