# finstack-chart

Rust-native grammar-of-graphics project with a portable core, headless publication export,
a standalone GPUI host and optional Kit integration.

**Status: original WP-11–23 implementation and local release documentation are complete.**
The shared engine supports the documented families, facets/themes, custom extensions,
indexed interaction, bounded streaming and coherent publication. Actual Rust/Python/Node
WASM fixtures and macOS native/Linux headless evidence are linked in the
[release evidence index](docs/release-evidence.md). The owner waived the 30-minute test;
[WP-22](docs/evidence/wp-22-completion-2026-09-07.md) records short measured performance,
actual GPU/display timing and failed-run limitations.

Current expanded parity/authoring and production G4 gates remain open. Packages are local
and unpublished. Start with the [developer and release guide](docs/release-guide.md) for
build commands, runnable examples, ownership, compatibility and source-candidate packaging.

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

The foundation development tasks are `fmt` (format Rust), `check` (repository boundaries,
local file links, dependency licenses/sources, formatting, compilation, Clippy, rustdoc
and core WASM compilation), and `test` (macOS workspace or Linux core/export tests).
The core suite checks identities/revisions, finite geometry, bounded scene construction
and synchronous service boundaries, plus data/schema, replay, retention and snapshot
ownership contracts. It also verifies identity/explicit-bin statistics, line gaps,
heterogeneous layer geometry and transform reuse. Scale/layout tests add domain policies,
UTC calendar and precision fixtures, categorical identity, measured margins and viewport
separation. Presented-scene inspection verifies clipping, provenance, keyboard steps and
stale-event rejection. Built-in statistics/positions, facets, themes and extensions are also covered; see the alpha matrix.
Use `mise exec -- cargo ...` for one-off Cargo commands, or activate
mise in your shell for editor and terminal tool selection.

The macOS `check` task also builds the standalone and Kit `host_bootstrap` examples,
and builds/lints the Kit-gated native capability example.
Run `mise exec -- cargo run -p chart-gallery --locked` in a macOS graphical session for
the standalone gallery (no Kit dependency required). Move over marks, click the chart and
use arrow keys/Escape; controls exercise zoom, malformed input, missing fonts, tiny bounds
and remount. [WP-07 evidence](docs/evidence/wp-07-completion-2026-09-06.md) records the actual
native checks. See ADR-001 for the separate host-example commands. CI checks native builds on macOS and headless packages on Linux;
hosted CI remains unverified. Actual Python/Node WASM proof execution is recorded in WP-09. The
[WP-03 report](docs/evidence/wp-03-completion-2026-09-06.md) records actual native visual,
input/lifecycle/accessibility-hook inspection and publication artifacts. These remain
proof examples; use the [fixture instructions](fixtures/capability/README.md) to run them.

The [portable contract](docs/portable-contract.md) documents strict version 1 envelopes and
the minimal Python/WASM API. Run `mise run bindings-proof` with Node, Poppler and an exact
wasm-bindgen-cli 0.2.128 on PATH (or set `WASM_BINDGEN`); the
[fixture instructions](fixtures/bindings/README.md) explain setup and the actual runner.
[WP-09 evidence](docs/evidence/wp-09-completion-2026-09-06.md) records three-runtime results,
large-integer/time precision, invalid memory-view tests and current support limits.

Run `mise exec -- cargo deny --locked check advisories` when reviewing dependencies.
The current scan fails on six unmaintained transitive packages; no advisory is ignored.
The refreshed scan and remaining release risks are in the [WP-09 evidence](docs/evidence/wp-09-completion-2026-09-06.md).

The [publication example](crates/chart-export/examples/publication_export.rs) demonstrates
`FigureSnapshot::capture` and bytes-only SVG/PDF/PNG export. See the
[fixture instructions](fixtures/publication/README.md) for headless generation, independent
artifact checks and the native publication preview, and the
[WP-08 report](docs/evidence/wp-08-completion-2026-09-06.md) for inspected results and limits.

## Packages and contributions

The core and authoring contracts are documented in
[ADR-002](docs/adr/002-minimal-core-contracts.md) (including WP-05 compiler decisions) and
[ADR-004](docs/adr/004-immutable-data-and-transactions.md). The latter specifies typed/column
snapshots, ordered atomic commits, bounded replay and provenance, with
[WP-04 evidence](docs/evidence/wp-04-completion-2026-09-06.md). The
[public Rustdoc example](crates/chart-core/src/lib.rs) constructs a small validated scene
without a host runtime. Run `mise exec -- cargo test -p chart-core --locked` for the
contract suite and example. [Changes](CHANGELOG.md) and
[WP-02 evidence](docs/evidence/wp-02-completion-2026-09-06.md) record compatibility and
the precise validation boundary. The [grammar Rustdoc example](crates/chart-core/src/grammar/mod.rs)
builds a typed histogram through the shared compiler; [WP-05 evidence](docs/evidence/wp-05-completion-2026-09-06.md)
records the generated-schema, domain, provenance and state contracts. The same example now
continues through destination layout. [ADR-005](docs/adr/005-foundational-scales-and-layout.md)
and [WP-06 evidence](docs/evidence/wp-06-completion-2026-09-06.md) record scale, tick, clip,
font-metric and layout policies. The [gallery source](examples/chart-gallery/src/main.rs)
shows `NativeFont::load` before rendering, `ChartInput::new`, a retained `ChartView` entity,
and caller-owned tooltips. Register font bytes before GPUI first resolves their family;
reserve that family's supplied faces for the adapter. Native metrics and painting share
GPUI's text system; headless publication has a separate destination bridge.

The root is a virtual Cargo workspace named by this repository, not a `finstack-chart`
facade crate. Package ownership follows ARC-01. Default members are `chart-core`,
`chart-export` and `gpui-charts`; Kit, gallery and binding proofs are opt-in. Every
package has `publish = false`. License selection and public package-name availability
remain open in [ADR-010](docs/adr/010-package-and-release-policy.md).

Read [AGENTS.md](AGENTS.md) for contributor instructions. Repository-local
`chart-work-package` and `chart-contract-review` skills provide scoped execution and
review workflows; see [AI development](docs/ai-development.md). No API keys, paid model
calls or external reference runtimes are needed for infrastructure checks.

The [state/action contract](docs/state-action-contract.md) covers origins, controlled
revision fences, transient previews, bounded annotation undo and explicit scene ownership.
Run `mise exec -- cargo run -p chart-gallery --example actions_gallery --locked` for
native preview/cancel/commit/undo/freeze controls.

Run `mise exec -- cargo run -p chart-gallery --example host_tools_gallery --locked`
for linked charts/table, snapped threshold/range handles and replaceable native controls.
[WP-17 evidence](docs/evidence/wp-17-completion-2026-09-07.md) records actual runtime
checks and the accessibility limitations of the supported native target.

[Streaming contract](docs/streaming-contract.md) and
[WP-18 evidence](docs/evidence/wp-18-completion-2026-09-07.md) document exact replay,
queue acknowledgements, retention/follow semantics and remaining performance gates.

[Scheduling and density contract](docs/scheduling-density-contract.md) and
[WP-19 evidence](docs/evidence/wp-19-completion-2026-09-07.md) cover bounded workers,
monotonic presentation, dense line/candle paint and preliminary CPU/index measurements.

[Live-export contract](docs/live-export-contract.md) and
[WP-20 evidence](docs/evidence/wp-20-completion-2026-09-07.md) cover immutable captures,
interaction inclusion, bounded jobs and release on cancellation/error. Run
`mise exec -- cargo run -p chart-gallery --example live_export_gallery --locked`
for deliberately slowed native SVG/PDF/PNG exports during atomic data updates.
