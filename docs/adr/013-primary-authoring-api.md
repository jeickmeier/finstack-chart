# ADR-013 — Primary authoring API over the shared engine

Date: 7 September 2026. Status: Accepted architecture direction; implementation planned.
Requirements: AUT-01–09, SCP-01, ARC-01–03, GRA-01–08, BND-01–04, QLT-05.
Owner instruction: the concise authoring API is the main public API, and every feature
must run through it. Plan against a completed original WP-01–23 baseline.

## Decision

Place primary plot/data/component builders in `chart-core`, with a small prelude and
specialized typed modules. A builder lowers to an immutable Plot using the existing
normalized definition/data representation. All built-in controls and registered
extensions must be available through this surface; editing raw DTOs cannot be the only
way to use an advanced feature. Keep standalone utility APIs directly callable.

Reuse/refactor the existing typed store/compiler/reducer operations as the retained
Chart entry point. Default ingestion owns its store; an explicit external-source route
consumes committed snapshots without creating a second writable authority. Keep
per-view state and worker-owned compiler caches independent. JSON sessions become
interchange adapters around typed operations. Native, export and language adapters
consume the same contracts; host tasks/resources and export request/job lifetimes
remain with their existing owners, never an alternate grammar compiler.
The [implementation plan](../impl_plans/primary-authoring-api-plan.md) owns the precise
sequence, capability register, examples and evidence requirements.

Generate routine identities and revisions while preserving durable handles, exact
data, atomic transactions and acknowledged-scene semantics. Preserve baseline default
policies and select reference compatibility profiles explicitly. Builder failures and
runtime failures keep the prior valid state. Native callbacks may materialize data or
use extension protocols but cannot become executable serialized code.

Ordinary Plot edits update definition only against the Chart's current source and an
expected definition revision; embedded historical Plot data cannot replace live data.
Data replacement remains a separate revision-fenced transaction. Factor structural
build validation from execution and portable/destination capability checks; valid
native-only registrations must not be rejected by a reused portable constructor.
Retain exact registries through Plot/Chart/capture without serializing implementations.

Export preserves Presented and Current capture bases independently of navigation
projection and interaction inclusion. FigureRequest acquisition remains cheap;
ExportQueue/ExportJob retain deferred execution and resource bounds. The plan's
current-library review reconciliation specifies the regression cases and preserves
legacy capture defaults. These clarify AUT contracts without changing envelope versions.

Reserve the labels builder for x/y-positioned annotations. Plot title, subtitle,
x/y axis labels and legends have separate builders that share the existing text and
guide machinery. The plan's section 3.1 owns the proposed component routes; the earlier
combined labels bag is superseded by this explicit owner direction.

The plan's section 3.3 names builder routes for implemented components, including
captions/notes/panel letters, callouts, insets, statistics/positions, facets and runtime
configuration. For every future feature, extend its existing builder or add a focused
component using the same typed slots and shared engine. Primary usage, applicable
bindings/serialization updates and behavioral evidence land in the feature's own
package. Keep runtime actions/queries as Chart/destination methods and ordinary flags
as options; do not mirror every internal struct with another builder.

Make the primary API the default for all production-facing recipes/docs/examples.
Retain specialist introspection/extension APIs and staged compatibility forwarding
paths. Hide internal implementation details only at an allowed compatibility boundary;
Rust moves must not silently rename wire fields or change statistical semantics.

## Relationship to earlier decisions

This refines ADR-002's previously unfrozen authoring boundary and ADR-006's session
ownership by making typed execution primary and JSON a boundary adapter. It retains
ADR-004 data identity, ADR-007 action ownership and ADR-012 extension contracts.
ADR-010's no-extra-façade-package decision remains: a new crate is unnecessary for the
chosen public surface. No dependencies, licenses, package publication decisions or
wire versions change in this planning task.

The completed-WP premise is prospective, not a new certification of the live checkout.
G-AUTH applies to a release incorporating this refactor; it does not retroactively
invalidate or fabricate historical WP/G4 evidence.

## Alternatives and consequences

A small recipe wrapper with raw-DTO fallbacks would leave advanced capabilities on a
second public authoring path and does not meet the owner instruction. Replacing the
engine would duplicate proven semantics and expand this refactor unnecessarily.
Mirroring every low-level struct as a public fluent object would retain its complexity.
Use concise defaults and typed optional components with complete capability coverage.

The costs are migrating all consumers and proving field/default/identity equivalence,
host behavior and sustained performance. AP-00–09 and G-AUTH make that work explicit.
No UI/runtime success is claimed by this ADR or its API sketches.
