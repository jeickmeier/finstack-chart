# WP-H03/H04 standalone layout acceptance

Status: COMPLETE for H03 tree/cluster and H04 partition; H07/H08 own chart/host acceptance.
Revision: working tree over `a6caa39`. HIR-03/04, FIX-H01-C/D.

Tidy tree uses iterative Buchheim walks/threads; cluster uses common leaf-depth alignment.
Both use the same topology and configurable extent/node-spacing mode, reference separation,
constant/depth policies and native separation callbacks. Fixed spacing roots at the origin;
configuration readback switches the inactive mode to absent. Immutable layouts retain their
source topology and owned finite geometry; invalid callbacks/parameters reject.

Partition uses explicit sum/count values, retained internal own-value slack, unit-size
zero-padding/no-round defaults, signed padding/midpoint collapse and shared JS rounding.
Its horizontal splitting helper is shared with the following treemap implementation.
No axes/renderer/runtime are needed for these kernels.

`cargo test -p chart-core --test hierarchy_layouts --test hierarchy_topology --locked`:
**6 tests pass**. Geometry includes **340 pinned tree/cluster layouts and 42 partitions**,
plus independently checked leaf alignment, root origin, callbacks/readback, missing-value
rejection and internal own-value slack. The 256 additional deterministic asymmetric cases
exercise contour threads and both normalization modes; all original 441 fixture cases
remain unchanged. The expanded 697-case oracle regenerates identically. Strict focused
Clippy and formatting pass; [logs](phase-2-hierarchy-rectangles/) are retained. Coordinate
tolerance stays `1e-10 + 1e-12 * abs(expected)` as set before implementation.

This is standalone core acceptance, not native/radial/portable/performance certification.
Next: H05 all treemap tilers, custom padding/tiling and explicit resquarify history.
