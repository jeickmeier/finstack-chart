# WP-18 — Streaming retention and incremental computation

Status: DONE. Parent: `6779e44`. Scope: the owner's original
WP-11–23 assignment, with one commit per completed package. Expanded parity/authoring
work and concurrent documentation remain separate. Prerequisites WP-04/10/15 passed.

## Contract evidence

| Requirements | Executed evidence |
| --- | --- |
| DAT-03/04, STM-01, FIX-08 | Explicit count/event-time retention; exact integer boundaries above 2^53; immutable old snapshots; insertion-order eviction after reorder; atomic late rejection and counted opt-in dropping; supplied nondecreasing watermark; future timestamps cannot advance it. |
| DAT-06, STM-01 | FIFO capacities in transactions/row operations/charged bytes, duplicate/reused IDs, separate queued/committed acknowledgements, backpressure, counted DropNewest, conflict and empty drain outcomes. |
| GRA-08, STM-03, FIX-08 | Exact explicit-bin chunk contribution reuse under append/correction/removal/eviction; fresh compiler equality for complete tables/domains, independent memberships and counts, invalid/outlier behavior, strict and budget failures. Other statistics declare batch fallback. |
| STM-02, DAT-06, FIX-09 | Selection eviction notifications, source and aggregate identity reconciliation, canceled invalidated preview, historical pin descriptions/ownership release, frozen old scene, inspect-history and explicit resume. Typed timestamp follow preserves exact window width and vertical state. |
| QLT-01 | A seeded 70-step replay runs in actual Rust, Python extension and Node WASM; independent Python row/retention/statistic oracle checks every selected operation. |

See the [streaming contract](../streaming-contract.md),
[nine core regression tests](../../crates/chart-core/tests/streaming.rs),
[replay generator](../../fixtures/streaming/generate.py), and
[native gallery](../../examples/chart-gallery/examples/streaming_gallery.rs).

## Validation

Environment:
macOS 26.5.2 arm64, Rust 1.97.1, Python 3.14.6/PyO3 0.29.2, Node 24.14.0,
wasm-bindgen 0.2.128; see [environment](wp18/environment.txt).

- `mise run fmt`: PASS [log](wp18/fmt.log).
- `mise run check`: PASS repository/dependency, native/Kit examples, Clippy, rustdoc and WASM checks; [log](wp18/check.log).
- `mise run test`: PASS **202 tests** (164 core, 28 export, 5 external-extension, 1 native-conversion, 4 Rustdoc); [log](wp18/test.log).
- `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-18/bindings`: PASS 36 existing cases, 23 actions, 47 input steps and 70 streaming steps in actual Rust/Python/WASM. [Log](wp18/bindings.log).
- Native gallery build and actual macOS/Metal execution: PASS scoped lifecycle below; [build log](wp18/gallery-build.log).
- Release-mode focused bin benchmark: PASS exact warm/fresh full-table/domain equality across 64 updates of 128 rows with 20,000 retained rows. [Log](wp18/benchmark.log).

Comparison retains existing tolerances (semantic 1e-12, scene 1e-10 pt) and exact SVG
bytes across hosts. Independent checks use Python math/statistics and a separately
maintained row/retention oracle; implementation numerical fixtures were not relaxed.
Final replay ends at source revision 46. The benchmark evaluated 12,288 bin values and
reused 1,267,712 versus 1,280,000 fresh evaluations; one run took 8,305,210 ns warm and
61,529,707 ns fresh. These are local preparation totals, not sustained-load certification.
Output membership materialization is included; ingestion, native paint and RSS are not.

An existing grammar cache regression initially failed after overly broad failure cleanup.
Cleanup was narrowed to the new contribution cache so failed preparation preserves the
prior valid graph cache. The existing test and new strict/budget counterexamples are
part of final validation. Clippy fixes were mechanical; no tolerances changed.

## Inspected native evidence

- [Queued acceptance](wp18/native-ui/queued.png) leaves committed/presented revisions unchanged; [backpressure](wp18/native-ui/backpressure.png) reports the next rejected acceptance.
- [Frozen eviction and historical pin](wp18/native-ui/frozen-eviction-historical-pin.png) keeps an older visible revision while live retention commits and removes the active selection. Historical values remain exact.
- [Resume](wp18/native-ui/resume-latest.png) presents the latest retained rows; [inspect-history](wp18/native-ui/inspect-preserved.png) preserves the explicit window on another commit.
- [Resolved follow axis](wp18/native-ui/follow-resolved-axis.png) captures state, prepared state and actual resolved axis all at +25..40 ns after the full lifecycle; marks agree with that interval.
- [Unpin](wp18/native-ui/unpin-release.png) removes the historical description. Core weak-reference checks verify ownership release separately.

The retained [headless watermark PNG](wp18/native/stream-advance-watermark.png)
was also visually inspected: two surviving observations, exact UTC labels and no stale
evicted mark.

An early capture [suspected mismatch](wp18/native-ui/follow-state-mismatch.png) was
initially interpreted as a stale axis because its full UTC nanosecond labels were hard
to read. A clean rebuilt process with resolved-axis diagnostics and a core layout
regression did not reproduce a mismatch. No speculative renderer fix was made and no
cause was established for the earlier visual suspicion.

## Remaining gates

This package does not claim whole-pipeline O(delta), process memory plateau, dashboard
fairness, non-starving worker scheduling or live-export certification. Those remain
WP-19/20/22. No Linux native or full OS screen-reader test ran. G3/G4 and expanded
parity/authoring gates remain open; known dependency advisories and the upstream `block`
future-compatibility warning remain unresolved. Next: WP-19 bounded scheduling/caches.
