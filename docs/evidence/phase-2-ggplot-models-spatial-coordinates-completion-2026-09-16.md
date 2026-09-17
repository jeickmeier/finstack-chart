# GG10, GG11 and GG13 completion — 16 September 2026

GG10, GG11 and GG13 are **COMPLETE** within their named package contracts on
`295e7a0` plus the owned worktree. The [manifest](phase-2-ggplot-models-spatial-coordinates-completion-2026-09-16.json)
records source, scene and publication fingerprints. GG14–18 and GG19/global
certification remain separate; this checkpoint does not close them.

Independent Rust/Python/Node WASM authors produce **120 byte-identical publications
per host**: 30 model, 48 spatial and 42 coordinate SVG/PDF/PNG files. All 40 scene
JSON files compare structurally across all three hosts. Original/replayed scenes
match, and strict Python/TypeScript consumers pass with fresh actual modules.
The final files are byte-identical to the independently rendered and inspected
publication set. Proof root: `/private/tmp/finstack-chart-proof-20260916/gg10-11-13-final2`;
runner log: `/private/tmp/gg10-11-13-final2-hosts.log`.

The macOS workspace run passed **1,096 tests across 214 nonempty / 226 total
targets**, with zero failures (`/private/tmp/gg10-13-final-workspace.log`). That run
predates the last localized band inspection and native paint corrections. The final
post-correction core run passed **47 tests across nine targets**
(`/private/tmp/gg10-13-final-focused.log`), and two native coalescing tests pass.
Five raster tests include the native Retina budget regression. These are literal
validation boundaries, not a claim that the entire suite was rerun after those fixes.

Repository/dependency/format/build/check/Clippy stages pass in
`/private/tmp/gg10-13-final-check2.log`. Its rustdoc stage found an unescaped interval
in one documentation comment; after correction, strict workspace rustdoc passed
(`/private/tmp/gg10-13-final-rustdoc.log`) and the remaining WASM core check passed
(`/private/tmp/gg10-13-final-wasm-check.log`). Formatting passes in
`/private/tmp/gg10-13-final-fmt3.log`. Earlier failures are retained in their logs;
the final result combines the successful stages with these explicit continuations.

All 14 coordinate, six representative model and six representative spatial native
panels were inspected with nonnull presented stamps. All owned windows were closed.
Model ribbons exposed a shared band polygon anchor/target mismatch, now corrected
without changing source targets. A regression verifies 41 keyboard prediction
identities in each native-sized model author. Native views now surface inspector
construction errors rather than silently discarding them.

Coordinate image sampling no longer multiplies an existing sample density by the
Retina scale twice. Native nearest-pixel paint skips transparent pixels and merges
exact-color rectangles. Exhaustive small-mask/alpha tests preserve pixel coverage;
a synthetic 600,000-cell sector reconstructs from 448 rectangles. The observed
debug native first-frame interval for the coordinate gradient changed from 24,629 ms
to 948 ms; subsequent observed frames were 684–724 ms. This is diagnostic evidence,
not a benchmark or 60 Hz claim. Native artifacts and stamps are in
`/private/tmp/gg13-native-presented-evidence.json`, with the final coalesced capture
at `/private/tmp/gg13-native-10-11-coalesced.png`.

Detailed numerical, topology and coordinate evidence remains with the
[model checkpoint](gg10-model-kernel-checkpoint-2026-09-15.md),
[spatial checkpoint](gg11-spatial-checkpoint-2026-09-15.md), and
[coordinate report](phase-2-ggplot-coordinates-2026-09-15.md).
[ADR-028](../adr/028-shared-statistical-models.md),
[ADR-029](../adr/029-post-statistical-coordinate-projection.md), and
[ADR-030](../adr/030-shared-two-dimensional-statistics.md) record ownership.

Unknown model formula/family/solver rows remain explicitly unqualified in the
model matrix; they do not silently fall back to OLS. Radial guide animation rejects
while retaining static scenes and guide snapshots. Reference-disclaimed nonlinear
dotplots reject; Cartesian/flip retain physical circles. No fresh Linux execution,
full cumulative binding runner, performance certification or global parity gate is
claimed. Next assigned work is GG14 theme/math, GG15 geography and GG16 extension
integration, followed by their dependent GG17/18 packages.
