# Package infrastructure setup

Date: 6 September 2026. Scope: repository/package/AI infrastructure only.
Contract version: 0.1.0. This supplements the [library implementation plan](gpui-charts-implementation-plan.md)
without changing [normative requirements](../spec/gpui-charts-specification.md).

## Outcome and scope boundary

Prepare a reproducible workspace that makes ownership, required evidence and the next
implementation step clear. Package shells contain crate documentation only; the gallery
explicitly reports unavailable functionality. IDs, revisions, diagnostics, scene types,
builders, statistics, rendering, schemas and bindings belong to later implementation.
Infrastructure completion cannot close WP-02, G0 or any behavioral requirement.

Initial inspection found one clean commit (`19f4a27`), three handoff documents, a
dependency-free Rust 2024 binary and no agent rules, CI, scripts or declared license.
The observed host is macOS Apple Silicon with Rust 1.97.1 and Python 3.9.6.

## Setup sequence and deliverables

| Slice | Deliverables | Requirement alignment | Acceptance |
| --- | --- | --- | --- |
| INF-01 Documentation and ledger | Repair companion links/kickoff paths; README; WP/gate ledger; support matrix; ADR-001/010 | WP-01, SCP-01/02/03, ARC-04, QLT-05/06 | All local file links resolve; state separates planned scope from verified support. |
| INF-02 Cargo workspace | Seven named boundaries; virtual root; minimal crate shells; shared metadata/lints; pinned toolchain and lockfile | Infrastructure subset of WP-02, ARC-01/02/04 | Metadata has the intended members/edges; no third-party dependencies; compile, lint and docs checks pass. |
| INF-03 Validation and CI | Three mise tasks; stdlib Python boundary/link checker; macOS/Linux CI; portable target check; PR template | WP-02 infrastructure, SCP-03, QLT-02/05/06 | Local checks execute; unavailable capabilities have no placeholder tasks; CI execution evidence is recorded separately. |
| INF-04 AI workflow | Root AGENTS.md; two repository-local skills; authority/routing guide | WP-01/02 handoff protocol | Rules match actual commands; skills have valid metadata and resolvable references; realistic scope walkthroughs preserve the assignment. |
| INF-05 Development resources | Fixture/provenance conventions; benchmark runner placement; evidence retention rules | QLT-02/03/04/05 | Clear homes for later data/fonts/benchmarks without fabricated fixtures or performance results. |

Current completion and commands are recorded once in the [status ledger](../implementation-status.md).

## Workspace and dependency policy

Use the specification's names internally while keeping `finstack-chart` as the repository
name. No root facade is justified yet. Select public names before release preparation.

| Package/path | Initial direct workspace dependencies | Later integration |
| --- | --- | --- |
| `crates/chart-core` | None | Portable synchronous computation; optional serialization evaluated in WP-02/09. |
| `crates/chart-export` | `chart-core` | Font/SVG/PDF/PNG route selected by WP-03 evidence. |
| `crates/gpui-charts` | `chart-core` | One exact GPUI package/source compatible with the intended host. |
| `crates/gpui-charts-kit` | `chart-core`, `gpui-charts` | Matching optional Kit; no second GPUI identity. |
| `crates/chart-python` | `chart-core`, `chart-export` | PyO3 plus executable WP-09 proof; no wheels/viewer now. |
| `crates/chart-wasm` | `chart-core` | wasm-bindgen plus executable WP-09 proof; basic SVG route chosen then. |
| `examples/chart-gallery` | Core/export/GPUI; optional Kit via `kit` | Native proof, visual QA and stress scenarios; never a production dependency. |

Default members serve the standalone desktop path without Kit, gallery or binding
adapters. Empty host shells can compile on Linux during bootstrap; this is not GPUI
platform evidence. Once real native dependencies land, narrow Linux CI to core/export
and keep native checks on the verified macOS target. Make that CI change in the same
dependency slice, before a platform-incompatible workspace command reaches Linux.

Select exact host dependencies in ADR-001 before adding public host types. Registry
host pins use an exact version; Git sources use a full revision. Commit Cargo.lock.
Add font/export dependencies only as a capability experiment until required fixtures
justify adoption. Do not add speculative Arrow, Polars, async, finance or reference
R/JavaScript dependencies. Review normal/build/dev and target/feature-specific edges.
The checker catches known forbidden packages in the resolved graph, not every possible
host dependency or semantic violation; review remains necessary.

## Tooling choices

- Pin Rust 1.97.1 in `mise.toml` and declare the same conservative initial
  `rust-version`. This is not a claim that dependencies or an older MSRV were tested.
  Reconcile the minimum compiler after GPUI/font selection at G0.
- Use mise for tool provisioning and exactly three development tasks: `fmt`, `check`
  and `test`. Pin Python 3.14.6 for the standard-library metadata/link checker. The
  separate `rust-toolchain.toml` and shell task dispatcher are removed; `mise.toml`
  owns tool selection and commands. Use `mise exec -- cargo ...` for focused commands.
  Extra task runners, pre-commit installation and custom xtask crates are unnecessary.
- Inherit workspace lints in every member: deny unsafe code, warn for missing docs and
  Clippy defaults; CI treats warnings as errors. Any later FFI exception must be local,
  justified and tested, without relaxing portable-core rules globally.
- Keep all packages unpublished. License, copyright attribution, registry availability
  and public source URL are unresolved release preparation decisions; do not invent them.
- Use a minimal GitHub Actions workflow with read-only permissions, pinned checkout
  and mise actions, and mise 2026.8.3. CI installs tools from `mise.toml` and invokes the
  same `check` and `test` tasks as local development. No publishing credentials, deployment jobs, automated baseline acceptance or
  paid AI calls. No remote is configured in the initial repository; CI is provisioned,
  not claimed to have executed.

## Validation rollout

| Command | Active now | Completion needed later |
| --- | --- | --- |
| `mise run fmt` | Format workspace Rust source | None. |
| `mise run check` | Graph/links, format check, workspace/Kit compilation, Clippy, rustdoc and core WASM compilation | Adapt host matrix when dependencies arrive. |
| `mise run test` | Workspace unit/doc tests; currently zero | Meaningful WP-02 and later canonical semantics. |

Desktop, export, bindings and release acceptance runners remain unimplemented. Add
scoped mise tasks with their actual WP-03/07, WP-03/08, WP-09 and WP-23 runners;
there are no placeholder tasks or successful stand-ins for those gates.

Do not use a blanket `--all-features` matrix for target-specific adapters. Add named
valid feature combinations when implemented: standalone desktop, desktop plus Kit,
headless formats, minimal Python, single-thread WASM. The `check` task also validates the gallery's
declared Kit feature. Record skipped/missing environments as unverified or
blocked. No zero-test count is a semantic pass. Add dependency advisory/license tooling
when selecting third-party dependencies; include its pinned version and policy then.

## AI infrastructure

Use [AGENTS.md](../../AGENTS.md) for always-relevant authority, package boundaries,
commands and evidence discipline. Use instruction-only local skills for repeated work:
`chart-work-package` for a bounded authorized slice and `chart-contract-review` for a
read-only requirement/evidence review. Both link the existing documents; neither copies
the specification or chooses a model. [AI development](../ai-development.md) defines
discovery and example routing.

Do not add nested instruction files until a crate needs distinct guidance. No global
agent settings, permission overrides, MCP servers, subagent fleet or mandatory agent
approval checkpoints are needed. Human task scope still controls whether to plan,
review, scaffold or implement. A skill must not turn a review into code changes.

## Resource and evidence conventions

Canonical fixture IDs and independent expected values remain in the specification;
actual stored cases and resource provenance go in [fixtures](../../fixtures/README.md)
as their packages implement them. Benchmarks belong to the owning crate with shared
protocol guidance in [benches](../../benches/README.md). Small reviewed evidence belongs
under `docs/evidence/`; generated bulky output goes under ignored `artifacts/` and needs
a durable retained location before it can support a release claim.

## Next implementation handoff

1. Complete WP-01: verify a host-compatible GPUI/Kit source pair, record exact pins and
   the benchmark machine assumption in ADR-001 and the support matrix.
2. Complete the behavioral remainder of WP-02 after that prerequisite: finite geometry,
   IDs/revisions, diagnostics, minimal scenes and host service contracts, with meaningful
   checks. Keep these public surfaces small and driven by WP-03/04 consumers.
3. Run WP-03 capability proofs and WP-04 data work when their dependencies are ready.
   Only actual renderer/font evidence and decisions can close G0.

Infrastructure acceptance ends here. G0–G4 and all library behavior remain open.
