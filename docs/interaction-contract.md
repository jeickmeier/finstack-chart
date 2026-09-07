# Presented-scene interaction contract

WP-16 implements indexed inspection, selection producers and navigation under
INT-03/04/05/06, SCL-01, SCN-04 and STM-05. It extends the
[state/action contract](state-action-contract.md). [ADR-007](adr/007-actions-gestures-and-controlled-state.md)
records the ownership decision. Linked views/annotation handles, ingestion reconciliation,
scheduling and coherent live publication remain WP-17–20.

## Inspection and keyboard order

`Inspector` retains one `Arc<LaidOutChart>` and one immutable shared index. Cloning an
inspector does not copy its candidate population. Building the index happens for a new
presented frame, not for each pointer event. Core has no window or background worker.

- `Auto` inspects a point/contained shape, then falls back to nearest-x line vertices.
- `NearestPoint` tests point centers within the positive configured destination radius.
- `Containment` tests clipped rectangles, wick/rule stroke widths and explicit custom
  geometry. It does not advertise polygon-interior selection for arbitrary filled paths.
- `NearestX` uses sorted presented x groups per clip. Only one exact x group is returned;
  equal-distance x groups choose the later-painted group. Results cap at 1..128 targets.
  No interpolation or invented source row fills a line gap.

Visible paint groups take priority; nearest distance chooses within a group and later
paint order breaks exact ties. This respects a later volume bar obscuring an earlier
candle wick. Inset paint groups follow actual scene order. Custom hit geometry retains
its declared semantic values and selection policy. Editable overlay/handle arbitration
is added with those features in WP-17.

The spatial index is a packed median-split bounding tree. A query tests only intersecting
nodes/candidates; sorted-x lookup tests adjacent groups. `query_scan` is a linear reference
path for differential tests and timing comparisons. Work counters count candidate tests
and visited spatial nodes, not source computation. Overlapping shapes can still require
linear candidate work; the index is not a worst-case constant-time promise.

Keyboard traversal follows presented panel/series/order, honors custom keyboard order,
and deduplicates compound marks by semantic identity. A candle's wick/body share one
keyboard target. `restore_focus` finds the same epoch/layer/panel/source-or-derived
identity in a replacement frame; removed targets clear focus. It never preserves an old
numeric candidate offset. Source/aggregate/derived provenance remains attached to each
inspection result and resolves through its retained source snapshot.

The reducer caches target/capability lookup by exact scene Arc identity. Equal stamps
cannot borrow another scene's target set. Only acknowledged, frozen and active-gesture
scenes retain indexes; normal dispatch clones bounded state and shared index handles.

## Selection

`Inspector::select` requires the exact presented stamp and returns stable `MarkTarget`s.
It supports point, panel/series, horizontal/vertical range, rectangle and lasso queries.
Scatter centers and source line vertices use point-in-region tests. Bars/cells intersect
the brush; candle wicks participate as clipped segments. Custom polygons/rectangles are
clipped before intersection, while custom circular point geometry uses its center.
Disabled custom selection targets are excluded. A closed lasso uses even-odd interior
and inclusive boundary semantics with 3..4096 vertices.

Results deduplicate semantic identities. A configurable limit of 1..4096 rejects an
oversized selection in full. There is no silent truncation, filtering or source mutation.
The reducer supplies Replace/Add/Toggle/Remove/Clear; a preview contains the complete
proposed set. Native overlays show selected/focused/hovered targets without preparing
statistics or layout. Publication of live interaction overlays is integrated in WP-20.

## Typed windows and navigation

`Navigator` pins the presented axes. It produces a complete `AxisWindows` map, preserving
unrelated named axes. `AxisWindow` distinguishes numeric calculation-unit endpoints,
exact integer source timestamps, and inclusive first/last category identities.
`SetAxisWindows` and `GesturePreview::AxisWindows` use the existing viewport revision,
follow/history policy and controlled-state fences. Legacy x/y actions replace the primary
0/1 named windows; legacy preview masks them transiently and cancellation restores them.
Unrelated named axes are preserved. The optional `interaction.windows`
wire field is absent for legacy empty state; transient preview values remain unserialized.

Zoom factors greater than one zoom in about the pointer. Pan uses displacement from the
pinned gesture start. Region zoom preserves axis direction. Numeric/nonlinear navigation
uses declared linear/log/symlog coordinates; UTC keeps exact relative ticks and sessions
use compressed active duration. Invalid, collapsed/sub-tick or unrepresentable intervals
reject. Session extension beyond the supplied calendar rejects rather than inventing
sessions. Guide-only secondary axes navigate through their primary scale.

`ClampToDomain` preserves span inside training and caps zoom-out at the full domain.
`Extend` allows numeric/UTC viewports beyond training. Categories always clamp to their
finite catalog and round the visible count to at least one. Band/point `domain()` retains
the complete trained/authored catalog; `visible_domain()` exposes the window. Projection
omits other categories without changing rows, statistics or domain contributions.

A named window applies to every panel bound to that scale ID. The optional panel argument
chooses the presented conversion basis; it does not create a separate per-panel state
namespace. Insets retain their authored view policies. Full-domain publication clears
both legacy and named windows in its captured copy, leaving the user state unchanged.

## Native and portable producers

`ChartView` uses the common index, navigator and reducer. The default bindings are active
only in the relevant chart scope. Click focuses/selects; wheel/trackpad zoom requires
chart focus. Drag defaults to pan; hosts choose rectangle, x/y range, lasso or region zoom
with `set_drag_tool`. Shift adds and Command/Control toggles selection. Arrows traverse,
Space toggles the focused target, Escape cancels, and Home resets. Hosts can disable the
whole default mapping and submit their own origin-fenced actions. `Focusable` and
`next_gesture_id` allow host controls to cooperate with the same owner.

A drag pins the old scene and native element origin; resize/new data cannot silently
change its coordinates. Window capture-phase move/up listeners continue beyond the chart
bounds. A missing pressed button cancels as CaptureLost. Focus-out/deactivation callbacks
and observed focus transitions release preview ownership; disposal drops retained input.
The render-time focus check is necessary on the pinned GPUI version: native testing found
that an explicit window blur did not reliably deliver the focus-out callback alone.

Rust/Python/WASM `query()` consumes the same bounded `InputQuery` JSON. It requires the
presented stamp (or explicitly selected active gesture basis), returns hits/targets or
windows, and never mutates user state. The caller then dispatches common actions and
acknowledges any newly displayed scene with `present()`. Native elements and interpreter
objects do not enter core. All three runtimes consume
[the same input cases](../fixtures/interaction/cases.json).

## Validation boundary

[WP-16 evidence](evidence/wp-16-completion-2026-09-07.md) records executed tests, actual
hosts, native gestures, inspected images and the focused index benchmark. FIX-09/10's
selection/focus/gesture portions are covered here; full eviction/pinning reconciliation,
linked echoes and constrained annotation dragging close with WP-17/18. G3/G4 and the
PERF-01–05 release workloads remain open. Supplemental parity lanes retain their own
prerequisites and acceptance gates.
