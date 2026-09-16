# GG-06 and GG-08 completion — 15 September 2026

GG-06 and GG-08 are complete on `96d044d` plus the owned worktree. The owner
assigned GG-06–18 and authorized parallel agents. GG-07 and GG-09–18 remain open;
GG-19/global certification and expanded WP-21–23 are separate.

GG-06 adds reference bin closure/alignment/padding/weights and normalized fields;
signed count/proportion and grouped/binned summary helpers; reference stack/fill,
dodge/dodge2, jitter-dodge and nudge controls. Exact membership/provenance remains
separate from weighted values. Bin training shares the physical-axis population
across layers and fixed facets, respects free axes and explicit limits, and retains
original validation before automatic-edge resolution. Both closures on reversed
axes match the existing pinned fixture; the former implicit-left test now explicitly
authors its closure without changing its expected values.

GG-08 adds source/stat row labels, padded rotated boxes, justification/overlap,
reference physical units, shared nudging, vector annotations and nearest/interpolated
RGBA images. Shared definitions use v72/v73 and image scenes v20 as required by
retained capabilities. Existing locked versions remain unchanged; optional renderer
image features are enabled. Core has no new dependency or I/O.

Validation:

- **926 macOS tests pass**, 194 nonempty / 206 total targets, using
  `mise exec -- cargo test --workspace --locked --no-fail-fast`.
- `mise run check` passes repository/dependency checks, formatting, builds, all-target
  Clippy, rustdoc and WASM core compilation. The subsequently updated reverse test
  also passes its scoped test and Clippy checks. `git diff --check` passes.
- Pinned R fixtures cover 100 bin cases, 32 position cases, six jitter setup cases,
  66 scalar summary helpers, count and fixed/free/transformed/limit/orientation
  populations, plus text rows and all five reference text-unit factors.
- GG-06 independently authors 100 bins with original/replay in Rust/Python/WASM,
  eight positions and five summaries. **63 publications per host match exactly**.
  Actual Python/WASM original/replay regressions also match pinned R for OOB Keep
  limits, horizontal fixed/free_x/free_y facets and mixed-orientation overlays.
- GG-08 independently authors seven plots, producing **29 identical publications
  per host**, including eight outline SVG/PDF outputs. Preserved rotated PDF retains
  logical characters; outlined PDF has no extractable text. A PNG pixel regression
  verifies the renderer actually honors nearest versus interpolated sampling.
- All 92 unique publications were inspected, including independent SVG/PDF
  rasterizations. Final GG-06 bytes equal the inspected checkpoint; the final GG-08
  nearest-image correction was reinspected. Twelve bin/position and four text/raster
  native panels were painted and inspected, then their owned windows closed.
- Strict Python/TypeScript scoped consumers pass. Legacy definitions retain their
  intended policies; weighted bins report exact batch fallback honestly. Retention,
  append/correction/removal and immutable snapshots compare with fresh computation.

Logs: `/private/tmp/gg06-08-acceptance-{check,test-all}.log`,
`/private/tmp/gg06-summary-tests.log`, `/private/tmp/gg06-reverse-closure.log`,
`/private/tmp/gg06-reverse-clippy.log`. Actual artifacts are under
`/private/tmp/finstack-chart-proof-20260915/acceptance-gg06` and
`/private/tmp/finstack-chart-proof-20260915/final-packages/ggplot_text_marks`.
The initial shared runner stopped on a proof-author dataset-name collision; after
naming the overlay explicitly, both summary host scripts were rerun successfully
against the same fresh modules. Final comparisons and source fingerprints are in
the accompanying [manifest](phase-2-ggplot-stats-text-completion-2026-09-15.json).

Boundaries: no fresh full cumulative primary/bindings runner, Linux, performance or
global certification is claimed. Hmisc wrappers are anchored by independent R
formulas; the Hmisc package was not executed. Jitter and bootstrap use the declared
portable seed/draw policy, not R RNG identity. PDF/native/resvg interpolation kernels
are not asserted pixel-identical across formats. Detailed contracts:
[binning](gg06-bin-contracts-2026-09-15.md),
[positions](gg06-position-contracts-2026-09-15.md),
[count/summary](gg06-count-summary-contracts-2026-09-15.md),
[text/raster](phase-2-ggplot-text-2026-09-15.md).
