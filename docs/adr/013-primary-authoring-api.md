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

Reuse/refactor the existing typed store/compiler/reducer runtime as the retained Chart
entry point. JSON sessions become interchange adapters around that runtime. Native,
export and language adapters consume the same plot/runtime/snapshot contracts; they
own destination resources and host lifecycle, never an alternate grammar compiler.
The [implementation plan](../impl_plans/primary-authoring-api-plan.md) owns the precise
sequence, capability register, examples and evidence requirements.

Generate routine identities and revisions while preserving durable handles, exact
data, atomic transactions and acknowledged-scene semantics. Preserve baseline default
policies and select reference compatibility profiles explicitly. Builder failures and
runtime failures keep the prior valid state. Native callbacks may materialize data or
use extension protocols but cannot become executable serialized code.

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
