# ADR 027: Analytical statistics and reference facet ownership

Status: accepted implementation decision; feature acceptance remains in the
[status ledger](../implementation-status.md).

## Decision

Distribution and univariate statistics remain core-owned typed operations. Their
shared numerical kernels accept bounded observations and return generated values;
the stage adapters own filtering, grouping, schema, diagnostics and provenance.
Weighted boxplots and later quantile models share one original BR tableau solver.
Density and violin share one R-compatible gridded convolution estimator and
bandwidth owner. Reference programs and offline fixtures are development oracles;
no interpreter, finance engine or statistical runtime dependency is introduced.

`StatisticalRow.count` and members retain the exact eligible source population.
Weighted/generated curve counts have distinct fields. Observed box outliers keep
source targets; fitted curves carry derived targets with exact input membership.
Only group-constant source aesthetics survive an analytical aggregate. Count's
joint-aesthetic partition contract remains separate.

Pure function and QQ distribution registrations reuse the existing versioned
extension registry discipline: immutable implementations, checked bounded
parameters, exact output cardinality, and explicit portability. The same canonical
operations execute from Rust, Python and WASM authoring. Source expressions remain
typed; no string evaluator or host callback enters core. Function sampling in a
transformed x space evaluates inverse coordinates. Population-dependent generated
transforms run on complete scoped vectors before position adjustments.

Distribution geometry reuses primitive paths, band runs and symbol projection.
Density retains explicit upper/lower/both/full boundary policy and independent
fill/outline alpha. Boxplot's larger options payload is boxed in the recipe enum;
this preserves its serialized shape while keeping other recipe variants compact.
Definition version 75 selects the new analytical capabilities.

Reference facets use an optional policy on the existing facet specification.
New ggplot-profile authoring resolves it explicitly; older wire definitions without
the policy preserve their semantics. Typed panel keys distinguish margin sentinels
from literal labels. Live catalogs, partial-field broadcast, grid axis-sharing and
shrink policies remain in the common grammar owner. Proportional space, strip
placement and interior guides use the common layout solver. Registered labellers
reuse existing vector formatter registrations. Definition version 76 selects this
policy; parsed mathematical labels remain a later package.

## Consequences

Generated-data inspection and source identity are independent of destination ink.
Geometry defaults, axis training and missing-value behavior require compiler and
publication tests in addition to kernel tests. Legacy definitions and explicit
compatibility choices remain reviewable rather than inheriting newly authored
reference defaults implicitly. Package acceptance requires the ledger's actual
host, publication and native evidence; the architecture decision alone closes no
feature gate.
