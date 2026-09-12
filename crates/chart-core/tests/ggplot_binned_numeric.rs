//! FIX-GG04: built-in numeric palettes composed with the shared binned policy.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
#[test]
fn binned_numeric_maps_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-numeric-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 216);
    let mut compared = 0;
    for case in cases {
        // R size/linewidth wrappers have no `right` argument. Our explicit generic
        // binned policy corresponds to binned_scale, which does accept it.
        if case["control"] == "left"
            && (case["palette"] == "size" || case["palette"] == "linewidth")
        {
            continue;
        }
        compared += 1;
        let kind = match case["palette"].as_str().unwrap() {
            "size" => GgplotNumericPalette::Size,
            "area" => GgplotNumericPalette::Area,
            "alpha" => GgplotNumericPalette::Alpha,
            "linewidth" => GgplotNumericPalette::Linewidth,
            _ => unreachable!(),
        };
        let control = case["control"].as_str().unwrap();
        let policy = GgplotBinnedPolicy {
            breaks: match control {
                "explicit" => GgplotBreaks::Explicit([1., 3., 5., 7., 9.].map(Number).to_vec()),
                "empty_breaks" | "null_breaks" => GgplotBreaks::Explicit(vec![]),
                "count_eight" => GgplotBreaks::Equal(8.),
                "count_sixteen" => GgplotBreaks::Equal(16.),
                _ => GgplotBreaks::Nice(5.),
            },
            limits: match control {
                "limits" => Some([Some(Number(-1.)), Some(Number(5.))]),
                "partial_limits" => Some([None, Some(Number(5.))]),
                _ => None,
            },
            right: control != "left",
            ..Default::default()
        };
        let inputs: Vec<_> = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().map(Number))
            .collect();
        let result = (|| -> chart_core::ChartResult<_> {
            let spec = ggplot_numeric_default(kind)?
                .with_ggplot(GgplotScalePolicy::Binned(Box::new(policy)))?
                .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))?
                .trained(&inputs)?;
            let json = serde_json::to_string(&spec).unwrap();
            let scale = MappedScale::for_numbers(serde_json::from_str(&json).unwrap())?;
            let values = inputs
                .iter()
                .map(|input| scale.numeric(input.map(|v| v.0)))
                .collect::<chart_core::ChartResult<Vec<_>>>()?;
            let guide = scale.binned_guide_entries(4096, 65536)?;
            Ok((values, guide))
        })();
        let expected = &case["raw_result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let (values, guide) = result.unwrap_or_else(|error| panic!("{error:?}: {case}"));
        let wanted = expected["values"].as_array().unwrap();
        assert!(wanted.len() == values.len() || wanted.len() == 1, "{case}");
        for (i, got) in values.iter().enumerate() {
            let want = &wanted[if wanted.len() == 1 { 0 } else { i }];
            if want.is_null() {
                assert_eq!(*got, Value::Missing, "{case}");
            } else {
                let Value::Number(got) = got else {
                    panic!("{got:?} {case}")
                };
                assert!(
                    (got.0 - want.as_f64().unwrap()).abs() < 2e-12,
                    "{got:?} {case}"
                );
            }
        }
        let entries = guide.unwrap();
        let cuts = expected["guide"]["breaks"].as_array().unwrap();
        assert_eq!(entries.len(), cuts.len(), "{case}");
        for (got, want) in entries.iter().zip(cuts) {
            assert!(
                (got.transformed.0 - want.as_f64().unwrap()).abs() < 2e-12,
                "{case}"
            );
        }
        assert_eq!(
            entries
                .iter()
                .map(|e| e.label.as_deref())
                .collect::<Vec<_>>(),
            expected["guide"]["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            "{case}"
        );
    }
    assert_eq!(compared, 204);
}
