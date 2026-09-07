# Theme, typography and composition contract

WP-13 implements THM-01/02/03, LAY-02/03/04, EXP-01/02/04 and GPU-03 at the
FIX-12/13 boundary. [Specification](spec/gpui-charts-specification.md) is normative;
[completion evidence](evidence/wp-13-completion-2026-09-07.md) records actual validation.

## Presentation and cache ownership

`ThemeSpec` version 1 supports editorial, terminal and grayscale presets. Resolution
is defaults → host → named theme → plot → layer → interaction → output. Optional
`ThemePatch` fields inherit; colors, spacing, typography size, strokes, dash arrays,
symbols, focus/selection accents and axis-aligned two-stop gradients are explicit.
Mapped colors/sizes retain priority over constant mark tokens; grayscale is an explicit
final paint conversion that leaves mapped values and legend semantics unchanged.
Output tokens do not mutate the authored definition. Kit copies its semantic snapshot
to generic host tokens and sends its reset button through the shared reducer.

The compiler retains one last valid prepared root for presentation-only changes.
Source identity, state, limits, mappings, transforms, layer definitions and facets must
match. Theme, figure, axes and definition revision can change without recomputing data
or numeric marks. Tables and mark arrays remain shared by `Arc`; new definitions are
rebound throughout panels. Invalid definitions are still rejected before reuse.

## Explicit text

`chart-text` is an adapter service depending on core, harfrust 0.12.0 and ttf-parser
0.25.1. Core owns numeric glyph runs and resource identity, and never loads a file,
consults a system font database, or depends on a native/interpreter/browser object.
The native and export adapters provide immutable explicit font bytes. Family, actual
weight/style, resource revision and byte length are validated; synthetic bold and silent
font substitution are rejected. Rich blocks carry an authored font, size, language,
LTR/RTL direction and tabular-number choice per run. Fallback is an explicit ordered
list and selects a whole face covering a whole run, producing a diagnostic. This is
run-level shaping, not an automatic paragraph bidi/line-breaking engine. The caller
splits mixed-direction text into runs. Missing glyphs without a valid fallback fail.

Positioned glyph IDs, UTF-8 cluster ranges, advances, outlines and actual font identity
are retained with logical Unicode text. Rotation uses the same numeric geometry for
measurement and drawing. Axis titles/ticks can use these runs. Number formatting
supports explicit decimal/scientific notation and EnUs/DeDe/FrFr formatting; no process
locale is consulted. Native ordinary text keeps its destination font service; advanced
runs use the shared shaper. Publication uses the explicit export service. The native
publication preview consumes the exact export outlines without remeasurement.

SVG preserve mode retains ordinary embedded-font text but emits advanced runs as exact
positioned paths plus logical text metadata (`MixedPositionedOutlines`). It does not
claim that every label is editable SVG text. Outline mode contains no font-dependent
text. PDF embeds/subsets the explicit faces and encodes each complete positioned run
with original clusters and searchable Unicode. PNG uses the SVG publication scene.
Existing font embedding/outline permissions remain enforced. Physical dimensions are
independent of DPI; 180 × 120 mm is 510.236220 × 340.157480 points. Raster rounding gives
2126 × 1417 at 300 DPI and 4252 × 2835 at 600 DPI.

## Figure furniture

`FigureComposition` version 1 includes rich title/subtitle, caption/source/footnotes,
panel letters with stable typed keys, annotations/direct labels and at most four insets.
Top/bottom furniture reserves layout space. Anchors explicitly select data/calculation
coordinates (with panel and scales), useful-plot fractions, figure fractions, or absolute
output units. Timestamp anchors retain exact integer values and units. Annotation
rotation, offset, clipping and optional callout endpoint are explicit. Leader endpoints
use the closest point on the label bounds. High-priority labels place first with stable
ties; keep/hide/shift-then-hide are deterministic. Shift tries eight finite candidates;
unplaceable labels produce pressure diagnostics without altering their authored data.

Insets require unique IDs, an existing parent, bounded plot-relative rectangles, explicit
layer lists and optional view intervals/guides. They reuse parent prepared tables/marks
and trained scales, replacing only destination bounds and explicit views. They never
rerun a stat on the cropped population. Parent/inset scenes retain true source targets;
portable scene arrays include aligned item/panel identities and inset metadata. Whole
figure and expanded inset work is budgeted before cloning or font callbacks. Legend
collection/per-panel placement uses the WP-12 guide contract.

Gradients are horizontal/vertical, two-stop, sRGB with alpha. Dashed paths support numeric
polylines and closed polylines with bounded positive even dash arrays; solid quadratic
and cubic paths remain supported. Curved dashes require caller flattening. Dash phase
resets at subpath boundaries and is continuous across vertices. Symbols lower to shared
vector geometry in native/SVG/PDF/PNG. Resource/command/text budgets apply before paint.
