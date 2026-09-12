# ADR-022: Bounded hierarchy topology, standalone layouts and retained history

Date: 9 September 2026. Status: ACCEPTED design for WP-H01; implementation and full
HIR-01–08 acceptance remain the following packages. Conforms to ADR-006/012 and the
[hierarchy plan](../impl_plans/d3-hierarchy-parity-plan.md).

## One core topology and operation boundary

`chart-core::hierarchy` owns an indexed immutable topology with explicit hierarchy and
node occurrence identities. Occurrence keys, parent-lookup labels and source keys are
separate concepts; duplicate leaf labels and anonymous leaves are valid when no ambiguous
parent lookup is required. Parent tables can be arbitrarily deep within their explicit
node/depth/work limits; decoder nesting limits apply to nested JSON syntax only.
Construction, traversals, validation and copy use iterative worklists. Native payloads
are shared immutable values. Child/value/comparator/layout callbacks receive typed node
context and return checked results, with registered versioned equivalents for portability.
Trusted Rust callbacks are not preemptible; their invocation/output budgets are enforced.

Nested/custom iterable children, ordered grouped entries, direct keyed topology, ID/parent
stratification and path imputation normalize into this owner. Path delimiters follow the
pinned escaped-slash algorithm and have no filesystem meaning. Synthetic ancestors have
null payload and derived provenance. Stable source occurrence keys must be unique even
where D3 permits repeated leaf lookup labels. Cross-hierarchy node queries reject.
Subtree copy creates independent topology/root context while sharing payloads and preserving
aggregated values; geometry and resquarify history are not copied implicitly.

Node operations remain explicit: construction does not sum; sum includes the node's own
value; count counts leaves; sorting is stable. Missing weights default to zero only under
the explicit missing-as-zero policy. Negative/non-finite weights, invalid extents/radii,
overflow and invalid custom output reject atomically. Finite nonnegative weights are the
common compatibility domain. Zero totals remain valid topology. Degenerate weighted
layouts retain finite collapsed geometry; fitted all-zero packing and zero fitting extent
use deterministic collapsed centers/radii instead of D3's non-finite coordinates.
Non-finite reference results are recorded as tagged numbers, never admitted to scenes.
Empty sibling/enclosure helpers return empty placement/no enclosure. Finite ratio inputs
clamp to at least one; non-finite ratios reject. Excess padding follows reference midpoint
collapse when arithmetic remains finite. Output sizes/spacings are finite and nonnegative;
finite custom separation and signed padding are checked with final-geometry validation.

## Algorithms and explicit state

One implementation of tidy tree, cluster, partition, six treemap tilers, hierarchical
packing, sibling placement and enclosure serves standalone and chart consumers. Algorithms
ported from D3 retain its ISC notice and pinned source attribution. Layout outputs are
owned immutable numeric values. Node-size/extent configuration is mutually exclusive;
callbacks and constant controls have explicit readback/reset equivalents. Rectangles and
circles keep layout units, with no accidental scale retraining or area-size encoding.
Each operation bounds node visits and algorithmic work, including packing candidates and
path imputation, and validates final finite coordinates before publishing output.

Resquarify history is an explicit owner, keyed by hierarchy identity, ordered child
occurrences and ratio. Compatible weight/size changes reuse row membership. Any topology
change, explicit sorting, ratio change or new hierarchy resets affected state; a conservative
whole-history reset is permitted and documented. Reset compares with a fresh reference
layout, while compatible updates compare with equivalent retained reference history.
Failed operations leave both prior history and prior result unchanged. Immutable snapshots
retain resolved geometry; exporting never consults the next mutable history.

## Portable and chart integration

A separate versioned standalone hierarchy envelope follows ADR-006 decimal-string u64
identity policy, explicit limits and shared diagnostic ownership. It does not advance the
chart envelope until chart grammar actually consumes hierarchy. Python/WASM adapters
forward operations to Rust, own their inputs/results, reject disposed handles and release
history on disposal. No per-node interpreter callbacks or browser objects enter core.
Registered operations follow the existing explicit immutable registry; native-only
operations reject serialization and unknown IDs reject resolution.

Preparation owns topology/filter/aggregation/order; destination panel layout owns numeric
extent. Recipes consume these same outputs through existing geometry/shape/scene paths.
Radial links and sunburst arcs use the shared shape modules. Source filtering that breaks
parent links rejects unless an explicit subtree-prune transform was selected. Zoom and
highlight do not alter aggregation. Synthetic parents and links cannot impersonate source
rows. Nested hits, annular holes, keyboard inspection, facets/themes and immutable exports
must be proved through existing scene/provenance/revision contracts in H07/H08.

The [reference catalog](../../fixtures/hierarchy/README.md) owns pinned fixtures and
algorithm-specific tolerances. This decision does not close any layout/runtime gate.

## H07 implementation boundary (10 September 2026)

Standalone `HierarchySession` version 1 captures the registry, topology, active layout
configuration, exact numeric geometry and compact history rows. Its strict snapshot decoder
checks topology, metadata, geometry and history together. `copy` retains independent history;
`copy_subtree` supplies a new owner and clears layout/history. Rust native callback traits and
registered operations share the same checked kernels. Flat native topology is independent of
the portable JSON depth/byte limits.

Primary chart envelopes containing a `HierarchyRecipe` use version 15. Scene envelopes with
hierarchy targets use version 15 and a separately versioned `hierarchies` metadata table.
Ordinary chart/scene outputs retain their existing versions. The topology source is an
ID/parent table or path field; construction precedes explicit sum/count/order. All source
selectors resolve to stable fields at build time. Source integers outside the exact f64
range reject numeric summation. Standalone payload accessors retain their explicit policies.

`PreparedHierarchy` stores one preorder source-key array. Each node target shares that array
and retains a half-open subtree range, so a chain has linear membership storage. Source-backed
node identity uses the caller's row key; imputed nodes use the normalized path. Reparenting
preserves node selection and changes the link identity. Source eviction removes active
selection; historical inspection cannot restore an evicted row. Links remain derived targets.
Semantic and scene metadata serialize the source-key table once per layer/panel scope.

Panel/inset layout fits the selected extent, while fixed node spacing remains in destination
units with the root centered across the plot. Cartesian/radial links and partition arcs use
existing shape kernels; explicit pack radii are not area-scaled. Painted paths retain exact
path containment, including annular holes. Keyboard inspection visits nodes before links and
includes label, depth, height, value and source-member count. Zero-area nodes remain structural
metadata even when no primitive can be painted.

Stateful chart history is isolated by panel/inset/layer scope. Native resize and current or
presented export seed history from the acknowledged frame. `FigureRequest::with_hierarchy_history`
(and the same Python/WASM method) supports explicit standalone continuation. It retains compact
rows rather than old scenes/sources. Ordinary requests remain fresh; displayed capture keeps
its exact immutable primitives. Facets, registered examples and reparent/failed-update tests
are recorded in the H07 evidence. Full integrated acceptance remains H08.

## H08 control and acceptance boundary (10 September 2026)

Treemap per-side scalar accessors are compiled once into the captured registry. Specific
side selection overrides global padding, which overrides numeric options. Outer padding
assigns four independent outer-side selections; it does not create a second layout engine.
Normalized configuration and the effective clamped ratio are queryable independently of
geometry. Native-only operations remain valid for native Rust compilation/layout but reject
portable host construction and publication. Constructor options are immutable caller-owned
values; session snapshots retain the resulting topology rather than mutable factory functions.

The [final acceptance report](../evidence/phase-2-hierarchy-integration-2026-09-10.md)
and per-method catalog close H01–08/G-HIERARCHY for the declared finite typed surface.
The component resource protocol is a bounded-work and ownership handoff to WP-22.
Allocation-event/native ingest-to-present instrumentation and PERF-01–05 remain that
separate release gate; the high-fanout query workload is an explicit optimization follow-up.
