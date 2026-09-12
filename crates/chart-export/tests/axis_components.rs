//! AXIS-05: the portable SVG hierarchy and outline labels come from one retained scene.
#[path = "../../../examples/common/axis_component_fixtures.rs"]
mod fixtures;
use chart_core::scene::GuideRole;
use chart_export::*;

#[test]
fn styled_components_keep_addressable_groups_and_logical_labels_in_text_and_outline() {
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let p = fixtures::figure().unwrap();
    let mut previous = None;
    for mode in [TextMode::Preserve, TextMode::Outline] {
        let frame = output
            .request(
                &p,
                export_options(PageSize::points(600., 400.).unwrap())
                    .text(mode)
                    .dpi(72),
            )
            .unwrap()
            .prepare()
            .unwrap();
        let artifact = frame.export(Format::Svg).unwrap();
        let svg = std::str::from_utf8(&artifact.bytes).unwrap();
        let doc = usvg::roxmltree::Document::parse(svg).unwrap();
        let count = |class: &str| {
            doc.descendants()
                .filter(|n| n.attribute("class") == Some(class))
                .count()
        };
        assert_eq!(count("axis"), 3);
        assert_eq!(count("domain"), 2);
        assert_eq!(count("tick"), 11);
        assert_eq!(count("tick-line"), 11);
        assert_eq!(count("label"), 8);
        for tick in doc
            .descendants()
            .filter(|n| n.attribute("class") == Some("tick"))
        {
            assert!(tick.attribute("data-value").is_some());
            assert!(tick.attribute("data-occurrence").is_some());
            assert!(tick.attribute("data-label").is_some());
            assert!(
                tick.descendants()
                    .any(|n| n.attribute("class") == Some("tick-line"))
            );
        }
        assert_eq!(
            doc.descendants().filter(|n| n.has_tag_name("text")).count(),
            if mode == TextMode::Preserve { 8 } else { 0 }
        );
        let component_labels: Vec<_> = frame
            .scene()
            .items()
            .iter()
            .filter_map(|i| i.guide.as_ref())
            .filter(|g| g.role == GuideRole::Label)
            .map(|g| g.label.clone())
            .collect();
        if let Some(previous) = &previous {
            assert_eq!(previous, &component_labels);
        } else {
            previous = Some(component_labels);
        }
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
        if mode == TextMode::Outline {
            // Independent reparse must retain positioned vector content without any font database.
            assert!(!tree.root().children().is_empty());
            for label in doc
                .descendants()
                .filter(|n| n.attribute("class") == Some("label"))
            {
                assert!(
                    label.descendants().any(|n| n.has_tag_name("path")
                        && n.attribute("d").is_some_and(|d| !d.is_empty()))
                );
            }
            assert!(!svg.contains("data:font"));
        }
    }
}

#[test]
fn fully_transparent_labels_retain_their_logical_outline_roles() {
    use chart_core::{
        layout::{GuideComponents, GuideProfile, GuideTextStyle},
        prelude::*,
    };
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let p = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .guide_profile(GuideProfile::D3_3_0_0)
            .tick_values(Some(vec![0f64.into(), 1f64.into()]))
            .guide_components(Some(GuideComponents {
                labels: GuideTextStyle {
                    color: Some(chart_core::color::Paint::from_css("#00000000").unwrap()),
                    ..Default::default()
                },
                ..Default::default()
            })),
    )
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap();
    let frame = output
        .request(
            &p,
            export_options(PageSize::points(400., 300.).unwrap()).text(TextMode::Outline),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let artifact = frame.export(Format::Svg).unwrap();
    let svg = std::str::from_utf8(&artifact.bytes).unwrap();
    let doc = usvg::roxmltree::Document::parse(svg).unwrap();
    assert_eq!(
        doc.descendants()
            .filter(|n| n.attribute("class") == Some("label"))
            .count(),
        2
    );
    assert_eq!(
        doc.descendants().filter(|n| n.has_tag_name("text")).count(),
        0
    );
}

#[test]
fn per_tick_typography_uses_the_supplied_shaper_and_portable_override_cascade() {
    use chart_core::scene::Primitive;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let p = fixtures::typography().unwrap();
    let frame = output
        .request(&p, export_options(PageSize::points(400., 300.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let labels: Vec<_> = frame
        .scene()
        .items()
        .iter()
        .filter(|i| i.guide.as_ref().is_some_and(|g| g.role == GuideRole::Label))
        .collect();
    assert_eq!(labels.len(), 3);
    for (i, item) in labels.iter().enumerate() {
        let Primitive::GlyphRun { run, rotation, .. } = &item.primitive else {
            panic!("supplied shaped glyph run");
        };
        assert_eq!(run.font_size, if i == 1 { 18. } else { 15. });
        assert_eq!(*rotation, if i == 1 { 30. } else { 0. });
        assert_eq!(
            run.text,
            item.guide.as_ref().unwrap().label.as_deref().unwrap()
        );
        assert!(!run.outlines.is_empty());
    }
    let svg = frame.export(Format::Svg).unwrap();
    assert_eq!(
        svg.capabilities.text,
        TextRepresentation::MixedPositionedOutlines
    );
}
