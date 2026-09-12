// Weighted adjacency layout adapted from d3-hierarchy 3.1.2 (ISC, LICENSE).
use super::*;
/// Partition controls with unit extent, no rounding and no padding by default.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartitionOptions {
    /// Destination width and height in layout units.
    pub size: [f64; 2],
    /// Round final coordinates using the shared JavaScript-compatible rounding owner.
    pub round: bool,
    /// Finite signed spacing, collapsed at midpoints when excessive.
    pub padding: f64,
}
impl Default for PartitionOptions {
    fn default() -> Self {
        Self {
            size: [1., 1.],
            round: false,
            padding: 0.,
        }
    }
}
impl Hierarchy {
    /// Weighted adjacency rectangles, retaining internal own-value slack.
    pub fn partition(&self, options: PartitionOptions) -> ChartResult<HierarchyLayout> {
        super::layout::extent(options.size)?;
        if !options.padding.is_finite() {
            return Err(numerical("Partition padding must be finite."));
        }
        self.require_values()?;
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        let levels = (self.nodes[self.root].height + 1) as f64;
        let mut rectangles = vec![[0.; 4]; self.len()];
        rectangles[self.root] = [
            options.padding,
            options.padding,
            options.size[0],
            options.size[1] / levels,
        ];
        for i in self.order(VisitOrder::PreOrder)? {
            work.charge(1)?;
            if !self.nodes[i].children.is_empty() {
                let [x0, _, x1, _] = rectangles[i];
                let depth = self.nodes[i].depth as f64;
                dice(
                    self,
                    &self.nodes[i].children,
                    self.nodes[i].value.expect("validated aggregation"),
                    [
                        x0,
                        options.size[1] * (depth + 1.) / levels,
                        x1,
                        options.size[1] * (depth + 2.) / levels,
                    ],
                    &mut rectangles,
                    &mut work,
                )?;
            }
            let [mut x0, mut y0, mut x1, mut y1] = rectangles[i];
            x1 -= options.padding;
            y1 -= options.padding;
            collapse(&mut x0, &mut x1);
            collapse(&mut y0, &mut y1);
            rectangles[i] = [x0, y0, x1, y1];
        }
        finish_rectangles(self, rectangles, options.round)
    }
    pub(super) fn require_values(&self) -> ChartResult<()> {
        for node in self.nodes.iter() {
            weight(node.value.ok_or_else(|| {
                invalid("Aggregate with sum/count before weighted hierarchy layout.")
            })?)?;
        }
        Ok(())
    }
}
pub(super) fn collapse(start: &mut f64, end: &mut f64) {
    if *end < *start {
        *start = (*start + *end) / 2.;
        *end = *start;
    }
}
pub(super) fn finish_rectangles(
    tree: &Hierarchy,
    rectangles: Vec<[f64; 4]>,
    round: bool,
) -> ChartResult<HierarchyLayout> {
    let geometry = rectangles
        .into_iter()
        .map(|mut r| {
            if round {
                r = r.map(crate::interpolate::js_round);
            }
            NodeGeometry::Rectangle {
                x0: r[0],
                y0: r[1],
                x1: r[2],
                y1: r[3],
            }
        })
        .collect();
    HierarchyLayout::new(tree.clone(), geometry)
}
pub(super) fn dice(
    tree: &Hierarchy,
    children: &[usize],
    value: f64,
    bounds: [f64; 4],
    rectangles: &mut [[f64; 4]],
    work: &mut Work,
) -> ChartResult<()> {
    let [mut x0, y0, x1, y1] = bounds;
    let k = if value != 0. { (x1 - x0) / value } else { 0. };
    for &child in children {
        work.charge(1)?;
        let left = x0;
        x0 += tree.nodes[child].value.expect("validated weight") * k;
        rectangles[child] = [left, y0, x0, y1];
    }
    Ok(())
}
