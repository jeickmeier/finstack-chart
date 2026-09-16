# GG09 distribution numerical implementation checkpoint

This checkpoint qualifies the shared numerical kernels and statistical adapter against the committed R 4.6.1 / ggplot2 4.0.3 fixtures. It does not close GG09 or claim fresh host/publication acceptance. The earlier fixture-readiness document describes the preceding source-only stage.

Implemented one original Rust bounded BR tableau owner for weighted intercept boxplot quantiles and future multivariate quantile regression. The same owner passes all 82 pinned intercept tie/permutation/weight cases, the existing weighted multivariate BR anchors, and 12 additional polynomial/duplicate/rank cases. Weighted rank, finite arithmetic and iteration budgets are explicit. No oracle source was transplanted; no runtime dependency was added. Other quantreg methods and confidence inversion are not implemented by this checkpoint.

Distribution kernels implement all nine quantile definitions; observed box whiskers, weighted hinges, notches and outlier indices; the seven variance-standardized density kernels using weighted linear binning, padded FFT correlation and interpolation; six bandwidth selectors; finite-bound reflection and retained-mass normalization; violin quantile insertion and panel normalization; and dotdensity/histodot construction. Signed and zero dotdensity widths have separate pinned source fixtures; histodot reuses the existing histogram edge/closure owner. Complete density grids are tested with numerical tolerances, not replaced by direct sums or visual similarity.

The adapter filters source populations, generates typed schemas and warnings, enforces row/FFT budgets, distinguishes observed outlier Source targets from fitted Derived targets, and retains exact group/bin membership. Fit rows share membership allocations. Mapped aesthetic values survive only if proven constant across the whole statistical group. Actual keyed source upserts are compared with fresh batch results. Singleton density produces the reference missing-value row; `Density.trim` defaults false and selects the observed group range when true.

The existing `ggplot_bin_training` owner now also resolves distribution sample ranges and Function/full-range QQ x ranges. Physical axes respect orientation; QQ observed samples train y rather than theoretical x. Existing filtered source populations, explicit limits and facet sharing policy are reused. Public-author tests qualify cross-layer density ranges in both orientations and horizontal facet free-axis behavior.

Validation on the macOS workspace:

- `mise exec -- cargo test -p chart-core --test ggplot_distribution_training --lib grammar:: --locked`: 21 grammar unit tests passed, including 12 distribution/adapter tests and four BR tests; log `/private/tmp/gg09-numeric-final.log`. The integration target was filtered out in this command and is not counted here.
- `mise exec -- cargo test -p chart-core --test ggplot_distribution_training --locked`: both actual public-author training tests passed; log `/private/tmp/gg09-training-final.log`.
- Earlier focused distribution-only and solver runs are recorded in `/private/tmp/gg09-distribution-final4.log` and `/private/tmp/gg09-br-workspace.log`.

Remaining package gates belong to coordinated integration: full workspace lint/check/tests, fresh Rust/Python/WASM execution and replay comparisons, generated declarations/schema ownership, native/export artifact inspection, independent geometry contracts, and package evidence/ledger closure. Numerical fixture coverage is bounded evidence, not a proof of all possible BR degeneracies or bandwidth inputs.
