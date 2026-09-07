# WP-11 completion evidence — 7 September 2026

WP-11 is **DONE** for the required built-in scale/geometry family slice. Starting revision
was `c5ec829`; this report, implementation and evidence belong to the WP-11 completion
commit. Owner assignment covers WP-11 through WP-23, with a commit after each completed
package. WP-12 is the next package. G2–G4 remain open.

## Contract and implementation

| Contract | Result and evidence |
| --- | --- |
| SCL-01/02/03, FIX-07 | PASS for checked log/symlog, point and color families; explicit/descending/empty/constant policies, independent viewport, round trips, missing/clamp rules and exact color legend metadata. [Scale tests](../../crates/chart-core/tests/full_scales.rs), [layout tests](../../crates/chart-core/tests/layout.rs), [contract](../scale-geometry-contract.md). |
| SCL-04 | PASS for supplied session compression, exact large-origin timestamp round trips, closed omission/nearest/error, UTC leap-day fixture and inherited calendar implementation. No exchange calendar or local DST provider is supplied. |
| SCL-05 | PASS for separate independent scales and guide-only one-to-one affine secondary units. Secondary coordinates inherit primary ticks; bounds/viewport conversions must remain finite/distinct. Shared clipping and bounded cross-axis label thinning remain active. |
| GRA-06, DAT-05, FIX-01/03 | PASS for area/ribbon gaps and explicit baseline/bound validation; grouped/stacked bars and histogram composition reuse the existing compiler/positions; heatmaps use interval cells plus color; OHLC validates all price bounds, while volume invalidity is independent. |
| SCN-01/03 | PASS for closed nonzero filled paths through core validation, native GPUI vectors and SVG/PDF/PNG. All 12 new families execute through the portable engine; no recipe computes charts in a host adapter. |

[ADR-005](../adr/005-foundational-scales-and-layout.md) and the
[scale/geometry contract](../scale-geometry-contract.md) record algorithms, defaults,
precision, color interpolation, session boundary conventions, recipe lowering and remaining
interaction scope. New optional portable fields are axes, layer color and low/high source
mappings. New Rust enum variants/fields require downstream exhaustive-match/literal updates.
Only a direct example dependency on already-locked `serde` was added; no package/version changed.

## Commands and environment

Working directory: `/Users/jeickmeier/Projects/finstack-chart`. macOS 26.5.2 arm64,
Rust 1.97.1, CPython 3.14.6/PyO3 0.29.2, Node 24.14.0 and wasm-bindgen 0.2.128.
[Environment](wp-11/environment.txt) and [module hashes](wp-11/module-sha256.json) identify
actual proof artifacts. Native captures used the loaded deterministic Noto Sans font and
real GPUI vector painting; the app window was 1050 × 700 logical pixels plus titlebar.

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-11/fmt.log). |
| `mise run check` | PASS repository/dependency/feature checks, native and optional Kit compilation, workspace Clippy, rustdoc and core WASM; [log](wp-11/check.log). |
| `mise run test` | PASS **133 tests**: 117 core, 13 export, 1 native conversion and 2 Rustdoc examples; [log](wp-11/test.log). Eight new focused tests cover scales and integrated family semantics. |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-11` | PASS actual Rust/Python/Node WASM: 12 prior statistics cases plus 12 new family cases. [Comparison](wp-11/compare.log), [complete runner](wp-11/bindings.log). |
| `mise exec -- cargo build -p chart-gallery --example family_gallery --locked` | PASS; [build](wp-11/native-build.log). The temporary app bundle runs this built example. |
| Native gallery and PDF/PNG inspection | PASS for the family captures below, representative PNGs and a rendered vector ribbon PDF. Final captures were taken after the nonlinear tick-label fix. |

All 24 cases retain exact identities, schemas, operation metadata, targets and source
values across hosts. Semantic numeric tolerance is 1e-12, final scene tolerance is 1e-10
points, and **SVG bytes are identical across all three runtimes**. Existing WP-09/10
assertions and tolerances are unchanged. Independent expectations additionally verify
log gaps, area/ribbon exclusions, color/null metadata, independent volume exclusion,
supplied-session gap splitting, leap day, signed stacks and zero PDF image objects.
[Rust results](wp-11/native/statistics.json), [Python](wp-11/python/statistics.json) and
[WASM](wp-11/wasm/statistics.json) retain all case semantics; their neighboring scene and
SVG artifacts provide the destination results. Rust family PNGs and PDFs are retained.

## Inspected visuals

All twelve native captures were visually reviewed:
[log](wp-11/native-ui/log-gaps.png), [symlog](wp-11/native-ui/symlog.png),
[point/color](wp-11/native-ui/point-color.png), [area](wp-11/native-ui/area.png),
[ribbon](wp-11/native-ui/ribbon.png), [heatmap](wp-11/native-ui/heatmap.png),
[grouped bars](wp-11/native-ui/grouped-bars.png), [signed stack](wp-11/native-ui/stacked-bars.png),
[OHLC/volume](wp-11/native-ui/ohlc-volume.png), [secondary units](wp-11/native-ui/secondary.png),
[sessions](wp-11/native-ui/session.png), [leap-day UTC](wp-11/native-ui/utc-leap.png).

Representative actual exports were also inspected:
[log PNG](wp-11/native/statistics-family-log-gaps.png),
[area PNG](wp-11/native/statistics-family-area.png),
[ribbon PNG](wp-11/native/statistics-family-ribbon.png),
[heatmap PNG](wp-11/native/statistics-family-heatmap.png),
[OHLC PNG](wp-11/native/statistics-family-ohlc-volume.png), and
[rendered ribbon PDF](wp-11/ribbon-pdf-render.png).
Gaps remain unbridged, heatmap nulls retain their declared style, signed intervals reach
both domain extremes, and price/volume axes remain independent. Edge symbols intentionally
clip at exact trained domains; authors can request axis padding. No visual baseline was
regenerated to hide a failure.

Initial native inspection found long transcendental rounding tails in log labels and a
missing exact-decade endpoint tick. Direct base-10/base-2 logarithms and guide-only decimal
formatting fixed this; the final log image shows 1/10/100/1000 and all runtime proofs were
rerun. A test's equal visible-guide-count assumption was isolated from documented global
corner-label thinning so it checks secondary-unit alignment without conflicting labels.

A UI discovery call stalled before returning; subsequent bounded capture succeeded. The
screenshot permission-request preflight was rejected by automatic review. A separate
read-only check confirmed existing Screen Recording access (`requested=false`), and the
capture-only request was approved. No system permission was changed.

## Remaining gates

WP-12 owns facets, common guide composition and shared layout. WP-13/14 own full themes,
publication composition and public extensions. Complete filled-interior hit testing,
selection, scheduling and streaming remain WP-15–20. No Linux native run, full screen-reader
certification, browser product, production packaging or PERF claim is made here. Existing
six unmaintained advisories from the WP-09 scan and the `block` 0.1.6 future-compiler warning
remain; the advisory scan was not repeated for this change. Full family platform coverage
remains WP-21/G4. Proceed with WP-12 after committing this package.
