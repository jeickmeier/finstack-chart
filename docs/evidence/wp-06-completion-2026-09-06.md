# WP-06 completion evidence — 6 September 2026

Verdict: **DONE for WP-06 foundational scales, ticks and destination layout**.
Assigned scope: **WP-06 only**, following WP-05. Baseline is committed WP-05 at `3cf1b33`;
result is that baseline plus uncommitted scales/layout, grammar integration, tests and
documentation. Dependencies, lockfile and capability visual assets are unchanged.
Requirements: SCL-01/02/04/05, LAY-01/02 and DAT-05 foundational subsets; FIX-07.

## Acceptance and independent evidence

| Deliverable | Verdict | Evidence |
| --- | --- | --- |
| Domain precedence and viewport, SCL-02 | PASS for foundational families | Exact explicit/descending domains, baseline/padding/nice order, [0,1] empty numeric fallback, fixed symmetric constants and invalid/overflow diagnostics. Nice training is independent of range and viewport. |
| FIX-07 | PASS for WP-06 scale and destination geometry scope | Empty/constant/descending linear domains; log name explicitly rejects as unsupported; large-origin UTC nanoseconds retain 17/257-tick offsets. Finite projected point/rectangle/path geometry and supported numeric/time inverses are checked. Full log mathematics remains WP-11. |
| UTC calendar, SCL-04/DAT-05 | PASS for UTC elapsed time | Independent Unix/Gregorian date literals, month/year transitions, leap days, Monday weeks, negative timestamps and source-unit fractional labels. Invalid calendar parameters, unrepresentable offsets and budgets reject. No local DST or market calendar claim. |
| Categorical identity, SCL-01/05 | PASS for bands | Explicit order/omission, known band centers/extents/gaps, lookup without numeric inverse; source dictionary code reassignment, authored row reorder/removal and historical labels retain category identity. Eligible labels train in retained relative order; multiple layer catalogs merge by label. Numeric category statistics reject. |
| Named scales and clipping, SCL-05 | PASS for independent axes | Separate left/right domains, named binding completeness/orientation errors, hidden contributions, explicit range reversal, default plot and declared figure clips. Omit splits authored lines into isolated targets; clamp and extend preserve upstream population. |
| Destination text/layout, LAY-01/02 | PASS for plain foundational layout | Known independent font metrics change margins and point positions while preserving the exact prepared Arc and operation records. Destination units/font identity/revision/size reach the provider and Scene unchanged. Four-pass bound, deterministic thinning/pressure and no-space outcomes are exercised. |
| Zoom/stat contract | PASS | Histogram [0,0.5,1,2] with edges [0,1,2] retains [2,2] counts, edges, complete members and domains. At 400 × 240 the first rectangle is [0,0,200,240]; viewport [0.5,1.5] changes it to [-200,0,400,240], using the same plot clip and targets. |
| Snapshot and failures | PASS | Layout pins its prepared source/stat snapshot and aligns targets with every Scene item. Visibility has a distinct state stamp. Font failure and projection overflow leave earlier scenes intact; invalid tiny-bound options, category/vertex/text/item budgets fail before destination callbacks where applicable. |

[scales.rs](../../crates/chart-core/tests/scales.rs) has nine independent scale fixtures;
[layout.rs](../../crates/chart-core/tests/layout.rs) has sixteen integration tests. Existing
WP-02/04/05 tests remain in the aggregate gate. The [grammar Rustdoc example](../../crates/chart-core/src/grammar/mod.rs)
now continues through measured destination layout. Exact algorithms, API compatibility,
work bounds and operation-specific tolerances are recorded in
[ADR-005](../adr/005-foundational-scales-and-layout.md).

The tests use independent expected endpoints/geometry, retained source keys/members and
provider observations. Ordinary numeric tolerance is 2e-15–1e-12; fractional destination
band centers use 1e-10 units; the 1e16 numeric-origin fixture allows one ULP (2 source units),
and large-origin nanosecond inversion allows one source tick. UTC reference literals are
cross-checked with Python UTC datetime arithmetic ([reference output](wp-06/reference-calendar.txt)). A band far-edge lookup defect and
coalescing tick candidates at a large numeric origin were corrected in code. No existing
fixture expectation/tolerance or visual baseline was weakened.

## Commands and environment

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Environment: [environment.txt](wp-06/environment.txt).

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-06/fmt.log) |
| `mise exec -- cargo test -p chart-core --test scales --locked` | PASS: nine tests during implementation |
| `mise exec -- cargo test -p chart-core --test layout --locked` | PASS during implementation; final aggregate run passes all sixteen layout tests |
| `mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings` | PASS during implementation |
| `mise run check` | PASS: repository/target isolation, local links, licenses/sources, formatting, native examples, workspace compile, strict Clippy/rustdoc and core WASM compile; [log](wp-06/check.log) |
| `mise run test` | PASS: 84 core tests (2 unit + 82 integration), 2 Rustdoc examples, none ignored; other package shells have zero tests; [log](wp-06/test.log) |
| `mise exec -- python3 scripts/check_repository.py` and `git diff --check` after final documentation | PASS; [log](wp-06/final-check.log) |

## Remaining boundaries and next action

WP-06 resolves a single-panel destination Scene with ordinary plain axes. Numerical and
service-double tests do not certify native font shaping/painting, export fidelity, screenshots,
rich/rotated text, legends, shared panels or hit testing. Existing WP-03 proof artifacts are
unchanged. Actual native chart rendering is WP-07; actual headless chart output is WP-08.
Their prerequisites are satisfied. WP-09 still needs
those consumers before actual Python/WASM runtime/wire parity can be established.

The complete scale and geometry family, explicit exclude-hidden training, alternate-unit
secondary axes, multi-panel layout, typography and extension contracts remain WP-11–14.
This package does not close every cited specification requirement or any G1–G4 cumulative
gate. Core remains dependency-free, synchronous and host-independent. These bounded-work
contracts make no performance/RSS claim. Linux runtime, hosted CI and full accessibility
remain unverified. Six existing unmaintained dependency advisories were not rescanned in
this no-dependency-change slice; the existing `block` 0.1.6 future-compiler warning remains.

Next assignment: WP-07 native vertical slice or WP-08 headless export.
This assignment stops at WP-06.
