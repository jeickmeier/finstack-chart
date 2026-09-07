# WP-16 — Hit testing, navigation and selection

Status: DONE. Parent: `127fe2d`. Completed 7 September 2026 in
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1.
Prerequisites WP-11 and WP-15 were complete. The owner's assignment remains the
original WP-11–23 scope; the owner explicitly leaves expanded Phase 2 acceptance open.
Concurrent parity/review documentation is preserved outside this package's commit.

## Contract evidence

| Requirement portion | Executed evidence |
| --- | --- |
| INT-03 / SCN-04 | Dense spatial and sorted-x queries agree with scan results. Nearest-x preserves gaps and provenance; point radius, clipped bars, candle wicks, paint priority and bounded results have independent expectations. A wick whose center is outside the clip remains hittable where its stroke is visible. |
| INT-04 | Point/series/range/rectangle/lasso selections retain stable source/aggregate/derived targets; centers versus bar intersection, disabled custom targets, add/toggle and rejection without truncation are checked. |
| INT-05 / FIX-10 portion | Navigation and selection pin their presented basis through previews and resize/new presentation; Escape, release outside, capture/focus loss and cancellation preserve committed state. Annotation editing/link cycles remain WP-17. |
| INT-06 | Default native producers are focus scoped and replaceable; shared portable queries preserve exact target semantics. Keyboard order deduplicates compound marks and restores stable identity across rebuilds, clearing removed focus. |
| SCL-01 | Pointer-anchored zoom, pinned pan, region/range navigation and reset use linear/log/symlog, exact UTC, supplied-session and typed category windows. Legacy primary-axis actions replace named windows coherently; cancellation restores them. |
| STM-05 | Index and target-availability construction occur at presentation. Clones share immutable indexes; hover retains the prepared Arc and performs no statistics/layout compilation. Focused timing is below; full scheduling/performance remains WP-19/22. |

Implementation details and limits: [interaction contract](../interaction-contract.md),
[state contract](../state-action-contract.md), [ADR-007](../adr/007-actions-gestures-and-controlled-state.md).
The [shared cases](../../fixtures/interaction/cases.json) drive Rust/Python/WASM.

## Execution

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp16/fmt.log). |
| `mise run check` | PASS repository links/dependency isolation, native/Kit/examples, Clippy, rustdoc and WASM check; [log](wp16/check.log). |
| `mise run test` | PASS **183 tests**: 145 core, 28 export, 5 external-extension, 1 native-conversion and 4 Rustdoc cases; [log](wp16/test.log). |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-16/bindings` | PASS 36 existing cases, 23 action transitions and **22 shared input steps** in actual Rust/PyO3/Node WASM; [runner](wp16/bindings.log), [comparison](wp16/compare.log), [environment](wp16/environment.txt). Existing semantic/scene tolerances and SVG equality remain unchanged. |
| `mise exec -- cargo build -p chart-gallery --example interaction_gallery --locked` | PASS; [build log](wp16/native-build.log). Native execution/inspection below. |
| `mise exec -- cargo run -p chart-core --example interaction_benchmark --release --locked` | PASS exact query equivalence and unchanged prepared Arc; [log](wp16/benchmark.log), [JSON](wp16/benchmark.json). |

All input traces and SVG/PNG outputs are retained under
[the evidence directory](wp16/). Native UI inspection used the
[interaction gallery](../../examples/chart-gallery/examples/interaction_gallery.rs)
on macOS/Metal with the pinned GPUI 0.3.3 dependency. Inspected captures:
[pointer zoom](wp16/native-pointer-zoom.png), [pan](wp16/native-pan.png),
[region zoom](wp16/native-region-zoom.png), [range selection](wp16/native-range-selection.png),
[rectangle selection](wp16/native-rectangle-selection.png),
[keyboard selection](wp16/native-keyboard-selection.png), [preview](wp16/native-preview.png),
[Escape cancellation](wp16/native-escape-cancel.png),
[release outside](wp16/native-release-outside.png),
[focus-loss cancellation](wp16/native-focus-loss-cancel.png),
[category window](wp16/native-category-window.png) and [reset](wp16/native-category-reset.png).
The focus-loss check exposed a missed native callback; render-time focus observation
now also cancels and releases ownership. Native captures precede the final off-clip wick
regression, which is checked by the final ten-test inspection suite and full runtime gates.

## Focused index comparison

One release run, 100,000 presented candidates and 512 deterministic queries per geometry.
These are elapsed totals, not per-query percentiles or a release performance gate.
The deterministic count improvement is reproducible; wall times depend on the machine.

| Geometry | Build ms | Indexed query ms | Scan query ms | Indexed candidate tests | Scan candidate tests |
| --- | ---: | ---: | ---: | ---: | ---: |
| scatter | 45.619 | 0.653 | 218.655 | 4,693 | 51,600,000 |
| line | 44.426 | 0.563 | 611.448 | 5,230 | 102,400,000 |

Target-index build and 512 reducer hover dispatch totals are recorded separately in JSON.
The reducer timing uses one effective hover followed by identical no-ops; it does not
measure moving-hover allocation, event delivery or input-to-paint latency. No source
statistics are recomputed. Heavy overlap can still require linear candidate work.

## Open gates and next action

WP-16's assigned selection/navigation acceptance is complete. Full FIX-09 retention/pin
reconciliation remains WP-18; linked echoes and annotation editing complete FIX-10 in
WP-17. Live overlay export, scheduling and complete PERF workloads remain WP-19/20/22.
No Linux runtime, full accessibility/screen-reader, release packaging or G3/G4 claim is
made here. Existing dependency advisories and the upstream `block` warning remain open.
The added Phase 2/D3/ggplot2 requirements retain their separate owners and acceptance.
Next: WP-17 linked views, constrained annotation editing and host controls.
