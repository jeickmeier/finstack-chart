//! Minimal synchronous inspection of the presented scene; full selection/navigation is WP-15/16.
use crate::grammar::Geom;
use crate::layout::LaidOutChart;
use crate::provenance::Target;
use crate::scene::{PathCommand, Primitive};
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Point, Rect, Revision, SceneStamp};
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
#[derive(Clone, Debug, PartialEq)]
pub struct InspectedTarget {
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
    hit: InspectedTarget,
    clip: Rect,
    rectangle: Option<Rect>,
    line: bool,
}
/// Bounded inspection state tied to one immutable presented chart snapshot.
#[derive(Clone, Debug)]
pub struct Inspector {
    presented: Arc<LaidOutChart>,
    candidates: Vec<Candidate>,
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
        let mut candidates = vec![];
        for (item, targets) in presented.scene().items().iter().zip(presented.targets()) {
            let Some(layer) = item.layer else { continue };
            let line = presented
                .prepared()
                .definition()
                .layers
                .iter()
                .find(|l| l.id == layer)
                .is_some_and(|l| matches!(l.geom, Geom::Line { .. }));
            let clip = item.clip.unwrap_or(presented.scene().bounds());
            let mut add = |position: Point, target: &Target, rectangle: Option<Rect>| {
                // Partially visible rectangles can be inspected where their actual clip intersects.
                if rectangle.is_some() || contains(clip, position) {
                    candidates.push(Candidate {
                        hit: InspectedTarget {
                            layer,
                            target: target.clone(),
                            position,
                        },
                        clip,
                        rectangle,
                        line,
                    });
                }
            };
            match &item.primitive {
                Primitive::Point { center, .. } => {
                    if let Some(t) = targets.first() {
                        add(*center, t, None);
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
                            );
                        }
                    }
                }
                Primitive::Path { commands, .. } => {
                    for (p, t) in commands
                        .iter()
                        .filter_map(|c| match c {
                            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => Some(*p),
                            _ => None,
                        })
                        .zip(targets)
                    {
                        add(p, t, None);
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
            presented,
            candidates,
            hits: vec![],
            focus: None,
            revision: Revision::INITIAL,
            radius,
            max_grouped,
        })
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
        self.candidates.len()
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
            InspectionAction::Hover(Some(p)) => (self.hit_test(p), None),
            InspectionAction::StepFocus { forward } => {
                if self.candidates.is_empty() {
                    (vec![], None)
                } else {
                    let n = self.candidates.len();
                    let index = match (self.focus, forward) {
                        (None, true) => 0,
                        (None, false) => n - 1,
                        (Some(i), true) => (i + 1) % n,
                        (Some(i), false) => (i + n - 1) % n,
                    };
                    (vec![self.candidates[index].hit.clone()], Some(index))
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
    fn hit_test(&self, p: Point) -> Vec<InspectedTarget> {
        if !contains(self.presented.scene().bounds(), p) {
            return vec![];
        }
        // Nearest scatter point/contained bar; equal distances prefer visible z-order.
        let mut best = None;
        let mut best_distance = self.radius;
        for c in self
            .candidates
            .iter()
            .rev()
            .filter(|c| !c.line && contains(c.clip, p))
        {
            let distance = if let Some(r) = c.rectangle {
                if !contains(r, p) {
                    continue;
                }
                0.
            } else {
                (c.hit.position.x() - p.x()).hypot(c.hit.position.y() - p.y())
            };
            if distance <= self.radius && (best.is_none() || distance < best_distance) {
                best = Some(c.hit.clone());
                best_distance = distance;
            }
        }
        if let Some(best) = best {
            return vec![best];
        }
        let mut distance = f64::INFINITY;
        let mut selected_x = None;
        let mut hits = vec![];
        for c in self
            .candidates
            .iter()
            .rev()
            .filter(|c| c.line && contains(c.clip, p))
        {
            let d = (c.hit.position.x() - p.x()).abs();
            if d < distance {
                distance = d;
                selected_x = Some(c.hit.position.x());
                hits.clear();
            }
            if d == distance
                && selected_x == Some(c.hit.position.x())
                && hits.len() < self.max_grouped
            {
                hits.push(c.hit.clone());
            }
        }
        hits
    }
}
