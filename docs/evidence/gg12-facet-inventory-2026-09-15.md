# GG12 facet completion inventory and proposed interfaces

Status: read-only planning and independent reference capture. GG12 is not implemented or accepted. Runtime/binding code is frozen for GG07 integration. Assignment follows GG-12 in `docs/impl_plans/phase-2-parity-implementation-plan.md` and normative GG2-07/GRA-07; accepted prerequisites are GG02/GG05/GG06. Parsed labellers remain GG14-owned, while descriptor/registration plumbing belongs here.

## Existing shared implementation

`grammar/facets.rs` owns exact PanelKey values, FacetLayout Wrap/Grid, explicit Keep/Drop, shared/free x/y flags, layer target scopes and prepared panel identity. Validation requires exactly one wrap field or two grid fields and bounds panels to 256. `plot/facet.rs` derives first-seen keys, skips missing facet values, and expands observed single row/column catalogs for basic grid. `row_matches` requires every facet field to match; missing fields require existing explicit Broadcast/Panels policy. Keep this legacy contract unchanged.

`prepare_facets` already performs one source/vector preparation pass, fixed-axis automatic histogram/summary bin edge synchronization, cross-panel position population synchronization, per-panel compilation and chart-level shared-domain collection. Reuse this ownership. Existing free x/y currently means independent per panel, including grid; ggplot grid instead shares free x within columns and free y within rows.

`layout/facets.rs::layout_facets` measures one `panel_label` per panel, allocates equal cells, puts all headers at top, copies the same axis request into every panel and merges existing guide scopes/inspection targets. Existing guide collection and stable panel metadata are valuable shared infrastructure; do not create another facet renderer. No current grammar fields encode shrink, margins, multi-variable side grouping, proportional space, direction, strip placement, labellers or interior-axis/label policy.

## Pinned independent fixtures

Added `tools/reference/r/facet-controls.R` and `fixtures/parity/ggplot2/facet-controls.json`. Command:

```
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library \
CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work \
mise exec -- python3 tools/reference/r/run.py tools/reference/r/facet-controls.R
```

Actual R 4.6.1 / ggplot2 4.0.3 capture: 38 cases, all successful ggplot_build and grid.draw on an 8 by 6 inch PDF device. Log `/private/tmp/gg12-facet-reference.log`. Fixture records panel catalog with ROW/COL/SCALE_X/SCALE_Y, each layer's generated data, panel ranges/breaks, gtable panel/strip/axis cell geometry, unit-valued row/column dimensions, strip/axis text and warning messages. This is oracle data, not implementation proof or inspected visual acceptance.

Coverage and independently observed contracts:

- Multi-variable wrap: drop=true retains 4 observed tuples; drop=false expands to 12 combinations including unused levels and missing identities.
- Grid rows `(r,nested)` and columns `(c,z)`: nesting within a side versus crossing across sides yields 16 observed-side panels and 72 panels with drop=false. Flat Cartesian expansion of every variable is wrong for drop=true.
- Margins: all produces 24 panels; margin `r` produces 10, and marginalizing outer `r` also marginalizes nested variable in the `(all)` row. Selecting `r,nested` produces 16. Use original selected source rows for each margin; do not sum ordinary panel statistics.
- Shrink=true summary y range for the R panel is `[29.95,30.05]`; shrink=false preserves raw population range `[-2.9,104.9]`. For L the respective ranges are `[2.6166666666666667,2.7166666666666663]` and `[0.85,4.15]`.
- All four scales policies crossed with fixed/free space. Free-grid x IDs are shared by column, y IDs by row. Expanded column spans in the all-free case are 9.9 versus 1.1, so proportional widths are 9:1; row spans differ independently.
- Six wrap direction encodings (`h,v,tr,rt,bl,lb`), all four strip sides, four interior-axis policies, label_both, contextual mapping labeller and wrapped labels.
- Missing all facet fields broadcasts one annotation row to six panels; missing only `c` with `r=B` broadcasts to just the two B panels. This is profile inference; explicit legacy targeting remains intact.
- Free-x wrap space with explicit ncol warns that custom row/column count is incompatible; observed layout still follows the free-space constraint. Record this normalization/diagnostic instead of silently accepting contradictory layout.

## Smallest shared completion design

1. Add an optional reference facet policy on FacetSpec (serde absent preserves legacy). Keep one flattened fields vector and PanelKey identity vector. Policy supplies row-field count or a multi-wrap route, levels/drop rules, margins, shrink, direction, proportional-space axes, strips, axes/labels and labeller descriptor. Prefer extending builder fields()/rows()/cols() to produce this policy over separate facet engines.
2. Introduce a compact execution-local panel plan before current `prepare_facets`: stable key, row/column, source-match mask, x/y sharing group IDs and strip label inputs. Represent NA with existing GroupValue::Missing. Represent margins with a distinct typed margin marker; literal text `(all)` must remain an ordinary category. Keep a panel's original row membership and target lineage; margin membership relaxes only the marked facet fields.
3. Reuse current panel compilation. Shared domains merge by `(ScaleId, sharing_group)`; grid column/row sharing is a grouping change, not cloned scale IDs. The existing global merge remains the fixed sharing group. Derive common automatic bin edges and position preserve populations over the same scope groups, including original filtered source membership.
4. Shrink=false contributes the existing pre-stat positional population domains in addition to post-stat geometry domains. Preserve explicit transform/OOB/limits staging; do not retrain statistics or expose raw rows as painted marks. Store only required domain contributions, not duplicate prepared tables.
5. Layout consumes panel row/column weights derived from resolved expanded scale spans. Resolve training before physical allocation, then solve aligned panel rows/columns through the existing solver. Wrap free-space restrictions normalize panel arrangement once. Strip measurement uses the existing measurer/wrapping services; resolved side and label policy alter furniture allocation, not panel identity.
6. Axis visibility/labels are per-panel derived requests: margin versus interior policy, sharing groups and side decide visibility; labels can be suppressed while ticks/axis lines remain. Preserve common guide IDs and panel-scoped inspection metadata.
7. Labeller descriptors cover value-only, variable-and-value, fixed lookup, separator and width wrapping. Registered portable labellers use the existing extension registration/resource identity model with bounded string outputs. GG14 plugs parsed rich labels into the same output; do not implement a second parser here.

## Suggested disjoint implementation slices after authorization

- Core facet owner: `grammar/facets.rs`, `plot/facet.rs`, facet policy declarations and compiler domain hooks; typed panel planning, catalogs/margins/broadcast, shrink and scale-sharing groups.
- Layout owner: `layout/facets.rs`, bounded strip/axis request helper; proportional cells, direction/strips/interior labels and existing solver integration.
- Root host/evidence owner: single generic policy descriptor bridges, version detection, declarations, independent Rust/Python/WASM authors, native/export integration and status ledger.

Avoid three agents editing central grammar simultaneously. Resolve the compact panel-plan interface before layout edits. Registered labellers can be a later bounded slice of this package once interface types are stable.

## Acceptance still required

Focused fixtures must assert exact identities/membership/counts and source/stat domains, not just panel counts. Add batch/update/reorder/resize cases, explicit legacy broadcast sentinels, literal `(all)` versus margin identity, NA versus string `NA`, free-grid constraints, resource budgets and multi-layer missing-field inference. Actual Rust/Python/WASM original/replay equality, native/export inspection, shared-guide identity, selection/provenance and final package checks remain entirely unrun for GG12. Status ledger should remain NOT STARTED or planning-only until root authorizes and completes implementation.
