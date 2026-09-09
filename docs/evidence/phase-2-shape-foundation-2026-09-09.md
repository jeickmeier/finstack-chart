# WP-S01 shape contract, oracle and path foundation — 9 September 2026

WP-S01 is complete for its entry/foundation scope, SHP-01/08/10 and FIX-S01.
Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`, within the owner's
uncommitted Phase 2 continuation. [ADR-020](../adr/020-shape-generators-and-curve-protocols.md)
selects the existing checked path engine and records typed generator/protocol boundaries.
No built-in shape generator is claimed implemented by this package; G-SHAPE stays open.

The pinned d3-shape 3.2.0/d3-path 3.1.0 [inventory](../../fixtures/shapes/inventory.json)
accounts for all 63 exports, 20 curves, four aliases, generator methods/defaults,
factory parameters, both symbol palettes and boundary-helper inheritance. The
[corpus](../../fixtures/shapes/cases.json) has 333 numeric-context cases and 42 pie/stack
layout records, plus observed custom line/area lifecycle events. It includes empty
and small paths, defined gaps, open/closed curves, parameter endpoints, arcs and
holes, symbols, radial/link seeds and every stack order/offset pair. Later packages
expand the argument matrices. Every geometric inventory row remains open until its
actual owner provides implementation evidence.

`tools/reference/node/shape.mjs` generated the reference independently under pinned
Node 24.14.0. A separate regeneration was byte-identical for cases, inventory and
manifest. Source/lock/generator hashes and ISC licensing are retained alongside it.
The two [Linux core tests](phase-2-shape-foundation/core-linux.log) and the same two
[macOS tests](phase-2-shape-foundation/core-macos.log) pass all 333
streams through the checked path engine, exact SVG strings at digits 0/3/12,
repeatability, full-precision external replay and bounded sink rejection. These are
context transport tests, not implementations of the reference generators. Existing
G-PATH evidence supplies the shared arc/state/overflow/destination-bound contracts.

Actual [macOS Python](phase-2-shape-foundation/python-macos.log),
[Linux Python](phase-2-shape-foundation/python-linux.log) and
[Node WASM](phase-2-shape-foundation/wasm.log) also pass all 333 contexts, exact rounded
strings, precision-independent replay, copied path ownership after source mutation
and disposal. Each host independently authors the same circular sector, annular hole
and Bézier stream. Rust uses an external `PathSink` and submits its checked collected
commands. All consumers use the existing numeric vector-path route, retain no false
source targets and export captures after source disposal where supported by the proof.

At 300/600 DPI, [full scene and PNG byte comparisons](phase-2-shape-foundation/compare.log)
are identical across Linux Rust, macOS Python and WASM. Separate pixel checks prove
blue sector/ring interiors and white outside/hole interiors at independently chosen
coordinates. Supplied Noto Sans is embedded in the PDFs. The [native capture](phase-2-shape-foundation/native.png),
[PNG](phase-2-shape-foundation/visual/png-300.png),
[PDF raster](phase-2-shape-foundation/visual/pdf-600.png) and
[external SVG raster](phase-2-shape-foundation/visual/svg.png) were visually inspected:
sector endpoints, open hole, unfilled Bézier, labels and layout agree. The resvg probe
uses exact supplied font bytes and substitutes the generated family alias only in
memory because it does not read SVG @font-face. Stored SVG bytes are untouched.

The examples are `chart-export --example shape_foundation` and
`chart-gallery --example shape_foundation`. The main primary authoring runner now
includes core, Rust/Python/WASM and full-scene/image comparison commands for this
package; these were executed separately after the chromatic runner's final build.
[Repository checks](phase-2-shape-foundation/check.log) cover the final shared
production source and new Rust examples/tests; [link/graph checks](phase-2-shape-foundation/repository.log)
cover the additional documentation. The passing native build and actual macOS Python
execution complement the Linux core/export tests; no new release/performance gate
is inferred. First macOS executable launches were delayed before program entry;
actual completed runs are the evidence, not the interrupted diagnostic attempts.

[Source hashes](phase-2-shape-foundation/source-sha256.json),
[runtime hashes](phase-2-shape-foundation/runtime-sha256.json) and
[artifact hashes](phase-2-shape-foundation/artifacts-sha256.json) identify this historical
snapshot. New work proceeds to WP-S02: Cartesian generators, all 20 curves,
boundary helpers, grammar/source-target integration and the corresponding host proofs.
