# WP-12 portable facet fixtures

All seven cases execute unchanged through Rust, the actual Python extension and actual
Node WebAssembly. Their optional publication profile is 600 × 400 points, 150 DPI, with
the same supplied Noto Sans face and outline export route used by earlier proofs.

Source values are A: `[1, 3, 9]`, B: `[100, 120]`, with group labels A: `[1, 1, 2]`,
B: `[1, 2]`. Exact row keys exceed JavaScript's safe integer limit. Independent expected
means are A groups `[2, 9]`, B groups `[100, 120]`; facet means `13/3` and `110`; whole-chart
mean `233/5`. The panel order is explicitly B then A.

| Case | Independent contract |
| --- | --- |
| `facet-shared-broadcast` | Both y domains train on `[1,120]`; a separate-schema y=50 annotation explicitly broadcasts; one compatible group legend. |
| `facet-free-target` | B uses `[100,120]`; A includes its targeted threshold and uses `[1,50]`; no annotation in B. |
| `facet-grid-empty` | B/A/C by group 2/1, six aligned cells; two C cells explicitly show no data and retain shared scales. |
| `facet-group-summary` | Independent exact groups within facets. |
| `facet-panel-summary` | One summary population per facet. |
| `facet-chart-summary` | Whole-chart summary explicitly broadcast into both panels. |
| `facet-shared-log` | Shared positive population through the logarithmic mapping. |

Each runtime additionally removes the annotation's facet policy and supplies an unknown
target panel. The expected stable wire errors are `CHART_SCHEMA_CONFLICT` and
`CHART_VALIDATION`, respectively. The [comparison runner](../../scripts/bindings/compare.py)
checks semantic values within 1e-12, final scenes within 1e-10 points, exact SVG bytes,
per-item panel identities and vector PDFs. Source fixtures/tolerances from WP-09–11 are
unchanged. Native and export visual inspection is recorded in the WP-12 report.
