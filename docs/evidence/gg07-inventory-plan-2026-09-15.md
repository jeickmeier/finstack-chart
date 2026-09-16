# GG-07 inventory and source fixture plan

Read-only inventory against GG-07 / GG2-06, following GG-06. No GG-07 feature or
acceptance is established by this note.

| Family | Current source owner | Required completion |
| --- | --- | --- |
| Blank | `Geom::Blank`, `plot::blank`, compiler blank preparation | Existing scale-training/no-paint behavior needs explicit reference fixture and source/stat variants. |
| Points/count marks | `Geom::Point`, numeric size scales, GG-06 count statistic | Add count-mark recipe defaults and weight/size semantics over the existing statistic, without another aggregation implementation. |
| Segments | `Geom::Rule`, `plot::rule` | Add named segment recipe and reference arrow/stroke endpoints. |
| Curves/spokes | D3 `ShapeLink`, curve kernels; radial shape controls | Add ggplot curve controls and data-space angle/radius conversion; D3 bump aliases alone are not parity. |
| Steps | `CurveSpec::{Step,StepBefore,StepAfter}` | Map reference hv/vh/mid after validating ordering/missing/group controls; reuse step kernels. |
| Errorbars/lineranges/crossbars/pointranges | Rules, rectangles and points exist independently | Add one shared interval recipe owner for stems/caps/box/center and orientation, with one row target across components. |
| Reference lines | General rules and line paths | Add data-space slope/intercept and horizontal/vertical intercept recipes with limits-aware clipping. |
| Polygons/holes | Area paths and path subpaths exist | Add group/subgroup geometry and explicit fill-rule/hit-test contract; do not treat precomputed paths as the built-in recipe. |
| Rugs | General rules | Add reference side masks, length/unit policy and clipping using existing scales. |
| Tiles/rectangles | `rectangle`, `cells`, `Geom::Rectangle` | Add center/width/height tile recipe and default resolution policy; reuse rectangle output. |
| Raster | GG-08 portable `RasterImage`/annotation work underway | Reuse shared resource and output renderer; add row-grid pixel ordering/extents and interpolation semantics. |
| Bars | `Geom::Bar` has explicit destination-unit width; histogram uses rectangles | Preserve that legacy contract. Reference data-width/count/identity column recipes should lower to intervals, composing GG-06 positions. |

## Source fixtures

Use the existing pinned R 4.6.1 / ggplot2 4.0.3 runner, one consolidated primitive
fixture script split into named records. Capture actual `ggplot_build` layer data
and selected `ggplot_gtable` primitive coordinates/arrow/fill-rule details, including
setup, transformation and drawing outcomes. Do not infer draw success from build.

1. Intervals: vertical/horizontal, explicit/default cap widths, signed and zero
   ranges, source rows and grouped summary outputs, point middle anchors.
2. Reference lines and steps: nonidentity axis transforms, clipped slopes,
   hv/vh/mid transitions, repeated x, singleton groups and missing breaks.
3. Curves/spokes/segments: signed curvature, angle/radius, arrow at first/last/both
   endpoints, open/closed tips and physical arrow lengths.
4. Polygons: outer ring plus inner hole, reversed ring winding, repeated/empty
   subgroup, independent fill-rule and inspection miss inside hole.
5. Tile/raster: unequal or missing grid cells, explicit pixel-center extents,
   row/column ordering, interpolation, and transformed-coordinate rejection.
6. Blank/rug/count: training-only extrema, no phantom targets, four side masks,
   weighted count sizes and zero populations.

## Implementation and validation order

Implement interval/primitive lowering first, on existing geometry and provenance.
Keep projected curve/arrow/rug operations in layout, reusable shapes in the existing
shape/path modules, and statistics in GG-06. Add polygon/raster cases only after
shared fill/hit-test and image contracts are explicit. Author each representative
case independently in Rust/Python/WASM; retain one compiled module build per
integration batch. Run focused numeric/source tests, then inspect one combined
native gallery and independently rasterized SVG/PDF/PNG contact sheet. Record
unresolved cases explicitly; no cumulative-suite rerun per recipe.

## Captured reference evidence

`tools/reference/r/primitive-recipes.R` now captures 45 pinned source cases in
`fixtures/parity/ggplot2/primitive-recipes.json`, with built layer tables, actual
PDF draw outcomes, panel grob coordinates/units, paints, arrow metadata, raster
pixels, warnings and messages. Forty-four draw successfully; nonfinite curve
curvature builds but fails drawing (`'to' must be a finite number`). Raster under
polar coordinates draws via an explicitly recorded rectangle fallback message.
The invalid polygon rule case is accepted by the source's ordinary polygon draw
path; do not infer uniform validation from its parameter name. Command log:
`/private/tmp/gg07-primitive-reference.log`. These fixtures are preparation for
GG-07, not implementation or acceptance evidence.

## Three-agent implementation ownership proposal

Root owns public declarations (`grammar/recipe_types.rs`, optional `Layer.recipe`),
normalization and builder/host dispatch, wire minimum detection, shared compiler/
layout hook wiring, and final proof runner. Freeze these small interfaces before
parallel implementation; agents should not each extend `Geom` or create independent
recipe engines.

| Lane | Disjoint implementation files | Scope and integration |
| --- | --- | --- |
| A: shared recipe emission and intervals | `grammar/recipe_emit.rs`, `grammar/recipe_intervals.rs`, `layout/recipe_intervals.rs`, `shape/arrow.rs`, `tests/ggplot_recipe_intervals.rs` | Own checked component emission/provenance utility. Linerange/errorbar/crossbar/pointrange, data-space reference lines, hv/vh/mid wrappers over existing step kernels, shared physical arrow endpoint generator. |
| B: surfaces | `grammar/recipe_surfaces.rs`, `layout/recipe_surfaces.rs`, `tests/ggplot_recipe_surfaces.rs`; scoped fill-rule changes in `scene.rs`, `path/flatten.rs`, `chart-export/src/svg.rs`, `gpui-charts/src/native.rs` | Group/subgroup polygon rings, tile extents, raster grids over GG08 RasterImage. Add explicit nonzero/evenodd fill semantics to existing filled paths, preserving legacy nonzero default and wire omission. Rendering and hit-testing use the same rule. |
| C: row and run recipes | `grammar/recipe_marks.rs`, `layout/recipe_marks.rs`, `tests/ggplot_recipe_marks.rs` | Count/default identity columns over GG06 statistics and positions, rugs, curves/spokes and segment controls. Reuse A's arrow/component helper; preserve legacy destination-width bars. |

Suggested minimal contract: each normalized recipe selects existing geometry and
shared mapped values. One preparation hook derives data-space interval extents
needed by positions; one emission hook expands the positioned row into bounded
primitive components; a layout hook handles operations requiring destination
bounds/physical units. Multiple visual components retain the original row target
and cached paint, with no invented statistics or duplicate source rows. Expanding
components *before* stacking would incorrectly multiply heights and is prohibited.
A point anchor and interval bounds must be distinct for pointrange/crossbar and
`vjust`; existing low/high channels should be available uniformly to source and
statistical mappings through the common resolver.

Data-coordinate endpoints contribute to domains. Slope/intercept, angle, and
physical widths are parameters, not independent positional data. Reference-line
clipping must therefore run against resolved axis extents without training domains
from slope values. Grouped runs retain every existing target and missing boundary.
Polygon holes are represented as path subpaths, not independent solid marks.
GPUI already exposes `FillOptions::with_fill_rule`; SVG currently hard-codes
nonzero, while `FlattenedPath::contains` sums winding numbers, so those three owners
must change together for evenodd support.

Exact existing integration points: `compiler::position_layer` (after encoded
style/scale resolution and around the existing position kernel), its geometry
emission match, `layout::project` around prepared Rule/Rectangle/LineRun/ShapePath
projection, and `orientation::{resolve,output}`. Root updates exhaustive geometry
matches/domain contribution and orientation once. Individual lanes submit bodies
behind the agreed hooks rather than editing these shared switches concurrently.
Generated/source variants share the same recipe inputs; count marks call the GG06
count owner, steps call `shape/curves/basic.rs`, arrows call a single new bounded
shape helper, and raster reuses the GG08 image owner. Root integrates one native
recipe gallery and one batch of fresh Rust/Python/WASM builds after focused tests.
