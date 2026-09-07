//! Indexed inspection and selection of the exact immutable presented scene.
mod index;
mod selection;
use crate::grammar::Geom;
use crate::layout::LaidOutChart;
use crate::provenance::Target;
use crate::scene::{PathCommand, Primitive};
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Point, Rect, Revision, SceneStamp};
use index::{Bounds, Index};
pub use index::{InspectionMode, InspectionQuery};
pub use selection::SelectionRegion;
use std::sync::Arc;

/// Input source, retained in effective-change outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputOrigin {
    /// Pointer movement or departure.
    Pointer,
    /// Keyboard focus/navigation.
    Keyboard,
    /// Caller controls or direct API invocation.
    Programmatic,
}
/// Minimal common action path for pointer, keyboard and programmatic inspection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InspectionAction {
    /// Inspect scene-local coordinates; None clears hover.
    Hover(Option<Point>),
    /// Advance or reverse through visible semantic targets in paint/vertex order.
    StepFocus {
        /// True advances, false moves backward; traversal wraps.
        forward: bool,
    },
    /// Clear hover and keyboard focus.
    Clear,
}
/// Exact presented mark or line vertex; no interpolation or invented source row.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct InspectedTarget {
    /// Exact custom semantic values; ordinary marks resolve their original target data.
    pub values: Vec<(String, crate::grammar::SemanticValue)>,
    /// Declared target selection behavior.
    pub selection: crate::grammar::SelectionPolicy,
    /// Stable facet identity; absent for a single-panel chart.
    pub panel: Option<crate::grammar::PanelKey>,
    /// Originating layer.
    pub layer: LayerId,
    /// Exact source or aggregate provenance.
    pub target: Target,
    /// Destination point used to anchor native inspection.
    pub position: Point,
}
/// Effective reducer outcome; redundant input does not advance the revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionOutcome {
    /// Whether inspected targets changed.
    pub changed: bool,
    /// Checked inspection-state revision.
    pub revision: Revision,
    /// Original event source.
    pub origin: InputOrigin,
}
#[derive(Clone, Debug)]
struct Candidate {
    custom: Option<crate::grammar::GeometryInteraction>,
    hit: InspectedTarget,
    clip: Rect,
    rectangle: Option<Rect>,
    line: bool,
    segment: Option<(Point, Point, f64)>,
    bounds: Bounds,
    layer_order: usize,
}
/// Bounded inspection state tied to one immutable presented chart snapshot.
#[derive(Clone, Debug)]
pub struct Inspector {
    presented: Arc<LaidOutChart>,
    index: Arc<Index>,
    hits: Vec<InspectedTarget>,
    focus: Option<usize>,
    revision: Revision,
    radius: f64,
    max_grouped: usize,
}
fn error(code: DiagnosticCode, message: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the currently presented scene stamp, finite coordinates and bounded inspection options.",
    )
}
fn contains(r: Rect, p: Point) -> bool {
    p.x() >= r.origin().x() && p.x() <= r.max_x() && p.y() >= r.origin().y() && p.y() <= r.max_y()
}
impl Inspector {
    /// Build once for a presented frame. Candidate work is bounded by the scene's existing
    /// item/path budgets; at most max_grouped (1..128) nearest-x line targets are returned.
    pub fn new(presented: Arc<LaidOutChart>, radius: f64, max_grouped: usize) -> ChartResult<Self> {
        if !radius.is_finite() || radius <= 0. || !(1..=128).contains(&max_grouped) {
            return Err(error(
                DiagnosticCode::Validation,
                "Inspection radius must be positive/finite and grouped limit 1..128.",
            ));
        }
        for data in presented.prepared().source().get()?.datasets() {
            data.prepare_lookup();
        }
        let mut candidates = vec![];
        let mut paint_group = None;
        let mut layer_order = 0;
        for (index, (item, targets)) in presented
            .scene()
            .items()
            .iter()
            .zip(presented.targets())
            .enumerate()
        {
            let Some(layer) = item.layer else { continue };
            let line = presented
                .prepared()
                .definition()
                .layers
                .iter()
                .find(|l| l.id == layer)
                .is_some_and(|l| matches!(l.geom, Geom::Line { .. }));
            let clip = item.clip.unwrap_or(presented.scene().bounds());
            let group = (layer, clip, presented.item_panels()[index].clone());
            if paint_group.as_ref() != Some(&group) {
                layer_order = index;
                paint_group = Some(group);
            }
            if let Some(info) = presented.interactions().get(&index) {
                if let Some(target) = targets.first() {
                    let bounds = info.hit.bounds()?;
                    let (left, right, top, bottom) = (
                        bounds.origin().x().max(clip.origin().x()),
                        bounds.max_x().min(clip.max_x()),
                        bounds.origin().y().max(clip.origin().y()),
                        bounds.max_y().min(clip.max_y()),
                    );
                    if left <= right && top <= bottom {
                        let anchor = info.hit.anchor()?;
                        let position = Point::new(
                            anchor.x().clamp(left, right),
                            anchor.y().clamp(top, bottom),
                        )?;
                        candidates.push(Candidate {
                            custom: Some(info.clone()),
                            hit: InspectedTarget {
                                values: info.values.clone(),
                                selection: info.selection,
                                panel: presented.item_panels()[index].clone(),
                                layer,
                                target: target.clone(),
                                position,
                            },
                            clip,
                            rectangle: None,
                            line: false,
                            segment: None,
                            bounds: Bounds {
                                x0: left,
                                x1: right,
                                y0: top,
                                y1: bottom,
                            },
                            layer_order,
                        });
                    }
                }
                continue;
            }
            let mut add = |position: Point,
                           target: &Target,
                           rectangle: Option<Rect>,
                           segment: Option<(Point, Point, f64)>| {
                // Partially visible rectangles can be inspected where their actual clip intersects.
                if rectangle.is_some() || segment.is_some() || contains(clip, position) {
                    candidates.push(Candidate {
                        custom: None,
                        hit: InspectedTarget {
                            values: vec![],
                            selection: crate::grammar::SelectionPolicy::AtomicTarget,
                            panel: presented.item_panels()[index].clone(),
                            layer,
                            target: target.clone(),
                            position,
                        },
                        clip,
                        rectangle,
                        line,
                        segment,
                        bounds: rectangle
                            .map(Bounds::from)
                            .or_else(|| segment.map(|(a, b, w)| Bounds::segment(a, b, w)))
                            .unwrap_or_else(|| Bounds::point(position))
                            .clipped(clip),
                        layer_order,
                    });
                }
            };
            match &item.primitive {
                Primitive::Point { center, .. } | Primitive::Symbol { center, .. } => {
                    if let Some(t) = targets.first() {
                        add(*center, t, None, None);
                    }
                }
                Primitive::Rectangle { bounds, .. } => {
                    if let Some(t) = targets.first() {
                        let left = bounds.origin().x().max(clip.origin().x());
                        let right = bounds.max_x().min(clip.max_x());
                        let top = bounds.origin().y().max(clip.origin().y());
                        let bottom = bounds.max_y().min(clip.max_y());
                        if left < right && top < bottom {
                            add(
                                Point::new(left.midpoint(right), top.midpoint(bottom))?,
                                t,
                                Some(*bounds),
                                None,
                            );
                        }
                    }
                }
                Primitive::Rule { from, to, stroke } => {
                    if let Some(t) = targets.first()
                        && Bounds::segment(*from, *to, stroke.width / 2.).intersects(clip.into())
                    {
                        // Clip the stroke's bounds, not its centerline: a wick centered just
                        // outside the viewport can still paint a visible strip inside it.
                        add(
                            Point::new(
                                from.x()
                                    .midpoint(to.x())
                                    .clamp(clip.origin().x(), clip.max_x()),
                                from.y()
                                    .midpoint(to.y())
                                    .clamp(clip.origin().y(), clip.max_y()),
                            )?,
                            t,
                            None,
                            Some((*from, *to, stroke.width / 2.)),
                        );
                    }
                }
                Primitive::Path { commands, .. }
                | Primitive::DashedPath { commands, .. }
                | Primitive::FilledPath { commands, .. } => {
                    for (p, t) in commands
                        .iter()
                        .filter_map(|c| match c {
                            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => Some(*p),
                            _ => None,
                        })
                        .zip(targets)
                    {
                        add(p, t, None, None);
                    }
                }
                _ => {}
            }
        }
        let index = Arc::new(Index::new(candidates));
        Ok(Self {
            index,
            presented,
            hits: vec![],
            focus: None,
            revision: Revision::INITIAL,
            radius,
            max_grouped,
        })
    }
    /// Change query bounds without rebuilding immutable geometry/indexes.
    pub fn with_options(&self, radius: f64, max_grouped: usize) -> ChartResult<Self> {
        if !radius.is_finite() || radius <= 0. || !(1..=128).contains(&max_grouped) {
            return Err(error(
                DiagnosticCode::Validation,
                "Inspection radius/group limit is invalid.",
            ));
        }
        let mut result = self.clone();
        result.radius = radius;
        result.max_grouped = max_grouped;
        Ok(result)
    }
    /// Exact immutable geometry/data/provenance used for painting and source resolution.
    pub fn presented(&self) -> &Arc<LaidOutChart> {
        &self.presented
    }
    /// Current effective targets, empty when nothing is inspected.
    pub fn hits(&self) -> &[InspectedTarget] {
        &self.hits
    }
    /// Whether keyboard traversal owns the current inspection.
    pub fn has_keyboard_focus(&self) -> bool {
        self.focus.is_some()
    }
    /// Candidate count, independent of GPUI entity count.
    pub fn candidate_count(&self) -> usize {
        self.index.candidates.len()
    }
    /// Distinct visible semantic targets in deterministic keyboard order, without cloning rows.
    /// Consumers can page this iterator for accessible tables or match identities across views.
    pub fn semantic_targets(&self) -> impl ExactSizeIterator<Item = &InspectedTarget> {
        self.index
            .keyboard
            .iter()
            .map(|i| &self.index.candidates[*i].hit)
    }
    /// Resolve the authoritative presented target in logarithmic identity lookup work.
    pub fn target(
        &self,
        target: &crate::state::MarkTarget,
    ) -> ChartResult<Option<&InspectedTarget>> {
        if target.epoch != self.presented.prepared().source().get()?.epoch() {
            return Ok(None);
        }
        Ok(self
            .index
            .identities
            .get(&(target.panel.clone(), target.layer, target.identity.clone()))
            .map(|entry| &self.index.candidates[entry.keyboard_index].hit))
    }
    /// Reduce an action only against its presented stamp; redundant targets are idempotent.
    pub fn dispatch(
        &mut self,
        stamp: SceneStamp,
        action: InspectionAction,
        origin: InputOrigin,
    ) -> ChartResult<InspectionOutcome> {
        if stamp != self.presented.scene().stamp() {
            return Err(error(
                DiagnosticCode::Superseded,
                "Inspection action belongs to another presented scene.",
            ));
        }
        let (hits, focus) = match action {
            InspectionAction::Clear => (vec![], None),
            InspectionAction::Hover(None) => (vec![], None),
            InspectionAction::Hover(Some(p)) => (self.query(p, InspectionMode::Auto).hits, None),
            InspectionAction::StepFocus { forward } => {
                if self.index.keyboard.is_empty() {
                    (vec![], None)
                } else {
                    let n = self.index.keyboard.len();
                    let index = match (self.focus, forward) {
                        (None, true) => 0,
                        (None, false) => n - 1,
                        (Some(i), true) => (i + 1) % n,
                        (Some(i), false) => (i + n - 1) % n,
                    };
                    (
                        vec![
                            self.index.candidates[self.index.keyboard[index]]
                                .hit
                                .clone(),
                        ],
                        Some(index),
                    )
                }
            }
        };
        let changed = hits != self.hits || focus != self.focus;
        if changed {
            self.revision = self.revision.checked_next()?;
            self.hits = hits;
            self.focus = focus;
        }
        Ok(InspectionOutcome {
            changed,
            revision: self.revision,
            origin,
        })
    }
    /// Indexed read-only query. Work counters count candidate geometry tests and visited tree nodes.
    pub fn query(&self, point: Point, mode: InspectionMode) -> InspectionQuery {
        self.index.query(
            point,
            mode,
            self.radius,
            self.max_grouped,
            false,
            self.presented.scene().bounds(),
        )
    }
    /// Linear reference path for differential validation and benchmark comparisons.
    pub fn query_scan(&self, point: Point, mode: InspectionMode) -> InspectionQuery {
        self.index.query(
            point,
            mode,
            self.radius,
            self.max_grouped,
            true,
            self.presented.scene().bounds(),
        )
    }
    /// Visible highlight extents for stable identities, with four destination units around vertices.
    /// Missing epochs/targets produce no geometry; lookup is proportional to selection size.
    pub fn highlights(&self, targets: &[crate::state::MarkTarget]) -> ChartResult<Vec<Rect>> {
        if targets.len() > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Highlight target limit is 4096.",
            ));
        }
        let epoch = self.presented.prepared().source().get()?.epoch();
        let scene = self.presented.scene().bounds();
        targets
            .iter()
            .filter(|t| t.epoch == epoch)
            .filter_map(|t| {
                self.index
                    .identities
                    .get(&(t.panel.clone(), t.layer, t.identity.clone()))
            })
            .map(|entry| {
                let b = entry.bounds;
                let x0 = (b.x0 - 4.).max(scene.origin().x());
                let x1 = (b.x1 + 4.).min(scene.max_x());
                let y0 = (b.y0 - 4.).max(scene.origin().y());
                let y1 = (b.y1 + 4.).min(scene.max_y());
                Rect::new(x0, y0, x1 - x0, y1 - y0)
            })
            .collect()
    }
    /// Restore keyboard ownership by stable identity after a new frame; removed targets clear it.
    pub fn restore_focus(&mut self, target: &crate::state::MarkTarget) -> ChartResult<bool> {
        let epoch = self.presented.prepared().source().get()?.epoch();
        let focus = self.index.keyboard.iter().position(|i| {
            crate::state::MarkTarget::from_inspected(&self.index.candidates[*i].hit, epoch)
                == *target
        });
        if let Some(focus) = focus {
            self.focus = Some(focus);
            self.hits = vec![
                self.index.candidates[self.index.keyboard[focus]]
                    .hit
                    .clone(),
            ];
            Ok(true)
        } else {
            self.focus = None;
            self.hits.clear();
            Ok(false)
        }
    }
}
