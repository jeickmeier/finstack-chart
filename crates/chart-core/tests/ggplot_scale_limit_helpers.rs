//! GG2-03/FIX-GG04: helper layers train mapped positions without painted marks.
use chart_core::{grammar::*, prelude::*};
use serde_json::{Value as Json, json};

fn equivalent(a: &Json, b: &Json) -> bool {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => a.as_f64() == b.as_f64(),
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
        }
        _ => a == b,
    }
}

fn limits(d: &DomainContributions, x: bool) -> Json {
    let space = if x { &d.x_space } else { &d.y_space };
    match space {
        Some(ValueSpace::Categorical { categories }) => {
            let mut values = categories.clone();
            values.sort();
            json!(values)
        }
        Some(ValueSpace::NullableCategorical { categories }) => {
            let mut values = categories.clone();
            values.sort();
            json!(values)
        }
        _ => {
            let v = if x { d.x } else { d.y }.unwrap();
            json!([v.minimum, v.maximum])
        }
    }
}

struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn rendered_checks(
    prepared: PreparedChart,
    expected: &[Json],
    category: bool,
    no_legend: bool,
    colors: &[chart_core::scene::Color],
) {
    use chart_core::{Rect, ResourceId, Revision, layout::*, services::*};
    let frame = layout(
        std::sync::Arc::new(prepared),
        &LayoutRequest::new(
            Rect::new(0., 0., 640., 360.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        ),
        &Metrics,
    )
    .unwrap();
    if no_legend {
        assert!(!frame.scene().items().iter().any(|item| item.layer.is_none() && matches!(&item.primitive, chart_core::scene::Primitive::Rectangle {fill,..} if colors.contains(fill))), "blank-only color must not create a legend");
    }
    if !category {
        return;
    }
    let panels = if frame.panels().is_empty() {
        vec![&frame]
    } else {
        frame.panels().iter().map(|p| p.chart.as_ref()).collect()
    };
    for (panel, expected) in panels.iter().zip(expected) {
        for (axis, side) in [("x", AxisSide::Bottom), ("y", AxisSide::Left)] {
            let guide = panel
                .guides()
                .values()
                .find(|g| g.spec.side == side)
                .unwrap();
            let mut ticks = guide.ticks.iter().collect::<Vec<_>>();
            ticks.sort_by(|a, b| {
                if axis == "x" {
                    a.position.total_cmp(&b.position)
                } else {
                    b.position.total_cmp(&a.position)
                }
            });
            assert_eq!(
                json!(ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>()),
                expected[axis]
            );
        }
    }
}

#[test]
fn blank_expansion_layers_match_pinned_domains_without_geometry() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/scale-limit-helpers.json"
    ))
    .unwrap();
    let cases = fixture["expansions"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    for c in cases {
        let category = c["family"] == "category";
        let data = Data::columns()
            .column(
                "x",
                if category {
                    categorical(["b", "c", "b", "c"])
                } else {
                    vec![1., 2., 3., 4.].into()
                },
            )
            .column(
                "y",
                if category {
                    categorical(["b", "c", "b", "c"])
                } else {
                    vec![2., 4., 6., 8.].into()
                },
            )
            .column("g", categorical(["A", "A", "B", "B"]))
            .build()
            .unwrap();
        let count = c["result"]["helper_rows"].as_u64().unwrap() as usize;
        let mut extra = Data::columns().name("expansion");
        let mut mapping = aes();
        for (name, values) in c["arguments"].as_object().unwrap() {
            let values = values.as_array().unwrap();
            let values = (0..count)
                .map(|i| &values[i % values.len()])
                .collect::<Vec<_>>();
            extra = extra.column(
                name,
                if category {
                    categorical(values.into_iter().map(|v| v.as_str().unwrap()))
                } else {
                    values
                        .into_iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect::<Vec<_>>()
                        .into()
                },
            );
            mapping = match name.as_str() {
                "x" => mapping.x("x"),
                "y" => mapping.y("y"),
                "colour" => mapping.color("colour"),
                _ => unreachable!(),
            };
        }
        let base = if c["axes"] == "colour_shared" {
            aes().x("x").y("y").color("y")
        } else {
            aes().x("x").y("y")
        };
        let mut p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(base)
            .layer(points())
            .layer(
                blank()
                    .data(extra.build().unwrap())
                    .independent()
                    .aes(mapping)
                    .facet_target(FacetTarget::Broadcast),
            );
        if c["facet"] != "none" {
            p = p.facet(
                facet_wrap("g")
                    .free_x(c["facet"] == "free")
                    .free_y(c["facet"] == "free"),
            );
        }
        let p = p.build().unwrap_or_else(|e| panic!("{c}: {e:?}"));
        let version = if c["axes"].as_str().unwrap().starts_with("colour") {
            45
        } else {
            41
        };
        assert_eq!(p.definition().wire_version(), version);
        let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
        assert_eq!(restored.definition(), p.definition());
        for candidate in [40, 41, 42, 43, 44, 45] {
            let envelope = chart_core::portable::ChartEnvelope {
                version: candidate,
                definition: p.definition().clone(),
            };
            assert_eq!(envelope.validate().is_ok(), candidate == version);
        }
        let prepared = Compiler::new()
            .prepare(
                p.definition(),
                &p.source(),
                &chart_core::state::ChartState::default(),
                CompileLimits::default(),
            )
            .unwrap_or_else(|e| panic!("{c}: {e:?}"));
        let panels = if prepared.panels().is_empty() {
            vec![&prepared]
        } else {
            prepared.panels().iter().map(|p| p.chart.as_ref()).collect()
        };
        let expected = c["result"]["panels"].as_array().unwrap();
        assert_eq!(panels.len(), expected.len());
        let mut painted = 0;
        let mut colors = vec![];
        let mut point_colors = vec![];
        for (panel, expected) in panels.iter().zip(expected) {
            for (axis, x) in [("x", true), ("y", false)] {
                let domain = if c["facet"] == "fixed" {
                    let scales = p.definition().layers[0].scales;
                    &prepared.scale_domains()[&if x { scales.x } else { scales.y }]
                } else {
                    panel.domains()
                };
                assert!(
                    equivalent(&limits(domain, x), &expected[axis]),
                    "{axis}: {c}: {:?}",
                    limits(domain, x)
                );
            }
            assert_eq!(panel.layers().len(), 2);
            assert!(panel.layers()[1].marks().is_empty(), "{c}");
            assert_eq!(panel.layers()[1].invalid_geometry(), 0, "{c}");
            if c["axes"] == "colour" || c["axes"] == "colour_shared" {
                let legend = panel.layers()[1]
                    .color_legend()
                    .expect("blank color training");
                colors.extend(legend.entries.iter().map(|v| v.1));
                let labels = if category {
                    json!(
                        legend
                            .entries
                            .iter()
                            .map(|v| v.0.as_str())
                            .collect::<Vec<_>>()
                    )
                } else {
                    json!(
                        legend
                            .numeric_breaks
                            .iter()
                            .map(|v| v.label.as_deref().unwrap_or(""))
                            .collect::<Vec<_>>()
                    )
                };
                assert_eq!(labels, c["result"]["colour_labels"], "{c}: {legend:?}");
            }

            painted += panel.layers()[0].marks().len();
            point_colors.extend(
                panel.layers()[0]
                    .marks()
                    .iter()
                    .map(|mark| mark.style.color),
            );
        }
        assert_eq!(painted, 4, "{c}");
        if c["axes"] == "colour_shared" {
            let expected = c["result"]["point_colours"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    chart_core::color::parse_r(value.as_str().unwrap())
                        .unwrap()
                        .resolve()
                })
                .collect::<Vec<_>>();
            assert_eq!(point_colors, expected, "{c}");
        }
        // Prepared catalogs preserve source ordinals. The destination scale sorts
        // character keys; verify the actual resolved guide order separately.
        rendered_checks(
            prepared,
            expected,
            category,
            c["result"]["guide_boxes"] == 0,
            &colors,
        );
    }
}

#[test]
fn blank_secondary_coordinates_train_independently_without_missing_row_diagnostics() {
    let data = Data::columns()
        .column("x", vec![1., 2.])
        .column("y", vec![2., 4.])
        .build()
        .unwrap();
    let extra = Data::columns()
        .name("bounds")
        .column("right", vec![-2., 10.])
        .column("low", vec![-5., 0.])
        .column("high", vec![12., 8.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(
            blank()
                .data(extra)
                .independent()
                .aes(aes().x2("right").low("low").high("high")),
        )
        .build()
        .unwrap();
    let prepared = Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &chart_core::state::ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert!(equivalent(
        &limits(prepared.domains(), true),
        &json!([-2, 10])
    ));
    assert!(equivalent(
        &limits(prepared.domains(), false),
        &json!([-5, 12])
    ));
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    assert!(prepared.layers()[1].marks().is_empty());
    assert_eq!(prepared.layers()[1].invalid_geometry(), 0);
}

#[test]
fn automatic_palette_edits_survive_appended_fields() {
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [2., 4.])
        .column("first", categorical(["a", "b"]))
        .column("second", categorical(["b", "c"]))
        .build()
        .unwrap();
    let original = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("first"))
        .layer(points())
        .build()
        .unwrap();
    let palette = vec![rgb(255, 0, 0), rgb(0, 255, 0), rgb(0, 0, 255)];
    let edited = original
        .edit()
        .scale(color_discrete("first").palette(palette.clone()))
        .layer("second", points().aes(aes().x("x").y("y").color("second")))
        .build()
        .unwrap();
    let colors = edited
        .definition()
        .layers
        .iter()
        .map(|l| l.color.as_ref().unwrap())
        .collect::<Vec<_>>();
    assert!(colors.iter().all(|c| c.automatic));
    assert_eq!(colors[0].id, colors[1].id);
    assert_eq!(colors[0].scale, colors[1].scale);
    assert_ne!(
        original.definition().layers[0]
            .color
            .as_ref()
            .unwrap()
            .scale,
        colors[0].scale
    );
    let mut chart = edited.chart().unwrap();
    let prepared = chart.prepare().unwrap();
    assert_eq!(
        prepared.layers()[0].marks()[1].style.color,
        prepared.layers()[1].marks()[0].style.color
    );
    assert_eq!(
        prepared.layers()[1]
            .color_legend()
            .unwrap()
            .entries
            .iter()
            .map(|v| v.1)
            .collect::<Vec<_>>(),
        palette
    );
}
