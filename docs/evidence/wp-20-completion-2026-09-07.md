# WP-20 — Coherent exports during live interaction

Status: DONE for the original owner assignment. Parent: `e57246d`.
Prerequisites WP-13/17/18/19 are complete. Expanded parity/authoring acceptance remains open.

## Contract and evidence

| Requirements | Evidence |
| --- | --- |
| EXP-03/04, DAT-06, SCN-04, FIX-14 | Immutable multi-dataset capture retains definition, state, annotations, fonts, theme, quality, physical profile and source epoch. Manifest includes original/effective state and resource hashes. |
| STM-02, EXP-03/04 | Visible and full-domain projection works during active annotation/navigation previews and freeze; current and presented capture are explicit. Exact data rebuilds at publication resolution. |
| EXP-03, QLT-04 | Bounded jobs/rows/input bytes; success, pending/active cancellation, encoder error, observer unwind, queue/job disposal and out-of-order completion release inputs. |
| FIX-14 | Five Rust tests include two real worker barriers and 80 atomic dual-dataset commits during held exports; bytes remain equal to the original capture despite later data/definition/theme/quality changes. |
| FIX-14 | Forty steps run through actual Rust/Python/Node WASM: independent supplied-value projection, exact SVG bytes, metadata, capture during annotation preview/freeze, capacity/cancellation/raster errors and owned outputs after chart disposal. |
| FIX-14, preliminary PERF-05 | Actual native SVG/PDF workers deliberately pause twice for two seconds; ingestion advances to commit 77 before commit-0 exports complete. All 400 atomic commits finish; active PNG cancellation and frozen PNG output are inspected. |

See [contract](../live-export-contract.md), [tests](../../crates/chart-export/tests/live_export.rs),
[fixture](../../fixtures/live-export/replay.json), [independent host oracle](../../scripts/bindings/compare_live_export.py)
and [native trace checker](../../scripts/check_live_export_trace.py).

## Validation

Environment: macOS 26.5.2 arm64, Apple M5 Max/128 GiB, Rust 1.97.1,
Python 3.14.6/PyO3 0.29.2, Node 24.14.0, wasm-bindgen 0.2.128.
Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
[Environment record](wp20/environment.txt).

- `mise run fmt`: PASS [log](wp20/fmt.log).
- `mise run check`: PASS build, Clippy, rustdoc, repository boundaries, native/Kit/examples and WASM checks; [log](wp20/check.log).
- `mise run test`: PASS **217 tests**: 174 core, 33 export, 5 external-extension, 1 native conversion, 4 Rustdoc; [log](wp20/test.log).
- `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-20`: PASS [log](wp20/bindings.log), [comparison](wp20/compare.log). All 36 prior cases, 23 actions, 47 inputs, 70 streaming steps, 3 density fixtures and 40 live-export steps execute in all three runtimes.
- `mise exec -- cargo build -p chart-gallery --example live_export_gallery --locked`, then the executable in the existing local GPUI app wrapper with `artifacts/wp-20/native-final` output: actual native execution; [trace](wp20/native-live.jsonl).
- `python3 scripts/check_live_export_trace.py docs/evidence/wp20/native-live.jsonl`: PASS [result](wp20/native-check.json).

Retained actual host traces: [Rust](wp20/native/live-export.json),
[Python](wp20/python/live-export.json), [WASM](wp20/wasm/live-export.json).
The same directories retain all seven exported SVGs. Baselines and tolerances were
not weakened. Full-domain interaction regression was fixed through a pure captured
state projection; ordinary action revisions remain original provenance.

## Native artifacts and preliminary timing

The [native view after initial exports](wp20/native-ui/final-live.png) and
[frozen view after PNG completion](wp20/native-ui/final-frozen.png) were visually
inspected. Native source values remain opposite-signed at each capture. The actual
[visible SVG](wp20/native-ui/native-live-0.svg), [full-domain PDF](wp20/native-ui/native-live-1.pdf)
and [frozen PNG](wp20/native-ui/native-live-3.png) retain the expected annotation,
geometry, theme and physical size. Their manifests sit beside each output.

Inspected [SVG preview](wp20/native-ui/visible-svg-preview.png) uses resvg/usvg 0.48.1
with the exact supplied Noto Sans bytes and maps the CSS font alias to the loaded
family for this preview tool; the original SVG is unchanged. The small retained
[preview renderer](wp20/native-ui/svg-preview-renderer.rs) records this adaptation.
The [PDF preview](wp20/native-ui/full-domain-pdf-preview.png) uses
`pdftoppm -png -singlefile -r 110`. PDF is one 500×300-point page; PNG is 1000×600
pixels at 144 DPI. SVG shows viewport 2–5, while PDF/PNG rebuild full 0–7 domain.

| Output | Capture ms | Prepare ms | Encode ms | Host save ms | Completion commit |
| --- | ---: | ---: | ---: | ---: | ---: |
| SVG | 35.523 | 13.384 | 3.736 | 1.090 | 77 |
| PDF | 25.357 | 12.890 | 4.030 | 1.047 | 77 |
| PNG | 19.996 | 12.595 | 619.451 | 0.875 | 400 |

These are single debug-build samples, not percentiles or formal performance passes.
Capture includes the gallery rebuilding its supplied font bank; library request
acquisition itself does not shape/encode. Preparation/encoding exclude the explicit
four seconds of worker waiting using phase/resume nanosecond timestamps. Host saving
occurs separately on the completion callback and is included above, not library I/O.
Two jobs are the observed peak; one third request is rejected. A later running PNG
is cancelled at the first phase boundary, returns no output, and releases inputs.
Final metrics: four submitted, three completed, one cancelled, zero failed, one rejected;
pending/running/retained rows/charged bytes are all zero after disposal.

## Remaining gates

This finite two-eight-row run at 50 ms per atomic update does not meet sustained
PERF-03/05, memory-plateau, GPU/total-frame or high-load latency requirements. No Linux
runtime or additional platform certification was performed. Input charges are
conservative logical bounds; they are not process RSS measurements.

An earlier native frozen capture exposed clipping after its surrounding gallery
layout changed height. The final gallery reserves a fixed footer height; general
frozen resize/reprojection remains WP-21 work, together with WP-19's unexplained
intermittent redraw observation. G3 remains open pending that native hardening;
G4 and expanded parity/authoring gates remain open. Existing dependency advisories,
upstream `block` future-incompatibility warning and accessibility limits are retained.
Next: original WP-21 correctness/fidelity/platform work, then WP-22/23, committing
separately under the owner's instruction.
