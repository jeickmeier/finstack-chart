//! Owned atomic standalone hierarchy operations and immutable result records.
use super::*;
use crate::grammar::ExtensionRegistry;
use serde_json::Value;
use std::sync::Arc;

/// Owned metadata plus optional geometry from the exact same snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeRecord {
    /// Owner-scoped stable occurrence.
    pub handle: NodeHandle,
    /// Original immutable payload.
    pub data: Arc<Value>,
    /// Lookup label, separate from identity.
    pub id: Option<String>,
    /// Synthetic ancestor marker.
    pub synthetic: bool,
    /// Parent occurrence.
    pub parent: Option<NodeHandle>,
    /// Current ordered children.
    pub children: Vec<NodeHandle>,
    /// Root-relative edge depth.
    pub depth: usize,
    /// Maximum leaf distance.
    pub height: usize,
    /// Explicit aggregate, absent before sum/count.
    pub value: Option<f64>,
    /// Last resolved geometry; absent after topology/value/order changes.
    pub geometry: Option<NodeGeometry>,
}
impl NodeRecord {
    /// Capture immutable owned metadata.
    pub fn from_node(node: NodeView<'_>, layout: Option<&HierarchyLayout>) -> ChartResult<Self> {
        Ok(Self {
            handle: node.handle(),
            data: node.data().clone(),
            id: node.id().map(str::to_owned),
            synthetic: node.synthetic(),
            parent: node.parent().map(NodeView::handle),
            children: node.children().map(NodeView::handle).collect(),
            depth: node.depth(),
            height: node.height(),
            value: node.value(),
            geometry: layout.map(|l| l.geometry(node.handle())).transpose()?,
        })
    }
}
/// Atomic state changes. Hosts forward these to one core session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchyChange {
    /// Replace topology, preserving compatible resquarify history until next layout.
    Replace(HierarchyEnvelope),
    /// Sum own values with descendant values.
    Sum(ScalarAccessor),
    /// Count leaves.
    Count,
    /// Stable sibling sort.
    Sort(Comparator),
    /// Resolve layout and retain exact configuration and result.
    Layout(LayoutSpec),
    /// Explicitly discard resquarify rows; retained geometry remains immutable.
    ResetHistory,
}
/// Query operations, all returning owned JSON at the host boundary.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchyQuery {
    /// All node metadata in breadth-first order.
    Nodes,
    /// Metadata for one scoped occurrence.
    Node(NodeHandle),
    /// Ancestors starting with the supplied node.
    Ancestors(NodeHandle),
    /// Breadth-first descendants including root.
    Descendants(NodeHandle),
    /// Leaves in pre-order.
    Leaves(NodeHandle),
    /// Parent/child pairs in breadth-first target order.
    Links(NodeHandle),
    /// Shortest inclusive path.
    Path {
        /// Start occurrence.
        start: NodeHandle,
        /// End occurrence.
        end: NodeHandle,
    },
    /// Callback-equivalent records with exact node/index/root context.
    Visit {
        /// Subtree root.
        root: NodeHandle,
        /// Traversal policy.
        order: VisitOrder,
    },
    /// First match by payload-field equality.
    Find {
        /// Subtree root.
        root: NodeHandle,
        /// Payload field.
        field: String,
        /// Exact JSON value.
        value: Value,
    },
    /// First registered predicate match with full callback context.
    FindRegistered {
        /// Subtree root.
        root: NodeHandle,
        /// Captured predicate.
        predicate: RegisteredOperation,
    },
    /// Last layout configuration and current retained row/member counts.
    Configuration,
    /// Invoke a built-in tiler on a parent in explicit bounds.
    Tile {
        /// Parent with children.
        parent: NodeHandle,
        /// Tiler including ratio factory.
        tiler: Tiler,
        /// Explicit rectangle bounds.
        bounds: [f64; 4],
    },
}
/// Stateful, owned standalone API. Copies capture immutable topology and independent history.
#[derive(Clone)]
pub struct HierarchySession {
    tree: Hierarchy,
    registry: ExtensionRegistry,
    history: TreemapHistory,
    layout: Option<HierarchyLayout>,
    configuration: Option<LayoutSpec>,
}
impl HierarchySession {
    /// Construct with captured native operations.
    pub fn new(envelope: HierarchyEnvelope, registry: &ExtensionRegistry) -> ChartResult<Self> {
        Ok(Self {
            tree: envelope.build(registry)?,
            registry: registry.clone(),
            history: TreemapHistory::default(),
            layout: None,
            configuration: None,
        })
    }
    /// Decode the bounded versioned construction envelope.
    pub fn from_json(input: &str, registry: &ExtensionRegistry) -> ChartResult<Self> {
        Self::new(crate::portable::decode(input)?, registry)
    }
    /// Current immutable topology.
    pub fn hierarchy(&self) -> &Hierarchy {
        &self.tree
    }
    /// Current immutable numeric layout, if one has been resolved.
    pub fn layout(&self) -> Option<&HierarchyLayout> {
        self.layout.as_ref()
    }
    /// Clone a subtree into a distinct owner; layout/history reset as with native copy.
    pub fn copy_subtree(&self, node: NodeHandle, identity: HierarchyId) -> ChartResult<Self> {
        Ok(Self {
            tree: self.tree.copy_subtree(node, identity)?,
            registry: self.registry.clone(),
            history: TreemapHistory::default(),
            layout: None,
            configuration: None,
        })
    }
    /// Apply checked mutation atomically, including any callback or layout failure.
    pub fn apply(&mut self, change: HierarchyChange) -> ChartResult<()> {
        match change {
            HierarchyChange::Layout(spec) => {
                let mut history = self.history.clone();
                let layout = spec.layout_with(&self.tree, &self.registry, &mut history, true)?;
                self.history = history;
                self.layout = Some(layout);
                self.configuration = Some(spec);
                return Ok(());
            }
            HierarchyChange::ResetHistory => {
                self.history.reset();
                return Ok(());
            }
            HierarchyChange::Replace(envelope) => self.tree = envelope.build(&self.registry)?,
            HierarchyChange::Sum(accessor) => {
                let accessor = accessor.compile(&self.registry)?;
                self.tree = self
                    .tree
                    .sum(|node| accessor.evaluate(node, crate::grammar::HierarchyScalar::Value))?;
            }
            HierarchyChange::Count => self.tree = self.tree.count()?,
            HierarchyChange::Sort(comparator) => {
                self.tree = comparator.sort(&self.tree, &self.registry)?
            }
        }
        self.history.retain_compatible(&self.tree);
        self.layout = None;
        self.configuration = None;
        Ok(())
    }
    /// Decode and execute one bounded operation.
    pub fn apply_json(&mut self, input: &str) -> ChartResult<()> {
        self.apply(crate::portable::decode(input)?)
    }
    /// Query exact owned results; serialization never borrows host memory.
    pub fn query(&self, query: HierarchyQuery) -> ChartResult<Value> {
        let tree = &self.tree;
        match query {
            HierarchyQuery::Nodes => json(
                &tree
                    .iter()?
                    .map(|n| NodeRecord::from_node(n, self.layout.as_ref()))
                    .collect::<ChartResult<Vec<_>>>()?,
            ),
            HierarchyQuery::Node(node) => json(&NodeRecord::from_node(
                tree.get(node)?,
                self.layout.as_ref(),
            )?),
            HierarchyQuery::Ancestors(node) => json(&tree.ancestors(node)?),
            HierarchyQuery::Descendants(node) => json(&tree.descendants(node)?),
            HierarchyQuery::Leaves(node) => json(&tree.leaves(node)?),
            HierarchyQuery::Links(node) => json(&tree.links(node)?),
            HierarchyQuery::Path { start, end } => json(&tree.path(start, end)?),
            HierarchyQuery::Visit { root, order } => {
                let mut records = Vec::new();
                tree.visit(root, order, |node, index, root| {
                    records.push((
                        NodeRecord::from_node(node, self.layout.as_ref())?,
                        index,
                        root.handle(),
                    ));
                    Ok(())
                })?;
                json(&records)
            }
            HierarchyQuery::Find { root, field, value } => {
                json(&tree.find(root, |node, _, _| {
                    Ok(node.data().get(&field) == Some(&value))
                })?)
            }
            HierarchyQuery::FindRegistered { root, predicate } => {
                let op = predicate.resolve(&self.registry)?;
                json(&tree.find(root, |node, index, root| {
                    op.implementation
                        .predicate(node, index, root, &predicate.parameters)
                })?)
            }
            HierarchyQuery::Configuration => {
                let effective_ratio = match &self.configuration {
                    Some(LayoutSpec::Treemap { options, .. }) => options.tile.ratio()?,
                    _ => None,
                };
                Ok(
                    serde_json::json!({"version": 1, "identity": tree.identity(), "limits": tree.limits(), "layout": self.configuration, "effective_ratio":effective_ratio, "history_rows": self.history.row_count(), "history_members": self.history.membership_count()}),
                )
            }
            HierarchyQuery::Tile {
                parent,
                tiler,
                bounds,
            } => json(&tree.tile_children(parent, bounds, tiler, None, &mut ())?),
        }
    }
    /// Decode one bounded query and return owned UTF-8 JSON.
    pub fn query_json(&self, input: &str) -> ChartResult<String> {
        crate::portable::encode(&self.query(crate::portable::decode(input)?)?)
    }
}
fn json<T: Serialize>(value: &T) -> ChartResult<Value> {
    serde_json::to_value(value).map_err(|e| invalid(e.to_string()))
}
/// Independent packing helper requests, requiring no hierarchy session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackingRequest {
    /// Current helper protocol version: 1.
    pub version: u32,
    /// If true place siblings; otherwise find their enclosure without moving them.
    pub siblings: bool,
    /// Finite input circles.
    pub circles: Vec<Circle>,
    /// Explicit work limits.
    #[serde(default)]
    pub limits: HierarchyLimits,
}
impl PackingRequest {
    /// Execute the checked canonical helper.
    pub fn execute(&self) -> ChartResult<Value> {
        if self.version != 1 {
            return Err(invalid("Unsupported packing helper version."));
        }
        if self.siblings {
            json(&pack_siblings(&self.circles, self.limits)?)
        } else {
            json(&pack_enclose(&self.circles, self.limits)?)
        }
    }
    /// Execute bounded portable helper input and encode an owned result.
    pub fn execute_json(input: &str) -> ChartResult<String> {
        crate::portable::encode(&crate::portable::decode::<Self>(input)?.execute()?)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionSnapshot {
    version: u32,
    identity: HierarchyId,
    limits: HierarchyLimits,
    nodes: Vec<NodeRecord>,
    history: TreemapHistory,
    configuration: Option<LayoutSpec>,
}
impl HierarchySession {
    /// Serialize exact ordered topology, values, geometry, configuration and layout rows.
    /// Registered native-only definitions are rejected during session construction/operations.
    pub fn to_json(&self) -> ChartResult<String> {
        let mut nodes = Vec::with_capacity(self.tree.len());
        self.tree.visit(
            self.tree.root().handle(),
            VisitOrder::PreOrder,
            |node, _, _| {
                nodes.push(NodeRecord::from_node(node, self.layout.as_ref())?);
                Ok(())
            },
        )?;
        crate::portable::encode(&SessionSnapshot {
            version: 1,
            identity: self.tree.identity(),
            limits: self.tree.limits(),
            nodes,
            history: self.history.clone(),
            configuration: self.configuration.clone(),
        })
    }
    /// Restore an exact bounded snapshot, validating topology, metadata, geometry and history.
    pub fn from_snapshot_json(input: &str, registry: &ExtensionRegistry) -> ChartResult<Self> {
        let snapshot: SessionSnapshot = crate::portable::decode(input)?;
        if snapshot.version != 1 {
            return Err(invalid("Unsupported hierarchy snapshot version."));
        }
        let rows = snapshot
            .nodes
            .iter()
            .map(|n| NodeInput {
                key: n.handle.node,
                parent: n.parent.map(|p| p.node),
                data: n.data.clone(),
                id: n.id.clone(),
                synthetic: n.synthetic,
            })
            .collect();
        let mut tree = Hierarchy::from_rows(snapshot.identity, rows, snapshot.limits)?;
        let values_present = snapshot.nodes.first().is_some_and(|n| n.value.is_some());
        let geometry_present = snapshot.configuration.is_some();
        let mut geometry = Vec::with_capacity(tree.len());
        for (i, record) in snapshot.nodes.iter().enumerate() {
            let view = tree.view(i);
            if record.handle != view.handle()
                || record.parent != view.parent().map(NodeView::handle)
                || !record
                    .children
                    .iter()
                    .copied()
                    .eq(view.children().map(NodeView::handle))
                || record.depth != view.depth()
                || record.height != view.height()
                || record.value.is_some() != values_present
                || record.geometry.is_some() != geometry_present
            {
                return Err(invalid(
                    "Hierarchy snapshot metadata is inconsistent with topology/configuration.",
                ));
            }
            if let Some(value) = record.value {
                weight(value)?;
            }
            if let Some(item) = record.geometry {
                item.validate()?;
                geometry.push(item);
            }
        }
        for (node, record) in Arc::make_mut(&mut tree.nodes)
            .iter_mut()
            .zip(&snapshot.nodes)
        {
            node.value = record.value;
        }
        snapshot.history.validate_snapshot(&tree)?;
        if let Some(spec) = &snapshot.configuration {
            // Revalidate all selected registrations/options through the normal checked path.
            // Retained geometry may precede an explicit history reset, so it is not regenerated.
            let _ = spec.layout_with(&tree, registry, &mut snapshot.history.clone(), true)?;
            let valid_kind = geometry.iter().all(|g| {
                matches!(
                    (spec, g),
                    (
                        LayoutSpec::Tree { .. } | LayoutSpec::Cluster { .. },
                        NodeGeometry::Point { .. }
                    ) | (
                        LayoutSpec::Partition(_) | LayoutSpec::Treemap { .. },
                        NodeGeometry::Rectangle { .. }
                    ) | (LayoutSpec::Pack { .. }, NodeGeometry::Circle { .. })
                )
            });
            if !valid_kind {
                return Err(invalid(
                    "Hierarchy snapshot geometry does not match its layout family.",
                ));
            }
        }
        let layout = if geometry_present {
            Some(HierarchyLayout::new(tree.clone(), geometry)?)
        } else {
            None
        };
        Ok(Self {
            tree,
            registry: registry.clone(),
            history: snapshot.history,
            layout,
            configuration: snapshot.configuration,
        })
    }
}

impl HierarchySession {
    /// Invoke a standalone tiler and atomically retain its optional row history.
    pub fn tile(&mut self, request: TilingRequest) -> ChartResult<Vec<(NodeHandle, [f64; 4])>> {
        let mut history = self.history.clone();
        let result = request.execute(&self.tree, &self.registry, &mut history)?;
        self.history = history;
        Ok(result)
    }
    /// Decode a standalone tiler request and return owned coordinates.
    pub fn tile_json(&mut self, input: &str) -> ChartResult<String> {
        let mut next = self.clone();
        let result = crate::portable::encode(&next.tile(crate::portable::decode(input)?)?)?;
        *self = next;
        Ok(result)
    }
}
