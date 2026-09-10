# D3 axis feature parity

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Original review specification: 0.2.0; current contract: 0.5.0.
State: WP-AX01 and WP-AX02 are COMPLETE for their package scope, with [integrated evidence and validation limits](../evidence/phase-2-axis-ticks-2026-09-09.md). WP-AX03–06 remain unstarted and G-AXIS is open. The owner requested a stop after AX01/02. Full parity is not achieved.
Original reviewed revision: `1cb955740c2dad2607b0a2330201125294cab5d0` plus its working tree.
Primary API handoff reconciled at `b631f0e6d7e41722b5774433d616f704234157d7`
plus active authoring edits using the [current API review](../evidence/phase-2-current-api-review-2026-09-07.md).
Historical review scope was planning only. The current owner stop boundary is after WP-AX01/02; the status ledger records the remaining scope and acceptance.

## Outcome and scope

The existing axes are a useful foundation, but neither the previous requirements nor
the implementation cover the complete axis feature surface. Add AXIS-01–07 and FIX-19,
implemented by WP-AX01–06 below. The [specification](../spec/gpui-charts-specification.md)
owns requirements; this document owns the comparison, sequence and acceptance design.
The [main plan](gpui-charts-implementation-plan.md) owns integration order and the
[ledger](../implementation-status.md) owns current completion evidence.

The target is the observable axis capabilities of
[d3-axis](https://d3js.org/d3-axis), using
[d3-axis 3.0.0 source](https://github.com/d3/d3-axis/blob/v3.0.0/src/axis.js)
as the implementation reference. It includes static output, customization and animated
updates. It does not require a JavaScript selection API or a browser renderer in Rust.
Exported SVG must expose equivalent axis components; native and PDF/PNG consumers use
the same semantic components through portable scenes.

Separate axis behavior from the algorithms supplied by scales. Match automatic output
for our shared linear, positive-log, symlog, UTC, band and point families under matched
parameters. Provide a checked positional-scale/guide provider boundary for other
numeric-output mappings and supplied interval/format providers. Implementing every
d3-scale family belongs to the [scale parity plan](d3-scale-parity-plan.md); any narrower family coverage must accompany a parity
claim. Session scales and alternate-unit guides remain repository extensions.
Local time requires an explicitly supplied zone/calendar provider with DST evidence;
the process timezone must never become an implicit dependency.

Use an explicit D3-compatible axis profile so existing chart defaults and ADR-005
fixtures remain meaningful. The profile selects D3 tick policy, typography placement,
geometry defaults and preservation of authored ticks. It must be available identically
in Rust and the portable definition. Plain chart defaults may retain adaptive layout.
Document the profile in all parity claims; do not claim identical default output.

## Reference and implementation matrix

“Pass” below means the named capability exists in inspected source, not certified D3
equivalence. “Fail” means an observable mismatch or missing API; “Uncertain” means
additional execution/visual evidence is required. Line references describe the reviewed
revision and should be rechecked when implementation begins.

| D3 capability | Current evidence | Verdict and planned closure |
| --- | --- | --- |
| `axisTop`, `axisRight`, `axisBottom`, `axisLeft` | `layout/types.rs:20–35`; side-specific geometry in `layout/engine.rs:318–352` | Pass for orientation; exact anchors/baselines remain Uncertain. AXIS-04. |
| Render an axis at a translated origin; reuse a scale | `layout/types.rs:78–108` combines scale and guide identity; `layout/engine.rs:38,87–92` caps four axes and rejects repeated IDs/sides | Fail: cannot attach top and bottom guides to the same scale ID, or place two guides on one side. AXIS-01/04. |
| `scale` assignment and reading configuration | `AxisSpec.scale`, `range`, `viewport`; `ResolvedAxis.scale` | Pass for typed configuration; scale replacement/update equivalence remains Uncertain. AXIS-01/06. |
| `ticks` / `tickArguments` | Chart-wide `LayoutRequest.target_ticks` and `max_ticks` in `layout/types.rs:160–165`; UTC interval in `AxisScale` | Fail: no per-guide count/specifier request or general provider dispatch. AXIS-02/03. |
| `tickValues`, including reset to automatic | `CustomGuideTick` requires a label (`layout/types.rs:352–357`); `layout/axes.rs:349–394` resolves automatic candidates before replacing them | Fail: values and formatting are coupled, empty labels rejected, supplied order sorted. AXIS-02/03. |
| `tickFormat`, including reset and scale-derived precision | `typography.rs:262–302` supports fixed/scientific/percent with explicit precision; `layout/axes.rs:149–157,367–380` rejects time/category numeric formats and custom-label/format combinations | Fail: missing scale-aware automatic precision, SI/general formatting and time/custom formatter route. AXIS-03. |
| `tickSize`, `tickSizeInner`, `tickSizeOuter`, `tickPadding` | Global nonnegative tick length/gap (`layout/types.rs:156–165`, `layout/engine.rs:47–55`); domain is a straight rule (`engine.rs:386–410`) | Fail: no independent end caps, signed inner ticks or per-guide padding. AXIS-04. |
| `offset` and categorical positioning | No axis pixel-offset setting; `BandScale.center` in `scales/band.rs`; no band rounding option | Fail for offset/rounded-band behavior; basic category centers exist. AXIS-01/04. |
| Public domain/tick/line/text components and styling | `GuideTick` retains only position/label (`layout/types.rs:227–233`); guides emit untargeted ordinary primitives; `chart-export/src/svg.rs` serializes primitives | Fail: no stable axis component roles, component-level styling or SVG axis structure. AXIS-05. |
| Re-render and transition | Immutable relayout exists; no axis-specific transition identity/configuration in `AxisSpec` or `GuideTick`; SCN-04 previously made transitions optional | Uncertain for re-render parity; Fail for a complete transition contract. AXIS-06. |
| Scale interoperability | Closed `ResolvedScale` enum; numeric/log/UTC/category implementations exist | Partial foundation, Fail for general axis provider and reference tick algorithms. AXIS-01/02/03. |
| Rust/Python/WASM delivery | `ChartDefinition.axes` is serializable (`grammar/definition.rs:770–772`); historical binding proofs cover existing built-ins | Uncertain for new parity surface. Existing proof counts cannot certify absent settings. AXIS-07. |

Core paths in this table are relative to
[`crates/chart-core/src`](../../crates/chart-core/src); export paths are relative to
[`crates`](../../crates). Relevant existing decisions are
[ADR-005](../adr/005-foundational-scales-and-layout.md),
[scale/geometry](../scale-geometry-contract.md),
[typography/composition](../theme-typography-composition-contract.md), and
[extensions](../extension-contract.md).

## Findings that determine implementation order

1. **P1 — Explicit ticks can fail before their override is used (AXIS-02/03).**
   `layout/axes.rs:365` calls the complete automatic resolver first. A UTC guide with a
   dense automatic interval can exhaust its tick budget even when the caller supplies
   an empty or short explicit list. Choose candidate source before generation; validate
   the request but do not enumerate unused automatic ticks. A supplied value list must
   also support scale-generated labels independently from an override formatter.
2. **P1 — Label policy changes the tick geometry (AXIS-02/04).**
   `layout/engine.rs:414–429` clears the resolved ticks and discards entries for duplicate
   labels, collisions and bounds. Two distinct values formatted as “0” lose one tick;
   blank labels cannot express unlabeled minor ticks. Keep semantic ticks and tick rules
   separately from label visibility. The parity profile preserves supplied order,
   repeated/empty labels and ticks; adaptive thinning is an explicit policy with diagnostics.
3. **P1 — Baseline ignores the authored scale range (AXIS-04).**
   `layout/axes.rs:117–130` resolves a custom range, but `layout/engine.rs:386–398`
   draws the baseline across the plot. A scale using only the middle half of a panel
   therefore has a longer baseline than its represented extent. Derive baseline and
   end caps from the resolved range endpoints, including reversed ranges.
4. **P1 — Automatic ticks are not numerically equivalent (AXIS-02/03).**
   `scales/linear.rs:265–303` uses span divided by `target - 1` and a ceiling-style
   1/2/5 choice. For `[0,1]`, target 10, source analysis yields six ticks at step 0.2;
   the [D3 tick algorithm](https://github.com/d3/d3-array/blob/v3.2.4/src/ticks.js)
   yields eleven at step 0.1. `scales/nonlinear.rs:226–240` generates a transformed-space
   grid instead of the [logarithmic tick/label contract](https://d3js.org/d3-scale/log#log_ticks).
   Consume the scale lane's reference tick policy; retain existing-definition behavior.
5. **P2 — Time labels and category placement need their own proof (AXIS-02/03/04).**
   `scales/utc.rs:127–160,437–469` uses our interval chooser and ISO-style labels;
   [D3 time scales](https://d3js.org/d3-scale/time#time_ticks) use their own interval
   selection and context-sensitive formatting. Monday-week defaults must not masquerade
   as Sunday-week defaults. Band rounding/offset behavior needs narrow-band and reversed
   range cases, not only evenly spaced category screenshots.
6. **P1 — Completed packages do not cover the newly requested scope (AXIS-01–07).**
   WP-06/11–14 accepted our former contracts; no work package explicitly required the
   missing controls or axis transitions. Add a separate dependency lane and gate it before
   production certification. Do not retrospectively rewrite historical acceptance results.

These are source-backed counterexamples and missing contracts, not new reproducer-test
results. FIX-19 must turn them into executable evidence before marking them resolved.

## Shared design and compatibility rules

- **One scale engine, multiple guides.** Separate stable guide identity from `ScaleId`.
  Train/project each named scale once and let guides reference it. Preserve one-to-one
  secondary-unit validation. A checked provider supplies numeric positions, range
  endpoints, domain values, optional bandwidth/rounding, ticks and formatting; it need
  not claim inversion. Custom native providers need registered portable equivalents or
  explicit unsupported diagnostics. Keep output and callback work bounded.
- **Value selection precedes formatting.** Store per-guide tick arguments independently
  from an optional typed value list and optional formatter. `None` restores automatic
  selection; `Some([])` draws no ticks. Explicit values win over generated values;
  tick arguments still inform the scale formatter when no formatter override exists.
  Band/point fallback uses domain order and ignores count hints. Native accessors expose
  effective configuration; owned collections prevent accidental aliasing.
- **Formatter inputs are semantic values.** Preserve numeric values, integer timestamps
  and categories until formatting; never reconstruct values from painted coordinates.
  Support scale-default, declarative numeric/time specifiers with explicit locale/zone,
  explicit labels and registered formatter IDs. Native callbacks receive value, index
  and semantic tick-list context; browser DOM nodes are not portable formatter inputs.
  Export captures resolved labels or the registered formatter/resource identity.
  Reject unsupported descriptors rather than approximating them silently.
- **Compatibility is opt-in and complete.** The D3 profile uses empty tick arguments
  (scale defaults), inner/outer size 6, padding 3 and a 10-unit font request. Resolve
  its default offset to 0.5 at device scale 1 and 0 at higher device scale. With no
  device context, use 0.5, matching the reference's headless branch; physical exports
  capture the chosen offset in output units and never consult PNG DPI implicitly.
  Callers can set a finite explicit offset. Typography resources remain caller-supplied.
- **Geometry has independent controls.** Provide inner and outer lengths, combined size,
  padding, domain/tick/label visibility and separate axis placement. Signed lengths and
  padding are valid if finite. Label spacing is `max(inner, 0) + padding`. Outer caps
  are parts of the domain path, not additional semantic ticks. A negative inner length
  can form a grid across the panel. Keep its clip policy distinct from label overflow.
  Pixel offset changes guide geometry, not data-domain training or mark coordinates.
- **Exact authored ticks are a first-class layout policy.** Keep order and tick identity;
  do not deduplicate by label or silently thin at a resource cap. In preservation mode,
  overlaps are allowed/reported and overflow follows explicit clipping/placement policy.
  Adaptive label hiding and tick thinning remain separate choices. Hard budgets still
  reject excessive work; preserve the last valid scene on failure.
- **Expose components without duplicating renderers.** Portable axis metadata identifies
  the guide, domain path, tick group, tick line and label. Support whole-axis and per-tick
  style/visibility overrides. SVG text mode emits addressable `domain`/`tick` groups,
  line/text children and logical values; outline mode retains roles and logical labels
  while documenting its different text representation. Native/PDF/PNG consume resolved
  styles. CSS added after export affects that SVG only; it is not a headless theme API.
- **Transitions are required for full parity.** Host clocks drive common interpolation
  from immutable prior/current axis geometry. Store stable guide/value identities plus
  occurrence discrimination; lock duplicate/projected-position behavior against the
  reference joins. Handle entering, moving and exiting ticks, domain-path changes,
  orientation replacement, interruption and reduced motion. Capture displayed geometry
  and labels consistently for inspection and export. No entity per tick and no browser
  scheduler in core. Broader chart morphing remains outside this work.

## Ordered work packages

Implement one reviewable package at a time. Estimates are provisional engineer-days
including relevant tests, not delivery promises; re-estimate after the reference harness.
All names below are proposed API spellings. Acceptance fixes behavior, not spelling.

| Package | Prerequisites | Deliverables / owned areas | Acceptance and estimate |
| --- | --- | --- | --- |
| WP-AX01 — Guide contract and reference harness | WP-14 | AXIS-01/07, AUT-01/04: guide/scale identity split and bounded provider boundary in core; primary AxisBuilder/AxisHandle/name-map and portable migration; D3 profile coordinated with P2-00/GG-02; reference fixture generator and provenance manifest; conforming ADR updating ADR-005 limits/default-policy interpretation | Same scale drives independently configured top/bottom guides through primary Rust and actual host builders; two translated guides coexist; named axes, layer bindings, navigation and existing definitions migrate without changing meaning/output; reference inputs and expected outputs are reproducible. Historical 2–4 days; re-estimate remaining migration with AP owners. |
| WP-AX02 — Tick selection and formatting | WP-AX01, SP-03, SP-05, SP-06 | AXIS-02/03: per-guide arguments/values/formatters and canonical tick metadata; consume shared numeric/time formatting, tick policies and rounded-band inputs from the scale lane | FIX-19 selection/format cases, including explicit-list bypass, reset, empty labels and boundary cases; same labels/values in actual Rust/Python/WASM. 2–4 days. |
| WP-AX03 — Axis geometry and bounded layout | WP-AX02 | AXIS-04: range-derived domain paths, four-side anchors, inner/outer/padding/offset, translation, preservation/adaptive policies, facet/composition integration | FIX-19 independent geometry expectations and inspected SVG/native examples, custom/reversed ranges, negative ticks, crowded/repeated labels, DPI and narrow bands. 3–5 days. |
| WP-AX04 — Styling and publication components | WP-AX03 | AXIS-05: guide/tick roles, portable overrides, SVG component structure, native/export consumer integration, text/outline policy | Styled domain/ticks/labels independently; retained logical labels and vector output in inspected SVG/PDF/PNG; actual portable outputs agree. 2–4 days. |
| WP-AX05 — Axis updates and transitions | WP-AX04, WP-15, WP-19, WP-IP02, WP-IP05 | AXIS-06: core transition plans; host timing/reduced motion/lifecycle; stable enter/update/exit identities; coherent capture with WP-20 | Deterministic start/mid/end/interruption traces, update-versus-fresh final scene, stale job/disposal cases and actual native animation; explicit export-during-transition policy. 3–6 days. |
| WP-AX06 — Parity certification and documentation | WP-AX05, WP-20, SP-07 | AXIS-07: full FIX-19 reference matrix; Rust/Python/WASM runtime comparison, supported-platform/native/publication evidence, API migration/examples and capability matrix | Every matrix row has evidence and a verdict; no full-parity claim while a required row is Fail/Uncertain. Feed WP-21/22/23. 2–3 days. |

Initial incremental axis budget: **14–26 engineer-days**, additional to the earlier
project estimate and excluding work already owned by the scale parity lane and unresolved
host capability changes. Scale providers and formatters are shared prerequisites, not
duplicated axis engines or separately counted implementations.

WP-AX01 can proceed after WP-14 while coordinating with the active AP implementation.
The current `plot/axis.rs` AxisBuilder holds AxisSpec; AxisHandle wraps ScaleId, and
naming an axis assigns a scale identity. The split must migrate `Plot::named_axes`,
primary wire name maps, layer axis selection and host navigation alongside low-level
guide/layout types. Keep scale bindings distinct from decorative guide identity;
prove existing named-axis usage and a shared top/bottom pair through primary Rust and
actual Python/WASM operations, exports and declarations. AP-07 owns shared host syntax;
WP-AX01 owns this capability's conversions and behavioral cases. Available Rust
dispatch tests do not replace runtime host proofs. Pure contract/kernel work need not
wait for final G-AUTH acceptance.

Coordinate public identity/schema changes with AP-03/04/07, P2-00 and SP-01 before
landing. Use the common profile propagation/migration decision in GG-02 instead of
an axis-only policy envelope; do not add a dependency on GG-19 or final authoring
certification. SP-01 owns the
scale capability model; SP-03 owns categorical alignment/rounding; SP-05/06 own numeric
and time tick/format algorithms and timezone resources. WP-AX02 consumes these accepted
contracts and adds guide-level precedence/presentation. Share the reference dependency
lock and scale fixtures; the axis harness adds DOM/component/transition expectations.
Expand FIX-19 interoperability to applicable numeric-output families delivered by SP-07,
including negative-log and local-time cases, before full axis certification.
The [interpolation plan](d3-interpolate-parity-plan.md) owns shared numeric and transform
sampling. WP-AX05 consumes WP-IP02/05; it retains axis identity, scheduling and lifecycle
work. WP-AX06 does not wait for WP-IP07; interpolation certification consumes WP-AX06.
WP-19 may finish without
axis transitions, supplying scheduling to WP-AX05. WP-20 and WP-AX05 integrate coherently;
WP-AX06 waits for both and SP-07 scale interoperability evidence. WP-21 requires WP-AX06; WP-22 includes axis-heavy resize/update
work within existing performance workloads; WP-23 cannot close G4 with AXIS requirements
open. Do not reset historical WP-06/11–14 completion to hide or absorb this added scope.

## FIX-19 acceptance matrix

Reference generation uses exact locked npm dependencies in a development-only harness:
d3-axis 3.0.0, plus explicitly resolved/pinned d3-scale, d3-array, d3-format, d3-time,
d3-time-format, d3-selection, d3-transition and a DOM harness. Record licenses, source
revisions, resolved lockfile, generator command, device scale, locale/timezone and font
identity. The axis source pin is selected here; the transitive harness lock is still
to be created in WP-AX01. Store reference values, labels, order, geometry and roles so
normal Rust tests need no Node installation. Do not compare unequal domain policies.

| Case | Inputs and independent acceptance |
| --- | --- |
| FIX-19-A: API/configuration | All four sides; two guides sharing a scale; translated guides; scale replacement; read/reset settings; owned list copies; old-definition migration. Domain/stat results do not change when guide configuration changes. |
| FIX-19-B: precedence | Count and format hints; explicit `[1,2,3,5,8]`; empty list; reset; explicit formatter then reset; automatic generator that would exceed its budget. Explicit values bypass enumeration but retain scale-format arguments. |
| FIX-19-C: numeric families | `[0,1]` with count 10 gives eleven 0.1-spaced reference ticks; fractional/negative/reversed/large-origin domains; count zero/one/fractional hints with separate resource caps. Log `[1,100]`, base 2 and noninteger base; symlog through zero. Compare values and blank minor labels independently. Match expanded domains explicitly for constant-domain comparisons. |
| FIX-19-D: formatting | SI, grouped fixed with automatic precision, percent, scientific/general, custom value/index formatter, repeated and empty labels, numeric/time/category explicit values, explicit locale, unsupported IDs/specifiers. Equal strings must not merge tick identities. |
| FIX-19-E: time/category | Fifteen-minute interval, subsecond/month/year/leap-day boundaries, Sunday versus explicitly Monday weeks, UTC automatic multi-resolution labels; explicit-zone DST gaps/folds with supplied provider. Band/point domain fallback ignores count, reversed ranges, align/padding/rounding, bandwidth below twice offset and exact integer timestamps above 2^53. Compare D3 only in its representable time range; retain our wider exact-time invariants separately. |
| FIX-19-F: geometry | All sides with inner/outer 0/6, signed inner -40, padding 0/3/negative; offset 0/0.5 and device scale 1/2; custom/reversed range. At origin, bottom range `[0,100]`, offset 0, inner/outer 6, padding 3: baseline y=0, caps end y=6, tick tips y=6, label anchor y=9. Mirror signs for top/left; test vertical anchor separately. |
| FIX-19-G: layout/styles | Dense ticks, identical/blank labels, resize/tiny bounds, independent component visibility/stroke/font/color/dashes, per-tick style, facets/shared axes/secondary units, grid clipping, arbitrary placement. Preservation mode retains requested ticks; adaptive mode reports changes; excessive requests fail before unbounded callbacks. |
| FIX-19-H: updates | Same definition twice; changed domain/range/format/side; add/remove/reorder/duplicate ticks; animation at 0/50/100 percent, interruption, reduced motion, superseded jobs, dispose and export mid-transition. Final geometry matches fresh batch layout and captures use one presented revision. |
| FIX-19-I: hosts | Actual native views at device scale 1/2 where supported, deterministic SVG structure, text/outline modes, vector PDF, 300/600 DPI PNG inspection and actual Python/Node WASM execution. Unsupported host evidence stays open. |

Require exact semantic tick values where representable, exact integer timestamps, labels,
order, roles and reset/error behavior. Set justified operation-specific numeric/geometry
tolerances after measuring the reference; do not borrow a global epsilon. Text baselines
and advances compare under an identical supplied font/size; OS font rasterization is a
separate visual check. Preserve existing tests and baselines; any reference-policy output
changes require a migration fixture, not a fixture rewrite.

## Evidence from this planning review

Environment: repository root `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026); reviewed 7 September 2026.

- Read the linked D3 documentation and versioned axis/tick sources and traced the core
  axis/scale/layout/typography contracts, SVG writer and portable definition surface.
- `mise exec -- cargo test -p chart-core --test scales --test layout --locked`:
  **29 passed** (9 scale, 20 layout), zero failed. These lock existing behavior,
  including adaptive thinning; they are not D3 differential tests.
- Documentation/link and whitespace verification are recorded in the status ledger.

No D3 fixture generator, new production code, new runtime binding proof, native capture,
export inspection, Linux run or transition benchmark was produced in this planning task.
The next concrete action is **WP-AX01**, within a separately assigned implementation slice.

## WP-AX01 reference entry — 9 September 2026

The [entry evidence](../evidence/phase-2-axis-entry-2026-09-09.md) records 372 actual D3 browser cases and the complete public inventory, repeated byte for byte. The shared lock now includes pinned development-only selection/transition support; the prior lock remains hash-verified for historical oracle provenance. This is reference preparation only. Production guide/scale migration, provider resolution and actual host acceptance remain open.
