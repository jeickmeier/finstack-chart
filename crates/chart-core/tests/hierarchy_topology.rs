//! Independent topology, operation, identity and deep-budget contracts.
use chart_core::{ChartResult, hierarchy::*};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
#[path = "support/hierarchy.rs"]
mod support;
use support::*;
#[test]
fn pinned_construction_and_all_node_operations() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/hierarchy/reference.json")).unwrap();
    let mut checked = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let op = c["op"].as_str().unwrap();
        let id = c["id"].as_str().unwrap();
        let input = &c["input"];
        let result: Option<ChartResult<Hierarchy>> = match op {
            "Node" => Some(Hierarchy::node(
                HierarchyId::new(1),
                key(0),
                Arc::new(input.clone()),
                HierarchyLimits::default(),
            )),
            "hierarchy" if id != "ordered-grouped" => Some(Ok(nested(input))),
            "stratify" if c["config"]["path"] == true => Some(Hierarchy::from_paths(
                HierarchyId::new(1),
                input
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (key(i), Arc::new(v.clone()), v.as_str().unwrap().into()))
                    .collect(),
                HierarchyLimits::default(),
            )),
            "stratify" => Some(Hierarchy::stratify(
                HierarchyId::new(1),
                input
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (key(i), Arc::new(v.clone())))
                    .collect(),
                &StratifyOptions::default(),
                HierarchyLimits::default(),
            )),
            _ => None,
        };
        if let Some(result) = result {
            if let Some(error) = c["expected"].get("error") {
                let actual = result.unwrap_err();
                assert_eq!(actual.message, error.as_str().unwrap(), "{id}");
            } else {
                assert_eq!(records(&result.unwrap()), c["expected"], "{id}");
            }
            checked += 1;
        }
        if op == "operations" {
            let tree = nested(input)
                .sum(|n| Ok(n.data().get("value").and_then(Value::as_f64)))
                .unwrap();
            let r = tree.root().handle();
            let mut actual = json!({"sum":records(&tree),"count":records(&tree.count().unwrap())});
            for (field, order) in [
                ("before", VisitOrder::PreOrder),
                ("after", VisitOrder::PostOrder),
                ("each", VisitOrder::BreadthFirst),
            ] {
                let mut list = Vec::new();
                tree.visit(r, order, |n, i, root| {
                    list.push(json!([name(n), i, root.handle() == r]));
                    Ok(())
                })
                .unwrap();
                actual[field] = json!(list);
            }
            actual["iterator"] = names(&tree, tree.descendants(r).unwrap());
            actual["leaves"] = names(&tree, tree.leaves(r).unwrap());
            let cnode = tree.find(r, |n, _, _| Ok(name(n) == "C")).unwrap().unwrap();
            let b = tree.find(r, |n, _, _| Ok(name(n) == "B")).unwrap().unwrap();
            actual["ancestors"] = names(&tree, tree.ancestors(cnode).unwrap());
            actual["path"] = names(&tree, tree.path(cnode, b).unwrap());
            actual["selfPath"] = names(&tree, tree.path(cnode, cnode).unwrap());
            actual["links"] = json!(
                tree.links(r)
                    .unwrap()
                    .into_iter()
                    .map(|(a, b)| vec![name(tree.get(a).unwrap()), name(tree.get(b).unwrap())])
                    .collect::<Vec<_>>()
            );
            actual["find"] = name(
                tree.get(tree.find(r, |n, _, _| Ok(n.depth() == 1)).unwrap().unwrap())
                    .unwrap(),
            );
            actual["noMatch"] = json!(tree.find(r, |_, _, _| Ok(false)).unwrap().is_none());
            let a = tree.root().children().next().unwrap();
            let copied = tree.copy_subtree(a.handle(), HierarchyId::new(2)).unwrap();
            actual["copy"] = records(&copied);
            actual["copyShared"] = json!(Arc::ptr_eq(copied.root().data(), a.data()));
            actual["sort"] = records(
                &tree
                    .sort(|a, b| Ok(b.value().partial_cmp(&a.value()).unwrap()))
                    .unwrap(),
            );
            assert_json(&actual, &c["expected"]);
            checked += 1;
        }
    }
    assert_eq!(checked, 31);
}
#[test]
fn custom_children_grouping_keys_atomicity_and_deep_flat_budget() {
    let payload = Arc::new(json!({"name":"shared"}));
    let map = BTreeMap::from([(1, vec![2, 3]), (2, vec![4])]);
    let tree = Hierarchy::with_children(
        HierarchyId::new(u64::MAX),
        (HierarchyNodeId::new(1), payload.clone()),
        |k, _| {
            Ok(map
                .get(&k.get())
                .into_iter()
                .flatten()
                .map(|&id| (HierarchyNodeId::new(id), payload.clone()))
                .collect::<Vec<_>>())
        },
        HierarchyLimits::default(),
    )
    .unwrap();
    assert_eq!(tree.len(), 4);
    assert!(
        tree.root()
            .children()
            .all(|n| Arc::ptr_eq(n.data(), &payload))
    );
    assert!(
        tree.get(NodeHandle {
            hierarchy: HierarchyId::new(2),
            node: key(0)
        })
        .is_err()
    );
    assert!(tree.sum(|_| Ok(Some(f64::NAN))).is_err());
    assert!(tree.sum(|_| Ok(Some(-1.))).is_err());
    assert_eq!(tree.root().value(), None);
    let rows = (0..20_000)
        .map(|i| NodeInput {
            key: key(i),
            parent: (i > 0).then(|| key(i - 1)),
            data: Arc::new(Value::Null),
            id: None,
            synthetic: false,
        })
        .collect::<Vec<_>>();
    let deep = Hierarchy::from_rows(
        HierarchyId::new(1),
        rows.clone(),
        HierarchyLimits::default(),
    )
    .unwrap();
    assert_eq!(deep.root().height(), 19_999);
    assert_eq!(deep.leaves(deep.root().handle()).unwrap().len(), 1);
    assert!(
        Hierarchy::from_rows(
            HierarchyId::new(1),
            rows,
            HierarchyLimits {
                max_depth: 100,
                ..HierarchyLimits::default()
            }
        )
        .is_err()
    );
    let grouped = Hierarchy::from_grouped(
        HierarchyId::new(3),
        key(0),
        vec![GroupedEntry {
            key: key(1),
            label: json!("a"),
            value: Value::Null,
            children: vec![GroupedEntry {
                key: key(2),
                label: json!("x"),
                value: json!(2),
                children: vec![],
            }],
        }],
        HierarchyLimits::default(),
    )
    .unwrap();
    assert_eq!(grouped.len(), 3);
    assert_eq!(
        grouped
            .get(grouped.leaves(grouped.root().handle()).unwrap()[0])
            .unwrap()
            .data()
            .as_ref(),
        &json!(["x", 2])
    );
    let before = records(&tree);
    assert!(
        tree.sort(|_, _| Err(chart_core::Diagnostic::error(
            chart_core::DiagnosticCode::Validation,
            "bad comparator",
            "fix"
        )))
        .is_err()
    );
    assert_eq!(records(&tree), before);
}

#[test]
fn native_deep_input_release_and_accessor_work_are_bounded() {
    let data = Arc::new(Value::Null);
    let make_chain = || {
        let mut node = NestedNode {
            key: key(20_000),
            data: data.clone(),
            children: vec![],
        };
        for i in (0..20_000).rev() {
            node = NestedNode {
                key: key(i),
                data: data.clone(),
                children: vec![node],
            };
        }
        node
    };
    let tree = Hierarchy::from_nested(
        HierarchyId::new(7),
        make_chain(),
        HierarchyLimits::default(),
    )
    .unwrap();
    assert_eq!(tree.root().height(), 20_000);
    assert!(
        Hierarchy::from_nested(
            HierarchyId::new(7),
            make_chain(),
            HierarchyLimits {
                max_depth: 1,
                ..HierarchyLimits::default()
            }
        )
        .is_err()
    );
    let mut calls = 0;
    assert!(
        Hierarchy::with_children(
            HierarchyId::new(8),
            (key(0), data.clone()),
            |_, _| {
                calls += 1;
                Ok((1..).map(|i| (key(i), data.clone())))
            },
            HierarchyLimits {
                max_nodes: 4,
                ..HierarchyLimits::default()
            }
        )
        .is_err()
    );
    assert_eq!(calls, 1);
    let rows = vec![(key(0), Arc::new(json!({"path":"/a/b/c"})))];
    let options = StratifyOptions {
        path_field: Some("path".into()),
        id_field: None,
        parent_field: None,
    };
    assert_eq!(
        Hierarchy::stratify(
            HierarchyId::new(1),
            rows,
            &options,
            HierarchyLimits::default()
        )
        .unwrap()
        .len(),
        1
    );
    let source = tree.root().handle();
    let copy = tree.copy_subtree(source, HierarchyId::new(9)).unwrap();
    assert!(tree.path(source, copy.root().handle()).is_err());
}

#[test]
fn custom_stratification_context_and_immutable_selector_reset() {
    let rows = vec![
        (
            key(1),
            Arc::new(
                json!({"custom_id":"r","custom_parent":null,"id":"r","parentId":null,"path":"/r"}),
            ),
        ),
        (
            key(2),
            Arc::new(
                json!({"custom_id":"a","custom_parent":"r","id":"a","parentId":"r","path":"/r/a"}),
            ),
        ),
    ];
    let mut id_calls = vec![];
    let mut parent_calls = vec![];
    let tree = Hierarchy::stratify_with(
        HierarchyId::new(1),
        rows.clone(),
        |value, index, all| {
            assert_eq!(all.len(), 2);
            assert_eq!(value, all[index].1.as_ref());
            id_calls.push(index);
            Ok(value["custom_id"].as_str().map(str::to_owned))
        },
        |value, index, all| {
            assert_eq!(all.len(), 2);
            parent_calls.push(index);
            Ok(value["custom_parent"].as_str().map(str::to_owned))
        },
        HierarchyLimits::default(),
    )
    .unwrap();
    assert_eq!(id_calls, [0, 1]);
    assert_eq!(parent_calls, [0, 1]);
    assert_eq!(tree.root().height(), 1);
    let baseline = StratifyOptions::default();
    let configured = StratifyOptions {
        id_field: Some("custom_id".into()),
        parent_field: Some("custom_parent".into()),
        path_field: None,
    };
    assert_eq!(configured.id_field.as_deref(), Some("custom_id"));
    assert_eq!(configured.parent_field.as_deref(), Some("custom_parent"));
    let path = StratifyOptions {
        path_field: Some("path".into()),
        id_field: None,
        parent_field: None,
    };
    for options in [&baseline, &configured, &path, &StratifyOptions::default()] {
        let h = Hierarchy::stratify(
            HierarchyId::new(1),
            rows.clone(),
            options,
            HierarchyLimits::default(),
        )
        .unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!(h.root().height(), 1);
    }
    assert_eq!(baseline.id_field.as_deref(), Some("id"));
    assert_eq!(baseline.parent_field.as_deref(), Some("parentId"));
    assert!(baseline.path_field.is_none());
}
