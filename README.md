# finstack-chart

Rust-native grammar-of-graphics project with a portable core, headless publication export,
a standalone GPUI host and optional Kit integration.

**Status: infrastructure scaffold. No chart library, native gallery, exporter or language
binding is implemented. No G0–G4 release gate has passed.**

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
the `wasm32-unknown-unknown` target and Python 3.14.6 (standard library only).
Cargo uses the checked-in workspace lockfile. There are currently no third-party
Rust dependencies.

```sh
mise trust
mise install
mise run fmt
mise run check
mise run test
```

The three development tasks are `fmt` (format Rust), `check` (repository boundaries,
local file links, formatting, compilation, Clippy, rustdoc and core WASM compilation),
and `test` (workspace tests). Tests currently number zero; these checks do not prove
chart semantics. Use `mise exec -- cargo ...` for one-off Cargo commands, or activate
mise in your shell for editor and terminal tool selection.

Desktop, export, binding and release acceptance runners are not implemented; add tasks
when those capabilities exist. `mise exec -- cargo run -p chart-gallery` exits 2.
CI configuration covers macOS and Linux infrastructure plus portable compilation;
native visual/accessibility checks and real binding execution remain future work.

## Packages and contributions

The root is a virtual Cargo workspace named by this repository, not a `finstack-chart`
facade crate. Package ownership follows ARC-01. Default members are `chart-core`,
`chart-export` and `gpui-charts`; Kit, gallery and binding proofs are opt-in. Every
package has `publish = false`. License selection and public package-name availability
remain open in [ADR-010](docs/adr/010-package-and-release-policy.md).

Read [AGENTS.md](AGENTS.md) for contributor instructions. Repository-local
`chart-work-package` and `chart-contract-review` skills provide scoped execution and
review workflows; see [AI development](docs/ai-development.md). No API keys, paid model
calls or external reference runtimes are needed for infrastructure checks.
