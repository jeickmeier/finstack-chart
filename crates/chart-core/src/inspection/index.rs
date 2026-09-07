use super::*;
use std::collections::{BTreeMap, btree_map::Entry};
type IdentityKey = (
    Option<crate::grammar::PanelKey>,
    LayerId,
    crate::state::TargetIdentity,
);

/// Query policy independent of event origin and committed state.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionMode {
    /// Frontmost point/contained shape, falling back to nearest-x line vertices.
    Auto,
    /// Nearest-x line vertices, grouped across visible series at one exact x.
    NearestX,
    /// Nearest point center within the configured radius; excludes bar/cell shapes.
    NearestPoint,
    /// Containment of visible rectangles, strokes and custom hit geometry.
    Containment,
}
/// Inspection results with actual query work, useful for deterministic cost evidence.
#[derive(Clone, Debug, Default)]
pub struct InspectionQuery {
    /// Original semantic targets, capped by the configured group limit.
    pub hits: Vec<InspectedTarget>,
    /// Candidate geometry comparisons (including nearest-x groups).
    pub examined: usize,
    /// Spatial tree nodes visited. Zero in reference scan mode.
    pub nodes: usize,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Bounds {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}
impl From<Rect> for Bounds {
    fn from(r: Rect) -> Self {
        Self {
            x0: r.origin().x(),
            x1: r.max_x(),
            y0: r.origin().y(),
            y1: r.max_y(),
        }
    }
}
impl Bounds {
    pub fn point(p: Point) -> Self {
        Self {
            x0: p.x(),
            x1: p.x(),
            y0: p.y(),
            y1: p.y(),
        }
    }
    pub fn segment(a: Point, b: Point, w: f64) -> Self {
        Self {
            x0: a.x().min(b.x()) - w,
            x1: a.x().max(b.x()) + w,
            y0: a.y().min(b.y()) - w,
            y1: a.y().max(b.y()) + w,
        }
    }
    pub fn clipped(self, r: Rect) -> Self {
        Self {
            x0: self.x0.max(r.origin().x()),
            x1: self.x1.min(r.max_x()),
            y0: self.y0.max(r.origin().y()),
            y1: self.y1.min(r.max_y()),
        }
    }
    fn union(self, b: Self) -> Self {
        Self {
            x0: self.x0.min(b.x0),
            x1: self.x1.max(b.x1),
            y0: self.y0.min(b.y0),
            y1: self.y1.max(b.y1),
        }
    }
    pub fn intersects(self, b: Self) -> bool {
        self.x0 <= b.x1 && self.x1 >= b.x0 && self.y0 <= b.y1 && self.y1 >= b.y0
    }
}
#[derive(Debug)]
struct Node {
    bounds: Bounds,
    children: Option<(usize, usize)>,
    range: std::ops::Range<usize>,
}
#[derive(Debug)]
struct XGroup {
    x: f64,
    indices: std::ops::Range<usize>,
}
#[derive(Debug)]
struct Lines {
    clip: Rect,
    groups: Vec<XGroup>,
    indices: Vec<usize>,
}
#[derive(Debug)]
pub(super) struct IdentityEntry {
    pub keyboard_index: usize,
    pub bounds: Bounds,
}
#[derive(Debug)]
pub(super) struct Index {
    pub candidates: Vec<Candidate>,
    pub keyboard: Vec<usize>,
    pub identities: BTreeMap<IdentityKey, IdentityEntry>,
    nodes: Vec<Node>,
    order: Vec<usize>,
    lines: Vec<Lines>,
}
impl Index {
    pub fn new(candidates: Vec<Candidate>) -> Self {
        // Panel/layer groups retain first appearance order, matching layout/definition order.
        let mut groups: Vec<Vec<usize>> = vec![];
        for (i, c) in candidates.iter().enumerate() {
            if let Some(g) = groups.iter_mut().find(|g| {
                let first = &candidates[g[0]];
                first.hit.panel == c.hit.panel && first.hit.layer == c.hit.layer
            }) {
                g.push(i)
            } else {
                groups.push(vec![i])
            }
        }
        let mut identities = BTreeMap::new();
        let mut keyboard = vec![];
        for mut group in groups {
            group.sort_by_key(|i| {
                candidates[*i]
                    .custom
                    .as_ref()
                    .map_or(*i as u64, |c| c.keyboard_order)
            });
            for i in group {
                let c = &candidates[i];
                if let Entry::Vacant(entry) = identities.entry((
                    c.hit.panel.clone(),
                    c.hit.layer,
                    crate::state::TargetIdentity::from(&c.hit.target),
                )) {
                    entry.insert(IdentityEntry {
                        keyboard_index: i,
                        bounds: c.bounds,
                    });
                    keyboard.push(i)
                }
            }
        }
        for c in &candidates {
            let key = (
                c.hit.panel.clone(),
                c.hit.layer,
                crate::state::TargetIdentity::from(&c.hit.target),
            );
            if let Some(entry) = identities.get_mut(&key) {
                entry.bounds = entry.bounds.union(c.bounds);
            }
        }
        let mut lines: Vec<Lines> = vec![];
        for (i, c) in candidates.iter().enumerate().filter(|(_, c)| c.line) {
            let n = lines
                .iter()
                .position(|l| l.clip == c.clip)
                .unwrap_or_else(|| {
                    lines.push(Lines {
                        clip: c.clip,
                        groups: vec![],
                        indices: vec![],
                    });
                    lines.len() - 1
                });
            lines[n].indices.push(i);
        }
        // One contiguous candidate array per clip avoids a heap allocation per x.
        // Equal x retains front-to-back order, including equivalent signed zeros.
        for l in &mut lines {
            l.indices.sort_unstable_by(|a, b| {
                let x = candidates[*a].hit.position.x();
                let y = candidates[*b].hit.position.x();
                if x == y { b.cmp(a) } else { x.total_cmp(&y) }
            });
            let mut start = 0;
            while start < l.indices.len() {
                let x = candidates[l.indices[start]].hit.position.x();
                let mut end = start + 1;
                while end < l.indices.len() && candidates[l.indices[end]].hit.position.x() == x {
                    end += 1;
                }
                l.groups.push(XGroup {
                    x,
                    indices: start..end.min(start + 128),
                });
                start = end;
            }
        }
        let order = (0..candidates.len()).collect();
        let mut index = Self {
            candidates,
            keyboard,
            identities,
            nodes: vec![],
            order,
            lines,
        };
        if !index.order.is_empty() {
            index.build(0, index.order.len());
        }
        index
    }
    fn build(&mut self, start: usize, end: usize) -> usize {
        let bounds = self.order[start + 1..end]
            .iter()
            .fold(self.candidates[self.order[start]].bounds, |b, i| {
                b.union(self.candidates[*i].bounds)
            });
        let id = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            children: None,
            range: start..end,
        });
        if end - start > 12 {
            let middle = (start + end) / 2;
            let x = bounds.x1 - bounds.x0 >= bounds.y1 - bounds.y0;
            self.order[start..end].select_nth_unstable_by(middle - start, |a, b| {
                let a = self.candidates[*a].bounds;
                let b = self.candidates[*b].bounds;
                let center = |r: Bounds| {
                    if x {
                        r.x0.midpoint(r.x1)
                    } else {
                        r.y0.midpoint(r.y1)
                    }
                };
                center(a).total_cmp(&center(b))
            });
            let left = self.build(start, middle);
            let right = self.build(middle, end);
            self.nodes[id].children = Some((left, right));
        }
        id
    }
    pub fn visit(&self, bounds: Bounds, mut f: impl FnMut(usize)) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }
        let mut stack = vec![0];
        let mut visited = 0;
        while let Some(i) = stack.pop() {
            visited += 1;
            let n = &self.nodes[i];
            if !n.bounds.intersects(bounds) {
                continue;
            }
            if let Some((a, b)) = n.children {
                stack.push(a);
                stack.push(b);
            } else {
                for i in &self.order[n.range.clone()] {
                    if self.candidates[*i].bounds.intersects(bounds) {
                        f(*i)
                    }
                }
            }
        }
        visited
    }
    pub fn query(
        &self,
        p: Point,
        mode: InspectionMode,
        radius: f64,
        limit: usize,
        scan: bool,
        bounds: Rect,
    ) -> InspectionQuery {
        let mut result = InspectionQuery::default();
        if !contains(bounds, p) {
            return result;
        }
        let mut best: Option<(usize, f64)> = None;
        if mode != InspectionMode::NearestX {
            let mut consider = |i: usize| {
                let c = &self.candidates[i];
                result.examined += 1;
                let shape = c.custom.is_some() || c.rectangle.is_some() || c.segment.is_some();
                if c.line
                    || !contains(c.clip, p)
                    || (mode == InspectionMode::NearestPoint && shape)
                    || (mode == InspectionMode::Containment && !shape)
                {
                    return;
                }
                let distance = if let Some(custom) = &c.custom {
                    if !custom.hit.contains(p) {
                        return;
                    }
                    0.
                } else if let Some(r) = c.rectangle {
                    if !contains(r, p) {
                        return;
                    }
                    0.
                } else if let Some((a, b, w)) = c.segment {
                    if segment_distance(p, a, b) > w {
                        return;
                    }
                    0.
                } else {
                    (c.hit.position.x() - p.x()).hypot(c.hit.position.y() - p.y())
                };
                if distance <= radius
                    && best.is_none_or(|(j, d)| {
                        c.layer_order > self.candidates[j].layer_order
                            || (c.layer_order == self.candidates[j].layer_order
                                && (distance < d || (distance == d && i > j)))
                    })
                {
                    best = Some((i, distance));
                }
            };
            if scan {
                for i in 0..self.candidates.len() {
                    consider(i);
                }
            } else {
                result.nodes = self.visit(
                    Bounds {
                        x0: p.x() - radius,
                        x1: p.x() + radius,
                        y0: p.y() - radius,
                        y1: p.y() + radius,
                    },
                    consider,
                );
            }
        }
        if let Some((i, _)) = best {
            result.hits.push(self.candidates[i].hit.clone());
            return result;
        }
        if !matches!(mode, InspectionMode::Auto | InspectionMode::NearestX) {
            return result;
        }
        let mut nearest = f64::INFINITY;
        let mut selected_x = None;
        let mut selected: Vec<usize> = vec![];
        let mut consider = |i: usize| {
            let c = &self.candidates[i];
            result.examined += 1;
            if !c.line || !contains(c.clip, p) {
                return;
            }
            let x = c.hit.position.x();
            let d = (x - p.x()).abs();
            if d < nearest
                || (d == nearest
                    && selected_x != Some(x)
                    && selected.first().is_none_or(|j| i > *j))
            {
                nearest = d;
                selected_x = Some(x);
                selected.clear();
            }
            if d == nearest && selected_x == Some(x) {
                selected.push(i);
                selected.sort_unstable_by(|a, b| b.cmp(a));
                selected.truncate(limit);
            }
        };
        if scan {
            for i in (0..self.candidates.len()).rev() {
                consider(i);
            }
        } else {
            for line in &self.lines {
                if !contains(line.clip, p) {
                    continue;
                }
                let n = line.groups.partition_point(|g| g.x < p.x());
                for g in [n.checked_sub(1), (n < line.groups.len()).then_some(n)]
                    .into_iter()
                    .flatten()
                {
                    for i in &line.indices[line.groups[g].indices.clone()] {
                        consider(*i);
                    }
                }
            }
        }
        result.hits = selected
            .into_iter()
            .map(|i| self.candidates[i].hit.clone())
            .collect();
        result
    }
}
pub(super) fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let length = dx.hypot(dy);
    if length == 0. {
        return (p.x() - a.x()).hypot(p.y() - a.y());
    }
    let u = (((p.x() - a.x()) * (dx / length) + (p.y() - a.y()) * (dy / length)) / length)
        .clamp(0., 1.);
    (p.x() - (a.x() + u * dx)).hypot(p.y() - (a.y() + u * dy))
}
pub(super) fn clip_segment(a: Point, b: Point, r: Rect) -> Option<(Point, Point)> {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let mut low: f64 = 0.;
    let mut high: f64 = 1.;
    for (p, q) in [
        (-dx, a.x() - r.origin().x()),
        (dx, r.max_x() - a.x()),
        (-dy, a.y() - r.origin().y()),
        (dy, r.max_y() - a.y()),
    ] {
        if p == 0. {
            if q < 0. {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0. {
                low = low.max(t);
            } else {
                high = high.min(t);
            }
            if low > high {
                return None;
            }
        }
    }
    Some((
        Point::new(a.x() + low * dx, a.y() + low * dy).ok()?,
        Point::new(a.x() + high * dx, a.y() + high * dy).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_x_signed_zeros_keep_frontmost_128_and_all_keyboard_identities() {
        let clip = Rect::new(-1., -1., 2., 2.).unwrap();
        let candidates = (0_u64..300)
            .map(|i| {
                let position = Point::new(if i.is_multiple_of(2) { -0. } else { 0. }, 0.).unwrap();
                Candidate {
                    custom: None,
                    hit: InspectedTarget {
                        values: vec![],
                        selection: crate::grammar::SelectionPolicy::AtomicTarget,
                        panel: None,
                        layer: LayerId::new(1),
                        target: Target::Source(crate::provenance::SourceRef {
                            dataset: crate::DatasetId::new(1),
                            key: crate::RowKey::new(i + 1),
                        }),
                        position,
                    },
                    clip,
                    rectangle: None,
                    line: true,
                    segment: None,
                    bounds: Bounds::point(position),
                    layer_order: 0,
                }
            })
            .collect();
        let index = Index::new(candidates);
        let point = Point::new(0., 0.).unwrap();
        let actual = index.query(point, InspectionMode::NearestX, 1., 128, false, clip);
        let reference = index.query(point, InspectionMode::NearestX, 1., 128, true, clip);
        assert_eq!(actual.hits, reference.hits);
        let keys: Vec<_> = actual
            .hits
            .iter()
            .map(|h| match h.target {
                Target::Source(s) => s.key.get(),
                _ => panic!("source"),
            })
            .collect();
        assert_eq!(keys, (173..=300).rev().collect::<Vec<_>>());
        assert_eq!(actual.examined, 128);
        assert_eq!(index.keyboard, (0..300).collect::<Vec<_>>());
        assert_eq!(index.identities.len(), 300);
    }
}
