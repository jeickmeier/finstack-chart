//! HIERARCHY-07: destination recipes retain structural provenance and actual path hits.
use chart_core::{
    Point,
    grammar::{HierarchyProjection, HierarchyRadius},
    inspection::{InputOrigin, InspectionAction, InspectionMode, Inspector},
    plot::LayerBuilder,
    prelude::*,
    provenance::Target,
};
use chart_export::*;
fn output() -> Output {
    Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap()
}
fn data() -> Data {
    Data::columns()
        .column("id", ["root", "a", "b", "c"])
        .column("parent", vec![None, Some("root"), Some("root"), Some("a")])
        .column("value", [0., 2., 3., 5.])
        .build()
        .unwrap()
}
fn capture(layer: LayerBuilder) -> FigureSnapshot {
    let plot = plot(data())
        .layer(layer.hierarchy_value("value").hierarchy_label("id"))
        .build()
        .unwrap();
    output()
        .request(
            &plot,
            export_options(PageSize::points(400., 300.).unwrap()).dpi(72),
        )
        .unwrap()
        .prepare()
        .unwrap()
}
#[test]
fn every_recipe_exports_and_retains_exact_structural_membership() {
    for layer in [
        hierarchy_tree("id", "parent"),
        hierarchy_cluster("id", "parent"),
        hierarchy_icicle("id", "parent"),
        hierarchy_sunburst("id", "parent"),
        hierarchy_treemap("id", "parent"),
        hierarchy_pack("id", "parent"),
    ] {
        let figure = capture(layer);
        let snapshots = figure.layout().hierarchy_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);
        let h = &snapshots[0];
        assert_eq!(h.nodes.len(), 4);
        assert_eq!(h.nodes[0].value, Some(10.));
        assert_eq!(h.source_keys.len(), 4);
        let Target::HierarchyNode { membership, .. } = &h.targets[0] else {
            panic!("structural target")
        };
        assert_eq!(membership.members().len(), 4);
        let scene: serde_json::Value = serde_json::from_str(&figure.scene_json().unwrap()).unwrap();
        assert_eq!(scene["hierarchies"]["version"], 1);
        assert_eq!(
            scene["hierarchies"]["snapshots"][0]["source_keys"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert!(!figure.export(Format::Svg).unwrap().bytes.is_empty());
        assert!(
            figure
                .export(Format::Pdf)
                .unwrap()
                .bytes
                .starts_with(b"%PDF-")
        );
        assert!(
            figure
                .export(Format::Png)
                .unwrap()
                .bytes
                .starts_with(b"\x89PNG")
        );
        let mut inspector = Inspector::new(figure.layout().clone(), 3., 8).unwrap();
        for _ in 0..4 {
            inspector
                .dispatch(
                    figure.scene().stamp(),
                    InspectionAction::StepFocus { forward: true },
                    InputOrigin::Keyboard,
                )
                .unwrap();
            let hit = &inspector.hits()[0];
            assert!(matches!(hit.target, Target::HierarchyNode { .. }));
            assert!(hit.values.iter().any(|(k, _)| k == "label"));
            assert!(hit.values.iter().any(|(k, _)| k == "source_members"));
        }
    }
}
#[test]
fn sunburst_hole_is_not_a_pointer_target() {
    let figure = capture(hierarchy_sunburst("id", "parent").hierarchy_projection(
        HierarchyProjection::Sunburst {
            inner_radius: 30.,
            radius: HierarchyRadius::Area,
        },
    ));
    let bounds = figure.layout().hierarchy_snapshots().unwrap()[0].bounds;
    let center = Point::new(
        bounds.origin().x() + bounds.width() / 2.,
        bounds.origin().y() + bounds.height() / 2.,
    )
    .unwrap();
    let inspector = Inspector::new(figure.layout().clone(), 3., 8).unwrap();
    assert!(
        inspector
            .query(center, InspectionMode::Auto)
            .hits
            .is_empty()
    );
}

#[test]
fn live_resize_uses_acknowledged_rows_and_keeps_prior_export_immutable() {
    use chart_core::hierarchy::{LayoutSpec, Tiler, TreemapOptions};
    let data = Data::columns()
        .name("hierarchy")
        .keys(1..=7)
        .column("id", ["root", "a", "b", "c", "d", "e", "f"])
        .column(
            "parent",
            vec![
                None,
                Some("root"),
                Some("root"),
                Some("root"),
                Some("root"),
                Some("root"),
                Some("root"),
            ],
        )
        .column("value", [0., 8., 5., 3., 7., 4., 2.])
        .build()
        .unwrap();
    let plot = plot(data)
        .layer(
            hierarchy_treemap("id", "parent")
                .hierarchy_value("value")
                .hierarchy_layout(LayoutSpec::Treemap {
                    options: TreemapOptions {
                        tile: Tiler::Resquarify(1.618_033_988_749_895),
                        ..Default::default()
                    },
                    history: true,
                    padding_sides: Default::default(),
                    padding: None,
                    tiler: None,
                }),
        )
        .build()
        .unwrap();
    let output = output();
    let mut chart = plot.chart().unwrap();
    let first = output
        .live_request(
            &chart,
            export_options(PageSize::points(500., 200.).unwrap()).basis(CaptureBasis::Current),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let bytes = first.export(Format::Svg).unwrap().bytes;
    chart
        .acknowledge_paint_with_layout(
            first.layout().clone(),
            &first.metadata().effective_state,
            &first.metadata().profile.layout,
        )
        .unwrap();
    let next = output
        .live_request(
            &chart,
            export_options(PageSize::points(200., 500.).unwrap()).basis(CaptureBasis::Current),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let original = first.layout().hierarchies().values().next().unwrap();
    let resized = next.layout().hierarchies().values().next().unwrap();
    assert!(original.history().row_count() > 0);
    assert_eq!(
        serde_json::to_value(original.history()).unwrap(),
        serde_json::to_value(resized.history()).unwrap()
    );
    let bounds = resized.bounds();
    let mut history = original.history().clone();
    let expected = resized
        .prepared()
        .hierarchy()
        .treemap_with_history(
            TreemapOptions {
                size: [bounds.width(), bounds.height()],
                tile: Tiler::Resquarify(1.618_033_988_749_895),
                ..Default::default()
            },
            &mut history,
        )
        .unwrap();
    let fresh = output
        .request(&plot, export_options(PageSize::points(200., 500.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    assert_ne!(
        serde_json::to_value(
            fresh
                .layout()
                .hierarchies()
                .values()
                .next()
                .unwrap()
                .history()
        )
        .unwrap(),
        serde_json::to_value(resized.history()).unwrap()
    );
    for node in resized.prepared().hierarchy().iter().unwrap() {
        assert_eq!(
            resized.layout().geometry(node.handle()).unwrap(),
            expected.geometry(node.handle()).unwrap()
        );
    }
    assert_eq!(first.export(Format::Svg).unwrap().bytes, bytes);
}

#[test]
fn reparenting_keeps_node_identity_changes_link_and_failed_layout_keeps_prior_frame() {
    use chart_core::{grammar::PreparedGeometry, state::TargetIdentity};
    let source = |parent: &str| {
        Data::columns()
            .name("hierarchy")
            .keys(1..=4)
            .column("id", ["root", "a", "b", "c"])
            .column(
                "parent",
                vec![None, Some("root"), Some("root"), Some(parent)],
            )
            .column("value", [0., 2., 3., 5.])
            .build()
            .unwrap()
    };
    let plot = plot(source("a"))
        .layer(hierarchy_tree("id", "parent").hierarchy_value("value"))
        .build()
        .unwrap();
    let mut chart = plot.chart().unwrap();
    let output = output();
    let opts =
        || export_options(PageSize::points(400., 300.).unwrap()).basis(CaptureBasis::Current);
    let first = output
        .live_request(&chart, opts())
        .unwrap()
        .prepare()
        .unwrap();
    chart
        .acknowledge_paint(first.layout().clone(), &first.metadata().effective_state)
        .unwrap();
    let identities = |p: &chart_core::grammar::PreparedChart| {
        p.layers()[0]
            .marks()
            .iter()
            .filter_map(|m| match m.geometry {
                PreparedGeometry::HierarchyNode(n) if n.node.get() == 4 => {
                    Some((0, TargetIdentity::from(&m.targets[0])))
                }
                PreparedGeometry::HierarchyLink { child, .. } if child.node.get() == 4 => {
                    Some((1, TargetIdentity::from(&m.targets[0])))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let before = identities(first.layout().prepared());
    let inspector = Inspector::new(first.layout().clone(), 3., 8).unwrap();
    let selected = chart_core::state::MarkTarget::from_inspected(
        inspector
            .semantic_targets()
            .find(|t| chart_core::state::TargetIdentity::from(&t.target) == before[&0])
            .unwrap(),
        chart.source().get().unwrap().epoch(),
    );
    chart
        .act(chart_core::state::ChartAction::Select {
            change: chart_core::state::SelectionChange::Replace,
            targets: vec![selected.clone()],
        })
        .unwrap();

    let tx = chart
        .transaction()
        .unwrap()
        .replace("hierarchy", source("b"))
        .build()
        .unwrap();
    chart.apply_transaction(tx).unwrap();
    let next = output
        .live_request(&chart, opts())
        .unwrap()
        .prepare()
        .unwrap();
    let after = identities(next.layout().prepared());
    assert_eq!(before[&0], after[&0]);
    assert!(chart.state().selection().contains(&selected));
    assert_ne!(before[&1], after[&1]);
    let bytes = next.export(Format::Svg).unwrap().bytes;
    chart
        .acknowledge_paint(next.layout().clone(), &next.metadata().effective_state)
        .unwrap();
    let tx = chart
        .transaction()
        .unwrap()
        .replace("hierarchy", source("absent"))
        .build()
        .unwrap();
    chart.apply_transaction(tx).unwrap();
    assert!(
        output
            .live_request(&chart, opts())
            .unwrap()
            .prepare()
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(
        chart.reducer().presented().unwrap(),
        next.layout()
    ));
    assert_eq!(next.export(Format::Svg).unwrap().bytes, bytes);
    let remove = chart
        .transaction()
        .unwrap()
        .remove("hierarchy", [4])
        .build()
        .unwrap();
    chart.apply_transaction(remove).unwrap();
    assert!(chart.state().selection().is_empty());
    assert!(
        chart
            .act(chart_core::state::ChartAction::Select {
                change: chart_core::state::SelectionChange::Replace,
                targets: vec![selected]
            })
            .is_err()
    );
}

#[test]
fn facets_isolate_topology_membership_history_and_inspection_values() {
    use chart_core::hierarchy::{LayoutSpec, Tiler, TreemapOptions};
    let data = Data::columns()
        .keys(1..=6)
        .column("id", ["root", "a", "b", "root", "a", "b"])
        .column(
            "parent",
            vec![
                None,
                Some("root"),
                Some("root"),
                None,
                Some("root"),
                Some("root"),
            ],
        )
        .column("panel", ["A", "A", "A", "B", "B", "B"])
        .column("value", [0., 2., 3., 0., 7., 11.])
        .build()
        .unwrap();
    let plot = plot(data)
        .layer(
            hierarchy_treemap("id", "parent")
                .name("hierarchy")
                .hierarchy_value("value")
                .hierarchy_label("id")
                .hierarchy_layout(LayoutSpec::Treemap {
                    options: TreemapOptions {
                        tile: Tiler::Resquarify(1.618_033_988_749_895),
                        ..Default::default()
                    },
                    history: true,
                    padding_sides: Default::default(),
                    padding: None,
                    tiler: None,
                }),
        )
        .facet(facet_wrap("panel"))
        .build()
        .unwrap();
    let panel = chart_core::grammar::PanelKey {
        values: vec![chart_core::grammar::GroupValue::Text("A".into())],
    };
    let plot = plot
        .edit()
        .inset(
            inset()
                .id("detail")
                .panel(panel.clone())
                .layer(plot.layer("hierarchy").unwrap()),
        )
        .build()
        .unwrap();
    let output = output();
    let frame = output
        .request(&plot, export_options(PageSize::points(500., 300.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let snapshots = frame.layout().hierarchy_snapshots().unwrap();
    assert_eq!(snapshots.len(), 3);
    assert_eq!(
        snapshots[2].scope,
        vec![
            chart_core::layout::GuideScope::Panel(panel),
            chart_core::layout::GuideScope::Inset("detail".into())
        ]
    );
    assert_ne!(snapshots[0].scope, snapshots[1].scope);
    assert_eq!(snapshots[0].nodes[0].value, Some(5.));
    assert_eq!(snapshots[1].nodes[0].value, Some(18.));
    assert_eq!(
        snapshots[0]
            .source_keys
            .iter()
            .map(|k| k.get())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        snapshots[1]
            .source_keys
            .iter()
            .map(|k| k.get())
            .collect::<Vec<_>>(),
        [4, 5, 6]
    );
    let inspector = Inspector::new(frame.layout().clone(), 3., 8).unwrap();
    assert_eq!(inspector.semantic_targets().len(), 6);
    assert!(
        inspector
            .semantic_targets()
            .all(|t| t.panel.is_some() && t.values.iter().any(|(k, _)| k == "label"))
    );
    let resized = output
        .request(&plot, export_options(PageSize::points(300., 500.).unwrap()))
        .unwrap()
        .with_hierarchy_history(frame.layout())
        .prepare()
        .unwrap();
    assert_eq!(resized.layout().hierarchy_history().len(), 3);
    for (a, b) in frame
        .layout()
        .hierarchy_history()
        .iter()
        .zip(resized.layout().hierarchy_history())
    {
        assert_eq!(a.scope, b.scope);
        assert_eq!(
            serde_json::to_value(&a.history).unwrap(),
            serde_json::to_value(&b.history).unwrap()
        );
    }
}

#[test]
fn registered_chart_accessors_and_tiler_are_captured_and_native_only_rejects() {
    use chart_core::{
        grammar::{HierarchyAggregation, HierarchyOrder},
        hierarchy::{LayoutSpec, RegisteredOperation, Tiler, TreemapOptions},
    };
    let operation = |id: &str, mode: &str| RegisteredOperation {
        operation: chart_core::grammar::OperationRef {
            id: id.into(),
            version: chart_core::Revision::new(1),
        },
        parameters: serde_json::json!({"mode":mode}),
    };
    let layer = hierarchy_treemap("id", "parent")
        .hierarchy_aggregation(HierarchyAggregation::Registered(operation(
            "example.hierarchy",
            "Value",
        )))
        .hierarchy_order(HierarchyOrder::Registered(operation(
            "example.hierarchy",
            "DescendingValue",
        )))
        .hierarchy_layout(LayoutSpec::Treemap {
            options: TreemapOptions {
                tile: Tiler::Custom,
                ..Default::default()
            },
            history: false,
            padding_sides: Default::default(),
            padding: None,
            tiler: Some(operation("example.hierarchy", "EqualTile")),
        });
    let plot = plot(data())
        .extensions(chart_extension_example::registry().unwrap())
        .layer(layer)
        .build()
        .unwrap();
    let output = output();
    let figure = output
        .request(&plot, export_options(PageSize::points(400., 300.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let h = figure.layout().hierarchy_snapshots().unwrap();
    assert_eq!(h[0].nodes[0].value, Some(10.));
    let plot = chart_core::plot::Plot::from_json_with_extensions(
        &plot.to_json().unwrap(),
        chart_extension_example::registry().unwrap(),
    )
    .unwrap();
    assert_eq!(
        output
            .request(&plot, export_options(PageSize::points(400., 300.).unwrap()))
            .unwrap()
            .prepare()
            .unwrap()
            .scene_json()
            .unwrap(),
        figure.scene_json().unwrap()
    );
    let bad = chart_core::plot::plot(data())
        .extensions(chart_extension_example::registry().unwrap())
        .layer(hierarchy_pack("id", "parent").hierarchy_aggregation(
            HierarchyAggregation::Registered(operation("example.native_hierarchy", "Value")),
        ))
        .build();
    let native = bad.unwrap();
    assert!(native.chart().unwrap().prepare().is_ok());
    assert!(native.to_json().is_err());
    assert!(
        output
            .request(
                &native,
                export_options(PageSize::points(400., 300.).unwrap())
            )
            .unwrap()
            .prepare()
            .is_err()
    );
}

#[test]
fn deep_membership_is_shared_bounded_and_released_and_filters_do_not_repair_orphans() {
    use chart_core::hierarchy::HierarchyLimits;
    use std::sync::Arc;
    let n = 2048u64;
    let data = Data::columns()
        .keys(1..=n)
        .column("id", (1..=n).collect::<Vec<_>>())
        .column(
            "parent",
            (1..=n)
                .map(|i| (i > 1).then_some(i - 1))
                .collect::<Vec<_>>(),
        )
        .column("value", vec![1.; n as usize])
        .build()
        .unwrap();
    let weak = {
        let plot = plot(data.clone())
            .layer(hierarchy_icicle("id", "parent").hierarchy_value("value"))
            .build()
            .unwrap();
        let mut chart = plot.chart().unwrap();
        let prepared = chart.prepare().unwrap();
        let hierarchy = prepared.layers()[0].hierarchy().unwrap();
        assert_eq!(hierarchy.source_keys().len(), n as usize);
        for node in hierarchy.hierarchy().iter().unwrap() {
            let Target::HierarchyNode { membership, .. } = hierarchy.target(node.handle()).unwrap()
            else {
                panic!("node target")
            };
            assert!(Arc::ptr_eq(
                membership.shared_keys(),
                hierarchy.source_keys()
            ));
            assert_eq!(membership.members().len(), n as usize - node.depth());
        }
        let weak = Arc::downgrade(hierarchy.source_keys());
        assert!(weak.upgrade().is_some());
        weak
    };
    assert!(
        weak.upgrade().is_none(),
        "membership has no process-global or old-scene owner"
    );
    for limits in [
        HierarchyLimits {
            max_nodes: n as usize - 1,
            ..Default::default()
        },
        HierarchyLimits {
            max_depth: 10,
            ..Default::default()
        },
        HierarchyLimits {
            max_payload_bytes: 8,
            ..Default::default()
        },
        HierarchyLimits {
            max_work: 8,
            ..Default::default()
        },
    ] {
        let p = plot(data.clone())
            .layer(hierarchy_tree("id", "parent").hierarchy_limits(limits))
            .build()
            .unwrap();
        assert!(p.chart().unwrap().prepare().is_err());
    }
    let p = plot(data)
        .layer(hierarchy_tree("id", "parent"))
        .build()
        .unwrap();
    let err = output()
        .request(
            &p,
            export_options(PageSize::points(400., 300.).unwrap()).layout(layout_options().limits(
                chart_core::Limits {
                    max_path_commands: 32,
                    ..Default::default()
                },
            )),
        )
        .unwrap()
        .prepare()
        .err()
        .expect("path budget must reject");
    assert_eq!(err.code, chart_core::DiagnosticCode::ResourceLimit);
    let orphan = plot(self::data())
        .layer(hierarchy_tree("id", "parent").filter(filter("value").minimum(1.)))
        .build()
        .unwrap();
    assert!(orphan.chart().unwrap().prepare().is_err());
}

#[test]
fn nested_circles_use_frontmost_leaf_and_clip_outside_the_panel() {
    use chart_core::hierarchy::NodeGeometry;
    let figure = capture(hierarchy_pack("id", "parent"));
    let snapshots = figure.layout().hierarchy_snapshots().unwrap();
    let h = &snapshots[0];
    let leaf = h.nodes.iter().find(|n| n.children.is_empty()).unwrap();
    let Some(NodeGeometry::Circle { x, y, .. }) = leaf.geometry else {
        panic!("circle")
    };
    let center = Point::new(h.bounds.origin().x() + x, h.bounds.origin().y() + y).unwrap();
    let inspector = Inspector::new(figure.layout().clone(), 0.01, 8).unwrap();
    let hits = inspector.query(center, InspectionMode::Auto);
    assert!(!hits.hits.is_empty());
    assert_eq!(
        hits.hits[0].target,
        h.targets[h
            .nodes
            .iter()
            .position(|n| n.handle == leaf.handle)
            .unwrap()]
    );
    let outside = Point::new(h.bounds.origin().x() - 1., center.y()).unwrap();
    assert!(
        inspector
            .query(outside, InspectionMode::Auto)
            .hits
            .is_empty()
    );
}

#[test]
fn sunburst_seam_sides_and_zero_area_nodes_keep_structural_metadata() {
    let d = Data::columns()
        .column("id", ["root", "a", "b", "zero"])
        .column(
            "parent",
            vec![None, Some("root"), Some("root"), Some("root")],
        )
        .column("value", [0., 1., 3., 0.])
        .build()
        .unwrap();
    let p = plot(d)
        .layer(
            hierarchy_sunburst("id", "parent")
                .hierarchy_value("value")
                .hierarchy_projection(HierarchyProjection::Sunburst {
                    inner_radius: 20.,
                    radius: HierarchyRadius::Linear,
                }),
        )
        .build()
        .unwrap();
    let f = output()
        .request(&p, export_options(PageSize::points(400., 300.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let h = &f.layout().hierarchy_snapshots().unwrap()[0];
    assert_eq!(h.nodes.len(), 4);
    assert_eq!(h.nodes[3].value, Some(0.));
    assert_eq!(h.targets.len(), 4);
    let outer = h.bounds.width().min(h.bounds.height()) / 2.;
    let radius = 20. + (outer - 20.) * 0.75;
    let cx = h.bounds.origin().x() + h.bounds.width() / 2.;
    let cy = h.bounds.origin().y() + h.bounds.height() / 2.;
    let inspector = Inspector::new(f.layout().clone(), 0.01, 8).unwrap();
    for (angle, node) in [(0.02_f64, 1), (std::f64::consts::TAU - 0.02, 2)] {
        let point = Point::new(cx + radius * angle.sin(), cy - radius * angle.cos()).unwrap();
        let hits = inspector.query(point, InspectionMode::Auto);
        assert_eq!(hits.hits[0].target, h.targets[node]);
    }
    assert!(
        inspector
            .query(Point::new(cx, cy).unwrap(), InspectionMode::Auto)
            .hits
            .is_empty()
    );
}
