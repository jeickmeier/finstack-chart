# WP-AX03 geometry acceptance — 9 September 2026

Status: COMPLETE for AXIS-04, FIX-19-F and the geometry portions of FIX-19-G/I.
Revision: working tree over `a6caa39`. The owner authorized axis, final interpolation
certification and hierarchy; subsequent packages remain active in the ledger.

The core implements signed independent inner/outer lengths, combined size, padding,
explicit/host-supplied pixel offsets, resolved-range domain paths and caps, four-side
anchors, offset-aware band/point rounding and independent guide translation. Semantic
values and mark mapping remain owned by the existing scale. D3 geometry defaults and
label preservation are explicit; legacy unconfigured guides keep existing behavior.
Optional version-13 geometry travels through primary Rust/Python/WASM builders and
portable round trips. Layout separates Preserve, HideLabels and ThinTicks, plus cell
overflow and inward-grid clipping, with aggregate finite/resource checks.

## Validation

- All 372 pinned d3-axis cases / 376 states pass exact values/order/labels and 1e-9
  absolute geometry tolerance in Rust, including four-side paths, rules, text anchors,
  custom/reversed ranges, signed/zero lengths and padding, offsets 0/0.5, both device
  profiles and narrow rounded bands. Actual fresh Python and Node/WASM adapters replay
  the same cases and independently assert semantic records and tick positions.
- Seven core axis tests pass, including explicit resets, invalid inputs, last-valid
  immutable scenes, repeated labels, independent adaptive policies, clipping, shared
  facets, resize and translation without mark changes. The additional guide/provider/
  layout regressions pass. The macOS core/export/text/external-extension regression
  command passes **437 tests/doctests**, zero failures.
- Strict TypeScript positive/negative directives and mypy positive geometry consumers
  pass. Fresh native/Python/WASM builds, focused strict Clippy and formatting pass.
- The signed/reversed Rust example retains version-13 round-trip SVG/PDF/PNG, all
  inspected. External SVG inspection supplies the committed Noto Sans font; Poppler
  renders the PDF independently. A retina native window was inspected and retained
  with paint instrumentation. Deliberately negative top padding intersects the domain
  line as specified; lower translated repeated labels remain distinct.

## Reproduction and evidence

[Retained logs, source hashes, reference records and images](phase-2-axis-geometry/)
include `validation.json`, `source-sha256.json`, `core-oracle.log`, `focused.log`,
`macos-tests.log`, `python.log`, `wasm.log`, type/lint/build logs, `native.png` and
`rust/geometry.svg`, `.pdf`, `.png` plus external inspection images.

```sh
cargo test -p chart-core --test axis_guides --test axis_providers --test axis_ticks --test layout --locked
cargo test -p chart-core -p chart-export -p chart-text -p chart-extension-example --locked
cargo run -p chart-export --example axis_geometry_proof --locked -- target/axis-geometry/rust
python3 scripts/bindings/axis_ticks.py PYTHON_MODULE OUTPUT/python
node scripts/bindings/axis_ticks.cjs WASM_MODULE OUTPUT/wasm
cargo build -p chart-gallery --example axis_geometry --locked
```

Use the pinned mise Rust 1.97.1, Python 3.14.6 and Node 24.14.0. Fresh module generation
uses the existing official local wasm-bindgen CLI **0.2.128**; the default PATH binary
is 0.2.122 and its first generation attempt correctly rejected the incompatible schema.
No dependency, oracle, tolerance or legacy baseline was weakened.

This package does not certify component roles/styles, timed updates, Linux execution
or the complete high-DPI/font/publication matrix. Those are required by WP-AX04–06.
G-AXIS and final interpolation certification remain open until that evidence is complete.
