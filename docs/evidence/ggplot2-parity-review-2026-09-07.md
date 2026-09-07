# ggplot2 feature parity review — 7 September 2026

The current library is a substantial Cartesian/publication alpha, but does not have
ggplot2 feature or behavioral parity. Its strongest overlap is layered points, lines,
bars, ribbons, histograms, basic summaries, Cartesian scales, small multiples and
headless publication. Statistical breadth, aesthetic scales, guides, coordinates and
authoring conveniences remain materially narrower. One current rendering defect was
also reproduced: a non-faceted chart prepares color legend metadata but does not paint
the legend.

## Scope and verdicts

Compared the live working tree at base `127fe2d4853f89b62ba59a17248485e3c378ba60`
against the official [ggplot2 reference index](https://ggplot2.tidyverse.org/reference/index.html),
which displayed version **4.0.3** when retrieved on 7 September 2026. This is a feature-family
and important-defaults review, not an exhaustive argument/alias compatibility certification.
Deprecated R aliases, example datasets and R syntax are not counted as missing chart features.
No percentage is assigned: function counts would double-count aliases and conflate a
drawable primitive with a complete statistical or aesthetic contract.

The [specification](../spec/gpui-charts-specification.md),
[implementation plan](../impl_plans/gpui-charts-implementation-plan.md) and
[alpha matrix](../alpha-api.md) were read alongside implementation and tests.
Their completed work packages describe the repository's own scope. SCP-03 explicitly
defers advanced fitting/density, geography, general polar-axis products and full math
typesetting. QLT-02 requires reference comparisons only where intended semantics agree.
This review does not make all ggplot2 features new requirements or change existing defaults.

- **Pass, scoped:** the stated capability exists and relevant local checks passed;
  this does not mean all corresponding ggplot2 options match.
- **Partial:** an implemented subset or manual composition exists; full family parity fails.
- **Absent:** no built-in semantic route was found in the inspected enums, dispatch or public API.
  A caller-written extension or precomputed dataset is not counted as built-in support.
- **Uncertain:** differential or destination evidence needed for equivalence is absent.

## Feature-family matrix

| Family | Current implementation and evidence | Parity assessment |
| --- | --- | --- |
| Plot/layer grammar | `ChartDefinition`, heterogeneous `Layer`, inherited source mappings, source/stat/bin types, named transforms and one compiler. [definition.rs](../../crates/chart-core/src/grammar/definition.rs), [typed.rs](../../crates/chart-core/src/grammar/typed.rs). | Pass, scoped, for composition. Partial for authoring: no R expression/formula interface, automatic plot dispatch or general staged expression engine. |
| Basic marks | Eight `Geom` variants: Point, Line, Area, Ribbon, Bar, Ohlc, Rule and Rectangle. Authored-order lines cover paths; cells and explicit rectangles cover basic heatmaps. Histogram/frequency-line composition shares bin outputs. [definition.rs:443](../../crates/chart-core/src/grammar/definition.rs). | Pass, scoped, for basic Cartesian marks. Rules and multiple layers can manually construct intervals/reference lines; this is not full interval/errorbar or automatic reference-line recipe parity. |
| Additional geometries | No built-in boxplot, violin, dotplot, contour/filled-contour, 2D density, hex bin, rug, quantile-regression, QQ, map/sf, spoke, step or curved-segment recipes. Generic path/fill primitives and custom geometry exist. [Geom](../../crates/chart-core/src/grammar/definition.rs), [scene primitives](../../crates/chart-core/src/scene.rs). | Absent as built-ins. Polygon/curve drawing through an extension or precomputed path does not supply grouping, statistics, defaults and guides. [Reference families](https://ggplot2.tidyverse.org/reference/index.html). |
| Statistics | Six built-in operations: identity, count, explicit bin, auto bin, summary and OLS. Summary supplies count/min/max/mean/sum/R-7 quantiles; OLS has an intercept and two fitted endpoints. [statistical_types.rs](../../crates/chart-core/src/grammar/statistical_types.rs). | Pass, scoped. Partial histogram/summary semantics; OLS is narrower than [smooth](https://ggplot2.tidyverse.org/reference/geom_smooth.html). No built-in KDE, boxplot stats, ECDF, ellipse, function sampling, 2D/hex summaries, unique, connect or area alignment statistics. |
| Positions | Identity, signed stack/normalize, fixed categorical dodge and deterministic data/display jitter. [Position](../../crates/chart-core/src/grammar/definition.rs), [parameters](../../crates/chart-core/src/grammar/statistical_types.rs). | Partial. No nudge/jitter-dodge/dodge2 position; one enum choice cannot compose dodge and jitter. Dodge has explicit retained slots, not the full [dodge options](https://ggplot2.tidyverse.org/reference/position_dodge.html). |
| Aesthetic mappings | x/y/endpoints/bounds, explicit group, one mapped size, and one stage-aware color channel. Theme can set constant symbols/dashes. [SourceAes/Style/Layer](../../crates/chart-core/src/grammar/definition.rs), [ColorEncoding](../../crates/chart-core/src/grammar/colors.rs). | Partial. No independent mapped fill/stroke, alpha, shape or linetype scales; no separately trained size/linewidth scales or corresponding legends. Constant theme controls do not establish mapped-aesthetic parity. |
| Positional scales | Linear, positive log, symlog, band/point, UTC calendar ticks, supplied sessions, reversible ranges, explicit domains/viewports, affine secondary guides. [AxisScale](../../crates/chart-core/src/layout/types.rs), [ScaleTransform](../../crates/chart-core/src/scales/nonlinear.rs). | Partial. No current sqrt/power or binned positional scale; no general transform registration. Secondary guides allow affine numeric conversion rather than the full reference secondary-axis surface. |
| Time scales | Exact integer source units and origins; UTC labels and supplied session compression. Timezone metadata is retained. [utc.rs:17–40](../../crates/chart-core/src/scales/utc.rs). | Partial. Labels deliberately ignore source timezone metadata; this does not implement local-time/DST formatting or the broader [date/time controls](https://ggplot2.tidyverse.org/reference/scale_date.html). |
| Color scales | Authored discrete palettes and equally spaced continuous sRGB-byte interpolation, explicit missing/clamp policy. [color.rs:8–170](../../crates/chart-core/src/scales/color.rs). | Partial. No installed named Brewer/viridis catalog, separate binned-color family or arbitrary stop positions. Current interpolation differs from ggplot2's [Lab gradients](https://ggplot2.tidyverse.org/reference/scale_gradient.html). |
| Axes and legends | Four named sides, labels/titles/format descriptors, custom ticks and affine secondary guides. Facets collect compatible color metadata and paint swatches. [types.rs:78](../../crates/chart-core/src/layout/types.rs), [facets.rs:77–160](../../crates/chart-core/src/layout/facets.rs). | Partial, with reproduced missing single-panel legends. Continuous legends are stop swatches, not a [continuous colorbar](https://ggplot2.tidyverse.org/reference/guide_colourbar.html). No general key-glyph, size/shape/linetype, binned or custom guide grammar. |
| Facets | One-field wrap, two-field grid, explicit key catalog, keep/drop, per-panel free x/y, explicit broadcast/targeting, aligned equal cells. [facets.rs:18–77](../../crates/chart-core/src/grammar/facets.rs), [layout](../../crates/chart-core/src/layout/facets.rs). | Partial. No multi-variable row/column expressions, marginal panels, `shrink` switch, proportional panel space or labeller functions. Free policies are per panel; ggplot2 grid also constrains scale sharing by grid dimension. [Grid reference](https://ggplot2.tidyverse.org/reference/facet_grid.html). |
| Coordinates | Cartesian projection/inversion/clipping; capability explicitly reports `path_subdivision=false`. [coordinates.rs:9–46](../../crates/chart-core/src/layout/coordinates.rs). | Partial. No dedicated fixed-aspect constraint, coordinate flip, post-stat nonlinear coordinate family, general polar/radial axes or geographic projection. Swapping fields or precomputing paths is a workaround, not equivalent coordinate semantics. [Transformed coordinates](https://ggplot2.tidyverse.org/reference/coord_transform.html), [polar](https://ggplot2.tidyverse.org/reference/coord_polar.html). |
| Text and annotation | Rich/rotated text, data/panel/figure/output anchors, bounded collision handling, leaders, titles/captions/notes and insets. [composition.rs](../../crates/chart-core/src/composition.rs), [typography.rs](../../crates/chart-core/src/typography.rs). | Pass, scoped, for authored publication text. Partial against [text/label layers](https://ggplot2.tidyverse.org/reference/geom_text.html): no `Geom::Text`/label aesthetic over source/stat rows, label-box recipe or math parser. Annotation count is capped at 256. |
| Themes | Versioned cascade; editorial, terminal and grayscale presets; layer/output overrides, symbols, dashes and rich typography. [theme.rs](../../crates/chart-core/src/theme.rs). | Pass, scoped, for the repository theme contract. Partial versus ggplot2's [element hierarchy](https://ggplot2.tidyverse.org/reference/theme.html), independent guide/strip component styling and legend placement controls. |
| Saving/publication | Immutable snapshots, explicit fonts, physical sizing, SVG/PDF/PNG and publication preview. [snapshot.rs](../../crates/chart-export/src/snapshot.rs), [profile.rs](../../crates/chart-export/src/profile.rs). | Pass, scoped, for these formats. No general [ggsave device surface](https://ggplot2.tidyverse.org/reference/ggsave.html); cross-library visual equivalence remains Uncertain. |
| Extension and host APIs | Registered Rust stats/geoms, generated schemas, native painter boundary; owned JSON Python/WASM proofs share core. [registry](../../crates/chart-core/src/grammar/extensions.rs), [extension contract](../extension-contract.md). | Partial versus [ggplot2 extensibility](https://ggplot2.tidyverse.org/articles/extending-ggplot2.html). No arbitrary scale/coordinate/facet registration, Python callback authoring, Python viewer or browser plotting product. WASM exposes scene/basic SVG; Python exposes SVG/PDF/PNG. |
| Helpers and bundled data | Typed materialization and supplied datasets; own canonical fixtures. | R metaprogramming aliases, bundled demonstration datasets, `fortify`/autoplot dispatch and vector convenience helpers are separate ergonomics/ecosystem gaps. They are not numerical parity evidence or production requirements by implication. |

## Ranked findings and concrete acceptance cases

Priorities below rank parity impact; an absent future-scope capability is not automatically
a defect against the present specification. GGP identifiers belong to this review only.

1. **GGP-01 — P1, Fail: single-panel color legends are not painted.** GRA-07,
   SCL-05, LAY-03 and THM-03 are the related guide contracts. The non-faceted branch
   in [engine.rs:557–570](../../crates/chart-core/src/layout/engine.rs) calls `solve_panels`
   directly. Legend measurement/painting is confined to [facets.rs:77–160](../../crates/chart-core/src/layout/facets.rs).
   A colored scatter probe prepared a legend titled `PARITY_LEGEND` with two entries,
   but the final scene had zero title items; a faceted positive control painted one.
   Every downstream renderer consumes that
   missing scene content. Extract common guide layout/painting for both paths; acceptance
   should compare single-panel and equivalent one-panel-facet legend semantics and scene
   items, followed by inspected export. This is a current defect, not just missing ggplot2 breadth.

2. **GGP-02 — P1, Fail for equivalence: statistical defaults differ.** GRA-02/03/04,
   FIX-02/05, QLT-02. [stats.rs:424–429](../../crates/chart-core/src/grammar/stats.rs)
   assigns interior edges to the bin on the right; source-unit statistics precede axis
   transforms. The [histogram reference](https://ggplot2.tidyverse.org/reference/geom_histogram.html)
   defaults to right-closed bins and applies scale transforms before binning.
   A documentation/source-derived counterexample is values `[0,1,2]` with breaks
   `[0,1,2]`: chart counts `[1,2]`, versus ggplot2 default `[2,1]`.
   A log-axis histogram also bins different populations in transformed coordinates.
   Built-in bins have no weight input, density/normalized-count fields, boundary/center
   control or closure choice. OLS alone does not cover smoothers and confidence bands.
   Preserve existing defaults; any compatibility route needs explicit stage/closure/weight
   policies and matched fixtures before claiming parity. No R oracle was executed here.

3. **GGP-03 — P1, Fail for equivalence: aesthetic semantics are substantially narrower.**
   GRA-01/02/06, SCL-01/05. [compiler.rs:1106](../../crates/chart-core/src/grammar/compiler.rs)
   copies mapped size into radius and stroke width and excludes nonpositive sizes;
   [the test](../../crates/chart-core/tests/grammar.rs) explicitly asserts that behavior.
   Sizes 1 and 4 therefore produce circle areas in ratio 1:16. This is not an
   [area scale](https://ggplot2.tidyverse.org/reference/scale_size.html); `scale_size_area`
   maps those values to areas in ratio 1:4. Add independent aesthetic encodings,
   scale policies and guide metadata if those capabilities are adopted.
   `StatAes`/`BinAes` provide typed generated-field reads, but cannot express general
   computed expressions or [after-scale evaluation](https://ggplot2.tidyverse.org/reference/aes_eval.html).

4. **GGP-04 — P1 parity gap, Absent: analytical layer breadth.** Related scope is
   GRA-04/06 and SCP-03. A requested [boxplot](https://ggplot2.tidyverse.org/reference/geom_boxplot.html)
   needs whisker/outlier classification and geometry, not just available quartiles.
   A requested [density curve](https://ggplot2.tidyverse.org/reference/geom_density.html)
   needs an estimator/bandwidth contract. The example `example.density_histogram`
   computes normalized histogram heights; it is not KDE. Prioritize commonly needed
   boxplots/intervals, ECDF and explicitly scoped density/violin/smoothing capabilities
   as separate decisions. Each needs independent expected statistics, generated schemas,
   provenance, native/export scenes and actual host proofs.

5. **GGP-05 — P2, Fail for equivalent authoring: grouping is explicit.** GRA-01/03/07.
   [compiler.rs:833](../../crates/chart-core/src/grammar/compiler.rs) selects `Grouping::All`
   unless `SourceAes.group` is supplied. The probe confirmed that category-colored lines
   without an explicit group fail with `UnsupportedCapability`; adding that group succeeds.
   In [ggplot2](https://ggplot2.tidyverse.org/reference/aes_group_order.html), discrete
   aesthetics normally infer groups. This is an intentional authoring difference, not
   justification to change core grouping silently. A convenience/compatibility layer could
   infer groups with an explicit override and fixtures for multiple discrete aesthetics.

6. **GGP-06 — P2, Partial: guides, colors and facets need their own parity work.**
   SCL-05, GRA-07, LAY-01/03, THM-03. An identical two-stop palette does not imply an
   identical gradient: current byte-space interpolation gives midpoint gray 128 for
   black/white, while the reference defaults to Lab. Colorbar rendering is absent even
   where stop metadata is present. `facet_grid(space="free", margins=TRUE)` has no
   direct definition. A Toronto-local time axis remains UTC despite metadata.
   Use the shared planned color/scale/axis engines, then add ggplot2-specific guide,
   facet and timezone acceptance. D3 module completion alone would not certify these.

7. **GGP-07 — P2, Partial/Absent: coordinate and extension limits block some compositions.**
   SCL-01, GRA-06/08, ARC-03 and SCP-03. A circle needing fixed data aspect, a general
   polar bar chart or a post-stat nonlinear path cannot be expressed as an equivalent
   coordinate configuration. The existing `Cartesian` wrapper exposes the current mapping;
   it is not a coordinate registry. Planned radial D3 shapes do not automatically implement
   polar axes, clipping, guides or statistical coordinate behavior. Accept these products
   explicitly before adding them; share geometry/scales across hosts.

8. **GGP-08 — P2, Uncertain: there is no ggplot2-wide differential certification.**
   QLT-02/03 and BND-03/04. [Statistical fixtures](../../fixtures/statistics/README.md)
   contain independent expectations and selected D3 R-7 examples. Existing Rust/Python/WASM
   agreement demonstrates a shared engine, not agreement with ggplot2. Pin an accepted
   ggplot2 baseline and store outputs for matched options/stages; use semantic tolerances
   separately from physical-layout/rendering expectations. Keep deliberate differences
   explicit. Do not broaden current G2 evidence into full library parity.

## Checks, limits and next action

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust `1.97.1 (8bab26f4f 2026-07-14)`:

| Command / evidence | Result |
| --- | --- |
| `mise exec -- cargo test -p chart-core --test grammar --test statistics --test full_scales --test facets --locked` | **47 passed, 0 failed:** grammar 20, statistics 14, full_scales 4, facets 9. |
| `mise exec -- cargo test -p chart-export --test composition --test publication --test extensions --locked` | **20 passed, 0 failed:** composition 6, publication 10, extensions 4. |
| Temporary Rust probe linked against the compiled core | Reproduced absent single-panel legend and explicit-group requirement; [retained source](ggplot2-parity-2026-09-07/probe.rs), [output and build identity](ggplot2-parity-2026-09-07/probe.log). |
| `mise exec -- python3 scripts/check_repository.py` | Passed workspace/dependency isolation and local Markdown links; this is not target execution. |
| `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`; inline Python review check | Passed ledger whitespace, new-report whitespace, eight finding IDs and test-count summaries. The retained probe was formatted with `mise exec -- rustfmt --edition 2024 docs/evidence/ggplot2-parity-2026-09-07/probe.rs`. |

These checks establish existing local contracts and the specific probe observations.
No fresh Python/WASM runtime, ggplot2/R oracle, native UI, image inspection, Linux,
full repository matrix or performance certification was run. `Rscript` was not found
on the current PATH. Existing export tests execute rendering assertions, but do not
establish inspected ggplot2 visual parity. The probe uses deterministic synthetic text
metrics and tests scene content, not font fidelity. Concurrent inspection/state/category
window and D3 planning edits were present and preserved; test results belong to the
binaries built during this review, not every subsequent live edit.

Review outcome: complete; implementation parity remains open. No implementation,
fixtures, baselines, dependency versions or normative requirements were changed by this
assignment. Only review evidence and the required status-ledger entry were added.

Next action: fix the reproduced single-panel legend defect first. Then select a bounded
ggplot2 capability target, retaining current semantics unless an explicit compatibility
policy is adopted. A practical order is independent aesthetics/guides and mapped text,
interval/boxplot/statistical breadth, then facets and coordinates. Reuse the already
planned D3 foundations where applicable; do not duplicate their color/scale/path engines.
