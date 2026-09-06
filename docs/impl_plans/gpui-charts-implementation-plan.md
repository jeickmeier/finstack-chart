# Rust-native GPUI Charts — Implementation Plan

Document version: 0.1.0  
Date: 6 September 2026  
Status: Executable project handoff; current progress is tracked in the [status ledger](../implementation-status.md).
Companion documents: [Project specification](../spec/gpui-charts-specification.md) and [Migration and architecture rationale](../spec/gpui-charts-migration-plan.md).

The [infrastructure setup plan](package-infrastructure-plan.md) covers the repository-only bootstrap. Package shells do not complete WP-02 or any release gate.

## 1. How to use this project package

Read the specification first for required behavior, this plan for execution, and the migration plan for reference research and architectural reasoning. Explicit project-owner instructions take precedence. The specification is authoritative if an illustrative API, estimate, or historical recommendation in another document differs.

This task package authorizes building and validating the library in the assigned repository when handed to an implementation developer. It does not imply that any library, repository, benchmark, or release already exists. Proposed crate names may be adjusted consistently before publishing. Version the three project documents together when scope changes.

Start at WP-01, then choose the earliest ready work package whose prerequisites have passed. A package is ready only when its dependencies provide the specified interfaces and evidence, not merely when their files exist. Each package should become one or more reviewable pull requests with requirement/fixture IDs in the description. Do not create every chart API as an unimplemented scaffold and call that an alpha.

## 2. Delivery strategy and initial planning budget

Default delivery uses one developer/integrator, potentially assisted by AI. Independent owners can accelerate bounded work after interfaces stabilize, but this is not an assumption about available staffing. Estimates are provisional engineering budgets including review, integration, tests, and documentation; re-estimate after G0. Platform/font/rendering gaps may materially change elapsed time.

| Milestone | Packages | Initial effort | Exit gate |
| --- | --- | --- | --- |
| M0 — Contracts and capability proofs | WP-01–WP-03 | 2–3 weeks | G0: dependency boundaries, native/export/font feasibility, decisions and benchmark protocol. |
| M1 — End-to-end portable core | WP-04–WP-09 | 3–4 weeks | G1: real native chart, headless output, atomic updates, minimal Python/WASM execution. |
| M2 — Cartesian/publication alpha | WP-10–WP-14 | 4–6 weeks | G2: required grammar, themes, composition, extension and publication output. |
| M3 — Interactive streaming beta | WP-15–WP-20 | 3–5 weeks | G3: complete scoped interaction, live corrections/retention, scheduling and coherent exports. |
| M4 — Production hardening | WP-21–WP-23 | 4–6 weeks | G4: all requirement/fixture/performance/platform/documentation evidence. |

Cumulative planning ranges: alpha 9–13 weeks; interactive beta 12–18 weeks; scoped production 16–24 weeks. These exclude production Python wheels/viewer/notebooks, a browser rendering product, and advanced chart families. AI availability does not remove the need to run platform, visual, and sustained-load checks.

Critical path: data/identity and statistical contracts -> scales/layout -> native/export integration -> complete grammar/publication -> interaction/streaming integration -> measured release evidence. Font and binding feasibility must be established early rather than discovered during final hardening.

## 3. Repository and working conventions

WP-01 should create a repository layout equivalent to the following table. Keep paths stable enough that independently assigned tasks have explicit ownership.

| Path | Purpose |
| --- | --- |
| `docs/spec/gpui-charts-specification.md` | Normative requirements copied from this handoff. |
| `docs/spec/gpui-charts-migration-plan.md` | Architecture rationale and pinned reference research. |
| `docs/impl_plans/gpui-charts-implementation-plan.md` | This plan, updated with implementation refinements. |
| `docs/adr/` | Small numbered decisions with options, chosen behavior, evidence and affected IDs. |
| `docs/implementation-status.md` | Resumable package/gate ledger; no unsupported success claims. |
| `docs/support-matrix.md` | GPUI/export/binding/platform capability evidence. |
| `crates/chart-core/src/` | Spec/data/stat/scale/layout/scene/interaction modules with bounded public interfaces. |
| `crates/chart-export/src/` | Figure snapshot and SVG/PDF/PNG implementation. |
| `crates/gpui-charts/src/` | GPUI host, painting, resources, input and lifecycle. |
| `crates/gpui-charts-kit/src/` | Optional Kit integration. |
| `crates/chart-python/`, `crates/chart-wasm/` | Minimal proofs initially, excluded from default desktop consumers. |
| `examples/chart-gallery/` | Runnable examples and visual/stress scenarios. |
| `fixtures/` | Small canonical data, expected semantic results and deterministic resources. |
| `benches/` and `scripts/` | Reproducible performance/validation runners and evidence generation. |

Use a workspace lockfile for reproducible development and pin reference fixtures to source revisions. Select a supported Rust toolchain/MSRV and exact dependency source at G0; do not copy an old version number just because it appears in the migration research. User/application compatibility determines the GPUI dependency line.

Default features should serve standalone desktop use without Python, a browser, Arrow/Polars, a finance engine, or network access. Core builds independently. Feature checks must cover valid combinations; avoid a blanket `--all-features` job if target-specific bindings make that combination meaningless.

Current development entry points are `mise run fmt`, `mise run check` and `mise run test`. Use `mise exec --` before one-off Cargo commands. The commands below describe package-level checks to extend as WP-02 and later acceptance runners become functional:

| Command | Purpose |
| --- | --- |
| `cargo fmt --all -- --check` | Formatting. |
| `cargo test -p chart-core` | Deterministic semantics, updates and state. |
| `cargo test -p chart-export` | Headless export/resources/figure fixtures. |
| `cargo test -p gpui-charts` | Host-level tests supported by the selected framework. |
| `cargo run -p chart-gallery` | Actual native gallery on a supported host. |
| `cargo check -p chart-core --target wasm32-unknown-unknown` | Portable core build. |
| `mise run check` / `mise run test` | Minimal current validation entry points; add scoped capability tasks only with real acceptance runners. |

Exact harness commands for GPUI, PyO3 and WASM follow selected toolchain APIs. A native test skipped because the worker lacks a supported display/OS is recorded as blocked for that environment, not passed. Headless/semantic work should continue independently.

## 4. Early decision register

These decisions need concrete recorded outcomes in the stated packages. Default choices let developers proceed without unnecessary clarification. They cannot reduce required behavior.

| ADR | Decision owner / deadline | Starting decision and acceptance basis |
| --- | --- | --- |
| ADR-001 | WP-01 / G0 | One exact GPUI package/source compatible with the intended host; standalone and optional Kit examples both build. |
| ADR-002 | WP-02, WP-05 / G1 | Typed Rust authoring lowered to normalized portable operations/scene; freeze ergonomic public signatures only after working examples. |
| ADR-003 | WP-03, WP-08 / G0 then G1 | Select native text bridge and publication font/shaping/export route by fixture evidence; evaluate existing Rust dependencies before bespoke font/PDF work. |
| ADR-004 | WP-04 / G1 | Immutable snapshots; chunked storage or equivalent avoids full history copying on every sustained append; atomic revisions and stable keys as specified. |
| ADR-005 | WP-06, WP-10 / G2 | Lock numeric/tick/constant-domain algorithms and tolerances; retain the specification's bin/quantile/stack/stat-space defaults. |
| ADR-006 | WP-09 / G1 | Portable envelopes, version policy, integer/null/handle ownership; baseline single-thread WASM and owned batch ingestion. |
| ADR-007 | WP-15–WP-19 / G3 | Gesture ownership, controlled-state revision rules, follow modes and non-starving bounded scheduling. |
| ADR-008 | WP-03, WP-22 / G0 then G4 | Reference hardware, timing boundaries, load generation and memory/lag reporting for PERF-01–PERF-05. |
| ADR-009 | WP-21 / G4 | Actual platform/accessibility and renderer support matrix; limitations cannot be hidden by broad marketing claims. |
| ADR-010 | WP-01, WP-23 / G0 then G4 | Project/package naming, contributor/source provenance and release/version policy; package publishing remains a separate release action. |

Only unresolved decisions that materially change a user requirement or incompatible application integration need product-owner input. Choose and record ordinary implementation details within this contract.

WP-01 selected and built the exact registry identities recorded in
[ADR-001](../adr/001-host-dependency-and-toolchain.md). The dependency examples are part
of the macOS `check` task; capability proofs and G0 remain open.

## 5. Work packages

Every package below includes requirements, prerequisites, deliverables, and evidence. Requirements remain incomplete until their full scope is implemented even if an early package proves a subset.

### WP-01 — Project bootstrap and scope ledger

**Prerequisites:** none.  
**Requirements:** SCP-01, SCP-02, SCP-03, ARC-04, QLT-05.  
**Owns:** repository root, project documents, initial ADRs/status/support matrix.

- Inspect an assigned existing repository and its instructions before creating/reorganizing files; preserve unrelated work.
- Install the three handoff documents, create the package/gate ledger, and record every package as NOT STARTED.
- Select the initial supported host/dependency identity, toolchain and naming defaults. Identify any existing application integration constraints.
- Create an initial feature matrix distinguishing required production support, proof-only binding support, and future products.
- Record source/provenance expectations and the benchmark host assumption.

**Acceptance:** another developer can find the authoritative requirements, identify the next ready task, and reproduce the chosen dependency setup. No hidden dependency on a finance workbench or Kit is introduced.

### WP-02 — Workspace, diagnostics and minimal contracts

**Prerequisites:** WP-01.  
**Requirements:** ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05.  
**Owns:** workspace manifests, minimal core types/errors/services, validation scripts.

- Create core/export/GPUI/Kit/gallery boundaries and small binding-proof placeholders that are explicitly marked nonfunctional until WP-09.
- Define IDs/revision stamps, diagnostic codes, finite geometry types, minimal scene primitives, text/resource service interfaces and result types.
- Establish target-specific dependency checks and working task validation commands.
- Compile portable core without GPUI/Python/JS dependencies; make error/resource limits explicit.

**Acceptance:** dependency graph enforces ARC-01/02; malformed minimal input returns structured diagnostics; core tests and portable-target compile checks run. Empty APIs alone do not satisfy G1.

### WP-03 — Native, font and export capability spike

**Prerequisites:** WP-02.  
**Requirements:** ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04.  
**Owns:** temporary gallery proof scenes, dependency experiments and capability ADRs.

- Present a fixed primitive scene with curved/dashed strokes, caps/joins, clipping, gradient, points, rotated/rich text and overlay controls through the selected GPUI line.
- Render an equivalent publication scene through candidate SVG/PDF/font dependencies and raster output. Inspect vector content, font behavior and physical dimensions.
- Demonstrate native focus/input/text lifecycle and inspect available platform accessibility hooks.
- Record unsupported primitives and a feasible implementation path for every required capability.
- Define benchmark timing boundaries and capture a starting profile. Proof code may be replaced, but retain fixtures and evidence.

**Acceptance:** G0 report includes actual native/export outputs and a selected or explicitly blocked renderer/font route. A dependency README or a compiling example is insufficient evidence of typography/vector fidelity.

### WP-04 — Immutable data, schemas and transactions

**Prerequisites:** WP-02.  
**Requirements:** DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01.  
**Owns:** core data/schema/identity/transaction modules.

- Implement typed snapshot handles and normalized columns with explicit validity and integer timestamps.
- Implement append/upsert/remove/replace, expected revisions, ordered multi-dataset atomic transactions and bounded deduplication.
- Preserve insertion ordinals and stable category/row identity; specify replacement and absent-key outcomes.
- Add source/aggregate/derived references and snapshot lifetime support.
- Prove that append storage does not require copying all retained history for every batch.

**Acceptance:** FIX-08/09/16 data portions pass; invalid transactions do not partially mutate data; large IDs retain precision; ownership remains valid while an older snapshot is held. Test independent expected outcomes, not only serialization round trips.

### WP-05 — Grammar compiler and minimal prepared scene

**Prerequisites:** WP-02, WP-04.  
**Requirements:** GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06.  
**Owns:** definition/layer/stat/geom/position contracts, normalization and basic compiler.

- Implement typed builders and heterogeneous layer data with explicit channel/constant-style distinction.
- Build identity plus explicit-edge bin stats and point/line/rule/rectangle geometry through the same staged engine.
- Track calculation space, generated output schemas, domain contributions and provenance.
- Detect incompatible inherited mappings and transform cycles; retain operation/version metadata.
- Implement only the minimal state/action boundary needed for the vertical slice; WP-15 completes its public behavior.

**Acceptance:** FIX-01/02 and a composite histogram produce known geometry/semantics; a layer with a different row schema works; derived outputs cannot be mistaken for source rows. There is one compiler route for recipes and composed layers.

### WP-06 — Foundational scales, ticks and layout

**Prerequisites:** WP-05.  
**Requirements:** SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05.  
**Owns:** linear/band/UTC scales, basic guides and layout solver.

- Implement explicit/derived domains, baseline/padding/nice policy, empty/constant/descending cases and viewport separation.
- Implement stable categorical order and timestamp-origin projection.
- Add destination text metrics, basic axes/tick generation and a bounded margin solver.
- Provide explicit range/clip and inversion capability contracts.

**Acceptance:** FIX-07 passes with documented tolerances and finite geometry; text changes recompute margins; zoom does not modify source population. The complete scale family follows in WP-11.

### WP-07 — Working standalone GPUI vertical slice

**Prerequisites:** WP-03, WP-06.  
**Requirements:** GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01.  
**Owns:** GPUI element/state, paint/text bridge and basic input.

- Render real compiled line/point/bar examples using retained state and bounded entities.
- Implement resize, basic hover/keyboard focus and caller-supplied tooltip content through a minimal shared action path.
- Translate events into local coordinates; use matching presented-scene scales/targets.
- Handle invalid data and entity disposal safely; do not recreate the complete data store on render.

**Acceptance:** actual native interaction and resize work; FIX-01/07/18 subset passes; inspect the selected host with real fonts. A raster image embedded in a UI is not a chart implementation.

### WP-08 — Headless export and snapshot foundation

**Prerequisites:** WP-03, WP-06.  
**Requirements:** ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03.  
**Owns:** chart-export resource handling, basic SVG/PDF/PNG, figure snapshot.

- Implement basic headless formats using the selected font/render route.
- Capture explicit definition/data/state/resource revisions and physical output settings.
- Return bytes/diagnostics and show a publication preview through the same publication layout path.
- Add missing-font/unsupported-effect diagnostics and text-preserving/outline capability reporting.

**Acceptance:** a real core chart exports without GPUI initialization; inspect vector marks and font policy in SVG/PDF. Minimal FIX-13/14 snapshot behavior works; complete figure furniture follows in WP-13.

### WP-09 — Portable schema and executable binding proofs

**Prerequisites:** WP-04, WP-05, WP-06, WP-07, WP-08.  
**Requirements:** BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01.  
**Owns:** portable envelopes/validation, chart-python, chart-wasm, binding fixture runner.

- Define versioned chart/data/action/state envelopes with strict required-field/operation validation and explicit 64-bit wire encodings.
- Run the same small definition/data/correction/action through native Rust, Python and an actual WASM runtime.
- Prove Python headless export and WASM scene/basic SVG output; test disposal and memory-view/copy rules.
- Reject unsupported native closures/resources instead of dropping them during serialization.
- Publish a precise proof-support matrix. Extend that matrix as later built-ins are added.

**Acceptance:** FIX-15/16 minimal fixtures execute and compare semantic results; core has no GPUI dependence. `cargo check` alone does not pass this package. Together with WP-04–WP-08 this establishes G1.

### WP-10 — Complete statistical and position semantics

**Prerequisites:** WP-09.  
**Requirements:** GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02.  
**Owns:** built-in statistics, positions, output schemas and provenance.

- Complete count/bin defaults, summary/quantile and intercept OLS behavior with explicit invalid/empty/degenerate handling.
- Implement data-versus-transformed stat space and prevent duplicate transformation.
- Implement mixed-sign stack/normalize, band-relative dodge and stable seeded jitter.
- Declare exact incremental capabilities and full-recompute fallbacks; WP-18 exercises them under streaming.
- Compare selected reference outputs only where operation definitions match this specification.

**Acceptance:** FIX-02/03/04/05 pass independently calculated expectations. Verify bin/stack conservation, input reorder invariance, filter-versus-zoom distinction and batch reference results.

### WP-11 — Required scale and geometry families

**Prerequisites:** WP-10, WP-07.  
**Requirements:** GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05.  
**Owns:** complete built-in scales, geoms and concise recipes.

- Complete log/symlog, point/ordinal/continuous color and UTC/calendar scale behavior; add supplied-calendar session-time mapping.
- Add area/ribbon, grouped/stacked interval bars, cells/heatmaps, histogram composition and OHLC/volume recipe.
- Implement explicit baselines, domains from interval/stack endpoints, gap policy and numeric/OHLC validation.
- Add named independent scales and alternate-unit secondary axes as separate APIs.
- Ensure every required geometry works through portable scenes and the native/export renderers.

**Acceptance:** required family gallery and FIX-01/03/07 cases pass. No chart recipe has a separate computation engine. Multiple scales, log-invalid values, time boundaries and color legend metadata have semantic tests.

### WP-12 — Facets, guides and shared layout

**Prerequisites:** WP-10, WP-11.  
**Requirements:** GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03.  
**Owns:** facet/panel identities, shared/free scales, guide generation and panel layout.

- Implement facet wrap and basic row/column grid, explicit panel order and empty-panel policy.
- Support layer broadcast/target declarations and per-facet/group/chart stat scope.
- Merge guides only for compatible scale/semantic identities; align shared axes and panes.
- Bound layout iterations; handle dense labels and tiny bounds with explicit fallback diagnostics.

**Acceptance:** FIX-06 passes; compatible shared scales agree across panels; free scales differ intentionally; multi-panel layout remains valid under resize and font changes.

### WP-13 — Full themes and publication composition

**Prerequisites:** WP-08, WP-12.  
**Requirements:** THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03.  
**Owns:** theme cascade, publication figure furniture, preview and complete export fidelity.

- Complete serialized theme tokens/overrides and editorial white, terminal and grayscale themes.
- Add titles, panel letters, insets, direct labels, annotations, captions/notes and shared/per-panel legend composition.
- Complete rich/rotated text, explicit resources, font fallback/outline policies and physical output sizing.
- Produce native publication preview plus SVG/PDF/300 and 600 DPI PNG examples.
- Add the optional Kit theme/control adapter without leaking its types into core.

**Acceptance:** FIX-12/13 pass numerical invariance and actual visual/vector/font inspection. Native and publication measurement differences are reconciled deliberately, not hidden by loose screenshots.

### WP-14 — Extension contracts and alpha API

**Prerequisites:** WP-09, WP-11, WP-12, WP-13.  
**Requirements:** SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05.  
**Owns:** extension examples, public API refinement and portable built-in coverage.

- Build an external-style custom stat/geom example using only supported public APIs.
- Exercise custom targets/guides and declared native-only export failure.
- Bring portable schema coverage up to all alpha built-ins; keep native closures explicitly unsupported where no portable equivalent exists.
- Replace illustrative API snippets with compiling recipe/layer examples; add compile-fail cases where type safety is an intended contract.
- Freeze alpha API boundaries and document chosen names, defaults and diagnostics.

**Acceptance:** FIX-17 passes; users can customize design/grammar without renderer forks; alpha feature matrix and G2 evidence are complete. Missing required geometry cannot be relabeled as a custom-extension exercise.

### WP-15 — Complete action reducer and state ownership

**Prerequisites:** WP-07, WP-14.  
**Requirements:** INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01.  
**Owns:** core state/actions/events, controlled-state adapter and command history.

- Complete viewport/follow, hover/focus/pinning, visibility, selection, annotation, reset and synchronization action families.
- Separate transient preview from durable commit and undoable edits.
- Add revisioned controlled state, origin tracking and idempotent effective-change events.
- Define gesture begin/update/commit/cancel and presented-scene snapshot references.

**Acceptance:** deterministic action traces work without a GPUI window; no-op events and stale controlled responses behave as specified. Cancellation leaves committed state consistent.

### WP-16 — Hit testing, navigation and selection

**Prerequisites:** WP-11, WP-15.  
**Requirements:** INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05.  
**Owns:** indexes, inspection, keyboard target order and native gesture translation.

- Implement sorted-time lookup, spatial lookup for scatter, shape containment and explicit paint-order priority.
- Add wheel/trackpad pointer-anchored zoom, pan, range navigation/reset and category windows.
- Add range/rectangular brush, lasso, additive/toggle selection and geometry-specific policies.
- Implement focus traversal and tooltip semantics for source/aggregate/derived targets.
- Respect clipping, target removal, gesture capture, Escape and focus/capture loss.

**Acceptance:** FIX-09/10 selection/navigation portions pass; hit results match the presented scene under resize and transforms. Hover does not recompile source statistics; benchmark index versus scan costs.

### WP-17 — Linked views, editable annotations and host controls

**Prerequisites:** WP-13, WP-15, WP-16.  
**Requirements:** INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06.  
**Owns:** linked-view examples, annotation tools, menus/toolbars and accessibility bridge.

- Link charts and a table through semantic domains and stable targets with cycle prevention.
- Implement constrained/snapped annotation, threshold and range-handle editing with preview/commit/cancel.
- Provide context-menu, tooltip, toolbar/copy/export and legend hooks with remappable chart-focus bindings.
- Expose chart summaries, meaningful focus targets and an accessible data alternative.
- Verify available screen-reader integration on the supported target and document actual limitations.

**Acceptance:** FIX-10 completes including linked echoes and interrupted editing. Source observations are only editable through explicit application commands. Native controls remain replaceable without changing core actions.

### WP-18 — Streaming retention and incremental computation

**Prerequisites:** WP-04, WP-10, WP-15.  
**Requirements:** DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01.  
**Owns:** retention, ingestion policy, running stats and observable update outcomes.

- Add bounded count/event-time windows, explicit watermark/lateness policy and eviction semantics.
- Add queue capacities, backpressure and optional explicitly lossy modes with accounting.
- Optimize exact append/correction/removal updates where supported; retain declared batch fallbacks.
- Preserve stable view/selection/annotation identities and follow/freeze transitions.
- Add a deterministic replayable update generator with corrections and out-of-order inputs.

**Acceptance:** FIX-08/09 agree with independent full recomputation after every selected operation, including retention. Queued and committed acknowledgements differ; no silent loss or implicit watermark advancement.

### WP-19 — Bounded scheduling, caches and dense representation

**Prerequisites:** WP-12, WP-16, WP-18.  
**Requirements:** STM-04, STM-05, SCN-04, GPU-02, QLT-04.  
**Owns:** worker scheduling, cancellation, stage caches and screen-density preparation.

- Implement active-plus-newest-pending scheduling or an equivalent bounded policy.
- Present monotonic compatible data revisions while rejecting incompatible spec/viewport/layout completions.
- Optimize invalidation by source, style, layout, viewport and interaction changes.
- Add line envelope reduction and bucketed candle rendering with retained exact lookup.
- Track lag, queue/work sizes, raw/prepared/rendered counts and disposal resources.
- Preserve fair progress across a dashboard, including one expensive chart.

**Acceptance:** FIX-11 proves no refresh starvation under continuous updates; old jobs cannot overwrite newer scenes. Tests exercise out-of-order worker completions, resize/theme/viewport changes and disposal. PERF-01/02 preliminary measurements identify concrete bottlenecks.

### WP-20 — Coherent exports during live interaction

**Prerequisites:** WP-13, WP-17, WP-18, WP-19.  
**Requirements:** EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04.  
**Owns:** runtime/export snapshot integration and resource lifetime checks.

- Capture coherent multi-dataset, definition, viewport, annotation/theme/font and quality revisions.
- Implement visible-view/full-domain export without freezing ingestion.
- Rebuild for publication resolution and provide explicit interaction-state inclusion settings.
- Bound simultaneous export jobs/snapshot retention and release resources on success, error and cancellation.
- Exercise stream updates and annotation edits while export is deliberately slowed.

**Acceptance:** FIX-14 and PERF-05 preliminary checks show one consistent figure with no mixed revisions. All M3 packages together establish G3; basic streaming alone does not.

### WP-21 — Correctness, fidelity and supported-platform hardening

**Prerequisites:** WP-14, WP-17, WP-20.  
**Requirements:** SCP-03, QLT-01, QLT-02, QLT-03, GPU-02, GPU-03, BND-03, BND-04.  
**Owns:** full fixture/CI matrix, host QA and diagnostic coverage.

- Complete FIX-01–FIX-18 across required features and repeat binding proofs after the complete built-in grammar.
- Run operation-specific numerical/property cases, malformed input, transaction replay, lifecycle and concurrency scheduling tests.
- Inspect all required export/font/theme fixtures and actual supported native behavior.
- Run core on macOS/Linux, WASM target/runtime proofs, and declared GPUI target checks.
- Produce actionable capability/accessibility limitations and close unexplained visual/semantic regressions.

**Acceptance:** every required fixture has an evidence artifact and a passing status on its declared environment. Unsupported environments are listed accurately. No ignored or mocked essential integration test is counted as proof.

### WP-22 — Measured performance and sustained-load release gate

**Prerequisites:** WP-19, WP-20.  
**Requirements:** STM-01, STM-03, STM-04, STM-05, EXP-03, QLT-04.  
**Owns:** reproducible benchmark runs, profiles and justified optimizations.

- Run PERF-01–PERF-05 using the G0 protocol, including the 30-minute sustained update workload.
- Account for accepted/rejected/dropped/coalesced operations and compare retained results against known reference outcomes.
- Measure frame/input latency, ingest-to-present lag, worker/index/preparation cost, memory plateau and snapshot disposal.
- Optimize measured bottlenecks without changing statistical/provenance/fidelity semantics.
- Rerun only benchmarks and fixtures affected by each concrete optimization, then the required final gate.

**Acceptance:** targets and behavioral budgets pass on recorded hardware or a material contract revision is explicitly documented as unresolved. Do not reduce retained data, silently drop rows, or hide aggregation to manufacture a passing result.

### WP-23 — Production documentation and release readiness

**Prerequisites:** WP-21, WP-22.  
**Requirements:** SCP-01, SCP-02, SCP-03, ARC-04, BND-01, QLT-05, QLT-06.  
**Owns:** final API/schema docs, examples, changelog, evidence index and release candidate.

- Reconcile implementation against every specification ID and every G4 criterion.
- Publish in-repository recipes, layered examples, a custom extension tutorial, streaming/interaction guides and publication recipes.
- Document dependency/toolchain/platform support, font/export caveats, data ownership, binding proof scope, semver/schema compatibility and source provenance.
- Package a local release candidate with reproducible build/check instructions and a complete requirement/evidence index.
- Record future Python/browser distribution and advanced chart-family work separately.

**Acceptance:** a new developer can build/run the gallery, execute fixture suites and understand supported features from the repository. G4 has no unresolved required feature or unexplained failing evidence. Package publication/deployment is a separate release action, not a condition for this local implementation handoff.

## 6. Dependency and ownership coordination

Work packages are deliberately bounded; this table shows useful independent lanes after their common contracts exist. It is an assignment guide, not an instruction to spawn agents automatically.

| Ready point | Work that can proceed independently | Integration boundary |
| --- | --- | --- |
| After WP-02 | WP-03 renderer/font proof and WP-04 data/transactions | Shared scene/service types and selected dependency line. |
| After WP-06 and WP-03 | WP-07 native host and WP-08 headless export | One compiled scene/text-resource contract. |
| After WP-15 | WP-16 input/indexes and WP-18 ingestion/retention | Stable targets, action schema, snapshot revisions. |
| After WP-16 | WP-17 linked/editing controls and remaining WP-18/19 runtime work | Presented-scene ownership and gesture lifecycle. |
| After WP-20 | WP-21 correctness/platform QA and WP-22 performance | Shared fixture definitions and unmodified semantic baselines. |

Assign one integration owner for public core interfaces, portable schema/version changes, and workspace dependency changes. Independent developers should own distinct modules or changesets; shared-interface changes land before dependents adopt them. When a real repository uses branches/worktrees, follow its existing workflow and avoid editing the same files concurrently.

Every task handoff records:

| Field | Required content |
| --- | --- |
| Package and revision | WP ID; specification/document revision; starting repository commit. |
| Scope | Requirement IDs, owned paths, dependencies and explicit exclusions. |
| Contract change | Public API/schema/behavior changes and affected consumers. |
| Evidence | Exact commands/environment, fixture IDs, inspected visuals and performance results. |
| State | Done, partial or blocked, with unresolved requirement IDs. |
| Resume note | Next concrete step, known issue and artifact paths. |

The status ledger should include a row for every work package with state, owner, commit/PR, requirement IDs, evidence links, blockers, and next action. States are NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED and DONE. A gate has its own status and cannot pass merely because its package rows say DONE.

## 7. Definition of done and change discipline

A package is DONE only when its required behavior exists in production code, relevant independent checks pass, examples/diagnostics are updated, and the next package can consume its documented interface. Record an actual run and result; never infer a pass from test code being present.

Use tests that constrain behavior: analytical statistical expectations, conservation/round-trip properties, transaction atomicity, update-versus-batch equivalence, action traces, native/export visual inspection and measured workloads. Do not mirror private implementation code in expected values. Do not add a large test matrix for a trivial documentation edit; use checks proportional to the change.

Do not bypass a failing gate by weakening fixtures, increasing tolerances without numerical evidence, catching errors and returning empty success, making required paths feature-disabled, skipping platform tests, or silently dropping/aggregating data. Explain a true capability limitation and keep the requirement open.

If a requirement is ambiguous, use the smallest coherent interpretation consistent with the specification and existing examples, record the decision, and continue. If two requirements conflict or a required capability cannot be implemented with the chosen host, produce a concrete decision record with evidence, affected requirements and proposed remedies; continue independent work. Do not invent authorization to weaken scope.

Any semantic change updates the relevant specification ID, tests, portable schema/migration policy and user-facing documentation together. Refactoring without behavior change should not require a schema version bump. Freeze interface names only after working consumers validate ergonomics.

## 8. Agent kickoff prompt

Copy the following prompt into the assigned repository after adding the three documents:

> Within the assigned task scope, build the Rust-native GPUI chart library defined by docs/spec/gpui-charts-specification.md, docs/impl_plans/gpui-charts-implementation-plan.md, and docs/spec/gpui-charts-migration-plan.md. Read repository instructions and those documents before changing code. The specification is authoritative for behavior; the implementation plan defines work packages and gates; the migration plan supplies rationale. An infrastructure-only or review-only assignment does not authorize library implementation.
>
> Inspect current implementation and docs/implementation-status.md. If no implementation exists, start with WP-01 and initialize the ledger. Otherwise select the earliest ready incomplete work package, preserving completed work. Implement a complete reviewable slice through its acceptance criteria, run the relevant checks on supported environments, and update the ledger with requirement IDs, commands, results, limitations and the next ready step.
>
> Use native Rust for the shared grammar, statistics, scales, layout, scenes and interaction state. Keep GPUI, Python, browser and export host concerns at their specified boundaries. Preserve the specified data identity, statistical semantics, theme/publication requirements and streaming behavior. Do not substitute screenshots, webviews or foreign chart engines for the native implementation.
>
> Make routine implementation decisions within the contract and record consequential ones as ADRs. Do not silently weaken requirements or count scaffolds, ignored tests, compile-only binding checks or unmeasured claims as completed features. When blocked, record concrete evidence and continue ready independent work. Report what was implemented, which requirements and gates passed, what remains open, and the next package.

For a narrowly assigned agent, prepend: “Your assigned package is WP-XX; own only these paths and coordinate shared-interface changes through the integration owner.” The full dependency requirements still apply.

## 9. Requirement traceability

The table below assigns every normative requirement to implementing/reviewing packages. The specification supplies detailed expected behavior and fixture/gate IDs; package acceptance criteria identify the evidence. Multiple owners reflect an early proof followed by completion/hardening, not duplicated engines.


| Requirement | Implementation/review packages |
| --- | --- |
| SCP-01 | WP-01, WP-14, WP-23 |
| SCP-02 | WP-01, WP-14, WP-23 |
| SCP-03 | WP-01, WP-21, WP-23 |
| ARC-01 | WP-02 |
| ARC-02 | WP-02, WP-08, WP-09 |
| ARC-03 | WP-02, WP-04, WP-14 |
| ARC-04 | WP-01, WP-03, WP-23 |
| DAT-01 | WP-04, WP-09 |
| DAT-02 | WP-04 |
| DAT-03 | WP-04, WP-18 |
| DAT-04 | WP-04, WP-18 |
| DAT-05 | WP-04, WP-06, WP-10, WP-11 |
| DAT-06 | WP-04, WP-05, WP-10, WP-17, WP-18, WP-20 |
| GRA-01 | WP-05, WP-14 |
| GRA-02 | WP-05 |
| GRA-03 | WP-05, WP-10 |
| GRA-04 | WP-05, WP-10 |
| GRA-05 | WP-10 |
| GRA-06 | WP-05, WP-11 |
| GRA-07 | WP-12 |
| GRA-08 | WP-05, WP-10, WP-12, WP-14, WP-18 |
| SCL-01 | WP-06, WP-11, WP-16 |
| SCL-02 | WP-06, WP-11 |
| SCL-03 | WP-11 |
| SCL-04 | WP-06, WP-11 |
| SCL-05 | WP-06, WP-11, WP-12 |
| LAY-01 | WP-06, WP-12 |
| LAY-02 | WP-03, WP-06, WP-12, WP-13 |
| LAY-03 | WP-12, WP-13, WP-17 |
| LAY-04 | WP-03, WP-08, WP-13 |
| THM-01 | WP-13 |
| THM-02 | WP-13 |
| THM-03 | WP-13, WP-14 |
| SCN-01 | WP-02, WP-05, WP-11 |
| SCN-02 | WP-05, WP-14 |
| SCN-03 | WP-03, WP-07, WP-08, WP-11, WP-14 |
| SCN-04 | WP-07, WP-15, WP-16, WP-19, WP-20 |
| GPU-01 | WP-03, WP-07 |
| GPU-02 | WP-07, WP-19, WP-21 |
| GPU-03 | WP-03, WP-13, WP-17, WP-21 |
| INT-01 | WP-07, WP-15, WP-17 |
| INT-02 | WP-15 |
| INT-03 | WP-07, WP-16 |
| INT-04 | WP-16, WP-17 |
| INT-05 | WP-15, WP-16, WP-17 |
| INT-06 | WP-14, WP-15, WP-16, WP-17 |
| STM-01 | WP-18, WP-22 |
| STM-02 | WP-15, WP-18, WP-20 |
| STM-03 | WP-18, WP-22 |
| STM-04 | WP-19, WP-22 |
| STM-05 | WP-16, WP-19, WP-22 |
| EXP-01 | WP-03, WP-08, WP-13 |
| EXP-02 | WP-03, WP-08, WP-13 |
| EXP-03 | WP-08, WP-20, WP-22 |
| EXP-04 | WP-08, WP-13, WP-20 |
| BND-01 | WP-02, WP-09, WP-14, WP-23 |
| BND-02 | WP-09 |
| BND-03 | WP-09, WP-21 |
| BND-04 | WP-09, WP-21 |
| QLT-01 | WP-02, WP-04, WP-07, WP-09, WP-15, WP-18, WP-21 |
| QLT-02 | WP-10, WP-21 |
| QLT-03 | WP-03, WP-21 |
| QLT-04 | WP-03, WP-19, WP-20, WP-22 |
| QLT-05 | WP-01, WP-02, WP-14, WP-23 |
| QLT-06 | WP-23 |
