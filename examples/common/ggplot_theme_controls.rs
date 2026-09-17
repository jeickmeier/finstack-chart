//! GG14 source-backed complete presets and inherited theme controls.
use chart_core::{grammar::*, prelude::*, theme::*};
pub const CASES: usize = 16;
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("y", [2., 1., 4., 3.])
        .column("g", ["A", "A", "B", "B"])
        .build()?;
    let presets = [
        ThemePreset::Grey,
        ThemePreset::Bw,
        ThemePreset::Linedraw,
        ThemePreset::Light,
        ThemePreset::Dark,
        ThemePreset::Minimal,
        ThemePreset::Classic,
        ThemePreset::Void,
        ThemePreset::Test,
    ];
    let mut elements = ElementTheme::preset(presets[mode.min(8)])?;
    if mode == 9 {
        elements = ElementTheme::preset_with(
            ThemePreset::Grey,
            ThemePresetOptions {
                base_size: 16.,
                ink: "#123456".into(),
                paper: Some("#F8EEDD".into()),
                accent: "#D020A0".into(),
                ..Default::default()
            },
        )?;
    } else if mode == 10 {
        elements = elements.update(
            &ElementTheme::default()
                .element("axis.text", ThemeEntry::Blank)?
                .element(
                    "axis.text.x",
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Text)
                            .property("inherit.blank", ThemeValue::Bool(false))?
                            .property("colour", ThemeValue::Text("red".into()))?
                            .property("size", ThemeValue::Relative(1.5))?,
                    ),
                )?,
        )?;
    } else if mode == 11 {
        elements = elements.update(&ElementTheme::subtheme(
            "axis_y",
            std::collections::BTreeMap::from([
                (
                    "ticks.length".into(),
                    ThemeEntry::Value(ThemeValue::Unit(vec![ThemeLength {
                        value: Some(-2.),
                        unit: "mm".into(),
                    }])),
                ),
                (
                    "line".into(),
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Line)
                            .property("colour", ThemeValue::Text("#008080".into()))?
                            .property("linewidth", ThemeValue::Number(1.))?
                            .property(
                                "arrow",
                                ThemeValue::Arrow(ArrowSpec {
                                    angle: 25.,
                                    length_mm: 4.,
                                    ends: ArrowEnds::Both,
                                    closed: true,
                                }),
                            )?
                            .property("arrow.fill", ThemeValue::Text("red".into()))?
                            .property("lineend", ThemeValue::Text("square".into()))?
                            .property("linetype", ThemeValue::Text("dashed".into()))?,
                    ),
                ),
            ]),
        )?)?;
    }
    if mode == 12 {
        elements = elements.update(
            &ElementTheme::default()
                .element(
                    "panel.spacing.x",
                    ThemeEntry::Value(ThemeValue::Unit(vec![ThemeLength {
                        value: Some(8.),
                        unit: "mm".into(),
                    }])),
                )?
                .element(
                    "legend.position",
                    ThemeEntry::Value(ThemeValue::Text("bottom".into())),
                )?
                .element(
                    "strip.background",
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Rect)
                            .property("fill", ThemeValue::Text("#CCEEDD".into()))?,
                    ),
                )?,
        )?;
    } else if mode == 13 {
        elements = elements.update(
            &ElementTheme::default().element(
                "axis.text.theta",
                ThemeEntry::Element(
                    ThemeElement::new(ElementKind::Text)
                        .property("colour", ThemeValue::Text("#CC2200".into()))?
                        .property("size", ThemeValue::Number(14.))?,
                ),
            )?,
        )?;
    }
    if mode == 15 {
        elements = elements.update(
            &ElementTheme::default()
                .element(
                    "legend.frame",
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Rect)
                            .property("fill", ThemeValue::Missing)?
                            .property("colour", ThemeValue::Text("red".into()))?,
                    ),
                )?
                .element(
                    "legend.ticks",
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Line)
                            .property("colour", ThemeValue::Text("#00AA55".into()))?
                            .property("linewidth", ThemeValue::Number(1.))?,
                    ),
                )?
                .element(
                    "legend.title.position",
                    ThemeEntry::Value(ThemeValue::Text("left".into())),
                )?
                .element(
                    "legend.position",
                    ThemeEntry::Value(ThemeValue::Text("bottom".into())),
                )?,
        )?;
    }
    if mode == 12 {
        elements=elements.update(&serde_json::from_value(serde_json::json!({"complete": false, "elements": {"panel.widths": {"Value": {"Unit": [{"value": 1.0, "unit": "null"}, {"value": 3.0, "unit": "null"}]}}, "strip.placement": {"Value": {"Text": "outside"}}, "strip.clip": {"Value": {"Text": "off"}}, "legend.box": {"Value": {"Text": "horizontal"}}, "legend.box.just": {"Value": {"Text": "top"}}, "legend.spacing.x": {"Value": {"Unit": [{"value": 6.0, "unit": "mm"}]}}, "legend.title.position": {"Value": {"Text": "left"}}, "legend.key.spacing.x": {"Value": {"Unit": [{"value": 2.0, "unit": "mm"}]}}}})).expect("independent theme group controls"))?;
    }
    let mut themed = theme().elements(elements)?;
    if mode == 14 {
        themed = theme()
            .reference_preset(
                ThemePreset::Test,
                ThemePresetOptions {
                    base_family: "Fixture Font".into(),
                    ..Default::default()
                },
            )?
            .fonts(vec![TextFont {
                family: "Fixture Font".into(),
                face: "plain".into(),
                font: chart_core::services::ResourceDescriptor {
                    id: chart_core::ResourceId::new(1),
                    revision: chart_core::Revision::INITIAL,
                    kind: chart_core::services::ResourceKind::Font,
                    byte_len: include_bytes!("../../fixtures/capability/fonts/NotoSans-Regular.ttf")
                        .len() as u64,
                },
                weight: 400,
            }])
            .update_elements(
                ElementTheme::default().element(
                    "axis.text.x",
                    ThemeEntry::Element(
                        ThemeElement::new(ElementKind::Text)
                            .property("angle", ThemeValue::Number(35.))?,
                    ),
                )?,
            )?;
    }
    let mut builder = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(if mode == 12 {
            points().aes(aes().size("x"))
        } else {
            points()
        })
        .title(title(if mode == 14 {
            "Thème • Δοκιμή • Пример"
        } else {
            "Reference theme"
        }))
        .subtitle(subtitle("Inheritance and physical controls"))
        .caption(caption("Explicit shared text service"))
        .theme(themed);
    if mode == 12 {
        builder = builder
            .aes(aes().x("x").y("y").color("g").color_scale("groups"))
            .scale(color_discrete("groups").domain(["A", "B"]))
            .facet(facet_wrap("g").columns(2).reference(FacetPolicy::default()));
    }
    if mode == 14 {
        builder = builder.layer(points().text_defaults().text_label("g"));
    }
    if mode == 13 {
        builder = builder.coordinate(CoordinateSpec::Radial(RadialCoordinate::default()));
    }
    if mode == 15 {
        builder=builder.aes(aes().x("x").y("y").color("x").color_scale("values")).scale(color_mapped("values",serde_json::from_value(serde_json::json!({"training": "Eligible", "function": {"Interpolated": {"normalization": {"Sequential": {"family": "Linear", "domain": [1.0, 4.0], "clamp": false}}, "output": {"Interpolate": {"operation": "GgplotPalette", "spec": {"Gradient": {"colors": ["#112244", "#FFCC22"], "values": [0.0, 1.0]}}}}, "unknown": {"kind": "Missing"}}}, "ggplot": {"Continuous": {"limits": [1.0, 4.0], "empty_population": false, "nonfinite_population": false, "oob": "Censor"}}, "guide": {"Colorbar": {"breaks": [1.0, 2.0, 3.0, 4.0]}}})).expect("independent mapped colorbar author")));
    }
    builder.build()
}
