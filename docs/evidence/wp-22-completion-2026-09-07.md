# WP-22 — Measured performance and original-scope qualification

Date: 7 September 2026. Scope: original WP-22, STM-01/03/04/05, EXP-03 and QLT-04;
PERF-01–05. Prerequisites WP-19/20 and native hardening WP-21 are committed.
Expanded parity/authoring performance remains outside the owner's assignment.
The owner explicitly waived the 30-minute run. Numerical budgets were not weakened.

## Changes and interpretation

The exact inspection index now shares keyboard/highlight/semantic identity metadata
in one map and stores nearest-x candidate groups in contiguous arrays. This removes
duplicate identity trees and per-x heap vectors while keeping exact source inspection,
front-to-back ties, the 128-target bound and keyboard access to every identity.
A 300-point signed-zero/duplicate-x regression compares indexed results against the
exhaustive oracle and independently checks frontmost keys and all keyboard identities.

New release-mode native examples record full GPUI draw CPU work, platform submission,
Metal GPU execution and actual drawable presentation. The benchmark-only injected
probe changes no library or installed framework. [ADR-008](../adr/008-benchmark-protocol.md)
defines clock joining, quantiles, warm-up, dimensions, sampling and the duration exception.
The short [runner](../../scripts/performance/run_native.py) is executable documentation.
All native measurements use M5 Max/128 GiB, macOS 26.5.2, Rust 1.97.1, scale 2 and
supplied Noto Sans. The developer desktop was active; thermal/power isolation is not
claimed. Raw reports retain counts, distributions and source/scene identities.

## Results

| Case | Measured result | Boundary |
| --- | --- | --- |
| PERF-01: ten × 10,000 line observations | 600 samples; hover/dispatch p95 **0.092666 ms**, full CPU+GPU frame-work p95 **11.689291 ms**. Both <4 ms / ≤16.7 ms budgets pass. | 1,200 chart pixels; 99,950 valid vertices → 42,310 paint vertices. Full layout/index stays unchanged during hover; ≤20 candidates examined. Draw-to-display p95 28.923293 ms is separately disclosed queue/display latency; display-interval p95 16.667500 ms. |
| PERF-02: 1,000,000 retained observations | 600 hover samples; p95 **0.112876 ms**, ≤2 exact candidates examined; 604 displayed frame-work samples, p95 **5.191000 ms**. | 999,422 valid vertices → 6,799 headless paint vertices; all retained data remains available to exact lookup. Native bounds are 1,200 pixels and hover does not rebuild the index. Caller budgets are explicitly 1,100,000 potential items/path commands. |
| PERF-03: visible short run | 60 measured seconds after 10-second warm-up; **600,000 row updates**, all 700 total commit IDs/revisions reconciled; actual ingest-to-present p95 **165.759375 ms**, maximum **216.526958 ms**. Budget ≤250 ms passes. | One correction per 1,000-row batch, periodic removal, 99,999/100,000 retained rows, concurrent hover/pan and independent ordered-reference checks. No rejected/dropped accepted operation; no unobserved tail; pending acknowledgments zero after drain. The 30-minute duration is owner-waived. |
| PERF-04: scatter + 12 updating charts | 600 displayed samples per mode; raw frame-work p95 **186.402791 ms**, dense **160.765042 ms**. Hover p95 **0.030291 / 0.031458 ms**. All 13 charts present revision 120 and drain active/pending to zero. | Scatter remains exact at 50,000 points; each line reduces 9,995 → 956 vertices in dense mode. This measures the existing dashboard line-reduction crossover, not scatter aggregation. The ten-line 16.7 ms target is not applied to this heavier dashboard. |
| PERF-05: concurrent publication | The short run captures and completes one coherent vector PDF during updates; all capture/output definition, source, state, font and profile identities agree. Chart/export weak references release, all queue resource counters return to zero. | The report gives export-overlap latency, baseline samples, worker time and RSS. The tiny overlap sample is descriptive, not a causal overhead estimate. Four additional coherent exports exist in the interrupted trace. |

`wp22/` retains reports and raw events/Metal/frame traces. Native finite reports carry
all measured frame samples, full-stage timings, bounds, source/render counts, scheduling
metrics and disposal snapshots. The dense CPU workload uses ten warm-up and thirty
measured repetitions per case; p95 numeric/layout/index/density costs are respectively
14.761/3.865/51.646/0.931 ms for PERF-01 and 203.678/33.950/572.633/6.073 ms for PERF-02.
Single pre-change index samples were 51.731/601.618 ms. Those are not statistically matched
before/after runs, so no speedup percentage is claimed from them.

## Memory, failed runs and limitations

The one-minute stream's sampled memory and export overlap details are in its report.
The independent reference is checked throughout. Finite dashboard raw RSS stays around
883,056–901,744 KiB; dense readings fluctuate
738,496–816,144 KiB after warm-up. Disposal leaves zero live chart entities. Allocator/GPUI
RSS remains resident after logical release and is not labeled a leak or fully reclaimed.
The repeated million-row headless process reaches roughly 8.1 million KiB after drops;
all tracked prepared/layout weak references release. This is significant allocator/high-water
memory, not a proven low-memory plateau. Consumers must budget for the exact retained index.

The owner-stopped normal-window run contains 9,484 accepted commits and 938.4 measured
seconds, with 9,383 observed post-warm-up commits and one unobserved tail commit at termination.
Its unfiltered actual-display p95 is **27.654 seconds**, so that trace **fails** the 250 ms
budget. It also contains long intervals without display callbacks while other native
benchmark windows were used. It was not visibility-isolated; activity flags are not a
complete OS occlusion trace. The failure is retained, and no library root-cause fix is
inferred from it. A normal-window dashboard similarly stopped displaying while commits
continued and could not supply enough samples. The final sequential temporary floating
windows resolve this measurement condition and close automatically after disposal.
These valid short results apply to visible-window serviceability, not occluded-window
presentation or an unrun duration. G4's expanded production acceptance stays open.

An initial million-row native run correctly rejected the default potential-item budget;
the benchmark now explicitly selects the larger budget rather than reducing source rows.
An intermediate dashboard checker inspected tick 720 immediately after the last submission;
it now requires a dedicated post-drain sample before disposal. Final traces satisfy the
same revision/zero-resource assertions. Neither adjustment changes feature expectations.

## Validation and handoff

`mise run fmt`, `mise run check`, `mise run test` and performance-feature all-target
Clippy with `-D warnings` PASS. **219 macOS tests** pass: 176 core, 33 export, five external
extension, one native conversion and four Rustdoc. The offline Linux aarch64 headless
rerun passes **213 tests**, including the new duplicate-x/signed-zero regression.

The actual `bindings-proof artifacts/wp-22-final` run PASSes unchanged independent
Rust/Python/Node WASM comparison: 36 cases, 23 actions, 47 input steps, 70 stream steps,
three density cases and 40 live-export steps. Commands, logs, reports and SHA-256 inventory
are retained here; full generated cross-host outputs remain in `artifacts/wp-22-final`.
No remote CI execution is claimed. The six previously recorded unmaintained dependency
advisories and `block` 0.1.6 future-compiler warning remain open.

The final short-run PDF was rendered with Poppler and visually inspected: supplied text,
clipped axes, retained extrema and dense exact source lines are present. Its preview is
`wp22/stream-short/export-inspected.png`. The four interrupted-run PDFs remain retained
with coherent manifests; no additional visual claim is made for them.
Package results and current release limitations are indexed by WP-23; expanded workloads,
external publication and unrestricted production certification remain separate.
