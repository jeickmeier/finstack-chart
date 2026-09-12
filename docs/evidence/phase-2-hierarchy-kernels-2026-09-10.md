# WP-H05/H06 treemap and packing kernel acceptance

Status: COMPLETE for standalone H05/H06; H07/H08 retain registered/host/chart and final
qualification. HIR-05/06, FIX-H01-E/F. Revision: working tree over `a6caa39`.

All six tilers execute independently over a parent/bounds or through shared treemap
layout. Complete constant/native-accessor padding, custom tiling, ratio factories and
rounding are implemented. Explicit resquarify history retains rows across compatible
value/size changes, resetting atomically for new topology/order/owner/ratio. Old layouts
own their geometry and never read the next cache. Membership storage is linear in tree
edges. Invalid custom output leaves history unchanged. The reference's zero-width binary
split can round a zero-height edge backwards; exact collapsed dimensions stay exact in
our checked result, within the predeclared coordinate tolerance.

Packing uses deterministic front-chain placement, shared enclosure and the exact pinned
LCG sequence. Hierarchical fitted/two-pass padding and explicit unscaled leaf radii share
those helpers, with native padding/radius callbacks. Empty helpers, all-zero fitted
collapse, invalid radii, overflow and work limits are explicit. No second circle-layout
implementation enters bindings or recipes. ISC attribution is retained with the kernels.

The combined command `cargo test -p chart-core --test hierarchy_topology --test
hierarchy_layouts --test hierarchy_treemap --test hierarchy_pack --locked` passes
**12 tests**. H05 covers **217 tiler/configuration cases**, three four-step retained
histories, six stable-key topology/ratio rebuild states and independent atomicity checks.
H06 covers **191 hierarchical/helper cases**, including 128 additional seeded cases,
with independent containment/non-overlap and exact unscaled-radius assertions.
All prior 697 oracle cases remain unchanged; the expanded **826-case** reference,
inventory, controls and manifest regenerate byte for byte. Strict focused Clippy and
formatting pass; [logs](phase-2-hierarchy-kernels/) are retained.

Rectangle coordinates keep `1e-10 + 1e-12 * abs(expected)` comparison; circles use
`1e-8 + 1e-10 * abs(expected)`. Placement invariants separately account for D3's intrinsic
`1e-6` radius-unit intersection slack (scaled for fitted output) and the enclosure's
relative weak-containment predicate. Those are local algorithm bounds, not a global
geometry epsilon. This does not claim optimal packing density or native FPS.

Next: H07 registered operations, versioned Rust/Python/WASM access, shared chart recipes,
compact subtree provenance, interaction and retained native/publication integration.
