# ADR-032: Registered authoring recipes and guide drawing

Status: Accepted for implementation; GG-16 acceptance remains open.

Authoring dispatch uses immutable entries in the existing `ExtensionRegistry`.
A materializer accepts an explicit bounded payload and returns the existing `Data`
owner, whose batch is revalidated against the caller's limits. An authoring recipe
receives that data, validated parameters and compile limits and returns ordinary
`LayerBuilder` values. `autoplot` and `PlotBuilder::autolayer` compose these through
the primary compiler. They neither introduce a host compiler nor retain Python,
JavaScript or R objects. Host syntax adapters invoke the same Rust methods. Recipes
execute once during authoring; prepared charts retain the resulting ordinary typed
layers and source. Subsequent source updates use normal compiler invalidation.

Registered legend keys receive the semantic value, occurrence, layer identity,
destination units and final guide-only aesthetic overrides. Their output is bounded
portable path content with declared positive local dimensions. Existing custom-guide
geometry validation checks output, and the common path transform/scene pipeline owns
publication and native paint. Prepared charts pin immutable registrations; replacing
another registry cannot change an old snapshot. Unknown versions, duplicate entries,
malformed output and native-only portable execution reject explicitly. Registrations
are trusted native code and cannot be preempted by the synchronous core; resource
limits are supplied and returned geometry is checked before scene emission.

Definition envelope 82 carries registered key selections. Materialized recipes lower
to existing authoring semantics and do not add a serialized executable recipe. The
external `chart-extension-example` implements diamond keys and an XY materializer/
recipe without private core APIs. Focused tests cover independent normalized data,
schema and budget errors, repeat/round-trip behavior, missing registrations and
native-only boundaries. Actual Rust/Python/WASM and native publication remain
required before GG-16 closes; these two protocols do not stand in for remaining
coordinate, facet and full guide extension requirements.

Full guide drawing extends the same registry with a separate callback over trained
scale metadata, complete colorbar samples, ordered labels/values, explicit destination
units and resource budgets. The result uses existing `CustomLegend` layout and vector
paint; no extra destination renderer is introduced. Envelope 83 retains the exact
selection. Invalid dimensions, path budgets and out-of-bounds geometry reject. Path
bounds include the shared lowering error envelope, which is accounted for separately
from the declared content bounds. Native-only entries reject portable serialization.
The external strip guide and malformed-output tests exercise this boundary; actual
host/native qualification remains open for this new slice.

Facet planning (envelope 84) consumes a coherent source snapshot and the resolved
ordinary catalog. It returns a concrete `FacetSpec` over the same exact variables;
shared validation rechecks unique keys, coverage, budgets and grid topology. The
prepared snapshot retains that plan for destination layout; future source updates
recompute it. The external reverse planner exercises order and column topology.

Paired coordinates (envelope 85) train immutable normalized x/y maps from explicit
domains and views, pinned by the prepared registry. Shared subdivision, clipping,
raster sampling, annotations, guides and navigation use the same checked map. Finite
output and explicit omission/inverse capability are distinct. This contract currently
uses a Cartesian base view, with nonlinear behavior inside the paired callback;
composing it after radial/geographic clips rejects instead of misrepresenting their
clip geometry. The external wave map has a closed-form paired inverse.

Custom coordinate maps must additionally return a conservative normalized enclosure
for any input rectangle. Point/corner samples do not bound arbitrary nonlinear
curves. The shared subdivision owner uses this enclosure and its existing work/depth
budgets; returned bounds must be finite, ordered and contain mapped corners. The
external wave implementation includes analytic sine extrema and a two-endpoint
regression proves that extrema absent from the input vertices survive projection.
