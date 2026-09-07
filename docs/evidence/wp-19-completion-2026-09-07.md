# WP-19 — Bounded scheduling, caches and dense representation

Status: DONE for the original owner assignment. Parent: `30ca2e8`.
Prerequisites WP-12/16/18 are complete. Expanded parity/authoring work remains separate.

## Contract and checks

| Requirements | Evidence |
| --- | --- |
| STM-04, SCN-04, FIX-11 | One active plus newest pending request; compatible older-than-latest progress, monotonic admitted/painted revisions, exact tokens, source epochs, six compatibility generations, unknown/duplicate completions, failure and disposal. Four deterministic scheduler tests include continuous arrivals and a heterogeneous dashboard. |
| SCN-04, GPU-02 | Three cache tests compare reused outputs with fresh batch tables/geometry after dataset-specific updates, style/viewport changes, corrections, reorder and retention. Equal public IDs from a different store do not reuse an unrelated cache. |
| STM-05, GPU-02 | Three density tests independently verify ordered extrema/endpoints, reversal/fallback, gap boundaries, supplied OHLC/compensated volume, retained exact source descriptions and eager source-key index construction. |
| STM-04, FIX-11 | Actual native four-chart run commits all 160 updates per chart; pending coalescing, viewport/theme stale rejection, advancing painted revisions and drained disposal. [Trace checker](../../scripts/check_scheduling_trace.py), [result](wp19/native-check.json), [raw trace](wp19/native-burst.jsonl). |
| STM-05, QLT-04 | Three actual Rust/Python/Node WASM dense fixtures preserve complete raw semantics and normal SVG bytes. Independent oracle checks every line bucket and candle population, including source keys above 2^53, grayscale, dashes and stroke widths. |

See the [contract](../scheduling-density-contract.md),
[scheduler tests](../../crates/chart-core/tests/scheduling.rs),
[cache tests](../../crates/chart-core/tests/stage_cache.rs),
[density tests](../../crates/chart-core/tests/dense.rs), and
[native example](../../examples/chart-gallery/examples/scheduling_gallery.rs).

## Validation

Environment: macOS 26.5.2 arm64, Apple M5 Max/128 GiB, Rust 1.97.1,
Python 3.14.6/PyO3 0.29.2, Node 24.14.0 and wasm-bindgen 0.2.128.
See [environment](wp19/environment.txt).

- `mise run fmt`: PASS [log](wp19/fmt.log).
- `mise run check`: PASS repository/dependency/build, Clippy, rustdoc, Kit/native examples and WASM checks; [log](wp19/check.log).
- `mise run test`: PASS **212 tests**: 174 core, 28 export, 5 external-extension, 1 native conversion and 4 Rustdoc; [log](wp19/test.log).
- `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-19`: PASS [log](wp19/bindings.log), [oracle](wp19/compare.log). All 36 prior cases, 23 actions, 47 input steps, 70 streaming steps and 3 dense fixtures execute through real Rust, Python extension and Node WASM.
- `python3 scripts/check_scheduling_trace.py docs/evidence/wp19/native-burst.jsonl`: PASS four charts, nonregressing actual presented revisions, bounded work, zero failures, stale rejection and drained disposal.

Normal publication and statistical fixtures/tolerances were not weakened. Retained
host dense JSON/SVG results reside in [native](wp19/native/dense-supplied-candles.json),
[Python](wp19/python/dense-supplied-candles.json) and
[WASM](wp19/wasm/dense-supplied-candles.json). The three PNG previews were visually
inspected: [gapped line and peak](wp19/native/dense-gapped-lines.png),
[supplied candles](wp19/native/dense-supplied-candles.png), and
[grayscale/dashed candles](wp19/native/dense-grayscale-candles.png).
PNG previews rasterize the exact saved density SVG with the supplied Noto Sans face.

## Native progress and limitations

One 100,000-row chart runs alongside 2,000/3,000/4,000-row charts. Bursts contain four
commits before yielding to the event loop. Every chart reaches revision 160 with zero
pending/active work and zero compatible lag. Each coalesces 118 pending requests,
rejects two viewport/theme-incompatible completions, and records 39 presentation
acknowledgments. The trace observes 39 positive painted revisions per chart, including
continuous progress before the final commit. All 640 accepted source commits survive.
The viewport is changed at tick 24 and destination theme at tick 48 while work is active.

The [drained dashboard](wp19/native-ui/burst-invalidation-drained.png) and
[disposed state](wp19/native-ui/disposed.png) were visually inspected. The earlier
[continuous run](wp19/native-continuous.jsonl) also advanced all four charts. Core weak
ownership tests cover disposal during active work; native disposal was checked after
drain. Actual native layout and index construction still execute on the UI thread.

An earlier burst run completed preparation but retained the initial painted UI until
an explicit window resize. Its [raw log](wp19/native-redraw-delay.jsonl) is retained.
A repeat with logged window activation and render events advanced correctly throughout;
no cause was established and no speculative fix was made. This leaves an intermittent
native redraw question for supported-platform hardening. The accepted active-window
run demonstrates bounded progress under its recorded conditions; it does not certify
inactive/occluded-window redraw policy or all desktop environments.

## Preliminary PERF-01/02

Command: `mise exec -- cargo build -p chart-export --example dense_benchmark --release --locked`,
then `/usr/bin/time -l target/release/examples/dense_benchmark`.
[Raw samples](wp19/benchmark.jsonl), [process resources](wp19/benchmark-resources.log),
[benchmark source](../../crates/chart-export/examples/dense_benchmark.rs).
These are single release CPU-stage measurements, with 10 warmup and 100 hover samples,
on the active desktop. The scene is 1200×600 logical units; the supplied export font
measurer maps identical numeric font size to points solely for this CPU study.

| Workload | Numeric ms | Layout ms | Density ms | Index ms | Hover p95 ms | Raw → rendered vertices |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| PERF-01: 10 × 10,000 | 15.772 | 5.901 | 0.870 | 58.237 | 0.0244 | 99,950 → 42,310 |
| PERF-02: 1 × 1,000,000 | 168.227 | 68.735 | 5.288 | 635.724 | 0.0156 | 999,422 → 6,799 |

Exact nearest-x lookup examines at most four candidates per series; source row
description uses the retained key index. Both runs assert complete index population
and weak-reference release of prepared/layout graphs after dropping all owners.
The million-row run reports 1,607,264 KiB RSS after index creation and 1,209,216 KiB
after disposal; `/usr/bin/time` reports maximum RSS 1,831,960,576 bytes. Allocator/OS
retention versus a process leak is not established by these samples. Index construction,
raw geometry/index memory, and UI-thread layout/index work are explicit bottlenecks.

No GPU completion, native total-frame p95, sustained update, memory plateau or
export-under-load certification is claimed. PERF-03/04/05 and G3/G4 remain open.
Known dependency advisories, upstream `block` future-compatibility warning and native
accessibility/platform limits remain unresolved. Next: WP-20 coherent live exports.
