# Primary authoring API — refactoring and build plan

Date: 7 September 2026. Status: planned, implementation not started.
Authority: specification AUT-01–09 and [ADR-013](../adr/013-primary-authoring-api.md).
Source review: [public API usability](../evidence/public-api-review-2026-09-07.md).
Execution evidence belongs in the [status ledger](../implementation-status.md).

## 1. Outcome and planning baseline

Make the concise authoring API the library's main public API. Every chart capability,
including advanced configuration, must be expressible through this API and its typed
component builders. Applications must not construct or edit normalized grammar/wire
structures to access a built-in feature. Recipes, native charts, export, bindings and
custom extensions all use the same authoring and execution contracts.

Per owner instruction, **assume original WP-01–23 are complete** for this plan. Their
data, statistics, renderers, interaction, streaming, scheduling, export and validation
infrastructure are existing capabilities to preserve, not work to rebuild. This is a
prospective refactor of that completed baseline. The actual checkout is
`127fe2d4853f89b62ba59a17248485e3c378ba60` plus uncommitted changes; its historical
ledger is not changed to DONE by this assumption. AP-00 records the actual completed
baseline revision when implementation begins, without reopening the original program.

The additional D3 and GG packages retain their own scope and evidence. They are not
silently declared complete by the original-WP assumption. Every delivered capability
must be reachable through the primary API; any capability delivered later must add its
primary API integration in the same package. This plan does not reimplement their
algorithms or duplicate their acceptance inventories. A release claiming those parity
capabilities still needs their existing gates as well as G-AUTH.

This assignment changes planning/authority documents only. It does not implement the
new API, change wire schemas, publish packages or certify a completed refactor.

## 2. Fixed architecture decisions

1. **One primary authoring surface in `chart-core`.** Introduce a focused `plot` module
   and a curated `prelude`. Keep specialized component modules discoverable. No new
   façade crate, mandatory framework dependency, macro language or generic plugin
   framework is required. Host crates expose destination-specific constructors and
   adapters, not alternative plot grammars.
2. **A builder produces an immutable plot.** `plot(data)` starts a `PlotBuilder`;
   fluent setters collect intent; `.build()?` produces an owned immutable `Plot` with
   validated normalized definition and data handles. Build resolves fields, defaults,
   identities, operation parameters and structural/stage constraints. Statistical
   execution and destination-dependent validation happen at preparation/capture and
   report the same structured diagnostics. Do not create a second persistent grammar
   AST that can diverge from the normalized definition.
3. **A live chart reuses the existing runtime.** A `Chart` owns the existing store,
   reducer and compiler/cache coordination. Refactor/reuse completed WP runtime code;
   do not add another store, reducer or scheduler behind the new name. `Plot` is
   reusable authored content; distinct `Chart`s have independent view/selection state
   unless explicitly linked. Hosts retain charts and own scheduling/event-loop objects.
4. **One public route for every feature.** Common defaults are concise; specialized
   controls use typed builders within the same grammar. An advanced option is not
   complete if its only route is editing `ChartDefinition`, JSON, or a private scene.
   Inspectable immutable normalized output remains available to integrators.
5. **Standalone utilities remain standalone.** Colors, interpolation, paths, scales
   and hierarchy helpers remain public family modules and feed plot components through
   their existing typed values. Parsing a color must not require constructing a plot.
6. **Versioned semantics stay explicit.** The default constructor retains the
   completed baseline's library defaults, recorded in the built definition. A
   ggplot2/D3 compatibility profile is selected explicitly when those semantics are
   wanted. Concise syntax alone must not change bin closure, grouping, size units,
   statistical ordering or scale policies. Changing a profile invalidates the relevant
   preparation/state just like the existing semantic change contract.
7. **Host services stay at destinations.** Supplied fonts, filesystem save operations,
   GPUI entities, interpreter attachment and worker execution stay outside core. One
   destination context retains supplied resources for many plots; core never scans
   system fonts or starts mandatory threads. Native and publication measurement remain
   distinct services over the same authored content.

Method/type spellings below are proposed until AP-00's API walkthrough; these ownership
and coverage decisions are fixed. Prefer one canonical name per operation and normal
Rust builders; no requirement to overload `+` or create aliases for every spelling.

## 3. Intended developer workflows

These sketches describe acceptance targets, **not currently executable examples**.
They must become compiling external-style examples as their owning AP package lands.

```rust,ignore
use chart_core::prelude::*;

let data = Data::columns()
    .column("time", vec![1.0, 2.0, 3.0])
    .column("value", vec![10.0, 12.0, 11.0])
    .build()?;

let plot = plot(data)
    .aes(aes().x("time").y("value"))
    .layer(line())
    .layer(points().size(2.0))
    .labels(labels().title("Prices").y("USD"))
    .theme(theme_minimal())
    .build()?;

// Explicit font resources were configured once on this export destination.
let svg = output.svg(&plot, Size::mm(180.0, 120.0))?;
```

Typed rows use `Data::rows(rows).field("time", |r| r.time)...build()?`, with optional
stable key accessors. Columns preserve their scalar kinds, null validity and metadata;
timestamp unit/timezone and exact integer data are not inferred through f64. No
mandatory Arrow/Polars dependency or derive macro. Nonnullable accessors may return
plain values; nullable accessors return `Option<T>`. Materialize native callbacks once
per ingestion batch, never per hover/frame; portable reconstruction uses fields or
registered operations rather than serializing closures.

The same composition vocabulary grows to `.layer(histogram().bins(30))`, layer data
and aesthetic overrides, `.facet(facet_wrap("region"))`, `.scale(...)`, `.coord(...)`
and explicit stat/position/after-stat controls. Mapped aesthetics belong in `aes`;
constants belong on geometry/style builders. Stage types remain checked. Inferred
grouping follows the selected profile, with an explicit group override. The façade
must not approximate an unavailable statistic or renderer capability.

For live use, create/retain a `Chart` from the built plot, obtain a dataset handle by
its authored name, submit updates and inspect typed receipts. Atomic multi-dataset
updates use the existing transaction boundary. Programmatic navigation, selection,
annotation editing, inspection, linking and export snapshots operate on that retained
chart. A native adapter mounts the chart once; it does not rebuild it during render.
Exporting a live chart captures its acknowledged coherent state; exporting a static
`Plot` uses explicitly documented initial-state behavior.

## 4. Data, identity and failure contracts

- Generate dataset/field/layer identities and initial revisions for ordinary use.
  Preserve them through cloning, immutable edits, runtime replacement and serialization.
  Allocate fresh identities for independently constructed data/layers. Stable handles
  are scoped to their owner; cross-chart/dataset misuse rejects rather than aliasing.
- Names are the default authoring references; resolve once to IDs with duplicate,
  missing and ambiguous-name diagnostics. Different datasets can have different schemas;
  inherited fields resolve against the selected layer data. Expose typed handles for
  refactor-safe advanced code and reusable transforms, without requiring them initially.
- Generate and return durable row keys for keyless ingestion. Append uses fresh keys;
  correction/removal uses returned or supplied keys. Replacing keyless rows is new
  identity unless retained keys are explicitly supplied. Never infer identity from
  value equality or rebuild keys from current row positions.
- Infer initial facet/category catalogs deterministically from the selected profile
  and source order. Retain catalog identity across updates, with explicit extend,
  fixed/reject, drop and reset policies. Keep frozen dashboard layouts possible.
- Serialize resolved identity/default/profile decisions. Clone/edit preserves stable
  IDs and advances effective revisions; constructing a new unrelated plot does not
  impersonate the old one. Overflow, duplicate names and invalid changes fail atomically.
- Builder failures never mutate an existing `Plot`/`Chart`. Runtime data/actions keep
  their typed applied/already-applied/rejected/conflict/queued outcomes. A queued update
  is not a commit. A convenience update may capture a current base revision but must
  not retry conflicts silently or bypass multi-dataset atomicity/replay semantics.
- Enrich diagnostics with authored dataset/field/layer/aesthetic names while retaining
  exact IDs, stage, stable codes and corrections. Binding errors expose properties,
  not just JSON hidden in a generic exception string.

## 5. Complete capability routing

AP-00 creates a capability register referencing the original and parity inventories;
the table below defines its required families, not a replacement enumeration. Each
delivered option/default/extension boundary needs a route and executable example ID.

| Capability / authoritative requirements | Primary public route | Refactor owner |
| --- | --- | --- |
| All data kinds, typed accessors, named datasets, exact metadata; DAT-01–06 | Data builders, field/data handles, owned plot data and typed receipts | AP-02/05 |
| Layer data/mapping inheritance, expressions/stages, shared transforms, scope/filter/zoom distinctions; GRA-01–08, GG2-02 | Plot/layer/aesthetic/stat/position/transform builders | AP-03 |
| Every geom/recipe, statistics/model family and position; GRA-04–06, SHP, GG2-05/06 | Typed layer components using existing kernels/registries | AP-03 |
| Every scale, named/multiple/secondary axes, color/fill/alpha/shape/size/linewidth/linetype and guides; SCL, COL, CHR, ITP, AXIS, GG2-03/04 | Aesthetic and scale/guide builders; same standalone family values | AP-03/04 |
| Facets, coordinates/geography, layouts/hierarchy, panel identities; GRA-07, HIR, GG2-07/08 | Facet/coordinate/layout components and explicit resources | AP-04 |
| Themes, rich/math text, annotations, labels/alt text, publication furniture/insets and linked panes; THM, LAY, GG2-09 | Theme/text/annotation/figure composition builders over the same plots | AP-04 |
| Queries, focus/accessibility, hover, pan/zoom, selection, controlled state, remapping and capture; INT, GPU | Retained Chart actions/events/inspection plus native hooks | AP-05/06 |
| Linked plots/table, echo prevention and constrained editable annotations; INT, LAY-03 | Existing link/edit controllers consuming stable chart handles | AP-05 |
| Atomic append/upsert/remove/replace, retention, backpressure, correction, follow/freeze, incremental/batch execution; DAT, STM | Chart data handles, transactions, retention and typed outcomes | AP-05 |
| Scheduling, cache invalidation, dense representation, resource/quality limits; STM, SCN, PERF | Existing runtime policies exposed through typed Chart options | AP-01/05/09 |
| Static and live coherent SVG/PDF/PNG and additional supported devices, preview, text/font/DPI/physical size; EXP, GG2-11 | Destination context and shared Chart snapshot/capture | AP-06 |
| Native GPUI and optional Kit hooks, tooltips/controls and platform capability reporting; GPU, SCP-03 | Thin retained native view over Chart; Kit theme/context adapter | AP-06 |
| Native and registered stat/geom/scale/coord/facet/guide/labeller/model/key-glyph extensions; ARC-03, GRA-08, GG2-10 | Public extension traits/registry and ordinary component builders | AP-03/04 |
| Standalone colors, interpolation, scales, paths/shapes and hierarchy helpers; COL, ITP, SCL, PTH, SHP, HIR | Public family modules and common typed values, no dummy plot | AP-04 |
| Python/WASM data, authoring, actions, snapshots, results, disposal and interchange; BND-01–04 | Thin host-native adapters to the same core builders/runtime | AP-07 |

Native-only widgets/painters remain explicitly nonportable with capability errors at
unsupported destinations. That is supported capability reporting, not permission to
omit their native primary-API route. A new standalone kernel needs both its direct API
and every chart consumer required by its original inventory.

## 6. Ordered work packages

Each package lands a reviewable working slice, updates its capability-register rows,
adds migration notes for changed public behavior and records actual evidence. Owners
are roles/paths, not invented assigned people. Do not scaffold hundreds of stubs.

### AP-00 — Freeze the public contract and baseline register

Prerequisite: assumed completed WP-01–23 baseline. Owns docs, public API inventory and
external example contracts; AUT-01/09, QLT-05. Record the baseline commit, selected
public/package/schema versions and supported platform evidence. Inventory all supported
functions, component options, default policies and extension protocols from Rustdoc,
schemas, existing examples and the parity registers. Classify primary/specialist/
compatibility/internal exports. Assign every row to AP-01–09 and its semantic owner.
Record the default profile and name/ID/catalog rules in API docs. Walk through the
section 3 examples and the seven acceptance workflows in the source review, including
complete imports/data/resources. Choose final names and a staged migration release;
do not freeze compile-only sketches as working APIs.

Acceptance FIX-AUTH00: no unowned existing capability, no unknown lifecycle owner,
explicit old→new API map, and an acyclic package/capability integration schedule.
The completion assumption is an input; no old gate is marked passed by this task.

### AP-01 — Refactor one typed runtime behind all entry points

Prerequisite: AP-00. Owns core runtime/session, data store/compiler/reducer integration
and portable boundary; AUT-01/05, ARC, DAT, STM. Extract typed construction, update,
action, inspection and capture entry points from existing runtime implementations as
needed. Route the legacy JSON session through these calls. Retain existing cache,
retention, scheduler and presented-scene ownership semantics. Keep destination objects
outside core; do not require serialize→decode for Rust calls or introduce a second
execution engine. Adapt the native/export constructors incrementally through shims.

Acceptance FIX-AUTH01: typed and legacy routes produce matching independent fixture
results, diagnostics and revision traces; failures preserve prior valid state. Existing
batch/update, stale completion and ownership fixtures pass before any API migration.

### AP-02 — Deliver the primary data/plot/layer path end to end

Prerequisite: AP-01. Owns `chart-core` authoring/prelude and the smallest native/export
consumer bridge; AUT-02/03/06, DAT-01/02, GRA-01. Implement Data/Plot builders, names and
handles, default dataset, layer identity allocation, source inheritance and static
line/point/histogram composition. Build an immutable Plot using the existing normalized
contracts. Supply minimal overloads/adapters so this real plot renders natively and
exports using the existing resource context; broad destination consolidation is AP-06.

Acceptance FIX-AUTH02: compiling external examples from ordinary columns and typed
rows, with no manual IDs/revisions/compiler/schema boilerplate; actual native and
inspected SVG/PDF/PNG output. Test duplicate/missing fields, nullable types, exact
integer/timestamp metadata, independent layer data, stable clone/edit IDs and keyless
replacement semantics. Compare output/provenance against baseline fixtures.

### AP-03 — Expose the complete grammar and aesthetic controls

Prerequisite: AP-02; each added parity capability also requires its semantic owner.
Owns grammar/stat/position/scale/guide authoring and registries; AUT-03/08. Cover every
available source/generated/expression stage, aesthetic, mapping override, geom/stat
override, position, named transform, scale family and guide control. Supply defaults
and explicit independent-data/inheritance overrides. Custom operations enter through
ordinary typed components and the existing registry. Builtin recipes lower to the same
generic components. No duplicate stat/scale dispatch in the new builders.

Acceptance FIX-AUTH03: all delivered grammar register rows have compiling usage and
behavior assertions; wrong-stage compile failures, runtime schema errors, profile
defaults, grouping, scale/guide consistency and shared-transform invalidation pass.
Exercise at least one custom stat and geom in an external crate through the primary API.

### AP-04 — Expose complete design, composition and specialist capabilities

Prerequisite: AP-03 and applicable semantic owners. Owns facet/coordinate/theme/text/
figure builders and specialist utility integration; AUT-04/08. Add short default
facets with explicit catalog controls, all coordinate/layout settings, themes, rich
and mathematical labels, annotations, furniture, insets and multi-plot composition.
Expose required extension boundaries and direct D3 family helpers without exposing
internal pipeline structs as the only control mechanism. Reuse prepared-data sharing
for insets and related plots. Implement no new geometry/model algorithms in this slice.

Acceptance FIX-AUTH04: each available design/specialist register row is reachable;
empty/new facet categories, free/shared axes, explicit broadcasts, linked panes,
fonts/text bounds, coordinates and extension capability errors retain baseline meaning.
Inspect actual native/export design artifacts; standalone helpers need no font or host.

### AP-05 — Make all live features accessible through Chart

Prerequisite: AP-04. Owns primary Chart data/actions/options/linking/edit interfaces;
AUT-05, DAT, INT, STM. Expose dataset handles, transaction builders and receipts,
retention/backpressure policies, controlled/uncontrolled state, query/selection,
navigation, annotation edits/undo, linked charts/table, follow/freeze, scheduling and
dense representation settings. Centralize definition/data updates on the typed runtime
from AP-01. Immutable Plot edits applied to a retained Chart preserve intended IDs and
state, validate the candidate and publish atomically. Capture uses acknowledged state.

Acceptance FIX-AUTH05: replay existing interactive and streaming fixtures exclusively
through the primary API, with update-versus-batch checks. Include conflict/no-op/queued
receipts, multi-dataset rollback, key eviction, link echoes, stale controlled replies,
gesture cancel, out-of-order workers, fair dashboard progress and resource release.

### AP-06 — Complete native, Kit and export integration

Prerequisite: AP-05. Owns `gpui-charts`, optional Kit, `chart-export` and explicit
resource contexts; AUT-06. Replace remaining duplicate session ownership with the
typed runtime. Provide retained native mounting, replaceable tooltip/menu/toolbar and
accessibility hooks. Provide concise bytes export and explicit host save helpers,
physical size/DPI/background/text/device controls and publication preview. Capture
visible/full-domain and selected interaction-state options coherently during ingestion;
preserve supplied font identities and release/cancel bounds. Static and live exports
share the same capture implementation rather than reconstructing state independently.

Acceptance FIX-AUTH06: actual native lifecycle/input/accessibility-hook checks and
inspected supported device artifacts; baseline logical/physical geometry and fonts;
slow export during updates/edits, queue bounds/cancellation and retained snapshot
lifetimes. GPUI is absent from headless dependency/runtime paths; Kit stays optional.

### AP-07 — Make Python/WASM authoring native to the host language

Prerequisite: AP-06. Owns binding APIs, declarations/stubs, converters and proof runner;
AUT-07, BND. Provide ordinary mapping/sequence/column and component builders over core;
remove the need to hand-author three JSON envelopes. Keep batch ingestion and exact
Python integer/JS bigint or explicit exact-value adapters; reject unsafe JS numbers.
Expose typed results/error properties and deterministic lifetime behavior. Forward
defaults, field resolution, compilation and runtime actions to Rust. Thin syntax/data
conversion is allowed; an independent Python/JS grammar compiler is not. Reuse existing
schema tooling; do not build a new binding generator unless the inventory proves need.

Acceptance FIX-AUTH07: actual Python and Node WASM authoring/data/update/action/capture
execution matches independent fixtures and Rust. Test invalid inputs, nulls, >2^53
values, mutation/copies, disposal, Python detachment and WASM memory growth. Validate
stubs/declarations against runtime behavior. No wheels, notebook/browser viewer or new
distribution product is implied.

### AP-08 — Migrate consumers and make the primary API the documented default

Prerequisite: AP-07. Owns gallery/recipes, public docs, export visibility, changelog and
compatibility shims; AUT-01/09, QLT-05. Move every production-facing example/recipe,
native/Kit integration, publication and binding tutorial to the primary route. Keep
raw compiler tests where they discriminate internal contracts. Lead README/Rustdoc
with the first complete chart; organize advanced component docs by task. Audit public
re-exports and stop promoting compiler/state/DTO internals as the beginner entry point.

Preserve old public paths through forwarding adapters in the migration release. Do not
change wire field names because Rust types moved. Add deprecations and specific
replacement examples, with the next semver-compatible removal boundary selected at
AP-00. If that boundary is not the migration release, retain shims with explicit
ownership; G-AUTH permits shims, not missing capabilities behind them. No broad public
visibility reduction until external consumers and the baseline version policy allow it.

Acceptance FIX-AUTH08: compile old supported usage against the migration release,
compile/run new examples, run old payload→new runtime→preserved semantics checks,
validate docs/stubs/schemas and confirm no production tutorial needs raw DTO mutation.

### AP-09 — Certify complete primary-API coverage and refactor safety

Prerequisite: AP-08 and acceptance for every capability included in the target release.
Owns the coverage evidence register, integrated validation and G-AUTH; AUT-09, QLT,
PERF. Close every delivered-capability row, including non-default arguments and custom
protocols, through the primary API. Use existing independent fixtures/reference gates;
equal output from two paths sharing the same defect is not adequate numerical proof.
Compare baseline and primary paths at identical configuration/data/resources, normalized
for deliberately allocated opaque identities while checking identity relationships.

Run existing supported-platform checks and actual Rust/Python/WASM runtime proofs,
inspect required native/export artifacts and repeat relevant PERF-01–05 workloads
through the new entry points, including the sustained-load case. Record cold authoring
cost, copies, update latency, hover work, cached reuse, memory plateau and export lag.
No extra per-frame normalization/JSON encode-decode/full source rebuild; no silent
fallback to a lower-quality result. Retain existing performance targets; unexplained
regressions remain open. Update the release candidate and migration evidence, without
re-running the original WP program as new feature work or auto-publishing anything.

Acceptance FIX-AUTH09 / **G-AUTH**: all target-release capabilities accessible through
the primary API, independent semantics/lifetimes preserved, all external examples and
host routes executed, intended diagnostics/compatibility proven, required artifacts
inspected and performance gates met. Missing or uncertain rows keep the gate open.

## 7. Sequence, handoffs and release rules

The default order is AP-00 → AP-01 → AP-02 → AP-03 → AP-04 → AP-05 → AP-06 → AP-07 →
AP-08 → AP-09. This deliberately proves a real chart early, completes static coverage,
then migrates runtime/hosts and consumers. Packages may contain several small PRs;
partial rows stay open. No team/delegation or delivery-duration assumption is required.

Each semantic parity package owns its algorithm and its primary component options.
AP-03/04 own shared builder integration, AP-07 shared binding syntax, and AP-09 coverage
certification. GG-16 retains extension protocols/recipe dispatch semantics, but no
longer owns an alternative late authoring façade. Do not make a semantic kernel depend
on G-AUTH or make G-PARITY and G-AUTH depend on each other. Capability acceptance feeds
both gates; the refactored release requires both when parity is in its scope.

Implementation begins with AP-00 under the owner's completed-baseline assumption.
Confirm the supplied interface/revision inventory once; if an assumed capability is
absent, record that discrepancy and continue independent authoring work without
silently rebuilding or closing the original package. Shipping a refactor requalifies
its changed paths even when the baseline previously passed WP-21–23.

For each handoff record package/requirement IDs, owned files, baseline/result revision,
capability rows delivered, old/new usage, exact evidence and limitations. API semantics
and serialized operations have separate version histories. Staged additive migration
is the default; the sole entrypoint status is achieved by complete feature reachability
and consumer migration, not by deleting all expert or compatibility interfaces.
