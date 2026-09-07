# Statistics and positions

WP-10 implements GRA-03/04/05, the built-in portion of GRA-08, DAT-05/06 and QLT-02
through the existing `grammar::Compiler`. Definitions, generated schemas, calculation
spaces, provenance and position parameters are portable. Python and WASM execute these
same implementations. [ADR-005](adr/005-foundational-scales-and-layout.md) fixes the
algorithms; [the fixture catalog](../fixtures/statistics/README.md) records expectations.

## Statistics and scope

| Operation version 1 | Parameters / output | Defaults and exceptional inputs |
| --- | --- | --- |
| `chart.identity` | Existing source or generated rows, with unchanged provenance | Source mappings and generated mappings remain disjoint. |
| `chart.count` | `CountSpec.required`, grouping; exact `Count` and `Group` | Empty required list counts rows with a valid group. Every explicitly required numeric field must be finite/exactly representable. Empty count is zero. |
| `chart.bin` | `BinSpec`; `BinnedRow` start/end/count/group/target | Explicit intervals are [left,right), final right included. `Exclude` reports below/above; `Error` rejects; `Overflow` includes outside members in the nearest finite edge bin, retaining below/above counts. No infinite edges are emitted. |
| `chart.auto_bin` | `AutoBinSpec`; same typed bin output | Thirty equal-width bins over the filtered finite population, with common edges across eligible groups. Empty population uses [0,1]. Constant zero uses [-1,1]; other constants expand by 5% on each side, with minimum half-width `f64::MIN_POSITIVE`. Nonfinite or indistinguishable edges reject with precision diagnostics; choose fewer or explicit bins. |
| `chart.summary` | `SummarySpec`; count, group, min/max, mean, sum and indexed quantiles | Default quantiles `[0.5]`; authored probabilities must be in [0,1]. Empty min/max/mean/sum/quantiles are missing. `empty_sum_zero` changes only empty sum. |
| `chart.ols` | `OlsSpec`; two fitted endpoint rows with count/group/X/Y/intercept/slope and membership | An intercept is always fitted. Both x and y must be finite. Each declared group needs at least two usable distinct x values. Singular or unrepresentable moments/coefficients/endpoints reject. No confidence bands are implied. |

All source filters run **before** statistical input transformation. The default population
is the whole filtered dataset or its declared `Grouping::Field` partition, independent of
viewport and highlighting/visibility. Zoom never changes bins, summary membership or model
coefficients. Facet/panel scope extensions remain WP-12; implicit visible/screen statistics
are not exposed by these built-ins.

`StatSpace::Data` uses source units. `Transformed` names the registered version-one affine
operation and finite nonzero factor/offset; OLS declares x and y spaces independently.
Generated fields carry their actual calculation-space metadata. Layout maps their already
computed values directly; it does not execute the input transform again. Count is always
dimensionless. OLS coefficients describe the declared model's calculation units and are not
silently transformed into source units. Mixed incompatible spaces on one axis reject.
Additional nonlinear scale/transform families belong to WP-11.

Source keys are sorted before numeric accumulation, making a reorder of unchanged keyed
observations immaterial. Sum uses Neumaier compensation and a scaled compensated fallback
if an intermediate partial sum overflows. Mean compensates values divided by n, avoiding
an overflowing total. A mathematically unrepresentable sum still rejects the summary.
Quantiles sort finite values and interpolate at `h=(n-1)p`, using a weighted expression for
opposite signs to avoid overflowing `b-a`. Endpoints and singleton quantiles are exact.

OLS shifts x by its minimum and scales by its finite span before computing compensated
centered moments. Responses are scaled by their largest magnitude. Slope/intercept and
endpoint predictions come from that one model. Centered predictor sum-of-squares at or
below machine epsilon is singular. A predictor span or coefficient that cannot be expressed
as finite binary64 rejects instead of producing invalid paint geometry.

## Generated types and provenance

`PreparedRows::Statistical` and `OutputSchema::Statistical` distinguish count/summary/model
outputs from original observations and existing bins. `StatAes`/`StatField` cannot read a
source `FieldId`. Schema fields declare physical kind, nullability and calculation space;
`Group` exposes an explicit categorical label catalog. Missing output fields or nonnumeric
size mappings reject before geometry. `StatisticalRow::count` stays u64, decimal-string on
the wire; numeric projection checks exactness before narrowing. Numeric values are finite
or `None`/JSON null. `StatField::Quantile(i)` addresses the authored probability list.

Count/summary rows carry aggregate identity, exact input revision and sorted usable source
members. Fitted endpoints carry a derived operation/model target with input version and
scope/group identity, and retain the exact filtered paired membership in their generated
rows. The operation record also retains source filters, grouping, full parameters and
exclusion counts. Inspection never substitutes one representative source row for a model
or aggregate. Old prepared snapshots and their targets remain valid at their pinned source
revision after a correction; resolving them against a different revision rejects.

The existing bounded named-transform graph caches unchanged inputs/definitions. Identity
consumers share the generated row allocation. A second nonidentity source statistic cannot
reinterpret generated rows as observations. Every operation record publishes exact
`incremental` capabilities: append=false, window=false, correction=false,
full_recompute=true. Changed source snapshots use exact batch recomputation; unchanged
snapshots reuse cached named outputs. WP-18 owns specialized update algorithms and their
streaming responsiveness tests. No approximation is used or advertised here.

Group catalogs and declared position orders respect `max_groups`; automatic edges and
quantiles respect `max_edges`. A statistical row charges one prepared budget unit per
schema field across the whole graph/layer preparation, preventing many small rows with
large quantile vectors from escaping `max_prepared_rows`. Source/bin rows retain their
existing budget accounting. These are allocation/work bounds, not performance measurements.

## Positions

`Position::Identity` remains the default. All other positions require explicit parameters;
there is no implicit seed, width, or series order.

- `Stack(StackSpec)` groups interval rows by matching x/x2 coordinates within a layer.
  The source y is signed height and y2 must explicitly be zero. Positive and negative
  cursors accumulate separately in the declared stable group order; ties use stable target
  identity. `normalize=true` divides by separately scaled positive and absolute-negative
  totals, avoiding overflow of the totals. Normalized heights carry dimensionless data-space
  metadata. Zero-only groups remain zero. Both positioned
  endpoints train domains. Numeric data and zero-preserving affine units are additive;
  offset, timestamp, categorical or otherwise incompatible height encodings reject.
  Stacking currently uses the implemented rectangle/rule interval geometries; further
  geometry families extend this route in WP-11.
- `Dodge(DodgeSpec)` resolves after categorical x-band layout. The total width is a fraction
  in (0,1] of the resolved oriented band width, split into the explicitly ordered slots.
  Missing groups retain their slots; undeclared groups reject. Point/line/rule locations
  use slot centers. Rectangles use slot edges and require both x endpoints in the same
  category. Dodged rules likewise require one category at both endpoints. Resizing and
  reversed bands resolve new destination positions; source domains are unchanged.
- `Jitter(JitterSpec)` requires an exact u64 seed, nonnegative x/y half-widths and `Data`
  or `Display` units. Version-one hashing uses FNV-1a over a stable source/aggregate/model
  identity plus group, then SplitMix64 for separate x/y samples in [-1,1). Mutable input
  revisions, membership order and row ordinal are excluded from the hash. Data jitter
  adjusts endpoints before domain training; categories/timestamps require display jitter
  on the corresponding axis. Display jitter runs after projection in the requested logical
  units/points and never changes domains. One model target moves coherently; line vertices
  with distinct source targets retain their own stable offsets. Clipping/inspection/export
  consume the adjusted final scene.

These are single-layer positions. Group aesthetic color, additional geometry recipes,
facets, complete scale families and production extension registration remain their
assigned packages; this package does not close G2.
