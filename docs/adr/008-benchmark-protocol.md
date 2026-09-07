# ADR-008: Benchmark protocol and starting profile

Status: ACCEPTED protocol at WP-03; PERF-01–PERF-05 execution remains WP-22.
Date: 6 September 2026. Requirements: QLT-04; PERF-01–PERF-05.

## Reference environment

Initial machine: Apple M5 Max, 18 physical/logical CPU cores, 128 GiB RAM; macOS 26.5.2
(25F84), arm64. Rust 1.97.1, mise 2026.8.3, Xcode 26.6 / SDK 26.5. Use optimized Cargo
release builds, the committed lockfile, explicit fixture fonts and device scale 2.
The starting run used the developer's active desktop, not a thermally controlled idle
machine; unrelated applications/build activity can add noise. Report such conditions.

The intended performance reference is approximately 1,200 **chart** logical pixels wide,
60 Hz, with viewport height, device scale, power/thermal conditions and window visibility
recorded. Validate actual chart bounds, not only requested window dimensions. In the
starting proof macOS supplied a 1168 × 697 logical-pixel canvas inside the requested
1200 × 880 window, and the aspect-preserving publication preview was 1045.5 pixels wide.
This is a disclosed feasibility baseline, not a compliant PERF reference workload.
WP-22 must provision the target chart width and capture presentation timing.

## Timing and accounting contracts

Use a monotonic clock; keep sample collection outside the measured section. Record cold
start separately from warm steady state. For finite cases use at least 10 warm-up samples
and 30 measured samples for a starting profile; release performance gates require enough
samples/duration to characterize each stated workload. Quantiles use nearest rank
`sorted[ceil(p*n)-1]`. Retain raw samples, sample counts and the exact inclusion interval.

Separate these intervals and never label CPU submission as GPU frame time:

1. Arrival/validation/acceptance and queue residence; assign accepted operation IDs and
   reject/backpressure before acceptance when capacity is unavailable.
2. Transaction application and coherent commit revision; report row/byte counts and
   correction/eviction accounting.
3. Numeric preparation, domain/statistics, layout/shaping, geometry, hit-index work and
   cache/reduction effects, with relevant input/output revision stamps.
4. Native path conversion/tessellation/submission, GPU execution and actual presentation.
   Ingest-to-present ends when the compatible revision is presented, not when built or queued.
5. Input arrival → exact presented-scene lookup → overlay submission/presentation. Report
   chart hover/cursor CPU cost separately from total frame time and dispatch latency.
6. Export snapshot acquisition, layout, conversion, encoding/I/O separately. The normal
   exporter returns bytes; filesystem timing belongs to the host measurement.

Record source/retained/prepared/represented rows, path vertices/triangles, resource bytes,
index size, jobs/queue depth, accepted/committed/rejected/dropped/coalesced operations and
presented revisions. Coalescing pending preparation never means dropping an accepted data
operation. Track RSS/peak RSS and live snapshot/font/geometry counts at warm-up, fixed
intervals, steady-state tail and after disposal. Declare allocator/OS measurement tools,
sampling overhead and comparison units; RSS alone does not prove live-object release.

## Required workload protocol

Use deterministic seed `0xF157AC03` and retain the generator/version and fixtures. Add
gaps, extrema and corrections as explicit seeded schedules with independent expected
results. Compare incremental output to batch reference snapshots; timings do not prove
economic/semantic equivalence.

| ID | Protocol and evidence |
| --- | --- |
| PERF-01 | Ten lines × 10,000 observations. After preparation, move cursor on a deterministic trajectory; separately record chart hover/cursor p95 (<4 ms) and total-frame p95 (≤16.7 ms). Disclose reduction and geometry counts. |
| PERF-02 | Retain 1,000,000 observations; bound the visible representation. Measure cold/update/index costs, memory and exact retained-source lookup. Instrument source scans to demonstrate hover does not rebuild/scan full history; verify extrema/gaps/endpoints. |
| PERF-03 | 10,000 row updates/sec in declared batches, 100,000-row retained window, concurrent hover/pan for a real 30 minutes after declared warm-up. Default proposed batch: 100 rows every 10 ms. Include corrections/removals and retention boundaries. Reconcile every accepted ID and revision, queues/live resources, advancing presentation and p95 ingest-to-present ≤250 ms. Count any overload explicitly. |
| PERF-04 | 50,000 scatter points plus 12 updating charts. Sweep raw versus aggregated representation without changing source semantics; record crossover, index rebuilds, per-chart scheduling latency, disposal and memory plateau under bounded retention. |
| PERF-05 | While PERF-03 continues, export the same declared physical scene at scheduled times. Verify coherent snapshot identities and output, ingestion serviceability, added lag/peak memory and release of held snapshot resources. |

These budgets are unchanged. No short fixture run or screenshot repetition satisfies
PERF-03 or PERF-05. Missing presentation instrumentation blocks the corresponding gate.

## Starting observations

[Raw native profile](../evidence/wp-03/native-profile.txt): discard the first ten paint
samples and use samples 11–40 (30 samples), before input/resize. The example requests
40 notifications; OS/input events can create more samples. Native path conversion,
stroke expansion, tessellation and submission: p50 **3.587 ms**, p95 **3.637 ms**, range
3.395–4.237 ms. Cold font/fixture load and shaping: 3.139 ms. This measures no chart data,
statistics, hit testing, GPU completion or ingest-to-present lag. The outlined SVG has
29 path nodes and 8,550 authored outline commands, including the clip; these are not a
count of native triangles. No geometry cache or reduction is used.

[Raw headless profile](../evidence/wp-03/export-profile.txt): cold font/parse/shape 2.472 ms;
PDF text convert/write 0.969 ms; PDF outline 9.449 ms; PNG allocate/render/encode/write
11.782 ms at 300 DPI and 35.006 ms at 600 DPI. Thirty repeated prepared-tree text-PDF
conversions after ten unmeasured warm-up conversions have p50 **0.367 ms**, p95 **0.491 ms**.
These are illustrative local measurements, not export-under-streaming
or formal performance passes. Memory was not profiled in this starting run.

## WP-19 preliminary scheduling and density evidence

The [scheduling/density contract](../scheduling-density-contract.md) places bounded
executor-independent admission in core and GPUI background numeric work in the native
host. Exact raw inspection is retained beside reduced destination paint. This avoids
changing statistics or publication semantics to obtain density reduction. A per-dataset
key index removes full retained-history scans from exact hover descriptions.

[WP-19 evidence](../evidence/wp-19-completion-2026-09-07.md) records actual four-chart
progress, compatible stale rejection and preliminary PERF-01/02 CPU stages. Index
construction and retained geometry memory remain concrete bottlenecks. These finite
measurements do not satisfy native total-frame/GPU, sustained PERF-03, PERF-04/05 or
memory-plateau gates; the original budgets above remain unchanged.

## WP-20 preliminary coherent publication

[WP-20 evidence](../evidence/wp-20-completion-2026-09-07.md) deliberately pauses each
native export at captured and prepared boundaries for two seconds. Phase/resume
nanosecond timestamps separate artificial waiting from preparation/encoding; host
save time is separate. Two eight-row datasets receive atomic updates every 50 ms,
with annotation/theme changes. This finite test demonstrates live serviceability and
resource release, not the required PERF-03/05 workload, memory plateau or latency gate.
