# WP-07 completion evidence — 6 September 2026

Verdict: **DONE for WP-07 standalone GPUI vertical slice**.
Assigned scope: WP-07 only, following WP-06. Baseline is WP-06 at `f8657fb`; result is that
baseline plus the uncommitted native adapter, shared inspection, gallery, tests and documents.
Requirements: GPU-01/02, SCN-03/04, INT-01/03, QLT-01; FIX-01/07/18 supported subsets.
No commit or PR was created. No existing fixture expectation, tolerance or baseline changed.

## Acceptance and actual native evidence

| Deliverable | Verdict | Evidence |
| --- | --- | --- |
| Compiled native marks, GPU-01/SCN-03 | PASS for initial line/point/bar | Standalone gallery paints core destination Scene quads/paths and real GPUI shaped text. Line gaps remain open; scatter shows the same 18 non-null observations; histogram uses all 21 x observations. No raster image is used as the chart. |
| Retained state, GPU-02 | PASS for this bounded slice | One gallery entity plus one chart entity, no per-mark entities or store construction in Render. `NativeMetrics` reports layout attempts unchanged at 4 while paints advance 29 → 33 through hover. No timing/FPS/sustained-load claim. |
| Presented-state coherence, SCN-04 | PASS for synchronous slice | Inspector owns the exact `Arc<LaidOutChart>` and source/stat snapshot. Publication follows successful paint, pointer coordinates subtract presented bounds, and tooltips are prepared only for that same frame. Replacement clears obsolete inspection. |
| Input, INT-01/03 | PASS for basic shared actions | Actual mouse hover and arrow-key focus use the core reducer. Scatter radius/nearest target, nearest-x line groups, rectangular bin containment, keyboard wrap/clear, event origin, idempotency and stale-stamp rejection have core tests. Caller tooltip resolves source values and aggregate members. |
| FIX-01 subset | PASS | Native line gaps, scatter and explicit histogram inspected. Core tests retain exact target keys and known bin membership; zoom preserves upstream counts/members. Full fixture families remain later work. |
| FIX-07 subset | PASS | Existing supported-scale tests plus native resize, 20 × 20 no-space and UTC large-origin nanoseconds inspected. Final keyboard tooltip shows `2024-02-29 00:00:00.000000017Z` for observation 2 while axes use integer-origin fractional ticks. No logarithmic/market-calendar claim. |
| FIX-18 subset | PASS | Native supplied Noto Sans regular face supplies both measurement and paint. Missing font resource 999 reports `MissingResource` while retaining the prior chart. Real plain text, margins and finite chart geometry inspected; full rich/rotated typography and fallback remain WP-13. |
| Invalid input/disposal, QLT-01 | PASS for exercised paths | Missing field 999 returns `SchemaConflict`, preserving the chart. Tiny bounds publish `NoSpace` with zero candidates. Five consecutive remounts (3–7 in log) release both old entity and old layout after two frames. Window close logs and process exits 0. |

Screenshots were captured and visually inspected in the actual macOS GPUI window:

- [Line with gaps and keyboard focus](wp-07/line-keyboard.png).
- [Nearest scatter source tooltip](wp-07/scatter.png).
- [Histogram aggregate tooltip](wp-07/histogram.png) and [zoomed bin counts](wp-07/histogram-zoom.png).
- [Final UTC exact-source keyboard tooltip](wp-07/utc.png).
- [Final missing-font recovery](wp-07/missing-font.png) and [invalid-input recovery](wp-07/invalid-input.png).
- [Tiny bounds](wp-07/tiny-bounds.png) and [resized scatter](wp-07/resized-scatter.png).

[Interaction/lifecycle log](wp-07/native-interaction.log) retains the complete native session,
including repeated releases, hover reuse, resize and close. Resize changes layout attempts
1 → 2, paints 3 → 6 and layout revision 1 → 2; all 18 candidates and store revision 0 remain.
The resized window capture is approximately 803 × 571 logical pixels. Histogram edges
[0,4,8,12,16,20] produce [4,4,4,4,5] counts, with viewport [4,16] preserving the population.
The source is normalized once; y is absent at x = 8, 9, 10, so lines have a real gap.

The [final rebuilt session](wp-07/native-final.log) verifies the exact UTC tooltip and the
final `MissingResource` classification after font ownership validation was tightened.
The earlier session's font diagnostic was `InvalidResource`; its code was corrected,
and the missing-font and UTC screenshots were recaptured. Other captures cover unchanged
paint/input behavior. The first plain sandbox launch could not connect to WindowServer;
an approved local graphical launch succeeded. That launch restriction was environmental.

## Implementation and public use

[Core inspection](../../crates/chart-core/src/inspection.rs) is synchronous and independent
of GPUI. Candidates derive from the presented geometry and explicit clips; scatter uses a
bounded positive radius, line hits group one nearest x without inventing interpolation,
and keyboard targets use visible anchors. Equal-distance groups never merge different x
positions. Histogram targets remain aggregates with complete source membership.
[Five core tests](../../crates/chart-core/tests/inspection.rs) assert those semantics using
independent known positions/keys, including clipped marks and stale action atomicity.

[The native adapter](../../crates/gpui-charts/src/native.rs) bridges core text metrics and
Scene primitives to GPUI. Binary32 conversion rejects non-finite values and error above
0.25 logical pixel (one native unit test). [The view](../../crates/gpui-charts/src/view.rs)
retains validated inputs, pending/presented frames and focus, and exposes data/definition,
layout and shared action methods. Failed preparation preserves the previous frame; input
is disabled when that old frame's bounds no longer match. Next-frame callbacks use weak
entities. Tooltip elements are caller-owned and have no implicit export representation.

Run `mise exec -- cargo run -p chart-gallery --locked` in a macOS graphical session.
[The gallery](../../examples/chart-gallery/src/main.rs) is the working public API example:
load supplied font bytes before the host resolves that family, construct `ChartInput`, mount
one `ChartView`, provide tooltip content and use retained update/action methods. Arrow keys
inspect, Escape clears; controls exercise zoom/restore, malformed mappings, missing fonts,
no-space, remount and report. No optional Kit dependency is required.

Font registration is host-owned and must precede family resolution. Reserve supplied faces
under that family for the adapter; identical descriptor/byte registration is idempotent,
while another resource under that family rejects. GPUI caches resolved font selections, so
arbitrary external/prior registrations are outside this contract. Missing/control glyphs
and shaped font substitutions reject. See [ADR-003](../adr/003-font-and-renderer-capability-route.md).
The already locked `ttf-parser` 0.25.1 is now a normal native dependency; no dependency
version or publication dependency graph changed, and core remains dependency-free.

## Commands and environment

Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Environment: [environment.txt](wp-07/environment.txt); macOS 26.5.2 arm64, Rust 1.97.1,
GPUI/platform 0.3.3. Native window inspected at display scale 2.

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-07/fmt.log) |
| `mise exec -- cargo build -p chart-gallery --locked` | PASS; [log](wp-07/build.log) |
| `/tmp/FinstackNativeCharts.app/Contents/MacOS/chart-gallery` | Actual native sessions above; temporary app bundle points to the built `target/debug/chart-gallery`; successful final close exits 0 |
| `mise run check` | PASS: repository/target isolation, local links, licenses/sources, formatting, host examples, workspace compilation, strict Clippy/rustdoc, core WASM compilation; [log](wp-07/check.log) |
| `mise run test` | PASS: 89 core tests (2 unit + 87 integration), 1 native conversion test and 2 core Rustdoc examples; no ignored/failed tests; [log](wp-07/test.log) |
| `mise exec -- python3 scripts/check_repository.py` and `git diff --check` | PASS after documentation; [log](wp-07/final-check.log) |

## Remaining boundaries and next action

This is single-panel initial grammar/native interaction, not a full specification gate.
There is no spatial index, full selection model, pan/gesture arbitration, controlled state,
streaming scheduler, shared panels, rich/rotated text or full typography. The chart exposes
an image description and exercises keyboard behavior; no full accessibility tree, data
alternative or screen-reader certification is claimed. Native failures are recoverable for
the validated inputs exercised here, not proof against arbitrary platform renderer faults.

No native Linux/other-platform execution, hosted CI, production performance/RSS claim or
binding runtime is established. Export remains the WP-03 proof implementation until WP-08.
Six known unmaintained dependency advisories and the `block` 0.1.6 future-compiler warning
remain open. The parser promotion does not waive successor/maintenance assessment in
WP-08 or release disposition in WP-23. License/source checks pass separately from advisories.

Next: **WP-08 headless export** is ready; WP-09 still needs WP-08. G1–G4 remain open.
This assignment stops at WP-07.
