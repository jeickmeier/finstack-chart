# Repository Guidelines

## Authority and Task Scope

Read [the specification](docs/spec/gpui-charts-specification.md),
[implementation plan](docs/impl_plans/gpui-charts-implementation-plan.md) and
[status ledger](docs/implementation-status.md) before implementation. Explicit owner
instructions lead, followed by the specification, conforming ADRs, implementation plan
and migration rationale. An infrastructure-only assignment does not authorize chart
implementation. The repository contains core data/grammar preparation, scales/layout and host capability
proofs; public native rendering/export and binding APIs remain unimplemented.

## Project Structure & Module Organization

`chart-core` owns shared semantics; `chart-export` owns headless publication;
`gpui-charts` owns native integration; `gpui-charts-kit` is optional. Python/WASM
packages are reserved proof adapters. Recipes and bindings must share the core engine.
Keep GPUI/interpreter/browser objects, compulsory I/O, system fonts, finance engines
and mandatory threading outside core. Export must not require a GPUI event loop.
Record dependency identity and consequential choices in `docs/adr/`; dated migration
versions are not adopted dependencies.

## Build, Test, and Development Commands

Use `mise install` to provision the tools in `mise.toml`. Run `mise run fmt` to format,
`mise run check` for repository/dependency/build/lint/docs/WASM checks, and `mise run test`
for macOS workspace or Linux core/export tests. Run a selected core contract test with
`mise exec -- cargo test -p chart-core TEST_FILTER --locked`. Use the committed lockfile
and inherited workspace lints. Rustfmt
enforces formatting; Clippy and rustdoc warnings fail checks. Add capability tasks only
when their acceptance runners exist.

## Testing and Evidence

Choose relevant requirement/FIX/PERF IDs before changing behavior. Use independent
expected values, operation-specific tolerances and batch-versus-update comparisons.
Never weaken fixtures or regenerate visual baselines to hide an unexplained failure.
Zero tests, compile-only bindings and uninspected images do not pass feature gates.
Record commands, environment, results, artifact paths and limitations in the ledger.

## Handoffs and Review

Preserve unrelated edits and own one reviewable work-package slice. Select the earliest
ready task only within the user's assignment; follow prerequisites. Before ending every
task, update `docs/implementation-status.md` with the outcome, revision, evidence,
unresolved requirements and next action, including partial or blocked work and reviews.
Use the PR template's contract and
evidence fields; the single initial commit establishes no strict message convention.
Use [local skills](docs/ai-development.md) when their workflow applies. Keep rules and
skills concise and link the authoritative contracts instead of copying them.
