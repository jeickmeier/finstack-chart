//! FIX-GG04: count palettes reuse trained bin classification and discrete generators.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
fn spec(case: &serde_json::Value) -> chart_core::ChartResult<MappedScaleSpec> {
    let palette = match case["palette"].as_str().unwrap() {
        "solid" => GgplotBinnedPalette::Shape { solid: true },
        "hollow" => GgplotBinnedPalette::Shape { solid: false },
        "linetype" => GgplotBinnedPalette::LineType,
        "brewer" => GgplotBinnedPalette::Brewer {
            id: chromatic::SchemeId::Blues,
            reverse: false,
        },
        _ => unreachable!(),
    };
    let control = case["control"].as_str().unwrap();
    let policy = GgplotBinnedPolicy {
        palette: Some(palette),
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
    MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Ggplot {
            timestamp: None,
            family: NumericFamily::Linear,
            domain: [Number(0.), Number(1.)],
            reverse: false,
            rescaler: GgplotRescaler::Range,
        },
        output: ScaleRangeFunction::Identity,
        unknown: if case["palette"] == "brewer" {
            Value::Color(
                chart_core::color::Paint::from_css("#7F7F7F")
                    .unwrap()
                    .value(),
            )
        } else {
            Value::Missing
        },
    }))
    .with_ggplot(GgplotScalePolicy::Binned(Box::new(policy)))?
    .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))
}
#[test]
fn count_palettes_match_216_reference_builds() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-style-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 216);
    for case in cases {
        let inputs: Vec<_> = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().map(Number))
            .collect();
        let result = spec(case).and_then(|s| s.trained(&inputs)).and_then(|s| {
            let json = serde_json::to_string(&s).unwrap();
            let scale = MappedScale::new(serde_json::from_str(&json).unwrap())?;
            for input in &inputs {
                scale.numeric(input.map(|v| v.0))?;
            }
            scale.binned_guide_entries(4096, 65536)?;
            Ok(scale)
        });
        let expected = &case["result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let scale = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
        for (input, want) in inputs.iter().zip(expected["values"].as_array().unwrap()) {
            let got = scale.numeric(input.map(|v| v.0)).unwrap();
            let want = if want.is_null() {
                Value::Missing
            } else if let Some(v) = want.as_f64() {
                Value::Number(Number(v))
            } else if case["palette"] == "brewer" {
                Value::Color(
                    chart_core::color::Paint::from_css(if want == "grey50" {
                        "#7F7F7F"
                    } else {
                        want.as_str().unwrap()
                    })
                    .unwrap()
                    .value(),
                )
            } else {
                Value::Text(want.as_str().unwrap().into())
            };
            assert_eq!(got, want, "{case} input {input:?}");
        }
        let guide = scale.binned_guide_entries(4096, 65536);
        if expected["guide"].get("error").is_some() {
            assert!(guide.is_err(), "{case}");
            continue;
        }
        let entries = guide.unwrap().unwrap();
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
}
