# Alpha extension contract

Status: WP-14. [Specification](spec/gpui-charts-specification.md) ARC-03, GRA-08,
SCN-02/03 and INT-06 govern this surface. [ADR-012](adr/012-registered-alpha-extensions.md)
records the boundary; [FIX-17 evidence](evidence/wp-14-completion-2026-09-07.md) records execution.

## Statistics and generated schemas

Register immutable `Arc<dyn CustomStat>` implementations in `ExtensionRegistry`, then
pass that registry to `Compiler::with_extensions`. Names are qualified, non-`chart.`
ASCII identifiers of at most 128 bytes; versions are positive and resolve exactly.
Duplicate registrations reject. A compiler owns one immutable registry and its caches.
Neither registration nor deserialization loads executable code or performs I/O.

`Statistic::custom` supplies an operation and `ExtensionParameters`: bounded JSON,
grouping and explicit data/pre-stat calculation space. `schema` validates source inputs
and returns 1–64 distinct `StatColumn`s. Custom numeric accessors are `StatField::Custom`;
`Count` stays exact UInt64 and `Group` stays categorical. Source, binned and statistical
accessors are separate Rust types and are also checked at portable preparation. The
compile-fail doctests reject source/generated accessor confusion.

`evaluate` receives the immutable filtered source population, scope, invalid policy,
operation and remaining compile budget. A viewport is not a statistical crop. It returns
finite/missing generated values and distinct invalid source keys. Core checks schema,
nullability, calculation-space metadata, group catalog, output size, membership/counts
and provenance before publishing the table. Each output row has a distinct aggregate
or registered derived-model identity. The target must refer to the actual input revision;
a generated row cannot impersonate an original source row. One target per output row
is the alpha custom-stat contract; repeated samples of a common model need distinct
stable derived target IDs.

Named transforms feed several layers through the ordinary graph and scales. Cache keys
include the operation/version/parameters/grouping, source revisions and effective facet
scope. View changes retain view-independent transform tables; corrections, removals,
retention and parameter changes recompute against the current full eligible population.
Alpha extensions declare only exact full recomputation. Append/window/correction fast-path
claims reject until a verified executor exists. Implementations are `Send + Sync` for
explicit host scheduling; synchronous use remains supported without threads.

Limits are checked before core copies extension parameters: 4,096 JSON nodes, depth 24,
64 KiB parameter strings/keys/scalars; at most 64 registered statistics and geometries.
Custom metadata has bounded depth/text/catalog sizes. Extension implementations are trusted
native code and must respect the supplied budget while computing their own output.

## Geometry, targets and native painters

`Layer::with_geometry_extension` selects an exact `CustomGeom`. The base geometry still
validates the required encoded channels. The callback receives checked prepared marks,
style, targets and limits after statistics/position and before scale training. Its output
uses ordinary numeric point/rule/rectangle/polygon geometry or an explicit native-paint
descriptor. The shared engine trains/maps/clips those marks and retains their targets.

Every output supplies `GeometryInteraction`: explicit point/rectangle/polygon hit region,
up to 32 named finite/exact semantic values, atomic-target or disabled selection policy,
and a stable keyboard order separate from paint order. Semantic text is bounded to 8 KiB.
Custom regions and values survive theme resolution, projection, facet and inset placement.
The presented inspector uses the declared hit shape, not its bounding box, and the
actual scene stamp. The [interaction contract](interaction-contract.md) defines the shared gesture/selection action system.

`NativePaint` carries numeric bounds, color, operation/version and bounded JSON, never
GPUI objects. `NativePainterRegistry` lives in `gpui-charts`; its `Rc<dyn NativePainter>`
prepares frame-local paint data before submission. `PreparedNativePaint::paint` executes
under the ordinary item/content clip. Missing versions report `UnsupportedCapability`.
A native callback has no portable representation. `FigureSnapshot` and SVG/PDF encoders
reject it explicitly with the painter identity; capability reports say
`native_painters = false`. It cannot silently vanish or become an unannounced raster.

## Scales, coordinates and custom guides

Extensions use the same resolved named scales as builtin marks. `ResolvedAxis::map_value`
checks numeric/category/timestamp kind and exact timestamp unit. `capabilities` distinguishes
numeric/time inverse from category lookup; secondary axes are guides only. `Cartesian`
binds primary horizontal/vertical axes, projects/inverts their semantic values and exposes
the exact rectangle used for clipping. Arbitrary path subdivision is explicitly false.
This alpha supports builtin Cartesian/scales with custom stat/geom/guide consumers;
it does not advertise arbitrary coordinate or scale registration.

`AxisSpec::guide_ticks` optionally supplies bounded semantic values and logical labels.
They replace automatic ticks and use the actual resolved transform and destination text
measurement. Wrong scale families/time units, empty labels, excessive text/tick counts
or simultaneous numeric formatting reject. Theme, layout and clipping remain shared.

## Portable execution and errors

Use `Session::with_extensions`, `PortableChart::with_extensions` or
`FigureRequest::with_extensions` to supply known implementations. Default
constructors have an empty extension registry. Portable session import/export rejects
unknown names, wrong versions and native-only definitions. Raw Rust serde can represent
a native descriptor for diagnostics; `Session::chart_json` is the guarded portable
serialization boundary. No wire format contains callback code.

The proof-only Python/WASM `extension-proof` feature links the same public example crate.
`Chart.with_example_extensions` / `Chart.withExampleExtensions` select its compiled
registry explicitly. Neither adapter reimplements its numerical or interaction logic.

Errors retain normal structured diagnostic context: `UnsupportedCapability` for unknown,
wrong-version or native-only operations; `SchemaConflict` for generated accessor/schema
mismatch; `Validation` for incorrect membership/identity; `NumericalDomain` for non-finite
semantic values; `ResourceLimit` for bounded input/output overflow. Failed preparation
cannot replace a previously valid prepared/presented snapshot.

## Compiling example

The separate [example crate](../examples/custom-extension/src/lib.rs) depends only on
public core APIs and serde. Its density histogram produces counts `[3,3]`, density
`[0.5,0.5]` and exact aggregate membership from six finite observations plus one null.
A custom chamfered-bar layer and a builtin point layer consume the same named transform.
The [native example](../examples/chart-gallery/examples/extension_gallery.rs) switches
between vector custom geometry and a native-only gradient painter; the
[export example](../crates/chart-export/examples/extension_proof.rs) emits SVG/PDF/PNG.
See [fixture commands](../fixtures/extensions/README.md) for executable checks.

## Registered positional scales (AXIS-01)

`ExtensionRegistry::register_scale` installs a versioned `CustomScale` factory. Its
`ScaleProviderInput` supplies semantic space, eligible extent, destination range,
explicit window/outside policy and hard limits. Return an immutable `PositionalScale`
with domain/range/forward mapping and optional ticks, formatting, bands and inverse.
Core checks values and budgets and shares the resolved mapping across marks and guides.
Provider callbacks are trusted native code and must honor the supplied work limits.

Use `scale_registered` with primary `coordinate_scale` and the plot's explicit
extension registry; native-only registrations cannot serialize or publish headlessly.
Version-10 definitions resolve installed operation IDs, never executable code from JSON.
The actual external implementation is
[`examples/custom-extension/src/scales.rs`](../examples/custom-extension/src/scales.rs).
The [provider decision](adr/021-independent-axis-guides.md#registered-positional-provider-boundary)
details exact timestamp/category preservation, registry snapshots and navigation limits.

### Registered guide formatting

`CustomGuideFormatter` declares a captured operation descriptor, validates bounded
JSON parameters and formats `GuideFormatInput` with the exact selected value, index,
complete selected list, parameters and limits. Register with
`CoreExtensionRegistry::register_guide_formatter`. The selection and mapping preflight
runs before callbacks; output obeys aggregate label-byte limits. Registrations are
trusted native code and cannot be preempted. Prepared charts retain the registry snapshot.
Portable versioned references require explicit code installation on load; native-only
formatters reject serialization and headless publication. The
[external formatter example](../examples/custom-extension/src/guides.rs) demonstrates
the contract without depending on a host interpreter.

## Registered interpolation factories

`ExtensionRegistry::register_interpolation` installs a `CustomInterpolationFactory`
with captured ID/version/portability. `InterpolationInput` supplies typed source/target
values and bounded parameters; `compile` returns an owned `Sample<Value>` implementation.
Compilation occurs during preparation, not once per mark. Native factory code must be
pure, terminating and independent of mutable host state. Sampling validates finite `t`
and the existing output value budgets. Missing/nonfinite numerical values retain the
interpolation profile and cannot silently enter scene coordinates.

Use `registry.interpolation_factory(operation, parameters)` for a checked selection and
`between_with_registry` or registry-backed scale constructors for execution. Registered
factories participate in piecewise, mapped colors/numeric aesthetics and time scales;
copies and immutable scale changes retain registry snapshots. Native-only operations
reject portable envelopes and publication before interpolation preparation. Registered
interpolation/standalone-scale envelopes use version 2; mapped chart definitions use
version 12. Builtin envelopes keep their existing versions. See
[ADR-017](adr/017-shared-interpolation-values.md) for ownership and migration details and
[the external example](../examples/custom-extension/src/interpolate.rs) for implementations
that reuse the scalar and floating Lab kernels.
