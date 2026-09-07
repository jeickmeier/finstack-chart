use crate::composition::Annotation;
use crate::grammar::PanelKey;
use crate::provenance::Target;
use crate::*;
use serde::{Deserialize, Serialize};

/// Input source retained in effective state events; synchronization carries its own sequence.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum ActionOrigin {
    /// Native pointer input.
    Pointer,
    /// Focused keyboard input.
    Keyboard,
    /// Host toolbar, legend or other control.
    Control,
    /// Direct application call.
    #[default]
    Programmatic,
    /// Application-defined linked view identity.
    Linked(String),
}
/// User view policy, independent of accepted source data and retained viewport.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FollowMode {
    /// Follow future latest-data extents according to host navigation policy.
    #[default]
    FollowLatest,
    /// Keep the user viewport while data is accepted.
    InspectHistory,
    /// Keep one coherent presented scene until explicit resume.
    FreezePresentation,
}
/// Revision-independent semantic identity; aggregate membership is resolved in its scene.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum TargetIdentity {
    /// Exact source row identity.
    Source {
        /// Dataset identity.
        dataset: DatasetId,
        /// Source key.
        key: RowKey,
    },
    /// Stable aggregate scope, never one representative source row.
    Aggregate {
        /// Input dataset.
        dataset: DatasetId,
        /// Aggregate identity.
        id: AggregateId,
        /// Group/bin scope.
        group: String,
    },
    /// Computed model identity and input dataset scope.
    Derived {
        /// Derived identity.
        id: DerivedId,
        /// Model name.
        model: String,
        /// Model version.
        version: Revision,
        /// Input dataset identities.
        datasets: Vec<DatasetId>,
    },
}
impl From<&Target> for TargetIdentity {
    fn from(t: &Target) -> Self {
        match t {
            Target::Source(s) => Self::Source {
                dataset: s.dataset,
                key: s.key,
            },
            Target::Aggregate {
                id, group, input, ..
            } => Self::Aggregate {
                dataset: input.dataset,
                id: *id,
                group: group.clone(),
            },
            Target::Derived {
                id,
                model,
                model_version,
                inputs,
            } => Self::Derived {
                id: *id,
                model: model.clone(),
                version: *model_version,
                datasets: inputs.iter().map(|v| v.dataset).collect(),
            },
        }
    }
}
/// Stable mark identity for hover, pin, selection and linking; no source values are copied.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct MarkTarget {
    /// Source epoch prevents accidental matching after replacing the source universe.
    pub epoch: SourceEpoch,
    /// Named layer identity.
    pub layer: LayerId,
    /// Exact facet identity, or the single panel.
    pub panel: Option<PanelKey>,
    /// Source/aggregate/model identity.
    pub identity: TargetIdentity,
}
impl MarkTarget {
    /// Preserve semantic identity from a presented inspection result.
    pub fn from_inspected(hit: &crate::inspection::InspectedTarget, epoch: SourceEpoch) -> Self {
        Self {
            epoch,
            layer: hit.layer,
            panel: hit.panel.clone(),
            identity: (&hit.target).into(),
        }
    }
}
/// Set operation used by point, brush, lasso and linked selection producers.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionChange {
    /// Replace the current set.
    Replace,
    /// Include all supplied targets.
    Add,
    /// Invert membership for supplied targets.
    Toggle,
    /// Exclude supplied targets.
    Remove,
    /// Clear; targets must be empty.
    Clear,
}
/// One gesture owner. Geometry hit priority is resolved by the input adapter before begin.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GestureKind {
    /// Navigation using a pinned coordinate basis.
    Viewport,
    /// Point/range/rectangle/lasso selection preview.
    Selection,
    /// Edit one authored annotation, never its source measurement.
    Annotation(String),
}
/// Validated transient replacement, excluded from serialized durable state and undo history.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GesturePreview {
    /// Complete visible interval preview.
    Viewport(super::Viewport),
    /// Complete named-axis navigation preview, retaining categorical/time identities.
    AxisWindows(super::AxisWindows),
    /// Complete selection preview from the pinned scene.
    Selection(Vec<MarkTarget>),
    /// Complete annotation preview after application/core constraints and snapping.
    Annotation(Box<Annotation>),
}
/// Explicit cancellation reason for host gesture lifecycle reporting.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelReason {
    /// Escape or explicit application cancellation.
    Explicit,
    /// Pointer capture was lost.
    CaptureLost,
    /// Chart keyboard/window focus was lost.
    FocusLost,
    /// Edited target was removed or definition invalidated its identity.
    TargetRemoved,
    /// Owning chart was disposed.
    Disposed,
}
/// Active gesture metadata; the reducer separately pins exactly one immutable scene.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ActiveGesture {
    /// Positive caller ID, scoped to this reducer lifetime.
    pub id: Revision,
    /// Exclusive owner.
    pub kind: GestureKind,
    /// Exact coordinate/hit basis selected on begin.
    pub basis: SceneStamp,
    /// Latest transient values, if any.
    pub preview: Option<GesturePreview>,
}
/// Bounded interaction configuration, separate from the authored chart definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InteractionConfig {
    /// Maximum retained selection/hover targets, in 1..=4096.
    pub max_targets: usize,
    /// Maximum undo commands, in 0..=128. Zero explicitly disables command history.
    pub history_capacity: usize,
    /// Manual viewport changes enter inspect-history by default.
    pub manual_view_enters_history: bool,
}
impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            max_targets: 4096,
            history_capacity: 64,
            manual_view_enters_history: true,
        }
    }
}
/// Distinct component revisions; each advances only for its effective value change.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StateRevisions {
    /// Committed durable state.
    pub durable: Revision,
    /// Follow policy.
    pub follow: Revision,
    /// Hover target set.
    pub hover: Revision,
    /// Keyboard focus target.
    pub focus: Revision,
    /// Pinned inspection target.
    pub pin: Revision,
    /// Selection target set.
    pub selection: Revision,
    /// Layer and legend visibility.
    pub visibility: Revision,
    /// Annotation edits.
    pub annotations: Revision,
    /// Active gesture/preview.
    pub gesture: Revision,
    /// Interaction configuration.
    pub configuration: Revision,
}
/// A revision-fenced action; scene references are checked for presentation-dependent input.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ActionRequest {
    /// Current authored definition revision.
    pub definition_revision: Revision,
    /// Current effective state revision; an old controlled response cannot overwrite it.
    pub expected_state: Revision,
    /// Source of the action, preserved in the event.
    pub origin: ActionOrigin,
    /// Exact presented scene, or pinned gesture basis for gesture updates.
    pub scene: Option<SceneStamp>,
    /// Typed shared action.
    pub action: super::ChartAction,
}
/// Effective changes only; repeated input returns no event.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StateEvent {
    /// Resulting total revision.
    pub revision: Revision,
    /// Original action source.
    pub origin: ActionOrigin,
    /// Whether committed state changed (previews never count).
    pub durable: bool,
    /// Whether destination scale/layout/visibility/furniture work is necessary.
    pub presentation_changed: bool,
    /// Explicit gesture cancellation reason, when applicable.
    pub cancellation: Option<CancelReason>,
}
/// Accepted transition with no-op and effective-change reporting.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct DispatchOutcome {
    /// Compatible basic state outcome.
    pub outcome: super::ActionOutcome,
    /// Absent for redundant input; acknowledgements need not emit application events.
    pub event: Option<StateEvent>,
}
