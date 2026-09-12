//! CLR-04 / COL-05: retained colors, one preparation boundary and legacy migration.
use chart_core::{
    color::{self, ColorDescriptor, ColorValue, Paint},
    grammar::{AfterScaleAesthetic, Compiler, Expression, ExpressionNode, ExpressionValue},
    prelude::*,
    scales::{Bounds, ColorScale},
    scene::Color,
    state::ChartState,
};
use std::sync::Arc;

fn bytes(r: u8) -> Color {
    Color {
        red: r,
        green: 0,
        blue: 0,
        alpha: 255,
    }
}
fn base() -> Plot {
    plot(
        Data::columns()
            .column("x", [0., 0.49, 1.])
            .column("y", [1., 2., 3.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .build()
    .unwrap()
}
#[test]
fn paint_retains_descriptors_and_legacy_bytes_with_strict_ingestion() {
    assert_eq!(std::mem::size_of::<Color>(), 4);
    // GG-03 adds independent paints, alpha, units and line type to the resolved
    // style. Colors remain four bytes; the expanded style has an explicit budget.
    assert_eq!(std::mem::size_of::<chart_core::grammar::Style>(), 56);
    for (css, expected) in [
        (
            "#12345678",
            Color {
                red: 18,
                green: 52,
                blue: 86,
                alpha: 120,
            },
        ),
        ("#ff0000", bytes(255)),
    ] {
        let paint = Paint::from_css(css).unwrap();
        assert!(!paint.is_floating());
        assert_eq!(paint.resolve(), expected);
        assert_eq!(
            serde_json::to_value(paint).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::from_value::<Paint>(serde_json::to_value(expected).unwrap()).unwrap(),
            paint
        );
    }
    for css in [
        "rebeccapurple",
        "hsl(120, 50%, 40%)",
        "transparent",
        "rgb(12.3%, 40%, 98%)",
    ] {
        let value = color::color(css).unwrap().unwrap();
        let paint = Paint::from_css(css).unwrap();
        assert!(paint.is_floating());
        assert_eq!(paint.value(), value);
        let wire = serde_json::to_string(&paint).unwrap();
        assert_eq!(serde_json::from_str::<Paint>(&wire).unwrap(), paint);
        assert_eq!(paint.resolve(), value.to_paint());
    }
    let signed = Paint::from(color::hcl(-0., -0., 50.));
    let wire = serde_json::to_string(&signed).unwrap();
    assert!(wire.contains("\"-0\""));
    let loaded = serde_json::from_str::<Paint>(&wire).unwrap();
    assert!(loaded.value().channel("h").unwrap().is_sign_negative());
    assert!(loaded.value().channel("c").unwrap().is_sign_negative());
    let exceptional =
        ColorValue::from(color::lab(f64::NAN, f64::INFINITY, f64::NEG_INFINITY)).with_opacity(-0.5);
    let paint = Paint::from(exceptional);
    let wire = serde_json::to_string(&paint).unwrap();
    assert!(wire.contains("NaN") && wire.contains("Infinity"));
    assert_eq!(serde_json::from_str::<Paint>(&wire).unwrap(), paint);
    assert_eq!(
        paint.resolve(),
        Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0
        }
    );
    for invalid in [
        "null",
        "true",
        "4",
        r#"{"red":1.5,"green":0,"blue":0,"alpha":255}"#,
        r#"{"red":0,"green":0,"blue":0,"alpha":255,"extra":0}"#,
        r#""currentColor""#,
    ] {
        assert!(serde_json::from_str::<Paint>(invalid).is_err(), "{invalid}");
    }
    let mut descriptor =
        serde_json::to_value(ColorDescriptor::new(color::hcl(30., 50., 60.))).unwrap();
    descriptor["version"] = 2.into();
    assert!(serde_json::from_value::<Paint>(descriptor.clone()).is_err());
    descriptor["version"] = 1.into();
    descriptor["extra"] = 0.into();
    assert!(serde_json::from_value::<Paint>(descriptor).is_err());
}

#[test]
fn floating_palette_avoids_endpoint_rounding_and_preserves_legacy_mapping() {
    let domain = Bounds::new(0., 1.).unwrap();
    let authored = ColorScale::Continuous {
        domain,
        palette: vec![
            Paint::from(color::rgb(0.49, 0., 0.)),
            Paint::from(color::rgb(1.01, 0., 0.)),
        ],
        clamp: false,
        missing: bytes(33).into(),
    };
    let prepared = authored.prepare().unwrap();
    // Independent arithmetic: 0.49 + 0.49*(1.01 - 0.49) = 0.7448, which rounds to one.
    assert_eq!(prepared.numeric(Some(0.49)).unwrap(), bytes(1));
    let legacy = ColorScale::Continuous {
        domain,
        palette: vec![bytes(0), bytes(1)],
        clamp: false,
        missing: bytes(33),
    };
    assert_eq!(legacy.numeric(Some(0.49)).unwrap(), bytes(0));
    let legacy_prepared = legacy.clone().map_colors(Paint::from).prepare().unwrap();
    for x in [
        None,
        Some(f64::NAN),
        Some(-1.),
        Some(0.),
        Some(0.49),
        Some(0.5),
        Some(1.),
        Some(2.),
    ] {
        assert_eq!(
            legacy_prepared.numeric(x).unwrap(),
            legacy.numeric(x).unwrap()
        );
    }
    assert_eq!(prepared.numeric(Some(-1.)).unwrap(), bytes(33));
    let legend = prepared.legend(chart_core::ScaleId::new(0), &[]).unwrap();
    for (i, (_, paint)) in legend.entries.iter().enumerate() {
        assert_eq!(*paint, prepared.numeric(Some(i as f64)).unwrap());
    }
    let p = plot(
        Data::columns()
            .column("x", [0., 0.49, 1.])
            .column("y", [1., 2., 3.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points().aes(aes().color("x").color_scale("ramp")))
    .scale(
        color_continuous("ramp", 0., 1.)
            .palette(vec![color::rgb(0.49, 0., 0.), color::rgb(1.01, 0., 0.)]),
    )
    .build()
    .unwrap();
    let chart = p.chart().unwrap().prepare().unwrap();
    assert_eq!(chart.layers()[0].marks()[1].style.color, bytes(1));
    assert_eq!(chart.definition().wire_version(), 4);
}

#[test]
fn every_retained_paint_location_requires_version_four_and_preserves_legacy_versions() {
    let old = base();
    let original = old.to_json().unwrap();
    assert_eq!(old.definition().wire_version(), 1);
    assert_eq!(
        Plot::from_json(&original).unwrap().to_json().unwrap(),
        original
    );
    let value = Paint::from(color::lab(60.25, -30.5, 40.75));
    let mut variants = vec![];
    let mut d = old.definition().clone();
    d.layers[0].style.color = value;
    variants.push(d);
    for field in [
        "background",
        "panel",
        "foreground",
        "grid",
        "mark",
        "annotation",
        "focus",
        "selection",
    ] {
        let mut d = old.definition().clone();
        let mut patch = serde_json::json!({});
        patch[field] = serde_json::to_value(value).unwrap();
        let mut theme =
            chart_core::theme::ThemeSpec::named(chart_core::theme::NamedTheme::Editorial);
        theme.plot = serde_json::from_value(patch).unwrap();
        d.theme = Some(theme);
        variants.push(d);
    }
    for endpoint in ["start", "end"] {
        let mut d = old.definition().clone();
        let mut theme =
            chart_core::theme::ThemeSpec::named(chart_core::theme::NamedTheme::Editorial);
        let mut gradient =
            serde_json::json!({"direction":"Horizontal","start":bytes(0),"end":bytes(255)});
        gradient[endpoint] = serde_json::to_value(value).unwrap();
        theme.plot.gradient = Some(serde_json::from_value(gradient).unwrap());
        d.theme = Some(theme);
        variants.push(d);
    }
    let mut text = chart_core::typography::RichText::plain("color");
    text.lines[0][0].color = Some(value);
    for slot in [
        "title",
        "subtitle",
        "caption",
        "source_notes",
        "footnotes",
        "panel_letters",
        "annotations",
    ] {
        let mut d = old.definition().clone();
        let mut f = chart_core::composition::FigureComposition::default();
        match slot {
            "title"=>f.title=Some(text.clone()), "subtitle"=>f.subtitle=Some(text.clone()), "caption"=>f.caption=Some(text.clone()),
            "source_notes"=>f.source_notes.push(text.clone()), "footnotes"=>f.footnotes.push(text.clone()),
            "panel_letters"=>f.panel_letters.push(serde_json::from_value(serde_json::json!({"panel":null,"text":text})).unwrap()),
            "annotations"=>f.annotations.push(serde_json::from_value(serde_json::json!({"id":"a","anchor":{"Output":{"x":20.,"y":20.}},"text":text,"overflow":false})).unwrap()),
            _=>unreachable!(),
        }
        d.figure = Some(f);
        variants.push(d);
    }
    let mut d = old.definition().clone();
    d.layers[0].after_scale.insert(
        AfterScaleAesthetic::Color,
        Expression {
            nodes: vec![ExpressionNode::Literal(ExpressionValue::Color(value))],
            output: 0,
        },
    );
    variants.push(d);
    for field in ["ink", "paper", "accent"] {
        let mut d = old.definition().clone();
        let mut theme =
            chart_core::theme::ThemeSpec::named(chart_core::theme::NamedTheme::Editorial);
        let mut geometry =
            serde_json::to_value(chart_core::theme::GeometryTheme::<Color>::default()).unwrap();
        geometry[field] = serde_json::to_value(value).unwrap();
        theme.geometry = Some(serde_json::from_value(geometry).unwrap());
        theme.version = 2;
        d.theme = Some(theme);
        variants.push(d);
    }
    let mut d = old.definition().clone();
    let mut theme = chart_core::theme::ThemeSpec::named(chart_core::theme::NamedTheme::Editorial);
    theme.layers.insert(
        d.layers[0].id,
        chart_core::theme::ThemePatch {
            mark: Some(value),
            ..Default::default()
        },
    );
    d.theme = Some(theme);
    variants.push(d);
    for title in [false, true] {
        let mut d = old.definition().clone();
        let mut axis = chart_core::layout::AxisSpec::new(
            chart_core::ScaleId::new(0),
            chart_core::layout::AxisSide::Bottom,
        );
        if title {
            axis.title = Some(text.clone());
        } else {
            axis.typography = Some(text.lines[0][0].clone());
        }
        d.axes.push(axis);
        variants.push(d);
    }
    for up in [false, true] {
        let mut d = old.definition().clone();
        d.layers[0].candle_colors = Some(chart_core::grammar::CandleColors {
            up: if up { value } else { bytes(0).into() },
            down: if up { bytes(0).into() } else { value },
        });
        variants.push(d);
    }
    for fill in [false, true] {
        let mut d = old.definition().clone();
        let mut path = path();
        path.rect(0., 0., 20., 20.).unwrap();
        d.figure = Some(chart_core::composition::FigureComposition {
            version: 2,
            paths: vec![chart_core::composition::VectorAnnotation {
                id: "p".into(),
                anchor: chart_core::composition::Anchor::Output { x: 0., y: 0. },
                geometry: path.geometry(),
                fill: fill.then_some(value),
                stroke: (!fill).then_some(chart_core::scene::Stroke {
                    color: value,
                    width: 1.,
                }),
                overflow: false,
            }],
            ..Default::default()
        });
        variants.push(d);
    }
    for missing in [false, true] {
        let mut d = old.definition().clone();
        d.layers[0].color = Some(chart_core::grammar::ColorEncoding {
            id: chart_core::ScaleId::new(2),
            title: None,
            input: chart_core::grammar::ColorInput::Group,
            scale: ColorScale::Discrete {
                domain: None,
                palette: vec![if missing { bytes(0).into() } else { value }],
                missing: if missing { value } else { bytes(0).into() },
            },
        });
        variants.push(d);
    }
    for definition in variants {
        assert_eq!(definition.wire_version(), 4);
        let encoded = chart_core::portable::encode(&chart_core::portable::ChartEnvelope {
            version: 4,
            definition: definition.clone(),
        })
        .unwrap();
        let decoded: chart_core::portable::ChartEnvelope =
            chart_core::portable::decode(&encoded).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded.definition, definition);
        for version in 1..=3 {
            assert!(
                chart_core::portable::ChartEnvelope {
                    version,
                    definition: definition.clone()
                }
                .validate()
                .is_err()
            );
        }
    }
    let changed = plot(old.data("data").unwrap().clone())
        .aes(aes().x("x").y("y"))
        .layer(points().color(value))
        .build()
        .unwrap();
    let wire = changed.to_json().unwrap();
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let mut malformed: serde_json::Value = serde_json::from_str(&wire).unwrap();
    malformed["version"] = 1.into();
    assert!(Plot::from_json(&malformed.to_string()).is_err());
}

#[test]
fn floating_color_only_edits_reuse_statistics_and_match_fresh_batch() {
    let p = base();
    let mut compiler = Compiler::new();
    let state = ChartState::default();
    let first = compiler
        .prepare(p.definition(), &p.source(), &state, p.compile_limits())
        .unwrap();
    let mut changed = p.definition().clone();
    changed.revision = chart_core::Revision::new(1);
    changed.layers[0].style.color = ColorValue::from(color::hcl(370., 65., 60.))
        .with_opacity(0.37)
        .into();
    let next = compiler
        .prepare(&changed, &p.source(), &state, p.compile_limits())
        .unwrap();
    let fresh = Compiler::new()
        .prepare(&changed, &p.source(), &state, p.compile_limits())
        .unwrap();
    assert!(Arc::ptr_eq(
        first.layers()[0].table(),
        next.layers()[0].table()
    ));
    assert_eq!(next.metrics().evaluated_layers, 0);
    assert_eq!(first.domains(), next.domains());
    assert_eq!(next.layers()[0].marks(), fresh.layers()[0].marks());
    assert_ne!(
        first.layers()[0].marks()[0].style.color,
        next.layers()[0].marks()[0].style.color
    );
    assert_eq!(first.definition(), p.definition());
    assert_eq!(next.definition(), &changed);
}

#[test]
fn perceptual_palette_changes_reuse_tables_targets_and_update_guides() {
    use chart_core::{
        interpolate::{FactoryKind, InterpolationFactory, Value},
        scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    };
    let mapped = |factory, end: &str| {
        ScaleConstructor::Linear
            .create(ScaleOptions {
                range: Some(vec![
                    Value::Text("rgba(255, 30, 30, 0.5)".into()),
                    Value::Text(end.into()),
                ]),
                factory: Some(InterpolationFactory::new(factory)),
                ..Default::default()
            })
            .unwrap()
            .mapped(ScaleTraining::Authored)
            .unwrap()
    };
    let p = plot(
        Data::columns()
            .column("x", [0., 0.25, 0.5, 0.75, 1.])
            .column("y", [1., 2., 3., 4., 5.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y").color("x").color_scale("ramp"))
    .layer(points())
    .scale(color_mapped(
        "ramp",
        mapped(FactoryKind::Lab, "rgba(30, 90, 255, 0.5)"),
    ))
    .legend(legend().scale("ramp"))
    .build()
    .unwrap();
    let source = p.source();
    let state = ChartState::default();
    let mut compiler = Compiler::new();
    let first = compiler
        .prepare(p.definition(), &source, &state, p.compile_limits())
        .unwrap();
    let mut previous = first.clone();
    for (i, factory) in [FactoryKind::Hcl, FactoryKind::Cubehelix, FactoryKind::Lab]
        .into_iter()
        .enumerate()
    {
        let mut d = p.definition().clone();
        d.revision = chart_core::Revision::new(i as u64 + 1);
        if let ColorScale::Mapped { scale, .. } = &mut d.layers[0].color.as_mut().unwrap().scale {
            *scale = mapped(factory, "rgba(30, 230, 90, 0.5)");
        } else {
            panic!("Expected mapped color");
        }
        let next = compiler
            .prepare(&d, &source, &state, p.compile_limits())
            .unwrap();
        let fresh = Compiler::new()
            .prepare(&d, &source, &state, p.compile_limits())
            .unwrap();
        assert_eq!(next.metrics().evaluated_layers, 0);
        assert!(Arc::ptr_eq(
            first.layers()[0].table(),
            next.layers()[0].table()
        ));
        assert_eq!(next.domains(), first.domains());
        assert_eq!(next.scale_domains(), first.scale_domains());
        assert_eq!(next.layers()[0].marks(), fresh.layers()[0].marks());
        assert_eq!(
            next.layers()[0].color_legend(),
            fresh.layers()[0].color_legend()
        );
        assert_ne!(
            next.layers()[0].color_legend(),
            previous.layers()[0].color_legend()
        );
        for (a, b) in first.layers()[0]
            .marks()
            .iter()
            .zip(next.layers()[0].marks())
        {
            assert_eq!(a.geometry, b.geometry);
            assert_eq!(a.targets, b.targets);
            assert_eq!(b.style.color.alpha, 128);
        }
        previous = next;
    }
    assert_eq!(first.definition(), p.definition());
}
