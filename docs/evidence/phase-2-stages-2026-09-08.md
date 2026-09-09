# GG-02 — Compatibility stages and grouping

Requirements: GG2-01/02, FIX-GG02. Base revision `fab2505`; uncommitted continuation.
The [manifest](phase-2-stages/manifest.json) identifies the actual source snapshot,
[environment](phase-2-stages/environment.json) and retained per-host artifacts.

## Implemented contract

Canonical `ExecutionSemantics` carries profile/version/source hash and resolved
scale-stage/group policies into preparation, workers, edits and export captures.
Profile and primary envelope must agree. New definitions use capability version 3;
legacy version-1/2 definitions retain their existing defaults. Source grammar survives
statistical lowering, including shared named transforms. Consumer scale-context
conflicts reject instead of changing a shared population silently.

The ggplot2 profile transforms positional values and applies explicit population
limits/OOB policy before statistics. Coordinate transforms and viewports are separate.
Generated-space metadata prevents double projection. After-stat expressions read
back-transformed coordinates and apply the scale once to expression results.
Discrete aesthetics infer interaction groups; explicit layer/stat grouping overrides
inference. Missing categories remain groups and numeric explicit group keys remain numeric.
Horizontal families reuse the statistical kernel and transpose physical output once.
Y-only histograms and categorical dependent-axis bars infer horizontal orientation;
unmeasured categorical bars select count, while authored measures retain columns.

Typed expression graphs support bounded numeric arithmetic/reductions, comparisons,
missingness and selection; core also validates boolean/text/color values. No R evaluator
enters production. Rust/Python/WASM expose source/stat/bin/post-scale/theme expressions.
Source reductions intentionally use the registered dataset before chart filters/facets;
generated reductions use the prepared layer population. Post-scale outputs share one
pre-modifier snapshot. Size/color are the current outputs, with size restricted to
point/rule geometry; independent aesthetic and physical-unit semantics remain GG-03.

## Evidence

| Check | Result and boundary |
| --- | --- |
| R 4.6.1 / ggplot2 4.0.3 stage oracle | 22 cases reproduce byte for byte; [cases](../../fixtures/parity/ggplot2/stages.json), [source/runner/lock hashes](../../fixtures/parity/ggplot2/stages-manifest.json). Historical GG-00 manifest remains intact. |
| Focused Rust regressions | Ten stage tests plus four expression tests pass: log versus coordinate-log, limits versus zoom, discrete/missing/explicit groups, shared aliases and conflicting scales, reductions, numeric robustness, graph type/cycle/work rejection, horizontal bars/bins, profile edits and old snapshots. |
| Actual primary proof | Full existing primary families/actions/streaming/type consumers pass. The runner now executes 13 independently authored stage figures per host, source-expression filters and shared aliases. Python and TypeScript accept valid staged authors and reject five wrong-stage combinations. |
| Capture/ownership | Python/WASM verify static, Presented and Current output before/after a profile edit, including a request prepared after the edit but captured before it, and bytes exported after chart/plot/data/output disposal. |
| Publication | All 13 Rust/Python/WASM PNGs match on every RGBA channel. Every PDF embeds supplied Noto Sans. Inspected both [PNG sheets](phase-2-stages/contact-1.png), [second sheet](phase-2-stages/contact-2.png) and [rasterized PDFs](phase-2-stages/pdf-contact-1.png), [second PDF sheet](phase-2-stages/pdf-contact-2.png). PNG uses the SVG publication path. |
| Native | Actual GPUI [window](phase-2-stages/native.png) and [paint stamps](phase-2-stages/native.log): log mean, coordinate-log mean, horizontal log histogram and theme expression. Captured before the final shared-transform/filter additions; those additions have separate runtime tests. |
| Repository / macOS / Linux / aggregate proofs | PASS `mise run check`; 276 macOS tests and 270 Linux core/export tests, zero ignored; actual aggregate bindings including the retained path proof. [Logs](phase-2-stages/logs/check.log), [test counts](phase-2-stages/logs/test-counts.json). Linux uses the existing Rust 1.97.1 container with networking disabled and read-only source/registry mounts. |

The inspected coordinate-zoom and Keep panels intentionally contain no visible point:
the retained arithmetic mean is 37 outside the displayed 1–10 viewport. The statistical
limit panel shows 5.5; Squish shows 7; the logarithmic mean is 10 in source units.
After-stat normalized bin heights are 3/7, 2/7, 2/7. Theme accent is `#1256ab`.
Existing narrow constant bar widths and verbose logarithmic ticks remain their later
geometry/axis owners; these artifacts do not certify full ggplot2 style or guide parity.

No independent expected values, tolerances or visual baselines were weakened.
G-GGPLOT, G-PARITY and the remaining family gates stay open. **GG-02 is COMPLETE for this package scope.** Next: shared color/interpolation/scale
foundations in dependency order.
