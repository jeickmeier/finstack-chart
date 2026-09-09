# Rust-native GPUI Charts — Implementation Plan

Document version: 0.5.0  
Date: 7 September 2026  
Status: Executable project handoff; current progress is tracked in the [status ledger](../implementation-status.md).
Companion documents: [Project specification](../spec/gpui-charts-specification.md) and [Migration and architecture rationale](../spec/gpui-charts-migration-plan.md).

The [infrastructure setup plan](package-infrastructure-plan.md) covers the repository-only bootstrap. Package shells do not complete WP-02 or any release gate.

The [primary authoring API plan](primary-authoring-api-plan.md) owns AP-00–09,
AUT-01–09 / FIX-AUTH00–09 and G-AUTH. Per owner instruction it assumes the original
WP-01–23 are complete and plans the subsequent API refactor; it does not rewrite
their actual status or rebuild their algorithms. Every delivered feature must be
reachable through the primary API. Additional D3/GG work retains its semantic owner
and integrates through this surface. The refactored release requalifies affected
behavior/performance and requires G-AUTH; historical evidence is not retroactively
reclassified. [ADR-013](../adr/013-primary-authoring-api.md) records ownership.

The [Phase 2 parity implementation plan](phase-2-parity-implementation-plan.md)
consolidates all eight D3 reviews and the ggplot2 review. It owns the combined sequence,
shared ownership and GG-00–19 packages under specification GG2-01–12. WP-01–20 retain
the Phase 1 foundation/interactive scope; Phase 2 can begin independent contracts and
kernels after WP-14, then waits for the named runtime prerequisites. WP-21–23 remain
the one final hardening/release sequence after G-PARITY. Existing D3 inventories and
package IDs remain authoritative for their detailed work. Historical budgets below
exclude the new ggplot2 scope; use the secondary plan's historical allowances and
remaining-work reconciliation. Original-scope WP completion is retained; expanded
parity and changed authoring paths require the specified requalification.

The [D3 shape parity plan](d3-shape-parity-plan.md) adds required SHP-01–10,
WP-S01–WP-S08 and FIX-S01–FIX-S09. It contains the full API inventory, current code
gaps, compatibility decisions and acceptance evidence. Completed WP-10/11/14 retain
their original scope; their completion does not establish shape parity.

The [D3 path parity plan](d3-path-parity-plan.md) refines that shared foundation
with PTH-01–06, WP-P01–04, FIX-P01–06 and G-PATH. It owns the complete path API/state/
arc/serialization inventory. Path packages implement the common engine once; WP-S01
consumes it. The provisional 9–16 developer-days overlap the existing shape foundation
estimate and must not be counted twice. Historical scene/Bézier support is not path parity.

The [D3 scale-chromatic parity plan](d3-scale-chromatic-parity-plan.md) adds required
CHR-01–06, CP-01–05, FIX-21 and G-CHROMATIC in specification 0.3.0. It owns the catalog
inventory and incremental implementation plan; WP-IP owns generic interpolation; SP-04 retains
scale-family ownership. Its provisional 9–16 engineer-days exclude shared SP work and
are additional to the original budget. Re-estimate after CP-01.

The [D3 color parity plan](d3-color-parity-plan.md) adds COL-01–06, CLR-01–05,
FIX-C01 and G-COLOR to the same 0.3.0 contract. It owns color values, parsing,
conversion, manipulation, formatting and portable paint integration. SP-04 owns
scale integration; the [interpolation plan](d3-interpolate-parity-plan.md) WP-IP04
owns interpolation and consumes CLR-03. CP-01–05 retain the named palette catalog.
Additional effort is provisionally 13–22 engineer-days, excluding SP-04 and final
hardening; re-estimate after CLR-01. Existing byte-color evidence keeps its old scope.

The [D3 interpolation parity plan](d3-interpolate-parity-plan.md) adds ITP-01–08,
WP-IP01–07, FIX-I01 and G-INTERPOLATE. It owns shared interpolation algorithms and
transfers that work from SP-04; scales and axis transitions consume the core engine.
Historical alpha evidence does not establish interpolation parity.

The [D3 hierarchy parity plan](d3-hierarchy-parity-plan.md) adds HIR-01–08,
WP-H01–08, FIX-H01 and G-HIERARCHY to specification 0.3.0. It owns the full
constructor/method/layout inventory and source-backed gaps. Hierarchy is required
production scope; completed Cartesian packages do not prove it is implemented.

## 1. How to use this project package

The [D3 scale parity plan](d3-scale-parity-plan.md) adds SCL-06–08, SP-01–07,
FIX-20 and G-SCALE to production scope. It owns the scale inventory, compatibility
changes, implementation sequence and acceptance cases. Original WP-06/11 and G2
evidence remains valid for its recorded 0.1.0 scope, not D3 scale parity. The original
effort budget excludes this addition; re-estimate it after SP-01.

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
| M-SHAPE — D3 shape completion | WP-S01–WP-S08 | 8–13 additional developer-weeks, provisional | G-SHAPE: full generators/layouts/custom protocols and integrated native/export/binding evidence. |
| M-INTERPOLATE — D3 interpolation completion | WP-IP01–07 | 27–45 engineer-days, provisional; includes interpolation transferred from SP-04 | G-INTERPOLATE: full standalone API and integrated evidence. |
| M-HIERARCHY — D3 hierarchy completion | WP-H01–WP-H08 | 35–57 additional engineer-days, provisional; shared shape work excluded | G-HIERARCHY: full standalone operations/layouts and integrated native/export/binding/history evidence. |
| M-PARITY — Phase 2 integration and ggplot2 completion | P2-00, all eight D3 lanes, GG-00–19 | Historical GG-only 182–315 engineer-days plus P2-00 2–4 days; reconcile delivered AP/legend work before estimating remaining effort; D3/shared work and final hardening excluded | G-PARITY: G3, eight D3 gates and G-GGPLOT; detailed sequence and risks in the secondary plan. |
| M4 — Production hardening | WP-21–WP-23 | 4–6 weeks | G4: all requirement/fixture/performance/platform/documentation evidence. |

Cumulative original-scope planning ranges: alpha 9–13 weeks; interactive beta 12–18 weeks; scoped production 16–24 weeks. These do not include the added M-SHAPE, scale, axis or interpolation effort. Subtract transferred interpolation work from scale estimates when combining budgets. Re-estimate the expanded delivery schedule after WP-S01 and WP-S02; independent algorithm work can overlap M3, but WP-S08 consumes WP-16/18/20. Production Python wheels/viewer/notebooks, a browser rendering product, and chart families outside the explicitly required parity inventories remain excluded. The original totals also exclude M-HIERARCHY; re-estimate after WP-H01/H02. AI availability does not remove the need to run platform, visual, and sustained-load checks.

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

WP-02's minimal Rust contracts and synchronous service boundaries are recorded in
[ADR-002](../adr/002-minimal-core-contracts.md). Authoring/compiler and wire-schema
decisions continue in their assigned packages; the foundational types do not freeze them.

## 5. Work packages

Every package below includes requirements, prerequisites, deliverables, and evidence. Requirements remain incomplete until their full scope is implemented even if an early package proves a subset.

Supplemental WP-S01–08 are defined once in the
[shape package table](d3-shape-parity-plan.md#work-packages-and-dependency-order).
WP-P01 starts after WP-14; WP-P02 → WP-P03 → WP-P04 deliver the shared path foundation.
WP-S01 completes after WP-14 and WP-P04. WP-15–20 keep their existing prerequisites; coordinate
shape semantics with WP-16 hit testing, WP-18 recomputation, WP-19 reduction/caches
and WP-20 snapshots. WP-S08 closes their additional shape integration before final
WP-21/22/23 acceptance. The status ledger tracks each supplemental package separately.

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

**Prerequisites:** WP-14, WP-17, WP-20, WP-S08, WP-AX06, SP-07, CP-05, CLR-05, WP-IP07, WP-P04, WP-H08, GG-19 and G-PARITY.  
**Requirements:** SCP-03, QLT-01, QLT-02, QLT-03, GPU-02, GPU-03, BND-03, BND-04.  
**Owns:** full fixture/CI matrix, host QA and diagnostic coverage.

- Complete FIX-01–FIX-18 across required features and repeat binding proofs after the complete built-in grammar.
- Include GG2-01–12 / FIX-GG00–19 and G-GGPLOT; verify the full Phase 2 compatibility profile, device/platform matrix, mathematical/geographic/model capabilities and combined reference/native/export/host evidence.
- Include FIX-S01–FIX-S09 and the complete SHP-01–10 inventory; preserve G2's recorded Cartesian scope and require G-SHAPE evidence for the expanded release.
- Include AXIS-01–07/FIX-19 and require G-AXIS evidence for D3 axis parity.
- Include CHR-01–06/FIX-21 and require G-CHROMATIC for the full named catalog, standalone host operations, mapped color/guide and update evidence.
- Include ITP-01–08/FIX-I01 and require G-INTERPOLATE: standalone operations, typed adaptations, actual bindings and integrated consumer evidence.
- Include SCL-06–08/FIX-20 and require G-SCALE evidence, including standalone scale operations and applicable chart/update/guide behavior.
- Include PTH-01–06/FIX-P01–06 and require WP-P04/G-PATH before WP-21 acceptance for standalone paths and renderer/binding consumers; reuse the same foundation evidence in FIX-S01.
- Include COL-01–06/FIX-C01 and G-COLOR: standalone color methods, exceptional-channel wire semantics and integrated paint/update evidence.
- Include HIR-01–08/FIX-H01-A–H and require G-HIERARCHY, including standalone node operations, all layouts/tilers/helpers, custom protocols and stateful resquarify histories.
- Run operation-specific numerical/property cases, malformed input, transaction replay, lifecycle and concurrency scheduling tests.
- Inspect all required export/font/theme fixtures and actual supported native behavior.
- Run core on macOS/Linux, WASM target/runtime proofs, and declared GPUI target checks.
- Produce actionable capability/accessibility limitations and close unexplained visual/semantic regressions.

**Acceptance:** every required fixture has an evidence artifact and a passing status on its declared environment. Unsupported environments are listed accurately. No ignored or mocked essential integration test is counted as proof.

### WP-22 — Measured performance and sustained-load release gate

**Prerequisites:** WP-19, WP-20, WP-S08, SP-07, CP-05, CLR-05, WP-IP07, WP-H08, GG-19 and G-PARITY.  
**Requirements:** STM-01, STM-03, STM-04, STM-05, EXP-03, QLT-04.  
**Owns:** reproducible benchmark runs, profiles and justified optimizations.

- Run PERF-01–PERF-05 using the G0 protocol, including the 30-minute sustained update workload.
- Measure GG-19's declared bin/model/density/contour/facet/guide/math/geographic workloads; record generated work, numerical accuracy, allocation/resource limits and supported update latency separately from simple-line frame targets.
- Measure the supplemental shape workloads in the parity plan, including spline/arc command growth, streamgraph recomputation, hit indexes and publication lowering; do not infer these from simple-line timings.
- Include the path plan's append/replay, serialization bytes, arc subdivision and retained-snapshot memory workloads, sharing arc measurements with the shape lane.
- Include axis-heavy resize/update and transition cases from WP-AX06 in the existing workloads; report guide/tick counts and layout/formatting costs.
- Measure CP-05 catalog footprint, evaluator allocations, large mapped-color scenes and repeated palette changes using the existing protocol. The [CP-05 measurements](../evidence/phase-2-chromatic-integration-2026-09-09.md) supply initial component/update timings and a WASM capacity plateau; native allocation counts and sustained-load budgets remain WP-22.
- Include WP-IP07 factory-versus-sampling costs, structured-output allocations, color/transform/zoom and shared consumer workloads under the existing PERF protocol.
- Measure SP-07 piecewise/category lookup, exact quantile retraining and interpolation allocations; use the existing PERF protocol and disclose added workload sizes. The [SP-07 measurements](../evidence/phase-2-scale-integration-2026-09-09.md) supply initial kernel timings and a WASM memory plateau, not the sustained-load pass.
- Include CLR-05 parse/conversion/palette and color-only update workloads; verify no per-mark parsing or unintended numerical recomputation. The [CLR-05 component/update measurements](../evidence/phase-2-color-acceptance-2026-09-09.md) are initial workload evidence, not release budget acceptance.
- Measure WP-H08 hierarchy workloads: balanced/deep/wide topology, skewed packing, treemap history reuse/reset, hit indexes, publication and cache/snapshot memory; record algorithm-specific budgets before measurement.
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
- Reconcile G-PARITY and GG2-01–12, publishing the exact reference/profile/argument coverage, typed adaptations and supported device/host evidence without reclassifying missing capabilities as passes.
- Publish in-repository recipes, layered examples, a custom extension tutorial, streaming/interaction guides and publication recipes.
- Document dependency/toolchain/platform support, font/export caveats, data ownership, binding proof scope, semver/schema compatibility and source provenance.
- Package a local release candidate with reproducible build/check instructions and a complete requirement/evidence index.
- Record future Python/browser distribution and advanced chart-family work separately.

**Acceptance:** a new developer can build/run the gallery, execute fixture suites and understand supported features from the repository. G4 has no unresolved required feature or unexplained failing evidence. Package publication/deployment is a separate release action, not a condition for this local implementation handoff.

## 6. Dependency and ownership coordination

The [Phase 2 plan](phase-2-parity-implementation-plan.md#5-entry-package-and-coordinated-execution-waves)
owns cross-lane scheduling and the additional ggplot2 consumers. Its P2-00 coordinates
one wire/resource/reference-tool strategy; existing entry packages retain their own
inventory/oracle work. G-PARITY precedes final WP-21/22 acceptance; a package can start
independent preparation earlier without advancing that gate. No D3 certification
package depends on GG-19, and no producer depends on its final consumer's certification.

The [current API handoff](phase-2-parity-implementation-plan.md#41-integration-through-the-current-primary-api)
extends Data/Plot/Chart/Output and the AP-00 register. First reconcile P2-00 contracts
with active AP owners, then assign GG-00 and ready D3 entry packages. GG-01 now owns
remaining acceptance of the delivered shared legend painter. GG-02 carries alternate
profile semantics through execution/wire/capture; WP-AX01 migrates primary axis handles,
names, layer bindings and host navigation. Every semantic package supplies primary
API usage and actual applicable host/export evidence using AP-07's shared syntax.
Pure kernels need not wait for G-AUTH; no package/gate is closed by this plan update.

**Shared path/shape ownership:** [WP-P01–04](d3-path-parity-plan.md#work-packages-and-dependency-order)
own the builder, path sink, arcs, SVG precision and primitive-level native/export/binding
proofs. WP-S01 consumes WP-P04 and owns shape inventories, generator/custom protocols
and shape-specific FIX-S01 cases. Reference both lanes from one path representation
ADR and one pinned d3-path corpus. There is no dependency from WP-P01–04 back to
WP-S01; the shape lane must not build a competing serializer or arc kernel.

Scale parity starts at SP-01 after WP-14: SP-02 follows SP-01 and WP-IP02;
SP-03 follows SP-01; SP-04 requires both, CLR-04 and WP-IP03/04; SP-05 follows SP-04; SP-06 follows SP-05; SP-07 also consumes
WP-16/18/20. See the scale plan for exact owned paths and acceptance. Preserve the
existing WP-15–20 assignment; required parity integration precedes WP-21/22/23.

**Shared scale/axis ownership:** SP-02–06 own mapping, scale tick generation, nice,
numeric/time formatting algorithms, and band metrics. WP-AX02 owns per-guide argument,
explicit-value and formatter precedence and consumes those APIs; it must not duplicate
their algorithms. WP-AX02's complete built-in acceptance requires SP-03, SP-05 and
SP-06 (guide/provider work can begin earlier). Use one pinned d3-scale oracle manifest
for FIX-19 and FIX-20. Guide geometry/styles/transitions remain WP-AX03–06. Both
workstreams share versioned timezone/locale resources and an integration owner.

### Required scale-chromatic lane (specification 0.3.0)

The [chromatic plan](d3-scale-chromatic-parity-plan.md) owns exact deliverables and
FIX-21 acceptance. CP-01 follows WP-14; CP-02 follows CP-01; CP-03 requires CP-02 and
SP-04; CP-04 follows CP-03; CP-05 requires CP-04, SP-07 and WP-20. CP-05 precedes
WP-21/22, and WP-23 documents G-CHROMATIC alongside G-SCALE. CP-01 coordinates oracle
and schema contracts with SP-01. CLR-03 owns color conversion; WP-IP04 owns interpolation; SP-04 owns scale algorithms;
CP-03 owns named recipes/tables and consumes those primitives. SP-04 and SP-07 do not
wait for CP-05. Keep one integration owner and preserve active WP-15–20 work.

### Required color parity lane (specification 0.3.0)

The [color package table](d3-color-parity-plan.md#work-packages-and-dependency-order)
owns deliverables and acceptance. CLR-01 follows WP-14; CLR-02 follows CLR-01;
CLR-03 follows CLR-02 and supplies WP-IP04; CLR-04 follows CLR-03; CLR-05 requires CLR-04, SP-04 and WP-20.
WP-21/22 require CLR-05; WP-23 inherits those gates. CLR-01/SP-01/CP-01 share oracle
provenance and coordinate migrations with active state work. CLR-03 owns color math;
CLR-04 owns color-value wire/paint lowering; WP-IP04 owns interpolators; SP-04 owns normalization;
CP-02/03 own palette catalog data and recipes. CLR-05 never depends on SP-07, CP-05
or WP-21/22. Keep existing WP-15–20 prerequisites and assignments intact.

### Required interpolation parity lane (specification 0.3.0)

The [interpolation package table](d3-interpolate-parity-plan.md#work-packages-and-dependency-order)
is authoritative for WP-IP01–07 prerequisites, ownership, effort and acceptance. WP-IP01
starts after WP-14 and coordinates the shared reference lock with SP-01/WP-AX01.
WP-IP02 owns numeric/round/spline/composition kernels; WP-IP03 owns structured values;
WP-IP04 owns color interpolation using CLR-03; WP-IP05 owns transforms/zoom. SP-02 consumes WP-IP02; SP-04 consumes
WP-IP03/04. WP-AX05 consumes WP-IP02/05 and retains axis transition lifecycle ownership.
WP-IP06 integrates portable/chart/runtime consumers after CLR-04, SP-04 and WP-16/19/20.
WP-IP07 waits for WP-IP06, SP-07, WP-AX06, CLR-05 and CP-05; neither scale nor axis certification waits
for WP-IP07. WP-21/22 require WP-IP07 and WP-23 inherits it. Existing WP-15–20 prerequisites
remain unchanged. Do not create duplicate kernels, oracle locks or performance budgets.

### Required axis parity lane (specification 0.3.0)

The [axis parity plan](d3-axis-parity-plan.md) owns the live-source comparison,
deliverables, owned areas, estimates and FIX-19 acceptance. It adds **14–26 engineer-days**
provisionally to the original 0.1.0 budget, excluding shared work owned by scale parity.
Execute one bounded package at a time, preserving concurrent action/state work.

| Package | Prerequisites | Requirements |
| --- | --- | --- |
| WP-AX01 — Guide contract and reference harness | WP-14 | AXIS-01, AXIS-07 |
| WP-AX02 — Tick selection and formatting | WP-AX01, SP-03, SP-05, SP-06 | AXIS-02, AXIS-03 |
| WP-AX03 — Axis geometry and bounded layout | WP-AX02 | AXIS-04 |
| WP-AX04 — Styling and publication components | WP-AX03 | AXIS-05 |
| WP-AX05 — Axis updates and transitions | WP-AX04, WP-15, WP-19, WP-IP02, WP-IP05 | AXIS-06 |
| WP-AX06 — Parity certification and documentation | WP-AX05, WP-20, SP-07 | AXIS-01–07, QLT-02, QLT-03 |

WP-AX01 records a conforming ADR for guide identity, the compatibility profile and
schema migration, preserving ADR-005's existing-definition behavior. WP-AX06 is required
before WP-21 certification; WP-22 measures axis-heavy updates in its existing workloads
and WP-23 documents certified coverage. Historical WP-06/11–14 completion retains its
original scope. No new build task exists until its acceptance runner does.

### Required hierarchy parity lane (specification 0.3.0)

The [hierarchy package table](d3-hierarchy-parity-plan.md#work-packages-and-dependency-order)
owns WP-H01–08 prerequisites, deliverables, path ownership and estimates. Begin WP-H01
after WP-14, then WP-H02 topology/operations. Tree/cluster, partition, treemap and packing
kernels depend on that shared topology. WP-H07 integrates these with WP-S03 arcs, WP-S04
links/radial projection and WP-16/18 interaction/update contracts. WP-H08 consumes WP-19/20
scheduling and coherent export before final WP-21/22 acceptance. No shape kernel depends
on hierarchy, and hierarchy does not duplicate shape/scale/axis algorithms.

The lane adds 35–57 engineer-days provisionally, excluding shared shape work and final
platform/performance/release packages. WP-H01 records the conforming ADR, exact reference
identity/license and versioned portable decisions. Keep FIX-H01 distinct from the
scale-chromatic FIX-21 catalog. Do not add a build task until its acceptance runner exists.
Historical G2 and completed Cartesian packages retain their original scope.

### Coordination of existing packages

Work packages are deliberately bounded; this table shows useful independent lanes after their common contracts exist. It is an assignment guide, not an instruction to spawn agents automatically.

| Ready point | Work that can proceed independently | Integration boundary |
| --- | --- | --- |
| After WP-02 | WP-03 renderer/font proof and WP-04 data/transactions | Shared scene/service types and selected dependency line. |
| After WP-06 and WP-03 | WP-07 native host and WP-08 headless export | One compiled scene/text-resource contract. |
| After WP-15 | WP-16 input/indexes and WP-18 ingestion/retention | Stable targets, action schema, snapshot revisions. |
| After WP-16 | WP-17 linked/editing controls and remaining WP-18/19 runtime work | Presented-scene ownership and gesture lifecycle. |
| After WP-20 and G-PARITY | WP-21 correctness/platform QA and WP-22 performance | Shared fixture definitions and unmodified semantic baselines; earlier runner preparation does not close either package. |

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
| SHP-01 | WP-S01, WP-S07 |
| SHP-02 | WP-S02 |
| SHP-03 | WP-S02 |
| SHP-04 | WP-S03 |
| SHP-05 | WP-S04 |
| SHP-06 | WP-S05 |
| SHP-07 | WP-S06 |
| SHP-08 | WP-S01, WP-S07 |
| SHP-09 | WP-S02–WP-S08, WP-16, WP-18, WP-19, WP-20, WP-21 |
| SHP-10 | WP-S01, WP-S08, WP-21, WP-22, WP-23 |
| PTH-01 | WP-P01, WP-P02, WP-P03, WP-P04 |
| PTH-02 | WP-P01, WP-P02, WP-P04 |
| PTH-03 | WP-P01, WP-P02, WP-P04 |
| PTH-04 | WP-P01, WP-P03, WP-P04 |
| PTH-05 | WP-P01, WP-P02, WP-P04, WP-21, WP-22 |
| PTH-06 | WP-P01, WP-P04, WP-21, WP-22, WP-23 |
| HIR-01 | WP-H01, WP-H02, WP-H07, WP-H08 |
| HIR-02 | WP-H01, WP-H02, WP-H07, WP-H08 |
| HIR-03 | WP-H03, WP-H07, WP-H08 |
| HIR-04 | WP-H04, WP-H07, WP-H08 |
| HIR-05 | WP-H06, WP-H07, WP-H08 |
| HIR-06 | WP-H05, WP-H07, WP-H08 |
| HIR-07 | WP-H02–08, WP-S03/04, WP-16, WP-18, WP-19, WP-20 |
| HIR-08 | WP-H01, WP-H08, WP-21, WP-22, WP-23 |
| SCL-01 | WP-06, WP-11, WP-16, SP-02, SP-03, SP-07 |
| SCL-02 | WP-06, WP-11, SP-02, SP-07 |
| SCL-03 | WP-11, SP-02, SP-04, SP-07 |
| SCL-04 | WP-06, WP-11, SP-06, SP-07 |
| SCL-05 | WP-06, WP-11, WP-12, SP-07 |
| SCL-06 | SP-01–07, WP-21, WP-23 |
| SCL-07 | SP-01–07, WP-AX02, WP-21 |
| SCL-08 | SP-01, SP-07, WP-21, WP-22, WP-23 |
| COL-01 | CLR-01, CLR-02, CLR-05 |
| COL-02 | CLR-01, CLR-02, CLR-03, CLR-05 |
| COL-03 | CLR-01, CLR-02, CLR-03, CLR-05 |
| COL-04 | CLR-01, CLR-02, CLR-03, CLR-05 |
| COL-05 | CLR-01, CLR-04, CLR-05, SP-04 |
| COL-06 | CLR-01, CLR-05, WP-21, WP-22, WP-23 |
| CHR-01 | CP-01–05, WP-21, WP-23 |
| CHR-02 | CP-02, CP-05, WP-21 |
| CHR-03 | CP-01, CP-03, CP-05, SP-04, WP-21 |
| CHR-04 | CP-04, CP-05, SP-04, WP-20, WP-21 |
| CHR-05 | CP-01, CP-04, CP-05, WP-21, WP-23 |
| CHR-06 | CP-01, CP-05, WP-21, WP-22, WP-23 |
| ITP-01 | WP-IP01, WP-IP03, WP-IP07 |
| ITP-02 | WP-IP02, WP-IP03, WP-IP07 |
| ITP-03 | WP-IP02, WP-IP07 |
| ITP-04 | WP-IP04, WP-IP07 |
| ITP-05 | WP-IP05, WP-IP07 |
| ITP-06 | WP-IP05, WP-IP07 |
| ITP-07 | WP-IP03, WP-IP06, WP-IP07, SP-04, WP-AX05 |
| ITP-08 | WP-IP01, WP-IP07, WP-21, WP-22, WP-23 |
| AXIS-01 | WP-AX01, WP-AX06, WP-21 |
| AXIS-02 | WP-AX02, WP-AX06, WP-21 |
| AXIS-03 | WP-AX02, WP-AX06, WP-21 |
| AXIS-04 | WP-AX03, WP-AX06, WP-21 |
| AXIS-05 | WP-AX04, WP-AX06, WP-21 |
| AXIS-06 | WP-AX05, WP-AX06, WP-20, WP-21 |
| AXIS-07 | WP-AX01, WP-AX06, WP-21, WP-22, WP-23 |
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
| GG2-01 | P2-00, GG-00, GG-02, GG-19, WP-23 |
| GG2-02 | GG-02, GG-06, GG-18, GG-19 |
| GG2-03 | GG-03, GG-04, GG-18, GG-19 |
| GG2-04 | GG-01, GG-05, GG-18, GG-19 |
| GG2-05 | GG-06, GG-09, GG-10, GG-11, GG-18, GG-19 |
| GG2-06 | GG-07, GG-08, GG-09, GG-10, GG-11, GG-18, GG-19 |
| GG2-07 | GG-12, GG-18, GG-19 |
| GG2-08 | GG-13, GG-15, GG-18, GG-19 |
| GG2-09 | GG-08, GG-14, GG-18, GG-19 |
| GG2-10 | GG-16, GG-18, GG-19 |
| GG2-11 | GG-17, GG-18, GG-19 |
| GG2-12 | P2-00, GG-00, GG-18, GG-19, WP-21, WP-22, WP-23 |
| AUT-01 | AP-00, AP-01, AP-08, AP-09 |
| AUT-02 | AP-02, AP-05, AP-07, AP-09 |
| AUT-03 | AP-02, AP-03, AP-07, AP-09 |
| AUT-04 | AP-04, AP-07, AP-09 |
| AUT-05 | AP-01, AP-05, AP-06, AP-07, AP-09 |
| AUT-06 | AP-02, AP-06, AP-09 |
| AUT-07 | AP-07, AP-08, AP-09 |
| AUT-08 | AP-03, AP-04, AP-07, AP-09 |
| AUT-09 | AP-00, AP-08, AP-09 |
