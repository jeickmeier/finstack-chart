# WP-13 completion evidence — 7 September 2026

WP-13 is **DONE** for full themes, explicit typography and publication composition.
The starting revision is `63dbcc2` (WP-12). This report belongs to the WP-13 completion
commit. The owner assignment remains WP-11 through WP-23, with a commit per package.
WP-14 follows; G2–G4 remain open.

## Contract results

| Requirement | Evidence and result |
| --- | --- |
| THM-01/02/03, FIX-12 | Versioned explicit token cascade and three named presets pass. Theme-only preparation retains identical source/statistical domains, rows, targets and actual table/mark allocations. Mapped colors remain semantic inputs; grayscale converts final paint only. Host/layer/output priority is tested. |
| LAY-02/03/04, FIX-13 | Rich title/subtitle, rotated axis titles/ticks, stable panel letters, caption/source/footnotes, data/panel/figure/output anchors, bounded label collision handling and insets pass independent numerical checks. Insets reuse parent prepared tables and marks. The existing collected/per-panel legend contract remains shared. |
| EXP-01/02/04 | Same captured figure emits vector SVG/PDF, 300/600 DPI PNG and exact-outline native preview. All themes have physical 180 × 120 mm output. Full-domain export clears both state and authored axis viewports. |
| GPU-03 | Native editorial/terminal/grayscale themes and publication preview inspected. Kit semantic host snapshot renders through the generic cascade. Its actual button restores a deliberately bounded x viewport from [0,1] to the full domain via the shared reducer. |
| BND proof extension | Three new cases execute through actual Rust, CPython and Node WASM: 34 total. Semantics match within 1e-12, scenes within 1e-10 points, and SVG bytes match exactly. Additional-font identity/bytes and all composition fields cross the shared wire format. |

[Contract](../theme-typography-composition-contract.md),
[ADR-011](../adr/011-explicit-typography-and-composition.md) and the
[fixture README](../../fixtures/composition/README.md) specify ownership, precedence,
font resources, collision behavior and bounded paint capabilities. `chart-text` is the
shared explicit-byte adapter service; no text/native/I/O dependency enters core.

## Commands and runtime evidence

Environment: macOS 26.5.2 arm64, Rust 1.97.1, CPython 3.14.6, PyO3 0.29.2,
Node 24.14.0, wasm-bindgen 0.2.128. See [environment](wp-13/environment.txt) and
[module/artifact hashes](wp-13/module-sha256.json).

| Command/check | Result |
| --- | --- |
| `mise run fmt` | PASS, [log](wp-13/fmt.log). |
| `mise run check` | PASS repository policy, core/native/export build isolation, optional Kit, Clippy, rustdoc and core WASM; [log](wp-13/check.log). |
| `mise run test` | PASS **148 tests**: 126 core, 19 export, 1 native conversion, 2 Rustdoc examples; [log](wp-13/test.log). Six new integration tests exercise shared typography/composition service behavior. Zero-test package shells are not acceptance evidence. |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-13/bindings` | PASS 34 cases per actual runtime; [runner](wp-13/bindings.log), [comparison](wp-13/compare.log). Existing checks/tolerances retained. |
| `mise exec -- cargo run -p chart-export --example composition_proof --locked` | PASS three themes, 102 scene items, two panels and one inset each; [log](wp-13/artifacts.log). |
| `mise exec -- cargo build -p chart-gallery --example family_gallery --example publication_preview --features kit --locked` | PASS native and optional Kit; [build](wp-13/native-build.log). `--composition` selects fixtures; temporary app executables were named `composition_gallery`. |
| Native/UI + `pdfinfo`, `pdffonts`, `pdfimages`, `pdftotext`, `pdftoppm` | PASS inspected native themes, preview, kit control, PDF raster, fonts/text, vector/image checks and physical dimensions; [inspection](wp-13/inspection.json). |

Independent text expectations include six tabular digits at 10 points = 34.32 points
from the font's hmtx widths, actual weight validation, contextual Arabic glyphs and
UTF-8 cluster coverage, resource revision rejection, missing glyph/fallback behavior,
rotated anchors, French grouping/decimal output and exact dash boundaries across turns.
Tests reject invalid theme versions, fonts, oversized furniture and dash expansion.
The small Arabic fixture intentionally reports one declared fallback warning and no
layout-pressure diagnostic.

Two issues were corrected during validation: a rustdoc `[0,1]` link needed literal
formatting, and the expanded comparison runner needed a local name for WP-11 scenes
so it did not overwrite the complete case map. Initial glyph-count assumptions were
replaced with independent contextual-glyph/cluster assertions after inspecting the
actual supplied font. Per-glyph PDF text emission inserted an unwanted Arabic gap;
complete positioned-run encoding fixed it. No pre-existing fixture or tolerance changed.

## Visual/vector/font inspection

Native captures: [editorial](wp-13/native-ui/editorial.jpg),
[terminal](wp-13/native-ui/terminal.jpg), [grayscale](wp-13/native-ui/grayscale.jpg),
[publication preview](wp-13/native-ui/publication-preview.jpg),
[kit bounded view](wp-13/native-ui/kit-before-reset.jpg),
[kit reset](wp-13/native-ui/kit-after-reset.jpg). All were actually inspected. A direct
sandboxed GUI launch could not access macOS GUI services; the approved native launch
outside that shell sandbox ran successfully and enabled the recorded kit control check.

Publication artifacts under [wp-13](wp-13/) include all three SVGs, PDFs, 300/600 DPI
PNGs, exact preview SVGs, and semantic/scene JSON. The editorial PDF was rasterized and
inspected alongside the theme PNGs and GPUI preview. Text/caption/notes, panel letters,
rotated labels, gradients, dash phase and inset are coherent. Native interactive layout
uses its own viewport dimensions; the publication preview uses the exact fixed physical
layout and positioned outlines. These are deliberately distinct measurement routes.

Every PDF has three embedded/subset Unicode Noto faces and no image objects; Arabic
copy text contains the exact complete authored run. SVG advanced runs retain Unicode
metadata with vector outlines; ordinary labels preserve embedded-font text. PNG sizes
are **2126 × 1417** at 300 DPI and **4252 × 2835** at 600 DPI. PDF dimensions are
**510.236 × 340.157 points**. Neither zero-test crates nor uninspected images were used
to close a feature gate.

## Limits and next action

SVG preserve mode reports `MixedPositionedOutlines` when advanced runs are present;
those labels are not editable SVG text elements. Rich text uses authored per-run
direction/language and explicit whole-run fallback, not automatic paragraph bidi layout.
Gradients support two axis-aligned sRGB stops; dashes support bounded polylines, with
curves requiring explicit flattening. The existing core resource/geometry budgets apply.

Linux runtime, full accessibility, high-density interaction, streaming, release artifacts,
performance and dependency advisory disposition remain assigned to later packages.
The existing `block` future-compiler warning persists. G2 requires WP-14 and the alpha
feature matrix; no cumulative milestone is closed here. **Next: WP-14 extension contracts
and alpha API**, preserving this committed WP-13 boundary.
