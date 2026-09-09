# Phase 2 retained path acceptance — 8 September 2026

WP-P01–04 implement PTH-01–06 and FIX-P01–06 against d3-path 3.1.0.
Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`, with uncommitted Phase 2 changes.
The [manifest](phase-2-paths/manifest.json) records the validated source/artifact hashes;
later Phase 2 changes require their own affected checks. The [representation decision](../adr/015-path-authoring-and-replay.md)
records typed adaptations, state, precision, version and exact identity contracts.

## Implemented contract

One core owned builder implements every constructor and all eight reference methods.
Immutable numeric geometry retains analytic circular arcs and signed rectangle edges.
Atomic operations/batches reject non-finite arguments, negative radii, arithmetic
overflow and exhausted command/operation budgets. SVG digits never change geometry.
External replay stops at sink failure and preserves the accepted external prefix.
Caller-supplied layer, clip, stamp and derived consumer metadata remain independent
of controls/subdivisions; fixed annotations have no source targets.

Primary `vector_path` annotations, Python/WASM standalone handles and their primary
builders use the same core. Scene, primary and normalized definition interchange
use version two when retained paths occur; old definitions remain version one.
Owned copies, edits and retained snapshots survive later mutation/disposal.
Legacy Bézier transport and its validation/defaults retain their previous contract.

Embedded-font SVG preserves analytic arcs. PDF/PNG receive shared bounded cubic
geometry before SVG parsing; native GPUI receives numeric commands directly.
The circular Hermite bound is `sqrt(2) * r * angle^4 / 384` per segment; a conservative
affine norm converts destination tolerance into source tolerance. Publication reserves
half its quarter-pixel budget for curve conversion and half for f32 coordinates;
native uses explicit physical-pixel conversion and tessellation budgets. Unrepresentable
coordinates and excessive subdivisions fail rather than silently reducing precision.
Curved dashes remain outside d3-path's methods and under their existing style owner.

## Validation

| Evidence | Result and scope |
| --- | --- |
| Pinned offline [corpus](../../fixtures/parity/d3-path/cases.json) and [manifest](../../fixtures/parity/d3-path/manifest.json) | 86 deterministic operation sequences, exact command/state topology and diagnostics; numerical trigonometric comparison at `2e-12 * max(1, abs(expected))` coordinate units. Canonical rounding/scientific cases compare exact strings. Regeneration into a separate directory matched the committed corpus. |
| `cargo test -p chart-core --test path_parity --test path_lowering --locked` | Eight tests: complete corpus, independent tangency, both-direction multi-radius sampled error bounds, affine reflection/shear, all 84 non-finite argument cases, limits/atomicity, version migration, supplied metadata and immutable scenes. |
| `scripts/run_path_proofs.py target/path-proof/final` | Actual Rust/Python/Node WASM: all 86 traces and complete primary numeric scenes/metadata; owned copies, disposal, invalid versions, failures, batches and external replay. Eighteen SVG/PDF/PNG exports across hosts at 300/600 DPI. |
| `mise run fmt`; `mise run check` | PASS formatting, repository/dependency rules, builds, Clippy, documentation and core WASM checks. |
| `mise run test` | PASS 261 tests in 51 macOS suites, no ignored tests, before the final additive metadata test; the final eight-test path run includes that test. |
| Linux core/export test run | PASS 255 tests in 37 suites, no ignored tests, before the final additive metadata test. Actual Linux arm64 `rust:1.97.1-bookworm`, Docker network disabled, source/registry mounted read-only, separate writable target. |
| `scripts/run_primary_authoring_proofs.py target/path-proof/primary-final` | PASS actual primary Rust/Python/WASM families, actions, streaming and type consumers, including path authoring; expected five negative typing errors retained. |
| `scripts/run_binding_proofs.py target/path-proof/bindings-final` | PASS all existing capability proofs plus complete path runtime comparison. |
| Native `path_authoring` example | Actual GPUI paint stamps and inspected [window](phase-2-paths/native.png), supplied Noto Sans, 450×300 logical chart at device scale two. |
| Publication inspection | Inspected [300-DPI PNG](phase-2-paths/rust/figure-300.png) and [600-DPI PDF rendering](phase-2-paths/pdf-600-preview.png). All six PNG sizes and 30 independent interior/clip samples pass. PDF inspection reports embedded subset Noto Sans and no raster images. |

All runtimes use Rust 1.97.1; macOS arm64 uses Python 3.14.6, Node 24.14.0 and
wasm-bindgen 0.2.128. Full commands/runtime versions and outputs are retained in
[logs](phase-2-paths/logs/final-paths.log) and the manifest.
Visual checks cover opposite-winding circular/rectangular holes, tangent corners,
closed joins followed by continued strokes, quadratic/cubic curves, affine ellipse,
empty/move-only paths, supplied text and edge clipping. No visual baseline was replaced.

The affine case exposed a real JSON decoding precision defect: the default serde_json
parser changed a control coordinate by one ULP. Enabling `float_roundtrip` on the
existing pinned dependency fixes exact interchange; the regression keeps its original
expected coordinate. Separately, platform trigonometry can differ by a few ULPs.
Cross-host computations use the declared geometry tolerance while revision assertions
use exact definition differences. Rebuilding within each runtime preserves identity.
No approximate equality was introduced into core definition/revision checks.

**G-PATH passes for this retained source snapshot.** WP-S01 may consume this shared
builder, serializer and lowering route. Shape semantics, curved style extensions,
AP-09/native performance, WP-22 measurements and cumulative G-PARITY/G-GGPLOT remain
with their existing packages. The next active work is GG-02 and the remaining
dependency-ordered Phase 2 packages.
