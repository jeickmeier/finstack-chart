# WP-08 completion evidence — 6 September 2026

WP-08 is DONE for the headless export and snapshot foundation. A real core chart exports
SVG/PDF/PNG without GPUI initialization; the native publication preview paints the same
captured vector layout. This closes the assigned basic/minimal acceptance scope, not full
FIX-13/14 or later typography/live-export gates. WP-09 is ready; G1 remains open for actual
Python/WASM runtime proofs.

## Scope and environment

Starting revision is `f8657fb` plus the preceding uncommitted WP-07 changes. Result is that
baseline plus uncommitted WP-08 exporter, fixture, preview, dependency selection and evidence.
No commit or staging was requested. WP-07 work is preserved. Assigned IDs: ARC-02,
EXP-01/02/03/04, LAY-04, SCN-03; minimal FIX-13/14. Owned code is `chart-export`, the
publication fixture and a standalone gallery example; no new quantitative or grammar
engine, binding API or application-owned worker is introduced.

Working directory: `/Users/jeickmeier/Projects/finstack-chart`. macOS 26.5.2 arm64,
Rust 1.97.1 via mise, locked dependencies, local graphical session at native scale 2.
Poppler independently reads and renders PDF artifacts; Chromium inspected the embedded-font
SVG. All evidence is local; no Linux runtime or hosted CI execution is claimed.

## Delivered behavior

`FigureSnapshot::capture` retains coherent definition/data/state/font inputs, compiles
through core, measures supplied fonts and lays out in points. Its immutable scene includes
page background and explicit portable annotation items. `export` returns bytes, diagnostics,
capabilities and reproduction metadata. Saving is the caller's responsibility. A snapshot
is Send + Sync without requiring threads. Tests export a retained snapshot in a worker
while the caller commits an upsert/append and changes definition/annotation revisions;
the old SVG remains unchanged, the next snapshot differs and retained references release
when the old snapshot is dropped.

Visible/full-domain capture preserves bin populations, aggregate membership and layer
visibility. Full-domain removes viewport restrictions without changing upstream statistics;
original captured viewport settings remain recorded. Metadata includes definition/store/
layout/state/viewport stamps, source epoch, dataset/schema versions, exact font hashes,
physical dimensions, DPI, text/background settings and annotation revision. The retained
prepared chart owns actual definition and data. This is not the WP-09 portable envelope.

The public exporter uses maintained resvg/usvg 0.48.1 and krilla 0.8.2. Krilla receives
already-shaped glyph IDs and positions; its optional simple-text and raster features are
disabled. [ADR-003](../adr/003-font-and-renderer-capability-route.md) records the upstream
assessment, feature selection, explicit font policies and numeric/resource bounds.
[The normal export graph](wp-08/export-dependencies.txt) has no GPUI, svg2pdf, rustybuzz
or ttf-parser. Core remains dependency-free. Historical WP-03 and native dependencies remain
available; the current advisory scan still fails on the same six unmaintained packages.
No advisory was ignored or fixture weakened. `block` 0.1.6 retains its future-compiler warning.

## Acceptance and independent evidence

| Contract | Verdict for WP-08 | Evidence |
| --- | --- | --- |
| ARC-02: headless shared-core export | PASS | Actual public-API fixture execution, normal graph isolation and full repository target checks; library accepts bytes/resources and performs no file or system-font lookup. |
| EXP-01/02, SCN-03: physical vector/raster formats and font policy | PASS basic subset | Both SVG/PDF modes, 300/600 DPI PNG, Poppler font/image/text inspection, browser rendering, explicit capability reporting, missing-font/glyph and embedding-permission negatives. |
| EXP-03, minimal FIX-14: coherent captured snapshot | PASS bounded fixture | Concurrent export versus data/definition/annotation edits, original stamp/bytes preserved, later capture differs, old layout/font bytes released. |
| EXP-04: visible/full-domain semantics and reproduction settings | PASS foundation | Same independently expected histogram counts [2,3] and aggregate members, viewport-only geometry difference, hidden-layer state retained, captured settings and exact font hash. |
| LAY-04, minimal FIX-13: publication preview | PASS basic single panel | Preview SVG is byte-identical to outlined export; actual native vector window and resize inspected. No native text remeasurement or raster chart paint. |

The fixture is 180 × 120 mm (510.2362204724 × 340.1574803150 pt). SVG/PDF retain two
line segments separated at missing y, four points, numeric axes and the literal annotation
`Captured publication - café Ω`. Rectangle clips deliberately cut boundary points; this is
the core plot clip, not accidental export cropping. Independent checks verify known projected
coordinates at 0.0001 pt tolerance, PDF dimensions at 0.002 pt (Poppler rounding), exact
font SHA-256, vector objects and zero PDF images. PDF preserve mode has one subset-embedded
Noto Sans face with Unicode maps and extracted annotation text; outline mode has no fonts
or extracted text. SVG preserve embeds the full face under its private alias; outline has
no text/image elements. Preview is the exact outlined SVG.

PNG 300/600 DPI are exactly 2126 × 1417 / 4252 × 2835 pixels with 11811/23622 pixels/metre.
A separate transparent pixel test verifies straight-alpha output and avoids double background
compositing. Noto Sans six tabular digits at 10 pt independently measure 34.32 pt, and AV
kerning is applied. Tests also exercise actual histogram exports, quadratic/cubic paths,
empty clips, XML escaping, missing resource revisions/glyphs, permission restrictions,
nonrepresentable coordinates and output/raster limits. Synthetic permission mutations are
negative probes, not additional distributed font fixtures.

### Inspected artifacts

- [Text SVG](wp-08/publication-text.svg), [outline SVG](wp-08/publication-outline.svg),
  [text PDF](wp-08/publication-text.pdf), [outline PDF](wp-08/publication-outline.pdf).
- [300 DPI PNG](wp-08/publication-300.png) and [600 DPI PNG](wp-08/publication-600.png).
- Independent PDF renders: [text](wp-08/pdf-text-render.png),
  [outline](wp-08/pdf-outline-render.png). Both reviewed for glyphs, clipping, line gaps,
  axes and annotation positioning; the 600 DPI image was also visually inspected at display scale.
- [Native preview](wp-08/native-preview.png) and [resized preview](wp-08/native-preview-resized.png):
  same marks/glyphs/spacing under uniform scaling, with host labels outside the figure.
  [Preview log](wp-08/native-preview.log) records vector loading and closure.
- [Browser text SVG capture](wp-08/svg-text-browser.png) and
  [DOM evidence](wp-08/svg-browser-dom.json): expected point dimensions and explicit aliases,
  glyph geometry consistent with the supplied font (single-digit width 7.625 CSS px versus
  7.6267 expected before browser rounding). Browser tab and temporary HTTP server were closed.
- [Captured metadata](wp-08/metadata.txt), [scene geometry](wp-08/geometry.txt),
  [independent checks](wp-08/artifact-checks.log), [advisory scan](wp-08/advisories.log).

The native preview was run as a temporary local macOS application wrapper around the built
example and exercised through actual window resize; the app was closed after capture.
No visual baseline was regenerated to conceal a failure. During implementation, independent
checks exposed background premultiplication/compositing and font-policy edge cases; fixes
now have direct regression assertions. The first artifact-checker draft counted axis rules
as data lines; it was corrected to select the fixture's blue data strokes, preserving all
geometry expectations.

## Commands and results

All commands ran from the working directory above, with the lockfile:

| Command | Result / evidence |
| --- | --- |
| `mise run fmt` | PASS; [format log](wp-08/fmt.log). |
| `mise run check` | PASS; repository graphs/links, license/source gate, formatting, workspace builds/Clippy/rustdoc, native/Kit examples and core WASM compile; [check log](wp-08/check.log). |
| `mise exec -- python scripts/check_repository.py` (after final documentation) | PASS; [final graph/link check](wp-08/repository-final.log). `git diff --check` also passes. |
| `mise run test` | PASS: 89 core + 10 export + 1 native conversion + 2 Rustdoc examples = 102 tests; [test log](wp-08/test.log). |
| `mise exec -- cargo run -p chart-export --example publication_export --locked -- docs/evidence/wp-08` | PASS; [export log](wp-08/export.log), all actual artifacts above. |
| `mise exec -- python scripts/check_publication_artifacts.py docs/evidence/wp-08` | PASS; dimensions, line/point geometry, font/hash/text/image policies and exact preview equality. Requires Poppler. |
| `mise exec -- cargo run -p chart-gallery --example publication_preview --locked -- docs/evidence/wp-08/publication-preview.svg` | Actual native vector rendering and resize inspected; application closure recorded. |
| `pdftoppm -scale-to 1400 -png -singlefile docs/evidence/wp-08/publication-text.pdf docs/evidence/wp-08/pdf-text-render` | PASS; same command for outline PDF, both images inspected. |
| `mise exec -- cargo tree -p chart-export --edges normal,build --locked` | Export isolation and selected maintenance route verified; graph retained. |
| `mise exec -- cargo deny --locked check advisories` | FAIL: same six existing unmaintained advisories, refreshed database; separate from passing license/source check. |

## Limits and next action

This implementation supports existing solid core primitives and plain static-font text.
It rejects unsupported resource types and never silently substitutes a font or rasterizes
the whole vector figure. Broader gradients, rich/rotated/multiline typography, complex scripts,
images, nested transforms, full figure furniture and multi-panel layout require their owning
scene/layout packages and new evidence. WP-03 demonstrated some broader rendering capability;
those dev-only proofs are not claims about the new public exporter.

SVG text depends on consumer web-font support and shaping; outline mode preserves the
captured glyph geometry. No PDF/A/X, tagged PDF, CMYK, screen-reader certification or pixel-
identical cross-renderer promise. Vector output byte limits apply to returned output,
not strict peak allocator memory. The snapshot test is bounded concurrency, not sustained
live interaction/queue/cache/performance certification. WP-13 completes publication
composition and WP-20 completes live exports. Linux, full platform behavior and release
maintenance disposition remain WP-21/23. Next assignment: WP-09 portable schema and actual
Python/WASM binding proofs. G1–G4 remain open.
