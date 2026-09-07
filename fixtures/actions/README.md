# Action trace fixture

`chart.json` uses the existing binding data/profile/font with one authored annotation.
`trace.json` supplies 23 authored transitions, expected event/state subsets and stale-input
errors. Rust, the actual PyO3 extension and Node WebAssembly consume it unchanged.
Requests only fill the current state fence and acknowledged scene stamp dynamically.
No expected values are generated from the implementation.

The comparison runner requires exact state/event parity and equal rendered SVG geometry
for preview/commit/redo and for cancel/undo, excluding revision metadata. Core tests add
resize/pinned-basis, many-preview single-commit, explicit cancellation reasons, resource caps and disposal cases.
