//! GG14 independent source layout topology and physical panel dimensions.
use chart_core::{grammar::FacetPolicy, prelude::*, theme::*};
struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.chars().count() as f64 * 6., 9., 3.)
    }
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::typography::ShapedRun> {
        Ok(chart_core::typography::ShapedRun {
            text: r.run.text.clone(),
            font: *r.default_font,
            font_size: r.font_size * r.run.size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: chart_core::services::TextMetrics::new(
                r.run.text.chars().count() as f64 * 6.,
                9.,
                3.,
            )?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
}

fn draw(p: &Plot, w: f64) -> chart_core::layout::LaidOutChart {
    use chart_core::{layout::*, services::*, *};
    let request = LayoutRequest::new(
        Rect::new(0., 0., w, 500.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 10,
        },
    );
    layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap()
}

#[test]
fn source_physical_and_weighted_panel_sizes_exclude_axis_furniture() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-consumer-layout.json"
    ))
    .unwrap();
    assert!(
        source["cases"]["physical_panels"]["widths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "2cm")
    );
    for unit in ["cm", "null"] {
        let e = ElementTheme::preset(ThemePreset::Test)
            .unwrap()
            .element(
                "panel.widths",
                ThemeEntry::Value(ThemeValue::Unit(vec![
                    ThemeLength {
                        value: Some(2.),
                        unit: unit.into(),
                    },
                    ThemeLength {
                        value: Some(4.),
                        unit: unit.into(),
                    },
                ])),
            )
            .unwrap();
        let p = plot(
            Data::columns()
                .column("x", [1., 2., 1., 2.])
                .column("y", [1., 2., 2., 3.])
                .column("g", ["A", "A", "B", "B"])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("g").columns(2).reference(FacetPolicy::default()))
        .theme(theme().elements(e).unwrap())
        .build()
        .unwrap();
        for width in [800., 1200.] {
            let laid = draw(&p, width);
            let panels = laid.panels();
            assert_eq!(panels.len(), 2);
            let a = panels[0].chart.plot().unwrap().width();
            let b = panels[1].chart.plot().unwrap().width();
            assert!((b / a - 2.).abs() < 1e-9, "{unit}: {a}, {b}");
            if unit == "cm" {
                assert!((a - 2. * 72. / 2.54).abs() < 1e-9, "actual {a}");
            }
        }
    }
}

#[test]
fn source_strip_order_and_panel_ontop_are_consumed_after_layout() {
    use chart_core::{layout::AxisSide, scene::Primitive};
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-consumer-layout.json"
    ))
    .unwrap();
    let row = |case: &str, name: &str| {
        source["cases"][case]["layout"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()["t"]
            .as_u64()
            .unwrap()
    };
    assert!(row("inside", "axis-t-1-1") < row("inside", "strip-t-1-1"));
    assert!(row("outside", "axis-t-1-1") > row("outside", "strip-t-1-1"));
    for outside in [false, true] {
        let e = ElementTheme::preset(ThemePreset::Test)
            .unwrap()
            .element(
                "strip.placement",
                ThemeEntry::Value(ThemeValue::Text(
                    if outside { "outside" } else { "inside" }.into(),
                )),
            )
            .unwrap();
        let p = plot(
            Data::columns()
                .column("x", [1., 2., 1., 2.])
                .column("y", [1., 2., 2., 3.])
                .column("g", ["A", "A", "B", "B"])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(x_axis().side(AxisSide::Top))
        .facet(facet_wrap("g").columns(2).reference(FacetPolicy::default()))
        .theme(theme().elements(e).unwrap())
        .build()
        .unwrap();
        let laid = draw(&p, 800.);
        for panel in laid.panels() {
            let guide = panel
                .chart
                .guides()
                .values()
                .find(|g| g.spec.side == AxisSide::Top)
                .unwrap();
            assert_eq!(guide.spec.translation[1] == 0., outside);
        }
    }
    for on_top in [false, true] {
        let e = ElementTheme::preset(ThemePreset::Grey)
            .unwrap()
            .element("panel.ontop", ThemeEntry::Value(ThemeValue::Bool(on_top)))
            .unwrap()
            .element("panel.background", ThemeEntry::Blank)
            .unwrap()
            .element(
                "panel.grid.major",
                ThemeEntry::Element(
                    ThemeElement::new(ElementKind::Line)
                        .property("colour", ThemeValue::Text("#FF0102".into()))
                        .unwrap(),
                ),
            )
            .unwrap();
        let p = plot(
            Data::columns()
                .column("x", [1., 2.])
                .column("y", [1., 2.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .theme(theme().elements(e).unwrap())
        .build()
        .unwrap();
        let laid = draw(&p, 800.);
        let items = laid.scene().items();
        let mark = items.iter().position(|i| i.layer.is_some()).unwrap();
        let grid = items
            .iter()
            .position(|i| match &i.primitive {
                Primitive::ShapePath { fill: Some(c), .. } => {
                    c.red == 255 && c.green == 1 && c.blue == 2
                }
                Primitive::Path { stroke, .. } => {
                    stroke.color.red == 255 && stroke.color.green == 1
                }
                _ => false,
            })
            .unwrap();
        assert_eq!(grid > mark, on_top);
    }
}

#[test]
fn theme_text_defaults_preserve_explicit_equal_default_controls() {
    use chart_core::grammar::TextGeom;
    for inherit in [true, false] {
        let layer = if inherit {
            points().text_defaults()
        } else {
            points().text_geom(TextGeom::default())
        }
        .text_label("label");
        let p = plot(
            Data::columns()
                .column("x", [1.])
                .column("y", [1.])
                .column("label", ["row"])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .theme(
            theme()
                .reference_preset(
                    ThemePreset::Test,
                    ThemePresetOptions {
                        base_size: 16.,
                        ..Default::default()
                    },
                )
                .unwrap(),
        )
        .build()
        .unwrap();
        let replay = Plot::from_json(&p.to_json().unwrap()).unwrap();
        assert_eq!(
            replay.definition().layers[0]
                .grammar
                .as_ref()
                .unwrap()
                .default_text,
            inherit
        );
        let laid = draw(&replay, 800.);
        let run = laid
            .scene()
            .items()
            .iter()
            .find_map(|i| match &i.primitive {
                chart_core::scene::Primitive::GlyphRun { run, .. } if run.text == "row" => {
                    Some(run)
                }
                _ => None,
            })
            .unwrap();
        let expected = if inherit { 16. } else { 3.88 * 72.27 / 25.4 };
        assert!(
            (run.font_size - expected).abs() < 1e-10,
            "{inherit}: {} vs {expected}",
            run.font_size
        );
    }
}

#[test]
fn source_multiple_guide_box_direction_uses_shared_measured_blocks() {
    use chart_core::{
        Point,
        scene::{GuideRole, Primitive},
    };
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/theme-consumer-layout.json"
    ))
    .unwrap();
    for horizontal in [false, true] {
        let direction = if horizontal { "horizontal" } else { "vertical" };
        let rows = source["legend_boxes"][direction]["layout"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["name"] == "guides")
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["t"] == rows[1]["t"], horizontal);
        let e = ElementTheme::preset(ThemePreset::Test)
            .unwrap()
            .element(
                "legend.position",
                ThemeEntry::Value(ThemeValue::Text("bottom".into())),
            )
            .unwrap()
            .element(
                "legend.box",
                ThemeEntry::Value(ThemeValue::Text(direction.into())),
            )
            .unwrap()
            .element(
                "legend.box.just",
                ThemeEntry::Value(ThemeValue::Text("top".into())),
            )
            .unwrap();
        let p = plot(
            Data::columns()
                .column("x", [1., 2., 3., 4.])
                .column("y", [1., 2., 2., 3.])
                .column("g", ["A", "A", "B", "B"])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("g").size("x"))
        .layer(points())
        .theme(theme().elements(e).unwrap())
        .build()
        .unwrap();
        let laid = draw(&p, 800.);
        let origins: Vec<Point> = laid
            .scene()
            .items()
            .iter()
            .filter(|i| {
                i.guide
                    .as_ref()
                    .is_some_and(|g| g.role == GuideRole::LegendTitle)
            })
            .filter_map(|i| match &i.primitive {
                Primitive::GlyphRun { origin, .. } | Primitive::Text { origin, .. } => {
                    Some(*origin)
                }
                _ => None,
            })
            .collect();
        assert_eq!(origins.len(), 2);
        if horizontal {
            assert!((origins[0].y() - origins[1].y()).abs() < 1e-9);
            assert!((origins[0].x() - origins[1].x()).abs() > 1.);
        } else {
            assert!((origins[0].y() - origins[1].y()).abs() > 1.);
        }
    }
}

#[test]
fn explicit_flat_tokens_override_reference_element_defaults() {
    use chart_core::{color::Paint, scene::Primitive};
    let red = Paint::from_css("#FF0102").unwrap();
    let p = plot(
        Data::columns()
            .column("x", [1., 2.])
            .column("y", [1., 2.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .title(title("heading"))
    .theme(
        theme()
            .reference_preset(ThemePreset::Grey, Default::default())
            .unwrap()
            .style(style().background(red).foreground(red).font_size(20.)),
    )
    .build()
    .unwrap();
    let laid = draw(&p, 800.);
    let heading = laid
        .scene()
        .items()
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::GlyphRun { run, color, .. } if run.text == "heading" => Some((run, *color)),
            _ => None,
        })
        .unwrap();
    assert!((heading.0.font_size - 24.).abs() < 1e-10);
    assert_eq!(heading.1, red.resolve());
    assert!(laid.scene().items().iter().any(
        |i| matches!(&i.primitive,Primitive::ShapePath{fill:Some(c),..} if *c==red.resolve())
    ));
}
