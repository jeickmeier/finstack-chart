# WP-10 statistical/position fixtures

[canonical.json](canonical.json) contains independent specification expectations:
FIX-02 bins [2,2]; FIX-03 heights [2,3,-1,-4] ending at +5/-5 and normalized +1/-1;
FIX-04 quartile 7.5 for [0,10,20,30]; FIX-05 OLS intercept/slope 0.8/2.3, unchanged by
zoom and 1/2 after filtering x<=2. The formulas are fixed by the specification, not generated
from implementation outputs. Mean/sum/quantile and OLS tolerances are 1e-12 absolute;
normalized stack endpoints use 1e-15 and final position checks use 1e-10 destination units.

[d3-quantiles.json](d3-quantiles.json) stores the published finite-input examples from the
linked official D3 R-7 quantile documentation, retrieved 6 September 2026. No R/JS reference
engine is installed or executed by Rust tests. Only this matching operation is compared;
D3's empty-sum and invalid-input policies do not override the specification.

[portable-cases.json](portable-cases.json) is a stored input catalog for twelve real runtime
cases: summary/missing input, grouped count, full/filtered OLS, transformed summary, automatic
and overflow bins, stack/normalize, missing-slot dodge, data and display jitter. Rust,
Python and Node WebAssembly all receive these same version-one chart/data envelopes.
The shared runner retains semantic/schema/operation results and final scene JSON, plus SVG
from each host and native PNGs. The comparator checks independent values, exact memberships,
model targets and update capability declarations, 1e-12 statistical and 1e-10 point scene
agreement, and identical SVG bytes. These are tolerance checks plus selected exact output
observations, not a cross-platform pixel-perfect guarantee.

```sh
mise exec -- cargo test -p chart-core --test statistics --locked
WASM_BINDGEN=/path/to/wasm-bindgen-0.2.128 mise run bindings-proof artifacts/wp-10
```

Additional Rust cases exercise strict/missing/empty/constant/singular inputs, extreme
summation and quantiles, large x origins, grouped output budgets, generated-schema errors,
non-additive positions, named output sharing, reorder invariance, and exact batch fallback
after append/upsert/removal/replacement. All source/operation algorithms remain in core.
[The contract](../../docs/statistics-contract.md) records API defaults and exclusions.
