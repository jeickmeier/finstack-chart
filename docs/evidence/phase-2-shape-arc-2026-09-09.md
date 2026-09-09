# WP-S03 arc geometry and pie layout — 9 September 2026

WP-S03 qualification is in progress for SHP-04 and its SHP-09 integration.
Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`, with the owner's
uncommitted Phase 2 work preserved. [ADR-020](../adr/020-shape-generators-and-curve-protocols.md)
records the generator, chart, coordinate, grouping and ownership contracts.

The core `Arc` generator implements radius swapping, signed and full-circle sweeps,
annular winding, corner-radius clamps and padding, through the shared analytic
path engine. Constants override selected datum fields; native accessors materialize
checked `ArcParameters`. The centroid follows D3's midpoint-angle/midpoint-radius
helper, independently of corner/padding geometry. It is not an area centroid.
`Pie` preserves input-order result ownership and separate angular rank, sums only
positive values, retains zero/negative entries, clamps sweep/padding and supports
native value/data comparisons and whole-input angle accessors. Portable built-ins
cover input order and ascending/descending value order; registered comparison
protocols remain WP-S07.

The pinned d3-shape 3.2.0 [oracle](../../fixtures/shapes/arc-pie.json) contains
620 arc cases and 192 pie cases. Of the arcs, 619 have finite geometry; one extreme
finite-input case overflows reference geometry and receives a checked numerical
domain error. Numeric operations use the existing `2e-12 * max(1, abs(expected))`
tolerance; zero/three-digit SVG strings match exactly. Rust exercises all 192 pies,
including 48 custom data-comparator cases; Python/WASM exercise the 144 portable
built-in cases. Separate mathematical tests check quarter circles, centroids,
positive-value allocation, defaults, limits, copies and accessor selection.

Primary `shape_arc`/`shape_pie` chart routes use the same generators. Centers pass
through data scales and layout; radii and angular geometry are display-space
values. Numeric parameter channels accept constants, fields, expressions and
statistical inputs through shared channel evaluation. Pie grouping is independent
of inferred slice color. `pie_grouped(false)` explicitly combines generated
category counts into a single pie while preserving aggregate source members.
Unexpected statistical-input fields reject consistently. Definition version seven
and scene version three retain the new shapes; older definitions keep their version.

The five integration tests cover log-projected centers, display radii, analytic
annular paths, hole-aware inspection, exact source keys, focus/clipping, wire
round trips, explicit grouping, invalid channels and generated-count provenance.
Linux passes these five tests, three generator tests and six existing Cartesian
integration tests, builds the Python extension, and executes its oracle,
interaction, gallery and update programs. Final WASM executes the same built-in
oracle and interaction programs. Each host update program compares 64 append,
upsert, remove and retain steps against fresh batch PNGs across four arc/pie
configurations and faceted/nonfaceted plots, retaining old exports after mutation.

An initial update comparison exposed a test-setup difference: a live categorical
color dictionary retains removed categories, while a fresh author infers surviving
categories. Geometry was identical; colors and one legend key differed. Both
compared authors now use the same explicit color catalogue, preserving that
existing behavior and the exact PNG assertion. No oracle fixture or numeric
threshold was weakened.

The gallery has eight independent arc configurations and two four-slice pie/donut
compositions. Rust/Python full scenes agree exactly at 300/600 DPI. WASM has four
arc-endpoint scalar differences at each resolution, no larger than
`5.684341886080802e-14`; all other scene values and PNG/PDF bytes agree. The comparison
runner permits the predeclared tolerance only at raw shape-coordinate fields and
records every nonexact scalar. PNG, external SVG raster and embedded-font PDF
raster were inspected for sweeps, holes, padding, corners, labels and layout.

Remaining qualification: final macOS Python execution, native launch/inspection,
repository checks and retained final runtime/source/artifact identities. G-SHAPE
remains open for WP-S04–08. Production platform/performance closure remains
WP-21/22/23; the preceding 369-test macOS suite belongs to WP-S02, before this change.

## Combined symbol regression supplement

After symbol legend integration and lint cleanup, 37 focused Linux core tests pass.
Fresh Linux Python and actual Node WASM each replay 620 arc/144 portable pie cases,
presented interactions and 64 updates. Full arc scenes and PNG/SVG/PDF bytes remain
identical to the prior retained output at 300/600 DPI on each host.
[Comparison](phase-2-shape-symbol/arc-regression.json) and final host logs in
[the symbol evidence](phase-2-shape-symbol-2026-09-09.md) retain this supplement.
Linux Clippy and rustdoc pass. macOS/native and whole-repository qualification remain
open; no completed WP-S03 gate is claimed.


Native follow-up: the standard `ShapeArcProof.app` launch completed after the
previous loader wait. Its paint log records three successful frames with one layout.
The screenshot was visually inspected: eight arc variants, pie, padded donut, holes,
legend, facet labels and title render correctly without overlap or cropping.
Screenshot, paint log and executable hash are retained under
`phase-2-shape-arc/native/`. macOS Python and final repository qualification remain
open. The formerly sleeping Python build resumed against source changed during its
wait and failed with mixed-revision imports; a fresh current-source build/check is
required rather than treating that result as a feature regression.


Final macOS qualification supersedes the pending platform statements above. The
fresh current-source Python extension passed the standalone and interaction corpus,
all family update/retention cases, and both publication resolutions. Publication was
compared with the independently authored Rust and WASM artifacts using the existing
family comparison contract. Native windows were inspected and retained as described
above. `CARGO_TARGET_DIR=target/shape-native-target mise run check` passed repository,
format, dependency/license, native/Kit build, workspace all-target checks/Clippy,
rustdoc with denied warnings, and WASM core compilation. `mise run test` with the
same private target passed 393 tests/doctests in 86 result blocks, zero failures and
zero ignored tests; empty test blocks are not counted as tests.

The accepted Python runtime, source hashes, publication, comparisons, build/check
and full-test logs are retained in this family's `macos/` evidence directory. Every
recorded source hash was checked again after the full suite and remained unchanged.
Native executables retain their separately recorded pre-radial standalone-addition
identities; the arc/symbol/stack implementation and native renderer were unchanged,
and the fresh host and workspace checks cover the subsequent shared-reader change.
The three transient proof applications were closed after inspection. This completes
the family package; cumulative G-SHAPE, remaining Phase 2 and WP-21/22/23 release
qualification are separate and remain open.
