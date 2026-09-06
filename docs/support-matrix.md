# Support and evidence matrix

Updated: 6 September 2026. Intended support is not verified product support.
Current dependency/build evidence is recorded in the
[WP-01 completion report](evidence/wp-01-completion-2026-09-06.md); earlier infrastructure
checks remain in the [bootstrap report](evidence/bootstrap-2026-09-06.md).

| Surface | Required scope | Current implementation/evidence | Open gate |
| --- | --- | --- | --- |
| Portable Rust core | Synchronous host-independent semantics on macOS/Linux | Empty documented crate; infrastructure checks only | WP-02 onward; G1/G4 |
| macOS Apple Silicon GPUI | First desktop host | Pinned `gpui-pre` / platform 0.3.3; standalone host example compiles and links on macOS 26.5.2 arm64; no chart/native visual evidence | WP-03/07/21 |
| Linux headless core/export | Core tests and headless use | CI configured; no Linux execution recorded locally | WP-08/21 |
| Browser WASM target | Core compilation; minimal runtime proof | Empty core compiles for `wasm32-unknown-unknown`; no runtime binding | WP-09 |
| Optional Kit | Compatible theme/control adapter | Kit 0.6.0 host example compiles and links against the same GPUI identity; no chart theme/control implementation | WP-03/13 |
| SVG/PDF/PNG | Vector marks, explicit fonts/dimensions and publication output | Export shell only; dependencies not selected | WP-03/08/13 |
| Python headless | Minimal batch/correction/action/export equivalence | Rust shell only; no extension or packaging | WP-09 |
| WASM scene/SVG | Minimal actual runtime and ownership proof | Rust shell only; no exported adapter | WP-09 |
| Accessibility | Keyboard equivalence plus verified platform exposure/data alternative | No implementation or platform evidence | WP-03/17/21 |
| Streaming/performance | FIX-08–11/14, PERF-01–05 | No implementation or measurement | WP-18–22 |
| Other desktop platforms | Optional expansion after evidence | Not declared supported | Separate capability work |
| Wheels/viewer/notebooks/browser product | Future distribution/host work | Out of current release scope | Separate plan |

## Feature checks

Current defaults include core/export and the standalone GPUI adapter shell. Kit is opt-in
via the gallery's `kit` feature; binding shells require explicit package selection.
ADR-001 records exact host dependencies; actual host examples establish linked GPUI/Kit
type compatibility only. The macOS `check` task builds both variants; Linux tasks compile
and test core/export. No renderer, theme, binding or other platform capability follows
from these builds. Avoid one cross-target `--all-features` success claim.

The dependency advisory check currently fails on six unmaintained transitive packages;
see the WP-01 report. Native future-compiler compatibility also remains open for `block`
0.1.6. These are tracked release risks, not failed chart fixture results.

## Capability evidence to add in WP-03

Record GPUI, SVG, PDF and PNG separately for curved/dashed paths, caps/joins, clips,
gradients, points, rich/rotated text, glyph coverage, font embedding/outline policy,
physical output size, high DPI, overlays/input and native accessibility. Each cell
needs artifact/command/revision evidence or a concrete unsupported/blocked explanation.
The current status for every such capability is UNVERIFIED.
