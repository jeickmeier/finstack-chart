//! FIX-GG04: farver's active white reference across count and luminance changes.
use chart_core::{color, scales::chromatic::ggplot::HuePalette};
#[test]
fn hue_counts_match_140_pinned_palettes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/hue-counts.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 140);
    for t in cases {
        let palette = match t["mode"].as_str().unwrap() {
            "default" => HuePalette::default(),
            "reverse" => HuePalette {
                reverse: true,
                ..Default::default()
            },
            "dark" => HuePalette {
                chroma: 15.,
                luminance: 7.99,
                ..Default::default()
            },
            "shifted" => HuePalette {
                h: [-100., 720.],
                chroma: 65.,
                luminance: 40.,
                start: 33.,
                reverse: false,
            },
            _ => unreachable!(),
        };
        let expected = t["colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                color::Paint::from_css(v.as_str().unwrap())
                    .unwrap()
                    .resolve()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            palette.colors(t["n"].as_u64().unwrap() as usize).unwrap(),
            expected,
            "{t}"
        );
    }
}
