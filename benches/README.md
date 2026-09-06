# Benchmark infrastructure

PERF-01–PERF-05 and their targets remain in the
[specification](../docs/spec/gpui-charts-specification.md). The
[WP-03 protocol and starting profile](../docs/adr/008-benchmark-protocol.md) are recorded;
none of PERF-01–PERF-05 has run. Add executable Cargo benchmark targets to the crate that owns the
measured code; this virtual workspace root is not a benchmark package.

ADR-008 records hardware/chip/RAM, OS, Rust/dependency
revisions, release profile, chart dimensions, pixel scale, workload/seed, warm-up,
timing boundaries and quantile method. Distinguish source, accepted, committed,
prepared, rendered, dropped, rejected and coalesced counts.

Later runners under `scripts/` orchestrate these targets and retain summaries under
`docs/evidence/`. Raw local traces belong under ignored `artifacts/`. Required release
evidence must be retained durably and linked. The 30-minute streaming case remains a
real sustained run; compilation and short smoke runs cannot satisfy it.
