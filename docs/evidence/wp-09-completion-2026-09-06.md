# WP-09 completion evidence — 6 September 2026

WP-09 is DONE for the portable schema and executable binding proof scope. The same authored
fixture now executes through native Rust, an actual PyO3 extension and actual single-threaded
WebAssembly in Node. Generated values/domains, stable targets, source precision, correction/
replay and state match independent expectations. Together with the preceding WP-04–08 evidence,
**G1 passes for the minimal end-to-end portable core**. G2–G4 remain open; this is not a
production Python/browser product or complete G4 built-in coverage.

## Revision, scope and environment

Started from clean commit `fac148a`, which contains WP-07/08. Result is `fac148a` plus the
uncommitted WP-09 implementation, fixtures, runner and documentation. No staging/commit was
requested. Scope: BND-01/02/03/04, ARC-02, DAT-01, QLT-01 and minimal FIX-15/16. Shared wire
contracts/session belong to core; shared publication integration belongs to export; PyO3/
wasm-bindgen and interpreter/module setup belong only to their adapters. No chart/stat/layout
algorithm is duplicated. [ADR-006](../adr/006-portable-specification-and-binding-proofs.md)
records decisions and [the portable contract](../portable-contract.md) records the exact subset.

Working directory: `/Users/jeickmeier/Projects/finstack-chart`. macOS 26.5.2 arm64, mise Rust
1.97.1, Python 3.14.6, Node 24.14.0, wasm-bindgen crate and CLI 0.2.128, PyO3/build-config
0.29.2. [Environment](wp-09/environment.txt) records actual runtime/toolchain versions.
The matching CLI was downloaded from its official immutable release into a temporary task
folder; the older global 0.2.122 tool was left unchanged. The runner requires an exact CLI
match and performs no downloads or Python installation. There was no Linux/hosted CI run.

## Result and contract evidence

| Requirement | Verdict for assigned scope | Evidence |
| --- | --- | --- |
| BND-01: versioned portable specification | PASS v1 subset | Strict normalized grammar DTOs plus separate chart/data/transaction/action/state/profile envelopes. Unknown/missing/duplicate constructs and operation versions reject. Definition round trip preserves meaning. Native accessor serialization returns a diagnostic; native resources have no wire variant. |
| BND-02, DAT-01: exact data and ownership | PASS fixture | All seven column kinds, metadata, independent validity and formatted cells survive actual host ingestion/correction; >2^53 identities, u64/i64 extremes and adjacent nanosecond timestamps stay exact decimal strings. Copies and disposal are exercised. |
| BND-03: executable Python proof | PASS CPython 3.14.6 | Actual headless extension import, construction, correction/replay/action, state restore, semantic/scene results and SVG/PDF/PNG output; concurrent Python thread progress during detached Rust work and recoverable disposal. |
| BND-04: executable WASM proof | PASS Node WebAssembly | Actual browser-target module and generated glue, equivalent fixture/results/SVG, forced memory growth, invalidated old view, independent retained bytes, 20 repeated actions, recoverable disposal and final wrapper free. No worker/shared memory baseline. |
| ARC-02: one host-independent engine | PASS | Core owns wire/data/compiler/state; export owns resources/capture/encoding; both hosts call the shared session. Target graph checks and actual headless execution pass. |
| QLT-01, FIX-15/16 minimal | PASS | Six new core tests, three export lifetime/profile/scene tests, three-runtime comparison with independent values, 13 constructor negatives in each host, state/transaction and runtime ownership counterexamples. |

The positive inputs are the committed JSON in [fixtures/bindings](../../fixtures/bindings/README.md),
not generated expected outputs. Initial x=[0.25,0.75,1.25,1.75], with the third invalid,
produces explicit-bin counts **[2,1]** for edges [0,1,2]. Correcting the second complete row
to x=1.5/y=4 produces **[1,2]** with exact members [row1] / [row2,row4]. Viewport [0.5,1.6]
clips the scene while leaving counts and trained x=[0,2] unchanged. The correction advances
store/dataset revision to `"1"`; replay is AlreadyApplied. The action advances state/viewport
revision to `"1"`. Line/point trained y changes from [1,2] to [1,4].

The four row keys are 9007199254743001–9007199254743004. Timestamp payloads remain distinct
at 1712345678901234567–1712345678901234570 ns. Source UInt64 max, Int64 min/max, categorical
codes/labels, boolean/UTF-8 fields and original `0.2500` text survive. The invalid third x
retains payload 1.25 and is still invalid; no null-to-zero substitution occurs. Core tests
also round-trip exact nonfinite Float64 bits and reject narrowed/noncanonical integers.

The comparison checks ordinary semantic numbers within **1e-10 absolute**, scene coordinates
within **0.01 pt**, and IDs, types, counts, target members, revisions and state exactly.
The outline SVG bytes also match exactly across all three actual runtimes. This measured
fixture property is stronger than the declared scene tolerance, but is not a general
cross-platform bitwise guarantee.

## Runtime artifacts and inspection

- [Comparison log](wp-09/compare.log), [native log](wp-09/native.log),
  [Python log](wp-09/python.log), [WASM log](wp-09/wasm.log).
- Final semantics: [Rust](wp-09/native/final.json), [Python](wp-09/python/final.json),
  [WASM](wp-09/wasm/final.json). Initial results, receipts, replay, action, definition and
  state snapshots are retained in the same respective directories.
- Scene DTOs: [Rust](wp-09/native/scene.json), [Python](wp-09/python/scene.json),
  [WASM](wp-09/wasm/scene.json). Each carries numeric primitives, clips, exact targets,
  fonts/hashes, resource identities and captured stamps.
- Actual SVG: [Rust](wp-09/native/chart.svg), [Python](wp-09/python/chart.svg),
  [WASM](wp-09/wasm/chart.svg), with matching hashes/bytes and vector-only structure.
- Python [PDF](wp-09/python/chart.pdf), [PNG](wp-09/python/chart.png), and
  [independent Poppler PDF render](wp-09/python-pdf-render.png) were visually inspected.
  Corrected bars, clipped line/point, axes and spacing are consistent. The boundary point
  is deliberately clipped by the core plot bounds. PDF has no raster images/fonts in this
  outline mode. Page is exactly 360 × 240 pt; 150 DPI PNG is 750 × 500 pixels. Rust PDF/PNG
  were independently dimension/font/image checked as well.
- [Python ownership](wp-09/python/ownership.json), [WASM ownership](wp-09/wasm/ownership.json).
  Python's test suppresses normal bytecode switching around the Rust call so another thread's
  progress requires detachment. WASM's test observes the constructed instance only inside
  the harness, grows memory and verifies the old buffer/view has zero length while owned
  SVG copies and later API calls remain valid. Input font mutation and output SVG mutation
  cannot change the chart. Disposal errors use the stable CHART_DISPOSED_HANDLE code.
- Inspected generated [TypeScript declarations](wp-09/chart_wasm.d.ts) and
  [JS glue](wp-09/chart_wasm.js) show string JSON inputs/results, Uint8Array font/SVG and
  generated lifetime methods. The glue uses copying `.slice()` for SVG output. These are
  retained inspection evidence; execution uses freshly generated modules in the artifact
  directory, not the evidence copy.
- [Module manifest](wp-09/module-manifest.json) records actual module/glue hashes and sizes.
  Build products stay under ignored `artifacts/bindings`, rather than committing large
  native/WASM binaries. [Python linked libraries](wp-09/python-linked-libraries.txt) show
  no GPUI linkage. Native export and browser-target core/export builds succeed.

Thirteen actual-host constructor cases cover unknown required fields, missing/future versions,
native operations/painters/resources, unknown operation versions, JS Number identities/times,
noncanonical/out-of-range integers, validity shape and duplicate keys. Core tests additionally
cover duplicate JSON fields, nesting/byte bounds, malformed transaction atomicity, revision
conflicts, native accessor rejection and state restore. A restore cannot reuse the same state
revision for different content, regress revisions or reuse the viewport revision after a
viewport change. Final review found that the publication background required an empty target entry before
the core targets; the scene DTO now aligns one target entry per item, including decorative
empties, and both a Rust test and the three-runtime checker assert that mapping.
A retained figure stays valid across updates and owner disposal, then its
weak layout reference expires after final drop.

## Validation commands and outcomes

All commands ran from the working directory above. Cargo dependencies use the checked-in
lockfile; the lockfile change only adds required dependency edges and six new PyO3/support
packages, with no previous package version changed.

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-09/fmt.log). |
| `mise run check` | PASS; repository/target graph/local links, license/source scan, formatting, native/Kit builds, all-target Clippy, rustdoc and core WASM check; [log](wp-09/check.log). |
| `mise run test` | PASS **111** Rust tests: 95 core, 13 export, 1 native conversion, 2 Rustdoc examples; [log](wp-09/test.log). Python/JS runtime assertions are additional, not counted as Cargo tests. |
| `mise exec -- cargo test -p chart-core -p chart-export --test portable --locked` | PASS six core and three export tests; [log](wp-09/portable-tests.log). |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof` | PASS actual native/Python/WASM execution and comparisons; [task log](wp-09/bindings-task.log). Runner records every build/execute command and environment. |
| `mise exec -- cargo tree -p chart-core --edges normal,build --locked` and same for export | Expected host-independent graphs; [core](wp-09/core-dependencies.txt), [export](wp-09/export-dependencies.txt). |
| `mise exec -- cargo deny --locked check advisories` | Refreshed scan still FAILS on the same six unmaintained packages; [log](wp-09/advisories.log). No new waiver or advisory ignore. |

The initial Python extension link needed PyO3's macOS dynamic-lookup helper, and the first
workspace test run exposed mise Python's missing runtime library search path. Both were
fixed in the adapter build script using PyO3's supported helpers; workspace tests were not
excluded or changed into compile-only checks. The first WASM harness assumed an older
`__wasm` JS export; it now observes instance construction in test code and does not change
the production API/glue. Existing `block` 0.1.6 future-compiler and six dependency maintenance
warnings remain release risks, separate from passing license/source and fixture checks.

## Gate and next work

G1 now has the existing data/atomicity/grammar/scales/native/headless evidence from WP-04–08
plus executable minimal FIX-15/16 host proof. Its subset boundary remains explicit: full
statistics/positions/scales/geometry, themes/facets/publication composition and complete
portable built-ins are later packages. The current schema cannot accept unknown future
constructs and silently ignore them. [The support matrix](../portable-contract.md#proof-support-matrix)
separates exposed current APIs from actually exercised runtime families.

No Linux runtime, browser renderer, hosted CI, sustained memory/performance measurement,
full script/font matrix, wheels, notebooks, free-threaded/subinterpreter support or public
package release is claimed. WP-10 is READY and unassigned: complete statistical and position
semantics. Extend and rerun the portable proof as further built-ins land; WP-21/G4 requires
full parity. G2–G4 and maintenance/release disposition remain open.
