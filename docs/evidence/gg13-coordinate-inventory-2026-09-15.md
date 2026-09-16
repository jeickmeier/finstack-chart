# GG-13 coordinate protocol inventory — read-only preparation

Status: source capture and implementation interface proposal only. GG-12 acceptance is a prerequisite and remains with the root integration owner. No GG-13 runtime or host implementation is claimed. This inventory follows GG-13 in `docs/impl_plans/phase-2-parity-implementation-plan.md` and GG2-08 in the specification.

## Independent fixture

`tools/reference/r/coordinate-controls.R` produced `fixtures/parity/ggplot2/coordinate-controls.json` with local R 4.6.1 / ggplot2 4.0.3. All 24 cases built, projected, munched and drew on an 8×6 inch PDF device. Log: `/private/tmp/gg13-coordinate-reference.log`.

The fixture retains original built layer rows, coordinate-transformed rows, source coordinate path subdivision at segment_length=0.01, panel ranges, coordinate class/linearity/aspect/clip, gtable furniture, and draw warnings/errors. No projection or subdivision expected values come from this repository. The two summary-bin examples intentionally retain the reference segment warning for missing interval bounds; their computed data remain available to distinguish scale and coordinate stage populations.

Cases cover Cartesian zoom/expansion/clip-off, fixed ratios 1/2, flipped intervals, post-stat log/sqrt/reverse transforms, pre-stat scale log versus coordinate log with summary binning, secondary axes under flip/nonlinear coordinates, polar x/y and start/direction, radial bars/ribbons, full and partial radial panels, inner radius, reverse, rotating text, angular/radial primary and secondary guides, and a near-full-circle seam path. Independent radial partial aspect is 0.8535533905932737 rather than an unconditional square.

Command:

```sh
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library \
CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work \
mise exec -- python3 tools/reference/r/run.py tools/reference/r/coordinate-controls.R
```

## Existing owners to reuse

| Existing owner | Reuse and actual boundary |
| --- | --- |
| `layout/coordinates.rs` | `Cartesian` already binds resolved primary axes, performs exact numeric/time inversion and reports capabilities. It explicitly reports `path_subdivision=false`; it is the natural resolved coordinate protocol owner. |
| `layout/types.rs`, `plot/axis.rs`, `grammar/scale_stage.rs` | `AxisSpec.scale_stage`, viewport and `AxisBuilder.coordinate_scale` already distinguish post-stat transforms from population changes. Reuse existing scale transforms and training; do not create another transform evaluator. |
| `layout/axes.rs`, `scales/secondary.rs`, `layout/secondary_discrete.rs` | Existing monotone secondary transforms, additive time transforms and identity discrete secondary guides retain primary scale identity. Secondary guides cannot become independent coordinate axes. GG13 needs their placement sampled through the coordinate map, not another secondary scale engine. |
| `layout/engine.rs`, `layout/facets.rs` | Shared measured margin solver and GG12 panel allocation own plot sizes. Fixed aspect belongs in this solver with explicit interaction with free facet sizes; a geometry-local shrink would make guides and inspection disagree. |
| `layout/project.rs` and recipe projection modules | Shared projected marks retain original targets and component geometry. Install one coordinate map and bounded path subdivision here, including rectangles/intervals and all recipe paths. Per-geom polar adapters would duplicate the protocol. |
| `path/flatten.rs`, `path/lower.rs` | Destination path flattening has explicit error/vertex limits and shared fill-rule hit behavior. Reuse its path representation, work accounting and geometry tests. Nonlinear coordinate subdivision must evaluate the transformed source segment; flattening already projected endpoints is insufficient. |
| `shape/radial.rs`, `grammar/radial_shapes.rs` | D3 radial line/area generators and numeric trigonometry are reusable low-level geometry. They do not supply panel ranges, radial guides, coordinate inverse, seams or clipping semantics. |
| `scene.rs`, `inspection/index.rs`, `inspection/selection.rs` | Current clips are rectangles. A radial annulus/hole needs a shared clip shape or preclipped common geometry consumed consistently by native/export and point/brush/keyboard inspection. A rectangular envelope alone would accept hole hits. |

## Minimal proposed contract

Add one optional chart-level `CoordinateSpec` with explicit legacy absence. Keep axis scale specifications and stat coordinates in their current owners. Proposed variants: Cartesian controls (flip, optional aspect ratio, view limits/expansion/clip), Transformed controls using existing transform descriptors, and Radial controls (theta axis, start/end, direction/reverse, inner radius, radial-axis placement and label rotation). Polar can lower to the radial engine with a documented source compatibility policy rather than a second projection implementation.

Resolve it once per actual panel into a coordinate object borrowing the existing primary axes and finite plot frame. Its interface needs project, bounded path projection, clip containment, and inverse capability. Return explicit unavailable/partial inverse at angular seams, holes, collapsed radii or non-invertible transforms; preserve exact timestamp/category capabilities. Require destination-unit error and total work/vertex budgets for common subdivision. Reuse pinned libm for cross-host deterministic geometry.

Guides should supply existing tick values/labels to this resolved coordinate object for guide curves, ticks and text tangents. Secondary values remain derived from their primary axis. The scene/inspection clip representation is the only likely cross-crate type expansion; determine whether shared preclipping can satisfy all filled/stroked shapes before adding a backend mask abstraction.

The builder/host surface can be one typed coordinate descriptor and optional controls, using the existing generic JSON bridges. It must not preprocess rows in Python/WASM. Wire capability belongs to root and should only advance for the final settled descriptor/scene contract.

## Remaining evidence gates

The source fixture is not implementation parity. Before GG13 acceptance: exact source controls/defaults matrix (including validation and direction/reverse conflicts), transformed straight-line error bounds and termination, disjoint/seam/hole clipping with fill rules, full and partial inverse round-trips, fixed-aspect circles at multiple destination sizes and facets, flipped intervals and text orientation, radial bars/ribbons, secondary guide placement, pointer/brush/keyboard identity, immutable snapshot/update equivalence, actual Rust/Python/WASM original/replay equality, inspected native and SVG/PDF/PNG output, and consolidated prerequisite/package checks. Geography/CRS remains GG15-owned; mathematical labels remain GG14-owned.
