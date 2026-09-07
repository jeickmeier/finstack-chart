# State and action contract

WP-15 implements the state ownership portion of INT-01/02/05/06, SCN-04,
STM-02 and QLT-01. [ADR-007](adr/007-actions-gestures-and-controlled-state.md)
records the decision; [completion evidence](evidence/wp-15-completion-2026-09-07.md)
records execution. [WP-16 interaction](interaction-contract.md) adds indexed producers
and typed windows. Editing constraints, streaming reconciliation, scheduling and live
publication policies are owned by WP-17–20.

## One reducer across hosts

Retain `ActionReducer` beside the authored `ChartDefinition`. `ChartState` contains
values and component revisions; it owns no window, worker, data snapshot or command
history. The reducer owns bounded history, the acknowledged scene, at most one gesture
basis and one frozen scene. Preparing a new scene does not acknowledge its presentation.
Call `present(Arc<LaidOutChart>)` only when the host selects that scene for display/input.

`ActionRequest` supplies the current definition revision, expected state revision,
origin, optional scene stamp and typed action. An outdated definition/state fence rejects
atomically. Scene-dependent input additionally needs the exact presented or pinned
scene stamp. `DispatchOutcome` contains a basic outcome and an optional `StateEvent`.
Redundant actions emit no event and advance no revisions. Effective events preserve
Pointer, Keyboard, Control, Programmatic or Linked origin, distinguish committed from
transient state, and indicate whether layout/presentation preparation is necessary.

`ChartState::apply` remains a one-shot compatibility path for actions needing no retained
scene/history. It rejects an active gesture instead of silently dropping its owner.
Native `ChartView::dispatch_action` and portable `Chart::dispatch` use the retained
reducer. The native adapter prepares a changed presentation before publishing state;
hover/focus changes reuse the existing frame. Source updates use the transaction engine.

## Independent state and revisions

| Value | Behavior |
| --- | --- |
| Viewport | Legacy numeric intervals plus typed named-axis numeric/time/category windows; a preview is effective but uncommitted. Selection never changes statistical populations. |
| Follow | FollowLatest, InspectHistory or FreezePresentation. Manual viewport changes enter InspectHistory by default. ResumeLatest clears freeze and preserves the viewport and edits; the streaming owner supplies latest-data navigation in WP-18. |
| Hover / focus | Separate transient identities and revisions; a focus change does not replace hover. ClearInspection clears both. |
| Pin / selection | Committed stable identities. Replace/Add/Toggle/Remove/Clear are common set operations. Custom disabled-selection targets reject selection. |
| Visibility | Independent layer and legend values with a visibility component revision. Hidden layers retain their statistical/domain contribution. |
| Annotations | Overrides/deletions of stable authored IDs, preserving authored paint order and appending new IDs deterministically. Source values are never edited. |
| Gesture | One owner, positive monotonically increasing ID, exact basis stamp, latest optional preview. |
| Configuration | Separate bounded target/history capacities and the manual-navigation follow policy. |

All effective actions advance the total revision once; only changed components advance
their own revisions. Effective viewport changes also advance the viewport revision.
Cancellation can therefore advance a transient viewport revision without changing the
committed viewport. Revision exhaustion rejects ordinary actions atomically.

Target identity includes source epoch, layer and panel plus an exact source key,
aggregate ID/group/input dataset, or derived ID/model/version/input datasets. It never
replaces an aggregate with a representative row. Scene resolution supplies membership
and source values. Selection and hover default to at most 4096 distinct targets;
configuration may lower that cap. Per-target metadata is bounded to 8192 bytes,
64 panel fields and 64 derived input datasets.

## Gestures, commands and lifetime

Begin pins the actual presented scene. A later resize/data preparation/presentation
cannot change that coordinate basis. Preview must match the owner kind and gesture ID;
a second owner and unrelated durable actions reject until commit/cancel. Viewport,
selection and one annotation are supported preview families. Preview equal to the
committed value removes the effective preview. Hover/focus may change independently.

Commit writes the final preview once. Cancel discards it and reports Explicit,
CaptureLost, FocusLost, TargetRemoved or Disposed. Native pointer capture, Escape and focus-loss translation are implemented by WP-16.
Constrained annotation editing remains WP-17. These reasons also execute through the
shared reducer without a window.

Only committed annotation edits enter undo/redo history. Fifty previews followed by a
commit create one command; hover, focus, selection and navigation create none. A new
edit clears redo. Commands contain annotation values, never source snapshots. Default
history is 64 commands, configurable to 0..128; zero disables history. Annotation state
is limited to 256 override identities and validated bounded rich text/coordinates.
Reset restores default durable state and clears inspection/gesture ownership; an
annotation change caused by reset is an undoable annotation command.

Freeze retains a coherent acknowledged scene while source transactions continue.
Resume releases its special retention; the previous displayed scene remains owned until
replacement presentation. Disposal releases all owned scene/history/link resources,
including when revision exhaustion prevents emitting a cancellation event. Disposed
reducers reject new dispatches and controlled replacements. An independently retained
export snapshot still owns its own resources.

## Controlled and portable state

`accept_controlled` requires the current expected state revision. An exact acknowledgment
is a no-op. A replacement must have a newer total revision and nonregressing component
revisions; changing a component under its old revision is rejected. A replacement cannot
inject hover/focus, interrupt an active gesture or fabricate an uncaptured freeze.
Successful replacement clears local undo/redo history.

The version-1 `StateEnvelope` has an optional `interaction` object for committed follow,
visibility, selection, pin, annotations, configuration, optional named-axis windows and
component revisions. Legacy
minimal envelopes still decode, but must satisfy current revision fences when restored.
Definitions contain no pointer/gesture state. Saved state excludes hover/focus values,
preview values, active gesture, command history and scene handles. Restoring a saved
state preserves the current transient hover/focus. A save taken during preview contains
the committed values; applying it cannot implicitly cancel a live gesture.

Rust `PortableChart::present()` and actual Python/WASM `Chart.present()` acknowledge the
scene used for subsequent input and return its JSON. `dispatch()` takes an ActionRequest
JSON without a version wrapper, using the definition/state/scene fences; the old `action()`
versioned envelope remains available. `scene()` prepares output without acknowledging
input presentation. PyO3 detaches Rust work; WASM results are owned copies. The adapters
do not implement independent state logic.

Linked synchronization pairs a `Linked(source)` origin with a positive source sequence
and optional semantic viewport/selection. Older sequences and exact echoes emit no
event. Reusing a sequence for other values rejects. At most 64 linked source records are
retained. Hosts propagate effective events while retaining the original source/sequence;
WP-17 supplies the multi-view coordinator and examples.
