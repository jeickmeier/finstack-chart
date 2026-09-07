# WP-23 — Local release documentation and readiness

Date: 7 September 2026. Scope: original WP-23; SCP-01/02/03, ARC-04, BND-01, QLT-05/06.
The owner kept expanded parity/authoring acceptance open and waived the 30-minute test.

This package provides a local developer/release guide, version and source-provenance
policy, changelog, all-original-requirement evidence index and deterministic committed-tree
source-candidate packaging. It does not publish packages or certify expanded production gates.

[The evidence index](../release-evidence.md) maps all 65 original requirement IDs to
implementing/reviewing completion reports, FIX-01–18 to the hardening index and PERF-01–05
to the measured report with its explicit duration exception and failed-run disposition.
[The release guide](../release-guide.md) links executable recipe/layer/custom-extension,
interaction/linking, streaming, themes and publication examples using the common engine.
[ADR-010](../adr/010-package-and-release-policy.md) records the exact unpublished package,
version, license/provenance and archive boundary. All nine packages remain version 0.1.0,
Rust 2024/MSRV 1.97.1 and `publish = false`; proof adapters are not public distributions.

## Validation and candidate

No executable Rust changed after WP-22. Its final **219 macOS / 213 Linux tests**, full
repository/build/lint/docs/WASM checks, performance-feature Clippy and actual
Rust/Python/Node WASM fixtures remain the runtime evidence. WP-23 independently reruns
repository graph and Markdown-link validation after the documentation changes.

The packaging tool was executed twice against committed runtime baseline `c58f5b2`:
**1,102 files / 37,966,920 bytes**, identical archive SHA-256
`bcd29012cdbcca1f3b5684b352637ff5ee785b5bf4dbaac52fa0be7d53d2e1c7`.
Gzip round-trip and a per-file inventory verify exact committed content. The extracted
archive passes offline repository/host-isolation/link checks and fresh all-target
`cargo check -p chart-core -p chart-export -p chart-text --locked --offline` in a separate
target directory. This verifies source completeness without borrowing uncommitted files.
The provisional archive deliberately precedes the WP-23 documentation commit.

The final candidate is generated from the WP-23 commit under `artifacts/local-release`.
Its filename, exact commit, compressed size, SHA-256 and every file hash are recorded
in the adjacent JSON manifest. The post-commit archive includes this report, guide,
changelog, policy and index; no uncommitted planning is implicitly shipped. Its Rust and
Cargo inputs must match the verified baseline, and its extracted repository/link check
is rerun before handoff. No registry package or remote release is created.

`cargo metadata --no-deps --locked` verifies all nine local package identities and
`publish = false`; `cargo package -p chart-core --list --allow-dirty --locked` inspects
Cargo's per-crate listing. The expected missing license/repository/distribution metadata
warning is retained, not waived. The workspace source archive preserves sibling package,
example and fixture relationships that a standalone crate listing cannot prove.

Logs, package identity assertions, reproducibility results and SHA-256 inventory are in
`wp23/`. The guide gives the exact setup, actual binding runner and source packaging
commands. No additional feature or remote-CI result is inferred from these packaging checks.

Current G4/expanded parity and primary-authoring certification remains OPEN. Actual macOS
native and Linux headless support, OS accessibility limits, font/export caveats, dependency
advisories and distribution metadata are explicitly retained in the support matrix and
referenced reports. Concurrent owner planning/review edits are not included in this package.
