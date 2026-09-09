# CLR-05 integrated color acceptance

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 work.
The [source hashes](phase-2-color-acceptance/source-hashes.json) identify this snapshot.
CLR-05 and G-COLOR pass for d3-color 3.1.0 / COL-01–06 / FIX-C01. This is the
floating color and shared paint contract; named palettes remain CP-01–05 and final
platform/performance release qualification remains WP-21/22.

The complete foundation and paint records are retained in the
[color math report](phase-2-color-foundations-2026-09-08.md),
[paint report](phase-2-paint-2026-09-09.md), and
[scale integration report](phase-2-scale-integration-2026-09-09.md).
This package adds the final perceptual, alpha, grayscale, update and measurement
checks without changing the core color implementation or the pinned expectations.

| Required surface | Result |
| --- | --- |
| FIX-C01 math and standalone operations | All 351 cases pass in Rust, fresh Python and fresh single-threaded WASM: eight constructors, 148 names, parsing failures, inherited conversion/manipulation/predicate/format methods, exceptional tags, strict versioning and owned disposal. CSS strings and canonical bytes remain exact; channel tolerances remain operation-specific. |
| Every paint input | The complete primary runner repeats CLR-04's four independently authored figures, nested paint inputs, v1 migration, wide channels, retained requests and color-only live edits. Presented/current exports remain independent after owners drop and current matches fresh batch. |
| Perceptual color integration | Three further figures independently authored in all hosts use shared Lab, HCL and Cubehelix scales, 63 translucent marks each, light/dark backgrounds, matching labeled endpoint guide swatches, and grayscale conversion after color evaluation. Marks retain alpha 128; grayscale marks have equal RGB channels. The guides are explicitly sampled endpoint swatches, not a continuous strip approximation. |
| Palette updates | Three mapped palette/factory changes reuse the same prepared table, retain positional domains and exact targets, update guide colors and match fresh preparation. Existing constant-color/live-snapshot tests also pass. |
| Native/publication | Actual native overlap and publication PNG, rendered SVG and rasterized PDF output were inspected on both backgrounds and in grayscale. All RGBA channels match exactly across Rust/Python/WASM, with supplied Noto Sans embedded in every PDF. [Checks](phase-2-color-acceptance/publication-checks-final.json), [PNG sheet](phase-2-color-acceptance/contact-1.png), [PDF sheet](phase-2-color-acceptance/pdf-contact-1.png), [native](phase-2-color-acceptance/native.png). |
| Validation | `mise run check` and the complete primary runtime/type runner pass. Eleven focused color/paint/interpolated-scale tests pass on macOS and offline Linux. [Core](phase-2-color-acceptance/core.log), [Linux](phase-2-color-acceptance/linux.log), [check](phase-2-color-acceptance/check.log), [primary](phase-2-color-acceptance/primary.log). Final gallery swatch sizing was rerun in all destinations after the complete runner. |

Release-profile timings use one warmup and seven samples. Median costs are 180.54 ns
for CSS parsing, 183.57 ns for RGB→Lab→RGB, 54.28 ns for HCL→paint, 341.73 ns for
HCL interpolation preparation and 241.90 ns for prepared sampling. Color-only
preparation takes 0.151 ms for 1,000 rows and 1.757 ms for 10,000 rows, with checked
statistics/table reuse. [Raw measurements](phase-2-color-acceptance/benchmark.json)
record ranges and operation counts; preparation includes color encoding and cache
assertions, and input generation is outside timing. Parsing/compiled interpolation
resources remain outside per-mark evaluation. These are WP-22 workload inputs, not
release performance or allocation budgets.

Run `cargo test -p chart-core --test paint_integration --test color_parity --test
scale_interpolated --locked` under mise for the focused core checks. Linux repeats
that command offline in Rust 1.97.1 Debian bookworm aarch64 with read-only source and
registry. `cargo run -p chart-core --example color_benchmark --release --locked`
records the component measurements. The regular primary proof runner now includes
the three new authors plus the complete public calendar replay. Its
[environment record](phase-2-color-acceptance/environment.json) captures actual
Rust, Python, Node and wasm-bindgen versions.

SVG inspection uses the [retained resvg probe](phase-2-scale-integration/svg-render-probe/src/main.rs)
and exact supplied font, with the embedded alias resolved in memory as documented
in SP-07. No stored SVG, reference expectation or visual baseline was rewritten. One final native launch stalled before chart code in `_dyld_start`; the [process sample](phase-2-color-acceptance/loader-sample.txt) records it. Restarting that owned process produced the inspected final frame.
Retained [dark SVG](phase-2-color-acceptance/publication/rust/dark.svg),
[light SVG](phase-2-color-acceptance/publication/rust/light.svg) and
[grayscale SVG](phase-2-color-acceptance/publication/rust/grayscale.svg) sit beside
all three hosts' scene, definition, PDF and PNG artifacts. Next package: CP-01.
