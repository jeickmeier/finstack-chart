# ADR-010: Package naming, provenance and release policy

Status: ACCEPTED for local setup; publication choices OPEN.
Date: 6 September 2026. Owners: WP-01 and WP-23. Requirements: QLT-05, ARC-04.

Keep `finstack-chart` as the repository name. Use the seven ARC-01 package names in a
virtual workspace; a root facade would duplicate an unproven API boundary. All packages
share version 0.1.0, Rust 2024 and `publish = false`. Retain the workspace lockfile.
Python/WASM now execute proof adapters through PyO3 0.29.2 and wasm-bindgen 0.2.128.
They are not distributed wheels/npm packages. The workspace also contains the shared
`chart-text` service and external extension example; all nine package identities remain
local and unpublished.

Coordinate library versions initially. Before 1.0, document breaking API changes in
the changelog and version/migrate affected portable schemas explicitly. Schema and
operation versions have their own semantics and do not change automatically with every
crate patch. Create release changelog entries when actual public APIs exist.

No license grant, copyright owner, registry-name availability or remote source URL was
supplied. Do not infer an open-source license or publish these packages. Resolve those
fields before any distribution. Record source/revision/license and transformations for
borrowed algorithms, fixtures and fonts when introduced; retain required notices and
font embedding permissions. Supplied Noto/Fira fonts retain their license/notice files in fixture directories;
ADR-003 records the shaping/export route and exact dependency identities. Cargo.lock
retains registry checksums. The release source archive includes these notices.

WP-23 prepares an **unpublished local source candidate** and an exact evidence index.
It does not promote the open expanded parity/authoring gates to production acceptance.
The owner limited this assignment to original WP-11–23 and explicitly waived the long
30-minute test. Numerical budgets and unresolved evidence remain visible.

Use `scripts/package_local_release.py` to archive an exact committed tree with a
deterministic gzip header and SHA-256 inventory. It includes all nine workspace
packages, lockfile, fixture fonts/notices and committed documentation; uncommitted
concurrent planning and build products stay outside the candidate. A source archive
does not bypass Cargo publish guards, select a license, or publish a release.
Distributable registry packages remain a later owner-authorized action.
