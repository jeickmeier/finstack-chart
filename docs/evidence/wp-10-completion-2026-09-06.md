# WP-10 completion evidence — 6 September 2026

**WP-10 is DONE for complete built-in statistical and position semantics.** The existing
shared compiler now supports count, default automatic bins, exact grouped summaries and
quantiles, intercept OLS, explicit statistical spaces, mixed-sign stack/normalize,
band-relative dodge and stable seeded jitter. FIX-02/03/04/05 pass independent expectations.
Twelve stored cases also execute in native Rust, actual Python and actual Node WebAssembly,
with equivalent schemas, values, provenance and final scene geometry. G2–G4 remain open.

## Revision, scope and environment

Starting revision: `fac148a` with the completed WP-09 changes already uncommitted. Result:
that same revision plus uncommitted WP-09/10 code, tests, fixtures and documentation.
WP-09 work was preserved; no staging or commit was requested. Assigned requirements are
GRA-03, GRA-04, GRA-05, the built-in portion of GRA-08, DAT-05, DAT-06 and QLT-02.

Owned WP-10 paths are core grammar/statistics/positions, the existing layout projection
and portable semantic DTO, semantic fixtures/tests, the existing native/Python/WASM proof
runner and contract/evidence documents. No dependency was added or updated by WP-10.
Host adapters continue calling the same core/export engine. Full scale/geometry families,
facets, typography/themes/composition, custom extensions and streaming optimization remain
WP-11–14/18; this package does not implement those independent assignments.

Working directory: `/Users/jeickmeier/Projects/finstack-chart`. macOS 26.5.2 arm64, Rust
1.97.1, Python 3.14.6 / PyO3 0.29.2, Node 24.14.0 and wasm-bindgen crate/CLI 0.2.128.
[Environment](wp-10/environment.txt) and [module hashes](wp-10/module-manifest.json) record the
actual run. The matching temporary CLI from WP-09 was reused without changing the installed
global CLI. No Linux, browser UI, hosted CI or production distribution run was performed.

## Contract evidence

| Contract | Verdict for assigned scope | Independent evidence |
| --- | --- | --- |
| GRA-03: source/statistical space and scope | PASS | Filters precede stats; zoom leaves the full OLS model and bins unchanged; explicit source filter x<=2 changes the fit. Transformed summary mean is 20, not transformed a second time. OLS transforms x/y independently. Normalized stacks publish dimensionless output metadata. |
| GRA-04: bins/count/summary/OLS | PASS | FIX-02 counts [2,2], exact final-right edge, overflow counts [3,3], automatic 30-bin conservation, constant/empty ranges; count excludes required invalid values; FIX-04 quartile 7.5, compensated sum/mean and missing empty outputs; FIX-05 intercept/slope 0.8/2.3 versus filtered 1/2. Singular/unrepresentable inputs reject. |
| GRA-05: positions | PASS | FIX-03 positive/negative endpoints +5/-5 and normalized +1/-1; zero-only and extreme finite totals; all interval endpoints train domains; non-additive encodings reject. Fixed dodge slots preserve an absent middle group. Data/display jitter is stable under row reorder and uses exact u64 seed. |
| GRA-08: built-in graph/recompute contract | PASS builtin scope | Named generated allocation sharing and unchanged graph reuse; explicit all-false specialized update flags plus exact full-recompute=true. Cached compiler outputs equal fresh batch preparation after append, correction, removal and replacement/reorder. Custom stat extension and streaming responsiveness remain WP-14/18. |
| DAT-05/06: validity and provenance | PASS | Invalid/filter populations counted; strict mode escalates. Typed schemas prevent source/generated accessor confusion and undeclared field use. Exact counts/members, model scope and immutable old-revision resolution; nonfinite geometry and imprecise values do not reach paint. Group, edge, quantile and aggregate generated-value budgets reject excess work. |
| QLT-02: independent semantics | PASS assigned FIX subset | Four canonical stored fixtures, operation-specific tolerances, reorder/conservation invariants and published finite-input D3 R-7 example values. Rust tests need no reference engine. Actual host comparison is additional evidence, not the source of expected answers. |

[The contract](../statistics-contract.md) and [ADR-005](../adr/005-foundational-scales-and-layout.md)
record algorithms, defaults, failure rules, output schemas and scope. The new portable
operation IDs use version one and reject mismatched/unknown registrations. Existing
version-one envelopes remain; exhaustive Rust enum matches must handle the new variants.
`semantics()` additionally exposes generated schemas and operation/capability records.
`GroupValue` is deserializable for explicit position order, with exact decimal integer keys.

## Validation

All final commands below completed successfully. Intermediate compile/lint/rustdoc issues
were corrected; notably a rustdoc interval needed code formatting. Boundary review also
added overflow-safe sum fallback, generated-value budgeting and dimensionless normalized
metadata before the final run. No fixture expectation or tolerance was relaxed.

| Command | Result / retained log |
| --- | --- |
| `mise run fmt` | PASS; [format log](wp-10/fmt.log) |
| `mise run check` | PASS repository/target dependency graphs, license/source checks, host examples including Kit, workspace compilation/Clippy, rustdoc and browser-target core compilation; [check log](wp-10/check.log) |
| `mise run test` | PASS **125 Rust tests/examples**: 109 core tests, 13 export tests, 1 native conversion test and 2 Rustdoc examples; [test log](wp-10/test.log) |
| `mise exec -- cargo test -p chart-core --test statistics --locked` | PASS **14 new focused tests**; [focused log](wp-10/statistics-tests.log) |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-10` | PASS final actual Rust/Python/WASM proof, including all existing WP-09 ownership/error/correction checks plus **12 WP-10 cases per runtime**; [runner log](wp-10/bindings-proof.log), [comparison](wp-10/compare.log), [Python](wp-10/python.log), [WASM](wp-10/wasm.log) |

The actual binding proof includes single-threaded WebAssembly instantiation and forced
memory growth with retained owned outputs, Python interpreter-detached progress, disposal,
13 malformed constructor cases per host, transaction/state fences and old fixture export
checks. These are actual runtime tests, not inferred from compilation.

Statistics compare at 1e-12 absolute across hosts; new destination scenes compare at
1e-10 points. IDs, counts, types, source/model membership and capability flags compare
exactly. Every new fixture's SVG bytes are identical across Rust/Python/WASM. Canonical
summary/OLS expectations use 1e-12; normalized stack endpoints use 1e-15. Original WP-09
semantic/scene tolerances remain 1e-10 and 0.01 points respectively.

Retained new semantic/schema/operation results:
[Rust](wp-10/native/statistics.json), [Python](wp-10/python/statistics.json),
[WASM](wp-10/wasm/statistics.json). Final numeric scenes:
[Rust](wp-10/native/statistics-scenes.json), [Python](wp-10/python/statistics-scenes.json),
[WASM](wp-10/wasm/statistics-scenes.json). SVGs and native PNGs are retained alongside them.
Large debug modules remain under ignored `artifacts/wp-10`; hashes, generated glue and
TypeScript declarations are retained with this evidence.

## Visual inspection and limits

Inspected actual native PNGs at 360×240 points / 750×500 pixels / 150 DPI:
[dodge](wp-10/native/statistics-dodge.png), [OLS](wp-10/native/statistics-ols.png),
[normalized stack](wp-10/native/statistics-normalize.png) and
[automatic bins](wp-10/native/statistics-auto-bin.png). Dodge shows two equal-width bars
with the missing middle slot retained; OLS is one straight fitted segment; the normalized
rectangle endpoints extend to both +1 and -1; automatic bins preserve four members over
the full [0,2] range. Shared constant styling is intentional; separate group colors/legends
are not implemented by this package. No visual baseline was regenerated to mask a failure.
This is representative statistical/position inspection, not full QLT-03 certification.

No new native UI interaction test, Linux runtime, production wheel/browser product,
free-threaded interpreter matrix, full built-in platform matrix or sustained-load benchmark
was claimed. Full recomputation is exact but has no incremental-performance claim. The six
unmaintained transitive dependency advisories recorded by WP-09 remain unresolved; WP-10
changed no dependencies and did not rescan advisories. The existing `block` 0.1.6
future-compiler warning persists in the final checks.

**Next:** WP-11 is READY and unassigned. Continue required scale and geometry families
through these shared generated/position/domain contracts, extending portable fixtures as
they land. G0/G1 retain their recorded scope; G2/G3/G4 remain open.
