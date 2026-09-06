# ADR-002: Minimal core contracts and normalization boundary

Status: ACCEPTED for WP-02 foundations; authoring/compiler decisions continue in WP-05.
Date: 6 September 2026. Requirements: ARC-01/02/03, SCN-01, BND-01, QLT-01/05.

## Decision and consumers

Keep `chart-core` dependency-free and synchronous. Its first public modules are
`identity`, `diagnostic`, `geometry`, `limits`, `scene` and `services`. The Rustdoc
example and integration tests consume these modules through the public crate boundary.
The next consumers are WP-03's native/font/export spike and WP-04's data contracts.
No chart builder, statistical compiler, data store, renderer or wire decoder is added
as an unimplemented placeholder API.

Typed authoring will lower into a shared portable representation in WP-05. The initial
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
tests, target graph checks and build validation. WP-03 and WP-04 prerequisites are met;
G0–G4 are not passed.
