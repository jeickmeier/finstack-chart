# Primary authoring API — refactoring and build plan

Date: 7 September 2026; implementation updated 8 September 2026. Status: final qualification; package acceptance is tracked in the ledger.
Authority: specification AUT-01–09 and [ADR-013](../adr/013-primary-authoring-api.md).
Source review: [public API usability](../evidence/public-api-review-2026-09-07.md).
Revised against the [current-library review](../evidence/primary-authoring-plan-review-2026-09-07.md),
addressing APR-01–04 in the contracts and acceptance below.
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
refactor of that completed baseline. The original draft used `127fe2d4853f89b62ba59a17248485e3c378ba60`
plus live changes. The current-library review checked completed original-scope code at
`c773fcba9a8c846322d08c9c1203fce415e91328`; the current revision
`b631f0e6d7e41722b5774433d616f704234157d7` adds unrelated skills only. Original-scope
completion is now recorded in the ledger, independently of this plan. AP-00 pins the
actual implementation baseline without reopening or recertifying the original program.

The additional D3 and GG packages retain their own scope and evidence. They are not
silently declared complete by the original-WP assumption. Every delivered capability
must be reachable through the primary API; any capability delivered later must add its
primary API integration in the same package. This plan does not reimplement their
algorithms or duplicate their acceptance inventories. A release claiming those parity
capabilities still needs their existing gates as well as G-AUTH.

The owner subsequently authorized implementation of the complete AP-00–09 plan.
The ledger records the active work and qualified boundaries; no publication is authorized
by this plan.

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
3. **A live chart reuses the existing runtime.** A `Chart` coordinates the existing
   reducer, compiler and source contracts. Ordinary construction owns ingestion/store
   state; an explicit external-source route consumes committed immutable snapshots
   from one external store authority. Never copy that store into a second writable
   authority. Distinct Charts retain independent view/selection state unless explicitly
   linked. Preserve worker-owned compiler caches and host scheduling lifetimes as
   mapped below; one engine does not require one mutable compiler instance. Hosts
   retain charts and own scheduling/event-loop objects.
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

### 2.1 Runtime ownership and validation boundaries

The implementation must preserve this map (APR-02/04), using existing components rather
than introducing a second runtime or a wrapper for each row:

| Existing owner | Primary-API responsibility and retained lifetime |
| --- | --- |
| `portable::Session` store/queue | Default Chart-owned ingestion uses the existing DataStore/IngestionQueue with one commit authority. JSON decoding/encoding forwards to typed operations. |
| Native external snapshots (`ChartInput`, `set_data`, `queue_data`) | Offer an explicit primary external-source route. The application/source owner commits and publishes coherent snapshots; Chart validates/reconciles them. Mutations go to that owner, never to an implicit Chart-local copy. Sharing a source does not share selection/view state. |
| `ActionReducer`, inspector pins and prepared/presented state | Retain per-view state and the actual acknowledged scene. Ingestion acceptance, preparation admission and paint acknowledgement stay separate observable transitions. |
| Native `Scheduling` and `PreparationScheduler` | Core owns synchronous admission policy; GPUI owns task/window/frame callbacks. Preserve one active worker and one newest pending preparation, with compiler/cache ownership moved into the worker and returned on completion. UI and worker compilers may remain separate as today; no shared mutable compiler or UI wait for its lock. |
| `FigureRequest` | Export owns cheap immutable acquisition of definition/source/state/profile/fonts/registry, before preparation/shaping/encoding. Chart supplies coherent core inputs; GPUI handles never enter a request. |
| `ExportQueue` / `ExportJob` | Export retains bounded job/input accounting and cancellation/drop semantics; the caller owns execution. Jobs may outlive their view while retaining only captured inputs. |

Reuse/factor current validator code into structural/schema/registry checks, statistical
execution and destination/serialization capability checks. `.build()` may materialize
data and resolve names/catalogs but must not run statistics, geometry generation,
layout or encoding. A built Plot retains the immutable registry needed by its operations;
Chart and capture retain that exact registry. Implementations themselves are not serialized.

The current `Session::with_extensions` both validates portability and prepares eagerly;
it cannot become the common builder constructor unchanged. Native construction accepts
registered native-only operations. Wire import/export and portable destinations apply
their explicit portable checks; export additionally applies renderer capabilities.
Unknown IDs/version mismatches still fail on every relevant route. Do not make native
validation weaker or force valid native-only operations through portable validation.

## 3. Intended developer workflows

These original sketches describe acceptance targets. The executable
[authoring guide](../authoring-guide.md) and examples own delivered API spellings;
unavailable future semantic options remain gated by their owning package.

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
    .title(title("Prices"))
    .subtitle(subtitle("Daily observations"))
    .x_axis(x_axis().label("Time"))
    .y_axis(y_axis().label("USD"))
    .theme(theme().preset(NamedTheme::Editorial))
    .build()?;

// Explicit font resources were configured once on this export destination.
let svg = output.request(&plot, export_options(Size::mm(180.0, 120.0)))?.prepare()?.export(Format::Svg)?.bytes;
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

### 3.1 Labels and figure components

Per owner direction, **labels are only annotations positioned by x/y**. Titles,
subtitles, x/y axis labels and legends have separate builders. A general-purpose
`labels()` bag must not contain title/subtitle/axis/legend settings. This supersedes
the combined labels sketch in the earlier API review; the examples here own the
proposed authoring vocabulary.

| Component | Proposed builder route | Responsibility |
| --- | --- | --- |
| X/y annotation | `.layer(labels().at(2.0, 12.0).text("Peak"))` | One annotation at an explicit x/y anchor, with text, offset, alignment and style. Data-driven labels use the same layer's aesthetic mappings, including text, and retain source/generated-stage rules. |
| Plot title | `.title(title("Prices"))` | Plot title text and its typography/layout settings. |
| Plot subtitle | `.subtitle(subtitle("Daily observations"))` | Subtitle text and its typography/layout settings. |
| X-axis label | `.x_axis(x_axis().label("Time"))` | Horizontal axis title and its own text/guide settings. |
| Y-axis label | `.y_axis(y_axis().label("USD"))` | Vertical axis title and its own text/guide settings. |
| Legend | `.legend(legend().scale("series").title("Series"))` | Select an authored scale/guide and configure its title, keys, labels, placement, ordering and visibility. The selected scale must exist; the builder does not create an aesthetic mapping. |

The axis routes edit the existing primary axis components; named/secondary axes use
the same underlying builder and scale identity. Axis tick labels/formatters remain
axis controls. Legend key labels remain legend controls. None are x/y annotation
layers. Automatic compatible legends remain available without an explicit builder;
the legend builder supplies overrides through the same guide engine.

Share RichText, typography and destination measurement internally rather than
duplicating text/rendering engines for these builders. Fixed annotations retain
annotation/derived identity and are not repeated once per source row or falsely
reported as observations. Mapped labels follow the selected data/stat provenance.
Captions, source notes and accessibility descriptions keep their figure/accessibility
components; they do not expand `labels()` into a general chart-metadata container.

### 3.2 Live charts and capture

For live use, create/retain a `Chart` from the built plot, obtain a dataset handle by
its authored name, submit updates and inspect typed receipts. Atomic multi-dataset
updates use the existing transaction boundary. Programmatic navigation, selection,
annotation editing, inspection, linking and export snapshots operate on that retained
chart. A native adapter mounts the chart once; it does not rebuild it during render.
Ordinary definition edits preserve the runtime's current data even when the edited
Plot was built from an older snapshot; data replacement is explicit and revision-fenced.
Exporting a live chart defaults to **Presented** basis and can explicitly select
**Current** committed definition/data/state. Neither selects visible versus full-domain
projection: that is an independent option. Presented rejects if no frame has been
acknowledged. Current can capture before first presentation. Exporting a static Plot
uses its own data and initial state, with no invented presented-scene stamp.

### 3.3 Explicit builders for the implemented baseline

This inventory maps implemented components at the reviewed revision to proposed primary
builders. It supplements section 5 and seeds AP-00's option-level register; it does not
claim the new builders exist. Use the linked semantic owners. Distinct compositional
concepts need constructors; ordinary options remain methods or typed values on their owner.

| Implemented component and source | Proposed route and controls to preserve | Owner |
| --- | --- | --- |
| Figure text and panel letters — [composition](../../crates/chart-core/src/composition.rs) | Separate `caption()`, `source_note()`, `footnote()` and `panel_letter()` components, alongside title/subtitle builders. Preserve ordered notes and explicit panel identity. | AP-04 |
| Direct annotations and callouts — [Annotation / Anchor](../../crates/chart-core/src/composition.rs) | `labels()` for x/y text; `callout()` for text plus a leader endpoint, sharing the annotation implementation. Expose data/panel/figure/output spaces, exact timestamp/category anchors, offsets, priority, collision and clipping. Connector controls retain existing threshold/range annotations without implying new shaded-range geometry. | AP-04 |
| Insets — [Inset](../../crates/chart-core/src/composition.rs) | `inset()` selects existing layer handles, parent panel, fractional rectangle, independent x/y viewports and guide visibility. Reuse prepared statistics; an inset is not a new data population. | AP-04 |
| Marks and recipes — [Geom / Layer](../../crates/chart-core/src/grammar/definition.rs) | `points()`, `line()`, `area()`, `ribbon()`, `bars()`, `ohlc()`, `rule()`, `rectangle()`, plus `histogram()`, `volume()` and `cells()` recipes over those components. Preserve baseline, width, order, gaps, clipping and provenance. | AP-02/03 |
| Statistics and positions — [statistical types](../../crates/chart-core/src/grammar/statistical_types.rs), [parameters](../../crates/chart-core/src/grammar/definition.rs) | `bin()` with explicit/automatic bin controls, `count()`, `summary()` and OLS `fit()` stat components; `stack()`, `dodge()` and `jitter()` position components. Preserve grouping/scope, generated-stage mappings, quantiles, normalization, ordering, seeded jitter and units. | AP-03 |
| Filters and shared transforms — [SourceFilter / TransformDefinition](../../crates/chart-core/src/grammar/definition.rs) | `filter()` and named `transform()` components with field predicates, affine transforms and stat-space/scope controls. Layer data/transform references use names or handles; filtering, transformed statistics and viewport zoom remain distinct. | AP-03 |
| Positional scales and axes — [AxisScale / AxisSpec](../../crates/chart-core/src/layout/types.rs), [scales](../../crates/chart-core/src/scales/mod.rs) | Typed linear/log/symlog/band/point/UTC/session scale builders attached to named x/y axes. Axis builders expose side, title, ticks, numeric format, rotation, visibility, domain/range/viewport/outside policies and affine secondary-unit guides. Secondary guides reference a primary scale and cannot bind layer coordinates. Session scales require a supplied calendar. | AP-03/04 |
| Color and legends — [ColorEncoding](../../crates/chart-core/src/grammar/colors.rs), [ColorScale](../../crates/chart-core/src/scales/color.rs) | Discrete/continuous color scale builders with palette, domain, null/outside policies and shared identity; the separate `legend()` builder configures guides. Preserve stage-aware source/generated/group mappings and automatic guides. Broader guide/key-glyph parity retains its semantic-package prerequisite. | AP-03 |
| Facets — [FacetSpec / FacetTarget](../../crates/chart-core/src/grammar/facets.rs) | `facet_wrap()` and `facet_grid()` with catalog/order, empty-panel, free/shared x/y, gap and collected-guide controls. Layers expose match/broadcast/selected-panel targeting and stat scope. | AP-04 |
| Theme and rich typography — [theme](../../crates/chart-core/src/theme.rs), [typography](../../crates/chart-core/src/typography.rs) | `theme()` with Editorial/Terminal/Grayscale presets and plot/layer overrides; reusable `text_style()` and rich-text/run builders. Gradients, symbols, dashes, padding, formatting, rotation and language/direction are typed options using supplied destination fonts/measurement. | AP-04 |
| Cartesian projection and layout — [coordinates](../../crates/chart-core/src/layout/coordinates.rs), [LayoutRequest](../../crates/chart-core/src/layout/types.rs) | `coord_cartesian()` binds the same named scales and clipping; `layout()` exposes padding, minimum plot size and layout budgets. Existing nonlinear axes do not establish polar/geographic or arbitrary path-subdivision support. | AP-04 |
| Linking and annotation editing — [linking](../../crates/chart-core/src/linking.rs), [editing](../../crates/chart-core/src/editing.rs) | `link()` and `annotation_edit()` configuration over Chart handles: axis matching/selection policy, echo prevention, constraints/snapping/order. Apply, preview, cancel and undo remain typed runtime operations. | AP-05 |
| Ingestion, scheduling and dense display — [ingestion](../../crates/chart-core/src/ingestion.rs), [retention](../../crates/chart-core/src/data/retention.rs), [scheduler](../../crates/chart-core/src/scheduling.rs), [dense](../../crates/chart-core/src/dense.rs) | Chart-owned `stream_options()` and `render_options()` expose existing queue/overload, retention/late-data, preparation and dense line/OHLC policies. Transactions, follow/freeze and receipts remain runtime methods. Dense display must not be presented as a statistical density estimator. | AP-05 |
| Native interaction and controls — [input](../../crates/gpui-charts/src/view/input.rs), [host](../../crates/gpui-charts/src/view/host.rs), [view](../../crates/gpui-charts/src/view.rs) | Native adapter configuration exposes `interaction()` policies and typed tooltip/toolbar/menu/accessibility-summary hooks. Reuse drag/selection/edit controllers and host command capability checks. Native factories stay in the adapter, outside portable grammar. | AP-05/06 |
| Publication and resources — [PublicationProfile](../../crates/chart-export/src/profile.rs), [export API](../../crates/chart-export/src/lib.rs) | Destination resource configuration and `export_options()` cover size, DPI, background, text mode, precision/output budgets and section 3.2's independent capture choices. Font resources are reusable destination inputs; export/capture/cancel/save are operations. | AP-06 |
| Registered components — [stat extensions](../../crates/chart-core/src/grammar/extensions.rs), [geometry extensions](../../crates/chart-core/src/grammar/geometry_extensions.rs) | Typed registered stat/geom components enter the same slots as built-ins. Retain operation version, parameter schema, resource and destination capability checks without requiring raw operation DTO construction. | AP-03/04 |

Required but undelivered parity features retain their semantic-package prerequisites:
data-driven text, broader guide/geom/stat families, mathematical typesetting,
polar/geographic coordinates and additional standalone families. Section 3.1's mapped
labels are a target for that work; existing fixed annotations can be exposed immediately.
An additional theme preset requires an actual theme definition; the introductory
example now uses the existing Editorial preset.

### 3.4 Rule for every future feature

Every future capability package must extend an existing typed builder when the feature
configures that concept, or add a focused component builder for a new compositional
concept. New statistics, geometries, scales and supported extensions use their ordinary
typed slots and applicable shared protocol. Do not create a parallel authoring API or
require raw DTO/JSON edits as the user-facing integration.

The same package must update the capability register and primary usage documentation,
provide a compiling consumer example and behavioral evidence, and update applicable
Python/WASM bindings, declarations and serialization contracts. Preserve defaults and
identities or document/version an intentional change. Host-specific capabilities use
destination builders and report unsupported destinations explicitly. Standalone
utilities retain direct access and integrate with chart components wherever required
by their capability contract.

Builder coverage is part of feature completion, not a later cleanup task. Keep runtime
commands, queries and receipts on the retained Chart/destination API rather than
inventing a builder per action or result. Existing AP packages own this integration;
no separate builder backlog or new package sequence is introduced.

### 3.5 Grouping and independent visual mappings

The primary `aes()` builder must expose independent `group`, `color`, `fill`, `shape`,
`linetype`, `size`, `linewidth` and `alpha` mappings, subject to the layer's supported
channels and source/generated-stage contracts (AUT-03, GG2-02/03). Grouping and visual
encoding may use the same field or different fields. Plot mappings are inherited by
layers; a layer may override an individual mapping without discarding the others.

```rust,ignore
// Proposed syntax; requires the mapped-aesthetic semantic packages below.
let plot = plot(data)
    .aes(aes().x("time").y("value")
        .group("series")
        .color("series"))
    .layer(points().aes(aes().shape("series")))
    .layer(line().aes(aes().linetype("series")))
    .build()?;
```

Here each series has a distinct connected line group and can receive its own color,
point symbol and line pattern. `group` controls observation membership for line
connection and applicable statistics, respecting explicit stat-scope overrides. It
does not itself assign visual styles. `color`/`fill`, `shape` and `linetype` map values
to the appropriate scale; scale builders select the actual palette, symbols and dash
patterns. Legends present those mappings through the separate legend builders.

Independent fields are equally valid: for data where each series belongs to a desk,
`.group("series").color("desk")` retains separate series lines while sharing colors
within each desk. A point layer can map `shape("instrument_type")`, and a line layer
can map `linetype("scenario")`. Explicit grouping takes precedence over profile-driven
inference; default/profile rules must not silently change when visual mappings are
added. Constants remain on layer/style builders, with documented mapped/constant
precedence. Do not require one manually authored layer per category to obtain mapped
symbols, line patterns or colors.

Current baseline grouping and color encoding can be routed through the primary API.
Mapped shape/linetype and the other independent channels require GG-03; inferred
grouping requires GG-02; scale policies and complete multi-aesthetic legends retain
GG-04/05 ownership. AP-03 integrates these into the primary builders when delivered,
using the existing package prerequisites rather than duplicating their implementations.
This example is an acceptance target, not a claim of current executable support.

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
- Applying an authored edit is **definition-only** by default (APR-01). Check the
  expected definition revision, resolve/validate against the runtime's current schema
  and source, retain current data/keys/store epoch/replay/ingestion queue, and reconcile
  view state using existing rules. An embedded older Plot snapshot must never replace
  current data. Incompatible fields or stale definition edits fail without publication.
  Data replacement uses an explicit expected-data/schema-revision transaction. Do not
  add a combined definition/data replacement convenience in the initial refactor;
  callers sequence the existing distinct operations and handle their typed outcomes.
- Enrich diagnostics with authored dataset/field/layer/aesthetic names while retaining
  exact IDs, stage, stable codes and corrections. Binding errors expose properties,
  not just JSON hidden in a generic exception string.

## 5. Complete capability routing

AP-00 creates a capability register referencing the original and parity inventories;
the table below defines its required families, not a replacement enumeration. Each
delivered option/default/extension boundary needs a route and executable example ID.

| Capability / authoritative requirements | Primary public route | Refactor owner |
| --- | --- | --- |
| All data kinds, typed accessors, named datasets, exact metadata; DAT-01–06 | Data builders, field/data handles, owned ingestion or external committed snapshots, typed receipts | AP-02/05 |
| Layer data/mapping inheritance, expressions/stages, shared transforms, scope/filter/zoom distinctions; GRA-01–08, GG2-02 | Plot/layer/aesthetic/stat/position/transform builders | AP-03 |
| Every geom/recipe, statistics/model family and position; GRA-04–06, SHP, GG2-05/06 | Typed layer components using existing kernels/registries | AP-03 |
| Every scale, named/multiple/secondary axes, color/fill/alpha/shape/size/linewidth/linetype and guides; SCL, COL, CHR, ITP, AXIS, GG2-03/04 | Aesthetic and scale/guide builders; same standalone family values | AP-03/04 |
| Facets, coordinates/geography, layouts/hierarchy, panel identities; GRA-07, HIR, GG2-07/08 | Facet/coordinate/layout components and explicit resources | AP-04 |
| Themes, rich/math text, x/y annotations, titles/subtitles, axis labels, alt text, publication furniture/insets and linked panes; THM, LAY, GG2-09 | Separate annotation/title/subtitle/axis/figure/accessibility builders sharing text/layout services; legends use guide builders | AP-04 |
| Queries, focus/accessibility, hover, pan/zoom, selection, controlled state, remapping and capture; INT, GPU | Retained Chart actions/events/inspection plus native hooks | AP-05/06 |
| Linked plots/table, echo prevention and constrained editable annotations; INT, LAY-03 | Existing link/edit controllers consuming stable chart handles | AP-05 |
| Atomic append/upsert/remove/replace, retention, backpressure, correction, follow/freeze, incremental/batch execution; DAT, STM | Chart data handles, transactions, retention and typed outcomes | AP-05 |
| Scheduling, cache invalidation, dense representation, resource/quality limits; STM, SCN, PERF | Existing runtime policies exposed through typed Chart options | AP-01/05/09 |
| Static and live coherent SVG/PDF/PNG and additional supported devices, preview, text/font/DPI/physical size; EXP, GG2-11 | Destination context; Presented/Current basis independently of visible/full-domain and interaction policy; FigureRequest/ExportQueue | AP-06 |
| Native GPUI and optional Kit hooks, tooltips/controls and platform capability reporting; GPU, SCP-03 | Thin retained native view over Chart; Kit theme mapping is a gallery recipe | AP-06 |
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
Use section 3.1's separate component builders in place of the review's historical
combined labels sketch; `labels()` is reserved for x/y annotations.
Expand section 3.3 into option-level rows and adopt section 3.4's same-package integration
rule for all future capability work.
Adopt section 2.1's source/view/worker/export ownership map and the definition-only
edit contract before extracting runtime code. Include APR-01–04 and their source
boundaries in the old→new map; retain the reviewed current-library snapshot as evidence.

Acceptance FIX-AUTH00: no unowned existing capability, no unknown lifecycle owner,
explicit old→new API map, and an acyclic package/capability integration schedule.
Every section 3.3 component and meaningful option has a builder/configuration/operation
owner, with implemented baseline coverage distinguished from future semantic work.
The completion assumption is an input; no old gate is marked passed by this task.

### AP-01 — Refactor one typed runtime behind all entry points

Prerequisite: AP-00. Owns core runtime/session, data store/compiler/reducer integration
and portable boundary; AUT-01/05, ARC, DAT, STM. Extract typed construction, update,
action, inspection and capture entry points from existing runtime implementations as
needed. Route the legacy JSON session through these calls. Retain existing cache,
retention, scheduler and presented-scene ownership semantics. Keep destination objects
outside core; do not require serialize→decode for Rust calls or introduce a second
execution engine. Adapt the native/export constructors incrementally through shims.
Implement the section 2.1 ownership map: default owned ingestion and external committed
source admission, per-view reducer/pins, core scheduling policy with host-owned worker
execution and compiler return, and export-owned request/job lifetimes. Factor structural
validation without statistics from eager preparation, and keep portable validation at
the portable boundary. Carry immutable extension registrations through typed construction;
do not retain the portable Session constructor's unconditional portability restriction.

Acceptance FIX-AUTH01: typed and legacy routes produce matching independent fixture
results, diagnostics and revision traces; failures preserve prior valid state. Existing
batch/update, stale completion and ownership fixtures pass before any API migration.
Also prove one external source can feed independent views without a copied writable
store; preparation failure cannot undo its already committed transaction. Test distinct
ingestion/preparation/paint receipts, worker cache return, stale completion/disposal,
and nonblocking UI ownership. Structural validation must not execute a counting custom
stat; native-only operations remain valid natively while portable routes reject them.

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
Verify `.build()` does not execute a custom statistic or generate/layout geometry;
native-only registry entries survive build/native preparation, serialization and
unsupported destinations reject correctly, and unknown/version-mismatched entries
fail with actionable diagnostics. Retained Plot/Chart/capture owners keep registry
implementations alive for their required lifetime.

### AP-03 — Expose the complete grammar and aesthetic controls

Prerequisite: AP-02; each added parity capability also requires its semantic owner.
Owns grammar/stat/position/scale/guide authoring and registries; AUT-03/08. Cover every
available source/generated/expression stage, aesthetic, mapping override, geom/stat
override, position, named transform, scale family and guide control. Supply defaults
and explicit independent-data/inheritance overrides. Custom operations enter through
ordinary typed components and the existing registry. Builtin recipes lower to the same
generic components. No duplicate stat/scale dispatch in the new builders.
Expose legend overrides through their separate builder; preserve automatic guides and
shared scale identity. Plot labels/title builders must not become another guide API.

Acceptance FIX-AUTH03: all delivered grammar register rows have compiling usage and
behavior assertions; wrong-stage compile failures, runtime schema errors, profile
defaults, grouping, scale/guide consistency and shared-transform invalidation pass.
Exercise at least one custom stat and geom in an external crate through the primary API.
For section 3.5, compile and execute grouped line/point examples using the same field
for color/shape/linetype and separate fields for grouping and appearance. Verify actual
resolved colors, symbols and dash patterns, distinct line membership, plot-to-layer
inheritance and individual overrides, explicit grouping versus profile inference,
mapped/constant precedence and matching legend keys. Use independent expected mappings
and inspect native/export output after the relevant GG-02–05 capabilities land; wire
the same cases into AP-07's actual Python/WASM proofs. Until then, keep the corresponding
coverage rows open rather than treating per-layer constant styles as mapped support.

### AP-04 — Expose complete design, composition and specialist capabilities

Prerequisite: AP-03 and applicable semantic owners. Owns facet/coordinate/theme/text/
figure builders and specialist utility integration; AUT-04/08. Add short default
facets with explicit catalog controls, all coordinate/layout settings, themes, rich
and mathematical text, x/y annotation labels, separate title/subtitle and x/y axis
label builders, furniture, insets and multi-plot composition. Follow section 3.1's
component separation and consume AP-03's separate legend builder; share existing
RichText/typography/layout services beneath all text-bearing components.
Expose required extension boundaries and direct D3 family helpers without exposing
internal pipeline structs as the only control mechanism. Reuse prepared-data sharing
for insets and related plots. Implement no new geometry/model algorithms in this slice.
Cover section 3.3's explicit caption/source-note/footnote/panel-letter, callout, inset,
theme/text-style and layout builders rather than leaving them behind raw figure DTOs.

Acceptance FIX-AUTH04: each available design/specialist register row is reachable;
empty/new facet categories, free/shared axes, explicit broadcasts, linked panes,
fonts/text bounds, coordinates and extension capability errors retain baseline meaning.
Inspect actual native/export design artifacts; standalone helpers need no font or host.
Compile examples combining an x/y annotation, title, subtitle, both axis labels and
an explicit legend override. Check that each builder affects only its intended
component: annotation x/y changes move the annotation, title changes do not alter
axis/legend text, and axis/legend changes do not create annotation marks. Fixed labels
produce one annotation with correct identity, mapped labels retain their provenance,
and automatic legends still work. Keep unsupported title/subtitle/axis/legend setters
off the labels builder and verify that boundary with compile-fail examples.
Include notes/panel letters, a callout and an inset in the artifact examples; verify
coordinate-space meaning after resize, collision policy and prepared-statistic reuse.

### AP-05 — Make all live features accessible through Chart

Prerequisite: AP-04. Owns primary Chart data/actions/options/linking/edit interfaces;
AUT-05, DAT, INT, STM. Expose dataset handles, transaction builders and receipts,
retention/backpressure policies, controlled/uncontrolled state, query/selection,
navigation, annotation edits/undo, linked charts/table, follow/freeze, scheduling and
dense representation settings. Centralize definition/data updates on the typed runtime
from AP-01. Apply immutable Plot edits as definition-only changes under the expected
definition revision, against the current source, preserving live data and ingestion
ownership. Data replacements remain explicit revision-fenced transactions. Validate
each candidate and publish atomically within its existing operation boundary. Preserve
both presented and current core inputs for AP-06's explicit capture basis.

Acceptance FIX-AUTH05: replay existing interactive and streaming fixtures exclusively
through the primary API, with update-versus-batch checks. Include conflict/no-op/queued
receipts, multi-dataset rollback, key eviction, link echoes, stale controlled replies,
gesture cancel, out-of-order workers, fair dashboard progress and resource release.
Add APR-01's regression: build Plot at r0, append through r1, edit title/theme/scale
using the original Plot, and prove r1 data/keys/store revision/replay receipts and pending
ingestion remain intact. Include incompatible schema, stale definition and explicit
data-replacement conflicts with unchanged prior state on failure.

### AP-06 — Complete native, Kit and export integration

Prerequisite: AP-05. Owns `gpui-charts`, optional gallery Kit recipes, `chart-export` and explicit
resource contexts; AUT-06. Complete adapters to section 2.1's typed runtime without
collapsing intentional source/view/worker/export lifetimes. Provide retained native
mounting, replaceable tooltip/menu/toolbar and
accessibility hooks. Provide concise bytes export and explicit host save helpers,
physical size/DPI/background/text/device controls and publication preview. Expose three
independent capture choices: Presented/Current basis, visible/full-domain projection,
and interaction inclusion. Presented is the new live default and retains the painted
source/profile/origin stamp; Current captures coherent committed definition/data/state
without inventing a presentation stamp. Preserve no-presentation rejection only for
Presented. Preserve the existing default clean committed InteractionCapture policy and
legacy direct FigureSnapshot::capture's earlier all-interaction behavior in its shim.
Acquire FigureRequest cheaply without statistics, layout, shaping or encoding; reuse
ExportQueue/ExportJob for bounded deferred work and explicit cancellation/drop. Static
and live export share this pipeline, preserving supplied font/registry identities and
the original/effective state manifests.

Acceptance FIX-AUTH06: actual native lifecycle/input/accessibility-hook checks and
inspected supported device artifacts; baseline logical/physical geometry and fonts;
slow export during updates/edits, queue bounds/cancellation and retained snapshot
lifetimes. GPUI is absent from headless dependency/runtime paths; Kit stays optional.
Add APR-03's matrix: r0 is displayed while r1 is committed/pending preparation; Presented
captures r0 with its origin stamp, Current captures r1 without one. Cross both bases
with visible/full-domain and interaction inclusion; test pre-first-paint behavior,
default versus legacy interaction policy, cheap acquisition and manifests. Run jobs
after later updates/disposal and verify captured resources release at existing boundaries.

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

For parity handoffs, reconcile the live Python/WASM registrations and package wrappers
before adding syntax. Each semantic owner extends those adapters and their public
exports/declarations, with actual primary chart and applicable standalone runtime
proofs in the same package. Rust `plot::host`/export dispatch coverage is infrastructure
evidence only. Keep legacy-envelope compatibility tests alongside host-native usage;
AP-07 remains responsible for shared conversions, exact values and diagnostics.

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
Contributor guidance and the capability register enforce section 3.4 for future work:
primary builder integration, applicable bindings and evidence belong in the feature's
own package before it can be marked complete.

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

The [Phase 2 API reconciliation](phase-2-parity-implementation-plan.md#41-integration-through-the-current-primary-api)
routes new work through the delivered Data/Plot/Chart/Output owners and AP-00 register.
GG-01 reconciles the existing shared legend painter's remaining acceptance; GG-02
owns alternate-profile policy propagation through execution/wire/capture; WP-AX01
owns the scale/guide split across primary handles, names, bindings and navigation.
AP owners integrate those contracts without recreating their semantic kernels or
charging for already delivered authoring foundations. These are interface handoffs,
not new dependencies between cumulative gates.

AP-00 establishes the inventory under the owner's completed-baseline assumption;
resume the next assigned incomplete slice from its register and ledger evidence.
Confirm the supplied interface/revision inventory once; if an assumed capability is
absent, record that discrepancy and continue independent authoring work without
silently rebuilding or closing the original package. Shipping a refactor requalifies
its changed paths even when the baseline previously passed WP-21–23.

For each handoff record package/requirement IDs, owned files, baseline/result revision,
capability rows delivered, old/new usage, exact evidence and limitations. API semantics
and serialized operations have separate version histories. Staged additive migration
is the default; the sole entrypoint status is achieved by complete feature reachability
and consumer migration, not by deleting all expert or compatibility interfaces.
