# WP-04 completion evidence — 6 September 2026

Verdict: **DONE for WP-04 data foundations**. Assigned scope: **WP-04 only**, requirements DAT-01–06,
ARC-03 and QLT-01 data portions; FIX-08/09/16 data acceptance subsets. WP-02 prerequisite
is satisfied at `435e127`. WP-03 is now committed at `3a86189`; this result is that baseline
plus the uncommitted WP-04 core modules/tests and documentation. No dependency/lockfile change.

## Acceptance and evidence

| Deliverable | Verdict | Independent evidence |
| --- | --- | --- |
| Typed snapshots, normalized columns and schemas, DAT-01/ARC-03 | PASS for data boundary | All seven portable kinds, metadata, exact signed/unsigned extrema and formatted text; malformed field/key/name/type/length/validity/dictionary input rejected. Non-Clone, non-Send typed rows supported. |
| Ordered atomic commits, DAT-03 / FIX-08 data | PASS | Explicit two-dataset append/upsert/remove expected rows/counts; invalid final operation rolls back earlier work and permits corrected reuse of rejected ID. 250 mixed operations compared after every commit to an independent whole-row Vec model, including replacement and count retention. |
| Stable identity/order, DAT-02 / FIX-09 data | PASS | Dataset/field/row IDs above 2^53, authored reorder, reused-key ordinals and insertion-based eviction; category identity survives dictionary recoding, removal and schema-version migration; explicit category domain/reset exercised. |
| Revision/replay, DAT-03/04 | PASS for synchronous commits | Empty/net-no-op commits, exact bases/schema fences, NaN replay, payload ID reuse, FIFO count/byte eviction, stale replay conflict, monotonic epoch reset, oversized transaction rejection, and store/dataset/ordinal overflow rollback. |
| Numeric validity, DAT-05 / FIX-16 data | PASS for projection | Independent validity, NaN/infinity exclusion, bounded aggregate diagnostics and strict errors; large signed/unsigned values stay exact in source; near-maximum timestamps subtract integer origins before conversion; overflow/imprecision is diagnosed. |
| Provenance/lifetime, DAT-06 / FIX-09/16 data | PASS | Exact aggregate membership and derived model scope, stale-revision refusal, explicit missing sources, removal receipt, continued historical resolution, idempotent disposal, non-Send drop counter, and weak-reference proof that unreferenced old normalized schema/storage ownership releases. |
| Shared append storage, ADR-004 | PASS for storage property | 10,000 retained rows in ten chunks followed by ten appends: every old batch address stays identical. One upsert changes only the affected chunk; pinned old values remain valid. This is not a PERF timing or memory-plateau result. |

Implementation: [data modules](../../crates/chart-core/src/data/mod.rs),
[transactions](../../crates/chart-core/src/transaction.rs),
[provenance](../../crates/chart-core/src/provenance.rs).
The public acceptance cases are in
[data_transactions.rs](../../crates/chart-core/tests/data_transactions.rs); the private
counter-exhaustion test is in the transaction module. Expected outcomes are explicit
values or a separate row model, not serialization round trips.
[ADR-004](../adr/004-immutable-data-and-transactions.md) documents public semantics,
resource accounting, replay horizon/reset, schema migration, ownership and next consumers.

## Commands and environment

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Environment: recorded in [environment.txt](wp-04/environment.txt).

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS |
| `mise exec -- cargo test -p chart-core --locked` | PASS: 1 new unit test, 16 new data integration tests, 21 existing contract tests, 1 Rustdoc example; no ignored tests |
| `mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings` | PASS |
| `mise run check` | PASS: repository/target isolation, links, licenses/sources, format, native capability builds, workspace checks, strict Clippy/rustdoc, core WASM compilation; [log](wp-04/check.log) |
| `mise run test` | PASS: 38 core tests and 1 Rustdoc example; other package shells have zero tests; [log](wp-04/test.log) |
| `mise exec -- python3 scripts/check_repository.py` and `git diff --check` after final documentation | PASS; [log](wp-04/final-check.log) |

An initial new resource-limit test set a byte budget below the initial dataset charge,
so construction failed before the intended append. Its setup was corrected to admit
initial storage while rejecting growth; no implementation limit or fixture was weakened.

## Limitations and next action

Only the data portions of FIX-08/09/16 pass. No full canonical FIX case, PERF case or
G1–G4 gate is passed. Core remains dependency-free with no I/O, host objects or mandatory
threading. Existing WP-03 visual proof assets and dependency choices are unchanged.
The six unmaintained dependency advisories recorded by WP-03 were not rescanned for this
no-dependency-change slice and remain open. Existing `block` 0.1.6 compiler warning remains.

WP-05 owns typed-channel normalization/compiler and scene target integration. WP-06/10/11
own chart operations, gap geometry and shared projection. WP-09 owns actual wire schema,
Python/WASM execution and host lifetime errors; compiling core to WASM does not pass those
runtime requirements. WP-15/16/18 own selection-removal events, pinned-tooltip labeling,
queued ingestion, time-window retention, watermarks and incremental derived outputs.
Lookup and validation still scan retained rows; bounded category growth can copy category
metadata. Retained external snapshots and allocator capacity are not included in logical
store byte accounting; owners must release handles. Sustained-load performance is unmeasured.

Next assignment: WP-05 is ready. WP-06–09 remain gated by their other prerequisites.
This assignment stops after WP-04.
