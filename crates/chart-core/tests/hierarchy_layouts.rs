//! Pinned standalone hierarchy geometry and independent layout invariants.
use chart_core::hierarchy::*;
use serde_json::Value;
#[path = "support/hierarchy.rs"]
mod support;
use support::*;
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-10 + 1e-12 * expected.abs(),
        "{actual} != {expected}"
    );
}
#[test]
fn pinned_tidy_tree_and_cluster_controls() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let op = c["op"].as_str().unwrap();
        if !["tree", "cluster"].contains(&op) {
            continue;
        }
        let tree = nested(&c["input"])
            .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
            .unwrap();
        let config = &c["config"];
        let mut options = TreeOptions::default();
        if let Some(size) = config.get("size") {
            options = options
                .with_size([size[0].as_f64().unwrap(), size[1].as_f64().unwrap()])
                .unwrap();
        }
        if let Some(size) = config.get("nodeSize") {
            options = options
                .with_node_size([size[0].as_f64().unwrap(), size[1].as_f64().unwrap()])
                .unwrap();
        }
        options.separation = match config["separation"].as_str() {
            Some("depth") => Separation::Depth,
            Some("constant") => Separation::Constant(config["distance"].as_f64().unwrap()),
            _ => Separation::Default,
        };
        let output = if op == "tree" {
            tree.tree(options)
        } else {
            tree.cluster(options)
        }
        .unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
        assert_eq!(
            output.hierarchy().len(),
            c["expected"].as_array().unwrap().len()
        );
        for ((node, geometry), expected) in output
            .nodes()
            .unwrap()
            .zip(c["expected"].as_array().unwrap())
        {
            assert_eq!(name(node), expected["name"]);
            let NodeGeometry::Point { x, y } = geometry else {
                panic!("Expected node point")
            };
            near(x, expected["x"].as_f64().unwrap());
            near(y, expected["y"].as_f64().unwrap());
        }
        checked += 1;
    }
    assert_eq!(checked, 340);
}
#[test]
fn tree_modes_native_separation_and_leaf_alignment() {
    let input = serde_json::json!({"name":"R","children":[{"name":"A","children":[{"name":"C"}]},{"name":"B"}]});
    let tree = nested(&input);
    let mut options = TreeOptions::default();
    assert_eq!(options.size(), Some([1., 1.]));
    assert_eq!(options.node_size(), None);
    options = options.with_node_size([2., 3.]).unwrap();
    assert_eq!(options.size(), None);
    assert_eq!(options.node_size(), Some([2., 3.]));
    for output in [tree.tree(options).unwrap(), tree.cluster(options).unwrap()] {
        assert_eq!(
            output.geometry(tree.root().handle()).unwrap(),
            NodeGeometry::Point { x: 0., y: 0. }
        );
    }
    options = options.with_size([8., 4.]).unwrap();
    assert_eq!(options.node_size(), None);
    let cluster = tree.cluster(options).unwrap();
    for leaf in tree.leaves(tree.root().handle()).unwrap() {
        let NodeGeometry::Point { y, .. } = cluster.geometry(leaf).unwrap() else {
            unreachable!()
        };
        assert_eq!(y, 4.);
    }
    let builtin = tree
        .tree(TreeOptions {
            separation: Separation::Constant(3.),
            ..options
        })
        .unwrap();
    let mut calls = 0;
    let native = tree
        .tree_with(options, |a, b| {
            calls += 1;
            assert_eq!(a.handle().hierarchy, b.handle().hierarchy);
            Ok(3.)
        })
        .unwrap();
    assert!(calls > 0);
    for (n, g) in native.nodes().unwrap() {
        assert_eq!(g, builtin.geometry(n.handle()).unwrap());
    }
    assert!(tree.tree_with(options, |_, _| Ok(f64::NAN)).is_err());
    assert!(TreeOptions::default().with_size([-1., 1.]).is_err());
}

#[test]
fn pinned_partition_internal_values_padding_and_rounding() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["op"] != "partition" {
            continue;
        }
        let tree = nested(&c["input"])
            .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
            .unwrap();
        let config = &c["config"];
        let mut options = PartitionOptions::default();
        if let Some(size) = config.get("size") {
            options.size = [size[0].as_f64().unwrap(), size[1].as_f64().unwrap()];
        }
        options.padding = config["padding"].as_f64().unwrap_or(0.);
        options.round = config["round"].as_bool().unwrap_or(false);
        let output = tree
            .partition(options)
            .unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
        assert_eq!(
            output.hierarchy().len(),
            c["expected"].as_array().unwrap().len()
        );
        for ((node, geometry), expected) in output
            .nodes()
            .unwrap()
            .zip(c["expected"].as_array().unwrap())
        {
            assert_eq!(name(node), expected["name"]);
            let NodeGeometry::Rectangle { x0, y0, x1, y1 } = geometry else {
                panic!("Expected rectangle")
            };
            for (actual, field) in [(x0, "x0"), (y0, "y0"), (x1, "x1"), (y1, "y1")] {
                near(actual, expected[field].as_f64().unwrap());
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 42);
    let tree = nested(
        &serde_json::json!({"name":"R","value":2,"children":[{"name":"A","value":1},{"name":"B","value":1}]}),
    );
    assert!(tree.partition(PartitionOptions::default()).is_err());
    let tree = tree.sum(|n| Ok(n.data()["value"].as_f64())).unwrap();
    let output = tree
        .partition(PartitionOptions {
            size: [8., 4.],
            ..PartitionOptions::default()
        })
        .unwrap();
    let last = tree.root().children().last().unwrap();
    let NodeGeometry::Rectangle { x1, .. } = output.geometry(last.handle()).unwrap() else {
        unreachable!()
    };
    assert_eq!(x1, 4.);
    assert_eq!(tree.root().value(), Some(4.));
}
