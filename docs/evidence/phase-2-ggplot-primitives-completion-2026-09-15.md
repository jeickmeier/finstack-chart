# GG-07 primitive and interval completion

GG-07 / GG2-06 / FIX-GG07 is complete on `96d044d` plus the owned worktree.
The [manifest](phase-2-ggplot-primitives-completion-2026-09-15.json) records source
and publication hashes, exact commands' logs and validation boundaries.

Shared typed recipes implement intervals and independent middle/box/point controls,
source/stat bounds, reference lines, steps, segments/arrows, curves/spokes, rugs,
blank training, count/column marks, polygon holes, tiles and row-grid rasters.
Missing mapped dimensions omit geometry while retaining positional training.
StatSum retains proven source aesthetic partitions and original aggregate members.
Optional portable stroke outlines implement cap/join controls with bounded geometry,
uniform alpha and original inspection targets. Definition 74 and scene 21 are
capability selected. See [ADR-026](../adr/026-shared-primitive-recipes.md).

## Acceptance evidence

- The final shared host runner passes: `/private/tmp/gg07-final-hosts2.log`.
  Seventeen interval, nine surface, five mark and six stroke authors produce
  **111 byte-identical SVG/PDF/PNG publications per host** through actual Rust,
  Python and WASM. Original/replay scenes, owned field/expression inputs and
  Count partitions pass; strict Python and TypeScript consumers pass.
- All 111 final Rust publications match the already inspected previous output
  byte for byte (`gg07-final2/inspection-reuse.json`). Independent SVG and Poppler
  PDF renders plus actual PNGs were inspected. Native interval/mark/stroke panels
  were painted and inspected; the unchanged surface gallery retains its prior proof.
  [Native report](phase-2-ggplot-native-2026-09-15.md) records images and traces.
- The macOS workspace run completed **972 passing tests and one failing style-size
  assertion**, across 202 nonempty / 214 total targets. The new cap/join fields
  intentionally expand `Style` from 56 to 64 bytes; the explicit budget and ADR now
  record that cost. The corrected assertion passes in the **40-test final focused
  run**, covering host, interval, mark, surface, paint and weight contracts.
- After the shared dash-length calculation was made deterministic, **six stroke
  unit tests and one actual export alpha regression pass**. The final host run
  proves exact native/WASM dash endpoints. No fixture tolerance was loosened.
- Final `mise run check` passes (`/private/tmp/gg07-final-check3.log`): repository,
  dependency licenses/sources, formatting, native examples, all-target compilation,
  Clippy, rustdoc and WASM core compilation. `git diff --check` passes.

The full workspace suite was not repeated after the localized dash calculation,
style-size expectation and behavior-preserving lint fixes; their affected tests
and final repository/host checks ran instead. This report does not describe the
original workspace invocation as green. No Linux execution, fresh full cumulative
binding runner, performance certification or GG-19/global gate is claimed.

## Detail and next work

[Intervals](phase-2-ggplot-intervals-2026-09-15.md),
[surfaces](phase-2-ggplot-surfaces-2026-09-15.md),
[marks](gg07-mark-recipes-2026-09-15.md), and
[strokes](phase-2-ggplot-strokes-2026-09-15.md) retain pinned independent reference
fixtures and focused command evidence. Actual artifacts are under
`/private/tmp/finstack-chart-proof-20260915/gg07-final2`.

GG-06/07/08 are accepted; GG-09–18 remain unfinished. Proceed with distribution
kernels and univariate statistics under GG-09 and independent facet work under
GG-12, using prepared source fixtures and preserving remaining dependencies.
