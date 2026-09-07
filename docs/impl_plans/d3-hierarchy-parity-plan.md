# D3 hierarchy parity — review and delivery plan

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Specification: 0.3.0. Review baseline: `1cb9557` plus
the existing working tree. Status: **planning complete; hierarchy parity absent**.
Scope: owner-requested hierarchy parity assessment and planning; no implementation,
fixture generation, dependency adoption or runtime certification in this task.

The [specification](../spec/gpui-charts-specification.md#52-d3-hierarchy-feature-parity)
owns HIR-01–08. The [main plan](gpui-charts-implementation-plan.md) owns release
dependencies and traceability; the [ledger](../implementation-status.md) owns execution
status. This document owns the hierarchy inventory, gap evidence and bounded packages.
Existing shape, scale, axis and action work retains its scope and evidence.

## Parity boundary and reference

Target the complete **d3-hierarchy 3.1.2** capability surface, including standalone
operations, configuration readback and custom accessors/tilers. The official
[module overview](https://d3js.org/d3-hierarchy) describes data operations and five
layout families. Its linked examples use other D3 modules to render the results;
hierarchy layout itself does not supply a renderer, interaction framework or clustering
algorithm. Force graphs, Sankey, geographic layouts, hierarchical clustering of raw
observations, DOM selections and general animation remain separate scope.

Reference evidence checked in this review:

- Official [hierarchy](https://d3js.org/d3-hierarchy/hierarchy),
  [stratify](https://d3js.org/d3-hierarchy/stratify),
  [tree](https://d3js.org/d3-hierarchy/tree),
  [cluster](https://d3js.org/d3-hierarchy/cluster),
  [partition](https://d3js.org/d3-hierarchy/partition),
  [pack](https://d3js.org/d3-hierarchy/pack) and
  [treemap](https://d3js.org/d3-hierarchy/treemap) documentation.
- Pinned [package identity/license](https://github.com/d3/d3-hierarchy/blob/v3.1.2/package.json)
  and [public exports](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/index.js).
  There are **16 named exports**, including `Node`, which the layout overview alone
  does not enumerate. D3's website-wide version is not this package's version.
- Pinned [node construction/copy](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/hierarchy/index.js),
  [path stratification](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/stratify.js),
  [treemap configuration](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/treemap/index.js),
  [resquarify state](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/treemap/resquarify.js),
  [pack](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/pack/index.js) and
  [enclosure helper](https://github.com/d3/d3-hierarchy/blob/v3.1.2/src/pack/enclose.js).

Rust APIs may use checked builders, iterators, immutable outputs and explicit state
instead of JavaScript mutation/prototypes/`this`. These adaptations must preserve
documented valid-input capabilities, ordering, defaults and outputs. Coercing arbitrary
JavaScript objects and allowing invalid paint coordinates are not parity requirements.
Every adaptation gets an explicit inventory row; an unsupported built-in or custom
operation cannot be declared equivalent merely because it returns a diagnostic.

## Required feature inventory

All rows currently **Fail — absent implementation**. These are capability gaps, not
newly reproduced numerical defects. Integrated renderer/runtime/performance evidence
is **Uncertain — not exercised**. A working rectangle or circle primitive does not
pass a layout row.

| Contract | Reference surface to deliver | Package |
| --- | --- | --- |
| HIR-01 | `hierarchy(data, children)` and exported `Node(data)` equivalent; nested children/iterables and ordered Map/group/rollup input adaptation; data, parent, ordered children, depth, height and optional value. Standalone node creation remains usable without a chart. | WP-H02 |
| HIR-01 | `stratify()` invocation plus `id`, `parentId`, `path` setters/readback/reset; ID/parent tables and slash-delimited paths with inferred ancestors. Match input order, missing leaf IDs, null/empty root-parent handling, path-mode precedence and structural errors. | WP-H02 |
| HIR-02 | `ancestors`, `descendants`, `leaves`, `find`, `path`, `links`; breadth-first iteration and `each`; pre-order `eachBefore`, post-order `eachAfter`; callback node/index/root context via Rust equivalents. | WP-H02 |
| HIR-02 | `sum`, `count`, `sort`, `copy`: internal own value participates in summation, count counts leaves, sibling sorting precedes layout, subtree copying rebuilds structure and shares immutable source payloads. | WP-H02 |
| HIR-03 | `tree` and `cluster`, invocation, `size`, `nodeSize`, `separation`; default unit extent, mutually exclusive extent/spacing modes, root origin in node-size mode, sibling/non-sibling separation and custom accessors. Output x/y supports Cartesian and radial projection. | WP-H03 |
| HIR-04 | `partition`, invocation, `size`, `round`, numeric `padding`; x0/y0/x1/y1 output with explicit aggregation/order; unit extent, no rounding/padding by default. | WP-H04 |
| HIR-05 | `pack`, invocation, `size`, `radius`, numeric/node-accessor `padding`; implicit value-derived radii versus explicit unscaled leaf radii, two-pass fitted padding, x/y/r output. `packSiblings` and `packEnclose` are independent public helpers. | WP-H06 |
| HIR-06 | `treemap`, invocation, `tile`, `size`, `round`, `padding`, `paddingInner`, `paddingOuter`, `paddingTop`, `paddingRight`, `paddingBottom`, `paddingLeft`; constant/accessor padding and custom tilers, unit extent and zero padding defaults. | WP-H05 |
| HIR-06 | Public `treemapBinary`, `treemapDice`, `treemapSlice`, `treemapSliceDice`, `treemapSquarify`, `treemapResquarify`; standalone parent/bounds invocation, squarify/resquarify `.ratio` factories, golden-ratio default and finite ratio clamping. Resquarify reuse is part of parity. | WP-H05 |
| HIR-07 | Core authoring and versioned portable access; node/link/rectangle/circle outputs and recipes for Cartesian/radial tree and dendrogram, icicle/sunburst, treemap and circle packing; target provenance, updates, themes/facets, inspection and immutable exports. | Each family, WP-H07 |
| HIR-08 | Export/method/default-level oracle coverage, independent expectations, actual Rust/Python/WASM operations, native and SVG/PDF/PNG inspection, bounded-work and measured release evidence. | WP-H01, WP-H08, WP-21/22/23 |

WP-H01 turns this inventory into a machine-readable method matrix, including every
setter/getter and reset behavior. It records the exact artifact integrity, source commit,
license notices, fixture generator environment and policy differences before code is
ported. D3 is a development oracle only; production computation stays in Rust and
normal Rust fixture tests consume stored expectations without Node installed.

## Source-backed gaps and fixes

Line references describe the inspected working tree and may move with active edits.

| Severity / verdict | Evidence and discriminating missing behavior | Required fix |
| --- | --- | --- |
| P1 / Fail — release scope | Before this change, SCP-03 explicitly deferred hierarchy/network algorithms; the migration scope table also deferred hierarchy. Original WP-01–23 had no hierarchy acceptance lane. A production release could satisfy those plans while shipping no treemap. | Promote HIR-01–08 into required scope, add FIX-H01 and G-HIERARCHY, and gate final certification. This task fixes the planning gap only. |
| P1 / Fail — data engine, HIR-01/02 | `crates/chart-core/src/lib.rs:46` exports the current modules; `grammar/definition.rs:175` enumerates statistics and `:233` defines transforms. None constructs a tree or exposes traversal. A three-node parent table cannot produce ancestors, depth or leaf counts. | Implement one checked core hierarchy representation and adapters; do not duplicate it in recipes or bindings. |
| P1 / Fail — layouts, HIR-03–06 | `grammar/definition.rs:443` provides the current geoms; `layout/mod.rs:1` and its modules resolve Cartesian panels. Repository-wide exact-name search found no hierarchy algorithms or fixtures in crates/examples/fixtures/scripts. Supplying already calculated x/y or rectangle endpoints only proves drawing. | Add standalone tree, cluster, partition, pack and treemap kernels with all listed controls/helpers. |
| P1 / Fail — stateful parity, HIR-06/07 | `portable/session.rs:8` retains definition/store/reducer/compiler state; no hierarchy topology or resquarify state exists. The reference caches rows of child nodes. Reconstructing nodes every frame would lose this behavior even if one static treemap matched. | Separate immutable topology/results from explicitly owned layout history; certify repeated value/size changes and reset behavior. |
| P1 / Fail — portable capability, HIR-07 | `portable/wire.rs:9`, `grammar/definition.rs:33` and the portable session expose chart/data contracts, without standalone hierarchy requests/results. Existing Python/WASM chart proofs cannot exercise these absent operations. | Extend the common versioned boundary, expose hierarchy methods/layouts in both proof adapters, and test actual executions. |
| P2 / Uncertain — integration, HIR-07/08 | `scene.rs:43` has numeric path commands; `provenance.rs:20` has source/aggregate/derived targets; `grammar/geometry_extensions.rs:1` validates custom paint and hits. These are reusable foundations, but no hierarchy scenes establish nested-hit precedence, parent membership, radial holes or node keyboard navigation. | Integrate through these contracts and the shared shape generators; inspect native and publication artifacts. |

## Implementation design and cross-package contracts

**Core ownership.** Start with a focused `chart-core::hierarchy` module, with topology,
operations and algorithms separated internally as they become real implementations.
No new crate or external runtime is justified. Use an indexed, bounded node store with
stable hierarchy/node keys and immutable source references; an array slot is not durable
identity. A hierarchy node occurrence is distinct from its source payload, so repeated
payloads can appear in different positions without aliasing topology. Use iterative
construction/traversal and explicit node/depth/work budgets; do not inherit the wire
decoder's JSON nesting limit as the maximum depth of a flat parent table.

**Construction and validation.** Nested, grouped and tabular inputs normalize into the
same engine. Preserve exact u64 identifiers through the existing decimal-string wire
policy. Domain IDs used to resolve parents are separate from stable node keys; anonymous
leaves still need durable handles. Path-generated parents carry derived provenance and
nullable payloads. Match escaped delimiters, leading/trailing separators, ancestor
imputation and redundant synthetic-root removal against the pinned source. A path is
data, never a filesystem operation. Diagnose missing/ambiguous parent references, cycles,
empty/no-root and multiple-root inputs before publishing results. Distinguish duplicate
source keys (always invalid) from reference label behavior; do not blanket-reject every
anonymous or duplicate leaf label in the compatibility adapter.

**Operations and numerical policy.** Keep input order unless callers explicitly sort.
Expose aggregation as its own operation before weighted layouts; no automatic leaf-only
sum. Missing/non-finite/negative weights use an explicit checked policy and diagnostics,
consistent with DAT-05; zero is valid. Freeze policies for all-zero trees, degenerate
extents, excessive padding, overflow and empty packing helpers in WP-H01. When reference
output is non-finite, document a finite degenerate result or recoverable error instead
of forwarding NaNs into scenes. Default D3 ordering and finite geometry must match on
the common valid domain. Copy preserves payload/value semantics and resets structural
root context; layout coordinates and private caches are not promised by `node.copy`.

**Layout versus presentation.** Topology, aggregation and sorting are preparation work.
Bounds-dependent layout follows destination panel allocation and precedes scene lowering;
resizing must not refilter data or rerun unrelated statistics. Expose typed node/link and
layout outputs without requiring axes or a window. Layout coordinates are not automatically
retrained as source-value scales. Recipes consume the same outputs. Use WP-S03 arcs for
sunbursts and WP-S04 radial projection/link generators for radial trees and dendrograms;
the hierarchy engine does not implement a second arc or link library. Icicle depth and
sunburst radius mappings are explicit recipe choices; publish the mapping and units.
Packing circle radius must retain its layout meaning instead of being decoded as the
area-size symbol encoding. Zero-area nodes remain queryable but need not emit paint.

**Extensibility and bindings.** Native children/value/comparator/separation/radius/padding
callbacks receive typed data/node context. Portable definitions use fields, constants,
ordered grouped entries, declarative comparator policies or registered versioned Rust
operations. Custom tilers receive bounded parent/children/bounds data and return checked
child rectangles; integrate with ADR-012 registration principles, without introducing a
general plugin runtime. At least one external custom tiler and accessor family must work
through native and registered portable paths. No per-node interpreter callbacks in the
core loop. Unknown registrations and native-only serialization fail explicitly. Keep
wire migrations coordinated with the other parity lanes under ADR-006; do not choose a
conflicting global envelope version independently. Schema version is distinct from this
document's version.

**State and identity.** Key a resquarify history by hierarchy identity, ordered child
topology and ratio. Value changes and compatible resize reuse rows; child insertion,
removal, reparenting, explicit resorting, ratio change or a new hierarchy invalidates the
affected history under a documented policy. D3's private object cache is not a portable
API to copy blindly. For preserved topology, compare the same operation sequence against
the reference. For topology edits, compare against a reference rebuild/reset. Stateless
layouts compare updates against a fresh batch. Stateful layouts compare equivalent
history, and explicit reset against a fresh layout; a fresh squarify result is not an
oracle for a history-preserving resquarify update.

Retain state and resolved geometry in immutable figure/presentation snapshots, or retain
sufficient immutable history to reproduce the same destination layout. Never export by
consulting a newer mutable cache. Define source-filter orphan handling explicitly (default
reject a broken parent table; subtree pruning is an explicit transform); zoom/highlight
does not alter aggregation. Reparenting preserves caller-stable node identity and updates
link identity; removal/retention follows DAT-06 selection policy. Parent aggregate targets
resolve actual membership; synthetic parents and links cannot impersonate source rows.
Use existing action/revision/inspection contracts, including nested hit precedence,
clipping, annular holes, keyboard order and last-valid-scene behavior. Collapse/drill-down
widgets and hierarchy morphing are not implied by d3-hierarchy parity.

WP-H01 records a conforming ADR for these representation, state, validation and portable
choices, using the next available ADR number. This plan does not adopt unimplemented
public type names or a new dependency.

## Work packages and dependency order

Estimates are additional developer effort including focused tests and review, **35–57
engineer-days** total. They exclude shared shape/scale/axis work and the existing final
WP-21/22/23 platform/performance/release effort. Re-estimate after WP-H01/H02; these are
planning ranges, not measured throughput or calendar promises.

| Package / effort | Prerequisites | Deliverable and exit evidence | Owned areas |
| --- | --- | --- | --- |
| WP-H01 — Contract and reference harness / 3–5 days | WP-14 | ADR, all 16 exports/method/default inventory, compatibility policies, pinned generator manifest, stored independently checked seed cases for FIX-H01. No family closes from this harness alone. | Hierarchy contract/ADR, planned `fixtures/hierarchy/` and hierarchy proof scripts |
| WP-H02 — Topology, stratification and operations / 6–9 days | WP-H01 | Nested/grouped/table/path constructors, standalone node, all traversal/query/aggregation/sort/copy methods, stable identity and bounded diagnostics; FIX-H01-A/B pass. | Core hierarchy topology/operations, data/provenance adapters, focused tests |
| WP-H03 — Tidy tree and cluster / 4–6 days | WP-H02 | Both standalone algorithms, extent/node-size modes, default/custom separation and deterministic coordinate outputs; FIX-H01-C passes. No radial renderer dependency for numerical kernel. | Core tree/cluster modules and tests |
| WP-H04 — Partition / 2–3 days | WP-H02 | Weighted rectangles, size/round/padding and internal-value behavior; FIX-H01-D passes. | Core partition module and tests |
| WP-H05 — Treemap and tilers / 6–10 days | WP-H02 | Six standalone tilers, all layout/padding/ratio controls, custom tiler protocol and explicit resquarify history/reset; FIX-H01-E passes, including stateful traces. | Core treemap modules, history contract and tests |
| WP-H06 — Packing and helpers / 5–8 days | WP-H02 | Hierarchical pack, radius/padding modes, sibling placement and minimum enclosure helpers; deterministic reference comparisons plus geometry invariants; FIX-H01-F passes. | Core packing modules and tests |
| WP-H07 — Grammar, portable API and presentation / 6–10 days | WP-H03/04/05/06, WP-S03/04, WP-16, WP-18 | Common pipeline and recipes, versioned standalone and chart adapters, identity/actions/updates, registered custom examples; FIX-H01-G across actual Rust/Python/WASM; inspect native and SVG/PDF/PNG examples. | Compiler/layout/scene/provenance integration, bindings, export, gallery and docs |
| WP-H08 — Integrated parity acceptance / 3–6 days | WP-H07, WP-19, WP-20 | Complete per-method Pass/Fail/Uncertain matrix; FIX-H01-A–H, coherent history/export/replay and disposal proofs, inspected supported artifacts; hand off benchmark workloads. G-HIERARCHY passes only with complete evidence. | Proof runners, integration fixtures, support matrix and evidence ledger |

The numerical packages can proceed independently after WP-H02 when separately assigned;
the default remains one bounded slice at a time. WP-S04 must not depend on hierarchy
layout, avoiding a dependency cycle. No dependence on completion of scale/axis parity is
needed to calculate hierarchy coordinates. WP-H07 consumes shared scene/shape interfaces;
WP-H08 must finish before WP-21 and WP-22 acceptance. WP-23 then documents the certified
surface. Preserve original G2 and WP-11–14 acceptance as evidence of their earlier scope.

## Acceptance catalog — FIX-H01

The cases below are **planned**, not fixtures or results already produced. Every oracle
record stores inputs, operation sequence, order, parameters, IDs, expected topology and
coordinates, reference version/hash and applicable tolerance. Compare integers, traversal
order, identity, errors and topology exactly. Set numerical tolerances per algorithm and
coordinate magnitude before accepting results; independently check conservation,
containment and geometric residuals. Packing's reference geometric slack is not a global
epsilon to reuse elsewhere. A stable screenshot cannot establish numerical parity.

| Subset | Discriminating cases and expected evidence |
| --- | --- |
| FIX-H01-A — Construction | Nested children, custom iterable accessors, ordered grouped/rollup equivalent, shuffled parent rows and anonymous leaves; empty input, missing/ambiguous parents, multiple roots and disconnected cycles. Path imputation, escaped slash, trailing slash and common-prefix root handling. Exact IDs above 2^53 and synthetic-parent provenance. Iterative deep-chain and wide-tree budget boundaries. |
| FIX-H01-B — Operations | For R with children A/B and A with C/D: breadth-first R,A,B,C,D; pre-order R,A,C,D,B; post-order C,D,A,B,R; leaves C,D,B. Counts R=3,A=2,B=C=D=1. Own values R=5,A=2,B=3,C=4,D=6 sum to R=20,A=12. C-to-B path C,A,R,B; ancestors, links, first-match find, no-match and cross-tree errors. Stable sort, copy subtree root depth/parent/height/value, shared payload but independent topology and no copied layout cache. |
| FIX-H01-C — Tree/cluster | Balanced, asymmetric, singleton, chain and wide trees; leaves at unequal input depths distinguish tidy tree from cluster's common leaf level. Extent versus node spacing, root at origin, switching/readback modes, default/custom/depth-aware separation and sort ties. Compare full coordinates and sibling order; project cardinal radial directions separately. |
| FIX-H01-D — Partition | Two leaves of weights 1 and 3 in size [8,4] with no padding: root [0,0,8,2], children [0,2,2,4] and [2,2,8,4]. Own internal value must not be silently renormalized away. Rounding, padding collapse, zero totals, single node, extreme magnitudes, resize and explicit sunburst projection. |
| FIX-H01-E — Treemap | Every tiler in wide/tall/square/degenerate bounds; weights 1/3 under dice in [0,0,8,4] give widths 2/6, under slice give heights 1/3. Check internal-value slack per tiler rather than asserting every tiler fills by the same rule. All padding controls and accessor invocation, default/custom/clamped ratios, rounding and extreme/zero weights. Custom tiler failure leaves prior output intact. Repeated resquarify value/size sequences, reset, ratio switch, reorder/insert/remove/reparent and retained old snapshots. |
| FIX-H01-F — Packing | Empty/single/two/three circle helpers; enclosing circles of radius 1 at (0,0) and (4,0) yields center (2,0), radius 3. Coincident/contained/tangent/nearly collinear and highly unequal radii; non-overlap and parent containment. Default fitted radii versus exact custom radii, numeric/accessor padding, zero total and overflow diagnostics. Check deterministic repeated calls and pinned coordinate results, not just a plausibly packed picture. |
| FIX-H01-G — Public surface | Execute standalone operations and every layout in Rust, Python and real Node WASM; registered custom accessors/comparator/separation/padding/radius/tiler, native-only and unknown-registration errors. Version/readback/roundtrip, large IDs, copies, disposal and memory-growth ownership. Same recipes through shared engine; native plus SVG/PDF/PNG artifacts with fonts, themes/facets, nested targets, radial seam/hole misses and keyboard inspection. |
| FIX-H01-H — Updates/resources | Stateless update-versus-batch and stateful equivalent-history comparisons; atomic invalid topology changes, source filtering versus zoom, retention, reparent/selection semantics, coherent presentation and export during ingestion, replay and old-snapshot isolation. Measure node/depth/path/geometry/membership/cache budgets and release after disposal; unbounded subtree membership copies are unacceptable. |

WP-H08 defines reproducible balanced-tree, long-chain, high-fanout, skewed-weight pack
and resquarify update/resize workloads for WP-22 under ADR-008. Record node/leaf/depth,
rendered mark/link counts, cold/warm layout, preparation, hit-index and export time,
allocations, cache/snapshot memory and ingest-to-present latency at several scales within
declared budgets. Set workload-specific acceptance budgets before measurement. Preserve
PERF-01–05; do not infer pack scalability from tidy-tree complexity or count numerical
layout throughput as native frame-rate evidence. Full recomputation is acceptable until
a specialized update path has independent equivalence and measured benefit.

## Review evidence and next action

Read the specification, implementation plan, ledger, local development instructions,
ADR-006/012 and the live core/grammar/layout/scene/provenance/portable boundaries. Exact
hierarchy-name searches across crates/examples/fixtures/scripts found no implementation
or hierarchy proof fixtures. Ignore unrelated `BTreeMap` matches from substring searches.
The pinned export list, resquarify and enclosure sources were retrieved with read-only
`curl -fsSL` requests after the browser source fetches were incomplete.

Environment: 7 September 2026, Darwin arm64,
`/Users/jeickmeier/Projects/finstack-chart`, starting revision `1cb9557`; pre-existing
uncommitted source and documentation changes were present. This task changes only this
plan and the specification/main-plan/migration/ledger links and requirements. Documentation
check outcomes are recorded in the ledger. No Rust feature tests, D3 differential runner,
binding runtimes, native/export visuals, Linux run or performance benchmark ran for this
planning task. **Next within the hierarchy assignment: WP-H01**, then WP-H02. Existing
action work continues under its separate assignment.
