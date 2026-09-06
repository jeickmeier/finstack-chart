# Rust-native GPUI Charts — Project Specification

Document version: 0.1.0  
Date: 6 September 2026  
Status: Initial project contract for implementation; no implementation or measured performance is implied.  
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
| Geometry and recipes | Points/scatter, lines, interval bars, grouped/mixed-sign stacked bars, area/ribbon, rules, text/annotations, cells/heatmaps, histogram, and candlestick/volume composition. |
| Statistics and positions | Identity, count, explicit binning, grouped summaries including quantiles, simple ordinary least-squares fit, stack/normalize/dodge, and deterministic jitter. |
| Coordinates and composition | Cartesian coordinates; multiple named scales; facet wrap and a basic row/column facet grid; shared/free axis policies; aligned panes and linked plots. |
| Scales | Linear, logarithmic, symmetric logarithmic, UTC/calendar time, band, point, ordinal/discrete color, and continuous color. |
| Design and export | Complete custom theme cascade, publication figure layout, SVG/PDF/PNG output, explicit font handling, and publication preview. |
| Live use | Atomic batched data changes, bounded retention and queues, streaming corrections, stable interaction state, and coherent export during ingestion. |
| Interaction | Inspection, navigation, selection, linked views, annotation editing, programmatic actions, keyboard behavior, and documented accessibility support. |
| Extensibility | Native custom statistics and geometry, explicit scale/coordinate capabilities, portable extension identifiers, and renderer capability reporting. |
| Portability proof | Minimal Python and WASM fixtures before API stabilization; core and headless use independent of GPUI. |

Candlestick recipes render supplied OHLC data; pricing, risk models, market feeds, portfolio analytics, and trading execution are application responsibilities. Recipes are examples of the general grammar.

**SCP-03 — Release boundaries.** Default first-class desktop target is macOS on Apple Silicon, subject to the selected GPUI release's measured capability report. Headless core tests MUST run on macOS and Linux, and portable core builds MUST run for the browser WASM target. Other GPUI OS/architecture combinations require their own capability and test evidence before being advertised. This is an initial support assumption that can be expanded without redesigning core.

Full Python packaging/distribution, a Python native viewer, interactive notebook widgets, a browser renderer/product, touch-specific browser interactions, WebGPU, geographic/polar/hierarchical/network charts, advanced fitting/density, complex morphing, full math typesetting, and press-specific PDF standards are future scope. Minimal binding proofs are required now. Required current features cannot be postponed merely by calling them extensions.

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

Dodge uses declared band-relative widths/order; missing groups preserve configured slots. Jitter declares data or display units and uses an explicit seed plus stable row/group keys; reordering input cannot move existing points. Defaults must be documented in recipes.

**GRA-06 — Geometries and recipes.** Required geoms consume prepared encodings and emit portable primitives plus semantic targets. Bar/area baselines are explicit, zero by default where valid. Log axes require a valid declared baseline for baseline-dependent geometry. Ribbon bounds with lower > upper are invalid. OHLC validation requires low <= open/close <= high; volume invalidity is reported independently. Gaps split line/area runs by default.

Default line interpolation is straight segments. Optional smoothing MUST document overshoot behavior; source values remain available for inspection. Candles have open/high/low/close semantic values and separate up/down styling. Histogram is a bin stat plus interval bars, rather than a separate chart engine.

**GRA-07 — Facets and groups.** Provide facet wrap and a basic row/column grid, explicit panel order, shared/free x/y scale policies, and stable panel keys. Empty panels follow an explicit keep/drop setting. A layer missing a facet field must explicitly broadcast to panels or name target panels. Shared scales train from compatible contributing panels; free scales use their own populations. Selection and provenance retain panel/group identity.

**GRA-08 — Transform reuse and extensions.** Named transform outputs form an acyclic revisioned graph and can feed several layers/charts. Cache by input revisions, operation ID/version, parameters, grouping/facet scope, and declared viewport dependence. Custom stats declare append/window/correction/full-recompute capabilities; these declarations are verified against batch results. Portable operation registration resolves known IDs and versions; unknown or native-only operations fail explicitly. Do not deserialize executable code or arbitrary dynamic plugins.

## 6. Scales, coordinates, and domains

**SCL-01 — Capabilities.** Continuous scale APIs expose mapping and, where valid, inversion; band/point scales expose category lookup and extents. Never pretend every scale has a numerical inverse. Cartesian coordinates are required; extension interfaces explicitly report inversion, clipping, and path-subdivision capabilities.

**SCL-02 — Domain precedence.** Resolve an explicit domain first; otherwise derive from eligible post-stat/post-position contributions. Apply the declared baseline/zero policy, then padding/nice policy only where enabled. Explicit domains are exact by default. A separate viewport determines the visible region. Hidden series remain in domain/stat training by default so legend toggles do not unexpectedly rescale; an explicit exclude-hidden policy recomputes affected domains.

For empty data, explicit domains are retained; absent domains use a documented neutral continuous domain [0,1] or an empty categorical domain, and display a no-data state. Constant numeric domains expand symmetrically using a scale-specific deterministic rule. Descending domains/ranges are valid. Tests MUST lock the expansion rules before the public alpha.

**SCL-03 — Numeric and color scales.** Linear, log, and symlog implementations MUST validate parameters, domain direction, finite projection, and round-trip behavior within stated numerical tolerance. Log requires a base > 1 and positive domain. Symlog has a positive linear threshold and documented forward/inverse functions. Continuous and discrete color scales have declared palettes, null styles, domain/clamp policy, and legend metadata. Color-only changes cannot change numeric domains.

**SCL-04 — Time.** UTC ticks use calendar interval logic. Timestamp units are explicit, and formatting never silently applies the machine's timezone. An explicit local-zone formatting option MAY use a supplied timezone implementation. Session-time mapping requires a supplied session calendar and a declared closed-session policy; it is a separate option from elapsed-time mapping. Test gaps, day/month/year boundaries, leap days, and any supported local DST conversion. Do not ship exchange calendars or imply market-calendar correctness.

**SCL-05 — Multiple scales and clipping.** Layers bind to named scale IDs. Independent scales and alternate-unit secondary axes are separate constructs. A secondary axis must be a valid one-to-one mapping over the represented domain. Clip to plot/panel bounds by default while allowing declared annotation overflow. Out-of-domain omission/clamping is explicit and does not retroactively change upstream stats. The painter, export, and hit tester use the same clips and transforms.

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

**SCN-04 — Shared presentation truth.** Painting, hit testing, focus navigation, tooltips, and animation MUST refer to the presented scene and its snapshot. Scales/clips or indexes from another revision cannot be mixed with visible geometry. Compatible opacity/geometry transitions are optional and must honor reduced motion, interruptions, and target identity.

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

The table below defines conceptual boundaries; concrete Rust method names and trait ergonomics are finalized in the first implementation phase. Production examples must compile against the selected API before the alpha API is frozen.

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

## 15. Release gates and implementation handoff

| Gate | Meaning | Required evidence |
| --- | --- | --- |
| G0 | Architecture and capability decisions | Crate/dependency boundary checks; grammar/default ADRs; native primitive and publication font/export proofs; measured baseline protocol; target capability matrix. |
| G1 | End-to-end portable core | Simple native chart plus headless output; data/action atomicity; minimal Python/WASM fixture execution; foundational FIX-01/02/07/08/15/16. |
| G2 | Cartesian/publication alpha | Required stats/geoms/scales/facets; themes, multi-panel composition, custom extension, and SVG/PDF/PNG; FIX-01 through FIX-07 plus FIX-12/13/17. |
| G3 | Interactive streaming beta | Full supported interaction actions, linking/editing, corrections/retention, safe scheduling and export under ingestion; FIX-08/09/10/11/14. |
| G4 | Scoped production release | All required fixtures, PERF-01 through PERF-05, supported-platform QA, lifecycle/accessibility evidence, API/schema docs and no unresolved release-blocking requirements. |

Gates are cumulative. Some proof fixtures run on a minimal feature subset at G1 and expand to the complete built-in portable feature set by G4. G1 does not claim production Python/browser products. The implementation plan assigns each requirement and fixture to work packages and gives AI developers a resumable execution protocol.

Reference-library behavior and the inspected dependency snapshots are documented in the migration plan. For binding/export implementation, consult the selected versions of [PyO3](https://pyo3.rs/), [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/), and candidate Rust rendering libraries such as [resvg](https://github.com/linebender/resvg) and [svg2pdf](https://github.com/typst/svg2pdf). These links guide implementation research; they do not override this contract or prove dependency suitability.
