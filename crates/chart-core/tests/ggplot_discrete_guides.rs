//! FIX-GG04: pinned discrete break and label rules and primary palette stability.
use chart_core::{interpolate::Number, scales::*};
fn key(s: &str) -> ScaleKey {
    ScaleKey::Text(s.into())
}
fn domain() -> Vec<ScaleKey> {
    vec![key("a"), key("b"), key("c"), ScaleKey::Null]
}
fn guide(b: &str, l: &str) -> GgplotDiscreteGuide {
    GgplotDiscreteGuide {
        breaks: match b {
            "auto" => None,
            "empty" => Some(vec![]),
            _ => Some(vec![key("c"), key("z"), key("a"), key("c"), ScaleKey::Null]),
        },
        break_names: (b == "named").then(|| {
            ["Third", "Outside", "First", "Again", "Missing"]
                .map(String::from)
                .to_vec()
        }),
        labels: match l {
            "auto" => GgplotGuideLabels::Automatic,
            "hidden" => GgplotGuideLabels::Hidden,
            "explicit" => GgplotGuideLabels::Explicit(
                ["C", "Z", "A", "C2", "Missing"]
                    .map(|s| Some(s.into()))
                    .to_vec(),
            ),
            "short" => {
                GgplotGuideLabels::Explicit(vec![Some("first".into()), Some("second".into())])
            }
            "named" => GgplotGuideLabels::Named(
                [
                    ("c", "cee"),
                    ("z", "zed"),
                    ("a", "aye"),
                    ("c", "last"),
                    ("unused", "unused"),
                ]
                .map(|(k, v)| (key(k), Some(v.into())))
                .to_vec(),
            ),
            _ => unreachable!(),
        },
    }
}
#[test]
fn sixty_pinned_scale_guide_cases_match_keys_labels_and_rejections() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete-guides.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 60);
    for case in cases {
        let spec = guide(
            case["break_mode"].as_str().unwrap(),
            case["label_mode"].as_str().unwrap(),
        );
        let actual = spec.resolve(&domain());
        let result = &case["result"];
        if result.get("error").is_some() {
            assert!(actual.is_err(), "{case}");
            continue;
        }
        let actual = actual.unwrap();
        let expected = result["breaks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map_or(ScaleKey::Null, key))
            .collect::<Vec<_>>();
        assert_eq!(
            actual.iter().map(|e| e.key.clone()).collect::<Vec<_>>(),
            expected,
            "{case}"
        );
        let expected = result["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>();
        let labels = actual.iter().map(|e| e.label.clone()).collect::<Vec<_>>();
        if result["hidden"] == true {
            assert!(labels.iter().all(Option::is_none));
        } else {
            assert_eq!(labels, expected, "{case}");
        }
    }
}
#[test]
fn primary_guide_selection_preserves_palette_and_updates_through_wire() {
    use chart_core::prelude::*;
    let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(false).unwrap() else {
        unreachable!()
    };
    scale.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
        breaks: Some(vec![key("c"), key("z"), key("a"), key("c")]),
        labels: GgplotGuideLabels::Explicit(
            ["Third", "Outside", "First", "Duplicate"]
                .map(|s| Some(s.into()))
                .to_vec(),
        ),
        ..Default::default()
    })));
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("category", ["a", "b", "c"])
        .build()
        .unwrap();
    let make = |scale| {
        plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).color("category").color_scale("category"))
            .scale(color_mapped("category", scale))
            .layer(points())
            .build()
            .unwrap()
    };
    let p = make(scale.clone());
    let round = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let chart = round.chart().unwrap().prepare().unwrap();
    let layer = &chart.layers()[0];
    let legend = layer.color_legend().unwrap();
    assert_eq!(
        legend
            .entries
            .iter()
            .map(|e| e.0.as_str())
            .collect::<Vec<_>>(),
        ["Third", "First"]
    );
    assert_eq!(legend.entries[0].1, layer.marks()[2].style.color);
    assert_eq!(legend.entries[1].1, layer.marks()[0].style.color);
    scale.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
        breaks: Some(vec![]),
        ..Default::default()
    })));
    let next = make(scale).chart().unwrap().prepare().unwrap();
    assert!(
        next.layers()[0]
            .color_legend()
            .is_none_or(|l| l.entries.is_empty())
    );
    assert_eq!(next.layers()[0].marks(), layer.marks());
}
#[test]
fn typed_break_matching_and_budget_guards() {
    let g = GgplotDiscreteGuide {
        breaks: Some(vec![key("TRUE"), key("1"), ScaleKey::Null, key("NA")]),
        ..Default::default()
    };
    let entries = g
        .resolve(&[
            ScaleKey::Boolean(true),
            ScaleKey::Number(Number(1.)),
            ScaleKey::Null,
            key("NA"),
        ])
        .unwrap();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].key, ScaleKey::Boolean(true));
    assert_eq!(entries[2].label, None);
    assert_eq!(entries[3].label, Some("NA".into()));
    assert!(
        GgplotDiscreteGuide {
            break_names: Some(vec!["invalid".into()]),
            ..Default::default()
        }
        .resolve(&domain())
        .is_err()
    );
    assert!(
        GgplotDiscreteGuide {
            labels: GgplotGuideLabels::Explicit(vec![Some("a".repeat(4097))]),
            ..Default::default()
        }
        .resolve(&domain())
        .is_err()
    );
    let mut s =
        MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(Default::default()));
    s.guide = Some(Box::new(GgplotScaleGuide::Discrete(Default::default())));
    assert!(MappedScale::new(s).is_err());
}

#[test]
fn manual_breaks_name_outputs_without_restricting_trained_domain() {
    use chart_core::{color::parse_r, interpolate::Value};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/manual-guide-mapping.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    let keys = |v: &serde_json::Value| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().map_or(ScaleKey::Null, key))
            .collect::<Vec<_>>()
    };
    for case in cases {
        let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(false).unwrap() else {
            unreachable!()
        };
        let GgplotScalePolicy::Discrete {
            palette, limits, ..
        } = scale.ggplot.as_deref_mut().unwrap()
        else {
            unreachable!()
        };
        *palette = GgplotDiscretePalette::Manual {
            values: ["red", "green", "blue"]
                .map(|s| Value::Text(s.into()))
                .to_vec(),
            names: (case["named"] == true).then(|| vec![key("c"), key("c"), key("a")]),
        };
        *limits = case["limits"].as_array().map(|_| keys(&case["limits"]));
        scale.guide = Some(Box::new(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
            breaks: (case["automatic"] != true).then(|| keys(&case["breaks"])),
            ..Default::default()
        })));
        let trained = scale.trained_keys(&domain());
        if case["result"].get("error").is_some() {
            assert!(trained.is_err(), "{case}");
            continue;
        }
        let trained = trained.unwrap();
        let ScaleFunctionSpec::Ordinal(ordinal) = &trained.function else {
            unreachable!()
        };
        assert_eq!(ordinal.domain, keys(&case["result"]["domain"]), "{case}");
        let mapped = MappedScale::for_colors(trained).unwrap();
        let missing = parse_r("grey50").unwrap().resolve();
        for (key, expected) in [key("a"), key("b"), key("c"), key("d"), ScaleKey::Null]
            .iter()
            .zip(case["result"]["mapped"].as_array().unwrap())
        {
            let color = mapped.color(None, Some(key), missing).unwrap();
            assert_eq!(
                color,
                parse_r(expected.as_str().unwrap()).unwrap().resolve(),
                "{case}"
            );
        }
    }
}
