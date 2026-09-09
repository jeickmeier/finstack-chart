# ADR-020 — Shape generators consume checked paths

Date: 9 September 2026. Status: Accepted for WP-S01 entry and subsequent family work.
Requirements: SHP-01–10; FIX-S01–09. [ADR-015](015-path-authoring-and-replay.md)
owns numeric path construction, state, limits, serialization and destination lowering.

`chart_core::shape` will own reusable geometric generators and pie/stack layouts.
Generators return the existing owned `path::Path`/`PathGeometry` representation and
replay through `PathSink`; they do not create a second serializer, arc kernel or
scene transport. Default generator precision is three digits; unrounded and other
checked precision settings affect SVG text only. An empty result has no commands
and serializes to the empty string, the typed equivalent of D3's null path result.
Scene adapters omit empty results. Fill/stroke and source provenance remain separate.

The pinned d3-shape 3.2.0 context operations and layout records are the external
oracle. `fixtures/shapes/inventory.json` records all 63 exports, methods, defaults,
factory controls, aliases, palettes and boundary helper inheritance. JavaScript is
strictly a development dependency. The alias names do not create extra algorithms.
The retained source and license hashes also pin its d3-path 3.1.0 dependency.

Built-in curves expose the areaStart/areaEnd/lineStart/lineEnd/point protocol through
checked native Rust interfaces. A generator feeds authored order and explicit defined
runs into one curve instance; areas reverse the lower boundary exactly as the
reference does. Built-in curve selection and finite parameters are portable data.
Native data accessors receive datum, index and input; portable callers materialize
coordinates/validity or resolve a versioned registered Rust operation. There is no
JavaScript/Python callback execution inside core and no serialization of arbitrary
native closures. The family packages fix their smallest working public descriptor
and consumer signatures before exposing them; the entry package adds no stub API.

Default chart line sorting/gaps/singletons, ribbon bound validation, radius-based
symbols and separate-sign stack normalization retain their existing meanings.
The explicit shape route preserves authored order, D3 curve-specific degeneracies,
independent area boundaries, area-sized symbols and the named D3 stack policies.
A curve acts on projected coordinates. Its controls and tessellation vertices never
become source observations; source/derived targets and keyboard order are carried
separately. Native painting never parses SVG strings.

The shape oracle records finite numeric context operations before path serialization,
as well as unrounded and digits 0/3/12 strings. Entry comparisons of context replay
and rounded SVG are exact. Subsequent generator arithmetic uses the existing path
coordinate tolerance (2e-12 times max(1, abs(expected))) for f64 control coordinates;
topology, masks, order, identity and invalid-policy outcomes remain exact. Independent
analytic invariants supplement each family. Pie angles, stack arithmetic, destination
lowering and raster evidence receive separate operation-specific checks rather than
reusing a global image tolerance. New tolerances must be justified at the corresponding
family gate, before acceptance, without changing unexplained expected geometry.

Path operation/command/replay/SVG/subdivision limits remain enforced. Shape entry
counts, curve state, custom operations and layout work receive explicit bounds at
their family API boundary; no unbounded native callback loop becomes a portable
operation. Unsupported bundle-area use diagnoses before drawing. Native-only custom
protocols and registered portable protocols are distinguished explicitly.

WP-S01 proves the shared foundation with a circular sector, a nonzero-winding annular
hole and a Bézier stream replayed through an external sink in Rust, Python, WASM,
native and headless output. These are pinned shape context consumers. They do not
claim that line, area, arc, pie, symbols or stack generators are implemented; those
remain WP-S02–07, with combined acceptance at WP-S08/G-SHAPE.

## Cartesian delivery decisions — WP-S02

`Line` and `Area` consume materialized numeric rows with column/constant coordinates
and an exact-length boolean mask. Native `generate_by` and `generate_with` accept
fallible accessors and the public checked curve factory/protocol. Boundary lines
inherit coordinates, curve and validity; the pinned reference resets their SVG
precision to three digits and treats an absent optional endpoint as constant zero.
`ShapeLine`/`ShapeArea` Python and WASM owners return independent existing `Path`
owners. Their typed configurations represent accessor materialization explicitly.

Chart `shape_line` and `shape_area` are explicit authored-order routes. General
areas require all four x/y/x2/y2 mappings, allow crossed bounds, and split a paired
run when either endpoint is missing. Ordinary line/area/ribbon policies remain
unchanged. Curve evaluation follows positional projection. Definition version 7
and scene version 3 retain the new capabilities; older envelopes cannot decode them
as their previous meanings. `ShapePath` reuses `PathGeometry` and all destination
renderers while carrying source anchors independently of controls.

Inspection flattens shared paths with a 0.01 destination-unit geometric error,
32 subdivision levels and one aggregate million-vertex budget per inspector.
Arc lowering consumes half the error; adaptive De Casteljau subdivision consumes
the other half. Nonzero fills preserve holes and implicit closure; solid stroke
inspection uses butt caps and miter joins with limit four. Boundary queries within
the declared error have that approximation uncertainty. Conservative control-hull
bounds include stroke miters. One path candidate evaluates geometry once per query;
source anchors provide nearest-x, keyboard identity and source-vertex selections.
Clipped source anchors can support focus on a visible filled interior, but do not
become fabricated brush vertices. Precision formatting never changes this geometry.

WP-S02 proves solid Cartesian strokes/fills and the above inspection contracts.
Registered portable curve factories and broader cross-family style/inspection,
including curved dash integration, remain WP-S07/08; those are not inferred from
the solid-curve gallery or the original Cartesian gates.

## Arc/pie integration (WP-S03)

Arcs retain analytic path arcs through similarity transforms. Chart x/y values locate
the projected center; radii/corners/padding radii are destination units and never
expand positional domains. Nonpositional numeric parameters reuse `NumericEncoding`
and its stage-aware readers. Only rows using these parameters allocate their optional
shape payload; ordinary encoded rows carry one optional pointer rather than a full
arc parameter block. The new prepared path carries local geometry and a source-mark
focus location; projection translates both through the checked shared affine route.

Pie is a layout feeding the arc kernel. Its standalone output keeps original data,
value and angular rank in original array order. Source grouping can partition chart
pie populations; an explicit `pie_grouped(false)` combines one layer/panel, allowing
category-level statistical counts to remain separate aggregate targets within a pie.
Slice color does not implicitly become a population grouping. The pie layout owns
its sweep/padding; incompatible per-row angle mappings diagnose before empty inputs.
Native comparator/accessor functions are supported; registered portable comparators
are owned by WP-S07, so finite host fixtures cover built-in portable orders and native
Rust additionally covers datum comparators.

The original `2e-12 * max(1, abs(expected))` coordinate contract applies to raw arc
commands across platforms. Four gallery endpoint scalars differ by at most
`5.684341886080802e-14` between native and WASM math; command topology and every other
scene value agree. PNG/PDF bytes and externally rendered SVG pixels agree. The
comparison runner records each coordinate difference instead of rounding scenes or
silently treating approximate numeric equality as byte equality. Update-versus-batch
proofs use identical explicit color catalogs because retained categorical dictionaries
intentionally keep removed labels under the existing legacy policy.

## Area-symbol integration (WP-S05)

`shape::Symbol` owns all thirteen built-in geometries, the ordered reference fill
and stroke palettes, and the `X` alias for `Times`. Native `SymbolDraw` writes into
the checked shared path destination. The portable built-in descriptor holds kind,
size and SVG precision; reusable host owners return independent paths. Zero size
is valid, while negative or nonfinite size diagnoses. Chart preparation omits zero
size paths and their targets. Open plus/times/asterisk shapes require stroke paint;
`Auto` selects the appropriate built-in family and explicit `Stroke` also supports
the circle in the stroked palette. Legacy point radius remains unchanged.

`AreaSize` is the named numeric channel for reference area/stroke-size semantics.
Source fields, expressions and statistical inputs reuse the existing numeric
mapping pipeline. Symbol type uses an explicit categorical domain and palette,
with optional missing fallback and source/prepared-group readers. Type guides use
the declared domain; size guides evaluate authored input samples through the same
trained scale as the actual marks. Legend glyphs retain their actual resolved size
and paint. Oversized glyphs clip within the reserved guide cell rather than being
silently normalized. Shared color legend layout retains its previous dimensions.

The local path is translated to the projected center after positional transforms.
Source and generated identities use the existing `ShapePath` anchors/targets;
filled interiors, open strokes, zero-size omission, clipping and focus therefore
share the curve/arc inspection implementation. No destination owns a second symbol
kernel. Native protocol registration and the full cross-family acceptance remain
WP-S07/08 obligations.

## Stack delivery decisions — WP-S06

`shape::Stack` owns all six rank policies and five offsets. Output series retain
configured key order; `index` records stacking rank, and each point owns its original
source datum. Portable callers supply rectangular sample-by-key values and may pass
original metadata separately. Absent cells use an explicit Gap (NaN endpoint), Zero
or Error policy. Native ordering and offset protocols validate permutations, output
shape, exceptional endpoints and work bounds. Built-in wiggle preserves the pinned
column/series summation order; allocation and work budgets are independent.

Primary `shape_stack(groups)` adapts tidy source or generated rows to that same
kernel. It requires a distinct explicit group catalog, a zero input baseline and
one row per group/sample. Samples sort by calculation-space x, then x2 for intervals;
signed zero addresses one sample. Duplicate cells require an explicit statistic.
Only bars, rectangles and `shape_area` consume this position. Areas require equal
x/x2 sample coordinates. Zero-preserving numeric scale-stage encodings are allowed;
Expand produces a dimensionless output space. These rules do not change the legacy
`stack(...).normalize(...)` separate-sign behavior.

An absent or null cell never acquires a source target. Gap splits an area run;
Zero contributes a boundary vertex with no source index. `StackBandRun` retains the
optional target indexes until the existing area projection produces one shared
`ShapePath`; only actual observations become anchors or keyboard/inspection targets.
Stack areas use y0 as the lower boundary and y1 as the upper boundary. Curves still
operate after axis projection. Source colors and after-scale styles use the existing
run-constant checks. Definition capability version 7 also covers stack positions
on legacy bar/rectangle geometry. JavaScript BigInt metadata materializes as exact
decimal text; shared pie/stack declarations describe that transformation recursively.

WP-S04 adds `LineRadial` and `AreaRadial` as thin wrappers over the existing
materialized line/area readers and run lifecycle. A private curve adapter converts
polar inputs using `r*sin(a), -r*cos(a)` before forwarding to any existing curve.
`point_radial` and the radial link tangent protocol preserve the reference's separate
`r*cos(a-pi/2), r*sin(a-pi/2)` evaluation. Generic links feed two endpoints into the
same line lifecycle; horizontal/vertical factories select BumpX/BumpY, and radial
links use a private radial tangent sink. No hierarchy layout or node identities are
required. Native accessors and materialized selectors share these paths.

Radial descriptors use named angle/radius fields; areas additionally expose separate
start/end angle and inner/outer radius selectors. Shared angle/radius setters clear
the corresponding optional endpoint. Portable shared and independent selectors cannot
coexist for the same dimension; such ambiguous descriptors reject explicitly. All
four boundary helpers preserve reference zero fallback and three-digit defaults.
The pinned radial corpus contains 40 point conversions and 697 path configurations.
Its six unrounded cases use the existing raw-coordinate tolerance for decimal
transcendental values, while all rounded cases require exact SVG bytes. The retained
initial diagnostic isolates a cosine ULP (`9.87581249560918` versus
`9.875812495609178`); fixtures, numerical tolerances and rounded checks are unchanged.

WP-S04 chart integration adds authored radial line/area runs around one x/y center
per run, with named Angle/Radius and independent StartAngle/EndAngle/InnerRadius/
OuterRadius channels. Centers alone train positional domains. Radii are destination
units; polar conversion precedes the Cartesian curve and center translation follows
axis projection. Radial run X ordering means increasing start angle; authored order
is the default. Shared and independent channels for the same dimension reject
ambiguity. Radial runs require identity positioning; distinct centers need distinct
groups. Lines read constant start_angle/inner_radius; paired areas/links read both
boundaries and preserve absent endpoint fallback. General polar-axis navigation is
not implied by this projection contract.

Cartesian links reuse prepared endpoint rules and curve only after both axis
projections. Radial links reuse the radial tangent generator around the projected
center. Both carry two actual endpoint anchors with the same source edge target;
keyboard and brush selection deduplicate that identity. Curve controls are never
source targets. `ShapePathRun` carries radial run anchors independently of geometry,
while existing atomic ShapePath behavior remains unchanged. Radial lines participate
in the existing nearest-x source-line policy. All new routes require definition
version 7 and use the existing scene shape-path version.

Integration exposed a shared facet resource-accounting defect: the old fallback
counted every atomic shape as two vertices and undercounted polygons and rectangles.
A retained pre-fix macOS Python runtime actually accepted two quarter sectors with
four commands plus one anchor each under max_vertices=7. Facet accounting now charges
actual commands plus anchors for both shape representations, polygon vertices and
four corners for bars/rectangles. The regression rejects seven and accepts ten for
the two-sector figure; expected geometry and limits were not relaxed.

Projection work also has one figure-wide preflight before destination callbacks.
Local path commands and source anchors count independently; facet panels and inset
replays consume the same total budget. The shared `layout::work` accounting replaces
separate panel/inset estimates. Rectangle/rule projection retains its two-endpoint
cost, while polygons count their actual vertices. Scene command/resource validation
continues after projection. This closes a reproduced layout-limit bypass that was
independent of the earlier compiler facet limit correction.

WP-S07 registration uses the existing `ExtensionRegistry`, with a separate bounded
catalog of 64 exact shape versions. `CustomShape` captures descriptor and protocol
family at registration, validates bounded JSON parameters and resolves a reusable
`ShapeProtocol` (`Curve`, `Symbol`, `PieComparator`, `StackOrder`, `StackOffset`).
Prepared layers retain resolved implementations by shared ownership; they do not
retain host callbacks or re-resolve a mutable external registry during layout.
The generic registry never loads executable code. Native callers can resolve known
native-only implementations explicitly. Portable resolution, primary plot JSON and
registry `shape_to_json` reject native-only registrations. A raw `ShapeOperation`
contains only identity/version/parameter data; it is not an executable callback.

Layer `shape_protocols` is an optional family-to-selection map and requires wire v9.
Versions 1–8 retain their capability boundaries. A registered curve operates after
Cartesian projection or after radial-to-Cartesian conversion. Registered symbols
use the same draw method for marks and size-guide glyphs; prepared guide geometry
is retained independently of paint/placement. A registered symbol replaces a builtin
palette selection. Selecting a builtin curve, symbol or pie order removes that
family's custom selection; replacing a position removes its custom order/offset.
Incompatible families and simultaneous geometry replacements reject.

Standalone pie comparisons see the supplied owned JSON datum and resolved value.
Chart comparisons run after statistics and encodings and see an explicit record:
`key` (decimal source key or null), `ordinal` (decimal), `target`, `group`, `x`, `y`,
plus the resolved pie value passed separately. These are stage/provenance records,
not a reconstruction of the original source row. Native generic comparators continue
to accept arbitrary caller-owned datum types. Fallible comparison uses bounded stable
merge sorting and propagates callback failures. Stack protocols retain checked rank
permutations, declared work, rectangular output and finite/missing endpoint policies.

Curve factories may declare a conservative `command_bound` covering the total input
and any partition into gapped runs. Registered chart curves must declare it, and
figure/inset preflight checks the aggregate before destination callbacks. Generated
paths also enforce a declared bound, so an understated implementation cannot return
oversized output. Native standalone factories without a declared bound remain limited
by their caller's explicit `PathLimits`. Registered code is trusted Rust code and
must respect its declared work contract; the registry is not a code-execution sandbox.

Python/WASM `ShapeRegistry` handles own explicitly installed Rust registrations.
`example()` exists in proof-enabled native builds; ordinary construction is empty.
Standalone `generate_registered` / `generateRegistered` and layout equivalents invoke
the same native kernels. Primary builders accept the registry and versioned layer
selections; primary `Plot.from_json` accepts an explicit registry for known wire-v9
operations. No interpreter object enters core and no callable is silently serialized.

## WP-S08 retained dash styling

`ShapePath` and `VectorPath` retain an optional even positive dash pattern alongside
fill and stroke. Empty patterns are omitted from scene JSON, preserving solid-scene
versions and bytes; a nonempty pattern requires scene capability version four.
Definition versions are unchanged because existing theme dashes already describe
this styling. Theme resolution sets the pattern only on stroked primitives. Geometry,
nonzero fill winding, source anchors, clipping and targets are retained unchanged.

SVG emits the original vector curve plus `stroke-dasharray`; PDF carries the same
native vector stroke pattern. Native rendering uses the shared bounded path flattener
at its destination tolerance, then the existing dash splitter with phase reset at
MoveTo. Its geometric tolerance does not claim an independent arc-length error bound.
Fill is drawn from the original geometry. Inspection keeps solid fill geometry and
an independently bounded dashed stroke path, so an unfilled dash gap does not hit;
nearest-source and keyboard identities still refer to the original observations.
Scene validation charges dash output in the aggregate path budget. Destinations
recheck their finer lowering budgets, and pathological tiny dashes fail explicitly.
