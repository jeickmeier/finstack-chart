# GG-07 polygon, tile and raster recipe evidence

Scope: GG2-06 / FIX-GG07 surface subpackage; shared recipe interfaces, compiler hooks and host wire/version integration are coordinated with the GG-07 integration owner. This report does not close the full package.

The implementation uses the common source/statistical row reader. Tile extents are established after scale-stage resolution and before positions. Polygon groups retain source order within subgroups and all original targets. Raster cells retain calculation-space bounds, source-order target identities and bounded integer grid indices; destination lowering creates one shared RGBA raster. Missing cells are transparent and duplicate grid positions follow source-order last-write painting. Raster hjust/vjust and truncating irregular-grid indexing follow the pinned reference. Raster metadata contains one hit rectangle per source target; holes in the grid create no target. Polygon even-odd and nonzero rules use the same scene path, export renderer, native renderer and inspection flattening engine.

Scene 21 is required only for even-odd compound paths or nonempty raster cell hit metadata. Prior ShapePath primitives retain nonzero fill and omit the new default field; existing GG-08 annotations omit empty cell metadata and retain scene 20. Source/static ownership and calculated pixel counts are checked against existing vertex/path budgets. No renderer owns a second raster recipe implementation.

Source evidence:

- `fixtures/parity/ggplot2/primitive-recipes.json`: pinned polygon winding cases, tiles and raster build/draw data.
- `tools/reference/r/surface-controls.R` and `fixtures/parity/ggplot2/surface-controls.json`: independent ggplot2 4.0.3 / R 4.6.1 captures of raster justification and irregular-grid placement, including reference warnings.
- Read-only exact method capture: `/private/tmp/gg07-surface-source.txt`, GeomRaster setup/draw and GeomTile setup.

Focused validation so far:

- `/tmp/gg07-surfaces-test2.log`: all four initial surface tests pass; source-order polygon hole queries, default/explicit tile extents and generated tile preparation, raster alpha/interpolation and individual target queries after portable plot replay.
- Same log: three authoring-host tests pass, including numeric field name, FieldHandle and source expression labels. This also fixes the inherited GG-08 host dispatch boundary and numeric-expression identity selection.
- Additional pinned raster justification/index and mapped-size/nudge/statistic tests have been added; their final results follow in the integrated qualification update.
- `examples/common/ggplot_surface_recipes.rs`, `crates/chart-export/examples/ggplot_surface_recipes.rs`, and independent `scripts/bindings/ggplot_surface_recipes.py` / `.cjs` cover nine plot authors and 27 SVG/PDF/PNG publications per host. Python/WASM also assert exact numeric field/expression text labels. Final actual host runtime results are owned by the integration run.

Remaining qualification: final focused tests after category-offset integration, fresh actual export/native visual inspection, actual Python/WASM publication comparison and the package-wide required checks. Initial Rust export exposed the generated categorical tile offset issue and was not accepted as complete evidence.

Final focused integration update:

- `/tmp/gg07-surfaces-test5.log`: seven surface contract tests PASS, including exact pinned justification, integer truncation on uneven raster spacing, mapped tile dimensions with one nudge, generated polygon/raster targets and logarithmic scale evaluation before raster extent construction.
- `/tmp/gg07-surfaces-export-test.log`: one actual export regression PASS; center PNG pixels prove even-odd holes and orientation-sensitive nonzero fill, SVG retains the requested rule, PDF publication succeeds, and both raster image-rendering modes reach SVG.
- `/tmp/gg07-surfaces-export-final.log`: all nine Rust authors, 27 publications PASS, with exact original-versus-portable-replay scene comparisons before export. Output: `/private/tmp/gg07-surfaces-final`.
- Visual inspection: all nine PNG/SVG/PDF triplets inspected at `/private/tmp/gg07-surfaces-final-inspection/review-1.png`, `review-2.png`, `review-3.png`. Holes, explicit tile separation, generated categorical tiles and raster missing-cell placement are correct. The smooth-alpha example exposes the pre-existing PDF/resvg interpolation-kernel difference; each consumer enables smooth interpolation, but cross-format pixel identity is not claimed.
- Surface defaults are resolved once from shared styles: grey fill, explicit/mapped independent outline, reference linewidth conversion and authored unit overrides. The generic legacy point/rectangle styling contract remains unchanged outside these recipes.

Native and actual Python/WASM qualification remain in the coordinated final package run. No separate host builds or full-suite reruns were performed by this subpackage owner.

Native surface qualification: six-case native gallery built in the combined three-gallery command, painted six valid scene stamps and was visually inspected at `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_20-21-25.png`. Trace `/private/tmp/gg07-native-surfaces.log`; all six native surface cases pass. See `phase-2-ggplot-native-2026-09-15.md` for commands and separate interval/column findings. The owned native process was closed after capture.

Final missing-control correction: mapped Width/Height values that are missing suppress the tile/raster cell instead of falling back to a constant or resolution. Unmapped controls still use defaults. The shared `number_or` and `retain_positions` helpers preserve positional scale training even for omitted paint. Independent `missing-recipe-controls.json` captures the pinned tile NA extent behavior. The focused regression checks both dimensions, explicit fallback constants, one retained mark and the omitted x=100 domain contribution: PASS `/private/tmp/gg07-surface-missing-final.log`.
