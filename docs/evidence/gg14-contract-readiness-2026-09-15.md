# GG-14 contract and fixture readiness — read-only, 15 September 2026

GG-14 remains gated by GG-05/08/12/13. This is source/ownership preparation, not
implementation or acceptance. The authoritative contract is GG2-09, LAY-02/04 and
THM-01/02/03, with FIX-GG14 in the Phase 2 implementation plan. No new theme/math
API or dependency is introduced by this review.

## Exact pinned surface

`fixtures/parity/ggplot2/inventory.json` assigns 41 exports to GG-14. Its companion
`documentation.json` already contains the complete `theme.Rd`, `element.Rd`,
`get_theme.Rd`, `calc_element.Rd`, `ggtheme.Rd` and subtheme descriptions. Live
read-only R 4.6.1 / ggplot2 4.0.3 inspection confirms **160 element-tree nodes** and
**144 authored entries in default theme_grey** (`/private/tmp/gg14-contract.txt`).
A generated fixture must retain all nodes, ordered parent lists and accepted classes;
merely copying the named `theme()` arguments misses parts of the tree.

Complete presets are grey (gray alias), bw, linedraw, light, dark, minimal, classic,
void and test: nine distinct presets and one alias. All accept base/header families,
base text/line/rectangle sizes and ink/paper/accent controls. The eleven `theme_sub_*`
constructors need exact name expansion and invalid-component behavior; they should
lower to the same typed element map.

Cross-package inventory edges are real: `get_theme`, `set_theme`, `update_theme`,
`replace_theme`, `get_element_tree`, `merge_element`, `complete_theme`, `class_theme`,
`register_theme_elements` and predicates currently have GG-16 ownership; their
semantics are prerequisites of GG-14 contexts/resolution. `label_parsed` belongs to
GG-12 and must feed the shared GG-14 parser/layout. These entries need integrated
ownership evidence, not duplicate implementations or a claim that only legacy alias
names are supported.

## Theme semantics that must be captured

| Contract | Independent fixture matrix |
|---|---|
| Element types | Blank, line, rectangle, text, point, polygon, geom, units/relative units, margins and scalar controls; accepted properties, aliases, invalid classes and missing fields. Point/polygon elements are documented extension points, not ordinary plot marks. |
| Inheritance | Entire 160-node graph, missing child versus explicit child, ordered multi-parent inheritance, root completion, unknown names and incomplete roots. `axis.minor.ticks.length.x.top` has two parents: a single-parent resolver is wrong. |
| Blanks | A blank draws nothing and reserves no space; `inherit.blank=true/false` and `calc_element(skip_blank=...)` are independent controls. Include nonblank children of blank parents, blank child of ordinary parent and complete-theme boundary. |
| Relative values | Nested relative text/line sizes, tick/spacing units, partially inherited margins, and zero/negative values according to the particular property. `margin_auto` copies top/right defaults; `margin_part` leaves unspecified sides to inherit. |
| Update/replace/context | Update merges specified properties within an element; replace replaces the whole element, resetting omitted properties. Set replaces the entire context and returns the previous theme. Test two isolated explicit contexts and drawing-time context selection with immutable captured requests. |
| Presets | Resolve every tree node under all nine presets, defaults plus altered base/header family and sizes and ink/paper/accent. Check gray/grey identity and actual strip/axis/legend layout. |
| Cross-stage stability | Statistics, grouping, positions, provenance and domains are identical across palette/typography changes; palette-only updates retain numerical preparation, text/margin updates invalidate layout. Geometry-theme defaults still feed the existing theme-expression owner. |

## Mathematical label contract

The live pinned `grDevices::plotmath` documentation is captured read-only in
`/private/tmp/gg14-contract.txt`. Its full syntax table is the parser/layout inventory,
not just superscript/fraction examples. It includes unary/binary arithmetic,
relations, set operators, arrows, Greek/Adobe Symbol aliases, scripts, indexed roots,
fractions/atop, large operators with limits, accents, underlines, display/text/script
styles, phantom dimensions, visible/invisible grouping and scalable delimiters.

Important source distinctions for fixture generation:

- `paste` juxtaposes without automatic spaces. `phantom` has dimensions but no paint.
- Math strings do not interpret newline control characters as ordinary multiline text.
- Numeric constants and symbol-font Greek letters are not changed by bold/italic wrappers.
- Scalable delimiters are parentheses/brackets/braces/bars and their partners; a dot or
  empty delimiter omits that side. Double bar has the same reference effect as single
  bar. Ceiling/floor/angle delimiters are not scalable in the reference.
- The typed parser must reject unsupported syntax or malformed input without executing
  R or host code. A TeX string renderer alone is not a plotmath-compatible parser.

Required fixture groups: complete syntax/alias/precedence acceptance table; nested
layout topology and width/ascent/descent from supplied fixed fonts; scripts and
large-operator style transitions; phantom/empty delimiters; unusual Unicode, rotation
and missing-face diagnostics; depth/node/text/path budgets; wrong syntax/nonfinite
parameters; all consumers (row marks, axis labels/titles, legend labels/titles, strips,
figure titles and annotations); immutable replay and actual Python/WASM parity.
Visual selection should cover the distinct layout families once at normal size and
300/600 DPI, including text-preserving and outline PDF/SVG plus native views.
Reference device/font metrics must be recorded separately from cross-host shared
layout identity; platform fonts are not an oracle.

## Existing owners and minimal integration sequence

`theme.rs::ThemeSpec/ThemePatch` currently supplies a flat plot/layer cascade, three
legacy named themes and geometry/palette contexts. `layout/guide_components.rs`
provides useful per-guide overrides but no full element hierarchy. `typography.rs`
and `layout/text.rs` own explicit rich-run shaping and bounds; `chart-text` owns actual
font parsing/shaping. GG-08 row text already uses those services, typed units and
rotated inspection regions. None of these paths currently implements math layout.

1. Freeze the complete graph/preset/context fixtures and introduce one typed element
   resolver inside the existing theme owner, with explicit merge/replace operations.
2. Bind its resolved values to existing guide, facet, composition and geom consumers
   after GG-12/13 establish their final strip/radial placement contracts.
3. Add a bounded typed plotmath AST/parser and one shared box-layout owner. Font
   metrics/shaping remain in existing services; operators/rules/delimiters lower to
   ordinary portable glyphs/paths. Retain logical math text and semantic accessibility.
4. Add every label consumer through this one owner, then run one shared actual-host
   build and selected native/publication matrix. Do not repeat a full cumulative host
   suite for each theme property or math operator.

Open prerequisites: GG-12 and GG-13 acceptance, final GG-08 integration evidence,
complete reference graph/preset/math fixtures, and the font/metric strategy for the
full plotmath symbol set. No GG-14 requirement or gate is closed by this document.
