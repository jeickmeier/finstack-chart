# Implementation status

Updated: 16 September 2026. Specification version: 0.5.0.

## Grammar simplify slices 1–13 — 16 September 2026

Internal `chart-core::grammar` cleanup only. Public `register_*` / `Custom*`
traits, ggplot/D3/`LibraryV1` forks, `plot::host`, and `ChartDefinition` are
unchanged. Profile / GuideProfile / ScaleCompatibility, OLS merge, `wire_version`,
and `PointwiseTransform` were not touched.

Outcome: one private `VersionedMap` installer; shared `StatSpace` / path-bounds
helpers; `encode_layer`, `position_layer`, and `finish_layer`/`emit_line_block`
moved out of `compiler.rs`; grammar `pub use` globs replaced with explicit
lists; prelude no longer re-exports ggplot/D3/hierarchy/spatial constructors
(those remain on `chart_core::plot`).

Revision: working tree on `5e82865`; not committed.
Evidence: `mise run check` passed (fmt, Clippy, rustdoc, isolation, licenses).
`mise run test` passed (1,192 tests, 0 failed, 242 suites, 526s, darwin).
`mise run primary-authoring-proof` did not run: PATH has wasm-bindgen 0.2.122,
the proof requires 0.2.128, and the script forbids downloads. Bindings were
not part of this cleanup (`plot::host` still owns those names).
Unresolved: H1–H3 unwraps, F14/F15, and constructor-family collapse remain out
of scope. Next action: commit on request.

Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`; WP-04 at `d0a6c48`.
WP-05 is committed at `3cf1b33`; WP-06 at `f8657fb`; WP-07/08 at `fac148a`.
WP-09/10 are committed at `c5ec829`; WP-11 at `a6fb2ea`; WP-12 at `63dbcc2`.
Original reports retain the revision context from their evidence runs.

## GG-06–18 authorized implementation — 15 September 2026

Current program status: **GG06–GG16 COMPLETE**. GG17 and GG18 implementation and
local qualification are delivered. Windows GDI playback of EMF is the remaining
external GG17 gate; GG18 retains that prerequisite. Final GG14–18 proofs produce
240 identical publications per Rust/Python/Node WASM host. Repository and workspace
checks completed through recorded resumed commands; 40 final focused regressions
also pass. Native public-API interaction and presented capture pass.
The [final wave report](evidence/phase-2-ggplot-final-wave-2026-09-16.md) and
[source/artifact manifest](evidence/phase-2-ggplot-final-wave-2026-09-16.json)
record revision `295e7a0` plus the working tree, actual-host proofs, strict typing,
inspection, resumed-check boundaries and the optional broader runner's partial scope.
Next: execute the prepared Windows EMF playback verifier, inspect its output and
record evidence to close GG17 and dependent GG18. GG19 remains outside this assignment.
The canonical package rows below record the latest status; older dated checkpoints
are historical.
The [16 September completion](evidence/phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.md)
and [manifest](evidence/phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.json)
record 120 identical publications per host, 1,096 workspace tests, 47 final focused
tests, corrected native inspection/painting and literal aggregate-check boundaries.
All remaining model/coordinate capability boundaries are retained in that report.
The dated sections below preserve earlier checkpoint history.

### Windows playback continuation — 16 September 2026

The remaining GG17/GG18 gate was rechecked on revision `295e7a0` plus the
validated working tree. No Git remote or local Windows VM tooling is configured;
no actual Windows runtime was found among available execution tools. Prepared
`/private/tmp/gg17-windows-playback.zip` with the 72/144 DPI EMFs, independently
exported PNGs, existing GDI verifier, hash manifest and batch runner. Input bytes
were rechecked equal across Rust/Python/WASM. Packaging is not Windows execution:
both playback/comparison and image inspection remain pending, including the
additional raster/hole/alpha cases named in the verifier README. No runtime code
changed and no completed test suite was repeated. Next action requires an available
Windows machine/runner; the owner has been asked for its location. GG17/GG18 remain
open and GG19 has not started.

The owner authorized GG-06 through GG-18 and parallel agents. Starting revision is
`96d044d`, with a clean worktree. GG-06 statistics/positions and GG-08 text/labels
are complete. A bounded dependency spike prepares GG-10/15/17.
Later packages retain the prerequisites and scope in the existing phase-2 plan.

Execution uses focused source-backed tests during development, one coordinated
host build per integration checkpoint, and reusable representative visual proofs.
Full workspace/repository suites run after coherent integration, not after each
small edit. Existing passing evidence is reused only where unchanged contracts and
artifact fingerprints justify it. No requirement, fixture tolerance or inspection
obligation is waived for speed. GG-19/global certification remains separate.

Current outcome: **GG-06 and GG-08 COMPLETE**. The final macOS workspace suite
passes **926 tests across 194 nonempty / 206 total targets**; repository, dependency,
Clippy, rustdoc and WASM core checks pass. Independent Rust/Python/WASM authors
produce **92 identical publications per host** across both packages, with numeric
reference, replay/update, native and publication inspection evidence. No fresh full
cumulative binding, Linux or global certification is claimed.

Evidence: [completion report](evidence/phase-2-ggplot-stats-text-completion-2026-09-15.md)
and [manifest](evidence/phase-2-ggplot-stats-text-completion-2026-09-15.json).
GG-07 primitive/interval completion is now **COMPLETE**. GG-09 and GG-12 are now complete; GG-10/11 and GG-13–18 remain unfinished.
Numerical/geography/device preparation is in
[evidence](evidence/gg06-18-dependency-spikes-2026-09-15.md).

### GG-07 complete — 15 September 2026

Shared primitive recipes, StatSum partitions, independent interval styling and
portable cap/join controls are accepted on `96d044d` plus the owned worktree.
Definition 74 and scene 21 are capability-selected. Missing mapped dimensions omit
geometry without losing positional scale training. This integration also repairs
GG-06 owned weight-expression and GG-08 text-label field/expression dispatch gaps.

Actual Rust/Python/WASM authors produce **111 identical publications per host**
across 37 charts; replay/ownership and strict declarations pass. All final files
match inspected Rust publications byte for byte, and native panels were inspected.
Final `mise run check` passes. The workspace run had **972 passes and one obsolete
style-size assertion**; its corrected 64-byte budget passes in the **40-test final
focused run**, followed by six stroke unit tests and one actual export regression.
The suite was not repeated after the localized deterministic dash fix and lint
cleanup; this is not a claim that the initial full-suite invocation passed.

Evidence: [GG-07 completion](evidence/phase-2-ggplot-primitives-completion-2026-09-15.md)
and [source/publication manifest](evidence/phase-2-ggplot-primitives-completion-2026-09-15.json).
Next: GG-09 distribution/univariate work and independent GG-12 facet work.
No Linux, cumulative bindings or global certification is claimed.

### GG-09 / GG-12 complete — 15 September 2026

Both packages are **COMPLETE**. Shared analytical statistics, distribution recipes,
reference facet controls and actual hosts pass their package evidence. The final
macOS workspace suite passes **1,029 tests across 206 nonempty / 218 total targets**;
`mise run fmt` and `mise run check` pass. Independent Rust/Python/WASM authors
produce **120 identical SVG/PDF/PNG publications per host**, with replay equality,
strict declarations and inspected final native/publication outputs.

The work started at `96d044d`; the validated worktree was externally committed as
`295e7a0` during acceptance. [Completion report](evidence/phase-2-ggplot-analysis-facets-completion-2026-09-15.md)
and [manifest](evidence/phase-2-ggplot-analysis-facets-completion-2026-09-15.json)
record source fingerprints, logs, artifacts and the corrected vector/density cases.
GG10 isolated model kernels are excluded from this acceptance. No Linux, cumulative
bindings or GG19/global certification is claimed. Next: GG10 model integration and
parallel GG11 spatial statistics / GG13 coordinates.

### GG-10 / GG-11 / GG-13 implementation checkpoint

GG10 now has 15 integrated numerical tests and two independent public author tests
passing, plus nine population/authoring tests covering weighted predictions,
registered models, integer grids, separate quantile paths, ribbons, all three source
update operations and automatic selection across the 999/1000 facet threshold.
Ten Rust/Python/WASM authors and a native gallery are prepared. Fresh host/native
proofs and final integration checks remain open; this is not package acceptance.
[ADR-028](adr/028-shared-statistical-models.md) records shared model ownership.
GG11 kernel/stage and GG13 projection/clip/guide work continue in parallel; their
readiness evidence and unresolved requirements remain package-specific.

GG13 now has one public numeric/sqrt-coordinate guide-view test passing, including
major/minor reselection, secondary units and unchanged mark-domain training
(`/private/tmp/gg13-guide-training-tests4.log`). Four raster tests pass, including
premultiplied interpolation, original cell identity, annular holes and stepped panel
gradients (`/private/tmp/gg13-raster-tests3.log`); one public warped-raster inspection
and portable replay test passes (`/private/tmp/gg13-raster-inspection.log`). Raster
hit coverage uses scene v22 only when present. Python/TypeScript declaration
consumers include GG10/11/13 and pass; TypeScript used the previous generated generic
WASM declaration plus current authoring declarations, so this is not a fresh runtime
binding result. Shared proof runner scopes now cover all three packages. Pending:
coordinate navigation/custom extension hits, final source guide/geometry checks,
actual host/native/publication proofs and final repository checks. GG14–18 remain
unimplemented; dependency and source preparation does not advance their gates.

The 16 September integration pass adds source-only readiness for GG14 (160 theme
nodes, 2,880 resolved preset records, inheritance/context/subtheme vectors and the
complete captured plotmath syntax table) and GG15 (41 mapproj methods across 426
calls plus state reuse; explicit proj4rs candidate/missing matrix). Fixture structure
and deterministic GG14 regeneration pass. These fixtures do not implement or close
either package. See [GG14 readiness](evidence/gg14-contract-readiness-2026-09-15.md)
and [GG15 readiness](evidence/gg15-mapproj-readiness-2026-09-16.md).

Native qualification found and corrected double raster oversampling on Retina
screens and a shared band-path inspection anchor/target mismatch exposed by model
confidence ribbons. Five raster tests and the six-model native-sized Inspector
regression pass. Radial guide transitions now reject explicitly while retaining static
snapshots; actual flip/sqrt transition samples and two legacy transition tests pass.
The first aggregate repository check found test-module ordering and a redundant
Copy clone; these are corrected, with the final rerun pending. The first actual host
pass proved model publication equality and exposed missing required Arrow/Raster
fields in independent coordinate authors; those author corrections are in progress.
[ADR-029](adr/029-post-statistical-coordinate-projection.md) and
[ADR-030](adr/030-shared-two-dimensional-statistics.md) record shared ownership.

## GG-08 complete — 15 September 2026

GG-08 is **COMPLETE** on `96d044d` plus the owned worktree for GG2-06/09 and
FIX-GG08. Source/generated row labels, rotated padded boxes, justification, overlap,
reference units, shared nudge positions, vector and nearest/interpolated RGBA
annotations use the common core/text/scene owners. Definition v73 and raster scene
v20 are capability-selected. Mathematical text remains GG-14.

Six focused core tests and the actual raster sampling regression pass. Seven
independent Rust/Python/WASM authors produce **29 identical publications per host**,
including eight outline SVG/PDF files; all were inspected using independent SVG/PDF
rasterizers. Four native panels were painted and inspected. Preserved rotated PDF
retains its logical label characters; outlined PDF contains no extractable text.
Strict Python/TypeScript consumers pass. `mise run check` passes for the integrated
worktree at this checkpoint. Whole-workspace test acceptance is not claimed here:
the unrelated GG-06 explicit-limit test exposed a bin-training bug, subsequently
fixed in focused tests; GG-06 horizontal training qualification remains in progress.

Evidence: [GG-08 report](evidence/phase-2-ggplot-text-2026-09-15.md),
[ADR-025](adr/025-row-text-and-portable-raster.md), and actual artifacts/logs under
`/private/tmp/finstack-chart-proof-20260915/final-packages/ggplot_text_marks`.
The shared proof command/log is `/private/tmp/gg06-08-final-hosts.log`; no Linux,
full cumulative binding, performance or GG-19/global certification is implied.
Next within the assignment: finish GG-06 qualification and start GG-07. GG-14
source/contract preparation is recorded but its prerequisite packages remain open.

## GG-05 complete — 14 September 2026

The owner's full GG-05 assignment is **COMPLETE** on `fbaa1d5` plus owned changes
(GG2-04 / FIX-GG05). Shared core now owns multi-aesthetic and binned keys, layer
awareness/overrides, grids and all placements, unequal steps and limit labels,
axis/logtick/stack policies, custom vectors, and scene-v19 component metadata.
Definition versions 68–71 and the portable envelope support the new controls.
Temporal colourbar endpoints now use the same timestamp units as their guide keys.

Final `mise run test` passes **895 tests across 189 nonempty / 201 total targets**;
`mise run check`, repository checks and `git diff --check` pass. Actual Python/WASM
proofs match 711 numeric-control and 93 default-control states, plus 7,680 temporal
selection, 21,872 temporal-label and 192 temporal-control states per host. Ten
independent Rust/Python/WASM composed authors produce 30 identical publications.
Numeric/default Rust PNG/PDF bytes match hosts; only process-local SVG guide IDs
require bijective normalization. Native composition/step controls and 105 selected
SVG/PDF/PNG triplets (315 files) were inspected. Original source mapping/callback
records remain intact; new draw outcomes verify constant-guide rejections.

Evidence: [completion report](evidence/phase-2-ggplot-guide-completion-2026-09-14.md)
and [manifest](evidence/phase-2-ggplot-guide-completion-2026-09-14.json).
The full cumulative primary runner is not claimed as passed: the temporal failure
it found was corrected and all affected temporal matrices rerun independently.
The legacy binding runner still fails on the pre-existing `family-secondary`
fixture (declared v1, required v17). These are explicit whole-project validation
boundaries; no global gate is closed. GG-06–19 and expanded WP-21/22/23 remain open.
Next implementation package: **GG-06**, with GG-08 also ready within its prerequisites.

## GG-05 default binned boundary cells — 14 September 2026

Core implementation for GG2-04 / FIX-GG05 on `6e74ae6` plus owned changes now
collapses finite outside guide intervals onto scale limits and retains infinite
edge cells. Guide key positions use the scale limits rather than the extended
classifier bounds. Classifier intervals and cached palette evaluation are preserved.
The regression reproduced five cells instead of the source's three for outside
breaks before the fix.

The pinned R 4.6.1 / ggplot2 4.0.3 capture contains 33 default-guide cases.
The new core test covers 20 ordinary/empty cases with both implicit and explicit
default guide selection (40 comparisons), plus single, collected and panel-local
layout colors, ticks and labels. Constant populations and NULL breaks remain outside
this slice's acceptance. All 39 tests across seven focused guide, palette, label,
temporal and degenerate-input targets pass. `mise run test` passes with 879 tests
across 187 nonempty / 199 total targets; `mise run check` and
`git diff --check` pass. Logs: `/tmp/gg05-default-{focused,test,check}.log`.

This is a core implementation checkpoint, not a completed GG-05 gate. Fresh
Python/WASM runtime parity and native/SVG/PDF/PNG inspection for these default cases
remain required. Next: qualify those surfaces, then unequal widths and limit-label
controls. GG-06–19 remain unfinished. Evidence:
[default boundary core checks](evidence/phase-2-ggplot-default-boundaries-2026-09-14.json).

## GG-05 even-step boundary handling — 14 September 2026

The explicit stepped-guide boundary slice is qualified on `6e74ae6` plus owned
changes. Shared scale metadata retains ordinal key positions before missing cuts
are removed, preserving duplicate and unsorted keys. Infinite edge intervals keep
their missing-color cells. Zero-interval guides preserve build-time labels and
callbacks and reject when drawn. Existing regression tests exposed and verified
fixes for an empty default-binned boundary panic and premature build-time rejection.

The pinned 60-case reference covers ordinary, constant and empty populations.
All 57 representable cases pass; three rejected binned NULL inputs have no authored
enum variant and are excluded. Fresh Python/WASM proofs agree across 2,692 states,
including 180 new boundary lifecycle states. All 729 publication files match the
independent Rust author; 677 of the earlier 681 retain qualified bytes. Four reversed-guide SVGs differ only
in floating-point coordinate serialization; their PNG/PDF bytes match and their SVGs were
reinspected. All 48 new
SVG/PDF/PNG files and the final four-chart native gallery were inspected.

`mise run test` and `mise run check` pass: 878 macOS tests across 187 nonempty /
199 total targets. Final logs are `/tmp/gg05-step-boundaries-final3-{test,check}.log`
and `/tmp/gg05-step-boundaries-hosts-final3.log`. Artifacts are under
`/private/tmp/finstack-chart-proof-20260914/gg05-step-boundaries`.
[Boundary qualification evidence](evidence/phase-2-ggplot-colorsteps-boundaries-2026-09-14.json)
records hashes, regressions and exclusions. No new full cumulative bindings or
Linux execution is claimed.

GG-04 is complete. Full GG-05 and GG-06–19 remain unfinished. Next: correct default
binned outside/nonfinite guide cells, then implement unequal widths and limit-label
controls. Temporary reference probes cover 30 default-guide and 240 control cases;
they are preparation only. Guide composition, components and the other GG-05
requirements remain open. No commits were created for this slice.

## GG-05 stepped color guides — 14 September 2026

The finite, distinct-cut even-step slice is qualified on `6e74ae6` plus owned changes.
A source-based test reproduced missing stepped bars. Shared guide metadata now
retains the existing interval palette batch and paints equal-width cells through
the existing stepped primitive. Default binned guides reuse trained palette colors;
explicit continuous/binned guides retain their midpoint batch without another callback.
Reversal and orientation leave mark mapping unchanged. Scene version 18 already
represents the resulting cells; this slice adds no authored definition capability.

The 66-case reference fixture includes 18 cases selected for this slice: 16 explicit
even-step cases plus two default binned cases. All 54 single/collected/local layout
comparisons pass. The focused colorbar and palette/pipeline/temporal suite passes
24 tests. Fresh Python/WASM builds agree across 2,512 states; all 681 publication
files also match the independent Rust author. All 54 new SVG/PDF/PNG files and the
four-chart native window were inspected. The earlier 627 files retain qualified bytes.

`mise run check` and `mise run test` pass: 877 macOS tests across 187 nonempty /
199 total targets. Logs are `/tmp/gg05-steps-final-{check,test}.log` and
`/tmp/gg05-steps-hosts.log`; artifacts are under
`/private/tmp/finstack-chart-proof-20260914/gg05-colorsteps`.
[Qualification and inspection evidence](evidence/phase-2-ggplot-colorsteps-even-2026-09-14.json)
record hashes and limitations. No new full cumulative bindings or Linux run is claimed.
Next: the prepared 60-case boundary fixture, including sequential duplicate keys,
nonfinite edge cells and degenerate-guide rejection; then nonuniform spacing and
limit-label controls. These cases are not qualified by this slice. Full GG-05 remains open.

## GG-05 colorbar alpha — 14 September 2026

The bounded alpha slice is qualified on `6e74ae6` plus owned changes. Shared
`GgplotColorbarOptions::alpha` replaces sampled guide alpha without changing marks
or keys; omission preserves palette alpha. Definition version 67 retains an explicit
override. Missing paint remains missing; explicit transparent paint can become opaque.
Invalid finite alpha and NaN reject even with hidden guides; infinities retain the
reference's zero coverage behavior through the existing alpha encoder.

The pinned source fixture captures 96 display/palette/alpha/direction/reversal
cases, ten constructor boundaries and six missing-paint comparisons. Eleven core
colorbar tests pass, including 288 alpha/layout comparisons and a registered-palette
missing-versus-transparent test. Fresh Python/WASM builds agree across 2,440 states;
all 627 publication files also match the independent Rust author. All 72 new
SVG/PDF/PNG files and the four-chart native window were inspected. The previous 555
files retain qualified bytes.

`mise run check` and `mise run test` pass: 876 macOS tests across 187 nonempty /
199 total targets. Logs are `/tmp/gg05-alpha-final-{check,test}.log` and
`/tmp/gg05-alpha-hosts.log`; artifacts are under
`/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-alpha`.
[Qualification and inspection evidence](evidence/phase-2-ggplot-colorbar-alpha-2026-09-14.json)
record hashes and limitations. No new full cumulative bindings or Linux run is claimed.
Next: stepped guide rendering, then remaining guide composition and component contracts.
The 64-case stepped-guide reference fixture is preparation only. Full GG-05 remains open.

## GG-05 gradient and rectangle displays — 14 September 2026

The bounded display slice is qualified on `6e74ae6` plus owned changes. Shared options select raster,
rectangles or gradient. Gradient defaults to 15 samples and uses endpoint-aligned
keys/stops; rectangles keep authored-count key padding and actual-count cells.
Definition version 66 retains non-raster display. Scene version 18 adds endpoint and stepped modes to the existing sampled-gradient
primitive; previous centered scenes remain version 17. Degenerate gradient domains lower to a solid bar while retaining
the prepared samples. Native endpoint paint crops through the first/last image sample centers. A pixel
regression reproduced light seams from separate rectangle cells; stepped mode now
uses coincident stops in one headless paint and native quads. The regression passes.

The new pinned `colorbar-display` fixture captures 80 actual source bars, including
constant domains, defaults and fractional counts. The focused core suite passes
528 display/layout comparisons plus the existing colorbar contracts. Fresh post-fix
Python/WASM builds agree across 2,056 states; all 555 publication files also match
the independent Rust author. All 120 new SVG/PDF/PNG files and the four-chart
native window were inspected. The earlier 435 files retain qualified bytes.
The initial iteration matched hosts but failed dense rectangle PNG inspection;
its replacement passes the regression and visual review.

`mise run check` and `mise run test` pass: 874 macOS tests across 187 nonempty /
199 total targets. Logs are `/tmp/gg05-display-final-{check,test,hosts}.log`;
artifacts are under `/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-display-final`.
[Qualification and inspection evidence](evidence/phase-2-ggplot-colorbar-display-2026-09-14.json)
record hashes and limitations. No new full cumulative bindings or Linux run is claimed.
Next: alpha controls, then remaining guide composition and component contracts.
Full GG-05 and GG-05–19 remain open.

## GG-05 raster presentation controls — 14 September 2026

The bounded raster presentation slice is qualified on `6e74ae6` plus owned changes.
Shared guide options retain horizontal/vertical direction, reversal and first/last
selected-tick controls. Nondefault presentation requires definition version 65;
sampling-only plots retain version 64. Reversal leaves mark colors unchanged.
Ticks and labels paint independently; non-finite keys have no ink. Horizontal
allocation reserves measured label overhang and the configured minimum plot span.

The pinned R 4.6.1 / ggplot2 4.0.3 presentation fixture captures 128 actual gtable
cases. Eight core colorbar tests and 22 core contracts pass, including 432
orientation/layout cases, 128 tick/label cases and endpoint/tiny-frame pressure.
Fresh Python/WASM builds agree across 1,352 states. All 435 corresponding
SVG/PDF/PNG files also match the independent Rust author. All 264 new publication files and the four-chart native window
were inspected. The remaining 171 files match previously qualified bytes.
One-sample keys intentionally coincide, including overlapping source labels.

`mise run check` and `mise run test` pass; the latter runs 872 macOS tests across
186 nonempty / 198 total targets. Logs are `/tmp/gg05-presentation-final-{check,test}.log`
and `/tmp/gg05-presentation-hosts.log`. Artifacts are under
`/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-presentation`.
[Qualification and inspection evidence](evidence/phase-2-ggplot-colorbar-presentation-2026-09-14.json)
record hashes and scope. No new full cumulative bindings or Linux run is claimed.

Next: gradient/rectangles display modes, then remaining guide composition and
component contracts. Full GG-05 and GG-05–19 remain open.

## GG-05 authored raster sampling — 14 September 2026

The bounded raster-sampling slice is qualified on `6e74ae6` plus owned changes.
`GgplotColorbarOptions::nbin` retains authored counts on the shared mapped scale.
Fractional counts round up for sampling while raster keys use the original count.
Zero uses unique limit colors and omits non-finite keys; one paints the lower-limit
sample as a solid rectangle. Visible keys demand validation; hidden guides or
absent breaks bypass it, while hidden labels still demand samples. The retained
control requires definition version 64; existing plots keep their earlier version.

Five pinned R 4.6.1 / ggplot2 4.0.3 generators retain 257 records: 26 constructor
controls, 144 display/direction/reversal boundaries, 60 demand cases, 15 fractional
edges and 12 equal-limit cases. Captured presentation controls beyond sampling are
preparation evidence, not implemented behavior. Five core colorbar tests plus 22
core contracts pass, including 81 authored-count single/shared/local layouts,
12 constant-limit builds, 60 demand cases, stale-envelope rejection and the bounded
sample guard. A source-backed regression reproduced two samples versus one for
zero count with equal limits, then passed after unique-limit fallback was fixed.

Fresh Python/WASM builds pass 108 default, 432 authored-count, 60 demand and 48
constant-limit states. All 171 corresponding Rust/Python/WASM SVG/PDF/PNG files
are byte-identical. The 108 sampled files and 36 constant-limit files were visually
inspected; the 27 default files remain byte-identical to the previously qualified
outputs. An actual native window was inspected for four nonconstant counts across
single/shared/local layouts. That capture precedes the equal-limit correction;
its captured configurations and final publication bytes are unchanged. No separate
constant-limit native capture is claimed.

`mise run check` passes and the final `mise run test` passes 869 macOS tests across
186 nonempty / 198 total targets (`/tmp/gg05-sampling-final-{check,test}.log`).
Fresh host evidence is in `/tmp/gg05-sampling-final-hosts.log` and
`/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-sampling-final`.
[Qualification and evidence boundaries](evidence/phase-2-ggplot-colorbar-sampling-2026-09-14.json)
retain hashes, source ownership and scoped/native inspection details. The cumulative
runner includes the new sampling, demand and constant modes; this slice does not
claim a new full cumulative or Linux run.

Next: direction/reversal, tick-limit controls and independent tick/label painting.
A new 96-case source-grob probe confirms that hidden labels must retain ticks;
the current painter still drops those ticks. This is a named GG-05 presentation
gap, not a sampling acceptance claim. Display modes, guide composition, component
metadata and the other guide contracts remain open. GG-05–19 are unfinished.

## GG-05 default continuous colorbar integration — 14 September 2026

After GG-04 qualification, 25 source paths were integrated from the qualified
isolated slice. All destination preconditions and incoming hashes were checked;
the complete baseline comparison found no unrelated production differences.
The first guide slice draws the full prepared ramp in shared single/faceted
layout, including independent fill/stroke guides. Full GG-05 remains open.

A single sampled gradient avoids seams reproduced with adjacent gradient segments.
Scene wire v17 retains uniformly spaced, center-aligned samples and charges their
payload against the existing aggregate path/paint budget. SVG/PDF use one gradient;
PNG consumes the SVG renderer. The native adapter uses an owned BGRA image with
replicated edge pixels and an interior crop, fixing atlas bleed reproduced in the
first native capture. ADR-024 records the existing locked image 0.25.10 dependency
edge and the rendering contract.

Three pinned default colorbar source builds cover ordinary, asymmetric and sharp
transition palettes, each with 300 samples and four normalized keys. Seventy focused
core tests pass, including 27 channel/facet configurations, hidden/restored guides,
scene cardinality/work budgets and related layout/aesthetic regressions. Actual
Python/WASM match 108 geometry/lifecycle states and 27 publication files; all files
also match the independent primary Rust author byte-for-byte. Nine SVG/PDF/PNG
triplets and the corrected three-chart native window were inspected. The initial
seamed export and atlas-bleed native images are retained as failed iterations.

Evidence is under `/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-sampled`
and sibling `gg05-colorbar-draft/rust-seamless`, including `inspection.json`,
`comparison.json` and `native-comparison.json`. Logs include
`/tmp/gg05-sampled-gradient-{native2,hosts2}.log` and
`/tmp/gg05-native-window-padded.log`. Full isolated `mise run check` passes
(`/tmp/gg05-colorbar-draft-full-check3.log`); the initial Clippy item-count guard
failure was corrected with a saturating comparison and formatted. The final 24
core contract/colorbar tests pass (`/tmp/gg05-colorbar-final-guard.log`). The full
isolated macOS suite passes 866 tests across 186 nonzero targets (198 total), and
fresh final Python/WASM proofs pass 108 states each. All 27 final Rust/Python/WASM
publications are byte-identical and match the previously inspected artifacts;
`gg05-colorbar-final/qualification.json` records hashes and validation boundaries.
The prior selection proof also passes 351 host states and 63 byte-identical files
(`selection-comparison.json`); those additional files were not separately inspected.
The cumulative runner now includes independent native/host colorbar authors and
an optional native byte comparison. No source grob pixel or broad device
certification is claimed.

The remaining guide source contracts and preparatory cases are grouped in
`/private/tmp/finstack-chart-proof-20260914/gg05-guide-entry/manifest.json` and
`readiness.md`: all 47 exports, display/sample-count/demand boundaries, unequal
stepped geometry, merged-key inclusion behavior and 153 actual draws for all 17
key glyphs. These captures are inputs to later slices, not additional implemented
capabilities; the key-glyph PDF has not been visually inspected.

Main `mise run check`, 24 focused core tests and fresh Rust/Python/WASM proofs
also pass. The main builds reproduce 108 colorbar states and 351 existing
selection states per host, with 27 and 63 byte-identical publication files.
All 207 exported file copies match the isolated artifacts, including the dedicated
inspected set. The full production-source comparison transfers the isolated
866-test evidence; that aggregate was not rerun in main. Final evidence:
[default colorbar qualification](evidence/phase-2-ggplot-colorbar-2026-09-14.json),
`/private/tmp/finstack-chart-proof-20260914/gg05-colorbar-main`, and
`/tmp/gg05-colorbar-main-{check,focused,hosts}.log`.

Next: implement the captured display/sample-count/direction controls. Authored guide controls, complete legend component metadata,
stepped/noncolor/multi-aesthetic keys, overrides, placement and AX/custom policies
remain GG-05; angular policies complete with GG-13. GG-05 is open.

## GG-04 scale qualification — 14 September 2026

GG-04 is complete for its owned scale contracts at `6e74ae6` plus the frozen owned
changes. The final cumulative primary-authoring runner exited zero: all 672
declared commands and their inline assertions passed. The integrated macOS suite
passes 862 tests/doctests across 185 nonzero targets (197 total), and full
`mise run check` passes. Final verification matches all 6,343 baseline paths except
the three permitted evidence documents. The temporary R-device PDF side effect was
restored to its baseline bytes; it was not executable source or a runner input.

The coverage reconciliation accounts for all 152 assigned exports, 960 formal
argument occurrences, 276 inherited method occurrences and 160 fields. Scale
computation is qualified through the shared core and actual Python/WASM adapters;
full reference class/export acceptance retains the explicitly named GG-05/GG-16
contracts and GG-19 certification. Numerical comparisons use the runner's declared
tolerances and exclusions. This does not claim fresh Linux or general device
pixel certification.

Evidence: [final qualification](evidence/phase-2-ggplot-scales-2026-09-14.json),
[per-control coverage](evidence/ggplot-scales-coverage.md), and the source/host/artifact
records in `/private/tmp/finstack-chart-proof-20260914/cumulative-final-positional`.
The complete log is `/tmp/ggplot-final-positional-cumulative.log`. Earlier slice
entries below preserve their own revision and validation context.

Follow-on work is recorded in the GG-05 entry above. GG-05–19 remain open.

## GG-04 explicit and minor vector guide selection — 14 September 2026

Eleven verified paths are integrated at `6e74ae6` plus the owned changes.
Explicit positional break lists and coupled labels use the shared batch transform
and label pipeline. Empty selections retain explicit-label length validation.
Explicit minor lists transform as a batch; registered minor callbacks receive full
inverse domains and major vectors, preserve transformed major context, and apply
the shared NULL protocol. The coupled-label adaptation retains the LibraryV1 guide
profile boundary. No new wire field is required.

Pinned source builds cover 144 explicit/coupled-label configurations (96 successes,
48 expected errors) and 240 minor configurations (180 successes, 60 expected errors).
Final native checks pass all 18 vector-transform tests and 14 related guide tests.
Strict core/example all-target Clippy passes. The isolated macOS suite passes 860
tests/doctests across 185 nonzero targets (197 total); this aggregate precedes the
final guide-profile guard, which is covered by the final focused checks and hosts.

Fresh Python/WASM match 360 explicit and 600 minor lifecycle/error records, with
54 and 36 byte-equal publication files. All thirty triplets were inspected. The
preceding callback proof remains unchanged at 180 records and 27 files per host;
the explicit proof also remains unchanged after the minor correction. Source minor
positions and callback inputs are checked numerically; the publications do not
claim minor-tick painting or source grob drawing equivalence. Guide painting and
coincident-label presentation remain GG-05.

Qualification, frozen source and integration manifests are under
`/private/tmp/finstack-chart-proof-20260914/vector-position-complete`; actual host
and inspection artifacts are in sibling `vector-position-explicit` and
`vector-position-minors` directories. Logs include
`/tmp/vector-position-complete-{native,macos,guide-regression,clippy,regression}.log`
and `/tmp/vector-position-minors-hosts2.log`. The earlier interrupted cumulative
run is retained as incomplete evidence.

The integrated macOS aggregate passes 862 tests/doctests across 185 nonzero targets
(197 total), and full `mise run check` passes. Logs are
`/tmp/ggplot-eleven-slices-main-{macos,check}.log`; their hashes and exit codes are
recorded in the integration manifest. The restarted cumulative proof uses the
integrated source, with 6,343 baseline path hashes, at
`/private/tmp/finstack-chart-proof-20260914/cumulative-final-positional` and
`/tmp/ggplot-final-positional-cumulative.log`. Next: finish these checks and reconcile
the final results with the per-control coverage index before closing GG-04 and
advancing GG-05. GG-04 and GG-05–19 remain open.

## GG-04 positional vector callback dispatch — 14 September 2026

Ten verified paths are integrated at `6e74ae6` plus the owned changes. Positional
break callbacks receive the complete inverse-domain vector. NULL results follow
the shared transform protocol, selected breaks transform as a batch, labels retain
their vector context, and empty panels suppress callback demand.

The pinned corpus contains 72 primary builds (54 successes and 18 expected
construction errors). Native checks cover callback demand/inputs, mapped marks,
major/minor positions and exact labels. The isolated macOS suite passes 858
tests/doctests across 185 nonzero targets (197 total), excluding the two main-only
named-limit tests. Strict core/example all-target Clippy passes. Fresh Python/WASM
match 180 lifecycle/error records and 27 byte-equal publications; all nine triplets
were inspected. The preceding NULL-break proof remains unchanged at 1,056 records
and 36 files per host. No new wire field is required.

Frozen source, qualification, integration, comparison and inspection manifests are
under `/private/tmp/finstack-chart-proof-20260914/vector-position-callbacks`.
Logs are `/tmp/vector-position-callbacks-{reference,native4,macos,clippy,hosts3,regression}.log`.
The first host attempts exposed stable-sort and expected error-stage count issues
in the harness; the final run asserts the precise 18 construction-error indices.
The combined main aggregate passes 860 tests/doctests across 185 nonzero targets
(197 total), and full `mise run check` passes. The cumulative actual-host run was
interrupted after a separate explicit positional break-list defect was reproduced;
it is not a complete pass. Inspections compare native formats; source build/key
semantics are checked independently, without claiming source grob drawing parity.

The coverage index now maps every export and formal control (152 exports, 960
formal occurrences, 276 inherited methods and 160 fields) to computation, typed
adaptation or GG-05/GG-16 ownership. Structural equality with the captured source
and all linked file paths is verified. This mapping does not pass a runtime gate.

Next: qualify explicit positional break vectors and coupled labels for registered
vector-dependent transforms. A pinned 144-build corpus reproduces incorrect scalar
mapping and an empty-panel explicit-label rejection gap in the isolated draft.
Then rerun final cumulative qualification. GG-04 and GG-05–19 remain open.

## GG-04 NULL transform and break dispatch — 14 September 2026

Thirteen verified paths are integrated at `6e74ae6` plus the owned changes.
Numeric limit and continuous/binned break callbacks share NULL transform dispatch,
including registered kernels and ordered compositions. Binned selection preserves
NULL versus typed empty output. A separate raw capture verifies forward/inverse
NULL and typed-empty behavior for all 29 built-in configurations (116 outcomes),
including the distinct Box–Cox and Yeo–Johnson branches.

`vector-null-breaks.json` captures 384 primary builds, with 272 successes and 112
expected errors. Native tests check callback inputs/demand, mark mappings and guide
keys. The isolated macOS suite passes 857 tests/doctests across 185 nonzero targets
(197 total), excluding the two main-only named-limit tests. Strict core/example
all-target Clippy passes. Fresh Python/WASM match 1,056 lifecycle/error records
and 36 byte-equal publications; all twelve triplets were inspected. The preceding
vector callback proof remains unchanged at 3,168 records and 39 files per host.
No new wire field is needed.

Evidence and frozen source/integration manifests are under
`/private/tmp/finstack-chart-proof-20260914/vector-null-breaks`. Logs are
`/tmp/vector-null-breaks-{reference,native2,macos,clippy,hosts,regression}.log` and
`/tmp/transform-null-contracts-{reference,native}.log`. Main `mise run check` passes
(`/tmp/ggplot-nine-slices-main-check.log`), including strict workspace Clippy,
rustdoc and WASM core compilation. The combined main aggregate
and cumulative host proof remain pending. Inspections compare native formats,
not source grob drawings. Numeric legend painting remains GG-05.

Next: the positional caller still has scalar inverse and blanket NULL dispatch;
72 pinned primary builds are captured for its bounded callback audit; its first
case reproduces the axis caller rejecting a valid registered NULL break result. Finish that
shared contract before final cumulative/export reconciliation. GG-04 and GG-05–19
remain open.

## GG-04 continuous and binned vector callback composition — 14 September 2026

At revision `6e74ae6` plus the owned changes, 14 verified paths are integrated
from the qualified vector-scale callback snapshot. Non-pointwise transformations
now retain per-layer training batches when limits/breaks are registered. The
complete inverse domain reaches callbacks, transformed limit pairs retain their
batch identity, and continuous/binned guide palettes consume transformed keys
without an invalid inverse/forward round trip. Registered transforms have checked
NULL forward dispatch; built-in logarithmic NULL rejection is preserved.

`vector-scale-functions.json` captures 1,152 primary builds: 808 successes and
344 expected failures, across continuous/binned size/paint, nullable/empty data,
hidden/visible guides, one/two layers and function limits/breaks. The native test
checks raw marks, mapped guide values, exact labels, callback demand and every
observed callback vector/NULL input. The isolated macOS suite passes 855 tests and
doctests across 185 nonzero targets (197 total); strict core/example all-target
Clippy passes. The two main-only named-limit tests are excluded from that count.

Fresh actual Python/WASM pass 3,168 exactly equal lifecycle/error records and
39 byte-equal publication files. All thirteen SVG/PDF/PNG triplets were inspected,
including the visible centered-transform colour legend. The prior identity
callback proof remains unchanged at 864 records and 18 files per host. Evidence,
source hashes, inspection and integration manifests are under
`/private/tmp/finstack-chart-proof-20260914/vector-scale-functions`; logs are
`/tmp/vector-scale-functions-{native7,macos2,clippy,hosts3,regression}.log`.
Ordinary authoring remains v55; existing retained transformed bounds use v56.

The combined main macOS aggregate passes 857 tests/doctests across 185 nonzero
targets (197 total). Full `mise run check` passes, including strict workspace
Clippy, rustdoc and WASM core compilation. Their logs are `/tmp/ggplot-eight-slices-main-{macos,check}.log`. Final cumulative host
qualification must use this combined source. The reference captures build/key
computation, not source grob image comparisons. Numeric legend painting remains
GG-05. Export reconciliation found a further NULL break-callback dispatch gap: a
384-case source corpus reproduces the blanket transform rejection. Its isolated
correction and built-in NULL/empty protocol audit are underway; final cumulative
qualification follows that correction. GG-04 and GG-05–19 remain open.

## GG-04 discrete identity callback demand and NULL limits — 14 September 2026

The colour/fill identity callback slice is integrated in 16 verified paths after
its isolated qualification. Primary hidden identity guides now retain observations
without invoking limit/break callbacks; visible identity scales accept the existing
discrete break protocol. A resolved NULL limit flag distinguishes NULL from typed
empty vectors when evaluating break functions. Retained NULL metadata requires
v63, with restoration, downgrade, missing-registration and replacement-reset
checks; ordinary authoring keeps versions 17/28/33.

`discrete-identity-functions.json` captures 192 primary builds and independent
R named-colour resolution: 182 successes and ten expected named-NULL rejections.
Native checks verify callback presence, every observed domain/NULL input, factor
ordering, named guides, raw paints and three authoring states. All six discrete
limit tests pass, including the prior 90-case standalone corpus. The isolated
macOS suite passes 854 tests/doctests across 185 nonzero targets (197 total),
excluding the two main-only named-limit tests. Strict core/example all-target
Clippy passes (`/tmp/discrete-identity-functions-{macos,clippy}.log`).

Fresh Python/WASM pass 577 exactly equal lifecycle/wire records and 18 byte-equal
publication files. All six triplets were inspected: unused factor colours and
callback names agree across formats. The prior discrete-limit proof remains
unchanged at 198 states and 12 files per host. All evidence, inspection, regression,
source and integration manifests are under
`/private/tmp/finstack-chart-proof-20260914/discrete-identity-functions`; execution
logs are `/tmp/discrete-identity-functions-{reference,native4,python,wasm,regression}.log`.
The combined main macOS suite passes 856 tests/doctests across 185 nonzero
targets (197 total), and full `mise run check` passes repository/dependency rules,
formatting, native examples, workspace all-target check/strict Clippy, warning-free
rustdoc and WASM core compilation. Logs are
`/tmp/ggplot-seven-slices-main-{macos,check,fmt,repository}.log`; the integration
manifest records their hashes. All 16 integrated source paths still match.

The source is primary-build evidence, not a source drawing comparison. Fill guide
metadata is verified, but publications still omit its legend; that painting remains
GG-05. The next concrete inherited contract is continuous/binned per-layer vector
transforms combined with function limits/breaks; 1,152 source builds are captured
for that audit. Final cumulative qualification must use the final combined source.
GG-04 and GG-05–19 remain open.

## GG-04 cumulative proof and six-slice integration — 14 September 2026

The complete primary-authoring runner exited successfully on the frozen main
baseline at revision `6e74ae6` plus the recorded changes. All 621 logged commands
completed, including actual Rust/Python/WASM execution, retained update proofs,
publication comparisons and host typing checks. The final 6,303-path hash audit
found only the two allowed evidence-document changes. The source-verification
record and baseline hashes are in
`/private/tmp/finstack-chart-proof-20260914/cumulative-current`; the complete log
is `/tmp/ggplot-current-cumulative-hosts.log`.

Following that exit, 57 verified source/reference paths were integrated from six
immutable slice snapshots: positional palettes, secondary guide callbacks, factor
labels, identity transform guides, identity vector batches, and identity vector
callbacks. Ordered baseline checks passed before writing. The integration manifest
is `/private/tmp/finstack-chart-proof-20260914/qualified-slices-integration.json`.
The combined main macOS suite passes 854 tests/doctests across 185 nonzero
targets (197 total). Full `mise run check` also passes repository/dependency rules,
formatting, native examples, workspace all-target check/strict Clippy, warning-free
rustdoc and WASM core compilation. Logs are
`/tmp/ggplot-six-slices-main-{macos,check,fmt,repository}.log`; hashes and scope are
recorded in the integration manifest. All 57 integrated source hashes still match. These results
supersede the integration-pending statements in the historical entries below.
The successful cumulative run predates the six slices; it is not the final
cumulative qualification for their combined source. GG-04 remains open while
that qualification and inherited discrete identity callback contracts are resolved.

## GG-04 identity vector limit and break callbacks — 14 September 2026

The isolated draft now passes 288 pinned primary identity builds with vector
transforms and callback limits/breaks: 198 successes and 90 source rejections.
The shared limit owner uses per-layer transformed training extents and complete
inverse endpoint vectors. Break callbacks receive the complete inverse domain,
and their results transform as one vector. Numeric identity scales now accept
registered break operations, and hidden primary guides invoke neither callback.
The prepared transform protocol adds an inverse-NULL operation so registered
identity inverses preserve NULL while arithmetic inverses return typed empty
vectors; reverse compositions retain their source rejection before callback entry.

The native proof compares mapped values, trained ranges, visible guide keys and
labels, callback presence, NULL identity and every observed callback input. It does
not require reproducing redundant R invocation counts. Existing identity cases
share the same test helper. Focused transform/limit/secondary regressions pass;
strict core/example all-target Clippy and formatting pass. The full draft macOS
suite passes 852 tests/doctests across 185 nonzero targets (197 total), excluding
the two main-only named-limit tests (`/tmp/identity-vector-functions-{regression,
macos,clippy,fmt-check}.log`).

Fresh Python/WASM pass 864 exactly equal original/layer-edit/theme-edit states
and existing v55 restoration. All 18 publication files are byte-identical, and
all six SVG/PDF/PNG triplets were inspected. The prior 288-state identity vector
proof and its 18 files remain unchanged on these modules. Evidence, inspection,
regression and frozen source manifests are under
`/private/tmp/finstack-chart-proof-20260914/identity-vector-functions`;
host logs are `/tmp/identity-vector-functions-{hosts,host-regression}.log`.
Eleven paths are frozen, and all six staged slices pass the 57-path integration
dry run. Main source still matches the running cumulative proof baseline except
for the two evidence documents. Integration and the final main aggregate remain
pending. Source evidence is build-only; numeric guide painting remains GG-05.
Discrete identity callback demand and remaining inherited contracts are next.
GG-04 and GG-05–19 remain open.

## GG-04 identity vector batches and shared edit identity — 14 September 2026

A follow-up reference probe found identity scales applying vector transforms to
individual observations: `x + length(x)` mapped `[1,2,4]` to `[2,3,5]`, while the
source produces `[4,5,7]`. The isolated draft now uses the existing vector transform
owner for identity mapping, per-layer guide training and authored limits. Guide
candidates retain their transformed range and map directly as identity values.
The host replay also exposed DAT-02 sharing loss when an unchanged numeric scale
was reauthored: layer editing now retains the previous ID when its input and scale
definition are unchanged. Two-layer sharing is explicit in the portable definition.

`identity-vector-transforms.json` captures 96 builds across cardinality/centering,
reverse composition, ordinary/nullable/empty data, hidden/legend selection,
automatic/fixed limits and one/two layers. All 72 successes and 24 invalid-composition
rejections match native mapping, shared training ranges and exact guide labels,
including reauthored layer scale identities. Fresh Python/WASM pass 288 exactly
equal original/layer-edit/theme-edit states, raw mark sizes and v55 restoration.
All 18 SVG/PDF/PNG files are byte-identical; all six triplets were inspected.
The prior 696-state identity proof and its 24 files remain unchanged on these
modules (`/tmp/identity-vector-{native,python,wasm,host-regression}.log`, comparison,
regression and inspection JSON under
`/private/tmp/finstack-chart-proof-20260914/identity-vector-transforms`).

The full draft macOS workspace passes 851 tests/doctests across 185 nonzero targets
(197 total); strict all-target core/example Clippy and formatting pass
(`/tmp/identity-vector-{macos,clippy,fmt}.log`). That aggregate excludes the two
main-only named-limit tests. Nine source/reference paths are frozen with baseline
and final hashes in `source-manifest.json`; all five staged slices pass the
52-path integration dry run. The running main cumulative proof still precedes
these slices, so main integration and its aggregate remain pending. The source
capture is build-only; numeric guide painting remains GG-05. Vector identity
limit/break-function compositions still require source reconciliation. GG-04 and
GG-05–19 remain open.

## GG-04 identity transform guide reconciliation — 14 September 2026

The inherited identity-scale contract exposed a concrete defect in the isolated
draft: `ScaleTransform::Ggplot` mapped values correctly but its guide path fell
back to a linear family. Reverse and reciprocal identity guides consequently
marked every candidate invisible. The correction selects the existing transform
owner and retains the full identity candidate vector before label formatting;
the latter also repairs natural-log and base-two-log label precision. Other guide
routes retain their existing candidate-censor policy.

The new `identity-transform-guides.json` captures 232 pinned builds across all 29
built-in transform configurations, ordinary/empty populations, hidden/legend
selection and automatic/explicit breaks: 231 successes and one reference error.
The focused native test compares mapped values, transformed guide candidates and
exact labels. All 28 tests across the transform, identity, registered-transform
plot and label targets pass. Strict all-target core Clippy and formatting pass
(`/tmp/identity-transform-guides-{native,regression,clippy,fmt}.log`).

Fresh Python and WASM pass 696 exactly equal original/layer-edit/theme-edit states
and version-53 restoration. All 24 publication files match byte-for-byte, and all
eight SVG/PDF/PNG triplets were inspected. Log/log1p small glyphs and reciprocal
descending sizes agree across destinations, with legible, unclipped axis labels
(`/tmp/identity-transform-guides-{python,wasm}.log`, inspection and comparison under
`/private/tmp/finstack-chart-proof-20260914/identity-transform-guides`). Numeric
identity guide painting remains GG-05; the reference capture uses `ggplot_build`,
so these publications do not certify full source visual parity. The existing host
script's original/label/secondary modes also pass: all 777 states and 594
publication files per host are unchanged from the main cumulative outputs
(`/tmp/identity-transform-guides-host-regression.log`, `regression.json`). The eight
source/reference paths are frozen in `source-snapshot`, with hashes and the
preceding factor-slice baseline in `source-manifest.json`. The four staged slices
now verify 47 unique paths in the integration dry run. Repository and whitespace
checks pass. Main integration and its aggregate validation remain pending;
GG-04 and GG-05–19 remain open.

## GG-04 factor label policy interaction — 14 September 2026

The existing non-color discrete label proof now covers both `drop` and
`na.translate` values with explicit retained factor levels: 640 successful source
builds across size, alpha, linewidth, shape and linetype, four populations,
trained/explicit limits, automatic/named breaks and indexed/scalar label callbacks.
All seven tests in `ggplot_label_functions` pass on a fresh isolated Cargo target,
including exact callback inputs/names and guide keys/labels for the new 640 cases
(`/tmp/ggplot-factor-label-native-fresh.log`). Production behavior is unchanged.

Actual Python/WASM use the qualified secondary-slice native modules and match 1,360
states, including all 80 replacement versus fresh cases and unchanged held scenes.
The existing 1,515-state mode still matches the main cumulative pre-change records
exactly. All 39 publication files match byte-for-byte; all 13 SVG/PDF/PNG triplets
were inspected (`/tmp/ggplot-factor-label-{python,wasm,base-python,base-wasm}.log`,
`/private/tmp/finstack-chart-proof-20260914/factor-labels/inspection/review.json`).
Scoped strict Clippy and formatting pass. The proof scripts and runner share the
existing test/adapter path through `--factors`; no second scale owner was introduced.
All six changed/reference paths are frozen in that output's `source-snapshot`, with
baseline and qualified hashes in `source-manifest.json`.

The full-draw capture separately retains 64 varying-linetype single-group geometry
rejections after successful scale/guide preparation. Those source `draw_error`
outcomes remain open for geometry work; successful-publication samples exclude them.
The initial combined-error capture is preserved at
`/private/tmp/factor-label-initial-draw-errors.json`. Non-color guide painting remains
GG-05; these publications certify destination consistency, not full visual parity.
No new full-workspace pass is claimed for this test-only slice. Main integration
waits for the running cumulative proof; GG-04 and GG-05–19 remain open.

## GG-04 discrete positional palette callbacks — 14 September 2026

A concrete constructor gap is being closed in the isolated draft at
`/private/tmp/finstack-chart-continuous-guide-work`. `scale_x_discrete` and
`scale_y_discrete` forward a population-dependent numeric palette. The existing
fixed-vector positional palette did not express this callback. The new
`positional-palette-functions.json` captures 84 pinned reference draws across
seven callback routes, four populations and automatic/retained/empty limits:
49 successful draws and 35 errors. Count inputs include missing-category slots;
returned names are ignored. Short and nonnumeric results reject, and untrained
empty populations bypass callback evaluation. The reference encoder was corrected
to retain signed infinities rather than letting JSON convert them to null.

The draft reuses the pure scale-palette registry, captures it with prepared charts,
and resolves a numeric palette after bounded categorical training. A typed
`palette_function` in the existing discrete policy requires primary/definition
version 61; older definitions retain their previous versions. Two new native tests
match direct mapping/range and primary band/point mapping/guides, restoration,
missing-registration rejection and envelope downgrade rejection. The existing six
discrete-position tests also pass. The final refinement records the exact count input and verifies one pure
palette evaluation per training pass. It passes in the complete draft macOS run:
847 tests/doctests across 184 nonzero targets (196 total). Strict all-target core
and example Clippy and formatting pass (`/tmp/ggplot-positional-palette-{macos,clippy,fmt}.log`).
The shared example supplies bounded count operations; the typed nonnumeric route
uses an explicit rejecting operation rather than retaining an R vector type.

Fresh Python/WASM pass 298 exactly equal records, including 32 replacement versus
fresh-batch states and preservation of held figures. All 18 SVG/PDF/PNG publications
are byte-identical; all six triplets were visually inspected. Reverse and squared
spacing, ignored names and retained unused levels are correct, with no missing
glyphs or clipped labels (`/tmp/ggplot-positional-palette-hosts.log`, inspection at
`/private/tmp/finstack-chart-proof-20260914/positional-palette/inspection/review.json`).
The primary runner now schedules this proof in the draft. The complete qualified
21-path source is frozen under that output's `qualified-source`, with hashes and
11 verified main baselines in `qualified-source-manifest.json`. Its suite excludes
the two main-only named-limit tests, which retain their separate six-test evidence.
Main production integration remains pending while `cumulative-current` runs.

The next reference capture, `secondary-guide-functions.json`, has 292 actual draws:
166 successes and 126 errors. Discrete secondary break callbacks reject in the
reference itself. Numeric/Date/datetime callbacks have successful source cases;
labels see complete candidates before tick filtering, and Date secondary candidates
retain fractional days. Empty numeric callbacks with the authored label function
reject its length mismatch, while the captured bare numeric empty/NULL outputs reject
in time transforms. The isolated draft now passes 778 native configurations across four timestamp
units, including exact callback names, resolved UTC resource metadata, source-unit
values within 2e-12, complete pre-filter label inputs, guide keys/positions and
version-62 restoration/downgrade/missing-registration checks. Native changes admit
numeric secondary callbacks, preserve NULL versus empty results and fractional Date
candidates, and use the sampled transformed range with sorted approximation knots.
The knot sorting/duplicate reduction is shared with gradient remapping. A registered
square function expresses R's x^2; the D3 signed-power family is not equivalent for
the expanded negative range of an empty chart. Numeric unexpanded zero-range
secondary guides retain their source interpolation rejection after label evaluation;
temporal constant secondary guides remain valid. Explicit UTC capture supersedes the
initial process-timezone-dependent source file, retained in /private/tmp.

Fresh Python/WASM now pass all 1,266 exactly equal states and 24 byte-identical
publication files (`/tmp/ggplot-secondary-guide-functions-hosts.log`). All eight
SVG/PDF/PNG triplets were inspected: numeric identity/affine/square, Date and UTC
datetime identity/shift, plus the empty square panel. Secondary labels align without
clipping across destinations; hashes and findings are in
`/private/tmp/finstack-chart-proof-20260914/secondary-guide-functions/inspection/review.json`.
The draft primary runner now schedules the native, both host and comparison proofs.
The complete draft macOS regression passes 848 tests/doctests across 185 nonzero
targets (197 total), with exit code zero
(`/tmp/ggplot-secondary-guide-functions-macos.log`).
Gradient remapping, nonfinite and invalid-position regressions pass on these fresh
modules: 288/468/240 exactly equal states, respectively, and 96/156/6 matching
publication files. All 258 files also match the main cumulative pre-change outputs
byte-for-byte (`secondary-guide-functions/gradient-regression.json`). Strict
all-target core/example Clippy passes after simplifying a conditional in the new
test; the final focused native test also passes
(`/tmp/ggplot-secondary-guide-functions-{clippy,native-final}.log`). Formatting, repository and whitespace checks pass. The complete 18-path secondary
source is frozen under that output's `source-snapshot`; `source-manifest.json` records
its hashes, evidence and pre-change baselines after the positional-palette slice.
The draft aggregate excludes the two main-only named-limit tests, which retain their
separate six-test evidence. Main integration remains
pending for both frozen positional-palette and secondary slices; GG-04 and
GG-05–19 remain open.

## GG-04 named limit dispatch — 14 September 2026

At `6e74ae6` plus working-tree changes, `named-limit-dispatch.R` captures 300
`lims` versus selected public-constructor comparisons: ten aesthetic spellings,
five input types and six endpoint modes. All 216 successful builds match in
mapped values, limits and selected breaks/labels; all 84 rejection outcomes agree
on rejection (diagnostic text is retained, not asserted identical). Two additional
reference compositions check x/y ordering and duplicate-axis last-scale wins.
The 48 numeric/character/Date/UTC positional outcomes now run through the existing
typed builders. No new production API, scale implementation or wire version was needed.

Validation: pinned R 4.6.1 / ggplot2 4.0.3 capture passes
(`/tmp/ggplot-named-limit-reference.log`); all six tests in
`ggplot_temporal_authored_limits` pass, including two new named-reference tests
(`/tmp/ggplot-named-limit-native.log`). Actual Python/WASM adapters reuse the
latest temporal-interval-label native modules, whose production source is unchanged
by this evidence slice. They pass 181 exactly equal records and 48 byte-identical
SVG/PDF/PNG files, including original/restored/edited definitions and exact timestamp
replacement checks (`/tmp/ggplot-named-limit-{python,wasm,compare}.log`). All 16
publication triplets were visually inspected across six contact sheets; hashes and
findings are in `/private/tmp/finstack-chart-proof-20260914/named-limits/inspection/review.json`.
The primary acceptance runner schedules the named-reference mode. Scoped strict
Clippy passes (`/tmp/ggplot-named-limit-clippy.log`).

This qualifies named positional dispatch through the typed API and reconciles the
R helper's constructor selection. It does not newly certify every nonpositional
constructor, factor interaction, composed native scale replacement or third-party
R dispatch. Those computations remain with their scale-family evidence. GG-04 and
GG-05–19 remain open. Next: reconcile the remaining constructor arguments against
current evidence, then complete the latest cumulative qualification before GG-05.

## GG-04 current cumulative run and binned shape follow-up — 14 September 2026

The current primary acceptance runner is active against the main checkout with
fresh native host modules (`/tmp/ggplot-current-cumulative-hosts.log`, output
`/private/tmp/finstack-chart-proof-20260914/cumulative-current`). Its initial
`source-baseline.json` records 6,303 paths. Production source remains fixed during
this run; later evidence-only files and ledger edits are outside that snapshot.
Completion has not been observed, so this is not cumulative acceptance.

The next constructor reconciliation captures 48 binned shape draws in
`binned-style-defaults.json`: omitted/TRUE/FALSE/NULL `solid`, absent/vector/count
theme palettes, and ordinary/constant/missing/empty populations. Omitted/TRUE/FALSE
bypass theme lookup. NULL consults `palette.shape.continuous` with normalized bin
midpoints; its absent-solid fallback fails only when evaluated. A count-style R
callback applied to vector input also retains its error/zero-length outcomes.
The source contains 43 successful draws and five errors. The original 32-case
count-only capture is retained at `/private/tmp/ggplot-binned-style-initial-count-reference.json`.

An isolated draft uses the existing palette-function and theme-selection APIs to
express the fallback and callbacks. No core scale change was needed. Two bounded
proof-operation modes in the shared extension example provide explicit failure
and repeat-count behavior. Its new native test matches all 48 outcomes, guide
labels/mapped keys and exact callback input vectors. Fresh Python/WASM now pass 134 exactly equal records and 30 byte-identical publication files (`/tmp/ggplot-binned-style-default-{python,wasm,compare}.log`). All ten ordinary-population triplets were inspected in four sheets, with hashes and findings under `/private/tmp/finstack-chart-proof-20260914/binned-style-default/inspection/review.json`. All 76 extension-example tests/doctests pass across 17 nonzero targets; strict all-target example Clippy and formatting also pass. The initial host descriptor omitted required binned `oob`/break fields; supplying the same explicit defaults as the native builder corrected the harness. There was no engine correction or fixture weakening. The draft runner schedules this new proof. Integration remains pending while main source stays fixed for the cumulative run. GG-04 and
GG-05–19 remain open.

## GG-04 cumulative integration corrections — 13 September 2026

The cumulative main-checkout run reached `shape-arc-typescript` and stopped because
its negative fixture used the now-supported `Radius` channel. Arc and symbol
Python/TypeScript fixtures now use `UnknownChannel`, preserving rejection coverage.
The resumed run passed the remaining stack checks, then found that Python's existing
`_point_radial` implementation was not registered. Registration is repaired; its
rebuilt Python module passes the radial scalar/path checks, strict declarations,
697 interaction comparisons and 760 update comparisons. The remaining run is at
`/tmp/ggplot-empty-glyph-cumulative-resume-radial.log`, output
`/private/tmp/finstack-chart-proof-20260913/cumulative-empty-glyph`; earlier portions
remain in the initial, arc and symbol logs. The next stop was a hierarchy input-type mismatch: Python explicitly supplied floating values while WASM inferred integer arrays. The WASM fixture now supplies Float64Array, preserving exact source-value comparison. The final resumed run passes all remaining hierarchy replay/ownership/publication, GG-03 aesthetics and strict declaration checks (`/tmp/ggplot-empty-glyph-cumulative-resume-hierarchy.log`). Together these runs qualify the pre-transform cumulative source with the recorded integration repairs; they do not claim a fresh cumulative run of the newly integrated transforms.

The broader draft workspace run additionally exposed an unhandled
`ScaleValue::MissingCategory` in native annotation descriptions and a stale
version-32 assertion in the example label test. Native descriptions now identify
missing categories. The label test requires version 45, matching its default
theme-palette selection in both continuous and discrete scales, and rejects
version 44; all reference label comparisons pass. These corrections are in both
checkouts. The full draft rerun is `/tmp/ggplot-transform-macos-final.log`.

## GG-04 transform integration — 14 September 2026

At `6e74ae6` plus working-tree changes, the qualified built-in and composition
slices are now transferred to the main checkout. Saved SHA-256 baselines verified
all 38 built-in and 39 composition paths before transfer; copied bytes were
verified afterward. The runner includes the original/label/secondary matrices,
composition chart matrix and both standalone wire suites. Draft qualification
below remains the evidence for the transferred source: 808 macOS Rust tests and
doctests, strict scoped Clippy/formatting, actual host records and inspected
publications. The integrated main-checkout contract target passes all ten tests (`/tmp/ggplot-transform-integrated-contracts.log`), and repository dependency/host-isolation/link checks pass (`/tmp/ggplot-transform-integrated-repository.log`).

The next custom-transform reference capture has 36 actual chart cases for affine
and cubic forward/inverse functions, custom default breaks/labels, positional and
continuous/binned paint routes, and empty/nonfinite populations
(`registered-transforms.json`, `/tmp/ggplot-registered-transforms-reference.log`).
The capture now separates successful build/draw from direct scale/panel queries:
all 36 charts draw, while six empty custom scale-label queries and two empty
custom panel-label queries reject their length mismatch. The initial capture
incorrectly reported those later getter errors as chart failures; the original
phase diagnosis is `/tmp/ggplot-registered-transforms-reference-phases.log`.
The registered-pointwise slice is now integrated from
`/private/tmp/finstack-chart-registered-work`: saved baselines verified 34 owned
paths, with 32 changed files transferred and two reference files already identical.
Nine focused Rust tests pass its 36 chart mapping/positional-guide outcomes,
24 separate paint-scale query outcomes, population/domain/inverse validation,
registry snapshots and retained-binned portability checks. The final macOS run
passes 817 tests/doctests across 176 nonzero targets (184 total), with strict scoped
Clippy and formatting (`/tmp/ggplot-registered-macos-final.log`,
`/tmp/ggplot-registered-clippy-final.log`). Actual Python/WASM runs match 108
lifecycle states and all 36 publication files; twelve ordinary plots were inspected
in PNG/SVG/PDF. Cubic custom labels overlap under the authored Preserve policy;
paint guides are hidden in this publication fixture. The final host log is
`/tmp/ggplot-registered-hosts-qualified.log`, with artifacts/comparison/inspection
under `/private/tmp/finstack-chart-proof-20260913/registered-transforms`.

Both hosts additionally pass five standalone containers at version 10, 45 rejected
envelope downgrades, copying, reconfiguration, option changes, missing registrations
and native-only serialization rejection. Primary/definition version 55 and direct
numeric version 4 preserve the older built-in envelopes. Strict Python/TypeScript
consumers pass on the existing generic descriptor surface
(`/tmp/ggplot-registered-mypy.log`, `/tmp/ggplot-registered-typescript.log`).
The first draft Python run used the wrong build feature and failed during import;
the final run uses the repository's `extension-module,extension-proof` command.
The integrated repository/dependency/link checks pass
(`/tmp/ggplot-registered-integrated-repository.log`).

The transform-owned minor-break slice is integrated from
`/private/tmp/finstack-chart-transform-minor-work`; saved SHA-256 baselines verified
all ten owned source paths before and after transfer. Two focused Rust tests pass all
120 captured charts, callback input/order/empty-population rules, six additional
registered explicit-override routes, callback errors/output budgets and collapsed
range precedence. Existing positional minor callbacks and all ten built-in/composed
transform tests also pass (`/tmp/ggplot-transform-minors-core-final.log` and earlier
focused runs). The shared resolver retains semantic major order before presentation
sorting, consumes transformed minor coordinates once, and resets transform defaults
under composition. The ggplot projection now uses trained constant limits before
reference expansion, matching the independently checked zero-width R range
(`/tmp/ggplot-transform-minors-zero-reference.log`). The existing source/pointwise
factory gains an optional bounded minor method; the example uses a separate versioned
`example.scale_transform_minor` identity without changing the original factory.

Fresh Python/WASM each pass 360 matching lifecycle states and 36 byte-identical
SVG/PDF/PNG publications (`/tmp/ggplot-transform-minors-hosts-final.log`, artifacts
under `/private/tmp/finstack-chart-proof-20260913/transform-minors`). All twelve
ordinary charts were inspected in all three formats. This proves retained minor
selection; minor tick/grid painting remains guide work. The initial harness treated
omitted empty `minor_ticks` as a missing required field; it now reads the documented
empty default. Strict scoped Clippy passes (`/tmp/ggplot-transform-minors-clippy-final.log`).
The final macOS workspace run passes 819 tests/doctests across 177 nonzero targets
(185 total) after the constant-limit and registered explicit-override fixes
(`/tmp/ggplot-transform-minors-macos-final.log`). The refreshed four inspection sheets
are byte-identical to those inspected. This remains scoped transform qualification;
it is not a new cumulative host or Linux/native-platform certification. The integrated
main-checkout target passes both focused tests, and repository/format/diff checks pass
(`/tmp/ggplot-transform-minors-integrated-contracts.log`,
`/tmp/ggplot-transform-minors-integrated-repository.log`).

The next reference capture has 72 vector-coupled transform cases, including
whole-vector cardinality/centering arithmetic, composition, position and continuous/
binned paint, counts and nonfinite/empty data (`vector-transforms.json`,
`/tmp/ggplot-vector-transforms-reference.log`). Fifty-two build/draw outcomes succeed;
eighteen composed-centering constructions reject an invalid domain and two binned
centering draws reject their breaks. This is reference evidence only. Arbitrary
vector-coupled transforms, additional probability distributions and remaining
argument combinations remain open. GG-04 and GG-05–19 remain unfinished.

## GG-04 vector transform draft — 14 September 2026

The reference capture now includes ten direct forward/inverse population records in
addition to the 72 chart outcomes. A separate qualification draft at
`/private/tmp/finstack-chart-vector-transform-work` extends the existing registry
with length-preserving batch arithmetic and retains transformed guide candidates.
Two external-example kernel tests pass against all ten R records, including empty,
nonfinite and composition-domain rejection; one positional test matches all 24
captured configurations for mapping, labels, major/minor positions and JSON restore
(`/tmp/ggplot-vector-transform-kernels.log`,
`/tmp/ggplot-vector-transform-position-minors.log`). Existing registered and built-in
transform tests also pass in the draft (`/tmp/ggplot-vector-transform-guides.log`).
This paragraph records the initial draft stage; the qualified integration and
expanded evidence are recorded later in this section.
The draft now retains transformed training bounds in optional scale metadata and
uses batch arithmetic for paint inputs, binned cuts and inverse endpoints. The two
chart tests pass all 72 captured mapping/error outcomes, including all 24 positional
cases (`/tmp/ggplot-vector-transform-paint-draft.log`). The extended paint comparison also passes candidate counts and visible labels
(`/tmp/ggplot-vector-transform-paint-guides-draft.log`), after fixing scalar binned-cut
handling and retained nonfinite bounds. Reference labels are captured before guide
censoring; this comparison asserts labels only for visible retained candidates. Eighteen existing transform
and guide tests passed before these latest paint edits
(`/tmp/ggplot-vector-transform-guides-batch.log`). The core cumulative suite then passed 691 tests and doctests across 145 targets
(`/tmp/ggplot-vector-transform-core-all.log`). Three external-example kernel tests
also pass invalid-length and callback-error handling, including empty input
(`/tmp/ggplot-vector-transform-kernel-errors.log`). The subsequent draft introduces
primary/definition version 56 only when transformed training metadata is retained;
all five external-example vector tests pass, including primary round trips and
version-55 downgrade rejection (`/tmp/ggplot-vector-transform-wire56.log`). Authored
registered selections without that metadata retain version 55. Metadata restrictions
still need negative qualification; authored limits, batch boundaries across
layers/facets and broader controls remain unresolved. Actual rebuilt hosts,
publication inspection and cumulative checks after versioning have not run for this
draft.
Authored-limit reference capture adds 360 single-layer configurations (reversed,
partial and infinite limits). The draft passes all of them plus the original 72;
seven example tests also pass structural metadata rejection and kernel error tests
(`/tmp/ggplot-vector-transform-limits-all.log`). Shared endpoint batching and positional
training fixes passed all 691 core tests/doctests across 145 targets
(`/tmp/ggplot-vector-transform-limits-core.log`); that run predates the following
multilayer changes. A further 72 two-layer reference charts exposed concatenated paint
training; retaining population batch boundaries fixes both layers' mapped outputs
(`/tmp/ggplot-vector-transform-layers-draft-final.log`, five passing example tests).
The test harness compares standalone guide APIs only to single-population oracles;
multilayer records exercise primary chart preparation and both layers' marks.
The additional 360 combined layer/limit cases exposed the same concatenation in
positional limit training. Retaining the collector's layer/input batch boundaries
fixes all 864 single-layer and multilayer cases
(`/tmp/ggplot-vector-transform-layer-limits-draft.log`, six example tests).
A further 144 fixed/free-facet records reproduce pre-panel paint transformation.
The draft retains source-row order and layer/input groups during shared training,
and maps the full source population before selecting each panel's rows. Seven
example tests now pass all 1,008 cases, including both layers' marks and positional
facet major/minor coordinates and labels through the combined facet layout
(`/tmp/ggplot-vector-transform-facets-guides-draft.log`). Directly laying out an
isolated fixed panel does not supply the figure's shared range and is not used as
the facet guide oracle. Strict Clippy passed before the facet edits
(`/tmp/ggplot-vector-transform-layers-clippy-final.log`). The updated macOS workspace
passes 829 tests/doctests across 187 test targets, excluding the host adapter crates,
and strict core/export/example Clippy passes
(`/tmp/ggplot-vector-transform-facets-workspace.log`,
`/tmp/ggplot-vector-transform-facets-clippy.log`). These runs precede the next shared-sampler edit.
The draft now shares source-vector sampling and panel-row selection across paint,
numeric and value style consumers. Twenty-four additional pinned R point-size facet
cases pass, with all eight example tests passing
(`/tmp/ggplot-vector-transform-size-tests.log`). Restoring the earlier panel-local
numeric path reproduces a dropped point in the first ordinary cardinality facet
(`/tmp/ggplot-vector-transform-size-counterexample.log`); the shared sampler is restored.
A further 720 facet/authored-limit R cases pass, bringing focused coverage to
1,752 reference charts across nine example tests
(`/tmp/ggplot-vector-transform-facet-limits-tests.log`). Panel reconstruction now
rebinds vector transforms, limit training transforms each full source vector before
selecting panel rows, and missing automatic guide candidates do not enter scene
identity metadata. Strict core/export/example Clippy passes after these edits
(`/tmp/ggplot-vector-transform-facet-limits-clippy.log`). The core/example aggregate
found three failing targets, all concerning empty binned callback limits
(`/tmp/ggplot-vector-transform-facet-limits-core.log`). Panel selection now preserves
the original training state for empty source inputs; all three affected targets pass
(`/tmp/ggplot-vector-empty-binned-regression.log`,
`/tmp/ggplot-vector-empty-callback-regressions.log`).
Eight additional R count-statistic charts reproduce scalar transformation of
generated aesthetics. Row encoding is now separated from palette/position mapping,
so generated vectors transform across all panels of each layer first. All ten
focused tests pass 1,760 reference charts
(`/tmp/ggplot-vector-statistics-batched.log`), and strict core/export/example Clippy
passes (`/tmp/ggplot-vector-statistics-clippy.log`). The fresh macOS workspace passes
832 tests/doctests across 187 targets, excluding host adapter crates
(`/tmp/ggplot-vector-statistics-workspace.log`). Fresh Python/WASM builds each pass
180 states over the original 72 vector configurations (54 configurations through
three states, plus 18 matching construction rejections). All 180 records match
exactly, and 54 SVG/PDF/PNG files are byte-identical
(`/tmp/ggplot-vector-{python,wasm}-final.log`, `/tmp/ggplot-vector-host-compare.log`).
The harness uses the reference-visible guide configuration; hiding binned guides
changes training and is not the same oracle. All 18 successful ordinary charts
were inspected in all three formats on six contact sheets under
`/private/tmp/finstack-chart-proof-20260913/vector-transforms/inspection`.
These host/publication proofs do not yet cover the expanded facet/statistic matrix
or a retained-metadata v56 host round trip.
The qualified slice is now integrated into the working tree over revision
`6e74ae6`: 44 files were transferred after checking their prior and qualified
hashes; the unchanged original fixture was preserved. Formatting, repository
structure/link validation and `git diff --check` pass
(`/tmp/ggplot-vector-integrated-{fmt,repository,diffcheck}.log`). All 13 focused external-example tests pass in the actual checkout
(`/tmp/ggplot-vector-integrated-contracts.log`).
Other numeric/value style outputs, broader generated-statistic populations, callback
combinations and expanded host coverage remain open.
Next: continue those remaining batch boundaries and the GG-04 constructor inventory.
GG-04 and GG-05–19 remain open.

The generated-style extension is also integrated over revision `6e74ae6`: 11 files
were transferred from `/private/tmp/finstack-chart-generated-style-work` after checking
prior and qualified hashes. Its 16 count-to-paint/size R fixtures include fixed facets
and reverse composition. The first faceted cardinality paint case reproduced an
incorrect light color where the reference uses the darkest color
(`/tmp/ggplot-vector-statistic-styles-counterexample.log`). Shared layer sampling now
retains generated panel/ordinal identities and transforms one generated layer vector.
All eleven focused tests pass 1,776 reference charts
(`/tmp/ggplot-vector-statistic-styles-batched.log`). Strict Clippy passes and the
macOS workspace passes 833 tests/doc tests across 187 targets, excluding host crates
(`/tmp/ggplot-vector-statistic-styles-{clippy,workspace}.log`).
Fresh Python and WASM extension-proof builds pass 44 original/edit/error states and
36 byte-identical PNG/SVG/PDF files
(`/tmp/ggplot-vector-statistic-styles-hosts-final.log`). Paint rejects the invalid
center/reverse composition during construction; size rejects it during chart evaluation.
The primary authoring runner includes these proofs. All 12 successful charts were
visually inspected in all three formats on four contact sheets at
`/private/tmp/finstack-chart-proof-20260913/vector-statistic-styles/inspection`;
`review.json` records hashes and the inspection boundary. This does not establish
full guide/layout parity. Broader generated statistics, callback combinations and
other retained-metadata variants remain open. Both hosts now also pass the 44
states with explicit transformed bounds in version-56 descriptors, reject version-55
downgrades, and retain immutable wires through edits. Their 36 exports are identical
to each other and the inspected version-55 originals
(`/tmp/ggplot-vector-retained-{python,wasm,compare}.log`). These descriptors use
eligible replacement training; binned host variants remain outside this evidence. Repository, format and diff checks pass
(`/tmp/ggplot-vector-style-integrated-{fmt,repository,diffcheck}.log`). Next: continue those boundaries
and the GG-04 constructor inventory. The actual-checkout external-example target
passes all eleven tests (`/tmp/ggplot-vector-style-integrated-contracts.log`).
The isolated frozen-scale probe at `/private/tmp/finstack-chart-authored-vector-work`
rejects construction under the existing rule that ggplot policies require eligible
training (`/tmp/ggplot-vector-authored-counterexample.log`). This is an intentional
policy boundary, not an unimplemented frozen ggplot mode; no draft changes were
transferred. Retained metadata is compatible with eligible replacement training.
The base vector matrix additionally passes 180 exactly matching retained-metadata
host states, version-55 downgrade rejection for paint/binned descriptors and 54
byte-identical publications (`/tmp/ggplot-vector-retained-base-{python,wasm}.log`).
All 54 files are identical to the previously inspected base-vector exports;
`/private/tmp/finstack-chart-proof-20260913/retained-vectors/inspection-equivalence.json`
records the hashes. The primary runner now includes both retained suites.
The cumulative main-checkout authoring run reached all 588 scheduled commands,
ending with the successful strict GG-03 Python typing stage. Its per-stage logs
and source hashes are under
`/private/tmp/finstack-chart-proof-20260914/cumulative-vectors`, with the aggregate
log at `/tmp/ggplot-vector-cumulative-hosts.log` and a recorded completion boundary
in `completion-boundary.json`. Host modules predate the later probability,
continuous/interval and temporal integrations; native stages used main at execution
time. This completes the recorded older-module run, not a fresh cumulative
qualification of the latest source or a new whole-corpus visual inspection.
GG-04 and GG-05–19 remain open.

The probability adapter is integrated over revision `6e74ae6`: eight files were
transferred from `/private/tmp/finstack-chart-probability-work` after checking prior
and qualified hashes (`transfer-baseline.json`). Four uniform/exponential
configurations qualify 36 chart draws and 120 raw quantile/CDF outcomes, including
domain endpoints, infinities and missing values. All four native tests pass
(`/tmp/ggplot-probability-native-final.log`) and strict external-example all-target
Clippy passes (`/tmp/ggplot-probability-clippy.log`). Fresh actual Python/WASM builds
match 108 original/edit states and 36 byte-identical PNG/SVG/PDF publications
(`/tmp/ggplot-probability-hosts.log`). All twelve ordinary charts were inspected in
all formats under `/private/tmp/finstack-chart-proof-20260914/probability-transforms/inspection`;
`review.json` records file hashes and findings. Exponential positional cases 18/27
retain crowded 0.00/0.25 labels under the authored `Preserve` policy; collision
handling and complete stepped-guide presentation remain GG-05.
The initial harness hid binned guides, changing reference training; retaining guide
selection fixes the color discrepancy without engine edits. Guide candidates
explicitly adapt raw reference labels to native visibility semantics. This
supplements normal/logistic built-in proofs and qualifies the existing registered
quantile/CDF adaptation, not an embedded R distribution library. The primary runner
now includes it. The already-running cumulative process started before this addition;
its eventual result must be reported with this separately qualified supplement.
The integrated four-test target, formatting, repository and whitespace checks pass
(`/tmp/ggplot-probability-integrated-{native,fmt,repository}.log`). Generic palette
reconciliation now links all three generic constructors to the existing 324-draw
`do.call` capture and 979-state/54-publication proof, avoiding duplicate fallback
work. Its explicit hidden guides leave default selection/NULL-break suppression
as the next distinct constructor boundary. No new runtime gate is claimed by that
coverage-index update. Next: reconcile that boundary and finish the cumulative run.
GG-04 and GG-05–19 remain open.

## GG-04 temporal interval label callbacks — 14 September 2026

The next isolated FIX-GG04 draft reuses the temporal selection proof with a
2,160-draw callback/format capture. It records 1,654 successful draws and 506
reference rejections over Date/datetime, color/size/alpha, bins/steps, five callback
result modes, two format controls, fixed/inferred limits and three populations.
Reference command: `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library
/usr/local/bin/Rscript tools/reference/r/temporal-interval-label-functions.R`;
log `/tmp/ggplot-temporal-interval-label-reference.log`.

The native counterexample found missing automatic calendar names on interval
callback inputs (`/tmp/ggplot-temporal-interval-label-counterexample.log`). The draft
now retains those names in the common temporal selector and honors explicit
`date_labels` precedence over registered labels at interior and endpoint calls.
Both native tests pass: the existing 648-case selection matrix plus 2,160 new
cases in all four units (`/tmp/ggplot-temporal-interval-label-native.log`). Callback
values, class, timezone, names, sequence and labels match the reference.

The fix is integrated at `6e74ae6` plus owned changes. Seven files were transferred
after checking all baseline hashes and the two shared captures; the identities are
in `/private/tmp/finstack-chart-proof-20260914/temporal-interval-label/integration.json`.
The macOS workspace passes 844 tests across 194 targets with no failed or ignored
tests (`/tmp/ggplot-temporal-interval-label-workspace.log`). Strict core Clippy,
formatting and repository checks pass
(`/tmp/ggplot-temporal-interval-label-{clippy-final,fmt,repository}.log`).

Fresh actual Python/WASM modules match 21,872 records: 1,654 successful cases in
four units and three original/edit states, plus 2,024 expected rejections. All 180
publication files are byte-identical, and all sixty triplets were inspected;
`temporal-interval-label/inspection/review.json` records hashes and findings.
The same modules also pass 7,680 temporal selection records and 2,720 numeric
interval-label records, with all 198 and 108 publication files unchanged from
previously inspected outputs. The full helper log is
`/tmp/ggplot-temporal-interval-label-hosts.log`; the primary runner now includes the
new mode of the existing host scripts. Interval, colorbar and numeric guide
presentation remain GG-05. No blanket interval DST or arbitrary callback-composition
claim follows from this UTC matrix. Next: reconcile remaining GG-04 constructor
and argument boundaries against the coverage index.
GG-04 and GG-05–19 remain open.

## GG-04 temporal interval selection — 14 September 2026

The isolated draft in `/private/tmp/finstack-chart-continuous-guide-work` extends
FIX-GG04 temporal guide selection to `TemporalBins` and `TemporalSteps` using the
existing interval and calendar owners. Definition version 60 rejects downgrades.
All 648 captured selections pass across four timestamp units: 2,544 successful
comparisons and 48 expected rejections (`/tmp/ggplot-temporal-interval-native.log`).
Typed Date/POSIX break vectors mask endpoints even for explicit empty selection;
constant inferred bins reject when no cuts survive. Palette evaluation converts
relative timestamp keys into absolute Date/POSIX units, and calendar pretty labels
are retained for interior cuts with separately formatted endpoint labels.

The changes are now integrated at `6e74ae6` plus owned changes. Twelve files were
transferred only after checking the baseline hashes and unchanged reference captures;
`/private/tmp/finstack-chart-proof-20260914/temporal-interval/integration.json` records
their identities. The native proof additionally compares retained interval key paints
and numbers (`/tmp/ggplot-temporal-interval-native-final.log`). The macOS workspace
passes 843 tests across 194 targets, with zero failed or ignored tests
(`/tmp/ggplot-temporal-interval-workspace.log`). Strict core Clippy, formatting and
repository checks pass (`/tmp/ggplot-temporal-interval-{clippy,fmt,repository}.log`).

Fresh Python and WASM modules pass 7,680 exactly equal records: 636 successful draws
in four timestamp units and three replay/edit states, plus 48 expected error records.
All 198 publication files are byte-identical across hosts. Logs:
`/tmp/ggplot-temporal-interval-{python,wasm,compare}.log`. An initial stronger key
comparison expected hex strings where the wire retains typed RGB values; the adapter
now verifies exact RGB channels, and both host proofs were rerun. No fixture changed.
All 36 new PNG/SVG/PDF triplets were inspected; hashes and findings are recorded in
`temporal-interval/inspection/review.json`. The 90 earlier temporal files are unchanged.
The same fresh modules pass the 2,720-state interval-label regression with all 108
files unchanged from prior inspected output. The primary runner includes expanded
counts. Guide presentation still uses ordinary swatches, including the final missing
bin swatch, and numeric guides remain unpainted; those are GG-05 requirements.
This matrix does not certify interval callback/format or DST combinations.
Next: reconcile the remaining temporal argument combinations and GG-04 coverage.
GG-04 and GG-05–19 remain open.

## GG-04 temporal guide selection — 14 September 2026

At `6e74ae6` plus owned changes, source-constructor reconciliation found that Date
and datetime color defaults select colorbars, but the temporal descriptor retained
only point keys. The new `temporal-guide-selection` capture records 648 draws:
two timestamp classes, color/size/alpha, inferred/fixed limits, ordinary/constant/
empty populations, six guide selections and automatic/NULL/empty breaks. It records
636 successful draws and 12 interval-guide rejections. The initial palette-call
instrumentation was corrected to wrap only materialized palettes; null numeric
palettes must keep their original deferred theme fallback. No reference behavior
was changed to accommodate the implementation.

The isolated draft in `/private/tmp/finstack-chart-continuous-guide-work` adds
version-59 `TemporalColorbar` to the existing calendar and colorbar owners.
The native counterexample retained zero samples versus the expected 300
(`/tmp/ggplot-temporal-guide-selection-counterexample.log`). The repaired native
proof now passes 432 legend/colorbar/hidden draws in all four timestamp units,
1,728 comparisons including source-backed marks, exact labels, sample colors and
bounded numeric sample comparisons (`/tmp/ggplot-temporal-guide-selection-native.log`).
The 216 interval selections remain outside this draft. Reference command:
`R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library /usr/local/bin/Rscript
tools/reference/r/temporal-guide-selection.R`; log
`/tmp/ggplot-temporal-guide-selection-reference.log`.

The temporal colorbar changes are now integrated. Eight existing source baselines
and both shared capture hashes were verified before transferring eleven files;
`/private/tmp/finstack-chart-proof-20260914/temporal-colorbar/integration.json`
records their identities. Fresh Python/WASM modules pass 5,184 original/replay,
layer-edit and theme-edit states, with exact host records and 90 byte-identical
publications (`/tmp/ggplot-temporal-colorbar-hosts.log`). All thirty PNG/SVG/PDF
triplets were inspected; `temporal-colorbar/inspection/review.json` records hashes
and findings. The same modules pass the 2,720-state interval-label supplement;
all 108 of those files are unchanged from the earlier inspected outputs.

The macOS workspace passes 843 tests across 194 targets, with no failed or ignored
tests (`/tmp/ggplot-temporal-colorbar-workspace.log`). Strict core Clippy, formatting
and repository checks pass (`/tmp/ggplot-temporal-colorbar-{clippy,fmt,repository}.log`).
The primary runner schedules the new native and both actual-host proofs. Temporal
point keys retain source-unit offsets; ramp values retain the normalizer's absolute
Date days/POSIX seconds. This is scale-side qualification: rendered colorbars still
use ordinary swatches, and numeric guides remain unpainted. The older cumulative
host run is separate evidence and continues on its originally built modules.
Next: temporal interval selection and remaining GG-04 reconciliation.
GG-04 and GG-05–19 remain open.

## GG-04 continuous guide selection — 14 September 2026

At `6e74ae6` plus owned working-tree changes, the new pinned
`continuous-guide-selection` capture records 162 primary draws across colour,
size and alpha, ordinary/missing/empty populations, six guide selections and
three break policies. The existing engine passes the 81 default/legend/hidden
cases in `ggplot_palette_selection`: palette callback batches, visible values and
labels match exactly after JSON integer/float normalization. The typed adaptation
lowers `breaks=NULL` or `guide="none"` to `Hidden`, and an empty vector to explicit
empty continuous candidates. This is source-backed adaptation, not R syntax support.

Commands: `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library /usr/local/bin/Rscript
tools/reference/r/continuous-guide-selection.R` and `mise exec -- cargo test -p
chart-core --test ggplot_palette_selection continuous_legend_and_suppression --locked`.
Logs are `/tmp/ggplot-continuous-guide-{reference,native}.log`. The Python/CJS
`ggplot_continuous_guide_selection` scripts run against the actual cumulative
modules under `/private/tmp/finstack-chart-proof-20260914/cumulative-vectors`:
243 original/layer-edit/theme-edit states match exactly, and all 27 publications
are byte-identical. No core or host implementation changed for this proof, so the
existing freshly built cumulative modules were reused. Comparison and all nine
inspected PNG/SVG/PDF triplets are under
`/private/tmp/finstack-chart-proof-20260914/continuous-guide-selection`; the inspection
`review.json` retains hashes. Color default swatches appear and NULL/empty suppress
them. Alpha/size guide metadata passes, but their missing painted guides remain
GG-05. This is not complete guide presentation parity.

The remaining 81 captured cases expose separate colourbar/bins/coloursteps
selection contracts. Reference colorbars invoke the palette on a separate 300-value
batch; continuous bins/steps use interval samples. Those selections are not yet
qualified by the existing continuous candidate descriptor. Next: implement their
scale-side computation/selection, reconcile generic discrete/binned suppression,
and finish the still-running cumulative validation. The primary runner now includes
this supplement, but its already-running process did not schedule the new calls.
The complete four-test palette-selection target, formatting, repository and
whitespace checks pass (`/tmp/ggplot-continuous-guide-{target,fmt,repository,diffcheck}.log`).
GG-04 and GG-05–19 remain open.

The subsequent colorbar draft is isolated in
`/private/tmp/finstack-chart-continuous-guide-work`; it has not changed main core.
The capture now adds nine all-outside colorbar cases (171 total), plus retained raw
scale breaks and decoration samples. Four native target tests pass, including 117
supported selection draws (`/tmp/ggplot-colorbar-qualified-target.log`). The draft
adds explicit v57 colorbar selection and 300 retained mapping samples, with
non-color and no-visible-key suppression. Fresh Python/WASM modules pass 351 exact
original/layer-edit/theme-edit states and 63 byte-identical publications
(`/tmp/ggplot-colorbar-hosts-final.log`). All 21 PNG/SVG/PDF triplets were inspected;
`/private/tmp/finstack-chart-proof-20260914/colorbar/inspection/review.json` records
hashes and limitations. Marks and suppression agree across formats, while default
colorbar selection still paints five ordinary swatches. The isolated primary runner
now includes the 351-state supplement. The isolated macOS workspace run passed
838 tests with no failures or ignored tests (`/tmp/ggplot-colorbar-workspace.log`);
strict chart-core Clippy, formatting and repository checks also passed
(`/tmp/ggplot-colorbar-{clippy-final,fmt-check,repository}.log`). These results precede
the following generic-suppression repair and do not qualify that later source.
Actual bar painting and key positioning remain GG-05; retaining decoration samples
does not complete that presentation contract.

The subsequent `generic-guide-suppression` capture records 324 discrete/binned
primary draws across three aesthetics and populations, automatic/fixed limits,
default/none/legend selection, and default/NULL/empty breaks. Command:
`R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library /usr/local/bin/Rscript
tools/reference/r/generic-guide-suppression.R`;
`/tmp/ggplot-generic-guide-suppression-reference.log` records success. Reference empty
breaks preserve discrete mapping but collapse binned mapping to one interval.

The isolated native proof reproduced redundant palette calls for an empty population
with fixed limits and explicit empty breaks, on both discrete and binned scales.
Repairs in the existing training/mapping owners skip those calls when no marks or
eligible guide keys demand them. All five palette-selection tests pass, including
324 exact callback-batch/suppression cases
(`/tmp/ggplot-generic-guide-suppression-target.log`). This test checks callback batches,
wire round trips and suppressed guide metadata; full default-guide keys and reference
mark outputs remain outside this proof. Fresh Python/WASM modules now pass 972
original/layer-edit/theme-edit states with exactly equal retained keys/styles and
108 byte-identical publications (`/tmp/ggplot-generic-hosts.log`, artifacts under
`/private/tmp/finstack-chart-proof-20260914/generic-guide`). The colorbar supplement
also passes again on these modules: 351 equal states and 63 identical publications.
All 36 generic PNG/SVG/PDF triplets were inspected; `inspection/review.json` records
hashes, consistent suppression and the still-missing numeric guide presentation.
The example extension adds an endpoint-inclusive palette mode to reproduce this
reference's count palette without changing the existing example modes. Strict
Clippy passes for chart-core and the example extension
(`/tmp/ggplot-generic-clippy.log`). The refreshed macOS workspace passes all 839
tests across 192 targets with no failures or ignored tests
(`/tmp/ggplot-generic-workspace.log`); formatting and repository checks pass too
(`/tmp/ggplot-generic-{fmt-check,repository}.log`). The qualified colorbar and
suppression changes are now integrated after verifying unchanged source baselines;
`generic-guide/integration.json` records the 18 integrated file hashes. The integrated
checkout also passes all five palette-selection tests
(`/tmp/ggplot-generic-integrated.log`). The existing
cumulative host run continues against its previously built modules; its outcome
will not be attributed to the newer integrated source. Next: complete that run's
evidence and address the remaining continuous interval-guide selections.
GG-04 remains open.

The continuous interval selections are now integrated over `6e74ae6` plus owned
changes. Definition version 58 retains `ContinuousBins`/`ContinuousSteps`, reusing
the existing interval parser, midpoint mapping and label machinery. The primary
selection capture now has 189 draws, including uneven breaks. All six native
palette-selection tests pass; the additional built-in-palette regression checks
that interval keys are sampled and continuous mark styles remain unchanged.

A separate pinned `continuous-interval-transforms` capture contains 108 draws over
log10, cardinality and center transforms, including index palettes that expose
missing-class pooling in actual marks. This reproduced incorrect interval endpoints
from scalar transformation and the merging of source NA with generated NaN.
The shared mapping now uses trained transformed bounds and samples guide values
without another forward transform. An internal missing marker preserves NA/NaN
pool identity; it does not add an R-specific NA token to numeric wire transport or
arbitrary callback parameters. The external example target passes all 108 callback,
key, label and mapped-key comparisons, plus index-palette mark comparisons
(`/tmp/ggplot-continuous-interval-transform-target.log`).

Reference commands use `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library
/usr/local/bin/Rscript` with `tools/reference/r/continuous-guide-selection.R` and
`continuous-interval-transforms.R`; logs are
`/tmp/ggplot-continuous-interval-{reference,transform-reference}.log`.
Fresh Python/WASM modules pass 216 linear interval states and 72 identical files;
they also requalify 351 colorbar states and 63 identical files
(`/tmp/ggplot-continuous-interval-complete-hosts.log`). The expanded transform host
proof passes 324 original/layer-edit/theme-edit states and 216 byte-identical files,
including independent reference mark checks for index palettes
(`/tmp/ggplot-continuous-interval-index-{python,wasm,compare}.log`). All 24 linear
and 72 transform PNG/SVG/PDF triplets were inspected. Hashes and findings are under
`/private/tmp/finstack-chart-proof-20260914/continuous-interval/{inspection,transforms-inspection}/review.json`.
Linear renders remain identical after the transform repairs. Color guides still
use ordinary swatches and numeric guides remain unpainted: GG-05 presentation,
positioning, decoration and composition are not qualified by these results.

The repaired macOS workspace passes 841 tests across 193 targets with no failures
or ignored tests (`/tmp/ggplot-continuous-interval-complete-workspace.log`); that run
used the initial 72 transform cases, followed by the successful 108-case focused
expansion. Strict core/example Clippy, formatting and repository checks pass
(`/tmp/ggplot-continuous-interval-complete-clippy.log`,
`/tmp/ggplot-continuous-interval-{fmt,repository}.log`). Integration verified fourteen
file baselines and four shared capture files; `continuous-interval/integration.json`
records their hashes. The integrated checkout passes all six palette-selection tests
and the 108-case external example target, plus formatting, repository and whitespace
checks (`/tmp/ggplot-continuous-interval-integrated-{core,example,fmt,repository,diffcheck}.log`).
All 63 colorbar publications also match the previously inspected files byte for byte,
after accounting for the renamed suffix. The primary runner includes both interval supplements.
Next: finish the earlier cumulative run's evidence, reconcile remaining GG-04
constructor arguments and registered guide-label combinations, then continue the
assigned GG-05–19 packages. GG-04 remains open.

The interval-label changes were qualified in
`/private/tmp/finstack-chart-continuous-guide-work` before integration. Their pinned
`continuous-interval-label-functions` capture records 1,080 combinations of color,
size and alpha; bins/steps; identity/log transforms; ordinary/constant/empty
populations; automatic/fixed limits; automatic/duplicate-nonfinite/empty breaks;
and five label-function modes. The shared native label runner now matches all 820
successful builds, 260 reference rejections and their callback vectors
(`/tmp/ggplot-continuous-interval-label-{reference,native}.log`).

This reproduced an outside break incorrectly retained in interval boundaries after
label censoring, and a continuous constant interval incorrectly emitting keys.
The draft excludes censored boundaries and removes the undefined-position constant
key only after evaluating the reference label callbacks. The original counterexamples
are retained in `/tmp/ggplot-continuous-interval-label-counterexample.log` and
`/tmp/ggplot-continuous-interval-label-constant-counterexample.log`. Captured parsed
boundaries remain distinct from visible key vectors: the proof uses the visible
prefix after ordinal key censoring. Existing discrete/continuous/binned label tests
also pass. The repairs are now integrated after verifying all five existing source
baselines; `continuous-interval-label/integration.json` records six transferred
files and two matching reference captures. The primary runner now schedules both
actual-host label proofs and their comparison.

Fresh Python/WASM modules each pass 2,720 states: 820 successful cases through
original/replay, layer edit and theme edit, plus 260 expected validation rejections.
The version-58 descriptor rejects downgrade to version 57. All state records match
exactly, as do 108 publication files (`/tmp/ggplot-interval-label-{python,wasm}.log`,
`continuous-interval-label/labels-comparison.json`). All 36 PNG/SVG/PDF triplets were
inspected; `continuous-interval-label/inspection/review.json` retains findings and
hashes. Constant bins suppress the undefined key; steps retain their reference key.
Ordinary color swatches and the missing numeric guide painting remain GG-05 scope.
Artifacts are under `/private/tmp/finstack-chart-proof-20260914`.

The same repaired modules requalify 216 interval states/72 files, 351 colorbar
states/63 files and 324 transformed interval states/216 files, with exact host
comparisons (`/tmp/ggplot-continuous-interval-label-hosts-regression.log`). The macOS
workspace passes 842 tests across 193 targets, with no failed or ignored tests
(`/tmp/ggplot-continuous-interval-label-workspace.log`). Strict chart-core Clippy,
formatting, repository and whitespace checks pass
(`/tmp/ggplot-interval-label-{clippy,fmt,repository,diffcheck}.log`). These scoped fresh
host proofs do not replace the earlier still-running cumulative host validation.
Next: reconcile remaining constructor arguments against the coverage index and
complete cumulative evidence before GG-04 closure. GG-04 and GG-05–19 remain open.

## GG-04 dependency transform reconciliation — 13 September 2026

The `transform` argument audit identifies an unresolved computational boundary:
the current numeric/positional enums do not represent the full pinned `scales`
transform family (for example, asinh, logit, Box–Cox and composed transforms).
`scale-transform-contracts.json` now captures all 24 dependency transform source
contracts, 29 forward/inverse configurations and 116 actual positional/paint plots
from scales 1.4.0. This is reference
evidence only; it does not certify the working-tree engine. An isolated draft at
`/private/tmp/finstack-chart-transform-work` now passes three Rust tests covering
the 29 scalar configurations and all 116 positional/paint outcomes, including
point coordinates, tick positions, labels, colors and population errors
(`/tmp/ggplot-transform-plots.log`). Strict core all-target Clippy also passes
(`/tmp/ggplot-transform-clippy.log`). The draft reuses the existing reference
projection adapter for ranges with nonfinite inverse endpoints. It is not yet
transferred into this checkout. The fresh draft core aggregate passes 672 tests
and doctests across 142 targets (`/tmp/ggplot-transform-core-all.log`), including
capability-version-53 serialization round trips. Fresh Python/WASM builds each
pass 342 states and produce 171 byte-identical publication files
(`/tmp/ggplot-transform-{python,wasm,compare}.log`). All 57 successful ordinary
plots were inspected in SVG/PDF/PNG on nineteen contact sheets under
`/private/tmp/finstack-chart-proof-20260913/builtin-transforms-draft/inspection`.
Finite inverse capability tests now pass for valid built-in branches and reject
nonfinite, degenerate and discontinuous branches (`/tmp/ggplot-transform-inverse.log`).
Strict Python and TypeScript declarations pass with the generated host declarations.
An additional 116-case R guide capture (`transform-guide-controls.json`) qualifies
58 default-label count cases in the draft, including base-0.5 logarithmic count-3
success and count-7 rejection (`/tmp/ggplot-transform-guide-test.log`, four tests).
The count extension passes 174 exact Python/WASM states and 171 byte-identical
SVG/PDF/PNG files; all 57 plots were inspected across the formats. Deliberate
`Preserve` labels can overlap, so this is scale semantics evidence, not GG-05 layout
acceptance. The refreshed original 342-state/171-file proof passes with every
previously inspected file unchanged. Draft core aggregate: 674 tests/doctests,
142 targets (`/tmp/ggplot-transform-core-all.log`); strict core Clippy passes.
The subsequent registered-label correction passes all 116 guide cases with a
fixed-two-decimal example callback, preserving existing formatter contracts
(`/tmp/ggplot-transform-guide-labels-test.log`). The callback extension passes 348 exact host states and 342 byte-identical files.
Its 171 default-label files match the inspected count proof; the 171 new files
were inspected on nineteen contact sheets. This does not certify the complete
`label_number` constructor argument surface. The subsequent finite-secondary
extension passes 29 R cases and existing secondary regressions (ten Rust tests),
plus 87 exact host states and 81 byte-identical files. All 27 secondary plots
were inspected across SVG/PDF/PNG on nine contact sheets under
`/private/tmp/finstack-chart-proof-20260913/builtin-transform-secondary/inspection`.
The refreshed original and registered-label matrices pass 342/348 states and
171/342 files. Thirty-five changed SVG/PDF files have zero rendered pixel
differences from their previously inspected versions; all PNGs remain unchanged
(`builtin-transform-secondary-refresh/render-comparison/comparison.json`).
Direct numeric envelopes additionally require version 2 for the new family and
reject version 1; legacy numeric envelopes retain version 1 (seven focused tests
in `/tmp/ggplot-transform-wire-final.log`). Standalone version-8 envelopes pass actual Python/WASM checks for five
containers, 35 rejected downgrades, copy/map preservation and a legacy version-1
control (`/tmp/ggplot-transform-wire-{python,wasm}.log`); parsed envelopes match
exactly. Fresh draft core aggregate: **676 tests/doctests across 142 targets**.
Final strict core/example-extension all-target Clippy passes
(`/tmp/ggplot-transform-final-clippy.log`). Full macOS draft qualification passes **805 tests/doctests across 174 nonzero
targets** (182 total targets), including native and example-extension targets;
Python/WASM crates are excluded from that Rust command and have the separate
actual-host proofs above (`/tmp/ggplot-transform-macos-final.log`). Composed and registered
transforms remain unresolved. GG-04 stays open. Next: transfer the qualified draft
after the ongoing cumulative host run, then reconcile remaining transform contracts. A new reference-only capture records
13 compositions and 195 actual positional/continuous/binned paint plots in
`transform-compositions.json`; an isolated ownership draft at
`/private/tmp/finstack-chart-compose-work` passes all 195 captured plots, including
ignored composed-transform counts, inherited logarithmic breaks, population rejection,
transformed-domain trimming and default labels without finite viewport inversion.
The draft borrows evaluation inputs while owning composition vectors. Ten focused
tests cover those cases and plot/standalone/direct-numeric envelopes v54/v9/v3,
empty/deep/wide rejection and nested composition; all 680 core tests/doctests and
strict all-target core Clippy pass. Actual Python/WASM standalone proofs each pass
five containers and 40 downgrade rejections with identical envelopes. The chart
matrix is 543 states (21 build errors and six errors in each of three prepared
states) and 177 publications; all states compare exactly and all files are byte-identical. All 59 plots were inspected across PNG/SVG/PDF on twenty contact sheets. Reciprocal/square-root labels crowd under the existing `Preserve` policy; this is not GG-05 layout acceptance. The refreshed original, label and secondary proofs pass 342/348/87 exact states and 171/342/81 files, with all 594 files unchanged from the previously inspected built-in outputs (`composed-refresh/publication-changes.json`). Strict Python/TypeScript declaration consumers pass, using the existing generic descriptor boundary. The full workspace rerun exposed two test ownership uses of the formerly Copy descriptors; explicit clones preserve their independent expectations. The corrected macOS workspace aggregate passes **808 tests/doctests across 174 nonzero targets** (182 total), excluding the separately qualified Python/WASM crates (`/tmp/ggplot-compose-macos-final2.log`). Strict all-target core/export/example-extension Clippy and workspace formatting checks pass (`/tmp/ggplot-compose-final-clippy.log`, `/tmp/ggplot-compose-fmt.log`). No
composition change has been transferred. Registered transforms and the remaining
GG-04 contracts stay open.

## GG-04 joint paint aesthetics — 13 September 2026

At `6e74ae6` plus working-tree changes, one scale shared by colour/fill matches
15 actual R builds/draws across continuous, binned, discrete, manual and identity
families, including missing and empty populations. The existing engine passes the
new focused Rust target; Python/WASM each pass 45 immutable lifecycle states and
15 byte-identical SVG/PDF/PNG files, inspected on three contact pages at
`/private/tmp/finstack-chart-proof-20260913/shared-paint/inspection`. The primary
runner now includes this route; the already-running cumulative process predates
the runner addition. No engine changes were needed. The 739-test aggregate below
predates this added test. GG-04 and GG-05–19 remain open; next: complete cumulative
host qualification and reconcile the remaining constructor/rejection contracts.

## GG-04 retained empty glyphs — 13 September 2026

The numeric constructor matrix now covers **180 outcomes** including
`scale_size_binned_area`, with **484 exact host states and 108 byte-identical
publications** before the following final layout correction. Inspection of all
eighteen contact pages exposed an incorrect “No data” label when negative binned
area sizes retained rows but painted no glyphs. Reference-profile layout now
recognizes retained populations with no projection omissions; actual empty
populations keep their `NoData` state. The new implicit/explicit-symbol regression,
all seventeen aesthetic tests, the 180-constructor target and all twenty layout
tests pass (`/tmp/ggplot-empty-glyph-status-native.log`). Rebuilt hosts now pass
484 states and 108 byte-identical publications. Only the three reversed-binned-area
files changed; their reinspection confirms removal of the incorrect label. Strict
Clippy and repository checks pass. The fresh macOS aggregate passes **739 tests/doctests
across 161 targets** at `/tmp/ggplot-empty-glyph-macos-aggregate.log`; the new cumulative run is at
`/tmp/ggplot-empty-glyph-cumulative.log` (output `cumulative-empty-glyph`). GG-04, cumulative qualification and GG-05–19 remain open.

## GG-04 numeric constructors and implicit circles — 13 September 2026

At `6e74ae6` plus working-tree changes, numeric constructor ranges and `max_size`
match **160 actual R outcomes** across eight constructors and five populations.
The publication check exposed implicit circles using raw size as radius; they now
share the existing reference size/stroke conversion with explicitly selected symbols.
Empty glyphs retain their prepared rows without emitting invalid scene primitives.
Native device-radius checks in points/pixels and sixteen related aesthetic tests pass.
Fresh Python/WASM match **432 states and 96 byte-identical SVG/PDF/PNG files** under
`/private/tmp/finstack-chart-proof-20260913/numeric-constructors`; all sixteen contact
pages inspected. Zero line-width hairlines retain device-dependent raster visibility.
Default-palette host regressions pass 168 states each. Strict all-target Clippy,
`/tmp/ggplot-numeric-radius-clippy.log`, passes. The fresh macOS aggregate **PASS 738 tests/doctests**,
`/tmp/ggplot-numeric-radius-macos-aggregate.log`. The cumulative run uses
`cumulative-numeric` under the proof root; it continues in
`/tmp/ggplot-numeric-cumulative-resume.log` after updating the stage assertion to
R's size/stroke/fontsize radius formula. Both stage host routes pass. Earlier cumulative attempts are partial: temporal wire assertions were updated
to their actual automatic-paint v42/numeric-theme v45 contracts, then execution was
stopped before the renderer correction. No cumulative acceptance is claimed.
Remaining formal-argument reconciliation and GG-04/GG-05–19 remain open.

## GG-04 gradient rejection timing — 13 September 2026

At `6e74ae6` plus working-tree changes, reference palette recipes now defer invalid
position errors until evaluation. Empty continuous/binned plots succeed; singleton,
ordinary, partially missing and all-missing populations still reject. Color-type
validation skips its fabricated sample only for empty fixed-output reference
palettes. Binned fallback preparation similarly skips gradient/registered evaluation,
while retaining named count-palette validation. The initial aggregate exposed the
latter distinction and its unchanged reference test now passes after narrowing the
bypass. Native **PASS 200 reference outcomes**, with finite/nonfinite constructor,
count-function and named count-palette regressions. Python/WASM match **240 states
and six byte-identical, inspected empty publications** under
`/private/tmp/finstack-chart-proof-20260913/gradient-invalid`; that host build precedes
the final deferred-sampling refinement. The aggregate additionally exposed later
standalone sampling after empty training; the fix now reuses the existing deferred
binned vector/cache owner. All **736 macOS core/export tests/doctests pass**, log
`/tmp/ggplot-gradient-values-macos-aggregate.log`. Strict all-target Clippy and
repository checks pass. The cumulative runner is rebuilding and executing under
`/private/tmp/finstack-chart-proof-20260913/cumulative`. Cumulative host requalification,
remaining formal-argument reconciliation and GG-04/GG-05–19 remain open.

## GG-04 nonfinite gradient remapping — 13 September 2026

At `6e74ae6` plus working-tree changes, the shared gradient now retains reference
NaN/infinite remapping positions using the existing portable `Number` type.
NaN pairs are removed after assigning coordinates; infinite endpoints and duplicate
positions follow the reference interpolation. A missing remapped value remains
missing for a single-color ramp. Native **PASS 156 pinned draws**, with finite
remapping/palette regressions and standalone descriptor identity/downgrade tests.
Special values require plot v52, interpolation v5 and standalone scale v7; finite
recipes retain previous envelopes. Strict all-target core/export/example Clippy
passes. Fresh Python/WASM match 468 lifecycle states and 156 byte-identical
publications; all twenty-six contact pages were visually inspected. Evidence is under
`/private/tmp/finstack-chart-proof-20260913/gradient-nonfinite`.
Empty-input rejection timing and the cumulative run remain open, as do GG-04/GG-05–19.

## GG-04 gradient remapping vector lengths — 13 September 2026

At `6e74ae6` plus working-tree changes, the shared reference Lab gradient now
normalizes explicit `values` by their own cardinality, independently of the number
of color anchors. The previous implementation incorrectly rejected shorter and
longer vectors. A new immutable 96-draw R capture reproduces that failure and now
passes for gradientn, stepsn, distiller and viridis_c through both paint channels,
including duplicate and descending positions. The original 200 constructor cases
and palette regressions pass. Fresh Python/WASM match 288 lifecycle states and all 96 publications byte for
byte; all sixteen PNG/SVG/PDF contact pages were inspected. Strict all-target
core/export/example Clippy and formatting pass. Evidence is under `/private/tmp/finstack-chart-proof-20260913/gradient-remap`.
The cumulative runner was stopped before this engine correction; its earlier
passed steps do not certify the changed engine. Nonfinite remapping positions and
empty-input rejection timing still need reconciliation. GG-04 and GG-05–19 remain open.

## GG-04 continuous/binned paint constructors — 13 September 2026

At `6e74ae6` plus working-tree changes, `CountGradient` composes the existing
count-palette and Lab owners for six-anchor viridis_c and seven-anchor distiller
recipes (plot wire v51, standalone interpolation v4, standalone scale v6).
The constructor capture found a gradient alpha midpoint mismatch: 156.5 encoded
as 157 instead of reference 156. Gradients now share the verified farver ties-to-even
alpha conversion with numeric aesthetic mappings. Two hundred pinned R draws
pass, including 40 binned rejection outcomes; palette/ordinal/named regressions
and all 765 alpha byte-boundary cases pass. Fresh Python/WASM match 480 lifecycle
states and 120 byte-identical publications, with all twenty contact pages inspected
under `/private/tmp/finstack-chart-proof-20260913/continuous-constructors`.
Strict all-target core/export/example Clippy and all 732 macOS core/export
tests/doctests pass. The cumulative primary host runner exposed an older GG-02
host assertion that compared reference aesthetic millimeters directly with scene
publication points. Both host assertions now apply the explicit 72/25.4 conversion;
the pinned reference is unchanged, and both 13-figure stage proofs pass. The
cumulative primary host runner has restarted under
`/private/tmp/finstack-chart-proof-20260913/cumulative`; completion and remaining
argument/rejection reconciliation remain open, as do GG-04 and GG-05–19.

## GG-04 discrete paint constructor forwarding — 13 September 2026

At `6e74ae6` plus working-tree changes, eighty pinned R draws now qualify default
and explicit hue, grey, Brewer and viridis discrete paint settings across both
channels and five populations. The existing palette and missing-paint adapters
match the different defaults (grey: red; hue: grey50; Brewer/viridis: NA).
Native proof and scoped strict Clippy pass. Actual v50 Python/WASM modules match
208 lifecycle states and 48 byte-identical, inspected publications under
`/private/tmp/finstack-chart-proof-20260913/discrete-constructors`. This adds recipe
and runtime evidence, with no new scale engine or wire capability. The primary
runner includes the route. Continuous/binned paint constructor reconciliation,
cumulative qualification and GG-04/GG-05–19 remain open.

## GG-04 qualitative type-list palettes — 13 September 2026

At `6e74ae6` plus working-tree changes, wire v50 retains qualitative `type`
color-vector lists with optional names and supplied hue fallback arguments. Core
selects the first shortest sufficient vector and reuses existing manual matching
without manual-limit filtering; insufficient vectors delegate to the hue owner.
Eighty pinned R draws pass, covering selection, ties, named/duplicate lookup,
missing/empty populations and lazy invalid names. Fresh Python/WASM match 206
edit/replacement states and 48 byte-identical SVG/PDF/PNG files; all eight contact
pages were inspected under `/private/tmp/finstack-chart-proof-20260913/qualitative-types`.
All five focused tests, strict core/export/example Clippy and repository checks pass.
The preceding v49 macOS core/export aggregate completed with 728 tests/doctests
passing (`/tmp/ggplot-ordinal-types-macos-aggregate.log`); it predates this policy.
The primary runner includes this route but has not run cumulatively. Fresh Linux,
remaining constructor forwarding and GG-04/GG-05–19 acceptance remain open.

## GG-04 ordinal type-vector palettes — 13 September 2026

At `6e74ae6` plus working-tree changes, v49 retains ordinal `type` color vectors.
The count adapter reuses existing Lab interpolation; named gradient palettes use
the same sampling helper. Seventy pinned R draws pass natively, including single
and empty vectors, alpha/transparent colors and lazy invalid-name errors. The
1,260-case named palette regression and all three default-constructor tests pass.
Fresh Python/WASM match 154 edit/replacement states and 30 byte-identical, inspected
SVG/PDF/PNG files under `/private/tmp/finstack-chart-proof-20260913/ordinal-types`.
Strict all-target core/export/example Clippy passes. The primary runner includes
the route; cumulative execution and fresh Linux remain open. Next: finish the
source-backed constructor/formal reconciliation. GG-04 and GG-05–19 remain open.

## GG-04 ordinal paint constructor defaults — 13 September 2026

At `6e74ae6` plus working-tree changes, the explicit `ggplot_color_ordinal` factory
selects the existing viridis count palette, bypasses theme lookup and preserves
reference NA paint instead of grey50. It applies to either color or fill without
adding an ordered-data representation or wire version. Twenty pinned R draws cover
ordinary, singleton, missing, all-missing and empty populations with/without a theme
palette. All three native default-palette tests pass (136 reference draws, including
240 temporal unit configurations). Current actual Python/WASM modules match 60 ordinal
states and 30 byte-identical, inspected publication files; the factory itself is tested in Rust.
Strict all-target core/export/example Clippy passes before the test-only shape-21
alignment. Artifacts: `/private/tmp/finstack-chart-proof-20260913/ordinal`.
The 726-test aggregate predates this additive factory. The cumulative host runner
remains unexecuted and Linux unavailable. Next: qualify ordinal constructor type
vectors and remaining forwarding; GG-04 and GG-05–19 remain open.

## GG-04 explicit binned constructor palettes — 13 September 2026

At `6e74ae6` plus working-tree changes, 108 new reference draws distinguish explicit
public binned paint palettes from generic normalized-vector palettes. The v48
`GgplotBinnedPalette::Discrete` adapter reuses the existing discrete count owner.
The new native proof reproduced a hue error at 16 colors (`#0CB702` versus reference
`#0BB702`): farver installs a D65 white point derived from chromaticities, while the
core polar-Luv conversion used rounded XYZ constants. That white point is corrected;
140 additional reference hue palettes cover counts, offsets, reversal and dark colors.
Five native regressions pass; fresh Python/WASM match 288 states and 30 inspected,
byte-identical publication files. Retained named palettes match 4,183 host states
and 54 files. Strict all-target core/export/example Clippy passes. The macOS
core/export aggregate passes 725 tests before the subsequent callback NULL fix.
The next 108-draw function capture reproduces NULL palette results incorrectly
using the grey50 fallback. The shared batch path now preserves NULL separately from
empty/short vectors. Six focused native tests pass, and fresh Python/WASM match
324 states and 36 byte-identical SVG/PDF/PNG files, all visually inspected. The final
macOS core/export aggregate passes 726 tests with zero failures
(`/tmp/ggplot-binned-functions-macos-aggregate.log`). Current artifacts are retained
in `/private/tmp/finstack-chart-proof-20260913/functions`; earlier `target` proof
artifacts and tools disappeared during execution and their old paths are historical.
The pinned R library and host tools were rebuilt outside `target`; regenerating the
108 function draws reproduces the committed fixture exactly.
Next: remaining constructor forwarding. GG-04 and GG-05–19 remain open.

## GG-04 style defaults and temporal exception — 13 September 2026

At `6e74ae6` plus working-tree changes, shape and linetype defaults request theme
lookup; explicit solid/hollow shape palettes bypass it. All 36 reference draws pass
natively; fresh hosts match 144 edit/replacement states and 36 inspected files under
`target/ggplot-default-style`. A separate 60-draw temporal capture reproduced that
Date/datetime paint constructors must ignore theme palettes while numeric defaults
use them. Automatic timestamp paint now clears lookup. All 26 focused native tests and strict
all-target core/export/example Clippy pass. Fresh macOS Python and WASM match all
180 temporal states and 30 byte-identical, inspected SVG/PDF/PNG files under
`target/ggplot-default-temporal-theme`; both new routes are in the primary runner.

The prior Linux aggregate passed 718 tests and failed three stale default-version
assertions; their numeric/color expectations are unchanged and their version checks
are corrected. The subsequent macOS core/export aggregate passed 723 tests with zero failures
(`/tmp/ggplot-default-temporal-theme-macos-aggregate.log`); the cumulative primary
runner has not run. Next:
finish retained host regressions and constructor argument reconciliation.
Docker stopped before the fresh Python build (HTTP 500, then no daemon socket);
macOS cannot launch its installed app (`kLSNoExecutableErr`). Local Python qualification
succeeded; fresh Linux aggregate execution remains blocked by that environment.
GG-04 and GG-05–19 remain open. Geometry follow-up: point-mapped linewidth has different
missing-row behavior from ggplot2's ignored point aesthetic; the temporal linewidth
proof uses segments, where the aesthetic is operative, and preserves all 60 original
mapped values. Point geometry behavior remains a GG-07 follow-up.

A scratch R theme-name probe inadvertently rewrote the already modified root
`Rplots.pdf`. Its prior uncommitted bytes were not captured, so it has not been
restored from HEAD. Subsequent reference generators use temporary PDF devices.

## GG-04 named theme palette registry — 13 September 2026

At `6e74ae6` plus working-tree changes, wire v47/theme v5 retain named palette strings
and equivalent singleton vectors. The entire pinned 138-name registry shares existing
palette/gradient owners; deterministic data adds the missing 79 HCL and 14 manual
tables. All 1,260 actual reference draws pass in both native wire forms. Fresh
Python/WASM match 4,183 states per form and 54 identical, inspected SVG/PDF/PNG files.
Strict all-target Clippy and consumer types pass. Earlier host routes also pass.

The previous Linux aggregate identified a test stripping the reference policy without
clearing its new palette lookup metadata; that setup is corrected. The v47 Linux
aggregate now runs all targets without stopping at the first failure. All 13 focused native regressions pass, including the vector test after its
loader startup delay; no aggregate or cumulative-runner pass is claimed. Next: remaining style defaults and constructor-argument reconciliation.
GG-04 and GG-05–19 remain open.

## GG-04 automatic/default theme palettes — 13 September 2026

At `6e74ae6` plus working-tree changes, automatic color/fill and continuous/ordinal
size, alpha and linewidth defaults now request the correct theme aesthetic. The
56-draw reference matrix reproduced the ignored automatic color palette and qualifies
its correction. Explicit numeric ranges, area and radius retain their reference
bypass behavior. Rust has 52 passing focused regressions and all-target core/export/
example strict Clippy. Fresh Python/WASM match 168 round-trip/layer-edit/theme-edit
states and 30 inspected publication files. Earlier registered/vector routes retain
979/438 equal states and 54 files each matching inspected output.

The earlier Linux run passed 719 tests but failed its final export doctest with
E0460 after a concurrent Python build replaced an artifact. A stable-source rerun
is running without overlapping builds. Next: named palette coercion and per-constructor
argument reconciliation; GG-04 and GG-05–19 remain open. No cumulative runner claim.

## GG-04 theme color vectors — 13 September 2026

At `6e74ae6` plus working-tree changes, wire v46/theme v4 retain color vectors,
using existing Lab gradients for continuous/binned scales and a count palette with
missing overflow for discrete scales. The 180-draw native test matches 110 successes
and 70 reference errors, including palette-NA versus input-NA with explicit missing
replacement. An additional counterexample reproduced theme selection being redirected
by an earlier constant-overridden paint mapping; mutable and read-only walks now
select the same active channels.

All 44 focused native regressions and all-target core/export strict Clippy pass.
Fresh Python/WASM match 438 vector states and 54 inspected SVG/PDF/PNG files; retained
registered palettes match 979 states and 54 files. Strict consumer typing passes.
The Linux run passed 719 tests before the export-doctest artifact conflict described above;
a stable-source rerun is running.
Next: named palette coercion and automatic/default constructor palette lookup.
GG-04 and GG-05–19 remain open; the cumulative primary proof runner has not run.

## GG-04 ordered theme palette lookup — 13 September 2026

At `6e74ae6` plus working-tree changes, 108 additional reference draws qualify
size/alpha aesthetic ordering, an absent first aesthetic, alias-only palettes,
`color` overriding `colour`, and the authored `color` aesthetic alias. All prior
216 records remain unchanged. The 324-draw native test passes; current hosts match
979 states and 54 files exactly matching inspected output. Explicit palette setter
replacement is also exercised natively. No production change beyond adding the
new field's empty default to three remaining export-test/benchmark initializers.
The first Linux build identified those initializers; the corrected aggregate is
running. Next: built-in theme palette values/coercions and remaining constructor
contracts. GG-04 and GG-05–19 remain open.

## GG-04 registered theme palette selection — 13 September 2026

At `6e74ae6` plus working-tree changes, wire v45/theme v3 preserve explicit versus
fallback palette selection. The shared compiler consults registered theme palettes
before training and retains the authored fallback. All 216 pinned reference draws
match callback order and marks across continuous/discrete/binned color, size and
alpha. The capture exposed and fixed two shared defects: continuous built-in NA
paint handling suppressed valid marks, and area-size square roots retained negative
infinity instead of missing. All 42 focused regressions, strict core-test Clippy,
repository checks and strict Python/TypeScript consumer checks pass. Fresh hosts
match 619 states, including theme edits, replacements, held snapshots and seven
rejections. All 54 publication files match across hosts and are inspected.
Next: ordered multi-aesthetic/alias lookup and remaining palette coercion and
constructor argument contracts. Built-in theme values, full inheritance and guide
presentation remain unqualified; GG-04 and GG-05–19 remain open. The prior 716-test
Linux result predates v45.

## GG-04 numeric constructor ranges — 13 September 2026

At `6e74ae6` plus working-tree changes, 72 additional reference builds cover
ascending, reversed and constant numeric ranges and area `max_size` values 9, 2
and 0. All prior 216 reference records are unchanged. The expanded native test
passes 276 applicable standalone and primary cases; strict Clippy passes. Current
Python/WASM match 502 states for constructor bins and 502 for the retained legacy
route, with 24 publication files matching inspected output. The primary proof
runner retains the expanded corpus. No production or wire change was required.
The completed Linux aggregate passed 716 tests before these latest assertion
extensions; `/tmp/ggplot-constructor-breaks-linux.log`. Theme-selected palette
fallback and remaining constructor forwarding are next. GG-04/GG-05–19 remain open.

## GG-04 named numeric constructor guide keys — 13 September 2026

At `6e74ae6` plus working-tree changes, all 204 applicable named size, area,
alpha and linewidth constructor cases now compare primary guide keys as well as
standalone values. The reference capture retains default bins keys, midpoint
palette outputs, endpoint labels and NULL-break suppression. The existing native
numeric test passes the expanded assertions; focused strict Clippy passes. Both
hosts reproduce 370 identical original/restored/edited/error records and 12
publication files matching inspected output. No production or wire change.
The 12 unsupported named-wrapper `right` calls remain reference-only rejection
cases. Theme-selected palettes, customized ranges and other constructor forwarding
still need qualification. GG-04 and GG-05–19 remain open.

## GG-04 omitted binned guide default — 13 September 2026

At `6e74ae6` plus working-tree changes, 108 actual reference draws with the generic
binned constructor's guide omitted exactly reproduce explicit bins, including
21 errors. This applies to color, size and alpha; the default is not inferred from
the aesthetic. All 12 native pipeline tests pass. The expanded 540-draw fixture
produces 1,068 identical Python/WASM records and 168 identical publication files,
all matching inspected output. No production or wire change was needed: host
lowering selects the retained `BinnedBins` descriptor. Named constructor palette
fallbacks, argument forwarding and full guide presentation remain open; this does
not close GG-04 or qualify GG-05–19. Next: named numeric constructor defaults and
fallback palettes, then the remaining argument contracts.

## GG-04 registered breaks with constructor guides — 13 September 2026

At `6e74ae6` plus working-tree changes, explicit bins preserve named, parsed break
candidates while deferring registered labels until after midpoint mapping. Previously,
the deferral discarded names and retained missing cuts. The existing 2,880 registered
and 80 default-label reference builds now also pass with explicit bins/colorsteps:
1,836 successes and 1,124 reference errors. All 30 focused native tests, strict Clippy and repository checks pass. Fresh
Python/WASM reproduce 4,802 identical original/restored/edited/replacement/error
records. This uses retained v44 descriptors and does not change constructor defaults.
Publication was not rerun for this metadata correction. Full guide presentation,
constructor defaults/fallbacks/forwarding and GG-04/GG-05–19 remain open.

## GG-04 binned label callback order — captured slice qualified, 13 September 2026

At `6e74ae6` plus working-tree changes, bins and stepped color map before invoking
registered labels, preserving reference palette/OOB/rescaler/label order. The new
108-draw capture includes indexed, short and missing labels across color/size/alpha,
ordinary/missing/empty inputs and default/indexed rescaling. All 84 successes and
24 errors reproduce; all 23 focused native tests and strict Clippy pass. Fresh
Python/WASM match 268 original/restored/edited/replacement/rejection states and
54 publication files, each exactly matching inspected output. No wire bump beyond
v44. Full guide painting, default-guide adaptation, and other constructor
composition remain unqualified. Next: constructor default selection and registered
break composition, then defaults/fallbacks/forwarding. GG-04 and GG-05–19 remain open.

## GG-04 explicit bins and colorsteps — captured selection qualified, 13 September 2026

At `6e74ae6` plus working-tree changes, wire v44 retains explicit bins/stepped
scale guide selection. Bins map interval midpoints; colorsteps map cuts and
midpoints and retain missing endpoint keys. Non-color stepped selection suppresses
the guide before training, preserving the reference's automatic-limit timing.
All 432 captured selections pass natively (383 successes and 49 reference errors),
including complete successful OOB/rescaler/palette sequences. The 21 focused native
pipeline/binned tests, strict focused Clippy and repository checks pass. Fresh
Python/WASM agree on 873 states and 168 publication files; 18 new renders are
inspected and 150 files exactly match previously inspected output. The painter
still shows existing interval swatches, so this does not qualify GG-05 guide
presentation. Registered break/label composition and constructor default selection
need reconciliation next. The 710-test Linux aggregate predates this change;
no new aggregate result is claimed. GG-04 and GG-05–19 remain unfinished.

## GG-04 hidden and legend binned constructors — 13 September 2026

At `6e74ae6` plus working-tree changes, 216 pinned actual-draw cases qualify hidden
and explicit legend selection across color, size and alpha; authored, automatic
and identity-callback limits; ordinary/missing/empty populations; and four vector
operations. The native test matches successful callback sequences, mark colors,
size values and guide keys, and reproduces the 21 reference errors. The shared
pipeline test helper preserves the original 288-case regression. Eight native
pipeline tests and focused strict Clippy pass. Existing current Python/WASM builds
match 433 states, including 18 callback-limit replacements and four rejection
checks, and all 108 publication files. Thirty-six new renders are inspected;
72 files exactly match inspected baseline/duplicate renders. No production scale
code changed in this qualification. Next: explicit bins/colorsteps scale-side
selection, then remaining constructor defaults/fallbacks/forwarding. Full guide
presentation remains GG-05. GG-04 and GG-05–19 remain unfinished.

## GG-04 constructor coverage reconciliation — 13 September 2026

At `6e74ae6` plus working-tree changes, the coverage index now explicitly enumerates
276 inherited method signatures and the captured fields of all 11 scale classes,
alongside the 141 function signatures. Exact source comparison passes for all 152
exports and 960 function formal occurrences; every linked test target exists.
The coverage narrative now records the qualified helper and positional-population
slices and identifies stale historical gaps as superseded. This documentation
reconciliation does not qualify full constructors or class protocols. Next:
source-backed default/fallback/forwarding acceptance and scale-side guide selection;
full presentation stays in GG-05. GG-04 and GG-05–19 remain unfinished.

## GG-04 mixed identity-source routes — focused qualification, 13 September 2026

At `6e74ae6` plus working-tree changes, source populations combine ancestor filters,
intersect panel targets and retain matched filtering through identity chains.
Matched source order is preserved at later callback stages even when the consuming
layer broadcasts. The earlier identical-target restriction is removed. All 28
focused native vector/facet tests pass, including 240 mixed-route configurations.
Fresh Python/WASM agree on 520 states and 108 publication files; all distinct
renders have been inspected, with exact equality to inspected files for duplicates.
Strict Clippy, rustdoc and repository graph checks pass. The Linux core/export
aggregate passes 710 tests, zero failures/ignored, across 146 executables and three
doctest targets (`/tmp/ggplot-mixed-source-linux-tests.log`). This aggregate predates
the new guide-selection test below. Next: constructor/formal reconciliation and any remaining
scale composition boundaries. GG-04 and GG-05–19 remain unfinished.

## GG-04 identity source chains — focused qualification, 13 September 2026

At `6e74ae6` plus working-tree changes, positional source callbacks resolve through
identity-transform ancestors and preserve every source filter. Matching facet
and chart-scope declarations retain the existing population adapter. All 27 focused
native vector/facet tests pass; the added 96 identity-chain configurations include
independent index expectations and filter-excluded sentinel observations. Fresh
Python/WASM agree on 208 states and 48 publication files, also byte-identical to
the inspected panel/scope publications. Strict focused Clippy, rustdoc and repository
checks pass. Statistics over generated statistical rows retain their existing
schema boundary; mixed facet/chart scope inside a source identity chain remains
explicitly unsupported by this adapter. Next: constructor/formal reconciliation
and remaining composition boundaries. GG-04 and GG-05–19 remain unfinished.

## GG-04 panel-target and chart-scope vectors — focused qualification, 13 September 2026

At `6e74ae6` plus working-tree changes, explicit panel targets expand callback
inputs only into their named panels, in facet order. Chart-wide source scope uses
the full filtered population per destination panel. Shared source statistics reuse
that adapter; their aggregate presentation still requires explicit targeting.
All 17 native vector tests pass, including 96 panel/scope composition configurations.
Fresh Python/WASM agree on 208 original/restored/replacement states and all 48
publications; all 16 charts have been inspected. Focused strict Clippy passes.
The 706-test Linux aggregate below predates this extension. Next: source-preserving
transform chains, distinguish existing generated-statistic rejection from GG-04
scale requirements, then complete constructor reconciliation. GG-04 remains open.

## GG-04 broadcast positional vectors — captured slice qualified, 13 September 2026

At `6e74ae6` plus working-tree changes, execution-local caches retain separate
source-row results per broadcast panel. Fixed populations preserve panel-block
order; matched populations retain source insertion order. The common registry
still owns callback invocation, recycling and bin classification. All 216 numeric
and 432 binned reference cases pass; their shared-statistic variants and all 15
native vector tests pass. Fresh Python/WASM agree on 384, 188, 752 and 364 states,
including replacements; all 108 publications match exactly and have been inspected.
Matched-facet host regressions pass 425/186/704/346 states; all 108 publications
match both hosts and their previously inspected baselines. Focused Clippy and
strict rustdoc pass. The Linux core/export aggregate passes 706 tests across 149
executables; this predates the subsequent panel/scope extension. Next:
remaining explicit panel,
chart-wide/generated-input boundaries and constructor reconciliation. GG-04 is open.

## GG-04 typed limit helpers — captured slice qualified, 13 September 2026

At `6e74ae6` plus working-tree changes, exact temporal endpoints retain their units
and refill missing limits after data changes through the existing population owner.
Wire v43 carries these endpoints. Typed `xlim`/`ylim` constructors select numeric
reversal, category order or Date/UTC behavior without duplicating a scale engine.
All 48 pinned cases are covered by native checks and 181 exactly equal Python/WASM
states; all 48 matching publications have been inspected. Fresh host regressions
also pass 180 temporal-color states and 96 blank states, with exact manifests and
all 54 blank publications matching. The nine final focused native tests, strict
focused Clippy, rustdoc and repository graph checks pass. The 630-pass/one-failure
macOS aggregate predates the constructor wrappers; its outdated automatic-paint
version assertion is fixed and all five tests in that suite pass. This is focused
qualification, not a rerun of the entire aggregate or GG-04 acceptance.
Next: broadcast source callback populations and constructor reconciliation.
A new 216-case broadcast reference capture is available; implementation is running.
GG-04 and GG-05–19 remain unfinished.

## GG-04 blank training layers — focused qualification, 13 September 2026

At `6e74ae6` plus working-tree changes, blank layers preserve optional positional
training without marks or missing-row diagnostics. All 30 reference expansion
cases pass, including shared color training and blank-only guide exclusion.
Automatic ggplot paint scales share ownership by aesthetic; explicit metadata
selects wire v42 and retains identity, palette and title through edits. Blank
layers without automatic paint scales use v41. All 30 focused authoring/aesthetic/
helper tests pass, plus the new palette-edit/append regression. Fresh Python and
WASM each pass 96 states; all manifests and 54 publications agree exactly and
all publications have been inspected. The primary proof runner includes this slice.
Strict focused Clippy and core/export rustdoc pass. The later macOS aggregate result is recorded above; earlier aggregates predate
these changes. Temporal regressions also
pass eight native tests and fresh host proofs of 1,328, 457 and 1,727 states.
All 102 temporal publications agree exactly between hosts; changed singleton
vectors remove censored off-viewport circles and retain identical PNG pixels.
Their SVG/PDF renders have now been inspected. Next: typed limit helpers,
broadcast/chart-wide callback sources and constructor reconciliation.
GG-04 and GG-05–19 remain unfinished.

## GG-04 joint positional-bin callbacks — Linux aggregate passed, 13 September 2026

At `6e74ae6` plus working-tree changes, source OOB ranges retain limit-vector
arity; scalar classification uses callback output. Break input distinguishes NULL
limits from empty numeric vectors, and bin reset preserves NULL break results.
All 1,728 joint reference cases and 14 final focused native tests pass. Fresh
Python/WASM pass 2,692 joint, 2,744 coordinate-stage and 1,164 unfaceted states;
all manifests and 84 export files agree exactly. New joint and dimension
publications have been inspected. A separate dimension reference capture exposed
eight build-only false acceptances, now rejected without changing the original
fixture. The shared inverse-log owner uses portable math after an observed one-ULP
host difference. All 689 Linux core/export tests pass across 144 executables;
strict Clippy passes. These runs predate the blank-layer changes above. The macOS
aggregate also passed 618 tests across 124 executables; strict rustdoc passed.
The temporal extension failures found afterward are corrected above.
All 152 constructor source contracts now match the formal inventory; 48 typed-limit
and 30 expansion-helper cases are captured but not implementation-qualified.
Next: finish aggregate checks, implement train-only blank layers and typed helper
dispatch, and reconcile the remaining constructor contracts. GG-04 remains open.

## GG-04 positional-bin facet vectors — qualified captured slice, 13 September 2026

At `6e74ae6` plus working-tree changes, bin classification rejects an empty vector
immediately, preserving callback ordering across free panels. A wholly empty layer
skips callbacks. Free-panel training defers callback-driven empty classification to
that stage. All 432 direct and 216 shared mean-facet reference cases and 40 focused native
tests pass. Fresh Python/WASM pass 704 direct, 346 shared and 1,164 unfaceted
regression states. All 54 facet exports agree exactly and have been inspected.
Strict Clippy and rustdoc pass. The empty-vector error now occurs at bin
classification rather than generic recycling. Next: joint callbacks and the
constructor inventory. GG-04 remains open.

## GG-04 positional-bin vectors — qualified captured slice, 13 September 2026

At `6e74ae6` plus working-tree changes, positional bins evaluate registered OOB
vectors before source classification through the common population adapter.
Post-statistic interval mapping reuses the existing bins without another callback.
All 648 pinned cases and 29 focused native tests pass. Fresh Linux Python and
WASM each pass 1,164 exactly equal states, including 12 retained replacements;
all 36 byte-identical publications have been inspected. Strict Clippy and rustdoc
pass. The 685-test Linux aggregate from before this slice remains historical;
the whole aggregate has not been rerun after these bin changes. Next: qualify
faceted/joint positional-bin callback populations and reconcile the constructor
inventory. GG-04 and GG-05–19 remain unfinished.

## GG-04 temporal positional vector functions — qualified temporal slice, 13 September 2026

At `6e74ae6` plus working-tree changes, the common vector adapter converts numeric
callback values and limits at the temporal unit boundary. A new 324-case pinned
reference capture covers duration/date/datetime points and means. All 972 native
unit configurations pass values, limits, guides and callback arguments. All 26
focused temporal/vector/stage regression tests and a represented-offset precision
regression pass. Untrained temporal limits and empty duration guide errors now
retain reference behavior. Checked represented-number conversions support distant
callback offsets while the source timestamp span restriction remains unchanged.
Fresh Python and WASM each pass 1,727 states, with 36 byte-identical, inspected
publications. Six export tests pass after a fix skips fully clipped circles before
backend precision conversion. Strict Clippy, rustdoc and repository checks pass.
The complete Linux core/export regression run passes all 685 tests across 144
executables. Next: implement positional-bin callbacks against the new 648-case
reference capture, then reconcile constructor contracts. GG-04 is open.

## GG-04 shared positional facet vectors — qualified matched slice, 13 September 2026

At `6e74ae6` plus working-tree changes, shared source nodes use the common fixed/free
population adapter and reuse callback samples across panels. One prepared-table
helper retains non-identity population provenance for mapping, empty-summary rejection
and panel retention. All 108 shared mean-facet reference cases and all 25 focused
native tests pass. Rebuilt Python/WASM pass 186 exactly equal states and 18 byte-identical,
inspected publications; 102 unfaceted and 425 unshared host regression states also
pass. Strict Clippy, rustdoc and repository checks pass. The clean Linux core
run passes all 612 unit/integration tests.
The preceding broad Linux attempt compiled the unfaceted implementation before a
new shared-facet test was added, so it is not aggregate acceptance evidence.
Next: primary temporal/duration/binned positional callbacks and reconcile constructor contracts. GG-04 remains in progress.

## GG-04 shared positional vector transforms — qualified unfaceted slice, 13 September 2026

At `6e74ae6` plus working-tree changes, unfaceted shared source statistics bind
positional callbacks once per node, after consumer scale-context checks. Identity
consumers preserve the summary empty-population rule. All 54 pinned summary cases
now also pass with two shared consumers; a separate regression verifies filtered
node input, reversed declaration order and callback-free conflict rejection.
All 24 focused native tests pass. Rebuilt Python/WASM pass 102 exactly equal
states and 27 byte-identical, inspected publications. Strict Clippy, rustdoc and
repository checks pass. The initial broader Linux run was superseded during the next facet slice;
see the current entry. Faceted shared vectors remain explicitly unsupported;
GG-04 remains in progress. Next: continue shared facet
populations and constructor contracts before GG-05–19.

## GG-04 positional facet populations — qualified matched numeric slice, 13 September 2026

At `6e74ae6` plus working-tree changes, all 246 matched numeric facet reference
cases pass callback, coordinate and guide comparisons. A shared population adapter
preserves fixed/free ordering and applies return-length recycling across the layer.
All 33 focused native/Linux tests pass. Rebuilt Python and WASM pass 425 exactly
equal states, including eight replacement comparisons; all 36 byte-identical
publications have been inspected. Final strict Clippy, rustdoc and repository
checks pass after an equivalent empty-row predicate cleanup. Broadcast,
chart-wide and shared transform sources remain unsupported by this adapter;
temporal/binned routes and overall GG-04 are still unqualified.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records exact boundaries.
Next: broaden positional populations and reconcile remaining
constructors before GG-05–19.

## GG-04 positional OOB vector functions — qualified numeric slice, 13 September 2026

At `6e74ae6` plus working-tree changes, a new positional adapter uses the existing
pure vector registry before statistics and after generated mapping. Authored axis
selection requires wire v40; source results retain row identity in execution-local
mapping state. Shared post-statistic limit training owns the second mapping pass.
All 324 point mapping/guide cases and 54 selected mean-summary cases pass in Linux.
Both actual hosts pass 732 exactly equal states, eight replacement comparisons and
36 byte-identical, inspected publications. All 23 focused native tests, strict
Clippy, rustdoc and repository checks pass. Both new proofs are registered in the
primary proof runner. Facets/shared transforms
were unsupported at this earlier cutoff; temporal/binned routes and further compositions
remain unqualified. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records the precise comparison scope. Next: broaden the
positional adapter and reconcile remaining GG-04 constructors before GG-05–19.

## GG-04 transformed limit/vector compositions — qualified slice, 13 September 2026

At `6e74ae6` plus working-tree changes, a new focused test reproduced 72 failures
in 144 callback-limit compositions. The shared pipeline now retains arbitrary
limit-vector arity through transformation and defers default rescaler validation.
Nonlinear/reverse transformations reject NULL callback limit results. Native
verification passes all 1,728 primary plot/guide compositions and 144 callback-input
cases. Both actual hosts pass 3,012 exactly equal states and 36 byte-identical,
inspected publications, including 12 update-versus-fresh/immutable-output states.
Focused native/Linux regressions pass. Final native and strict all-target Clippy
checks pass after allocation cleanup; rustdoc, formatting and repository checks pass.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the boundaries.
Next: positional OOB and remaining GG-04 constructor
contracts before GG-05–19.

## GG-04 scalar and empty binned sampling — qualified slice, 13 September 2026

At `6e74ae6` plus working-tree changes, scalar vector sampling respects the
selected bin and empty binned preparations defer sampling until it is needed.
Tests cover 96 reference singleton cases, 504 actual numeric-limit/rescaler builds
and retained guide key values/labels. Explicit binned legends require wire v39.
Both actual hosts pass 1,348 corrected numeric-limit states and 283 states per
pipeline family; all 105 publications match each other and prior inspected bytes.
Strict Clippy, rustdoc, formatting and repository checks pass. All 604 native
unit/integration tests pass at this slice boundary. Two doctests could not compile
because an overlapping build replaced their library artifact. All six doctests pass
on a sequential Linux rerun with the later positional draft; the failed native
command is not recorded as a full pass. The earlier 609-test Linux run predates this
follow-up.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the correction
to older standalone-derived primary proof claims. Next: remaining GG-04
callback/constructor work before GG-05–19.

## GG-04 binned vector pipeline — qualified slice, 13 September 2026

At `6e74ae6` plus working-tree changes, all 19 focused native/Linux tests pass,
including 288 continuous/binned reference build/draw cases and 765 alpha-byte
boundary cases. Both actual hosts pass 283 binned and 283 continuous regression
states, with exact JSON and 36 byte-identical publications per family. The 36
binned files were inspected. Strict Clippy, rustdoc, formatting and repository
checks pass; all 609 Linux core tests pass on the final rebuild. The initial full
attempt used a pre-fix cache build and is not counted as passing. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records an additional open constructor issue: default binned-guide behavior on
empty function limits differs from a generic legend. The older numeric-limit
primary proof is being reconciled against 378 actual source builds. Next: preserve empty binned preparations without invoking sampling, reconcile
guide selection and remaining callback/constructor contracts, then
cumulative GG-04 acceptance and GG-05–19.

## GG-04 continuous vector pipeline — qualified slice, 13 September 2026

At `6e74ae6` plus working-tree changes, 144 pinned continuous OOB/rescaler cases
match raw values, guide outcomes and complete callback sequences. Actual primary
plots preserve all 144 draw outcomes. Both actual hosts pass 283 exactly equal
states and 36 byte-identical, inspected publications. Wire v38 retains pure vector
operation identities. Infinite point sizes remain inspectable and produce empty
glyphs, consistent with 78 additional reference raster cases. Expanded focused tests,
strict Clippy, rustdoc, formatting and repository checks pass. All 607 native core
tests pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records scope and limitations. Next: binned/positional vector pipelines, remaining
constructor reconciliation and cumulative GG-04 acceptance, then GG-05–19.

## GG-04 continuous vector palettes — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, four native/Linux tests cover 675 pinned
continuous palette cases, including actual two-layer primary plots. Both actual
hosts pass 1,240 exactly equal states and 27 byte-identical, inspected publications.
The shared batch mapper retains ordered unique inputs and guide outputs, and NULL
palettes use geometry defaults. All 13 focused palette regressions and all 603 core
tests pass, along with strict Clippy, rustdoc, formatting and repository checks; [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records the boundaries. GG-04 remains in progress. Next: complete constructor
reconciliation and cumulative acceptance, then GG-05–19.

## GG-04 binned vector palette callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, four native/Linux tests cover 675 pinned
binned cases: 540 mapping/build cases, 54 missing-color draws and 81 vector-sensitive
two-layer cases over a shared prepared scale. Actual Python/WASM each pass 1,057
states with 24 updated/fresh comparisons and four rejection cases. All 27 publication
files are byte-identical and inspected. All 599 core tests passed on retry; final
focused regressions, strict Clippy, rustdoc, formatting and repository checks pass.
The existing palette registry now accepts
typed count or normalized-vector input; binned callbacks use the shared midpoint
and threshold owners. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records the scope. GG-04 remains in progress: continuous vector callbacks and complete
constructor reconciliation precede GG-05–19. Full guide composition remains GG-05.

## GG-04 discrete palette callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, 594 pinned discrete cases match count inputs,
named/short/empty/NULL results, restricted/hidden guides and missing-color behavior.
Both actual hosts pass 1,063 exactly equal states with 24 updated/fresh comparisons
and four rejection cases. All 27 publication files are byte-identical and inspected.
Five focused native/Linux tests, all 595 core tests, strict Clippy, rustdoc, formatting
and repository checks pass. Wire v37 retains callback identity and fallback roles;
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the boundaries.
GG-04 remains in progress. Next: continuous/binned vector palettes, constructor
reconciliation, then GG-05–19. Non-color legend composition and the known nonlinear
endpoint-label omission remain GG-05 work.

## GG-04 temporal minor callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, 4,800 pinned cases and 60 width overrides
match native outcomes, typed metadata, major labels and drawable minors in all four
timestamp units. Actual Python/WASM each pass 31,025 matching states, including 25
updated/fresh layouts. All 24 publication files are byte-identical and inspected.
All 590 core tests, focused Linux tests, strict Clippy, rustdoc and repository checks
pass. [GG-04 evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records precision
and JSON-number comparison boundaries. GG-04 remains in progress. Next: custom palette
functions (540 source cases captured), then complete constructor reconciliation and
GG-05–19. The known nonlinear endpoint-label omission remains assigned to GG-05.

## GG-04 discrete minor callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, 300 source cases pass through both band and
point axes; 300 binned constructor calls retain their unsupported-argument outcome.
Both actual hosts pass 1,520 exactly equal states, including 20 updated/fresh layouts.
Twelve publication files are byte-identical and inspected. Focused native/Linux,
strict Clippy, rustdoc and repository checks pass; full core passes 588 tests.
The [GG-04 evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) owns the boundaries.
GG-04 remains in progress. Next: typed temporal minor callbacks; the captured 4,800
main cases and 60 width overrides require retaining automatic major-break names.

## GG-04 joint major/minor callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, guide resolution retains the selected major
vector before projection and supplies it to minor callbacks without reevaluation.
All 1,600 source cases pass native checks; actual Python/WASM each pass 2,898 states
including 40 updated/fresh layouts. Labels and positions match exactly; 14 major and
18 minor inverse-log values differ by one ULP. The original 5,756-state host corpus
also still passes. All 12 publications are byte-identical and inspected. Full core
passes 587 tests; Linux regressions, Clippy, rustdoc and repository checks pass.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the boundaries.
Inspection found omitted nonlinear endpoint labels in publication despite correct
guide snapshots; GG-05 must resolve this rendering defect before guide acceptance.
GG-04 remains in progress. Next: discrete minor callbacks (300 captured source
builds), the unsupported binned constructor argument, temporal minors, remaining
callbacks and constructor reconciliation; GG-05–19 follow. Discrete implementation
and its focused checks are currently in progress.

## GG-04 numeric minor-break callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, v36 registered minor policies share the
break registry and capture one/two-argument behavior. All 3,200 source cases match
native outcomes, major labels, minor values and positions. Actual Python/WASM each
pass 5,756 states with 40 updated/fresh layouts; only 34 inverse-log raw minor values
differ by at most one ULP. All 12 publication files are byte-identical and inspected.
Full core passes 586 tests; focused Linux, strict Clippy, rustdoc and repository checks
pass. The [GG-04 evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) owns exact
commands and limitations. GG-04 remains in progress. Next: qualify joint registered
major/minor selection, including semantic major values on unbounded axes.

## GG-04 temporal positional break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, 4,730 pinned source cases match 18,920
native comparisons in four timestamp units. Actual Python/WASM each pass 28,705
exactly matching states, plus 25 updated/fresh layout checks including expected
errors. All 12 publication files are byte-identical and inspected. Final full core
passes 584 tests; focused Linux tests, strict Clippy, rustdoc and repository checks
pass. See the [GG-04 evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) for
commands, precision boundaries and retained scope. GG-04 remains in progress.
Next: implement the captured numeric minor-break callback contract, then finish
remaining callbacks and constructor reconciliation before closing GG-04.

## GG-04 joint positional binned functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, registered positional limits and bin cuts
compose across initial classification and post-statistic reset. All 1,120 source
builds match 1,680 native routes; actual Python/WASM each pass 2,752 states and 38
supported replacements. NULL/empty limits, missing infinite endpoints and empty
panel extents retain their distinct behavior. Eleven publications are byte-identical;
one SVG differs only in inverse-log numeric metadata. All 12 renders were inspected.
All 581 core tests, Linux regressions, strict Clippy and rustdoc pass.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records callback/error
boundaries and commands. GG-04 remains IN PROGRESS. Next: temporal positional/minor
break functions, palette callbacks and constructor reconciliation, then GG-05–19.
Cumulative gates remain open.

## GG-04 default labels from named breaks — qualified follow-up, 12 September 2026

At `6e74ae6` plus working-tree changes, continuous, binned, numeric positional and
temporal break functions retain returned names for default labels. The focused
330 source builds match 720 native comparisons; actual Python/WASM each pass 1,396
states with exactly equal records, including temporal width/format precedence.
Twelve positional publications are byte-identical and inspected. All 579 core tests
pass, plus the final override test separately; Linux regressions, strict Clippy and
rustdoc pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records
commands and boundaries. GG-04 remains IN PROGRESS. Next: joint positional limit/break
functions, temporal positional/minor-break functions, palettes and constructor
reconciliation, then GG-05–19. Cumulative gates remain open.

## GG-04 binned positional break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, wire v35 positional bins share registered
cuts between classification, post-statistic mapping and guides, preserving names,
blank added-limit labels and callback count semantics. All 5,760 pinned builds match
8,640 native routes. Actual Python/WASM each pass 14,620 states and 40 replacements;
labels/positions match exactly, with at most `8.9e-16` inverse-log tick-value rounding.
Eleven publications are byte-identical; one SVG differs only in numeric tick metadata.
All 12 destination renders were inspected. All 575 core tests, Linux regressions,
strict Clippy and rustdoc pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 remains IN PROGRESS. Next: fix reproduced
named-default-label gaps in earlier break routes, joint limit/break functions,
temporal positional/minor-break functions, palettes and constructor reconciliation;
then GG-05–19. Cumulative gates remain open.

## GG-04 discrete positional break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, automatic/band/point axes use registered break
functions over the trained discrete domain, preserving names, missing levels and
positional empty-selection label behavior. All 400 pinned builds pass in 1,800 native
route comparisons. Actual Python/WASM each pass 3,660 states including 60 layout
replacements, with exactly equal records, unchanged v34 round-trips and 12 byte-equal
publications. Four PNGs and four PDF renders were inspected. All 573 core tests,
focused Linux tests, strict Clippy and rustdoc pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 remains IN PROGRESS. Next: binned/temporal
positional break functions, minor-break functions, palette callbacks and constructor
reconciliation, then GG-05–19. Cumulative gates remain open.

## GG-04 numeric positional break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, primary numeric guides select registered
breaks from expanded panel limits and retain names through positional censoring.
All 1,440 pinned panels match native callback inputs and guide results. Actual
Python/WASM each pass 2,128 states including 40 layout replacements with exactly
equal records, stable v34 round-trips and 12 byte-identical publications. Four PNGs
and four PDF renders were inspected. All 571 core tests, focused Linux tests,
strict Clippy and rustdoc pass; the later registration/version test also passes
native/Linux. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records
commands and callback-count boundaries. GG-04 remains IN PROGRESS. Next: discrete,
binned and temporal positional break functions, minor-break functions, palette
callbacks and constructor reconciliation, then GG-05–19. Cumulative gates remain open.

## GG-04 temporal break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, Date/datetime color and size break functions
retain typed limits/results, names, timezone context and count capability through the
shared guide engine. Native comparisons pass 4,500 pinned builds in four timestamp
units, plus 30 width/format override builds in all four units. Actual Python/WASM each
pass 30,280 function states including 40 replacements, plus 240 override states, with
exactly equal records and stable v33 round-trips. All 569 core tests, focused Linux
tests, strict Clippy and rustdoc pass; the subsequently added override test also
passes on native/Linux. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 remains IN PROGRESS. Next: positional and
minor-break functions, palette callbacks and constructor/argument reconciliation,
then GG-05–19. Cumulative gates remain open.

## GG-04 binned break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, numeric binned break functions execute once
in shared training and supply both mapping cuts and guide candidates. Sorted callback
limits, `n.breaks`/`n` count selection, names, constant populations and NULL behavior
match 2,880 pinned reference builds, including mapped color/size values and callback
counts. Actual Python/WASM each pass 4,686 states with exactly equal records, including
six replacement-versus-batch checks and stable v33 round-trips. The 567-test core suite,
10 final focused native/Linux tests, an additional count-precedence/caching test,
strict core/example Clippy and core rustdoc pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 remains IN PROGRESS. Next: temporal and positional
break functions, minor-break functions, palette callbacks and constructor
reconciliation, followed by GG-05–19. Aggregate gates remain open.

## GG-04 discrete break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, registered discrete break functions use the
trained domain, preserve returned names and first-duplicate selection, and skip label
callbacks when selection is empty. All 600 pinned reference builds match native
callback and guide semantics across color, size, alpha, linewidth, shape and linetype.
Actual Python/WASM each pass 1,230 states with exactly equal records, including 30
replacement-versus-batch checks and stable v33 round-trips. All 566 core tests and
12 focused Linux tests, strict core/example Clippy and core rustdoc pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 remains IN PROGRESS. Next: qualify the binned
break-function corpus, then temporal/positional/minor break functions, palette callbacks
and constructor reconciliation, followed by GG-05–19. Aggregate gates remain open.

## GG-04 numeric continuous break functions — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, registered continuous break functions execute
in core after training and preserve count capability, names, duplicate/missing values,
constant/empty bypass and NULL transformation errors. Portable definitions use v33.
All 2,880 pinned reference builds match native callback and guide semantics; actual
Python/WASM each pass 5,336 states, including eight replacement-versus-batch checks,
with exactly equal host records. All 565 core tests, 14 focused Linux tests, four
export regression tests and strict Clippy/rustdoc checks pass. Format/repository
checks pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records
commands, artifacts and constructor boundaries. GG-04 remains IN PROGRESS. Next:
discrete/binned/temporal/positional break functions, minor-break functions, palette
callbacks and constructor/argument reconciliation, then GG-05–19. The full aggregate
proof and cumulative gates remain open; no new guide painting is claimed.

## GG-04 non-color temporal labels — qualified semantics, 12 September 2026

At `6e74ae6` plus working-tree changes, Date/datetime size/alpha/linewidth retain
temporal guide metadata. Guide evaluation now invokes label callbacks once, removing
an extra temporal color call during scale construction. All 1,200 non-color reference
cases pass in four timestamp units; the original color corpus now also requires exact
callback counts. Python/WASM pass 8,400 non-color and 2,800 color states per host,
with 60 replacement checks and 12 unchanged, previously inspected publications.
All 563 core tests, nine focused Linux tests, strict Clippy/rustdoc and repository/
format checks pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records source contracts and boundaries. GG-04 stays IN PROGRESS. Next: break/minor-break
functions, palette callbacks and constructor/argument reconciliation, then GG-05–19.
Guide painting and cumulative gates remain open.

## GG-04 non-color binned labels — qualified semantics, 12 September 2026

At `6e74ae6` plus working-tree changes, binned non-color guides retain selected
boundaries and independently labelled limits through shared scale preparation.
Guide callbacks precede mark geometry validation, matching reference error ordering.
All 1,800 pinned binned cases match natively (939 successes, 861 expected errors);
Python/WASM each pass 2,748 exactly matching states. Existing numeric/count-palette
proofs pass 370/392 states; the numeric proof now includes two previously deferred
primary guide failures while standalone raw mapping remains valid. All 24 regression
publications match previously inspected artifacts byte for byte. All 562 core tests,
eight focused Linux tests, strict Clippy/rustdoc and repository/format checks pass.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records boundaries and commands.
GG-04 stays IN PROGRESS. Next: temporal non-color metadata/callback-count reconciliation,
break/minor-break functions, palette callbacks and constructor reconciliation, then
GG-05–19. Guide painting and cumulative gates remain open.

## GG-04 non-color continuous labels — qualified semantics, 12 September 2026

At `6e74ae6` plus working-tree changes, size/alpha/linewidth now retain numeric guide
candidates and labels through the existing vector resolver. All 1,440 continuous
reference cases match (1,044 successes, 396 expected errors). Actual Python/WASM
pass 2,496 exactly matching states per host, including 12 replacement-versus-fresh
checks. All 561 core tests including doctests, 16 focused Linux tests and strict
Clippy/rustdoc/repository/format checks pass.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the boundaries.
Guide painting remains GG-05. GG-04 stays IN PROGRESS. Next: binned non-color guide
labels (1,440 source cases generated), remaining temporal compositions,
break/minor-break functions, palette callbacks and constructor reconciliation,
then GG-05–19. No cumulative gate closes.

## GG-04 non-color discrete labels — qualified semantics, 12 September 2026

At `6e74ae6` plus working-tree changes, size/alpha/linewidth/shape/linetype prepare
selected discrete guide keys and labels through the same resolver as color scales.
The semantic DTO retains results for inspection; hidden/disabled guides and empty
breaks suppress callbacks. All 800 pinned actual reference builds match in native
Rust (695 successes, 105 expected errors); actual Python/WASM pass 1,515 exactly
matching states per host, including 20 replacement-versus-fresh checks. All 560
core tests including doctests, 12 focused Linux tests, strict Clippy/rustdoc and
repository/format checks pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and limitations. Guide painting remains GG-05. GG-04 stays IN
PROGRESS. Next: continuous/binned non-color labels (source oracle in progress),
break/minor-break functions, palette callbacks and constructor reconciliation,
then GG-05–19. No cumulative gate closes.

## GG-04 positional policy-label routes — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, registered labels authored inside discrete and
binned positional policies reach the same final-candidate resolver as axis formatters.
Training defers those callbacks; registration/portability validation includes the policy
routes, which require definition version 32. The expanded Rust corpora pass alongside
registry, downgrade, native-only and explicit-override checks (13 scoped macOS tests;
three Linux targets). Python/WASM pass 4,700 discrete and 4,944 binned states per host.
Axis/policy routes match exactly within each host; cross-host binned numeric differences
remain within the previously qualified tolerances. All 30 policy publications are
byte-identical to the earlier inspected axis publications. Strict Clippy/rustdoc and
repository/format checks pass. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and boundaries. GG-04 stays IN PROGRESS. Next: remaining non-color
label behavior, break/minor-break functions, palette callbacks and constructor
reconciliation, then GG-05–19.

## GG-04 binned positional labels — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, binned axes retain censored callback candidates,
train finite reset ranges independently from infinite cut sentinels, recompute shown
limits after reset, and select untrained-axis breaks against the expanded panel.
Equal-break arithmetic now follows R's step-first sequence. All 1,600 pinned cases
pass, including default labels and callbacks (872 successful layouts, 728 expected
rejections); finite ranges and tick positions are independently checked. Full macOS
core tests pass (558 including doctests); three relevant Linux targets pass 9 tests.
Strict Clippy/rustdoc and repository/format checks pass. Rebuilt Python/WASM pass 2,472
states with exact labels/errors and source-backed numeric tolerances; all 18 selected
publications are byte-identical and inspected. Closely spaced explicit labels overlap
under the authored Preserve policy; collision avoidance is not claimed.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records numeric differences
and commands. GG-04 stays IN PROGRESS. Next: reconcile registered positional policy
labels and other non-color label routes, break/minor-break functions, palette callbacks
and remaining constructors, then GG-05–19.

## GG-04 discrete positional labels — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, band/point axis label callbacks use the shared
discrete selection/recycling owner. All 640 pinned cases pass through both families,
including named breaks, duplicates, factor levels, missing-category translation,
empty/all-missing populations and recyclable callback results. The seven related Rust
targets pass 17 tests on macOS and offline Linux; Clippy, strict rustdoc and repository/
format checks pass. Rebuilt Python/WASM match exactly on 2,350 original/edited states
and 12 byte-identical, inspected publications.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records commands and limits.
GG-04 stays IN PROGRESS. Next: binned positional and remaining non-color labels,
break/minor-break functions, palette callbacks and constructor reconciliation,
including direct discrete-position policy label registrations, then GG-05–19.

## GG-04 temporal positional labels — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, primary-axis label functions receive temporal
class/unit/timezone metadata, complete censored candidate vectors and automatic names.
Empty calendar-axis domains infer population endpoints through the shared time engine;
explicit knots and standalone time-scale validation retain their contracts. All 400
pinned cases pass across four integer resolutions (740 successful layouts and 860
expected rejections). Full macOS core tests pass (556 tests including doctests); the
numeric/temporal label targets also pass on offline Linux. Clippy, rustdoc, formatting
and repository checks pass. Rebuilt Python/WASM match exactly on 2,365 states, including
25 replacement/batch checks, and 12 byte-identical, inspected publications.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records commands and limits.
GG-04 stays IN PROGRESS. Next: discrete/binned positional labels, remaining non-color
label contracts, break/minor-break functions, palette callbacks and constructor
reconciliation, then GG-05–19. No full host aggregate or fresh native GPUI capture ran.

## GG-04 numeric positional labels — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, numeric axis callbacks retain the complete
candidate vector and censor out-of-panel breaks to missing values before formatting.
All 480 pinned cases pass (188 successful layouts, 292 expected rejections), including
identity/sqrt/log10/reverse and empty/all-missing populations. The three focused Rust
targets pass 16 tests on macOS and offline Linux. Clippy, strict rustdoc and repository/
format/diff checks pass. Rebuilt Python/WASM agree exactly on 708 states and 15
byte-identical, inspected publications. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records the commands and boundaries. GG-04 stays IN PROGRESS. Next: temporal, discrete
and binned positional labels, other non-color label contracts, break/minor-break
functions, palette callbacks and constructor reconciliation, then GG-05–19.

## GG-04 binned color-label callbacks — qualified semantics, 12 September 2026

At `6e74ae6` plus working-tree changes, the default binned color-guide callback path
matches 640 pinned builds (378 successful, 262 expected rejections), including missing
and repeated cuts, censoring, transforms, and label-result lengths. It retains raw
scale candidates separately from color-step callback inputs. Four focused Rust targets
pass 14 tests on macOS and offline Linux; Clippy, strict rustdoc and repository/format/
diff checks pass. Rebuilt Python/WASM pass 1,042 states; 28 inverse-transform numeric
fields differ within the existing 3e-12 tolerance, with all other fields exact.
Twelve publications match byte-for-byte and were inspected. Their interval-swatch
rendering does not yet display the color-step callback key labels; GG-05 owns that
composition. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records this
boundary. All 640 standalone direct-label cases also pass in Rust (260 successes,
380 expected rejections), including callback invocation on empty vectors.
GG-04 stays IN PROGRESS. Next: remaining positional and non-color label contracts,
break/minor-break functions, palette callbacks and constructor reconciliation, then
GG-05–19. No cumulative gate closes.

## GG-04 temporal color-label callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, temporal label callbacks preserve exact
origin/unit, Date/POSIXct semantics, immutable timezone rules and generated break
names. The 400 pinned Date/UTC/New York reference cases pass in four timestamp units
(1,180 builds, 420 expected rejections). Explicit date formats override callbacks.
The four focused Rust targets pass 13 tests on macOS and offline Linux; Clippy,
strict rustdoc and format/repository/diff checks pass. Rebuilt Python/WASM agree on
2,800 states and 12 byte-identical, inspected publications, including DST fold labels.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records the scoped checks
and remaining boundaries. GG-04 remains IN PROGRESS. Next: binned/non-color and
positional label contracts, break/minor-break functions, palette callbacks and
constructor reconciliation, then GG-05–19. No cumulative gate closes.

## GG-04 color-scale label callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, registered continuous/discrete color-scale
label vectors pass 180 reference cases: 146 primary builds and 34 expected rejections.
They share the existing guide registry, preserve selected break names and source-space
candidates, and use wire 32. The macOS and Linux full regressions each pass 606 tests/
doctests; the final two-test registry target also passes on both platforms.
Clippy, strict rustdoc, format/repository/diff checks pass. Rebuilt Python/WASM match
on 338 states and 15 byte-identical, inspected publications. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands and limits. GG-04 remains IN PROGRESS. Next: remaining temporal/
binned/non-color and positional label contracts (400 temporal oracle builds captured,
not yet implemented/qualified), break/minor-break functions, palette
callbacks and constructor reconciliation, then GG-05–19. No cumulative gate closes.

## GG-04 Calendar callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, Calendar positional callbacks preserve explicit
timezone resources and validate source units. The 216 pinned UTC/New York DST cases
pass across three timestamp units; separate unit/resource rejection controls pass.
macOS and offline Linux each pass 603 Rust tests/doctests, with Clippy, strict rustdoc,
repository/format/diff checks passing. Rebuilt Python/WASM match on 444 states and
12 inspected publication files. Spring/fall labels retain the skipped/repeated hour.
[Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records scope and limitations.
GG-04 stays IN PROGRESS. Next: remaining break/label/palette callback and constructor
contracts in the coverage reconciliation, followed by GG-05–19. No cumulative gate closes.

## GG-04 inferred timestamp callbacks — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, automatic timestamp axes retain exact origin/
unit metadata for positional callbacks, with mixed-source rejection in either layer
order. Ordinary/fractional/facet reference cases pass across three timestamp units.
macOS and offline Linux each pass 601 Rust tests/doctests; Clippy, strict rustdoc and
repository/format/diff checks pass. Actual Python/WASM match on 1,122 states and 30
publication files, all unchanged from inspected explicit-datetime output. The
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records commands and
boundaries. GG-04 stays IN PROGRESS. Next: Calendar callbacks with explicit timezone
resources (216 pinned DST/control builds captured), remaining callback/constructor
contracts, then GG-05–19. No cumulative gate closes.

## GG-04 authored limits through facets — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, 720 pinned point/mean-summary builds qualify
complete/partial/reversed limits on fixed/free facets: 714 successes and six expected
rejections. Retaining transformed guide bounds fixes an all-missing square-root panel
failure. macOS and offline Linux each pass 598 Rust tests/doctests; Clippy, strict
rustdoc, format/repository/diff checks pass. Rebuilt Python/WASM each pass 1,482 exact
states, including 48 retained-semantics replacements; 36 identical SVG/PDF/PNG files
are inspected. Existing No data text and edge-label clipping remain presentation
work. [Scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) records commands,
artifacts and limitations. GG-04 stays IN PROGRESS. Next: timestamp inference and
unresolved callback/constructor contracts, then GG-05–19. No cumulative gate closes.

## GG-04 inventory reconciliation — 12 September 2026

At `6e74ae6` plus working-tree changes, the [coverage reconciliation](evidence/ggplot-scales-coverage.md)
indexes all 152 owned reference exports and 960 formal argument occurrences, with
related family tests and explicit unqualified contracts. Exact inventory identity,
formal-argument keys and evidence-path checks pass. This is documentation evidence;
no runtime or feature gate closes. GG-04 stays IN PROGRESS. Next: authored limits
through statistics/facets, timestamp inference, then the identified callback and
constructor contracts before GG-05–19.

## GG-04 authored numeric limits — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, authored numeric limits pass 1,728 exact-view
cases and 648 new partial-limit reference builds, plus wire-version/profile rejection
checks. The callback regression remains passing. Final macOS validation passes all
597 core/extension tests/doctests; Linux passes the preceding 596-test full suite and
the four final authored-limit tests. Clippy, strict rustdoc, mypy/TypeScript and
format/repository/diff checks pass. Rebuilt Python/WASM each pass 4,428 identical
states and two rejection checks; all 54 identical publications are inspected or retain
previously inspected hashes. [The scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
records commands, artifacts and remaining boundaries.
GG-04 stays IN PROGRESS. Next: reconcile all 152 owned exports/arguments, including
new authored limits with statistics/facets and remaining timestamp controls, before
GG-05–19. No gate closes.

## GG-04 unbounded coordinate views — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, primary Rust passes 1,350 exact-view reference
cases (1,044 successes, 306 rejections). The kernel also passes 735 finite reference
views and 643 infinite positions. All 594 core/extension tests and doctests pass on
macOS and Linux; all-target Clippy, strict rustdoc, formatting/repository/diff checks pass.
Rebuilt Python/WASM each pass 2,426 identical states and 36 identical publications.
Callback regressions retain 1,584 identical states and 48 unchanged publications;
missing-value regressions pass 2,540 states within their existing tolerance. All 39
new/changed SVG/PDF/PNG files have been inspected. Exact viewports and R's default
coordinate expansion retain distinct scope. Commands, artifacts and limits are in
[the scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).
GG-04 remains IN PROGRESS; next are the 378 excluded authored-vector cases, remaining
timestamp controls and argument/inventory reconciliation before GG-05–19. No gate closes.

## Optional Kit crate withdrawal — 11 September 2026

Infrastructure slice: delete `gpui-charts-kit` and keep Kit as an optional gallery
recipe. [ADR-023](adr/023-withdraw-optional-kit-crate.md) records the boundary.
A-KIT is WITHDRAWN. Native/export AP-06 evidence is unchanged; hosts map Kit tokens
into `ThemePatch` and `ChartInput::from_plot`. The composition gallery inlines the
former adapter. Historical evidence snapshots and prior clippy `-p gpui-charts-kit`
logs remain dated records.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:

- `python3 scripts/check_repository.py`: PASS workspace edges, host isolation, single
  GPUI identity and local Markdown links. Members are eight packages; Kit is absent.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy -p chart-gallery --example family_gallery --example host_bootstrap --features kit --locked -- -D warnings`: PASS.
- `cargo check -p gpui-charts --locked` and `cargo check -p chart-gallery --example family_gallery --locked`: PASS.
- `cargo check -p gpui-charts-kit --locked`: fails with no matching package, as intended.
- `mise run check`: repository/deny/fmt/workspace check and Clippy passed; rustdoc
  then failed on pre-existing `[0,1]` intra-doc links in `interpolate/spline.rs` and
  `scales/ggplot_identity.rs` from the open ggplot2 assignment. Not part of this slice.

No Python/WASM, native window, or publication artifacts were re-run. GG-04 and G-AUTH
stay open. Next action: continue the authorized ggplot2 scale work; no Kit library
crate remains to maintain.

## Remaining-work reconciliation — 11 September 2026

Status-only review at `a6caa39` plus current working-tree changes: P2-00 and GG-00–03
are complete; GG-04 is in progress; GG-05–19 are not started as acceptance packages.
Existing baseline implementations within those packages remain reusable. All eight
D3 prerequisite gates have accepted scoped evidence; they are not a new implementation
backlog. G-GGPLOT/G-PARITY and the expanded production gates remain open.

The full package backlog is owned by [GG-04 through GG-19 in the Phase 2 plan](impl_plans/phase-2-parity-implementation-plan.md#gg-04--ggplot2-scale-and-palette-policies).
Current confirmed scale gaps and qualification limits are in the
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).
The pinned inventory still has 643 `OPEN` rows, including delivered capabilities;
it is an inventory baseline, not a current missing-feature count. Completing the
export/argument-to-evidence reconciliation remains required before claiming an
exhaustive API-level residual or closing GG-04/GG-19. This review read the live plan,
ledger, inventory and selected scale/guide source; it ran no feature/runtime tests.
Next: positional pre/post-statistic limit integration and remaining argument audit,
then the package-specific work and acceptance in the existing plan.

## Free-facet positional bins — 12 September 2026

The 1,152-build binned supplement passes 626 reference successes and 526 rejections.
All 591 core/extension tests and doctests, all-target Clippy, strict rustdoc and
format/repository/diff checks pass. Rebuilt Python/WASM each pass 5,504 identical facet
states, including 144 replacements against fresh batches. All 93 byte-identical
SVG/PDF/PNG files are inspected; the previous 57 hashes are unchanged. GG-04 remains
IN PROGRESS. Next: finite viewports over unbounded position populations (1,728 reference
builds captured, implementation pending), remaining timestamp controls and scale
inventory reconciliation before the requested GG-05–19 continuation.

## Faceted positional callbacks — 12 September 2026

Shared/free numeric callbacks pass 1,008 reference builds. Date/datetime/duration
callbacks pass another 1,260 builds at three source resolutions. Final core/extension
validation passes 590 tests/doctests; Clippy, strict rustdoc, formatting, repository
and diff checks pass. Rebuilt Python/WASM each pass 3,678 identical states, including
96 source replacements versus fresh batches. All 57 byte-identical publications are
inspected. The scale evidence records exact scope, logs and remaining presentation
limits. GG-04 remains IN PROGRESS. Next: free-facet positional bins, remaining scale
inventory and subsequent GG-05–19 acceptance work.

## Fractional timestamp statistic projection — 12 September 2026

The 72-build thirds/sevenths supplement now passes at three timestamp resolutions.
Absolute Date/POSIXct projection rounding matches the reference while retaining exact
source metadata. Full core/extension validation passes 587 tests/doctests; all-target
Clippy, strict rustdoc, formatting and repository checks pass. Rebuilt Python/WASM
each pass 1,328 identical states and produce 48 byte-identical, inspected publications;
the previous 36 publication hashes are unchanged. See the linked scale evidence for
commands and artifacts. GG-04 remains IN PROGRESS. Next: shared/free facet callback
training and inventory reconciliation, followed by GG-05–19.

## Duration positional callback qualification — 12 September 2026

The duration supplement passes 252 pinned reference builds. Full core/extension
validation passes 586 tests/doctests; Clippy and strict rustdoc pass. Fresh Python/WASM
each pass 1,184 matching states and produce 36 byte-identical, inspected publications.
The scale evidence records the preserved NULL-input and infinite hms guide rules.
GG-04 remains IN PROGRESS. Next: fractional timestamp statistic precision probes,
shared/free faceted callbacks, then inventory reconciliation and GG-05–19. Captured
72-case precision and 1,008-case facet fixtures are not acceptance evidence yet.

## Date/datetime positional callback qualification — 12 September 2026

Date/UTC callbacks now preserve exact origin/unit metadata and fractional limits across
source/statistic/position stages and reference axis projection. The 504 pinned builds
pass at three source resolutions: 726 successes and 786 expected rejections. Full
core/extension validation passes 584 tests/doctests; the expanded primary matrix and
additional precision guard pass subsequently. Clippy, rustdoc and repository checks
pass. Fresh Python/WASM each pass 770 matching states and produce 24 byte-identical,
inspected SVG/PDF/PNG publications. See the [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
for exact commands, tolerances and qualification boundaries.
GG-04 remains IN PROGRESS. Next: duration and faceted positional callbacks, then
remaining inventory reconciliation before GG-05–19; cumulative gates remain open.

## Duration missing-value qualification — 12 September 2026

The 192-case duration supplement passes all 144 reference successes and 48 errors.
It preserves ggplot2 4.0.3's ignored `na.value` argument on time scales and its empty
automatic guide rejections. Full core/extension validation passes **583 tests and
doctests**; Clippy passes. Fresh Python/WASM each pass **2,540 states**, including
source replacement and immutable captures. All thirty publication samples are
inspected; 28 are byte-identical, and two logarithmic SVGs retain only the previously
recorded numeric differences. See the [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).
GG-04 remains IN PROGRESS. Next: Date/datetime positional callbacks (504 pinned R
builds captured, implementation pending), then faceted callbacks and remaining
inventory reconciliation before GG-05–19.

## Positional missing-value qualification — 12 September 2026

Numeric positional `missing_value` now applies replacement in transformed units after
OOB handling, before statistics and at the generated-position map. Version 30 retains
the control in Rust/Python/WASM. Reference cases also fixed empty-summary marks,
empty automatic guides, all-missing automatic ranges, and negative square-root
replacement layouts. Callback training now consumes transformed observations directly;
a residual mean correction avoids a one-ULP error that changed singleton censoring.

All **1,152** pinned R builds (1,032 successes and 120 rejections), two nullable-endpoint
cases, and structural guards pass. Full core/extension validation passes **582 tests
and doctests**; Clippy, rustdoc, formatting, repository checks and strict host consumers
pass. Fresh Python/WASM each pass **2,196 states**, including twelve source replacements
versus fresh batches and immutable captures. Twenty-two of 24 publications are byte
identical; two SVGs differ only in numeric attributes, at most `2.56e-13`, within the
explicit tolerance. All 24 SVG/PDF/PNG samples were inspected at full canvas size.
See the [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md) for commands,
artifacts and remaining scope. GG-04 remains IN PROGRESS. Next: duration missing-value
qualification, temporal/faceted positional callbacks and the remaining argument
inventory, then GG-05–19; cumulative gates remain open.

## Positional callback log/OOB qualification — 12 September 2026

The 756-case logarithmic/explicit OOB supplement passes primary/core tests without
production changes. Together with the original 252 cases, both rebuilt hosts pass
1,584 matching states and 48 identical publication files. All six new logarithmic
SVG/PDF/PNG files were inspected; prior samples retain their hashes. The original
oracle remains unchanged. See the [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md)
for commands, comparisons and limitations. GG-04 is IN PROGRESS; next: positional
missing-value controls, then temporal/faceted callbacks and the remaining inventory.
GG-05–19 and cumulative gates remain open.

## Binned positional callback qualification — 12 September 2026

GG2-03/FIX-GG04 at `6e74ae6` plus working-tree changes: binned positional callbacks
retain initial cuts across scale reset and map statistic indices without retraining
those cuts. All 126 binned oracle cases match (74 renderable, 49 build failures,
three projection failures); all 48 continuous/binned summary/shared-layer cases pass.
571 unit/integration tests pass; an isolated rerun passes six doctests after an
artifact race in the first run. Final Clippy, strict host consumers and repository
checks pass. Rebuilt Python/WASM pass 432 matching states and 42 identical publication
files; all new SVG/PDF/PNG samples were inspected. Details and commands are in the
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).

GG-04 stays IN PROGRESS. Faceted/timestamp callbacks, unbounded viewport handling,
remaining scale arguments and cumulative acceptance are open. Next: logarithmic/OOB
callback qualification, remaining GG-04 scope, then GG-05–19.

## Continuous positional callback qualification — 12 September 2026

GG2-03/FIX-GG04 at `6e74ae6` plus working-tree changes: numeric positional limit
functions now run before statistics and after positions across shared layers.
Wire 29 retains the operation; scene wire 16 preserves infinite guide identities
without introducing nonfinite destination geometry. All 126 continuous R cases and
24 summary/shared-layer cases pass. The macOS core/external suite passes 576
tests/doctests, rebuilt Python/WASM pass 222 matching states each, and 24 identical
SVG/PDF/PNG files were inspected. Strict host consumers pass. Commands, artifacts
and limitations are in the [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).

GG-04 stays IN PROGRESS. Binned/faceted/timestamp positional callbacks, finite
viewports over unbounded limits, remaining argument reconciliation and cumulative
acceptance remain open. No native window/transition or full cumulative binding-runner
pass is claimed. Next: binned callback reset and retained-cut training, followed by
remaining GG-04 scope and GG-05–19.

## Active ggplot2 assignment — 10 September 2026

Active follow-on: binned count/numeric palettes and reference physical linewidths
now pass 860 states per rebuilt Python/WASM host and 36 byte-identical, inspected
publication files. Wire 27 retains count palettes; linewidth uses the profile's
physical conversion and zero hairline at shared scene projection. The final Linux core suite
passes 547 tests, with Clippy and repository checks also passing. R guide-build
failures are distinguished from successful scale mapping. Details and limits:
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md). Next: remaining
GG-04 arguments/registered callbacks. GG-04, GG-05
presentation and cumulative gates remain open; no fresh macOS/native pass is claimed.

Registered discrete limit functions now pass all 90 pinned R cases, four external
contract tests and 198 matching states per rebuilt Python/WASM host. Wire 28 retains
the installed operation selection. Twelve publication files match byte for byte;
PNG and independently rasterized SVG/PDF inspection is complete. Core/external
Clippy passes. The full core/external suite passed 569 tests before the final
hidden-identity callback correction; focused tests and rebuilt hosts pass afterward.
Numeric limit callbacks now also pass all 378 pinned R cases and 677 exactly matching
states per rebuilt host, with eighteen identical, inspected publications. The shared
API is `CustomScaleLimits` / `with_limits_function`; wire 28 uses `limits_function`.
The full Linux core/external suite passes 571 tests/doctests; all-target Clippy passes.
The prior positional proof scripts are restored under `ggplot_positional_limits` after
a filename collision: both rebuilt hosts pass 3,582 matching states and twelve
unchanged publications. Discrete callbacks retain 198 matching host states and twelve
unchanged publications. Other rescalers, temporal/positional callbacks and remaining
registered arguments are next. The 547-test/860-state results above predate callback changes. Details and artifact
paths are in the scale evidence. GG-04 and cumulative gates remain open.

Follow-on rescaler work: 126 additional pinned R cases now pass focused external
checks (504 numeric cases total). Maximum rescaling preserves valid maps with
missing/empty limit vectors; binned single-boundary selection precedes rescaling.
The primary grammar now treats ggplot2 numeric NaN output as missing; a focused
test preserves legacy rejection. Both rebuilt hosts pass 892 exactly matching states
and eighteen publications identical to the inspected files. Final Clippy/rustdoc
pass; the stable-source full-suite passes 573 tests/doctests with no failures or
ignored tests. This qualification precedes the next temporal callback changes. Temporal callbacks now pass the 84-case Date/datetime oracle over 336 Rust
configurations and 1,408 WASM states including the existing numeric lane. The shared
callback context retains exact origin/unit and Date semantics; empty domains reject
before invocation. Both actual hosts match all 1,408 states and 36 inspected publication files. The
full Linux core/external suite passes 574 tests/doctests without failures or ignores;
Clippy, rustdoc, formatting and repository checks pass. Positional callbacks are next.
Their 252-panel oracle now verifies 238 first callback inputs and fourteen errors
before invocation through the shared evaluator. Internal callback evaluation and
filtered positional collection were extracted for reuse; seven positional-bin and
nine external callback tests pass, with five numeric tests rerun after the input
comparison was added. Core/external Clippy and formatting pass. The 574-test/full-host
qualification predates this internal extraction. Post-statistic callback retraining,
primary positional output and other registered arguments remain open; GG-05–19
remain outstanding. See the scale evidence for commands and the next integration step.

Latest extension: materialized positional palettes pass 144 primary R panels plus
116 nested secondary outcomes, 520 configurations per actual host, twelve identical
inspected publication files and four Rust replacement states. Full core passes 537
tests/doctests; all-target Clippy and strict Python/TypeScript consumers pass. Wire 23
retains authored numeric palettes. Remaining argument/callback reconciliation and
cumulative destination/platform acceptance keep GG-04 open.

Latest GG-04 slice (11 September): discrete secondary axes pass 176 pinned R panels
through 352 band/point JSON configurations in Rust and each actual Python/WASM host.
Both hosts produce identical records and nine identical inspected SVG/PDF/PNG exports.
Full core passes 533 tests/doctests; two focused tests additionally cover reversed ranges
and both orientations. Clippy passes. Starting revision remains `a6caa39` plus retained
working-tree changes. GG-04 is IN PROGRESS; GG-05–19 and cumulative gates remain open.
Next: reconcile custom positional palettes and remaining GG-04 arguments with the
pinned inventory. Details: [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).

The owner authorized continuing through GG-03–19. GG-03 completed on
accepted GG-02, SP-04 and WP-S05 prerequisites, starting at `a6caa39` with the
previously qualified axis/interpolation/hierarchy changes retained in the working tree.
GG-03 is COMPLETE: [acceptance evidence](evidence/phase-2-aesthetics-2026-09-10.md)
records twelve focused regressions, 104 reference glyphs, actual Python/WASM ownership
and eight update states per host, matching Rust/Python/WASM publication, inspected
PDF/native output, 487 macOS and 468 Linux tests/doctests, and a passing repository
check. The resolved style grows from 24 to 56 bytes for independent channels; this
explicit cost is recorded in ADR 014. G-GGPLOT and G-PARITY remain open. The latest guide slices pass 60
discrete selection/label records, 30 manual break-to-palette records and 160 continuous
guide cases, plus eighteen zero-row continuous/identity cases including JSON and
retraining, and 192 all-nonfinite candidate/label/censor cases. Primary guide edits
preserve marks. Binned candidates and labels pass 288 R records, primary metadata
and independent boundary-color checks, plus 192 binned population cases and 135 primary time-width string panels. Explicit R date/time formatting additionally passes 220 reference labels and primary
axis/JSON/resource checks, plus 40 primary duration-format panels. Fresh Python/WASM
proofs each pass 246 publication cases and 14 explicit tab-glyph rejections, with
identical records/PNGs and inspected SVG/PDF/PNG samples. Multiline labels pass all
four axis sides and supplied subsecond timezone-resource checks. All 487 core
tests/doctests and all-target core Clippy pass on the final code. Full guide presentation,
remaining scale arguments and actual host/destination acceptance remain open. The latest authored-limit extension passes 2,688 focused R cases, 120 related tests, all 495 core tests/doctests and all-target core Clippy; the hidden-guide/fractional extension passes all 499 core tests/doctests. Fresh Python/WASM each pass 2,304 matching records with three identical PNGs and inspected SVG/PDF/PNG samples. The final structural-validation fix passes 16 focused tests, all 500 core tests/doctests and all-target Clippy. Degenerate-domain handling additionally passes 256 R guide/mapping records and 192 primary chart cases, including constant-zero logarithmic bins, collapsed-cut rejection and correct reference missing-color defaults; all 494 core tests/doctests and Clippy pass. Transformed missing-limit population handling additionally passes 320 scale records and 24 actual R chart cases, including mapping rejections and fully specified-limit controls; all 491 core tests/doctests and Clippy pass. Binned count and transformed missing-limit boundaries additionally pass 228 R records and eight focused tests, including primary JSON/retraining checks; all 489 core tests/doctests and Clippy pass. Positional bins now pass 2,400 direct R records, all 960 primary panels (including infinite limits), 96 statistic/filter cases and live replacement versus fresh batches. Fresh Python/WASM each pass 960 matching records with four identical PNGs and inspected SVG/PDF renders; version 18 retains the shared positional state. Full core validation passes 507 tests/doctests without exclusions, final focused tests pass, and all-target Clippy passes. R's infinite-limit cases produce undefined coordinates and omitted points; an unbounded range adapter preserves that behavior with finite destinations and explicitly unavailable inversion. Free-facet populations, finite viewports over unbounded bin geometry and remaining scale arguments stay open. Minor selection now covers numeric, temporal and discrete policies, with 761 matching fresh Python/WASM records within the reference tolerance, 514 passing core tests/doctests, all-target Clippy and strict host declarations. Four precision cases preserve exact timestamp origins and promote fractional units; Date major flooring is isolated to the ggplot2 profile. Version 19 retains nullable raw minor values and finite positions. Repository, formatting and diff checks pass. Discrete continuous-limit control now passes 756 direct R records, 3,528 primary configurations and 24 replacement states. Fresh Python/WASM each pass 3,552 exactly matching records and four byte-identical PNGs, with SVG/PDF/PNG inspection completed. Version 20 retains the authored vector and distinguishes all-excluded observations from zero rows. All 518 core tests/doctests, all-target Clippy, strict host declarations and repository/format/diff checks pass. Automatic character-order training now follows pinned C collation only in the ggplot2 profile, preserving explicit domains. Six new R records pass 30 actual panels, bringing both host proofs to 3,582 exactly matching records. All four focused Rust tests and all-target Clippy pass; twelve publication files match each other and the previously inspected artifacts byte-for-byte. The latest change uses focused tests; the full 518-test run above predates this ordering correction. Nullable authored/factor domains preserve missing-level order and distinguish zero rows from a trained empty palette. The new 512-record direct oracle and 256 primary hue cases pass; 72 R position panels verify that missing paint preserves numeric/category training. Both hosts pass 346 exactly matching records and fifteen byte-identical publication files, all SVG/PDF/PNG samples inspected. All 523 core tests/doctests, all-target Clippy and repository/format/diff checks pass. Explicit retained empty-population state uses version 21; ordinary policies remain version 17. Identity primary missing colours now pass 512 actual R point-count configurations and twelve focused Rust tests. Both hosts pass 874 matching records, including 32 replacement states, with eighteen identical inspected publication files. The full 523-test run above predates this identity correction; all-target Clippy passes from a fresh target directory. Untrained fallback colour lookups now pass 64 direct R cases and 32 empty primary charts, preserving lazy short-manual-palette errors and empty guides. All 525 core tests/doctests and all-target Clippy pass from the fresh qualification target. Rebuilt hosts retain 874 identical records and eighteen unchanged inspected publications. Positional null identity now retains typed catalogs through domain union, projection, guides and inspection. The 576-case R matrix passes direct policy checks and 1,152 primary band/point configurations. Wire 22 retains factor/drop/translation/nullable-limit policies. Both actual hosts pass 1,152 exactly matching records and six identical inspected SVG/PDF/PNG files. Six focused Rust tests additionally cover 16 replacement states, immutable captures, reversed axes and explicit D3 null-omission controls. All 531 core tests/doctests, all-target Clippy, strict host declarations and repository/format/diff checks pass. Nullable navigation/continuous-limit combinations, broader geometry/facet and host replacement acceptance remain open. Coincident labels and empty-panel presentation remain GG-05 gaps; minor painting/animation, arbitrary callbacks and remaining scale arguments stay open. GG-04 is IN PROGRESS: [the current scale slice](evidence/phase-2-ggplot-scales-2026-09-10.md) passes 766 palette, Nullable continuous-limit/expansion combinations now add 1,152 pinned R panels, bringing direct coverage to 1,728 records and primary coverage to 3,456 configurations. Fresh Python/WASM each pass 3,456 exact-matching records and nine identical publication files, including inspected expanded-axis SVG/PDF/PNG. Full core remains 531 passing tests/doctests; all-target Clippy and repository checks pass. Evidence and next open minor/navigation work are recorded in the linked GG-04 scale ledger. Nullable numeric minor selection adds 192 reference panels: 1,920 direct records, 3,840 matching primary Python/WASM configurations, nine unchanged inspected publication files and 532 passing core tests/doctests. Clippy and repository checks pass. Next: reconcile the remaining GG-04 arguments, including discrete secondary axes, against the pinned inventory.
93 scale and 284 break records, plus focused point/after-scale and trained-bin budget regressions.
Automatic and authored linear/logarithmic axes now use reference expansion and
default breaks/labels: 120 expansion records, 499 label vectors and 32 complete panel
records pass, including collapsed linear/log viewports with midpoint projection.
Discrete range policies pass 36 R cases and 54 axis configurations against nine R
panels. Fixed elapsed-second time breaks and supplied-zone DST labels pass 30 R
panel cases; structured calendar widths pass 16 additional R panels with exact
timestamps/labels, bounded generation and v17 serialization. Datetime expansion passes
16 R cases through three primary forms, three fine-resolution cases and 16 paired
numeric contraction cases; fractional views retain exact source timestamps. Automatic
datetime selection and labels pass 534 primary R panels, including default restoration,
format overrides and supplied DST resources. Date axes pass 81 R records in two
source units plus 12 width panels through the shared timestamp engine. Timestamp
population limits pass eight R summary cases and focused exact-integer/stage regressions,
448 full core tests/doctests and all-target core Clippy. Numeric secondary guides pass 36 affine and 16 custom-transform R
cases, with independent breaks/labels and reference sampled/rounded placement.
Secondary Date/datetime guides pass 54 further R panels, including supplied DST
resources, plus edit, exact-unit and v17 rejection checks. Full core validation passes
450 tests/doctests before the final fixture expansion; the expanded focused run passes.
Reference point size units, zero/negative observation retention and outline conversion
pass nine R configurations / 63 glyph inputs, 452 core tests/doctests and all-target
core Clippy. Explicit `.radius()` retains its literal-radius contract. Portable point hairlines use the reference
PDF device's physical minimum; other R device hairline equivalence remains open.
Elapsed duration scales pass 45 break configurations, 66 primary and nine secondary
panels plus seven additional label vectors, reusing linear mapping and the extended
break kernel. The isolated oracle now pins hms 1.1.4. Named duration widths, four UTC
label patterns, inherited secondary labels, zero-range selection and v17 round-trips
pass. Full core validation passes 455 tests/doctests before the final duration extensions;
31 related tests and final all-target Clippy pass after them. Actual host acceptance
and complete format/argument compatibility remain open.
R color parsing now covers 657 named colors and 42 parsing records, with an explicit
portable rejection for overflowing palette indexes. Reference manual text colors pass
eight ggplot2 selections, 24 related tests and all-target Clippy. Numeric/discrete
identity mappings now pass 122 R records and primary colour/linewidth integration,
with raw output independent of guide limits and hidden default colour guides.
Reference alpha lowering and after-scale arithmetic pass 192 additional point-grob
paint comparisons, including infinite values and missing-value power identities.
All-target core Clippy and 464 full core tests/doctests pass. Numeric colour
indexes, complete identity guide presentation and actual host/destination acceptance
remain open.
Geometry-extent training, remaining time policies and ggplot guide-presentation defaults
remain open (zero-expansion projection tests explicitly preserve endpoint labels).
Remaining policies and full package acceptance stay open; the other authorized
packages follow in prerequisite order.

## Completed axis, interpolation and hierarchy assignment — 10 September 2026

The owner authorized final interpolation certification followed by axis and hierarchy.
WP-AX03–06 are COMPLETE and G-AXIS passes for the declared D3 typed/profile and supported
platform boundary. [Integrated axis certification](evidence/phase-2-axis-certification-2026-09-09.md)
records all FIX-19 verdicts, 372 static cases / 376 states, 125 timed samples,
actual Rust/Python/WASM and inspected native/SVG/PDF/PNG, exact retained capture,
449 macOS/448 Linux tests and a full repository-check pass.

WP-IP07 and G-INTERPOLATE now pass: the existing 27-export/reference/configuration
catalog and measured integration evidence are joined by the qualified axis consumer and
fresh 18-artifact host replay. [Final interpolation certification](evidence/phase-2-interpolation-integration-2026-09-09.md)
records that closure. H01–08 are COMPLETE and G-HIERARCHY now passes for the declared
finite typed surface. [Hierarchy final acceptance](evidence/phase-2-hierarchy-integration-2026-09-10.md)
records 898 independent oracle cases plus 27 control sequences per Rust/Python/WASM
surface, all 65 method/control/FIX verdicts, nine inspected native projections, 81 matching
three-host artifacts, update/replay/ownership proofs and measured resource handoff.
Final qualification passes 474 macOS and 455 Linux tests/doctests, strict host typing and
full repository checks. Starting revision `a6caa39`; changes remain in the working tree.
Earlier stop boundaries and remaining-work counts below are historical.

| Active remaining lane | Packages | Count |
| --- | --- | ---: |
| Axis, interpolation, hierarchy | None — assigned work complete | 0 |
| ggplot2 (outside this assignment) | GG-03–19 | 17 |
| Phase 2 total | 17 remaining of 71; 54 accepted | 17 |

All eight D3 lane gates pass. The hierarchy high-fanout hit query measured 18.49 ms p95
at 1,000 nodes; WP-22 retains optimization and interactive-latency/allocation/presentation
qualification. This component parity acceptance does not pass PERF-01–05.

Global WP-21/22/23, G-PARITY/G4 and other release gates remain open.

## Interpolation handoff — 9 September 2026

WP-IP06 is COMPLETE. Registered factories now feed shared interpolation, scale,
color/legend/theme and Rust/Python/WASM consumers, with inspected native/SVG/PDF/PNG
sampled frames and retained update/capture proofs. The
[interpolation report](evidence/phase-2-interpolation-integration-2026-09-09.md) records
commands, source identity, acceptance evidence and validation limits.

WP-IP07 is PARTIAL: its 27-export/configuration verdict catalog, fresh reference replay,
host/integration evidence and allocation/timing profile are complete, but WP-AX06 is a
required unfinished prerequisite. G-INTERPOLATE remains NOT PASSED. WP-AX03–06 were
not started by this assignment. The fresh repository check passed, including the
previously stalled standalone WASM metadata check; that requalification item is resolved.

| Lane | Remaining packages | Count | Next prerequisite action |
| --- | --- | ---: | --- |
| Axis | WP-AX03–06 | 4 | AX03 is ready; outside this interpolation assignment. |
| Hierarchy | WP-H01–08 | 8 | H01 is ready. |
| Interpolation | WP-IP07 final certification | 1 | Complete AX06, then qualify the axis transition consumer and final cross-lane gate. |
| ggplot2 | GG-03–19 | 17 | Retain the listed prerequisite order. |
| **Phase 2 total** | **30 remaining of 71; 41 accepted** | **30** | **No further package started in this task.** |

Expanded WP-21/22/23, AP-09/G-AUTH, G-GGPLOT, G-PARITY and G4 remain separate open
qualification work. The following axis handoff is retained as historical context.

## Phase 2 axis handoff — 9 September 2026

The owner narrowed the active assignment to finish WP-AX01 and WP-AX02, update the
remaining-work list and stop before WP-AX03. Their integrated implementation and focused
acceptance are complete; the [axis report](evidence/phase-2-axis-ticks-2026-09-09.md) records final repository validation and its limits. The D3 profile entry, checked
providers and independent tick selection/formatting are covered. Full D3 axis geometry,
components, transitions and certification remain open.

The following is the remaining Phase 2 todo list after these two package acceptances:

| Lane | Remaining packages | Count | Next prerequisite action |
| --- | --- | --- | --- |
| Axis | WP-AX03–06 | 4 | AX03 consumes accepted AX02; outside this assignment. |
| Hierarchy | WP-H01–08 | 8 | H01 reference/contract harness is ready. |
| Interpolation | WP-IP06–07 | 2 | IP06 integration; IP07 also requires AX06. |
| ggplot2 | GG-03–19 | 17 | GG-03 and GG-06 have their listed prerequisites; retain each package's dependency order. |
| **Phase 2 total** | **31 remaining of 71; 40 accepted after this handoff** | **31** | **Stop here; do not start another package in this task.** |

The remaining standalone WASM core metadata check stalled after the other repository steps passed and is explicitly incomplete; fresh actual WASM runtime proofs pass. Retain this check as a requalification item.

Expanded WP-21/22/23 requalification remains additional to those 31 packages. AP-09 /
G-AUTH's existing native performance gate is also separate. Five of eight D3 lane gates
pass; G-AXIS, G-HIERARCHY and G-INTERPOLATE remain open, alongside G-GGPLOT, G-PARITY
and G4. Package details and evidence below remain authoritative; this list does not
relax prerequisites or claim final production acceptance.

Earlier accepted work remains recorded below: WP-S01–08/G-SHAPE, WP-P01–04/G-PATH,
GG-00–02, the color/chromatic/scale lanes and WP-IP01–05.
The [oracle entry report](evidence/phase-2-oracles-2026-09-08.md) records 643 ggplot2
exports, 57 locked R sources, 32 reproducible records and 96 inspected artifacts.
The [path acceptance report](evidence/phase-2-paths-2026-09-08.md) records 86 reference
sequences, eight focused tests, actual three-host proofs, inspected native/publication
output, 261 macOS tests and 255 Linux core/export tests. Final repository, primary API
and aggregate binding proofs pass. These results do not close other D3, ggplot2 or
performance gates. GG-02 and color foundation results are recorded below.
SP-04 is COMPLETE for shared typed interpolation/distribution mapping, numeric aesthetics and guide metadata.
SP-05 is COMPLETE for exact numeric ticks/labels, locale specifiers, nice and publication integration.
SP-06 is COMPLETE for shared calendars/time axes and actual same-revision host proofs.
SP-07 is COMPLETE and G-SCALE passes for the declared typed FIX-20 scope.
[Integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md) records 661
cases / 19,562 operations per surface, complete public calendar replay, actual keyed
updates and interactions, five inspected native/publication figures, exact three-host
RGBA output, 343 macOS tests, 40 focused Linux tests and passing repository/type checks.
Piecewise/category/quantile timings and a stable WASM memory plateau feed WP-22;
no sustained-load or aggregate platform release gate is closed.
CLR-05 is COMPLETE and G-COLOR passes for COL-01–06 / FIX-C01.
[Integrated color evidence](evidence/phase-2-color-acceptance-2026-09-09.md) records
all 351 reference cases, actual host methods and paint inputs, three independently
authored perceptual/alpha/grayscale figures, mapped palette cache/target invariants,
11 passing macOS/Linux tests, inspected native/SVG/PDF/PNG and measured component/update
costs. CP-01 is COMPLETE: the [entry report](evidence/phase-2-chromatic-entry-2026-09-09.md) records the complete 76-export / 218-array / 38-interpolator oracle with 160,666 samples, byte-identical regeneration and the Rust reader. CP-02/03 are COMPLETE for core scope: [foundation evidence](evidence/phase-2-chromatic-foundations-2026-09-09.md) records exact tables, all 160,666 sampled rows, 27,434 exact V8 trigonometric anchors, 19 macOS / 21 Linux tests and passing repository checks. Fixed fused trigonometry resolved reproduced byte-rounding differences. Integrated certification is recorded immediately below.
CP-04/05 are COMPLETE and G-CHROMATIC passes for CHR-01–06 / FIX-21. The final 304-case exceptional-normalization matrix passes on Rust and actual macOS/Linux Python and Node WASM.
[Integrated chromatic evidence](evidence/phase-2-chromatic-integration-2026-09-09.md)
records 218 exact arrays and 160,666 ramp rows in actual Rust/Python/WASM, 90 composed
scale cases / 1,268 exact scene colors, v6 migration and checked guide identity,
24 update steps per host, 352 macOS suite tests plus focused macOS/Linux supplements,
five inspected native/SVG/PDF/PNG figures, cross-platform exact publication, a stable
18,000-owner WASM plateau and measured catalog/evaluation/update costs. A reproduced
non-finite ordinal-key training shift was fixed without changing legacy typed keys.
Final fresh primary runtime/type proofs, rebuilt native inspection and repository checks pass. No global release or
sustained-load gate is inferred from these measurements.
WP-S01 is COMPLETE for entry/foundation scope. Its [acceptance report](evidence/phase-2-shape-foundation-2026-09-09.md)
records all 63 exports/methods/defaults, 333 exact numeric contexts, 42 layout records,
actual macOS/Linux Python and Node WASM, identical 300/600 DPI scenes/PNG output,
and inspected native/SVG/PDF/PNG sector, hole and external-sink geometry. This consumes
G-PATH. WP-S02 is COMPLETE: [829 Cartesian cases](evidence/phase-2-shape-cartesian-2026-09-09.md),
all 20 curves, general areas, actual hosts and inspected output; the full macOS suite
passes 369 tests. WP-S03 is IN PROGRESS: [arc/pie implementation and qualification](evidence/phase-2-shape-arc-2026-09-09.md)
passes Linux core/Python and final WASM oracle, interaction, updates and publication;
macOS build/launch qualification is pending. While those loader waits persist, the independent
Earlier shape-stage progress (superseded by WP-S08 acceptance below): WP-S05 symbol implementation is IN PROGRESS on its accepted WP-S01 prerequisite: [156 fixtures per host, actual area/type guides, 104 updates per host and exact inspected publication](evidence/phase-2-shape-symbol-2026-09-09.md) pass. WP-S06 is IN PROGRESS: [complete stack kernels and tidy bars/areas](evidence/phase-2-shape-stack-2026-09-09.md) pass 435 numerical cases per host, 1,620 core tidy comparisons, 480 updates per host, exact inspected three-host publication and 385 Linux tests/doctests. Native/macOS qualification continues for these families before the remaining ordered shape gates. G-SHAPE remains open. GG-03 retains its WP-S05 prerequisite. All authorized Phase 2 work continues.


## CLR-01 reference and CLR-02/03 color math — 8 September 2026

[ADR-016](adr/016-color-values-and-paint-boundary.md) defines floating values, explicit
exceptional tags, parser/formatter behavior and the authored-value/byte-paint boundary.
The d3-color 3.1.0 oracle has 351 cases: all 148 names, eight constructors, every
inherited method, RGB/HSL clamps and exceptional channels. Separate regeneration
matches the corpus and manifest byte for byte. CLR-01 is COMPLETE for entry scope;
G-COLOR remains open.

CLR-02/03 are COMPLETE. The new core color module implements bounded CSS parsing,
RGB/HSL/D50 Lab/HCL/LCh/Cubehelix, copy/brightness, conversions, predicates, formatters
and explicit descriptor round trips. All 351 cases pass in actual Rust/Python/WASM,
including exact strings/bytes. Shared atan2, power and ECMAScript formatter corrections
fixed reproduced platform differences without weakening fixtures. Repository checks,
279 macOS tests, six focused Linux tests and complete primary runtime/type proofs pass.
[Retained evidence](evidence/phase-2-color-foundations-2026-09-08.md) records the source,
commands and limits. CLR-04 is COMPLETE: every paint input retains floating values, strict wire migrations pass, and Rust/Python/WASM/native/publication use the shared lowering boundary.
[Paint evidence](evidence/phase-2-paint-2026-09-09.md) records four independently authored figures, exact cross-host RGBA output, inspected SVG/PDF/native, retained live snapshots, 298 macOS tests, 14 focused Linux tests and passing repository/type checks. CLR-05 and G-COLOR are now qualified in the [integrated color report](evidence/phase-2-color-acceptance-2026-09-09.md).

## WP-IP01 reference and interpolation foundations — 8 September 2026

[ADR-017](adr/017-shared-interpolation-values.md) freezes target dispatch, exceptional
values, owned lifetimes, operation-specific bounds and explicit typed adaptations.
All 27 pinned exports, three gamma factories, rho and duration are inventoried. The
Node oracle generates 370 scalar/value/color/zoom cases; real Chromium 151.0.7922.34
generates 36 CSS/SVG transform pairs with source/binary hashes. Both regenerate byte
for byte. WP-IP01 is COMPLETE for contract/reference scope; G-INTERPOLATE stays open.

The new shared scalar/composition module passes 56 applicable reference cases plus
independent rounding, spline seam, factory-count and owned-sample tests. The floating
color interpolation module passes 106 applicable cases with exact CSS output, plus
independent hue/gamma/alpha anchors. Structured values pass 191 cases including the complete
source/target kind matrix and explicit adaptations; 17 zoom and 36 browser transform cases
pass. Strict standalone descriptors, every public factory/configuration, actual Python/WASM
samples, six invalid type cases per host, copies and disposal pass. Linux passes all 11
interpolation kernel tests plus six color/path regressions; two new descriptor tests pass
on macOS. Clippy, repository checks and combined primary runtime/type proofs pass.
[Retained evidence](evidence/phase-2-interpolation-foundations-2026-09-09.md) identifies this source snapshot and validation limits.
Chart paint/scale/axis consumers, inspected integrated publication and final parity gates
remain outstanding; no G-INTERPOLATE closure is claimed.

## GG-02 stage implementation and acceptance — 8 September 2026

Against `fab2505` plus the uncommitted continuation, canonical profile provenance,
scale/stat ordering, inferred discrete groups, horizontal recipes and typed bounded
expression graphs are implemented in the shared core. Ten stage tests and four
expression tests pass. The independent R 4.6.1 / ggplot2 4.0.3 corpus has 22 cases
and repeats byte for byte, including log/coordinate order, scale limits/zoom,
back-transform-before-after-stat expressions, missing/explicit groups and orientation.
Shared named statistics now apply the same source policies as inline operations;
conflicting consumer scale contexts explicitly reject.

The regular primary proof runner includes all new stage authors and positive/negative
Python/TypeScript consumers. Actual Rust/Python/WASM execution passes 13 figures per
host, source-expression filters and shared aliases. Static/Presented/Current capture
and profile edits preserve old requests after disposal. All 13 three-host PNGs have
identical RGBA channels. Native four-panel rendering, publication PNGs and rasterized
PDFs have been inspected; every PDF uses the supplied embedded Noto Sans.

GG-02 is COMPLETE for its assigned stage scope. `mise run check` passes; macOS
276 tests and Linux core/export 270 tests pass with none ignored. Actual
primary and aggregate binding proofs pass. The [acceptance report](evidence/phase-2-stages-2026-09-08.md)
retains commands, outputs, source hashes, images and limitations. Source reductions explicitly use the registered dataset before chart
filters/facet splits, while generated reductions use the prepared layer population.
Post-scale outputs cover size/color (size for point/rule); independent aesthetics,
physical units and complete default family behavior remain in GG-03 and later owners.
No broader ggplot2 or cross-library gate closes from this package alone.

## Integrated shape acceptance — 9 September 2026

[WP-S08](evidence/phase-2-shape-acceptance-2026-09-09.md) is COMPLETE at the retained
source snapshot over `fab2505951061eaafe9adb52c86b248ee0dfa6bf`. All 63 exports and
220 methods have per-item Pass verdicts under the finite typed d3-shape 3.2.0 profile.
Fresh macOS/Linux Python and Node/WASM generator, interaction and 1,544-update runs
pass; all host update records agree. There are 420 macOS workspace and 419 Linux
core/export/extension tests-doctests, and repository plus actual binding checks pass.
Native and nine publication images were inspected. All PNG/PDF bytes agree; two
Terminal SVG comparisons preserve control-coordinate differences no larger than
5.684341886080802e-14 within the existing tolerance and rasterize identically.
The remaining theme dash gap and two acceptance regressions are fixed without changing
oracle expectations. G-SHAPE passes; G-PARITY, G4 and WP-21/22/23 remain open.
WP-AX01/02 integration is recorded in the current axis handoff above. The owner's stop boundary is before WP-AX03.

## Phase 2 handoff verified; P2-00 integration contract delivered — 8 September 2026

The owner-requested predecessor **Review API simplicity** finished with completed
turns and no reported turn error. Verified `fab2505951061eaafe9adb52c86b248ee0dfa6bf`
(`Add primary chart authoring and shared host runtime`, 8 September 2026 23:11:16 UTC)
is HEAD and contains the primary Rust/Python/WASM implementation and completion
evidence. `git merge-base --is-ancestor fab2505 HEAD` passed. The task reader exposed
no final message for those completed turns; commit content, date and the committed
completion report establish the handoff. AP-09 performance qualification remains
open; it is not a prerequisite for independent Phase 2 contracts/legend acceptance.
The waiting heartbeat was paused for implementation and then deleted after its
handoff condition was fulfilled; it no longer polls the predecessor.

P2-00 is COMPLETE for its entry scope: [ADR-014](adr/014-phase-2-integration-contract.md)
records canonical-definition policy/provenance, compatible migration, explicit resource
ownership and the shared Node/separate R reference-lock layout. The
[coverage register](phase-2-coverage.md) links all eight lane inventories and GG-00–19
to requirements, semantic owners, AP-00 routes, fixtures, surfaces and open gaps.
It excludes delivered API foundations from remaining effort and preserves the combined
plan's dependency edges. Lane entries/GG-00 still own concrete reference installation,
locks and exhaustive method/argument fixtures; no oracle ran for this entry package.

Validation: `mise exec -- python3 scripts/check_repository.py` passed workspace/host
isolation and local Markdown file links on Darwin arm64. This is contract validation,
not runtime/parity certification. Baseline worktree contained the earlier Phase 2
plans/review/ledger edits; these are retained. No production dependency, portable
version or package publication changed. GG-01's remaining shared legend acceptance
is delivered in the following entry. G-AUTH, all D3 gates, G-GGPLOT, G-PARITY and
expanded G4 remain OPEN.

## GG-01 shared legend acceptance delivered — 8 September 2026

GG-01 / FIX-GG01 is COMPLETE for its specified scene and inspected SVG/PDF/PNG
matrix against `fab2505` plus this uncommitted slice. Shared layout now omits empty
guides, and primary `legend().untitled()` omits the title while retaining keys.
`generic_title()` explicitly retains the legacy Color/Value fallback in Rust, Python
and WASM. Existing primary fixture authors now use that explicit fallback; independent
expected fixtures and comparators remain unchanged. No second guide engine was added.

The [acceptance report](evidence/phase-2-entry-and-legends-2026-09-08.md) and
[retained evidence](evidence/phase-2-legends/README.md) cover 24 ordinary/collected/local
facet cases, including empty/hidden/tight/shared/incompatible guides and untitled
builds/edits. Actual Python and WASM each match all 24 Rust portable scenes exactly,
produce 72 exports and pass six host-dispatched untitled/generic edits. All 24 direct
PNGs and independently rendered PDFs were visually inspected. The full primary
runtime/type proof passes, including baseline component-family equivalence and replay.
Final `mise run check` and `mise run test` pass on Darwin arm64: 253 tests, zero
failed/ignored. The focused export run passes six tests including the 24-case matrix.
Final outputs equal the retained inspected artifacts; logs and source/artifact hashes
identify this validation snapshot.

The initial full test run exposed reliance on the old `untitled()` behavior in fixture
authors; that failure is retained alongside the corrected final evidence. Full guides,
colorbars and advanced collection remain GG-05. No fresh native-window, Linux, R oracle
or sustained performance run is claimed. Next package: GG-00's reference inventory/oracle
or a ready independent D3 entry; cumulative gates and AP-09 remain OPEN.

## Primary authoring implemented; final performance qualification pending — 8 September 2026

AP-00–08 are complete for the delivered LibraryV1 baseline. Data/Plot/Chart/Output,
all current component families, typed live operations, native/Kit/export destinations,
actual Python/WASM syntax and production-consumer migration are implemented. The
[completion report](evidence/primary-authoring-completion-2026-09-08.md) maps the
[35-row register](primary-authoring-api.md) to source, independent fixture and host evidence.
Revision: `b631f0e6d7e41722b5774433d616f704234157d7` plus this implementation;
the report's source inventory identifies the tested files. Unrelated Phase 2 edits remain
separate. All packages stay unpublished 0.1.0 and portable schemas stay version 1.

Final `mise run fmt`, `mise run check` and `mise run test` pass on Darwin arm64;
252 tests pass. Offline Linux aarch64 core/export/text tests/check/docs pass, 246 tests.
Actual `primary-authoring-proof` and `bindings-proof` pass. Primary execution covers
34 full component-family cases and 23 action + 47 input + 70 streaming steps in Rust,
Python and WASM, exact data, typed transactions, capture/disposal, detachment/memory
and strict declarations. Linux primary outputs pass the same independent comparison.
Native primary/streaming/Kit keyboard, selection, exact-value accessibility hooks,
historical pin/freeze/retention/queue/resume and Kit reset were executed and inspected.
SVG/PNG and searchable/outlined PDF artifacts were generated and inspected.

The native check exposed and fixed repeated frozen-frame acknowledgement entering the
resize-only path. The regression verifies repeat painting after live commit and failed
pending-scene admission without losing the retained source/capture policy. Updated
native execution passes. Field/identity validation and seeded-jitter fixture identity
were also corrected without changing existing expected values or tolerances.

AP-09 / G-AUTH remain open pending final native performance qualification. The initial
five primary workloads passed their applicable baseline protocols. A final-code rerun
recorded ten-line frame-work p95 21.095374 ms, followed by 20.712834 ms on repetition
(target 16.7 ms); hover remained below 4 ms. Platform submission dominates the slow
samples. Both dashboard attempts completed all 120 preparations per chart but stopped
painting before the drain snapshot; those attempts fail acceptance and are retained.
The unchanged baseline also failed the matched ten-line comparison at 21.160960 ms;
the paired primary run had fewer than 30 displayed samples. A refactor-specific
regression is not established, and a stable visible-window rerun has been requested. The ADR-008 baseline 30-minute
exception, full-assistive/other-platform limitations, dependency advisories and expanded
parity/G4 requirements remain explicit. Next action: complete the requested visible
native measurements, preserve failed traces, then close or explicitly leave G-AUTH open.

## Phase 2 implementation awaiting API commit — 7 September 2026

Historical waiting entry, superseded by the verified 8 September handoff above.

Owner authorized implementation after the task **Review API simplicity** completes
and commits its code. At this handoff its local task snapshot was active; no completion
or qualifying commit was verified. Phase 2 implementation remains pending that condition.
A follow-up in the Phase 2 task checks every five minutes, verifies the completed task's
commit and resumes from P2-00 against that baseline. It stays quiet while unchanged.
The handoff baseline remains `b631f0e6d7e41722b5774433d616f704234157d7` plus active
edits; the implementation start must record the actual qualifying revision. This
waiting step ran no feature checks and advanced no package or gate.

## Phase 2 plans reconciled with the current API — 7 September 2026

Owner-authorized planning update against `b631f0e6d7e41722b5774433d616f704234157d7`
plus active AP edits. Updated the [Phase 2 plan](impl_plans/phase-2-parity-implementation-plan.md),
[axis handoff](impl_plans/d3-axis-parity-plan.md),
[primary-authoring handoffs](impl_plans/primary-authoring-api-plan.md) and
[main integration plan](impl_plans/gpui-charts-implementation-plan.md) using the
[current API review](evidence/phase-2-current-api-review-2026-09-07.md).
Scope: GG2-01/02/04/12, AXIS-01/07, AUT-01/03–07 and BND-01/03/04.

GG-01 now reconciles the delivered shared painter and remaining FIX-GG01 edge/visual
cases. P2-00 is the first new parity handoff, linking AP-00's register and current
Data/Plot/Chart/Output owners. GG-02 specifies profile execution/wire/cache/capture
propagation; WP-AX01 includes primary handles/names/layer bindings and host navigation;
AP-07 and each semantic package share host syntax and actual export/declaration/runtime
proof responsibilities. Live Python/WASM authoring additions supersede the review's
earlier JSON-only source observation; their acceptance remains with AP-07's evidence.
Historical effort allowances now require subtraction of delivered AP/legend work before
estimating remaining effort. Original-scope WP completion is retained and expanded
requalification remains required. Historical review reports remain unchanged.

Validation in the repository on Darwin arm64 via mise:

- `mise exec -- python3 scripts/check_repository.py`: passed dependency/host-isolation
  graph and local Markdown file-link checks; no target execution or external-link/anchor validation.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/phase-2-parity-implementation-plan.md docs/impl_plans/d3-axis-parity-plan.md docs/impl_plans/primary-authoring-api-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md`:
  passed, retaining the repository's Markdown hard-break convention.
- Reviewed the updated handoffs: all eight D3 inputs, GG-00–19 and cumulative gate
  prerequisites are retained; the new API section is shared by the companion plans.

No feature tests or acceptance gates are claimed by this update. No production code, dependency,
fixture, public wire version or package/gate completion state changed in this task;
unrelated live edits were preserved. Next action: P2-00 contract/evidence reconciliation,
then GG-00 and ready D3 entry packages; GG-01 remaining acceptance can proceed independently.

## Phase 2 current API review — 7 September 2026

Reviewed the Phase 2 plan against `b631f0e6d7e41722b5774433d616f704234157d7`
plus the active primary-authoring working tree. The
[current API review](evidence/phase-2-current-api-review-2026-09-07.md) maps each
parity family to its current public owner. The capability scope remains applicable;
the GG-01 implementation kickoff is stale because ordinary and faceted legends now
share a painter and an export regression. Full GG-01 edge and visual acceptance
remains unresolved. Future packages must extend existing Data/Plot/Chart/Output,
define compatibility-profile propagation, migrate primary axis consumers with the
scale/guide identity split, and deliver actual Python/WASM exports and declarations.

On Darwin arm64, `mise exec -- cargo test -p chart-core --test authoring --test authoring_runtime --test authoring_host --locked`
passed 20 tests and `mise exec -- cargo test -p chart-export --test authoring --locked`
passed 4 tests. `mise exec -- python3 scripts/check_repository.py` passed graph and
local Markdown file-link checks; scoped `git diff --check` passed for tracked changes.
These checks cover the built Rust test binaries; Rust host dispatch
does not establish Python/WASM runtime support. No fresh native UI, Linux, external
oracle, performance or visual-inspection evidence was produced by this review.
No production code, plan or gate state was changed. Next action: reconcile the GG-01
handoff and attach the report's concrete API owners and acceptance cases to the
Phase 2 packages while coordinating with the active AP implementation.

## Primary authoring implementation in progress — 7 September 2026

Owner authorized the complete AP-00–09 plan. AP-00's baseline/public contract,
35 source-owner coverage rows, migration policy and acceptance handoff are recorded in
[the capability register](primary-authoring-api.md). Baseline remains
`b631f0e6d7e41722b5774433d616f704234157d7`; earlier owner-authorized planning edits are
preserved. No publication or parity-kernel implementation is implied.

AP-01 extracted typed Chart store/reducer/compiler/queue/query ownership; legacy JSON
Session now forwards to it, retaining eager portable preparation. New structural
validation accepts registered native-only operations without running statistics.
Owned and external-source routes remain distinct. Four new FIX-AUTH01 runtime tests
pass, including independent external views, native-only execution counting,
definition-only data/replay/queue preservation, typed/legacy histogram/replay parity.
The existing portable/streaming/scheduling/stage-cache selection passes 22 tests.

AP-02 has the first Data columns/rows, owner-scoped field/layer handles, immutable
Plot and inherited source aes/layer/histogram route. Four FIX-AUTH02 core tests pass:
independent geometry/bin membership, nullable/exact integer/timestamp preservation,
foreign-handle and missing-field diagnostics, and independent dataset overlays.
Native ChartInput::from_plot and the headless Output destination compile. Component
and publication tests, native lifecycle integration, complete runtime/binding/consumer
migration and performance acceptance remain in progress. No cumulative gate closes.

Commands on Darwin arm64 with mise Rust 1.97.1: `cargo metadata --no-deps --format-version 1 --locked`;
`cargo test -p chart-core --test portable --test streaming --test scheduling --test stage_cache --locked`;
`cargo test -p chart-core --test authoring_runtime --locked`;
`cargo test -p chart-core --test authoring --locked`;
`cargo check -p chart-export -p gpui-charts --locked` (all via `mise exec --`).
Next action: finish real publication proofs and component/runtime coverage, then the
remaining ordered packages. G-AUTH remains OPEN; the register records partial coverage.

### Continuing authoring implementation evidence

The later primary-authoring slice adds regression coverage for original-Plot edits after
live append, retry/queue preservation, retention rollback and queued epoch conflicts.
Automatic timestamp mappings now share an exact integer origin across owned datasets of
the same representation. Shared automatic color catalogs are trained before per-layer
color assignment. Facet authoring rejects ambiguous same-ordinal/different-name fields;
matching datasets must currently align facet-field schema positions or explicitly use
broadcast/panel targeting. Explicit timestamp origins remain authoritative and incompatible
origins reject. Shared automatic colors use ordered catalog union per coherent snapshot;
an explicit color domain fixes category-to-palette assignments across changing catalogs.

`mise exec -- cargo test -p chart-core --locked` passed the entire core suite and doctests
on this slice. `mise exec -- cargo test -p chart-export --test authoring --locked` passed
4 tests, including the 8-case Presented/Current × visible/full-domain × interaction matrix,
with requests executed after runtime disposal. Acknowledged destination layout is retained
as capture provenance and as the default Presented layout policy, with explicit publication
size/font/options applied. Native input now has tooltip/control/accessibility/edit/density
builders, and Kit has a primary plot mount helper. Native committed receipts remain committed
when later scheduling fails; that separate failure uses the existing diagnostic surface.

`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p gpui-charts-kit --all-targets --locked -- -D warnings`
passed before the subsequent host-binding dispatch additions. The refreshed
`mise exec -- cargo run -p chart-gallery --example primary_authoring --locked -- --headless`
produced updated `target/authoring/native-primary.{svg,pdf,png}`; PNG and a Poppler-rendered
PDF (`/private/tmp/finstack-primary-refreshed.png`) were visually inspected and the Peak
annotation is inside the plot. These checks do not certify native input/lifecycle or bindings.

AP-07 now has shared Rust host dispatch into actual typed component/draft builders,
typed host runtime/transaction/capture adapters, and primary Plot version-1 interchange
with names/profile/exact data. PyO3 and wasm-bindgen owned handles now expose these paths;
`packages/python/finstack_chart` and `packages/wasm/authoring.cjs` provide ordinary
column/row/component syntax without a separate grammar compiler. Source arrays cross
typed native vectors; Python integers and WASM BigInt retain exact 64-bit payloads.
Output resources, immutable requests/frames and deferred export queue/jobs are separate
owned handles with explicit disposal. Existing portable Chart APIs remain available.

The first smoke checks have been superseded by committed
[scripts/run_primary_authoring_proofs.py](../scripts/run_primary_authoring_proofs.py)
and `mise run primary-authoring-proof`. On Darwin arm64, Rust 1.97.1, Python 3.14.6,
Node 24.14.0, TypeScript 6.0.2, mypy 2.3.0 and wasm-bindgen 0.2.128, the actual Rust
executable, PyO3 extension and Node WASM module passed shared primary-author assertions.
The runner compares initial/final source semantics, scenes, actions, navigation,
transactions/replay, exact 64-bit timestamp/integer/null/category values, summary means
and OLS against independent expectations. Scene/navigation tolerances are 1e-9 destination
units and statistical tolerances 1e-12; opaque allocated identities are normalized only
after source identity relationships are checked. Actual SVG/PDF/PNG encoders execute.

Owned editor handles, named runtime commands, event-time retention, structured error
properties and separate public component classes now have Python stubs and TypeScript
declarations. JavaScript offers camelCase aliases. Positive typing examples pass;
five negative examples reject misplaced titles/subtitles/annotations/legends and data
replacement through definition edits. Host proofs also cover input mutation and one-time
row callbacks, display/boolean/null metadata, invalid/foreign fields, queue acceptance
versus commit, stale queued bases, update-versus-batch domains, count/event retention,
stale state commands, eight capture combinations, editor/request survival after disposal,
export-job limits/cancellation and repeated disposal. Python made 710 independent thread
steps during one 100,000-row Rust semantic call (0.891 seconds, debug build); this is a
detachment observation, not a performance claim. WASM memory stayed 6,815,744 bytes across
six batches of 100 create/dispose cycles with 1,000 rows, explicitly freeing all fluent
intermediate handles. A prior GC-timing-only run was unstable; garbage collection timing
is not a deterministic disposal guarantee.

Evidence: `target/authoring/{environment.json,*primary*.log,native,python,wasm,typing}`.
The Python/WASM PNGs were inspected; a fixture-only label offset was corrected so the
annotation remains inside the plot, then all three runtime fixtures were rerun.
Command: `WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen TSC_JS=/Users/jeickmeier/.npm/_npx/e04ecd76da0b5726/node_modules/typescript/lib/tsc.js PYTHONPATH=/Users/jeickmeier/.cache/uv/archive-v0/PKSDgIDzoTfeXd75NY8Rf/lib/python3.14/site-packages mise run primary-authoring-proof`.
`mise run fmt`, Python/export all-target Clippy and wasm32 all-target Clippy passed.
Baseline revision remains `b631f0e6d7e41722b5774433d616f704234157d7` plus these uncommitted
changes. The runner also now exercises the compiled custom density-histogram extension through
primary components in all three hosts. `custom_stat(...).field_parameter(name, field)`
resolves owner-checked source mappings only during authoring. The example owns its
`density_histogram`/`chamfered_bars` helpers and explicit registry installation/loading;
no parameter payload installs code. Independent counts/members/density and both generated
consumers match; the headless image was inspected. Native-only variants build as Rust
plots but reject portable runtime/serialization and headless preparation. Full remaining
component/standalone coverage, native lifecycle qualification, consumer migration and
AP-09 performance/platform gates remain open. The existing aggregate compatibility
runner also passed with
`WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof`:
36 stat/position/geometry cases, 23 action transitions, 47 input steps, 70 streaming steps,
40 held-capture steps, extension/schema/capability failures, rich text/facets/density and
independent vector/raster assertions across actual Rust/Python/WASM. Evidence is under
`artifacts/bindings`. This establishes preserved legacy contracts; the remaining delivered
component families still need primary-builder evidence. This shared host proof does not close every capability row or G-AUTH.

The current working tree adds grammar/stat/position/axis/color/text/theme/facet/figure
builders, named transforms, generated color mappings, grouping overrides, runtime
transaction/configuration helpers and immutable component edits. Compiler structural
validation now covers facets, axes, figure references and color stages without executing
statistics. Grid builders infer the complete row/column product. Ordinary charts and
facets reuse the same legend painter. Future GG mapped-symbol/linetype kernels remain
unimplemented and their coverage stays open.

Native ChartInput/ChartView now adopt the same typed Chart; GPUI retains tasks, a separate
worker compiler, frame acknowledgements and native resources. Committed source admission
is separate from worker preparation; an older admitted result cannot replace newer
committed data. Paint acknowledgement retains actual inspection state for Presented
capture. Supplied fonts have an automatic native resource constructor. Full native
lifecycle, sustained workloads and bindings still require requalification.

Additional commands, all in the repository on Darwin arm64 with mise Rust 1.97.1:

- `mise exec -- cargo test -p chart-core --test authoring --test facets --test layout --locked`:
  36 passed at that slice (7 authoring, 9 facets, 20 layout).
- `mise exec -- cargo test -p chart-core --test authoring --locked`: subsequently 8 passed,
  adding independent shared-transform/generated-color/reuse assertions.
- `mise exec -- cargo test -p chart-core --test authoring_runtime --test portable --test scheduling --test streaming --test stage_cache --locked`:
  27 passed, including older-worker/newer-commit and disposal checks.
- `mise exec -- cargo test -p chart-export --test authoring --locked`: 3 passed, including
  real SVG/PDF/PNG publication and ordinary/faceted legend text. A missing facet selector
  was corrected in the test and now also rejects during structural build.
- `mise exec -- cargo run -p chart-gallery --example primary_authoring --locked` and
  `mise exec -- cargo build -p chart-gallery --example primary_authoring --locked`:
  built the shared native/publication example. A sandboxed launch could not connect to
  macOS services; the compiled local example was subsequently launched outside the
  sandbox and as a temporary local app bundle for visual inspection. The actual native
  chart displayed both colored series, title/subtitle, axes, legend and caption. Keyboard
  interaction was not verified by this inspection.
- Artifacts: `target/authoring/primary-authoring.{svg,pdf,png}` and
  `target/authoring/native-primary.{svg,pdf,png}`. PNG and Poppler-rendered PDF were inspected.
  Inspection identified an annotation offset outside the default plot clip; the example
  now places that label inside the plot and awaits refreshed image inspection.
- `mise run fmt` passed at the recorded intermediate slice. Focused Clippy identified
  new clone-on-Copy/large-enum issues; fixes are applied but the final rerun is pending.
  Latest `mise exec -- cargo check -p chart-core --locked` passes after immutable edit work.

These counts precede the newest edit/configuration changes; they do not certify those
changes, actual Python/WASM execution, complete native lifecycle or performance gates.
Next: behavioral coverage for primary edits/transactions/configuration, remaining
component ownership checks, native/export consolidation, then AP-07–09. G-AUTH stays OPEN.

### Primary component-family qualification

The primary Rust, Python and WASM authors now execute 34 existing independent
statistic/position/geometry/scale/facet/composition cases through explicit builders.
The shared Rust fixture checks exact source materialization, full semantic output and
resolved marks against the original fixture definitions, rebasing only allocated
layer/dataset/field/scale identities. Python/WASM use ordinary columns and explicit
components, then compare complete semantics and scenes with Rust at the existing
1e-12/1e-9 tolerances. No fixture expectations or visual baselines were changed.
Coverage includes overflow/automatic bins, summary/count/OLS and affine/filter variants,
stack/normalization/dodge/data/display jitter, log/symlog/point/UTC/session scales,
area/ribbon/cells/OHLC/volume, group color and affine secondary axes; seven facet cases
cover catalogued empty grids, free axes, broadcasts/targeted layers and group/facet/chart
stat populations. Three publication cases cover named themes, gradients/symbols/dashes,
explicit regular/bold/Arabic fonts, rich titles/axis labels, ordered notes, panel letters,
callout/coordinate-space labels and insets using existing prepared layers.

This exposed and fixed two missing primary controls (`LayerBuilder::color_group` and
`LegendBuilder::untitled`) and bar/volume defaults discarded by `.aes(...)`. Bar baselines
are now recipe options used when y2 is unmapped; explicit y2 mappings retain precedence.
Python/WASM dispatch and declarations include the added controls. Native editorial
composition, OHLC/volume and grid artifacts were inspected; the six-panel grid's initial
400×260-point page correctly reported layout pressure and was enlarged to 600×540 points
for its proof artifact. Editorial composition uses 180×120 mm at 96 DPI.

Commands: `mise exec -- cargo test -p chart-core --test authoring_families --locked`;
`mise run primary-authoring-proof` with the task-local tool paths above; after extending
facets/composition, each actual Rust/Python/WASM producer and
`mise exec -- python3 scripts/bindings/authoring/compare.py target/authoring` ran again.
The comparison passed all 34 primary families. Outputs are
`target/authoring/{native,python,wasm}/{families,family-scenes}.json` and native PNGs.
The expanded WASM workload plateaued at 19,070,976 bytes in its six explicitly freed
batches; its higher peak includes the additional families and supplied fonts. The latest
Python detachment check recorded 627 thread steps over 0.786 seconds (debug build).
Full native lifecycle/input/Kit qualification, remaining option/runtime coverage,
consumer migration and AP-09 performance/platform gates remain open.

### Consumer migration in progress

The main native gallery now uses ordinary `Data::rows`, named recipe builders and
immutable Plot edits, with owner-derived field handles for inspection. Family/facet/
composition galleries reuse explicit recipes in `examples/common/authoring_fixtures.rs`;
the independent raw-fixture comparison remains in tests. Kit mounts these primary plots
through its supplied-theme input helper. Extension, action and interaction galleries now
use primary construction. The linked host-tools example shares Data between charts,
builds callouts, derives actual layer identities and uses `link` through native
`capture_link`/`resolve_link`; captured host-command export still uses its specialist
snapshot boundary to retain the event's exact scene.

The curated prelude exposes builders and relevant option enums without host dispatch or
ambiguous profile exports. Root Rustdoc and README now lead with Data/Plot authoring;
`docs/authoring-guide.md` documents current components, runtime/capture ownership, host
usage and the 0.2/0.3 additive migration policy. No old public module was removed.
Migration exposed named-axis insertion discarding prior primary-axis settings and missing
Rust edit x/y-axis conveniences; both are fixed. The meaningful regression checks
non-default bar baselines before/after aes, retained primary axes and stable edit handles.

`mise exec -- cargo test -p chart-core --test authoring --test authoring_families --locked`
passed 12 focused tests plus two corpus tests covering 34 cases. `cargo check` passed
main/family/Kit/extension/actions/interaction/host-tools consumers via mise. `mise run fmt`, repository graph/link validation and
`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p chart-gallery --all-targets --locked -- -D warnings` passed on the latest slice. Streaming,
scheduling/live-export/benchmark consumers, native interaction/lifecycle/Kit execution,
complete option/runtime coverage, docs/type/schema validation and AP-09 measurements
remain in progress. G-AUTH stays open.

### Primary runtime consumers and documentation

The streaming gallery now owns one Chart and uses named transaction/retention and
queue builders. The scheduling gallery and finite/sustained native benchmarks author
ordinary columns and components, commit through the mounted Chart, and keep explicit
CPU/GPU/presentation instrumentation. The live-export gallery and sustained benchmark
use Output::live_request; annotation definition edits and later commits cannot alter an
accepted request. The publication tutorial now uses Data rows, Plot and Output with
physical dimensions, supplied fonts and text/DPI options. Raw fixtures remain only in
compatibility/diagnostic proof programs and internal stage benchmarks.

Primary `external_view`/`accept_from` now preserve authored handles and definition edit
ownership while sharing committed snapshots and keeping independent reducers. Rust
checks prove pointer-shared snapshots, explicit source admission, rejection of a copied
writer and continued view preparation after writer disposal. Python/WASM expose the
same operations with fresh actual-runtime checks included in the primary proof runner.

README, the authoring guide, release guide, portable compatibility contract, historical
alpha API guide and changelog now lead with the implemented primary surface. The planned
0.2.0 migration retains old public paths and wire version 1; removal is no earlier than
0.3.0. Current packages remain 0.1.0 and unpublished. No separate Phase 2 semantic gate
is advanced. Performance requalification and remaining integrated/native checks follow.

Validation so far: native example compilation and all-example Clippy with
`--features performance,kit --locked -- -D warnings` pass. The focused core external-view
suite passes all eight tests. These are intermediate results; final aggregate and actual
host/native/performance results will be recorded separately below.

## Grouped aesthetic contract recorded in the plan — 7 September 2026

Added section 3.5 to the [primary authoring plan](impl_plans/primary-authoring-api-plan.md)
with the proposed grouped line/point example and independent group/color/fill/shape/
linetype/size/linewidth/alpha channels. The contract distinguishes group membership
from scale-selected appearance, supports same or different mapping fields, preserves
plot/layer inheritance and overrides, and keeps constants and legends with their
respective builders. Mapped styles must not require a separate layer per category.
AP-03 acceptance now covers actual resolved styles, line membership, inference and
override rules, legend keys, inspected output and AP-07 host proofs. GG-02–05 retain
semantic ownership; the example is explicitly prospective.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; outcome is uncommitted
plan/ledger edits preserving earlier work. AP-00–09 remain PLANNED, GG-02–05 remain
NOT STARTED and G-AUTH remains OPEN. Next action: AP-00's API walkthrough includes
section 3.5. Validation: `mise exec -- python3 scripts/check_repository.py`, temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py`, and
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/implementation-status.md`
passed on Darwin arm64. Documentation-only checks; no proposed API/runtime tests ran.

## Grouped aesthetic API clarification — 7 September 2026

Reviewed AUT-03 / GG2-03 against current `SourceAes`, `ColorEncoding`, `ThemePatch`
and GG-02/03. The planned primary `aes()` supports independent group, color, shape
and linetype mappings, inherited or overridden per layer. Group selects connected
observations/statistical populations; scales choose visual encodings. Current source
mappings provide explicit group and size; separate color encoding supports source
categories and prepared groups. Symbols/dashes are currently plot/layer theme values,
not mapped shape/linetype channels. GG-02/03 remain NOT STARTED; no inferred grouping
from color or complete mapped symbol/line-style support is certified by this answer.

Revision `b631f0e6d7e41722b5774433d616f704234157d7`, with existing uncommitted planning
edits preserved. Evidence is source/plan inspection only; no runtime tests ran.
Validation: `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
passed. Next action remains the recorded AP-00 walkthrough; mapped aesthetic semantics
retain GG-02/03 ownership and primary builder integration in AP-03. G-AUTH remains OPEN.

## Implemented builder inventory and future integration rule — 7 September 2026

Added a source-linked 16-family component inventory to section 3.3 of the
[primary authoring plan](impl_plans/primary-authoring-api-plan.md): explicit figure
furniture/callout/inset builders, existing marks/statistics/positions, transforms,
scales/axes/legends, facets, theme/text/layout, runtime configuration, host hooks,
publication and registered components. Builder names remain proposed. The inventory
separates implemented fixed annotations from future mapped labels and other parity
requirements; the introductory sketch now uses the existing Editorial theme.

Section 3.4, AUT-01 and ADR-013 require every future feature to extend an existing
typed builder or add a focused component within the primary API. Its own package
includes primary examples, applicable bindings/serialization updates and behavioral
evidence. Runtime operations remain Chart/destination methods. AP-00/04/08 acceptance
now carries inventory expansion, composition evidence and the ongoing integration rule.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
plan/specification/ADR/ledger edits, preserving earlier changes. Source inspection
covered the linked current core grammar/composition/layout/runtime and native/export
declarations. No runtime implementation or schema changed; AP-00–09 remain PLANNED
and G-AUTH remains OPEN. Next action: AP-00's option-level register and API walkthrough.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` and temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed documentation links,
package/fixture/ledger and requirement/dependency consistency.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime test or proposed-builder compilation ran; this is planning evidence.

## Separate annotation and figure builders — 7 September 2026

Updated the [primary authoring plan](impl_plans/primary-authoring-api-plan.md),
AUT-04 and ADR-013 to the owner's direction: labels are x/y-positioned annotations;
titles, subtitles, x/y axis labels and legends have separate builders. Section 3.1
owns the proposed routes and supersedes the older combined-labels sketch. AP-00/03/04
now require separate component examples, correct annotation identity/provenance,
automatic legends and shared text/guide implementation. FIX-AUTH04 includes component
isolation and compile-fail checks against misplaced labels-builder setters.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/specification/ADR/ledger edits, preserving prior review and plan updates.
No implementation or wire format changed; AP-00–09 remain PLANNED and G-AUTH OPEN.
Next action remains AP-00, using the revised builder boundaries. Intended acceptance
has not been executed; this is a documentation-only update.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed; the temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed the unchanged package,
requirement and dependency checks. `git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime or proposed-builder compilation was run.

## Primary authoring plan review findings incorporated — 7 September 2026

Updated the [plan](impl_plans/primary-authoring-api-plan.md) and
[ADR-013](adr/013-primary-authoring-api.md) at the owner's request. APR-01–04 are
addressed in the planning contract: definition-only edits preserve current data;
owned ingestion and external committed sources retain one commit authority, with
independent view/worker/export lifetimes; Presented/Current export basis is separate
from navigation/interaction policy; structural build and native/portable validation
are separate. AP-00/01/02/05/06 now carry the corresponding handoff and regression
acceptance. The AP-00–09 sequence and AUT-01–09 remain unchanged.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/ADR/review-follow-up/ledger documentation. Prior review edits were preserved.
The plan records the reviewed completed original-scope baseline, without reclassifying
expanded parity or original acceptance. No implementation, dependency, wire schema,
fixture or release setting changed. Review findings are addressed in the plan only;
all AP packages remain PLANNED and G-AUTH remains OPEN. No runtime tests or proposed
API execution ran for this documentation-only update. Next action: AP-00's baseline
and public-contract handoff, then AP-01 under the clarified ownership map.

Validation in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed repository/dependency
and local-link checks; `mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed
ten packages/fixtures/ledger rows, nine requirement mappings, fifteen capability
families and an acyclic sequence. An inline `mise exec -- python3 -` check verified
the four review findings' representation and updated-document whitespace.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/implementation-status.md`
passed. These are documentation checks, not AP implementation or runtime evidence.

## Primary authoring plan checked against the current library — 7 September 2026

Outcome: reviewed the plan against clean revision
`c773fcba9a8c846322d08c9c1203fce415e91328`, including the now-completed original
WP-17–23 source and contracts. The original-WP assumption matches the ledger's
original-scope completion records; expanded parity and G-AUTH remain open.
HEAD later advanced to `b631f0e6d7e41722b5774433d616f704234157d7`; its changes were
unrelated skill additions and did not alter the reviewed library or plan files.
[Review evidence](evidence/primary-authoring-plan-review-2026-09-07.md) records
four actionable handoff gaps: APR-01 definition-only edits must preserve live data;
APR-02 map externally managed sources and worker/view lifetimes before runtime
consolidation; APR-03 retain Current/Presented export bases; APR-04 separate native
execution from portable capability validation. AUT-01/02/03/05/06/08/09, DAT, STM,
ARC, EXP and BND contracts are affected. These are plan gaps, not new demonstrated
runtime regressions. Overall architecture/reuse direction passes; the specific
handoff contracts remain Fail or Uncertain as detailed in the report.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:

- `mise exec -- cargo metadata --no-deps --format-version 1 --locked`: succeeded;
  inspected nine workspace packages and direct dependency ownership.
- `mise exec -- cargo test -p chart-core --test streaming --test scheduling --test stage_cache --locked`:
  **16 passed**, zero failed (nine streaming, four scheduling, three cache).
- `mise exec -- cargo test -p chart-export --test live_export --test extensions --locked`:
  **9 passed**, zero failed (five live export, four extensions).
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown links; graph checks do not establish runtime support.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`:
  passed. An inline `mise exec -- python3 -` check passed new-report whitespace and
  the four APR finding identifiers; no feature acceptance was implied.

Result: uncommitted review report and this required ledger entry only. Plan,
implementation, fixtures, dependencies and prior evidence were preserved. No proposed
API compilation, actual Python/WASM proof, native/artifact inspection, Linux run or
sustained benchmark ran. Next action: tighten AP-00/01's ownership/edit/validation
handoff and AP-06's capture contract using APR-01–04; retain AP-00–09 and all open gates.

## Primary authoring API planning — 7 September 2026

Planning complete in the [primary authoring plan](impl_plans/primary-authoring-api-plan.md).
Owner direction: concise authoring is the main public API and every feature runs
through it. Per explicit instruction, the plan assumes original WP-01–23 complete;
this is a prospective input, not a change to their recorded implementation states.
Additional D3/GG work retains its scope/owners and must integrate through the primary
API when delivered. No historical gate or feature is certified by this assumption.

Specification/main-plan/migration versions advance together to 0.5.0. AUT-01–09,
FIX-AUTH00–09 and G-AUTH govern capability-complete authoring, a shared typed runtime,
data/identity, design, live interactions, native/export, bindings and migration.
[ADR-013](adr/013-primary-authoring-api.md) records accepted ownership: builders in
chart-core, existing normalized engine/runtime reused, host resources in adapters,
standalone utilities directly callable, no new façade crate or second compiler.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60` plus live edits. Result:
uncommitted plan/ADR and focused authority/traceability/ledger updates. Existing source,
reviews and parity plans were preserved. No dependencies, implementation, fixtures,
portable versions or release/publication settings changed. Those planning-only statements retain their original revision context. The current
package states below are updated by the implementation evidence above.

| Package | State | Prerequisites | Next action / evidence |
| --- | --- | --- | --- |
| AP-00 — Public contract and baseline register | COMPLETE | Assumed completed original WP-01–23 | Contract/register and migration policy in primary-authoring-api.md; FIX-AUTH00. |
| AP-01 — Shared typed runtime | COMPLETE | AP-00 | Typed owned/external Chart, independent worker caches and legacy forwarding qualified; E1/E3/E4. |
| AP-02 — Primary data/plot/layer path | COMPLETE | AP-01 | Ordinary Data/Plot route, exact ownership/diagnostics and actual native/publication examples qualified; E1/E4. |
| AP-03 — Complete grammar and aesthetics | COMPLETE | AP-02, applicable semantic owners | All delivered grammar/scale/extension families qualified through primary authors; E2. |
| AP-04 — Design/composition/specialists | COMPLETE | AP-03, applicable semantic owners | Facets/themes/text/composition and retained standalone helpers qualified; E2/E4. |
| AP-05 — Live Chart features | COMPLETE | AP-04 | 23 action, 47 input and 70 streaming steps, edits/rollback/replay and native scheduling qualified; E1/E3/E5. |
| AP-06 — Native/Kit/export integration | COMPLETE | AP-05 | Actual native/Kit input hooks, capture matrix, deferred resources and inspected SVG/PDF/PNG qualified; E4. |
| AP-07 — Host-native Python/WASM authoring | COMPLETE | AP-06 | Actual Rust/Python/WASM family/runtime proofs, typing, exact values, disposal/detachment/memory and compatibility pass; E2/E3/E5. |
| AP-08 — Consumer/docs/API migration | COMPLETE | AP-07 | Primary production consumers/docs and version-1 compatibility migration pass; E4/E5. |
| AP-09 — Full coverage and requalification | IN PROGRESS | AP-08, all target-release capability acceptance | Full register/platform proof complete; final visible native performance rerun pending; E5 / G-AUTH OPEN. |

Planning validation executed in `/Users/jeickmeier/Projects/finstack-chart`,
7 September 2026, Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links. Target graph checks are not runtime proof.
- `mise exec -- python3 /tmp/check_chart_authoring_plan.py`: passed ten package/fixture/
  ledger rows and an acyclic sequence, nine normative/traceability IDs, fifteen routing
  families, open-gate/assumed-baseline wording, synchronized 0.5.0 authority versions,
  and new-plan/ADR links and whitespace. This temporary documentation checker is not
  a feature acceptance runner; its compact requirement-ID parsing was corrected before
  the passing run.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md docs/alpha-api.md`:
  passed, retaining the repository's Markdown hard-break convention. The temporary
  checker also checks the untracked plan and ADR, which Git's tracked diff omits.

G-AUTH remains OPEN. No Rust runtime tests, actual Python/WASM execution, native/export
inspection, sustained performance run or proposed-API compilation was performed for
this documentation-only task. Next implementation action: AP-00 under the stated
completed-baseline assumption. Detailed acceptance commands are the existing scoped
and final repository gates; the plan specifies intended evidence, not results.

## Phase 2 parity implementation planning — 7 September 2026

Planning complete in the [secondary implementation plan](impl_plans/phase-2-parity-implementation-plan.md).
Scope: consolidate all eight D3 module plans and the ggplot2 review into Phase 2,
with shared ownership, prerequisites, full capability coverage and intended evidence.
Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; result is uncommitted
documentation. Existing source, review and planning changes are preserved. No runtime,
dependency, fixture baseline or portable envelope version changes were made by this task.

Specification/main-plan/migration document versions advance together to 0.4.0.
GG2-01–12 explicitly adopt ggplot2 capability scope, including models/density,
coordinates/geography, mathematical text and saving devices. D3 package inventories
remain in their existing plans. P2-00 owns integration coordination; GG-00–19 own the
new ggplot2 slices and FIX-GG00–19. CLR-04 is now an explicit SP-04 prerequisite because
the latter consumes its descriptors and paint lowering. Historical WP/G2 evidence is
unchanged; WP-16 remains under its active assignment and WP-17–20 remain prerequisites
for final integration. G-PARITY joins G3, all eight D3 gates and G-GGPLOT before the
existing WP-21–23 final certification. No implementation gate advanced.

At the original planning run, all new package owners were unassigned and implementation
evidence was absent. The table below incorporates the subsequent current API handoff
reconciliation; historical paragraphs above retain their original revision context.
READY permits the stated next action and does not certify the feature. Exact owned
work and FIX-GG cases are defined once in the plan.

| Package | State | Requirement scope | Evidence / next action |
| --- | --- | --- | --- |
| P2-00 — Integration contract and coverage register | COMPLETE | GG2-01/12, ARC-03/04, BND-01, QLT-02/05 | ADR-014 and phase-2-coverage.md; committed API baseline fab2505 reconciled. Reference locks/oracles remain with lane entries and GG-00. |
| GG-00 — Reference inventory and executable oracle | COMPLETE | GG2-01/12 | Pinned 643-export inventory, 57 R sources, 32 reproducible seed records/96 artifacts; [entry evidence](evidence/phase-2-oracles-2026-09-08.md). Semantic argument matrices remain with delivering packages. |
| GG-01 — Reconcile shared legend acceptance | COMPLETE | GG2-04, GRA-07, SCL-05, LAY-03, THM-03 | FIX-GG01: 24 cases, exact actual Python/WASM scenes, 216 exports and inspected PNG/PDF sheets; empty-guide and untitled defects repaired. Evidence: phase-2-entry-and-legends-2026-09-08.md. Full guides remain GG-05. |
| GG-02 — Compatibility profile, stages and inferred grouping | COMPLETE | GG2-01/02 | [Stage acceptance](evidence/phase-2-stages-2026-09-08.md); primary/binding proofs pass. Later ggplot2 families remain separate. |
| GG-03 — Independent aesthetic encodings | COMPLETE | GG2-03 | [Accepted](evidence/phase-2-aesthetics-2026-09-10.md): 104 R glyph records, 12 core regressions, actual hosts/update/publication/native, 487 macOS / 468 Linux tests and full check. |
| GG-04 — ggplot2 scale and palette policies | COMPLETE | GG2-03 | [Qualified owned scale contracts](evidence/phase-2-ggplot-scales-2026-09-14.json): 862 macOS tests/doctests, full check and 672 cumulative runner commands with inline assertions pass against verified source. [Coverage](evidence/ggplot-scales-coverage.md) maps all 152 exports, 960 formal occurrences, 276 methods and 160 fields. Named guide/extension and full export certification remain GG-05/GG-16/GG-19. |
| GG-05 — Complete guides and legend composition | COMPLETE | GG2-04 | [Full package acceptance](evidence/phase-2-ggplot-guide-completion-2026-09-14.md): 895 workspace tests and repository checks pass; source-backed guide composition, final actual host proofs and inspected native/SVG/PDF/PNG output. Angular guides remain GG-13; global host/certification gates remain open. |
| GG-06 — Bin/count/summary and position semantics | COMPLETE | GG2-02/05 | Reference bins/count/summary and positions; physical-axis/facet training, retention, actual hosts and 63 identical publications; see 15 September completion report. |
| GG-07 — Primitive and interval recipe completion | COMPLETE | GG2-06 | Shared recipes, interval controls, StatSum, polygon/raster and stroke semantics; 111 exact publications per host, inspected native/export, final focused tests and repository checks. See completion evidence for full-suite correction boundary. |
| GG-08 — Data-driven text, labels and annotations | COMPLETE | GG2-06/09 | Source/stat labels, boxes/units/overlap, vectors and nearest/interpolated raster; 29 identical Rust/Python/WASM publications and native inspection; see 15 September evidence. |
| GG-09 — Distributional and one-dimensional analytical layers | COMPLETE | GG2-05/06 | [Combined acceptance](evidence/phase-2-ggplot-analysis-facets-completion-2026-09-15.md):1,029 workspace tests, repository checks,69 analytical publications per actual host and inspected native/export output. |
| GG-10 — Smoothers, confidence bands and quantile regression | COMPLETE | GG2-05/06 | Weighted LM/canonical GLM, LOESS, automatic cs-REML GAM, BR/FN quantiles, model registration, grids/uncertainty and update/facet/inspection proofs. 30 identical publications per host; [16 September completion](evidence/phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.md) retains named model capability boundaries. |
| GG-11 — Two-dimensional statistics and contours | COMPLETE | GG2-05/06 | Shared rectangular/hex bins, KDE, rotated-grid contours/isobands and ellipses, generated fields, colors and guides. 48 identical publications per host and inspected native output; [16 September completion](evidence/phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.md). |
| GG-12 — Facet semantics and layout breadth | COMPLETE | GG2-07 | [Combined acceptance](evidence/phase-2-ggplot-analysis-facets-completion-2026-09-15.md): reference populations/layout,51 publications per actual host, all17 native authors,1,029 workspace tests and repository checks pass. Parsed labellers remain GG14. |
| GG-13 — Cartesian, transformed and polar/radial coordinates | COMPLETE | GG2-08 | Shared post-stat maps, typed views/guides, clipping, raster coverage, inspection/navigation and coordinate-aware static presentation. 42 identical publications per host, all14 native authors inspected; [16 September completion](evidence/phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.md) retains explicit inverse/animation/Dot boundaries. |
| GG-14 — Theme hierarchy and mathematical typography | COMPLETE | GG2-09 | 160-node theme/source controls, nine presets and measured furniture; 60 theme, 27 math and 24 furniture publications agree across actual hosts. Signed-atom proofs qualify vertical metric adaptation; rotated multilingual math and text/outline at 300/600 DPI inspected. Final workspace/repository checks pass through resumed runs; see final wave report. |
| GG-15 — Geographic layers and coordinates | COMPLETE | GG2-08 | All41projection methods,318source successes/108descriptor errors,13integration+five source units and19native modes pass. Fresh Rust/Python/Node WASM57publications per host and scene JSONs match, including300DPI/dateline/holes; pinned libm corrects native/WASM UTM transcendental differences without output rounding. ADR031 records projection/source/license provenance. Final workspace/repository checks complete through resumed runs; see final wave report. |
| GG-16 — Extensibility and authoring conveniences | COMPLETE | GG2-10 | Ten independent external authors now cover each boundary, including prescribed-slope models and existing registered facet-labeller callbacks with exact typed panel context. Fresh Rust/Python/Node WASM30publications per host and scene JSONs match; native ten cases inspected. Malformed/native-only/replay/copy/disposal,35vector controls and combined update/batch/rollback pass. Final workspace/repository checks complete through resumed runs; see final wave report. |
| GG-17 — Saving and device capability completion | IN PROGRESS | GG2-11 | Implementation and local qualification pass: 26 save dimensions/custom devices, retained PDF/PS/TIFF pages, independent device decoders and 30 exact publications per actual host. Final checks pass through resumed runs. Windows EMF playback remains the sole package acceptance gap; prepared verifier and device matrix record the next action. |
| GG-18 — Full grammar, update and host integration | IN PROGRESS | GG2-02–12 | Five Rust integration tests, four independent authors and 12 exact publications per host; updates/batch, source/derived selection, alternate profiles and post-disposal captures pass. Native public-API interaction, presented capture, required legacy host proof and scoped primary proofs pass. Workspace/repository checks complete through resumed runs. Final acceptance awaits GG17 Windows playback; optional broader primary runner was deliberately partial, as recorded in final wave evidence. |
| GG-19 — Capability certification and handoff | NOT STARTED | GG2-01–12 | Requires GG-00–18 and all eight D3 certification packages; then G-GGPLOT/G-PARITY and WP-21/22. |

Planning evidence: all nine review/plan documents, normative contracts and selected
ADR/validation boundaries read; official ggplot2 4.0.3 index, tagged namespace,
aesthetic-stage, smoothing and saving references checked. No R oracle or D3 reference
suite ran. New GG-only effort is provisionally 182–315 engineer-days plus 2–4 for
P2-00, excluding shared D3, Phase 1 and final hardening; re-estimation gates are in
the plan. This is a scope-based allowance, not a delivery promise.

Validation executed in `/Users/jeickmeier/Projects/finstack-chart` on 7 September 2026,
Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown file links. Resolved target graphs are not platform
  runtime execution.
- `mise exec -- python3 /tmp/check_chart_phase2_plan.py`: passed 94 package nodes plus
  12 gate nodes with no cycles/unresolved dependencies, eight D3 inputs/backlinks,
  eight GGP findings, 20 GG package/fixture/ledger entries, 12 normative/traceability
  IDs, 231 local file/heading links, new-plan whitespace and 182–315 day arithmetic.
  This is a temporary documentation consistency check, not a feature acceptance runner.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving the
  established Markdown hard-break convention. New-plan whitespace was checked above
  because untracked files are outside `git diff --check`.

No Rust feature tests, native UI/artifact inspection, Python/WASM runtime, Linux or
performance gates were run for this planning-only assignment. Next implementation:
GG-01's current legend defect; next parity-program entry: P2-00, then GG-00 and D3
entry packages. The linked plan and these ledger rows are the retained planning result.

## ggplot2 feature parity review — 7 September 2026

Review complete against the official ggplot2 reference displaying version 4.0.3;
[full comparison and ranked findings](evidence/ggplot2-parity-review-2026-09-07.md).
Reviewed base `127fe2d4853f89b62ba59a17248485e3c378ba60` plus the concurrent working
tree. Result is uncommitted review evidence; existing inspection/state/category-window
and D3 planning edits are preserved. No production code, fixtures, baselines,
dependencies or normative parity requirements were changed by this review.

**Full ggplot2 parity is not established.** Basic Cartesian grammar/publication is a
tested subset. Missing statistical families, independent aesthetic scales, full
guides/facets/coordinates and deliberate binning/grouping/size differences remain.
GGP-01 reproduces a current defect: a non-faceted colored scatter prepares a two-entry
legend but its final scene contains no legend title. GGP-02–08 distinguish parity
gaps and absent evidence from violations of current scope. Relevant existing IDs:
GRA-01–08, SCL-01/05, LAY-01/03, THM-03, ARC-03, QLT-02/03, BND-03/04 and SCP-03.
GGP identifiers are review findings, not new normative requirements.

Evidence from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust `1.97.1 (8bab26f4f 2026-07-14)`:

- `mise exec -- cargo test -p chart-core --test grammar --test statistics --test full_scales --test facets --locked`:
  **47 passed, 0 failed** (20 grammar, 14 statistics, 4 full_scales, 9 facets).
- `mise exec -- cargo test -p chart-export --test composition --test publication --test extensions --locked`:
  **20 passed, 0 failed** (6 composition, 10 publication, 4 extensions).
- A Rust scene probe confirmed missing single-panel legends and the explicit grouping
  requirement for category-colored lines. Retained [source](evidence/ggplot2-parity-2026-09-07/probe.rs)
  and [command/output/core binary hash](evidence/ggplot2-parity-2026-09-07/probe.log).
  Its faceted positive control painted the legend successfully.
- `mise exec -- python3 scripts/check_repository.py` passed workspace/dependency
  isolation and local Markdown links. Ledger whitespace and an inline new-report
  whitespace/finding-ID/count check passed; commands are retained in the review.

Tests certify the binaries built during this review, not subsequent concurrent edits.
No fresh R/ggplot2 oracle, Python/WASM runtime, native UI/image inspection, Linux,
performance or full matrix ran. Existing G2 evidence retains its stated scope;
no gate was advanced. Next action: fix GGP-01, then select a bounded ggplot2 capability
target and explicit compatibility policies while reusing the planned D3 foundations.

## D3 interpolation parity planning handoff

Planning is complete in [the interpolation plan](impl_plans/d3-interpolate-parity-plan.md).
The implementation does **not** have d3-interpolate parity. ITP-01–08/FIX-I01 and
G-INTERPOLATE are required by specification 0.3.0. This is uncommitted documentation
on starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; existing action/binding
changes and concurrent shape/scale/axis/color/chromatic/path planning are preserved.
CLR-02/03 own color parsing/conversion; WP-IP owns interpolation; SP owns normalization;
CP owns named ramps. No implementation, fixture, dependency or wire version was changed
by this assignment. Next interpolation package: **WP-IP01** after the accepted WP-14.

| Interpolation package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-IP01 — Contract and reference harness | COMPLETE | Core | ADR-017, 370 Node and 36 pinned Chromium cases, exact regeneration | ITP-01/08 | Requires WP-14; reconcile pinned export/source inventory and share the reference lock with color/scale/chromatic/axis. |
| WP-IP02 — Scalar kernels and composition | COMPLETE | Core/hosts | 56 reference cases, independent contracts and actual host proofs pass | ITP-02/03 | Requires WP-IP01. |
| WP-IP03 — Structured values | COMPLETE | Core/hosts | 191 reference cases, target dispatch matrix, ownership and actual hosts pass | ITP-01/02/07 | Requires WP-IP02; color dispatch joins WP-IP04 in integration. |
| WP-IP04 — Color interpolation | COMPLETE | Core/hosts | 106 reference cases, exact CSS, independent anchors and actual hosts pass | ITP-04 | Requires WP-IP02 and CLR-03; consumes the shared color engine. |
| WP-IP05 — Transform and zoom interpolation | COMPLETE | Core/hosts | 17 zoom and 36 pinned browser cases plus actual hosts pass | ITP-05/06 | Requires WP-IP02. |
| WP-IP06 — Portable and chart integration | COMPLETE | Core/hosts/publication | [Integration proof](evidence/phase-2-interpolation-integration-2026-09-09.md): registered factories, shared scale/mark/guide consumers, actual Rust/Python/WASM, three inspected native/publication states and retained updates | ITP-07 | Version 2 standalone / version 12 registered charts; native-only export rejection and registry snapshot lifetime qualified. |
| WP-IP07 — Parity certification | COMPLETE | Cross-lane acceptance | [27-export verdicts](evidence/phase-2-interpolation-integration/verdict-catalog.md), [axis consumer](evidence/phase-2-axis-certification-2026-09-09.md), fresh host replay and finite benchmark | ITP-01–08 | G-INTERPOLATE passes for the declared typed profile; release gates remain separate. |

## D3 path parity planning handoff

The [path gap review and delivery plan](impl_plans/d3-path-parity-plan.md) is complete
as documentation, reviewed at `1cb955740c2dad2607b0a2330201125294cab5d0` plus the live
working tree. The specification now requires PTH-01–06, FIX-P01–06 and G-PATH.
**Implementation does not have d3-path parity.** Missing capabilities include the
standalone builder/serializer, arcs/arcTo, signed rectangle subpaths, D3 state behavior
and precision controls. WP-P01–04 own the common foundation consumed by WP-S01;
their provisional 9–16 developer-days overlap the shape estimate. WP-S01 completion
now requires WP-P04; WP-21 and G4 require G-PATH. Existing completed packages retain
their recorded scope; concurrent action and other parity work is preserved.

Evidence: 7 September 2026, Darwin arm64, Rust 1.97.1, Node v24.14.0, working directory
`/Users/jeickmeier/Projects/finstack-chart`. Compared live sources with official docs
and pinned d3-path 3.1.0 source/exports/tests/manifest. Executed 14 exploratory sequences
and two invalid-digit cases against the pinned upstream source; its hash and findings
are in the plan. Temporary downloads are not a retained parity corpus; WP-P01 owns it.
The two commands below each passed **1 test, 0 failures**:

```sh
mise exec -- cargo test -p chart-core --test contracts numeric_paths_preserve_segments_and_reject_invalid_subpath_order --locked
mise exec -- cargo test -p chart-export --test publication plain_text_xml_escaping_curves_and_empty_clips_are_supported --locked
```

These validate existing behavior, not D3 differential parity or visual fidelity.
No Rust implementation, dependency, baseline or wire version changed in this assignment.
No new path fixtures, native/export image inspection, binding parity, Linux execution
or performance evidence was produced. All PTH requirements and G-PATH remain open.
Next action within the path assignment: **WP-P01 — Contract and reference corpus**.

Documentation checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace/dependency isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (retains existing Markdown
hard-break spaces). These checks do not execute the declared target platforms.
An inline `python3 -` consistency check also passed six PTH requirement rows, six
FIX-P fixtures, four path package rows, gate/traceability links, 12 acyclic path/shape
packages and path-plan whitespace/anchors. Result: uncommitted documentation, with
implementation and all path acceptance gates still open.

| Path package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-P01 — Contract and reference corpus | COMPLETE | Core | ADR-015; pinned 86-sequence corpus and request/result DTO | PTH-01–06, ARC-04, QLT-02 | Source/license hashes, independent expectations and standalone consumers retained; G-PATH remains open. |
| WP-P02 — Checked builder and complete geometry | COMPLETE | Core | Seven focused path tests; actual three-host traces | PTH-01/02/03/05 | All operations, owned numeric results, atomic failures, independent geometry and explicit budgets; renderer gate remains P04. |
| WP-P03 — Shared SVG path output | COMPLETE | Core/export | FIX-P04 sequence/precision checks; actual representative SVG | PTH-01/04 | Shared formatter; analytic retained arcs; legacy path output defaults preserved. |
| WP-P04 — Renderers, portable APIs and acceptance | COMPLETE | Core/native/export/bindings | [Path acceptance](evidence/phase-2-paths-2026-09-08.md), retained hashes/artifacts | PTH-01–06 | FIX-P01–06 actual three-host proof, native/publication inspection and macOS/Linux checks pass. |


## D3 color parity planning handoff

The [color review and plan](impl_plans/d3-color-parity-plan.md) is complete as
uncommitted documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`.
Current byte colors/palettes do **not** establish d3-color parity. Specification 0.3.0
now also requires COL-01–06, CLR-01–05, FIX-C01 and G-COLOR. Historical WP-11/13/14
and G2 evidence retains its original scope. Active WP-15 and concurrent scale/shape/
axis/chromatic changes are preserved. CLR identifiers and FIX-C01 avoid CP/FIX-21
catalog ownership collisions. SP-04 requires CLR-03's color kernels; CLR-05 requires
SP-04 and WP-20, without depending on SP-07 or CP-05. WP-21/22 require CLR-05.

Evidence: 7 September 2026; `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026). Official d3-color documentation and retrieved
source compared with live core, themes, grammar, wire, host and export code. Target
module 3.1.0; tagged Lab source fetch failed, so main-branch Lab was inspected and
CLR-01 must verify it against the pinned tarball. No D3 oracle was executed.
`mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`:
**1 passed, 0 failed, 3 filtered out**, proving only the existing palette subset.
`mise exec -- python3 scripts/check_repository.py` passed workspace edges,
host isolation and local Markdown file links (graph inspection, not target execution).
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed, preserving the
existing Markdown hard-break convention. Inline `python3 -` consistency checks passed:
six COL definitions/traceability rows, five CLR plan/ledger packages, FIX-C01/G-COLOR,
WP-21/22 dependencies, shared interpolation ownership and 61 acyclic package nodes.
New color-plan whitespace passed separately. Concurrent interpolation planning was
also reconciled: WP-IP04 consumes CLR-03, and SP-04 consumes WP-IP04. No implementation, fixtures,
dependency or wire version changed. No fresh color binding runtime, native/export
artifact inspection, Linux or performance proof was produced. All COL requirements
remain open. Next within this plan: **CLR-01 — contract and reference oracle**.

| Color package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CLR-01 — Contract and reference oracle | COMPLETE | Core | 351-case pinned oracle, deterministic regeneration; ADR-016 | COL-01–06, ARC-04, BND-01, QLT-02 | WP-14 prerequisite met; pin complete source/oracle and freeze typed/wire/paint contracts. |
| CLR-02 — RGB/HSL, parsing and common operations | COMPLETE | Core | Actual Rust/Python/WASM FIX-C01 and type proofs; see foundation report | COL-01–04 | Requires CLR-01. |
| CLR-03 — Lab, HCL/LCh and Cubehelix | COMPLETE | Core | Full conversion/method corpus and Linux regression pass | COL-02–04 | Requires CLR-02; supplies SP-04 color kernels. |
| CLR-04 — Authoring, portable operations and paint integration | COMPLETE | Core/hosts | [Qualified paint snapshot](evidence/phase-2-paint-2026-09-09.md): every input, migration, actual hosts/native/publication, retained live updates | COL-05, BND-01/03/04, THM-01/02/03, SCN-03 | CLR-05 awaits SP-04. |
| CLR-05 — Integrated parity acceptance | COMPLETE | Core/hosts/native/export | [Integrated evidence](evidence/phase-2-color-acceptance-2026-09-09.md) | COL-01–06, QLT-02/03/04, SCN-04 | G-COLOR passes for declared typed snapshot; release gates remain separate. |

## D3 scale-chromatic parity planning handoff

The [chromatic review and plan](impl_plans/d3-scale-chromatic-parity-plan.md) is complete
as documentation. Starting revision: `1cb955740c2dad2607b0a2330201125294cab5d0`; outcome:
uncommitted documentation, including specification/implementation/migration version
0.3.0, CHR-01–06, CP-01–05, FIX-21 and G-CHROMATIC. The implementation has raw palettes
and linear RGB interpolation, **not full scale-chromatic parity**. All six CHR requirements
remain open. The previous scale plan's catalog exclusion is replaced with explicit
chromatic ownership; SP-04 retains shared RGB/Cubehelix and scale algorithms. No source,
fixture, dependency or wire version was changed by this planning assignment. Existing
WP-15 and shape/scale/axis changes were preserved. Historical G2 acceptance is unchanged.

Evidence: 7 September 2026, `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
configured Rust 1.97.1. Official docs and pinned d3-scale-chromatic 3.1.0 source were
compared with live core color, compiler, legend, theme and portable host paths.
Read-only Python/urllib source inventory assertions passed: **76 exports, 38 schemes,
38 interpolators, 218 discrete arrays**, every export name present in the plan.
Source retrieval succeeded using network-enabled read-only execution after sandbox DNS
failure. This inspected source; it did not execute D3 or generate a numerical oracle.

Checks run:

- `mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`: **1 passed**, 0 failed, 3 filtered out. Existing custom palette behavior only.
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/host isolation and local Markdown file links; platform graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving existing Markdown hard breaks.
- Inline Python documentation assertions: six normative CHR IDs and traceability rows, five CP plan/ledger packages, aligned 0.3.0 document headers and release dependencies checked.

No fresh chromatic differential, Python/WASM execution, native/export inspection,
Linux or performance evidence was produced. Estimated incremental effort is 9–16
engineer-days, excluding shared SP work; re-estimate after CP-01. Next within this
assignment: **CP-01**, reference oracle/compatibility contract, followed by CP-02's exact
catalog. CP-03 requires SP-04, and CP-05 requires SP-07/WP-20 before WP-21/22.

| Chromatic package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CP-01 — Reference contract and oracle | COMPLETE | Core/reference | [Entry](evidence/phase-2-chromatic-entry-2026-09-09.md) | CHR-01/03/05/06, ARC-04 | All 76 exports and reference provenance pinned. |
| CP-02 — Exact discrete catalog | COMPLETE | Core | [Foundation](evidence/phase-2-chromatic-foundations-2026-09-09.md) | CHR-01/02 | All 218 actual arrays exact. |
| CP-03 — Complete interpolator catalog | COMPLETE | Core | [Foundation](evidence/phase-2-chromatic-foundations-2026-09-09.md) | CHR-01/03 | All 38 ramps and 160,666 sampled rows. |
| CP-04 — Chart, guide and portable integration | COMPLETE | Core/hosts | [Integration](evidence/phase-2-chromatic-integration-2026-09-09.md) | CHR-04/05, SCL-03, THM-02, BND-01/03/04 | v6 metadata, actual hosts, composition/themes and exceptional normalization pass. |
| CP-05 — Integrated certification | COMPLETE | Core/native/export/hosts | [G-CHROMATIC](evidence/phase-2-chromatic-integration-2026-09-09.md) | CHR-01–06, QLT-02/03/04 | Qualified snapshot; WP-21/22 release rechecks remain. |

## D3 hierarchy parity planning handoff

The [hierarchy review and delivery plan](impl_plans/d3-hierarchy-parity-plan.md) is
complete as documentation. **Implementation parity is absent**: HIR-01–06 have no
hierarchy engine/layout implementations; HIR-07/08 integrated runtime, renderer and
performance evidence remains unverified. Specification 0.3.0 now requires HIR-01–08,
FIX-H01-A–H and G-HIERARCHY. WP-H08 precedes WP-21/22 acceptance. Historical G2 and
completed Cartesian packages retain their original scope.

Review context: 7 September 2026, Darwin arm64,
`/Users/jeickmeier/Projects/finstack-chart`; started at `1cb9557` with live action and
parity-plan edits. HEAD advanced independently to `127fe2d` during review; this handoff
is uncommitted documentation on that working tree. Source and other planning changes
were preserved. The review compared the official D3 hierarchy documentation and pinned
3.1.2 exports/construction/stratify/treemap/packing source with core, grammar, layout,
scene, provenance and portable interfaces. It inventories all 16 exports, including
`Node`, all methods/controls, reference adaptations and discriminating planned cases.

No hierarchy code, wire version, dependencies or baselines changed. No Rust feature
tests, D3 differential runner, actual binding proofs, native/export visual inspection,
Linux execution or benchmarks ran for this task. Checks below validate documentation
only. G-HIERARCHY and all hierarchy implementation packages remain open.
Next within this assignment: **WP-H01**, then WP-H02; existing action and other parity
assignments retain their own next steps.

Planning validation passed:

- `mise exec -- python3 scripts/check_repository.py`: workspace edges, host isolation,
  pinned GPUI identity, optional Kit and local Markdown file links. Target graph checks
  are not target runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: no whitespace errors,
  preserving existing Markdown hard breaks.
- Inline `python3 -` consistency check: eight normative HIR IDs, eight package rows in
  the hierarchy plan and ledger, eight fixture subsets, all 16 reference export names,
  main-plan traceability, WP-21/22 dependencies, removed hierarchy deferrals, local
  hierarchy heading links and new-plan whitespace passed.

| Hierarchy package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-H01 — Contract and reference harness | COMPLETE | Core contract/reference | [Entry evidence](evidence/phase-2-hierarchy-entry-2026-09-09.md): 16 exports, all methods/defaults, 441 reproducible cases, ADR-022 | HIR-01–08, ARC-04, BND-01, QLT-02 | Seed harness only; production/runtime certification remains H02–08. |
| WP-H02 — Topology, stratification and operations | COMPLETE | Core hierarchy | [Topology evidence](evidence/phase-2-hierarchy-topology-2026-09-10.md): 31 pinned cases, deep/bounded native inputs and 3 passing contracts | HIR-01/02/07 | Standalone core scope; registered/host/chart acceptance remains H07/H08. |
| WP-H03 — Tidy tree and cluster | COMPLETE | Core hierarchy | [Layout evidence](evidence/phase-2-hierarchy-layouts-2026-09-10.md): 340 reference layouts and mode/callback invariants | HIR-03 | H07/H08 retain radial/presentation/host obligations. |
| WP-H04 — Partition | COMPLETE | Core hierarchy | [Layout evidence](evidence/phase-2-hierarchy-layouts-2026-09-10.md): 42 reference partitions and own-value slack | HIR-04 | H07/H08 retain icicle/sunburst/host obligations. |
| WP-H05 — Treemap and tilers | COMPLETE | Core hierarchy | [Kernel evidence](evidence/phase-2-hierarchy-kernels-2026-09-10.md): 217 cases, retained histories and six topology reset states | HIR-06 | Standalone/native callback scope; registered/host/chart acceptance remains H07/H08. |
| WP-H06 — Packing and helpers | COMPLETE | Core hierarchy | [Kernel evidence](evidence/phase-2-hierarchy-kernels-2026-09-10.md): 191 cases plus containment/non-overlap/budget checks | HIR-05 | Standalone/native callback scope; integration and measured release evidence remain. |
| WP-H07 — Grammar, portable API and presentation | COMPLETE | Core/hosts/native/export | [Final integration evidence](evidence/phase-2-hierarchy-integration-2026-09-10.md): nine chart contracts, external registered operations, typed actual hosts, native and SVG/PDF/PNG inspection | HIR-07, BND-01/03/04, SCN-04 | Complete within assignment; preserve source-qualified evidence for release rechecks. |
| WP-H08 — Integrated parity acceptance | COMPLETE | Shared parity acceptance | [65-row catalog](evidence/phase-2-hierarchy-integration/verdict-catalog.md), 898 oracle cases plus 27 controls per host, replay/disposal/resources, 474 macOS/455 Linux tests and repository checks | HIR-01–08, QLT-02/03/04 | G-HIERARCHY passes; benchmark workloads and high-fanout query follow-up handed to WP-22. |

## D3 scale parity planning handoff

The [scale review and implementation plan](impl_plans/d3-scale-parity-plan.md) is
complete as documentation at `1cb9557` plus uncommitted documentation changes.
The implementation does **not** have D3 scale feature parity. Specification 0.2.0
now requires SCL-06–08/FIX-20 and G-SCALE; SP-01–07 own the missing families,
behavioral compatibility, oracle and end-to-end evidence. WP-06/11 and G2 retain
their original 0.1.0 acceptance. Shape/axis plans and active WP-15 edits are preserved.
Scale tick/format algorithms have one owner; WP-AX02 consumes SP-03/05/06.

Review evidence: 7 September 2026, working directory
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1.
Official D3 documentation and d3-scale 4.0.2 source compared with live scale,
axis/formatting and portable contracts. `mise exec -- cargo test -p chart-core
--test scales --test full_scales --locked` passed **13 tests, 0 failed**.
These verify existing contracts, not D3 parity. No fresh D3 differential, Python/WASM,
native/export, Linux or performance evidence was produced. No scale code, fixtures,
dependencies or wire version changed. Next within the scale assignment: **SP-01**.

Planning checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace edges, host isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (preserves the existing
Markdown hard-break convention). Graph validation is not platform runtime execution.

| Scale package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| SP-01 — Compatibility contract and oracle | COMPLETE | Core/reference | [Reference and gap report](evidence/phase-2-scale-reference-2026-09-09.md): 26 factories, 461 cases, 200 formats, four zones, complete method dispositions | SCL-06/07/08, ARC-04, BND-01, QLT-02 | Measurement only; G-SCALE OPEN. Proceed SP-02. |
| SP-02 — Continuous mapping and numeric families | COMPLETE | Core | [Numeric qualification](evidence/phase-2-numeric-scales-2026-09-09.md): 161 configurations, explicit nice, named axes/navigation, v5, 43 macOS and 20 Linux tests | SCL-01/02/03/06/07, DAT-05 | D3 ticks/formatters remain SP-05; host qualification SP-07. |
| SP-03 — Ordinal, band and point | COMPLETE | Core | [Categorical qualification](evidence/phase-2-categorical-scales-2026-09-09.md): all 130 cases, exact keys, named axes/dodge/update snapshots, 55 macOS and 30 Linux tests; repository checks pass | SCL-01/06/07, DAT-02 | Standalone host/native/performance qualification remains SP-07. |
| SP-04 — Interpolation, distribution and color | COMPLETE | Core | [Distribution qualification](evidence/phase-2-distribution-scales-2026-09-09.md): 160 pinned configurations, defaults, numeric styles, interval/midpoint guides, post-stat correction/zoom and one paint boundary; 66 macOS and 36 Linux tests; checks pass | SCL-03/06/07, GRA-03, BND-01 | Registered interpolation remains WP-IP06; standalone host/native/performance qualification SP-07/WP-21/22. |
| SP-05 — Numeric ticks, nice and formatting | COMPLETE | Core | [Numeric format qualification](evidence/phase-2-numeric-format-2026-09-09.md): 269 exact tick/label configurations, 200 standalone format cases, expanded 11,176 labels, nice/locale/empty-label publication; 99 macOS and 33 Linux tests; checks pass | SCL-07, LAY-01/02, THM-03 | Standalone host qualification remains SP-07; shared API consumed by WP-AX02. |
| SP-06 — UTC and explicit local calendars | COMPLETE | Core/hosts | [Calendar qualification](evidence/phase-2-calendar-scales-2026-09-09.md): 2,750 filtered intervals, 16,632 exact custom labels, 160 mapping configurations; same-revision Python/WASM and byte-identical DST SVGs; 52 macOS/50 Linux tests and repository checks pass | SCL-04/06/07, DAT-05, BND-01 | Shared API consumed by WP-AX02; public facade/type/native/performance integration remains SP-07. |
| SP-07 — Integrated parity proof | COMPLETE | Core/hosts/native/export | [Integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md) | SCL-01–08, BND-01/03/04, SCN-04, QLT-02/03/04 | G-SCALE passes for declared typed snapshot; release gates remain separate. |

## D3 shape parity planning handoff

The owner-requested [shape parity review and plan](impl_plans/d3-shape-parity-plan.md)
is complete as documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`
plus uncommitted documentation changes. Specification/implementation/migration documents
now use version 0.2.0 and require SHP-01–10, FIX-S01–09 and G-SHAPE before G4.
Existing WP-10/11/14 and G2 passes retain their original Cartesian scope. No D3 parity
implementation or runtime pass is claimed; all SHP requirements remain open.

Evidence: 7 September 2026, Darwin arm64, working directory
`/Users/jeickmeier/Projects/finstack-chart`; official D3 documentation and pinned
d3-shape 3.2.0 exports/source compared with the live implementation. Exact source
locations and missing contracts are in the linked plan.
`mise exec -- python3 scripts/check_repository.py` passed workspace/dependency isolation
and local Markdown file links. Inline `python3 -` inventory/dependency checks passed:
10 SHP IDs, nine supplemental fixtures, eight shape ledger packages and 31 acyclic
original-plus-shape package nodes at the time of this check.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md`
passed; the default whitespace check flags these documents' intentional Markdown
hard-break spaces. The new shape plan has no trailing whitespace.
The review ran no Rust feature tests, D3 oracle, native visuals or binding
runtime comparisons. Concurrent WP-15 source and status edits were preserved.
Next within the shape assignment: WP-S01 reference fixtures and shared path foundation;
the ongoing WP-15 assignment below continues independently.

## Current handoff

**The owner's original WP-11 through WP-23 assignment is complete, with a separate
commit for every package. Expanded acceptance remains OPEN.** The owner explicitly
waived the 30-minute test; its interrupted trace and numerical failure remain retained.
No package was published and no distribution license was inferred.

WP-23 supplies the [developer/release guide](release-guide.md),
[65-requirement evidence index](release-evidence.md), changelog, package/source policy,
and deterministic unpublished local source-candidate tool. The index explicitly lists
75 additional identifiers from concurrent planning as OPEN and does not certify later
clauses added to original IDs. See [WP-23 evidence](evidence/wp-23-completion-2026-09-07.md).

Final runtime evidence is **219 macOS tests, 213 Linux headless tests**, required
fmt/check/lint/docs/WASM checks and actual Rust/Python/Node WASM fixtures (36 cases,
23 actions, 47 input steps, 70 stream steps, three density cases, 40 live-export steps).
The extracted committed runtime archive passes offline repository/isolation/link checks
and fresh headless all-target compilation. Repeated archives have identical SHA-256.
The final WP-23 archive/manifest is generated under `artifacts/local-release` after commit.

[WP-22](evidence/wp-22-completion-2026-09-07.md) records ten-line p95 hover/frame work
0.092666/11.689291 ms and visible short-stream actual ingest-to-display p95 165.759375 ms.
Exact accounting and concurrent publication drain/release pass. Heavy-dashboard timings,
failed normal-window display traces, allocator high-water and the duration/visibility
limits remain explicit. These results do not certify expanded workloads or current G4.

**Next owner work:** select the separately planned parity/primary-authoring lane and its
acceptance packages. Preserve the open OS accessibility/platform/distribution boundaries,
license metadata and dependency advisories. There is no active implementation package
left in this original assignment; concurrent owner edits remain separate.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap accepted; WP-03 capability follow-up complete; dependency maintenance/release disposition remain WP-08/23. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `435e127` | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | DONE | Unassigned | `3a86189` | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | [Completion evidence](evidence/wp-03-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md); [ADR-008](adr/008-benchmark-protocol.md) | Actual proof artifacts, native lifecycle/hooks and starting profile inspected; public integration/full fixtures remain later packages. |
| WP-04 — Immutable data, schemas and transactions | DONE | Unassigned | `d0a6c48` | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | [Completion evidence](evidence/wp-04-completion-2026-09-06.md); [ADR-004](adr/004-immutable-data-and-transactions.md) | Data portions accepted; full transform/selection behavior, queued ingestion/time retention and binding runtimes remain later packages. |
| WP-05 — Grammar compiler and minimal prepared scene | DONE | Unassigned | `3cf1b33` | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | [Completion evidence](evidence/wp-05-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Known data-space geometry/semantics accepted; scales/layout and destination scene projection are WP-06; complete grammar/extensions remain later packages. |
| WP-06 — Foundational scales, ticks and layout | DONE | Unassigned | `f8657fb` | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | [Completion evidence](evidence/wp-06-completion-2026-09-06.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Foundational scale/scene/text-layout contracts accepted; actual native/export consumers are WP-07/08, full families/typography/shared layout remain WP-11–13. |
| WP-07 — Working standalone GPUI vertical slice | DONE | Unassigned | `fac148a` | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | [Completion evidence](evidence/wp-07-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Standalone line/point/bar, native text, presented-snapshot inspection, resize and disposal accepted for FIX-01/07/18 subsets; complete interaction/indexing/accessibility remain WP-15–21. |
| WP-08 — Headless export and snapshot foundation | DONE | Unassigned | `fac148a` | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | [Completion evidence](evidence/wp-08-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Basic headless formats, explicit resources and minimal FIX-13/14 accepted; full composition/live exports remain WP-13/20. |
| WP-09 — Portable schema and executable binding proofs | DONE | Unassigned | `c5ec829` | BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01 | [Completion evidence](evidence/wp-09-completion-2026-09-06.md); [ADR-006](adr/006-portable-specification-and-binding-proofs.md) | Version 1 subset and actual Rust/Python/WASM FIX-15/16 accepted; extend builtin coverage through WP-10–14 and full parity at WP-21. |
| WP-10 — Complete statistical and position semantics | DONE | Unassigned | `c5ec829` | GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02 | [Completion evidence](evidence/wp-10-completion-2026-09-06.md); [contract](statistics-contract.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Built-in stats/positions and FIX-02–05 accepted; exact full-recompute fallback declared. Extend families in WP-11 and specialized streaming in WP-18. |
| WP-11 — Required scale and geometry families | DONE | Unassigned | `a6fb2ea` | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | [Completion evidence](evidence/wp-11-completion-2026-09-07.md); [contract](scale-geometry-contract.md) | Required families and FIX-01/03/07 scope accepted through native/export/actual bindings; proceed to shared layout in WP-12. |
| WP-12 — Facets, guides and shared layout | DONE | Unassigned | `63dbcc2` | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | [Completion evidence](evidence/wp-12-completion-2026-09-07.md); [contract](facet-layout-contract.md) | FIX-06 and facet/shared-layout scope accepted through core/native/export/actual bindings; full typography and composition remain WP-13. |
| WP-13 — Full themes and publication composition | DONE | Unassigned | `f6c41c1` | THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03 | [Completion evidence](evidence/wp-13-completion-2026-09-07.md); [contract](theme-typography-composition-contract.md) | FIX-12/13 accepted through actual core/native/export/bindings; cumulative G2 remains WP-14. |
| WP-14 — Extension contracts and alpha API | DONE | Unassigned | `1cb9557` | SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05 | [Completion evidence](evidence/wp-14-completion-2026-09-07.md); [contract](extension-contract.md); [alpha matrix](alpha-api.md) | FIX-17 and cumulative G2 passed; full reducer/interaction begins WP-15. |
| WP-15 — Complete action reducer and state ownership | DONE | Unassigned | `127fe2d` | INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01 | [Completion evidence](evidence/wp-15-completion-2026-09-07.md); [contract](state-action-contract.md); [ADR-007](adr/007-actions-gestures-and-controlled-state.md) | Deterministic action/controlled/gesture/history/lifetime scope accepted through actual native/export/Python/WASM. Input producers and full G3 remain WP-16–20. |
| WP-16 — Hit testing, navigation and selection | DONE | Unassigned | `4148793` | INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05 | [Completion evidence](evidence/wp-16-completion-2026-09-07.md); [contract](interaction-contract.md) | Assigned indexed inspection/navigation/selection acceptance passes; FIX-09/10 remaining portions and G3 stay with WP-17–20. |
| WP-17 — Linked views, editable annotations and host controls | DONE | Unassigned | `6779e44` | INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06 | [Completion evidence](evidence/wp-17-completion-2026-09-07.md); [contract](host-tools-contract.md) | Original linked/editing/host acceptance passes; native accessibility limitations recorded. Full G3 remains open. |
| WP-18 — Streaming retention and incremental computation | DONE | Unassigned | `30ca2e8` | DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01 | [Completion evidence](evidence/wp-18-completion-2026-09-07.md); [contract](streaming-contract.md) | Original queue/retention/incremental/follow acceptance passes; 70-step three-host replay and native lifecycle inspected. Sustained PERF and G3 remain open. |
| WP-19 — Bounded scheduling, caches and dense representation | DONE | Unassigned | `e57246d` | STM-04, STM-05, SCN-04, GPU-02, QLT-04 | [Completion evidence](evidence/wp-19-completion-2026-09-07.md); [contract](scheduling-density-contract.md) | Original bounded-worker/cache/density acceptance passes; actual four-chart progress and three-host dense proofs. Preliminary PERF identifies index/memory bottlenecks; intermittent native redraw question retained. |
| WP-20 — Coherent exports during live interaction | DONE | Unassigned | `9d86ef7` | EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04 | [Completion evidence](evidence/wp-20-completion-2026-09-07.md); [contract](live-export-contract.md) | Original coherent capture/bounded lifetime acceptance passes; actual 40-step three-host replay and finite native exports during 400 atomic commits. Sustained PERF and native hardening remain open. |
| WP-21 — Correctness, fidelity and supported-platform hardening | DONE (original scope) | Unassigned | `4c099ee` | SCP-03, QLT-01/02/03, GPU-02/03, BND-03/04, FIX-01–18 | [Completion evidence](evidence/wp-21-completion-2026-09-07.md); [ADR-009](adr/009-supported-platform-and-accessibility.md) | Original acceptance passes including fixed native frozen resize/redraw and actual macOS/Linux/headless bindings. Expanded parity/authoring acceptance remains open and requires the separately listed packages/gates. |
| WP-22 — Measured performance and sustained-load release gate | DONE (original scope; owner duration waiver) | Unassigned | `c58f5b2` | STM-01/03/04/05, EXP-03, QLT-04, PERF-01–05 | [Completion evidence](evidence/wp-22-completion-2026-09-07.md); [ADR-008](adr/008-benchmark-protocol.md) | Visible short budgets and exact accounting pass; interrupted/occluded failures and memory limits retained. Thirty-minute duration explicitly owner-waived. Expanded workloads and current G4 stay open. |
| WP-23 — Production documentation and release readiness | DONE (original local candidate scope) | Unassigned | Included in this completion commit | SCP-01/02/03, ARC-04, BND-01, QLT-05/06 | [Completion evidence](evidence/wp-23-completion-2026-09-07.md); [release index](release-evidence.md); [guide](release-guide.md) | All original package handoffs complete; source candidate remains unpublished. Expanded parity/authoring and current production G4 remain OPEN; duration waiver, failed evidence, platform/accessibility/metadata limits are explicit. |

## Supplemental shape work packages

The [shape plan](impl_plans/d3-shape-parity-plan.md#work-packages-and-dependency-order)
owns deliverables and acceptance criteria; planning approval is not implementation evidence.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-S01 — Shape contract, oracle and path foundation | COMPLETE | Core/reference/native/export/hosts | [FIX-S01 evidence](evidence/phase-2-shape-foundation-2026-09-09.md) | SHP-01, SHP-08, SHP-10 | 63 exports, 333 contexts, 42 layouts; actual hosts and inspected destinations | Proceed WP-S02; generator families remain open. |
| WP-S02 — Cartesian generators and complete curves | COMPLETE | Core/hosts/native/export | [FIX-S02/03 evidence](evidence/phase-2-shape-cartesian-2026-09-09.md) | SHP-02, SHP-03, SHP-09 | 20 curves, 829 cases; 369 macOS tests; actual hosts, 64 updates each and inspected destinations | Curved dash styling subsequently accepted in WP-S08. |
| WP-S03 — Arc geometry and pie layout | COMPLETE | Core/hosts | WP-S01 accepted; ADR-020 | SHP-04, SHP-09 | [620 arc/192 pie Rust cases, 144 built-in host pies, Linux/WASM interactions, 64 updates and publication](evidence/phase-2-shape-arc-2026-09-09.md); implementation complete | Fresh macOS Python, inspected native output and 393-test workspace/repository checks pass; proceed WP-S04. |
| WP-S04 — Radial generators and links | COMPLETE | Core/hosts/native/export | WP-S02 and WP-S03 accepted; ADR-020 | SHP-05, SHP-09 | [40 point/697 standalone and chart paths, 760 updates per host, exact inspected publication](evidence/phase-2-shape-radial-2026-09-09.md) | Fresh macOS/Linux Python and WASM budget regressions pass; figure-wide projection preflight includes facets/insets. Native inspected; 339 core + 54 export Linux tests, 404 macOS workspace tests/doctests and repository checks pass at recorded snapshots. Proceed WP-S07. |
| WP-S05 — Complete symbol encoding | COMPLETE | Core/hosts | WP-S01 accepted; ADR-020 | SHP-06, SHP-09 | [13 types, 156 fixtures per host, 104 updates per host, mapped guides and exact inspected publication](evidence/phase-2-shape-symbol-2026-09-09.md); Linux lint/docs pass | Fresh macOS Python, inspected native output and complete workspace/repository qualification pass. |
| WP-S06 — Complete stack layouts | COMPLETE | Core/hosts/native/export | WP-S01 and WP-10 accepted; ADR-020 | SHP-07, SHP-09 | [435 numerical cases per host, 1,620 tidy comparisons, 480 updates per host, exact inspected three-host publication and 385 Linux tests/doctests](evidence/phase-2-shape-stack-2026-09-09.md) | Fresh macOS Python, 480 updates, inspected native output and complete workspace/repository qualification pass. |
| WP-S07 — Custom protocols and public portability | COMPLETE | Core/hosts/native/export | WP-S02–06 accepted | SHP-01, SHP-08, SHP-09 | [Five registered protocols, chart/legend integration, wire-v9, 72 updates per host and exact inspected publication](evidence/phase-2-shape-custom-2026-09-09.md) | Native and all three publication themes inspected; actual Python/WASM, strict types and repository checks pass. Full regression follow-up recorded with the evidence; WP-S08 subsequently accepted. |
| WP-S08 — Integrated parity acceptance | COMPLETE | Core/hosts/native/export | WP-S07, WP-16, WP-18, WP-20 accepted | SHP-09, SHP-10 | [63 exports/220 methods accepted; retained curved dashes and complete cross-family qualification](evidence/phase-2-shape-acceptance-2026-09-09.md) | 420 macOS / 419 Linux tests-doctests, 1,544 updates per actual host, native and nine publication images inspected; G-SHAPE passes. Expanded WP-21/22/23 remain open. |

## Cumulative gates

### Required D3 axis packages

The [axis parity plan](impl_plans/d3-axis-parity-plan.md) adds AXIS-01–07/FIX-19 to
specification 0.2.0. Historical WP-06/11–14 and G2 evidence retains its original scope.
The axis review/planning assignment is complete; implementation parity remains open.
WP-21 additionally requires WP-AX06, and WP-22/23 include axis performance/release proof.

| Package | State | Owner | Revision | Requirements | Evidence / next action |
| --- | --- | --- | --- | --- | --- |
| WP-AX01 — Guide contract and reference harness | COMPLETE | Core/hosts | WP-14 accepted; reference entry | AXIS-01, AXIS-07 | [372 actual-browser reference cases and complete 4-factory/10-method inventory repeat exactly](evidence/phase-2-axis-entry-2026-09-09.md). Identity, shared-scale guide resolution, primary builders/edits/name maps and version-8 migration implemented; five guide tests plus 39 existing core tests pass on Linux; actual Linux Python/WASM prove shared placement, named navigation, stable scale replacement and retained outputs. Strict positive types and macOS repository/workspace checks pass. Dedicated macOS Python and inspected native guide identity checks pass. [Registered provider acceptance](evidence/phase-2-axis-provider-2026-09-09.md) passes 21 focused macOS/Linux tests, actual macOS/Linux Python and WASM, strict declarations and byte-identical publication output plus inspected native output. [Integrated acceptance](evidence/phase-2-axis-ticks-2026-09-09.md) completes the explicit profile entry and retains final regression/validation limits. Complete geometry remains AX03. |
| WP-AX02 — Tick selection and formatting | COMPLETE | Core/hosts | Working tree over `51f2eda`; retained source hashes | AXIS-02, AXIS-03 | [Integrated acceptance](evidence/phase-2-axis-ticks-2026-09-09.md): 372 reference cases / 376 states per actual Rust/Python/WASM host, independent reset/selection/formatting, exact timestamps, registered callbacks, strict types, byte-identical publication and inspected native/SVG/PDF/PNG. 430 macOS and 430 Linux tests/doctests; final focused tests pass. Stop before AX03 as requested; aggregate-check limits remain explicit. |
| WP-AX03 — Axis geometry and bounded layout | COMPLETE | Core/hosts/native/export | WP-AX02 accepted | AXIS-04 | [Geometry acceptance](evidence/phase-2-axis-geometry-2026-09-09.md): 437 macOS tests, 372 pinned cases/376 states in Rust/Python/WASM; signed geometry, policies, facets and native/publication inspection. |
| WP-AX04 — Styling and publication components | COMPLETE | Core/hosts/native/export | WP-AX03 accepted | AXIS-05 | [Component acceptance](evidence/phase-2-axis-components-2026-09-09.md): v14 styles/roles, per-tick typography, 443 macOS tests, 26 matching three-host artifacts and inspected native/text-outline/high-DPI publication. |
| WP-AX05 — Axis updates and transitions | COMPLETE | Core/native/hosts/export | [Timed/native/capture proof](evidence/phase-2-axis-transitions-2026-09-09.md): 125 reference samples, 75 host artifacts, 449 macOS/448 Linux tests | AXIS-06 | Native clock/interruption/reduced motion/disposal and exact displayed capture pass. |
| WP-AX06 — Parity certification and documentation | COMPLETE | Integrated acceptance | [Capability matrix](evidence/phase-2-axis-certification-2026-09-09.md), 449 macOS/448 Linux tests, full repository check | AXIS-01–07 | G-AXIS passes for declared supported surface; no global release/FPS claim. |

### Gate results

| Gate | State | Evidence required next |
| --- | --- | --- |
| G0 | PASSED — architecture/capability scope | WP-01/02/03 evidence and ADRs establish the initial macOS route and starting protocol; this does not pass full requirements, FIX/PERF or release support. |
| G1 | PASSED — minimal portable core | WP-04–08 foundation/native/headless evidence plus WP-09 actual Python/WASM FIX-15/16 runtime comparison; this does not certify full grammar or production host/distribution products. |
| G2 | PASSED — Cartesian/publication alpha | [Alpha matrix](alpha-api.md) maps complete grammar/facets/themes/publication/extensions and actual portable evidence. G3/G4 retain their remaining scope. |
| G3 | PASSED (original interactive streaming scope) | WP-15–20 interaction/streaming/export plus WP-21 frozen-resize and native redraw regression fixes; actual native and Rust/Python/WASM evidence. Sustained PERF and expanded parity remain separate. |
| G-PATH | PASSED | PTH-01–06/FIX-P01–06; [WP-P04 source snapshot and evidence](evidence/phase-2-paths-2026-09-08.md). Shape and performance qualification remain separate. |
| G-AXIS | PASSED | AXIS-01–07/FIX-19; pinned reference matrix, actual bindings, native/publication inspection and transition evidence. Required before WP-21 and G4. |
| G-SHAPE | PASSED for finite typed snapshot | SHP-01–10/FIX-S01–09 through WP-S08; [integrated evidence](evidence/phase-2-shape-acceptance-2026-09-09.md). Complete per-item verdicts, actual hosts and inspected destinations; WP-21/22/23 remain open. |
| G-SCALE | PASSED for declared typed snapshot | SCL-01–08/FIX-20 through SP-07; [integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md). WP-21/22 release rechecks remain open. |
| G-CHROMATIC | PASSED for retained typed snapshot | CHR-01–06/FIX-21 through CP-05; [final evidence](evidence/phase-2-chromatic-integration-2026-09-09.md). Other Phase 2/release gates remain open. |
| G-COLOR | PASSED for declared typed snapshot | COL-01–06/FIX-C01 through CLR-05; [integrated evidence](evidence/phase-2-color-acceptance-2026-09-09.md). WP-21/22 release rechecks remain open. |
| G-INTERPOLATE | PASSED | ITP-01–08/FIX-I01; all 27 exports/configuration/result controls, shared consumers, actual Rust/Python/WASM and applicable inspected native/publication evidence. WP-IP07 precedes WP-21/22. |
| G-HIERARCHY | PASSED for declared finite typed surface | HIR-01–08/FIX-H01-A–H through WP-H08; [complete method/history/host/native/publication/resource evidence](evidence/phase-2-hierarchy-integration-2026-09-10.md). Global WP-21/22/23 and PERF gates remain separate. |
| G-GGPLOT | NOT PASSED | GG2-01–12 / FIX-GG00–19 and GG-19 complete reference/host/destination capability evidence. |
| G-PARITY | NOT PASSED | G3, all eight D3 gates and G-GGPLOT; Phase 2 integrated capabilities before final WP-21/22 acceptance. |
| G4 | OPEN — local candidate only | G-PARITY and all required FIX/PERF/platform/accessibility/documentation evidence through WP-21–23, including D3 and ggplot2 supplemental workloads. |

## Axis planning evidence — 7 September 2026

Outcome: reviewed current axis implementation against D3 and added the parity plan,
normative requirements, fixture coverage, package dependencies and open gate. No axis
implementation changed. Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`;
result is uncommitted documentation. Existing action/state/binding edits and concurrent
shape planning were preserved. Source findings and external references are retained in
[the plan](impl_plans/d3-axis-parity-plan.md#evidence-from-this-planning-review).

Working directory `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026).
`mise exec -- cargo test -p chart-core --test scales --test layout --locked` passed:
29 tests (9 scales, 20 layout), zero failed. Existing behavior passes; these are not
D3 differential tests. `mise exec -- python3 scripts/check_repository.py` passed workspace
boundaries, dependency isolation and local Markdown file links. The default whitespace
check flagged the documents' existing Markdown hard-break convention; verification with
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed. An inline `python3 -`
consistency check passed seven normative axis IDs, six package rows in each of the axis
plan/main plan/ledger, requirement traceability, G-AXIS, shared-scale prerequisites and
the new plan's whitespace. These are documentation checks, not feature certification.
No new D3 fixtures, runtime binding proofs, native/export inspections, Linux execution
or performance evidence were produced. AXIS-01–07 and G-AXIS remain open.
Next within this assignment's plan: WP-AX01; existing WP-15 work continues independently.

## Interpolation planning evidence — 7 September 2026

Outcome: documented the complete 27-export d3-interpolate inventory, source-backed gaps,
ITP-01–08/FIX-I01, seven implementation packages and G-INTERPOLATE. Reconciled ownership
with the concurrent color/scale/chromatic/axis plans and preserved other live edits.
Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; result is uncommitted
planning documentation. The [plan](impl_plans/d3-interpolate-parity-plan.md) retains
source links, retrieval limitations, compatibility decisions, estimates and intended
acceptance. No production implementation, dependency, schema or fixture was changed
by this task. ITP-01–08 and G-INTERPOLATE remain open; next action is WP-IP01.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --test scales --test full_scales --locked`:
  **13 passed, 0 failed** (9 foundational and 4 full-scale tests). These establish the
  existing subset only; they are not a D3 interpolation differential suite.
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries, host isolation and local Markdown file links; no platform runtime claim.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, retaining the
  existing Markdown hard-break convention.
- `mise exec -- python3 /tmp/chart-interpolate-plan-check.py`: passed 27 export names,
  eight normative/traceability IDs, seven plan/ledger package rows, eight fixture groups,
  WP-21/22 dependencies, new-plan whitespace and an acyclic 30-node cross-lane model.
  This temporary planning check is not a committed feature runner or CI gate.

No D3 runtime/oracle regeneration, actual Python/WASM interpolation proof, native/export
inspection, Linux execution or performance measurement was run. Available reference
source included the upstream main export index and pinned zoom implementation; some
pinned source requests and shell DNS failed. WP-IP01 must reconcile and lock release
sources before committing acceptance fixtures. No absent evidence is recorded as a pass.

## Public API usability review — 7 September 2026

Outcome: reviewed the live public Rust authoring, data, native/export, documentation
and proof-binding surfaces against the owner's ggplot2-like simplicity goal. The
[review](evidence/public-api-review-2026-09-07.md) records six ranked findings,
source locations, current/proposed boundaries and concrete developer-workflow
acceptance criteria. Current usability fails that goal; tested existing grammar
contracts pass their scoped checks. Proposed authoring and developer usability remain
unverified. Scope: SCP-01, DAT-01/02, GRA-01/07, THM-01, ARC-01/02, EXP-01/02,
BND-03/04, QLT-05 and GG2-02/03/07/09/10/11. Existing minimal bindings are not
reclassified as failed proof adapters merely because their authoring is low-level.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; reviewed existing
uncommitted work. Result: uncommitted review report and this ledger entry only;
implementation, defaults, fixtures and unrelated edits preserved. The root repository
instruction to record every review outcome governs this ledger update.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --doc --locked`: **4 passed**, zero failed
  (two runnable examples and two intended compile failures).
- `mise exec -- cargo test -p chart-core --test grammar --test facets --locked`:
  **29 passed**, zero failed (20 grammar, nine facets).
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links; target graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
  and `git -c core.whitespace=-blank-at-eol diff --no-index --check -- /dev/null docs/evidence/public-api-review-2026-09-07.md`:
  passed whitespace checks, retaining the existing Markdown hard-break convention.

No proposed API was implemented or compile-certified. No actual Python/WASM proof,
native/export inspection, differential reference suite, performance gate or user
study ran. GG2 and G4 remain open. Next action: prioritize a bounded plot/data/layer
authoring slice and define compiling everyday examples early in the existing parity
plan; semantic prerequisites and legacy defaults must be preserved.

## chart-export simplification — 16 September 2026

Owner-assigned `finstack-simplify` pass over `crates/chart-export/src` (read-only audit,
then four behaviour-preserving slices; the plan checkpoint was waived by the owner).
Scope: EXP-01/02 encoder parity, BND-03/04 binding shape, QLT-05. No wire field, serde
name, diagnostic code, output byte or fixture changed; diagnostic *messages* for the
PostScript/EMF alpha, mask and image rejections were unified to one device-parameterised
wording and the PicTeX budget message now names PicTeX.

Slices and revisions: (1) `5e82865` micro-duplication sweep — `Format::name` reused by
`SaveOptions::resolve`, `resource_error` helper, one `PathCommand` point visitor, shared
linear-gradient emission in SVG/PDF, impl blocks co-located, serde aliases replace
hand-parsed `basis`/`view`/`text`; (2) landed inside owner commit `dab6746` — new private
`devices.rs` with `BoundedString`/`BoundedBytes`, `AlphaPolicy` and one usvg leaf walker
shared by PostScript and EMF, single-page forwarders `encode::pdf`,
`devices_vector::postscript`, `svg::build_with_outlines` removed; (3) landed inside owner
commit `6ddefda` — public surface: `FigureSnapshot::capture_with_extensions`,
`Output::export/svg/pdf/png`, `host::Options::request` removed, `FigureSnapshot::export_pages`
is `pub(crate)` behind `FigurePages`, bindings/README/authoring guide updated;
(4) this revision — provenance `engines` string corrected (stale `skrifa /0.42.1` dropped)
and pinned to `Cargo.lock` by a unit test, `host::format` error text lists every format,
doc snippets bind `.bytes`. Net source delta across slices ≈ −190 lines in
`crates/chart-export/src` with two new unit tests.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:
after every slice `cargo fmt --all`, `cargo clippy -p chart-export --all-targets --locked
-- -D warnings`, `RUSTDOCFLAGS=-D warnings cargo doc -p chart-export --no-deps --locked`,
`cargo test -p chart-export --locked` (all suites pass; slice 3 additionally
`cargo clippy -p chart-python -p chart-wasm --all-targets -- -D warnings` and
`cargo check -p chart-gallery --examples`) and `cargo check -p chart-python -p chart-wasm
--locked`. Limitations: `mise run check` currently fails at `scripts/check_repository.py`
on the owner-committed removal of `benches/README.md` (`6e4bc28`) referenced from
`docs/impl_plans/package-infrastructure-plan.md`; the full workspace `mise run test` and
`bindings-proof` were not rerun for this pass. Open: audit F12 (`ExportOptions` ↔
`PublicationProfile` field duplication) and H4 (dev-only second `resvg 0.45.1` +
`svg2pdf` used solely by `examples/capability_export.rs`, which WP-03 evidence cites)
are recorded, not changed. Next action: owner decision on H4 and repair of the
`benches/README.md` link, then rerun `mise run check && mise run test`.

## Evidence updates

Before ending every task, including reviews and partial or blocked slices, update this
ledger with its outcome and evidence. Record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
