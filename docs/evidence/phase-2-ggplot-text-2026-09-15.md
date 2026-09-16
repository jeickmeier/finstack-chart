# GG-08 implementation evidence — 15 September 2026

Implementation covers source/generated row labels, rotated/padded label boxes,
justification, overlap control, five pinned reference size units, common position
nudging, explicit font catalogs, vector annotations and portable nearest/interpolated
RGBA annotations. See [ADR-025](../adr/025-row-text-and-portable-raster.md).

The shared `LayerBuilder.text_geom`, `text_label`, `text_stat_label` and `annotation`
methods are exposed through Python/WASM scalar authoring. `ValueAesthetic` owns mapped
label/style values; no host computes labels, statistics or coordinates. Definition
v73 and raster scene v20 are owned by the common version detectors. Text inspection
uses the rotated whole-label polygon and exact source/generated targets. Bounds,
resource, text and output/sample limits apply independently of the 256 fixed-annotation
limit.

Focused evidence:

- `tools/reference/r/text-marks.R` captures nine built text populations, generated
  counts and all five text unit factors from R 4.6.1 / ggplot2 4.0.3.
- `cargo test -p chart-core --test ggplot_text_marks --locked`: six focused tests
  passed, including final whole-label inspection targets (log `/tmp/gg08-test.log`).
- `cargo check -p chart-export -p gpui-charts --locked`: passed.
- `cargo run -p chart-export --example ggplot_text_marks --locked -- /private/tmp/gg08-rust`:
  seven independent authors, 21 standard publications and eight additional outlined
  SVG/PDF publications passed (log `/tmp/gg08-export.log`). All 29 files initially
  matched actual Python/WASM bytes without normalization. Eleven standard/outline
  image comparisons were inspected using independent SVG and PDF rasterizers; the
  nearest-mode issue found there was corrected as described below. The integration
  checkpoint reruns hosts against that correction. Final Rust rendering is inspected
  in `/private/tmp/gg08-rust-inspection/review-1.png` through `review-4.png`. Only
  `text-4.svg` and `text-4.png` differ from the initial host files; all other 27
  publications remain byte-identical. The corrected nearest raster now has sharp
  matching cells in PNG, independently rasterized SVG and PDF.
- `scripts/bindings/ggplot_text_marks.py` and `.cjs` independently author all seven
  modes and check source/stat labels, replay and raster interpolation metadata.
- `ggplot_text_marks_native` built and painted four charts successfully, recorded in
  `/tmp/gg08-native-build.log` and `/tmp/gg08-native.log`. Native text, padded rotated
  labels, nearest pixels and interpolated pixels were inspected in
  `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_19-31-09.png`.
  The owned process was closed after inspection.
- `cargo test -p chart-export --test ggplot_raster --locked` passes an actual PNG
  sampling regression. Initial SVG `pixelated` was ignored by the pinned renderer;
  `optimizeSpeed` now selects nearest filtering, verified by a pure red interior
  sample where the smooth mode mixes red and green. This is a behavior correction,
  not a visual-baseline replacement.
- `pdftotext` on the rotated preserved PDF retains all Alpha/béta/Delta characters
  (rotation changes extraction line breaks); the outlined PDF has no text content.

No whole-project, Linux, performance or release gate is implied by these focused
checks. Source-built label contents and unit policies are verified; supplied-font
shaping is validated in actual destination evidence, not the deterministic core
service double. PDF/native/resvg interpolation kernels are not claimed pixel-identical.
The root task owns final ledger acceptance and the integrated command/artifact record.
