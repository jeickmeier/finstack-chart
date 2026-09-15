//! FIX-GG04: every pinned palette name through both retained theme representations.
use chart_core::{
    ChartResult, DiagnosticCode, grammar::*, prelude::*, scales::*, theme::ThemeScalePalette,
};
use serde_json::{Value as Json, json};
#[test]
fn named_palettes_match_1260_reference_draws_in_both_wire_forms() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/named-theme-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1260);
    for t in cases {
        for scalar in [false, true] {
            let result = (|| -> ChartResult<_> {
                let discrete = t["family"] == "discrete";
                let inputs = t["inputs"].as_array().unwrap();
                let data = Data::columns()
                    .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>());
                let data = if discrete {
                    data.column(
                        "v",
                        inputs
                            .iter()
                            .map(|v| v.as_str().map(str::to_owned))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    data.column(
                        "v",
                        inputs
                            .iter()
                            .map(|v| {
                                v.as_f64()
                                    .or_else(|| v.as_str().map(|v| v.parse::<f64>().unwrap()))
                            })
                            .collect::<Vec<_>>(),
                    )
                }
                .build()?;
                let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(!discrete)? else {
                    unreachable!()
                };
                scale.missing_paint_is_na = true;
                scale = scale.with_guide(GgplotScaleGuide::Hidden)?;
                if t["family"] == "binned" {
                    scale = scale.with_ggplot(GgplotScalePolicy::Binned(Box::default()))?;
                }
                let palette: ThemeScalePalette = serde_json::from_value(if scalar {
                    t["palette"].clone()
                } else {
                    t["palette_values"].clone()
                })
                .unwrap();
                let p = plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                    .scale(color_mapped("v", scale))
                    .layer(points())
                    .theme(
                        theme().scale_palettes(
                            [(
                                format!(
                                    "palette.colour.{}",
                                    if discrete { "discrete" } else { "continuous" }
                                ),
                                palette,
                            )]
                            .into(),
                        ),
                    )
                    .build()?;
                let wire = p.to_json()?;
                let mut old: Json = serde_json::from_str(&wire).unwrap();
                assert_eq!(old["version"], 47);
                assert_eq!(old["definition"]["theme"]["version"], 5);
                old["version"] = json!(46);
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
            for (m, v) in marks
                .iter()
                .zip(t["result"]["point_colours"].as_array().unwrap())
            {
                assert_eq!(
                    m.style.color,
                    chart_core::color::parse_r(v.as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "{t}"
                );
            }
        }
    }
}
