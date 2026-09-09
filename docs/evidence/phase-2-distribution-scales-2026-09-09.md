# SP-04 distribution and typed scale qualification

Date: 9 September 2026. Requirements: SCL-03/06/07, GRA-03, BND-01; FIX-20 plus
existing scale, paint, authoring and grammar-stage regressions. Revision: `fab2505`
plus the retained uncommitted Phase 2 work; [source hashes](phase-2-distribution-scales/source-hashes.json)
identify this package snapshot. SP-04 is COMPLETE for its core/chart integration scope.
G-SCALE and the independent host, native, visual and performance gates remain open.

Numeric and typed continuous scales now share knot selection/normalization. Sequential,
diverging and sample-rank scales use common transforms and existing interpolation
factories, including floating color spaces and native custom `Sample<T>` outputs.
Quantile/quantize/threshold classifiers share right-bisection lookup, preserve typed
keys/ranges and expose real inverse intervals. Reference defaults are available through
constructors. Missing inputs, duplicate ranges, empty/constant populations, descending
inputs and IEEE operation results retain explicit semantics.

Mapped chart colors and numeric size/opacity/stroke width consume the same engines.
Eligible quantile populations are collected after statistics and before viewport mapping,
including shared layers. Guides retain intervals, midpoint and full mapping identity.
Exact unsigned threshold keys remain distinct beyond binary64 precision. New descriptors
require definition v5. Encoded paints retain floating channels through after-scale
expressions and opacity multiplication, then lower once to existing scene bytes.

| Evidence | Result |
| --- | --- |
| Pinned classifiers | All 58 configurations pass mapping, breakpoint and inverse-extent comparisons, including the six empty-quantize-range reference errors. Generic structured outputs, exact large keys, population correction and descriptor round trips pass. |
| Pinned interpolation scales | All 87 sequential/diverging/rank configurations and 15 typed continuous-range configurations pass. Reference errors are compared per operation; diverging null coercion uses an explicit equivalent numeric zero while typed missing is checked separately. Asymmetric midpoint, native custom vector output, rounded ranges, singleton NaN and empty negative-zero ranks pass. |
| Constructor defaults | All seven family/default construction routes execute independently; existing comparisons pass again. [Defaults log](phase-2-distribution-scales/defaults.log). |
| Chart/publication | Four tests pass: named color/radius/opacity/width; shared quantile correction versus fresh batch and zoom invariance; post-stat training and exact unsigned keys; floating alpha 0.7 × 0.5 rounds once to 89 across constant, discrete, classifier and interpolated paints. V5 round trips and v4 rejection pass. Earlier SVG bytes remain identical after updates. [Current export](phase-2-distribution-scales/export-current.log). |
| Regression | Existing numeric, paint, authoring, portable, ggplot stage and shared scalar/value/color/descriptor interpolation suites pass. 56 core tests plus the added default-constructor test and nine export tests give **66 distinct focused macOS tests**. [Core](phase-2-distribution-scales/core.log), [export](phase-2-distribution-scales/export.log). |
| Linux | Rust 1.97.1 Debian aarch64 container, offline with read-only source/registry: 35 core/paint/grammar/interpolation tests pass, then the added constructor/default suite passes. **36 distinct Linux tests**. [Linux](phase-2-distribution-scales/linux.log), [defaults](phase-2-distribution-scales/linux-defaults.log). |
| Repository | `mise run check`, formatting and whitespace checks pass, including all-target Clippy, rustdoc and WASM compilation. [Check log](phase-2-distribution-scales/check.log). |

The initial standalone typed-range test omitted the reference's Lab factory; correcting
the test configuration exposed no mapping defect. The initial threshold export used an
infinity glyph unavailable in the supplied fixture font; unbounded labels now use `<`
and `>=` text while retaining explicit interval metadata. Opacity integration also exposed
a real early alpha-rounding error; retaining paints until final styling fixes it and the
four-route precision regression proves the boundary. Oracle values/tolerances were not changed.

Commands: core `mise exec -- cargo test -p chart-core --test scale_classifier --test
scale_interpolated --test scale_numeric --test paint_integration --test full_scales
--test authoring --test authoring_host --test portable --test ggplot_stages --test
interpolate_scalar --test interpolate_values --test interpolate_color --test
interpolate_descriptors --locked`; export `--test scale_distributions --test authoring`;
focused constructor reruns; corresponding offline Linux core invocations. All use the
committed lockfile. The host is macOS arm64 with Rust 1.97.1.

These are scoped Rust/chart/export checks. No new standalone Python/WASM scale API,
registered custom chart interpolator, native capture, inspected distribution plot image,
aggregate Linux acceptance or measured performance budget is claimed. Registered
interpolation belongs to WP-IP06; standalone host and integrated scale qualification
remain SP-07. Numeric candidates/formatting are next in SP-05, followed by calendar
behavior in SP-06. Current numeric axes provide the compatible positional mapping route;
sequential/diverging typed scales deliberately have no general numerical inverse.
