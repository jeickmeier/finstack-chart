# WP-AX05 transition qualification — 9 September 2026

Status: COMPLETE for AXIS-06/FIX-19-H and the transition portion of FIX-19-I.
Revision: working tree over `a6caa39`; source/runtime fingerprints are retained.
[WP-AX06](phase-2-axis-certification-2026-09-09.md) owns cumulative certification.

## Implementation and contracts

`GuideTransitionPlan` owns bounded D3 joins and explicit samples. Raw new-scale positions
are join keys; numeric zero follows D3 key equality, duplicates retain the first old node,
remaining duplicates exit, and enters receive new lifecycle identities. A linked index
list reproduces D3 update ordering and enter insertion without a quadratic tick scan.
The immutable frame retains semantic values, selected indices, logical labels, continuous
opacity and domain geometry. Domain cap changes pair numeric tokens by ordinal in the
new compact command pattern, including at fraction zero. Shared `ScalarInterpolator`
and `TransformInterpolator` own numeric and translation sampling.

`LayoutGuideTransition` combines the plan with actual laid-out primitives. Data marks
use the current target. Enter/update labels change immediately; exits retain old labels
and styles. Interruption uses displayed positions/opacity for existing groups and the
previous target scale for entering positions. Completed scenes retain exact target
primitives and stable lifecycle identities. Facet/inset scopes remain separate.
New/removed guides and side replacement are immediate; resource/unit changes require
immediate presentation. A sampled scene adds optional group metadata (wire v15);
static scene/authoring compatibility remains unchanged.

Native `ChartInput::guide_transition(Duration)` opts into one plan and host clock;
`ChartView::set_guide_transition` changes the duration. GPUI reduced motion completes
immediately. Frozen gestures stop the active animation; disposed preparation drops it.
Core contains no clock, browser scheduler or entity per tick. Native paint uses GPUI
continuous primitive alpha; no cross-renderer pixel-identical compositing claim is made.
SVG/PDF use group opacity, and PNG consumes that SVG representation.

`FigureSnapshot::guide_transition(previous)` compiles an owned `FigureTransition`;
`sample(fraction)` exposes it through Rust/Python/WASM without relayout. Captured figures
outlive input/output/transition handles. `Chart.acknowledge_frame` admits an explicitly
rendered sample only while definition, source, viewport, visibility and annotation state
still match. `basis("displayed")` freezes its exact primitives, including exits, with one
scene unit per point and an equal-size viewport. Reflow/full-domain capture is separate:
existing Presented capture rebuilds from acknowledged inputs at publication dimensions.

## Evidence completed

- Pinned Chromium 151.0.7922.34 and locked D3 packages generated
  [41 timed cases](../../fixtures/axes/transitions.json) and a separate
  [provenance manifest](../../fixtures/axes/transitions-manifest.json). Real D3 public
  transitions use a controlled performance/RAF clock. Two generations were byte-identical.
  Static AX01/02 fixtures were not modified. The matrix includes four sides, 24 exhaustive
  update reorderings with exits, duplicate/projected collisions, caps, offset changes,
  bands, empty selection, immediate labels, interruption and explicit side replacement.
- Core offline tests pass **125 timed samples**, using shared numeric/band scales for
  positions and captured formatter results as label inputs. A scene-level test checks
  painted positions, opacity, current data marks, exact final primitives, interruption
  mapping and retained final identity. Invalid fractions and resource limits reject.
- Three publication tests pass: text/outline SVG, PDF and PNG at start/mid/end;
  interrupted publication; exact displayed capture and stale-frame rejection; separate
  facet/inset joins with empty decoration targets.
- Fresh real Python/WASM handles passed independently authored inputs, retained owner
  lifetimes, invalid fractions, disposal, stale acknowledgement and displayed capture.
  [75 artifacts](phase-2-axis-transitions/host-comparison.json) agree across Rust/Python/WASM:
  JSON metadata/labels exact with 1e-9 absolute coordinate tolerance; SVG/PDF/PNG bytes
  identical. Text/outline PNG outputs match. Final rebuilds pass.
- Strict TypeScript positive/negative directives and mypy consumers pass.
- The actual native run records moving/fading ticks and an interruption, immediate
  reduced-motion completion with preserved identities, and disposal. A moving sample
  is acquired at approximately eight seconds, then exported after completion/disposal.
  The retained sample still contains exits and fractional opacity. Latest trace:
  [native-trace.jsonl](phase-2-axis-transitions/native-trace.jsonl); latest run reports
  four layout attempts, 1086 paint submissions, and no diagnostics. Submission count is
  not an FPS/performance certification.
- Inspected [native running image](phase-2-axis-transitions/native-running-final.png), publication
  PNGs and independently rasterized SVG/Poppler PDF mid/interruption outputs show expected
  positions, immediate new labels and faded exits/enters. Final native-captured SVG/PDF/PNG were also inspected; source/runtime fingerprints
  and exact acquired-versus-exported metadata validation are retained.

## Commands and remaining qualification

Completed focused commands: `cargo test -p chart-core --test axis_transitions --locked`,
`cargo test -p chart-export --test axis_transitions --locked`, the
`axis_transition_proof` example, actual `axis_transitions.py` / `.cjs` proofs, and the
shared `axis_components_compare.py ... transitions` comparator. The pinned generator is
`tools/reference/node/axis-transitions.mjs`. All use the committed lockfile.

Python requires `extension-module,extension-proof`; an initial extension-proof-only build
failed at import due to Python library linkage. Rebuilding with the correct extension
feature succeeded. Type checks use the already installed mypy/TypeScript runtimes.

Final qualification passes **449 macOS workspace tests/doctests** and **448 offline
Linux core/export/text/external-extension tests/doctests**, zero failures. The complete
`mise run check` passes repository/dependency/format/all-target check/Clippy/rustdoc and
standalone WASM core checking. Its first run exposed two unescaped interval doc links;
those documentation-only errors were fixed and the complete task passed on retry.

Fresh actual Python/WASM and Rust samples again pass all 75 artifact comparisons.
The native trace validator checks 89 acknowledged samples (28 moving first-transition,
59 interruption/final samples), immediate reduced motion and exact retained capture
metadata after disposal. Native/SVG/PDF/PNG visual inspection passed. Reproduction:
`python3 scripts/check_axis_transition_trace.py docs/evidence/phase-2-axis-transitions`.
`native-validation.json`, `source-sha256.json`, `runtime-sha256.json`, macOS/Linux logs and
`repository-check-final.log` preserve evidence. No benchmark/FPS or release gate follows
from paint-submission counts. Native inspection uses the available device-scale-two
macOS display; Linux acceptance is headless.
