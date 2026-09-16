# GG09 distribution geometry qualification

Scope: GG2-05/06 and FIX-GG09, shared distribution geometry and coordinated host/native proof. Root owns statistical dispatch, overall package acceptance, the status ledger and aggregate repository checks; the numerical owner supplies the independent estimator kernels. This report records the completed geometry slice rather than independently closing GG09.

## Shared implementation

`grammar/recipe_distributions.rs` and its boxplot/violin/dotplot children consume source or generated recipe channels after positional scale staging. Boxplot summaries supply hinges, median, whisker/notch endpoints, optional relative width and typed outliers. Generated outliers retain original targets and transformed dependent values, and receive the summary's post-position delta exactly once. A bounded source-outlier descriptor uses stable RowKey values and finite lists; duplicates and mixing with generated schemas are rejected. Outliers never participate independently in positions.

Boxplots reuse shared rules, polygons and Symbol/SymbolPaint for notches, staples, whiskers, medians and outliers, with independent component paint and physical point units. Violin contours retain ordered source/generated anchors and normalized widths; optional quantile strokes consume probability markers. Trimming and normalization remain statistical operations. Dot plots expand bounded integer counts with bin-width diameter, four stack directions, stackgroups, dotsize and stackratio. Signed DotDensity widths preserve signed stack displacement and absolute radius; zero widths train positions without painting. Circle construction reuses the existing deterministic, bounded shared stroker.

Density keeps the existing BandRun compiler and shared renderer. `DensityRecipe` retains Upper/Lower/Both/Full boundary selection, absent default fill and Round joins. Filled regions apply alpha independently from opaque boundaries. Explicit joins remain authored. AreaBandRun's data-curve and baseline slots are selected correctly; the focused test checks actual destination boundaries after stroke expansion, including the horizontal Lower boundary.

The general independent-paint wrapper no longer overwrites already resolved distribution point payloads with their parent box paint. The regression checks filled outlier shape 19 and its physical stroke conversion. All primitives preserve original targets, clips and existing resource budgets. No renderer or dependency was added.

## Independent source and focused tests

`tools/reference/r/distribution-geometry-controls.R` captures **26** complete build/draw cases from pinned ggplot2 4.0.3 / R 4.6.1 into `fixtures/parity/ggplot2/distribution-geometry-controls.json`: six box, six violin, nine dot and five density cases. The fixture covers controls, orientation, component alpha, physical units and density outline policy. Source method captures remain in `/private/tmp/gg09-geometry-contract.txt`, `/private/tmp/gg09-dotstack-contract.txt`, `/private/tmp/gg09-crossbar-notches.txt`, `/private/tmp/gg09-density-default-paint.txt` and `/private/tmp/gg09-density-ribbon-draw.txt`.

**Nine focused geometry tests passed** in `crates/chart-core/tests/ggplot_distribution_geometries.rs`; final log `/private/tmp/gg09-density-round-final.log`. They qualify source and generated recipes, replay, targets, one-time outlier nudging, widths/orientation, independent paint/units, signed and zero dot widths, and density boundaries/default paint. Initial native inspection exposed the density fill/boundary defects and outlier paint overwrite; these were corrected and checked in the final artifacts below.

## Actual host publication proof

Final frozen-runtime run: `/private/tmp/gg09-gg12-final3-hosts.log`, artifacts `/private/tmp/finstack-chart-proof-20260915/gg09-gg12-final3`, results `comparisons.json` and environment `environment.json` beneath that directory. Command: `mise exec -- python scripts/run_ggplot_package_proofs.py /private/tmp/finstack-chart-proof-20260915/gg09-gg12-final3 --package GG-09 --package GG-12`, using the pinned wasm-bindgen 0.2.128 tool, mise Python/Rust/Node and the existing local mypy/TypeScript tools.

All **120 SVG/PDF/PNG publications per host match byte-for-byte between actual Rust, Python and WASM**: distribution geometries 15 authors/45 publications, univariate controls 8/24, facets 17/51. Each independent author compares original and serialized/replayed scenes before exporting. Python/WASM scene JSON also matches. The geometry authors include source summaries, generated factories, FieldHandle and SourceExpression input, orientation, quantiles, stacking and all four density outline modes. Root owns the eight analytical authors; the facet owner owns the seventeen facet cases.

Strict Python and TypeScript consumers passed against the final modules. The focused consumers now exercise GG09 factories, input/weight mappings, recipe/stat options, functions, connections and GG12 fields/row-fields/reference policies. Additional final declaration logs: `/private/tmp/gg09-gg12-final3-python-types.log` and `/private/tmp/gg09-gg12-final3-ts-types.log`. These consumer-only additions required no runtime rebuild.

## Final visual and native inspection

All fifteen geometry SVG/PDF/PNG triplets were visually inspected using an independent SVG renderer and Poppler PDF rendering, alongside the published PNG. Review boards: `ggplot_distribution_geometries/python-inspection/review-{1,2,3,4,5}.png` under the final artifact root. All eight analytical triplets were similarly inspected in `ggplot_univariate_controls/python-inspection/review-{1,2,3}.png`. Outcomes: PASS; shape, boundaries, alpha, symbols, labels and layout agree across inspected formats. Byte equality extends these inspections to the final Rust and WASM publications. Inspection logs: `/private/tmp/gg09-final3-visual.log` and `/private/tmp/gg09-final3-univariate-visual.log`.

Final native build log: `/private/tmp/gg09-gg12-native-final3-build.log`. The supplied-font galleries were launched and visually inspected:

- Distribution modes 1/6/8/9/11/14: `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-15-20.png`; paint trace `/private/tmp/gg09-distribution-native-final3.log`. Notches, Y dot stacks, generated box/violin, Upper density and Full outline passed.
- Analytical modes 0/1/2/3/5/7: `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-15-52.png`; paint trace `/private/tmp/gg09-univariate-native-final3.log`. Unfilled density, ECDF, QQ points/line, function and matrix connection passed.

Each trace reports six painted charts with one layout attempt per chart. Both owned processes were closed. Earlier geometry native modes 2/10 were inspected before the final density-only change. The facet owner separately inspected all seventeen modes and recaptured the six corrected empty-panel modes; its final manifest is `/private/tmp/gg12-native-final/manifest.jsonl`.

No geometry/runtime blockers remain in this slice. Aggregate checks, the cumulative workspace suite, package acceptance and next work-package selection remain with the coordinating owner. This evidence does not claim Linux or other unexecuted platform coverage.
