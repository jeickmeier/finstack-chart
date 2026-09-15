//! Independent pinned GG-04 palette contracts.
use chart_core::{color, scales::chromatic::ggplot};
use serde_json::Value;
fn expected(value: &Value) -> Option<chart_core::scene::Color> {
    value
        .as_str()
        .map(|s| color::Paint::from_css(s).unwrap().resolve())
}
#[test]
fn ggplot_palettes_match_pinned_scales_reference() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/palettes.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let a = &case["args"];
        let n = a["n"].as_u64().unwrap_or(0) as usize;
        let result = match case["kind"].as_str().unwrap() {
            "hue" => {
                let p = ggplot::HuePalette {
                    h: [a["h"][0].as_f64().unwrap(), a["h"][1].as_f64().unwrap()],
                    chroma: a["c"].as_f64().unwrap(),
                    luminance: a["l"].as_f64().unwrap(),
                    start: a["h_start"].as_f64().unwrap(),
                    reverse: a["direction"] == -1,
                };
                if n == 0 {
                    assert!(p.colors(n).is_err());
                    count += 1;
                    continue;
                }
                p.colors(n)
                    .unwrap()
                    .into_iter()
                    .map(Some)
                    .collect::<Vec<_>>()
            }
            "grey" => ggplot::grey(n, a["start"].as_f64().unwrap(), a["end"].as_f64().unwrap())
                .unwrap()
                .into_iter()
                .map(Some)
                .collect(),
            "brewer" => ggplot::brewer(
                a["name"].as_str().unwrap().parse().unwrap(),
                n,
                a["direction"] == -1,
            )
            .unwrap(),
            "gradient" => {
                let colors = a["colours"]
                    .as_array()
                    .cloned()
                    .unwrap_or_else(|| vec![a["colours"].clone()])
                    .iter()
                    .map(|v| color::Paint::from_css(v.as_str().unwrap()).unwrap().value())
                    .collect::<Vec<_>>();
                let stops = a["values"]
                    .as_array()
                    .map(|v| v.iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
                let gradient = ggplot::Gradient::new(&colors, stops.as_deref()).unwrap();
                a["t"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        gradient.sample(v.as_f64().unwrap_or_else(|| match v["number"].as_str() {
                            Some("Infinity") => f64::INFINITY,
                            Some("-Infinity") => f64::NEG_INFINITY,
                            _ => f64::NAN,
                        }))
                    })
                    .collect()
            }
            "viridis" => {
                ggplot::ViridisPalette::new(serde_json::from_value(a["option"].clone()).unwrap())
                    .unwrap()
                    .colors(
                        n,
                        a["begin"].as_f64().unwrap(),
                        a["end"].as_f64().unwrap(),
                        a["direction"] == -1,
                        a["alpha"].as_f64().unwrap(),
                    )
                    .unwrap()
                    .into_iter()
                    .map(Some)
                    .collect()
            }
            other => panic!("unknown fixture {other}"),
        };
        let want = case["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(expected)
            .collect::<Vec<_>>();
        assert_eq!(result.len(), want.len(), "{}", case["id"]);
        for (i, (got, want)) in result.iter().zip(&want).enumerate() {
            assert_eq!(got, want, "{} sample {i}", case["id"]);
        }
        count += 1;
    }
    assert_eq!(count, 766);
}

#[test]
fn ggplot_palette_uses_shared_interpolator_and_missing_color_policy() {
    use chart_core::{
        interpolate::{InterpolationSpec, Interpolator, Number, Value},
        scales::*,
    };
    let spec = InterpolationSpec::GgplotPalette {
        spec: ggplot::PaletteSpec::Gradient {
            colors: ["red", "white", "blue"]
                .map(|s| color::Paint::from_css(s).unwrap())
                .to_vec(),
            values: Some(vec![0.0.into(), 0.2.into(), 1.0.into()]),
        },
    };
    let operation = Interpolator::new(spec.clone()).unwrap();
    let wire = operation.to_json().unwrap();
    assert!(wire.contains("\"version\":3"));
    assert_eq!(
        Interpolator::from_json(&wire).unwrap().sample(0.2).unwrap(),
        operation.sample(0.2).unwrap()
    );
    assert!(Interpolator::from_json(&wire.replace("\"version\":3", "\"version\":1")).is_err());
    let mapped =
        MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
            normalization: NormalizationSpec::Sequential {
                family: NumericFamily::Linear,
                domain: [Number(10.), Number(20.)],
                clamp: false,
            },
            output: ScaleRangeFunction::Interpolate(spec),
            unknown: Value::Missing,
        }));
    assert!(mapped.has_ggplot());
    let scale = MappedScale::for_colors(mapped).unwrap();
    let missing = color::Paint::from_css("#abcdef").unwrap().resolve();
    assert_eq!(
        scale.color(Some(12.), None, missing).unwrap(),
        color::Paint::from_css("white").unwrap().resolve()
    );
    for input in [
        None,
        Some(9.),
        Some(21.),
        Some(f64::NAN),
        Some(f64::INFINITY),
    ] {
        assert_eq!(scale.color(input, None, missing).unwrap(), missing);
    }
}
