# GG-07 native gallery inspection

Environment: local macOS native GPUI, pinned workspace dependencies and supplied Noto Sans. One combined build completed in 21.42 seconds:

`mise exec -- cargo build -p chart-gallery --example ggplot_interval_recipes_native --example ggplot_surface_recipes_native --example ggplot_recipe_marks_native --locked`

Build log: `/private/tmp/gg07-native-build.log`. The initial sandboxed interval launch could not access macOS window services; it was closed and the three galleries were launched with approved desktop access. Screenshot helper captures were limited to each owned app window. No unrelated processes/windows were closed. All three owned gallery processes were stopped after capture with Ctrl-C.

| Gallery | Cases | Paint trace | Inspected screenshot |
| --- | --- | --- | --- |
| Intervals | 2, 3, 6, 9, 11, 12 | `/private/tmp/gg07-native-interval.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-20-44.png` |
| Surfaces | 0, 2, 4, 5, 6, 7 | `/private/tmp/gg07-native-surfaces.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-21-25.png` |
| Marks | 0, 1, 2, 3, 4 | `/private/tmp/gg07-native-marks.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-22-14.png` |

`/private/tmp/gg07-native-summary.json` independently parses the paint traces and checks six/six/five charts, non-null scene stamps and positive paint counts. The screenshots were opened and visually inspected, not merely generated.

Surfaces pass visual inspection: even-odd hole, same-winding nonzero fill, adjacent/default tiles, explicit separated tiles, sharp nearest raster missing cell and smoothly interpolated alpha. Native smooth sampling resembles the PDF consumer; resvg uses a different interpolation kernel. Pixel identity across device types is not claimed.

Intervals show correct vertical/horizontal errorbars, steps, data-space reference line and arrow endpoint geometry. The initial crossbar example exposes a default-fill defect: three black rectangles obscure their center lines, while `primitive-recipes.json` declares null fill and black outlines. This was sent to the interval owner for correction; this initial capture does not pass crossbar default acceptance.

Marks show count area variation, signed columns, four-sided rugs, curved arrows and signed spokes. The initial column default is black instead of the pinned reference `#595959FF` (fresh local ggplot2 4.0.3 geom_col build, colour NA, linewidth 0.5). This was sent to the marks owner. Other visible geometry is correct. Corrections require a fresh targeted native capture before treating all GG-07 native defaults as accepted.

## Source-backed paint audit before rebuild

The native findings triggered a bounded source audit of default emitted paints across interval, step, reference, segment, curve, rug, spoke and count families. Added `crates/chart-core/tests/ggplot_recipe_paint_defaults.rs`: independent source linewidth/device conversions in both destination units, black line paint, transparent crossbar fill, 2.5-times crossbar centerline, pointrange draw radius from reference fontsize, grey column fill and count maximum size. Fixed-default tests pass after the owners' corrections. The kind-driven sentinel additionally found Count attached to a rule builder emitted no circles; that issue was sent to the common recipe owner and remains pending at this report update.

Exact source method/formal capture: `/private/tmp/gg07-interval-controls.txt`. Crossbars expose independent middle/box colour, linetype and linewidth, plus accepted deprecated fatten; middle alpha resets to NA independently of box alpha. Pointranges expose accepted fatten, independent point size/shape/fill/stroke and stem linewidth, plus lineend. The current single-style IntervalRecipe surface cannot express all of these, so the integration owner was notified before final rebuild. Fixed defaults alone are insufficient for complete control acceptance.

## Final native and publication inspection

After the component/default corrections, one combined native build passed: `/private/tmp/gg07-native-final-build.log` (1m24s including shared target lock wait). It built intervals, marks and stroke controls. Final owned app windows were launched sequentially, captured, opened for inspection, and closed (Ctrl-C) before the next launch.

| Gallery | Final cases | Trace | Inspected screenshot |
| --- | --- | --- | --- |
| Intervals | 2,3,6,11,15,16 | `/private/tmp/gg07-native-interval-final.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-53-11.png` |
| Marks | 0–4 | `/private/tmp/gg07-native-marks-final.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-53-35.png` |
| Stroke controls | 0–5 | `/private/tmp/gg07-native-strokes-final.log` | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-54-02.png` |

`/private/tmp/gg07-native-final-summary.json` verifies six/five/six charts with non-null scene stamps and 12/4/14 painted frames. All final native cases pass inspection: default crossbars are transparent with thicker center lines; independent middle/box controls show opaque red dashed middle, translucent gold fill and blue dotted outline; point controls show gold-filled black-outline circles and thin stems. Column defaults are grey. Butt/round/square dash caps and miter/round/bevel joins are visibly distinct; translucent bends have uniform paint with no cancellation holes or dark doubled overlaps. The stroke screenshot includes an ordinary focus/highlight outline on one chart, outside the publication geometry.

The final Rust publication directory `/private/tmp/finstack-chart-proof-20260915/gg07-final` was independently rendered through SVG and Poppler PDF and visually compared with actual PNG exports: all 17 interval triplets, five mark triplets and six stroke triplets passed inspection. Contact sheets are in each scope's `rust-inspection/review-*.png` directory (six interval sheets, two marks, two strokes). Final surface publications are byte-identical to all 27 previously inspected `/private/tmp/gg07-surfaces-final` files, so the prior surface visual evidence remains applicable. Native surface gallery inputs contain no missing Width/Height controls and were unaffected by that correction.

The root integration coordinator subsequently replaced the common dash splitter's platform `hypot` with pinned libm to eliminate native/WASM last-bit differences for stroke cases 0–2. That numerical-only update requires refreshed cross-host bytes; it does not change the source contracts or the visual findings above. The root owns the final host rerun and aggregate acceptance evidence.

Final qualification update: the refreshed host runner passed all 111 GG07 publications with exact Rust/Python/WASM equality (`/private/tmp/gg07-final-hosts2.log`; `/private/tmp/finstack-chart-proof-20260915/gg07-final2`). The coordinator compared all 111 final2 Rust publication bytes with the previously inspected final outputs and found them identical; `gg07-final2/inspection-reuse.json` records this check. The completed native and publication visual findings remain applicable. There is no outstanding cross-host publication discrepancy from the dash splitter correction.
