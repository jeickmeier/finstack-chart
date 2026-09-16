# GG10 one-dimensional LOESS checkpoint

`grammar/model_loess.rs` remains unwired while GG09/GG12 acceptance runs. It implements real tricube local polynomial fitting, default kd-partitioned cubic Hermite interpolation, direct evaluation, robust iterations, influence calculations and uncertainty. It reuses the new weighted QR owner. No dependencies, shared model variants or host APIs are added here.

Controls distinguish span, degree 0–2, cell size, direct/interpolated surface, Gaussian/symmetric family, iteration count, normalization, exact/approximate residual statistics and approximate trace. One-dimensional normalization is identity. Interpolated prediction returns missing values outside the observed range; direct prediction permits extrapolation. Fit vertices and influence matrices have explicit budgets.

The numerical algorithm was independently implemented. Reference approximate residual traces use identified numerical calibration coefficients; they are not replaced by exact residual traces under the default setting. Mathematical calibration and the robust-direct uncertainty call distinction are documented in the [R LOESS numerical source](https://raw.githubusercontent.com/wch/r-source/trunk/src/library/stats/src/loessf.f) and [R LOESS interface](https://raw.githubusercontent.com/wch/r-source/trunk/src/library/stats/src/loessc.c). Executed oracle results, rather than the moving source branch, anchor behavior to R 4.6.1.

`tools/reference/r/loess-controls.R` captured 17 actual pinned R cases in `fixtures/parity/ggplot2/loess-controls.json`: direct/interpolated surfaces, degrees 0–2, exact/approximate statistics, large span, small cell, approximate trace, symmetric direct fitting and disabled normalization. Both mean-only and uncertainty-enabled predictions are retained. The weighted robust-direct result actually changes its mean when `se=TRUE`; the kernel exposes `Uncertainty.fitted` to preserve that source contract explicitly.

The isolated harness compiles the actual new source files. Seven combined LM/GLM/LOESS tests pass, logged in `/private/tmp/gg10-loess-kernels.log`. LOESS comparisons cover all 17 new cases plus the five earlier model-control LOESS cases and automatic-999. Means, standard errors and residual scale match within 1e-9; residual degrees of freedom match within 1e-8. Explicit tests also reject undersized neighborhoods, excessive influence allocations and invalid prediction grids. Source capture command:

```
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work mise exec -- python3 tools/reference/r/run.py tools/reference/r/loess-controls.R
```

Interface: `fit(x,y,weights,controls,max_vertices)` returns `LoessFit`; `predict(grid)` returns nullable means; `uncertainty(grid,max_cells)` returns nullable means and standard errors, residual scale and degrees of freedom. The model adapter must use uncertainty-enabled means when bands are requested and reuse the existing Student quantile owner for intervals.

This does not certify GG10. Multivariate LOESS terms and normalization, pathological singular-neighborhood pseudoinverse/diagnostics, broader invalid/zero-weight cases, integrated lint/tests, source mutations/filter/zoom, fresh hosts and publication remain unqualified. The automatic GAM route and additional quantile-regression methods are still separate required work. No gate is closed by this checkpoint.
