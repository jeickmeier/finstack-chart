# ADR-007 — Actions, gesture ownership and controlled state

Status: accepted for WP-15/16, 7 September 2026. WP-17–19 add linking, reconciliation and
scheduling decisions without changing the synchronous core ownership boundary.

## Decision

Use one synchronous, window-independent `ActionReducer` for native, programmatic,
Python and WASM actions. It owns the acknowledged scene, bounded annotation command
history, one gesture basis, one frozen scene and bounded linked-source sequences.
`ChartState` remains a cloneable value snapshot with distinct component revisions.
No GPUI objects, source mutations or mandatory threads enter the reducer.

An action validates definition/state/scene fences before publishing a cloned proposed
transition. Only effective changes advance revisions or emit origin-preserving events.
A controlled host can acknowledge the exact proposal or submit a newer validated
replacement; it cannot reuse a component revision, inject ephemeral inspection, interrupt
an active gesture or manufacture a frozen snapshot. Replacements reset local history.

Begin captures one immutable presented basis through commit/cancel. Preview values
remain separate from committed state and saved state. Only committed annotation changes
enter bounded undo history. Source observations remain transaction-owned. Freeze retains
one coherent scene while ingestion remains independent; disposal releases ownership even
if a final event cannot be emitted because its revision is exhausted.

## Consequences and evidence

The public native adapter and actual portable adapters call this same reducer. Input
producers and application controls can be replaced independently. Explicit presentation
acknowledgment prevents pending layout or data from silently becoming an input basis.
The owner must retain the reducer for gestures/history; the old `ChartState::apply`
compatibility method cannot provide that lifetime.

[State/action contract](../state-action-contract.md) defines defaults, limits, events,
serialization and linking. [WP-15 evidence](../evidence/wp-15-completion-2026-09-07.md)
includes deterministic traces, controlled-response/cancellation failures, resource
release, real-font exports, actual Python/WASM execution and native controls.

WP-16 implements gesture geometry, paint priority and capture translation; constraints and linked-view
coordination remain WP-17; retention/follow reconciliation remains WP-18. The bounded
active-plus-pending scheduling decision and evidence belong to WP-19. G3 is open until
all WP-15–20 acceptance is complete.


## WP-16: presented indexes and typed navigation

Own spatial, sorted-x and semantic-target indexes beside an immutable presented scene.
Share indexes through Arc and retain only bounded result/state copies per input transition.
Use exact scene Arc identity for cache membership and stamps for external input fences.
Cloning an inspector/reducer must not copy a scene-sized candidate set or rebuild target
availability per pointer event. Overlapping geometry can still require linear hit work.

Navigation produces typed named-axis windows rather than editing domains or filtering
rows. Category endpoints are stable labels and timestamps are exact source ticks. Keep
legacy x/y intervals compatible; named windows override them under the existing viewport
revision and gesture lifecycle. One named scale has one window across its panels; this
adds no separate per-panel state namespace. Categories preserve full trained catalogs.

Native code owns event translation/capture/focus subscriptions and calls shared producers
before the reducer. Drags retain the original presented frame and native origin. Observe
focus transitions during rendering as well as through callbacks: the actual native blur
check exposed a missed focus-out callback on the pinned GPUI version. Python/WASM use
pure query DTOs and the same reducers, with no hidden presentation acknowledgment.

No dependency changed. Full-domain capture clears named windows only in its captured
copy. Live interaction overlays, linking, streaming reconciliation and scheduling remain
with their assigned packages. The [interaction contract](../interaction-contract.md)
and [WP-16 evidence](../evidence/wp-16-completion-2026-09-07.md) record the current scope.
