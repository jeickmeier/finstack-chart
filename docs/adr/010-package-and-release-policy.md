# ADR-010: Package naming, provenance and release policy

Status: ACCEPTED for local setup; publication choices OPEN.
Date: 6 September 2026. Owners: WP-01 and WP-23. Requirements: QLT-05, ARC-04.

Keep `finstack-chart` as the repository name. Use the seven ARC-01 package names in a
virtual workspace; a root facade would duplicate an unproven API boundary. All packages
share version 0.1.0, Rust 2024 and `publish = false`. Retain the workspace lockfile.
Python/WASM are ordinary nonfunctional Rust crate shells until WP-09 selects actual
binding tooling and distribution targets.

Coordinate library versions initially. Before 1.0, document breaking API changes in
the changelog and version/migrate affected portable schemas explicitly. Schema and
operation versions have their own semantics and do not change automatically with every
crate patch. Create release changelog entries when actual public APIs exist.

No license grant, copyright owner, registry-name availability or remote source URL was
supplied. Do not infer an open-source license or publish these packages. Resolve those
fields before any distribution. Record source/revision/license and transformations for
borrowed algorithms, fixtures and fonts when introduced; retain required notices and
font embedding permissions. The bootstrap imports no reference source or font assets.

At WP-23, inspect local package contents, reconcile public names/metadata and prepare a
release candidate only after cumulative gates pass. Publishing is a separate action.
