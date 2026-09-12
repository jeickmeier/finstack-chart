use super::*;
use serde_json::Value;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

/// A caller-keyed occurrence before topology construction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeInput {
    /// Stable occurrence identity, independent of position and lookup labels.
    pub key: HierarchyNodeId,
    /// Parent occurrence; exactly one root has no parent.
    pub parent: Option<HierarchyNodeId>,
    /// Shared immutable original payload; synthetic ancestors carry null.
    pub data: Arc<Value>,
    /// Optional parent-lookup label. Duplicate unreferenced leaf labels are valid.
    #[serde(default)]
    pub id: Option<String>,
    /// True only for an imputed ancestor with derived provenance.
    #[serde(default)]
    pub synthetic: bool,
}
/// Immutable occurrence metadata; indices are private implementation details.
#[derive(Clone, Debug)]
pub struct HierarchyNode {
    pub(crate) key: HierarchyNodeId,
    pub(crate) parent: Option<usize>,
    pub(crate) children: Vec<usize>,
    pub(crate) depth: usize,
    pub(crate) height: usize,
    pub(crate) value: Option<f64>,
    pub(crate) data: Arc<Value>,
    pub(crate) id: Option<String>,
    pub(crate) synthetic: bool,
}
/// Exact node handle scoped to one explicit hierarchy identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeHandle {
    /// Owning topology identity.
    pub hierarchy: HierarchyId,
    /// Stable occurrence identity.
    pub node: HierarchyNodeId,
}
/// A checked immutable tree. Copies share data, never mutable topology or layout history.
#[derive(Clone, Debug)]
pub struct Hierarchy {
    pub(crate) identity: HierarchyId,
    pub(crate) nodes: Arc<Vec<HierarchyNode>>,
    pub(crate) by_key: Arc<BTreeMap<HierarchyNodeId, usize>>,
    pub(crate) root: usize,
    pub(crate) limits: HierarchyLimits,
}
/// Visitation policy with reference-defined sibling ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisitOrder {
    /// Root, then each complete depth level.
    BreadthFirst,
    /// Parent before children, siblings in authored order.
    PreOrder,
    /// Children before parent, siblings in authored order.
    PostOrder,
}
/// Borrowed node context; no raw index escapes into a durable handle.
#[derive(Clone, Copy, Debug)]
pub struct NodeView<'a> {
    pub(crate) tree: &'a Hierarchy,
    pub(crate) index: usize,
}
impl<'a> NodeView<'a> {
    fn node(self) -> &'a HierarchyNode {
        &self.tree.nodes[self.index]
    }
    /// Scoped stable handle.
    pub fn handle(self) -> NodeHandle {
        self.tree.handle(self.index)
    }
    /// Shared immutable source payload.
    pub fn data(self) -> &'a Arc<Value> {
        &self.node().data
    }
    /// Optional lookup label.
    pub fn id(self) -> Option<&'a str> {
        self.node().id.as_deref()
    }
    /// Whether this is an imputed ancestor.
    pub fn synthetic(self) -> bool {
        self.node().synthetic
    }
    /// Distance from this hierarchy root.
    pub fn depth(self) -> usize {
        self.node().depth
    }
    /// Maximum distance to a descendant leaf.
    pub fn height(self) -> usize {
        self.node().height
    }
    /// Explicitly calculated sum/count, absent before aggregation.
    pub fn value(self) -> Option<f64> {
        self.node().value
    }
    /// Parent, absent only at this root.
    pub fn parent(self) -> Option<Self> {
        self.node().parent.map(|index| Self {
            tree: self.tree,
            index,
        })
    }
    /// Authored or explicitly sorted child order.
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + 'a {
        self.node().children.iter().map(move |&index| Self {
            tree: self.tree,
            index,
        })
    }
}
impl Hierarchy {
    /// Validate keyed rows atomically, preserving row order among each parent's children.
    pub fn from_rows(
        identity: HierarchyId,
        rows: Vec<NodeInput>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        Self::from_rows_work(identity, rows, limits, &mut Work::new(limits))
    }
    pub(super) fn from_rows_work(
        identity: HierarchyId,
        rows: Vec<NodeInput>,
        limits: HierarchyLimits,
        work: &mut Work,
    ) -> ChartResult<Self> {
        within(
            rows.len() <= limits.max_nodes,
            "Hierarchy node budget exceeded.",
        )?;
        if rows.is_empty() {
            return Err(invalid("no root"));
        }
        work.charge(rows.len())?;
        let mut by_key = BTreeMap::new();
        let mut bytes = 0usize;
        for (i, row) in rows.iter().enumerate() {
            if by_key.insert(row.key, i).is_some() {
                return Err(invalid("Duplicate hierarchy occurrence key."));
            }
            // Payload inspection is iterative, before cloning any nested values.
            bytes = bytes
                .checked_add(payload_bytes(&row.data, work)?)
                .and_then(|n| n.checked_add(row.id.as_ref().map_or(0, String::len)))
                .ok_or_else(|| invalid("Payload size overflow."))?;
            within(
                bytes <= limits.max_payload_bytes,
                "Hierarchy payload budget exceeded.",
            )?;
            if row.synthetic && !row.data.is_null() {
                return Err(invalid("Synthetic ancestors require a null payload."));
            }
        }
        let mut nodes: Vec<_> = rows
            .iter()
            .map(|r| HierarchyNode {
                key: r.key,
                parent: None,
                children: Vec::new(),
                depth: 0,
                height: 0,
                value: None,
                data: r.data.clone(),
                id: r.id.clone(),
                synthetic: r.synthetic,
            })
            .collect();
        let mut root = None;
        for (i, row) in rows.iter().enumerate() {
            if let Some(parent) = row.parent {
                let &p = by_key
                    .get(&parent)
                    .ok_or_else(|| invalid("Missing parent occurrence."))?;
                nodes[i].parent = Some(p);
                nodes[p].children.push(i);
            } else if root.replace(i).is_some() {
                return Err(invalid("multiple roots"));
            }
        }
        let root = root.ok_or_else(|| invalid("no root"))?;
        let mut queue = VecDeque::from([root]);
        let mut seen = vec![false; nodes.len()];
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(i) = queue.pop_front() {
            work.charge(1)?;
            if seen[i] {
                return Err(invalid("cycle"));
            }
            seen[i] = true;
            order.push(i);
            let depth = nodes[i].depth;
            within(
                depth <= limits.max_depth,
                "Hierarchy depth budget exceeded.",
            )?;
            for j in 0..nodes[i].children.len() {
                let c = nodes[i].children[j];
                nodes[c].depth = depth + 1;
                queue.push_back(c);
            }
        }
        if order.len() != nodes.len() {
            return Err(invalid("cycle"));
        }
        for &i in order.iter().rev() {
            nodes[i].height = nodes[i]
                .children
                .iter()
                .map(|&c| nodes[c].height + 1)
                .max()
                .unwrap_or(0);
        }
        Ok(Self {
            identity,
            nodes: Arc::new(nodes),
            by_key: Arc::new(by_key),
            root,
            limits,
        })
    }
    /// Construct one standalone occurrence with no aggregation or rendering requirement.
    pub fn node(
        identity: HierarchyId,
        key: HierarchyNodeId,
        data: Arc<Value>,
        limits: HierarchyLimits,
    ) -> ChartResult<Self> {
        Self::from_rows(
            identity,
            vec![NodeInput {
                key,
                parent: None,
                data,
                id: None,
                synthetic: false,
            }],
            limits,
        )
    }
    /// Explicit owner identity.
    pub fn identity(&self) -> HierarchyId {
        self.identity
    }
    /// Number of retained occurrences, including synthetic parents.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Whether there are no nodes (validated hierarchies always have a root).
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Budgets retained for each subsequent operation.
    pub fn limits(&self) -> HierarchyLimits {
        self.limits
    }
    /// Breadth-first iteration over this snapshot, checked against its work budget.
    pub fn iter(&self) -> ChartResult<impl ExactSizeIterator<Item = NodeView<'_>>> {
        Ok(self
            .order(VisitOrder::BreadthFirst)?
            .into_iter()
            .map(|index| self.view(index)))
    }
    /// Root context.
    pub fn root(&self) -> NodeView<'_> {
        self.view(self.root)
    }
    pub(crate) fn view(&self, index: usize) -> NodeView<'_> {
        NodeView { tree: self, index }
    }
    pub(crate) fn handle(&self, index: usize) -> NodeHandle {
        NodeHandle {
            hierarchy: self.identity,
            node: self.nodes[index].key,
        }
    }
    pub(crate) fn index(&self, node: NodeHandle) -> ChartResult<usize> {
        if node.hierarchy != self.identity {
            return Err(invalid("Node belongs to another hierarchy."));
        }
        self.by_key
            .get(&node.node)
            .copied()
            .ok_or_else(|| invalid("Unknown hierarchy node."))
    }
    /// Resolve an exact scoped handle in this snapshot.
    pub fn get(&self, node: NodeHandle) -> ChartResult<NodeView<'_>> {
        Ok(self.view(self.index(node)?))
    }
    pub(crate) fn order_from(&self, root: usize, order: VisitOrder) -> ChartResult<Vec<usize>> {
        let mut work = Work::new(self.limits);
        let mut result = Vec::new();
        match order {
            VisitOrder::BreadthFirst => {
                let mut q = VecDeque::from([root]);
                while let Some(i) = q.pop_front() {
                    work.charge(1)?;
                    result.push(i);
                    q.extend(self.nodes[i].children.iter().copied());
                }
            }
            VisitOrder::PreOrder | VisitOrder::PostOrder => {
                let mut stack = vec![root];
                while let Some(i) = stack.pop() {
                    work.charge(1)?;
                    result.push(i);
                    if order == VisitOrder::PreOrder {
                        stack.extend(self.nodes[i].children.iter().rev().copied());
                    } else {
                        stack.extend(self.nodes[i].children.iter().copied());
                    }
                }
                if order == VisitOrder::PostOrder {
                    result.reverse();
                }
            }
        }
        Ok(result)
    }
    pub(crate) fn order(&self, order: VisitOrder) -> ChartResult<Vec<usize>> {
        self.order_from(self.root, order)
    }
    /// Visit a subtree with reference node/index/root context; errors stop callbacks.
    pub fn visit(
        &self,
        node: NodeHandle,
        order: VisitOrder,
        mut callback: impl FnMut(NodeView<'_>, usize, NodeView<'_>) -> ChartResult<()>,
    ) -> ChartResult<()> {
        let root = self.index(node)?;
        for (i, n) in self.order_from(root, order)?.into_iter().enumerate() {
            callback(self.view(n), i, self.view(root))?;
        }
        Ok(())
    }
    /// Breadth-first descendants, including the supplied root.
    pub fn descendants(&self, node: NodeHandle) -> ChartResult<Vec<NodeHandle>> {
        Ok(self
            .order_from(self.index(node)?, VisitOrder::BreadthFirst)?
            .into_iter()
            .map(|i| self.handle(i))
            .collect())
    }
    /// Rootward ancestors, starting with the supplied node.
    pub fn ancestors(&self, node: NodeHandle) -> ChartResult<Vec<NodeHandle>> {
        let mut next = Some(self.index(node)?);
        let mut out = Vec::new();
        let mut work = Work::new(self.limits);
        while let Some(i) = next {
            work.charge(1)?;
            out.push(self.handle(i));
            next = self.nodes[i].parent;
        }
        Ok(out)
    }
    /// Leaves in depth-first authored/sorted order.
    pub fn leaves(&self, node: NodeHandle) -> ChartResult<Vec<NodeHandle>> {
        Ok(self
            .order_from(self.index(node)?, VisitOrder::PreOrder)?
            .into_iter()
            .filter(|&i| self.nodes[i].children.is_empty())
            .map(|i| self.handle(i))
            .collect())
    }
    /// First breadth-first match; the callback sees the subtree root.
    pub fn find(
        &self,
        node: NodeHandle,
        mut predicate: impl FnMut(NodeView<'_>, usize, NodeView<'_>) -> ChartResult<bool>,
    ) -> ChartResult<Option<NodeHandle>> {
        let root = self.index(node)?;
        for (i, n) in self
            .order_from(root, VisitOrder::BreadthFirst)?
            .into_iter()
            .enumerate()
        {
            if predicate(self.view(n), i, self.view(root))? {
                return Ok(Some(self.handle(n)));
            }
        }
        Ok(None)
    }
    /// Parent-child links in breadth-first target order, excluding this subtree root.
    pub fn links(&self, node: NodeHandle) -> ChartResult<Vec<(NodeHandle, NodeHandle)>> {
        Ok(self
            .order_from(self.index(node)?, VisitOrder::BreadthFirst)?
            .into_iter()
            .skip(1)
            .map(|i| {
                (
                    self.handle(self.nodes[i].parent.expect("nonroot parent")),
                    self.handle(i),
                )
            })
            .collect())
    }
    /// Shortest tree path, including both endpoints.
    pub fn path(&self, start: NodeHandle, end: NodeHandle) -> ChartResult<Vec<NodeHandle>> {
        let a = self.ancestors(start)?;
        let b = self.ancestors(end)?;
        let (mut i, mut j) = (a.len(), b.len());
        while i > 0 && j > 0 && a[i - 1] == b[j - 1] {
            i -= 1;
            j -= 1;
        }
        let mut out = a[..=i].to_vec();
        out.extend(b[..j].iter().rev().copied());
        Ok(out)
    }
    /// Sum own nonnegative finite weights and descendant values; missing is explicitly zero.
    pub fn sum(
        &self,
        mut own: impl FnMut(NodeView<'_>) -> ChartResult<Option<f64>>,
    ) -> ChartResult<Self> {
        let mut nodes = (*self.nodes).clone();
        for i in self.order(VisitOrder::PostOrder)? {
            let mut value = weight(own(self.view(i))?.unwrap_or(0.))?;
            for &c in nodes[i].children.iter().rev() {
                value += nodes[c].value.expect("postorder aggregate");
            }
            nodes[i].value = Some(weight(value)?);
        }
        Ok(Self {
            nodes: Arc::new(nodes),
            ..self.clone()
        })
    }
    /// Count descendant leaves; internal own payload values do not participate.
    pub fn count(&self) -> ChartResult<Self> {
        let mut nodes = (*self.nodes).clone();
        for i in self.order(VisitOrder::PostOrder)? {
            let value = if nodes[i].children.is_empty() {
                1.
            } else {
                nodes[i]
                    .children
                    .iter()
                    .map(|&c| nodes[c].value.expect("postorder count"))
                    .sum()
            };
            nodes[i].value = Some(value);
        }
        Ok(Self {
            nodes: Arc::new(nodes),
            ..self.clone()
        })
    }
    /// Stable sibling sorting. A failed comparator leaves this snapshot unchanged.
    pub fn sort(
        &self,
        mut compare: impl FnMut(NodeView<'_>, NodeView<'_>) -> ChartResult<Ordering>,
    ) -> ChartResult<Self> {
        let mut sorted = self.clone();
        let mut work = Work::new(self.limits);
        let mut stack = vec![self.root];
        while let Some(i) = stack.pop() {
            work.charge(1)?;
            let mut children = sorted.nodes[i].children.clone();
            let mut failure = None;
            children.sort_by(|&a, &b| {
                if failure.is_some() {
                    return Ordering::Equal;
                }
                match work
                    .charge(1)
                    .and_then(|()| compare(sorted.view(a), sorted.view(b)))
                {
                    Ok(order) => order,
                    Err(error) => {
                        failure = Some(error);
                        Ordering::Equal
                    }
                }
            });
            if let Some(error) = failure {
                return Err(error);
            }
            stack.extend(children.iter().rev().copied());
            Arc::make_mut(&mut sorted.nodes)[i].children = children;
        }
        Ok(sorted)
    }
    /// Copy a subtree into a distinct owner, retaining keys/values and shared payloads.
    pub fn copy_subtree(&self, node: NodeHandle, identity: HierarchyId) -> ChartResult<Self> {
        if identity == self.identity {
            return Err(invalid(
                "A subtree copy requires a distinct hierarchy identity.",
            ));
        }
        let root = self.index(node)?;
        let indices = self.order_from(root, VisitOrder::PreOrder)?;
        let rows = indices
            .iter()
            .map(|&i| {
                let n = &self.nodes[i];
                NodeInput {
                    key: n.key,
                    parent: if i == root {
                        None
                    } else {
                        n.parent.map(|p| self.nodes[p].key)
                    },
                    data: n.data.clone(),
                    id: n.id.clone(),
                    synthetic: n.synthetic,
                }
            })
            .collect();
        let mut copied = Self::from_rows(identity, rows, self.limits)?;
        let nodes = Arc::make_mut(&mut copied.nodes);
        for (j, &i) in indices.iter().enumerate() {
            nodes[j].value = self.nodes[i].value;
        }
        Ok(copied)
    }
}
pub(super) fn payload_bytes(value: &Value, work: &mut Work) -> ChartResult<usize> {
    let mut stack = vec![(value, 0)];
    let mut bytes = 0usize;
    while let Some((v, depth)) = stack.pop() {
        work.charge(1)?;
        within(depth <= 32, "Hierarchy payload nesting budget exceeded.")?;
        let amount = match v {
            Value::String(s) => s.len(),
            Value::Array(a) => {
                stack.extend(a.iter().map(|x| (x, depth + 1)));
                0
            }
            Value::Object(o) => {
                stack.extend(o.values().map(|x| (x, depth + 1)));
                o.keys().map(String::len).sum()
            }
            _ => 8,
        };
        bytes = bytes
            .checked_add(amount)
            .ok_or_else(|| invalid("Payload size overflow."))?;
    }
    Ok(bytes)
}
