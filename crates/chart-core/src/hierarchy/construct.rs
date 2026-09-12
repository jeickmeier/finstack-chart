use super::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Native nested input with explicit durable occurrence keys and shared payloads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NestedNode {
    /// Stable occurrence identity.
    pub key: HierarchyNodeId,
    /// Original immutable payload.
    pub data: Arc<Value>,
    /// Ordered child occurrences.
    pub children: Vec<NestedNode>,
}
/// Ordered grouped/rollup input. Entries do not use an unordered object dictionary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupedEntry {
    /// Durable occurrence identity.
    pub key: HierarchyNodeId,
    /// Group key retained as the first entry of the source payload pair.
    pub label: Value,
    /// Rollup/leaf value; grouping nodes may retain a separate own value.
    pub value: Value,
    /// Ordered nested grouping entries.
    pub children: Vec<GroupedEntry>,
}
/// Declarative table/path accessors, with explicit disabled/reset states.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StratifyOptions {
    /// ID field, or no labels when disabled.
    pub id_field: Option<String>,
    /// Parent-label field, or all rows are roots when disabled.
    pub parent_field: Option<String>,
    /// Path field; when present it takes precedence over ID/parent fields.
    pub path_field: Option<String>,
}
impl Default for StratifyOptions {
    fn default() -> Self {
        Self {
            id_field: Some("id".into()),
            parent_field: Some("parentId".into()),
            path_field: None,
        }
    }
}
impl Hierarchy {
    /// Consume native nested input iteratively. Payloads remain shared and immutable.
    pub fn from_nested(
        identity: HierarchyId,
        root: NestedNode,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        let mut stack = vec![(root, None, 0)];
        let mut rows = Vec::new();
        let mut work = Work::new(limits);
        while let Some((mut node, parent, depth)) = stack.pop() {
            work.charge(1)?;
            within(
                depth <= limits.max_depth,
                "Hierarchy depth budget exceeded.",
            )?;
            within(
                rows.len() < limits.max_nodes,
                "Hierarchy node budget exceeded.",
            )?;
            let key = node.key;
            rows.push(NodeInput {
                key,
                parent,
                data: node.data.clone(),
                id: None,
                synthetic: false,
            });
            stack.extend(
                std::mem::take(&mut node.children)
                    .into_iter()
                    .rev()
                    .map(|c| (c, Some(key), depth + 1)),
            );
        }
        Self::from_rows_work(identity, rows, limits, &mut work)
    }
    /// Construct from native child accessors over shared immutable payloads.
    /// Accessors provide durable keys; repeated payloads still get distinct occurrence keys.
    pub fn with_children<I>(
        identity: HierarchyId,
        root: (HierarchyNodeId, Arc<Value>),
        mut children: impl FnMut(HierarchyNodeId, &Arc<Value>) -> ChartResult<I>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self>
    where
        I: IntoIterator<Item = (HierarchyNodeId, Arc<Value>)>,
    {
        let mut stack = vec![(root, None, 0)];
        let mut rows = Vec::new();
        let mut seen = BTreeSet::new();
        let mut work = Work::new(limits);
        while let Some(((key, data), parent, depth)) = stack.pop() {
            work.charge(1)?;
            within(
                depth <= limits.max_depth,
                "Hierarchy depth budget exceeded.",
            )?;
            within(
                rows.len() < limits.max_nodes,
                "Hierarchy node budget exceeded.",
            )?;
            if !seen.insert(key) {
                return Err(invalid("Duplicate hierarchy occurrence key."));
            }
            let mut list = Vec::new();
            for child in children(key, &data)? {
                work.charge(1)?;
                within(
                    rows.len() + stack.len() + list.len() < limits.max_nodes,
                    "Hierarchy child accessor exceeded node budget.",
                )?;
                list.push(child);
            }
            rows.push(NodeInput {
                key,
                parent,
                data,
                id: None,
                synthetic: false,
            });
            stack.extend(list.into_iter().rev().map(|c| (c, Some(key), depth + 1)));
        }
        Self::from_rows_work(identity, rows, limits, &mut work)
    }
    /// Normalize ordered grouping nodes, retaining group key/value pairs as payloads.
    pub fn from_grouped(
        identity: HierarchyId,
        root_key: HierarchyNodeId,
        entries: Vec<GroupedEntry>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        let mut rows = vec![NodeInput {
            key: root_key,
            parent: None,
            data: Arc::new(Value::Null),
            id: None,
            synthetic: true,
        }];
        let mut stack: Vec<_> = entries
            .into_iter()
            .rev()
            .map(|e| (e, root_key, 1))
            .collect();
        let mut work = Work::new(limits);
        while let Some((mut entry, parent, depth)) = stack.pop() {
            work.charge(1)?;
            within(
                rows.len() < limits.max_nodes,
                "Hierarchy node budget exceeded.",
            )?;
            within(
                depth <= limits.max_depth,
                "Hierarchy depth budget exceeded.",
            )?;
            let key = entry.key;
            rows.push(NodeInput {
                key,
                parent: Some(parent),
                data: Arc::new(Value::Array(vec![
                    std::mem::take(&mut entry.label),
                    std::mem::take(&mut entry.value),
                ])),
                id: None,
                synthetic: false,
            });
            stack.extend(
                std::mem::take(&mut entry.children)
                    .into_iter()
                    .rev()
                    .map(|e| (e, key, depth + 1)),
            );
        }
        Self::from_rows_work(identity, rows, limits, &mut work)
    }
    /// Apply declarative field accessors through the shared table/path constructors.
    pub fn stratify(
        identity: HierarchyId,
        rows: Vec<(HierarchyNodeId, Arc<Value>)>,
        options: &StratifyOptions,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        preflight_payloads(&rows, limits, &mut Work::new(limits))?;
        if let Some(field) = &options.path_field {
            let paths = rows
                .into_iter()
                .map(|(key, data)| {
                    let path = field_text(&data, Some(field))?
                        .ok_or_else(|| invalid("A path accessor must return text."))?;
                    Ok((key, data, path))
                })
                .collect::<ChartResult<Vec<_>>>()?;
            Self::from_paths(identity, paths, limits)
        } else {
            Self::stratify_with(
                identity,
                rows,
                |data, _, _| field_text(data, options.id_field.as_deref()),
                |data, _, _| field_text(data, options.parent_field.as_deref()),
                limits,
            )
        }
    }
    /// Native ID/parent accessors receive source row/index/complete input context.
    pub fn stratify_with(
        identity: HierarchyId,
        rows: Vec<(HierarchyNodeId, Arc<Value>)>,
        mut id: impl FnMut(
            &Value,
            usize,
            &[(HierarchyNodeId, Arc<Value>)],
        ) -> ChartResult<Option<String>>,
        mut parent: impl FnMut(
            &Value,
            usize,
            &[(HierarchyNodeId, Arc<Value>)],
        ) -> ChartResult<Option<String>>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        Self::stratify_work(
            identity,
            rows,
            &mut id,
            &mut parent,
            limits,
            &mut Work::new(limits),
        )
    }
    fn stratify_work(
        identity: HierarchyId,
        rows: Vec<(HierarchyNodeId, Arc<Value>)>,
        id: &mut impl FnMut(
            &Value,
            usize,
            &[(HierarchyNodeId, Arc<Value>)],
        ) -> ChartResult<Option<String>>,
        parent: &mut impl FnMut(
            &Value,
            usize,
            &[(HierarchyNodeId, Arc<Value>)],
        ) -> ChartResult<Option<String>>,
        limits: HierarchyLimits,
        work: &mut Work,
    ) -> ChartResult<Self> {
        within(
            rows.len() <= limits.max_nodes,
            "Hierarchy node budget exceeded.",
        )?;
        preflight_payloads(&rows, limits, work)?;
        let mut label_bytes = 0usize;
        let mut labels = Vec::with_capacity(rows.len());
        let mut parents = Vec::with_capacity(rows.len());
        let mut lookup = BTreeMap::new();
        for (i, (_, data)) in rows.iter().enumerate() {
            work.charge(2)?;
            let label = id(data, i, &rows)?.filter(|s| !s.is_empty());
            let p = parent(data, i, &rows)?.filter(|s| !s.is_empty());
            label_bytes = label_bytes
                .checked_add(label.as_ref().map_or(0, String::len))
                .and_then(|n| n.checked_add(p.as_ref().map_or(0, String::len)))
                .ok_or_else(|| invalid("Lookup label size overflow."))?;
            within(
                label_bytes <= limits.max_payload_bytes,
                "Hierarchy accessor labels exceed byte budget.",
            )?;
            if let Some(s) = &label {
                lookup
                    .entry(s.clone())
                    .and_modify(|v| *v = None)
                    .or_insert(Some(i));
            }
            labels.push(label);
            parents.push(p);
        }
        let mut result = Vec::with_capacity(rows.len());
        for (i, (key, data)) in rows.iter().enumerate() {
            let parent = match &parents[i] {
                None => None,
                Some(p) => match lookup.get(p) {
                    None => return Err(invalid(format!("missing: {p}"))),
                    Some(None) => return Err(invalid(format!("ambiguous: {p}"))),
                    Some(Some(index)) => Some(rows[*index].0),
                },
            };
            result.push(NodeInput {
                key: *key,
                parent,
                data: data.clone(),
                id: labels[i].clone(),
                synthetic: false,
            });
        }
        Self::from_rows_work(identity, result, limits, work)
    }
    /// Slash-path stratification with escaped delimiters and minimal imputed root.
    pub fn from_paths(
        identity: HierarchyId,
        input: Vec<(HierarchyNodeId, Arc<Value>, String)>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        within(
            input.len() <= limits.max_nodes,
            "Hierarchy node budget exceeded.",
        )?;
        let mut work = Work::new(limits);
        let mut keys: BTreeSet<_> = input.iter().map(|(k, _, _)| *k).collect();
        if keys.len() != input.len() {
            return Err(invalid("Duplicate hierarchy occurrence key."));
        }
        let original = input.len();
        let mut rows = Vec::with_capacity(original);
        let mut paths = Vec::with_capacity(original);
        let mut parents = Vec::with_capacity(original);
        let mut total_bytes = 0usize;
        for (key, data, path) in input {
            work.charge(path.len())?;
            let normalized = normalize_path(path);
            total_bytes = total_bytes
                .checked_add(normalized.len())
                .ok_or_else(|| invalid("Path size overflow."))?;
            within(
                total_bytes <= limits.max_payload_bytes,
                "Hierarchy path byte budget exceeded.",
            )?;
            parents.push(parent_path(&normalized));
            paths.push(normalized);
            rows.push((key, data));
        }
        let mut known: BTreeSet<String> = paths.iter().cloned().collect();
        known.insert(String::new());
        let mut cursor = 0;
        let mut next_key = 0u64;
        while cursor < parents.len() {
            work.charge(1)?;
            let p = parents[cursor].clone();
            if known.insert(p.clone()) {
                within(
                    rows.len() < limits.max_nodes,
                    "Imputed ancestors exceed node budget.",
                )?;
                work.charge(p.len())?;
                total_bytes = total_bytes
                    .checked_add(p.len())
                    .ok_or_else(|| invalid("Path size overflow."))?;
                within(
                    total_bytes <= limits.max_payload_bytes,
                    "Imputed paths exceed byte budget.",
                )?;
                while keys.contains(&HierarchyNodeId::new(next_key)) {
                    next_key = next_key
                        .checked_add(1)
                        .ok_or_else(|| invalid("Synthetic key space exhausted."))?;
                }
                let key = HierarchyNodeId::new(next_key);
                keys.insert(key);
                rows.push((key, Arc::new(Value::Null)));
                parents.push(parent_path(&p));
                paths.push(p);
            }
            cursor += 1;
        }
        let mut tree = Self::stratify_work(
            identity,
            rows,
            &mut |_, i, _| Ok(Some(paths[i].clone())),
            &mut |_, i, _| Ok(Some(parents[i].clone())),
            limits,
            &mut work,
        )?;
        for node in Arc::make_mut(&mut tree.nodes).iter_mut().skip(original) {
            node.synthetic = true;
        }
        // Drop redundant imputed single-child prefixes without recursive copying.
        let mut root = tree.root;
        while tree.nodes[root].synthetic && tree.nodes[root].children.len() == 1 {
            root = tree.nodes[root].children[0];
        }
        if root != tree.root {
            let indices = tree.order_from(root, VisitOrder::PreOrder)?;
            let rows = indices
                .iter()
                .map(|&i| {
                    let n = &tree.nodes[i];
                    NodeInput {
                        key: n.key,
                        parent: if i == root {
                            None
                        } else {
                            n.parent.map(|p| tree.nodes[p].key)
                        },
                        data: n.data.clone(),
                        id: n.id.clone(),
                        synthetic: n.synthetic,
                    }
                })
                .collect();
            tree = Self::from_rows_work(identity, rows, limits, &mut work)?;
        }
        Ok(tree)
    }
}
fn field_text(data: &Value, field: Option<&str>) -> ChartResult<Option<String>> {
    match field.and_then(|f| data.get(f)) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        _ => Err(invalid(
            "Hierarchy label/path fields must be text or null; exact IDs use decimal strings.",
        )),
    }
}
fn delimiter(path: &[u8], i: usize) -> bool {
    if path.get(i) != Some(&b'/') {
        return false;
    }
    let mut j = i;
    while j > 0 && path[j - 1] == b'\\' {
        j -= 1;
    }
    (i - j).is_multiple_of(2)
}
fn normalize_path(mut path: String) -> String {
    let n = path.len();
    if n > 0 && delimiter(path.as_bytes(), n - 1) && (n < 2 || !delimiter(path.as_bytes(), n - 2)) {
        path.pop();
    }
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    path
}
fn parent_path(path: &str) -> String {
    if path.len() < 2 {
        return String::new();
    }
    let mut i = path.len() - 1;
    while i > 1 && !delimiter(path.as_bytes(), i) {
        i -= 1;
    }
    path[..i].to_string()
}

// Deep invalid native inputs also release iteratively when construction returns early.
impl Drop for NestedNode {
    fn drop(&mut self) {
        let mut stack = std::mem::take(&mut self.children);
        while let Some(mut n) = stack.pop() {
            stack.append(&mut n.children);
        }
    }
}
impl Drop for GroupedEntry {
    fn drop(&mut self) {
        let mut stack = std::mem::take(&mut self.children);
        while let Some(mut n) = stack.pop() {
            stack.append(&mut n.children);
        }
    }
}

fn preflight_payloads(
    rows: &[(HierarchyNodeId, Arc<Value>)],
    limits: HierarchyLimits,
    work: &mut Work,
) -> ChartResult<()> {
    within(
        rows.len() <= limits.max_nodes,
        "Hierarchy node budget exceeded.",
    )?;
    let mut bytes = 0usize;
    for (_, data) in rows {
        bytes = bytes
            .checked_add(super::topology::payload_bytes(data, work)?)
            .ok_or_else(|| invalid("Payload size overflow."))?;
        within(
            bytes <= limits.max_payload_bytes,
            "Hierarchy payload byte budget exceeded.",
        )?;
    }
    Ok(())
}
