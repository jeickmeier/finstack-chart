# WP-15 — Complete action reducer and state ownership

Status: DONE. Parent: `1cb9557`. Working directory:
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1, 7 September 2026.
Prerequisites WP-07 and WP-14 have actual native/core/extension evidence. This package
owns the synchronous state/reducer, controlled adapter, gesture references, command
history and common native/portable dispatch. Supplemental D3 planning edits are separate.

## Contract evidence

| Requirement portion | Independent acceptance |
| --- | --- |
| INT-01 / INT-06 | Programmatic, control, pointer, keyboard and linked origins reach the same reducer; effective events preserve their origin, duplicate actions emit no event. Shared source identities above 2^53 survive actual Python/WASM dispatch. |
| INT-02 | Distinct viewport/follow/hover/focus/pin/selection/visibility/annotation/gesture/configuration revisions. Stale expected state, reused component revisions and malformed replacements reject without state/history changes. Saved state excludes ephemeral values and preserves current inspection on restore. |
| INT-05 | Exclusive gesture IDs; a 50-preview annotation drag produces one command. Cancel restores committed values and creates no command. Undo/redo, bounded history, invalid previews and competing owners execute without a window. |
| SCN-04 | Resize/new presentation retains the old gesture coordinate basis until cancellation. Weak handles prove release. Input with another scene stamp rejects. Native paint acknowledges the submitted scene. |
| STM-02 | Manual viewport changes enter history mode; explicit freeze retains the presented figure while a real source correction commits. Export bytes remain frozen, resume exposes new data, and replacement presentation releases the frozen figure. |
| QLT-01 | Atomic revision failures, resource caps, owned resource release and terminal disposal at revision exhaustion. Source observations and authored annotation definitions remain unchanged by edits. |

Implementation: [shared contract](../state-action-contract.md),
[ADR-007](../adr/007-actions-gestures-and-controlled-state.md),
[core traces](../../crates/chart-core/tests/actions.rs),
[actual export/restore tests](../../crates/chart-export/tests/actions.rs),
[portable trace fixture](../../fixtures/actions/trace.json),
[native controls](../../examples/chart-gallery/examples/actions_gallery.rs).

## Execution

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-15/fmt.log). |
| `mise run check` | PASS repository isolation/link checks, default/Kit/example builds, Clippy, rustdoc and WASM target check; [log](wp-15/check.log). |
| `mise run test` | PASS **170 tests**: 134 core, 26 export, 5 external-extension, 1 native-conversion, 4 Rustdoc; [log](wp-15/test.log). No feature gate is inferred from zero-test package shells. |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-15/bindings` | PASS all 36 existing cases plus 23 action transitions in Rust, actual PyO3 and Node WebAssembly; [runner](wp-15/bindings.log), [comparison](wp-15/compare.log), [environment](wp-15/environment.txt). |
| `mise exec -- cargo build -p chart-gallery --example actions_gallery --locked` | PASS; [build](wp-15/native-build.log), actual native inspection below. |

The [Rust trace](wp-15/native/actions-state-trace.json),
[Python trace](wp-15/python/actions-state-trace.json) and
[WASM trace](wp-15/wasm/actions-state-trace.json) agree exactly, including effective events,
component revisions, bounded configuration, stale fences and excluded preview values.
Existing semantic/scene tolerances and expected values remain unchanged.
The [preview PNG](wp-15/native/actions-preview.png) and
[cancel PNG](wp-15/native/actions-cancel.png) were visually inspected at 750×500 pixels.
Real-font SVG primitive content is identical for preview/commit/redo and for cancel/undo;
revision metadata differs as expected. Native [commit SVG](wp-15/native/actions-commit.svg),
[Python commit SVG](wp-15/python/actions-commit.svg) and
[WASM commit SVG](wp-15/wasm/actions-commit.svg) are retained with all trace exports.

Initial checks caught an import ambiguity, large action variants, a collapsible condition
and the new durable-component restore fence in an existing test. Imports were made
explicit, annotation payloads boxed, the condition simplified, and the test extended to
reject a reused component revision before accepting its increment. No fixture or tolerance
was weakened. Final gates pass after these fixes. Existing dependency advisories and the
upstream `block` future-compatibility warning retain their previously documented status.

The actual native controls were inspected on the supported macOS/Metal target:
[reset](wp-15/native-ui/reset.jpg), [preview](wp-15/native-ui/preview.jpg),
[cancel](wp-15/native-ui/cancel.jpg), [commit](wp-15/native-ui/commit.jpg),
[undo](wp-15/native-ui/undo.jpg), [redo](wp-15/native-ui/redo.jpg),
[freeze](wp-15/native-ui/freeze.jpg) and [zoom](wp-15/native-ui/zoom.jpg).
The label moves from figure x=0.1 to x=0.7 while committed x stays 0.1 in preview;
cancel restores x=0.1, commit saves x=0.7, undo restores x=0.1 and redo restores x=0.7.
Freeze and resumed manual zoom show their explicit follow modes. Initial native launch
was delayed by the app-control service; the subsequent visible control operations completed.
Native controls use the same reducer but are not pointer-drag/capture-loss certification.

## Remaining gates

WP-15 acceptance does not close G3. WP-16 owns indexed inspection/navigation/selection
producers, pointer capture, Escape/focus loss translation and traversal. WP-17 owns
constraints/snapping, linked chart/table orchestration and complete replaceable controls.
WP-18 owns retention and latest-data/target reconciliation; WP-19 owns scheduling;
WP-20 owns export inclusion policies, full-domain export during previews and concurrent
export budgets. Linux/native accessibility/performance/release checks remain open.
D3 shape/axis/scale supplemental requirements are separate and remain unverified here.

Next: commit WP-15, then WP-16 within the owner assignment.
