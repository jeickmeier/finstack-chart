# Support and evidence matrix

Updated: 7 September 2026. Intended support is not verified product support.
Current family/native/runtime evidence is in the [WP-11 report](evidence/wp-11-completion-2026-09-07.md).
Prior statistical/position and portable runtime evidence is in the
[WP-10 report](evidence/wp-10-completion-2026-09-06.md), extending the
[WP-09 report](evidence/wp-09-completion-2026-09-06.md),
with the precise [proof-support matrix](portable-contract.md#proof-support-matrix). G1 is passed
for its minimal subset; G2–G4 remain open. Headless publication/snapshot evidence is in the
[WP-08 report](evidence/wp-08-completion-2026-09-06.md). Standalone native chart evidence is in the
[WP-07 report](evidence/wp-07-completion-2026-09-06.md). Scale/destination-layout evidence is recorded in the
[WP-06 completion report](evidence/wp-06-completion-2026-09-06.md), following
[WP-05 grammar evidence](evidence/wp-05-completion-2026-09-06.md); native/export capability
evidence remains in the [WP-03 report](evidence/wp-03-completion-2026-09-06.md). Dependency/build
selection remains in the [WP-01 report](evidence/wp-01-completion-2026-09-06.md), with
earlier infrastructure checks in the [bootstrap report](evidence/bootstrap-2026-09-06.md).

| Surface | Required scope | Current implementation/evidence | Open gate |
| --- | --- | --- | --- |
| Portable Rust core | Synchronous host-independent semantics on macOS/Linux | Data, built-in statistics/positions and required scale/geometry families and layout implemented; 117 core tests and 2 Rustdoc examples pass on macOS, including FIX-01–05 statistical/geometry subsets and WP-06 FIX-07 scope; actual binding and export consumers exercised; facets/themes/extensions remain pending | WP-12 onward; G2/G4 |
| macOS Apple Silicon GPUI | First desktop host | Pinned GPUI/platform 0.3.3; standalone compiled line/point/bar/UTC scenes inspected on macOS 26.5.2 arm64 at scale 2; real fonts, keyboard/hover, remount, resize, failed updates and teardown exercised | Full capabilities WP-12–21 |
| Linux headless core/export | Core tests and headless use | Target dependency isolation verified; CI configured; no Linux execution recorded locally | WP-21 |
| Browser WASM target | Core compilation; minimal runtime proof | Core/export compile for `wasm32-unknown-unknown`; the actual wasm-bindgen module executes in single-threaded Node WebAssembly with scene/SVG, exact data and lifetime proofs | Full coverage WP-14/21 |
| Optional Kit | Compatible theme/control adapter | Kit 0.6.0 input/buttons exercised against the same GPUI identity; optional dependency preserved; chart theme/control adapter pending | WP-13/17 |
| SVG/PDF/PNG | Vector marks, explicit fonts/dimensions and publication output | Actual text/outline SVG/PDF and 300/600 DPI PNG inspected; explicit embedded/subset fonts, physical sizes and zero PDF image objects verified; public immutable shared-core capture and inspected native publication preview; 13 export tests pass, including shared binding session ownership | WP-13/20 |
| Python headless | Minimal batch/correction/action/export equivalence | Actual PyO3 0.29.2 / CPython 3.14.6 module: shared fixture plus 24 WP-10/11 statistics/position/family cases, correction/action/state/export, detached interpreter and owned/disposed resources; full packaging remains pending | Full coverage WP-14/21 |
| WASM scene/SVG | Minimal actual runtime and ownership proof | Actual wasm-bindgen 0.2.128 / Node 24.14.0 module: shared fixture plus 24 WP-10/11 statistics/position/family cases and exact SVG, exact integers/times, forced memory growth, returned copies, repeated actions and disposal | Full coverage WP-14/21 |
| Accessibility | Keyboard equivalence plus verified platform exposure/data alternative | Native input/button/status/image hooks observed and keyboard actions exercised; no screen-reader/data-alternative certification | WP-17/21 |
| Streaming/performance | FIX-08–11/14, PERF-01–05 | Ordered atomic data operations and count retention pass data-only FIX-08/09 subsets; ADR-008 starting profile retained; queues/time windows/incremental chart computation and PERF cases unverified | WP-18–22 |
| Other desktop platforms | Optional expansion after evidence | Not declared supported | Separate capability work |
| Wheels/viewer/notebooks/browser product | Future distribution/host work | Out of current release scope | Separate plan |

## Feature checks

Current defaults include core/export and the standalone GPUI chart adapter. Kit is opt-in
via the gallery's `kit` feature; binding proof adapters require explicit package selection.
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
viewport/visibility actions. WP-06 adds linear/band/UTC scales, named domains, explicit clips,
finite destination scenes and bounded plain-text layout with deterministic measurement doubles.
WP-07 adds native plain-text shaping/painting and inspected chart captures with retained-source
inspection. WP-08 adds immutable physical publication capture, headless formats and a vector
preview through the same layout. These subsets do not establish complete output fidelity.
WP-09 executes the shared portable fixture in actual Rust/Python/WASM runtimes. WP-10
adds count/automatic bins/summary/OLS and stack/normalize/dodge/jitter, with 14 semantic
tests and 12 actual cases per runtime; generated schemas/provenance and final position
scenes agree, and every new SVG matches byte-for-byte. Exact full recomputation after
data updates is verified; this does not certify incremental streaming performance.
Complete primitive/font/export and built-in binding parity cannot be inferred from these
minimal fixtures. The separate `bindings-proof` task runs the actual host comparisons;
ordinary Cargo checks alone are not runtime evidence.

The dependency advisory check currently fails on six unmaintained transitive packages;
see the refreshed WP-09 report. Native future-compiler compatibility also remains open for `block`
0.1.6. These are tracked release risks, not failed chart fixture results.

## Capability evidence and remaining paths

[ADR-003's capability matrix](adr/003-font-and-renderer-capability-route.md) records
native and export results for the fixed primitives, typography, clips, gradients,
dimensions and controls, with concrete paths for unsupported capabilities.
[The completion report](evidence/wp-03-completion-2026-09-06.md) links actual artifacts,
inspection and commands. The basic native Scene renderer, plain-text measurement bridge and failure recovery are
implemented in WP-07. Full themes, multi-panel layout, rich destination typography, sustained live exports
and full platform accessibility
remain unverified until their owning packages implement and exercise them.

WP-11 adds 12 portable family cases, all inspected as actual native GPUI vector charts,
plus SVG/PDF/PNG output. Eight additional focused core tests verify scales, area/ribbon
gaps, color invariance, secondary units and independent price/volume validity. The
[family contract](scale-geometry-contract.md) describes remaining limitations; no other
platform, screen-reader or performance gate is inferred from these macOS results.
