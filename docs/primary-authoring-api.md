# Primary authoring contract and capability register

AP-00 / FIX-AUTH00, 7 September 2026. Baseline:
`b631f0e6d7e41722b5774433d616f704234157d7`, plus the owner's authoring-plan edits.
Authority: [AUT-01–09](spec/gpui-charts-specification.md),
[ADR-013](adr/013-primary-authoring-api.md),
[AP-00–09](impl_plans/primary-authoring-api-plan.md).
This register records implementation coverage, not original-WP or parity certification.

## Public contract

`chart_core::prelude` exposes `Data`, `plot`, `aes`, layer/component constructors and
`Chart`. `Data::columns().column(name, values).build()` and
`Data::rows(rows).field(name, accessor).build()` preserve owned values, nullable masks,
exact signed/unsigned integers and explicit timestamp unit/timezone metadata.
`plot(data).aes(...).layer(...).build()` resolves into an immutable `Plot` containing
one normalized definition, coherent source handles and immutable registrations.
Builders own temporary unresolved names; there is no second stored grammar AST.

Use named fields by default, typed handles for advanced reuse. Distinct data/layers
receive checked fresh identities; clones and edits preserve them. Primary axes retain
reserved x/y identities. Names must be nonempty, bounded and unambiguous in their
owner; a missing field error includes the dataset, layer and aesthetic. Default row
keys are durable allocated identities; explicit keys are retained exactly. A keyless
replacement allocates new keys. Nulls never become zeroes; exact integers never pass
through f64 during materialization. Callbacks execute once per batch, not per frame.

The initial supported semantic profile is `LibraryV1`, preserving explicit grouping,
point-radius/line-width units and current bin closure/stat-before-axis ordering. This
identity is retained by the Plot/primary interchange envelope. Future profiles are
explicit additions by their semantic packages; selecting an unavailable profile must
not silently approximate it. Facet catalogs infer first-seen exact values at build,
then default to fixed/reject until explicitly extended/rebuilt. Color catalog behavior
continues to follow the shared scale engine. Cloning a Plot does not create a new
identity or advance its revision; an effective immutable edit does.

`.build()` checks schema/stages/registrations/parameters without executing statistics,
geometry, shaping or layout. `Chart` owns typed runtime operations and defaults to one
DataStore/IngestionQueue; an explicit external-source constructor borrows committed
immutable snapshots with no writable store copy. Independent Chart instances have
independent reducer/view state. Definition-only edits are revision-fenced, use current
runtime data, and never replace it with an embedded older Plot snapshot. Data changes
remain separate atomic transactions with existing receipts/replay/retention contracts.

UI and worker compilers can be distinct. GPUI owns tasks, frame acknowledgement,
window and native factories. Export owns reusable supplied fonts, immutable
FigureRequest acquisition and bounded ExportQueue/ExportJob execution. Live export
selects Presented (default) or Current independently of visible/full-domain and
interaction inclusion. Presented needs an acknowledged frame; Current does not.
Native-only registrations remain valid natively; portable and export boundaries apply
appropriate capability checks without serializing implementations.

Titles, subtitles, axis labels, legends, captions/notes, panel letters and insets have
separate components. `labels()` is only x/y annotations; `callout()` adds a leader.
Grouping and appearance are independently mapped, inherited and overridden, as in
[plan section 3.5](impl_plans/primary-authoring-api-plan.md#35-grouping-and-independent-visual-mappings).
Scale builders choose appearances, legend builders choose guide presentation, and
constants live on the layer/style builder. Host hooks are destination configuration.
Commands, queries, receipts and standalone helpers are not turned into plot builders.

## Migration and ownership

Workspace/package baseline is unpublished 0.1.0; portable chart/data/state/transaction
schemas remain version 1. The additive primary API is developed in the current
workspace; 0.2.0 is the intended migration release, without publication or version
bump in AP-00. All currently public paths remain available through forwarding or
specialist modules. A removal is no earlier than 0.3.0, with deprecation evidence and
consumer migration first; no removal is required to close G-AUTH. A primary authoring
interchange envelope may version its own names/profile independently of existing wire
payloads. Existing payload field names/defaults must not change merely due to Rust moves.

Primary APIs: plot/data/component builders, retained Chart, host/export destinations.
Specialist APIs: immutable normalized inspection, schema/resource/registry descriptors,
custom traits, standalone scales/color/geometry and typed action/query values.
Compatibility APIs: raw definition/Session constructors and legacy tutorial entrypoints.
Internal owners: compiler caches, prepared tables, scene lowering, layout internals,
GPUI scheduling mechanics. Their currently public paths are preserved during migration.

The module ownership table below covers every baseline public source module and its
options; entries listing a type include all of its existing fields/variants/methods.
The option inventory is the live source, not a copied definition that can drift.
Each row owns required primary usage and behavior evidence. The completed baseline
rows below are qualified by the linked evidence groups; existing low-level tests remain
independent contract controls. Future semantic rows are separate and remain gated.
Rows close only with recorded commands, actual outcomes and artifact evidence in the
[ledger](implementation-status.md). A future public option belongs to an existing row
or a new row in its own feature package, including applicable host proofs.

| ID / source owner (all existing options) | Primary route / baseline details | Package / evidence case | Status |
| --- | --- | --- | --- |
| A-DATA — core data/{schema,columns,snapshot,retention} | Data columns/rows; seven kinds, metadata/formatted/nulls, keys, schema/dataset handles; immutable row inspection and retention values | AP-02/05; FIX-AUTH02 data_exact and rows | QUALIFIED ([E1](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-IDENTITY — core identity, transaction | Checked identity allocation; handles/transactions; all mutation variants, revision/replay outcomes, count/event-time retention and watermark | AP-02/05; FIX-AUTH05 transactions | QUALIFIED ([E1](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-MAPPING — grammar/{definition,typed,colors} SourceAes/BinAes/StatAes | aes source or generated stage; all coordinates/bounds/size/group/color, per-layer data/inheritance, stable stage typing | AP-03; FIX-AUTH03 stages | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-GEOM — grammar/definition Geom, Layer, Style | points/line/area/ribbon/bars/ohlc/rule/rectangle, constant style, gaps/order/baselines/candle colors/clip | AP-03; FIX-AUTH03 marks | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-RECIPE — grammar/definition recipe constructors | histogram/volume/cells/fit compose ordinary components; no independent kernel | AP-02/03; FIX-AUTH02 histogram | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-STAT — grammar/{definition,statistical_types,statistics,stats,incremental_bins} | identity/bin/count/summary/fit; bin edges/count/outliers/stat space; grouping/scope, summary quantiles, OLS grid; generated mappings and update metrics | AP-03; FIX-AUTH03 statistics | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-POSITION — grammar/{statistical_types,positions} | identity/stack/dodge/jitter with normalization/order/width/seed/units | AP-03; FIX-AUTH03 positions | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-TRANSFORM — grammar/definition SourceFilter, NumericTransform, TransformDefinition | Named transforms, filters, affine stat space, DAG references, scope/panel targets | AP-03; FIX-AUTH03 transform_dag | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-FACET — grammar/facets | wrap/grid, ordered catalog, keep/drop, free axes, gap, guides; explicit broadcast/panel targeting and stat scope | AP-04; FIX-AUTH04 facets | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-SCALE — scales/{linear,nonlinear,band,utc,session,mod} | Direct scale utilities plus builder domains/baselines/padding/nice/clamp; log/symlog, point/band catalogs, UTC precision, supplied session calendars | AP-03/04; FIX-AUTH03 scales | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-AXIS — layout/types AxisSpec/AxisScale/CustomGuideTick | x_axis/y_axis and named axes, sides, independent viewports/ranges, guide-only affine secondary scales, ticks/format/title/rotation/visibility | AP-03/04; FIX-AUTH04 axes | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-COLOR — scales/color and grammar/colors | Color scale palettes/domain/null/outside policies; source/generated/group channels, shared identities | AP-03; FIX-AUTH03 color | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-GUIDE — layout/{engine,theme} and scales/color legends | Automatic guides and separate legend overrides; collected/per-panel legends; retain known baseline limitations until semantic owner fixes them | AP-03/04; FIX-AUTH04 legends | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-THEME — theme and layout/theme | Named Editorial/Terminal/Grayscale, complete patch tokens, plot/layer cascade and destination/interaction overrides | AP-04; FIX-AUTH04 theme | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-TEXT — typography and chart-text | Rich text/runs, font descriptor/size/weight/language/direction/rotation/line spacing, number notation/locale; explicit destination shaping | AP-04/06; FIX-AUTH04 typography | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-FIGURE — composition and layout/composition | title/subtitle/caption/source_note/footnote/panel_letter; labels/callout anchors/priority/collision/offset/overflow; insets with shared prepared layers | AP-04; FIX-AUTH04 composition | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-LAYOUT — layout/{types,coordinates,engine,mod} | Cartesian named scales, destination geometry/units/font, padding/ticks/categories/vertices/limits/minimum dimensions; immutable resolved inspection | AP-04/06; FIX-AUTH04 layout | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-STATE — state (including reducer/types/windows) | Chart actions/controlled state, hover/focus/pin/selection/history/visibility/view/follow/freeze/annotations/gesture/disposal; typed requests/events | AP-01/05; FIX-AUTH05 state | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-INSPECT — inspection, accessibility, provenance | Retained scene queries/indexes, hit options, selection regions, accessible pages/targets, exact source/aggregate/model provenance | AP-05; FIX-AUTH05 inspection | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-NAV — navigation | Chart navigation using presented/pinned named axes, all boundary/navigation operations, exact windows | AP-05; FIX-AUTH05 navigation | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-EDIT — editing | annotation_edit constraints and snapping/order; preview/nudge/apply/undo on stable handles | AP-05; FIX-AUTH05 editing | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-LINK — linking | link source/destination axes, selection, missing policy, origin/revision echo fencing and source identity | AP-05; FIX-AUTH05 linking | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-STREAM — ingestion, transaction | stream_options; queue limits/backpressure/drop/status, checked atomic queue commits and reconciliations | AP-01/05; FIX-AUTH05 ingestion | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-SCHEDULE — scheduling and grammar/compiler | render_options/admission policy, compatibility stamps/job tokens, one active/newest pending, cache return and stale completion | AP-01/05/06; FIX-AUTH05 workers | QUALIFIED ([E5](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-DENSE — dense | render_options line/candle bucket width/volume/max columns, honest representations and metrics | AP-05; FIX-AUTH05 dense | QUALIFIED ([E5](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-EXTENSION — grammar/{extensions,geometry_extensions} | Ordinary registered stat/geom slots, complete descriptors/version/schema/parameters/resource/capability validation, native-only routes | AP-01/03/04; FIX-AUTH03 extensions | QUALIFIED ([E2](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-FOUNDATION — geometry, scene, services, diagnostic, limits, grammar/prepared | Direct immutable typed values/helpers; geometry/resource/precision/capability limits on owning builder; retained structured diagnostics/inspection | AP-02/04/06; FIX-AUTH02 errors | QUALIFIED ([E1](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-RUNTIME — portable/{session,wire,input,stream,encoding,mod} | Typed Chart owns execution; portable Session forwards decode/encode boundaries; strict versions/exact values/state/receipts preserved | AP-01/07; FIX-AUTH01 legacy | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-NATIVE — gpui-charts native/view/paint and extensions | Retained native builder/input, font/painter resources, tooltip and controls, command/event/accessibility hooks, scheduling and presentation | AP-06; FIX-AUTH06 native | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-KIT — gpui-charts-kit | Optional theme/context adapter over same native destination | AP-06; FIX-AUTH06 kit | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-EXPORT — chart-export profile/fonts/request/snapshot/encode/jobs/svg/portable | Output destination and export_options; all page/view/text/DPI/background/precision/budget controls, capture bases/interactions, manifests/cancel/queue | AP-06; FIX-AUTH06 capture | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-PYTHON — chart-python | Host-native data/plot/components/runtime/results, exact ints/nulls, retained resources, errors/properties, detachment/disposal | AP-07; FIX-AUTH07 python | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-WASM — chart-wasm | Same core API via host-native builders, bigint/exact adapters, nullable fields, errors/properties and memory/disposal | AP-07; FIX-AUTH07 wasm | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-CONSUMERS — examples, docs, scripts, schemas | Migrate first chart/overlay/color/histogram/facet/export/invalid-field workflows, gallery and custom extension; compatibility schemas/declarations | AP-08; FIX-AUTH08 consumers | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-REQUALIFY — tests/fixtures/benchmarks | Independent semantics, bindings/native/export artifacts, supported platforms, PERF-01–05 sustained tests and no per-frame normalization | AP-09; FIX-AUTH09 | final measurement in progress |

## Future capability rows

All D3/GG inventories referenced by the main plan remain authoritative. Their source
kernels are not delivered by this refactor. The mapped shape/linetype/fill/alpha and
size/linewidth separation requires GG-03, inferred grouping GG-02, manual/reference
scale policies GG-04 and full guide composition GG-05. Mapped labels/math and advanced
coordinates require their respective GG packages. All other D3 color/interpolate/path/
shape/hierarchy/scale families integrate in A-SCALE/A-COLOR/A-GEOM/A-FOUNDATION as they
land. Every future package extends these primary builders, examples and applicable
bindings in that package; unavailable options reject rather than approximate behavior.

## AP-00 evidence and handoff

Baseline source inspection includes the nine manifests (`cargo metadata --no-deps
--format-version 1 --locked`), exports, all primary option owners above, seven usability
workflows and APR-01–04 lifetime/validation corrections. Baseline platforms are the
ledger's historical original-scope macOS/Linux/WASM evidence, not fresh qualification.
API spellings in executable examples become authoritative when their owning slice
passes; the plan's unsupported future sketches remain visibly prospective.

The acyclic handoff is AP-00 → 01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09.
Within AP-01, extract typed ownership and structural validation before migrating JSON.
Within AP-02, implement Data/Plot then native/export bridges; register those concrete
examples before proceeding through component families. Complete delivered baseline
rows before claiming G-AUTH; unavailable parity rows remain explicitly gated.

## Primary host proof

`mise run primary-authoring-proof` builds the actual typed Rust, Python and WASM authors,
compares independent source/statistical/scene fixtures and checks Python/TypeScript usage.
The runner requires installed mypy, TypeScript and wasm-bindgen CLI 0.2.128; use `TSC_JS`
and `WASM_BINDGEN` for task-local tools. See the ledger for the executed environment and
the qualified baseline and retained release limitations. No package publication is performed.

Python primary syntax lives in `packages/python/finstack_chart`; the generated native
extension must be on its import path. WASM `authoring.cjs` and `authoring.d.cts` sit beside
the generated `chart_wasm.js` module. This is an executable proof adapter, not a new
wheel/browser distribution product. Python uses snake_case; JavaScript also provides
camelCase aliases. Component families are separate public types, and a definition edit
has a separate declared interface from a new plot draft.

All handles support explicit `dispose()`. Captured requests and annotation editors own
their original inputs and may survive runtime/output disposal. JavaScript also exposes
wasm-bindgen `free()` to release the wrapper allocation; deterministic loops must free
intermediate persistent builder handles as well as final plot/runtime handles. Ordinary
unreferenced builders use wasm-bindgen finalization, whose timing is controlled by the
JavaScript host. Python's Rust execution detaches from the interpreter; row accessors
run once during Python materialization and are never retained by a chart.

Registered statistics can bind a source name or owned field with
`custom_stat(...).field_parameter("input", field)`. The owning extension's parameter
object receives the canonical checked numeric mapping at build; callers need not
encode source IDs. The compiled example provides `density_histogram` and
`chamfered_bars` in Rust and the proof-host `examples` modules. Registry installation
and registry-aware loading remain explicit host operations; native-only geometry is
valid for native authoring and rejects at portable/headless execution boundaries.
