# ADR-002: Minimal core contracts and normalization boundary

Status: ACCEPTED for WP-02 foundations and WP-05 authoring/compiler boundaries.
Date: 6 September 2026. Requirements: ARC-01/02/03, SCN-01/02, BND-01, QLT-01/05;
WP-05 adds GRA-01/02/03/04/06/08 and DAT-06 foundational portions.

## Decision and consumers

Keep `chart-core` dependency-free and synchronous. Its first public modules are
`identity`, `diagnostic`, `geometry`, `limits`, `scene` and `services`. The Rustdoc
example and integration tests consume these modules through the public crate boundary.
The next consumers are WP-03's native/font/export spike and WP-04's data contracts.
WP-04 implements data storage; the WP-05 section below adds working authoring/compiler
contracts. No renderer or wire decoder is introduced as a placeholder API.

Typed authoring now lowers into normalized values/field mappings in WP-05. The initial
`Scene` can be constructed directly for capability fixtures, without introducing a
second chart engine. Ergonomic authoring and full scene interfaces remain unfrozen until
real consumers establish their requirements.

## Identity, geometry and diagnostics

- Dataset, layer, field, resource and row identities are distinct `u64` wrappers. Zero
  is valid; all 64 bits are preserved. Identity allocation, uniqueness and ownership are
  caller responsibilities. Row keys are not indexes. JSON/BigInt encoding is not defined
  by these Rust types and remains WP-09 work.
- `Revision` advances through checked addition. It is scoped to an owner/epoch; a
  `SceneStamp` records definition, coherent store commit, layout/resource/profile and
  viewport revisions. Stamps do not implement scheduling or claim presentation freshness.
  Per-dataset revision maps and transaction semantics follow in WP-04.
- `Point` and `Rect` have private validated fields. Coordinates/extents and far-edge
  additions must be finite; negative extents are rejected and zero extents remain empty.
  There are no unchecked arithmetic operators or silent clamps. Scene coordinates are
  destination `f64` values; native `f32` conversion/precision checks remain host work.
- `ChartResult<T>` returns a `Diagnostic` with stable code, severity, message, correction
  and available dataset/layer/field/resource/revision context. Context is boxed to keep
  result values small. Validation fails at the first error, retaining caller-owned prior
  results. Bounded repeated-row aggregation belongs to the later data/statistical paths;
  defining its error categories does not implement those paths.

## Minimal scenes and budgets

`Scene::new` accepts borrowed items/resource descriptors, checks limits and references,
then owns copies with read-only access. Paint order is slice order. Items carry optional
layer metadata and an explicit clip override; an absent clip means scene bounds.
Decorative items have no invented source targets. The primitive subset is solid rules,
rectangles, circular points, numeric stroked paths and plain text. Paths require an open
subpath before a drawing/close command, at least one line/curve, and a new `MoveTo` after
`Close`. Text retains its logical UTF-8 string and explicit font identity.

The [Limits defaults](../../crates/chart-core/src/limits.rs) bound item/resource counts,
total path commands, total UTF-8 bytes, per-resource bytes and total resource bytes.
Scene preflight uses remaining budgets to avoid aggregate arithmetic overflow and runs
before map creation, path scans or payload copies. These are configurable per-construction
budgets, not performance claims. Callers own input allocation; a future decoder must
enforce its own preallocation limits. Raising budgets increases allowed memory/work;
the API does not promise recovery from process-wide allocator exhaustion.

Groups, transforms, gradients, dash/cap/join policies, filled paths/areas, rich shaping,
target metadata and renderer capabilities remain explicit later work. WP-02 does not
close full SCN-01 or introduce a serializable scene DTO.

## Host services and ownership

Resource descriptors identify immutable bytes by ID/revision, kind and expected length.
Duplicate IDs, even with differing revisions, cannot coexist in one scene. Text references
must name declared font resources. The host must resolve the requested revision, validate
the actual format/content, and honor font permissions; descriptor validation alone does
not prove byte identity or font suitability.

`resolve_resource` checks the declared byte limit before invoking `ResourceProvider`,
then checks the returned length. Returned bytes borrow the provider; core does not copy
or own a resource cache. `measure_text` checks UTF-8 bytes, resource kind and positive font
size before invoking `TextMeasurer`. Requests explicitly carry destination units and font
revision; metrics expose finite width/ascent/descent and a finite summed height. Native
and publication measurements are not assumed interchangeable. WP-03 must extend these
contracts as required by actual shaping, ink-bound and publication-font evidence.

Service traits require neither `Send` nor `Sync`; a single-threaded `Rc`-based host works.
No host object or callback is retained inside `Scene`, and no filesystem access, system
font lookup or background thread is created by core. A renderer must resolve resources
and verify its capabilities before submitting this minimal scene.

## Compatibility and remaining gates

These are initial 0.1.0 Rust foundations, with changes recorded in the
[changelog](../../CHANGELOG.md). There is no supported wire schema or serializer yet;
unknown constructs/version rejection and complete portable authoring remain WP-05/WP-09.
Python/WASM packages remain nonfunctional proof-adapter shells. Full QLT-01/05, BND-01,
ARC-03 extension behavior and SCN-01 remain open beyond their WP-02 subsets.

The [WP-02 evidence](../evidence/wp-02-completion-2026-09-06.md) records actual contract
tests, target graph checks and build validation. WP-03/04 are complete and G0 capability evidence is established. WP-05 evidence below
establishes its grammar foundation; G1–G4 remain open.


## WP-05: One staged grammar preparation route

`grammar::ChartDefinition` owns chart defaults, heterogeneous layers and named transforms.
A layer is input + source filters + registered statistic + stage-specific mappings + geom
+ position + constant style. Layer vector order is paint order; identity is a stable
`LayerId`. Inherited source mappings must exist with compatible types in each layer, or
be overridden/disabled. Generated mappings never inherit arbitrary source accessors.

`TypedDataBuilder<T>` calls native `Fn(&T)` accessors synchronously after schema/count
preflight, materializes all seven column kinds with independent validity, then discards
the closures. The result is ordinary `NormalizedBatch` data consumed by the same engine.
This materializes a snapshot, not a portable reconstruction of native code. Typed dataset
identity/revision stays with its caller; normalization does not commit a transaction.
Portable definition serialization and native-closure rejection at serialization remain
WP-09. The compiler itself holds no original typed rows, native functions or host objects.

`SourceAes` binds source fields/literals; `BinAes` binds the distinct `BinField` enum.
The generated schema has a version and exact field kinds: finite start/end/midpoint
coordinates and an unsigned membership count. A source accessor cannot receive a bin
row. Source amounts and timestamp integers remain recoverable from the captured snapshot.
Constant `Style` is separate from mapped sizes; mapped size supports point radius and
rule width. Variable-width lines and mapped size on filled rectangles fail explicitly.

Preparation validates all operation IDs/versions, graph dependencies, inferred output
schemas, mappings, calculation-space compatibility and constant styles before population
work. It then filters source rows, computes stats, binds generated encodings, applies the
implemented identity position, collects endpoint domains and emits finite data-space
geometry/provenance. There is no feedback edge from layout/viewport into statistics.
Unknown operations, wrong versions and graph cycles fail explicitly. Builtins are
`chart.identity`, `chart.bin` and the explicit pre-stat `chart.affine`, all version one;
there is no dynamic plugin execution or custom-stat capability claim.

## WP-05: Statistics, geometry and provenance

Identity preserves original row targets or already-generated aggregate rows. Explicit bins
use [left,right), including the final right edge. Default outliers are excluded with below/
above counts; an error policy rejects. Null/non-finite/imprecise required inputs and missing
groups exclude with bounded diagnostics, or reject in strict mode. Source filters are ordered
inclusive numeric bounds and change the population before stats. Whole-population scope is
the recipe default; exact category/string/integer/boolean grouping is supported. For an empty
whole population, all explicit bins remain present with zero counts and empty memberships.
Grouped bins retain groups with valid group values, even if their numeric inputs are all
excluded; a population with no valid group labels creates no groups.

Statistics default to source units. An explicit affine statistical space transforms inputs
before binning and records the exact operation/parameters and output space. Generated edges
are already in that space and must not be transformed again. Timestamp mappings subtract a
checked integer origin in original ticks. Layers with incompatible transforms or timestamp
origins on the same minimal axis reject. Independent named scales remain WP-06/11.
Operation records retain input dataset/schema revisions, exact parameters, filters,
grouping, space and population counts. All implemented computations use full recomputation;
custom/incremental capability declarations are deferred to WP-10/14/18.

Points, straight line runs, rules and rectangles share the same prepared table engine.
Lines sort by finite x and stable insertion ordinal (including equal signed zeros), or use
explicit authored order. Invalid y splits runs after ordering; invalid x splits authored
blocks before ordering because it has no sortable location. A missing group separates
runs across that authored boundary. Explicit gap connection bridges exclusions; the default
does not. One-point runs remain explicit, preserving FIX-01 without inventing a segment.
Rectangles retain both exact data endpoints, including a supplied baseline; data-space
origin-plus-width arithmetic must not discard small far endpoints. Domain contributions
include all eligible endpoints and hidden layers. Bin midpoint uses overflow/underflow-safe
standard-library arithmetic. Finite scalar data coordinates are not a pixel precision claim.

`PreparedChart` is the minimal prepared scene foundation: immutable definition, coherent
source snapshot, captured state, typed stat tables, data geometry, endpoint contributions,
calculation-space metadata and per-mark/per-line-vertex targets. It is deliberately distinct
from the destination `scene::Scene`: axes, range mapping, clips and text-aware bounds are
WP-06 work, followed by native/export consumers. The renderer must never interpret these
data coordinates as pixels. Scale preparation will construct destination rectangle bounds
from the retained endpoints. Full portable primitive families/hit indexes remain later work.

Identity marks retain `Target::Source`. Binned marks carry complete `Target::Aggregate`
membership, group/bin identity and input revision; they never impersonate a representative
source row. A line has one target per vertex, not an invented aggregate target for its path.
Pinned prepared charts keep historical target resolution valid across corrections/disposal
of external handles. Selection removal events and historical tooltip labeling remain WP-15/18.

## WP-05: Reuse, state, budgets and next consumer

Named outputs form an iteratively checked DAG and share immutable row arrays across layers.
The compiler retains one successful graph cache, keyed by exact captured source snapshot
identity, full graph definitions (operations/versions/parameters/filters/grouping/space/invalid
policy), and compile budgets. This conservatively includes all data/schema revisions and
source epoch. Different stores cannot alias merely because their revision numbers match.
Viewport, visibility and layer palette changes reuse the graph; corrections, parameters,
source snapshots or budgets invalidate it. Sharing occurs across calls using the same
compiler; no process-global cache or mandatory threading is added. A failed preparation
leaves the previous cache and returned charts valid. `clear_cache` releases its ownership.

The minimal state reducer handles viewport, layer visibility and reset. Effective changes
advance checked state revisions; idempotent actions do not. Viewport revision advances only
for viewport changes. Invalid actions and counter exhaustion are atomic failures. Visibility
is captured separately from domain/stat training, and viewport changes never filter rows or
move explicit bin boundaries. Full gesture/selection/focus/presentation coordination belongs
to WP-15; these actions make no asynchronous presentation guarantee.

`CompileLimits` bounds layers, graph nodes, filters, groups, edges, total prepared-output rows
and geometry vertices. Shared outputs are conservatively charged at each consuming layer;
retained graph history has one entry, while user-held prepared charts have explicit independent
ownership. Source processing and geometry still perform row scans. These bounds are neither
RSS measurements nor sustained-load/performance certification. The [WP-05 report](../evidence/wp-05-completion-2026-09-06.md)
records independent semantic/geometry, cache, lifetime and numerical counterexamples.
WP-06 consumes this output for scales, ticks and layout; G1 still requires WP-06–09 evidence.

## WP-06 consumer update

[ADR-005](005-foundational-scales-and-layout.md) now implements the scale/layout handoff
above: named positional scale IDs and categorical encodings, independently trained domains,
linear/band/UTC mapping, four-pass plain-text layout and a destination Scene retaining clips
and per-mark/vertex targets. `PreparedChart` remains the upstream data/stat snapshot;
`LaidOutChart` owns its destination result. `SceneStamp::state` now distinguishes visibility
changes from viewport changes. Full native/export consumers and richer typography remain
WP-07/08/12/13. The earlier WP-05 statements describe that package's historical boundary.
