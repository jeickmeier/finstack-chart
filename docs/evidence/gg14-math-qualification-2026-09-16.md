# GG14 mathematical text qualification checkpoint

Status: **COMPLETE** under the [final wave acceptance](phase-2-ggplot-final-wave-2026-09-16.md). Earlier checkpoints below retain their historical pending statements; final signed-metric and multilingual proofs resolve them.
The parent package owner retains the status ledger and the shared host/build gate.

## Single owner and interfaces

`typography/math.rs` owns a bounded nonexecuting plotmath parser and serializable
`MathExpression { source, ast, fonts }`. Source/AST coherence is validated on replay.
`typography/math_layout.rs` owns mathematical boxes using only the destination's
existing `TextMeasurer::shape` and explicitly supplied font resources. All output is
ordinary portable glyph outlines and paths. Original source remains logical text.
There is no TeX subprocess, R execution, system-font discovery or runtime dependency.

`RichRun.math` is optional and omitted when absent. Row `TextGeom.math` is likewise
omitted when absent. Existing rich titles, axis text, annotations and other shared
text consumers use this owner. Replacing a parsed guide template reparses the new
label through `RichRun::replace_text`. Fraction/radical paths participate in shared
text placement and animated guide-label translation. Font descriptors are validated
even before an empty row population is rendered.

Explicit `MathFonts` regular/italic/bold/bold-italic/symbol resources select real
faces. Existing `RichRun.fallback` propagates to mathematical subruns. The committed
DejaVu 2.37 regular/style faces, DejaVu Math TeX Gyre symbol face, and deliberate
DejaVu Sans fallback are the test resources; see `fixtures/math/fonts/manifest.json`
and licenses. The symbol font contains historical U+2329/U+232A but lacks U+03D2;
Upsilon1 therefore requires the explicitly declared Sans fallback. Cairo can perform
that fallback silently, so font-device metrics and exact cross-host identity remain
separate qualifications.

## Independent source evidence

`tools/reference/r/plotmath-metrics.R` requires pinned R 4.6.1 and a functioning
Cairo device. It installs a private Fontconfig configuration containing only the
committed font directory and asserts actual `cairo_pdf` device creation. Missing
Cairo warnings are errors, preventing silent fallback to the PDF default font.
`fixtures/parity/ggplot2/plotmath-metrics.json` contains all 96 parseable syntax rows
and 53 expanded aliases at 12 and 24 points: **298 source cases**. Font identities,
units, device and symbol settings accompany the fixture.

Command:

```
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work mise exec -- python3 tools/reference/r/run.py tools/reference/r/plotmath-metrics.R
```

All 298 source metric captures succeeded. Private pinned XQuartz libraries are
used by the oracle launcher; no system installation or runtime dependency was made.

The external R mathematical typesetting source was inspected at
`https://raw.githubusercontent.com/wch/r-source/R-4-6-branch/src/main/plotmath.c`
(private investigation copy `/private/tmp/gg14-plotmath-source.c`). The original Rust
implementation follows mathematical box composition, font metrics, glyph identities
and measured reference conventions; external implementation code is not transplanted.
In particular, default display style uses Plain font; script scales are 0.7/0.5;
spacing depends on measured M/x/X/+ metrics; signed glyph depth is retained internally;
public metric values use magnitudes; delimiters use explicit glyph assemblies;
fractions and radicals use fixed physical reference rule spacing.

## Evidence and remaining gate

Parser fixture tests previously passed all 96 parseable cases and malformed-input
counterexamples. Shared mocked-shaper tests cover all 149 expressions plus actual
fraction/script/phantom/root topology. The actual-font diagnostic shapes all 298
cases. It deliberately reports residual source metric differences rather than
asserting an invented loose tolerance. Latest completed diagnostic before final
root/list corrections: `/private/tmp/gg14-math-fonts-signed.log`, maxima width
2.7461 points, ascent 3.2682 points, descent 2.625 points. These are **not accepted
source parity tolerances**. Device hinting and mathematical convention differences
must be separately resolved or bounded with explicit evidence.

Five independent authors exist in `examples/common/ggplot_math_controls.rs`, the
chart-export wrapper, and `scripts/bindings/ggplot_math_controls.py/.cjs`. They retain
original-vs-replayed scene comparisons and export SVG/PDF/PNG. These are not yet
qualified host publications. Required remaining evidence includes actual rendered
inspection, fresh native/Python/WASM runs, all label consumers including legends,
facet strips and annotations, source updates, animated fraction-guide regressions,
physical-unit/rotation cases, malformed/budget cases, and the remaining source metric
qualification. No GG14 completion claim follows from this checkpoint.


## Subsequent focused checkpoint

The independent ordinary-glyph fixture now captures 180 controls, of which 132
ASCII controls are compared through the exact supplied fonts at 6, 8.4, 12, 16.8,
24 and 30 points. Observed Cairo/device versus design-unit differences are at most
0.98 points width, 0.953125 ascent and 0.92 descent per control. This quantifies the
underlying device difference independently of compound mathematical expressions;
it does not justify a blanket compound-expression tolerance.

`/private/tmp/gg14-math-consumers-final.log` confirms the two metric diagnostics;
the expanded consumer test caught a missing explicit Color legend selection in an
author, which was corrected in Rust/Python/WASM. The subsequent standalone Rust
run passed all eight original/replay author scenes and **24 SVG/PDF/PNG
publications**, `/private/tmp/gg14-math-authors4.log`, artifacts
`/private/tmp/gg14-math-authors`. All eight PNGs were visually inspected: scripts,
fractions, radicals, operator limits, accents, font styles, assembled parentheses,
angle brackets, parsed ticks/titles, strips, legend and annotation paint correctly
and stay within their intended panels. A follow-up author-only change moves the
legend's point marks below the text for easier inspection; final mode6 recapture is
pending. Native and actual Python/WASM inspection remains pending.

Numeric literals now canonicalize the value as the pinned scalar R formatter does
(e.g. 1e3 becomes 1000, 1e5 becomes 1e+05), while preserving the original expression
source separately. There is still no expression execution. Final six-test parser/
layout checkpoint is running after this change.

## Final focused math/native checkpoint (16 September)

`/private/tmp/gg14-math-168-final.log` records four passing export tests after the
ordinary text baseline correction. The source fixture contains 204 ordinary glyph
controls; 168 ASCII plus explicitly covered symbol-font controls are compared.
Maximum ordinary device/design-unit deltas are width 0.98pt, ascent 0.96pt and
descent 0.92pt. The worst compound width delta, 2.82971875pt for `list(x,y,z)`, is
exactly decomposed into its independently measured atoms (1e-12 decomposition
check). Compound ascent/descent maxima remain diagnostic, not accepted tolerances:
2.2366796875pt (`dot(x)`) and 2.11799296875pt (integral). Thus these tests do not
claim identical Cairo and design-unit font metrics.

The native gallery was built and both batches inspected: all eight panels have
nonnull presentation stamps, positive paint counts and no diagnostic. Evidence:
`/private/tmp/gg14-math-native-0-5.{log,png}` and
`/private/tmp/gg14-math-native-6-7.{log,png}`. The latter includes the final legend
point placement. Source number spelling, all eight consumers, immutable scene
replay, fraction guide animation, and supplied-font SVG/PDF/PNG output pass their
focused tests. Parent reports all108 GG14 publications match across fresh Rust,
Python and WASM hosts (gg14-comparisons.json in the final GG14/16/18 run).

The accepted metric adaptation is explicit supplied-font design-unit geometry,
not Cairo pixel-rounded glyph bounds. Ordinary atom metrics and compound width
rounding decomposition are qualified above. The final independent vertical proof
is `vertical_accent_and_integral_differences_decompose_into_signed_atoms`, passing
at 12pt and 24pt (`/private/tmp/gg14-vertical-decomposition.log`). The original
oracle-only C bridge calls public GEMetricInfo and retains signed glyph depth;
ordinary text grob metrics clamp negative depth and therefore could not explain
these cases. Fixtures are `plotmath-vertical-controls.json` (16 signed atoms),
with reproducible R/C generators in tools/reference/r.

For dot ascent, both owners use body ascent + signed accent depth + 0.1 cap height
+ accent ascent. For the integral descent, both use 0.99 lower-half ascent - half
plus-sign ascent + lower-half depth + half(subscript ascent + subscript depth).
Source formulas match captured compound metrics within 1e-12; independent font
header bounds reproduce runtime compound metrics within 1e-10. Their difference
also exactly reproduces the previously reported maxima. No placement-rule defect
or new tolerance was introduced. Exact Cairo vertical bounds remain an intentional
device/design metric adaptation, now qualified rather than an unresolved gate.
Parent owns rotated/multilingual mathematical author coverage and inspection.

Final rotated/multilingual author: mode8 combines quoted Latin accents, Greek and Cyrillic with fractions, radicals, scripts and accents at30degrees. Fresh Rust/Python/Node WASM27mathematical publications per host and scene JSONs match (`/private/tmp/gg14-math-final3.log`, `/private/tmp/finstack-chart-proof-20260916/gg15-16-18-final2/gg14-math-comparisons.json`). Independent SVG and Poppler PDF rasterizations plus PNG were inspected in `ggplot_math_controls/rust/inspection/all-publications.png`; supplied DejaVu Serif fallback was passed explicitly to the independent SVG renderer. Native mode8 was inspected at `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-16_20-38-35.png`, with `/private/tmp/gg14-math-final8-native` traces. This extends the previous108GG14 outputs to111per host without replacing prior source fixtures or visual baselines.
