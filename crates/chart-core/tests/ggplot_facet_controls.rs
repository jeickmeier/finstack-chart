//! GG12 pinned panel catalogs, memberships and scale-sharing groups.
use chart_core::{
    grammar::{FacetPolicy, GroupValue},
    prelude::*,
};
fn data() -> Data {
    Data::columns()
        .column("x", [1., 2., 1., 4., 1., 10., 2.])
        .column("y", [1., 3., 2., 8., 10., 100., 4.])
        .column(
            "r",
            [
                Some("B"),
                Some("B"),
                Some("A"),
                Some("A"),
                Some("B"),
                Some("B"),
                None,
            ],
        )
        .column("nested", ["u", "u", "v", "v", "w", "w", "u"])
        .column("c", ["L", "L", "R", "R", "R", "R", "L"])
        .column("z", ["z1", "z2", "z1", "z2", "z1", "z2", "z1"])
        .build()
        .unwrap()
}
fn levels(values: &[&str]) -> Option<Vec<GroupValue>> {
    Some(
        values
            .iter()
            .map(|s| GroupValue::Text((*s).into()))
            .collect(),
    )
}
#[test]
fn pinned_multivariable_catalog_nesting_na_and_unused_levels() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/facet-controls.json"
    ))
    .unwrap();
    for (drop, wrap, n) in [
        (true, true, 4),
        (false, true, 12),
        (true, false, 16),
        (false, false, 72),
    ] {
        let policy = FacetPolicy {
            row_fields: 2,
            drop,
            levels: if wrap {
                vec![levels(&["B", "A", "unused"]), levels(&["u", "v", "w"])]
            } else {
                vec![
                    levels(&["B", "A", "unused"]),
                    levels(&["u", "v", "w"]),
                    levels(&["R", "L", "unused"]),
                    levels(&["z1", "z2"]),
                ]
            },
            ..Default::default()
        };
        let facet = if wrap {
            facet_wrap("r").fields(["r", "nested"])
        } else {
            facet_grid("r", "c").fields(["r", "nested", "c", "z"])
        }
        .reference(policy);
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet)
            .build()
            .unwrap();
        let name = format!(
            "{}-drop-{}",
            if wrap {
                "wrap-multiple"
            } else {
                "grid-nested-crossed"
            },
            if drop { "TRUE" } else { "FALSE" }
        );
        let expected = source["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        assert_eq!(expected["layout"].as_array().unwrap().len(), n);
        let q = p.chart().unwrap().prepare().unwrap();
        assert_eq!(q.panels().len(), n);
        assert!(
            q.panels()
                .iter()
                .any(|p| p.key.values.contains(&GroupValue::Missing))
        );
    }
}
#[test]
fn reference_grid_free_axes_share_columns_and_rows() {
    let p = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_grid("r", "c").free_x(true).free_y(true))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.panels().len(), 6);
    for a in q.panels() {
        for b in q.panels() {
            if a.column == b.column {
                assert_eq!(a.x_group, b.x_group);
                assert_eq!(
                    a.chart.scale_domains().get(&a.chart.layers()[0].scales().x),
                    b.chart.scale_domains().get(&b.chart.layers()[0].scales().x)
                );
            }
            if a.row == b.row {
                assert_eq!(a.y_group, b.y_group);
            }
        }
    }
}
#[test]
fn margins_are_typed_and_nested_source_populations_repeat_once() {
    for (margins, n) in [(vec![0, 1, 2], 24), (vec![0], 10), (vec![0, 1], 16)] {
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(
                facet_grid("r", "c")
                    .fields(["r", "nested", "c"])
                    .reference(FacetPolicy {
                        row_fields: 2,
                        margins,
                        levels: vec![
                            levels(&["B", "A", "unused"]),
                            levels(&["u", "v", "w"]),
                            levels(&["R", "L", "unused"]),
                        ],
                        ..Default::default()
                    }),
            )
            .build()
            .unwrap();
        let q = p.chart().unwrap().prepare().unwrap();
        assert_eq!(q.panels().len(), n);
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/parity/ggplot2/facet-controls.json"
        ))
        .unwrap();
        let name = match n {
            24 => "margins-TRUE",
            10 => "margins-r",
            _ => "margins-r-nested",
        };
        let expected = source["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        for (panel, row) in q
            .panels()
            .iter()
            .zip(expected["layout"].as_array().unwrap())
        {
            for (i, key) in ["r", "nested", "c"].iter().enumerate() {
                let value = &row[*key];
                let expected = if value.is_null() {
                    GroupValue::Missing
                } else if value == "(all)" {
                    GroupValue::All
                } else {
                    GroupValue::Text(value.as_str().unwrap().into())
                };
                assert_eq!(panel.key.values[i], expected, "{name}");
            }
        }
        for panel in q.panels() {
            if panel.key.values.iter().all(|v| *v == GroupValue::All) {
                assert_eq!(panel.chart.layers()[0].marks().len(), 7);
            }
        }
    }
    let d = Data::columns()
        .column("x", [1., 2.])
        .column("y", [1., 2.])
        .column("r", ["(all)", "a"])
        .build()
        .unwrap();
    let p = plot(d)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("r").reference(FacetPolicy {
            margins: vec![0],
            ..Default::default()
        }))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.panels().len(), 3);
    assert!(
        q.panels()
            .iter()
            .any(|p| p.key.values == [GroupValue::Text("(all)".into())])
    );
    assert!(q.panels().iter().any(|p| p.key.values == [GroupValue::All]));
}
#[test]
fn partial_fields_broadcast_by_name_and_add_new_live_catalog_levels() {
    let annotation = Data::columns()
        .name("annotation")
        .column("r", ["C"])
        .column("y", [9.])
        .column("x", [9.])
        .build()
        .unwrap();
    let p = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(points().data(annotation))
        .facet(facet_grid("r", "c"))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.panels().len(), 8);
    let added = q
        .panels()
        .iter()
        .filter(|p| p.key.values[0] == GroupValue::Text("C".into()))
        .collect::<Vec<_>>();
    assert_eq!(added.len(), 2);
    for panel in added {
        assert_eq!(panel.chart.layers()[0].marks().len(), 0);
        assert_eq!(panel.chart.layers()[1].marks().len(), 1);
    }
}
#[test]
fn shrink_false_retains_source_ranges_before_summary() {
    use chart_core::grammar::SummaryHelper;
    for shrink in [true, false] {
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .layer(
                points().stat(
                    summary()
                        .x(1.)
                        .y("y")
                        .summary_helper(SummaryHelper::default()),
                ),
            )
            .facet(facet_wrap("c").free_y(true).reference(FacetPolicy {
                shrink,
                ..Default::default()
            }))
            .build()
            .unwrap();
        let q = p.chart().unwrap().prepare().unwrap();
        let right = q
            .panels()
            .iter()
            .find(|p| p.key.values == [GroupValue::Text("R".into())])
            .unwrap();
        let d = right
            .chart
            .scale_domains()
            .get(&right.chart.layers()[0].scales().y)
            .unwrap()
            .y
            .unwrap();
        if shrink {
            assert_eq!(d.minimum, 30.);
            assert_eq!(d.maximum, 30.);
        } else {
            assert_eq!(d.minimum, 2.);
            assert_eq!(d.maximum, 100.);
        }
    }
}
#[test]
fn author_profile_infers_reference_but_old_definition_keeps_legacy() {
    let d = Data::columns()
        .column("x", [1., 10., 2., 20.])
        .column("y", [1., 2., 3., 4.])
        .column("r", ["a", "b", "a", "b"])
        .column("c", ["a", "a", "b", "b"])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_grid("r", "c").free_x(true))
        .build()
        .unwrap();
    assert!(p.definition().facets.as_ref().unwrap().reference.is_some());
    let mut old = p.definition().clone();
    old.facets.as_mut().unwrap().reference = None;
    let serialized = serde_json::to_value(&old).unwrap();
    assert!(serialized["facets"].get("reference").is_none());
    let replay: chart_core::grammar::ChartDefinition = serde_json::from_value(serialized).unwrap();
    assert!(replay.facets.as_ref().unwrap().reference.is_none());
    let source = p.source();
    let mut compiler = chart_core::grammar::Compiler::default();
    let q = compiler
        .prepare(&replay, &source, &Default::default(), Default::default())
        .unwrap();
    let a = &q.panels()[0];
    let b = &q.panels()[2];
    assert_eq!(a.column, b.column);
    assert_ne!(a.x_group, b.x_group);
}
#[test]
fn six_wrap_directions_match_pinned_panel_coordinates() {
    use chart_core::grammar::FacetDirection::*;
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/facet-controls.json"
    ))
    .unwrap();
    for (name, direction) in [
        ("h", Lt),
        ("v", Tl),
        ("tr", Tr),
        ("rt", Rt),
        ("bl", Bl),
        ("lb", Lb),
    ] {
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(
                facet_wrap("r")
                    .fields(["r", "c"])
                    .columns(2)
                    .reference(FacetPolicy {
                        direction,
                        levels: vec![levels(&["B", "A", "unused"]), levels(&["R", "L", "unused"])],
                        ..Default::default()
                    }),
            )
            .build()
            .unwrap();
        let q = p.chart().unwrap().prepare().unwrap();
        let case = source["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == format!("wrap-direction-{name}"))
            .unwrap();
        let expected = case["layout"].as_array().unwrap();
        assert_eq!(q.panels().len(), expected.len());
        for (p, e) in q.panels().iter().zip(expected) {
            assert_eq!(
                (p.row + 1, p.column + 1),
                (
                    e["ROW"].as_u64().unwrap() as usize,
                    e["COL"].as_u64().unwrap() as usize
                ),
                "{name}"
            );
        }
    }
}
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
            font_size: r.font_size,
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
fn free_space_widths_follow_expanded_spans_at_two_output_sizes() {
    use chart_core::grammar::FacetSpace;
    let p = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(
            facet_grid("r", "c")
                .fields(["c"])
                .free_x(true)
                .reference(FacetPolicy {
                    row_fields: 0,
                    space: FacetSpace::FreeX,
                    ..Default::default()
                }),
        )
        .build()
        .unwrap();
    for width in [800., 1200.] {
        let q = draw(&p, width);
        let right = q
            .panels()
            .iter()
            .find(|p| p.key.values == [GroupValue::Text("R".into())])
            .unwrap();
        let left = q
            .panels()
            .iter()
            .find(|p| p.key.values == [GroupValue::Text("L".into())])
            .unwrap();
        let ratio = right.chart.plot().unwrap().width() / left.chart.plot().unwrap().width();
        assert!((ratio - 9.).abs() < 1e-10, "{ratio}");
    }
}
#[test]
fn interior_axes_and_labels_are_independent_from_scale_training() {
    use chart_core::grammar::FacetAxes;
    for axes in [FacetAxes::Margins, FacetAxes::All] {
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(
                facet_wrap("r")
                    .fields(["r", "c"])
                    .columns(2)
                    .reference(FacetPolicy {
                        axes,
                        axis_labels: FacetAxes::Margins,
                        ..Default::default()
                    }),
            )
            .build()
            .unwrap();
        let q = draw(&p, 900.);
        let top = &q.panels()[0];
        let bottom = top
            .chart
            .guides()
            .values()
            .find(|g| g.spec.side == chart_core::layout::AxisSide::Bottom)
            .unwrap();
        assert_eq!(bottom.spec.style.visible, axes == FacetAxes::All);
        if axes == FacetAxes::All {
            assert_eq!(
                bottom
                    .spec
                    .style
                    .components
                    .as_ref()
                    .unwrap()
                    .labels
                    .visible,
                Some(false)
            );
        }
    }
}
#[test]
fn four_strip_sides_reserve_measured_space_and_preserve_logical_labels() {
    use chart_core::{grammar::FacetStripPosition::*, scene::Primitive};
    for side in [Top, Bottom, Left, Right] {
        let p = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_wrap("r").reference(FacetPolicy {
                strip_position: side,
                ..Default::default()
            }))
            .build()
            .unwrap();
        let q = draw(&p, 900.);
        let backgrounds = q
            .scene()
            .items()
            .iter()
            .filter_map(|i| match i.primitive {
                Primitive::Rectangle { bounds, fill }
                    if fill.red == 217 && fill.green == 217 && fill.blue == 217 =>
                {
                    Some(bounds)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(backgrounds.len(), q.panels().len());
        for (b, p) in backgrounds.iter().zip(q.panels()) {
            let plot = p.chart.plot().unwrap();
            match side {
                Top => assert!(b.origin().y() + b.height() <= plot.origin().y()),
                Bottom => assert!(b.origin().y() >= plot.origin().y() + plot.height()),
                Left => assert!(b.origin().x() + b.width() <= plot.origin().x()),
                Right => assert!(b.origin().x() >= plot.origin().x() + plot.width()),
            }
        }
    }
}
#[test]
fn source_updates_recompute_catalogs_and_keep_original_panel_keys() {
    fn d(values: &[&str], keys: &[u64]) -> Data {
        Data::columns()
            .column("x", vec![1.; values.len()])
            .column("y", vec![2.; values.len()])
            .column("facet", values.to_vec())
            .keys(keys.iter().copied())
            .build()
            .unwrap()
    }
    let source = d(&["a", "b"], &[1, 2]);
    let p = plot(source.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("facet"))
        .build()
        .unwrap();
    let mut chart = p.chart().unwrap();
    let old = chart.prepare().unwrap();
    let tx = chart
        .transaction()
        .unwrap()
        .append(&source, d(&["c"], &[3]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        chart_core::transaction::CommitOutcome::Applied(_)
    ));
    let updated = chart.prepare().unwrap();
    assert_eq!(old.panels().len(), 2);
    assert_eq!(updated.panels().len(), 3);
    let batch = plot(d(&["a", "b", "c"], &[1, 2, 3]))
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("facet"))
        .build()
        .unwrap();
    let batch = batch.chart().unwrap().prepare().unwrap();
    for (a, b) in updated.panels().iter().zip(batch.panels()) {
        assert_eq!(a.key, b.key);
        assert_eq!(
            a.chart.layers()[0].marks().len(),
            b.chart.layers()[0].marks().len()
        );
        assert_eq!(a.chart.scale_domains(), b.chart.scale_domains());
    }
    assert_eq!(old.panels()[0].key, updated.panels()[0].key);
}
struct Labeller;
impl chart_core::grammar::CustomGuideFormatter for Labeller {
    fn descriptor(&self) -> chart_core::grammar::ExtensionDescriptor {
        chart_core::grammar::ExtensionDescriptor::batch(
            "test.facet-label",
            chart_core::Revision::new(1),
            true,
        )
    }
    fn validate(&self, _: &serde_json::Value) -> chart_core::ChartResult<()> {
        Ok(())
    }
    fn format_labels(
        &self,
        input: chart_core::grammar::GuideLabelsInput<'_>,
    ) -> chart_core::ChartResult<Vec<Option<String>>> {
        Ok(input
            .names
            .unwrap()
            .iter()
            .map(|name| Some(format!("registered {name}")))
            .collect())
    }
}
#[test]
fn registered_strip_labeller_reuses_checked_vector_registration() {
    use chart_core::grammar::{
        ExtensionRegistry, FacetLabelOperation, FacetLabeller, OperationRef,
    };
    let mut registry = ExtensionRegistry::new();
    registry
        .register_guide_formatter(std::sync::Arc::new(Labeller))
        .unwrap();
    let p = plot(data())
        .extensions(std::sync::Arc::new(registry))
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("r").reference(FacetPolicy {
            labeller: FacetLabeller {
                registered: Some(FacetLabelOperation {
                    operation: OperationRef::new("test.facet-label", chart_core::Revision::new(1)),
                    parameters: serde_json::Value::Null,
                }),
                ..Default::default()
            },
            ..Default::default()
        }))
        .build()
        .unwrap();
    let q = draw(&p, 900.);
    assert!(q.scene().items().iter().any(|i|matches!(&i.primitive,chart_core::scene::Primitive::Text{text,..}if text=="registered r")));
    assert!(p.to_json().is_ok());
}
#[test]
fn generic_facet_host_controls_roundtrip_with_capability_version() {
    use chart_core::plot::host::{Component, Draft};
    let data = data();
    let aes = Component::new("aes", "[]")
        .unwrap()
        .set("x", r#"["x"]"#)
        .unwrap()
        .set("y", r#"["y"]"#)
        .unwrap();
    let facet=Component::new("facet_grid",r#"["r","c"]"#).unwrap().set("fields",r#"[["r","nested","c"]]"#).unwrap().set("row_fields","[2]").unwrap().set("reference",r#"[{"row_fields":2,"margins":[0],"shrink":false,"direction":"Tr","space":"FreeX","axes":"All","axis_labels":"Margins"}]"#).unwrap();
    let p = Draft::new(&data)
        .with("aes", &aes)
        .unwrap()
        .with("layer", &Component::new("points", "[]").unwrap())
        .unwrap()
        .with("facet", &facet)
        .unwrap()
        .build()
        .unwrap();
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 76);
    let restored = Plot::from_json(&wire).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    assert_eq!(
        restored.chart().unwrap().prepare().unwrap().panels().len(),
        10
    );
}
#[test]
fn switched_strips_follow_pinned_order_between_axes_and_plot() {
    use chart_core::{grammar::FacetSwitch, layout::AxisSide, scene::Primitive};
    let d = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("y", [1., 2., 3., 4.])
        .column("r", ["A", "A", "B", "B"])
        .column("c", ["L", "R", "L", "R"])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_grid("r", "c").reference(FacetPolicy {
            switch: FacetSwitch::Both,
            ..Default::default()
        }))
        .build()
        .unwrap();
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/facet-edge-controls.json"
    ))
    .unwrap();
    let case = source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "grid-switch-both")
        .unwrap();
    assert_eq!(case["draw"]["ok"], true);
    let q = draw(&p, 1000.);
    for panel in q.panels() {
        let plot = panel.chart.plot().unwrap();
        let guides = panel.chart.guides();
        for (item, key) in q.scene().items().iter().zip(q.item_panels()) {
            if key.as_ref() != Some(&panel.key) {
                continue;
            }
            let Primitive::Rectangle { bounds, fill } = item.primitive else {
                continue;
            };
            if fill.red != 217 || fill.green != 217 || fill.blue != 217 {
                continue;
            }
            if (bounds.max_x() - plot.origin().x()).abs() < 1e-9 {
                assert_eq!(bounds.height(), plot.height());
                let guide = guides
                    .values()
                    .find(|g| g.spec.side == AxisSide::Left)
                    .unwrap();
                assert!(
                    (plot.origin().x() + guide.spec.translation[0] - bounds.origin().x()).abs()
                        < 1e-9
                );
            } else {
                assert_eq!(bounds.width(), plot.width());
                assert!((bounds.origin().y() - plot.max_y()).abs() < 1e-9);
                let guide = guides
                    .values()
                    .find(|g| g.spec.side == AxisSide::Bottom)
                    .unwrap();
                assert!((plot.max_y() + guide.spec.translation[1] - bounds.max_y()).abs() < 1e-9);
            }
        }
    }
}

#[test]
fn reference_empty_panels_remain_blank_with_no_data_status() {
    let source = include_str!("../../../fixtures/parity/ggplot2/facet-controls.json");
    assert!(!source.contains("No data"));
    use chart_core::layout::LayoutStatus;
    for reference in [false, true] {
        let facet = facet_wrap("r")
            .order(
                ["A", "empty"]
                    .map(|v| chart_core::grammar::PanelKey {
                        values: vec![GroupValue::Text(v.into())],
                    })
                    .to_vec(),
            )
            .columns(2);
        let facet = if reference {
            facet.reference(FacetPolicy::default())
        } else {
            facet
        };
        let p = plot(
            Data::columns()
                .column("x", [1.])
                .column("y", [2.])
                .column("r", ["A"])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet)
        .build()
        .unwrap();
        let frame = draw(&p, 800.);
        let empty = frame
            .panels()
            .iter()
            .find(|p| p.key.values == [GroupValue::Text("empty".into())])
            .unwrap();
        assert_eq!(empty.chart.status(), LayoutStatus::NoData);
        let scene = serde_json::to_string(empty.chart.scene().items()).unwrap();
        assert_eq!(scene.contains("No data"), !reference);
        assert!(empty.chart.plot().is_some());
    }
}
