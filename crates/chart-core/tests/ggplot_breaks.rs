//! FIX-GG04 pinned R break-selection values; exact candidates, finite numeric tolerance.
#[test]
fn ggplot_extended_breaks_match_labeling_reference() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/breaks.json")).unwrap();
    let mut cases = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let domain = [
            case["domain"][0].as_f64().unwrap(),
            case["domain"][1].as_f64().unwrap(),
        ];
        let n = case["count"].as_f64().unwrap();
        let got = if case["kind"] == "extended" {
            chart_core::scales::ggplot_breaks_extended(domain, n, 100)
        } else {
            chart_core::scales::ggplot_breaks_log(domain, n, case["base"].as_f64().unwrap(), 100)
        }
        .unwrap();
        let normalizer = chart_core::scales::ScaleNormalizer::new(
            chart_core::scales::NormalizationSpec::Ggplot {
                timestamp: None,
                family: if case["kind"] == "extended" {
                    chart_core::scales::NumericFamily::Linear
                } else {
                    chart_core::scales::NumericFamily::Log {
                        base: case["base"].as_f64().unwrap(),
                    }
                },
                domain: domain.map(chart_core::interpolate::Number),
                reverse: false,
                rescaler: chart_core::scales::GgplotRescaler::Range,
            },
        )
        .unwrap();
        assert_eq!(normalizer.ticks(n, 100).unwrap(), got);
        let want = case["breaks"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{case}");
        for (got, want) in got.iter().zip(want) {
            let want = want.as_f64().unwrap();
            assert!(
                (got - want).abs() <= 2e-12 * want.abs().max(1.),
                "{case}: {got} vs {want}"
            );
        }
        cases += 1;
    }
    assert_eq!(cases, 284);
    assert!(chart_core::scales::ggplot_breaks_extended([0., 1.], 0., 100).is_err());
    assert!(chart_core::scales::ggplot_breaks_extended([0., 1.], 1., 100).is_err());
    assert!(chart_core::scales::ggplot_breaks_extended([0., 1.], 101., 100).is_err());
}
