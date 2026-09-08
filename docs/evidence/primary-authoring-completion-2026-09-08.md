# AP-00–09 — Primary authoring implementation and qualification

Date: 8 September 2026. Requirements: AUT-01–09 / FIX-AUTH00–09.
Baseline: `b631f0e6d7e41722b5774433d616f704234157d7`, plus the owner's accepted
primary-authoring plan clarifications. This report covers the current implemented
LibraryV1 capability set. It does not close D3/ggplot2 parity or G4.

## Delivered contract

`Data`, `plot`, component builders, immutable `Plot`, retained `Chart` and reusable
`Output` are the default authoring route. The Rust engine owns field resolution,
validation, statistics, geometry, runtime actions and source transactions. Python and
WASM provide ordinary language syntax and batch conversion into that same engine.
There is no second stored grammar or host compiler. Existing public low-level paths
and version-1 envelopes remain available as compatibility/specialist APIs.

The default Chart owns one store and ingestion queue. External views share committed
immutable snapshots and keep independent reducer state; a source owner is not copied
into another writer. Native scheduling retains its separate worker compiler and bounded
active/pending jobs. Original-Plot edits are definition-only changes against current
live data. Supplied fonts, native callbacks and export workers retain their separate
lifetimes. Static and live requests preserve independent Presented/Current,
visible/full-domain and interaction choices, including deferred execution after disposal.

## Evidence groups

The 35 rows in the [capability register](../primary-authoring-api.md) map to the groups
below. “Qualified” concerns access to and preservation of the delivered baseline;
future semantic features remain with their existing work packages. Existing standalone
Rust helpers remain directly public and pass their existing contracts; this refactor
does not invent a new standalone language-binding product.

### E1 — Data, identity, validation and ownership

[Core authoring tests](../../crates/chart-core/tests/authoring.rs) cover ordinary rows
and columns, all seven scalar kinds, exact signed/unsigned integers and timestamps,
nullability/metadata, source/field ownership, inferred grids, diagnostic context,
non-default recipe/axis options and shared generated-stage transformations.
[Runtime tests](../../crates/chart-core/tests/authoring_runtime.rs) cover definition
edits after append, unchanged source/keys/replay/pending queue, rollback, stale epochs,
external views, structural validation without executing native-only statistics and
older-worker/newer-commit admission with cache reuse.
[Host structural tests](../../crates/chart-core/tests/authoring_host.rs) cover component
ownership, immutable edits and checked primary interchange.

Dataset imports can preserve an explicit identity. Seeded jitter already hashes that
identity in LibraryV1; the fixture authors preserve it instead of normalizing away a
numeric difference. Two different Data owners with the same identity in one Plot reject;
clones of the same Data share safely. Default authors still receive fresh identities.

### E2 — Complete delivered component families

[Independent fixture comparison](../../crates/chart-core/tests/support/authoring_families.rs)
executes 34 primary-authored stat/position/geometry/scale/facet/composition cases against
the original definitions and unchanged expected populations/coordinates. Only allocated
opaque identities are rebased, with their relationships checked. Non-default cases
include overflow bins, affine statistics, filtering, summary quantiles, OLS, normalized
stack/dodge/seeded data/display jitter, log/symlog/point/UTC/session and secondary axes,
OHLC/volume, generated and grouped color, free facets, broadcasts, empty grids, rich
text/fonts, theme cascade, labels/leaders, insets and figure furniture.

Actual Python and Node WASM authors produce matching full semantics at 1e-12 and scene
geometry at 1e-9 using the same source inputs. The compiled external density/chamfered
geometry example uses ordinary registered components; independent count, membership,
density and shared generated-consumer checks pass. Native-only registrations remain
valid in native Rust and reject unsupported portable/export boundaries.

### E3 — Actions, inspection, streaming and host lifetimes

[Primary replay](../../crates/chart-export/tests/support/authoring_replay.rs) and the
[Python](../../scripts/bindings/authoring/replay.py)/[WASM](../../scripts/bindings/authoring/replay.cjs)
authors execute all **23 action transitions, 47 input/host-tool steps and 70 streaming
steps** from the existing fixtures. They start from primary plots and use Chart
operations and typed transaction builders. Expected retained rows, exact bin membership,
summary statistics and historical pins are checked independently; all three complete
receipts/states/semantics agree. Original fixture payloads and expectations are unchanged.
Coverage includes stale controlled replies, selection/links/echoes, gesture preview and
cancel, annotation undo/redo, exact UTC/category navigation, atomic late-data rejection,
retention, correction/replacement/removal, queue backpressure/drop accounting and replay.

The actual host proof also checks data copies/accessor execution, invalid/unsafe values,
error properties, runtime/output/request/editor/queue disposal, deferred captures,
Python interpreter detachment and explicitly freed WASM batches reaching a memory plateau.
TypeScript strict checking and mypy strict checking pass valid usage and reject the
five intended invalid component usages. Generated WASM wrapper allocations additionally
expose `free()`; unreferenced ordinary builders use host-controlled finalization.

### E4 — Native, Kit, publication and consumers

Actual macOS primary-native observations:

- The first chart displays two series, separate title/subtitle/axes/legend/caption and
  an in-plot Peak annotation. Arrow navigation advances source focus; Space selects.
  Accessibility exposes the exact selected source row and field values.
- Streaming: pin first → freeze → supplied-watermark retention retains the displayed
  revision 0 while revision 1 commits and removes the evicted selection. A second
  enqueue reports Backpressure. Commit → resume drains the queue and displays revision
  2 with three retained targets. Unpin releases the historical label; keyboard selection
  exposes `9007199254741016 Nanoseconds UTC` and value 35.
- Kit composition applies the captured Kit theme to the same primary Plot. The bounded
  x view 0..1 resets to 0..2 through the real Kit Reset view button. Target count changes
  from 10 to 12; keyboard selection exposes row `9007199254743004`, facet B and group 1.
  Regular/bold/Arabic text, facet letters, legends, notes and shared-source inset render.

The streaming check found repeated frozen-frame paint acknowledgement incorrectly
entering the resize-only path. The fix permits repaint of the exact same scene while
retaining strict prepared-input/layout checks for a different frozen projection.
[The regression](../../crates/chart-export/tests/authoring.rs) verifies repeated paints
after a live commit, retained Presented source/policy and rejection of a pending scene
without losing the previous capture policy. The updated native sequence above passed.

The export tests cover all eight capture choices with r0 painted/r1 committed,
pre-first-paint rejection only for Presented, clean/default versus legacy interaction,
cheap acquisition, immutable manifests and execution after updates/disposal. Existing
queue/cancellation/resource-budget tests remain active. The primary publication example
writes real SVG, searchable/outlined PDF and PNG at 300/600 DPI. PNG 300 and both PDF
modes rendered through Poppler were visually inspected: matching axes/positions,
explicit null gap, complete title including café/Ω and supplied-font output. Sizes are
2126×1417 and 4252×2835 for the 180×120 mm page.

Migrated consumers include the main gallery, actions, interactions, host tools,
streaming, scheduling, live export, families/facets/composition, custom extensions,
publication/preview and performance examples. Raw capability/binding/compiler fixtures
remain explicit compatibility or internal-contract tests. README, authoring guide,
release/portable/API docs and CHANGELOG lead with the primary surface. Packages remain
unpublished 0.1.0; the intended migration is 0.2.0, with removal no earlier than 0.3.0.
No deprecated public path was removed and no wire schema version changed.

### E5 — Supported-platform and performance qualification

Final Darwin arm64 checks use mise Rust 1.97.1, Python 3.14.6, Node 24.14.0,
wasm-bindgen 0.2.128, TypeScript 6.0.2, mypy 2.3.0, supplied Noto fonts and Poppler.
`mise run fmt`, `mise run check` and `mise run test` pass; **252 tests** pass with no
failures or ignored tests. The zero-test package/doc groups are not counted as feature
proofs. `check` includes repository/dependency boundaries, all targets, denied-warning
Clippy/rustdoc and wasm32 checks. Performance-feature examples are separately compiled
by the measured runner.

The offline Linux aarch64 run uses the existing `rust:1.97.1-bookworm` image, a read-only
checkout/registry and separate writable target directory. Core/export/text tests,
all-target checking and denied-warning rustdoc pass: **246 tests**. Its primary Rust
output also passes the same independent comparison against actual macOS Python and
Node WASM outputs, including the complete stream trace. No remote CI or Linux native
GPUI qualification is claimed.

Both actual runners pass on the final code: `mise run primary-authoring-proof` and
`mise run bindings-proof`. The latter retains all legacy 36 stat/geometry cases,
23 actions, 47 inputs, 70 stream steps, three density cases and 40 held-capture steps,
including byte-equal cross-host SVG and supplied-font PDF/PNG checks.

The first complete primary run is retained below together with failed final-code
measurements. Passing earlier measurements do not override a failed final gate. The existing ADR-008 baseline duration exception is retained:
60 measured seconds after 10 seconds warm-up, not a claimed 30-minute success. The
original interrupted trace and its visibility failure remain in WP-22 evidence.


| Initial primary workload | Measured result | Boundary |
| --- | --- | --- |
| PERF-01 ten lines × 10,000 rows | Hover p95 0.090501 ms; CPU+GPU frame-work p95 10.268500 ms, 600 samples | Both original numerical budgets pass in this run. 99,950 valid source vertices → 42,310 rendered vertices; no layout/index rebuild on hover. |
| PERF-02 million retained rows | Hover p95 0.134376 ms; frame-work p95 5.161209 ms | Exact retained-source lookup, at most four examined candidates; larger caller-selected geometry budgets remain explicit. |
| PERF-03 live 10,000 updates/sec | 60 measured seconds, 600,000 measured row updates, 700 total commits; actual ingest-to-present p95 109.199750 ms, maximum 147.127292 ms | Independent correction/removal/retention checks pass; no accepted-operation loss or unobserved tail, graceful drain. Duration exception retained. |
| PERF-04 50,000 scatter + 12 updating charts | Raw/dense frame-work p95 183.872126 / 156.491208 ms; hover 0.033833 / 0.029833 ms | All 13 charts present revision 120 and drain. Scatter remains exact; this compares existing line-envelope reduction. The ten-line 16.7 ms target does not apply to this dashboard. |
| PERF-05 concurrent publication | One coherent PDF, worker 103.478750 ms, peak sampled RSS 775,712 KiB | Ingestion remains serviceable and captured resources/queues/entities release. One overlap sample is descriptive, not a causal overhead estimate. |

Initial cold authoring/mount costs (single observations, milliseconds) are respectively:
ten lines 2.393792/19.087708; million rows 17.742833/212.725333;
raw dashboard 2.987583/23.712542; dense dashboard 2.959875/24.142833;
stream 2.553042/21.461834. These are cold observations, not repeated means or a claimed
speedup. Rust materialization owns columns once; later Plot/Chart/source clones share
immutable batches. The unchanged hover layout/index counters and worker-cache tests
check that authoring/JSON conversion does not enter the frame loop. Finite entities and
stream/capture weak references release; allocator RSS is not claimed fully reclaimed.

The final-code attempts in `performance-final` and `performance-repeat` did **not**
qualify: ten-line frame-work p95 was 21.095374 and 20.712834 ms, above 16.7 ms. Hover p95
was 0.148374 and 0.159125 ms, still below 4 ms. Slow samples concentrate in platform
submission: its p95 rose from 4.3975 ms in the initial run to approximately 15.9 ms,
while full GPUI draw p95 stayed below 1 ms. This localizes measured cost; it is not a
proven root cause. The final million-row run passed with frame-work p95 6.363167 ms
and hover p95 0.087001 ms.

Both final dashboard attempts finished every source commit/preparation and released all
entities, but window painting stopped before their drain: last presented revisions
103 and 11 against committed 120. The first switched to inactive; the second stopped
emitting window/paint observations. No stale or failed preparation was recorded. These
are failed native-presentation observations, not successful drain checks. Their raw
traces remain intact; no samples, assertions or thresholds were removed. A matched comparison rebuilt unchanged baseline `b631f0e` and then the final primary
code. The baseline also failed: frame-work p95 21.160960 ms over 104 displayed samples,
hover p95 0.145583 ms. The paired primary process stopped painting before the minimum
30 samples and correctly failed the checker. Thus a refactor-specific slowdown is not
established; current desktop presentation is insufficient to certify either path.
A stable visible-window rerun remains necessary. G-AUTH stays OPEN.

## Reproduction, artifact ownership and remaining gates

Executed commands include:

```sh
mise run fmt
mise run check
mise run test
mise run primary-authoring-proof
mise run bindings-proof
mise exec -- cargo clippy -p chart-gallery --examples --features performance,kit --locked -- -D warnings
mise exec -- cargo run -p chart-export --example headless_authoring --locked -- target/authoring/headless-example
mise exec -- cargo run -p chart-export --example publication_export --locked -- target/authoring/publication
python3 scripts/performance/run_native.py target/authoring/performance --stream-seconds 60
python3 scripts/performance/run_native.py target/authoring/performance-final --stream-seconds 60
python3 scripts/performance/run_native.py target/authoring/performance-repeat --stream-seconds 60 --modes lines dashboard-raw dashboard-dense stream
```

The recorded primary runner uses explicit `WASM_BINDGEN` and `TSC_JS` paths and the
installed mypy Python path; its [log](primary-authoring/primary-proof.log) records each
actual subprocess. The [Linux log](primary-authoring/linux-check.log) records its
compiler/OS, offline test/check/doc commands and primary producer; the
[cross-platform comparison](primary-authoring/linux-primary-comparison.log) preserves
the actual assertions. Other qualification logs, compressed host results and native
traces are in `primary-authoring/`; generated original outputs remain under
`target/authoring` and `artifacts/bindings`. Compressed traces contain the complete
original JSONL. [Source hashes](primary-authoring/sources.sha256) pin the tested
source, and [artifact hashes](primary-authoring/artifacts.sha256) pin retained evidence.
The public primary [guide](../authoring-guide.md) is the migration entry point.

No fixture expectations, numerical tolerances or visual baselines were regenerated to
hide a failure. Final qualification found and repaired a duplicate fixture-module
Clippy failure, a row-builder example typo and frozen paint acknowledgement; all scoped
and aggregate checks were rerun. A temporary baseline comparison reused a Cargo target;
its stale local-crate artifacts were cleared and the final primary release binaries
rebuilt, leaving the source tree unchanged.

AP-00–08 are implemented and qualified for the delivered baseline. AP-09 remains open
only for final native performance acceptance; the 35-row register retains that open row.
G4, D3/ggplot2 parity, 30-minute-duration certification, other native platforms, full
VoiceOver speech/traversal, a browser viewer and package distribution remain outside
this result. Existing dependency advisories and the `block` future-compiler warning
remain recorded release risks. There was no publication, release/version bump or
change to unrelated Phase 2 work.
