//! Shared revision-fenced state/actions. The retained reducer owns gesture snapshots and history.
//! Data mutations remain in the transaction engine; selection never filters source populations.
mod reducer;
mod types;
use crate::composition::{Annotation, FigureComposition};
use crate::grammar::ChartDefinition;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Revision};
pub use reducer::ActionReducer;
use std::collections::{BTreeMap, BTreeSet};
pub use types::*;

/// Explicit visible intervals. Descending endpoints are supported; equal/non-finite are not.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
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

/// Shared semantic actions produced by pointer, keyboard, controls and programmatic clients.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ChartAction {
    /// Set viewport without changing statistical populations; manual changes enter history mode.
    SetViewport(Viewport),
    /// Change presentation visibility without changing domain/population contributions.
    SetLayerVisible {
        /// Existing layer.
        layer: LayerId,
        /// Desired visibility.
        visible: bool,
    },
    /// Change legend presentation independently of layer visibility.
    SetLegendVisible(bool),
    /// Set explicit follow/inspect/freeze mode. Freeze requires a presented scene.
    SetFollow(FollowMode),
    /// Resume following accepted data; clears the frozen snapshot, preserving selection/edits.
    ResumeLatest,
    /// Replace hover targets from the exact presented scene.
    SetHover(Vec<MarkTarget>),
    /// Clear hover and keyboard focus with one transient transition.
    ClearInspection,
    /// Set the keyboard target independently of hover.
    SetFocus(Option<MarkTarget>),
    /// Pin or unpin an inspected semantic target.
    SetPinned(Option<MarkTarget>),
    /// Selection producers use the same set operations after querying presented geometry.
    Select {
        /// Set operation.
        change: SelectionChange,
        /// Exact scene-derived stable identities.
        targets: Vec<MarkTarget>,
    },
    /// Add/replace one annotation with validated explicit coordinates and text.
    SetAnnotation(Box<Annotation>),
    /// Remove an existing annotation (definition or state override).
    RemoveAnnotation(String),
    /// Replace bounded interaction configuration, advancing its separate revision.
    Configure(InteractionConfig),
    /// Begin exclusive ownership and pin the presented scene until commit/cancel.
    BeginGesture {
        /// Positive monotonic gesture identity.
        id: Revision,
        /// Gesture owner.
        kind: GestureKind,
    },
    /// Replace transient preview against the pinned scene; never writes command history.
    PreviewGesture {
        /// Active gesture identity.
        id: Revision,
        /// Validated semantic preview.
        preview: GesturePreview,
    },
    /// Commit a gesture; annotation edits produce one durable undo command.
    CommitGesture {
        /// Active gesture identity.
        id: Revision,
    },
    /// Discard preview and release its pinned scene; committed state is untouched.
    CancelGesture(CancelReason),
    /// Restore the previous durable command, retaining transient inspection separately.
    Undo,
    /// Reapply the last undone command.
    Redo,
    /// Apply domain/identity values from a named linked source, never pixels.
    Synchronize {
        /// Positive monotonic sequence for this linked source.
        revision: Revision,
        /// Optional complete viewport update.
        viewport: Option<Viewport>,
        /// Optional complete target set, including explicit clear.
        selection: Option<Vec<MarkTarget>>,
    },
    /// Restore automatic viewport, default follow/visibility and clear edits/inspection/selection.
    Reset,
}
/// Basic compatible transition outcome. No asynchronous presentation is implied.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionOutcome {
    /// False for redundant input.
    pub changed: bool,
    /// Effective state revision.
    pub revision: Revision,
    /// Effective visible interval changed; statistics remain unchanged.
    pub viewport_changed: bool,
}
#[derive(Clone, Debug, PartialEq)]
struct DurableState {
    viewport: Viewport,
    follow: FollowMode,
    hidden: BTreeSet<LayerId>,
    legend_visible: bool,
    selection: BTreeSet<MarkTarget>,
    pinned: Option<MarkTarget>,
    // None is an explicit deletion of an authored annotation.
    annotations: BTreeMap<String, Option<Annotation>>,
}
impl Default for DurableState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            follow: FollowMode::default(),
            hidden: BTreeSet::new(),
            legend_visible: true,
            selection: BTreeSet::new(),
            pinned: None,
            annotations: BTreeMap::new(),
        }
    }
}
/// Serializable committed interaction state. Ephemeral hover/focus/gestures are excluded.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InteractionSnapshot {
    /// Committed follow policy. A frozen snapshot itself is an owned runtime resource.
    pub follow: FollowMode,
    /// Legend visibility, independent of series visibility.
    pub legend_visible: bool,
    /// Stable selected identities.
    pub selection: Vec<MarkTarget>,
    /// Pinned semantic identity, resolved against the selected presented snapshot.
    pub pinned: Option<MarkTarget>,
    /// Annotation overrides/deletions by ID.
    pub annotations: BTreeMap<String, Option<Annotation>>,
    /// Explicit bounded behavior configuration.
    pub configuration: InteractionConfig,
    /// Component revision fences.
    pub revisions: StateRevisions,
}
/// Owned semantic state cloned into preparation; no window, history or scene handle is stored.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartState {
    revision: Revision,
    viewport_revision: Revision,
    durable: DurableState,
    hover: BTreeSet<MarkTarget>,
    focus: Option<MarkTarget>,
    active: Option<ActiveGesture>,
    configuration: InteractionConfig,
    revisions: StateRevisions,
}
impl ChartState {
    pub(crate) fn from_portable(
        definition: &ChartDefinition,
        revision: Revision,
        viewport_revision: Revision,
        viewport: Viewport,
        hidden_layers: Vec<LayerId>,
    ) -> ChartResult<Self> {
        viewport.validate()?;
        let hidden: BTreeSet<_> = hidden_layers.iter().copied().collect();
        if viewport_revision > revision
            || hidden.len() != hidden_layers.len()
            || hidden
                .iter()
                .any(|id| !definition.layers.iter().any(|l| l.id == *id))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid state revisions or hidden layer identities.",
            ));
        }
        Ok(Self {
            revision,
            viewport_revision,
            durable: DurableState {
                viewport,
                hidden,
                ..Default::default()
            },
            ..Default::default()
        })
    }
    /// Total effective state revision, including transient changes.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Effective viewport revision, including previews.
    pub fn viewport_revision(&self) -> Revision {
        self.viewport_revision
    }
    /// Independent component revision counters.
    pub fn revisions(&self) -> &StateRevisions {
        &self.revisions
    }
    /// Effective visible intervals; active navigation previews override committed values.
    pub fn viewport(&self) -> Viewport {
        match self.active.as_ref().and_then(|g| g.preview.as_ref()) {
            Some(GesturePreview::Viewport(v)) => *v,
            _ => self.durable.viewport,
        }
    }
    /// Committed intervals for durable serialization, excluding transient previews.
    pub fn committed_viewport(&self) -> Viewport {
        self.durable.viewport
    }
    /// Current committed follow policy.
    pub fn follow(&self) -> FollowMode {
        self.durable.follow
    }
    /// Current independent legend visibility.
    pub fn legend_visible(&self) -> bool {
        self.durable.legend_visible
    }
    /// True unless explicitly hidden; statistics are unaffected.
    pub fn is_visible(&self, layer: LayerId) -> bool {
        !self.durable.hidden.contains(&layer)
    }
    /// Committed selection identities. Selection previews are exposed by active_gesture.
    pub fn selection(&self) -> &BTreeSet<MarkTarget> {
        &self.durable.selection
    }
    /// Current exact hover targets, independent of keyboard focus.
    pub fn hover(&self) -> &BTreeSet<MarkTarget> {
        &self.hover
    }
    /// Current keyboard target.
    pub fn focus(&self) -> Option<&MarkTarget> {
        self.focus.as_ref()
    }
    /// Current pinned target.
    pub fn pinned(&self) -> Option<&MarkTarget> {
        self.durable.pinned.as_ref()
    }
    /// Active owner/preview metadata. The retained reducer owns its exact basis.
    pub fn active_gesture(&self) -> Option<&ActiveGesture> {
        self.active.as_ref()
    }
    /// Configured bounds and manual navigation policy.
    pub fn configuration(&self) -> &InteractionConfig {
        &self.configuration
    }
    /// Merge authored annotations with effective committed/preview edits in authored order, appending new identities deterministically.
    pub fn annotations(&self, definition: &ChartDefinition) -> Vec<Annotation> {
        let mut values: Vec<_> = definition
            .figure
            .iter()
            .flat_map(|f| f.annotations.iter().cloned())
            .collect();
        let replace = |values: &mut Vec<Annotation>, id: &str, value: Option<&Annotation>| {
            if let Some(index) = values.iter().position(|a| a.id == id) {
                if let Some(a) = value {
                    values[index] = a.clone();
                } else {
                    values.remove(index);
                }
            } else if let Some(a) = value {
                values.push(a.clone());
            }
        };
        for (id, value) in &self.durable.annotations {
            replace(&mut values, id, value.as_ref());
        }
        if let Some(GesturePreview::Annotation(a)) =
            self.active.as_ref().and_then(|g| g.preview.as_ref())
        {
            replace(&mut values, &a.id, Some(a));
        }
        values
    }
    /// Effective composition with edited annotations; data and definition remain immutable.
    pub(crate) fn figure(&self, definition: &ChartDefinition) -> Option<FigureComposition> {
        if definition.figure.is_none()
            && self.durable.annotations.is_empty()
            && !matches!(
                self.active.as_ref().and_then(|g| g.preview.as_ref()),
                Some(GesturePreview::Annotation(_))
            )
        {
            return None;
        }
        let mut figure = definition.figure.clone().unwrap_or_default();
        figure.annotations = self.annotations(definition);
        Some(figure)
    }
    /// Durable wire state, with transient state explicitly omitted.
    pub fn interaction_snapshot(&self) -> InteractionSnapshot {
        InteractionSnapshot {
            follow: self.durable.follow,
            legend_visible: self.durable.legend_visible,
            selection: self.durable.selection.iter().cloned().collect(),
            pinned: self.durable.pinned.clone(),
            annotations: self.durable.annotations.clone(),
            configuration: self.configuration.clone(),
            revisions: self.revisions.clone(),
        }
    }
    pub(crate) fn retain_transient_from(&mut self, current: &Self) {
        self.hover = current.hover.clone();
        self.focus = current.focus.clone();
        self.revisions.hover = current.revisions.hover;
        self.revisions.focus = current.revisions.focus;
    }
    pub(crate) fn restore_interaction(
        &mut self,
        definition: &ChartDefinition,
        snapshot: InteractionSnapshot,
    ) -> ChartResult<()> {
        let selection: BTreeSet<_> = snapshot.selection.iter().cloned().collect();
        if selection.len() != snapshot.selection.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Duplicate serialized selection targets.",
            ));
        }
        self.durable.follow = snapshot.follow;
        self.durable.legend_visible = snapshot.legend_visible;
        self.durable.selection = selection;
        self.durable.pinned = snapshot.pinned;
        self.durable.annotations = snapshot.annotations;
        self.configuration = snapshot.configuration;
        self.revisions = snapshot.revisions;
        reducer::validate_state(definition, self)
    }
    /// Compatibility entry point for one-shot actions. Retain ActionReducer for gestures/history.
    pub fn apply(
        &mut self,
        definition: &ChartDefinition,
        action: ChartAction,
    ) -> ChartResult<ActionOutcome> {
        if self.active.is_some() {
            return Err(error(
                DiagnosticCode::Validation,
                "Retain the owning reducer to update or cancel an active gesture.",
            ));
        }
        let mut reducer = ActionReducer::new(self.clone());
        let result = reducer.dispatch(
            definition,
            ActionRequest {
                definition_revision: definition.revision,
                expected_state: self.revision,
                origin: ActionOrigin::Programmatic,
                scene: None,
                action,
            },
        )?;
        *self = reducer.state().clone();
        Ok(result.outcome)
    }
}
fn error(code: DiagnosticCode, message: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use current definition/state revisions, an exact presented scene and bounded valid semantic values.",
    )
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
