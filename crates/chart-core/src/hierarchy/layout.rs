use super::*;
use std::sync::Arc;
/// One node's destination-independent hierarchy geometry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeGeometry {
    /// Cartesian position, also usable as angle/radius before radial projection.
    Point {
        /// Horizontal coordinate.
        x: f64,
        /// Vertical coordinate.
        y: f64,
    },
    /// Adjacency rectangle with finite ordered bounds.
    Rectangle {
        /// Left edge.
        x0: f64,
        /// Top edge.
        y0: f64,
        /// Right edge.
        x1: f64,
        /// Bottom edge.
        y1: f64,
    },
    /// Circle with actual layout radius, independent of mark area encodings.
    Circle {
        /// Center x.
        x: f64,
        /// Center y.
        y: f64,
        /// Nonnegative radius.
        r: f64,
    },
}
impl NodeGeometry {
    pub(crate) fn validate(self) -> ChartResult<()> {
        let valid = match self {
            Self::Point { x, y } => x.is_finite() && y.is_finite(),
            Self::Rectangle { x0, y0, x1, y1 } => {
                [x0, y0, x1, y1].iter().all(|n| n.is_finite()) && x0 <= x1 && y0 <= y1
            }
            Self::Circle { x, y, r } => x.is_finite() && y.is_finite() && r.is_finite() && r >= 0.,
        };
        if valid {
            Ok(())
        } else {
            Err(numerical("Hierarchy layout produced invalid geometry."))
        }
    }
}
/// Owned geometry and the exact immutable topology/values used to calculate it.
#[derive(Clone, Debug)]
pub struct HierarchyLayout {
    pub(crate) tree: Hierarchy,
    pub(crate) geometry: Arc<Vec<NodeGeometry>>,
}
impl HierarchyLayout {
    pub(crate) fn new(tree: Hierarchy, geometry: Vec<NodeGeometry>) -> ChartResult<Self> {
        if geometry.len() != tree.len() {
            return Err(invalid("Hierarchy geometry count does not match topology."));
        }
        for item in &geometry {
            item.validate()?;
        }
        Ok(Self {
            tree,
            geometry: Arc::new(geometry),
        })
    }
    /// Retained topology/aggregation snapshot.
    pub fn hierarchy(&self) -> &Hierarchy {
        &self.tree
    }
    /// Exact geometry for a node in this snapshot.
    pub fn geometry(&self, node: NodeHandle) -> ChartResult<NodeGeometry> {
        Ok(self.geometry[self.tree.index(node)?])
    }
    /// Breadth-first node/geometry pairs.
    pub fn nodes(
        &self,
    ) -> ChartResult<impl ExactSizeIterator<Item = (NodeView<'_>, NodeGeometry)>> {
        Ok(self
            .tree
            .order(VisitOrder::BreadthFirst)?
            .into_iter()
            .map(|i| (self.tree.view(i), self.geometry[i])))
    }
}
pub(crate) fn extent(size: [f64; 2]) -> ChartResult<()> {
    if size.iter().all(|v| v.is_finite() && *v >= 0.) {
        Ok(())
    } else {
        Err(numerical("Hierarchy size must be finite and nonnegative."))
    }
}
/// Mutually exclusive extent-fitting and fixed node-spacing modes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TreeSize {
    /// Fit output to this width/height.
    Extent([f64; 2]),
    /// Preserve spacing with the root at the origin.
    NodeSize([f64; 2]),
}
/// Portable built-in separation policies; native callbacks use the same kernel.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Separation {
    /// One unit between siblings, two between unrelated nodes.
    Default,
    /// Explicit finite distance for every pair.
    Constant(f64),
    /// Default separation divided by at least one unit of first-node depth.
    Depth,
}
impl Separation {
    pub(crate) fn evaluate(self, a: NodeView<'_>, b: NodeView<'_>) -> f64 {
        let distance = if a.parent().map(NodeView::handle) == b.parent().map(NodeView::handle) {
            1.
        } else {
            2.
        };
        match self {
            Self::Default => distance,
            Self::Constant(v) => v,
            Self::Depth => distance / a.depth().max(1) as f64,
        }
    }
}
/// Shared tree/cluster controls, with unit extent and reference separation defaults.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TreeOptions {
    /// Active extent/spacing mode.
    pub mode: TreeSize,
    /// Built-in separation; native custom callbacks are supplied explicitly at invocation.
    pub separation: Separation,
}
impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            mode: TreeSize::Extent([1., 1.]),
            separation: Separation::Default,
        }
    }
}
impl TreeOptions {
    /// Current extent, absent while fixed node spacing is active.
    pub fn size(&self) -> Option<[f64; 2]> {
        match self.mode {
            TreeSize::Extent(v) => Some(v),
            _ => None,
        }
    }
    /// Current fixed node spacing, absent while extent fitting is active.
    pub fn node_size(&self) -> Option<[f64; 2]> {
        match self.mode {
            TreeSize::NodeSize(v) => Some(v),
            _ => None,
        }
    }
    /// Set extent and disable fixed node spacing.
    pub fn with_size(mut self, size: [f64; 2]) -> ChartResult<Self> {
        extent(size)?;
        self.mode = TreeSize::Extent(size);
        Ok(self)
    }
    /// Set fixed node spacing and disable extent fitting.
    pub fn with_node_size(mut self, size: [f64; 2]) -> ChartResult<Self> {
        extent(size)?;
        self.mode = TreeSize::NodeSize(size);
        Ok(self)
    }
    pub(crate) fn validate(self) -> ChartResult<[f64; 2]> {
        let size = match self.mode {
            TreeSize::Extent(v) | TreeSize::NodeSize(v) => v,
        };
        extent(size)?;
        if let Separation::Constant(v) = self.separation
            && !v.is_finite()
        {
            return Err(numerical("Separation must be finite."));
        }
        Ok(size)
    }
}
