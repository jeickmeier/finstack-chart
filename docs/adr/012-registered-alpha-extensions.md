# ADR-012 — Explicit alpha extension registration

Date: 7 September 2026. Status: Accepted for WP-14.

## Decision

Keep stat/geom implementations at the preparation boundary in an immutable explicit
registry. Use exact qualified operation IDs, positive versions and bounded declarative
parameters. Validate generated schemas, provenance, paint and non-paint interaction
metadata before accepting results. Portable sessions resolve only compiled known
registrations. Native painters live in the GPUI adapter; the core scene stores only a
closed numeric descriptor, and headless export rejects unsupported painters explicitly.

Use the builtin resolved scales and Cartesian coordinate capability API for extensions;
custom guides consume those transforms. Keep full-recompute fallback declarations honest:
no specialized custom update capability is accepted without an executor and batch proof.
The [extension contract](../extension-contract.md) specifies input/output, invalidation,
limits and diagnostics. The [alpha matrix](../alpha-api.md) defines milestone coverage.

## Rationale and consequences

This preserves one core engine for recipes, bindings, custom marks and exports without
moving callbacks, dynamic plugins or interpreter/window objects into the portable scene.
A separate public-API example crate proves the intended third-party authoring boundary;
only proof features/dev dependencies reference it from consumers. No new third-party
dependency version is selected. Native extensions remain trusted code with documented
budgets; output validation cannot prevent allocations inside a caller's implementation.

Rust enum variants and public struct fields change during alpha and affect exhaustive
matches/literal construction. Portable envelope version 1 adds optional fields and known
operation descriptors; old default constructors retain an empty extension registry.
This is an internal alpha boundary, not a package publication or production release.
