# WP-S08 integrated shape acceptance — 9 September 2026

Status: COMPLETE for the finite typed d3-shape 3.2.0 compatibility profile. SHP-09/10 and
FIX-S01–09 pass; G-SHAPE passes for this snapshot. WP-S07, WP-16, WP-18 and WP-20
are accepted prerequisites. Base revision `fab2505951061eaafe9adb52c86b248ee0dfa6bf`
plus the owner's retained Phase 2 work. The [source inventory](phase-2-shape-acceptance/integrated/source-sha256.json)
identifies the actual qualified files; the later independent-axis example is a
separate WP-AX01 addition. No oracle expectations or tolerances were changed.

The [per-item inventory](phase-2-shape-acceptance/inventory.json) maps all 63 pinned
exports and 220 methods to Rust APIs, typed/native/registered adaptations and the
accepted family evidence. Every required built-in has an integrated Pass verdict.
The immutable oracle inventory remains a reference inventory; its historical open
implementation labels are not rewritten to manufacture acceptance.

## Implementation and discriminating fixes

The remaining curved-dash styling gap is implemented across retained scenes, themes,
SVG, PDF, native rendering and containment. Scene version four carries nonempty
path dash patterns; solid scene versions and omitted empty-pattern serialization
remain compatible. SVG/PDF preserve original vector curves. Native strokes and
inspection use the shared bounded flattener and dash splitter at their own destination
precision. Fills, clips and authored semantic anchors retain their original geometry.
Closing seams join the last and first visible dash where both contain ink.
[ADR-020](../adr/020-shape-generators-and-curve-protocols.md) records work and precision
limits; geometric flattening tolerance is not an independent arc-length phase-error
bound. Original geometry remains unrounded outside explicit SVG string formatting.

Fresh acceptance exposed two additional defects. Vector annotations are appended
after the main theme pass; they now capture effective dash tokens when appended.
WP-S07's temporary curve-factory command bound also leaked into a returned Path's
later edit budget. Generation still enforces that bound, then restores the original
caller command limit on success. Empty/singleton line and area outputs remain
independently editable; an explicitly small caller budget still rejects excess work.
The unchanged full Cartesian host fixture and a new core regression pass.

Four dash core tests and one export test cover independent straight dash endpoints,
subpath reset, painted/gap containment, exact source keys, retained cubic commands,
empty/malformed patterns, work exhaustion, closed joins, annular vector fill/stroke,
and immutable SVG/PDF/PNG outputs. The actual Python/WASM dash proofs also verify
clipping, wire round trips and export after disposal.

## Retained acceptance evidence

| Evidence | Result and boundary |
| --- | --- |
| `CARGO_TARGET_DIR=target/shape-native-target mise run check` | PASS repository/dependency/build/lint/rustdoc/WASM checks for the qualified source. The existing `block` future-compatibility notice is retained in the log. |
| `CARGO_TARGET_DIR=target/shape-native-target mise run test` | PASS 420 macOS workspace tests/doctests across 92 result blocks. |
| Offline Linux `cargo test -p chart-core -p chart-export -p chart-extension-example --locked --offline` | PASS 419 tests/doctests across 81 result blocks in the established Rust 1.97.1 Bookworm environment. This is the core/export/extension boundary, not Linux native-app certification. |
| `mise run bindings-proof target/shape-acceptance/bindings` with matching `WASM_BINDGEN` | PASS actual Rust, Python and Node/WASM retained capture, update, disposal and publication contracts, including 86 path operation traces and destination comparisons. |
| `scripts/run_shape_acceptance.py` against fresh macOS Python, Linux Python and Node/WASM modules | PASS every family generator, interaction and update proof; the runner uses committed expectations and never regenerates the oracle. |
| Six update families | PASS 1,544 append/upsert/remove/retention versus fresh-batch comparisons per host: Cartesian 64, arc/pie 64, symbol 104, stack 480, radial/link 760 and registered protocols 72. All three hosts' result records agree exactly. |
| Complete dash gallery | PASS all 20 curve strokes and 19 supported area fills, source anchors, three themes and 300/600 DPI publication. All non-coordinate scene values and PNG/PDF bytes agree exactly across Rust, Python and WASM. Control coordinates satisfy the locked tolerance; the two Terminal SVG comparisons retain the tiny numeric differences described below. |
| Actual image/native inspection | PASS nine representative publication images: PNG, externally rendered PDF and SVG for each of Editorial/Terminal/Grayscale. All labels, dash gaps, markers, open/closed geometry and area fills remain legible/correct. Native capture records three paints and one layout and was inspected separately. Both DPI sizes and all host pixel equality are checked programmatically. |

The 12 publication comparisons preserve the existing command-coordinate tolerance
`2e-12 * max(1, abs(expected))`. macOS Python scenes are exact; WASM differs in two
Terminal CatmullRomClosed area control coordinates by at most `5.684341886080802e-14` at each DPI. Its two
Terminal SVG files preserve those numeric-string differences; the other ten SVG
comparisons are byte-identical. PNG/PDF bytes and externally rasterized SVG pixels
agree exactly. This is measured f64 variation within the existing contract, not an
exact-command or identical-SVG-text claim. No tolerance was widened.

The fresh standalone corpus includes 333 context/replay cases; 829 Cartesian cases;
620 arc cases and 144 portable pie layouts; 40 radial points, 697 radial/link paths
and six unrounded comparisons; 156 symbol cases; and 435 stack cases. Interaction
proofs cover source versus derived membership, exact large IDs, non-source controls,
donut/stroke holes, clipped interior hits/focus, transformed centers, sparse paths,
facets and aggregate work budgets. Existing WP-S01–07 reports retain their independently
authored family galleries, native inspection, helper/default inheritance, custom
callbacks and strict host declarations; this package reruns their generator and
interaction contracts against the final common implementation.

Explicit diagnostic cases remain part of acceptance: two nonfinite Cartesian reference
outputs, one arc overflow and 48 stack chart configurations outside publication
precision are checked failures. They are not counted as numerical matches. The 48
native pie-comparator fixtures run in Rust; the portable registered comparator and
all five custom protocols run in actual Python/WASM. JavaScript implicit coercion,
DOM/context object identity and serialization of arbitrary native closures are outside
the declared portable profile. Unknown/version-mismatched registrations and native-only
serialization reject explicitly.

[The integrated evidence directory](phase-2-shape-acceptance/integrated/) retains logs,
artifacts, comparisons, environment details, exact source/runtime hashes and an artifact
hash inventory. [Native evidence](phase-2-shape-acceptance/native/inspection.json) retains
the inspected app image and binary identity. macOS is 26.5.2 arm64, Rust is 1.97.1 and
Python is 3.14.6. The shape runner uses mise Node 24.14.0; the broader binding task's
own environment records Node 26.7.0. Both use wasm-bindgen 0.2.128. An initial generator
attempt selected the older 0.2.122 CLI and was rerun with the matching installed CLI;
no dependency downgrade was made. One earlier macOS extension startup exited -9 after
a module copy; acceptance uses a fresh owned module copy and the complete successful
run, without attributing an unproven operating-system cause.

## Remaining work

G-SHAPE is accepted for this retained profile and these supported surfaces. Expanded
platform/release QA and measured shape workloads remain WP-21/22/23: long natural and
CatmullRom paths, rounded arcs/symbols, streamgraphs, tessellation, hit-index memory and
large exports still require their separate workload records. This package does not
close G4 or G-PARITY. Continue WP-AX01 provider integration, the remaining axis packages,
and the other authorized Phase 2 work in prerequisite order.
