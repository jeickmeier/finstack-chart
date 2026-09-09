# CP-04/05 chromatic integration — 9 September 2026

Scope: CHR-01–06 / FIX-21, composed with the accepted SP-07 typed scale contract,
CLR color primitives and WP-20 immutable publication. Base revision
`fab2505951061eaafe9adb52c86b248ee0dfa6bf`; implementation remains in the owner's
uncommitted Phase 2 continuation. CP-04/05 are complete for CHR-01–06 / FIX-21. The final transformed-parameter matrix
passes on Rust and actual Python/WASM, including macOS and Linux qualification;
G-CHROMATIC passes for this retained source snapshot.

## Implemented contract

The core catalog owns all 38 scheme names, 218 actual arrays and 38 named ramps.
[Entry evidence](phase-2-chromatic-entry-2026-09-09.md) pins d3-scale-chromatic 3.1.0,
all 76 exports and original source/lock/license hashes. [Foundation evidence](phase-2-chromatic-foundations-2026-09-09.md)
records exact RGBA comparison for 160,666 sampled rows, including dense/off-grid,
lookup-boundary, analytic byte-boundary, large/outside inputs and tagged exceptional
outcomes. It also records the independently generated 27,434 trigonometric anchors
and the shared fixed fused V8-compatible kernels that resolved native rounding drift.

Public Python and WASM expose the complete catalog, exact discrete scheme lookup
and an owned chromatic interpolator. Scheme results are independent color owners;
ramp copies and sampled colors outlive their source. Unknown names, invalid sizes,
malformed descriptors, non-finite direct parameters and disposed owners diagnose.
The type declarations list every supported identity and exact option shape, with
positive and negative consumer checks. The adapters perform no color arithmetic. Null direct parameters, counts zero/one and
oversized bounded sampling requests reject; three-sample output includes both endpoints
and the independently pinned midpoint.

`MappedScaleSpec.catalog` retains a checked version-one discrete identity, size and
reversal alongside its canonical range; ingestion rejects metadata/range disagreement.
The shared interpolator descriptor retains named ramp identity and reversal. Definitions
carrying either route require v6; v1–v5 retain their prior meanings, and old-version
envelopes cannot carry new capabilities. Full mapping metadata owns guide equality.

Sequential, asymmetric diverging and empirical-rank normalization stay in the shared
scale owner. Diverging [-10, 0, 100] maps its middle knot to t=0.5; descending domains
and ramp reversal are independent operations. Chart null/NaN/Inf observations retain
missing paint. A reproduced categorical-number counterexample additionally showed
NaN/Inf keys shifting an eligible named ordinal palette; the final shared fix excludes
those keys from training and paints them as missing. Direct finite ramp behavior and
legacy unnamed ordinal semantics are unchanged.

## Evidence and acceptance

The main [primary proof](phase-2-chromatic-integration/primary.log) builds fresh
macOS Python and WASM modules and runs existing primary/stage/color/interpolation/
scale/calendar regressions, typed consumers and the chromatic catalog/gallery/update
proofs. The retained earlier test-only supplement logs remain available for diagnostic history:
[Python standalone](phase-2-chromatic-integration/standalone-python.log),
[WASM standalone](phase-2-chromatic-integration/standalone-wasm.log),
[Python composition](phase-2-chromatic-integration/composition-python.log) and
[WASM composition](phase-2-chromatic-integration/composition-wasm.log).
The final fresh primary run includes those supplements and all 304 exceptional-normalization cases.

`mise run test` passed [352 macOS tests](phase-2-chromatic-integration/macos.log).
The [six-test supplemental core run](phase-2-chromatic-integration/core-supplement.log)
adds the 90-case / 1,268-sample composition matrix and equal-endpoint guide identity
case to the four integration tests already included in that suite. The offline Linux
runs passed [25 focused tests](phase-2-chromatic-integration/linux.log) and the same
[six-test supplement](phase-2-chromatic-integration/linux-supplement.log).
[Repository checks](phase-2-chromatic-integration/check.log) and
[supplemental Clippy](phase-2-chromatic-integration/clippy-supplement.log) pass.
An [independent Linux runtime run](phase-2-chromatic-integration/linux-primary.log)
rebuilds Python/export/WASM and repeats reference, missing-key, composition/theme and
update proofs after the ordinal-key fix. All ten Linux Rust/Python publication comparisons
against macOS are [exact RGBA](phase-2-chromatic-integration/linux-publication-comparison.json).

| Requirement | Evidence | Qualification |
| --- | --- | --- |
| CHR-01 catalog inventory | 38+38 identities, all 218 authored arrays; normalized source extraction regenerates byte-for-byte | Pass |
| CHR-02 discrete arrays | Exact Rust/Python/WASM RGBA for every size, reversal, isolated results and size rejection | Pass |
| CHR-03 continuous evaluation | Every one of 160,666 oracle rows through core and both actual host runtimes; shared color/interpolation regressions | Pass |
| CHR-04 scale/guide composition | 90 pinned combinations / 1,268 exact scene colors: every sequential/diverging/rank family, quantile/quantize/threshold, ordinal reuse/unknown, descending/clamp/reversal, missing paint, full guide identity and editorial/terminal/grayscale output | Pass |
| CHR-05 host/wire contract | Public query/evaluate/copy/dispose operations; v1/v6 round trips and strict rejection; typed consumers | Pass |
| CHR-06 integration | Five independent three-host figures; 24 update steps per host; native/SVG/PDF/PNG inspection; component costs and memory plateau | Pass |

Each update runner covers named ordinal, quantile and empirical-rank mappings with
and without facets. Append, keyed upsert, remove and count retention compare full
rendered output with a fresh authored batch. Exact uint64 keys exceed 2^53. Earlier
captured requests remain unchanged after all updates and disposal. Core palette-only
reconfiguration preserves table identity, geometry, source targets and positional
domains while updating paints and guide identity; results match fresh preparation.

Five fixtures independently authored in Rust, Python and WASM cover Category10,
Blues k=5, Viridis lookup, asymmetric RdBu and cyclic Rainbow. Guides are accurately
labeled endpoint/center or category swatches; no smooth strip is inferred from those
samples. Marks at the declared panel endpoints are clipped by the existing panel
clip contract. PNG RGBA is identical across hosts. The supplied Noto Sans is embedded
in all PDFs; independently rasterized PDFs and SVGs were visually inspected alongside
the native GPUI view. The external resvg probe registers the exact font bytes and
replaces the generated family alias with Noto Sans only in memory because resvg does
not load SVG `@font-face`; stored SVG bytes are unchanged.

## Measured costs and limitations

The release component benchmark uses one warmup and seven samples on Linux aarch64
with Rust 1.97.1 on a shared host, without CPU affinity or load isolation.
[Raw timing samples](phase-2-chromatic-integration/benchmark.json) retain min/median/max.
Final-source median prepared sampling measured approximately 4.4 ns for Viridis, 21.9 ns
for Blues and RdBu, and 15.8 ns for Rainbow and Sinebow.
Palette-only preparation at 1,000 / 10,000 / 100,000 marks measured approximately
0.17 / 1.61 / 24.87 ms, with zero statistical layer evaluation and retained table
identity asserted during each update. These are scoped measurements, not release
budgets or native frame/RSS measurements.

The catalog contains 1,427 discrete u32 colors (5,708 literal bytes), plus four
256-color lookup tables (4,096 bytes). These counts exclude metadata, instructions,
allocator overhead and compiled-object duplication. The actual WASM ownership stress
creates/disposes 18,000 owners across 300 cycles and four ramp families; six collected
batches in the final Node run plateaued at 2,621,440 bytes of linear-memory capacity. This does not count
native allocations or establish a sustained-load release budget.

Some first macOS launches stalled in `dyld` before program entry. The sampled failure
and interrupted build logs are diagnostic history; successful retries and fresh final
builds are used for acceptance. No loader workaround or signature modification is part
of the successful final commands. An independent offline Linux run also executes
actual Python and export proofs. Its WASM build mounts the installed target standard
library read-only; host and target compiler versions and commit hashes match exactly.
No fixture, expected byte or tolerance was weakened. G-PARITY, G-AUTH and the remaining
Phase 2/release gates are outside this package's acceptance.

## Retained artifacts and reproducibility

[Native capture](phase-2-chromatic-integration/native.png),
[PNG contact sheet](phase-2-chromatic-integration/visual/contact-1.png),
[PDF contact sheet](phase-2-chromatic-integration/visual/pdf-contact-1.png),
[external SVG example](phase-2-chromatic-integration/visual/svg/cyclic.png),
[publication checks](phase-2-chromatic-integration/visual/publication-checks-final.json),
[source hashes](phase-2-chromatic-integration/source-sha256.json),
[runtime hashes](phase-2-chromatic-integration/runtime-sha256.json) and
[artifact hashes](phase-2-chromatic-integration/artifacts-sha256.json) identify the snapshot.
The complete per-host plots, scenes and SVG/PDF/PNG files are under the adjacent
`publication/{rust,python,wasm}` directories. The resvg probe and lock are retained
under `svg-probe`; the offline Linux runner is retained alongside its logs.

Reference regeneration uses `mise exec -- node tools/reference/node/chromatic.mjs`,
`chromatic-trig.mjs` and `chromatic-composition.mjs` with the locked development
workspace. The independent table extractor accepts a separate output directory and
normalizes with the repository Rustfmt edition before exact comparison. Final host
proofs use `scripts/run_primary_authoring_proofs.py` with the recorded Python, Node,
TypeScript and wasm-bindgen 0.2.128 environment. The composition reader deliberately
materializes JavaScript Number cutpoints as Python floats; Python integer keys retain
the separate exact-type contract rather than silently coercing into float cutpoints.

The reproduced non-finite category failure is retained in
[the failing counterexample](phase-2-chromatic-integration/nonfinite-reproduction.log).
Final Rust/Python/WASM tests prove both missing paint and stable eligible ordering.
The final correction preserves IEEE parameters internally while keeping direct
non-finite ramp calls invalid. The [304-case transformed corpus](../../fixtures/parity/d3-scale-chromatic/transformed.json)
covers every ramp, both reversals, positive/descending log zero, negative log values
and power overflow. The reference has 292 valid colors and 12 undefined outputs
(four lookups plus Cividis and Turbo, in both directions). Valid colors match exactly;
undefined outputs diagnose. WASM chart values deliberately use Float64Array so 1e300
is a floating observation, respecting the separate checked exact-integer source API.

The [final Linux run](phase-2-chromatic-integration/final-linux.log) passes all selected
core/color/interpolation/inventory tests, fresh actual Python methods and scenes,
updates, export, WASM build and the refreshed component benchmark. The
[final actual Node WASM run](phase-2-chromatic-integration/final-wasm.log) passes all
color/interpolation/scale/chromatic matrices, all 304 exceptional cases, five figures,
24 update steps and ownership stress. Its WASM binary was built on Linux from the
same source with the matching Rust/target standard library. Fifteen final
[Linux Rust/Python and Node comparisons](phase-2-chromatic-integration/final-publication-comparison.json)
are exact PNG RGBA and byte-identical SVG to the five previously inspected finite
fixtures. The rebuilt final native executable also rendered all five fixtures; its inspected
capture and scene-stamp log replace the earlier native artifacts.

The delayed macOS launches eventually resumed without changing code, signatures or
system settings. The final fresh primary run passes existing authoring, stages,
color/interpolation/scales/calendars, typed consumers, the full chromatic matrix,
updates, ownership and all 304 transformed cases through both actual host runtimes.
`mise run check` passes on the final production source. The focused final core run
records 14 passing chromatic/color/interpolation tests; the additional shape-foundation
tests belong to WP-S01. These supplement the earlier 352-test macOS aggregate, which predates the
last normalization fix. The final Linux focused run passes 28 tests. Scope and timing
boundaries remain explicit; no global release or sustained-load gate is inferred.

WP-S01 now consumes the accepted path foundation, followed by WP-S02's Cartesian
generators and full curves. GG-03 retains its WP-S05 symbol prerequisite.
