# ADR-003: Font, native path and publication route

Status: ACCEPTED for WP-03 feasibility, WP-07 native integration and WP-08 basic headless
publication. The WP-08 adoption section supersedes the historical publication candidate;
full publication fixtures remain WP-13/20.
Date: 6 September 2026. Requirements: ARC-04, LAY-02/04, SCN-03, GPU-01/03, EXP-01/02, QLT-03.

## Decision

Use existing Rust shaping/vector dependencies instead of implementing a font shaper or PDF
writer. The shared capability fixture selects `usvg`/`resvg` 0.45.1 and `svg2pdf` 0.13.0
as the publication candidates. `svg2pdf` 0.13.0 requires the 0.45 `usvg` tree, so a newer
incompatible tree is not substituted merely because its version number is higher.
GPUI's own 0.46 renderer dependency remains internal to the pinned host. No such tree type
is exposed in a chart public API. Exact packages/checksums are in the workspace lockfile.

Publication candidate dependencies remain **dev dependencies** of the two proof consumers. They do
not change `chart-export`'s normal dependency graph (`chart-export → chart-core`) or core's
dependency-free boundary. Promote only the needed packages/features when WP-08 integrates
the public exporter. `resvg` and `svg2pdf` default features are disabled; only text support
is requested. The experiment rejects unsupported native effects and uses no export
raster fallback. Font parsing is explicitly pinned to `ttf-parser` 0.25.1; full TTF font
data in editable SVG uses base64 0.22.1, and PNG density metadata uses png 0.17.16.

The native proof draws numeric paths with GPUI `PathBuilder`/`Window::paint_path`, uses
`with_content_mask` for a rectangular clip and an sRGB GPUI linear gradient. It never
paints a rasterized copy of the figure. The temporary adapter traverses the same shaped
fixture tree as export, converts stroke outlines with tiny-skia-path, and tessellates them
as nonzero fills. The same glyph outlines supply native publication-preview geometry,
including rotation and rich font/color runs. Logical strings and explicit resources remain
in the input tree. The proof consumes core resource contracts but is not yet a core Scene
renderer. The authoritative typed-authoring/normalization decision remains ADR-002;
specification grammar and statistical defaults are unchanged.

## Text and dimensions

For native controls and future desktop layout, use GPUI's own loaded-font/text-system
measurement and painting together. The actual Kit input/labels exercised that native
path. Core's plain `TextMeasurer` bridge, extended shaping metadata, language/direction,
line spacing and font fallback policy still need implementation with real chart layout.
Never use publication metrics as if they were OS-native control metrics.

For publication preview, preserve the point-layout geometry and shaped glyph outlines;
scale the preview to logical pixels and let GPUI apply device scale. The inspected fixture
is such a preview, not proof that OS-native and publication metrics coincide. Ordinary
rotated glyph painting is not exposed by the selected GPUI `paint_glyph` interface;
explicit vector glyph outlines are the demonstrated route. Cache shaped/tessellated
geometry when the chart renderer is integrated; this experiment rebuilds paths on paint
to expose a starting cost rather than claim production caching performance.

Publication uses 72 points/inch and supplied resources, never a system-font scan. SVG
declares `180mm × 120mm` with a point viewBox. PDF uses the same point dimensions. PNG
uses uniform DPI/72 scaling, nearest-integer pixel dimensions and explicit pixels/metre
metadata. 300/600 DPI yield 2126 × 1417 / 4252 × 2835 pixels. Integer pixel rounding can
vary physical extent by less than half a pixel; it must not stretch geometry independently
in x/y to force an integer aspect ratio.

| Output mode | Preserved behavior | Tradeoff |
| --- | --- | --- |
| SVG text | Logical text, two explicit embedded TTF faces through CSS, vector marks | Browsers supporting SVG web fonts render it; other editors may ignore embedded CSS or reshape text differently. |
| SVG outline | Self-contained glyph/mark paths, clipping and gradient | Search/editable text and per-glyph accessibility are lost. |
| PDF text | Subset-embedded Noto Sans/Fira Mono, Unicode maps, selectable text, vector marks | Text extraction may normalize composed/decomposed accents; no byte/string-normalization guarantee. |
| PDF outline | Vector glyph/mark paths with no font objects | Text search/editing is lost; larger output. |
| PNG | Same geometry rasterized at explicit 300/600 DPI | Raster output, no selectable text. |

See [font provenance and permission policy](../../fixtures/capability/README.md). Supplied
unsupported glyphs/permissions must fail with resource identity; fallback requires an
explicit mapping and a diagnostic. The fixed fixture's preflight and missing-glyph negative
probe establish feasibility, not a complete arbitrary-text/resource validator. `pdfa: true`
enables stricter dependency checks; **no PDF/A conformance is claimed**, nor PDF/X, CMYK,
tagged PDF, press color management or screen-reader certification.

## Measured capability and remaining paths

All PASS entries below mean this fixed fixture was actually rendered and inspected.
They are not full requirement or canonical-fixture certification.

| Capability | GPUI proof | SVG / PDF / PNG | Remaining implementation path |
| --- | --- | --- | --- |
| Cubic/quadratic curves, points, thin rules | PASS vector paths | PASS | Lower full core primitive set to bounded numeric paths. |
| Round dashes; butt/round/square caps; miter/round/bevel joins | PASS stroked outlines | PASS | Make styles explicit in the normalized scene; cache against geometry/scale. |
| Rectangular clipping | PASS content mask | PASS clip paths | Core clip stack and transform validation; arbitrary clip paths need geometry clipping or a tested native mask route. |
| Horizontal two-stop sRGB linear gradient | PASS | PASS | Add explicit gradient coordinate/stops contract. Other gradients/patterns are rejected by this proof; tessellated color bands or a tested native shader are feasible vector-native routes. |
| Rich font/color runs, rotation, Greek/minus, composed/decomposed accents, monospaced numerals | PASS glyph outlines | PASS | Shared shaping metadata/cluster mapping, explicit fallback and multiline baseline fixtures; other scripts/faces need supplied resources and tests. |
| Fill rules and transforms | PASS nonzero fills and rotated text | PASS | Preserve local group transforms, winding rules and finite conversion checks in the real adapter. |
| Native input/overlay controls | PASS Kit input, buttons, keyboard Apply/remount | Portable annotation text is in fixture | Convert committed annotation state to scene content; do not export native controls or claim snapshot capture here. |
| Native accessibility hooks | Actual text-field/button/status/image descriptions exposed | PDF text extraction works | Meaningful chart targets, data alternative, focus announcements and screen-reader sessions remain WP-17/21. |
| Images, opacity groups, masks, filters, custom painting, general gradients/clips | Not exercised; unsupported by this adapter | Not certified by this fixture | Add explicit scene/export representation and a per-capability proof. Reuse supplied-image paths, geometry clipping, opacity composition or localized opt-in fallback with region/cause diagnostics; no whole-figure flattening. |

Initial native inspection found missing rotation and holes at joins. Composing local group
transforms and selecting nonzero fill fixed the causes; retained before/after images show
the explained change. Immediate post-remount references survived until frame cleanup;
checking two frames later confirmed release across five remounts. View drop/window-close
events were observed. See the [completion evidence](../evidence/wp-03-completion-2026-09-06.md).

## Dependency maintenance disposition

The refreshed advisory scan still fails on the six previously recorded unmaintained
packages, with no safe upgrade reported: bincode, instant, paste, rustls-pemfile,
rustybuzz and ttf-parser. This candidate route increases direct exposure to the last two;
they are not silently waived. The host already depends on them. Keep the font/shaping
boundary replaceable; WP-08 must assess maintained parser/shaper successors and the
upstream usvg/GPUI transition as part of promotion, and WP-23 must resolve or explicitly
disposition release risks. `block` 0.1.6 also retains its future-compiler warning.
No dependency advisory was ignored and no message was sent upstream.


## WP-07 native integration

`gpui-charts` now consumes `LaidOutChart` from the shared compiler/layout route. It retains
GPUI quads, paths and shaped lines in a native frame; hover redraws reuse them. It paints
explicit Scene clips and vector marks, never a rasterized chart. Conversion to GPUI binary32
coordinates rejects non-finite values or error above 0.25 logical pixel. Text measurement
and painting use the same `WindowTextSystem`, supplied regular face, size and shaped-line
ascent/descent; baseline placement follows those metrics. This is plain, unrotated native
text, not the WP-03 publication outline preview or WP-13 rich typography.

`NativeFont` parses and checks the supplied resource length, budget and family. Register
before GPUI first resolves the family and reserve that family's supplied faces for this
adapter: GPUI caches family selections. The pinned macOS implementation searches its memory
font source before the system source. An App-scoped registry makes an identical registration
idempotent and rejects a different resource under the same family. Missing/control glyphs
and shaped runs using another font reject explicitly. No implicit fallback is supported.
The host retains the registered bytes through GPUI's process text cache; register outside
mount/render loops. This host ownership contract does not certify arbitrary prior/external
font registrations, other native platforms, complex scripts or full font sandboxing.

Promote only the already locked `ttf-parser` **0.25.1** to the normal native dependency
for resource/glyph preflight. No parser/shaper version or publication graph changes. Its
known maintenance advisory remains open; this narrow promotion does not waive WP-08's
successor/transition assessment or WP-23's release disposition.

One `ChartView` owns prepared input, at most a pending and a presented native frame, focus,
and a core `Inspector`. It publishes inspection only after successful paint, with pointer
coordinates relative to that frame's actual bounds. Tooltip content is laid out/painted
only against the same presented chart; a new frame clears obsolete inspection. Weak
next-frame callbacks do not retain disposed chart entities. On preparation failure the
old frame and diagnostic remain; if bounds changed, inspection is disabled until a matching
frame can be painted. The shared inspection reducer is the WP-07 subset, not the full
selection/controlled-state/indexing work of WP-15/16. See the
[WP-07 evidence](../evidence/wp-07-completion-2026-09-06.md).

## WP-08 headless publication adoption

This section supersedes the WP-03 publication candidate for the normal library graph;
the original dev-only experiment and its evidence remain unchanged. Adopt `usvg`/`resvg`
**0.48.1** with only text/writer features, `krilla` **0.8.2** with default features disabled,
`skrifa` **0.44.0** for resource/permission preflight, SHA-256 **0.10.9**, base64 **0.22.1**
and PNG **0.17.16**. The lockfile fixes all transitive versions. No GPUI dependency,
system-font loading feature, mandatory threads, custom font shaper or custom PDF writer
is introduced in `chart-export` or core.

The maintenance assessment found that [svg2pdf was archived on 10 July 2026 and directs
users to krilla/krilla-svg](https://github.com/typst/svg2pdf). The
[resvg 0.48 changelog](https://github.com/linebender/resvg/blob/main/CHANGELOG.md) records
its move from rustybuzz/ttf-parser to harfrust/skrifa. `krilla-svg` 0.8.1 would force the
older 0.47 usvg/font stack, so the library instead uses
[krilla's positioned-glyph/vector API](https://github.com/LaurenzV/krilla) directly.
Its optional simple-text/rustybuzz and raster features are disabled. The bounded adapter
maps only the existing solid core primitives and exact usvg-shaped glyph IDs/transforms;
it does not implement general SVG interpretation. Existing native and WP-03 dev dependencies
retain their locked old versions. The six workspace maintenance advisories remain open,
with no ignores; the normal export graph excludes rustybuzz, ttf-parser and svg2pdf.
See the [actual dependency graph](../evidence/wp-08/export-dependencies.txt).

`FigureSnapshot::capture` clones definition/state/profile, retains the coherent source
snapshot and font bytes, runs the shared core compiler/layout in physical points and
builds one immutable publication tree. It records definition/store/state/layout/viewport
stamps, source epoch, dataset/schema versions, font SHA-256 identities, annotation revision,
physical settings and engine versions. The retained prepared chart owns the actual definition
and data. Encoding consults no current application state. `FullDomain` removes viewport
restrictions, including named-axis viewports, while preserving visibility, explicit domain
policy and upstream statistics. Original capture settings remain in reproduction metadata.

The profile supplies page dimensions (points or millimetres), DPI, plain annotation scene
items, explicit background, resource/output/raster bounds and a binary32 conversion error
limit (default 0.01 pt, maximum 0.25 pt). Dimensions and coordinates outside representable
precision fail; PNG uses uniform DPI/72 scaling and checked rounded dimensions. Transparent
PNG emits straight RGBA, with no extra background compositing. Encoders enforce returned-byte
limits; raster allocation is bounded by pixel count. This is not a strict allocator peak-memory
quota: vector serialization/PDF internals can allocate before the final output-size check.

Fonts are supplied single-face static TTF/CFF bytes with exact resource ID/revision and
permission preflight. The font database contains only these bytes; private aliases and an
explicit resolver prevent system fallback. Missing/control characters, substituted glyphs,
unsupported variable/color/bitmap/SVG fonts and prohibited embedding reject with diagnostics.
Editable SVG embeds the full supplied font and rejects preview/print-only permission;
PDF embeds subsets and rejects no-subsetting permission in preserve mode. Outline mode
removes embedded fonts and selectable text. Restricted or bitmap-only permission is rejected
at resource creation. These conservative policies and format representation are public,
not an implicit fallback. Browser text SVG can reshape text; the outline preview is the
exact point-layout route when geometry must be preserved.

SVG/PDF keep vector marks and rectangle clipping; PNG rasterizes the same positioned tree.
The native publication example paints its outlined SVG as vector paths, retaining glyph
geometry through resizing. Plain text, solid rules/paths/rectangles/points and literal
annotations are the implemented core subset. Gradients, rich/rotated text, images, groups,
full figure furniture and fallback require later scene contracts and evidence; the broader
WP-03 feasibility matrix does not certify them in this library. No PDF/A/X, tagged PDF,
CMYK or full complex-script claim is made. See the
[WP-08 completion report](../evidence/wp-08-completion-2026-09-06.md).
