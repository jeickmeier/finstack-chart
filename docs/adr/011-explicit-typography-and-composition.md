# ADR-011 — Explicit shared typography and presentation composition

Status: Accepted, 7 September 2026. Implements WP-13 and extends ADR-003/005.

## Decision

Keep theme/figure definitions and numeric positioned glyph runs in `chart-core`. Add
`chart-text` as a small explicit-byte service shared by native and export adapters,
using exact harfrust 0.12.0 and ttf-parser 0.25.1 packages already present in the lockfile.
Core does not acquire text-engine, GPUI, I/O or system-font dependencies. Declare every
fallback face by immutable identity/revision; retain logical text with actual glyphs.

Rich/rotated SVG runs use exact outlines with Unicode metadata even under preserve mode,
and report mixed text representation. PDF keeps searchable complete shaped runs and
embedded explicit faces. This avoids relying on viewer font selection or assuming
per-glyph PDF emission reconstructs RTL clusters. A publication preview paints the exact
export outlines. Ordinary native text retains the established native destination service.

Themes and furniture are presentation inputs. A bounded last-root compiler cache reuses
numeric marks/tables when all semantic inputs remain identical. Insets reuse the parent
population and never execute a statistic on its cropped viewport. Theme precedence,
annotation collision rules, budgets and vector paint limits are explicit in the
[contract](../theme-typography-composition-contract.md).

## Consequences and evidence

Advanced SVG labels are vector but not editable text elements; complete automatic bidi
paragraph layout is outside the authored-run contract. Font resources must accompany
figures. Two-stop axis-aligned gradients and bounded polyline dashes are supported;
unsupported paint capabilities fail instead of rasterizing. Named presets and Kit host
tokens cannot change statistics. Source and host tests verify table/mark identity, and
actual Rust/Python/WASM, PDF/font/image and native evidence is in the
[WP-13 report](../evidence/wp-13-completion-2026-09-07.md).
