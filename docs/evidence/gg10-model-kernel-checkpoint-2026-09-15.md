# GG10 initial model kernel checkpoint

This is the first bounded LM/GLM numerical slice, not GG10 acceptance. New `grammar/model_linear.rs` and `grammar/model_glm.rs` are deliberately not declared in the core module tree while GG09/GG12 host qualification runs. No dependency or shared statistic/authoring/schema change is included.

The weighted linear owner uses twice-orthogonalized QR, preserving accepted column order and exposing aliases, rank, positive-weight residual degrees of freedom, residual variance and unscaled covariance. It supports caller-supplied row-major polynomial designs, including no-intercept models. Prediction checks the estimable subspace for aliased columns instead of inventing values for incompatible designs. QR arithmetic rejects overflow, negative weights and empty weighted populations. Designs are bounded to 64 columns; input row allocation remains proportional to the supplied design.

The generalized-linear owner reuses this QR fit in canonical-family IRLS: Gaussian identity, binomial logit and Poisson log. Initialization, deviance convergence, working covariance, dispersion and link predictions are tested against pinned R output. Finite-step halving has a bounded iteration limit. Results retain convergence, iteration count and boundary status; reaching the iteration budget does not silently claim convergence. Gaussian dispersion uses residual degrees of freedom; binomial/Poisson dispersion is one.

Four isolated tests compile the actual new source files using a temporary test harness and cached workspace serde_json/libm libraries. They pass:

- All four pinned LM formula cases, including quadratic, no intercept and aliased terms, with prediction means and standard errors within 1e-10.
- Six canonical GLM cases, with link prediction and standard-error tolerance 1e-9, response predictions and exact reference iteration counts.
- The pinned one-iteration nonconverged Poisson case, including prediction/standard-error outputs.
- Aliased/non-estimable prediction, zero/negative weight, invalid binomial response and saturated residual-degree cases.

Harness: `/private/tmp/gg10-model-harness.rs`; executable `/private/tmp/gg10-model-tests`. Command uses `mise exec -- rustc --edition=2024 --test` with the actual source paths, workspace dependency rlibs and `-L dependency=target/debug/deps`, then runs the binary. This is isolated numerical evidence, not an integrated cargo, lint, host or publication pass.

Remaining work includes typed model/formula authoring and adapters, interval quantile reuse, complete rank/conditioning diagnostic policy, broader link/family controls, robust convergence edge fixtures, LOESS, automatic GAM, other quantile solvers, and all shared pipeline/host/publication gates. Existing GG09 BR and probability owners must be reused; these files introduce neither a second quantile solver nor a confidence-distribution implementation.

## Integrated numerical checkpoint after GG09/GG12 acceptance

The parent released the numerical modules and added typed model descriptors and pipeline integration. `mise exec -- cargo test -p chart-core --lib grammar::model_ --locked` now passes 13 focused tests (`/private/tmp/gg10-integrated-numerics.log`). This includes 18 source model-dispatch cases comparing means, SE, confidence bounds and missing values within 1e-7; all earlier LM/GLM/LOESS/GAM kernel tests; and rejection of invalid controls even for empty populations. GLM nonconvergence diagnostics retain the actual iteration count, and nonfinite LOESS residual scale produces a diagnostic. The Student probability owner is reused for LM/LOESS intervals; the existing normal transform supplies GLM/GAM critical values. GLM SE remains on the link scale while bounds are inverse-linked.

The canonical GAM Cholesky/solve/inverse operations moved, without arithmetic changes, into `model_symmetric.rs` for shared numerical reuse. All 13 tests passed after that extraction. This is focused core evidence; fresh host/publication/global acceptance remains parent-owned.

## Distinct FN numerical checkpoint

`model_quantile_fn.rs` implements an original bounded primal-dual quantile solver over the weighted design, reusing the shared SPD owner. It is still unwired at this checkpoint. Least-squares initialization of the equality multipliers, signed residual slack initialization, separate primal and dual boundary steps, predictor/corrector centering and the pinned default gap tolerance are observable parts of nonunique coefficient selection. An initial mathematically valid alternative initialization reached the same objective but different coefficients; it was not accepted as parity. The final algorithm matches all six existing pinned weighted/unweighted FN coefficient and objective vectors within 1e-9, including nonunique median and upper-quantile solutions. Two isolated tests also cover singular design, negative weights and explicit nonconvergence (`/private/tmp/gg10-fn-kernel.log`).

Source oracle: quantreg 6.1 `rq.fit.fnb`, default boundary fraction 0.99995 and complementarity tolerance 1e-6; existing `model-controls.json` retains the source outputs. Source was inspected externally to establish numerical conventions; no source was transplanted and no runtime dependency was added. The FN kernel is distinct from the existing BR tableau owner. PFN, SFN and spline quantile regression remain unimplemented routes; these results do not qualify those algorithms or expose them through an alias.

The parent subsequently authorized public `ModelQuantileSolver::{Br,Fn}` selection. BR remains the serde default for existing Quantile descriptors. The dispatcher now routes FN independently and reports its actual nonconverged iteration count. All 15 integrated numerical tests pass (`/private/tmp/gg10-integrated-fn2.log`), including 20 pinned dispatcher cases. Ten author modes and a native gallery are provided in the `ggplot_model_controls` triplet; the package runner accepts `--package GG-10`. Author syntax/prepare checks are recorded separately from actual host runs.

| Model capability | Implemented numerical route | Explicit remaining boundary |
| --- | --- | --- |
| Weighted LM | Typed polynomial/expression design, aliases, covariance and Student bands | Arbitrary R formula language is not interpreted |
| GLM | Gaussian/identity, binomial/logit, Poisson/log; weighted IRLS and link-normal bands | Other R families/links, offsets and arbitrary family functions |
| LOESS | Univariate direct/interpolated surface, degree/span/cell, robust family, uncertainty controls | Multivariate predictor normalization and broader singular-neighborhood diagnostics |
| Automatic GAM | Gaussian centered shrinkage cubic spline, canonical `cs` basis, REML, knots and basis dimension | Other bases, families, smoothing-selection methods and multivariate/additive formula terms |
| Quantile BR | Shared weighted tableau numerical owner and source nonunique selection | Broader quantreg method-specific options |
| Quantile FN | Distinct weighted primal-dual solver with source default tolerance/step policy | Public epsilon/boundary overrides; PFN/SFN are not aliases |
| Quantile spline (`rqss`) | No built-in implementation | Source fixture is retained as an open capability row |
| Registered models | Parent-owned typed registry adapter | A hook supplements these built-ins and does not close missing built-in capability rows |

Public author qualification: `mise exec -- cargo test -p chart-core --test ggplot_model_authors --locked` passes two tests (`/private/tmp/gg10-model-authors4.log`). The first prepares all ten authors and their serialized replay, checks identical prepared marks and actual confidence ribbons, including weighted horizontal and full-range inputs. The second compares seven pinned LM/automatic/BR/FN source cases through the public `model_stat`/`smooth` pipeline, with generated means, SE and bounds at 1e-7. Python/WASM script syntax checks pass. These results do not claim fresh compiled host execution, native inspection or package acceptance.
