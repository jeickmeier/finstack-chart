# Support and evidence matrix

Updated: 6 September 2026. Intended support is not verified product support.
Current grammar/prepared-geometry evidence is recorded in the
[WP-05 completion report](evidence/wp-05-completion-2026-09-06.md); native/export capability
evidence remains in the [WP-03 report](evidence/wp-03-completion-2026-09-06.md). Dependency/build
selection remains in the [WP-01 report](evidence/wp-01-completion-2026-09-06.md), with
earlier infrastructure checks in the [bootstrap report](evidence/bootstrap-2026-09-06.md).

| Surface | Required scope | Current implementation/evidence | Open gate |
| --- | --- | --- | --- |
| Portable Rust core | Synchronous host-independent semantics on macOS/Linux | Data and initial grammar compiler implemented; 59 core tests and 2 Rustdoc examples pass on macOS, including FIX-01/02 prepared geometry and composite layers; scale/layout and full statistical families remain pending | WP-06 onward; G1/G4 |
| macOS Apple Silicon GPUI | First desktop host | Pinned GPUI/platform 0.3.3; actual vector capability fixture inspected on macOS 26.5.2 arm64 at scale 2; input, remount, resize and teardown exercised; public chart renderer pending | WP-07/21 |
| Linux headless core/export | Core tests and headless use | Target dependency isolation verified; CI configured; no Linux execution recorded locally | WP-08/21 |
| Browser WASM target | Core compilation; minimal runtime proof | Implemented core including data/transactions/grammar compiles for `wasm32-unknown-unknown`; target graph isolation verified; no runtime binding | WP-09 |
| Optional Kit | Compatible theme/control adapter | Kit 0.6.0 input/buttons exercised against the same GPUI identity; optional dependency preserved; chart theme/control adapter pending | WP-13/17 |
| SVG/PDF/PNG | Vector marks, explicit fonts/dimensions and publication output | Actual text/outline SVG/PDF and 300/600 DPI PNG inspected; explicit embedded/subset fonts, physical sizes and zero PDF image objects verified; candidate dependencies dev-only | WP-08/13 |
| Python headless | Minimal batch/correction/action/export equivalence | Rust shell only; no extension or packaging | WP-09 |
| WASM scene/SVG | Minimal actual runtime and ownership proof | Rust shell only; no exported adapter | WP-09 |
| Accessibility | Keyboard equivalence plus verified platform exposure/data alternative | Native input/button/status/image hooks observed and keyboard actions exercised; no screen-reader/data-alternative certification | WP-17/21 |
| Streaming/performance | FIX-08–11/14, PERF-01–05 | Ordered atomic data operations and count retention pass data-only FIX-08/09 subsets; ADR-008 starting profile retained; queues/time windows/incremental chart computation and PERF cases unverified | WP-18–22 |
| Other desktop platforms | Optional expansion after evidence | Not declared supported | Separate capability work |
| Wheels/viewer/notebooks/browser product | Future distribution/host work | Out of current release scope | Separate plan |

## Feature checks

Current defaults include core/export and the standalone GPUI adapter shell. Kit is opt-in
via the gallery's `kit` feature; binding shells require explicit package selection.
ADR-001 records exact host dependencies; actual host examples establish linked GPUI/Kit
type compatibility. The macOS `check` task builds both variants and builds/lints the
Kit capability example; Linux tasks compile and test core/export. Actual native and
publication proof artifacts are separate WP-03 evidence. Avoid one cross-target
`--all-features` success claim.

The repository checker now resolves three explicit target graphs for core/export host
isolation. WP-02 also verifies finite geometry, bounded immutable scenes and synchronous
services with deterministic test doubles. WP-04 adds data validity/revisions/provenance,
bounded replay, count retention and snapshot-sharing/lifetime checks. WP-05 adds typed
authoring, identity/explicit-bin outputs, endpoint geometry, transform reuse and minimal
viewport/visibility actions. These are data-space preparations, not destination rendering.
Full primitive/font/export and binding parity
cannot be inferred from foundation tests or the small capability fixture.

The dependency advisory check currently fails on six unmaintained transitive packages;
see the refreshed WP-03 report. Native future-compiler compatibility also remains open for `block`
0.1.6. These are tracked release risks, not failed chart fixture results.

## Capability evidence and remaining paths

[ADR-003's capability matrix](adr/003-font-and-renderer-capability-route.md) records
native and export results for the fixed primitives, typography, clips, gradients,
dimensions and controls, with concrete paths for unsupported capabilities.
[The completion report](evidence/wp-03-completion-2026-09-06.md) links actual artifacts,
inspection and commands. General rendering/diagnostics, destination measurement bridges,
all themes, multi-panel/tight bounds, live snapshots and full platform accessibility
remain unverified until their owning packages implement and exercise them.
