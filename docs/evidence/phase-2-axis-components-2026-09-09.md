# WP-AX04 component acceptance — 9 September 2026

Status: COMPLETE for AXIS-05 and the styling/publication portions of FIX-19-G/I.
Revision: working tree over `a6caa39`; source/runtime hashes distinguish this package
from later changes. The ledger owns cumulative axis and interpolation certification.

## Contract

`GuideComponents` independently configures domain stroke/visibility, tick line
stroke/visibility and label color/font size/typography/rotation. Bounded per-tick
overrides use original selected indices; invalid widths, dash patterns, typography
and duplicate override indices fail through the shared validation boundary. Owned
input copies, reset and version-14 round trips execute through all three primary APIs.

Core scenes carry guide identity, panel/inset scope, side, role, selection index,
logical label and value/occurrence metadata. Equal signed zeros retain distinct
occurrences. Decorations retain empty data-target lists. Component hiding preserves
selected values in the coherent guide snapshot. The existing text shaper, paint
resolver and stroke/dash lowering remain the single owners. Old unconfigured
LibraryV1 scenes omit the new metadata and retain their earlier output/version.

Text SVG exposes axis/tick groups, domain paths, tick line elements and logical label
roles. Outline SVG preserves that structure and takes its glyph paths from the
already positioned usvg tree, including logical groups for transparent text. Rich
per-tick runs retain supplied resource/weight/scale/rotation policy and explicitly
report positioned outlines under text mode. External CSS changes affect that SVG
only. Native/PDF/PNG consume the same resolved scene styles.

## Evidence

- **443 macOS core/export/text/external-extension tests and doctests passed**, zero
  failures. Ten focused core tests include the full 372-case / 376-state geometry
  replay, styling, signed-zero identities, facet scopes and bounded failures. Three
  dedicated export tests check SVG roles/groups, transparent outlined labels and
  per-tick typography through the supplied shaper.
- Actual rebuilt Python 3.14.6 and Node 24.14.0/WASM independently author the styled
  plots, check retained scenes and run owned-input/reset/error cases. **26 artifacts
  agree across Rust/Python/WASM**: exact semantic labels/values/IDs/roles, 1e-9 absolute
  geometry tolerance for JSON, and byte-identical SVG/PDF/PNG. Text/outline PNG bytes
  are identical. WASM explicitly authors floating columns to match Rust/Python input
  schema; integer inference was detected by the initial strict comparison and the
  fixture input was corrected, without weakening the expected contract.
- Strict TypeScript positive/negative directives, mypy positive consumers, all-target
  core/export Clippy with warnings denied, formatting and fresh native builds pass.
- Native component and typography galleries were inspected at device scale two.
  SVG text/outline outputs were independently rendered with the supplied Noto Sans
  font; Poppler text/outline PDFs and 300/600 DPI PNGs were inspected. Domain dashes,
  inward tick dashes/alpha, independent font size/color, hidden top domain, repeated
  labels and the rotated middle label are visible as declared. The simple typography
  figure retains data endpoint clipping at the plot edge. No baseline was regenerated.

## Reproduction

[Retained evidence](phase-2-axis-components/) includes `validation.json`, source and
runtime SHA-256 manifests, `macos-tests.log`, `final-core.log`, `publication.log`,
`python.log`, `wasm.log`, `compare.log`, strict type/lint/build logs, independently
produced artifacts and inspected images. Use mise Rust 1.97.1 and the matching local
wasm-bindgen CLI 0.2.128 for actual modules.

```sh
cargo test -p chart-core --test axis_components --test axis_ticks --locked
cargo test -p chart-export --test axis_components --locked
cargo run -p chart-export --example axis_component_proof --locked -- OUTPUT/rust
python3 scripts/bindings/axis_components.py PYTHON_MODULE OUTPUT/python
node scripts/bindings/axis_components.cjs WASM_MODULE OUTPUT/wasm
python3 scripts/bindings/axis_components_compare.py OUTPUT
cargo test -p chart-core -p chart-export -p chart-text -p chart-extension-example --locked
cargo clippy -p chart-core -p chart-export --all-targets --locked -- -D warnings
```

The primary authoring proof runner includes the geometry/component steps; this package
ran its individual steps, not every historical package in the aggregate script.
Linux and cumulative supported-platform/reference certification remain WP-AX06.
WP-AX05 must still provide timed enter/update/exit, interruption, reduced motion and
coherent capture. G-AXIS and G-INTERPOLATE remain open.
