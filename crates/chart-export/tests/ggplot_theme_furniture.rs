//! Shared theme furniture placement over the real measured panel and figure.
use chart_core::{layout::*, prelude::*, scene::Primitive, theme::*, *};
fn frame(
    position: &str,
    tag_position: ThemeValue,
    location: &str,
    blank: bool,
) -> ChartResult<std::sync::Arc<LaidOutChart>> {
    let mut elements = ElementTheme::preset(ThemePreset::Grey)?
        .element(
            "plot.title.position",
            ThemeEntry::Value(ThemeValue::Text(position.into())),
        )?
        .element(
            "plot.caption.position",
            ThemeEntry::Value(ThemeValue::Text(position.into())),
        )?
        .element("plot.tag.position", ThemeEntry::Value(tag_position))?
        .element(
            "plot.tag.location",
            ThemeEntry::Value(ThemeValue::Text(location.into())),
        )?;
    if blank {
        elements = elements.element("plot.tag", ThemeEntry::Blank)?;
    }
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0., 1.])
        .build()?;
    let p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .theme(theme().elements(elements)?)
        .title(title("TITLE"))
        .caption(caption("CAPTION"))
        .tag(rich_text("TAG"))
        .build()?;
    assert_eq!(p.definition().wire_version(), 80);
    let p = Plot::from_json(&p.to_json()?)?;
    let output = chart_export::Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    Ok(output
        .request(
            &p,
            chart_export::export_options(chart_export::PageSize::points(600., 400.)?),
        )?
        .prepare()?
        .layout()
        .clone())
}
fn origin(frame: &LaidOutChart, label: &str) -> Point {
    frame
        .scene()
        .items()
        .iter()
        .find_map(|i| match &i.primitive {
            Primitive::Text { text, origin, .. } if text == label => Some(*origin),
            Primitive::GlyphRun { run, origin, .. } if run.text == label => Some(*origin),
            _ => None,
        })
        .expect("text")
}
#[test]
fn panel_title_and_caption_align_to_measured_panel() {
    let a = frame("panel", ThemeValue::Text("topright".into()), "plot", false).unwrap();
    let b = frame("plot", ThemeValue::Text("topright".into()), "plot", false).unwrap();
    assert!((origin(&a, "TITLE").x() - a.plot().unwrap().origin().x()).abs() < 1e-10);
    assert!(origin(&a, "TITLE").x() > origin(&b, "TITLE").x());
    assert!(origin(&a, "CAPTION").x() < origin(&b, "CAPTION").x());
    assert!(origin(&a, "TAG").x() > origin(&a, "TITLE").x());
}
#[test]
fn tag_locations_blank_and_invalid_numeric_margin() {
    let named = ThemeValue::Text("topleft".into());
    let margin = frame("panel", named.clone(), "margin", false).unwrap();
    let overlay = frame("panel", named.clone(), "panel", false).unwrap();
    assert!(margin.plot().unwrap().origin().x() > overlay.plot().unwrap().origin().x());
    assert!(origin(&overlay, "TAG").x() >= overlay.plot().unwrap().origin().x());
    let blank = frame("panel", named, "margin", true).unwrap();
    assert!(!blank.scene().items().iter().any(|i| match &i.primitive {
        Primitive::Text { text, .. } => text == "TAG",
        Primitive::GlyphRun { run, .. } => run.text == "TAG",
        _ => false,
    }));
    let numeric = ThemeValue::Vector(vec![ThemeValue::Number(0.5), ThemeValue::Number(0.5)]);
    assert!(frame("panel", numeric.clone(), "margin", false).is_err());
    assert!(frame("panel", numeric, "panel", false).is_ok());
}

#[test]
fn all_reference_tag_placements_remain_available() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-furniture-controls.json"
    ))
    .unwrap();
    let mut reference_errors = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let position = case["position"].as_str().unwrap();
        let location = case["location"].as_str().unwrap();
        let result = frame("panel", ThemeValue::Text(position.into()), location, false).unwrap();
        let p = origin(&result, "TAG");
        assert!(p.x().is_finite() && p.y().is_finite());
        if case.get("error").is_some() {
            reference_errors += 1;
            assert!(matches!(position, "left" | "right"));
            assert_ne!(location, "margin");
        } else {
            assert_eq!(case["tag"][0]["clip"], "off");
        }
    }
    // The pinned source has an unbound-vjust bug for four side-centered locations.
    // Preserve their documented placement capability with explicit inherited vjust.
    assert_eq!(reference_errors, 4);
    assert_eq!(fixture["numeric_margin_rejected"], true);
}
