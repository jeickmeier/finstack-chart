//! FIX-GG04: numeric constructor argument forwarding against actual reference draws.
use chart_core::{
    grammar::*,
    interpolate::{InterpolationSpec, Number},
    prelude::*,
    scales::*,
};
use serde_json::Value as Json;

#[test]
fn numeric_constructors_match_180_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/numeric-paint-constructors.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 180);
    for t in cases {
        let channel = t["channel"].as_str().unwrap();
        let constructor = t["constructor"].as_str().unwrap();
        let inputs = t["inputs"].as_array().unwrap();
        let kind = match constructor {
            "size_area" | "size_binned_area" => GgplotNumericPalette::Area,
            "radius" => GgplotNumericPalette::Radius,
            _ => match channel {
                "size" => GgplotNumericPalette::Size,
                "alpha" => GgplotNumericPalette::Alpha,
                _ => GgplotNumericPalette::Linewidth,
            },
        };
        let mut scale = ggplot_numeric_default(kind).unwrap();
        if t["configuration"] != "default" {
            let ScaleFunctionSpec::Interpolated(s) = &mut scale.function else {
                unreachable!()
            };
            let ScaleRangeFunction::Interpolate(InterpolationSpec::PowerRange { range, .. }) =
                &mut s.output
            else {
                unreachable!()
            };
            *range = if constructor.ends_with("_area") {
                [Number(0.), Number(t["args"]["max_size"].as_f64().unwrap())]
            } else {
                [
                    Number(t["args"]["range"][0].as_f64().unwrap()),
                    Number(t["args"]["range"][1].as_f64().unwrap()),
                ]
            };
            scale = scale.with_theme_palette(vec![]).unwrap();
        }
        if constructor.contains("_binned") {
            scale = scale
                .with_ggplot(GgplotScalePolicy::Binned(Box::default()))
                .unwrap();
        }
        scale = scale.with_guide(GgplotScaleGuide::Hidden).unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "end",
                (0..inputs.len())
                    .map(|i| i as f64 + 0.4)
                    .collect::<Vec<_>>(),
            )
            .column("v", inputs.iter().map(Json::as_f64).collect::<Vec<_>>())
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(if channel == "linewidth" {
                aes().x("x").y(1.).x2("end").y2(1.)
            } else {
                aes().x("x").y(1.)
            })
            .layer(
                (if channel == "linewidth" {
                    rule()
                } else {
                    points()
                })
                .numeric_scale(
                    match channel {
                        "size" => NumericAesthetic::Size,
                        "alpha" => NumericAesthetic::Alpha,
                        _ => NumericAesthetic::StrokeWidth,
                    },
                    "v",
                    scale,
                ),
            )
            .build()
            .unwrap();
        let wire = p.to_json().unwrap();
        let restored = Plot::from_json(&wire).unwrap();
        let result = restored.chart().and_then(|mut c| c.prepare());
        if t["result"].get("error").is_some() {
            assert!(result.is_err(), "{t}");
            continue;
        }
        let prepared = result.unwrap_or_else(|e| panic!("{t}: {e}"));
        let marks = prepared.layers()[0].marks();
        let expected = t["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| channel == "alpha" || !v.is_null())
            .collect::<Vec<_>>();
        assert_eq!(marks.len(), expected.len(), "{t}");
        assert_eq!(
            marks.len(),
            t["result"]["mark_count"].as_u64().unwrap() as usize,
            "{t}"
        );
        for (i, (mark, expected)) in marks.iter().zip(expected).enumerate() {
            if channel == "alpha" {
                let color =
                    chart_core::color::parse_r(t["result"]["mark_colours"][i].as_str().unwrap())
                        .unwrap()
                        .resolve();
                assert_eq!(mark.style.color, color, "{t}");
            } else {
                let actual = if channel == "size" {
                    mark.style.radius
                } else {
                    mark.style.stroke_width
                };
                assert!(
                    (actual - expected.as_f64().unwrap()).abs() < 2e-12,
                    "{t}: {actual}"
                );
            }
        }
        assert_eq!(p.to_json().unwrap(), wire);
    }
}
