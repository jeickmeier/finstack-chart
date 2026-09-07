# Headless publication fixture (WP-08)

`support.rs` builds a real core line/point chart with a missing-y gap, numeric axes and
one literal annotation. It supplies the unchanged OFL Noto Sans resource from
[the capability fixture](../capability/README.md), SHA-256
`2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9`.
No installed fonts, GPUI event loop or filesystem access are used by the export library.
Only the example reads its output argument and writes files.

```sh
mise exec -- cargo run -p chart-export --example publication_export --locked -- docs/evidence/wp-08
mise exec -- python scripts/check_publication_artifacts.py docs/evidence/wp-08
mise exec -- cargo run -p chart-gallery --example publication_preview --locked -- docs/evidence/wp-08/publication-preview.svg
```

The Python checker needs Poppler (`pdfinfo`, `pdffonts`, `pdfimages`, `pdftotext`). Native
preview needs the supported macOS graphical session. The preview consumes the already
positioned vector/glyph paths, uniformly scales the point viewBox and does not remeasure text.

Independent expectations: 180 × 120 mm equals 510.2362204724 × 340.1574803150 pt;
300/600 DPI yield 2126 × 1417 / 4252 × 2835 pixels and 11811/23622 pixels/metre.
Noto Sans digit advance is 572/1000 em: `012345` at 10 pt measures 34.32 pt.
Input (x,y) pairs are (0,1), (0.5,3), (1,missing), (1.5,2), (2,4): two line segments
and four points, with no connection across the gap. Axis bounds are x=[0,2], y=[1,4].
The checked projection uses 30 pt padding, 10 pt measured labels, 4 pt ticks and 4 pt
label gaps; coordinate tolerance is 0.0001 pt. PDF dimensions allow 0.002 pt because
Poppler prints three decimals. PNG density and dimensions compare exactly.

[Integration tests](../../crates/chart-export/tests/publication.rs) also cover immutable
capture during concurrent updates, visible/full-domain bins, resource lifetime,
font permissions, missing glyphs, precision/budgets, XML escaping, curves, empty clips
and straight-alpha pixels. These are minimal FIX-13/14 subsets; full figure composition,
rich typography and sustained live-export behavior remain WP-13/20.

[Completion evidence](../../docs/evidence/wp-08-completion-2026-09-06.md) separates automated
checks from actual SVG/PDF/PNG/native inspection. Do not regenerate artifacts to conceal
an unexplained mismatch. Export bytes and metadata are not a WP-09 portable wire envelope.
