# Deterministic streaming replay

`python3 fixtures/streaming/generate.py` recreates `replay.json` from seed 180923.
The generator maintains an independent Python retained-row oracle, declared count or
event-time policy and expected source revision for each of 70 operations. It includes
exact nanosecond timestamps/keys beyond 2^53, explicit future/watermark/late policies,
corrections, removal, reorder, queue acceptance/commit/conflict/backpressure/loss and
historical pin/follow actions. Random mutations are reproducible and bounded.

`mise run bindings-proof OUTPUT` executes the same envelope in Rust, actual PyO3 and
Node WASM. `scripts/bindings/compare.py` checks every host trace, independent row/retention
expectations, exact bin membership/count and stdlib mean/median/min/max/sum, plus exact
SVG bytes. Numerical tolerances remain 1e-12. This fixture is not a sustained-load or
worker-scheduling benchmark; those acceptance runners belong to WP-19/22.
