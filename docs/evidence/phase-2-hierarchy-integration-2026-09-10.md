# Hierarchy integration and final acceptance

**WP-H07 and WP-H08 are COMPLETE; G-HIERARCHY passes** for the complete finite typed
capability surface of d3-hierarchy 3.1.2, with the explicit native/portable adaptations
in [ADR-022](../adr/022-hierarchy-topology-and-layout-history.md). All HIR-01–08 and
FIX-H01-A–H rows have evidence. This closes the owner's hierarchy assignment after
axis and interpolation certification. Revision is the working tree over `a6caa39`;
[source hashes](phase-2-hierarchy-integration/source-snapshot.json) identify the qualified
implementation. No commit or release is implied.

The [65-row method catalog](phase-2-hierarchy-integration/verdict-catalog.md) and
[machine catalog](phase-2-hierarchy-integration/verdict-catalog.json) enumerate all
16 exports, 14 node methods, 25 factory controls, two ratio factories and eight integrated
FIX subsets. Original oracle inventory labels describe its historical entry stage; the
final catalog owns current verdicts. No in-scope capability is left deferred or uncertain.

## Shared implementation and capability boundary

`chart-core::hierarchy` owns standalone construction, topology, traversal, aggregation,
sorting, tree/cluster, partition, six treemap tilers, packing and enclosure. Native
callbacks and registered portable operations enter the same bounded kernels. Python and
WASM forward owned descriptors to the core session; no per-node interpreter callback or
second layout implementation is introduced. Registered children, value, comparator,
separation, radius, padding and custom tiler examples execute from the external example
crate. Unknown operations and native-only portable construction/publication reject.
Native Rust construction/layout continues to accept native-only implementations.

Primary recipes cover Cartesian/horizontal/radial tree and cluster, icicle, sunburst,
treemap and packing. Preparation constructs/filter-checks/aggregates/sorts once; panel
allocation fits numeric layouts before shared shape/path lowering. Fixed node spacing
retains destination units; extent mode fits the panel. Sunburst exposes inner radius and
linear/area depth mapping. Pack uses layout radii directly. Source color, themes, facets,
clips and inset destinations use existing owners. Node labels are semantic inspection
metadata; automatic visible label placement is not a D3 hierarchy capability.

Standalone session/snapshot version 1 retains exact topology, normalized configuration,
geometry, registry and compact resquarify rows. Hierarchy primary/scene envelopes use
version 15; non-hierarchy envelopes retain their previous versions. Constructor selectors
are immutable caller-owned options: read/change/reset those values, then construct the
result. Layout configuration is queryable. Per-side treemap accessors override global
padding, then numeric options; outer padding assigns four sides. Effective ratio readback
clamps finite authored ratios to at least one. Host declarations test these descriptors
with strict mypy and TypeScript, including negative TypeScript controls.

Checked finite-input differences are intentional: exact occurrence IDs use decimal u64
strings; negative/non-finite weights and invalid final coordinates reject; all-zero fitted
packing returns finite collapsed geometry. Native payload sharing becomes owned results
at host boundaries. Arbitrary JavaScript coercions, mutable function identity, DOM objects
and interpreter callbacks are outside this typed adaptation. These policies are stated
in the method catalog and ADR rather than reported as missing implementations.

## Acceptance evidence

| Contract | Executed evidence | Verdict |
| --- | --- | --- |
| HIR-01/02, FIX-H01-A/B | Nested, custom iterable and ordered grouped input; ID/parent and escaped paths; structural errors; all traversal/query/sum/count/sort/copy methods; exact large keys; native callback row/index/root context and 20,001-node iterative chain. | PASS |
| HIR-03/04, FIX-H01-C/D | Tree/cluster extent/node spacing, registered separation, partition rounding/padding, own-value slack, zero/degenerate/extreme cases and independent coordinate expectations. | PASS |
| HIR-05/06, FIX-H01-E/F | All six tilers and standalone parent/bounds calls; padding/radius/custom operations; ratios; retained histories and topology rebuilds; deterministic packing/helpers with independent containment and overlap bounds. | PASS |
| HIR-07, FIX-H01-G | Nine independently authored projections in actual Rust/Python/WASM, versioned roundtrips, registered chart operations, supplied fonts/themes, facets/insets, compact provenance, nested hits, radial seam/hole behavior, keyboard metadata, SVG/PDF/PNG and native inspection. | PASS |
| HIR-08, FIX-H01-H | Stateless batch equivalence, equivalent-history replay, duplicate transactions, resize/reparent/selection/eviction, failed-layout last-valid frame, captured exports after updates/disposal, linear membership and explicit node/depth/payload/work/path/cache budgets. | PASS |

The original **826 oracle sequences**, **72 additional independent per-side accessor
cases** and **27 configuration set/readback/reset sequences** pass in each of Rust,
Python and real Node WASM. The original corpus is unchanged. The padding corpus covers
four tilers, all five sides plus outer-equivalent assignment, and depth/field/registered
accessors. Configuration tests include every factory control and both clamped ratio
factories. Independent reference cases retain their predeclared coordinate tolerances:
`1e-10 + 1e-12 * abs(expected)` for points/rectangles, and
`1e-8 + 1e-10 * abs(expected)` for circles. Packing's local intersection/enclosure residual
policies remain separate. Topology, identities, order, metadata and snapshot restoration
compare exactly.

Nine chart contracts cover all recipes and formats; sunburst holes and either side of the
radial seam; frontmost nested leaf hits and clipping; zero-area structural metadata;
retained resquarify rows across resize; reparented node identity with changed link identity;
selection persistence/eviction and historic reselection rejection; facet/inset-specific
membership/history; registered and native-only boundaries; a 2,048-node membership chain,
weak-owner release and explicit resource errors. Source filtering that removes a required
parent rejects. Invalid topology can enter the data store, but layout rejects it and the
last acknowledged frame/export stays intact; this is not a claim that data ingestion itself
rolls back arbitrary invalid hierarchy edits.

Each actual Python/WASM host executes **18 update/resize/reparent traces** against a second identical
replay, with duplicate-transaction detection and retained exports. The five stateless
families additionally perform **15 fresh-batch comparisons** per host. Stateful resquarify
compares equivalent histories; numerical reference traces separately prove retained rows,
reset and topology changes. The [cross-host replay comparison](phase-2-hierarchy-integration/updates-compare.log)
checks exact node identities and family-specific numeric tolerances.

## Inspected destinations and retained artifacts

The [81 master chart artifacts](phase-2-hierarchy-integration/artifacts/) contain nine
primary definitions and 18 text/outline scene/SVG/PDF/PNG sets. Three independently authored
hosts have identical SVG/PDF/PNG bytes and equivalent semantic JSON, with exact source
keys above 2^53 and root total 29. [Hashes](phase-2-hierarchy-integration/chart-artifact-hashes.json)
retain all three host outputs, including two additional host resize scenes. Gzip files
retain the standalone, padding and control results without duplicated uncompressed data.

All nine cases were inspected in the freshly rebuilt macOS GPUI gallery, including
Cartesian and radial links, fixed node spacing, icicle depth bands, sunburst inner hole,
nested packing and treemap padding. Native rendering and accessibility exposed the expected
9-node or 17-node/link target counts. The test app was closed after inspection.

All nine text and outline SVGs were independently rasterized with resvg 0.45.1; PDFs
were independently rasterized with Poppler; all nine text/outline PNGs were inspected.
The [inspection sheets](phase-2-hierarchy-integration/inspection/) retain these six grids
and [native screenshot](phase-2-hierarchy-integration/inspection/native.png). Geometry,
colors, hierarchy depth, title/legend text, inner holes and padding are consistent.
Root and leaf point glyphs at panel edges obey the authored plot clip. This is inspection
at the recorded 500×360-point/144-DPI publication profile and local native display, not
certification of arbitrary system fonts, every display, or a Linux native renderer.

## Resource measurements and remaining release work

The [resource protocol](phase-2-hierarchy-integration/resource-protocol.json) was fixed
before measurement. At 1,000 nodes, after 20 warmup and 60 disposal cycles, WASM linear
memory retains **zero growth** from 7,929,856 bytes; Python `tracemalloc` retained growth
is **4,828 bytes**, below the predefined 4 MiB bound. Python instrumentation measures
Python allocations, not Rust heap usage. Owned node/scene/SVG results survive mutation,
double disposal and forced WASM memory growth; disposed handles reject. Rust weak-owner
checks independently establish source-membership release. Each hierarchy shares one
preorder key array and per-node ranges instead of quadratic subtree membership copies.

The release-profile [component benchmark](phase-2-hierarchy-integration/component-benchmark.json)
records five workloads at 100 and 1,000 nodes: balanced tree, long chain, high fanout,
skewed pack and resquarify update/resize. It preserves cold measurements, 10 warmups,
30 raw samples, nearest-rank p50/p95/max and geometry/history/serialized-size counts.
All stages meet the declared bounded-completion budgets. At 1,000 nodes, numerical
preparation p95 is 1.47–2.07 ms and layout p95 is 0.64–3.05 ms. High-fanout hit-index
construction reaches 60.00 ms p95 and its query reaches **18.49 ms p95**. This is a
measured WP-22 follow-up, not an interactive latency pass. Measurements ran locally
alongside qualification jobs and are not isolated throughput certification.

WP-22 retains allocator-event instrumentation, native ingest-to-present latency and
PERF-01–05 acceptance. WP-21/23, G-GGPLOT, G-PARITY, G-AUTH's remaining native performance
scope and G4 remain separate. These releases are not approved by component parity.
Within Phase 2, all eight D3 lane gates now pass; **17 ggplot2 packages remain outside
this assignment (54 of 71 accepted)**.

## Reproduction and qualification

[Environment](phase-2-hierarchy-integration/environment.json) records Rust 1.97.1,
Python 3.14.6, Node 24.14.0, wasm-bindgen 0.2.128, renderer identities and the source
snapshot. The stored reference pins d3-hierarchy 3.1.2 at
`7bea49efdc1093b28a8d60d2f8717d8a17803564`, with source/license/package hashes.
No production dependency or oracle version upgrade was introduced.

The permanent `scripts/run_primary_authoring_proofs.py` now includes hierarchy builds,
standalone/padding/control replay, nine chart projections, updates, ownership and typed
consumer checks. This task ran those hierarchy components directly against freshly built
extension-proof modules; it does not claim a new aggregate run of every older host proof.
The exact command sequence is retained in [commands](phase-2-hierarchy-integration/commands.md).

Final qualification: **474 macOS workspace tests/doctests and 455 Linux headless
tests/doctests pass**, with zero failures. Full `mise run check` covers repository rules,
dependency licenses/sources, formatting, all targets, strict Clippy, rustdoc and the WASM
core build. Strict Python/TypeScript consumers pass. Source changes remain uncommitted;
subsequent edits require requalification against their own source snapshot.
