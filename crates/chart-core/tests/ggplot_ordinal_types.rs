//! FIX-GG04: public ordinal type vectors interpolate by category count.
use chart_core::{
    ChartResult, DiagnosticCode,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};

#[test]
fn ordinal_type_vectors_match_70_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/ordinal-type-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 70);
    for t in cases {
        let result = (|| -> ChartResult<_> {
            let inputs = t["inputs"].as_array().unwrap();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "v",
                    inputs
                        .iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Vec<_>>(),
                )
                .build()?;
            let ColorScale::Mapped { mut scale, .. } = ggplot_color_ordinal()? else {
                unreachable!()
            };
            let Some(GgplotScalePolicy::Discrete { palette, .. }) = scale.ggplot.as_deref_mut()
            else {
                unreachable!()
            };
            *palette = GgplotDiscretePalette::OrdinalColors(
                t["palette"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect(),
            );
            let mapping = if t["channel"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("v")
            } else {
                aes().x("x").y(1.).color("v").color_scale("v")
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .scale(color_mapped(
                    "v",
                    scale.with_guide(GgplotScaleGuide::Hidden)?,
                ))
                .layer(points().aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.))))
                .build()?;
            let wire = p.to_json()?;
            let mut downgraded: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(downgraded["version"], 49);
            downgraded["version"] = json!(48);
            assert_eq!(
                Plot::from_json(&downgraded.to_string()).unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
            let restored = Plot::from_json(&wire)?;
            let prepared = restored.chart()?.prepare()?;
            assert_eq!(p.to_json()?, wire);
            Ok(prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    if t["channel"] == "fill" {
                        m.style.fill
                    } else {
                        Some(m.style.color)
                    }
                })
                .collect::<Vec<_>>())
        })();
        if t["result"].get("error").is_some() {
            assert!(result.is_err(), "{t}: {result:?}");
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{t}: {e}"));
        let expected = t["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| t["channel"] == "fill" || !v.is_null())
            .map(|v| {
                Some(v.as_str().map_or(
                    chart_core::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    },
                    |v| chart_core::color::parse_r(v).unwrap().resolve(),
                ))
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{t}");
        assert_eq!(
            actual.len(),
            t["result"]["point_count"].as_u64().unwrap() as usize
        );
    }
}
