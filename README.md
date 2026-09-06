# finstack-chart

Rust-native grammar-of-graphics project with a portable core, headless publication export,
a standalone GPUI host and optional Kit integration.

**Status: WP-03 capability spike complete; G0 architecture/capability gate passed for
the initial macOS host. Actual native and SVG/PDF/PNG proofs are retained. Public chart
rendering/export APIs, chart compilation and bindings remain unimplemented; G1–G4 are open.**

## Start here

1. [Specification](docs/spec/gpui-charts-specification.md): authoritative behavior and requirements.
2. [Implementation plan](docs/impl_plans/gpui-charts-implementation-plan.md): WP-01–WP-23 and gate order.
3. [Infrastructure plan](docs/impl_plans/package-infrastructure-plan.md): setup decisions and remaining infrastructure work.
4. [Implementation status](docs/implementation-status.md): evidence, blockers and next action.
5. [Support matrix](docs/support-matrix.md): measured support versus intended support.

[Migration rationale](docs/spec/gpui-charts-migration-plan.md) provides dated reference
research. It does not override the specification or select dependency versions.

## Development

Use mise 2026.8.3 or newer. [mise.toml](mise.toml) manages Rust 1.97.1, rustfmt, Clippy,
the `wasm32-unknown-unknown` target, Python 3.14.6 and cargo-deny 0.19.8.
Cargo uses the checked-in workspace lockfile. [ADR-001](docs/adr/001-host-dependency-and-toolchain.md)
records GPUI 0.3.3 / Kit 0.6.0 package identities, features and the verified macOS/Xcode
environment. Native development requires a full selected Xcode installation.

```sh
mise trust
mise install
mise run fmt
mise run check
mise run test
```

The three development tasks are `fmt` (format Rust), `check` (repository boundaries,
local file links, dependency licenses/sources, formatting, compilation, Clippy, rustdoc
and core WASM compilation), and `test` (macOS workspace or Linux core/export tests).
The core suite checks identities/revisions, finite geometry, bounded scene construction
and synchronous service boundaries; it does not certify chart statistics or rendering.
Use `mise exec -- cargo ...` for one-off Cargo commands, or activate
mise in your shell for editor and terminal tool selection.

The macOS `check` task also builds the standalone and Kit `host_bootstrap` examples,
and builds/lints the Kit-gated native capability example.
`mise exec -- cargo run -p chart-gallery` still exits 2. See ADR-001 for the separate
host-example commands. CI checks native builds on macOS and headless packages on Linux;
hosted CI and real binding execution remain unverified. The
[WP-03 report](docs/evidence/wp-03-completion-2026-09-06.md) records actual native visual,
input/lifecycle/accessibility-hook inspection and publication artifacts. These remain
proof examples; use the [fixture instructions](fixtures/capability/README.md) to run them.

Run `mise exec -- cargo deny --locked check advisories` when reviewing dependencies.
The current scan fails on six unmaintained transitive packages; no advisory is ignored.
The refreshed scan and remaining release risks are in the [WP-03 evidence](docs/evidence/wp-03-completion-2026-09-06.md).

## Packages and contributions

The first usable core contracts are documented in
[ADR-002](docs/adr/002-minimal-core-contracts.md). The
[public Rustdoc example](crates/chart-core/src/lib.rs) constructs a small validated scene
without a host runtime. Run `mise exec -- cargo test -p chart-core --locked` for the
contract suite and example. [Changes](CHANGELOG.md) and
[WP-02 evidence](docs/evidence/wp-02-completion-2026-09-06.md) record compatibility and
the precise validation boundary.

The root is a virtual Cargo workspace named by this repository, not a `finstack-chart`
facade crate. Package ownership follows ARC-01. Default members are `chart-core`,
`chart-export` and `gpui-charts`; Kit, gallery and binding proofs are opt-in. Every
package has `publish = false`. License selection and public package-name availability
remain open in [ADR-010](docs/adr/010-package-and-release-policy.md).

Read [AGENTS.md](AGENTS.md) for contributor instructions. Repository-local
`chart-work-package` and `chart-contract-review` skills provide scoped execution and
review workflows; see [AI development](docs/ai-development.md). No API keys, paid model
calls or external reference runtimes are needed for infrastructure checks.
