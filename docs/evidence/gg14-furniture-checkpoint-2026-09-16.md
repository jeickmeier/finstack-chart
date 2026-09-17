# GG-14 figure furniture checkpoint

Revision `295e7a0ec2e9145b140c838751b615ee40e9f64a` plus shared worktree, macOS arm64, Rust 1.97.1. GG-14 remains open pending integrated theme/math and actual-host/native qualification.

`plot.title.position` and `plot.caption.position` use the measured panel union or full plot span. Global tags use explicit rich text, named/numeric positions and plot/panel/margin locations. Blank tags reserve no space. Numeric margin positions reject. These use existing rich shaping, physical theme margins and figure publication layout. Explicit per-panel lettering remains its separate existing capability. Figure composition version3 and capability wire80 preserve prior absent-field serialization.

Pinned R4.6.1/ggplot2 4.0.3 produced24 tag-placement cases in `fixtures/parity/ggplot2/theme-furniture-controls.json` via `tools/reference/r/theme-furniture-controls.R`. Twenty succeed; four source left/right plot/panel cases fail with an unbound-vjust error. The fixture preserves those errors. This implementation supplies inherited vertical justification for the documented side-centered placements; this is an explicit defect adaptation, not a claim that the source bug reproduces.

Three real supplied-font tests pass (`/private/tmp/gg14-16-root-tests.log`): measured panel title/caption alignment, tag locations/blank/numeric-margin rejection/wire replay, and all24 placements. Earlier minimal fake text measurer lacked rich shaping; tests moved to chart-export and use committed Noto Sans bytes.

Eight independently authored Rust cases exported24 SVG/PDF/PNG artifacts (`/private/tmp/gg14-furniture-rust`, log `/private/tmp/gg14-furniture-export.log`). Independent SVG renderer and Poppler rendered all vector outputs; every cell in `inspection/all-publications.png` was inspected. Title/caption spans, margin placement, numeric center, global tag across facets and blank omission agree across the three devices. Python/WASM independent authors and native gallery exist; their fresh execution is pending.

Final focused qualification: fresh Rust/Python/Node WASM24publications per host and exact scene JSONs match in `/private/tmp/finstack-chart-proof-20260916/gg14-16-18-final/gg14-comparisons.json`. All eight native modes inspected with nonnull presentation stamps and no diagnostics (`/private/tmp/gg14-furniture-native-final/manifest.jsonl`). Both preparation and paint reuse the final resolved theme cascade.
