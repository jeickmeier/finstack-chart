# Facet and shared-layout contract

WP-12 extends the same compiler and scene solver used for single-panel charts.
The [specification](spec/gpui-charts-specification.md) remains normative. This contract
records the concrete GRA-07/08, SCL-05 and LAY-01/02/03 slice; publication furniture,
rich typography and complete interaction are later packages.

## Panel identity and population

`ChartDefinition.facets` optionally contains a `FacetSpec`. Wrap uses one field and a
positive column count. Grid uses two fields, with the first identifying rows and the
second columns. `order` is an explicit catalog of distinct `PanelKey` values. A key
contains exact text, signed/unsigned integer or boolean values; it is independent of
source dictionary codes, row positions, panel order, bounds and destination units.
Integers retain their full wire precision. Null keys are excluded with diagnostics,
or reject under a strict matched layer's invalid-input policy.

The catalog must cover keys observed after the contributing layer's source filters,
including filters inherited through named transforms. Undeclared observed keys reject;
this API does not silently append panels. Wrap fills successive rows in catalog order.
Grid row/column order follows first appearance in that catalog and requires every
row/column combination. Grid holes remain explicit cells. No exchange/calendar or
finance interpretation is attached to facet keys.

`EmptyPanels::Keep` preserves empty cells. `Drop` removes panels without a matched
filtered source population; broadcast annotations do not keep an otherwise empty
panel alive. Wrap packs retained panels; grid preserves their catalog row/column
coordinates. An all-dropped figure has the ordinary no-data state. Numeric invalidity
and outlier exclusion are reported separately from source filtering.

`FacetTarget::Match` is the layer/transform default and requires every facet field.
`Broadcast` explicitly repeats the input in all panels. `Panels(keys)` repeats it only
in distinct declared target panels. A missing facet field never implicitly broadcasts.
Consumer targets must be compatible with the availability of their named dependencies.
Each panel retains the same immutable source store, original row keys, generated schema
and complete aggregate/model provenance; it does not create replacement datasets.

`PreparedChart.panels()` exposes ordered panel preparations. Root `layers()` is their
paint-order concatenation, so layer IDs can repeat there; `(panel key, layer ID)` is the
scoped identity. `LaidOutChart.panels()` retains each cell, plot and resolved axes.
`item_panels()` aligns one-for-one with scene items and targets; figure furniture has no
panel. `InspectedTarget.panel` prevents inspection in two panels from becoming the same
effective target merely because a broadcast source row is identical.

## Statistics and reuse

`StatScope::Group` preserves the statistic's declared groups within a matched facet.
`Facet` computes one population per facet by setting statistical grouping to `All`.
`Chart` uses the whole filtered chart input and one group. Source filters remain active
in every scope. Scope controls computation, while `FacetTarget` controls panel matching
and placement; a missing field still requires an explicit broadcast/target declaration.
Generated chart-wide aggregates cannot be refaceted: their presentation must explicitly
broadcast or target panels. A chart-wide operation cannot recover observations already
removed by a facet-scoped dependency. These invalid dependency combinations reject.

Operation records retain scope and matched panel metadata. Facet aggregate/model scope
strings include the field/key identity. Named graph outputs are shared within a panel
and cached by source snapshot, exact transform definitions, compiler limits and facet
scope. The cache retains at most the authored panel catalog for the current source/graph;
source or transform changes release prior entries. Whole-chart computations currently
may evaluate once per panel cache; this is semantic sharing, not an incremental or
performance claim. The existing exact full-recompute fallback remains unchanged.

## Domains, guides and clipping

Shared x/y policies union compatible post-stat/post-position contributions across the
retained panels. Each orientation can independently opt into free training. Categorical
unions retain layer-local ordinal decoding; incompatible units/origins/transforms reject
only where sharing is requested. Log training scans positive eligible geometry across
all contributing panels. Explicit domains and independent viewports retain their
existing precedence. No scale/layout step reruns source statistics.

Color guides merge only when scale identity, semantic title, complete domain/palette,
continuous/discrete flag and missing style agree. Incompatible guides remain separate;
different scale IDs never merge just because their colors match. Automatic per-panel
color domains may remain separate; an explicit common color domain supplies a shared mapping. `collect_guides` paints
compatible guides once outside the figure's panels; false paints each panel's guides
outside its plot. `ColorEncoding.title` supplies a human-readable legend title.

Ordinary geometry clips to its resolved plot. Explicit `ClipPolicy::Figure` uses the
enclosing figure even inside a facet. Painter, export scene and inspector receive the
same explicit clip and panel identity.

## Bounded layout and evidence

At most **four synchronized text-measurement passes** solve all panels. Every panel uses
the largest required side margins, so corresponding plot edges and sizes align even
when free-axis labels differ. Native and publication measurers receive their exact
destination font, revision, size and units. A destination/font change may alter margins
and tick thinning, but never source/stat values.

Facet headers retain full logical text and clip to their cells if too long. Legends
reserve at most 30% of their enclosing width; long labels clip and trailing swatches
can be omitted under pressure. Complete legend metadata remains prepared. Headers,
legend pressure, tick thinning and the four-pass cap have explicit layout-pressure
diagnostics. Unusable cells show `Not enough space`; useful empty cells show `No data`.
No zero-width scale or invalid rectangle is fabricated.

The catalog is capped at `min(max_groups, 256)` panels; keys and titles have 4096-byte
text limits. Compiler row/value and geometry budgets cover the entire figure, including
work for dropped panels. Destination scene/text/path/resource budgets remain enforced.
The [nine facet tests](../crates/chart-core/tests/facets.rs),
[shared runtime fixtures](../fixtures/facets/portable-cases.json) and
[WP-12 evidence](evidence/wp-12-completion-2026-09-07.md) establish this scope. They do not
close full typography, publication, selection, accessibility or performance gates.
