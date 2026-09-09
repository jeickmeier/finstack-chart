# WP-S02 Cartesian generators and curves — 9 September 2026

WP-S02 is complete for SHP-02/03 and the Cartesian portion of SHP-09 through the
shared core, primary APIs, grammar, renderers and inspection; G-SHAPE remains open for subsequent families and integrated
acceptance. Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`, within the
owner's uncommitted Phase 2 continuation. [ADR-020](../adr/020-shape-generators-and-curve-protocols.md)
records lifecycle, projection, source-target and wire contracts.

All 20 curve factories are implemented, including open/closed variants, bundle
beta, cardinal tension and Catmull–Rom alpha. Reusable `shape::Line` and
`shape::Area` support constants/columns, native accessors, defined masks, independent
boundaries, four boundary extractors, numeric/external-context paths and SVG digits.
Area helpers preserve D3's accessor/default inheritance, including resetting digits
to three. Bundle rejects area use. Existing chart line/area/ribbon recipes keep
their prior ordering and singleton rules; new `shape_line`/`shape_area` routes
preserve authored order and D3's degenerate geometry.

The independent pinned d3-shape 3.2.0 oracle adds
[561 extended cases](../../fixtures/shapes/cartesian.json) to 268 line/area seeds.
The 829 cases cover every factory, small/gapped runs, duplicate/reversed/vertical
coordinates, setter order, helper inheritance and finite parameter extrapolation.
827 finite cases match numeric command topology and coordinates within
`2e-12 * max(1, abs(expected))`, with exact three-digit SVG output. Two negative-alpha
Catmull–Rom cases produce nonfinite reference controls; the checked implementation
returns `CHART_NUMERICAL_DOMAIN`. Fixtures were not relaxed. Independent tests
also establish natural endpoint second derivatives, monotone bounds and
alpha-zero/cardinal equivalence.

The chart routes generate curves **after destination projection**. A log-axis
BumpX test proves the projected control coordinates independently. General areas
require lower x/y and upper x2/y2 channels, permit crossing boundaries and split
paired runs when either boundary is missing. Definition version seven and scene
version three preserve the new descriptors and numeric shape paths; older versions
reject the new capability. Existing definitions retain their original version.

`ShapePath` retains one source anchor per observation, separate from curve controls.
Inspection uses one bounded flattened path per painted shape and resolves its
winning hit to the nearest actual source anchor. Nonzero fill winding preserves
holes; solid stroke tests exercise butt caps and miter joins. Keyboard, nearest-x,
brush and series selection retain source identities, including exact keys above
2^53 and clipped areas whose source anchors lie outside the viewport. A clipped
keyboard anchor is not introduced as a visible brush vertex. No control point is
reported as an observation. Flattening uses a 0.01 destination-unit error bound,
a depth bound and an aggregate one-million-vertex inspection budget.

The six generator tests and six integration tests pass on [macOS](phase-2-shape-cartesian/focused-macos.log) and [Linux](phase-2-shape-cartesian/linux-core.log).
The macOS focused run also passes ten existing inspection tests (22 total).
Linux passes 41 core/library/path/inspection/foundation/Cartesian tests, then
builds and executes its Python extension, generators, interaction, 64 update
steps and publication author. Actual macOS Python and Node WASM each run all
829 generator cases, repeated invocation, copies, disposal, invalid descriptors,
log projection, clipped hit/focus and exact-key transport. Each host compares
64 append/upsert/remove/retain steps against fresh batch PNGs over four curves,
line/area and faceted/nonfaceted views, retaining earlier immutable exports.
Positive Python/TypeScript consumers pass; the Python negative consumer produces
its five expected errors.

The independent Rust/Python/WASM gallery contains 20 lines, 19 general areas and
explicit source points. At 300/600 DPI the full scenes and PNG bytes agree across
all three authors. Linux Rust/Python SVG and PNG bytes also match the inspected
outputs. Supplied Noto Sans is embedded in PDF output. Native, PNG, PDF raster
and external SVG raster were inspected for each open/closed/stepped/curved family,
paired fill boundaries, source points and layout. The resvg probe uses the supplied
font and replaces its generated family alias only in memory; stored SVG bytes
remain unchanged.

Final [repository checks](phase-2-shape-cartesian/check.log) pass formatting, dependency/repository validation, builds,
Clippy, rustdoc and WASM checks. The [full macOS suite](phase-2-shape-cartesian/tests-macos.log) passes 369 tests.
The [consolidated primary runner](phase-2-shape-cartesian/primary-all.log) completed its
earlier API/scale/color/chromatic/foundation proofs and the full Cartesian tail.
Independently executed final [Python](phase-2-shape-cartesian/python-shape_cartesian.log)/
[WASM](phase-2-shape-cartesian/wasm-shape_cartesian.log), interaction/update/gallery/compare
and strict type commands also pass on those same freshly built modules.  Some first executable launches waited in `_dyld_start` before test entry;
those waits are not counted as passing tests. The broader platform/performance
acceptance remains WP-S08 and WP-21/22/23. Curved dash styling is explicitly open
for WP-S08; the present shape paths and gallery use solid strokes.

[Final source snapshot](phase-2-shape-cartesian/source-sha256.json),
[runtime hashes](phase-2-shape-cartesian/runtime-sha256.json) and
[artifact hashes](phase-2-shape-cartesian/artifacts-sha256.json) identify the accepted
Cartesian implementation before the subsequent arc/pie module registration.
[Byte comparisons](phase-2-shape-cartesian/inspected-output-equality.json) confirm all
24 final publication artifacts match their previously inspected versions; the
[native capture](phase-2-shape-cartesian/native.png) is from the final rebuilt binary. Next: WP-S03 arc geometry and pie
layout, whose independent 620-arc/192-pie corpus has passed its first kernel tests.
