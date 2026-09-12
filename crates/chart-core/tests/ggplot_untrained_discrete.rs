//! FIX-GG04: lazy lookup defaults for an untrained discrete population.
use chart_core::{
    color::parse_r,
    interpolate::{Number, Value},
    scales::*,
};
use serde_json::Value as Json;
fn keys(v: &Json) -> Vec<ScaleKey> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v.as_str()
                .map_or(ScaleKey::Null, |s| ScaleKey::Text(s.into()))
        })
        .collect()
}
#[test]
fn untrained_discrete_lookups_match_reference_without_inventing_guides() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/untrained-discrete-lookups.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 64);
    for case in fixture["cases"].as_array().unwrap() {
        let palette = match case["palette"].as_str().unwrap() {
            "hue" => GgplotDiscretePalette::Hue(Default::default()),
            kind => GgplotDiscretePalette::Manual {
                values: if kind == "manual_one" {
                    vec![Value::Text("#112233".into())]
                } else {
                    vec![Value::Text("#112233".into()), Value::Text("#445566".into())]
                },
                names: (kind == "named")
                    .then(|| vec![ScaleKey::Text("0".into()), ScaleKey::Text("1".into())]),
            },
        };
        let mut spec =
            MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
                .with_ggplot(GgplotScalePolicy::Discrete {
                    empty_population: false,
                    limits: None,
                    levels: case["levels"].as_array().map(|_| keys(&case["levels"])),
                    drop: false,
                    na_translate: case["translate"].as_bool().unwrap(),
                    palette,
                })
                .unwrap();
        if case["breaks"].is_array() {
            spec.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
                breaks: Some(keys(&case["breaks"])),
                ..Default::default()
            })));
        }
        if case["population"] == "empty" {
            use chart_core::prelude::*;
            assert_eq!(case["result"]["empty_chart_ok"], true);
            let data = Data::columns()
                .column("x", Vec::<f64>::new())
                .column("v", Vec::<Option<String>>::new())
                .build()
                .unwrap();
            let mut chart = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("v").color_scale("v"))
                .scale(color_mapped("v", spec.clone()))
                .layer(points())
                .build()
                .unwrap()
                .chart()
                .unwrap();
            let prepared = chart.prepare().unwrap();
            assert!(prepared.layers()[0].marks().is_empty());
            assert!(
                prepared.layers()[0]
                    .color_legend()
                    .is_none_or(|legend| legend.entries.is_empty())
            );
        }
        let trained = spec.trained_keys(&keys(&case["inputs"]));
        if let Err(error) = &trained {
            assert!(case["result"]["error"].is_string(), "{case}: {error}");
            continue;
        }
        let trained = trained.unwrap();
        let mapped = MappedScale::for_colors(trained.clone()).unwrap();
        assert_eq!(mapped.spec(), &trained);
        for (kind, query) in [
            (
                "text",
                vec![
                    ScaleKey::Text("0".into()),
                    ScaleKey::Text("1".into()),
                    ScaleKey::Text("2".into()),
                    ScaleKey::Null,
                ],
            ),
            (
                "numeric",
                vec![
                    ScaleKey::Number(Number(0.)),
                    ScaleKey::Number(Number(1.)),
                    ScaleKey::Number(Number(2.)),
                    ScaleKey::Null,
                ],
            ),
            (
                "logical",
                vec![
                    ScaleKey::Boolean(false),
                    ScaleKey::Boolean(true),
                    ScaleKey::Null,
                ],
            ),
        ] {
            for (i, key) in query.iter().enumerate() {
                let actual = mapped.color(None, Some(key), parse_r("grey50").unwrap().resolve());
                if case["result"]["error"].is_string() {
                    assert!(actual.is_err(), "{case}: {actual:?}");
                    continue;
                }
                let value = &case["result"][kind][i];
                let expected = value.as_str().map_or(
                    chart_core::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    },
                    |v| parse_r(v).unwrap().resolve(),
                );
                assert_eq!(actual.unwrap(), expected, "{case}, {kind}, {key:?}");
            }
        }
    }
}
