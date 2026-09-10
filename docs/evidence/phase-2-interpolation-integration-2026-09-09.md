# WP-IP06 integration and independent WP-IP07 evidence

Date: 9 September 2026. Revision: `51f2edafdf5ec65121966dedcb1d5ffa838f80de` plus
uncommitted interpolation changes, preserving preceding axis and concurrent host work.
Requirements: ITP-07/08, FIX-I01-H; ITP-01–06 regressions. WP-IP06 is COMPLETE.
WP-IP07/G-INTERPOLATE remain open because WP-AX06 is incomplete; WP-AX03–06 are outside
this assignment.

## Delivered contract

The existing extension registry now captures pure native interpolation factories with
exact version identity, bounded parameters and portability. Prepared samplers own their
state; registry copies/disposal cannot invalidate retained interpolation, scale, chart
or export snapshots. Python and JavaScript select installed Rust implementations with
`registered_interpolation` / `registeredInterpolation`, including custom piecewise
composition. Continuous, sequential/diverging/rank and exact calendar scales retain
the registry through copies and immutable changes. Registered factory data is boxed to
keep shared descriptors compact without changing wire representation.

Mapped numeric aesthetics, floating color ramps and legend/theme preparation use the
same sampler. External examples demonstrate squared numeric sampling and floating Lab
color sampling using the canonical kernels. Core checks operation identity, parameter
and output budgets, duplicate registration, unavailable versions and finite sample
parameters. Native-only factories work in native charts and reject serialization and
headless publication before interpolation preparation. Other extension families retain
their existing destination diagnostics.

Builtin standalone envelopes remain version 1. Registered interpolation/scales require
version 2, and charts containing registered mappings require version 12. The intentional
Rust source migration changes `InterpolationFactory` from `Copy` to `Clone` so it can
own registration parameters. [ADR-017](../adr/017-shared-interpolation-values.md), the
[extension contract](../extension-contract.md) and
[primary API](../primary-authoring-api.md) describe the supported boundaries.

## Evidence

- Actual Python and WASM replay all **370 value/color/zoom cases plus 36 browser
  transform cases**. The fresh Node and pinned Chromium generators reproduce both
  committed corpora byte for byte. Every recorded interpolation/color source hash
  matches. The shared development lock has evolved; current manifests preserve its
  current hash without rewriting the original fixture manifests or expectations.
- Independent Rust/Python/WASM authors produce three sampled states (`t=0, 0.5, 1`).
  All nine SVG/PDF/PNG payload comparisons are byte-identical. Canonical descriptors,
  scene and guide records match under the established floating bounds; the observed
  zoom endpoint geometry difference is below `2e-14`. Identity, string and integer
  metadata remain exact. [Comparison](phase-2-interpolation-integration/host-comparison.json).
- Every mapped mark's RGBA matches standalone floating Lab sampling at the final byte
  boundary, and sizes match the independent squared formula. Ramp marks are sampled
  directly; guides use domain-entry swatches. No backend two-stop gradient approximation
  is used or certified.
- Both hosts test four keyed append/upsert/remove/retention updates against fresh PNG,
  exact keys above `2^53`, portable round trips, copied/disposed factories, retained
  captures after chart/registry disposal, and native-only rejection. Strict Python and
  TypeScript positive consumers pass; five intentional invalid uses reject in each host.
- Native-only factories were inspected in the GPUI gallery at all three explicit sample
  states. PNG, independently rendered SVG (resvg 0.45.1 plus the supplied font), and PDF
  (Poppler) were inspected at each state. Shapes, ramp marks, labels and endpoint marks
  are visible without clipping. An initially clipped fixture endpoint was corrected by
  fitting the authored x range inside the plot; no reference expectation was changed.
- The [27-export/configuration catalog](phase-2-interpolation-integration/verdict-catalog.md)
  records standalone support and the open axis consumer requirement. The
  [benchmark](phase-2-interpolation-integration/benchmark.md) contains 23 preparation,
  owned/reusable sample and allocation workloads, ten warmups and thirty measured
  batches. It is a finite component profile, not a PERF release qualification.

The [source snapshot](phase-2-interpolation-integration/source-hashes.json) records
final working-tree identity against the assignment baseline; it does not attribute
concurrent edits to this assignment.

The reproducible primary proof runner includes the new Rust, Python, WASM, comparison
and type stages. Source, environment, logs and inspected artifacts are retained under
[the evidence directory](phase-2-interpolation-integration/).

## Validation boundaries and next action

The repository check passes: graph/dependency/license checks, formatting, native example
builds, workspace all-target checking, strict Clippy, rustdoc and standalone WASM core
metadata. The earlier loader delay resolved without changing tests or platform policy.
The final macOS and offline Linux core/export/text/external-extension suites each pass
435 tests/doctests. The final focused interpolation suite passes 18 tests, including all
five registration/consumer contracts. See `macos-final-tests.log`,
`linux-final-tests.log` and `final-core-tests.log` in the evidence directory.
Actual final Python/WASM modules pass the interpolation and existing primary replay
(23 action transitions, 47 independent input steps and 70 streaming steps per host).
These scoped suites do not claim a fresh GPUI test suite, sustained performance or
expanded release/platform acceptance. The proof runner stages were exercised directly;
the entire accumulated `bindings-proof` command was not rerun in this assignment.
WP-IP07 cannot close until the WP-AX03–06 chain supplies the shared axis transition,
interruption, reduced-motion and capture evidence. That work remains with its existing
owners and is not implemented by this interpolation assignment. Expanded WP-21/22/23,
G-AUTH, G-PARITY, G-GGPLOT and G4 remain separate ledger gates.
