# WP-01 completion evidence — 6 September 2026

Verdict: **DONE for WP-01**. The dependency selection/build acceptance missing in the
[earlier review](wp-01-review-2026-09-06.md) now passes. This does not complete WP-02,
the production scope of any shared requirement, or G0–G4.

Scope: WP-01; SCP-01/02/03, ARC-04 and QLT-05 bootstrap deliverables. Starting revision:
`fbc97826f8bb2571942f7382168576ba7277108f`, with the preceding review's uncommitted
AGENTS/status/report edits preserved. Result: that revision plus uncommitted WP-01
manifests/lockfile, host example, validation/CI/policy and documentation changes.
No core contracts, chart semantics, exporter or binding APIs were implemented.

## Acceptance

| WP-01 deliverable | Verdict | Evidence |
| --- | --- | --- |
| Authoritative documents and package/gate ledger | PASS | Three handoff documents, all 23 package rows and separate G0–G4 states; latest status points to WP-02. |
| Host/toolchain/naming/integration constraints | PASS | ADR-001 records standalone macOS Apple Silicon, verified tools, no existing application integration, and one exact upstream GPUI identity. ADR-010 retains naming and unpublished-package policy. |
| Reproducible dependency setup | PASS | Exact registry pins and lockfile; standalone and Kit examples compile and link. Kit's root/button and initializer consume the same GPUI types. |
| Standalone and portable isolation | PASS | Resolved graph has one `gpui-pre` 0.3.3, no Kit in the standalone adapter closure and no host dependency in core/export. Core WASM compile passes. |
| Required/proof/future feature matrix | PASS for scope documentation | Support matrix distinguishes build proofs from absent chart behavior and future distribution products. |
| Provenance and benchmark assumption | PASS for bootstrap scope | ADR-001 records package sources, revisions and licenses; lockfile retains checksums. ADR-010 and fixture conventions retain source/notice expectations. Benchmark class remains Apple Silicon at approximately 1,200 logical pixels; measurements belong to WP-03. |

## Environment and executed checks

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Observed environment: macOS 26.5.2 arm64; Xcode 26.6 (17F113), macOS SDK 26.5;
mise 2026.8.3, Rust/Cargo 1.97.1, Python 3.14.6, cargo-deny 0.19.8.
The native examples use the selected runtime-shader path. No window was launched or
visually inspected for this build-only acceptance.

| Command | Result / retained local output |
| --- | --- |
| `mise exec -- cargo info gpui-kit@0.6.0`, `cargo info gpui-pre@0.3.3`, `cargo info gpui-pre-platform@0.3.3`, `cargo info gpui-component@0.6.0` (each through mise) | Published sources fetched and inspected. Exact adopted identities and provenance are in ADR-001. |
| `mise exec -- cargo generate-lockfile` | PASS; registry checksums retained in `Cargo.lock`; no Git sources or patches. |
| `mise exec -- cargo build -p chart-gallery --example host_bootstrap --locked` | PASS, linked standalone host example; `artifacts/wp-01/standalone-build.log`. |
| `mise exec -- cargo build -p chart-gallery --example host_bootstrap --features kit --locked` | PASS, linked Kit host example; `artifacts/wp-01/kit-build.log`. |
| `mise exec -- cargo metadata --format-version 1 --locked --filter-platform aarch64-apple-darwin` | PASS; one registry `gpui-pre` 0.3.3 identity, 581 resolved macOS nodes; `artifacts/wp-01/metadata-macos.json`. |
| `mise run fmt` | PASS. |
| `mise run check` | PASS: repository graph/links, licenses/sources, formatting, both linked host examples, workspace compilation, Clippy/rustdoc with warnings denied and core WASM compilation; `artifacts/wp-01/check.log`. |
| `mise run test` | PASS with zero unit/doc tests; no chart semantic evidence; `artifacts/wp-01/test.log`. |
| `mise exec -- cargo clippy -p chart-gallery --example host_bootstrap --features kit --locked -- -D warnings` | PASS for the optional example branch; `artifacts/wp-01/kit-clippy.log`. |
| `mise exec aqua:rhysd/actionlint@1.7.12 -- actionlint .github/workflows/ci.yml` | PASS; one-off installed validator, not another project task. Hosted CI was not run. |
| `mise exec -- python3 scripts/check_repository.py`, `git diff --check` | PASS after final documentation/status updates; repository boundaries/identities/local links and tracked patch whitespace. |
| `mise exec -- cargo deny --locked check advisories` | FAIL, six unmaintained dependencies listed below; exit 1; `artifacts/wp-01/advisories.log`. No advisory IDs ignored. |
| `mise exec -- cargo report future-incompatibilities --id 1` | Recorded upstream `block` 0.1.6 warning; `artifacts/wp-01/future-incompatibilities.log`. |

The example executable is `target/debug/examples/host_bootstrap`; the second build
replaces it with the Kit variant. Rebuild with the corresponding command to reproduce
either variant. Build logs and metadata are ignored local artifacts, not durable release
certification; this report and the locked source setup are the retained repository evidence.

## Dependency review and open work

The source/license check passes under the new `deny.toml` policy, including exact
MPL-2.0 allowances for unmodified `cbindgen` 0.28.0 and `option-ext` 0.2.0. The initial
license scan identified these and the 0BSD/bzip2 license entries missing from the initial
policy; their published metadata was inspected before recording the selected policy.
This does not choose a license for the local workspace or authorize distribution.

RustSec database revision: `5a0ebedfe8bdd2e295b171f4162f8c977bcad9a5`.
The scan's six findings are maintenance advisories; it reported no vulnerability-class
advisory for this selected macOS graph at that database revision. The command still fails.

| Transitive package | Advisory | Next action |
| --- | --- | --- |
| `bincode` 1.3.3 | [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141) | Track upstream replacement; no compatible fixed upgrade reported. |
| `instant` 0.1.13 | [RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384) | Track upstream replacement; no compatible fixed upgrade reported. |
| `paste` 1.0.15 | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436) | Track upstream replacement; no compatible fixed upgrade reported. |
| `rustls-pemfile` 2.2.0 | [RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134) | Track upstream replacement; no compatible fixed upgrade reported. |
| `rustybuzz` 0.20.1 | [RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206) | Reassess native typography dependencies during WP-03; no compatible fixed upgrade reported. |
| `ttf-parser` 0.25.1 | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192) | Reassess font dependencies during WP-03; no compatible fixed upgrade reported. |

Cargo also reports `block` 0.1.6's uninhabited static as a future Rust incompatibility.
The current pin builds, but compiler upgrades require renewed native verification.
These risks are retained in ADR-001 and the ledger for WP-03/WP-23; no alert is suppressed
and no release gate is passed. Upstream native use of font/export libraries does not
adopt or validate a `chart-export` implementation.

Linux execution, hosted CI, native window behavior, primitive/font fidelity, accessibility,
actual Python/WASM runtimes and PERF workloads remain unverified. Next assigned slice
can complete WP-02's minimal contracts/diagnostics with meaningful tests. WP-03/G0 still
requires actual native/export/font proofs and the benchmark protocol.
