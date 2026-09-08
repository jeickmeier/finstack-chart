# Rust-native GPUI Charts — Project Specification

Document version: 0.5.0  
Date: 7 September 2026  
Status: Project contract with required Phase 2 D3 and ggplot2 capability parity and a primary authoring API; implementation evidence is recorded separately in the status ledger.  
Companion documents: [Implementation plan](../impl_plans/gpui-charts-implementation-plan.md) and [Migration and architecture rationale](gpui-charts-migration-plan.md).

## 1. Authority, purpose, and vocabulary

This document defines what to build and what observable behavior must hold. The implementation plan defines dependency order, tasks, evidence, and release gates. The migration plan records architectural reasoning, reference-library research, and alternatives.

Apply these documents in this order: explicit project-owner instructions; this specification; implementation decisions recorded in ADRs within this specification's permitted choices; implementation plan; migration rationale. An ADR cannot silently weaken a MUST, change a documented statistical meaning, or reclassify a required feature as optional. Report a conflict and continue independent work while the product decision is resolved. Routine implementation choices within this contract do not require confirmation.

MUST means required for the scoped production release unless a milestone or future boundary is stated. SHOULD permits a documented alternative with equivalent outcomes. MAY indicates optional scope. Requirement IDs remain stable as wording evolves; retired IDs are not reused. API/type names are proposed names, with contract semantics authoritative over spelling.

| Term | Meaning |
| --- | --- |
| Definition | Authored data bindings, grammar, scale/theme/layout settings, and interaction configuration. |
| Dataset snapshot | Immutable, revisioned source data with stable identities. |
| Prepared data | Validated channels, statistical outputs, grouping, and position results. |
| Scene | Resolved geometry, text/resources, clipping, and semantic target metadata at specified bounds. |
| Presented scene | The scene actually visible and used for input at a given moment. |
| Figure snapshot | Immutable definition/data/state/resources used for reproducible export. |
| Source target | An original row identified by dataset and row key. |
| Aggregate target | A bin/group result with membership and computation provenance. |
| Derived target | A model, annotation, or computed value that is not one original row. |
| Host | GPUI desktop, headless exporter, Python adapter, or future browser adapter. |
| Production scope | The feature set below; not the union of every reference library's capabilities. |

## 2. Product scope and supported surfaces

**SCP-01 — Product.** Deliver a reusable native Rust grammar-of-graphics library with convenient chart recipes and composable layers. Both authoring routes MUST use the same preparation, scales, layout, geometry, state, and error contracts. GPUI is the first interactive host; its optional Kit integration MUST NOT be required to use the standalone GPUI component or headless core.

**SCP-02 — Required scope.** The first production release MUST include:

| Area | Required coverage |
| --- | --- |
| Geometry and recipes | Points/scatter, lines, interval bars, grouped/mixed-sign stacked bars, area/ribbon, rules, text/annotations, cells/heatmaps, histogram, candlestick/volume composition, and the complete D3 shape surface defined by SHP-01–10. |
| Statistics and positions | Identity, count, explicit binning, grouped summaries including quantiles, simple ordinary least-squares fit, stack/normalize/dodge, deterministic jitter, pie layout and explicit D3 stack order/offset policies. |
| Coordinates and composition | Cartesian coordinates and radial shape projection; multiple named scales; facet wrap and a basic row/column facet grid; shared/free axis policies; aligned panes and linked plots. |
| Scales | D3 scale feature parity: linear, power/sqrt, identity, radial, log, symlog, UTC and explicit local calendar time, ordinal, band, point, sequential and diverging variants, quantile, quantize and threshold; supplied-session time remains additional required coverage. See SCL-06–08. |
| Color schemes | Complete d3-scale-chromatic 3.1.0 catalog and interpolation behavior, standalone and integrated; CHR-01–06. |
| Colors | D3 color capability parity: parsing, RGB/HSL/Lab/HCL/LCh/Cubehelix values, conversion, manipulation, formatting and portable authoring/runtime proofs; COL-01–06. |
| Interpolation | Complete d3-interpolate capability parity through typed Rust/portable values, colors, composition, 2D transforms and smooth zoom; ITP-01–08. |
| Axes | D3 axis capability parity through a portable compatibility profile: independent guides, tick selection/formatting, geometry, component styling and transitions; AXIS-01–07. |
| Hierarchy | Complete D3 hierarchy capability parity: construction/stratification, node operations, tree, cluster, partition, pack and treemap layouts, packing helpers and all tilers; HIR-01–08. |
| Paths | Complete d3-path construction and standalone SVG path-data output, shared with shape generators and portable numeric scenes; PTH-01–06. |
| Design and export | Complete custom theme cascade, publication figure layout, SVG/PDF/PNG output, explicit font handling, and publication preview. |
| Live use | Atomic batched data changes, bounded retention and queues, streaming corrections, stable interaction state, and coherent export during ingestion. |
| Interaction | Inspection, navigation, selection, linked views, annotation editing, programmatic actions, keyboard behavior, and documented accessibility support. |
| Extensibility | Native custom statistics and geometry, explicit scale/coordinate capabilities, portable extension identifiers, and renderer capability reporting. |
| Portability proof | Minimal Python and WASM fixtures before API stabilization; core and headless use independent of GPUI. |

Candlestick recipes render supplied OHLC data; pricing, risk models, market feeds, portfolio analytics, and trading execution are application responsibilities. Recipes are examples of the general grammar.

**SCP-03 — Release boundaries.** Default first-class desktop target is macOS on Apple Silicon, subject to the selected GPUI release's measured capability report. Headless core tests MUST run on macOS and Linux, and portable core builds MUST run for the browser WASM target. Other GPUI OS/architecture combinations require their own capability and test evidence before being advertised. This is an initial support assumption that can be expanded without redesigning core.

Full Python packaging/distribution, a Python native viewer, interactive notebook widgets, a browser renderer/product, touch-specific browser interactions, WebGPU, general radar products, general network/force/Sankey layout algorithms, complex morphing, and press-specific PDF standards are future scope. Section 2.1 promotes ggplot2 geographic, polar/radial coordinate, advanced fitting/density, mathematical text and device capabilities into required Phase 2 scope. D3 radial shapes, arcs/pies and link generators remain required by SHP-01–10; hierarchy operations/layouts and their Cartesian/radial recipes remain required by HIR-01–08. Minimal binding proofs are required now. Required current features cannot be postponed merely by calling them extensions.

### 2.1 Phase 2 parity scope

Version 0.4.0 adopts the owner's Phase 2 parity expansion, sequenced in the
[secondary implementation plan](../impl_plans/phase-2-parity-implementation-plan.md).
Phase 1 retains WP-01–20 and its historical G0–G3 evidence. Phase 2 delivers the eight
existing required D3 module inventories plus ggplot2 4.0.3 chart capability parity,
then consumes the existing WP-21–23 final certification sequence. All remain required
before G4; this phase naming does not defer existing D3 requirements beyond production.

| Requirement | Required Phase 2 behavior |
| --- | --- |
| GG2-01 — Coverage and compatibility | Provide an explicit versioned ggplot2 capability profile with complete release export/argument/default/generated-field coverage and pinned reference provenance. Typed authoring and registered operations may adapt R syntax/objects, but cannot omit defined chart capabilities. Preserve existing definitions/defaults and report profile/resource identity. |
| GG2-02 — Grammar and stages | Support mapping inheritance, inferred discrete grouping with explicit override, orientation, stat/geom overrides and bounded source/after-stat/after-scale/theme-derived expressions. Under the compatibility profile, reproduce reference scale-transform/limit/OOB/stat/position ordering; distinguish coordinate zoom from population filtering and avoid double transforms. |
| GG2-03 — Aesthetics and scales | Provide independent fill, stroke/color, alpha, shape, linetype, size, linewidth and text mappings; continuous/discrete/binned/manual/identity scales, area/radius policies, reference color catalogs/interpolation, limits/expansion/missing/break/label rules, date/time and secondary guides. Reuse D3 kernels where behavior agrees. |
| GG2-04 — Guides | Paint guides in single and faceted charts. Support continuous/stepped/binned and multi-aesthetic legends, key glyphs, overrides, collection, placement, and reference axis/custom/angular guide capabilities; resolved guide colors and transforms MUST match marks. |
| GG2-05 — Statistics and positions | Provide complete reference built-in count/bin/summary, distributional, fitting, 2D/contour and helper capabilities with weights, uncertainty, generated schemas and provenance. Include KDE/violin/boxplot/ECDF/QQ, model smoothing and quantile regression, 2D/hex statistics, ellipse/function/alignment; all reference position controls including dodge2, jitter-dodge and nudge. |
| GG2-06 — Geometries and recipes | Complete reference primitive, interval, polygon, raster, analytical and source/stat-driven text/label families and their documented controls. Each built-in combines actual statistics/defaults/guides/provenance with shared portable geometry; caller-precomputed paths alone do not establish support. |
| GG2-07 — Facets | Support multi-variable wrap/grid, margins, shrink, proportional free space, dimension-specific scale sharing, direction/order, strips, labellers and interior axes; preserve panel/group identities, population semantics and compatible guides. |
| GG2-08 — Coordinates and geography | Support fixed-aspect/flip/post-stat transformed and full polar/radial coordinates; geographic map/sf-equivalent inputs, projections, CRS controls, graticules and labels. Projection, bounded subdivision, clipping, guides and inspection MUST agree across hosts with explicit resources. |
| GG2-09 — Themes and mathematical text | Provide the reference theme element hierarchy, complete presets, component/geom controls, relative units/margins and explicit theme contexts; typed mathematical label capabilities across marks, axes, strips and annotations with supplied fonts, correct bounds and native/publication output. |
| GG2-10 — Extensibility and authoring | Supply equivalent typed/registered stat, geom, scale, coordinate, facet, guide, labeller, model and key-glyph protocols, demonstrated externally. Add typed recipe/data dispatch, composable authoring, built-layer inspection, labels/alt text and reference vector helpers through the common engine. No mandatory R runtime or second host compiler. |
| GG2-11 — Saving and destinations | Complete reference saving controls and device capabilities through explicit host/export adapters, with a per-device/platform matrix. SVG/PDF/PNG remain universal publication evidence on supported exporter hosts; other devices require actual encoding/decoding evidence on their declared reference-supported hosts. WASM retains its required scene/SVG route; unavailable devices must be reported. |
| GG2-12 — Acceptance and integration | FIX-GG00–19 MUST supply a complete reference/independent-value matrix, actual Rust/Python/WASM capability execution, inspected supported native/export artifacts, migrations, update/batch and resource/lifetime evidence. G-GGPLOT cannot pass with a required missing, failed or uncertain row. WP-21/22 retain platform/fidelity and measured performance certification. |

The secondary plan's capability boundary and GG-00–19 inventories elaborate these
requirements. Existing GRA/ADR defaults continue for legacy definitions; explicit
compatibility-profile choices are permitted alternatives for the named semantics above.
No R metaprogramming syntax, bundled example data, arbitrary third-party R runtime,
browser product or complete union of D3 modules is implied. Optional dependency
isolation does not turn an absent required capability into parity. Device-specific
host adaptations must be explicit; reference-language identity is not numerical proof.
Specification version 0.4.0 does not select a new portable envelope version.

## 3. Architecture and host boundaries

**ARC-01 — Workspace.** Start with these boundaries; splitting additional crates is an implementation choice requiring a demonstrated dependency or distribution benefit.

| Crate | Owns | Must not own |
| --- | --- | --- |
| `chart-core` | Definitions, data contracts, grammar, stats, scales, layout contracts, scenes, hit testing, state reducer, diagnostics. | GPUI/Python/JS object types, native windows, compulsory network/filesystem/system-font access. |
| `chart-export` | Publication resources/layout integration, figure snapshots, SVG/PDF/PNG encoding. | GPUI event-loop dependency or application file dialogs. |
| `gpui-charts` | GPUI element/entity integration, paint/text bridge, events, focus, host accessibility. | A second statistical/compiler engine. |
| `gpui-charts-kit` | Optional Kit theme/control adapters and integration examples. | Core numerical or geometry semantics. |
| `chart-gallery` | Executable examples, visual fixtures, stress scenarios. | Production dependencies of the library crates. |
| `chart-python`, `chart-wasm` | Initially small binding proofs; later independently distributable adapters. | Independent copies of chart/stat/interaction algorithms. |

**ARC-02 — Host services.** Core MUST accept explicit text metrics, resource descriptors, and required execution inputs. It MUST support synchronous execution without compulsory native threads. Scheduling, clocks, resource retrieval, interpreter attachment, and presentation belong to hosts. `std` is allowed; `no_std` is not a requirement. Font files and output files are host-supplied bytes/resources; no hidden downloads.

**ARC-03 — Ownership and extensions.** Use immutable snapshots and explicit revision handles. Keep typed native accessors/statistics at authoring/preparation boundaries; normalize or erase types before the common renderer. Common scene primitives SHOULD use closed enums; custom stat/geom/scale/coord APIs MUST have documented input/output, invalidation, capability, and error contracts. Require thread-safety bounds where work crosses threads, without forcing interpreter or window objects into core.

**ARC-04 — Dependency isolation.** Select and record an exact GPUI package/source/revision compatible with the host. Do not mix similarly named GPUI packages in public types. Export/font dependencies MUST pass the required capability fixtures before adoption. Arrow/Polars, async runtimes, finance libraries, Python, and browser packages MUST NOT become mandatory core dependencies. Pin fixture dependencies and record source/license provenance when adapting code.

## 4. Data, identity, and atomic change contracts

**DAT-01 — Data model.** Support typed native row snapshots and normalized column batches. Required portable field kinds are float64, signed/unsigned 64-bit integer, boolean, UTF-8, categorical dictionary, and timestamp with explicit integer unit and timezone metadata. Null validity is independent from values. Schema metadata can carry units, display labels, and original formatted values. Source amounts requiring decimal/exact display MUST remain recoverable; rendered floating-point coordinates do not replace source precision.

Typed closures may compute channels, but portable definitions use fields, literal values, supported declarative operations, or registered versioned operations. A serialization attempt that encounters an arbitrary closure MUST return an actionable unsupported-operation diagnostic.

**DAT-02 — Identity and ordering.** Dataset, layer, panel, scale, series/group, and row identities MUST be stable. Mutable/streaming datasets require caller-supplied unique row keys or a library-generated stable key handle returned at insertion. Array indices are not durable IDs. Replace operations preserve targets only for keys intentionally reused within the same dataset identity; a new identity resets that association.

Dataset insertion order is stable. Lines default to explicit x ordering within each group; equal x values break ties by stable insertion ordinal. A path-order option preserves authored order. Category order uses an explicit domain when supplied, otherwise first-seen order retained across updates until explicitly reset. Dictionary code reassignment MUST NOT change category identity.

**DAT-03 — Delta protocol.** Provide `AppendBatch`, `UpsertByKey`, `RemoveKeys`, `ReplaceSnapshot`, and retention-policy updates. Append rejects duplicate existing or within-batch keys. Upsert replaces the complete row for an existing key and inserts absent keys; partial field patches are future scope. Remove of absent keys is an idempotent no-op reported in the result. Multiple operations in a transaction execute in declared order.

A transaction contains an opaque transaction ID, source epoch, expected base revision for each touched dataset, schema versions, and ordered operations. Dataset revisions increase monotonically on effective committed change. A store-wide commit revision also increases once per effective transaction, providing an order for coherent multi-dataset snapshots. Empty/no-op changes return existing revisions. A transaction can change several datasets atomically; validation failure leaves all datasets and chart state unchanged.

**DAT-04 — Acknowledgement and replay.** Distinguish queued from committed. Return typed outcomes: applied with resulting revisions/counts; already applied; rejected with structured diagnostics; or conflict with observed revisions. A bounded deduplication cache recognizes a repeated transaction ID and identical payload within its documented horizon. Reusing that ID with a different payload is an error. After eviction from that cache, expected revisions prevent an old non-empty transaction being silently applied again; sources MUST resynchronize after an epoch/revision conflict. Expose the horizon and reset policy.

A bare transport sequence does not replace dataset revision checking. Source timestamps may arrive out of order; insertion, event time, and transaction sequence are separate concepts.

**DAT-05 — Validity and numeric precision.** Null and non-finite channel values MUST NOT reach the painter as invalid coordinates. Default line behavior splits at missing or invalid coordinates; connecting gaps is explicit. Invalid rows are excluded from the affected operation with reported counts; strict mode escalates to an error. Missing values MUST NOT silently become zero.

Use float64 in data-space mathematics and integer timestamps in source data. Subtract a suitable integer origin before floating-point time conversion, with checked arithmetic. Convert to GPUI pixel precision after local projection. Geometry, ticks, hit results, and export MUST use the same declared transformation. Extremely large domains that cannot preserve requested resolution produce a precision diagnostic.

**DAT-06 — Provenance and lifetime.** Targets MUST distinguish source, aggregate, and derived values. Aggregates carry group/bin identity, input revision, and a membership resolver or compact membership representation. A fitted curve identifies its model and input scope. Never report an aggregate as if it were one source row.

Target resolution takes the relevant immutable scene/data snapshot. Snapshots held for a gesture/export keep their resources valid; retention and disposal must release unreferenced snapshots. If a selected source is evicted, default behavior removes it from active selection and emits a removal event; a pinned tooltip can retain an explicitly labeled historical snapshot. The host may choose another documented policy.

## 5. Grammar and statistical semantics

**GRA-01 — Layer composition.** A layer is data + aesthetic mappings + statistic + geometry + position. Layers MAY have different schemas and row counts. Chart mappings are inherited only when compatible; an incompatible layer MUST provide overrides or fail validation. Layer order determines paint order unless an explicit order is supplied. Constant style and mapped aesthetics are separate APIs.

**GRA-02 — Compilation stages.** Implement this dependency order:

1. Validate definitions, resource/operation IDs, data schemas, and transform dependencies.
2. Apply source filters; resolve source mappings, facets, groups, and computation scope.
3. Prepare declared statistical input space; run the stat; bind typed generated outputs.
4. Apply semantic positions; collect domains from all relevant endpoints and intervals.
5. Resolve scale domains, ticks, guides, and bounded text-aware panel layout.
6. Resolve range mappings and display-space adjustments; project geometry and geometry-affecting styles.
7. Resolve paint/resources, clips, semantic targets, and matching hit indexes.
8. Present a compatible scene and apply input through the common state/action path.

Dependency cycles and unsupported feedback from layout into statistical scope MUST be diagnosed. A custom stat output MUST have its own schema; a generated count accessor cannot accidentally receive an original observation.

**GRA-03 — Statistical space and scope.** Default statistics operate in source data units over the filtered population within their declared facet/group. Scale transforms and viewport zoom MUST NOT silently change that population. `StatSpace::Transformed` names a transform and carries output-space metadata so it is not applied twice. Visible-range and screen-space statistics are explicit opt-ins and invalidate on viewport/layout changes.

Source filtering changes the statistical population. Highlighting does not. An independent layer can declare chart-wide, per-panel, or per-group scope; recipes must document defaults.

**GRA-04 — Required statistics.** Identity retains row provenance. Count excludes invalid required inputs and reports exclusions. Bins MUST support explicit edges; default intervals are left-closed/right-open, with the final right edge included. Values outside explicit edges follow an explicit exclude/overflow policy; default excludes and reports them. Automatic binning uses 30 equal-width bins over the eligible population by default, with a documented nonzero-width rule for constant input. Viewport zoom does not move bin boundaries unless requested.

Grouped summaries include count, min, max, arithmetic mean, sum, and configurable quantiles. Default quantiles use linear interpolation at `h = (n - 1) p` over sorted finite values, with endpoints at p=0 and p=1. Empty results are missing, not zero; an empty sum may be requested explicitly as zero. Use numerically stable accumulation. Ordinary least-squares fit has an intercept, finite paired x/y inputs, and fails with a diagnostic for fewer than two usable distinct x values or numerically singular input. Confidence bands are not implied by the fit recipe.

**GRA-05 — Positions.** Stack positive and negative values separately from zero in stable declared group order. Normalized stacks divide by separate positive and absolute-negative totals; positive heights sum to +1 and negative heights to -1 when those sides exist. Zero-only groups remain zero without division errors. Interval endpoints contribute to the scale domain. Reject additive stacks on incompatible nonlinear encodings rather than drawing misleading stacks.

These remain the existing chart stack defaults. SHP-07 adds explicitly selected D3 order/offset semantics, including a distinct expand policy; it MUST NOT reinterpret existing normalize definitions or weaken FIX-03.

Dodge uses declared band-relative widths/order; missing groups preserve configured slots. Jitter declares data or display units and uses an explicit seed plus stable row/group keys; reordering input cannot move existing points. Defaults must be documented in recipes.

**GRA-06 — Geometries and recipes.** Required geoms consume prepared encodings and emit portable primitives plus semantic targets. Bar/area baselines are explicit, zero by default where valid. Log axes require a valid declared baseline for baseline-dependent geometry. Ribbon bounds with lower > upper are invalid. OHLC validation requires low <= open/close <= high; volume invalidity is reported independently. Gaps split line/area runs by default.

Default chart line interpolation is straight segments. SHP-03 requires the full D3 curve family as explicit interpolation choices, documenting endpoint and overshoot behavior; source values remain available for inspection. SHP-02 adds general paired-boundary areas without weakening the existing ribbon bound validation. Candles have open/high/low/close semantic values and separate up/down styling. Histogram is a bin stat plus interval bars, rather than a separate chart engine.

**GRA-07 — Facets and groups.** Provide facet wrap and a basic row/column grid, explicit panel order, shared/free x/y scale policies, and stable panel keys. Empty panels follow an explicit keep/drop setting. A layer missing a facet field must explicitly broadcast to panels or name target panels. Shared scales train from compatible contributing panels; free scales use their own populations. Selection and provenance retain panel/group identity.

**GRA-08 — Transform reuse and extensions.** Named transform outputs form an acyclic revisioned graph and can feed several layers/charts. Cache by input revisions, operation ID/version, parameters, grouping/facet scope, and declared viewport dependence. Custom stats declare append/window/correction/full-recompute capabilities; these declarations are verified against batch results. Portable operation registration resolves known IDs and versions; unknown or native-only operations fail explicitly. Do not deserialize executable code or arbitrary dynamic plugins.

### 5.1 D3 shape feature parity

The first scoped production release MUST provide the capabilities in the
[D3 shape parity inventory](../impl_plans/d3-shape-parity-plan.md#required-feature-inventory),
using d3-shape 3.2.0 as the pinned behavioral baseline. The inventory is normative for
feature coverage; its work-package states and estimates are planning information.
Rust-native APIs and portable data descriptors may replace JavaScript syntax and
callback plumbing. Document all adaptations, preserve existing chart defaults, and
provide an explicit D3-compatible route for differing valid-input behavior. Feature
parity does not require JavaScript coercion, non-finite paint coordinates, DOM/Canvas
objects in core, or unrelated D3 modules.

| Requirement | Required observable behavior |
| --- | --- |
| SHP-01 — Shape outputs | Standalone checked numeric generators, external path sinks and SVG path-data output with configurable digits; constants/accessors, empty/degenerate behavior, repeatability and bounded ownership. Formatting MUST NOT quantize sink geometry. |
| SHP-02 — Lines and areas | Cartesian line and general paired-boundary area generators, defined masks, input-order compatibility, all coordinate controls and area boundary-line helpers. Preserve existing chart gap/order/singleton and ribbon defaults through explicit API distinctions. |
| SHP-03 — Curves | All 20 D3 curve factories, parameter defaults/controls, open/closed endpoint behavior and custom lifecycle compatibility. Unsupported combinations such as bundle areas diagnose explicitly. |
| SHP-04 — Arcs and pies | Arc radii, sweeps, corners, padding and center-line centroid; pie values, sorting and angle layout with stable source identity and input-order results. |
| SHP-05 — Radial shapes and links | Radial point/line/area and boundary helpers; generic, horizontal, vertical and radial links with complete coordinate/accessor controls and reference angle/tangent conventions. |
| SHP-06 — Symbols | All 13 distinct symbols, fill/stroke palettes and aliases, area-size and type mappings, open-stroke behavior and matching legends. Existing radius-based size decoding retains its meaning. |
| SHP-07 — Stacks | Six built-in orders, five offsets, explicit/custom order and offset, series/point metadata and source identity. D3 expand and existing separate-sign normalize remain distinct. |
| SHP-08 — Custom shapes | Public curve lifecycle, symbol drawing, pie comparator and stack order/offset protocols; native accessors and versioned portable registrations use the same Rust algorithms. Arbitrary callback serialization rejects. |
| SHP-09 — Shared integration | Required shapes flow through grammar, scales/layout, facets/themes, scenes, native/export and actual Python/WASM proofs. Inspection preserves source versus derived identity, actual curves, clips and holes; updates and retained snapshots agree with batch semantics. |
| SHP-10 — Parity evidence | Complete export/method inventory, pinned reference fixtures plus independent expectations, operation-specific numerical tolerances, inspected native/SVG/PDF/PNG artifacts, actual binding execution and bounded-resource/performance evidence. No missing item may be hidden by a broad family-level pass. |

Reference dependencies remain optional development tools. Normal Rust fixture tests
MUST run from stored expectations without installing D3. Source/license provenance,
exact reference identities and compatibility differences belong in the fixture manifest
and decision record. G-SHAPE is required before G4; existing Cartesian G2 evidence is
historical proof of that narrower milestone, not proof of these added requirements.

### 5.2 D3 hierarchy feature parity

The first scoped production release MUST provide the full d3-hierarchy 3.1.2 surface
in the [hierarchy inventory](../impl_plans/d3-hierarchy-parity-plan.md#required-feature-inventory).
That inventory is normative for feature coverage; estimates and package states are
planning information. Checked Rust APIs, immutable results and explicit layout history
may replace JavaScript syntax and object mutation. Preserve common valid-input defaults,
ordering and output semantics; document language/invalid-input adaptations explicitly.
Hierarchy kernels MUST be usable without chart construction, rendering or host runtimes.

| Requirement | Required observable behavior |
| --- | --- |
| HIR-01 — Construction and identity | Nested/custom-child and ordered grouped inputs, standalone node construction, ID/parent and path stratification including inferred ancestors; node payload, parent/children, depth/height/value metadata. Stable occurrence identity is separate from parent-lookup labels and source identity. Validate malformed topology and resource budgets before publishing results. |
| HIR-02 — Node operations | Ancestors, descendants, leaves, find, shortest tree path, links, breadth-first iteration/visitation, pre/post-order visitation, sum/count/sort and subtree copy. Preserve traversal and callback context, own internal values, shared immutable source payloads and independent copied structure. |
| HIR-03 — Tree and cluster | Both tidy tree and leaf-aligned cluster kernels with extent/node-size modes, configuration readback and default/custom separation. Provide x/y results for Cartesian and radial recipes; node-size mode roots at the origin. |
| HIR-04 — Partition | Weighted adjacency layout with rectangle bounds, size, rounding and padding controls. Icicle/sunburst recipes declare depth/radius mapping and reuse shared shape projection. |
| HIR-05 — Packing | Hierarchical circle packing with fitted or explicit leaf radii and constant/accessor padding, plus standalone sibling-packing and minimum-enclosure helpers. Preserve deterministic output and diagnose invalid/degenerate cases without non-finite scenes. |
| HIR-06 — Treemaps | All six built-in tilers, standalone tiler invocation, custom tilers, ratio factories, size/rounding and complete constant/accessor inner/outer/side padding controls. Resquarify MUST preserve compatible layout history and expose a documented reset/invalidation policy. |
| HIR-07 — Shared integration | One core engine for standalone APIs, grammar/recipes, native presentation, immutable export and versioned Python/WASM adapters. Native accessors and registered portable equivalents, exact identity/provenance, nested/radial hit testing, keyboard inspection, themes/facets and atomic update behavior use existing contracts. |
| HIR-08 — Acceptance | FIX-H01 MUST cover every export/method/default with pinned reference provenance, independent expectations, operation-specific tolerances, actual Rust/Python/WASM execution and inspected native/SVG/PDF/PNG artifacts. Include update/history/resource cases and supplemental measured workloads before release. |

Topology/aggregation are prepared independently of destination layout. Bounds-dependent
hierarchy layout runs after panel allocation and before primitive lowering, without
retraining source statistics on zoom or resize. Shape generation remains owned by
SHP-01–10. Source filtering must explicitly handle broken parent relationships. Synthetic
ancestors, aggregate parents and links MUST retain truthful source/aggregate/derived
provenance. Zero-valued nodes remain structurally available even if they emit no paint.

Stateless updates MUST agree with fresh batch computation. Resquarify updates MUST agree
with the same reference history; explicit reset agrees with fresh layout. Changes to
ordered topology or ratio require a defined invalidation/reset policy. Snapshot/export
must retain resolved geometry or sufficient immutable history to avoid consulting a
newer mutable layout cache. All operations preserve DAT/BND/QLT validation, exact identity,
ownership and bounded-work contracts; no JS engine or compulsory I/O enters core.

G-HIERARCHY is required before WP-21/22 acceptance and G4. Existing G2 evidence proves
its recorded Cartesian scope only. Hierarchical clustering, general network layout,
collapse/drill-down widgets and hierarchy morphing are not required by this addition.

## 6. Scales, coordinates, and domains

**SCL-01 — Capabilities.** Continuous scale APIs expose mapping and, where valid, inversion; band/point scales expose category lookup and extents. Never pretend every scale has a numerical inverse. Cartesian coordinates and SHP-05 radial shape projection are required; extension interfaces explicitly report inversion, clipping, and path-subdivision capabilities. Radial shape support does not imply a general polar-axis navigation product.

**SCL-02 — Domain precedence.** Resolve an explicit domain first; otherwise derive from eligible post-stat/post-position contributions. Apply the declared baseline/zero policy, then padding/nice policy only where enabled. Explicit domains are exact by default. A separate viewport determines the visible region. Hidden series remain in domain/stat training by default so legend toggles do not unexpectedly rescale; an explicit exclude-hidden policy recomputes affected domains.

For empty data, explicit domains are retained; absent domains use a documented neutral continuous domain [0,1] or an empty categorical domain, and display a no-data state. Automatic chart training and existing recipe policies expand constant numeric domains using a scale-specific deterministic rule. D3-compatible scale descriptors MUST also preserve explicitly supplied constant/repeated knots and provide their defined mapping behavior without automatic expansion. Descending domains/ranges are valid for families supporting them; ordered classifiers validate their own ordering requirements. Tests MUST lock expansion and degenerate mapping rules. Explicit nice operations are distinct from automatic training policy and may deliberately change authored endpoints.

**SCL-03 — Numeric and color scales.** Numeric scale implementations MUST validate parameters, domain direction, finite projection, and applicable round-trip behavior within stated numerical tolerance. Log supports strictly positive or strictly negative domains without crossing zero, with a finite positive base other than one; operation-specific validity and tick behavior follow SCL-07. Symlog has a positive linear threshold and documented forward/inverse functions. Continuous and discrete color scales have declared palettes/interpolators, null styles, domain/clamp policy, and legend metadata. Color-only changes cannot change positional domains. The earlier positive-only/base > 1 implementation remains historical WP-11 coverage, not full compliance with this revision.

**SCL-04 — Time.** UTC and explicit local-time scales MUST use calendar interval logic for ticks and nice, with default and custom formatting. Timestamp units are explicit, and formatting never silently applies the machine's timezone. Local-time floor/ceil/offset, ticks and formatting use the same supplied versioned timezone implementation/resources; changing labels alone is insufficient. Session-time mapping requires a supplied session calendar and a declared closed-session policy; it is a separate option from elapsed-time mapping. Test gaps, day/month/year boundaries, leap days, and local DST folds/transitions. Preserve integer-origin precision; compare D3 on the common millisecond range and independently verify finer source units. Do not ship exchange calendars or imply market-calendar correctness.

**SCL-05 — Multiple scales and clipping.** Layers bind to named scale IDs. Independent scales and alternate-unit secondary axes are separate constructs. A secondary axis must be a valid one-to-one mapping over the represented domain. Clip to plot/panel bounds by default while allowing declared annotation overflow. Out-of-domain omission/clamping is explicit and does not retroactively change upstream stats. The painter, export, and hit tester use the same clips and transforms.

**SCL-06 — D3 scale families.** Production MUST provide equivalent capabilities for the 26 scale factories in d3-scale 4.0.2: linear, pow, sqrt, identity, radial, log, symlog, time, utc, ordinal, band, point, quantile, quantize, threshold; sequential plus log/pow/sqrt/symlog/quantile variants; and diverging plus log/pow/sqrt/symlog variants. The [scale parity plan](../impl_plans/d3-scale-parity-plan.md) maps the official reference surface to implementation and evidence. Support applicable positional and nonpositional aesthetics, generic typed ranges and interpolation customization, classifier inverse extents, and meaningful guides. Required families cannot be deferred as arbitrary custom extensions.

**SCL-07 — D3 scale operations.** For equivalent typed inputs, provide the reference's defined mapping, piecewise knots, inversion where supported, clamp/unknown, range rounding/interpolation, domain/range configuration, family parameters, independent copying, ticks/nice and tick formatting. Include ordinal implicit-domain construction, band/point alignment/padding/step/bandwidth, quantile/quantize breakpoints, threshold inverse extents, and top-level tickFormat. Family-specific methods and defaults MUST be inventoried against pinned source, including inherited methods and degenerate inputs. D3-compatible defaults must be available; chart recipes may retain explicit existing defaults. Separate immutable scene resolution from authoring-time changes, and candidate ticks from layout thinning. JS coercion/object identity is not required; typed adaptations and undefined/non-finite behavior MUST be documented and tested. Unsupported defined behavior remains an open parity gap. Native custom interpolation and portable built-in/registered equivalents follow ARC-03 and BND-01; no JS runtime is required by core.

**SCL-08 — Parity acceptance.** FIX-20 MUST cover the complete constructor/method inventory using version-pinned D3 results plus independently calculated cases, with operation-specific tolerances and explicit timezone/locale/resource inputs. Rust tests consume committed fixtures without reference-engine installation. Actual Rust, Python and WASM executions MUST prove standalone scale operations and applicable chart/guide/update behavior; native and headless outputs require inspected artifacts. Existing FIX-07 or a passing family enum/compile is insufficient. G4 requires all scale parity cases and supported-surface evidence; the accepted 0.1.0 G2 milestone does not certify this added scope. AXIS-02/03 consume these shared scale tick/formatting operations rather than implementing them again.

### 6.1 Axis parity contract

Version 0.2.0 adds required observable parity with [d3-axis](https://d3js.org/d3-axis),
referenced to version 3.0.0. The [axis parity plan](../impl_plans/d3-axis-parity-plan.md)
maps its API surface, implementation gaps, compatibility choices and FIX-19 cases.
This requires native/portable capability equivalents, not JavaScript method syntax or
a browser renderer. Automatic comparisons use matched parameters on shared scale
families; other numeric-output mappings use a checked provider boundary. Retain existing
chart defaults through an explicit D3-compatible profile and versioned migration.
Every parity claim MUST name that profile, scale-family coverage and supported hosts.

**AXIS-01 — Scale and guide ownership.** Guides MUST have independent stable identities
and reference shared scales without retraining them. Support all four orientations,
multiple guides per scale/side, explicit placement and scale replacement. Provide a
bounded positional provider contract with domain/range, mapping and optional tick,
formatting and band capabilities; inversion is not compulsory. Keep providers portable
or report unsupported serialization/export explicitly.

**AXIS-02 — Tick selection.** Support per-guide tick arguments, explicit typed values,
automatic reset and an explicit empty list. Explicit values bypass automatic enumeration;
arguments still inform default formatting. Band/point guides fall back to domain order.
The compatibility profile MUST match reference tick policies for shared scale families,
preserve authored order and tick identities, and separate resource limits from count hints.
Label collisions or duplicate strings MUST NOT silently remove requested ticks.

**AXIS-03 — Formatting.** Value selection and formatting MUST be independent. Support
scale-default formatting with appropriate precision, numeric/time descriptors, explicit
locale/zone, custom formatters and reset. Preserve empty/repeated labels and exact source
timestamps. Portable descriptors or registered IDs MUST resolve through the shared
engine; native callbacks cannot be silently serialized. Unsupported formats fail explicitly.
Local-zone behavior requires a supplied provider and DST evidence, never machine defaults.

**AXIS-04 — Geometry and layout.** Support independent inner/outer tick sizes, combined
size, padding, pixel offset and axis translation. Finite signed sizes/padding are valid.
Domain paths and caps MUST use resolved range endpoints. Match orientation, label anchors
and categorical centering/rounding under the compatibility profile. Hosts supply device
scale; publication captures explicit output-unit policy. Preserve authored ticks in that
profile; adaptive label hiding/tick thinning are separate declared policies. Retain bounded
layout, explicit clipping/overflow and last-valid-scene failure behavior.

**AXIS-05 — Customization.** Expose stable domain/tick/line/label component roles and
independent whole-guide/per-tick style and visibility through portable scenes. SVG text
output MUST expose equivalent addressable components; outline output retains logical
labels and roles with its text limitation documented. Native/PDF/PNG use resolved styles
from the same core. Decorative axis components do not become fake data targets.

**AXIS-06 — Updates and transitions.** Repeated rendering and scale/guide changes MUST
produce coherent results; final updated output agrees with fresh layout. Axis enter/update/
exit geometry and opacity transitions are required for full parity, with stable identity,
orientation replacement, interruption, reduced motion and disposal behavior. Host clocks
drive portable transition plans. Inspection and export MUST capture one declared presented
state, including during transitions. General chart morphing remains outside this requirement.

**AXIS-07 — Evidence and delivery.** FIX-19 MUST cover every required axis capability with
pinned reference provenance, independent expectations, actual Rust/Python/WASM runtime
proofs and inspected native/SVG/PDF/PNG artifacts. Rust tests consume stored fixtures
without a JavaScript runtime. Maintain versioned schema/API migrations and a per-capability
verdict matrix. Full axis parity and G4 remain open while any required row lacks evidence.

### 6.2 Scale-chromatic parity contract

Version 0.3.0 adds required parity with d3-scale-chromatic 3.1.0. The
[chromatic parity plan](../impl_plans/d3-scale-chromatic-parity-plan.md) owns the complete
export inventory, source review, compatibility decisions and CP-01–05 delivery.
This supplements SCL-06–08; generic scale normalization/interpolation stays shared.

**CHR-01 — Complete catalog.** Production MUST expose equivalent typed capabilities
for all 76 reference exports: 38 schemes and 38 interpolators, including Observable10.
Built-ins must be independently discoverable/queryable/evaluable from Rust and actual
Python/WASM proof APIs as well as usable in chart definitions. JS spelling, mutable
array identity and CSS output string formatting are not required. Unknown IDs and
unsupported family/size combinations MUST produce contextual diagnostics.

**CHR-02 — Exact discrete schemes.** Supply all 11 fixed categorical arrays and every
size k=3–9 of the 18 sequential Brewer schemes and k=3–11 of the nine diverging Brewer
schemes: 218 arrays in total. Preserve exact order and sRGB bytes with opaque alpha.
Smaller tables MUST NOT be synthesized by sampling or truncation. Host-visible copies
cannot mutate shared built-ins. Custom palettes and stable category policies remain.

**CHR-03 — Reference interpolators.** Implement all 38 interpolators with reference
behavior: Brewer RGB basis splines, Viridis-family lookup tables, Turbo/Cividis
polynomials, long-hue Cubehelix and Rainbow/Sinebow cycles. Match canonical RGBA results
on equivalent finite inputs, including defined outside-domain results; no blanket clamp
or linear interpolation replacement is allowed. Intermediate calculations retain f64
precision before final channel conversion. Direct non-finite input rejects explicitly;
chart missing/non-finite data uses its declared missing color. Reversal is explicit and
must distinguish reversing a discrete array from evaluating an interpolator at 1-t.

**CHR-04 — Shared chart integration.** Named schemes/interpolators compose with SCL-06
ordinal, sequential/diverging variants and distribution scales through one core engine.
Domain normalization is separate from color evaluation. Marks and guides MUST agree;
legend identity and invalidation include complete catalog/interpolator configuration.
Color-only changes preserve positional domains, statistics and stable targets. Palette
changes, data updates and captured publication snapshots MUST agree with fresh batch
results. Theme conversion follows canonical color evaluation.

**CHR-05 — Portability and compatibility.** Version portable built-in descriptors and
expose standalone catalog, discrete lookup and interpolation operations in actual Rust,
Python and WASM proofs. Preserve existing v1 palette/linear-RGB and missing-outside
meaning through explicit migration/policies. Unknown new descriptors must fail on older
readers. No required built-in may depend on callbacks, host objects or a JS runtime.

**CHR-06 — Acceptance and provenance.** FIX-21 MUST cover every export/size with pinned
source/license provenance, independent cases, exact canonical color expectations and
explicit invalid-input behavior. Include real binding execution, inspected native and
SVG/PDF/PNG artifacts, update/snapshot equivalence and bounded resource measurements.
Rust tests run from committed fixtures without D3 installation. G-CHROMATIC is required
before WP-21/22 and G4; historical WP-11/13/G2 evidence does not certify this addition.

### Color values and operations

**COL-01 — Parsing.** Provide explicit, host-independent color parsing compatible with
d3-color 3.1.0 for named colors, transparent, 3/4/6/8-digit hex and its RGB/RGBA/HSL/HSLA
functional syntax. Preserve the parsed RGB/HSL space and distinguish invalid input from
a valid color with undefined channels. The [color parity plan](../impl_plans/d3-color-parity-plan.md)
owns the complete operation/fixture catalog. Full modern CSS parsing is outside this
module's parity claim; CHR-01–06 separately own the scale-chromatic catalog.

**COL-02 — Representations and conversion.** Provide binary64 RGB, HSL, Lab,
HCL/LCh and Cubehelix values, numeric/color/string constructors and Lab gray. Preserve
opacity, fractional/out-of-gamut channels and operation-specific undefined components.
Use reference-compatible D50 Lab conversion and direct Lab↔HCL conversion. HCL and
LCh differ in constructor argument order. Do not quantize intermediate color math.

**COL-03 — Manipulation and ownership.** Expose each space's channels/opacity,
independent copies with typed overrides, and reference-compatible brighter/darker
operations including defaults and fractional/negative amounts. Equivalent owned Rust
values/builders replace JS mutation and object identity; required color behavior cannot
be omitted as a host-language adaptation.

**COL-04 — Display and formatting.** Provide per-space displayability, RGB/HSL
clamp, formatHex/formatHex8/formatRgb/formatHsl, default RGB string output and an
equivalent deprecated hex alias. Match boundary rounding, hue wrapping, opacity,
exceptional-channel fallback and string formatting. These operations MUST NOT mutate
the source. A displayability check is distinct from clamping or gamut quantization.

**COL-05 — Integration and portability.** One core engine MUST serve standalone
Rust/Python/WASM color operations and every authored paint input. Versioned color
descriptors preserve space, precision and exceptional-channel tags; invalid parse,
missing data and undefined channels remain distinct. Existing byte-color definitions
retain their meaning through explicit migration. Convert to validated unpremultiplied
sRGB paint at one declared boundary, documenting final byte/alpha quantization without
losing authored values. Renderers MUST NOT independently parse or perform color-space
math. SCL-03/07 interpolation consumes this engine; existing theme precedence,
grayscale policy and color-only invalidation contracts remain intact.

**COL-06 — Acceptance.** FIX-C01 MUST cover every required constructor/method and
exceptional state with pinned provenance, independent expectations, operation-specific
tolerances, actual Rust/Python/WASM execution and inspected native/SVG/PDF/PNG output.
Stored Rust fixtures run without D3. G-COLOR remains open until standalone and integrated
color/update/snapshot evidence passes; historical byte-paint/palette tests do not certify
these additions. CLR-01–05 deliver this scope before WP-21/22 and G4.

### 6.3 Interpolation parity contract

Version 0.3.0 requires the complete [d3-interpolate](https://d3js.org/d3-interpolate)
3.0.1 capability surface. The [interpolation plan](../impl_plans/d3-interpolate-parity-plan.md)
owns its 27-export/configuration inventory, typed compatibility profile and FIX-I01 cases.
This includes standalone interpolation without a chart, window, clock or JavaScript runtime.

**ITP-01 — Shared engine and profile.** Core MUST own reusable interpolation factories
and explicit-parameter sampling. All documented exports and factory/result controls need
native and portable equivalents. Specify target-kind dispatch, mixed-kind conversions,
missing/non-finite outcomes, limits and ownership. Host-language adaptations may replace
JS coercion, prototype behavior and result aliasing, but MUST NOT omit a defined numerical,
color, structural, transform or zoom capability. No browser/interpreter objects enter core.

**ITP-02 — Value interpolation.** Provide number, D3-compatible rounded number, embedded-
number string, millisecond Date-equivalent, generic array, typed numeric array and record
interpolation, including nested target-directed dispatch. Preserve target shape, numeric-
array conversion and declared output formatting. Exact source timestamps/IDs retain their
integer representations. Default returned samples own their data; optional reusable output
has explicit overwrite semantics. Exceptional numerical results never become invalid scenes.

**ITP-03 — Splines and composition.** Provide scalar basis and closed basis, discrete
interpolation, piecewise composition with default/custom factories, and quantize sampling.
Match each operation's defined clamping, wrapping, extrapolation, boundary and sampling
behavior; do not impose universal t clamping. Document empty/singleton/invalid cases.
Retained structured samples MUST be independent. Shape path curves remain SHP-03.

**ITP-04 — Color interpolation.** Provide RGB, RGB basis/open and closed, HSL/HSL-long,
Lab, HCL/HCL-long, Cubehelix/Cubehelix-long and hue interpolation, with gamma on the
reference's applicable factories. Support pinned color parsing/conversion, missing channels,
hue paths, opacity and output formatting. Compute with floating channels; only lower to
scene bytes at the declared boundary. Match the reference RGB-spline opacity limitation.
Existing byte-palette recipes retain explicit defaults; a palette catalog is not required.

**ITP-05 — Transform interpolation.** Provide 2D CSS/SVG transform equivalents using shared
affine decomposition and component interpolation, including rotation path, reflection and
singular-case handling. Support headless absolute transform inputs, typed matrices and
CSS/SVG syntax/output adapters. Context-dependent inputs require explicit resolved inputs;
a DOM/CSS engine and 3D transforms are not implied. Prove matrix/point and syntax behavior.

**ITP-06 — Zoom interpolation.** Provide smooth center/width trajectories, configurable rho
and recommended-duration metadata equivalent to the pinned reference, including near-
coincident centers and duration sign. Host scheduling policy is separate from reference
metadata. Validate width and exceptional results. The complete d3-zoom interaction product
is not implied, and no hidden clock or animation scheduler is allowed in core.

**ITP-07 — Integration and delivery.** Scales, axis transitions and applicable publication
consumers MUST call the shared interpolation engine. Native custom factories and portable
built-in/registered equivalents follow ARC-03/BND-01. Version descriptor/API migrations,
preserve existing recipe behavior explicitly and reject unavailable portable operations.
Actual Rust/Python/WASM executions MUST exercise standalone and composed interpolation.
Host scheduling, reduced motion, interruption, disposal and capture obey SCN-04/AXIS-06.

**ITP-08 — Acceptance.** FIX-I01 MUST inventory every export and configuration/result control,
with exact reference provenance, independent expectations, per-operation tolerances,
output-lifetime tests and inspected applicable native/SVG/PDF/PNG artifacts. Rust fixtures
run offline. Compare updates with fresh computation, and measure preparation/sampling and
allocations under the existing PERF protocol. G-INTERPOLATE and G4 remain open while any
required capability or supported-surface proof is missing. Existing alpha/scale tests do
not certify interpolation parity.

## 7. Layout, typography, and themes

**LAY-01 — Constraint layout.** Support responsive desktop bounds, minimum useful plot size, shared-axis alignment, legends outside panels, and bounded text-aware margin solving. Cap iterative layout at a documented constant and emit a layout-pressure diagnostic when deterministic tick thinning/truncation is used. Layout cannot loop indefinitely. At tiny bounds, show a meaningful compact/no-space state without invalid geometry.

**LAY-02 — Typography.** Text measurement and painting for each destination MUST agree on font identity, shaping, size, weight, language/direction, rotation, and relevant scale. Support rich text runs, rotated axis labels, Unicode text/minus signs, tabular numerals when available, numeric formatting, and explicit line spacing. Preserve logical text even if display uses shaped glyphs. Font fallback must be deliberate and diagnosable.

**LAY-03 — Figure composition.** Publication figures MUST support titles/subtitles, axis titles, panel letters, aligned multi-panel layout, inset plots, shared or per-panel legends, direct labels, callouts, source notes, and footnotes. Annotations declare coordinate space: data, panel-relative, figure-relative, or output units. User constraints and positions must survive resizing with their declared meaning. Automatic collision avoidance must expose its priority and fallback behavior.

**LAY-04 — Destination fidelity.** Desktop layout uses logical pixels and native metrics. Publication layout uses physical dimensions and points with explicitly supplied fonts/resources. A publication preview MUST use publication layout results or a demonstrably equivalent measurement path. OS-native and publication metrics are not assumed identical. Inspect exported output as well as native screenshots.

**THM-01 — Complete theme model.** Theme tokens cover typography, spacing, axes/ticks/grids, colors, strokes/dashes, fills/gradients, symbols, legends, panels, annotations, and focus/selection states. Supply at least an editorial white theme, a dense terminal theme, and a print/grayscale theme. Themes MUST work headlessly and must not require Kit tokens or CSS.

**THM-02 — Precedence and serialization.** Resolve defaults -> host theme -> named theme -> plot overrides -> layer overrides -> applicable interaction styling. A separate output profile controls export state visibility and output-specific overrides. Mapped data colors and constant styles are resolved through explicit rules, not guessed from strings. Theme definitions are versioned, composable, serializable, and validated. Palette-only updates preserve numerical preparation; typography/spacing updates invalidate layout.

**THM-03 — User control.** Expose styling and formatting at chart, scale/guide, layer, and annotation boundaries without requiring a renderer fork. Native formatters may be closures; portable formatters use documented descriptors or registered IDs and locales. Missing portable equivalents fail export/serialization explicitly when needed. Interaction styling must preserve readable focus cues in each supplied theme.

## 8. Scene and native renderer

**SCN-01 — Portable primitives.** Scene representation MUST include groups, paths/curves, rules, rectangles, points/symbols, area/ribbon fills, text, transforms, clip scopes, z-order, paints, and resource references. Store numerical path buffers rather than reparsing SVG strings in normal native rendering. All coordinates/resources are validated before submission.

**SCN-02 — Semantic separation.** Geometry and semantic target metadata are separate from original typed rows. Decorative geometry does not create fake data targets. Scene metadata includes compilation stamps, bounds, scale/coordinate descriptors needed for interactions, and target resolvers. A serializable scene DTO is an explicit narrower format; native callbacks/objects cannot be serialized implicitly.

**SCN-03 — Renderer capabilities.** Required primitives MUST have a tested GPUI and export path. Report unsupported features with layer/resource context. Allow explicitly requested localized raster fallback for export; never silently flatten an entire vector figure. Custom native painting MUST declare an export representation or report unsupported export. Browser renderer parity is future scope.

**SCN-04 — Shared presentation truth.** Painting, hit testing, focus navigation, tooltips, and animation MUST refer to the presented scene and its snapshot. Scales/clips or indexes from another revision cannot be mixed with visible geometry. Compatible opacity/geometry transitions are optional except for the required axis transition scope in AXIS-06; all transitions must honor reduced motion, interruptions, and target identity.

### 8.1 D3 path feature parity

Production MUST provide the capabilities of **d3-path 3.1.0** through one checked
Rust implementation. The [path parity inventory](../impl_plans/d3-path-parity-plan.md#required-feature-inventory)
is normative for methods and edge coverage. This refines SHP-01's shared path foundation;
it does not introduce a second generator engine or require a browser Canvas runtime.

| Requirement | Required contract |
| --- | --- |
| PTH-01 — Standalone path API | Public equivalents of `Path`, `path()` and `pathRound()`, independent mutable builders, reusable numeric sink/replay, owned immutable results and non-destructive SVG path-data output without a chart, fonts, GPUI or interpreter. |
| PTH-02 — Drawing and subpath state | Move, line, quadratic and cubic Béziers, close and signed/zero-size rectangle subpaths. Match pinned finite-input sequence behavior, including empty/move-only output, multiple subpaths, repeated close, implicit starts and continuation after close. Distinguish serializer results from valid submitted scene geometry; preserve closure and paint-relevant degeneracy in conversion. |
| PTH-03 — Circular arcs | Center/radius/angle `arc` and tangent `arcTo`, both directions, connecting segments, wrapping/full circles, zero/negative radii, coincident/collinear inputs and reference threshold behavior. Retain circular geometry until bounded destination lowering; circle point primitives or coarse polygons are insufficient. |
| PTH-04 — SVG precision | Unrounded `path`/default `Path`, default-three-digit `pathRound`, configurable nonnegative fractional-digit limits with flooring and >15 fallback. Match defined numeric rounding including negative ties and signed zero; document equivalent spelling differences. Formatting MUST NOT quantize builder state, numeric sinks, native/hit geometry or snapshots. Shape outputs consume the same formatter with their own explicit defaults. |
| PTH-05 — Safety and consumers | Finite checked geometry, atomic owned-builder errors and bounded commands/bytes/replay/subdivision. Preserve existing scene/Rect contracts through explicit normalization or documented migration. One geometry route feeds native and SVG/PDF/PNG, transforms, clips, winding and snapshots; controls/tessellation do not create source targets. Actual Python/WASM APIs invoke core and expose equivalent path operations/results. |
| PTH-06 — Acceptance | FIX-P01–06 cover every export/method and operation sequence with pinned reference results and independent geometry, actual Rust/Python/WASM execution, supported-platform checks and inspected native/publication artifacts. Existing Bézier transport or compile-only bindings do not establish G-PATH. |

Host-language spelling, ownership and prototype identity may differ. JS coercion and
non-finite/overflowing SVG output are explicit safety adaptations; reject invalid core
geometry without partial mutation. Defined geometric capabilities cannot be excluded
as language adaptations. The standalone serializer preserves reference behavior for
finite sequences even when their string is not a valid submitted SVG path; scene
submission diagnoses such sequences. Full Canvas features beyond d3-path, SVG parsing
and path measurement are outside this parity requirement.

### 8.2 Native host integration

**GPU-01 — GPUI integration.** Implement one chart element or a bounded number of pane elements with retained state. Rendering an element MUST NOT recreate all datasets or allocate an entity per data point. Layout/prepaint/paint stages use the selected GPUI API. Native tooltip/control bodies may be caller-supplied GPUI elements; publication content needs portable equivalents.

**GPU-02 — Thread and lifecycle.** Keep GPUI window/text operations on their required thread. Background tasks operate on immutable thread-safe inputs and return stamped results. Disposal cancels subscriptions/jobs, releases snapshots and resource handles, and prevents callbacks into a destroyed entity. Repeated mount/unmount and failed preparation must remain safe.

**GPU-03 — Kit and accessibility integration.** Kit integration is optional and maps its selected dependency/theme/control APIs into the generic host. Provide chart summaries, focusable meaningful targets, keyboard equivalence, and an accessible data alternative. Verify actual platform accessibility exposure; keyboard support alone does not justify claiming screen-reader support. Publish any platform limitation with the release.

## 9. Interaction and application state

**INT-01 — Common actions.** Pointer, keyboard, controls, and programmatic callers MUST dispatch through a shared typed reducer. Required action families are viewport/navigation, follow mode, hover/focus/pinning, series visibility, selection/brush/lasso, annotations, reset, and linked-view synchronization. Actions include origin and an idempotency/revision strategy appropriate to their lifetime; resulting events report effective changes, not every redundant input.

**INT-02 — Interaction state.** Keep viewport, follow mode, hover, focus, selection, pinned target, legend visibility, annotations, active gesture, and configuration revisions distinct. Support internally managed state and an application-controlled adapter. Controlled updates use revision checking; stale host responses cannot overwrite a newer action. Definitions describe behavior; ephemeral pointer state is not serialized as part of the definition.

**INT-03 — Inspection and navigation.** Provide nearest-x/grouped inspection for time series, point-nearest inspection with a configurable radius for scatter, and geometry containment for bars/cells/candles. Hit priority follows visible z-order with explicit overlays/handles first. Tooltips must identify interpolated or aggregated values as such and retain source access.

Support pointer-anchored wheel/trackpad zoom, drag pan, zoom-to-region, range setters, reset, and optional overview/navigator controls. Numeric/temporal scales navigate in their declared transformed coordinate space; categorical navigation operates on category windows. Clamp/infinite-range behavior is explicit. Keyboard navigation visits meaningful targets in deterministic panel/series/order sequence, with accessible announcements where supported.

**INT-04 — Selection and linking.** Provide point/series selection, range and rectangular brushes, lasso for suitable 2D geometry, add/toggle/clear, and linked selections. Default scatter selection tests point centers; bars/cells use intersection; lines use source vertices unless segment selection is explicitly requested. Return provenance-aware targets; an interpolated segment location is derived, not an invented source row.

Brushes select/highlight by default; filtering is a separate application command changing the population. Linked views exchange domain values, category identities, and stable target IDs rather than pixels. Include origin/revision information to prevent loops. Missing values/keys and incompatible scales follow declared matching/rejection policies.

**INT-05 — Editing and gesture arbitration.** Editable annotations, thresholds, and range handles support snapping, constraints, preview, commit, cancel, and durable command history. Editing plotted source data requires an application-defined handler; the library does not mutate source measurements merely because a mark is dragged.

At most one interaction owns a gesture. Specify hit priority, pointer capture, modifiers, Escape, capture loss, focus loss, disposal, and removal of the edited target. A drag uses a pinned presented-scene coordinate basis or explicitly rebases without discontinuity. Data arriving mid-gesture cannot move its coordinate basis silently. Transient hover/preview events do not flood undo history.

**INT-06 — Host controls and extensions.** Expose tooltip, context-menu, toolbar, copy/export, and legend hooks. Default bindings apply only while the chart has the relevant focus and can be remapped by the host. Custom geoms supply hit geometry, semantic values, selection policy, and keyboard order as well as paint output. Unsupported operations are reported per capability instead of being advertised as working.

## 10. Streaming, scheduling, and invalidation

**STM-01 — Retention and overload.** Support bounded count-based retention and explicit time-window retention. Time-window policy declares timestamp field, watermark source, permitted lateness, and eviction boundary. Default event-time eviction uses a nondecreasing supplied watermark; receipt of a future-dated row cannot silently advance it. Count retention evicts by declared insertion/order policy, not incidental memory position.

Ingestion queues have explicit capacity in bytes and/or rows. Default overload returns backpressure without accepting the batch. Optional latest-value, drop, or summarization policies must be explicit and observable. A frame being skipped does not mean an accepted data operation was skipped. The chart does not promise durable event-log storage.

**STM-02 — Follow and inspection.** Provide follow-latest, inspect-history, freeze-presentation, and resume-latest actions. Manual pan/zoom enters inspect-history by default. Freeze retains a coherent visible snapshot while ingestion follows retention policy; bounded historical resources remain held only as needed. Resume is explicit. New batches preserve user zoom, compatible selections, legend visibility, and annotations.

**STM-03 — Incremental correctness.** Append-only sorted updates SHOULD use incremental channel/domain/stat preparation where exact. Upserts, removal, and retention invalidate all affected outputs. Each built-in stat documents its incremental support and fallback. A full recompute is permitted for a stat that cannot update exactly, subject to measured responsiveness and bounded memory. Approximate quantiles/aggregation require explicit algorithms and error/quality metadata; they cannot silently replace exact semantics.

**STM-04 — Scheduling and convergence.** Bound worker jobs and pending work. Default scheduling allows an active preparation and a newest pending snapshot, coalescing obsolete pending work. Results may be presented when their spec/layout/viewport stamps are compatible, their resources are valid, and their store commit revision is not older than the last presented compatible scene. They need not match the newest accepted data revision during continuous ingestion. Per-dataset revisions still drive fine-grained cache reuse.

Never cancel every build merely because another batch arrived: continuous input must still produce progressing visible revisions. Reject results with obsolete incompatible spec/viewport/layout stamps and prevent older completions from overwriting newer presented results. Input uses the presented scene. Expose lag between committed and presented revisions. Where several datasets are linked transactionally, present their coherent transaction snapshot rather than mixing revisions.

**STM-05 — Cache and dense representation.** Cache dependencies by stage and revision. Pointer movement ordinarily changes only hit queries and overlays; palette changes preserve numeric preparation; font changes invalidate text/layout; data changes invalidate affected computation/geometry. Core statistical work cannot run once per pointer event.

Dense line reduction MUST preserve order, gaps, endpoints, and visible extrema under the declared algorithm. Keep exact raw lookup available within retention. Candle aggregation uses first open, maximum high, minimum low, last close, and sum of valid volume within declared buckets. Downsampling is presentation-only unless an explicit statistical approximation is selected. Record source rows, prepared rows, geometry vertices, and represented samples separately.

## 11. Publication and export

**EXP-01 — Formats and dimensions.** Produce vector SVG and PDF, plus PNG with explicit dimensions/DPI. Support point-based physical layout and at least 300/600 DPI fixtures. SVG/PDF MUST retain vector marks for the supported publication feature set. Text-preserving and outline modes expose their editing/search tradeoffs. Support clipping, gradients, rotated text, annotations, and figure furniture consistently.

**EXP-02 — Fonts and fidelity.** Export uses explicitly resolved font resources and documented fallback/embedding/subsetting rules. Respect provided font embedding permissions. Missing glyphs or unsupported effects produce resource-specific diagnostics. An explicitly allowed localized raster fallback must identify its region and cause. Ordinary PDF export does not claim PDF/X, CMYK, spot color, ICC press workflows, or tagged-PDF accessibility.

**EXP-03 — Figure snapshot.** Export captures one coherent definition, transaction/data revisions, declared viewport, selected interaction-state policy, annotations, theme, font/resource identities, dimensions, and quality settings. Provide visible-view and full-domain modes. Streaming can continue while export uses its immutable snapshot. Annotation edits or late data must not enter halfway through the export.

Rebuild layout and geometry for requested output dimensions/tolerance; do not silently reuse a reduced screen envelope as the publication source. Export returns bytes and diagnostics; saving, clipboard, file dialogs, and network transfer are host operations.

**EXP-04 — Reproducibility and preview.** The same snapshot, dependency versions, resources, and settings MUST produce equivalent geometry/text within documented tolerances. Byte-identical PDF output is optional because metadata/encoding can differ. Store or emit reproducibility metadata including versions, font hashes, data/schema revisions, and geometry quality. Publication preview uses these settings and passes comparison fixtures. Headless export cannot require an initialized GPUI application.

## 12. Public API and portability contract

The table below defines engine and lifecycle boundaries, not separate mandatory
application setup steps. Version 0.5.0 adopts the owner's direction that concise
authoring is the main public API for all features. The
[primary authoring plan](../impl_plans/primary-authoring-api-plan.md) and
[ADR-013](../adr/013-primary-authoring-api.md) define its refactor over an assumed
completed original WP-01–23 baseline. This premise does not change historical ledger
evidence or automatically complete additional D3/GG packages. Concrete method names
are finalized through compiling external examples. No portable envelope version
changes merely because the public Rust surface is refactored.

| Conceptual entry point | Inputs | Result/ownership |
| --- | --- | --- |
| Build definition | Typed builders or a portable specification plus registered operations | Validated immutable definition or diagnostics. |
| Create data store | Schemas and immutable snapshots/columns | Dataset handles, stable keys and initial revisions. |
| Apply transaction | Expected revisions, operations, policy | Atomic outcome and resulting revisions/counts. |
| Prepare/layout | Definition, data snapshot, state, bounds, host text metrics | Prepared scene with compilation stamps; no window side effects. |
| Dispatch action | State, presented-scene reference, semantic action | New state/change set plus events/effects. |
| Present | Scene, compatible resources, host state | Native output and consistent hit/focus metadata. |
| Snapshot/export | Definition/data/state/resources and output profile | Immutable snapshot; bytes plus diagnostics. |
| Resolve target | Target reference and matching snapshot | Source row, aggregate membership, or derived metadata. |
| Dispose | Runtime/host handles | Cancellation and release; later calls return disposed/invalid-handle errors. |

### 12.1 Primary authoring requirements

| Requirement | Required behavior |
| --- | --- |
| AUT-01 — Primary and complete | The concise plot/data/component API MUST be the main public authoring route. Every delivered chart feature and meaningful option MUST be available through it, including advanced controls and extensions. Every future capability MUST extend an existing typed builder or add a focused component builder within this API, with applicable host adapters, examples and behavioral evidence in the same capability package. Runtime commands/queries remain on the retained Chart/destination API; standalone utilities retain direct access. Production recipes/examples use it. Raw grammar/JSON mutation cannot be the only route for a built-in capability. Reuse the existing shared engine and normalized representation. |
| AUT-02 — Data and identity | Support named columns and typed native rows, default plot data, layer overrides and generated routine identities/revisions. Preserve scalar kinds, null validity, exact time/integer metadata, stable handles/keys and atomic update contracts. Identity MUST NOT be reconstructed from current row offsets or value equality. Provide explicit catalog/order and live identity controls. |
| AUT-03 — Full composable grammar | Provide inherited and overridden aesthetics, separate mapped versus constant style, all delivered geom/stat/position/transform/scale/guide controls, named outputs and checked source/generated stages. Preserve baseline semantics; explicit compatibility profiles determine alternate defaults/grouping/staging. No independent recipe or host compiler is permitted. |
| AUT-04 — Complete design and composition | Expose all delivered facets, coordinates, themes, text/math/labels, annotations, figure furniture, insets and multi-plot composition through typed components with concise defaults and explicit advanced policies/resources. The labels builder MUST be limited to x/y-positioned annotations; plot titles, subtitles, x/y axis labels and legends MUST have separate builders, sharing existing text/layout/guide services. Deterministic inferred catalogs MUST retain stable identity and a defined update policy. |
| AUT-05 — One live runtime | Primary Chart operations MUST reuse the existing data store, compiler, state reducer and scheduling contracts. Expose updates/receipts, controlled state, queries, navigation, selection, linking, editing, retention/backpressure and quality policies without raw protocol authoring. Preserve acknowledged-scene input/capture, coherent snapshots, bounded work and atomic failure. |
| AUT-06 — Destinations | Native, optional Kit and export adapters MUST accept the same authored plot/runtime contracts. Retain native mounts and supplied resource contexts; offer concise bytes export and explicit host save/capture controls. No compulsory filesystem, system-font lookup, interpreter, GPUI object or threading enters core. Static and live capture semantics remain distinguishable and reproducible. |
| AUT-07 — Host-native adapters and diagnostics | Python/WASM MUST offer host-native data/authoring/action/result adapters to the primary Rust engine without requiring hand-written JSON envelopes. Preserve exact values, owned lifetimes, disposal and interpreter/memory rules. Errors expose actionable authored names/stages and structured properties while retaining stable diagnostic codes and identity context. Interchange remains versioned. |
| AUT-08 — Extensions and standalone capabilities | Supported native/registered extension protocols MUST work through ordinary typed plot components with existing schema/resource/capability validation. Standalone colors, paths, interpolation, scales and hierarchy helpers remain directly callable and feed the same chart components; no dummy plot or alternate engine is required. Native-only destinations report unsupported export explicitly. |
| AUT-09 — Migration and evidence | Maintain a complete capability-to-primary-API/evidence register, compiling external examples, old/new semantic and lifetime checks, actual native/export/Python/WASM evidence, compatible payload migrations and required performance requalification. Preserve historical defaults and wire identity unless explicitly versioned. G-AUTH MUST pass for a release incorporating this refactor; missing or uncertain delivered-capability rows keep it open. |

These requirements add a primary API acceptance gate; they do not rebuild completed
WP algorithms, weaken parity coverage or assert that any implementation has run.
The plan's FIX-AUTH00–09 identify the required evidence slices. Specialist public
modules and compatibility shims may remain, but cannot conceal a primary-API coverage gap.

### 12.2 Portable and binding contracts

**BND-01 — Portable specification.** Supply a versioned serializable specification covering all required built-in grammar constructs, named datasets/fields, stats, positions, scales, coordinates, facets, guides, themes, annotations, and interaction configuration. Native-only closures/widgets may remain outside it. The wire schema MUST reject unknown required constructs and validate resource limits before expensive allocation. Never serialize Rust pointers, GPUI handles, or interpreter objects.

Schema version and operation versions are explicit. Define supported-version ranges and explicit migrations; never silently reinterpret a statistical setting. State snapshots and data transactions use separate versioned envelopes. A built-in portable chart must round-trip without changing semantic meaning, even if field order or serialization formatting changes.

**BND-02 — Cross-language data.** Use batched columns and stable handles. Define copies, borrowed lifetimes, mutation rights, null validity, categorical dictionaries, and disposal. Default to owned immutable ingestion. No API advertises zero-copy without tests demonstrating its exact lifetime/mutation constraints.

On JSON/browser boundaries, preserve 64-bit integers and timestamps using decimal strings, BigInt-capable adapters, or a defined binary representation. Plain JavaScript Number is not the wire representation for arbitrary 64-bit identity/time values. Typed-array views into WASM memory must not be retained across invalidating growth/reallocation without renewal. Recoverable malformed input returns typed errors rather than process termination.

**BND-03 — Python proof.** Before core API stabilization, a minimal PyO3 adapter MUST construct a portable chart, ingest a batch, apply a correction/action, and receive headless export/state results matching native semantic fixtures. Avoid Python callbacks per point/frame; callbacks are prepared into data or resolved through Rust operations. Long Rust-only work uses the selected PyO3 interpreter-detachment API correctly. Test lifetime/disposal and importing headless functionality without GPUI initialization. Full wheels, native event-loop integration, and interactive notebooks remain later deliverables.

**BND-04 — WASM proof.** Before core API stabilization, a minimal wasm-bindgen adapter MUST compile core, construct the same portable fixture, apply a batch/correction/action, and return equivalent scene/state results in a real WASM runtime. Prove a basic scene/SVG output route. Test large IDs/timestamps, nulls, handle disposal, repeated updates, and memory-view ownership. Single-threaded execution is the required baseline; browser workers/shared memory are optional later acceleration. A browser interactive host is separate from GPUI and is not part of this proof.

## 13. Diagnostics, quality, performance, and release evidence

**QLT-01 — Error model.** User-data and unsupported-feature errors MUST be recoverable `Result`/outcome values, not normal-use panics. Diagnostics carry a stable code, severity, message, relevant dataset/layer/field/resource IDs, revision context, and actionable correction. Aggregate repeated invalid-row diagnostics with counts and bounded samples. Distinguish validation, schema conflict, numerical domain, unsupported capability, resource/font, layout pressure, cancelled/superseded, disposed handle, and export fidelity errors.

A failed update/export retains the last valid state/scene where possible. Do not substitute an empty successful output for a failed required feature. Definition validation rejects transform cycles, unreasonable allocation requests, and invalid scales before painting.

**QLT-02 — Semantic verification.** Own canonical fixtures and independent invariants are authoritative. Compare selected ggplot2/D3/TanStack/ECharts outputs only when parameters, operation order, and intended semantics agree. Rust tests must run from stored fixtures without installing R/JavaScript reference engines. Test stats/scales/provenance, layout/geometry snapshots, action traces, atomicity, incremental-versus-batch results, and malformed inputs. Floating-point tolerances are operation-specific and justified, not a global loose epsilon.

**QLT-03 — Visual and host verification.** Review representative native screenshots and actual SVG/PDF/PNG exports. Required cases cover fonts, high DPI, clipping, gradients, dashes/joins, rotation, missing data, tight bounds, multi-panel alignment, annotations, and all supplied themes. Use deterministic fonts/data for reproducible fixtures. Visual baselines cannot be regenerated solely to hide unexplained regression. Compile and run the declared supported platform matrix before release claims.

**QLT-04 — Performance budgets.** Measure release builds on a declared macOS Apple Silicon reference machine at approximately 1,200 logical chart pixels wide; record hardware, OS, dependency revisions, pixel scale and settings. These are initial engineering targets, not current achieved performance. Confirm their measurement protocol in the renderer spike; a proposed material reduction must be explicit in the project contract.

| Benchmark ID | Workload | Required evidence / initial target |
| --- | --- | --- |
| PERF-01 | Ten line series, 10,000 observations each | After preparation, p95 chart hover/cursor work <4 ms and p95 total frame time <=16.7 ms at a 60 Hz target; disclose reduction and geometry counts. |
| PERF-02 | One million retained time-series observations, bounded visible representation | No full source scan/rebuild per hover; exact retained-source lookup and extrema/gap preservation. Report memory and cold/update costs. |
| PERF-03 | 10,000 row updates/sec in batches, 100,000-row retained window, hover/pan for 30 minutes | No silent accepted-operation loss; coherent advancing presentation, bounded queues/live resources, correct correction/eviction results. Target p95 ingest-to-present lag <=250 ms after warm-up for the declared simple-line workload. |
| PERF-04 | 50,000 scatter points and a dashboard of 12 updating charts | Report raw versus aggregated crossover, index rebuild cost, fair chart scheduling, disposal and memory plateau under bounded retention. |
| PERF-05 | Publication export while PERF-03 continues | One consistent export snapshot; ingestion remains serviceable; report added lag/peak memory and verify released snapshot resources. |

A feature passes its performance gate only with measured evidence or an explicitly revised contract. Isolate ingestion, computation, layout, native tessellation/submission, presentation, and input latency. Report dropped/rejected/queued/committed/coalesced counts separately. Repeating cheap screenshots is not a substitute for the sustained streaming workload.

**QLT-05 — Compatibility and documentation.** Document semver/API policy, specification versions, feature flags, compiler/toolchain support, dependency pins, target support, renderer capabilities, provenance/licenses, limitations, and migration notes. Before 1.0, breaking changes still require an explicit changelog and spec migration where applicable. Built-in examples include both recipes and layered grammar, custom extensions, themes, streaming, linking, publication, and minimal bindings.

**QLT-06 — Release truthfulness.** Every required feature must have an implementation, executable/inspectable acceptance evidence, and no unresolved blocking diagnostic. A scaffold, ignored test, compile-only binding, screenshot without semantics, or reference-library feature list does not establish support. Publication fonts, platform accessibility gaps, approximations, and binding distribution limits must be stated accurately. This specification is a build contract, not a claim that the library exists.

## 14. Canonical acceptance fixtures

Implement these as small understandable fixtures before broad randomized/stress suites. Expected values must be calculated independently of the code being tested. Fixture IDs are referenced by the implementation plan.

| Fixture | Inputs/action | Required result |
| --- | --- | --- |
| FIX-01 | Line rows with y = [1, null, 3] | Two isolated valid runs; no zero substitution or connecting segment by default. |
| FIX-02 | Bin edges [0,1,2], values [0,0.5,1,2] | Counts [2,2]; final right edge included; four source members accounted for. |
| FIX-03 | Group values [2,3,-1,-4] | Positive endpoint 5, negative endpoint -5; normalized sides end at +1 and -1. |
| FIX-04 | Values [0,10,20,30], p=0.25 | Quantile 7.5 under the declared interpolation rule; missing values reported separately. |
| FIX-05 | Points (0,1),(1,3),(2,5),(3,8); zoom to x<=2, then apply a source filter x<=2 | Full-population fit has intercept 0.8/slope 2.3 and stays unchanged under zoom; the explicit filter yields intercept 1/slope 2. |
| FIX-06 | Two facets, shared scale, annotation missing facet field | Shared domain includes both; annotation requires explicit broadcast/target policy. |
| FIX-07 | Empty, constant, descending, log-invalid and large-origin time domains | Documented fallback/error behavior; finite paint geometry and valid supported round trips. |
| FIX-08 | Append, upsert, removal, duplicate/conflicting transaction, multi-dataset invalid transaction | Correct revisions/counts; no partial commit; batch recomputation and incremental results agree. |
| FIX-09 | Reorder and evict source rows while selecting/pinning aggregate/source targets | Stable identity; accurate membership/revision; declared target-removal behavior. |
| FIX-10 | Brush/lasso, linked view echoes, keyboard traversal and annotation drag cancelled by capture loss | One gesture owner; no loop; correct preview/commit/cancel and target semantics. |
| FIX-11 | Continuous stream plus deliberately slow/alternately finishing preparation jobs | Presented revisions progress monotonically; stale incompatible results rejected; no refresh starvation. |
| FIX-12 | Same composed figure under editorial, terminal and grayscale themes | Complete style change without numerical/statistical change; typography triggers appropriate layout. |
| FIX-13 | 180 mm by 120 mm multi-panel figure, inset, rotated labels, rich text, gradients, callouts, notes | Native publication preview and SVG/PDF retain physical dimensions, fonts, clips and vector marks; nearest-integer raster dimensions are 2126 by 1417 at 300 DPI and 4252 by 2835 at 600 DPI. |
| FIX-14 | Export while appending/correcting data and editing an annotation | Export uses one coherent captured revision and releases held resources afterward. |
| FIX-15 | Equivalent portable Rust, Python and WASM chart/update/action | Matching semantic domains, statistic outputs, stable targets and resulting state within explicit tolerances. |
| FIX-16 | ID > 2^53, precise timestamp, null column, disposed handle and WASM memory invalidation | No identity/time narrowing; valid null behavior; controlled lifetime errors. |
| FIX-17 | Custom histogram stat/geom extension and native-only painter | Shared scales/guides/targets work; portable extension succeeds when registered; unsupported export is explicit. |
| FIX-18 | Repeated resize/mount/unmount, unavailable font, tiny bounds and malformed spec | Bounded resources, recoverable diagnostics, last-valid-state behavior and no normal-input panics. |
| FIX-19 | [D3 axis parity matrix](../impl_plans/d3-axis-parity-plan.md#fix-19-acceptance-matrix): orientations, configuration/reset, scales, formatting, geometry, styling and transitions | AXIS-01–07 pass semantic/reference, actual binding and inspected native/publication evidence; preserve existing definitions through explicit profile/migration. |
| FIX-I01 | [D3 interpolation matrix](../impl_plans/d3-interpolate-parity-plan.md#fix-i01-acceptance-matrix): values, colors, splines/composition, CSS/SVG transforms, zoom, ownership and consumers | ITP-01–08 pass pinned reference and independent cases, actual Rust/Python/WASM runtime and applicable inspected native/publication outputs; WP-IP01–07 own delivery. |
| FIX-20 | [D3 scale parity catalog](../impl_plans/d3-scale-parity-plan.md): all scale factories/methods, knots, interpolation, distributions, calendar time, formatting, copies and updates | SCL-06–08 pass with independent expectations, pinned reference results, actual Rust/Python/WASM operations and inspected native/export guides; SP-01–07 own delivery. |
| FIX-H01 | [D3 hierarchy catalog](../impl_plans/d3-hierarchy-parity-plan.md#acceptance-catalog--fix-h01): constructors/node operations/layouts/tilers/helpers, history and integration | HIR-01–08 pass FIX-H01-A–H with exact topology/identity, pinned numerical comparisons, independent invariants, actual bindings, inspected outputs and bounded update/snapshot behavior; WP-H01–08 own delivery. |
| FIX-21 | [D3 scale-chromatic matrix](../impl_plans/d3-scale-chromatic-parity-plan.md#fix-21-acceptance-matrix): 76 exports, 218 arrays, evaluator boundaries, scale composition, guides, portability and updates | CHR-01–06 pass exact color/reference, actual runtime and inspected native/publication evidence; CP-01–05 own delivery. |
| FIX-C01 | [D3 color catalog](../impl_plans/d3-color-parity-plan.md#fix-c01-acceptance-matrix): parsing, constructors, spaces, manipulation, predicates, clamp, formatting and integration | COL-01–06 pass independent/reference cases, actual Rust/Python/WASM methods and inspected native/SVG/PDF/PNG artifacts; CLR-01–05 own delivery. |

FIX-S01–FIX-S09 are the required supplemental shape fixtures defined in the
[shape acceptance inventory](../impl_plans/d3-shape-parity-plan.md#acceptance-fixtures-and-evidence).
They cover output/sinks, lines/areas, curves, arcs/pies, radial/links, symbols, stacks,
custom protocols and integrated runtime/update/export proofs respectively. Existing
FIX-01–18 expectations remain unchanged.

FIX-P01–FIX-P06 are the required supplemental path fixtures in the
[path acceptance inventory](../impl_plans/d3-path-parity-plan.md#acceptance-fixtures-and-evidence).
They cover builder state, arcs, signed rectangles, serialization, safety/budgets and
actual consumers/portability. FIX-S01 consumes this shared foundation evidence.

FIX-GG00–FIX-GG19 are the required Phase 2 fixtures defined with their owning packages
in the [secondary plan](../impl_plans/phase-2-parity-implementation-plan.md#6-ggplot2-work-packages).
They cover the oracle, legend regression, grammar/stages, aesthetics, scales, guides,
statistics/positions, primitive/text/analytical geoms, facets, coordinates, themes/math,
geography, extension authoring, devices, integrated updates/hosts and certification.
Existing FIX expectations and historical acceptance remain unchanged.

## 15. Release gates and implementation handoff

| Gate | Meaning | Required evidence |
| --- | --- | --- |
| G0 | Architecture and capability decisions | Crate/dependency boundary checks; grammar/default ADRs; native primitive and publication font/export proofs; measured baseline protocol; target capability matrix. |
| G1 | End-to-end portable core | Simple native chart plus headless output; data/action atomicity; minimal Python/WASM fixture execution; foundational FIX-01/02/07/08/15/16. |
| G2 | Cartesian/publication alpha | Original 0.1.0 stats/geoms/scales/facets; themes, multi-panel composition, custom extension, and SVG/PDF/PNG; FIX-01 through FIX-07 plus FIX-12/13/17. Added 0.2.0 parity work is required by G4. |
| G3 | Interactive streaming beta | Full supported interaction actions, linking/editing, corrections/retention, safe scheduling and export under ingestion; FIX-08/09/10/11/14. |
| G-PATH | Complete D3 path parity | PTH-01–06 and FIX-P01–06 pass standalone construction/serialization, numeric replay, inspected native/headless and actual binding evidence through WP-P04; required before WP-S01 completion, WP-21 and G4. |
| G-AXIS | D3 axis capability parity | AXIS-01–07/FIX-19 pass reference, actual binding, native/publication and transition evidence under the declared compatibility profile; required before WP-21 and G4. |
| G-SHAPE | Complete D3 shape parity | SHP-01–10 and FIX-S01–09 pass through native/headless/actual bindings; every reference export/method has coverage or an explicit host-language adaptation, with no missing geometric capability. |
| G-SCALE | Complete D3 scale parity | SCL-06–08 and FIX-20 pass through all applicable native/headless/actual binding surfaces; SP-07 acceptance and method-level coverage with no missing defined scale capability. |
| G-CHROMATIC | Complete D3 scale-chromatic parity | CHR-01–06/FIX-21 pass all catalog, evaluator, composition, actual binding, update and inspected native/publication evidence; CP-05 precedes WP-21/22. |
| G-COLOR | Complete D3 color parity | COL-01–06/FIX-C01 pass standalone methods and integrated native/headless/actual binding evidence; CLR-05 precedes WP-21/22. |
| G-INTERPOLATE | Complete D3 interpolation parity | ITP-01–08/FIX-I01 and WP-IP07 pass the full export/configuration matrix, typed profile, actual bindings, shared consumers and applicable native/publication evidence. |
| G-HIERARCHY | Complete D3 hierarchy parity | HIR-01–08/FIX-H01-A–H pass standalone and integrated APIs, actual bindings and inspected native/publication artifacts, including stateful resquarify; WP-H08 precedes WP-21/22 acceptance. |
| G-GGPLOT | Complete ggplot2 capability profile | GG2-01–12 / FIX-GG00–19 and GG-19 accepted against the complete pinned capability/argument inventory, with actual host and inspected destination evidence. |
| G-PARITY | Phase 2 integrated capabilities | G3 and all eight D3 gates plus G-GGPLOT; required before final WP-21/22 acceptance. |
| G-AUTH | Complete primary authoring API | AUT-01–09 / FIX-AUTH00–09 pass complete capability reachability, external examples, actual host/export/binding behavior, compatibility and performance requalification for the refactor. Historical WP evidence is retained; this is additional acceptance for the refactored release. |
| G4 | Scoped production release | G-PARITY, all required fixtures, PERF-01 through PERF-05 and D3/ggplot2 supplemental workloads, WP-21–23 supported-platform QA, lifecycle/accessibility evidence, API/schema docs and no unresolved release-blocking requirements. A release incorporating the primary authoring refactor additionally requires G-AUTH. |

Gates are cumulative. Some proof fixtures run on a minimal feature subset at G1 and expand to the complete built-in portable feature set by G4. G1 does not claim production Python/browser products. The implementation plan assigns each requirement and fixture to work packages and gives AI developers a resumable execution protocol.

Reference-library behavior and the inspected dependency snapshots are documented in the migration plan. For binding/export implementation, consult the selected versions of [PyO3](https://pyo3.rs/), [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/), and candidate Rust rendering libraries such as [resvg](https://github.com/linebender/resvg) and [svg2pdf](https://github.com/typst/svg2pdf). These links guide implementation research; they do not override this contract or prove dependency suitability.

Version 0.2.0 also requires AXIS-01–07/FIX-19 before WP-21 and G4. Historical alpha
acceptance under 0.1.0 does not establish D3 axis parity. Axis transitions integrate with
G3 scheduling and coherent-export contracts before axis certification.
