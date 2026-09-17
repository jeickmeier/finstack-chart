# ADR-028: shared statistical model operations

Status: accepted for the authorized GG10 implementation; package acceptance remains in the status ledger.

The reference smoother selects local regression or a Gaussian shrinkage spline using the largest group across a layer's panels. Weighted models, link-scale intervals, prediction grids and quantile fits must be identical across native, export, Python and WASM. The legacy two-endpoint OLS operation retains its existing behavior.

A new typed `ModelSpec` feeds the existing source/generated statistic pipeline. `ModelOptions` selects the required built-in algorithms and typed model-matrix terms. It does not serialize executable formulas or a host interpreter. `CustomModel` is an exact, immutable, versioned registration with aligned prediction results and explicit portability; it supplements the built-ins. Unknown serialized controls reject, and unimplemented formula/model capabilities remain identified in the numerical checkpoint.

One core QR owner supplies weighted LM and canonical GLM iteration. LOESS implements local-polynomial surfaces and uncertainty. Gaussian GAM uses the shrinkage cubic basis and REML selection with shared bounded symmetric matrix operations. The original BR tableau is shared with weighted distribution quantiles; FN remains a distinct interior-point algorithm with its own source-selected nonunique solutions. No finance engine, interpreter, platform BLAS or new runtime numerical dependency is adopted. Development-only R fixtures supply independent evidence, not runtime computation.

The shared pre-stat scale-population planner resolves full-range predictions and the largest panel group. Coordinate limits do not enter model fitting. Integer source predictors retain the reference distinct-value grid rule; floating predictors use the requested evenly spaced grid. Quantile curves retain distinct generated group identities and exact source memberships. Mean lines and confidence ribbons use existing line/band geometry, positional transforms and target ownership. Standard errors retain model/link units rather than being treated as coordinates.

Invalid options reject during preflight, including empty datasets. Numerical smoother fit failures produce group diagnostics and omit that group's predictions, as the pinned wrapper does. Quantile failures retain their operation error behavior. Resource, registration, schema and unsupported-capability failures are never converted into successful empty output.

Definition capability 77 selects model operations or smooth recipes. Higher capabilities required by composed facets/coordinates/spatial operations still win. Existing wire definitions and OLS behavior are preserved. Actual host, publication, native, update and final repository evidence is required before the ledger closes GG10.
