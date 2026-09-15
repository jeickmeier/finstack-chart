//! FIX-GG04: explicit public binned paint palettes use the resolved bin count.
use chart_core::{
    ChartResult, DiagnosticCode, grammar::*, interpolate::Value, prelude::*, scales::*,
};
use serde_json::{Value as Json, json};

#[test]
fn explicit_binned_palettes_match_108_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-constructor-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 108);
    for t in cases {
        let result = (|| -> ChartResult<_> {
            let inputs = t["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    inputs
                        .iter()
                        .map(|v| {
                            v.as_f64()
                                .or_else(|| v.as_str().map(|v| v.parse::<f64>().unwrap()))
                        })
                        .collect::<Vec<_>>(),
                )
                .build()?;
            let values = t["palette"].as_array().unwrap();
            let palette = if values.len() == 1 {
                GgplotDiscretePalette::Named(values[0].as_str().unwrap().into())
            } else {
                GgplotDiscretePalette::Values(
                    values
                        .iter()
                        .map(|v| {
                            Value::Color(
                                chart_core::color::parse_r(v.as_str().unwrap())
                                    .unwrap()
                                    .value(),
                            )
                        })
                        .collect(),
                )
            };
            let scale = MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(
                InterpolatedScaleSpec::sequential(NumericFamily::Linear),
            ))
            .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                palette: Some(GgplotBinnedPalette::Discrete(Box::new(palette))),
                breaks: GgplotBreaks::Nice(t["count"].as_f64().unwrap()),
                ..Default::default()
            })))?
            .with_guide(GgplotScaleGuide::Hidden)?;
            let mapping = if t["channel"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("v")
            } else {
                aes().x("x").y(1.).color("v").color_scale("v")
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .scale(color_mapped("v", scale))
                .layer(points())
                .build()?;
            let wire = p.to_json()?;
            let mut old: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(old["version"], 48);
            old["version"] = json!(47);
            assert_eq!(
                Plot::from_json(&old.to_string()).unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
            let restored = Plot::from_json(&wire)?;
            assert_eq!(restored.to_json()?, wire);
            restored.chart()?.prepare()
        })();
        if t["result"].get("error").is_some() {
            assert!(result.is_err(), "{t}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{e}: {t}"));
        let marks = prepared.layers()[0].marks();
        assert_eq!(
            marks.len(),
            t["result"]["point_count"].as_u64().unwrap() as usize,
            "{t}"
        );
        for (mark, expected) in marks
            .iter()
            .zip(t["result"]["point_colours"].as_array().unwrap())
        {
            let actual = if t["channel"] == "fill" {
                mark.style.fill.unwrap()
            } else {
                mark.style.color
            };
            assert_eq!(
                actual,
                chart_core::color::parse_r(expected.as_str().unwrap())
                    .unwrap()
                    .resolve(),
                "{t}"
            );
        }
    }
}
