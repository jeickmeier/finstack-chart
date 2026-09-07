# ADR-005: Foundational scale, tick and destination-layout algorithms

Status: ACCEPTED for WP-06 and WP-10; full scale families remain WP-11.
Date: 6 September 2026. Requirements: SCL-01/02/04/05, LAY-01/02, DAT-05. Fixture: FIX-07.

## Shared preparation and destination boundary

`grammar::Compiler` remains the only source/filter/stat/position route. `layout::layout`
consumes an `Arc<PreparedChart>`, explicit destination bounds/units/font, axis policies
and a synchronous `TextMeasurer`. It produces `LaidOutChart`: the exact retained prepared
snapshot, validated destination `Scene`, plot rectangle, named resolved transforms,
final guide labels, parallel per-item targets, status and aggregate diagnostics. Layout
never invokes source accessors, changes a source population, moves bin edges or reruns a
statistic. Resizing/font changes resolve new ranges/margins; viewport changes affect only
mapping and ticks. Hidden layers still contribute to training by default.

`Layer::scales` binds stable `ScaleId`s; zero and one are primary x and y. Named scales
train independently and cannot serve both orientations. Every used scale needs an
`AxisSpec`, even if its guide is hidden. The foundation supports at most four independent
scales and at most one visible guide on each bottom/left/top/right side. Independent axes
are not alternate-unit secondary axes. Explicit exclude-hidden training and secondary
one-to-one mappings remain WP-11/12, not an implicit interpretation of independent axes.
`PreparedChart::domains` remains the primary-axis view; `scale_domains` includes all IDs.

The existing `SceneStamp` gains the captured presentation `state` revision, so a visibility
change cannot share a stamp with a different painted result. Definition, coherent store,
layout/profile/font and viewport revisions remain separate. The caller advances the layout
revision when effective destination inputs change. Errors retain the requested stamp;
scale failures identify the named scale and text-service failures retain the exact font
identity/revision. Returned scenes/snapshots remain immutable after subsequent failures.

## Numeric domains, mapping and ticks

Continuous precedence is explicit domain, otherwise eligible post-stat/position endpoints
plus the declared none/zero/value baseline, then enabled padding and nice policy. Explicit
nonconstant domains are exact and preserve direction; baseline/padding/nice never alter them.
All options are validated, even if precedence leaves them inactive. Empty numeric training
uses [0,1]. Constant zero expands to [-1,1]; other constants expand symmetrically by 5% of
absolute value, with a minimum step of the smallest positive binary64 value. A symmetric
expansion that cannot remain finite fails with precision diagnostics. Explicit constants
use the same expansion. Padding is a nonnegative fraction of domain width on each side.
Nice expands to a 1/2/5 decimal step grid using its fixed 2–128 target; layout/zoom does not
change that training target.

`LinearScale` retains domain, distinct viewport and distinct destination range separately.
Ascending and descending domains/ranges are valid. Mapping uses checked normalization,
halved differences when subtraction overflows, and weighted interpolation inside the range.
Unrepresentable extrapolation/inversion fails; no NaN/infinity is published. `Extend`
(default), `Clamp` and `Omit` are explicit visible-domain policies, independent of upstream
statistics and destination clipping. Inversion is an unclamped numeric operation.

Numeric ticks use a bounded 1/2/5 decimal grid. Decimal rounding up to 15 places avoids
accumulation labels such as 0.30000000000000004 on a 0.1 grid. Coalescing grid candidates
at a large binary64 origin yield only distinct representable ticks. Endpoint fallback is
used when no grid tick lies inside the interval. At most `max_ticks + 1` candidates are
visited; unsupported targets/budgets or excess output reject. Labels use portable shortest
binary64 formatting, Unicode minus and normalized signed zero. No machine locale is read.
The logarithmic family is explicitly unsupported here; FIX-07's log-invalid entry proves
rejection, not log mathematics. Log/symlog/color/session-time families remain WP-11.

## Categorical identity and projection

`Numeric::Category(FieldId)` is an explicit positional encoding for categorical fields.
It is invalid in numeric filters/statistics/size inputs. Source geometry stores an ordinal
into its owning layer's retained label catalog; ordinals are not row/category identities.
Global training collects eligible geometry labels in retained first-seen relative order
and unions catalogs by label across layers. Empty eligible data gives an empty categorical
domain. Removing/filtering categories can change band positions; reappearance preserves
relative first-seen order until the existing explicit category-order reset. An explicit
band domain fixes membership/order and can retain absent labels without changing statistics.

`BandScale` resolves centers, oriented extents and destination category lookup. It exposes
no numerical inverse. For n labels, denominator = max(1, n − inner + 2 × outer), center i
is `(outer + i + (1 − inner)/2) / denominator` of the oriented range. Inner padding is
[0,1), outer padding is nonnegative; defaults are 0.1 and 0.05. Missing explicit-domain
categories are omitted at projection with counts. Gaps/outside positions have no category;
a shared unpadded boundary belongs to the next band, and the final far edge is included. Numeric band viewports and
continuous clamp/omit settings reject; changing the explicit category domain is separate.
Points/lines use centers. Rules/rectangles use their explicitly encoded endpoints; dedicated
categorical bar edge mappings remain with the later geometry family.

## UTC integer/calendar contract

Unix timestamp units remain explicit seconds/milliseconds/microseconds/nanoseconds. Layout
adds the layer origin in checked integer arithmetic before calling `UtcScale`. The scale
subtracts its integer viewport start before any float conversion. Relative offsets and
visible spans must fit exactly within 2^53 source ticks; otherwise choose a closer viewport,
origin or coarser source unit. The original timestamp remains in the pinned source snapshot.
Clamp/omit checks happen in integer space before a potentially imprecise outside conversion.
Inversion rounds to the nearest source tick (half away from zero), with checked integer
origin addition. Constant time domains expand by one source second on each side; overflow
rejects. Empty automatic UTC training uses integer epoch endpoints [0,1] in source units.
Explicit integer domains are retained, independently of the visible interval.

UTC ticks support source-tick/second/day steps, Monday weeks, calendar months and calendar
years. Fixed steps align to the Unix epoch (Monday weeks offset by four days); months/years
align on calendar starts. Calendar arithmetic is proleptic Gregorian with the 400-year
146097-day cycle, anchored to 1 January 2000 = Unix day 10957. Formatting/ticks accept years
0001–9999 and validate real month/day/leap-day boundaries. Tick generation has an explicit
output budget (maximum 4096); no process timezone, locale, I/O, leap-second table, exchange
calendar or local DST conversion is implied. Subsecond labels retain all source-unit decimal
places and `Z`; date/month/year labels use UTC calendar components.

## Bounded plain-text layout and scene contract

At most **four axis measurement passes** grow margins monotonically. Each pass uses the
exact requested font descriptor, size and destination units. The final plot, scale ranges,
ticks and label coordinates come from the same pass. Final label placement follows axis-ID
and destination-coordinate order, discarding duplicates, overlaps and labels outside the
figure/plot span deterministically. Category candidates are stride-thinned before measuring
when the catalog exceeds the per-axis tick budget. Thinning or reaching the pass cap emits
`LayoutPressure`; there is no unbounded convergence loop. The maximum axis callback count
is four times axis count times max ticks, plus at most one compact-state label measurement.

Defaults are 8-unit figure padding, 4-unit ticks/gaps, 12-unit plain font size, a 24 × 24
minimum useful plot and six desired numeric/time ticks. At tiny bounds or excessive text
pressure, `NoSpace` returns no plot/transforms and a fitting “Not enough space” label, or
an empty finite scene when even that cannot fit. Empty/hidden/fully omitted geometry and
empty-count histograms return `NoData`, with a fitting logical label. No invalid zero-width
scale or fake zero-valued observation is manufactured.

Geometry stays finite and uses a common rectangular clip: plot by default, explicit figure
clip for annotation overflow. Omitted line vertices split runs; singletons become points of
half the stroke width, preserving a valid isolated mark without inventing a segment. Normal
lines become numeric paths; rules/rectangles project their original endpoints before bounds
arithmetic. Targets remain per source/generated mark or per path vertex; decorations have
empty targets. Native/export/hit-test consumers must use these same transforms and clips.

Layout preflights axes, semantic compatibility, options, font/resource budgets, category
counts/bytes, vertices, potential split-line items, guide items and path work. Potential-item
accounting conservatively includes labels before thinning. Category bytes count trained and
explicit catalogs against the text budget before cloning. All candidate label bytes are
checked before each measurement pass; final Scene construction checks aggregate output
bytes/resources/structure again. These are work/allocation bounds, not timing or RSS claims.

Plain runs preserve logical text and explicit font identity/size/units. This consumes the
existing WP-02 service and the WP-03 destination strategy; it does not implement a shaper or
claim font-ink bounds from advance metrics. Real native/publication measurement/painting
bridges are WP-07/08. Rich runs, rotation, shaping descriptors, language/direction, deliberate
fallback, shared panels and legends remain WP-12/13. No new dependency is introduced.

## Acceptance tolerances and remaining gates

[Scale tests](../../crates/chart-core/tests/scales.rs) lock domain endpoints/category identity
exactly; ordinary numeric round trips use operation-specific absolute tolerances from 2e-15
to 1e-12. [Layout tests](../../crates/chart-core/tests/layout.rs) compare independently known
400 × 240 point/rectangle coordinates exactly where representable and use 1e-10 destination
units for fractional band centers. The narrow 1e16-origin numeric fixture permits one ULP
(2 source units) on inversion; 17/257-nanosecond offsets at a large time origin permit at
most one source tick. UTC calendar timestamp literals are independently checked against
Python's UTC datetime arithmetic. No fixture tolerance or visual baseline was loosened.

WP-06 evidence does not close all cited specification requirements or G1–G4. WP-07 and
WP-08 now consume this one shared layout/scene contract. Full scale/coordinate/typography,
publication, binding and interaction evidence remains with its assigned later packages.

## WP-10 statistical and position completion — 6 September 2026

The [statistics contract](../statistics-contract.md) extends this accepted decision with
exact count/automatic bins/grouped summaries/intercept OLS, source/transformed spaces,
explicit stack/normalize/dodge/jitter, typed generated schemas and membership. It records
nonzero-width bin defaults, compensated arithmetic and centered/scaled OLS, failure rules,
per-field metadata, position hashing/units/order, work budgets and exact full-recompute
capability declarations. These use the existing compiler and scene projection; no dependency
or host-specific statistics engine was added. Additional scale families remain WP-11.

[Canonical fixtures](../../fixtures/statistics/README.md) own the expectations. A narrow
external comparison uses the published finite-input R-7 examples from
[D3's quantile documentation](https://d3js.org/d3-array/summarize#quantile), retrieved
6 September 2026 and stored locally; its empty/invalid summation semantics are not adopted.
Rust tests never require a reference engine. Actual Rust/Python/WASM cases independently
compare stats/provenance, final destination scenes and SVG output. The WP-10 evidence report
records the executed scope; this is not full G2 or G4 certification.

## Accepted WP-11 extension — 7 September 2026

The [scale and geometry contract](../scale-geometry-contract.md) now defines implemented
log/symlog/point/color/supplied-session families, portable axis policies and distinct
secondary unit guides, area/ribbon fills and OHLC/volume/interval/cell recipes. These use
the existing compiler and destination scene pipeline. The palette is declared sRGB-byte
interpolation; supplied calendars imply no exchange correctness. Nonlinear labels round
to twelve significant digits while mapping values remain unchanged.

The native family example adds a direct dependency on already-locked workspace `serde`
only to decode the shared fixture catalog; no package/version was added to the lockfile.
Actual core tests, macOS vector captures, SVG/PDF/PNG and Rust/Python/WASM evidence are in
the [WP-11 report](../evidence/wp-11-completion-2026-09-07.md). This closes the required
family slice, not facets/theme/extension G2 or full release gates.
