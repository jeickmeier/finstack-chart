# ADR-014 — Phase 2 profiles, references and coverage ownership

Date: 8 September 2026. Status: Accepted integration direction for P2-00.
Requirements: GG2-01/12, ARC-03/04, BND-01, QLT-02/05.
Baseline: `fab2505951061eaafe9adb52c86b248ee0dfa6bf` (primary authoring implementation).
This decision introduces no runtime API, dependency or wire-version change.

## One definition carries execution semantics

New compatibility policies lower into the canonical `ChartDefinition`, including
their immutable profile/version provenance. Do not introduce a separate mutable
profile on Chart, Session, a worker or Output. The current primary envelope's
`LibraryV1` field remains unchanged until GG-02 implements migration. Today it has
only one supported value; passing definitions through runtime/export is sufficient
for that baseline, not for a future profile selected only on Plot.

GG-02 owns the common profile contract. It resolves default grouping, stage order,
bin closure, aesthetic units and theme policies into explicit normalized semantics.
Core evaluates those semantics; hosts cannot select different defaults. The profile
identity records provenance and participates in definition equality/revision and
cache invalidation even when a particular plot resolves to equal numeric output.
Layout/device profiles remain distinct destination inputs.

D3 lane owners add explicit versioned descriptors for standalone algorithms and
their component consumers. Selecting a D3 descriptor does not globally change
unrelated chart/stat defaults. GG adapters reuse those kernels with explicit
reference policies. Libraries and version numbers alone never select behavior.
The existing LibraryV1 definition path preserves all current defaults.

Every effective policy edit uses the existing definition revision fence and current
runtime source. Old prepared scenes and Presented captures retain their original
definition/resources; Current uses the latest committed definition. Definition,
resource and layout changes invalidate their existing dependent caches. No profile
change may reuse an incompatible worker result or mutate a retained request.

## Migration and resources

The integrator owns the common migration; the first package needing new semantic
wire fields implements it with tests. Keep existing version-1 readers and defaults.
Write a new envelope version for new required constructs or changed interpretation;
never emit new semantics disguised as a legacy payload. Normalize legacy inputs
through an explicit conversion. Primary and portable envelopes have separate version
histories; no package independently increments both or redefines existing fields.
Downgrade succeeds only when the definition is exactly representable, otherwise it
returns a structured unsupported-capability diagnostic. Record the concrete version
and migration in the delivering package, not speculatively in this entry package.

Existing explicit resource descriptors/registries remain the boundary for fonts,
locale/calendar/CRS data and registered operations. The owning package adds version,
content digest, license/source provenance and validated limits where needed. Missing
or mismatched resources reject before evaluation. Core performs no resource lookup
or downloads. Captures retain the exact immutable resources and registrations used.

WP-AX01 owns guide/scale identity separation: preserve ScaleId-based layer bindings,
primary x/y identities and navigation targets while adding distinct guide identities.
Migrate AxisBuilder/AxisHandle, names, serialization and host navigation together.
The guide-to-scale relation must be explicit; guide IDs never become data scale IDs.
SP-01 supplies the shared scale capability contract. This work does not wait for
final G-AUTH, G-GGPLOT or G-PARITY certification.

## Shared reference workspace and fixture convention

Use one development-only Node workspace at `tools/reference/node/`. Its package.json
declares all eight plan-pinned D3 modules; package-lock.json pins transitive sources
and integrity. A runtime manifest pins Node and any browser oracle/runtime used.
The first D3 entry package creates and verifies the lock; subsequent lanes extend
that same workspace. No npm dependency enters a Rust production package.

GG-00 owns `tools/reference/r/`, with an exact R/runtime manifest, renv.lock and
source/dependency checksums for ggplot2 4.0.3 and its required capability dependencies.
It records platform/native-library constraints separately from R package versions.
Neither reference workspace or lock is claimed to exist or execute in P2-00.

Store checked outputs under `fixtures/parity/<reference>/`. Each corpus manifest
records reference release/source digest, dependency-lock digest, generator revision,
input/output digests, licenses, platform, locale, timezone, RNG and supplied fonts or
other resources. Every case carries a stable reference-item ID, operation/options,
input, expected output or diagnostic, requirement/package/FIX IDs and the comparison
rule. Encode large integers, non-finite values and missing values explicitly, using
the existing portable conventions where applicable. Do not encode process addresses
or incidental object names as semantic identities.

Compare exact topology/order/categories/IDs/strings first; numeric tolerances are
operation-specific with units and rationale. Keep numeric/semantic comparisons
separate from inspected SVG/PDF/PNG/native artifacts. Regenerate into a temporary
directory and compare digests before accepting changes. Rust tests consume committed
fixtures offline; R/Node generators never run as a prerequisite for ordinary cargo
tests. Missing oracle/runtime evidence leaves its row open.

## Coverage and delivery ownership

The [coverage register](../phase-2-coverage.md) links existing inventories to AP-00
rows, semantic owners, fixtures and required surfaces. Lane entry packages expand
their source inventories to method/argument/default rows; GG-00 owns that expansion
for ggplot2. P2-00 does not claim those exhaustive oracle inventories are finished.
Every new reference item must acquire an existing or new semantic owner before its
lane can close. One item can have several consumers but only one kernel owner.

AP-00–08's delivered builders/runtime/hosts are reused. Each semantic package owns
its Rust public operations, shared host dispatch, actual Python/WASM exports and
declarations, primary usage, fixture comparison and destination evidence. GG-18
broadens combinations; it does not postpone first-use binding evidence. A future
standalone utility remains directly callable as well as usable by plot components.

The integrator checks cross-lane edges from the combined plan, especially CLR-04 →
SP-04 and the absence of backward certification dependencies. AP-09's remaining
performance qualification stays with its owner. Pure contracts/kernels and the
existing legend acceptance can proceed on the committed baseline independently.
Completed P2-00 certifies this handoff only; no feature or cumulative gate closes.

## GG-02 implementation of the migration

The continuation implements the common contract with optional canonical
`ExecutionSemantics` and capability-checked definition version 3. The primary profile
must match that object; LibraryV1 retains absent semantics and legacy wire defaults.
Layer/shared-transform source grammar preserves grouping and positional inputs.
Current and Presented capture continue using the existing immutable definition owners.
Shared statistic consumers with incompatible positional scale contexts explicitly reject.
The source-expression population is the registered dataset before chart filters/facets;
computed-expression population is the prepared layer. Post-scale outputs use one
pre-modifier snapshot. Independent aesthetic units and remaining statistic defaults
continue in their owning GG packages; this stage implementation does not certify them.
