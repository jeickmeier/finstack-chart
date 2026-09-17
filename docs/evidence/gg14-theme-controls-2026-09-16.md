# GG14 theme control qualification — 16 September 2026

This report covers the theme half of GG14. The math owner and aggregate package acceptance are recorded separately. The implementation uses the existing grammar, text measurement, stroke outline, facet, guide, and immutable publication owners.

## Source and contracts

The development oracle is ggplot2 4.0.3 under R 4.6.1, installed under `/private/tmp/finstack-chart-tools/r-library`. Runtime does not load R, perform I/O, select system fonts, or hold a process-global mutable theme.

- `theme-hierarchy-controls.json` and its generator capture the complete 160-node element tree, nine presets with normal and custom constructor controls, inheritance/blank/relative/partial-margin cases, isolated theme operations, and eleven subtheme expansions.
- `theme-resolution-vectors.json` retains independently computed R results for 2,880 resolved nodes. Runtime `theme/reference_data.json` contains only the source tree and authored preset values, not the expected resolution results.
- `theme-unit-context.json` distinguishes grid's physical TeX points (72.27/inch) from text gpar big points (72/inch), and captures eleven units in two font/line-height contexts.
- `theme-line-controls.json` captures line width, dash, cap, join, and arrow controls from `element_grob()`.
- `theme-consumer-layout.json` captures inside/outside strip ordering, physical and null-weight panel widths, and horizontal/vertical multiple-guide table placement.

The source's serialized `italic`, `fontweight`, and `fontwidth` S7 slots are NULL in these presets. Passing those names to the pinned `element_text()` constructor warns that `...` must be empty; they are not evidence of supported variable-font authoring. Font family and face selection uses explicit `TextFont` resources.

## Implementation boundary

`ElementTheme` resolves complete/partial themes, inherited blanks, `inherit.blank`, relative values, and partial margins in one owner. `ThemeContext` owns isolated snapshots and implements get/set/update/replace; the Python/WASM contexts delegate element operations to Rust and retain their own handles.

Destination configuration resolves the hierarchy once and caches contextual lengths and the caller's font catalog. Final authored/destination flat overrides retain the existing cascade precedence. Themes affect geometry defaults and the existing axis/grid, legend/key/colorbar, panel/strip, and composition consumers. Physical/null panel sizes are allocated after axis furniture has been measured. `panel.ontop` changes decoration order without changing targets; `panel.border` suppresses fill as the source panel renderer does.

Single and multiple guides share measured blocks for direction, title/text side placement, spacing, margins, and justification. Colorbar frames, axes, and ticks use the existing common stroke geometry. The strip path retains the legacy plain measurer when no hierarchy or math is requested.

`Layer.text_defaults()` is additive: it retains `LayerGrammar.default_text=true`, resolves inherited font size through the existing prepared text-size aesthetic, and uses the common supplied font catalog. Calling `text_geom(options)` clears that marker, including when explicitly supplied options equal their historical defaults. Existing authored text options and row mappings remain authoritative.

## Focused evidence

- `/private/tmp/gg14-theme-hierarchy9.log`: six source tests PASS, including all 2,880 resolved nodes, isolated contexts/subthemes, physical units, and unchanged statistical rows/provenance across all nine presets.
- `/private/tmp/gg14-theme-layout-tests5.log`: three private consumer tests PASS for axis physical/sign controls, text margins/font resources/0.9 line height, and source line/arrow controls.
- `/private/tmp/gg14-theme-cascade-final.log`: five layout tests PASS for physical/null panel dimensions at two destination sizes, strip ordering/ontop, multiple-guide direction, inherited versus explicitly equal text defaults, and final flat font/foreground/background cascade.
- `/private/tmp/gg14-theme-focused-final2.log`: legacy colorbar target (17 tests) and hierarchy target (six tests) PASS. Its text-layout failure identified a test-shaper multiplier issue followed by a real prepared-text default handoff issue; the latter is corrected and qualified by source5 above. No expected values were weakened.
- `/private/tmp/gg14-theme-rust-fifth`: checkpoint of sixteen independent Rust authors and 48 SVG/PDF/PNG publications with original-versus-replay scene equality. This predates final border/default/cascade corrections and is not the final acceptance artifact.

Independent authors live in `examples/common/ggplot_theme_controls.rs` and `scripts/bindings/ggplot_theme_controls.{py,cjs}`. The sixteen modes cover nine presets, custom constructor controls, inherited blank reactivation, physical arrow/tick controls, weighted facets and multiple guides, radial labels, explicit supplied fonts with multilingual rotated text, inherited row text, and a styled continuous colorbar. Each host authors its plot independently and checks original/replay scene equality before publication.

## Final gate

The initial checkpoint above has been superseded by the final runtime/native/publication evidence below. Strict declarations and aggregate repository gates remain root-coordinated; this theme-half report does not independently close cumulative GG14 math or workspace gates.

## Native checkpoint

The final theme/furniture native build passed (`/private/tmp/gg14-theme-furniture-native-build2.log`). All sixteen theme windows were captured and individually inspected: `/private/tmp/gg14-theme-native-final/manifest.jsonl` records screenshots and paint traces. The eight-panel furniture gallery was also inspected at `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-16_20-19-34.png`, with manifest `/private/tmp/gg14-furniture-native-final/manifest.jsonl`. Every chart painted with null diagnostic. Presets, weighted panel sizes, multiple guides, blank inheritance, arrows, radial labels, multilingual rotated text and colorbar framing/ticks were visible as authored. All owned processes were closed.

The independent theme publication authors now produce60 files per host:48 ordinary publications plus mode14 Preserve/Outline at300/600dpi acrossSVG/PDF/PNG. Every supplemental request asserts scene equality before exporting. Fresh host equality and publication inspection are recorded below.

## Final publication inspection

All60 Python publications from `/private/tmp/finstack-chart-proof-20260916/gg14-16-18-final/ggplot_theme_controls/python` were inspected, including the four multilingual rotated-text Preserve/Outline300/600dpi variants. SVGs were independently rasterized with the pinned local resvg inspection tool and PDFs with Poppler; ten comparison sheets are under the sibling `inspection/review-1.png` through `review-10.png`. All controls, text and layout were visible as authored. Poppler Splash makes the Linedraw preset's subpixel filled grid contours darker than PNG/SVG; independently rendering the samePDF with Cairo (`/private/tmp/gg14-theme2-cairo.png`) preserves the expected thin grid. This is a rasterizer antialiasing difference, not changed scene widths. All60 SVG/PDF/PNG files are byte-identical across independently authored Rust/Python/WASM outputs; `/private/tmp/gg14-resume.log` reports `PASS ggplot_theme_controls 60 publications/host`, and an independent direct comparison confirms all60. Aggregate repository gates remain root-owned.
