//! FIX-GG14 independent calc_element fixtures, including all ordered-parent nodes.
use chart_core::theme::{ElementTheme, ThemeEntry, ThemePreset};
#[test]
fn all_source_preset_nodes_resolve_exactly() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-resolution-vectors.json"
    ))
    .unwrap();
    for case in cases.as_array().unwrap() {
        if case.get("preset").is_none() {
            continue;
        }
        let theme: ElementTheme = serde_json::from_value(case["theme"].clone()).unwrap();
        let resolved = theme
            .resolve_all()
            .unwrap_or_else(|e| panic!("{} {e:?}", case["name"]));
        for (name, value) in case["expected"].as_object().unwrap() {
            let expected: Option<ThemeEntry> = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(resolved[name], expected, "{} {name}", case["name"]);
        }
    }
}
#[test]
fn default_preset_authors_and_inheritance_vectors() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-resolution-vectors.json"
    ))
    .unwrap();
    for case in cases.as_array().unwrap() {
        if case.get("preset").is_some() {
            if case["variant"] != "default" {
                continue;
            }
            let preset = match case["preset"].as_str().unwrap() {
                "grey" => ThemePreset::Grey,
                "bw" => ThemePreset::Bw,
                "linedraw" => ThemePreset::Linedraw,
                "light" => ThemePreset::Light,
                "dark" => ThemePreset::Dark,
                "minimal" => ThemePreset::Minimal,
                "classic" => ThemePreset::Classic,
                "void" => ThemePreset::Void,
                "test" => ThemePreset::Test,
                _ => unreachable!(),
            };
            assert_eq!(
                ElementTheme::preset(preset).unwrap(),
                serde_json::from_value::<ElementTheme>(case["theme"].clone()).unwrap()
            );
            continue;
        }
        let base: ElementTheme = serde_json::from_value(case["base"].clone()).unwrap();
        let patch: ElementTheme = serde_json::from_value(case["patch"].clone()).unwrap();
        let result = base.update(&patch);
        if !case["errors"].as_array().unwrap().is_empty() {
            assert!(result.is_err());
            continue;
        }
        let theme = result.unwrap_or_else(|e| panic!("{}: {e:?}", case["name"]));
        for (name, value) in case["expected"].as_object().unwrap() {
            let expected: Option<ThemeEntry> = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(
                theme
                    .resolve_element(name, case["skip_blank"].as_bool().unwrap())
                    .unwrap(),
                expected,
                "{} {name}",
                case["name"]
            );
        }
    }
}

#[test]
fn custom_preset_controls_match_source_authored_values() {
    use chart_core::theme::ThemePresetOptions;
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-resolution-vectors.json"
    ))
    .unwrap();
    for case in cases.as_array().unwrap() {
        if case.get("preset").is_none() || case["variant"] != "custom" {
            continue;
        }
        let preset = match case["preset"].as_str().unwrap() {
            "grey" => ThemePreset::Grey,
            "bw" => ThemePreset::Bw,
            "linedraw" => ThemePreset::Linedraw,
            "light" => ThemePreset::Light,
            "dark" => ThemePreset::Dark,
            "minimal" => ThemePreset::Minimal,
            "classic" => ThemePreset::Classic,
            "void" => ThemePreset::Void,
            "test" => ThemePreset::Test,
            _ => unreachable!(),
        };
        let actual = ElementTheme::preset_with(
            preset,
            ThemePresetOptions {
                base_size: 16.,
                base_family: "Fixture Body".into(),
                header_family: Some("Fixture Header".into()),
                base_line_size: Some(0.7),
                base_rect_size: Some(0.9),
                ink: "#123456".into(),
                paper: Some("#F8EEDD".into()),
                accent: "#D020A0".into(),
            },
        )
        .unwrap();
        let expected: ElementTheme = serde_json::from_value(case["theme"].clone()).unwrap();
        for (name, value) in expected.elements {
            assert_eq!(
                actual.elements.get(&name),
                Some(&value),
                "{} {name}",
                case["name"]
            );
        }
    }
}

#[test]
fn source_subtheme_expansion_and_isolated_context_operations() {
    use chart_core::theme::{ElementKind, ThemeContext, ThemeElement, ThemeValue};
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-context-vectors.json"
    ))
    .unwrap();
    for (name, case) in oracle["subthemes"].as_object().unwrap() {
        let entries = serde_json::from_value(case["authored"].clone()).unwrap();
        let actual = ElementTheme::subtheme(name, entries).unwrap();
        assert_eq!(
            actual,
            serde_json::from_value::<ElementTheme>(case["expected"].clone()).unwrap(),
            "{name}"
        );
    }
    let patch = |colour: &str, size: Option<f64>| {
        let mut e = ThemeElement::new(ElementKind::Text)
            .property("colour", ThemeValue::Text(colour.into()))
            .unwrap();
        e = e
            .property("inherit.blank", ThemeValue::Bool(false))
            .unwrap();
        if let Some(size) = size {
            e = e.property("size", ThemeValue::Number(size)).unwrap();
        }
        ElementTheme::default()
            .element("axis.text", ThemeEntry::Element(e))
            .unwrap()
    };
    let grey = ElementTheme::preset(ThemePreset::Grey).unwrap();
    let mut first = ThemeContext::new(grey.clone()).unwrap();
    let second = ThemeContext::new(grey.clone()).unwrap();
    assert_eq!(
        first
            .set(ElementTheme::preset(ThemePreset::Bw).unwrap())
            .unwrap(),
        grey
    );
    for (name, p, replace) in [
        ("updated", patch("red", Some(14.)), false),
        ("merged", patch("blue", None), false),
        ("replaced", patch("green", None), true),
    ] {
        if replace {
            first.replace(&p).unwrap();
        } else {
            first.update(&p).unwrap();
        }
        let expected: ElementTheme =
            serde_json::from_value(oracle["contexts"][name].clone()).unwrap();
        // Explicit missing variable-font attributes from S7 are not authored by this API.
        let actual = first.get().resolve_element("axis.text", false).unwrap();
        let expected = expected.resolve_element("axis.text", false).unwrap();
        assert_eq!(actual, expected, "{name}");
    }
    assert_eq!(second.get(), grey);
}

#[test]
fn grid_unit_context_preserves_bigpoint_text_and_texpoint_lengths() {
    use chart_core::{services::Units, theme::*};
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-unit-context.json"
    ))
    .unwrap();
    for case in source["cases"].as_array().unwrap() {
        let size = case["fontsize"].as_f64().unwrap();
        let lineheight = case["lineheight"].as_f64().unwrap();
        let base = ElementTheme::preset(ThemePreset::Grey)
            .unwrap()
            .update(
                &ElementTheme::default()
                    .element(
                        "text",
                        ThemeEntry::Element(
                            ThemeElement::new(ElementKind::Text)
                                .property("lineheight", ThemeValue::Number(lineheight))
                                .unwrap(),
                        ),
                    )
                    .unwrap(),
            )
            .unwrap();
        for (unit, expected) in case["inches"].as_object().unwrap() {
            let theme = base
                .update(
                    &ElementTheme::default()
                        .element(
                            "spacing",
                            ThemeEntry::Value(ThemeValue::Unit(vec![ThemeLength {
                                value: Some(1.),
                                unit: unit.clone(),
                            }])),
                        )
                        .unwrap(),
                )
                .unwrap();
            let e = ResolvedElements::new(&theme).unwrap();
            let actual = e
                .length("spacing", "", 0, Units::Points, size, 600.)
                .unwrap()
                .unwrap();
            assert!(
                (actual - expected.as_f64().unwrap() * 72.).abs() < 2e-12,
                "{unit}: {actual}"
            );
        }
    }
}

#[test]
fn all_presets_leave_statistical_population_and_provenance_unchanged() {
    use chart_core::{grammar::Profile, prelude::*};
    let data = Data::columns()
        .column("x", [1., 1., 2., 3.])
        .build()
        .unwrap();
    let base = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram())
        .build()
        .unwrap();
    let mut baseline = None;
    for preset in [
        ThemePreset::Grey,
        ThemePreset::Bw,
        ThemePreset::Linedraw,
        ThemePreset::Light,
        ThemePreset::Dark,
        ThemePreset::Minimal,
        ThemePreset::Classic,
        ThemePreset::Void,
        ThemePreset::Test,
    ] {
        let p = base
            .edit()
            .theme(
                theme()
                    .reference_preset(preset, Default::default())
                    .unwrap(),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let table = serde_json::to_value(prepared.layers()[0].table().rows()).unwrap();
        if let Some(expected) = &baseline {
            assert_eq!(&table, expected, "{preset:?}");
        } else {
            baseline = Some(table);
        }
    }
}
