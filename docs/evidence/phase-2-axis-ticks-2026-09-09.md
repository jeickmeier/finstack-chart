# WP-AX01 / WP-AX02 integrated axis acceptance — 9 September 2026

Revision: working tree over `51f2edafdf5ec65121966dedcb1d5ffa838f80de`.
Requirements: AXIS-01/02/03/07 and the selection/formatting portion of FIX-19.
The owner narrowed this assignment to these two packages and requested a stop before
WP-AX03. The [ledger](../implementation-status.md) owns the remaining-work list.

## Implemented contract

The explicit per-guide `D3_3_0_0` profile completes the WP-AX01 profile entry contract.
The [provider report](phase-2-axis-provider-2026-09-09.md) retains identity, reference
harness and custom-provider evidence. Default `LibraryV1` behavior and older wire
versions remain covered by regression tests. Complete D3 geometry is WP-AX03.

WP-AX02 independently selects tick arguments, explicit typed values and a formatter.
Resetting one control preserves the others. Explicit empty lists bypass automatic
enumeration; complete order, duplicate values, blank and repeated labels survive.
Numeric and calendar defaults consume shared scale algorithms and locale owners.
Category fallback keeps full domain order and fails hard budgets without subsampling.
Timestamps remain exact signed integers with units through Rust/Python/WASM metadata.

Registered native formatters capture versioned descriptor/parameter and registry
snapshots. Callbacks receive the original value, index and complete selected list.
Input/mapping preflight happens before callbacks, with bounded count and label bytes.
Missing registrations and invalid parameters fail; native-only callbacks reject
portable serialization and headless execution. Version 11 carries new guide controls;
old capabilities retain their existing envelopes. A separate immutable `Frame.guides()`
observation exposes coherent guide specs and semantic ticks with nested scope paths.

## Evidence

- `axis_ticks` replays the pinned 372 browser cases / 376 states in Rust. The actual
  macOS Python and Node/WASM adapters independently replay all cases with exact values,
  order and labels, including two device profiles and four sides. These compare tick
  semantics, not the still-open AXIS-04 geometry.
- Additional core/host checks cover count zero/default hints, independent resets,
  explicit-list bypass of an excessive calendar enumeration, exact nanoseconds above
  2^53, callback context, invalid values before callbacks, portable round trips and
  missing/native-only registrations.
- Actual Python/WASM primary regression proofs pass 23 action transitions, 47 independent
  input steps and a 70-step streaming replay, plus ownership, disposal and retained frames.
  Fresh provider host proofs pass. The shared mutable-owner accessor recursion discovered
  in concurrent host refactoring was corrected to access its stored option; these actual
  mutable-operation regressions validate that fix.
- Strict TypeScript positive and negative cases pass. Mypy positive cases pass and rejects
  the three intentionally invalid calls. Host formatter operation versions use the existing
  exact integer conversion rather than a new representation rule.
- Independently authored Rust/Python/WASM figures produce byte-identical SVG, PDF and PNG.
  Native, PNG and Poppler PDF output were inspected: five marks, distinct indexed labels,
  empty labels preserving tick rules and three repeated labels without clipping. The native
  screenshot and paint instrumentation are retained. External resvg 0.45.1 output was also inspected after configuring the supplied Noto Sans as its font fallback; the SVG itself was unchanged. This inspection does not certify browser font loading.
- The macOS and offline Linux core/export/text/external-extension regressions each completed 430 tests/doctests. Final focused tests pass 16 guide/provider/tick contracts. Native inspection and full regression preceded representation-only boxing of formatter payloads; final actual Python/WASM and focused Rust checks cover the boxed representation. Repository/dependency checks, formatting, native example builds, all-target workspace checking, strict Clippy and rustdoc pass. The last standalone WASM core metadata check stalled in both the native build directory and a separate retry and was interrupted; the aggregate `mise run check` is therefore incomplete, not a pass. The freshly rebuilt actual WASM module and its full axis proof pass. A stale unused Kit import from the concurrent refactor was removed so strict lint remains enabled.

## Commands and retained artifacts

The directory [phase-2-axis-ticks](phase-2-axis-ticks/) contains logs, source/runtime
hashes, independent host records, publication comparisons and inspection evidence.
Reproducible package commands (under `mise exec --` where appropriate):

```sh
cargo test -p chart-core --test axis_guides --test axis_providers --test axis_ticks --locked
cargo run -p chart-export --example axis_tick_proof --locked -- OUTPUT/rust
python3 scripts/bindings/axis_ticks.py PYTHON_MODULE OUTPUT/python
node scripts/bindings/axis_ticks.cjs WASM_MODULE OUTPUT/wasm
cargo test -p chart-core -p chart-export -p chart-text -p chart-extension-example --locked
mise run check
```

The complete `scripts/run_primary_authoring_proofs.py` runner now includes this package's
core, actual host, strict type and publication comparison steps. Its package steps were
executed directly for this acceptance; this does not claim a fresh run of every historical
package in that aggregate runner. WASM generation uses the official task-local
wasm-bindgen CLI 0.2.128. Linux uses the existing offline Rust 1.97.1 Bookworm container,
read-only source/caches and a task build volume. No fixtures or tolerances were weakened.

## Limits and next action

Automatic D3 ticks on the legacy session scale and new controls on secondary conversion
guides are explicitly unsupported. Registered code is trusted native code and cannot be
preempted. The profile does not change scale training or claim D3 geometry defaults.
Range-derived paths, inner/outer sizes, padding/offset and DPI geometry are WP-AX03;
component styling/metadata, transitions and full certification remain WP-AX04–06.
G-AXIS, G-PARITY, G4 and expanded WP-21/22/23 are open. Stop after WP-AX01/02 as requested.


## Final outcome

WP-AX01 and WP-AX02 are COMPLETE for their bounded contract/reference/provider and
selection/formatting acceptance. All selected package behaviors have actual runtime,
reference and inspected destination evidence. No further axis package was started.
The remaining standalone WASM metadata check is an explicit repository requalification
item; it does not turn the completed runtime proof into an aggregate-check pass.
The older provider report's interrupted aggregate runs remain historical. The current
macOS run covers core/export/text/extension tests, not a fresh full native workspace
test run; native execution is supported by the retained inspected example.

Final evidence files: `source-sha256.json`, `runtime-sha256.json`, `environment.json`,
`publication-comparison.json`, `inspection.json`, `final-focused.log`,
`macos-tests.log`, `linux-full-tests.log`, `python-proof.log`, `wasm-proof.log`,
`python-types.log`, `python-types-invalid.log`, `typescript.log`, `check.log` and
`wasm-core-check.log` under the linked artifact directory. SVG inspection used the
unmodified SVG and the supplied font through an external renderer fallback; this
is not a browser embedded-font-loading claim.
