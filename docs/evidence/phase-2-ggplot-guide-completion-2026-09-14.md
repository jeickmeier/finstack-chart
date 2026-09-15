# GG-05 guide completion evidence

Revision: `fbaa1d5` plus the owned GG-05 changes. Requirement GG2-04 / FIX-GG05.
GG-05 is COMPLETE for its assigned scope. Cumulative G-GGPLOT/G-PARITY and
whole-project host certification remain open. The [evidence manifest](phase-2-ggplot-guide-completion-2026-09-14.json) records source fingerprints and artifacts.

## Implementation and ownership

`chart-core` owns guide preparation, compatible key merging, automatic per-layer
awareness, overrides, grids, placement, collected/local facets, axis policies and
component metadata. Keys consume retained scale samples. Binned keys separate
interval glyphs from boundary labels and ticks. Uneven color steps use transformed
interval widths; palettes retain their existing shared batch/cache ownership.

The guide grammar adds definition versions 68–71 without bumping unconfigured old
definitions. Scene v19 carries legend component roles. Host builders delegate to
that grammar, and native/export consume the same immutable scene. Custom guides
contain portable vector paths with checked bounds and resource budgets.

The reference is the locally provisioned R 4.6.1 / ggplot2 4.0.3 oracle. New committed
captures and comparisons cover:

- 228 representable numeric step-control cases, including constants and empty
  populations; 12 binned NULL cases have no authored enum representation.
- 16 Date/POSIX control cases in four timestamp units: 64 comparisons.
- 14 multi-aesthetic key/override cases, six layer-awareness policies with renamed
  labels, and eight direction/reversal/limit-label binned-key cases.
- 21 endpoint-first overlap populations and 24 logarithmic tick ladders; rendered
  dodge, caps, minor identities, stacking, signed and prescaled log coordinates.
- Shared/local facets, all four outside positions and fractional inside placement,
  custom-vector replay/rejection and explicit versus unspecified guide order.

`Guides$merge` maps order zero to 99. Equal orders use deterministic authoring order
here; the adapter does not reproduce R's opaque serialized hash ordering. The
portable custom-guide representation uses paths, including supplied text outlines,
not runtime R/grid objects. Angular policies remain assigned to GG-13; theme-wide
inheritance and mathematical text remain assigned to GG-14. Non-invertible infinite
log-guide domains return a capability error.

Default binned constants were also qualified in both implicit and explicit guide
modes: 20 source comparisons. The old fast path skipped constant palette/key
validation and silently omitted a required zero-interval draw error. It now uses
the same checks as explicit stepped guides. All 17 colorbar tests and the existing
binned guide, label and break-function targets pass after that fix.

## Validation record

`mise run test` passes **895 tests across 189 nonempty / 201 total targets**
(`/tmp/gg05-accepted-test-v2.log`). `mise run check` passes, including Clippy,
rustdoc and WASM (`/tmp/gg05-accepted-check-v2.log`); repository and diff checks
also pass. The final temporal colourbar correction adds eight source-backed
rendered-position comparisons across Date/POSIX and all four timestamp units.
Artifacts are task-local under `/private/tmp/finstack-chart-proof-20260914`.

- `gg05-accepted-runtime-v2`: ten independent Rust/Python/WASM authors; all 30 SVG/PDF/PNG
  publications match byte-for-byte. All ten PNG and independently rendered SVG/PDF
  contact sheets were inspected, including layer awareness, custom vectors, bins,
  placement and logarithmic stacks.
- `gg05-temporal-final/controls/{python,wasm}`: 192 lifecycle states per host and 48
  publications per host for the temporal step controls. Exact cross-host comparison and all 16 triplet inspections pass.
- Native executable `ggplot_guide_composition_native` was inspected for modes
  5/7/8/9. Screenshot:
  `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-14_22-19-18.png`.
  `/tmp/gg05-native-final.log` records successful scene paint frames, including a
  viewport change. Earlier modes 0/2/5/6 were inspected at the intermediate snapshot.

The 60 numeric-control SVG/PDF/PNG triplets and all 16 temporal triplets were
inspected. Explicit repeated temporal breaks intentionally retain overlapping
labels, as the reference does; they remain inside the bounded guide clip. Automatic
and even-spaced controls, endpoint labels and unequal cells were checked separately.
All nine default-guide triplets were also inspected, for 95 triplets / 285 new
publication files in total. Ten additional temporal colourbar/default triplets
were inspected after the unit correction (315 inspected files overall). The final composed runtime files retain the previously
inspected bytes. Numeric/default Python and WASM publications are byte-identical.
Rust PNG/PDF bytes match; Rust SVG differs only in process-local guide IDs, compared
with a bijective first-encounter renumbering that preserves all identity relations
and every other byte. Composition publications match raw bytes across all three hosts.

Native unequal-width/end-label controls were inspected for continuous, binned,
reversed, collected and panel-local guides using `ggplot_colorbars --steps-controls`.
Screenshot: `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-14_22-32-59.png`;
paint trace: `/tmp/gg05-native-controls.log`.

A fresh reference capture adds 36 draw outcomes to the existing interval-label
fixture without changing any original key, boundary or callback record. Two
constant color-step cases build successfully and reject at draw time; the host
proof now asserts those rejections and exports the other 34 triplets. A second
capture adds 192 draw outcomes to the degenerate-mapping fixture, preserving all
256 original records. Twenty-four binned cases map successfully but reject their
default guide; the six-test target now checks preparation/layout at the source
boundary. Continuous cases retain the explicitly authored key-guide policy.

The cumulative primary proof reached the temporal colourbar matrix and exposed
an absolute-versus-offset unit mismatch in key layout. The shared timestamp owner
now converts sample endpoints before projection; publication precision is unchanged.
The focused final host run is `/tmp/gg05-temporal-final.log`, with source artifacts
under `gg05-temporal-final`: 7,680 selection states / 198 files, 21,872 interval-label
states / 180 files, and 192 step-control states / 48 files per host. All three exact Python/WASM state and publication comparisons pass. The full cumulative
runner is not claimed as passed; later unrelated matrices were not rerun in that
invocation. Its earlier numeric guide, palette and callback sections are supplemental
regression evidence, not a global binding certification.

## Remaining project issues after this package

These are planned incomplete capabilities, not newly discovered regressions.
The ledger owns their current status and prerequisites.

| Package | Remaining work |
| --- | --- |
| GG-06 | Bin/count/summary statistics and position semantics. Earliest sequential next package. |
| GG-07 | Primitive and interval recipes. |
| GG-08 | Data-driven text, labels and annotations. |
| GG-09 | Distributional and one-dimensional analytical layers. |
| GG-10 | Smoothers, confidence bands and quantile regression. |
| GG-11 | Two-dimensional statistics and contours. |
| GG-12 | Full facet semantics and layout breadth. |
| GG-13 | Cartesian/transformed/polar/radial coordinates, including angular guides. |
| GG-14 | Theme hierarchy and mathematical typography. |
| GG-15 | Geographic layers and coordinates. |
| GG-16 | Extensibility and authoring conveniences. |
| GG-17 | Saving and device capability completion. |
| GG-18 | Full grammar, update and host integration. |
| GG-19 | Capability certification and handoff. |

All eight D3 gates are passed for their declared typed surfaces. `G-GGPLOT` and
`G-PARITY` remain not passed; `G4` remains open. Expanded WP-21/22/23 acceptance still
needs the integrated FIX/PERF/platform/accessibility/documentation and release
workloads. Historical acceptance and waivers retain their original scope.

Legacy `scripts/run_binding_proofs.py` fails at `family-secondary`: its committed
fixture declares definition v1 while the unchanged existing capability dispatch
requires v17. This predates GG-05. The failure is reproducible in
`/tmp/gg05-binding-debug.log`; no fixture was weakened or rewritten. The portable
envelope maximum itself has been advanced to v71 for the new guide capabilities,
with a custom-guide envelope validation check.

New guide builders pass strict Python and generated-WASM TypeScript consumer checks
(`/tmp/gg05-guide-types-python.log`, `/tmp/gg05-guide-types-ts.log`). Native proof windows
were closed after inspection. A `block v0.1.6` future-compatibility warning is
upstream dependency debt; current Clippy/rustdoc checks treat project warnings as
errors and remain separate from that future compiler warning.
