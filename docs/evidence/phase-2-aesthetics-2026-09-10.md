# GG-03 independent aesthetic encodings

Date: 10 September 2026. Revision: `a6caa39` plus the retained authorized working-tree
changes. Requirements: GG2-03 / FIX-GG03. Prerequisites: accepted GG-02, SP-04 and
WP-S05. Status: **COMPLETE for GG-03**. G-GGPLOT and G-PARITY remain open.

## Delivered contract

Fill and stroke have independently trained color encodings and prepared legend
metadata. Alpha replaces embedded alpha; legacy opacity multiplies it. Equivalent
circle area, legacy radius and linewidth remain separate. Constants suppress their
active mappings, and ggplot constant linewidth removes the corresponding training
input. Post-scale fill/stroke/alpha/linewidth join the existing snapshot-based size
and color outputs. Solid ggplot line segments retain each start row's style after
sorting; variable styles on non-solid groups reject. LibraryV1 retains constant-run
styling. Text and line-type channels reuse the typed scale engine and preserve typed
values and exact trained descriptors; actual data-driven text geometry remains GG-08.

R glyphs 0–25 share WP-S05's path generator. Outlined marks and dashes use the existing
scene/path hit engine with exact source anchors. Millimeter and point dimensions
convert at the destination boundary. Version 16 covers the added descriptors and
reference glyphs while absent fields retain older envelopes. The architectural choice
and resolved-style memory increase from 24 to 56 bytes are recorded in
[ADR 014](../adr/014-phase-2-integration-contract.md).

## Evidence

| Check | Result and boundary |
| --- | --- |
| Pinned R reference | R 4.6.1 r90187 and ggplot2 4.0.3 produce [14 built-layer cases and 104 glyph records](../../fixtures/parity/ggplot2/aesthetics.json), with [source and lock hashes](../../fixtures/parity/ggplot2/aesthetics-manifest.json). The existing uncompressed R PDF device supplies glyph paths; extraction does not implement the glyph formulas. Coordinate tolerance is 0.011 publication points for two-decimal device output. |
| Core regressions | Twelve aesthetic tests pass on macOS: independent paints and trained metadata, immutable edits/version 16, area ratio 1:4 versus legacy 1:16, outlined hits, alpha semantics, physical units/dash-gap hits, typed text, sorted segment styles, theme precedence and source reductions. Manual paints, constant paints, embedded alpha and linewidth compare directly with the pinned R builds. All 104 glyph topology/extent/paint-mode cases pass on Linux. |
| Actual hosts | Rust independently authors the 26-glyph publication. Python 3.14.6 and actual WASM/Node each execute 104 copy/disposal glyph cases, independent paints/alpha/metadata, area/radius ratios, source-expression text values, invalid inputs, version-16 round trips and immutable edits. Earlier Linux Python 3.11 execution also passed the scoped host proof. |
| Updates and identity | Each host passes eight append/upsert/remove/retention states across single and faceted figures. Independent fill/stroke/alpha/area/linewidth and glyph changes match fresh PNG output, preserve old captured output and retain source keys above 2^53. Complete mark geometry, paints and anchors agree across Python/WASM. |
| Publication | Independently authored Rust/Python/WASM glyph SVG and PNG are byte-identical. Python/WASM independent-paint SVG and PNG are byte-identical. PDF is separately rendered with Poppler and inspected; no PDF-byte equality claim is made. |
| Visual/native | Inspected the 26-glyph PNG, rasterized PDF and actual GPUI window. All glyphs are visible; open/solid/outlined modes and source target cardinality are correct. Initial gallery clipping was corrected with explicit domain margins before the final artifacts. The native accessibility tree reports 26 visible targets. |
| Static declarations | Strict mypy and TypeScript accept the new channels, reference glyph descriptors, units and source-expression inputs. The existing primary-authoring runner now executes the scoped GG-03 proof and declarations. |
| macOS workspace | `CARGO_TARGET_DIR=target/axis-rust-proof mise run test`: 487 tests/doctests pass, zero failed or ignored. |
| Repository and Linux | `CARGO_TARGET_DIR=target/axis-rust-proof mise run check` passes. Offline Linux core/export/text: 468 tests/doctests pass, zero failed or ignored. [Check log](phase-2-aesthetics/logs/check.log), [test counts](phase-2-aesthetics/logs/test-counts.json), [environment](phase-2-aesthetics/environment.json). |

The 14 R build cases include policies owned by GG-04; this package does not claim
reference automatic palette or default size-scale equivalence from those captured
records. GG-04 owns palette/scale/zero policies, GG-05 full multi-aesthetic guide
composition, GG-08 text drawing and GG-18 broader integration combinations. G-GGPLOT
and G-PARITY remain open.

An earlier macOS executable and temporary R capture library stalled in the system
loader before user code. No protection or system setting was changed. The existing
R PDF device and offline Linux tools allowed progress; the final macOS core suite,
Python extension and native app subsequently ran successfully.

Retained artifacts: [native capture](phase-2-aesthetics/native.png),
[glyph PNG](phase-2-aesthetics/rust/glyphs.png), [glyph PDF](phase-2-aesthetics/rust/glyphs.pdf),
[inspected PDF raster](phase-2-aesthetics/wasm/glyphs-pdf.png),
[independent paints](phase-2-aesthetics/python/independent.png),
[host comparison](phase-2-aesthetics/comparison.json), [artifact hashes](phase-2-aesthetics/manifest.json).
The native glyph capture preceded the final theme-precedence guard; that guard has
separate core regressions and refreshed host publication evidence. The primary runner
was extended, but only the scoped new proof stages were rerun for this package.

Next: GG-04 palette and scale policies on the accepted GG-03/SP-06/CP-04/CLR-04 interfaces.
