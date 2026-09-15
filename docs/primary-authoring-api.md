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
| A-KIT — withdrawn Kit crate | No first-party Kit adapter crate; hosts map Kit tokens into `ThemePatch` / `ChartInput::from_plot` | AP-06; [ADR-023](adr/023-withdraw-optional-kit-crate.md) | WITHDRAWN (E4 Kit gallery observation remains historical) |
| A-EXPORT — chart-export profile/fonts/request/snapshot/encode/jobs/svg/portable | Output destination and export_options; all page/view/text/DPI/background/precision/budget controls, capture bases/interactions, manifests/cancel/queue | AP-06; FIX-AUTH06 capture | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-PYTHON — chart-python | Host-native data/plot/components/runtime/results, exact ints/nulls, retained resources, errors/properties, detachment/disposal | AP-07; FIX-AUTH07 python | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-WASM — chart-wasm | Same core API via host-native builders, bigint/exact adapters, nullable fields, errors/properties and memory/disposal | AP-07; FIX-AUTH07 wasm | QUALIFIED ([E3](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-CONSUMERS — examples, docs, scripts, schemas | Migrate first chart/overlay/color/histogram/facet/export/invalid-field workflows, gallery and custom extension; compatibility schemas/declarations | AP-08; FIX-AUTH08 consumers | QUALIFIED ([E4](evidence/primary-authoring-completion-2026-09-08.md)) |
| A-REQUALIFY — tests/fixtures/benchmarks | Independent semantics, bindings/native/export artifacts, supported platforms, PERF-01–05 sustained tests and no per-frame normalization | AP-09; FIX-AUTH09 | final measurement in progress |

## Future capability rows

All D3/GG inventories referenced by the main plan remain authoritative. Their source
kernels are not delivered by this refactor. GG-03 extends the same builders with mapped
shape/linetype/fill/alpha and size/linewidth separation; inferred grouping is GG-02,
manual/reference scale policies GG-04 and full guide composition GG-05. Mapped labels/math and advanced
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

## Checked Cartesian shape generators

Rust `chart_core::shape::{Line, Area, CurveSpec}` and Python/WASM `ShapeLine` and
`ShapeArea` expose all 20 pinned D3 curve factories. Materialized controls use
`{"Column": 0}` or `{"Constant": 1.0}`, a boolean `defined` value or an exact-length
boolean mask, and `{"kind": "CatmullRom", "alpha": 0.5}` style curve descriptors.
Area controls are `x0`, `y0`, optional `x1` and optional `y1`. `generate(rows)` returns
an independently owned `Path`; `boundary("X0" | "X1" | "Y0" | "Y1")` derives an
independent line generator. Three SVG digits are the default; `None`/`null` retains
unrounded text without changing numeric geometry. Bundle is line-only.

Use `shape_line().curve(...)` or `shape_area().curve(...)` for source-aware charts.
Both retain authored order by default. General areas require explicit x/y lower
and x2/y2 upper mappings and permit crossed boundaries. Curves act after scale
projection, with source anchors retained separately from Bézier controls. Existing
`line`, `area` and `ribbon` defaults keep their established meanings. Native Rust
accessors and curve protocols are available directly; portable custom registrations
use the registered protocols described below. See [ADR-020](adr/020-shape-generators-and-curve-protocols.md)
for finite-input, clipping, precision, resource and compatibility boundaries.

### Arcs and pie layouts

`shape::Arc` and the owned `ShapeArc` host class generate circular/annular sectors
with datum radii, clockwise-from-twelve angles, optional constants, corners, padding,
centroids and shared numeric paths. Missing required datum fields diagnose unless
constants replace them. The centroid is the midpoint of the center line, not the
area centroid. Native `generate_by`/`centroid_by` resolve an `ArcParameters` value;
materialized hosts use `ArcDatum` fields and a checked `ShapeArcConfig`.

`shape::Pie` and `ShapePie.layout(data, values)` retain owned source data, original
value and sorted index while returning slices in input order. Nonpositive values
receive zero angular weight. The default order is descending value; `Input` and
`ValuesAscending` are portable alternatives. Native datum/value comparators and
whole-input angle accessors use the same layout. Registered portable custom
comparators use the same core layout through `ShapeRegistry`. Use strings for exact large identifiers in arbitrary
WASM JSON metadata; finite numeric weights remain binary64.

`shape_arc()` and `shape_pie()` are chart layers with centers mapped through x/y
(default zero) and radii in destination units (default outer radius 40, inner zero).
`shape_value(channel, source)` supplies an unnormalized named parameter; it accepts
constants, field names/handles, source expressions and explicit statistical fields.
`numeric_scale` supplies the same parameter through a chosen scale. Channels are
`InnerRadius`, `OuterRadius`, `CornerRadius`, `PadRadius`, and, for standalone arc
marks, `StartAngle`, `EndAngle`, `PadAngle`. Pie layers require `PieValue`; their
`pie_angles` configuration owns start/end/padding for the whole layout. `arc_parameters`
sets constant defaults, and pie layout replaces its datum angles.

Slice color does not implicitly split pie populations. Explicit row groups normally
produce separate pies; `pie_grouped(false)` combines the current layer/panel. This
allows a count statistic grouped by category to feed one pie without collapsing its
aggregate targets. Facets still partition the panel population. Source/aggregate
identity belongs to each wedge; the center-line centroid supplies a focus location,
and the actual filled path supplies containment, including annular holes and clips.
Display radii do not train data-axis domains. The layers use definition version seven
and the existing scene version-three shape path.

For example, `shape_pie().shape_value(PieValue, "weight")` authors weighted source
slices. `shape_pie().stat(count().group("category")).after_stat(stat_aes().x(0.).y(0.))
.pie_grouped(false).shape_value(PieValue, StatField::Count)` feeds generated counts
through the same engine; callers import the corresponding enum variants in Rust.

### Area-sized symbols

Use `shape_symbol()` (Python/Rust) or `shapeSymbol()` (JavaScript) for D3-compatible
area and stroke sizes. `points()` retains its legacy radius behavior. The new layer
supports `symbol_kind`, `symbol_size` and `symbol_paint` (`Auto`, `Fill`, `Stroke`),
with camelCase JavaScript equivalents. All thirteen built-in kinds and the `X`
alias are available. `Auto` fills the seven filled types and strokes the six
additional stroke-oriented types; explicit `Stroke` supports a stroked circle.

```python
data = Data.columns({"type": ["a", "b", "c"], "area": [0., 1., 2.]})
layer = (shape_symbol()
    .symbol_types(data.field("type"), ["a", "b", "c"], ["Circle", "Square", "Plus"])
    .numeric_scale("AreaSize", data.field("area"),
                   StandaloneScale("linear", domain=[0., 2.], range=[16., 256.]))
    .symbol_title("Type")
    .symbol_size_guide("Input", [0., 1., 2.]))
```

Size guide values are input-domain samples: the example produces areas 16, 136 and
256 for both marks and guide glyphs. `shape_value("AreaSize", field_or_expression)`
uses the identity numeric mapping; generated statistics can use the statistical
input descriptor. `symbol_groups(domain, palette)` maps prepared group labels.
Unknown type labels omit marks unless `symbol_missing(kind)` supplies a fallback.
Zero size produces no chart ink or target; invalid sizes and filled open symbols
raise diagnostics. Coordinates position the center through the selected axes;
sizes remain in destination units.

`ShapeSymbol({"kind": "Star", "size": 64.})` is the reusable standalone generator.
Its `generate()` result owns an independent shared `Path`; `palettes()` exposes
the ordered fill and stroke palettes. Custom native `SymbolDraw` uses the Rust
checked path protocol; portable custom registration uses `ShapeRegistry` below.

### Reference stack orders and offsets

`ShapeStack` is the reusable layout owner. `layout(values)` accepts a rectangular
sample-by-series matrix; `layout(data, values)` retains separate original metadata.
Keys determine output series order, while each series `index` records its stacking
rank. Points retain `data`, `y0` and `y1`. Missing values are `None`/`null`, with an
explicit `Gap`, `Zero` or `Error` policy. The six orders are `None`, `Reverse`,
`Ascending`, `Descending`, `Appearance` and `InsideOut`; an `Explicit` permutation
addresses configured key indexes. Offsets are `None`, `Expand`, `Diverging`,
`Silhouette` and `Wiggle`. `Expand` divides by the signed column sum: heights 2 and
-1 yield intervals [0,2] and [2,1]. Legacy `stack(...).normalize(...)` continues to
normalize positive and negative sides separately.

```python
layout = ShapeStack({"keys": ["a", "b"], "order": "InsideOut", "offset": "Wiggle"})
series = layout.layout([[1., 2.], [3., None], [2., 4.]])

position = (shape_stack(["a", "b"])
    .stack_order("InsideOut").stack_offset("Wiggle").stack_missing("Zero"))
chart = (plot(data)
    .aes(aes().x("sample").x2("sample").y("height").y2(0.).group("series"))
    .layer(shape_area().position(position)).build())
```

The chart adapter accepts tidy bars, rectangles and shape areas. Declare every
possible group, an explicit zero baseline, and one row per group/sample; use a
statistic to aggregate duplicates. Samples sort by x (then x2 for intervals),
independently of source insertion order. Area boundaries address the same x sample.
`Gap` splits runs; `Zero` inserts boundary geometry without fabricating a source or
focus target. Null and absent cells never become observations. A `connect_gaps`
override conflicts with the explicit stack missing policy and diagnoses.

Use a declared color domain when group colors must remain stable across deletion
and fresh-batch reconstruction. Facets independently stack their scoped populations.
The position allows zero-preserving numeric scale-stage heights, and Expand produces
dimensionless output. Publication keeps its existing point-precision limits; a
huge off-scale value can fail publication even when its stack arithmetic is valid.

Rust uses `shape::Stack` plus `shape_stack(...).stack_order(...).stack_offset(...)`;
JavaScript uses `ShapeStack` plus `shapeStack(...).stackOrder(...).stackOffset(...)`.
JavaScript BigInt metadata becomes exact decimal strings on the portable wire,
including nested values; Python integer metadata stays exact. Native `StackOrdering`
and `StackOffsetting` protocols execute within explicit series, cell and work limits.
Registered portable custom operations use `ShapeRegistry` below.

### Registered shape protocols

`ShapeOperation` carries `operation: {id, version}` and bounded JSON `parameters`.
`ExtensionRegistry::register_shape` installs trusted `CustomShape` implementations;
its five protocol families reuse the native curve, symbol, pie-comparison and stack
interfaces. See [ADR-020](adr/020-shape-generators-and-curve-protocols.md) for bounds,
chart comparator records, native-only behavior and the wire-v9 migration.

Python `ShapeRegistry.example()` and JavaScript `ShapeRegistry.example()` install
the external example protocols in proof builds. `ShapeRegistry()` constructs an empty
registry. `registry.selection(operation, family)` validates and returns a portable
selection. Registries and generated paths have independent copy/disposal lifetimes.

```python
registry = ShapeRegistry.example()
shift = {"operation": {"id": "example.shift_curve", "version": "1"},
         "parameters": {"amount": 5}}
path = ShapeLine().generate_registered([[0., 1.], [2., 3.]], registry, shift)
layer = shape_line().shape_protocol("Curve", shift)
p = plot(data).with_shape_registry(registry).aes(aes().x("x").y("y")).layer(layer).build()
restored = Plot.from_json(p.to_json(), registry)
```

JavaScript uses `generateRegistered`, `shapeProtocol`, `withShapeRegistry` and
`Plot.fromJson`. Symbols take `(registry, selection)`; pie layout takes
`(data, values, registry, selection)`; stack layout takes
`(data, values, registry, order=None, offset=None)` with independent optional selections
(`null` in JavaScript). Cartesian/radial lines and areas and generic Cartesian links
accept registered curves. Radial-tangent links and arcs retain their fixed geometry
algorithms. Choosing a builtin layer curve/symbol/pie order replaces the corresponding
custom selection. Unknown versions, incompatible families, invalid outputs and
native-only portable operations fail explicitly. Actual consumers are in
`scripts/bindings/shape_custom.*` and `shape_custom_updates.*`.

Generated curve strokes now honor ordinary theme/layer `dashes` tokens. Patterns
contain an even number of positive on/off lengths in destination units. SVG/PDF
retain vector dash styling; native lowering is bounded at its display tolerance.
Filled areas and source anchors remain intact. Containment follows dashed ink,
including gap misses, while keyboard targets retain the original observations.
Nonempty retained path dashes require scene wire version four; solid path scenes
retain their previous versions. Definition capability versions are unchanged.

The complete shape surface is qualified under the finite typed d3-shape 3.2.0 profile;
see [the per-item acceptance evidence](evidence/phase-2-shape-acceptance-2026-09-09.md)
for exact supported contracts, destination tolerances and remaining release/performance
work. Native callbacks require explicit portable registrations for wire transport.

### Positional provider registrations

A positional provider is installed native code with a versioned identity and bounded
parameters. One resolved mapping serves its marks and every independent guide. It may
supply ticks, formatting, bands and an inverse; an inverse is not required. The native
implementation uses `CustomScale`/`PositionalScale` and the same `ExtensionRegistry`
that owns other extensions. See the [external example](../examples/custom-extension/src/scales.rs)
and [provider contract](extension-contract.md#registered-positional-scales-axis-01).

```python
registry = ExtensionRegistry.example()  # proof-enabled build; installs known Rust code
p = (plot(data).with_registry(registry)
     .aes(aes().x("x").y("y")).layer(points())
     .x_axis(x_axis().coordinate_scale(
         scale_registered("example.fold", 1, {"limit": 10.0})))
     .guide(axis_guide("top", "x").side("Top"))
     .build())
restored = Plot.from_json(p.to_json(), registry)
```

Rust uses `.extensions(registry)` with the same scale selector; JavaScript provides
`withRegistry`, `coordinateScale`, `scaleRegistered` and `axisGuide`. The host
`ExtensionRegistry` name is an alias of the existing registry class, and
`with_shape_registry` remains supported. A registry copy and a captured request retain
their installed implementations after the original owner is disposed.

Provider definitions use version 10. Loading requires an explicitly supplied registry;
JSON never installs code. Native-only providers reject portable serialization and
headless publication. Under the ggplot profile, `coordinate_scale` is required because
a provider has no implicit before-statistics transformation. Generic provider pan/zoom
rejects without an explicit navigation metric, even when forward positions are numeric.
The [qualification report](evidence/phase-2-axis-provider-2026-09-09.md) distinguishes
this provider/identity scope from the still-open D3 guide profile and axis gate.

### Independent guide ticks (WP-AX02)

Default axes and additional guides accept `guide_profile`, `tick_arguments`,
`tick_values` and `tick_format` (camelCase in JavaScript). The default profile is
`LibraryV1`; opt into `D3_3_0_0` for the shared D3 tick policies and complete selected
order, repeated labels and empty labels. This selects presentation policy over the
chosen scale; complete D3 geometry remains WP-AX03.

Arguments, values and formatter reset independently with `None` / `null`. An empty
explicit value list selects no ticks without automatic enumeration. Values retain
numeric, category or exact timestamp types. Formatters are explicit labels, a shared
numeric/calendar description, or a registered semantic formatter. Register native
code explicitly before loading its versioned reference. The same captured registry
survives author/registry disposal; native-only operations reject portable/headless use.

New controls use wire version 11, while plots with older capabilities keep their
existing wire versions. `Frame.guides()` returns the coherent guide specs, exact
selected values, order, labels and positions from the immutable frame, with nested
facet/inset scopes. See [ADR-021](adr/021-independent-axis-guides.md) for bounds and
unsupported legacy session/secondary combinations and the
[executable Python proof](../scripts/bindings/axis_ticks.py) and
[JavaScript proof](../scripts/bindings/axis_ticks.cjs) for complete examples.

## Registered interpolation (WP-IP06)

Explicit installed Rust factories can be used from each primary host. Python:

```python
registry = ExtensionRegistry.example()  # proof-enabled build only
factory = registered_interpolation(
    registry, "example.interpolation", 1, {"mode": "SquaredNumber"}
)
samples = piecewise(factory, [0., 100., 200.]).quantize(5)
scale = StandaloneScale("linear", registry=registry, factory=factory,
                        domain=[0., 10.], range=[0., 100.])
# samples == [0., 25., 100., 125., 200.]; scale.map(5.) == 25.
```

JavaScript uses `registeredInterpolation(registry, id, version, parameters)` and
`new StandaloneScale('linear', {registry, factory, domain: [0, 10], range: [0, 100]})`.
Both `Interpolator.from_json`/`fromJson` and `StandaloneScale.from_json`/`fromJson`
accept a registry for loading registered portable definitions. JavaScript operation
versions accept exact BigInt/decimal strings or safe integer Numbers; Python accepts
integer/decimal strings. Registrations are installed Rust code, never arbitrary host
callbacks. Factory `copy`/`dispose` owns an independent registry lifetime; already
prepared consumers survive disposal. Standalone scale construction takes the registry
explicitly, and later `configure`/`nice` operations retain its snapshot.

Registered interpolation/scales use envelope version 2 and registered mapped charts
version 12. Native-only factories reject serialization and headless export. Builtin
interpolation wire/API defaults remain unchanged. The
[three-host proof](../scripts/bindings/interpolation_integration.py) exercises registered
floating color and size scales, legends/themes, explicit transform/zoom frames, exact
keys, updates and retained publication. Axis transition lifecycle certification remains
with WP-AX05/06; this API supplies explicit samples without scheduling animation.

## Positional guide geometry and components

Axis and independent guide builders share `guide_geometry` / `guideGeometry` and
`guide_components` / `guideComponents`. Geometry supports combined `tick_size`, separate
inner/outer sizes, padding and an optional offset; `None`/`null` resets inherited controls.
D3 profiles preserve semantic ticks by default. `GuideGeometry.labels` explicitly selects
`Preserve`, `HideLabels` or `ThinTicks`; clipping of inward ticks is separate from cell
overflow. `layout_options().device_scale(...)` supplies the output-unit offset policy.
Native uses the window scale factor; PNG DPI does not silently change guide geometry.

`GuideComponents` has `domain`, `ticks`, `labels` and `per_tick` fields. Line options are
`visible`, `color`, `width` and `dashes`; label options are `visible`, `color`, `font_size`,
`typography` and `rotation`. Typography is the existing portable RichRun record with
supplied font/fallback/weight/size policy; its text is replaced by the selected label.
Per-tick overrides specify an original selection `index`, optional combined `visible`,
and independent `line` / `label` overrides. These settings never change the mapped data
marks. Invalid dimensions, dashes, font policy or repeated style indices fail normally.

Geometry requires wire version 13 and component styles require version 14. Older
unconfigured definitions retain their earlier versions. `Frame.scene()` exposes
optional guide component metadata and `Frame.guides()` retains complete selected values
and labels, including hidden components. See [ADR-021](adr/021-independent-axis-guides.md)
and the [component example](../examples/common/axis_component_fixtures.rs).

## Axis transition sampling and displayed capture

Native `ChartInput::guide_transition(Duration)` opts into timed guides; the default is
immediate presentation. `ChartView::set_guide_transition` updates that duration. Native
reduced motion completes the transition immediately while retaining tick identities.

Rust and Python use `target.guide_transition(previous)` to obtain an owned
`FigureTransition`; WASM uses `target.guideTransition(previous)`. All three expose
`sample(fraction)` for finite fractions in `[0, 1]`. The result is an ordinary owned
figure supporting scene, guide, presentation and SVG/PDF/PNG export. Interruption uses
that sample as the previous figure. Plans and sampled figures retain resources after
input handles are disposed; disposing the plan rejects subsequent sampling.

```python
plan = target.guide_transition(previous)
mid = plan.sample(0.5)
interrupted = next_target.guide_transition(mid)
image = interrupted.sample(0.5).export("png")
chart.acknowledge_frame(mid)  # Only if the chart still matches this target state.
frozen = chart.request(output, export_options(900, 300).basis("displayed")).prepare()
```

Use the exact presented scene dimensions for `displayed` capture; resizing/reflow and
full-domain capture reject with this basis. Default `presented` captures acknowledged
inputs and reflows them at publication dimensions. `current` captures current inputs.
`presentation()` returns retained lifecycle IDs, positions and opacities. Animated scene
metadata uses scene wire v15; static definitions keep their earlier version. See the
[three-state example](../examples/common/axis_transition_fixtures.rs) and
[native clock/capture example](../examples/chart-gallery/examples/axis_transitions.rs).

## Hierarchy authoring (HIR-07)

`hierarchy_tree(id, parent)`, `hierarchy_cluster`, `hierarchy_icicle`,
`hierarchy_sunburst`, `hierarchy_treemap` and `hierarchy_pack` create layers over a keyed
source table. Rust also accepts owner-scoped field mappings; Python/WASM factories use
field names and their value/label setters accept owned `Field` handles. The default
aggregation counts leaves. Select `.hierarchy_value("value")` to sum each node's own
value plus descendants, `.hierarchy_order(...)` for stable sorting and
`.hierarchy_label(...)` for inspection labels. These layers need no x/y aesthetic.
A pure hierarchy plot hides unused default axes. The generic `hierarchy(recipe)` takes
explicit owner, source, aggregation, order, layout, projection and resource limits;
`HierarchySource::Paths` (host descriptor `{"Paths":"path"}`) imputes missing ancestors.

```python
import finstack_chart as c

data = c.Data.columns({
    "id": ["root", "a", "b"],
    "parent": [None, "root", "root"],
    "value": [0., 1., 3.],
}, keys=[101, 102, 103])
plot = c.plot(data).layer(
    c.hierarchy_treemap("id", "parent")
     .hierarchy_value("value").hierarchy_label("id")
).build()
```

`.hierarchy_layout` takes the canonical Tree/Cluster/Partition/Treemap/Pack descriptor.
All standalone numeric controls are available, including registered separation, padding,
radius and tiler operations. Destination extent is fitted to the panel; fixed tree node
spacing remains in destination units. `.hierarchy_projection` selects Cartesian,
horizontal, radial or sunburst with explicit inner radius and linear/area depth mapping.
Circle radius is not area-scaled. Source color aesthetics and theme styling use the common
pipeline. Labels are semantic inspection labels rather than an automatic collision/layout
policy for visible node text.

Python/WASM expose `Hierarchy` independently of charts, with version-1 input descriptors,
node/traversal/search/sum/count/sort methods, layout/configuration, copy/subtree copy,
strict snapshot round trips and independent packing helpers. IDs in descriptors are
exact decimal strings. JavaScript includes snake_case and camelCase aliases. Registered
callbacks are native implementations captured by an explicit registry; host functions are
not per-node callbacks. See the public type declarations and [ADR-022](adr/022-hierarchy-topology-and-layout-history.md)
for the full ownership and finite-value policy.

Resquarify continuation requires retained history. Live/native consumers use the
acknowledged frame; standalone publication can call
`request.with_hierarchy_history(previous_frame)` in Python/WASM or pass the previous
layout in Rust. A fresh request has fresh history. Immutable frames retain exact geometry,
compact source membership and coherent scene identities after subsequent updates/disposal.
Scene version 15 includes `hierarchies.snapshots`: node records, recipe, panel/inset scope,
source keys once, per-node subtree ranges and history counts. Inspection uses painted paths
(including sunburst holes), visits nodes before links and reports label/depth/height/value
and member count. Zero-area nodes remain queryable in structural metadata.

Treemap accessor precedence is explicit: `padding_sides` overrides `padding`, which
overrides the numeric `options.padding_*` fields. For example,
`{"Treemap":{"options":{},"history":False,"padding":{"Constant":1.},"padding_sides":{"Top":"Depth"}}}`
uses depth as top padding and one unit on the remaining sides. `Inner`, `Top`, `Right`,
`Bottom` and `Left` accept constants, fields, value/depth or registered accessors. An
outer-padding operation assigns the four outer sides. Replace the immutable descriptor
to reset controls; `Hierarchy.configuration()` returns normalized values and
`effective_ratio` reports the clamped squarify/resquarify ratio. Tree/cluster node spacing
and extent remain mutually exclusive. Constructor field selections live in the caller's
`StratifyOptions`/input descriptor; changing or resetting them constructs a new topology.
The [final hierarchy catalog](evidence/phase-2-hierarchy-integration/verdict-catalog.md)
maps every D3 export, method, default and adaptation to executed evidence.

## ggplot guide composition

Under `Profile::Ggplot2_4_0_3`, guides use prepared scale outputs for color, fill,
stroke, size, area, alpha, width, shape and line type. Compatible titles, labels and
presentation options merge their keys. Automatic layer awareness leaves a key's
glyph blank when that layer has no matching value; labels and scale limits remain
shared. `LayerLegend.show` and its `aesthetics` map control inclusion, while
`key_glyph` selects the geometry drawn in the key. Guide overrides never alter marks.

`legend().scale(name).options(LegendOptions { .. })` configures a named color scale;
`legend().aesthetic(channel)` selects the relevant mapped aesthetic. Options include
title, order, reverse, direction, row/column layout, fill order, key overrides and
right/left/top/bottom or fractional inside placement. Order zero sorts as 99;
equal orders retain authoring order, without depending on R's internal serialized
hash order. Facet collection uses the existing shared/per-panel policy. Stepped
color guides support `GgplotColorbarOptions.even_steps` and `show_limits`; binned
key guides draw interval glyphs between labelled boundaries.

`axis.ggplot_axis(Some(GgplotAxisOptions { .. }))` and the same independent-guide
method provide label dodge, endpoint-first overlap removal, domain caps, minor
breaks, log ticks and measured stacks. ggplot axes preserve labels by default.
Log ticks reuse numeric projection, including finite secondary units; prescaled
logs and signed ranges have explicit controls. Non-numeric or non-invertible
infinite domains return a capability error. Angular guides belong to GG-13.

`legend().custom(CustomLegend { .. })` adds portable paths with intrinsic bounds,
optional title and ordinary guide placement. Content uses the shared vector path,
paint and stroke types; text may be supplied as glyph outlines. It has no runtime
R/grid object dependency. Theme-wide inheritance and mathematical typography
remain owned by GG-14.

Definition versions are 68 for nondefault step/limit controls, 69 for legend/layer
controls, 70 for authored axis policies and 71 for custom vector guides. Scene
version 19 retains legend title, key, label, bar and tick component roles, with
original values and panel scope. Python uses the corresponding snake-case builder
methods; WASM uses camelCase. See the independently authored
[Rust example](../examples/common/ggplot_guide_composition.rs),
[Python proof](../scripts/bindings/ggplot_guide_composition.py) and
[WASM proof](../scripts/bindings/ggplot_guide_composition.cjs).
