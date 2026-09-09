# WP-S06 complete stack layouts — implementation and qualification

Status: **COMPLETE**. Final macOS qualification below supersedes the historical pending entries. Working tree over `fab2505951061eaafe9adb52c86b248ee0dfa6bf`.
SHP-07/09 and FIX-S07/09 apply. WP-S01 and WP-10 are accepted; G-SHAPE remains open.
[ADR-020](../adr/020-shape-generators-and-curve-protocols.md) records shared ownership,
reference arithmetic, missing cells, source targets and compatibility decisions.

## Implemented and verified

- The pinned d3-shape 3.2.0 corpus has 435 independent cases: every six-order / five-offset
  pair, explicit permutations, positive/mixed/signed/zero/tie/peak/sparse/empty/extreme
  inputs, and explicit missing policies. Two regenerations agree byte for byte.
  Negative zero uses the existing tagged-number transport. Core arithmetic compares
  with the predeclared `2e-12 * max(1, abs(expected))` tolerance; key/rank/source shape,
  signed zero, NaN placement and permutations are exact.
- `shape::Stack` and actual Linux Python / Node WASM owners pass all 435 cases.
  Native order/offset interfaces, callback/constant behavior, copied ownership,
  disposal, exact metadata and bounded failures are exercised. Expand's signed
  denominator remains distinct from both Diverging and unchanged legacy normalization.
- Six focused core tests cover the standalone generator and primary integration.
  Tidy bars and areas pass 1,620 fixture/geometry/input-order comparisons. Missing
  Zero cells contribute geometry without source targets; Gap splits runs. Duplicate
  cells, unknown catalogs, nonzero baselines, unsupported geometry and explicit
  missing-error cases diagnose. Retained source keys exceed 2^53.
- Each actual host passes 762 chart comparisons and 48 expected publication-precision
  diagnostics for enormous coordinates outside the deliberately fixed 0..4 display
  domain. Valid scenes match independent projected stack endpoints and complete linear
  area path commands. Wire v7, sparse source anchors, clipping/focus and generated count
  aggregate membership pass. The 48 diagnostics are not rendered-chart passes.
- Python and WASM each pass 480 append/upsert/remove/retention comparisons across all
  30 policies, both bars and areas, with and without facets. Each current image matches
  a fresh batch exactly. Older exports remain unchanged after mutation and disposal.
  The fixture declares stable stack and color catalogs. An initial unqualified color
  catalog caused deletion to change fresh first-seen colors; scene comparison isolated
  that existing color-training behavior, and the author now states its intended domain.
  No stack fixture, expected endpoint or tolerance was changed.
- Independently authored Rust/Python/WASM galleries contain all five offsets with
  InsideOut order, signed values, a sparse sample, ten panels and 170 actual targets.
  All three themes export SVG/PDF/PNG at 300/600 DPI. Twelve comparisons have exact
  complete scenes and byte-identical SVG/PDF/PNG output, with zero coordinate differences.
  All three PNGs, three original SVG renders and three PDF renders were inspected:
  expected boundaries, negative intervals, missing cells, readable panel titles and
  guides, and no overlapping furniture. Noto Sans is supplied and embedded.
- Linux Rust 1.97.1 passes Clippy (warnings denied) and rustdoc for core/export/Python/WASM.
  The full core/export run passes **379 tests plus 6 doctests**, zero failed or ignored.
  This includes the unchanged FIX-03 legacy stack and prior shape/facet contracts.
  TypeScript positive/negative consumers and Python positives/six intended negatives
  pass. Shared pie/stack TypeScript metadata types now describe runtime BigInt-to-text
  materialization, including nested values; arc/symbol/stack type consumers pass together.

## Remaining qualification

Fresh macOS Python, native stack inspection, and a complete macOS repository check
remain pending. The macOS check compiled the native/default/Kit capability examples
and progressed into workspace checking, then stalled while starting a Python build
script. The arc preview similarly waits before its first dyld instruction; a normal
app-bundle launch is being tried using the same binary. No system security setting
was changed. Linux execution and inspected headless artifacts do not close native
or complete-repository gates. The primary proof runner includes this family, but
its full combined invocation has not completed on macOS.

The tested host numerical builds preceded a lint-only attribute and additional test
assertions; no executable stack/projection behavior changed afterward. Full current
Linux Rust tests and the final generated-identity host checks pass. Runtime identities
below identify the actual tested binaries, rather than implying platform equivalence.
Registered portable order/offset protocols remain WP-S07 scope, and combined shape
performance/compatibility acceptance remains WP-S08 scope.

## Reproduction and retained evidence

The Linux runs use a read-only workspace and Cargo dependency mounts, isolated
`target/path-proof/linux-target`, and `rust:1.97.1-bookworm` on aarch64 Linux.
Build/test commands use `--locked --offline`; only the matching official Clippy
component installation used network access. Matching Rust 1.97.1 WASM standard
libraries and wasm-bindgen CLI 0.2.128 produce the actual single-threaded Node module.
The canonical runner is `scripts/run_primary_authoring_proofs.py`; family scripts
are `scripts/bindings/shape_stack{,_interaction,_updates,_gallery}.{py,cjs}` and
`scripts/bindings/shape_stack_compare.py`.

[Source identities](phase-2-shape-stack/source-sha256.json),
[runtime identities](phase-2-shape-stack/runtime-sha256.json),
[artifact identities](phase-2-shape-stack/artifact-sha256.json),
[publication comparisons](phase-2-shape-stack/comparison.json),
[Linux full tests](phase-2-shape-stack/linux-all-tests.log),
[Python updates](phase-2-shape-stack/linux-updates.json),
[WASM updates](phase-2-shape-stack/updates-wasm.json),
[visual checks](phase-2-shape-stack/visual/checks.json).


Native follow-up: the task-owned ShapeStackProof application painted three
frames with one layout. Its window was visually inspected: the complete stack
gallery, facets, labels and legends are readable with the expected shape semantics.
The screenshot, paint log and executable identity are retained under
`phase-2-shape-stack/native/`. macOS Python and final repository qualification
remain open.


Final macOS qualification supersedes the pending platform statements above. The
fresh current-source Python extension passed the standalone and interaction corpus,
all family update/retention cases, and both publication resolutions. Publication was
compared with the independently authored Rust and WASM artifacts using the existing
family comparison contract. Native windows were inspected and retained as described
above. `CARGO_TARGET_DIR=target/shape-native-target mise run check` passed repository,
format, dependency/license, native/Kit build, workspace all-target checks/Clippy,
rustdoc with denied warnings, and WASM core compilation. `mise run test` with the
same private target passed 393 tests/doctests in 86 result blocks, zero failures and
zero ignored tests; empty test blocks are not counted as tests.

The accepted Python runtime, source hashes, publication, comparisons, build/check
and full-test logs are retained in this family's `macos/` evidence directory. Every
recorded source hash was checked again after the full suite and remained unchanged.
Native executables retain their separately recorded pre-radial standalone-addition
identities; the arc/symbol/stack implementation and native renderer were unchanged,
and the fresh host and workspace checks cover the subsequent shared-reader change.
The three transient proof applications were closed after inspection. This completes
the family package; cumulative G-SHAPE, remaining Phase 2 and WP-21/22/23 release
qualification are separate and remain open.
