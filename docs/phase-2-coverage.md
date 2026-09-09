# Phase 2 coverage and integration register

P2-00, 8 September 2026. Integrator: the Phase 2 implementation task.
Baseline: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`.
Contracts: [ADR-014](adr/014-phase-2-integration-contract.md),
[combined plan](impl_plans/phase-2-parity-implementation-plan.md),
[AP-00 register](primary-authoring-api.md). Execution states belong to the
[ledger](implementation-status.md).

This register links the existing inventories; it does not duplicate their method
lists or claim a completed oracle. OPEN means required evidence or implementation
is missing. PARTIAL means a specified baseline subset exists, with the named gap
still open. PASS requires the actual evidence for the row's declared scope. Lane
entry packages expand reference items into method/argument/default fixture rows.
Those rows retain these owners and AP routes; discovering a capability cannot
silently shrink a required inventory.

Surfaces: **S** = standalone Rust plus actual Python/WASM operations; **C** = primary
Rust/Python/WASM chart authoring and shared engine; **D** = native and headless
SVG/PDF/PNG, with the existing WASM scene/SVG requirement. All required surfaces
must have evidence before a row passes; package-specific device/platform limits
remain explicit. The declared versions below come from the approved lane plans;
P2-00 has not downloaded, locked or executed the references.

## D3 inventory links

| Reference item / inventory authority | Requirement | Kernel packages / entry owner | AP-00 integration | Planned fixture | Surfaces | Verdict / evidence gap |
| --- | --- | --- | --- | --- | --- | --- |
| [d3-path 3.1.0: complete builder/state/arc/serialization inventory](impl_plans/d3-path-parity-plan.md) | PTH-01–06 | WP-P01–04; entry WP-P01 | A-FOUNDATION, A-GEOM, A-EXPORT | FIX-P01–06 | S/C/D | PASS: [WP-P01–04/G-PATH acceptance](evidence/phase-2-paths-2026-09-08.md), all methods/state, actual hosts and inspected destinations. |
| [d3-shape 3.2.0: all generators, curves, symbols, stacks and custom protocols](impl_plans/d3-shape-parity-plan.md) | SHP-01–10 | WP-S01–08; entry WP-S01 after WP-P04 | A-GEOM, A-POSITION, A-EXTENSION | FIX-S01–09 | S/C/D | PASS for the finite typed profile: [WP-S01–08 integrated acceptance](evidence/phase-2-shape-acceptance-2026-09-09.md), all 63 exports/220 methods, actual hosts, 1,544 updates each and inspected native/publication output. Expanded performance/release gates remain open. |
| [d3-color 3.1.0: parsing, models, conversion, manipulation and formatting](impl_plans/d3-color-parity-plan.md) | COL-01–06 | CLR-01–05; entry CLR-01 | A-COLOR, A-FOUNDATION | FIX-C01 | S/C/D | PASS for d3-color 3.1.0 typed scope: CLR-01–05 complete; [integrated evidence](evidence/phase-2-color-acceptance-2026-09-09.md). |
| [d3-interpolate 3.0.1: scalar, structured, color, transform and zoom](impl_plans/d3-interpolate-parity-plan.md) | ITP-01–08 | WP-IP01–07; entry WP-IP01 | A-COLOR, A-SCALE, A-FOUNDATION | FIX-I01 | S/C/D | OPEN: no complete shared reference interpolation surface. |
| [d3-scale 4.0.2: factories and all family operations](impl_plans/d3-scale-parity-plan.md) | SCL-06–08 | SP-01–07; entry SP-01 | A-SCALE, A-COLOR, A-NAV | FIX-20 | S/C/D | PASS for declared typed FIX-20 scope: SP-01–07 complete; [integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md). WP-21/22 release rechecks remain open. |
| [d3-scale-chromatic 3.1.0: complete catalog and evaluators](impl_plans/d3-scale-chromatic-parity-plan.md) | CHR-01–06 | CP-01–05; entry CP-01 | A-COLOR, A-GUIDE | FIX-21 | S/C/D | PASS for the declared typed FIX-21 scope: all catalog/composition/exceptional cases, actual hosts and inspected destinations; [integrated evidence](evidence/phase-2-chromatic-integration-2026-09-09.md). |
| [d3-axis 3.0.0: guide identity, ticks, geometry, styling and transitions](impl_plans/d3-axis-parity-plan.md) | AXIS-01–07 | WP-AX01–06; entry WP-AX01 | A-AXIS, A-NAV, A-LINK, A-SCHEDULE | FIX-19 | S/C/D | PARTIAL: named axes exist; independent guides, full tick policies and transitions remain. |
| [d3-hierarchy 3.1.2: topology, methods, layouts, tilers and helpers](impl_plans/d3-hierarchy-parity-plan.md) | HIR-01–08 | WP-H01–08; entry WP-H01 | A-FOUNDATION, A-GEOM, A-EXTENSION | FIX-H01-A–H | S/C/D | OPEN: no complete standalone topology/layout family. |

## ggplot2 4.0.3 item ownership

The [review](evidence/ggplot2-parity-review-2026-09-07.md) supplies initial families;
[GG-00–19](impl_plans/phase-2-parity-implementation-plan.md#6-ggplot2-work-packages)
define exact delivery and prerequisites. GG-00 must inventory release exports,
inherited arguments/defaults/generated fields and dependency-backed capabilities.

| Reference item family | Requirement | Owner | AP-00 integration | Planned fixture | Surfaces | Verdict / evidence gap |
| --- | --- | --- | --- | --- | --- | --- |
| Release inventory, methods, defaults and oracle | GG2-01/12 | GG-00 | A-REQUALIFY | FIX-GG00 | Reference + offline Rust | PASS for reference entry: [GG-00 locked inventory and deterministic corpus](evidence/phase-2-oracles-2026-09-08.md). Semantic capability rows remain with their owners. |
| Single/faceted discrete legend regression | GG2-04, GRA-07, SCL-05, LAY-03, THM-03 | GG-01 | A-GUIDE | FIX-GG01 | C + SVG/PDF/PNG | PASS for the specified shared-legend matrix: [24 cases, actual hosts and inspected output](evidence/phase-2-entry-and-legends-2026-09-08.md); full guides remain GG-05. |
| Mapping inheritance, stages, profile, grouping, orientation | GG2-01/02 | GG-02 | A-MAPPING, A-TRANSFORM, A-RUNTIME | FIX-GG02 | C/D | COMPLETE for GG-02 stage scope: canonical provenance/migration, typed expressions, inferred groups and orientation; [evidence](evidence/phase-2-stages-2026-09-08.md). Independent aesthetics remain GG-03. |
| Independent aesthetic mappings and units | GG2-03 | GG-03 | A-MAPPING, A-GEOM | FIX-GG03 | C/D | OPEN: independent fill/stroke/alpha/shape/linetype/size/linewidth semantics remain. |
| Scale, palette, limits, breaks and secondary policies | GG2-03 | GG-04 | A-SCALE, A-COLOR, A-AXIS | FIX-GG04 | S/C/D | PARTIAL: LibraryV1 builders exist; reference policies await shared D3 kernels. |
| Colorbars, binned/multi-aesthetic legends and custom guides | GG2-04 | GG-05 | A-GUIDE, A-AXIS | FIX-GG05 | C/D | OPEN: shared discrete painter is not complete guide capability. |
| Bin/count/summary, weights and positions | GG2-02/05 | GG-06 | A-STAT, A-POSITION | FIX-GG06 | C/D | PARTIAL: baseline statistics exist; reference defaults/generated fields/positions remain. |
| Primitive/interval/reference-line/polygon/raster/blank geoms | GG2-06 | GG-07 | A-GEOM, A-RECIPE | FIX-GG07 | C/D | PARTIAL: current marks exist; complete reference options/families remain. |
| Row-driven text, labels and custom annotations | GG2-06/09 | GG-08 | A-TEXT, A-FIGURE, A-GEOM | FIX-GG08 | C/D | PARTIAL: fixed annotations exist; source/stat-row text and label boxes remain. |
| Boxplot, density/violin/dotplot, ECDF, QQ and helpers | GG2-05/06 | GG-09 | A-STAT, A-RECIPE | FIX-GG09 | C/D | OPEN: baseline quantiles/fit are not these statistical families. |
| Smoothing, confidence bands and quantile regression | GG2-05/06 | GG-10 | A-STAT, A-EXTENSION | FIX-GG10 | C/D | PARTIAL: OLS exists; full reference model/default/uncertainty behavior remains. |
| 2D/hex/density/contour/ellipse statistics | GG2-05/06 | GG-11 | A-STAT, A-GEOM | FIX-GG11 | C/D | OPEN: full reference 2D analytical layers remain. |
| Multi-variable facets, margins, shrink, strips and free space | GG2-07 | GG-12 | A-FACET, A-LAYOUT, A-GUIDE | FIX-GG12 | C/D | PARTIAL: baseline wrap/grid exists; expanded semantics remain. |
| Fixed/flip/transformed/polar/radial coordinates | GG2-08 | GG-13 | A-LAYOUT, A-EXTENSION | FIX-GG13 | C/D | OPEN: projection/clipping/inverse/guide protocol and reference coordinates remain. |
| Theme hierarchy, presets and mathematical text | GG2-09 | GG-14 | A-THEME, A-TEXT, A-FIGURE | FIX-GG14 | C/D | PARTIAL: baseline theme/rich text exists; reference hierarchy/math remain. |
| Map/sf geometry, CRS, projections and geography | GG2-08 | GG-15 | A-GEOM, A-LAYOUT, A-EXTENSION | FIX-GG15 | C/D | OPEN: resources/projection kernels and full geographic capability remain. |
| Extensions, typed data/recipe dispatch and vector helpers | GG2-10 | GG-16 | A-EXTENSION, A-RECIPE, A-INSPECT | FIX-GG16 | S/C/D | PARTIAL: stat/geom registrations and inspection exist; remaining protocols/dispatch remain. |
| Saving and complete device/platform capability | GG2-11 | GG-17 | A-EXPORT | FIX-GG17 | C/D + declared devices | PARTIAL: Output/SVG/PDF/PNG exist; full device/filename/platform behavior remains. |
| Cross-family authoring, update, capture and host combinations | GG2-02–12 | GG-18 | A-RUNTIME, A-STREAM, A-EXPORT, A-PYTHON, A-WASM | FIX-GG18 | C/D | OPEN: new family combinations await their owners; AP proofs certify baseline only. |
| Complete reference capability certification | GG2-01–12 | GG-19 | A-REQUALIFY | FIX-GG19 | S/C/D | OPEN: requires every required inventory row and all eight D3 gates. |

## Concrete integration gaps and remaining work

1. GG-02 implements ADR-014's normalized policy/provenance and migration across Plot,
   Chart edits, worker/cache identity and static/Presented/Current captures. Current
   `Profile::LibraryV1` is preserved; adding enum variants alone is insufficient.
2. WP-AX01 migrates primary axis handles/names, guide-to-scale relations, layer
   bindings, navigation/linking and wire/host constructors together, using SP-01.
3. Each semantic owner extends `plot/host`, export host dispatch and actual
   `chart-python`/`chart-wasm` registrations plus package declarations/proofs.
   Baseline AP-07 qualification does not cover future operations.
4. GG-01 completes its seven required cases: two-entry scatter, one-panel facet control,
   empty guide, hidden guide, tight layout, compatible shared guides and incompatible
   guides. Source targets and clipping/omission pressure are verified in the linked
   acceptance report, which also covers untitled builds/edits and generic-title migration.

AP-00–08 are delivered in the baseline; reuse their 35-row register and evidence.
Do not charge again for Data/Plot/Chart/Output, shared host syntax, static/live capture,
the existing legend painter or their baseline proof runners. Remaining work is the
open semantic delta and its affected integration/evidence. Historical aggregate
budgets in the plans are not a current remaining estimate: the complete argument
inventory is not yet available. Entry owners re-estimate that delta after locking
their inventory, counting each shared kernel once. P2-00's coordination is completed
by this contract/register, without assigning a new multi-day budget to delivered work.

Next sequence: independent lane entries/GG-00 per the
combined plan. AP-09's visible native performance qualification remains OPEN under
its existing owner and is not a prerequisite for these independent slices.
