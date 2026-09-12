// Tidy/Buchheim and cluster algorithms adapted from d3-hierarchy 3.1.2 (ISC, LICENSE).
use super::*;
#[derive(Clone, Debug)]
struct Scratch {
    parent: usize,
    children: Vec<usize>,
    ancestor: usize,
    default_ancestor: Option<usize>,
    z: f64,
    m: f64,
    change: f64,
    shift: f64,
    thread: Option<usize>,
    number: usize,
}
impl Scratch {
    fn new(index: usize, parent: usize, children: Vec<usize>, number: usize) -> Self {
        Self {
            parent,
            children,
            ancestor: index,
            default_ancestor: None,
            z: 0.,
            m: 0.,
            change: 0.,
            shift: 0.,
            thread: None,
            number,
        }
    }
}
fn left(nodes: &[Scratch], i: usize) -> Option<usize> {
    nodes[i].children.first().copied().or(nodes[i].thread)
}
fn right(nodes: &[Scratch], i: usize) -> Option<usize> {
    nodes[i].children.last().copied().or(nodes[i].thread)
}
fn sep(
    tree: &Hierarchy,
    a: usize,
    b: usize,
    callback: &mut impl FnMut(NodeView<'_>, NodeView<'_>) -> ChartResult<f64>,
    work: &mut Work,
) -> ChartResult<f64> {
    work.charge(1)?;
    let value = callback(tree.view(a), tree.view(b))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numerical("Custom separation returned a non-finite value."))
    }
}
impl Hierarchy {
    /// Tidy tree using the shared checked built-in separation policy.
    pub fn tree(&self, options: TreeOptions) -> ChartResult<HierarchyLayout> {
        self.tree_with(options, |a, b| Ok(options.separation.evaluate(a, b)))
    }
    /// Tidy tree with a typed native separation accessor.
    pub fn tree_with(
        &self,
        options: TreeOptions,
        mut separation: impl FnMut(NodeView<'_>, NodeView<'_>) -> ChartResult<f64>,
    ) -> ChartResult<HierarchyLayout> {
        let size = options.validate()?;
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        let dummy = self.len();
        let mut nodes: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| Scratch::new(i, n.parent.unwrap_or(dummy), n.children.clone(), 0))
            .collect();
        nodes.push(Scratch::new(dummy, dummy, vec![self.root], 0));
        for i in 0..self.len() {
            for j in 0..nodes[i].children.len() {
                let child = nodes[i].children[j];
                nodes[child].number = j;
            }
        }
        for v in self.order(VisitOrder::PostOrder)? {
            work.charge(1)?;
            let parent = nodes[v].parent;
            let prior = nodes[v]
                .number
                .checked_sub(1)
                .map(|i| nodes[parent].children[i]);
            if !nodes[v].children.is_empty() {
                let (mut shift, mut change) = (0., 0.);
                for i in (0..nodes[v].children.len()).rev() {
                    work.charge(1)?;
                    let w = nodes[v].children[i];
                    nodes[w].z += shift;
                    nodes[w].m += shift;
                    change += nodes[w].change;
                    shift += nodes[w].shift + change;
                }
                let midpoint = (nodes[nodes[v].children[0]].z
                    + nodes[*nodes[v].children.last().expect("nonempty")].z)
                    / 2.;
                if let Some(w) = prior {
                    nodes[v].z = nodes[w].z + sep(self, v, w, &mut separation, &mut work)?;
                    nodes[v].m = nodes[v].z - midpoint;
                } else {
                    nodes[v].z = midpoint;
                }
            } else if let Some(w) = prior {
                nodes[v].z = nodes[w].z + sep(self, v, w, &mut separation, &mut work)?;
            }
            let ancestor = nodes[parent]
                .default_ancestor
                .unwrap_or(nodes[parent].children[0]);
            nodes[parent].default_ancestor = Some(apportion(
                self,
                &mut nodes,
                v,
                prior,
                ancestor,
                &mut separation,
                &mut work,
            )?);
        }
        nodes[dummy].m = -nodes[self.root].z;
        let mut x = vec![0.; self.len()];
        for v in self.order(VisitOrder::PreOrder)? {
            work.charge(1)?;
            x[v] = nodes[v].z + nodes[nodes[v].parent].m;
            nodes[v].m += nodes[nodes[v].parent].m;
        }
        let geometry = match options.mode {
            TreeSize::NodeSize(_) => self
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| NodeGeometry::Point {
                    x: x[i] * size[0],
                    y: n.depth as f64 * size[1],
                })
                .collect(),
            TreeSize::Extent(_) => {
                let (mut l, mut r, mut bottom) = (self.root, self.root, self.root);
                for i in self.order(VisitOrder::PreOrder)? {
                    if x[i] < x[l] {
                        l = i;
                    }
                    if x[i] > x[r] {
                        r = i;
                    }
                    if self.nodes[i].depth > self.nodes[bottom].depth {
                        bottom = i;
                    }
                }
                let s = if l == r {
                    1.
                } else {
                    sep(self, l, r, &mut separation, &mut work)? / 2.
                };
                let tx = s - x[l];
                let kx = size[0] / (x[r] + s + tx);
                let ky = size[1] / self.nodes[bottom].depth.max(1) as f64;
                self.nodes
                    .iter()
                    .enumerate()
                    .map(|(i, n)| NodeGeometry::Point {
                        x: (x[i] + tx) * kx,
                        y: n.depth as f64 * ky,
                    })
                    .collect()
            }
        };
        HierarchyLayout::new(self.clone(), geometry)
    }
    /// Leaf-aligned cluster with reference-default or declarative separation.
    pub fn cluster(&self, options: TreeOptions) -> ChartResult<HierarchyLayout> {
        self.cluster_with(options, |a, b| Ok(options.separation.evaluate(a, b)))
    }
    /// Leaf-aligned cluster with typed native separation.
    pub fn cluster_with(
        &self,
        options: TreeOptions,
        mut separation: impl FnMut(NodeView<'_>, NodeView<'_>) -> ChartResult<f64>,
    ) -> ChartResult<HierarchyLayout> {
        let size = options.validate()?;
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        let mut xy = vec![[0., 0.]; self.len()];
        let mut previous = None;
        let mut cursor = 0.;
        for i in self.order(VisitOrder::PostOrder)? {
            work.charge(1)?;
            let children = &self.nodes[i].children;
            if children.is_empty() {
                if let Some(prior) = previous {
                    cursor += sep(self, i, prior, &mut separation, &mut work)?;
                }
                xy[i] = [cursor, 0.];
                previous = Some(i);
            } else {
                let x = children.iter().map(|&c| xy[c][0]).sum::<f64>() / children.len() as f64;
                let y = 1. + children.iter().map(|&c| xy[c][1]).fold(0., f64::max);
                xy[i] = [x, y];
            }
        }
        let mut l = self.root;
        while let Some(&c) = self.nodes[l].children.first() {
            work.charge(1)?;
            l = c;
        }
        let mut r = self.root;
        while let Some(&c) = self.nodes[r].children.last() {
            work.charge(1)?;
            r = c;
        }
        let x0 = xy[l][0] - sep(self, l, r, &mut separation, &mut work)? / 2.;
        let x1 = xy[r][0] + sep(self, r, l, &mut separation, &mut work)? / 2.;
        let root = xy[self.root];
        let geometry = xy
            .into_iter()
            .map(|[x, y]| match options.mode {
                TreeSize::NodeSize(_) => NodeGeometry::Point {
                    x: (x - root[0]) * size[0],
                    y: (root[1] - y) * size[1],
                },
                TreeSize::Extent(_) => NodeGeometry::Point {
                    x: (x - x0) / (x1 - x0) * size[0],
                    y: (1. - if root[1] != 0. { y / root[1] } else { 1. }) * size[1],
                },
            })
            .collect();
        HierarchyLayout::new(self.clone(), geometry)
    }
}
fn apportion(
    tree: &Hierarchy,
    nodes: &mut [Scratch],
    v: usize,
    prior: Option<usize>,
    mut ancestor: usize,
    separation: &mut impl FnMut(NodeView<'_>, NodeView<'_>) -> ChartResult<f64>,
    work: &mut Work,
) -> ChartResult<usize> {
    if let Some(w) = prior {
        let (mut vip, mut vim) = (Some(v), Some(w));
        let (mut vop, mut vom) = (v, nodes[nodes[v].parent].children[0]);
        let (mut sip, mut sop, mut sim, mut som) =
            (nodes[v].m, nodes[v].m, nodes[w].m, nodes[vom].m);
        loop {
            vim = vim.and_then(|i| right(nodes, i));
            vip = vip.and_then(|i| left(nodes, i));
            let (Some(im), Some(ip)) = (vim, vip) else {
                break;
            };
            work.charge(1)?;
            vom = left(nodes, vom).ok_or_else(|| invalid("Invalid tidy-tree outer contour."))?;
            vop = right(nodes, vop).ok_or_else(|| invalid("Invalid tidy-tree outer contour."))?;
            nodes[vop].ancestor = v;
            let shift =
                nodes[im].z + sim - nodes[ip].z - sip + sep(tree, im, ip, separation, work)?;
            if shift > 0. {
                let a = nodes[im].ancestor;
                let wm = if nodes[a].parent == nodes[v].parent {
                    a
                } else {
                    ancestor
                };
                let change = shift / (nodes[v].number - nodes[wm].number) as f64;
                nodes[v].change -= change;
                nodes[v].shift += shift;
                nodes[wm].change += change;
                nodes[v].z += shift;
                nodes[v].m += shift;
                sip += shift;
                sop += shift;
            }
            sim += nodes[im].m;
            sip += nodes[ip].m;
            som += nodes[vom].m;
            sop += nodes[vop].m;
        }
        if vim.is_some() && right(nodes, vop).is_none() {
            nodes[vop].thread = vim;
            nodes[vop].m += sim - sop;
        }
        if vip.is_some() && left(nodes, vom).is_none() {
            nodes[vom].thread = vip;
            nodes[vom].m += sip - som;
            ancestor = v;
        }
    }
    Ok(ancestor)
}
