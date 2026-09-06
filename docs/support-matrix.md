# Support and evidence matrix

Updated: 6 September 2026. Intended support is not verified product support.
Current infrastructure evidence is recorded in the
[bootstrap report](evidence/bootstrap-2026-09-06.md).

| Surface | Required scope | Current implementation/evidence | Open gate |
| --- | --- | --- | --- |
| Portable Rust core | Synchronous host-independent semantics on macOS/Linux | Empty documented crate; infrastructure checks only | WP-02 onward; G1/G4 |
| macOS Apple Silicon GPUI | First desktop host | Adapter shell; no GPUI dependency selected or native rendering | ADR-001, WP-03/07/21 |
| Linux headless core/export | Core tests and headless use | CI configured; no Linux execution recorded locally | WP-08/21 |
| Browser WASM target | Core compilation; minimal runtime proof | Compilation check provisioned; no runtime binding | WP-09 |
| Optional Kit | Compatible theme/control adapter | Optional gallery feature plus crate shell; no Kit dependency | ADR-001, WP-03/13 |
| SVG/PDF/PNG | Vector marks, explicit fonts/dimensions and publication output | Export shell only; dependencies not selected | WP-03/08/13 |
| Python headless | Minimal batch/correction/action/export equivalence | Rust shell only; no extension or packaging | WP-09 |
| WASM scene/SVG | Minimal actual runtime and ownership proof | Rust shell only; no exported adapter | WP-09 |
| Accessibility | Keyboard equivalence plus verified platform exposure/data alternative | No implementation or platform evidence | WP-03/17/21 |
| Streaming/performance | FIX-08–11/14, PERF-01–05 | No implementation or measurement | WP-18–22 |
| Other desktop platforms | Optional expansion after evidence | Not declared supported | Separate capability work |
| Wheels/viewer/notebooks/browser product | Future distribution/host work | Out of current release scope | Separate plan |

## Feature checks

Current defaults include core/export/standalone GPUI package shells. Kit is opt-in via
the gallery's `kit` feature; binding shells require explicit package selection. No host
packages are third-party dependencies yet. Shell compilation cannot establish renderer,
Kit or binding compatibility. Introduce named valid target/feature checks with each real
adapter; avoid one cross-target `--all-features` success claim.

## Capability evidence to add in WP-03

Record GPUI, SVG, PDF and PNG separately for curved/dashed paths, caps/joins, clips,
gradients, points, rich/rotated text, glyph coverage, font embedding/outline policy,
physical output size, high DPI, overlays/input and native accessibility. Each cell
needs artifact/command/revision evidence or a concrete unsupported/blocked explanation.
The current status for every such capability is UNVERIFIED.
