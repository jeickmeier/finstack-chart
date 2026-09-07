# D3 shape parity: gap review and implementation plan

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Project contract: 0.2.0. Status: planning complete;
implementation and parity acceptance remain open.
Reviewed checkout: `1cb955740c2dad2607b0a2330201125294cab5d0`, including the live
working tree. Concurrent state/action/binding edits were present and were not changed.

## Scope and reference

The owner requested feature parity with [d3-shape](https://d3js.org/d3-shape).
The target is the complete shape module's geometric and data-layout capabilities,
implemented in Rust and usable through the existing chart engine. It includes radial
shapes, pie layout and link generators. It does not require the separate D3 hierarchy,
force, geographic, chord, selection or transition modules, a browser renderer, or a
complete polar-axis/radar product.

Use [d3-shape 3.2.0](https://github.com/d3/d3-shape/tree/v3.2.0) as the reproducible
baseline; its [manifest](https://raw.githubusercontent.com/d3/d3-shape/v3.2.0/package.json)
declares ISC licensing and `d3-path ^3.1.0`. The documentation site's D3 umbrella
version is not the shape package version. WP-S01 must lock an exact reference artifact,
its hash, and exact transitive fixture dependencies before generating expectations.
The [public export list](https://raw.githubusercontent.com/d3/d3-shape/v3.2.0/src/index.js)
is the completeness checklist, supplemented by generator methods in the documentation
linked below. Reference JavaScript is development tooling, never a runtime dependency.

“Parity” means equivalent supported inputs, controls, defaults on the explicit D3
route, outputs and extension capabilities within justified geometric tolerances.
Rust spelling, builder ownership and portable field mappings may differ from JavaScript.
Do not emulate JavaScript coercion, `this`, DOM selection, or executable callback
serialization. Native accessors and registered operations provide those computation
capabilities; Python/WASM supply materialized columns or registered Rust operations.
All such adaptations must be listed in the compatibility table, not hidden as passes.
Non-finite input continues to obey DAT-05 and never creates invalid scene coordinates.

This document owns the detailed feature inventory and package decomposition. The
[specification](../spec/gpui-charts-specification.md) owns SHP requirements, the
[main plan](gpui-charts-implementation-plan.md) owns integration with existing WPs,
and the [ledger](../implementation-status.md) owns current states and evidence.

## Current implementation assessment

Pass means the stated narrow capability exists in inspected source. Fail means the
requested parity surface is absent or demonstrably different. Uncertain means the
source provides a route but the necessary runtime/visual comparison has not been run.
No full D3 family is certified by this review. Existing WP completion evidence remains
valid for its original Cartesian scope.

| Surface / requirement | Verdict and concrete evidence | Missing behavior / consequence |
| --- | --- | --- |
| Numeric path transport; SHP-01 | Pass, foundation only: [scene.rs](../../crates/chart-core/src/scene.rs), `PathCommand` lines 42–53; [native.rs](../../crates/gpui-charts/src/native.rs), Bézier dispatch around 492; [svg.rs](../../crates/chart-export/src/svg.rs), path encoding around 283. | Move/line/quadratic/cubic/close already exist. Circular arc commands, generator contexts and precision controls need a shared route; arbitrary D3 command streams are not proven. |
| Lines and areas; SHP-02/03 | Fail: [definition.rs](../../crates/chart-core/src/grammar/definition.rs), `Geom` lines 443–490; [project.rs](../../crates/chart-core/src/layout/project.rs), `BandRun` and `LineRun` arms. | Only straight runs; no curve selection. Area uses a constant baseline; ribbon shares x and constrains lower <= upper. General two-boundary areas, curve lifecycle and boundary extractors are absent. Singleton lines become points and singleton bands become rules, which requires an explicit compatibility distinction. |
| Arcs, pies, radial shapes and links; SHP-04/05 | Fail: `Geom` inventory above and [coordinates.rs](../../crates/chart-core/src/layout/coordinates.rs), `Cartesian` lines 19–44. | No built-in arc/pie/link/radial generator route. Cartesian reports no arbitrary path subdivision. Custom polygons do not establish these capabilities. |
| Symbols; SHP-06 | Fail: [theme.rs](../../crates/chart-core/src/theme.rs), `Symbol` lines 37–48; [scene.rs](../../crates/chart-core/src/scene.rs), `symbol_path` around 617; source `Aes.size` at definition.rs:284. | Circle/square/diamond/triangle use radius-based sizing; triangle vertices are not an equal-area D3 triangle. Nine additional symbol geometries, fill/stroke palettes, area sizing and mapped symbol types are missing. |
| Stacks; SHP-07 | Fail: [statistical_types.rs](../../crates/chart-core/src/grammar/statistical_types.rs), `StackSpec` lines 297–302; [positions.rs](../../crates/chart-core/src/grammar/positions.rs), stack arm around 142–235. | Explicit order plus separate positive/negative normalization only. No six-order/five-offset API or D3 series result. Existing signed endpoint representation also differs from D3 diverging output. |
| Custom shape protocols; SHP-08 | Fail: [geometry_extensions.rs](../../crates/chart-core/src/grammar/geometry_extensions.rs) and [extension contract](../extension-contract.md). | Custom stat/geom registration exists, but no reusable curve lifecycle, symbol draw, or stack order/offset interfaces. Users should not implement a whole geometry compiler to customize a curve. |
| Inspection and all-host proof; SHP-09/10 | Uncertain for existing generic paths; Fail for absent families: [inspection.rs](../../crates/chart-core/src/inspection.rs), path candidates lines 188–202. | Candidate construction reads only MoveTo/LineTo and zips them with targets. Bézier controls/endpoints cannot substitute for source identity; curved strokes, filled interiors and donut holes need explicit hit evidence. Existing bindings prove the current grammar, not D3 parity. |

Priority: the original scope conflict and missing generators are P1 parity blockers.
Size/stack/order compatibility and path-to-target association are P1 integration
risks. Missing reference fixtures and cross-host evidence are P1 acceptance blockers,
not evidence that the existing Cartesian implementation is defective.

## Required feature inventory

Every item below requires a native public API, a documented portable equivalent where
applicable, fixture coverage, and downstream rendering/semantic evidence. Native-only
custom functions must fail serialization explicitly unless registered portably.

| Requirement | Complete target surface | Package |
| --- | --- | --- |
| SHP-01 — Generator output | Owned numeric paths and an external path sink/context; SVG path-data output with configurable fractional digits (default 3, including unrounded mode); output precision does not change sink geometry. Constants and data accessors; empty/degenerate results; repeated invocation and immutable result ownership. | WP-S01, WP-S07 |
| SHP-02 — Lines and areas | `line`: x/y, defined, curve, context, digits. `area`: x/x0/x1, y/y0/y1, defined, curve, context, digits; `lineX0`, `lineX1`, `lineY0`, `lineY1`. Preserve input order on the D3 route, paired-boundary traversal and independent gaps. [Lines](https://d3js.org/d3-shape/line), [areas](https://d3js.org/d3-shape/area). | WP-S02 |
| SHP-03 — Curves | All 20 factories: linear and linearClosed; basis, basisOpen, basisClosed; bumpX/bumpY; bundle with beta; cardinal, cardinalOpen, cardinalClosed with tension; CatmullRom, CatmullRomOpen, CatmullRomClosed with alpha; monotoneX/Y; natural; step, stepBefore, stepAfter. Defaults: beta 0.85, tension 0, alpha 0.5. Bundle is line-only; unsupported area use must diagnose. [Curves](https://d3js.org/d3-shape/curve). | WP-S02 |
| SHP-04 — Arcs and pies | Arc inner/outer radius, start/end angle, corner radius, pad angle/radius, centroid, context and digits. Pie value, data comparator/value comparator, start/end angle and padding; output retains source datum identity, value, sorted index and angles in input-array order. Pie is a layout feeding the arc geometry. [Arcs](https://d3js.org/d3-shape/arc), [pies](https://d3js.org/d3-shape/pie). | WP-S03 |
| SHP-05 — Radial shapes and links | `pointRadial`; `lineRadial` angle/radius and common line controls; `areaRadial` angle/startAngle/endAngle/radius/innerRadius/outerRadius and common area controls, plus lineStartAngle/lineEndAngle/lineInnerRadius/lineOuterRadius. `link(curve)`, linkHorizontal/Vertical with source/target/x/y/context/digits; linkRadial with source/target/angle/radius/context/digits. [Radial lines](https://d3js.org/d3-shape/radial-line), [radial areas](https://d3js.org/d3-shape/radial-area), [links](https://d3js.org/d3-shape/link), [radial links](https://d3js.org/d3-shape/radial-link). | WP-S04 |
| SHP-06 — Symbols | `symbol` type/size/context/digits; 13 distinct types: circle, cross, diamond, square, star, triangle, wye, plus, times, asterisk, diamond2, square2, triangle2. Filled palette: circle/cross/diamond/square/star/triangle/wye; stroked palette: circle/plus/times/triangle2/asterisk/square2/diamond2. Default circle, size 64; D3 area-based sizing and stroke-oriented size conventions. [Symbols](https://d3js.org/d3-shape/symbol). | WP-S05 |
| SHP-07 — Stack layout | keys/value/order/offset; series key/index and point source identity with y0/y1. Orders: none, reverse, ascending, descending, appearance, insideOut. Offsets: none, expand, diverging, silhouette, wiggle. Explicit order permutations and custom order/offset operations. [Stacks](https://d3js.org/d3-shape/stack). | WP-S06 |
| SHP-08 — Custom protocols | Curve areaStart/areaEnd/lineStart/lineEnd/point lifecycle; symbol draw(context,size); custom pie comparison and stack order/offset callbacks through typed native interfaces and versioned portable registrations. Standalone built-in curve/symbol/order/offset invocation must be usable without a chart. | WP-S01, WP-S07 |
| SHP-09 — Engine integration | One core engine for generators, grammar and recipes; domains, clips, source/derived provenance, shape-aware hit tests and keyboard targets; facets/themes/publication; Rust/Python/WASM definitions and outputs; correction/retention versus batch equality. | Each family, WP-S07, WP-S08 |
| SHP-10 — Acceptance | Pinned reference fixtures, independent mathematical expectations, actual native/SVG/PDF/PNG inspection, Python/WASM execution, error/ownership checks, supported-platform and measured complexity evidence. | WP-S01, WP-S08, WP-21/22/23 |

Compatibility names are not separate algorithms: record `radialLine -> lineRadial`,
`radialArea -> areaRadial`, `symbols -> symbolsFill`, and `symbolX -> symbolTimes`.
The public export list is authoritative for aliases; source inspection must also cover
inherited methods such as radial generator digits even where a documentation page
does not repeat them. Boundary-line helpers must preserve the reference's actual
accessor/curve/context inheritance, rather than assuming every setting is copied.

## Compatibility decisions and architecture

These are implementation constraints under specification 0.2.0, not current behavior.
WP-S01 records the selected shape API and consumes the shared path representation ADR
from WP-P01–04 with measured capability evidence; it cannot remove an inventory item.

| Existing behavior | Required addition and discriminating example |
| --- | --- |
| Chart lines sort by x; path order is explicit. | Standalone D3-compatible generators consume authored order. Chart recipes keep their defaults and expose explicit curve/order options. `(2,0),(0,1),(1,0)` must not silently reorder on the compatibility route. |
| Missing coordinates split; singleton lines paint a point. | Add a defined mask/predicate independent of source filtering. An intentionally undefined middle datum splits an otherwise finite run. The D3 route retains each curve's empty/one/two-point behavior; ordinary chart recipes retain their documented singleton policy. |
| Ribbon rejects lower > upper. | Add a general area definition with independent paired coordinates; do not weaken ribbon validation. Curves operate after scale projection, so a spline through log-projected values is not a transformed data-space spline. |
| Point size means radius; symbols are theme constants. | Add explicit area-size and type mappings with legend metadata. Preserve radius-mode decoding. D3 circle size 64 yields radius `sqrt(64/pi)`; equal numerical radius and area settings must not be conflated. Stroke-only symbols retain open strokes. |
| Stack normalize divides each sign separately. | Preserve `StackSpec` and FIX-03. Add explicit D3 offset/order descriptors, not a rename of normalize. For one column `[2,-1]`, D3 expand emits `[0,2]`, `[2,1]`; existing normalized geometry spans `[0,1]` and `[0,-1]`. D3 diverging reports the negative interval as `[-1,0]`, while source value remains -1. [Expand source](https://raw.githubusercontent.com/d3/d3-shape/v3.2.0/src/offset/expand.js), [diverging source](https://raw.githubusercontent.com/d3/d3-shape/v3.2.0/src/offset/diverging.js). |
| Finite checked core values and exact identities. | Retain them. Compare D3 on finite typed inputs and explicit validity masks. Record malformed/coercion differences and typed errors separately; never put NaN path coordinates into scenes or map missing stack cells to zero implicitly. |
| Immutable portable definitions and registrations. | Native closures are evaluated at authoring/preparation boundaries, with datum/index/input access where needed. Portable predicates/comparators use fields, materialized values or registered Rust operations. Exact group/row IDs survive sorting and pie/stack layout. |

Implement generators in a focused `chart-core::shape` module (proposed path), sharing
the existing scene buffers. Keep pie/stack data layouts reusable outside rendering and
call them from grammar/stat/position adapters. Do not add a separate chart engine, a
shape crate without a dependency need, or a mandatory JavaScript/GPUI dependency.

Use one checked path sink for built-ins and custom drawing. It must express subpaths,
lines, Béziers, circular arcs and rectangle/close behavior needed by the reference
contexts. Reuse current commands; add the smallest representation needed to retain
arc parameters until SVG encoding or destination lowering. Native/PDF arc lowering
uses a shared, bounded, destination-specific error policy; a fixed coarse polygon or
four-cubic circle cannot be assumed exact at every output size. Text outlines and
existing custom paths remain consumers of the same scene infrastructure.

The [d3-path parity plan](d3-path-parity-plan.md) now owns this foundation in
WP-P01–04: PTH-01–06 and FIX-P01–06 include the complete standalone API, state rules,
arcs/arcTo, signed rectangles and precision. WP-S01 consumes that implementation and
its ADR/corpus; it must not duplicate the sink, arc lowering or serializer. Path-only
evidence does not replace generator-specific shape acceptance.

Stroke/fill policy is separate from geometric construction. Preserve nonzero winding,
holes, open stroke paths and multiple subpaths; support both fill and stroke without
duplicating source targets. Empty shape results must be representable without submitting
invalid empty primitives. Precision formatting belongs to the SVG path serializer;
it must not quantize geometry used for native paint or hit tests.

Radial projection is `x = r*sin(a)`, `y = -r*cos(a)` in local destination units,
followed by an explicit center transform. Preserve radians, winding and seam behavior.
Radial line/area curves reuse the Cartesian curve machinery after point projection;
radial links need the reference's radial tangent construction. General polar axes,
automatic tree layout and network routing are independent future features.

Maintain source vertices and semantic targets separately from control points and
tessellation vertices. A curve control point is not an observation. Interior/segment
inspection returns source or labeled derived targets, respects clipping and holes,
and uses the presented scene revision. Bounds and indexes account for actual curve
extrema; data-domain training retains the documented scale policy.

Cache keys include curve family/parameters, masks, ordering, shape mappings, projection,
layout and quality settings. Start with exact full recomputation where necessary:
natural splines, automatic stack orders, wiggle and pie sums can affect the whole
group. Do not apply straight-line downsampling before a spline unless equivalence or
a separately declared approximation is proven. All command/subdivision/registration
work obeys explicit resource limits.

## Work packages and dependency order

WP-S identifiers are additive so completed WP-10/11/14 records are not rewritten.
All family packages extend their native definition, portable descriptors, shared
compiler route, unit fixtures and representative headless consumer together.
WP-S07 completes the cross-family public/custom surface; it is not permission to
defer all bindings until the end. Detailed signatures are fixed by working consumers.

| Package | Prerequisites | Deliverable and acceptance |
| --- | --- | --- |
| WP-S01 — Shape contract, oracle and path foundation | WP-14, WP-P04 | SHP-01/08/10. Lock shape reference artifact/dependencies/license; machine-readable export/method inventory and compatibility map; stored numeric command/layout fixtures and regeneration instructions. Consume the checked path sink, arc lowering, SVG digits, empty-result handling and budgets from WP-P01–04 and their shared ADR/corpus. Prove one circular sector, multi-subpath hole and custom sink in core/export/native using that engine. FIX-S01 consumes FIX-P01–06 and passes before broader generators depend on this interface. |
| WP-S02 — Cartesian generators and complete curves | WP-S01 | SHP-02/03/09. Implement line and general area, defined masks and boundary extractors. Land reviewable slices: linear/step/bump; basis/cardinal/CatmullRom/bundle; monotone/natural and complete open/closed/lifecycle cases. Preserve chart defaults. FIX-S02/03 plus existing FIX-01/07 pass; all 20 factories are accounted for. |
| WP-S03 — Arc geometry and pie layout | WP-S01 | SHP-04/09. Implement radii/sweeps/padding/corners/centroid and pie value/order/angle outputs; add pie/donut composition through the shared grammar. FIX-S04 proves geometry, totals, order, invalid policy, tiny wedges, full circles and annular holes. |
| WP-S04 — Radial generators and links | WP-S02, WP-S03 | SHP-05/09. Radial point/line/area, all boundary helpers, generic/horizontal/vertical/radial links, center transform and named channel bindings. FIX-S05 proves radial conversion, winding/seams and link endpoints/tangents without a hierarchy-layout dependency. |
| WP-S05 — Complete symbol encoding | WP-S01 | SHP-06/09. All 13 geometries, ordered fill/stroke palettes, aliases, area-size/type mappings and legends; preserve legacy radius mode. FIX-S06 checks independent areas/bounds and open-stroke topology, multiple sizes, all themes and headless/native paths. |
| WP-S06 — Complete stack layouts | WP-S01, WP-10 | SHP-07/09. Six order policies, five offsets, explicit permutation and stable series/point results; adapt grouped/tidy inputs and explicit missing-cell policy; connect to bars and areas. FIX-S07 proves every order/offset pair, zero/mixed-sign/sparse cases, identities and signed endpoints; FIX-03 remains unchanged. |
| WP-S07 — Custom protocols and public portability | WP-S02, WP-S03, WP-S04, WP-S05, WP-S06 | SHP-01/08/09. Public reusable curve/symbol/order/offset interfaces, custom pie comparators, versioned registry resolution and bounds; external examples; complete descriptor/schema migration and generated docs. FIX-S08 and actual Rust/Python/WASM FIX-S09 exercise every family and a registered example of each custom protocol. Native-only serialization fails explicitly. |
| WP-S08 — Integrated parity acceptance | WP-S07, WP-16, WP-18, WP-20 | SHP-09/10. Shape-aware inspection, clipping/holes, facets/themes, updates/retention and immutable exports; execute FIX-S01–09 across required hosts and inspect artifacts. Deliver per-item Pass/Fail/Uncertain inventory with no missing built-in. G-SHAPE passes only with actual evidence; feed final platform QA to WP-21 and measured shape workloads to WP-22. |

Recommended integration order: WP-S01 -> WP-S02 -> WP-S03 -> WP-S04 -> WP-S05 ->
WP-S06 -> WP-S07 -> WP-S08. WP-S03/05/06 have independent algorithm dependencies
after WP-S01, but shared grammar/schema changes have one integration owner. This is
sequencing guidance, not an instruction to launch additional agents.

WP-15–20 may continue with their existing prerequisites. Coordinate their path-target,
cache, clipping and snapshot contracts early; their original completion alone does
not certify new shapes. WP-S08 consumes their finished interfaces, then WP-21/22/23
close the expanded production gate. No dependency on WP-21 is introduced into WP-S08.

Planning estimate: 8–13 additional developer-weeks including fixtures, review and
host validation (S01 1–2; S02 2–3; S03 1–2; S04 0.5–1; S05 0.5–1; S06 1;
S07 1–2; S08 1). These are provisional effort estimates, not a delivery date or a
performance claim; they exclude unfinished WP-15–23 effort. Re-estimate after S01's
arc/native proof and S02's public API. The original 16–24 week project estimate did
not budget full D3 shapes.

## Acceptance fixtures and evidence

The fixture names below are planned, not claims that files or runners exist. WP-S01
creates `fixtures/shapes/` and the reference generator only when it can run. Store
reference paths at full context precision as well as rounded strings; numeric paths
and layout records are the oracle, not screenshots or JavaScript object identities.

| Fixture | Discriminating cases and expected evidence |
| --- | --- |
| FIX-S01 | Sink versus returned path versus SVG digits 0/3/high/unrounded; empty results, repeated calls, disconnected subpaths and winding holes; finite/overflow and command-budget errors; destination arc error bounds. |
| FIX-S02 | Finite defined-mask gaps, null/NaN policy, authored versus x order, singleton/two-point runs, duplicate points, horizontal/vertical general areas, varying both boundaries and all Cartesian boundary helpers. Compare path topology and helper inheritance. |
| FIX-S03 | Every curve factory with 0–4 points and longer irregular runs; open/closed endpoint behavior; alpha/beta/tension endpoints and defaults; bundle area rejection; monotone no-overshoot on admissible monotone-axis data; natural endpoint curvature; transformed-axis and repeated lifecycle cases. |
| FIX-S04 | Quarter arc from 0 to pi/2 at radius 10 ends at (10,0) and starts at (0,-10); reverse/full/>full sweeps, swapped/negative radii, zero radius, tiny sectors, clamped/intersecting corners, explicit/auto padRadius, centroid distinct from area centroid. Pie [1,1,2] with no sorting/padding spans pi/2, pi/2, pi; test default sorting, comparator exclusivity, zeros/negative values, tied values, reversed/partial span and padding limits. |
| FIX-S05 | pointRadial cardinal directions, negative/zero radii and wrap seams; radial line/area projection and all four boundary helpers; horizontal/vertical Bézier controls and radial tangents; coincident/reversed endpoints and a custom two-point link curve. |
| FIX-S06 | All 13 symbols at multiple sizes; circle size 64 radius sqrt(64/pi), filled square side sqrt(size), independent triangle/star/wye geometry checks; palette and alias identity; zero/invalid sizes; stroked plus/times remain unfilled; mapped type/size matches legend and hit geometry. |
| FIX-S07 | All 30 built-in order/offset combinations with source identities; tied sums/peaks, explicit permutation validation, one/zero series or samples, zero-only columns, negative/missing values under recorded policy; [2,-1] discriminates expand/diverging/legacy normalize. Verify order before offset, result key order versus stack rank, silhouette centering and wiggle against pinned output. |
| FIX-S08 | Custom curve lifecycle on multiple gapped areas; custom symbol draw; custom pie comparator and stack order/offset; standalone invocation; unknown/version-mismatched registrations, invalid outputs, budget exhaustion, deterministic reuse and native-only serialization errors. |
| FIX-S09 | Same portable fixtures in Rust/Python/real Node WASM; large IDs, masks and defaults; source/derived targets on curves, donut hole misses, interior/clip/z-order hits and keyboard order. Append/upsert/remove/retention versus batch; resize/zoom, themes/facets, 300/600 DPI SVG/PDF/PNG and native preview, disposal and old-snapshot export. |

For each family, store inputs, parameter/default settings, operation/version identity,
expected commands/layout data, source references/hashes and license provenance.
Use independent analytic cases alongside the external oracle; do not derive both
expected and actual values from the Rust implementation. Compare topology, ordering,
validity and target IDs exactly. Establish separate, justified tolerances for f64
control coordinates, pie angles/stack endpoints, formatted SVG, arc lowering and
raster comparisons. S01 locks those tolerances before acceptance; no global loose
epsilon or regenerated visual baseline may hide a discrepancy.

Final implementation checks use existing `mise run fmt`, `mise run check`,
`mise run test`, `mise run bindings-proof`, and the actual shape gallery/export
commands introduced with their runners. Rust fixtures run without D3 installed;
reference regeneration is a separate pinned development command. A compile-only
WASM check cannot pass FIX-S09. Inspect native and actual exported images; verify
vector marks and holes in SVG/PDF, not just raster similarity.

WP-22 must retain PERF-01–05 and add reported shape workloads covering long natural
and CatmullRom paths, many rounded arcs/symbols, streamgraph recomputation, tessellation
growth, hit-index cost and large-output export. Report source rows, control commands,
flattened segments, peak memory and frame/update/export cost separately. Resource
growth must be bounded; an exact batch fallback is acceptable only within the stated
supported workload. Do not claim existing straight-line timing certifies all curves.

## Review evidence and next action

This planning review read the specification, implementation plan, status ledger,
geometry/statistics/extension/portable contracts and the source paths above; inspected
official D3 documentation for all eleven shape families and the pinned export list,
manifest, line and stack-offset source. Commands included `git status --short`,
`git rev-parse HEAD`, `uname -sm`, scoped `rg` searches and `sed` source reads in
`/Users/jeickmeier/Projects/finstack-chart` on Darwin arm64. Shell HTTP fetch failed
DNS resolution; the cited sources were read through the web tool. No D3 package was
installed or executed and no new shape acceptance fixture was run.

Documentation validation is recorded in the status ledger. Existing implementation
test counts are historical evidence, not rerun or parity certification here.
Next implementation action: complete WP-P01–04 from the path plan, then assign WP-S01
to consume that foundation and lock the shape contracts before adding generators. All SHP-01–10 and
G-SHAPE implementation gates remain open.
