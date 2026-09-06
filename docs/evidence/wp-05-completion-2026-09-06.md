# WP-05 completion evidence — 6 September 2026

Verdict: **DONE for WP-05 grammar and prepared-scene foundations**. Assigned scope: **WP-05 only**, after WP-02/04.
Starting checkout was clean at `d0a6c48` (committed WP-04); result is that baseline plus
uncommitted grammar/state modules, tests and documentation. No dependency/lockfile change.
Requirements: GRA-01/02/03/04/06/08, SCN-01/02 and DAT-06 foundational subsets.

## Acceptance and evidence

| Deliverable | Verdict | Independent acceptance evidence |
| --- | --- | --- |
| Typed and heterogeneous authoring, GRA-01/ARC-03 | PASS | Distinct Rust observation/annotation row types normalize into unrelated field schemas, then compile in one chart. Non-Send accessors run once; schema/count preflight suppresses invalid callback execution; typed rows release before the prepared scene exists. |
| One staged compiler, GRA-02/06 | PASS for WP-05 stages | Histogram recipe equals explicit bin + rectangle composition. Shared generated output feeds rectangles and midpoint points, alongside an independently mapped source rule. Source/generated mapping confusion, incompatible inheritance and calculation-space conflicts reject with context. |
| FIX-01 | PASS for required semantics and prepared geometry | y=[1,null,3] produces two isolated valid runs, correct source targets and one exclusion; no zero or connecting segment. Explicit gap connection, strict mode, authored order, grouped lines, invalid x and stable equal-x ordinals are checked. |
| FIX-02 and composite histogram | PASS for required semantics and prepared geometry | Edges [0,1,2], inputs [0,0.5,1,2] produce counts [2,2], exact [0,1] and [1,2] rectangles with zero baseline, and all four source members. Composite overlay has independent expected point/rule geometry and domain endpoints. |
| Statistical spaces/schemas, GRA-03/04 | PASS for identity/explicit bins | Source filters alter counts; viewport/visibility do not. Explicit affine space produces [10,12]/[12,14] edges once. Generated schema/version/kinds, operation parameters/input revisions, group order, empty bins, final edge, outliers, non-finite and filter exclusions are checked. |
| Reuse and validation, GRA-08 | PASS for builtin graph foundation | Forward dependencies work; cycles/missing IDs/unknown versions/native-only names reject. Shared row arrays and cross-call graph identity are checked. Correction, parameters and source snapshot identity invalidate; viewport/palette reuse. Schema preflight occurs before population allocation. |
| Minimal state/lifetime, DAT-06/SCN-02 | PASS for WP-05 boundary | Captured definition/data/state, per-vertex/aggregate targets and old membership survive correction/external disposal. Visibility/viewport/reset are idempotent checked actions; invalid actions and state/viewport counter exhaustion leave state unchanged. |
| Numeric/budget boundaries | PASS for preparation | Exact timestamp origin and large source values, signed-zero ties, rectangle endpoints at [-1e16,1], subnormal/large midpoints, bounded invalid-row samples, compiler quotas and last-valid-result/cache behavior. |

The [grammar module](../../crates/chart-core/src/grammar/mod.rs) contains an executable
public typed-histogram Rustdoc example. [grammar.rs](../../crates/chart-core/tests/grammar.rs)
contains 20 integration tests; [state.rs](../../crates/chart-core/src/state.rs) adds one
counter-exhaustion unit test. Tests use explicit expected values/membership/geometry,
not serializer round trips. [ADR-002](../adr/002-minimal-core-contracts.md) records the
WP-05 authoring, calculation-space, graph, geometry and state handoff.

Prepared rectangles retain exact data endpoints rather than computing data-space widths;
this avoids losing a small far endpoint through cancellation before scale projection.
The scene remains in declared calculation units. It is not a destination paint/export
artifact and has no axes or text layout yet.

## Commands and environment

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Environment: [environment.txt](wp-05/environment.txt).

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS |
| `mise exec -- cargo test -p chart-core --test grammar --locked` | PASS: 20 integration tests |
| `mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings` | PASS |
| `mise run check` | PASS: repository/target isolation, local links, licenses/sources, formatting, native examples, workspace compile, strict Clippy/rustdoc and core WASM compile; [log](wp-05/check.log) |
| `mise run test` | PASS: 59 core tests (2 unit + 57 integration), 2 Rustdoc examples, none ignored; other package shells have zero tests; [log](wp-05/test.log) |
| `mise exec -- python3 scripts/check_repository.py` and `git diff --check` after final documentation | PASS; [log](wp-05/final-check.log) |

## Remaining boundaries and next action

WP-05 implements the requested grammar/prepared-geometry slice. It does not close all
GRA-01–08/SCN-01–02 features or any cumulative G1–G4 gate. WP-06 owns linear/band/time scales,
explicit domain policy, viewport projection, ticks, guides and text-aware layout. WP-07/08
own actual chart rendering/export; existing WP-03 capability artifacts are unchanged.
No visual baseline is regenerated or inferred from numerical tests.

Automatic binning, summaries/fits, stacking/dodging/jitter, facets, full scale/style families,
custom-stat capabilities and incremental/statistical updates remain WP-10–18. WP-05 supports
only identity position and explicit bins; source filters apply only before generated rows.
Mapped size supports point radius and rule stroke width. Bin target identity is scoped to
its layer/transform, group/interval and immutable input; no cross-definition selection
migration is certified. Cache reuse is conservative, has one graph entry and remains bounded
by caller-selected compile limits. User-held prepared charts can retain historical source
snapshots until released; no RSS/memory-plateau or PERF case is claimed.

Core is still dependency-free, synchronous and host-independent. Python/WASM wire/runtime
proofs and callback serialization remain WP-09. Linux runtime, hosted CI and full accessibility
remain unverified. Prior six unmaintained dependency advisories are unchanged and were not
rescanned for this no-dependency-change slice; the existing `block` 0.1.6 future-compiler
warning remains a release risk.

Next assignment: WP-06 foundational scales, ticks and layout. This assignment stops at WP-05.
