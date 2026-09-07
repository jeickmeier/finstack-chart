# ADR-007 — Actions, gesture ownership and controlled state

Status: accepted for WP-15, 7 September 2026. WP-16–19 add input, reconciliation and
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

Gesture geometry/priority/capture translation remains WP-16; constraints and linked-view
coordination remain WP-17; retention/follow reconciliation remains WP-18. The bounded
active-plus-pending scheduling decision and evidence belong to WP-19. G3 is open until
all WP-15–20 acceptance is complete.
