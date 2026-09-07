# WP-11 required family fixtures

[portable-cases.json](portable-cases.json) is the single canonical catalog used by native
Rust export, Python, real Node WebAssembly and the standalone native family gallery.
The [contract](../../docs/scale-geometry-contract.md) specifies defaults and limitations.

The 12 cases cover log-invalid line gaps, signed symlog, point/discrete color, area gaps,
ribbon gaps/reversed bounds, numeric-color heatmap/null style, fixed-slot grouped bars,
signed stacks, OHLC with independently invalid volume, alternate units, supplied session
gaps and UTC leap-day boundaries. They extend the existing 12 statistics cases.

Run `mise run bindings-proof artifacts/wp-11` with the matching `wasm-bindgen` 0.2.128 CLI
on PATH or in `WASM_BINDGEN`. The proof compares exact identities, original values, operation
metadata and targets; numeric semantics use 1e-12 and scenes use 1e-10 points. SVG bytes
must match across all three runtimes. Independent family expectations are in
[scripts/bindings/compare.py](../../scripts/bindings/compare.py); existing WP-09/10 assertions
and tolerances are preserved. Rust emits SVG/PDF/PNG for each family; PDF checks reject
image objects. These files are evidence inputs, not self-updating visual baselines.

Run `mise exec -- cargo run -p chart-gallery --example family_gallery --locked` for native
vector painting. An optional numeric argument selects the initial catalog entry. Buttons
switch among the same portable fixtures; no example-specific chart calculation is used.
Actual captures and the validation environment are recorded in the
[WP-11 report](../../docs/evidence/wp-11-completion-2026-09-07.md).
