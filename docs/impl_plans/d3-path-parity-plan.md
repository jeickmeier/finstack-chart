# D3 path parity: gap review and implementation plan

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Project contract: 0.3.0. Status: planning complete;
implementation and parity acceptance remain open.
Reviewed revision: `1cb955740c2dad2607b0a2330201125294cab5d0` plus the live working tree.
Existing action/state/binding work and shape/scale/axis plans are preserved.

## Scope and reference

The target is [d3-path](https://d3js.org/d3-path), specifically **d3-path 3.1.0**:
standalone Canvas-style path construction and SVG path-data serialization. The
[export list](https://github.com/d3/d3-path/blob/v3.1.0/src/index.js) includes
`Path`, `path` and `pathRound`. The
[implementation](https://github.com/d3/d3-path/blob/v3.1.0/src/path.js),
[upstream tests](https://github.com/d3/d3-path/blob/v3.1.0/test/path-test.js) and
[manifest](https://github.com/d3/d3-path/blob/v3.1.0/package.json) establish the
behavioral baseline and ISC provenance. The umbrella D3 version is not this package's
version. Reference JavaScript is development tooling, never a core runtime dependency.

This covers all eight drawing methods and string output, not the complete modern
Canvas API. Ellipse/roundRect, SVG parsing, path length/point-at-length, boolean geometry,
stroke styling, animation and d3-shape generators are separate capabilities. Shape
generators remain required by SHP-01–10; they consume this foundation.

The [specification](../spec/gpui-charts-specification.md) owns PTH-01–06,
FIX-P01–06 and G-PATH. This document owns the detailed inventory and delivery sequence;
the [main plan](gpui-charts-implementation-plan.md) owns integration and the
[ledger](../implementation-status.md) owns completion evidence. This is a documentation
assignment; no implementation, fixture, dependency or wire-version change is included.

## Current implementation assessment

Pass is limited to the stated existing capability. Fail means absent or demonstrably
incompatible behavior. Uncertain means the required execution evidence is missing.

| Surface / requirement | Verdict and source evidence | Gap and impact |
| --- | --- | --- |
| Numeric line/Bézier transport; PTH-02 | Pass, foundation only: [scene.rs](../../crates/chart-core/src/scene.rs), `PathCommand` at lines 40–53. [native.rs](../../crates/gpui-charts/src/native.rs), lines 486–501, and [svg.rs](../../crates/chart-export/src/svg.rs), lines 279–295, dispatch these commands. | Move, line, quadratic, cubic and close exist. This is not a public stateful D3 path API. |
| Standalone builder, context and serialization; PTH-01/04 | Fail: [core exports](../../crates/chart-core/src/lib.rs); SVG command writer is private to chart-export. | No public `Path`/`path`/`pathRound` equivalent, shared external sink, standalone `toString` or configurable digits. Requiring a chart/publication to serialize a path is insufficient. |
| State/subpaths; PTH-02 | Fail: [scene.rs](../../crates/chart-core/src/scene.rs), validation at lines 431–467; [contracts.rs](../../crates/chart-core/tests/contracts.rs), lines 381–412. | Scene validation rejects empty/move-only paths, repeated close and drawing after close. D3's `moveTo(0,0); lineTo(10,0); closePath(); lineTo(20,0)` produces a continuing path. Directly exposing the scene validator as the builder would lose this capability. |
| Circular and tangent arcs; PTH-03 | Fail: `PathCommand` and native/SVG dispatch above have no arc command or builder operation. | Both `arc` and `arcTo` are absent. Circles as point primitives do not supply arbitrary sweeps, tangent joins or compound paths. |
| Rectangle subpaths; PTH-02 | Fail: [geometry.rs](../../crates/chart-core/src/geometry.rs), `Rect::new` at lines 43–54, rejects negative extents. | A positive standalone rectangle primitive is not D3's signed `rect` subpath; it cannot preserve reversed winding, current-point state or compound-path use. Do not change layout Rect semantics to implement this. |
| SVG precision; PTH-04 | Fail: `write_command` above writes Rust display numbers, with no rounding policy. | No rounded/default-three-digit/unrounded route or D3 tie behavior. Native geometry must remain full precision when SVG digits change. |
| Rendering and host proofs; PTH-05/06 | Uncertain for the existing subset; Fail for absent APIs: [portable](../../crates/chart-core/src/portable/wire.rs), [Python](../../crates/chart-python/src/lib.rs), [WASM](../../crates/chart-wasm/src/lib.rs). | No standalone path request/result proof across actual runtimes. Existing scene/export tests do not certify complete path behavior. |

The first five missing surfaces are P1 parity blockers. The representation/state
boundary is also a P1 integration risk: adding arc syntax alone does not fix continuation
or empty paths. Two focused existing tests passed during this review; they confirm the
current contract, including its deliberate scene restrictions, rather than D3 parity.

## Required feature inventory

All rows need public Rust access and a portable operation/result equivalent invoking
the same core implementation. Rust names and checked return types may differ.

| Reference surface | Required behavior | Requirement / fixture |
| --- | --- | --- |
| `path()` | Fresh independent empty builder; unrounded SVG output. | PTH-01 / FIX-P01 |
| `new Path(digits?)` | Public owned builder equivalent, unrounded when digits is omitted; optional precision shares the same implementation as the factories. JS prototype identity is a documented language adaptation. | PTH-01/04 / FIX-P01/04 |
| `pathRound(digits = 3)` | Fractional-digit limit, default 3. Finite nonnegative digits are floored; values above 15 select unrounded output. Reject negative/invalid digits; document typed handling of JS null/coercion/Infinity separately. | PTH-04 / FIX-P04 |
| `moveTo(x,y)` | Start a new subpath and update start/current point; preserve multiple and move-only subpaths in authoring/output. | PTH-02 / FIX-P01 |
| `closePath()` | Empty no-op; close to the active start; repeated close and subsequent drawing match pinned observable behavior. | PTH-02 / FIX-P01 |
| `lineTo(x,y)` | Append a straight segment and advance current point. | PTH-02 / FIX-P01 |
| `quadraticCurveTo(cpx,cpy,x,y)` | Preserve one control and endpoint; advance to endpoint. | PTH-02 / FIX-P01 |
| `bezierCurveTo(cp1x,cp1y,cp2x,cp2y,x,y)` | Preserve both controls and endpoint in order; advance to endpoint. | PTH-02 / FIX-P01 |
| `arcTo(x1,y1,x2,y2,r)` | Tangent circular arc; optional line to first tangent; current point becomes second tangent. Cover empty state, coincident points, collinearity, zero/negative radii and large radii extending beyond the supplied segments. | PTH-03 / FIX-P02 |
| `arc(x,y,r,a0,a1,anticlockwise=false)` | Radian angles; zero angle on positive x; default clockwise in y-down coordinates. Initial move/connecting line, both sweep directions, wrapping, large-arc flags, full circles split into two arcs, and degenerate/negative radii follow the reference. | PTH-03 / FIX-P02 |
| `rect(x,y,w,h)` | New closed subpath with signed or zero dimensions; winding and continuation from `(x,y)` retained. | PTH-02 / FIX-P03 |
| `toString()` | Owned SVG path-data result; empty string for empty builder; non-destructive repeated reads; later appends do not mutate prior results. Shared output policy, no figure/fonts/host required. | PTH-01/04 / FIX-P01/04 |

## Compatibility and architecture decisions

1. **One core path implementation.** Add a focused `chart-core::path` module (proposed
   name) containing the builder, numeric buffer, sink/replay contract and path-data
   formatter. Reuse `PathCommand` and existing limits/diagnostics; introduce the smallest
   arc representation that retains circular geometry until destination lowering.
   Do not add a separate crate or a second shape-specific builder. The exporter still
   owns SVG documents, resources and PDF/PNG encoding.
2. **Authoring state differs from submitted scene validation.** Keep start/current
   state at full precision. Record the pinned serializer's behavior for every call
   sequence, including empty, implicit starts and post-close continuation. Normalize
   numeric commands for the scene boundary, preserving closed joins and continuation
   rather than blindly splitting a stroke at every close. If the existing command
   model cannot preserve cap/join semantics, extend it with an explicit compatibility
   migration; do not simply weaken the old validator or remove its tests. Empty and
   move-only outputs remain valid builder results; conversion must preserve any
   paint-relevant degenerate behavior and omit only demonstrably nonpainting results.
3. **D3 is not a full Canvas validator.** For example, an initial `lineTo(1,2)` serializes
   as `L1,2`, which is not a valid standalone SVG path. The standalone compatibility
   serializer retains that observable result; scene submission reports an explicit
   invalid-path diagnostic. Do not silently prepend a move and claim identical output.
   Implicit `arc`/`arcTo` starts and their subsequent closes have source-specific state
   behavior: fixture whole sequences instead of assuming textbook Canvas state rules.
4. **Finite typed inputs and atomic errors.** Keep Point/Rect safety and DAT-05.
   Reject non-finite inputs, invalid radii, arithmetic overflow, unrepresentable
   destination coordinates and exhausted budgets with structured diagnostics, leaving
   builder state unchanged. JS coercion/truthiness and invalid/non-finite SVG output
   are explicitly excluded language/safety adaptations, not missing geometry. External
   sink failure semantics must be documented; our owned buffer must not partially commit.
5. **Precision is an output policy.** Share unrounded and rounded formatting with
   SHP-01; standalone `path` defaults to unrounded while shape generators and
   `pathRound` default to three digits. Preserve negative-tie behavior, signed zero,
   small/large exponent values and >15-digit fallback. In particular, rounding at zero
   digits maps `(1.5,-1.5)` to `(2,-1)`; Rust's usual away-from-zero tie rounding is
   insufficient. State, replay, native paint, hit geometry and snapshot coordinates
   must not change when digits change. Existing publication defaults remain explicit.
6. **Geometry and spelling have separate acceptance.** Match command topology, sweep,
   state effects and rounded numeric values. Equivalent SVG whitespace or expanded
   rectangle commands are permitted and documented; universal byte identity with JS
   is not required. Compare exact strings for canonical formatting cases; compare
   unrounded trigonometric coordinates with justified f64 tolerances. Invalid-sequence
   diagnostics and supported serializer-only results get their own cases.
7. **Retain arcs until lowering.** Preserve analytic arcs for SVG. Native and PDF/PNG
   routes must use shared numerical geometry and an explicitly bounded destination
   error policy, including transforms, output scale and subdivision limits. Audit the
   current SVG-mediated PDF/PNG conversion in [encode.rs](../../crates/chart-export/src/encode.rs)
   before assuming parser arc approximation meets that policy. Do not parse SVG for
   normal native painting. Bounds, clipping, text-outline transforms and snapshot
   transforms must handle the new representation. Four cubics per circle is not an
   unconditional fidelity guarantee.
8. **Keep integration ownership explicit.** Path construction does not invent chart
   targets. [inspection.rs](../../crates/chart-core/src/inspection.rs), lines 188–202,
   currently consumes only move/line vertices; controls and subdivision points must
   never become source rows. PTH-05 proves primitive replay and supplied metadata
   preservation; SHP-09/WP-16 retain shape-aware hit/keyboard/selection ownership.
   Likewise, curved dashing is a renderer/style concern (current `dash_polyline` rejects
   curves), not a new method in d3-path. Keep that existing capability limitation visible.

WP-P01 records the representation, state normalization, number policy and any wire/API
migration in one conforming ADR, shared with WP-S01. Freeze signatures only after a
standalone Rust consumer, shape-style draw callback and portable request use them.

## Work packages and dependency order

These packages refine the path foundation already budgeted in WP-S01. They do not
authorize duplicate sink, arc or serializer work in the shape lane.

| Package | Prerequisites | Owned work and acceptance | Provisional effort |
| --- | --- | --- | --- |
| WP-P01 — Contract and reference corpus | WP-14 | PTH-01–06 contract inventory, compatibility ADR and exact reference artifact/license/hash; fixtures and standalone replay format. Independently expected topology/geometry cases plus stored reference results; Rust tests must not install D3. This establishes the inputs and budgets for the remaining packages, not G-PATH. | 1–2 developer-days |
| WP-P02 — Checked builder and complete geometry | WP-P01 | Core path/sink state, move/line/quadratic/cubic/close, signed rectangles, arcs/arcTo, immutable results, limits and atomic errors. Minimal scene consumer exercises the chosen normalization. FIX-P01/02/03/05 pass, including operation sequences and threshold cases. | 3–5 developer-days |
| WP-P03 — Shared SVG path output | WP-P02 | Standalone `toString`, unrounded and pathRound policy; reuse from chart-export without changing legacy defaults implicitly. FIX-P04 passes; precision leaves numeric replay unchanged. Produce an actual representative SVG artifact. | 1–3 developer-days |
| WP-P04 — Renderers, portable APIs and acceptance | WP-P03 | Native replay, scene validation/transform/bounds/snapshot integration, SVG/PDF/PNG, portable operation/result DTO and actual Python/WASM calls; docs/examples. Inspect FIX-P06 artifacts and run the complete FIX-P01–06 corpus across runtimes; record G-PATH. | 4–6 developer-days |

Total **9–16 developer-days**, provisional and overlapping the shape foundation budget;
do not add this total wholesale to M-SHAPE. Re-estimate after WP-P01 resolves the
scene continuation and destination arc contracts. P04 includes path-only runtime and
visual evidence; full shape integration and performance certification stay in
WP-S08/WP-21/22/23. Actual code and support evidence, not an enum or compile, close a row.

WP-S01 now consumes WP-P04 and retains the shape export/method inventory, shape oracle,
custom lifecycle design and shape-specific examples/FIX-S01. Its path output checks
reference FIX-P01–06. This dependency is one-way: WP-P01–04 never depend on WP-S01.
WP-S02–08 retain their existing shape dependencies. WP-15–20 continue independently;
WP-21 and G4 additionally require G-PATH. Coordinate shared scene/portable edits with
the integration owner. No new `mise` task is added before its acceptance runner exists.

## Acceptance fixtures and evidence

| Fixture | Discriminating cases and required evidence |
| --- | --- |
| FIX-P01 — Constructors and state | All exports; independent builders; all basic drawing methods; exact control order; multiple subpaths; empty/move-only/zero-length output; repeated close; close then each drawing method; rect and implicit arc starts followed by close/arcTo; bare segment serializer versus invalid scene submission; repeated string reads and immutable prior results. |
| FIX-P02 — Arc geometry | Both directions; radians and wrap; sweeps at zero, pi, 2pi and beyond; just below/at/above the reference's `1e-6` and `2pi-1e-6` branches; zero/negative radius; implicit start/connecting line; two-arc circles. arcTo empty/coincident/collinear/nearly-collinear points, zero radius, large radius, tangent start coincidence, both turns and follow-up commands. Independently check tangency, radius and endpoints, not only D3 output. |
| FIX-P03 — Rectangles | Positive/negative/zero width and height, both-negative dimensions, compound paths, opposite-winding holes, continuation at rectangle origin. Zero extents must not be confused with an absent builder or forced positive dimensions. |
| FIX-P04 — Serialization | Unrounded defaults for path/Path; default-three-digit pathRound; digits 0/1/3/15/>15 and fractional flooring; invalid digits; positive/negative ties, negative zero, scientific notation, tiny/large finite magnitudes and overflow diagnostics. Assert numeric buffer/state equivalence across digit settings. A rounding-collapsed small arc must not corrupt subsequent geometry. |
| FIX-P05 — Validation and budgets | Each numeric argument non-finite; radius errors before mutation; finite inputs causing overflow; aggregate commands, emitted bytes, replay work and subdivision budgets; external sink error handling; valid operation after a rejected operation. Replayed operations versus one batch produce identical owned state/results. |
| FIX-P06 — Consumers and portability | Standalone and custom-sink examples; paths through supplied derived metadata, transforms, clips, winding holes and open/closed strokes; legacy text outlines and existing scene contracts. Compare numeric commands and serialization in actual Rust/Python/Node WASM; unknown versions, invalid requests, owned copies and disposal. Inspect native and SVG/PDF/PNG at publication scale and 300/600 DPI; vector arcs/curves, holes, bounds and snapshot independence must hold. |

The pinned implementation uses different predicates with the same numeric epsilon:
arc connections compare coordinate differences; arcTo also compares squared distance
and a cross product. Preserve those decisions on the compatibility route; one generic
distance epsilon would change topology. Independently test threshold boundaries and
document extreme finite cases that correctly trigger precision diagnostics.

Store operation sequences, constructor policy, expected strings/numeric geometry,
reference identity/hash/license and per-case compatibility status. Topology, command
flags and diagnostics compare exactly; use separate justified tolerances for f64
trigonometry, rounded serialization, destination lowering and raster evidence. Record
units and thresholds before acceptance, never a global epsilon. Fixture regeneration
is explicit and reviewed; preserve unexplained failures and existing visual baselines.

Use existing `mise run fmt`, `mise run check`, `mise run test` and
`mise run bindings-proof` after implementing their path coverage. macOS native evidence,
macOS/Linux core/export execution and actual WASM/Python proofs must be recorded on
their real environments. WP-22 measures command/byte growth, long-path append and replay,
many arcs/rectangles, subdivision limits and snapshot memory under its existing
protocol; no unmeasured throughput claim follows from this plan.

## Review evidence and next action

Environment: 7 September 2026, Darwin arm64, Rust 1.97.1, Node v24.14.0,
working directory `/Users/jeickmeier/Projects/finstack-chart`. Read the specification,
main/shape plans, ledger and source cited above. Retrieved pinned source/export/test/
manifest files; inspected upstream cases and executed 14 exploratory operation sequences
and two invalid-digit cases directly against upstream `src/path.js`. These were reference
observations, not a Rust differential suite or a run of the complete upstream tests.
The source SHA-256 was
`2c2d30e2c5279c8f6f42edc6731a4af029949ec83f47ffb48c3e35d21f877800`.
WP-P01 must retain a complete reproducible corpus and provenance in the repository;
temporary review downloads are not acceptance fixtures.

Observed examples: a rounded right-angle `arcTo` from `(0,0)` through `(10,0)` and
`(10,10)` at radius 2 reaches tangencies `(8,0)` and `(10,2)`; `rect(10,20,-5,4)`
retains signed winding; implicit arc/close/arcTo sequences differ from an explicit
move-start sequence. These observations drive FIX-P01–04.

Focused existing checks, both passing with **1 test, 0 failures** each:

```sh
mise exec -- cargo test -p chart-core --test contracts numeric_paths_preserve_segments_and_reject_invalid_subpath_order --locked
mise exec -- cargo test -p chart-export --test publication plain_text_xml_escaping_curves_and_empty_clips_are_supported --locked
```

The export check verifies existing encoding behavior, not visual fidelity (its curve
uses an empty clip). No native images, PDF/PNG inspection, fresh binding/runtime parity,
Linux execution or performance measurements were produced. Documentation validation
is recorded in the ledger. All PTH requirements and G-PATH remain open.
**Next implementation action: WP-P01**, then WP-P02 → WP-P03 → WP-P04; WP-S01 consumes
the resulting shared foundation.

## Delivered package evidence — 8 September 2026

WP-P01–04 and G-PATH are accepted for the [retained source snapshot](../evidence/phase-2-paths-2026-09-08.md).
The earlier review above describes the pre-implementation baseline. WP-S01 now consumes
the delivered builder, formatter and bounded lowering; shape and performance gates
remain open under their existing packages.
