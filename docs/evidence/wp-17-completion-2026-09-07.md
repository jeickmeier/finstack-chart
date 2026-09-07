# WP-17 — Linked views, editable annotations and host controls

Status: DONE. Parent: `4148793`. Completed 7 September 2026 in
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1.
Prerequisites WP-13/15/16 were complete. Scope is the owner's original WP-11–23
assignment, with one commit per package. Expanded parity and authoring work remains
separate; concurrent documentation is preserved outside this commit.

## Executed contract evidence

| Requirement portion | Evidence |
| --- | --- |
| INT-01/04, DAT-06 | Links map exact source keys across layers 1/99, preserve epoch and aggregate input revisions, retain clipped/hidden selection, map numeric/category windows, reject incompatible meaning, and suppress echoes/repeated revisions. The native table toggles the same observation in both charts. |
| INT-05, FIX-10 | Ten focused core tests cover snapping, locked axes, pinned resize basis, ordered ranges, exact nanoseconds/categories, connector positions and preview/commit/cancel/one-entry undo. Native keyboard/pointer edits, Escape and focus-loss cancellation were executed and inspected. |
| INT-06, GPU-03, LAY-03 | Replaceable toolbar/legend/context-menu/tooltip factories and explicitly enabled copy/export operations use retained scene context and common actions. Native toolbar interception was discovered and fixed by stopping overlay input propagation. Copy, export, legend and menu actions then worked. |
| INT-04/06 accessibility | Bounded shared descriptions preserve exact keys/values, units, missing values and aggregate provenance. Native tree exposes summaries, table cells, selection and editable annotation values. OS focus and untested screen-reader limits are explicit below. |

See the [host tools contract](../host-tools-contract.md),
[ADR-007](../adr/007-actions-gestures-and-controlled-state.md),
[ten focused tests](../../crates/chart-core/tests/host_tools.rs), and
[shared host-tool fixtures](../../fixtures/host-tools/cases.json).
No source row changes during the action/query proofs; annotations use existing history.

## Validation

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp17/fmt.log). |
| `mise run check` | PASS repository/dependencies, formatting, native examples/Kit, Clippy, rustdoc and WASM; [log](wp17/check.log). |
| `mise run test` | PASS **193 tests**: 155 core, 28 export, 5 external-extension, 1 native-conversion, 4 Rustdoc; [log](wp17/test.log). |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-17/bindings` | PASS actual Rust/PyO3/Node WASM: 36 existing cases, 23 action transitions and **47 shared input steps**, including 25 new host-tool steps; [runner](wp17/bindings.log), [comparison](wp17/compare.log), [environment](wp17/environment.txt). |
| `mise exec -- cargo build -p chart-gallery --example host_tools_gallery --locked` | PASS; [log](wp17/gallery-build.log); actual native run inspected below. |

Existing tolerances were retained: input traces use 1e-12 numerical comparison; existing
scene comparisons remain 1e-10 pt. All input SVG bytes match across hosts. Independent
checks require preview/commit/redo geometry equality and cancel/undo restoration. The
last edit after the binding proof only stopped native overlay event propagation; native
build, runtime inspection and full fmt/check/test were repeated after that edit.
Retained traces, new host-tool SVGs and native PNGs are in [wp17](wp17/).

## Actual native inspection

The [host tools gallery](../../examples/chart-gallery/examples/host_tools_gallery.rs)
ran on macOS/Metal with GPUI 0.3.3. Inspected captures include:

- [Linked table selection](wp17/native-ui/table-linked-selection.png),
  [keyboard target description](wp17/native-ui/keyboard-accessible-target.png), and
  [keyboard linked selection](wp17/native-ui/keyboard-linked-selection.png).
- [Keyboard threshold edit](wp17/native-ui/annotation-keyboard.png),
  [undo](wp17/native-ui/annotation-undo.png), [redo](wp17/native-ui/annotation-redo.png),
  [snapped threshold drag](wp17/native-ui/threshold-pointer.png), and
  [range endpoint drag](wp17/native-ui/range-pointer.png).
- [Held preview](wp17/native-ui/held-preview.png),
  [Escape cancellation](wp17/native-ui/escape-cancel.png), and
  [focus-loss cancellation](wp17/native-ui/focus-loss-cancel.png).
- [Copy one selected observation](wp17/native-ui/host-copy.png),
  [successful export](wp17/native-ui/host-export.png),
  [replaceable legend](wp17/native-ui/host-legend.png), and
  [context menu](wp17/native-ui/host-context-menu.png); its copy action also executed.

The exported [PNG](wp17/native-host.png) was visually inspected: five observations,
exact horizontal threshold/range and supplied-font labels. The [SVG](wp17/native-host.svg)
is retained. The example explicitly chooses a 480 by 300 pt export and supplied font;
this is not yet the complete WP-20 live-overlay export policy.

The [accessibility tree](wp17/native-ui/annotation-accessibility.txt) exposes meaningful
chart/table labels and the edited threshold value. Keyboard inspection and selection
work. The OS inspector nevertheless reports the focused UI element as the window,
despite GPUI chart focus/active-descendant wiring. No VoiceOver speech or complete
screen-reader traversal was tested, and no OS accessibility settings were changed.
Linux native/accessibility remains unverified. These are actual supported-target limits,
not a claim of full screen-reader conformance.

## Remaining gates

WP-17's original acceptance and FIX-10 linked/interrupted-editing portions pass.
Retention/follow reconciliation remains WP-18, scheduling WP-19, coherent live exports
WP-20, and sustained performance/release certification later packages. G3/G4 and expanded
parity/authoring gates stay open. Known dependency advisories and the upstream `block`
future-compatibility warning remain unresolved. Next: WP-18.
