# Infrastructure validation — 6 September 2026

Scope: INF-01–INF-05, infrastructure portions of WP-01/02 only.
Working directory: repository root. Starting commit: `19f4a27`; changes uncommitted.
Environment: macOS Apple Silicon, Rust/Cargo 1.97.1, Python 3.9.6.

## Original bootstrap checks (before mise migration)

| Command/check | Result | Evidence boundary |
| --- | --- | --- |
| `cargo generate-lockfile --offline` | PASS | Seven local packages; no registry/Git library dependencies. |
| `bash scripts/check.sh bootstrap` | PASS | Metadata/boundaries/local file links, rustfmt, workspace compilation, Clippy with warnings denied, rustdoc with warnings denied. |
| `bash scripts/check.sh core` | PASS, zero unit tests and zero doc tests | Test invocation works; no semantic evidence. |
| `bash scripts/check.sh portable` | PASS | `chart-core` compiles for `wasm32-unknown-unknown`; no runtime proof. |
| `cargo check -p chart-gallery --features kit --locked` | PASS | Optional local package wiring only; no GPUI/Kit dependency compatibility proof. |
| `bash -n scripts/check.sh` | PASS | Shell syntax. |
| Bundled skill-creator `quick_validate.py` on each local skill | PASS | Frontmatter/structure; no independent agent execution. |
| `bash scripts/check.sh desktop`, `export`, `bindings`, `release` | Expected exit 2 for each | Correct blocked behavior; capabilities not implemented. |
| `bash scripts/check.sh unknown` | Expected exit 64 | Invalid mode rejected. |
| `cargo run -p chart-gallery --locked` | Expected exit 2 with unavailable message | No fake native-gallery success. |
| Temporary repository copy: add `chart-python -> chart-wasm` and a missing README link | Both rejected | Negative validation probes; no mutation of real package contracts. |
| Local YAML structure/pinned-action and 23-row ledger assertions | PASS | Static configuration validation; no GitHub Actions execution. |
| `git diff --check` | PASS | Tracked patch whitespace validation. |

The shell-dispatcher commands above are historical evidence; that dispatcher was
subsequently replaced by mise tasks. Commands ran from the repository root unless noted. Skill validation used
`python3 /Users/jeickmeier/.codex/skills/.system/skill-creator/scripts/quick_validate.py`
with `.agents/skills/chart-work-package` and `.agents/skills/chart-contract-review`.
That bundled authoring validator is not a repository/CI dependency. See
[AI development](../ai-development.md) for the manual routing walkthroughs and sources.

## Mise migration validation

The user's follow-up selected mise as the project manager with a minimal task set.
`mise.toml` now owns Rust 1.97.1, Python 3.14.6 and the three tasks `fmt`, `check` and
`test`. The separate Rust toolchain file and shell dispatcher were removed. CI uses
the same tasks through pinned mise 2026.8.3 and
[mise-action v4.2.4](https://github.com/jdx/mise-action/commit/7e36c90d9ab29c415a2384db3006f3ec8a8cc654).

- `mise tasks ls`: exactly `fmt`, `check`, `test`.
- `mise install rust python`: PASS; mise repaired a stale local Python build cache.
- `mise exec -- rustc --version` / `mise exec -- python3 --version`: selected pins verified.
- `mise run fmt`: PASS.
- `mise run check`: PASS, including existing checks and core WASM compilation.
- `mise run test`: PASS across all seven packages, with zero unit/doc tests.
- `mise exec aqua:rhysd/actionlint@1.7.12 -- actionlint .github/workflows/ci.yml`: PASS;
  used an already-installed validator without adding a project dependency or task.

Local validation ran on macOS Apple Silicon. Hosted CI remains unrun. The initial
sandboxed trust attempt could not write mise's local state; project trust was then
registered with the necessary filesystem permission. Task runs passed despite initial
config-tracking warnings; provisioning completed the tracking setup and subsequent
task discovery ran without those warnings.

## Review and limitations

The seven package manifests, lockfile, command dispatcher, CI configuration and local
links were inspected. The checkout action is pinned to the verified upstream
[v4.2.2 revision](https://github.com/actions/checkout/commit/11bd71901bbe5b1630ceea73d27597364c9af683).
All crate sources contain documentation only, except the explicitly unavailable gallery
entry point. No normative statistical or rendering requirements were changed.

GitHub Actions has not executed: no remote is configured. Linux checks, native UI,
screen-reader exposure, renderer/font fidelity, actual Python/WASM execution and all
PERF workloads remain unverified. Dependency checking covers declared workspace edges
and known forbidden names in the default resolved graph; additional feature/target
combinations and unlisted dependency behavior still require review. Link checks cover
local file targets, not remote availability or Markdown anchors.

INF-01–INF-05 local infrastructure acceptance is complete. WP-01/02 remain partial,
as recorded in the [status ledger](../implementation-status.md); exact host dependency
selection and minimal behavioral contracts remain the next work.

No chart feature, library semantic fixture, native renderer, export fidelity, binding
runtime, performance or G0–G4 gate is certified by this infrastructure report.
