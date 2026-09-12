//! Shared independent hierarchy fixture inputs and exact structural comparison.
#![allow(dead_code)]
use chart_core::hierarchy::*;
use serde_json::{Value, json};
use std::sync::Arc;
pub fn key(i: usize) -> HierarchyNodeId {
    HierarchyNodeId::new(i as u64 + 1)
}
pub fn nested(input: &Value) -> Hierarchy {
    let mut rows = Vec::new();
    let mut stack = vec![(input.clone(), None)];
    while let Some((data, parent)) = stack.pop() {
        let k = data
            .get("key")
            .and_then(Value::as_str)
            .map(|v| HierarchyNodeId::new(v.parse().unwrap()))
            .unwrap_or_else(|| key(rows.len()));
        let children = data
            .get("children")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        rows.push(NodeInput {
            key: k,
            parent,
            data: Arc::new(data),
            id: None,
            synthetic: false,
        });
        stack.extend(children.into_iter().rev().map(|d| (d, Some(k))));
    }
    Hierarchy::from_rows(HierarchyId::new(1), rows, HierarchyLimits::default()).unwrap()
}
pub fn name(node: NodeView<'_>) -> Value {
    node.data().get("name").cloned().unwrap_or(Value::Null)
}
pub fn label(node: NodeView<'_>) -> Value {
    node.data()
        .get("name")
        .cloned()
        .or_else(|| node.id().map(|s| json!(s)))
        .unwrap_or(Value::Null)
}
pub fn records(tree: &Hierarchy) -> Value {
    Value::Array(tree.descendants(tree.root().handle()).unwrap().into_iter().map(|h|{let n=tree.get(h).unwrap();let mut r=json!({"name":name(n),"parent":n.parent().map(label).unwrap_or(Value::Null),"children":n.children().map(label).collect::<Vec<_>>(),"depth":n.depth(),"height":n.height()});if let Some(id)=n.id(){r["id"]=json!(id);}
if let Some(v)=n.value(){r["value"]=json!(v);}r}).collect())
}
pub fn names(tree: &Hierarchy, handles: Vec<NodeHandle>) -> Value {
    json!(
        handles
            .into_iter()
            .map(|h| name(tree.get(h).unwrap()))
            .collect::<Vec<_>>()
    )
}
pub fn assert_json(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64()),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b) {
                assert_json(x, y);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                assert_json(v, b.get(k).unwrap());
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
