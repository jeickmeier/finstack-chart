//! FIX-GG04: public discrete constructor defaults and explicit palette arguments.
use chart_core::{
    ChartResult,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::Value as Json;

#[test]
fn discrete_paint_constructors_match_80_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-paint-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 80);
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
            let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(false)? else {
                unreachable!()
            };
            scale.palette_theme_aesthetics.clear();
            let Some(GgplotScalePolicy::Discrete { palette, .. }) = scale.ggplot.as_deref_mut()
            else {
                unreachable!()
            };
            let custom = t["configuration"] == "custom";
            *palette = match t["constructor"].as_str().unwrap() {
                "hue" => GgplotDiscretePalette::Hue(if custom {
                    chromatic::ggplot::HuePalette {
                        h: [30., 300.],
                        chroma: 50.,
                        luminance: 80.,
                        start: 10.,
                        reverse: true,
                    }
                } else {
                    Default::default()
                }),
                "grey" => GgplotDiscretePalette::Grey {
                    start: if custom { 0.1 } else { 0.2 },
                    end: if custom { 0.9 } else { 0.8 },
                },
                "brewer" => GgplotDiscretePalette::Brewer {
                    id: if custom {
                        chromatic::SchemeId::RdBu
                    } else {
                        chromatic::SchemeId::Blues
                    },
                    reverse: custom,
                },
                "viridis_d" => GgplotDiscretePalette::Viridis {
                    option: if custom {
                        chromatic::ggplot::ViridisOption::Magma
                    } else {
                        chromatic::ggplot::ViridisOption::Viridis
                    },
                    begin: if custom { 0.2 } else { 0. },
                    end: if custom { 0.8 } else { 1. },
                    alpha: if custom { 0.5 } else { 1. },
                    reverse: custom,
                },
                _ => unreachable!(),
            };
            scale.missing_paint_is_na =
                matches!(t["constructor"].as_str(), Some("brewer" | "viridis_d"));
            let mut authored = color_mapped("v", scale.with_guide(GgplotScaleGuide::Hidden)?);
            if t["constructor"] == "grey" {
                authored = authored.missing(chart_core::color::parse_r("red")?);
            }
            let mapping = if t["channel"] == "fill" {
                aes().x("x").y(1.).fill("v").fill_scale("v")
            } else {
                aes().x("x").y(1.).color("v").color_scale("v")
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .scale(authored)
                .layer(points().aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.))))
                .build()?;
            let wire = p.to_json()?;
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
