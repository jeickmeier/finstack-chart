# WP-02 completion evidence — 6 September 2026

Verdict: **DONE for WP-02's minimal contracts**. WP-01's required dependency/build
setup exists at committed baseline `dfe38e891a9b0ee0df907924c789c3bdf4df38f6`; the checkout
was clean at assignment. Result: that revision plus uncommitted WP-02 core modules/tests,
target-isolation validation and documentation updates. No dependency/lockfile change.

Scope: ARC-01/02/03, SCN-01, BND-01 and QLT-01/05 foundational subsets; the malformed
minimal-input portion of FIX-18. No complete FIX-01–FIX-18 or G0–G4 gate is claimed.
Full data, grammar, rendering/export, wire schema and binding behavior remain later work.

## Acceptance and interfaces

| WP-02 deliverable | Verdict | Implementation/evidence |
| --- | --- | --- |
| Workspace/host isolation, ARC-01/02 | PASS | Existing seven-package boundaries retained. Core has no dependencies, I/O, system-font lookup or mandatory threading. Checker resolves macOS, Linux and WASM target graphs and rejects known host dependencies in core/export. |
| IDs and revision stamps, ARC-03 | PASS for foundations | Distinct `u64` identity types preserve zero, values above 2^53 and `u64::MAX`; revisions advance without wrapping. Scene stamps preserve separate definition/store/layout/viewport revisions. |
| Structured diagnostics, QLT-01 | PASS for minimal failures | Stable code/severity/message/correction plus contextual identities/revisions. Invalid geometry, paths, limits and resources return errors. Existing immutable scenes survive failed construction and mutation of original input buffers. |
| Finite geometry/minimal primitives, SCN-01 | PASS for initial subset | Private checked point/rectangle fields; finite bounds, rule/point sizes; owned solid primitives, numeric path buffers and plain text with explicit font references. Paint order/clips/units/stamps retained. |
| Host text/resource interfaces, ARC-02/03 | PASS for interface behavior | Synchronous non-Send hosts tested; requests preserve font identity/revision and destination units; budgets checked before callbacks; returned resource length and metric construction checked. No actual font parsing/shaping or rendering is claimed. |
| Error/resource limits, BND-01/QLT-01 | PASS for Rust construction | Aggregate item/text/path/resource budgets precede payload copying; total-resource overflow is rejected without allocating resource bytes. Rust types have no host objects. There is no wire decoder or versioned serializable chart schema yet. |
| Public use and handoff, QLT-05 | PASS for WP-02 | Public Rustdoc example executes; ADR-002 records initial contracts and deferred behavior; changelog, support matrix and ledger updated. |

The [ADR](../adr/002-minimal-core-contracts.md) is the interface/ownership handoff for
WP-03 and WP-04. Tests reside in [contracts.rs](../../crates/chart-core/tests/contracts.rs);
the public example is in [lib.rs](../../crates/chart-core/src/lib.rs).

## Commands and results

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Environment: macOS 26.5.2 arm64, Rust/Cargo 1.97.1, Python 3.14.6; the repository's
existing mise and native dependency setup was retained.

| Command/check | Actual result | Artifact / boundary |
| --- | --- | --- |
| `mise run fmt` | PASS | Core source and tests formatted. |
| `mise exec -- cargo test -p chart-core --locked` | PASS: 21 integration tests and 1 Rustdoc example, none ignored | Public contract checks listed below; no core unit tests hidden behind this count. |
| `mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings` | PASS | Focused core code/tests. |
| `mise run check` | PASS | `artifacts/wp-02/check.log`: graph/links, dependency licenses/sources, formatting, both native host example builds, workspace checks, strict Clippy/rustdoc and core `wasm32-unknown-unknown` compilation. |
| `mise run test` | PASS: same 21 core integration tests and 1 Rustdoc example across the workspace | `artifacts/wp-02/test.log`; other package shells still have zero tests. |
| `mise exec -- python3 scripts/check_repository.py` | PASS | Target graphs: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `wasm32-unknown-unknown`; graph resolution is not Linux or WASM runtime execution. |
| `mise exec -- python3 scripts/check_repository.py && git diff --check` after status/documentation updates | PASS | Final graph/local-link verification and tracked patch whitespace check. |
| `mise exec -- python3 -` (temporary workspace negative probe) | PASS: injected WASM-only `js-sys =0.3.105` rejected only for WASM core/export | `artifacts/wp-02/isolation-negative-probe.log`; copied repository, added target-specific dependency, regenerated only the temporary lockfile offline, invoked checker and asserted the target-specific failures. Real manifests/lockfile unchanged. |

The 21 integration tests exercise exact 64-bit identity preservation; revision exhaustion;
NaN/infinite coordinates; negative extents and far-edge overflow; scene order/clip/stamps;
owned-input isolation and failed replacement; missing/duplicate/wrong-kind resources;
count preflight; aggregate UTF-8 and path budgets; resource sum overflow; invalid stroke/
point sizes; numeric path ordering; resource callback suppression, borrowing/length and
error context; exact text request propagation; invalid metrics and text-request preflight.
Expected coordinates/metrics/counts are small explicit values, not generated by chart
algorithms. The font/service test doubles exercise contracts only; their bytes are not
font assets and their metrics do not certify typography.

## Limitations and next action

No new third-party packages were added. The prior WP-01 advisory findings were not
rescanned; their recorded maintenance risks remain open. Native builds still report the
existing future-Rust incompatibility in `block` 0.1.6. Some initial sandboxed mise calls
reported unrelated cache-write warnings; the commands returned success.

Linux execution, hosted CI, real font/native/export behavior, binding runtimes and
performance remain unverified. Scene descriptor validation does not verify resource
content, glyph coverage or renderer capabilities. Full groups/transforms/paints/shaping,
semantic targets, data transactions, repeated invalid-row aggregation and versioned
portable schema support remain their assigned later work. Allocation limits apply to
core processing of caller-owned Rust input, not to an unimplemented wire decoder.

Local logs under `artifacts/wp-02/` are ignored diagnostics; this report, test sources
and reproducible commands form the repository evidence for this uncommitted slice.
WP-03 and WP-04 are now ready for assignment. WP-03 is the earliest next package;
G0 still requires actual native/font/export capability proofs and the benchmark protocol.
