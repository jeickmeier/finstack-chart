//! FIX-GG04: complete R color catalog and parser boundaries, separate from CSS.
use chart_core::{
    DiagnosticCode,
    color::{parse_r, parse_r_with_palette},
    scene::Color,
};
#[test]
fn complete_r_catalog_and_parser_match_grdevices() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/color-names.json"
    ))
    .unwrap();
    assert_eq!(fixture["catalog"].as_array().unwrap().len(), 657);
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 43);
    for row in fixture["catalog"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["cases"].as_array().unwrap())
    {
        let input = row["input"].as_str().unwrap();
        let result = parse_r(input);
        // The reference's platform-dependent overflow conversion is outside the portable index domain.
        if input == "99999999999999999999" {
            assert_eq!(result.unwrap_err().code, DiagnosticCode::PrecisionLoss);
            continue;
        }
        if row.get("error").is_some() {
            assert!(result.is_err(), "{row}");
            continue;
        }
        let c = result.unwrap().resolve();
        assert_eq!(
            serde_json::json!([c.red, c.green, c.blue, c.alpha]),
            row["rgba"],
            "{row}"
        );
    }
    assert!(parse_r(&"a".repeat(4097)).is_err());
    let palette = [
        Color {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        },
        Color {
            red: 5,
            green: 6,
            blue: 7,
            alpha: 8,
        },
    ];
    assert_eq!(
        parse_r_with_palette("3", &palette).unwrap().resolve(),
        palette[0]
    );
    assert!(parse_r_with_palette("1", &[]).is_err());
    assert!(chart_core::color::parse("red4").unwrap().is_none());
}

#[test]
fn reference_manual_color_text_uses_r_catalog_and_retains_transparent_channels() {
    use chart_core::{interpolate::Value, scales::*};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/manual-color-text.json"
    ))
    .unwrap();
    let values = fixture["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Value::Text(v.as_str().unwrap().into()))
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 8);
    let keys = (b'a'..=b'h')
        .map(|v| ScaleKey::Text(char::from(v).to_string()))
        .collect::<Vec<_>>();
    let spec = MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec {
        domain: vec![],
        range: vec![],
        unknown: OrdinalUnknown::Explicit(None),
    }))
    .with_ggplot(GgplotScalePolicy::Discrete {
        empty_population: false,
        limits: None,
        levels: None,
        drop: true,
        na_translate: true,
        palette: GgplotDiscretePalette::Manual {
            values,
            names: None,
        },
    })
    .unwrap()
    .trained_keys(&keys)
    .unwrap();
    let scale = MappedScale::for_colors(spec.clone()).unwrap();
    let missing = Color {
        red: 9,
        green: 9,
        blue: 9,
        alpha: 255,
    };
    for (i, key) in keys.iter().enumerate() {
        let c = scale.color(None, Some(key), missing).unwrap();
        assert_eq!(
            serde_json::json!([c.red, c.green, c.blue, c.alpha]),
            fixture["rgba"][i]
        );
    }
    let wire = serde_json::to_string(&spec).unwrap();
    assert_eq!(
        serde_json::from_str::<MappedScaleSpec>(&wire).unwrap(),
        spec
    );
    let mut css = spec;
    css.ggplot = None;
    css.training = ScaleTraining::Authored;
    assert!(MappedScale::for_colors(css).is_err());
}
