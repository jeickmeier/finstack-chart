//! Every built-in tiler, complete padding controls, custom output and history sequences.
use chart_core::{ChartResult, hierarchy::*};
use serde_json::Value;
#[path = "support/hierarchy.rs"]
mod support;
use support::*;
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-10 + 1e-12 * b.abs(), "{a} != {b}");
}
fn kind(name: &str, ratio: f64) -> Tiler {
    match name {
        "treemapBinary" => Tiler::Binary,
        "treemapDice" => Tiler::Dice,
        "treemapSlice" => Tiler::Slice,
        "treemapSliceDice" => Tiler::SliceDice,
        "treemapSquarify" => Tiler::Squarify(ratio),
        "treemapResquarify" => Tiler::Resquarify(ratio),
        "customEqual" => Tiler::Custom,
        _ => panic!("unknown tiler {name}"),
    }
}
struct Accessors {
    depth: bool,
}
impl TreemapAccessors for Accessors {
    fn padding(&mut self, n: NodeView<'_>, _: PaddingSide, configured: f64) -> ChartResult<f64> {
        Ok(if self.depth {
            n.depth() as f64 + 1.
        } else {
            configured
        })
    }
    fn tile(
        &mut self,
        parent: NodeView<'_>,
        [x0, y0, x1, y1]: [f64; 4],
    ) -> ChartResult<Vec<[f64; 4]>> {
        let n = parent.children().len();
        Ok((0..n)
            .map(|i| {
                [
                    x0 + (x1 - x0) * i as f64 / n as f64,
                    y0,
                    x0 + (x1 - x0) * (i + 1) as f64 / n as f64,
                    y1,
                ]
            })
            .collect())
    }
}
fn compare(output: &HierarchyLayout, expected: &Value) {
    assert_eq!(output.hierarchy().len(), expected.as_array().unwrap().len());
    for ((n, g), e) in output.nodes().unwrap().zip(expected.as_array().unwrap()) {
        assert_eq!(name(n), e["name"]);
        let NodeGeometry::Rectangle { x0, y0, x1, y1 } = g else {
            panic!("rectangles")
        };
        for (a, k) in [(x0, "x0"), (y0, "y0"), (x1, "x1"), (y1, "y1")] {
            near(a, e[k].as_f64().unwrap());
        }
    }
}
#[test]
fn pinned_six_standalone_tilers_and_complete_treemap_controls() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let op = c["op"].as_str().unwrap();
        if !["tile", "treemap"].contains(&op) {
            continue;
        }
        let tree = nested(&c["input"])
            .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
            .unwrap();
        let config = &c["config"];
        let tile = kind(
            config["tile"].as_str().unwrap(),
            config["ratio"].as_f64().unwrap_or(GOLDEN_RATIO),
        );
        let mut accessors = Accessors {
            depth: config["paddingAccessor"] == true,
        };
        if op == "tile" {
            let bounds = std::array::from_fn(|i| config["bounds"][i].as_f64().unwrap());
            let result =
                tree.tile_children(tree.root().handle(), bounds, tile, None, &mut accessors);
            if c["expected"].get("error").is_some() {
                assert!(result.is_err());
            } else {
                let result = result.unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
                for (h, r) in result {
                    let n = tree.get(h).unwrap();
                    let e = c["expected"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["name"] == name(n))
                        .unwrap();
                    for (i, k) in ["x0", "y0", "x1", "y1"].into_iter().enumerate() {
                        near(r[i], e[k].as_f64().unwrap());
                    }
                }
            }
        } else {
            let mut o = TreemapOptions {
                tile,
                ..TreemapOptions::default()
            };
            if let Some(size) = config.get("size") {
                o.size = [size[0].as_f64().unwrap(), size[1].as_f64().unwrap()];
            }
            if let Some(p) = config["padding"].as_f64() {
                o = o.with_padding(p);
            }
            o.padding_inner = config["paddingInner"].as_f64().unwrap_or(o.padding_inner);
            o.padding_top = config["paddingTop"].as_f64().unwrap_or(o.padding_top);
            o.padding_right = config["paddingRight"].as_f64().unwrap_or(o.padding_right);
            o.padding_bottom = config["paddingBottom"].as_f64().unwrap_or(o.padding_bottom);
            o.padding_left = config["paddingLeft"].as_f64().unwrap_or(o.padding_left);
            o.round = config["round"] == true;
            let output = tree
                .treemap_with(o, None, &mut accessors)
                .unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
            compare(&output, &c["expected"]);
        }
        checked += 1;
    }
    assert_eq!(checked, 217);
}
#[test]
fn pinned_resquarify_value_resize_and_explicit_reset_histories() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    for c in fixture["cases"].as_array().unwrap() {
        if c["op"] != "history" {
            continue;
        }
        let original = nested(&c["input"]);
        let mut history = TreemapHistory::default();
        let mut old = None;
        for (step, expected) in c["expected"].as_array().unwrap().iter().enumerate() {
            let tree = original
                .sum(|n| {
                    let label = n.data()["name"].as_str().unwrap();
                    let index = label.strip_prefix('N').map(|i| i.parse::<usize>().unwrap());
                    Ok(Some(match (index, step) {
                        (Some(i), 1) => (21 - i) as f64,
                        (Some(i), 2 | 3) => {
                            if i % 2 == 1 {
                                20.
                            } else {
                                1.
                            }
                        }
                        _ => n.data().get("value").and_then(Value::as_f64).unwrap_or(0.),
                    }))
                })
                .unwrap();
            if step == 3 {
                history.reset();
            }
            let size = match step {
                0 => [80., 40.],
                1 => [40., 80.],
                _ => [100., 100.],
            };
            let options = TreemapOptions {
                size,
                tile: Tiler::Resquarify(c["config"]["ratio"].as_f64().unwrap()),
                ..TreemapOptions::default()
            };
            let output = tree.treemap_with_history(options, &mut history).unwrap();
            compare(&output, expected);
            assert!(history.row_count() > 0);
            assert_eq!(history.membership_count(), tree.len() - 1);
            if let Some(previous) = &old {
                compare(previous, &c["expected"][0]);
            } else {
                old = Some(output);
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 3);
}
#[test]
fn invalid_custom_output_ratio_changes_and_topology_reset_are_atomic() {
    let tree=nested(&serde_json::json!({"name":"R","children":[{"name":"A","value":1},{"name":"B","value":3},{"name":"C","value":2}]})).sum(|n|Ok(n.data().get("value").and_then(Value::as_f64))).unwrap();
    let mut history = TreemapHistory::default();
    let options = TreemapOptions {
        size: [80., 40.],
        tile: Tiler::Resquarify(GOLDEN_RATIO),
        ..TreemapOptions::default()
    };
    let retained = tree.treemap_with_history(options, &mut history).unwrap();
    let rows = history.row_count();
    struct Invalid;
    impl TreemapAccessors for Invalid {
        fn tile(&mut self, _: NodeView<'_>, _: [f64; 4]) -> ChartResult<Vec<[f64; 4]>> {
            Ok(vec![[f64::NAN; 4]; 3])
        }
    }
    assert!(
        tree.treemap_with(
            TreemapOptions {
                tile: Tiler::Custom,
                ..options
            },
            Some(&mut history),
            &mut Invalid
        )
        .is_err()
    );
    assert_eq!(history.row_count(), rows);
    let sorted = tree
        .sort(|a, b| Ok(b.value().partial_cmp(&a.value()).unwrap()))
        .unwrap();
    let actual = sorted.treemap_with_history(options, &mut history).unwrap();
    let fresh = sorted.treemap(options).unwrap();
    for (n, g) in actual.nodes().unwrap() {
        assert_eq!(g, fresh.geometry(n.handle()).unwrap());
    }
    let ratio = TreemapOptions {
        tile: Tiler::Resquarify(3.),
        ..options
    };
    let actual = sorted.treemap_with_history(ratio, &mut history).unwrap();
    let fresh = sorted.treemap(ratio).unwrap();
    for (n, g) in actual.nodes().unwrap() {
        assert_eq!(g, fresh.geometry(n.handle()).unwrap());
    }
    let fresh = tree.treemap(options).unwrap();
    for (n, g) in retained.nodes().unwrap() {
        assert_eq!(g, fresh.geometry(n.handle()).unwrap());
    }
    assert!(Tiler::Squarify(f64::NAN).ratio().is_err());
    assert_eq!(Tiler::Squarify(0.).ratio().unwrap(), Some(1.));
    let o = TreemapOptions::default()
        .with_padding(2.)
        .with_padding_outer(3.);
    assert_eq!(o.padding(), 2.);
    assert_eq!(o.padding_outer(), 3.);
}

#[test]
fn pinned_insert_remove_reparent_and_reorder_reset_history() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let c = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["op"] == "historyTopology")
        .unwrap();
    let mut history = TreemapHistory::default();
    let mut retained = Vec::new();
    for (step, expected) in c["config"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .zip(c["expected"].as_array().unwrap())
    {
        let tree = nested(&step["tree"])
            .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
            .unwrap();
        let options = TreemapOptions {
            size: [80., 40.],
            tile: Tiler::Resquarify(step["ratio"].as_f64().unwrap()),
            ..TreemapOptions::default()
        };
        let output = tree.treemap_with_history(options, &mut history).unwrap();
        compare(&output, expected);
        assert_eq!(history.membership_count(), tree.len() - 1);
        retained.push(output);
    }
    for (output, expected) in retained.iter().zip(c["expected"].as_array().unwrap()) {
        compare(output, expected);
    }
}
