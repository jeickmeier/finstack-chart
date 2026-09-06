//! Minimal synchronous presentation action boundary. Full interaction belongs to WP-15.
//! Viewport/visibility never changes statistical source population in this foundation.

use crate::grammar::ChartDefinition;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Revision};
use std::collections::BTreeSet;

/// Explicit visible intervals. Descending endpoints are supported; equal/non-finite are not.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Viewport {
    /// Optional visible x interval in declared data/calculation units.
    pub x: Option<(f64, f64)>,
    /// Optional visible y interval in declared data/calculation units.
    pub y: Option<(f64, f64)>,
}
impl Viewport {
    pub(crate) fn validate(self) -> ChartResult<()> {
        for (a, b) in [self.x, self.y].into_iter().flatten() {
            if !a.is_finite() || !b.is_finite() || a == b {
                return Err(Diagnostic::error(
                    DiagnosticCode::NumericalDomain,
                    "Viewport endpoints must be finite and distinct.",
                    "Supply a finite visible interval; automatic/constant-domain expansion belongs to scale preparation.",
                ));
            }
        }
        Ok(())
    }
}

/// Implemented vertical-slice actions shared by programmatic and future host input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartAction {
    /// Change visible intervals without filtering source rows or moving bin edges.
    SetViewport(Viewport),
    /// Change presentation visibility, retaining statistical/domain contributions.
    SetLayerVisible {
        /// Layer belonging to the supplied definition.
        layer: LayerId,
        /// Desired presentation visibility.
        visible: bool,
    },
    /// Restore visible layers and automatic viewport.
    Reset,
}

/// Resulting effective state transition; no queued input or asynchronous presentation implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionOutcome {
    /// False for an idempotent action.
    pub changed: bool,
    /// Resulting state revision.
    pub revision: Revision,
    /// Whether scale/range preparation needs the new viewport; stats remain unchanged.
    pub viewport_changed: bool,
}

/// Owned minimal presentation state, cloned into each prepared result.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartState {
    revision: Revision,
    viewport_revision: Revision,
    viewport: Viewport,
    hidden: BTreeSet<LayerId>,
}
impl ChartState {
    /// Total effective state revision.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Effective viewport revision for later projection/layout invalidation.
    pub fn viewport_revision(&self) -> Revision {
        self.viewport_revision
    }
    /// Explicit visible intervals.
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }
    /// Visibility defaults to true; it does not filter data/statistics.
    pub fn is_visible(&self, layer: LayerId) -> bool {
        !self.hidden.contains(&layer)
    }
    /// Validate first, then publish one checked effective transition.
    pub fn apply(
        &mut self,
        definition: &ChartDefinition,
        action: ChartAction,
    ) -> ChartResult<ActionOutcome> {
        let mut next = self.clone();
        match action {
            ChartAction::SetViewport(viewport) => {
                viewport.validate()?;
                next.viewport = viewport;
            }
            ChartAction::SetLayerVisible { layer, visible } => {
                if !definition.layers.iter().any(|l| l.id == layer) {
                    let mut e = Diagnostic::error(
                        DiagnosticCode::Validation,
                        "The visibility action names an absent layer.",
                        "Use a layer identity from the current definition.",
                    );
                    e.context.layer = Some(layer);
                    return Err(e);
                }
                if visible {
                    next.hidden.remove(&layer);
                } else {
                    next.hidden.insert(layer);
                }
            }
            ChartAction::Reset => {
                next.viewport = Viewport::default();
                next.hidden.clear();
            }
        }
        let changed = next != *self;
        let viewport_changed = next.viewport != self.viewport;
        if changed {
            next.revision = self.revision.checked_next()?;
            if viewport_changed {
                next.viewport_revision = self.viewport_revision.checked_next()?;
            }
            *self = next;
        }
        Ok(ActionOutcome {
            changed,
            revision: self.revision,
            viewport_changed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_and_viewport_revision_exhaustion_leave_state_unchanged() {
        for viewport_exhausted in [false, true] {
            let mut state = ChartState::default();
            if viewport_exhausted {
                state.viewport_revision = Revision::new(u64::MAX);
            } else {
                state.revision = Revision::new(u64::MAX);
            }
            let before = state.clone();
            let outcome = state.apply(
                &ChartDefinition::new(Revision::INITIAL),
                ChartAction::SetViewport(Viewport {
                    x: Some((0., 1.)),
                    y: None,
                }),
            );
            assert_eq!(outcome.unwrap_err().code, DiagnosticCode::RevisionOverflow);
            assert_eq!(state, before);
        }
    }
}
