//! Pinned packing/enclosure values with independent containment and non-overlap checks.
use chart_core::{ChartResult, hierarchy::*};
use serde_json::Value;
#[path = "support/hierarchy.rs"]
mod support;
use support::*;
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-8 + 1e-10 * b.abs(), "{a} != {b}");
}
fn circle(value: &Value) -> Circle {
    Circle {
        x: value["x"].as_f64().unwrap_or(0.),
        y: value["y"].as_f64().unwrap_or(0.),
        r: value["r"].as_f64().unwrap(),
    }
}
fn non_overlap(circles: &[Circle], scale: f64) {
    for (i, a) in circles.iter().enumerate() {
        for b in &circles[i + 1..] {
            let distance = (a.x - b.x).hypot(a.y - b.y);
            let slack = 1e-6 * scale + 1e-8 * (1. + a.r.max(b.r));
            assert!(
                distance + slack >= a.r + b.r,
                "circles overlap beyond reference placement slack: {a:?} {b:?}"
            );
        }
    }
}
fn contains_exception(value: &Value) -> bool {
    match value {
        Value::Array(a) => a.iter().any(contains_exception),
        Value::Object(o) => o.contains_key("number") || o.values().any(contains_exception),
        _ => false,
    }
}
struct Accessors {
    depth: bool,
}
impl PackAccessors for Accessors {
    fn radius(&mut self, node: NodeView<'_>) -> ChartResult<f64> {
        Ok(node
            .data()
            .get("value")
            .and_then(Value::as_f64)
            .unwrap_or(0.))
    }
    fn padding(&mut self, node: NodeView<'_>, configured: f64) -> ChartResult<f64> {
        Ok(if self.depth {
            node.depth() as f64 + 1.
        } else {
            configured
        })
    }
}
#[test]
fn pinned_hierarchical_pack_and_independent_helpers() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    let mut collapsed = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let op = c["op"].as_str().unwrap();
        match op {
            "pack" => {
                let tree = nested(&c["input"])
                    .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
                    .unwrap();
                let config = &c["config"];
                let mut options = PackOptions::default();
                if let Some(size) = config.get("size") {
                    options.size = [size[0].as_f64().unwrap(), size[1].as_f64().unwrap()];
                }
                if config.get("radius").is_some() {
                    options.radius = PackRadius::Explicit;
                }
                options.padding = config["padding"].as_f64().unwrap_or(0.);
                let output = tree
                    .pack_with(
                        options,
                        &mut Accessors {
                            depth: config["paddingAccessor"] == true,
                        },
                    )
                    .unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
                assert_eq!(tree.len(), c["expected"].as_array().unwrap().len());
                if contains_exception(&c["expected"]) {
                    collapsed += 1;
                    assert_eq!(options.radius, PackRadius::Fitted);
                    for (_, g) in output.nodes().unwrap() {
                        assert_eq!(
                            g,
                            NodeGeometry::Circle {
                                x: options.size[0] / 2.,
                                y: options.size[1] / 2.,
                                r: 0.
                            }
                        );
                    }
                } else {
                    for ((n, g), e) in output
                        .nodes()
                        .unwrap()
                        .zip(c["expected"].as_array().unwrap())
                    {
                        assert_eq!(name(n), e["name"]);
                        let NodeGeometry::Circle { x, y, r } = g else {
                            panic!("circle")
                        };
                        near(x, e["x"].as_f64().unwrap());
                        near(y, e["y"].as_f64().unwrap());
                        near(r, e["r"].as_f64().unwrap());
                    }
                }
                let mut scale = 1.;
                if options.radius == PackRadius::Fitted {
                    for leaf in tree.leaves(tree.root().handle()).unwrap() {
                        let n = tree.get(leaf).unwrap();
                        if n.value().unwrap() > 0. {
                            let NodeGeometry::Circle { r, .. } = output.geometry(leaf).unwrap()
                            else {
                                unreachable!()
                            };
                            scale = r / n.value().unwrap().sqrt();
                            break;
                        }
                    }
                }
                for (n, g) in output.nodes().unwrap() {
                    let NodeGeometry::Circle { x, y, r } = g else {
                        unreachable!()
                    };
                    let children = n
                        .children()
                        .map(|child| {
                            let NodeGeometry::Circle {
                                x: cx,
                                y: cy,
                                r: cr,
                            } = output.geometry(child.handle()).unwrap()
                            else {
                                unreachable!()
                            };
                            assert!((x - cx).hypot(y - cy) + cr <= r + 1e-8 * (1. + r));
                            Circle {
                                x: cx,
                                y: cy,
                                r: cr,
                            }
                        })
                        .collect::<Vec<_>>();
                    non_overlap(&children, scale);
                }
            }
            "packSiblings" => {
                let input = c["input"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| Circle {
                        r: r.as_f64().unwrap(),
                        ..Circle::default()
                    })
                    .collect::<Vec<_>>();
                let output = pack_siblings(&input, HierarchyLimits::default())
                    .unwrap_or_else(|e| panic!("{}: {e:?}", c["id"]));
                assert_eq!(output.len(), input.len());
                for (a, e) in output.iter().zip(c["expected"].as_array().unwrap()) {
                    let e = circle(e);
                    near(a.x, e.x);
                    near(a.y, e.y);
                    assert_eq!(a.r, e.r);
                }
                non_overlap(&output, 1.);
                assert_eq!(
                    output,
                    pack_siblings(&input, HierarchyLimits::default()).unwrap()
                );
            }
            "packEnclose" => {
                let input = c["input"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(circle)
                    .collect::<Vec<_>>();
                let output = pack_enclose(&input, HierarchyLimits::default()).unwrap();
                if c["expected"].is_null() {
                    assert_eq!(output, None);
                } else {
                    let a = output.unwrap();
                    let e = circle(&c["expected"]);
                    near(a.x, e.x);
                    near(a.y, e.y);
                    near(a.r, e.r);
                    for b in input {
                        assert!((a.x - b.x).hypot(a.y - b.y) + b.r <= a.r + 1e-8 * (1. + a.r));
                    }
                }
            }
            _ => continue,
        }
        checked += 1;
    }
    assert_eq!(checked, 191);
    assert!(collapsed > 0);
}
#[test]
fn explicit_unscaled_radii_overflow_and_work_budgets() {
    let input = [
        Circle {
            x: 0.,
            y: 0.,
            r: 1.,
        },
        Circle {
            x: 4.,
            y: 0.,
            r: 1.,
        },
    ];
    assert_eq!(
        pack_enclose(&input, HierarchyLimits::default()).unwrap(),
        Some(Circle {
            x: 2.,
            y: 0.,
            r: 3.
        })
    );
    assert!(
        pack_enclose(
            &input,
            HierarchyLimits {
                max_work: 0,
                ..HierarchyLimits::default()
            }
        )
        .is_err()
    );
    assert!(
        pack_siblings(
            &[Circle {
                r: -1.,
                ..Circle::default()
            }],
            HierarchyLimits::default()
        )
        .is_err()
    );
    assert!(
        pack_enclose(
            &[
                Circle {
                    x: -1e308,
                    y: 0.,
                    r: 1.
                },
                Circle {
                    x: 1e308,
                    y: 0.,
                    r: 1.
                }
            ],
            HierarchyLimits::default()
        )
        .is_err()
    );
    let tree = nested(
        &serde_json::json!({"name":"R","children":[{"name":"A","value":2},{"name":"B","value":3}]}),
    );
    let options = PackOptions {
        size: [1., 1.],
        radius: PackRadius::Explicit,
        padding: 0.,
    };
    let output = tree
        .pack_with(options, &mut Accessors { depth: false })
        .unwrap();
    assert_eq!(tree.root().value(), None);
    for leaf in tree.leaves(tree.root().handle()).unwrap() {
        let NodeGeometry::Circle { r, .. } = output.geometry(leaf).unwrap() else {
            unreachable!()
        };
        assert_eq!(r, tree.get(leaf).unwrap().data()["value"].as_f64().unwrap());
    }
    assert!(tree.pack(options).is_err());
}
