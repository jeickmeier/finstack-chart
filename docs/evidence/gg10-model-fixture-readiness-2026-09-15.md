# GG10 model oracle readiness — 2026-09-15

Development-only preparation for GG10, not implementation or package acceptance. Canonical runtime, dependency manifests, wire versions and ledger were not changed by this capture.

## Reproduction and identity

`tools/reference/r/model-controls.R` writes `fixtures/parity/ggplot2/model-controls.json`. Executed:

```sh
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work mise exec -- python3 tools/reference/r/run.py tools/reference/r/model-controls.R
```

42 cases captured successfully; log `/private/tmp/gg10-model-capture.log`. Actual oracle: R 4.6.1, ggplot2 4.0.3, mgcv 1.9.4, nlme 3.1.169, MASS 7.3.65, quantreg 6.1, SparseM 1.84.2, MatrixModels 0.5.4, Matrix 1.7.5. Quantreg preprocessing sampling uses recorded seed 20260915. Every case retains inputs, requested controls, actual results and warnings/messages/errors. A successful plot build with an empty layer is not a successful model fit.

## Captured contracts

| Family | Actual independent anchors | Remaining implementation obligation |
|---|---|---|
| Weighted LM | Three confidence levels; exact weighted line; noisy fit; no intercept, quadratic and aliased terms; rank and residual degrees of freedom; se false; explicit grid; full range; zoom versus filtering; negative weights and single distinct x | Weighted rank-revealing factorization and residual variance/covariance, Student-t confidence intervals, explicit rank policy and typed formula terms |
| Weighted GLM | Gaussian, binomial/logit, Poisson/log at .8/.95; coefficients, convergence/iteration count, link predictions and link standard errors; deliberately nonconverged one-iteration Poisson | Family/link-specific IRLS, deviance/convergence diagnostics, covariance and inverse-link interval endpoints. Do not replace with LM or response-symmetric intervals |
| LOESS | Automatic default, span .5/.75, degree one, robust symmetric family, weights and out-of-domain predictions | R-compatible local polynomial neighborhoods, weighting, robust iterations, interpolation surface and prediction uncertainty. A generic direct local fit alone does not establish default interpolation/SE parity |
| Automatic GAM | 999 rows selects LOESS; 1000 selects GAM; two panels of 600 remain LOESS despite total1200; two panels of1000 select GAM | Largest group-panel dispatch; shrinkage cubic regression spline `y ~ s(x, bs="cs")`; REML smoothing estimation and covariance. Generic smoothing splines are insufficient |
| Quantiles | rq br/fn/pfn/sfn, weighted/unweighted, three taus, coefficients and weighted check-loss objectives; tied/singular/negative-weight behavior; default rqss with lambda1 | Solver-specific controls, ties/nonunique solutions, objective agreement and stable fit metadata. rqss requires an actual penalized quantile smoother, not a line alias |

Pinned sources inspected directly from the installed namespace: `StatSmooth` setup/compute, `StatQuantile` compute, `predictdf.default`, `predictdf.glm`, `predictdf.loess`, `quant_pred`. Temporary extracted transcripts: `/private/tmp/gg10-pinned-source.txt` and `/private/tmp/gg10-predictors.txt`. The first transcript ends before predictor extraction because there is no `predictdf.lm`; LM uses `predictdf.default`, captured in the second transcript.

## Concrete numeric sentinels

Weighted LM prediction at x0 is 1.1650000000000007, se .6749481461564285 and 95% interval [-.5700094445061186,2.9000094445061197]. Zero weights do not contribute residual degrees of freedom.

Weighted binomial x0 gives link fit -2.44997124994443 and link se1.6074622308588793, response .07944065164536318 and interval [.003682260732724435,.6683200835658666]. The published `se` is on the link scale. Gaussian GLM uses the GLM normal-quantile interval contract even though an LM fit could have the same point estimate.

Weighted rq br objectives at taus .25/.5/.75 are3.75/6.5/6.5, with coefficients [0,1]/[0,1]/[2,1.5]. fn achieves nearby objectives but its .25 intercept is approximately -6.4491e-8; exact coefficient equality across solver families is unjustified, particularly for nonunique optima. pfn records fixup warnings. Negative weights and singular designs yield warnings plus empty ggplot layers. The intentionally nonconverged GLM retains five predictions and its warning; silent success would lose observable diagnostics.

## Open gates

This is a bounded source fixture set, not an exhaustive model/control census. Additional grouped/faceted mixed group-size thresholds, explicit method overrides at the threshold, integer versus double automatic grids, transformed axes, missing/nonfinite inputs, nonlinear rank/convergence edges, GAM basis/control families and custom registered model lifecycle need dedicated cases during implementation. Formula/registration authoring, generated schema and provenance, update versus batch, fresh Rust/Python/WASM execution, rendering, resource limits, deterministic math and performance remain unimplemented/unqualified here. No candidate model crate was adopted or compiled in this step.
