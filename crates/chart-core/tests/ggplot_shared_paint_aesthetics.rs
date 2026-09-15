//! FIX-GG04: multiple paint aesthetics train one declared scale identity.
use chart_core::{grammar::*, interpolate::Value, prelude::*, scales::*};
use serde_json::{Value as Json, json};

#[test]
fn joint_colour_fill_training_matches_fifteen_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/shared-paint-aesthetics.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 15);
    for t in cases {
        let family = t["family"].as_str().unwrap();
        let numeric = matches!(family, "continuous" | "binned");
        let color = t["colour"].as_array().unwrap();
        let fill = t["fill"].as_array().unwrap();
        let builder =
            Data::columns().column("x", (0..color.len()).map(|i| i as f64).collect::<Vec<_>>());
        let data = if numeric {
            builder
                .column("colour", color.iter().map(Json::as_f64).collect::<Vec<_>>())
                .column("fill", fill.iter().map(Json::as_f64).collect::<Vec<_>>())
        } else {
            builder
                .column(
                    "colour",
                    color
                        .iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Vec<_>>(),
                )
                .column(
                    "fill",
                    fill.iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Vec<_>>(),
                )
        }
        .build()
        .unwrap();
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(numeric).unwrap() else {
            unreachable!()
        };
        scale.palette_theme_aesthetics.clear();
        if family == "binned" {
            scale.ggplot = Some(serde_json::from_value(json!({"Binned":{"limits":null,"oob":"Squish","breaks":{"Nice":5},"right":true}})).unwrap());
        } else if family == "manual" {
            let Some(GgplotScalePolicy::Discrete { palette, .. }) = scale.ggplot.as_deref_mut()
            else {
                unreachable!()
            };
            *palette = GgplotDiscretePalette::Manual {
                values: ["#FF0000", "#008000", "#0000FF"]
                    .into_iter()
                    .map(|v| Value::Color(chart_core::color::parse_r(v).unwrap().value()))
                    .collect(),
                names: None,
            };
        } else if family == "identity" {
            scale = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotDiscreteIdentity(
                GgplotDiscreteIdentity::default(),
            ));
        }
        scale.training = ScaleTraining::Eligible;
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(
                aes()
                    .x("x")
                    .y(1.)
                    .color("colour")
                    .fill("fill")
                    .color_scale("shared")
                    .fill_scale("shared"),
            )
            .scale(color_mapped(
                "shared",
                scale.with_guide(GgplotScaleGuide::Hidden).unwrap(),
            ))
            .layer(points().aesthetic_value(
                ValueAesthetic::Shape,
                Value::Number(chart_core::interpolate::Number(21.)),
            ))
            .build()
            .unwrap();
        let wire = p.to_json().unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        let wanted = t["result"]["colour"]
            .as_array()
            .unwrap()
            .iter()
            .zip(t["result"]["fill"].as_array().unwrap())
            .filter(|(c, _)| !c.is_null())
            .collect::<Vec<_>>();
        assert_eq!(marks.len(), wanted.len(), "{t}");
        assert_eq!(
            marks.len(),
            t["result"]["mark_count"].as_u64().unwrap() as usize,
            "{t}"
        );
        for (m, (c, f)) in marks.iter().zip(wanted) {
            let paint = |v: &Json| {
                v.as_str().map_or(
                    chart_core::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    },
                    |v| chart_core::color::parse_r(v).unwrap().resolve(),
                )
            };
            assert_eq!(m.style.color, paint(c), "{t}");
            assert_eq!(m.style.fill, Some(paint(f)), "{t}");
        }
        assert_eq!(p.to_json().unwrap(), wire);
    }
}
