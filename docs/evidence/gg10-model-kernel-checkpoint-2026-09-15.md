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
