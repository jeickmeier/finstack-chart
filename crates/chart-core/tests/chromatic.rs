//! CP-02/03 exact independently generated catalog and ramp comparisons.
use chart_core::{interpolate::Number, scales::chromatic::*};
fn corpus() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale-chromatic/cases.json"
    ))
    .unwrap()
}
fn rgba(c: chart_core::scene::Color) -> [u8; 4] {
    [c.red, c.green, c.blue, c.alpha]
}
#[test]
fn every_actual_scheme_array_is_exact_and_owned() {
    let c = corpus();
    let rows = c["schemes"].as_array().unwrap();
    assert_eq!(rows.len(), 218);
    for row in rows {
        let id: SchemeId = row["name"].as_str().unwrap().parse().unwrap();
        let size = row["size"].as_u64().map(|n| n as usize);
        let result = scheme(id, size, false).unwrap();
        assert_eq!(
            serde_json::to_value(result.iter().copied().map(rgba).collect::<Vec<_>>()).unwrap(),
            row["rgba"],
            "{} {:?}",
            id.name(),
            size
        );
        assert_eq!(
            scheme(id, size, true).unwrap(),
            result.iter().copied().rev().collect::<Vec<_>>()
        );
        let mut copy = result.clone();
        copy[0].red ^= 255;
        assert_eq!(scheme(id, size, false).unwrap(), result);
        for k in [0, 1, 2, 10, 12, 256] {
            if !id.info().sizes.contains(&k) || id.info().family == SchemeFamily::Categorical {
                assert!(scheme(id, Some(k), false).is_err());
            }
        }
    }
    assert!("Viridis".parse::<SchemeId>().is_err());
    assert!("bogus".parse::<InterpolatorId>().is_err());
    assert_eq!(SchemeId::ALL.len(), 38);
    assert_eq!(InterpolatorId::ALL.len(), 38);
}
#[test]
fn every_interpolator_sample_matches_exact_reference_bytes() {
    let c = corpus();
    let mut n = 0;
    let mut failures = vec![];
    for row in c["ramps"].as_array().unwrap() {
        let id: InterpolatorId = row["name"].as_str().unwrap().parse().unwrap();
        let ramp = ChromaticRamp::new(ChromaticSpec { id, reverse: false }).unwrap();
        let reverse = ChromaticRamp::new(ChromaticSpec { id, reverse: true }).unwrap();
        for sample in row["samples"].as_array().unwrap() {
            n += 1;
            let t: Number = serde_json::from_value(sample[0].clone()).unwrap();
            if !t.0.is_finite() {
                assert!(ramp.evaluate(t.0).is_err());
                continue;
            }
            if sample[2].is_null() {
                assert!(
                    ramp.evaluate(t.0).is_err(),
                    "invalid reference {} {t:?}",
                    id.name()
                );
                continue;
            }
            let actual = serde_json::to_value(rgba(ramp.evaluate(t.0).unwrap())).unwrap();
            if actual != sample[2] && failures.len() < 40 {
                failures.push(format!(
                    "{} t={:.17e}: {actual} != {}",
                    id.name(),
                    t.0,
                    sample[2]
                ));
            }
            assert_eq!(
                reverse.evaluate(1. - t.0).unwrap(),
                ramp.evaluate(1. - (1. - t.0)).unwrap()
            );
        }
    }
    assert_eq!(n, 160666);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
