//! GG-00 oracle integrity and independent seed values; not a ggplot2 parity claim.
use serde_json::Value;
fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parity/ggplot2")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn numbers(case: &Value, field: &str) -> Vec<f64> {
    case["layers"][0]["columns"][field]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect()
}
#[test]
fn complete_reference_inventory_retains_owners_and_independent_seed_expectations() {
    let inventory = fixture("inventory.json");
    assert_eq!(inventory["reference"], "ggplot2 4.0.3");
    let exports = inventory["exports"].as_array().unwrap();
    assert_eq!(exports.len(), 643);
    let mut ids = std::collections::BTreeSet::new();
    for row in exports {
        assert!(ids.insert(row["id"].as_str().unwrap()));
        let owner = row["owner"].as_str().unwrap();
        assert!(owner.starts_with("GG-"));
        let number: usize = owner[3..].parse().unwrap();
        assert!((2..=17).contains(&number));
        assert_eq!(
            row["status"], "OPEN",
            "An oracle row cannot certify its implementation"
        );
        assert!(!row["class"].is_null());
    }
    for (name, owner) in [
        ("StatSmooth", "GG-10"),
        ("GeomDensity2d", "GG-11"),
        ("CoordSf", "GG-15"),
        ("GeomSf", "GG-15"),
    ] {
        assert_eq!(
            exports.iter().find(|r| r["name"] == name).unwrap()["owner"],
            owner
        );
    }
    let corpus = fixture("cases.json");
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);
    let find = |id: &str| cases.iter().find(|c| c["id"] == id).unwrap();
    // These are externally specified counts/group identities, not values copied from Rust.
    assert_eq!(numbers(find("histogram_right"), "count"), [2., 1.]);
    assert_eq!(numbers(find("histogram_left"), "count"), [1., 2.]);
    assert_eq!(numbers(find("weighted_bins"), "count"), [3., 3.]);
    assert_eq!(numbers(find("inferred_groups"), "group"), [1., 1., 2.]);
    assert_eq!(numbers(find("explicit_all"), "group"), [1., 1., 1.]);
    for (column, expected) in [
        ("lower", [2.25, 8.5]),
        ("middle", [3.5, 10.5]),
        ("upper", [4.75, 11.75]),
        ("ymin", [1., 7.]),
        ("ymax", [8., 15.]),
    ] {
        assert_eq!(numbers(find("boxplot"), column), expected);
    }
    let ys = [1., 3., 2., 5., 4., 8., 7., 10., 8., 12., 11., 15.];
    let xm = 6.5;
    let ym = ys.iter().sum::<f64>() / 12.;
    let slope = ys
        .iter()
        .enumerate()
        .map(|(i, y)| ((i + 1) as f64 - xm) * (y - ym))
        .sum::<f64>()
        / (1..=12).map(|x| (x as f64 - xm).powi(2)).sum::<f64>();
    for (x, y) in numbers(find("lm"), "x")
        .iter()
        .zip(numbers(find("lm"), "y"))
    {
        assert!((y - (ym + slope * (x - xm))).abs() < 1e-12);
    }
    for case in cases {
        assert!(
            case["layers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|l| l["row_count"].as_u64().unwrap() > 0)
        );
        assert!(!case["geometry"]["kind"].is_null());
    }
}

#[test]
fn chromatic_reference_inventory_retains_every_export_size_and_sample() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale-chromatic/manifest.json"
    ))
    .unwrap();
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale-chromatic/cases.json"
    ))
    .unwrap();
    let exports = manifest["exports"].as_array().unwrap();
    assert_eq!(exports.len(), 76);
    let schemes = cases["schemes"].as_array().unwrap();
    assert_eq!(schemes.len(), 218);
    for row in schemes {
        let css = row["css"].as_array().unwrap();
        let rgba = row["rgba"].as_array().unwrap();
        assert_eq!(css.len(), rgba.len());
        if let Some(size) = row["size"].as_u64() {
            assert_eq!(size as usize, css.len());
        }
        assert!(
            rgba.iter()
                .all(|c| c.as_array().is_some_and(|v| v.len() == 4
                    && v.iter().all(|x| x.as_u64().is_some_and(|n| n <= 255))
                    && v[3] == 255))
        );
    }
    let ramps = cases["ramps"].as_array().unwrap();
    assert_eq!(ramps.len(), 38);
    let count: usize = ramps
        .iter()
        .map(|r| r["samples"].as_array().unwrap().len())
        .sum();
    assert_eq!(count, manifest["samples"].as_u64().unwrap() as usize);
    for export in exports {
        let name = export.as_str().unwrap();
        if let Some(id) = name.strip_prefix("scheme") {
            assert!(schemes.iter().any(|r| r["name"] == id));
        } else {
            let id = name.strip_prefix("interpolate").unwrap();
            assert!(ramps.iter().any(|r| r["name"] == id));
        }
    }
    // Independent ColorBrewer anchor: not a sampled maximum-size or endpoint ramp.
    let blues = schemes
        .iter()
        .find(|s| s["name"] == "Blues" && s["size"] == 3)
        .unwrap();
    assert_eq!(
        blues["css"],
        serde_json::json!(["#deebf7", "#9ecae1", "#3182bd"])
    );
}
