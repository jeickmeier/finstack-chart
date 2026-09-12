// Six reference tilers and retained resquarify rows; adapted from D3 (ISC, LICENSE).
use super::partition::{collapse, dice, finish_rectangles};
use super::*;
use std::collections::BTreeMap;
/// Default squarify aspect ratio.
pub const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;
/// Built-in tilers or an explicitly supplied checked custom operation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tiler {
    /// Recursively split weighted prefixes along the longer dimension.
    Binary,
    /// Horizontal strips proportional to each child's share.
    Dice,
    /// Vertical strips proportional to each child's share.
    Slice,
    /// Alternate strips by depth.
    SliceDice,
    /// Greedy aspect-ratio rows; finite ratios clamp to at least one.
    Squarify(f64),
    /// Reuse compatible row history; without a history owner, perform a fresh layout.
    Resquarify(f64),
    /// Native or registered operation supplied at invocation.
    Custom,
}
impl Default for Tiler {
    fn default() -> Self {
        Self::Squarify(GOLDEN_RATIO)
    }
}
impl Tiler {
    /// Validated effective ratio for either squarify family.
    pub fn ratio(self) -> ChartResult<Option<f64>> {
        match self {
            Self::Squarify(r) | Self::Resquarify(r) => {
                if r.is_finite() {
                    Ok(Some(r.max(1.)))
                } else {
                    Err(numerical("Treemap ratio must be finite."))
                }
            }
            _ => Ok(None),
        }
    }
}
/// Independent padding accessor slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PaddingSide {
    /// Gap between siblings.
    Inner,
    /// Parent left edge.
    Left,
    /// Parent top edge.
    Top,
    /// Parent right edge.
    Right,
    /// Parent bottom edge.
    Bottom,
}
/// Treemap configuration with complete independent padding controls.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TreemapOptions {
    /// Destination extent.
    pub size: [f64; 2],
    /// Round final rectangles.
    pub round: bool,
    /// Built-in or explicitly supplied custom tiler.
    pub tile: Tiler,
    /// Gap between siblings.
    pub padding_inner: f64,
    /// Top edge inset.
    pub padding_top: f64,
    /// Right edge inset.
    pub padding_right: f64,
    /// Bottom edge inset.
    pub padding_bottom: f64,
    /// Left edge inset.
    pub padding_left: f64,
}
impl Default for TreemapOptions {
    fn default() -> Self {
        Self {
            size: [1., 1.],
            round: false,
            tile: Tiler::default(),
            padding_inner: 0.,
            padding_top: 0.,
            padding_right: 0.,
            padding_bottom: 0.,
            padding_left: 0.,
        }
    }
}
impl TreemapOptions {
    /// Set all inner and outer padding controls together.
    pub fn with_padding(mut self, value: f64) -> Self {
        self.padding_inner = value;
        self = self.with_padding_outer(value);
        self
    }
    /// Set the four outer controls together.
    pub fn with_padding_outer(mut self, value: f64) -> Self {
        self.padding_top = value;
        self.padding_right = value;
        self.padding_bottom = value;
        self.padding_left = value;
        self
    }
    /// Reference alias readback: combined padding reads the inner slot.
    pub fn padding(&self) -> f64 {
        self.padding_inner
    }
    /// Reference alias readback: outer padding reads the top slot.
    pub fn padding_outer(&self) -> f64 {
        self.padding_top
    }
    fn padding_at(self, side: PaddingSide) -> f64 {
        match side {
            PaddingSide::Inner => self.padding_inner,
            PaddingSide::Left => self.padding_left,
            PaddingSide::Top => self.padding_top,
            PaddingSide::Right => self.padding_right,
            PaddingSide::Bottom => self.padding_bottom,
        }
    }
    pub(super) fn validate(self) -> ChartResult<()> {
        super::layout::extent(self.size)?;
        self.tile.ratio()?;
        if [
            self.padding_inner,
            self.padding_top,
            self.padding_right,
            self.padding_bottom,
            self.padding_left,
        ]
        .iter()
        .all(|p| p.is_finite())
        {
            Ok(())
        } else {
            Err(numerical("Treemap padding must be finite."))
        }
    }
}
/// Native custom accessors; registered portable operations implement this same boundary.
/// Returned child rectangles are in supplied child order and are validated before commit.
pub trait TreemapAccessors {
    /// Resolve one padding slot. The default returns its configured constant.
    fn padding(
        &mut self,
        _node: NodeView<'_>,
        _side: PaddingSide,
        configured: f64,
    ) -> ChartResult<f64> {
        Ok(configured)
    }
    /// Produce rectangles for a Custom tiler, or reject missing native code explicitly.
    fn tile(&mut self, _parent: NodeView<'_>, _bounds: [f64; 4]) -> ChartResult<Vec<[f64; 4]>> {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Custom hierarchy tiler was not supplied.",
        ))
    }
}
impl TreemapAccessors for () {}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    children: Vec<HierarchyNodeId>,
    dice: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Topology {
    identity: HierarchyId,
    children: Vec<(HierarchyNodeId, Vec<HierarchyNodeId>)>,
}
impl Topology {
    fn new(tree: &Hierarchy) -> Self {
        Self {
            identity: tree.identity,
            children: tree
                .by_key
                .iter()
                .map(|(&key, &i)| {
                    (
                        key,
                        tree.nodes[i]
                            .children
                            .iter()
                            .map(|&c| tree.nodes[c].key)
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}
/// Explicit resquarify row owner. Failed layouts leave it unchanged.
/// Ordered topology/new owner/ratio changes conservatively reset the complete cache.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreemapHistory {
    topology: Option<Topology>,
    ratio: Option<f64>,
    rows: BTreeMap<HierarchyNodeId, Vec<Row>>,
}
impl TreemapHistory {
    /// Forget all retained rows; the next layout matches fresh squarify.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Number of retained row partitions (not a pixel/performance metric).
    pub fn row_count(&self) -> usize {
        self.rows.values().map(Vec::len).sum()
    }
    /// Number of retained child memberships; bounded by the topology's edge count.
    pub fn membership_count(&self) -> usize {
        self.rows.values().flatten().map(|r| r.children.len()).sum()
    }
    pub(super) fn retain_compatible(&mut self, tree: &Hierarchy) {
        if self
            .topology
            .as_ref()
            .is_some_and(|t| *t != Topology::new(tree))
        {
            self.reset();
        }
    }
    pub(super) fn validate_snapshot(&self, tree: &Hierarchy) -> ChartResult<()> {
        if self.topology.is_none() {
            if self.ratio.is_some() || !self.rows.is_empty() {
                return Err(invalid("Unowned treemap history."));
            }
            return Ok(());
        }
        if self.topology.as_ref() != Some(&Topology::new(tree))
            || !self.ratio.is_some_and(|r| r.is_finite() && r >= 1.)
        {
            return Err(invalid("Treemap history topology or ratio is invalid."));
        }
        within(
            self.membership_count() < tree.len(),
            "Treemap history membership budget exceeded.",
        )?;
        for (key, rows) in &self.rows {
            let parent = tree.get(NodeHandle {
                hierarchy: tree.identity(),
                node: *key,
            })?;
            if rows.is_empty()
                || rows.iter().any(|r| r.children.is_empty())
                || !rows
                    .iter()
                    .flat_map(|r| r.children.iter().copied())
                    .eq(parent.children().map(|n| n.handle().node))
            {
                return Err(invalid(
                    "Treemap history must partition each parent's ordered children exactly once.",
                ));
            }
        }
        Ok(())
    }
    fn prepare(&mut self, tree: &Hierarchy, tile: Tiler) -> ChartResult<()> {
        if !matches!(tile, Tiler::Resquarify(_)) {
            self.reset();
            return Ok(());
        }
        let ratio = tile.ratio()?;
        let topology = Topology::new(tree);
        if !matches!(tile, Tiler::Resquarify(_))
            || self.topology.as_ref() != Some(&topology)
            || self.ratio != ratio
        {
            self.reset();
        }
        self.topology = Some(topology);
        self.ratio = ratio;
        Ok(())
    }
}
impl Hierarchy {
    /// Fresh standalone treemap. Use an explicit history owner for resquarify reuse.
    pub fn treemap(&self, options: TreemapOptions) -> ChartResult<HierarchyLayout> {
        self.treemap_with(options, None, &mut ())
    }
    /// Treemap with atomic retained resquarify rows.
    pub fn treemap_with_history(
        &self,
        options: TreemapOptions,
        history: &mut TreemapHistory,
    ) -> ChartResult<HierarchyLayout> {
        self.treemap_with(options, Some(history), &mut ())
    }
    /// Shared full layout boundary for native or registered padding/tiler accessors.
    pub fn treemap_with(
        &self,
        options: TreemapOptions,
        history: Option<&mut TreemapHistory>,
        accessors: &mut impl TreemapAccessors,
    ) -> ChartResult<HierarchyLayout> {
        options.validate()?;
        self.require_values()?;
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        let mut next = history.as_deref().cloned().unwrap_or_default();
        next.prepare(self, options.tile)?;
        let mut rectangles = vec![[0.; 4]; self.len()];
        rectangles[self.root] = [0., 0., options.size[0], options.size[1]];
        let mut padding = vec![0.; self.nodes[self.root].height + 2];
        for i in self.order(VisitOrder::PreOrder)? {
            work.charge(1)?;
            let depth = self.nodes[i].depth;
            let p = padding[depth];
            let [mut x0, mut y0, mut x1, mut y1] = rectangles[i];
            x0 += p;
            y0 += p;
            x1 -= p;
            y1 -= p;
            collapse(&mut x0, &mut x1);
            collapse(&mut y0, &mut y1);
            rectangles[i] = [x0, y0, x1, y1];
            if !self.nodes[i].children.is_empty() {
                let mut get = |side| {
                    work.charge(1)?;
                    let v = accessors.padding(self.view(i), side, options.padding_at(side))?;
                    if v.is_finite() {
                        Ok(v)
                    } else {
                        Err(numerical("Padding accessor returned a non-finite value."))
                    }
                };
                let p = get(PaddingSide::Inner)? / 2.;
                padding[depth + 1] = p;
                x0 += get(PaddingSide::Left)? - p;
                y0 += get(PaddingSide::Top)? - p;
                x1 -= get(PaddingSide::Right)? - p;
                y1 -= get(PaddingSide::Bottom)? - p;
                collapse(&mut x0, &mut x1);
                collapse(&mut y0, &mut y1);
                tile(
                    self,
                    i,
                    [x0, y0, x1, y1],
                    options.tile,
                    &mut rectangles,
                    &mut next,
                    &mut work,
                    accessors,
                )?;
            }
        }
        let output = finish_rectangles(self, rectangles, options.round)?;
        within(
            next.membership_count() <= self.len().saturating_sub(1),
            "Treemap history membership budget exceeded.",
        )?;
        if let Some(history) = history {
            *history = next;
        }
        Ok(output)
    }
    /// Invoke a single tiler on one parent and explicit bounds, without a chart/layout pass.
    /// Children retain authored order. Leaves reject because tilers require a parent.
    pub fn tile_children(
        &self,
        parent: NodeHandle,
        bounds: [f64; 4],
        kind: Tiler,
        history: Option<&mut TreemapHistory>,
        accessors: &mut impl TreemapAccessors,
    ) -> ChartResult<Vec<(NodeHandle, [f64; 4])>> {
        self.require_values()?;
        validate_rect(bounds)?;
        kind.ratio()?;
        let i = self.index(parent)?;
        if self.nodes[i].children.is_empty() {
            return Err(invalid("Standalone tilers require a parent with children."));
        }
        let mut next = history.as_deref().cloned().unwrap_or_default();
        next.prepare(self, kind)?;
        let mut rectangles = vec![[0.; 4]; self.len()];
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        tile(
            self,
            i,
            bounds,
            kind,
            &mut rectangles,
            &mut next,
            &mut work,
            accessors,
        )?;
        let output = self.nodes[i]
            .children
            .iter()
            .map(|&c| {
                validate_rect(rectangles[c])?;
                Ok((self.handle(c), rectangles[c]))
            })
            .collect::<ChartResult<Vec<_>>>()?;
        if let Some(history) = history {
            *history = next;
        }
        Ok(output)
    }
}
fn validate_rect(r: [f64; 4]) -> ChartResult<()> {
    NodeGeometry::Rectangle {
        x0: r[0],
        y0: r[1],
        x1: r[2],
        y1: r[3],
    }
    .validate()
}
fn slice(
    tree: &Hierarchy,
    children: &[usize],
    value: f64,
    bounds: [f64; 4],
    rectangles: &mut [[f64; 4]],
    work: &mut Work,
) -> ChartResult<()> {
    let [x0, mut y0, x1, y1] = bounds;
    let k = if value != 0. { (y1 - y0) / value } else { 0. };
    for &c in children {
        work.charge(1)?;
        let top = y0;
        y0 += tree.nodes[c].value.expect("checked value") * k;
        rectangles[c] = [x0, top, x1, y0];
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)] // One shared dispatch boundary retains atomic output/history and callback budgets.
fn tile(
    tree: &Hierarchy,
    parent: usize,
    bounds: [f64; 4],
    kind: Tiler,
    rectangles: &mut [[f64; 4]],
    history: &mut TreemapHistory,
    work: &mut Work,
    accessors: &mut impl TreemapAccessors,
) -> ChartResult<()> {
    let children = &tree.nodes[parent].children;
    let value = tree.nodes[parent].value.expect("checked value");
    match kind {
        Tiler::Dice => dice(tree, children, value, bounds, rectangles, work)?,
        Tiler::Slice => slice(tree, children, value, bounds, rectangles, work)?,
        Tiler::SliceDice => {
            if tree.nodes[parent].depth.is_multiple_of(2) {
                dice(tree, children, value, bounds, rectangles, work)?;
            } else {
                slice(tree, children, value, bounds, rectangles, work)?;
            }
        }
        Tiler::Binary => binary(tree, children, value, bounds, rectangles, work)?,
        Tiler::Squarify(r) | Tiler::Resquarify(r) => {
            let key = tree.nodes[parent].key;
            if matches!(kind, Tiler::Resquarify(_)) && history.rows.contains_key(&key) {
                reuse_rows(tree, &history.rows[&key], value, bounds, rectangles, work)?;
            } else {
                let rows = squarify(tree, children, value, bounds, r.max(1.), rectangles, work)?;
                if matches!(kind, Tiler::Resquarify(_)) {
                    history.rows.insert(key, rows);
                }
            }
        }
        Tiler::Custom => {
            work.charge(children.len())?;
            let output = accessors.tile(tree.view(parent), bounds)?;
            if output.len() != children.len() {
                return Err(invalid(
                    "Custom tiler must return one rectangle per child in child order.",
                ));
            }
            for (&c, rect) in children.iter().zip(output) {
                validate_rect(rect)?;
                rectangles[c] = rect;
            }
        }
    }
    Ok(())
}
fn binary(
    tree: &Hierarchy,
    children: &[usize],
    value: f64,
    bounds: [f64; 4],
    rectangles: &mut [[f64; 4]],
    work: &mut Work,
) -> ChartResult<()> {
    let mut sums = Vec::with_capacity(children.len() + 1);
    sums.push(0.);
    for &c in children {
        work.charge(1)?;
        sums.push(sums.last().expect("prefix") + tree.nodes[c].value.expect("checked value"));
    }
    let mut stack = vec![(0, children.len(), value, bounds)];
    while let Some((i, j, value, [x0, y0, x1, y1])) = stack.pop() {
        work.charge(1)?;
        if i >= j - 1 {
            rectangles[children[i]] = [x0, y0, x1, y1];
            continue;
        }
        let offset = sums[i];
        let target = value / 2. + offset;
        let (mut k, mut hi) = (i + 1, j - 1);
        while k < hi {
            work.charge(1)?;
            let mid = (k + hi) / 2;
            if sums[mid] < target {
                k = mid + 1;
            } else {
                hi = mid;
            }
        }
        if target - sums[k - 1] < sums[k] - target && i + 1 < k {
            k -= 1;
        }
        let left = sums[k] - offset;
        let right = value - left;
        if x1 - x0 > y1 - y0 {
            let split = if x0 == x1 {
                x0
            } else if value != 0. {
                (x0 * right + x1 * left) / value
            } else {
                x1
            };
            stack.push((k, j, right, [split, y0, x1, y1]));
            stack.push((i, k, left, [x0, y0, split, y1]));
        } else {
            let split = if y0 == y1 {
                y0
            } else if value != 0. {
                (y0 * right + y1 * left) / value
            } else {
                y1
            };
            stack.push((k, j, right, [x0, split, x1, y1]));
            stack.push((i, k, left, [x0, y0, x1, split]));
        }
    }
    Ok(())
}
fn maximum(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}
fn squarify(
    tree: &Hierarchy,
    children: &[usize],
    mut value: f64,
    bounds: [f64; 4],
    ratio: f64,
    rectangles: &mut [[f64; 4]],
    work: &mut Work,
) -> ChartResult<Vec<Row>> {
    let [mut x0, mut y0, x1, y1] = bounds;
    let (mut i0, mut i1) = (0, 0);
    let mut rows = Vec::new();
    while i0 < children.len() {
        work.charge(1)?;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let mut sum;
        loop {
            work.charge(1)?;
            sum = tree.nodes[children[i1]].value.expect("checked value");
            i1 += 1;
            if sum != 0. || i1 == children.len() {
                break;
            }
        }
        let (mut min, mut max) = (sum, sum);
        let alpha = maximum(dy / dx, dx / dy) / (value * ratio);
        let beta = sum * sum * alpha;
        let mut min_ratio = maximum(max / beta, beta / min);
        while i1 < children.len() {
            work.charge(1)?;
            let next = tree.nodes[children[i1]].value.expect("checked value");
            sum += next;
            if next < min {
                min = next;
            }
            if next > max {
                max = next;
            }
            let beta = sum * sum * alpha;
            let new_ratio = maximum(max / beta, beta / min);
            if new_ratio > min_ratio {
                sum -= next;
                break;
            }
            min_ratio = new_ratio;
            i1 += 1;
        }
        let is_dice = dx < dy;
        let row_children = &children[i0..i1];
        if is_dice {
            let next = if value != 0. {
                y0 + dy * sum / value
            } else {
                y1
            };
            dice(
                tree,
                row_children,
                sum,
                [x0, y0, x1, next],
                rectangles,
                work,
            )?;
            if value != 0. {
                y0 = next;
            }
        } else {
            let next = if value != 0. {
                x0 + dx * sum / value
            } else {
                x1
            };
            slice(
                tree,
                row_children,
                sum,
                [x0, y0, next, y1],
                rectangles,
                work,
            )?;
            if value != 0. {
                x0 = next;
            }
        }
        rows.push(Row {
            children: row_children.iter().map(|&c| tree.nodes[c].key).collect(),
            dice: is_dice,
        });
        value -= sum;
        i0 = i1;
    }
    Ok(rows)
}
fn reuse_rows(
    tree: &Hierarchy,
    rows: &[Row],
    mut value: f64,
    bounds: [f64; 4],
    rectangles: &mut [[f64; 4]],
    work: &mut Work,
) -> ChartResult<()> {
    let [mut x0, mut y0, x1, y1] = bounds;
    for row in rows {
        let mut sum = 0.;
        let mut children = Vec::with_capacity(row.children.len());
        for key in &row.children {
            work.charge(1)?;
            let &i = tree
                .by_key
                .get(key)
                .ok_or_else(|| invalid("Stale resquarify row member."))?;
            sum += tree.nodes[i].value.expect("checked value");
            children.push(i);
        }
        if row.dice {
            let next = if value != 0. {
                y0 + (y1 - y0) * sum / value
            } else {
                y1
            };
            dice(tree, &children, sum, [x0, y0, x1, next], rectangles, work)?;
            if value != 0. {
                y0 = next;
            }
        } else {
            let next = if value != 0. {
                x0 + (x1 - x0) * sum / value
            } else {
                x1
            };
            slice(tree, &children, sum, [x0, y0, next, y1], rectangles, work)?;
            if value != 0. {
                x0 = next;
            }
        }
        value -= sum;
    }
    Ok(())
}
