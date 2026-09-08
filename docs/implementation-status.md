# Implementation status

Updated: 8 September 2026. Specification version: 0.5.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`; WP-04 at `d0a6c48`.
WP-05 is committed at `3cf1b33`; WP-06 at `f8657fb`; WP-07/08 at `fac148a`.
WP-09/10 are committed at `c5ec829`; WP-11 at `a6fb2ea`; WP-12 at `63dbcc2`.
Original reports retain the revision context from their evidence runs.

## Primary authoring implemented; final performance qualification pending — 8 September 2026

AP-00–08 are complete for the delivered LibraryV1 baseline. Data/Plot/Chart/Output,
all current component families, typed live operations, native/Kit/export destinations,
actual Python/WASM syntax and production-consumer migration are implemented. The
[completion report](evidence/primary-authoring-completion-2026-09-08.md) maps the
[35-row register](primary-authoring-api.md) to source, independent fixture and host evidence.
Revision: `b631f0e6d7e41722b5774433d616f704234157d7` plus this implementation;
the report's source inventory identifies the tested files. Unrelated Phase 2 edits remain
separate. All packages stay unpublished 0.1.0 and portable schemas stay version 1.

Final `mise run fmt`, `mise run check` and `mise run test` pass on Darwin arm64;
252 tests pass. Offline Linux aarch64 core/export/text tests/check/docs pass, 246 tests.
Actual `primary-authoring-proof` and `bindings-proof` pass. Primary execution covers
34 full component-family cases and 23 action + 47 input + 70 streaming steps in Rust,
Python and WASM, exact data, typed transactions, capture/disposal, detachment/memory
and strict declarations. Linux primary outputs pass the same independent comparison.
Native primary/streaming/Kit keyboard, selection, exact-value accessibility hooks,
historical pin/freeze/retention/queue/resume and Kit reset were executed and inspected.
SVG/PNG and searchable/outlined PDF artifacts were generated and inspected.

The native check exposed and fixed repeated frozen-frame acknowledgement entering the
resize-only path. The regression verifies repeat painting after live commit and failed
pending-scene admission without losing the retained source/capture policy. Updated
native execution passes. Field/identity validation and seeded-jitter fixture identity
were also corrected without changing existing expected values or tolerances.

AP-09 / G-AUTH remain open pending final native performance qualification. The initial
five primary workloads passed their applicable baseline protocols. A final-code rerun
recorded ten-line frame-work p95 21.095374 ms, followed by 20.712834 ms on repetition
(target 16.7 ms); hover remained below 4 ms. Platform submission dominates the slow
samples. Both dashboard attempts completed all 120 preparations per chart but stopped
painting before the drain snapshot; those attempts fail acceptance and are retained.
The unchanged baseline also failed the matched ten-line comparison at 21.160960 ms;
the paired primary run had fewer than 30 displayed samples. A refactor-specific
regression is not established, and a stable visible-window rerun has been requested. The ADR-008 baseline 30-minute
exception, full-assistive/other-platform limitations, dependency advisories and expanded
parity/G4 requirements remain explicit. Next action: complete the requested visible
native measurements, preserve failed traces, then close or explicitly leave G-AUTH open.

## Primary authoring implementation in progress — 7 September 2026

Owner authorized the complete AP-00–09 plan. AP-00's baseline/public contract,
35 source-owner coverage rows, migration policy and acceptance handoff are recorded in
[the capability register](primary-authoring-api.md). Baseline remains
`b631f0e6d7e41722b5774433d616f704234157d7`; earlier owner-authorized planning edits are
preserved. No publication or parity-kernel implementation is implied.

AP-01 extracted typed Chart store/reducer/compiler/queue/query ownership; legacy JSON
Session now forwards to it, retaining eager portable preparation. New structural
validation accepts registered native-only operations without running statistics.
Owned and external-source routes remain distinct. Four new FIX-AUTH01 runtime tests
pass, including independent external views, native-only execution counting,
definition-only data/replay/queue preservation, typed/legacy histogram/replay parity.
The existing portable/streaming/scheduling/stage-cache selection passes 22 tests.

AP-02 has the first Data columns/rows, owner-scoped field/layer handles, immutable
Plot and inherited source aes/layer/histogram route. Four FIX-AUTH02 core tests pass:
independent geometry/bin membership, nullable/exact integer/timestamp preservation,
foreign-handle and missing-field diagnostics, and independent dataset overlays.
Native ChartInput::from_plot and the headless Output destination compile. Component
and publication tests, native lifecycle integration, complete runtime/binding/consumer
migration and performance acceptance remain in progress. No cumulative gate closes.

Commands on Darwin arm64 with mise Rust 1.97.1: `cargo metadata --no-deps --format-version 1 --locked`;
`cargo test -p chart-core --test portable --test streaming --test scheduling --test stage_cache --locked`;
`cargo test -p chart-core --test authoring_runtime --locked`;
`cargo test -p chart-core --test authoring --locked`;
`cargo check -p chart-export -p gpui-charts --locked` (all via `mise exec --`).
Next action: finish real publication proofs and component/runtime coverage, then the
remaining ordered packages. G-AUTH remains OPEN; the register records partial coverage.

### Continuing authoring implementation evidence

The later primary-authoring slice adds regression coverage for original-Plot edits after
live append, retry/queue preservation, retention rollback and queued epoch conflicts.
Automatic timestamp mappings now share an exact integer origin across owned datasets of
the same representation. Shared automatic color catalogs are trained before per-layer
color assignment. Facet authoring rejects ambiguous same-ordinal/different-name fields;
matching datasets must currently align facet-field schema positions or explicitly use
broadcast/panel targeting. Explicit timestamp origins remain authoritative and incompatible
origins reject. Shared automatic colors use ordered catalog union per coherent snapshot;
an explicit color domain fixes category-to-palette assignments across changing catalogs.

`mise exec -- cargo test -p chart-core --locked` passed the entire core suite and doctests
on this slice. `mise exec -- cargo test -p chart-export --test authoring --locked` passed
4 tests, including the 8-case Presented/Current × visible/full-domain × interaction matrix,
with requests executed after runtime disposal. Acknowledged destination layout is retained
as capture provenance and as the default Presented layout policy, with explicit publication
size/font/options applied. Native input now has tooltip/control/accessibility/edit/density
builders, and Kit has a primary plot mount helper. Native committed receipts remain committed
when later scheduling fails; that separate failure uses the existing diagnostic surface.

`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p gpui-charts-kit --all-targets --locked -- -D warnings`
passed before the subsequent host-binding dispatch additions. The refreshed
`mise exec -- cargo run -p chart-gallery --example primary_authoring --locked -- --headless`
produced updated `target/authoring/native-primary.{svg,pdf,png}`; PNG and a Poppler-rendered
PDF (`/private/tmp/finstack-primary-refreshed.png`) were visually inspected and the Peak
annotation is inside the plot. These checks do not certify native input/lifecycle or bindings.

AP-07 now has shared Rust host dispatch into actual typed component/draft builders,
typed host runtime/transaction/capture adapters, and primary Plot version-1 interchange
with names/profile/exact data. PyO3 and wasm-bindgen owned handles now expose these paths;
`packages/python/finstack_chart` and `packages/wasm/authoring.cjs` provide ordinary
column/row/component syntax without a separate grammar compiler. Source arrays cross
typed native vectors; Python integers and WASM BigInt retain exact 64-bit payloads.
Output resources, immutable requests/frames and deferred export queue/jobs are separate
owned handles with explicit disposal. Existing portable Chart APIs remain available.

The first smoke checks have been superseded by committed
[scripts/run_primary_authoring_proofs.py](../scripts/run_primary_authoring_proofs.py)
and `mise run primary-authoring-proof`. On Darwin arm64, Rust 1.97.1, Python 3.14.6,
Node 24.14.0, TypeScript 6.0.2, mypy 2.3.0 and wasm-bindgen 0.2.128, the actual Rust
executable, PyO3 extension and Node WASM module passed shared primary-author assertions.
The runner compares initial/final source semantics, scenes, actions, navigation,
transactions/replay, exact 64-bit timestamp/integer/null/category values, summary means
and OLS against independent expectations. Scene/navigation tolerances are 1e-9 destination
units and statistical tolerances 1e-12; opaque allocated identities are normalized only
after source identity relationships are checked. Actual SVG/PDF/PNG encoders execute.

Owned editor handles, named runtime commands, event-time retention, structured error
properties and separate public component classes now have Python stubs and TypeScript
declarations. JavaScript offers camelCase aliases. Positive typing examples pass;
five negative examples reject misplaced titles/subtitles/annotations/legends and data
replacement through definition edits. Host proofs also cover input mutation and one-time
row callbacks, display/boolean/null metadata, invalid/foreign fields, queue acceptance
versus commit, stale queued bases, update-versus-batch domains, count/event retention,
stale state commands, eight capture combinations, editor/request survival after disposal,
export-job limits/cancellation and repeated disposal. Python made 710 independent thread
steps during one 100,000-row Rust semantic call (0.891 seconds, debug build); this is a
detachment observation, not a performance claim. WASM memory stayed 6,815,744 bytes across
six batches of 100 create/dispose cycles with 1,000 rows, explicitly freeing all fluent
intermediate handles. A prior GC-timing-only run was unstable; garbage collection timing
is not a deterministic disposal guarantee.

Evidence: `target/authoring/{environment.json,*primary*.log,native,python,wasm,typing}`.
The Python/WASM PNGs were inspected; a fixture-only label offset was corrected so the
annotation remains inside the plot, then all three runtime fixtures were rerun.
Command: `WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen TSC_JS=/Users/jeickmeier/.npm/_npx/e04ecd76da0b5726/node_modules/typescript/lib/tsc.js PYTHONPATH=/Users/jeickmeier/.cache/uv/archive-v0/PKSDgIDzoTfeXd75NY8Rf/lib/python3.14/site-packages mise run primary-authoring-proof`.
`mise run fmt`, Python/export all-target Clippy and wasm32 all-target Clippy passed.
Baseline revision remains `b631f0e6d7e41722b5774433d616f704234157d7` plus these uncommitted
changes. The runner also now exercises the compiled custom density-histogram extension through
primary components in all three hosts. `custom_stat(...).field_parameter(name, field)`
resolves owner-checked source mappings only during authoring. The example owns its
`density_histogram`/`chamfered_bars` helpers and explicit registry installation/loading;
no parameter payload installs code. Independent counts/members/density and both generated
consumers match; the headless image was inspected. Native-only variants build as Rust
plots but reject portable runtime/serialization and headless preparation. Full remaining
component/standalone coverage, native lifecycle qualification, consumer migration and
AP-09 performance/platform gates remain open. The existing aggregate compatibility
runner also passed with
`WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof`:
36 stat/position/geometry cases, 23 action transitions, 47 input steps, 70 streaming steps,
40 held-capture steps, extension/schema/capability failures, rich text/facets/density and
independent vector/raster assertions across actual Rust/Python/WASM. Evidence is under
`artifacts/bindings`. This establishes preserved legacy contracts; the remaining delivered
component families still need primary-builder evidence. This shared host proof does not close every capability row or G-AUTH.

The current working tree adds grammar/stat/position/axis/color/text/theme/facet/figure
builders, named transforms, generated color mappings, grouping overrides, runtime
transaction/configuration helpers and immutable component edits. Compiler structural
validation now covers facets, axes, figure references and color stages without executing
statistics. Grid builders infer the complete row/column product. Ordinary charts and
facets reuse the same legend painter. Future GG mapped-symbol/linetype kernels remain
unimplemented and their coverage stays open.

Native ChartInput/ChartView now adopt the same typed Chart; GPUI retains tasks, a separate
worker compiler, frame acknowledgements and native resources. Committed source admission
is separate from worker preparation; an older admitted result cannot replace newer
committed data. Paint acknowledgement retains actual inspection state for Presented
capture. Supplied fonts have an automatic native resource constructor. Full native
lifecycle, sustained workloads and bindings still require requalification.

Additional commands, all in the repository on Darwin arm64 with mise Rust 1.97.1:

- `mise exec -- cargo test -p chart-core --test authoring --test facets --test layout --locked`:
  36 passed at that slice (7 authoring, 9 facets, 20 layout).
- `mise exec -- cargo test -p chart-core --test authoring --locked`: subsequently 8 passed,
  adding independent shared-transform/generated-color/reuse assertions.
- `mise exec -- cargo test -p chart-core --test authoring_runtime --test portable --test scheduling --test streaming --test stage_cache --locked`:
  27 passed, including older-worker/newer-commit and disposal checks.
- `mise exec -- cargo test -p chart-export --test authoring --locked`: 3 passed, including
  real SVG/PDF/PNG publication and ordinary/faceted legend text. A missing facet selector
  was corrected in the test and now also rejects during structural build.
- `mise exec -- cargo run -p chart-gallery --example primary_authoring --locked` and
  `mise exec -- cargo build -p chart-gallery --example primary_authoring --locked`:
  built the shared native/publication example. A sandboxed launch could not connect to
  macOS services; the compiled local example was subsequently launched outside the
  sandbox and as a temporary local app bundle for visual inspection. The actual native
  chart displayed both colored series, title/subtitle, axes, legend and caption. Keyboard
  interaction was not verified by this inspection.
- Artifacts: `target/authoring/primary-authoring.{svg,pdf,png}` and
  `target/authoring/native-primary.{svg,pdf,png}`. PNG and Poppler-rendered PDF were inspected.
  Inspection identified an annotation offset outside the default plot clip; the example
  now places that label inside the plot and awaits refreshed image inspection.
- `mise run fmt` passed at the recorded intermediate slice. Focused Clippy identified
  new clone-on-Copy/large-enum issues; fixes are applied but the final rerun is pending.
  Latest `mise exec -- cargo check -p chart-core --locked` passes after immutable edit work.

These counts precede the newest edit/configuration changes; they do not certify those
changes, actual Python/WASM execution, complete native lifecycle or performance gates.
Next: behavioral coverage for primary edits/transactions/configuration, remaining
component ownership checks, native/export consolidation, then AP-07–09. G-AUTH stays OPEN.

### Primary component-family qualification

The primary Rust, Python and WASM authors now execute 34 existing independent
statistic/position/geometry/scale/facet/composition cases through explicit builders.
The shared Rust fixture checks exact source materialization, full semantic output and
resolved marks against the original fixture definitions, rebasing only allocated
layer/dataset/field/scale identities. Python/WASM use ordinary columns and explicit
components, then compare complete semantics and scenes with Rust at the existing
1e-12/1e-9 tolerances. No fixture expectations or visual baselines were changed.
Coverage includes overflow/automatic bins, summary/count/OLS and affine/filter variants,
stack/normalization/dodge/data/display jitter, log/symlog/point/UTC/session scales,
area/ribbon/cells/OHLC/volume, group color and affine secondary axes; seven facet cases
cover catalogued empty grids, free axes, broadcasts/targeted layers and group/facet/chart
stat populations. Three publication cases cover named themes, gradients/symbols/dashes,
explicit regular/bold/Arabic fonts, rich titles/axis labels, ordered notes, panel letters,
callout/coordinate-space labels and insets using existing prepared layers.

This exposed and fixed two missing primary controls (`LayerBuilder::color_group` and
`LegendBuilder::untitled`) and bar/volume defaults discarded by `.aes(...)`. Bar baselines
are now recipe options used when y2 is unmapped; explicit y2 mappings retain precedence.
Python/WASM dispatch and declarations include the added controls. Native editorial
composition, OHLC/volume and grid artifacts were inspected; the six-panel grid's initial
400×260-point page correctly reported layout pressure and was enlarged to 600×540 points
for its proof artifact. Editorial composition uses 180×120 mm at 96 DPI.

Commands: `mise exec -- cargo test -p chart-core --test authoring_families --locked`;
`mise run primary-authoring-proof` with the task-local tool paths above; after extending
facets/composition, each actual Rust/Python/WASM producer and
`mise exec -- python3 scripts/bindings/authoring/compare.py target/authoring` ran again.
The comparison passed all 34 primary families. Outputs are
`target/authoring/{native,python,wasm}/{families,family-scenes}.json` and native PNGs.
The expanded WASM workload plateaued at 19,070,976 bytes in its six explicitly freed
batches; its higher peak includes the additional families and supplied fonts. The latest
Python detachment check recorded 627 thread steps over 0.786 seconds (debug build).
Full native lifecycle/input/Kit qualification, remaining option/runtime coverage,
consumer migration and AP-09 performance/platform gates remain open.

### Consumer migration in progress

The main native gallery now uses ordinary `Data::rows`, named recipe builders and
immutable Plot edits, with owner-derived field handles for inspection. Family/facet/
composition galleries reuse explicit recipes in `examples/common/authoring_fixtures.rs`;
the independent raw-fixture comparison remains in tests. Kit mounts these primary plots
through its supplied-theme input helper. Extension, action and interaction galleries now
use primary construction. The linked host-tools example shares Data between charts,
builds callouts, derives actual layer identities and uses `link` through native
`capture_link`/`resolve_link`; captured host-command export still uses its specialist
snapshot boundary to retain the event's exact scene.

The curated prelude exposes builders and relevant option enums without host dispatch or
ambiguous profile exports. Root Rustdoc and README now lead with Data/Plot authoring;
`docs/authoring-guide.md` documents current components, runtime/capture ownership, host
usage and the 0.2/0.3 additive migration policy. No old public module was removed.
Migration exposed named-axis insertion discarding prior primary-axis settings and missing
Rust edit x/y-axis conveniences; both are fixed. The meaningful regression checks
non-default bar baselines before/after aes, retained primary axes and stable edit handles.

`mise exec -- cargo test -p chart-core --test authoring --test authoring_families --locked`
passed 12 focused tests plus two corpus tests covering 34 cases. `cargo check` passed
main/family/Kit/extension/actions/interaction/host-tools consumers via mise. `mise run fmt`, repository graph/link validation and
`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p chart-gallery --all-targets --locked -- -D warnings` passed on the latest slice. Streaming,
scheduling/live-export/benchmark consumers, native interaction/lifecycle/Kit execution,
complete option/runtime coverage, docs/type/schema validation and AP-09 measurements
remain in progress. G-AUTH stays open.

### Primary runtime consumers and documentation

The streaming gallery now owns one Chart and uses named transaction/retention and
queue builders. The scheduling gallery and finite/sustained native benchmarks author
ordinary columns and components, commit through the mounted Chart, and keep explicit
CPU/GPU/presentation instrumentation. The live-export gallery and sustained benchmark
use Output::live_request; annotation definition edits and later commits cannot alter an
accepted request. The publication tutorial now uses Data rows, Plot and Output with
physical dimensions, supplied fonts and text/DPI options. Raw fixtures remain only in
compatibility/diagnostic proof programs and internal stage benchmarks.

Primary `external_view`/`accept_from` now preserve authored handles and definition edit
ownership while sharing committed snapshots and keeping independent reducers. Rust
checks prove pointer-shared snapshots, explicit source admission, rejection of a copied
writer and continued view preparation after writer disposal. Python/WASM expose the
same operations with fresh actual-runtime checks included in the primary proof runner.

README, the authoring guide, release guide, portable compatibility contract, historical
alpha API guide and changelog now lead with the implemented primary surface. The planned
0.2.0 migration retains old public paths and wire version 1; removal is no earlier than
0.3.0. Current packages remain 0.1.0 and unpublished. No separate Phase 2 semantic gate
is advanced. Performance requalification and remaining integrated/native checks follow.

Validation so far: native example compilation and all-example Clippy with
`--features performance,kit --locked -- -D warnings` pass. The focused core external-view
suite passes all eight tests. These are intermediate results; final aggregate and actual
host/native/performance results will be recorded separately below.

## Grouped aesthetic contract recorded in the plan — 7 September 2026

Added section 3.5 to the [primary authoring plan](impl_plans/primary-authoring-api-plan.md)
with the proposed grouped line/point example and independent group/color/fill/shape/
linetype/size/linewidth/alpha channels. The contract distinguishes group membership
from scale-selected appearance, supports same or different mapping fields, preserves
plot/layer inheritance and overrides, and keeps constants and legends with their
respective builders. Mapped styles must not require a separate layer per category.
AP-03 acceptance now covers actual resolved styles, line membership, inference and
override rules, legend keys, inspected output and AP-07 host proofs. GG-02–05 retain
semantic ownership; the example is explicitly prospective.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; outcome is uncommitted
plan/ledger edits preserving earlier work. AP-00–09 remain PLANNED, GG-02–05 remain
NOT STARTED and G-AUTH remains OPEN. Next action: AP-00's API walkthrough includes
section 3.5. Validation: `mise exec -- python3 scripts/check_repository.py`, temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py`, and
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/implementation-status.md`
passed on Darwin arm64. Documentation-only checks; no proposed API/runtime tests ran.

## Grouped aesthetic API clarification — 7 September 2026

Reviewed AUT-03 / GG2-03 against current `SourceAes`, `ColorEncoding`, `ThemePatch`
and GG-02/03. The planned primary `aes()` supports independent group, color, shape
and linetype mappings, inherited or overridden per layer. Group selects connected
observations/statistical populations; scales choose visual encodings. Current source
mappings provide explicit group and size; separate color encoding supports source
categories and prepared groups. Symbols/dashes are currently plot/layer theme values,
not mapped shape/linetype channels. GG-02/03 remain NOT STARTED; no inferred grouping
from color or complete mapped symbol/line-style support is certified by this answer.

Revision `b631f0e6d7e41722b5774433d616f704234157d7`, with existing uncommitted planning
edits preserved. Evidence is source/plan inspection only; no runtime tests ran.
Validation: `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
passed. Next action remains the recorded AP-00 walkthrough; mapped aesthetic semantics
retain GG-02/03 ownership and primary builder integration in AP-03. G-AUTH remains OPEN.

## Implemented builder inventory and future integration rule — 7 September 2026

Added a source-linked 16-family component inventory to section 3.3 of the
[primary authoring plan](impl_plans/primary-authoring-api-plan.md): explicit figure
furniture/callout/inset builders, existing marks/statistics/positions, transforms,
scales/axes/legends, facets, theme/text/layout, runtime configuration, host hooks,
publication and registered components. Builder names remain proposed. The inventory
separates implemented fixed annotations from future mapped labels and other parity
requirements; the introductory sketch now uses the existing Editorial theme.

Section 3.4, AUT-01 and ADR-013 require every future feature to extend an existing
typed builder or add a focused component within the primary API. Its own package
includes primary examples, applicable bindings/serialization updates and behavioral
evidence. Runtime operations remain Chart/destination methods. AP-00/04/08 acceptance
now carries inventory expansion, composition evidence and the ongoing integration rule.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
plan/specification/ADR/ledger edits, preserving earlier changes. Source inspection
covered the linked current core grammar/composition/layout/runtime and native/export
declarations. No runtime implementation or schema changed; AP-00–09 remain PLANNED
and G-AUTH remains OPEN. Next action: AP-00's option-level register and API walkthrough.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` and temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed documentation links,
package/fixture/ledger and requirement/dependency consistency.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime test or proposed-builder compilation ran; this is planning evidence.

## Separate annotation and figure builders — 7 September 2026

Updated the [primary authoring plan](impl_plans/primary-authoring-api-plan.md),
AUT-04 and ADR-013 to the owner's direction: labels are x/y-positioned annotations;
titles, subtitles, x/y axis labels and legends have separate builders. Section 3.1
owns the proposed routes and supersedes the older combined-labels sketch. AP-00/03/04
now require separate component examples, correct annotation identity/provenance,
automatic legends and shared text/guide implementation. FIX-AUTH04 includes component
isolation and compile-fail checks against misplaced labels-builder setters.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/specification/ADR/ledger edits, preserving prior review and plan updates.
No implementation or wire format changed; AP-00–09 remain PLANNED and G-AUTH OPEN.
Next action remains AP-00, using the revised builder boundaries. Intended acceptance
has not been executed; this is a documentation-only update.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed; the temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed the unchanged package,
requirement and dependency checks. `git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime or proposed-builder compilation was run.

## Primary authoring plan review findings incorporated — 7 September 2026

Updated the [plan](impl_plans/primary-authoring-api-plan.md) and
[ADR-013](adr/013-primary-authoring-api.md) at the owner's request. APR-01–04 are
addressed in the planning contract: definition-only edits preserve current data;
owned ingestion and external committed sources retain one commit authority, with
independent view/worker/export lifetimes; Presented/Current export basis is separate
from navigation/interaction policy; structural build and native/portable validation
are separate. AP-00/01/02/05/06 now carry the corresponding handoff and regression
acceptance. The AP-00–09 sequence and AUT-01–09 remain unchanged.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/ADR/review-follow-up/ledger documentation. Prior review edits were preserved.
The plan records the reviewed completed original-scope baseline, without reclassifying
expanded parity or original acceptance. No implementation, dependency, wire schema,
fixture or release setting changed. Review findings are addressed in the plan only;
all AP packages remain PLANNED and G-AUTH remains OPEN. No runtime tests or proposed
API execution ran for this documentation-only update. Next action: AP-00's baseline
and public-contract handoff, then AP-01 under the clarified ownership map.

Validation in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed repository/dependency
and local-link checks; `mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed
ten packages/fixtures/ledger rows, nine requirement mappings, fifteen capability
families and an acyclic sequence. An inline `mise exec -- python3 -` check verified
the four review findings' representation and updated-document whitespace.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/implementation-status.md`
passed. These are documentation checks, not AP implementation or runtime evidence.

## Primary authoring plan checked against the current library — 7 September 2026

Outcome: reviewed the plan against clean revision
`c773fcba9a8c846322d08c9c1203fce415e91328`, including the now-completed original
WP-17–23 source and contracts. The original-WP assumption matches the ledger's
original-scope completion records; expanded parity and G-AUTH remain open.
HEAD later advanced to `b631f0e6d7e41722b5774433d616f704234157d7`; its changes were
unrelated skill additions and did not alter the reviewed library or plan files.
[Review evidence](evidence/primary-authoring-plan-review-2026-09-07.md) records
four actionable handoff gaps: APR-01 definition-only edits must preserve live data;
APR-02 map externally managed sources and worker/view lifetimes before runtime
consolidation; APR-03 retain Current/Presented export bases; APR-04 separate native
execution from portable capability validation. AUT-01/02/03/05/06/08/09, DAT, STM,
ARC, EXP and BND contracts are affected. These are plan gaps, not new demonstrated
runtime regressions. Overall architecture/reuse direction passes; the specific
handoff contracts remain Fail or Uncertain as detailed in the report.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:

- `mise exec -- cargo metadata --no-deps --format-version 1 --locked`: succeeded;
  inspected nine workspace packages and direct dependency ownership.
- `mise exec -- cargo test -p chart-core --test streaming --test scheduling --test stage_cache --locked`:
  **16 passed**, zero failed (nine streaming, four scheduling, three cache).
- `mise exec -- cargo test -p chart-export --test live_export --test extensions --locked`:
  **9 passed**, zero failed (five live export, four extensions).
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown links; graph checks do not establish runtime support.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`:
  passed. An inline `mise exec -- python3 -` check passed new-report whitespace and
  the four APR finding identifiers; no feature acceptance was implied.

Result: uncommitted review report and this required ledger entry only. Plan,
implementation, fixtures, dependencies and prior evidence were preserved. No proposed
API compilation, actual Python/WASM proof, native/artifact inspection, Linux run or
sustained benchmark ran. Next action: tighten AP-00/01's ownership/edit/validation
handoff and AP-06's capture contract using APR-01–04; retain AP-00–09 and all open gates.

## Primary authoring API planning — 7 September 2026

Planning complete in the [primary authoring plan](impl_plans/primary-authoring-api-plan.md).
Owner direction: concise authoring is the main public API and every feature runs
through it. Per explicit instruction, the plan assumes original WP-01–23 complete;
this is a prospective input, not a change to their recorded implementation states.
Additional D3/GG work retains its scope/owners and must integrate through the primary
API when delivered. No historical gate or feature is certified by this assumption.

Specification/main-plan/migration versions advance together to 0.5.0. AUT-01–09,
FIX-AUTH00–09 and G-AUTH govern capability-complete authoring, a shared typed runtime,
data/identity, design, live interactions, native/export, bindings and migration.
[ADR-013](adr/013-primary-authoring-api.md) records accepted ownership: builders in
chart-core, existing normalized engine/runtime reused, host resources in adapters,
standalone utilities directly callable, no new façade crate or second compiler.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60` plus live edits. Result:
uncommitted plan/ADR and focused authority/traceability/ledger updates. Existing source,
reviews and parity plans were preserved. No dependencies, implementation, fixtures,
portable versions or release/publication settings changed. Those planning-only statements retain their original revision context. The current
package states below are updated by the implementation evidence above.

| Package | State | Prerequisites | Next action / evidence |
| --- | --- | --- | --- |
| AP-00 — Public contract and baseline register | COMPLETE | Assumed completed original WP-01–23 | Contract/register and migration policy in primary-authoring-api.md; FIX-AUTH00. |
| AP-01 — Shared typed runtime | COMPLETE | AP-00 | Typed owned/external Chart, independent worker caches and legacy forwarding qualified; E1/E3/E4. |
| AP-02 — Primary data/plot/layer path | COMPLETE | AP-01 | Ordinary Data/Plot route, exact ownership/diagnostics and actual native/publication examples qualified; E1/E4. |
| AP-03 — Complete grammar and aesthetics | COMPLETE | AP-02, applicable semantic owners | All delivered grammar/scale/extension families qualified through primary authors; E2. |
| AP-04 — Design/composition/specialists | COMPLETE | AP-03, applicable semantic owners | Facets/themes/text/composition and retained standalone helpers qualified; E2/E4. |
| AP-05 — Live Chart features | COMPLETE | AP-04 | 23 action, 47 input and 70 streaming steps, edits/rollback/replay and native scheduling qualified; E1/E3/E5. |
| AP-06 — Native/Kit/export integration | COMPLETE | AP-05 | Actual native/Kit input hooks, capture matrix, deferred resources and inspected SVG/PDF/PNG qualified; E4. |
| AP-07 — Host-native Python/WASM authoring | COMPLETE | AP-06 | Actual Rust/Python/WASM family/runtime proofs, typing, exact values, disposal/detachment/memory and compatibility pass; E2/E3/E5. |
| AP-08 — Consumer/docs/API migration | COMPLETE | AP-07 | Primary production consumers/docs and version-1 compatibility migration pass; E4/E5. |
| AP-09 — Full coverage and requalification | IN PROGRESS | AP-08, all target-release capability acceptance | Full register/platform proof complete; final visible native performance rerun pending; E5 / G-AUTH OPEN. |

Planning validation executed in `/Users/jeickmeier/Projects/finstack-chart`,
7 September 2026, Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links. Target graph checks are not runtime proof.
- `mise exec -- python3 /tmp/check_chart_authoring_plan.py`: passed ten package/fixture/
  ledger rows and an acyclic sequence, nine normative/traceability IDs, fifteen routing
  families, open-gate/assumed-baseline wording, synchronized 0.5.0 authority versions,
  and new-plan/ADR links and whitespace. This temporary documentation checker is not
  a feature acceptance runner; its compact requirement-ID parsing was corrected before
  the passing run.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md docs/alpha-api.md`:
  passed, retaining the repository's Markdown hard-break convention. The temporary
  checker also checks the untracked plan and ADR, which Git's tracked diff omits.

G-AUTH remains OPEN. No Rust runtime tests, actual Python/WASM execution, native/export
inspection, sustained performance run or proposed-API compilation was performed for
this documentation-only task. Next implementation action: AP-00 under the stated
completed-baseline assumption. Detailed acceptance commands are the existing scoped
and final repository gates; the plan specifies intended evidence, not results.

## Phase 2 parity implementation planning — 7 September 2026

Planning complete in the [secondary implementation plan](impl_plans/phase-2-parity-implementation-plan.md).
Scope: consolidate all eight D3 module plans and the ggplot2 review into Phase 2,
with shared ownership, prerequisites, full capability coverage and intended evidence.
Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; result is uncommitted
documentation. Existing source, review and planning changes are preserved. No runtime,
dependency, fixture baseline or portable envelope version changes were made by this task.

Specification/main-plan/migration document versions advance together to 0.4.0.
GG2-01–12 explicitly adopt ggplot2 capability scope, including models/density,
coordinates/geography, mathematical text and saving devices. D3 package inventories
remain in their existing plans. P2-00 owns integration coordination; GG-00–19 own the
new ggplot2 slices and FIX-GG00–19. CLR-04 is now an explicit SP-04 prerequisite because
the latter consumes its descriptors and paint lowering. Historical WP/G2 evidence is
unchanged; WP-16 remains under its active assignment and WP-17–20 remain prerequisites
for final integration. G-PARITY joins G3, all eight D3 gates and G-GGPLOT before the
existing WP-21–23 final certification. No implementation gate advanced.

All new package owners are unassigned and all implementation evidence is absent.
READY means prerequisite records permit starting the task; it does not certify the
unimplemented feature. Exact owned work and FIX-GG cases are defined once in the plan.

| Package | State | Requirement scope | Evidence / next action |
| --- | --- | --- | --- |
| P2-00 — Integration contract and coverage register | READY | GG2-01/12, ARC-03/04, BND-01 | Planning only; verify WP-14 interfaces and agree one profile/wire/reference strategy. |
| GG-00 — Reference inventory and executable oracle | NOT STARTED | GG2-01/12 | Requires P2-00; pin release/dependencies and generate complete coverage. |
| GG-01 — Repair the existing legend omission | READY | GG2-04, GRA-07, SCL-05, LAY-03, THM-03 | Historical GGP-01 probe only; reproduce, fix and inspect focused artifacts. |
| GG-02 — Compatibility profile, stages and inferred grouping | NOT STARTED | GG2-01/02 | Requires GG-00. |
| GG-03 — Independent aesthetic encodings | NOT STARTED | GG2-03 | Requires GG-02, SP-04, WP-S05. |
| GG-04 — ggplot2 scale and palette policies | NOT STARTED | GG2-03 | Requires GG-03, SP-06, CP-04, CLR-04. |
| GG-05 — Complete guides and legend composition | NOT STARTED | GG2-04 | Requires GG-01, GG-04, WP-AX04. |
| GG-06 — Bin/count/summary and position semantics | NOT STARTED | GG2-02/05 | Requires GG-02, SP-03, WP-S06. |
| GG-07 — Primitive and interval recipe completion | NOT STARTED | GG2-06 | Requires GG-03, GG-06, WP-S02/03/05. |
| GG-08 — Data-driven text, labels and annotations | NOT STARTED | GG2-06/09 | Requires GG-03, WP-P04. |
| GG-09 — Distributional and one-dimensional analytical layers | NOT STARTED | GG2-05/06 | Requires GG-06/07. |
| GG-10 — Smoothers, confidence bands and quantile regression | NOT STARTED | GG2-05/06 | Requires GG-06/07; spike model algorithms and dependencies. |
| GG-11 — Two-dimensional statistics and contours | NOT STARTED | GG2-05/06 | Requires GG-06/07/09. |
| GG-12 — Facet semantics and layout breadth | NOT STARTED | GG2-07 | Requires GG-02/05/06. |
| GG-13 — Cartesian, transformed and polar/radial coordinates | NOT STARTED | GG2-08 | Requires GG-07/12, WP-S04, WP-AX04. |
| GG-14 — Theme hierarchy and mathematical typography | NOT STARTED | GG2-09 | Requires GG-05/08/12/13. |
| GG-15 — Geographic layers and coordinates | NOT STARTED | GG2-08 | Requires GG-07/08/13. |
| GG-16 — Extensibility and authoring conveniences | NOT STARTED | GG2-10 | Requires GG-02/05/07/12/13. |
| GG-17 — Saving and device capability completion | NOT STARTED | GG2-11 | Requires GG-08/14/15 and WP-20. |
| GG-18 — Full grammar, update and host integration | NOT STARTED | GG2-02–12 | Requires GG-03–17 and WP-16–20. |
| GG-19 — Capability certification and handoff | NOT STARTED | GG2-01–12 | Requires GG-00–18 and all eight D3 certification packages; then G-GGPLOT/G-PARITY and WP-21/22. |

Planning evidence: all nine review/plan documents, normative contracts and selected
ADR/validation boundaries read; official ggplot2 4.0.3 index, tagged namespace,
aesthetic-stage, smoothing and saving references checked. No R oracle or D3 reference
suite ran. New GG-only effort is provisionally 182–315 engineer-days plus 2–4 for
P2-00, excluding shared D3, Phase 1 and final hardening; re-estimation gates are in
the plan. This is a scope-based allowance, not a delivery promise.

Validation executed in `/Users/jeickmeier/Projects/finstack-chart` on 7 September 2026,
Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown file links. Resolved target graphs are not platform
  runtime execution.
- `mise exec -- python3 /tmp/check_chart_phase2_plan.py`: passed 94 package nodes plus
  12 gate nodes with no cycles/unresolved dependencies, eight D3 inputs/backlinks,
  eight GGP findings, 20 GG package/fixture/ledger entries, 12 normative/traceability
  IDs, 231 local file/heading links, new-plan whitespace and 182–315 day arithmetic.
  This is a temporary documentation consistency check, not a feature acceptance runner.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving the
  established Markdown hard-break convention. New-plan whitespace was checked above
  because untracked files are outside `git diff --check`.

No Rust feature tests, native UI/artifact inspection, Python/WASM runtime, Linux or
performance gates were run for this planning-only assignment. Next implementation:
GG-01's current legend defect; next parity-program entry: P2-00, then GG-00 and D3
entry packages. The linked plan and these ledger rows are the retained planning result.

## ggplot2 feature parity review — 7 September 2026

Review complete against the official ggplot2 reference displaying version 4.0.3;
[full comparison and ranked findings](evidence/ggplot2-parity-review-2026-09-07.md).
Reviewed base `127fe2d4853f89b62ba59a17248485e3c378ba60` plus the concurrent working
tree. Result is uncommitted review evidence; existing inspection/state/category-window
and D3 planning edits are preserved. No production code, fixtures, baselines,
dependencies or normative parity requirements were changed by this review.

**Full ggplot2 parity is not established.** Basic Cartesian grammar/publication is a
tested subset. Missing statistical families, independent aesthetic scales, full
guides/facets/coordinates and deliberate binning/grouping/size differences remain.
GGP-01 reproduces a current defect: a non-faceted colored scatter prepares a two-entry
legend but its final scene contains no legend title. GGP-02–08 distinguish parity
gaps and absent evidence from violations of current scope. Relevant existing IDs:
GRA-01–08, SCL-01/05, LAY-01/03, THM-03, ARC-03, QLT-02/03, BND-03/04 and SCP-03.
GGP identifiers are review findings, not new normative requirements.

Evidence from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust `1.97.1 (8bab26f4f 2026-07-14)`:

- `mise exec -- cargo test -p chart-core --test grammar --test statistics --test full_scales --test facets --locked`:
  **47 passed, 0 failed** (20 grammar, 14 statistics, 4 full_scales, 9 facets).
- `mise exec -- cargo test -p chart-export --test composition --test publication --test extensions --locked`:
  **20 passed, 0 failed** (6 composition, 10 publication, 4 extensions).
- A Rust scene probe confirmed missing single-panel legends and the explicit grouping
  requirement for category-colored lines. Retained [source](evidence/ggplot2-parity-2026-09-07/probe.rs)
  and [command/output/core binary hash](evidence/ggplot2-parity-2026-09-07/probe.log).
  Its faceted positive control painted the legend successfully.
- `mise exec -- python3 scripts/check_repository.py` passed workspace/dependency
  isolation and local Markdown links. Ledger whitespace and an inline new-report
  whitespace/finding-ID/count check passed; commands are retained in the review.

Tests certify the binaries built during this review, not subsequent concurrent edits.
No fresh R/ggplot2 oracle, Python/WASM runtime, native UI/image inspection, Linux,
performance or full matrix ran. Existing G2 evidence retains its stated scope;
no gate was advanced. Next action: fix GGP-01, then select a bounded ggplot2 capability
target and explicit compatibility policies while reusing the planned D3 foundations.

## D3 interpolation parity planning handoff

Planning is complete in [the interpolation plan](impl_plans/d3-interpolate-parity-plan.md).
The implementation does **not** have d3-interpolate parity. ITP-01–08/FIX-I01 and
G-INTERPOLATE are required by specification 0.3.0. This is uncommitted documentation
on starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; existing action/binding
changes and concurrent shape/scale/axis/color/chromatic/path planning are preserved.
CLR-02/03 own color parsing/conversion; WP-IP owns interpolation; SP owns normalization;
CP owns named ramps. No implementation, fixture, dependency or wire version was changed
by this assignment. Next interpolation package: **WP-IP01** after the accepted WP-14.

| Interpolation package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-IP01 — Contract and reference harness | READY | Unassigned | Planning only; no oracle run | ITP-01/08 | Requires WP-14; reconcile pinned export/source inventory and share the reference lock with color/scale/chromatic/axis. |
| WP-IP02 — Scalar kernels and composition | NOT STARTED | Unassigned | None | ITP-02/03 | Requires WP-IP01. |
| WP-IP03 — Structured values | NOT STARTED | Unassigned | None | ITP-01/02/07 | Requires WP-IP02; color dispatch joins WP-IP04 in integration. |
| WP-IP04 — Color interpolation | NOT STARTED | Unassigned | None | ITP-04 | Requires WP-IP02 and CLR-03; consumes the shared color engine. |
| WP-IP05 — Transform and zoom interpolation | NOT STARTED | Unassigned | None | ITP-05/06 | Requires WP-IP02. |
| WP-IP06 — Portable and chart integration | NOT STARTED | Unassigned | None | ITP-07 | Requires WP-IP03/04/05, CLR-04, SP-04 and WP-16/19/20. |
| WP-IP07 — Parity certification | NOT STARTED | Unassigned | None | ITP-01–08 | Requires WP-IP06, SP-07, WP-AX06, CLR-05 and CP-05; precedes WP-21/22/23. |

## D3 path parity planning handoff

The [path gap review and delivery plan](impl_plans/d3-path-parity-plan.md) is complete
as documentation, reviewed at `1cb955740c2dad2607b0a2330201125294cab5d0` plus the live
working tree. The specification now requires PTH-01–06, FIX-P01–06 and G-PATH.
**Implementation does not have d3-path parity.** Missing capabilities include the
standalone builder/serializer, arcs/arcTo, signed rectangle subpaths, D3 state behavior
and precision controls. WP-P01–04 own the common foundation consumed by WP-S01;
their provisional 9–16 developer-days overlap the shape estimate. WP-S01 completion
now requires WP-P04; WP-21 and G4 require G-PATH. Existing completed packages retain
their recorded scope; concurrent action and other parity work is preserved.

Evidence: 7 September 2026, Darwin arm64, Rust 1.97.1, Node v24.14.0, working directory
`/Users/jeickmeier/Projects/finstack-chart`. Compared live sources with official docs
and pinned d3-path 3.1.0 source/exports/tests/manifest. Executed 14 exploratory sequences
and two invalid-digit cases against the pinned upstream source; its hash and findings
are in the plan. Temporary downloads are not a retained parity corpus; WP-P01 owns it.
The two commands below each passed **1 test, 0 failures**:

```sh
mise exec -- cargo test -p chart-core --test contracts numeric_paths_preserve_segments_and_reject_invalid_subpath_order --locked
mise exec -- cargo test -p chart-export --test publication plain_text_xml_escaping_curves_and_empty_clips_are_supported --locked
```

These validate existing behavior, not D3 differential parity or visual fidelity.
No Rust implementation, dependency, baseline or wire version changed in this assignment.
No new path fixtures, native/export image inspection, binding parity, Linux execution
or performance evidence was produced. All PTH requirements and G-PATH remain open.
Next action within the path assignment: **WP-P01 — Contract and reference corpus**.

Documentation checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace/dependency isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (retains existing Markdown
hard-break spaces). These checks do not execute the declared target platforms.
An inline `python3 -` consistency check also passed six PTH requirement rows, six
FIX-P fixtures, four path package rows, gate/traceability links, 12 acyclic path/shape
packages and path-plan whitespace/anchors. Result: uncommitted documentation, with
implementation and all path acceptance gates still open.

| Path package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-P01 — Contract and reference corpus | READY | Unassigned | Planning review only | PTH-01–06, ARC-04, QLT-02 | WP-14 prerequisite met; pin the complete corpus and decide state/arc/precision contracts in the shared ADR. |
| WP-P02 — Checked builder and complete geometry | NOT STARTED | Unassigned | None | PTH-01/02/03/05 | Requires WP-P01; FIX-P01/02/03/05. |
| WP-P03 — Shared SVG path output | NOT STARTED | Unassigned | None | PTH-01/04 | Requires WP-P02; FIX-P04 and actual SVG. |
| WP-P04 — Renderers, portable APIs and acceptance | NOT STARTED | Unassigned | None | PTH-01–06 | Requires WP-P03; complete FIX-P01–06 and G-PATH before WP-S01 completion/WP-21. |


## D3 color parity planning handoff

The [color review and plan](impl_plans/d3-color-parity-plan.md) is complete as
uncommitted documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`.
Current byte colors/palettes do **not** establish d3-color parity. Specification 0.3.0
now also requires COL-01–06, CLR-01–05, FIX-C01 and G-COLOR. Historical WP-11/13/14
and G2 evidence retains its original scope. Active WP-15 and concurrent scale/shape/
axis/chromatic changes are preserved. CLR identifiers and FIX-C01 avoid CP/FIX-21
catalog ownership collisions. SP-04 requires CLR-03's color kernels; CLR-05 requires
SP-04 and WP-20, without depending on SP-07 or CP-05. WP-21/22 require CLR-05.

Evidence: 7 September 2026; `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026). Official d3-color documentation and retrieved
source compared with live core, themes, grammar, wire, host and export code. Target
module 3.1.0; tagged Lab source fetch failed, so main-branch Lab was inspected and
CLR-01 must verify it against the pinned tarball. No D3 oracle was executed.
`mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`:
**1 passed, 0 failed, 3 filtered out**, proving only the existing palette subset.
`mise exec -- python3 scripts/check_repository.py` passed workspace edges,
host isolation and local Markdown file links (graph inspection, not target execution).
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed, preserving the
existing Markdown hard-break convention. Inline `python3 -` consistency checks passed:
six COL definitions/traceability rows, five CLR plan/ledger packages, FIX-C01/G-COLOR,
WP-21/22 dependencies, shared interpolation ownership and 61 acyclic package nodes.
New color-plan whitespace passed separately. Concurrent interpolation planning was
also reconciled: WP-IP04 consumes CLR-03, and SP-04 consumes WP-IP04. No implementation, fixtures,
dependency or wire version changed. No fresh color binding runtime, native/export
artifact inspection, Linux or performance proof was produced. All COL requirements
remain open. Next within this plan: **CLR-01 — contract and reference oracle**.

| Color package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CLR-01 — Contract and reference oracle | READY | Unassigned | Planning only | COL-01–06, ARC-04, BND-01, QLT-02 | WP-14 prerequisite met; pin complete source/oracle and freeze typed/wire/paint contracts. |
| CLR-02 — RGB/HSL, parsing and common operations | NOT STARTED | Unassigned | None | COL-01–04 | Requires CLR-01. |
| CLR-03 — Lab, HCL/LCh and Cubehelix | NOT STARTED | Unassigned | None | COL-02–04 | Requires CLR-02; supplies SP-04 color kernels. |
| CLR-04 — Authoring, portable operations and paint integration | NOT STARTED | Unassigned | None | COL-05, BND-01/03/04, THM-01/02/03, SCN-03 | Requires CLR-03. |
| CLR-05 — Integrated parity acceptance | NOT STARTED | Unassigned | None | COL-01–06, QLT-02/03/04, SCN-04 | Requires CLR-04, SP-04, WP-20; precedes WP-21/22. |

## D3 scale-chromatic parity planning handoff

The [chromatic review and plan](impl_plans/d3-scale-chromatic-parity-plan.md) is complete
as documentation. Starting revision: `1cb955740c2dad2607b0a2330201125294cab5d0`; outcome:
uncommitted documentation, including specification/implementation/migration version
0.3.0, CHR-01–06, CP-01–05, FIX-21 and G-CHROMATIC. The implementation has raw palettes
and linear RGB interpolation, **not full scale-chromatic parity**. All six CHR requirements
remain open. The previous scale plan's catalog exclusion is replaced with explicit
chromatic ownership; SP-04 retains shared RGB/Cubehelix and scale algorithms. No source,
fixture, dependency or wire version was changed by this planning assignment. Existing
WP-15 and shape/scale/axis changes were preserved. Historical G2 acceptance is unchanged.

Evidence: 7 September 2026, `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
configured Rust 1.97.1. Official docs and pinned d3-scale-chromatic 3.1.0 source were
compared with live core color, compiler, legend, theme and portable host paths.
Read-only Python/urllib source inventory assertions passed: **76 exports, 38 schemes,
38 interpolators, 218 discrete arrays**, every export name present in the plan.
Source retrieval succeeded using network-enabled read-only execution after sandbox DNS
failure. This inspected source; it did not execute D3 or generate a numerical oracle.

Checks run:

- `mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`: **1 passed**, 0 failed, 3 filtered out. Existing custom palette behavior only.
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/host isolation and local Markdown file links; platform graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving existing Markdown hard breaks.
- Inline Python documentation assertions: six normative CHR IDs and traceability rows, five CP plan/ledger packages, aligned 0.3.0 document headers and release dependencies checked.

No fresh chromatic differential, Python/WASM execution, native/export inspection,
Linux or performance evidence was produced. Estimated incremental effort is 9–16
engineer-days, excluding shared SP work; re-estimate after CP-01. Next within this
assignment: **CP-01**, reference oracle/compatibility contract, followed by CP-02's exact
catalog. CP-03 requires SP-04, and CP-05 requires SP-07/WP-20 before WP-21/22.

| Chromatic package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CP-01 — Reference contract and oracle | READY | Unassigned | Planning/source inventory only | CHR-01/03/05/06, ARC-04 | WP-14 prerequisite met; coordinate shared oracle and interpolation/schema boundary with SP-01/04. |
| CP-02 — Exact discrete catalog | NOT STARTED | Unassigned | None | CHR-01/02 | Requires CP-01. |
| CP-03 — Complete interpolator catalog | NOT STARTED | Unassigned | None | CHR-01/03 | Requires CP-02 and SP-04. |
| CP-04 — Chart, guide and portable integration | NOT STARTED | Unassigned | None | CHR-04/05, SCL-03, THM-02, BND-01/03/04 | Requires CP-03. |
| CP-05 — Integrated certification | NOT STARTED | Unassigned | None | CHR-01–06, QLT-02/03/04 | Requires CP-04, SP-07 and WP-20; precedes WP-21/22. |

## D3 hierarchy parity planning handoff

The [hierarchy review and delivery plan](impl_plans/d3-hierarchy-parity-plan.md) is
complete as documentation. **Implementation parity is absent**: HIR-01–06 have no
hierarchy engine/layout implementations; HIR-07/08 integrated runtime, renderer and
performance evidence remains unverified. Specification 0.3.0 now requires HIR-01–08,
FIX-H01-A–H and G-HIERARCHY. WP-H08 precedes WP-21/22 acceptance. Historical G2 and
completed Cartesian packages retain their original scope.

Review context: 7 September 2026, Darwin arm64,
`/Users/jeickmeier/Projects/finstack-chart`; started at `1cb9557` with live action and
parity-plan edits. HEAD advanced independently to `127fe2d` during review; this handoff
is uncommitted documentation on that working tree. Source and other planning changes
were preserved. The review compared the official D3 hierarchy documentation and pinned
3.1.2 exports/construction/stratify/treemap/packing source with core, grammar, layout,
scene, provenance and portable interfaces. It inventories all 16 exports, including
`Node`, all methods/controls, reference adaptations and discriminating planned cases.

No hierarchy code, wire version, dependencies or baselines changed. No Rust feature
tests, D3 differential runner, actual binding proofs, native/export visual inspection,
Linux execution or benchmarks ran for this task. Checks below validate documentation
only. G-HIERARCHY and all hierarchy implementation packages remain open.
Next within this assignment: **WP-H01**, then WP-H02; existing action and other parity
assignments retain their own next steps.

Planning validation passed:

- `mise exec -- python3 scripts/check_repository.py`: workspace edges, host isolation,
  pinned GPUI identity, optional Kit and local Markdown file links. Target graph checks
  are not target runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: no whitespace errors,
  preserving existing Markdown hard breaks.
- Inline `python3 -` consistency check: eight normative HIR IDs, eight package rows in
  the hierarchy plan and ledger, eight fixture subsets, all 16 reference export names,
  main-plan traceability, WP-21/22 dependencies, removed hierarchy deferrals, local
  hierarchy heading links and new-plan whitespace passed.

| Hierarchy package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-H01 — Contract and reference harness | READY | Unassigned | Planning only; no oracle run | HIR-01–08, ARC-04, BND-01, QLT-02 | WP-14 prerequisite passed for its scope; pin method/default fixtures and record the hierarchy ADR. |
| WP-H02 — Topology, stratification and operations | NOT STARTED | Unassigned | None | HIR-01/02/07 | Requires WP-H01. |
| WP-H03 — Tidy tree and cluster | NOT STARTED | Unassigned | None | HIR-03 | Requires WP-H02. |
| WP-H04 — Partition | NOT STARTED | Unassigned | None | HIR-04 | Requires WP-H02. |
| WP-H05 — Treemap and tilers | NOT STARTED | Unassigned | None | HIR-06 | Requires WP-H02; include explicit resquarify history/reset. |
| WP-H06 — Packing and helpers | NOT STARTED | Unassigned | None | HIR-05 | Requires WP-H02. |
| WP-H07 — Grammar, portable API and presentation | NOT STARTED | Unassigned | None | HIR-07, BND-01/03/04, SCN-04 | Requires WP-H03/04/05/06, WP-S03/04, WP-16/18. |
| WP-H08 — Integrated parity acceptance | NOT STARTED | Unassigned | None | HIR-01–08, QLT-02/03/04 | Requires WP-H07, WP-19/20; full FIX-H01-A–H evidence precedes WP-21/22. |

## D3 scale parity planning handoff

The [scale review and implementation plan](impl_plans/d3-scale-parity-plan.md) is
complete as documentation at `1cb9557` plus uncommitted documentation changes.
The implementation does **not** have D3 scale feature parity. Specification 0.2.0
now requires SCL-06–08/FIX-20 and G-SCALE; SP-01–07 own the missing families,
behavioral compatibility, oracle and end-to-end evidence. WP-06/11 and G2 retain
their original 0.1.0 acceptance. Shape/axis plans and active WP-15 edits are preserved.
Scale tick/format algorithms have one owner; WP-AX02 consumes SP-03/05/06.

Review evidence: 7 September 2026, working directory
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1.
Official D3 documentation and d3-scale 4.0.2 source compared with live scale,
axis/formatting and portable contracts. `mise exec -- cargo test -p chart-core
--test scales --test full_scales --locked` passed **13 tests, 0 failed**.
These verify existing contracts, not D3 parity. No fresh D3 differential, Python/WASM,
native/export, Linux or performance evidence was produced. No scale code, fixtures,
dependencies or wire version changed. Next within the scale assignment: **SP-01**.

Planning checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace edges, host isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (preserves the existing
Markdown hard-break convention). Graph validation is not platform runtime execution.

| Scale package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| SP-01 — Compatibility contract and oracle | READY | Unassigned | Planning only; no oracle run | SCL-06/07/08, ARC-04, BND-01, QLT-02 | After WP-14, pin the complete fixture oracle and record method/default/migration contracts. |
| SP-02 — Continuous mapping and numeric families | NOT STARTED | Unassigned | None | SCL-01/02/03/06/07, DAT-05 | Requires SP-01 and WP-IP02. |
| SP-03 — Ordinal, band and point | NOT STARTED | Unassigned | None | SCL-01/06/07, DAT-02 | Requires SP-01. |
| SP-04 — Interpolation, distribution and color | NOT STARTED | Unassigned | None | SCL-03/06/07, GRA-03, BND-01 | Requires SP-02/03, CLR-04 and WP-IP03/04; CLR-04 supplies descriptors/paint lowering as well as preceding CLR-03 kernels. |
| SP-05 — Numeric ticks, nice and formatting | NOT STARTED | Unassigned | None | SCL-07, LAY-01/02, THM-03 | Requires SP-02/04; shared API consumed by WP-AX02. |
| SP-06 — UTC and explicit local calendars | NOT STARTED | Unassigned | None | SCL-04/06/07, DAT-05, BND-01 | Requires SP-02/05; shared API consumed by WP-AX02. |
| SP-07 — Integrated parity proof | NOT STARTED | Unassigned | None | SCL-01–08, BND-01/03/04, SCN-04, QLT-02/03/04 | Requires SP-03–06 and WP-16/18/20; acceptance precedes WP-21/22/23. |

## D3 shape parity planning handoff

The owner-requested [shape parity review and plan](impl_plans/d3-shape-parity-plan.md)
is complete as documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`
plus uncommitted documentation changes. Specification/implementation/migration documents
now use version 0.2.0 and require SHP-01–10, FIX-S01–09 and G-SHAPE before G4.
Existing WP-10/11/14 and G2 passes retain their original Cartesian scope. No D3 parity
implementation or runtime pass is claimed; all SHP requirements remain open.

Evidence: 7 September 2026, Darwin arm64, working directory
`/Users/jeickmeier/Projects/finstack-chart`; official D3 documentation and pinned
d3-shape 3.2.0 exports/source compared with the live implementation. Exact source
locations and missing contracts are in the linked plan.
`mise exec -- python3 scripts/check_repository.py` passed workspace/dependency isolation
and local Markdown file links. Inline `python3 -` inventory/dependency checks passed:
10 SHP IDs, nine supplemental fixtures, eight shape ledger packages and 31 acyclic
original-plus-shape package nodes at the time of this check.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md`
passed; the default whitespace check flags these documents' intentional Markdown
hard-break spaces. The new shape plan has no trailing whitespace.
The review ran no Rust feature tests, D3 oracle, native visuals or binding
runtime comparisons. Concurrent WP-15 source and status edits were preserved.
Next within the shape assignment: WP-S01 reference fixtures and shared path foundation;
the ongoing WP-15 assignment below continues independently.

## Current handoff

**The owner's original WP-11 through WP-23 assignment is complete, with a separate
commit for every package. Expanded acceptance remains OPEN.** The owner explicitly
waived the 30-minute test; its interrupted trace and numerical failure remain retained.
No package was published and no distribution license was inferred.

WP-23 supplies the [developer/release guide](release-guide.md),
[65-requirement evidence index](release-evidence.md), changelog, package/source policy,
and deterministic unpublished local source-candidate tool. The index explicitly lists
75 additional identifiers from concurrent planning as OPEN and does not certify later
clauses added to original IDs. See [WP-23 evidence](evidence/wp-23-completion-2026-09-07.md).

Final runtime evidence is **219 macOS tests, 213 Linux headless tests**, required
fmt/check/lint/docs/WASM checks and actual Rust/Python/Node WASM fixtures (36 cases,
23 actions, 47 input steps, 70 stream steps, three density cases, 40 live-export steps).
The extracted committed runtime archive passes offline repository/isolation/link checks
and fresh headless all-target compilation. Repeated archives have identical SHA-256.
The final WP-23 archive/manifest is generated under `artifacts/local-release` after commit.

[WP-22](evidence/wp-22-completion-2026-09-07.md) records ten-line p95 hover/frame work
0.092666/11.689291 ms and visible short-stream actual ingest-to-display p95 165.759375 ms.
Exact accounting and concurrent publication drain/release pass. Heavy-dashboard timings,
failed normal-window display traces, allocator high-water and the duration/visibility
limits remain explicit. These results do not certify expanded workloads or current G4.

**Next owner work:** select the separately planned parity/primary-authoring lane and its
acceptance packages. Preserve the open OS accessibility/platform/distribution boundaries,
license metadata and dependency advisories. There is no active implementation package
left in this original assignment; concurrent owner edits remain separate.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap accepted; WP-03 capability follow-up complete; dependency maintenance/release disposition remain WP-08/23. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `435e127` | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | DONE | Unassigned | `3a86189` | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | [Completion evidence](evidence/wp-03-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md); [ADR-008](adr/008-benchmark-protocol.md) | Actual proof artifacts, native lifecycle/hooks and starting profile inspected; public integration/full fixtures remain later packages. |
| WP-04 — Immutable data, schemas and transactions | DONE | Unassigned | `d0a6c48` | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | [Completion evidence](evidence/wp-04-completion-2026-09-06.md); [ADR-004](adr/004-immutable-data-and-transactions.md) | Data portions accepted; full transform/selection behavior, queued ingestion/time retention and binding runtimes remain later packages. |
| WP-05 — Grammar compiler and minimal prepared scene | DONE | Unassigned | `3cf1b33` | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | [Completion evidence](evidence/wp-05-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Known data-space geometry/semantics accepted; scales/layout and destination scene projection are WP-06; complete grammar/extensions remain later packages. |
| WP-06 — Foundational scales, ticks and layout | DONE | Unassigned | `f8657fb` | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | [Completion evidence](evidence/wp-06-completion-2026-09-06.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Foundational scale/scene/text-layout contracts accepted; actual native/export consumers are WP-07/08, full families/typography/shared layout remain WP-11–13. |
| WP-07 — Working standalone GPUI vertical slice | DONE | Unassigned | `fac148a` | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | [Completion evidence](evidence/wp-07-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Standalone line/point/bar, native text, presented-snapshot inspection, resize and disposal accepted for FIX-01/07/18 subsets; complete interaction/indexing/accessibility remain WP-15–21. |
| WP-08 — Headless export and snapshot foundation | DONE | Unassigned | `fac148a` | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | [Completion evidence](evidence/wp-08-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Basic headless formats, explicit resources and minimal FIX-13/14 accepted; full composition/live exports remain WP-13/20. |
| WP-09 — Portable schema and executable binding proofs | DONE | Unassigned | `c5ec829` | BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01 | [Completion evidence](evidence/wp-09-completion-2026-09-06.md); [ADR-006](adr/006-portable-specification-and-binding-proofs.md) | Version 1 subset and actual Rust/Python/WASM FIX-15/16 accepted; extend builtin coverage through WP-10–14 and full parity at WP-21. |
| WP-10 — Complete statistical and position semantics | DONE | Unassigned | `c5ec829` | GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02 | [Completion evidence](evidence/wp-10-completion-2026-09-06.md); [contract](statistics-contract.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Built-in stats/positions and FIX-02–05 accepted; exact full-recompute fallback declared. Extend families in WP-11 and specialized streaming in WP-18. |
| WP-11 — Required scale and geometry families | DONE | Unassigned | `a6fb2ea` | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | [Completion evidence](evidence/wp-11-completion-2026-09-07.md); [contract](scale-geometry-contract.md) | Required families and FIX-01/03/07 scope accepted through native/export/actual bindings; proceed to shared layout in WP-12. |
| WP-12 — Facets, guides and shared layout | DONE | Unassigned | `63dbcc2` | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | [Completion evidence](evidence/wp-12-completion-2026-09-07.md); [contract](facet-layout-contract.md) | FIX-06 and facet/shared-layout scope accepted through core/native/export/actual bindings; full typography and composition remain WP-13. |
| WP-13 — Full themes and publication composition | DONE | Unassigned | `f6c41c1` | THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03 | [Completion evidence](evidence/wp-13-completion-2026-09-07.md); [contract](theme-typography-composition-contract.md) | FIX-12/13 accepted through actual core/native/export/bindings; cumulative G2 remains WP-14. |
| WP-14 — Extension contracts and alpha API | DONE | Unassigned | `1cb9557` | SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05 | [Completion evidence](evidence/wp-14-completion-2026-09-07.md); [contract](extension-contract.md); [alpha matrix](alpha-api.md) | FIX-17 and cumulative G2 passed; full reducer/interaction begins WP-15. |
| WP-15 — Complete action reducer and state ownership | DONE | Unassigned | `127fe2d` | INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01 | [Completion evidence](evidence/wp-15-completion-2026-09-07.md); [contract](state-action-contract.md); [ADR-007](adr/007-actions-gestures-and-controlled-state.md) | Deterministic action/controlled/gesture/history/lifetime scope accepted through actual native/export/Python/WASM. Input producers and full G3 remain WP-16–20. |
| WP-16 — Hit testing, navigation and selection | DONE | Unassigned | `4148793` | INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05 | [Completion evidence](evidence/wp-16-completion-2026-09-07.md); [contract](interaction-contract.md) | Assigned indexed inspection/navigation/selection acceptance passes; FIX-09/10 remaining portions and G3 stay with WP-17–20. |
| WP-17 — Linked views, editable annotations and host controls | DONE | Unassigned | `6779e44` | INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06 | [Completion evidence](evidence/wp-17-completion-2026-09-07.md); [contract](host-tools-contract.md) | Original linked/editing/host acceptance passes; native accessibility limitations recorded. Full G3 remains open. |
| WP-18 — Streaming retention and incremental computation | DONE | Unassigned | `30ca2e8` | DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01 | [Completion evidence](evidence/wp-18-completion-2026-09-07.md); [contract](streaming-contract.md) | Original queue/retention/incremental/follow acceptance passes; 70-step three-host replay and native lifecycle inspected. Sustained PERF and G3 remain open. |
| WP-19 — Bounded scheduling, caches and dense representation | DONE | Unassigned | `e57246d` | STM-04, STM-05, SCN-04, GPU-02, QLT-04 | [Completion evidence](evidence/wp-19-completion-2026-09-07.md); [contract](scheduling-density-contract.md) | Original bounded-worker/cache/density acceptance passes; actual four-chart progress and three-host dense proofs. Preliminary PERF identifies index/memory bottlenecks; intermittent native redraw question retained. |
| WP-20 — Coherent exports during live interaction | DONE | Unassigned | `9d86ef7` | EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04 | [Completion evidence](evidence/wp-20-completion-2026-09-07.md); [contract](live-export-contract.md) | Original coherent capture/bounded lifetime acceptance passes; actual 40-step three-host replay and finite native exports during 400 atomic commits. Sustained PERF and native hardening remain open. |
| WP-21 — Correctness, fidelity and supported-platform hardening | DONE (original scope) | Unassigned | `4c099ee` | SCP-03, QLT-01/02/03, GPU-02/03, BND-03/04, FIX-01–18 | [Completion evidence](evidence/wp-21-completion-2026-09-07.md); [ADR-009](adr/009-supported-platform-and-accessibility.md) | Original acceptance passes including fixed native frozen resize/redraw and actual macOS/Linux/headless bindings. Expanded parity/authoring acceptance remains open and requires the separately listed packages/gates. |
| WP-22 — Measured performance and sustained-load release gate | DONE (original scope; owner duration waiver) | Unassigned | `c58f5b2` | STM-01/03/04/05, EXP-03, QLT-04, PERF-01–05 | [Completion evidence](evidence/wp-22-completion-2026-09-07.md); [ADR-008](adr/008-benchmark-protocol.md) | Visible short budgets and exact accounting pass; interrupted/occluded failures and memory limits retained. Thirty-minute duration explicitly owner-waived. Expanded workloads and current G4 stay open. |
| WP-23 — Production documentation and release readiness | DONE (original local candidate scope) | Unassigned | Included in this completion commit | SCP-01/02/03, ARC-04, BND-01, QLT-05/06 | [Completion evidence](evidence/wp-23-completion-2026-09-07.md); [release index](release-evidence.md); [guide](release-guide.md) | All original package handoffs complete; source candidate remains unpublished. Expanded parity/authoring and current production G4 remain OPEN; duration waiver, failed evidence, platform/accessibility/metadata limits are explicit. |

## Supplemental shape work packages

The [shape plan](impl_plans/d3-shape-parity-plan.md#work-packages-and-dependency-order)
owns deliverables and acceptance criteria; planning approval is not implementation evidence.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-S01 — Shape contract, oracle and path foundation | NOT STARTED | Unassigned | — | SHP-01, SHP-08, SHP-10 | Planning review only | Requires WP-14 and WP-P04. Consume the shared path foundation; pin shape artifacts and prove shape-specific FIX-S01 cases. |
| WP-S02 — Cartesian generators and complete curves | NOT STARTED | Unassigned | — | SHP-02, SHP-03, SHP-09 | None | Prerequisite: WP-S01. |
| WP-S03 — Arc geometry and pie layout | NOT STARTED | Unassigned | — | SHP-04, SHP-09 | None | Prerequisite: WP-S01. |
| WP-S04 — Radial generators and links | NOT STARTED | Unassigned | — | SHP-05, SHP-09 | None | Prerequisites: WP-S02, WP-S03. |
| WP-S05 — Complete symbol encoding | NOT STARTED | Unassigned | — | SHP-06, SHP-09 | None | Prerequisite: WP-S01. |
| WP-S06 — Complete stack layouts | NOT STARTED | Unassigned | — | SHP-07, SHP-09 | None | Prerequisites: WP-S01, WP-10. |
| WP-S07 — Custom protocols and public portability | NOT STARTED | Unassigned | — | SHP-01, SHP-08, SHP-09 | None | Prerequisites: WP-S02, WP-S03, WP-S04, WP-S05, WP-S06. |
| WP-S08 — Integrated parity acceptance | NOT STARTED | Unassigned | — | SHP-09, SHP-10 | None | Prerequisites: WP-S07, WP-16, WP-18, WP-20. |

## Cumulative gates

### Required D3 axis packages

The [axis parity plan](impl_plans/d3-axis-parity-plan.md) adds AXIS-01–07/FIX-19 to
specification 0.2.0. Historical WP-06/11–14 and G2 evidence retains its original scope.
The axis review/planning assignment is complete; implementation parity remains open.
WP-21 additionally requires WP-AX06, and WP-22/23 include axis performance/release proof.

| Package | State | Owner | Revision | Requirements | Evidence / next action |
| --- | --- | --- | --- | --- | --- |
| WP-AX01 — Guide contract and reference harness | READY | Unassigned | — | AXIS-01, AXIS-07 | Planning review only; WP-14 prerequisite met. Implement guide/scale separation, migration and pinned reference harness. |
| WP-AX02 — Tick selection and formatting | NOT STARTED | Unassigned | — | AXIS-02, AXIS-03 | Requires WP-AX01, SP-03, SP-05, SP-06; consumes shared scale algorithms. |
| WP-AX03 — Axis geometry and bounded layout | NOT STARTED | Unassigned | — | AXIS-04 | Requires WP-AX02. |
| WP-AX04 — Styling and publication components | NOT STARTED | Unassigned | — | AXIS-05 | Requires WP-AX03. |
| WP-AX05 — Axis updates and transitions | NOT STARTED | Unassigned | — | AXIS-06 | Requires WP-AX04, WP-15, WP-19, WP-IP02 and WP-IP05. |
| WP-AX06 — Parity certification and documentation | NOT STARTED | Unassigned | — | AXIS-01–07 | Requires WP-AX05, WP-20 and SP-07; must pass before WP-21. |

### Gate results

| Gate | State | Evidence required next |
| --- | --- | --- |
| G0 | PASSED — architecture/capability scope | WP-01/02/03 evidence and ADRs establish the initial macOS route and starting protocol; this does not pass full requirements, FIX/PERF or release support. |
| G1 | PASSED — minimal portable core | WP-04–08 foundation/native/headless evidence plus WP-09 actual Python/WASM FIX-15/16 runtime comparison; this does not certify full grammar or production host/distribution products. |
| G2 | PASSED — Cartesian/publication alpha | [Alpha matrix](alpha-api.md) maps complete grammar/facets/themes/publication/extensions and actual portable evidence. G3/G4 retain their remaining scope. |
| G3 | PASSED (original interactive streaming scope) | WP-15–20 interaction/streaming/export plus WP-21 frozen-resize and native redraw regression fixes; actual native and Rust/Python/WASM evidence. Sustained PERF and expanded parity remain separate. |
| G-PATH | NOT PASSED | PTH-01–06/FIX-P01–06; WP-P04 standalone path, inspected native/export and actual Rust/Python/WASM proof. Required before WP-S01 completion, WP-21 and G4. |
| G-AXIS | NOT PASSED | AXIS-01–07/FIX-19; pinned reference matrix, actual bindings, native/publication inspection and transition evidence. Required before WP-21 and G4. |
| G-SHAPE | NOT PASSED | SHP-01–10, FIX-S01–09; complete generator/method inventory and actual native/headless/binding parity evidence. |
| G-SCALE | NOT PASSED | SCL-06–08/FIX-20; complete method inventory, actual Rust/Python/WASM scale operations, integrated chart/update behavior and inspected outputs. SP-07 precedes WP-21/22. |
| G-CHROMATIC | NOT PASSED | CHR-01–06/FIX-21; all 76 exports/218 arrays, actual Rust/Python/WASM operations, scale/guide/update composition and inspected native/publication artifacts. CP-05 precedes WP-21/22. |
| G-COLOR | NOT PASSED | COL-01–06/FIX-C01; complete color methods, exceptional channels, actual Rust/Python/WASM, coherent paint/update and inspected native/publication evidence. CLR-05 precedes WP-21/22. |
| G-INTERPOLATE | NOT PASSED | ITP-01–08/FIX-I01; all 27 exports/configuration/result controls, shared consumers, actual Rust/Python/WASM and applicable inspected native/publication evidence. WP-IP07 precedes WP-21/22. |
| G-HIERARCHY | NOT PASSED | HIR-01–08/FIX-H01-A–H; complete method/layout/history coverage, actual Rust/Python/WASM and inspected native/publication artifacts. WP-H08 precedes WP-21/22. |
| G-GGPLOT | NOT PASSED | GG2-01–12 / FIX-GG00–19 and GG-19 complete reference/host/destination capability evidence. |
| G-PARITY | NOT PASSED | G3, all eight D3 gates and G-GGPLOT; Phase 2 integrated capabilities before final WP-21/22 acceptance. |
| G4 | OPEN — local candidate only | G-PARITY and all required FIX/PERF/platform/accessibility/documentation evidence through WP-21–23, including D3 and ggplot2 supplemental workloads. |

## Axis planning evidence — 7 September 2026

Outcome: reviewed current axis implementation against D3 and added the parity plan,
normative requirements, fixture coverage, package dependencies and open gate. No axis
implementation changed. Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`;
result is uncommitted documentation. Existing action/state/binding edits and concurrent
shape planning were preserved. Source findings and external references are retained in
[the plan](impl_plans/d3-axis-parity-plan.md#evidence-from-this-planning-review).

Working directory `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026).
`mise exec -- cargo test -p chart-core --test scales --test layout --locked` passed:
29 tests (9 scales, 20 layout), zero failed. Existing behavior passes; these are not
D3 differential tests. `mise exec -- python3 scripts/check_repository.py` passed workspace
boundaries, dependency isolation and local Markdown file links. The default whitespace
check flagged the documents' existing Markdown hard-break convention; verification with
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed. An inline `python3 -`
consistency check passed seven normative axis IDs, six package rows in each of the axis
plan/main plan/ledger, requirement traceability, G-AXIS, shared-scale prerequisites and
the new plan's whitespace. These are documentation checks, not feature certification.
No new D3 fixtures, runtime binding proofs, native/export inspections, Linux execution
or performance evidence were produced. AXIS-01–07 and G-AXIS remain open.
Next within this assignment's plan: WP-AX01; existing WP-15 work continues independently.

## Interpolation planning evidence — 7 September 2026

Outcome: documented the complete 27-export d3-interpolate inventory, source-backed gaps,
ITP-01–08/FIX-I01, seven implementation packages and G-INTERPOLATE. Reconciled ownership
with the concurrent color/scale/chromatic/axis plans and preserved other live edits.
Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; result is uncommitted
planning documentation. The [plan](impl_plans/d3-interpolate-parity-plan.md) retains
source links, retrieval limitations, compatibility decisions, estimates and intended
acceptance. No production implementation, dependency, schema or fixture was changed
by this task. ITP-01–08 and G-INTERPOLATE remain open; next action is WP-IP01.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --test scales --test full_scales --locked`:
  **13 passed, 0 failed** (9 foundational and 4 full-scale tests). These establish the
  existing subset only; they are not a D3 interpolation differential suite.
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries, host isolation and local Markdown file links; no platform runtime claim.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, retaining the
  existing Markdown hard-break convention.
- `mise exec -- python3 /tmp/chart-interpolate-plan-check.py`: passed 27 export names,
  eight normative/traceability IDs, seven plan/ledger package rows, eight fixture groups,
  WP-21/22 dependencies, new-plan whitespace and an acyclic 30-node cross-lane model.
  This temporary planning check is not a committed feature runner or CI gate.

No D3 runtime/oracle regeneration, actual Python/WASM interpolation proof, native/export
inspection, Linux execution or performance measurement was run. Available reference
source included the upstream main export index and pinned zoom implementation; some
pinned source requests and shell DNS failed. WP-IP01 must reconcile and lock release
sources before committing acceptance fixtures. No absent evidence is recorded as a pass.

## Public API usability review — 7 September 2026

Outcome: reviewed the live public Rust authoring, data, native/export, documentation
and proof-binding surfaces against the owner's ggplot2-like simplicity goal. The
[review](evidence/public-api-review-2026-09-07.md) records six ranked findings,
source locations, current/proposed boundaries and concrete developer-workflow
acceptance criteria. Current usability fails that goal; tested existing grammar
contracts pass their scoped checks. Proposed authoring and developer usability remain
unverified. Scope: SCP-01, DAT-01/02, GRA-01/07, THM-01, ARC-01/02, EXP-01/02,
BND-03/04, QLT-05 and GG2-02/03/07/09/10/11. Existing minimal bindings are not
reclassified as failed proof adapters merely because their authoring is low-level.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; reviewed existing
uncommitted work. Result: uncommitted review report and this ledger entry only;
implementation, defaults, fixtures and unrelated edits preserved. The root repository
instruction to record every review outcome governs this ledger update.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --doc --locked`: **4 passed**, zero failed
  (two runnable examples and two intended compile failures).
- `mise exec -- cargo test -p chart-core --test grammar --test facets --locked`:
  **29 passed**, zero failed (20 grammar, nine facets).
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links; target graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
  and `git -c core.whitespace=-blank-at-eol diff --no-index --check -- /dev/null docs/evidence/public-api-review-2026-09-07.md`:
  passed whitespace checks, retaining the existing Markdown hard-break convention.

No proposed API was implemented or compile-certified. No actual Python/WASM proof,
native/export inspection, differential reference suite, performance gate or user
study ran. GG2 and G4 remain open. Next action: prioritize a bounded plot/data/layer
authoring slice and define compiling everyday examples early in the existing parity
plan; semantic prerequisites and legacy defaults must be preserved.

## Evidence updates

Before ending every task, including reviews and partial or blocked slices, update this
ledger with its outcome and evidence. Record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
