# ADR-018: Scale compatibility, descriptors and supplied resources

Status: ACCEPTED for SP-01 contract and measurement. Date: 9 September 2026.
Requirements: SCL-06/07/08, ARC-04, BND-01, QLT-02. Fixture: FIX-20.
Amends [ADR-005](005-foundational-scales-and-layout.md). Implementations remain with
SP-02–07; accepting this contract does not advance G-SCALE.

## Reference and method dispositions

The reference is d3-scale 4.0.2, source commit
`83555bd759c7314420bd4240642beda5e258db9e` (peeled `v4.0.2` tag). The shared
[Node lock](../../tools/reference/node/package-lock.json) pins all runtime oracle
dependencies. [Inventory](../../fixtures/parity/d3-scale/inventory.json) enumerates
all 26 factories, every inherited own method and default getter result. The two
additional exports are `scaleImplicit` and `tickFormat`. [Manifest](../../fixtures/parity/d3-scale/manifest.json)
records source digests, integrity, Node and generator identities. D3 is development-only.

All numerical mapping, inversion, ticks, nice, range sampling, breakpoint and formatter
results on equivalent typed inputs target equivalent results. Domain/range/parameter
setters become checked immutable reconfiguration and getters return independent owned
values. `copy` retains independent configuration while immutable compiled resources may
share ownership. Existing method gaps remain explicit in the executable report.

`scaleImplicit` is an explicit authoring/training policy. Preparing an ordinal catalog
may add missing keys in deterministic input order; scene mapping and inspection cannot
mutate it. Typed keys distinguish strings, numbers, booleans and timestamps; exact
signed/unsigned 64-bit identities are independently tested beyond JS Number's range.
Object identity and implicit JS coercion are excluded; bounded structured outputs use
[ADR-017's shared values](017-shared-interpolation-values.md), never stringified keys.
The accidental d3-scale 4.0.2 quantize `unknown()` getter returns its scale function;
the typed getter returns the authored unknown value. Forward behavior remains required.

Missing, undefined and unbounded results are typed, separate from diagnostics. Shared
Number tags preserve NaN, infinities and negative zero in standalone operation results.
Undefined numerical reference results may return the documented NumericalDomain
operation diagnostic; the original descriptor remains inspectable. Defined finite
results from constant/repeated domains, exponent zero/negative, empty collections or
unusual finite parameters cannot be rejected by a broad family-level ban. Ordering
constraints apply to knot/breakpoint search; unsorted reference inputs outside its
ordered-domain precondition receive explicit validation diagnostics. Every exception
must be recorded at the operation level, rather than counted as parity.

## One mapping implementation and compatibility policy

Extend `chart-core::scales`, consuming shared interpolation and numeric formatting.
The existing finite `LinearScale`/`NonlinearScale` projection routes will delegate to
the same prepared numerical kernel used by compatible standalone scales. No duplicate
D3 engine, scale crate, renderer callback, Python math or JS math is introduced.
Compiled interpolators and search tables are prepared once outside mark loops.

The established recipe behavior is the explicit **Legacy** policy: automatic/explicit
constant-domain expansion, existing count bounds/grid, unbounded navigation inverse,
positive log validation, categorical placement and byte-palette interpolation retain
their historical meaning. New **D3** descriptors provide matching constructor defaults,
constant and repeated knots, aligned bands, unknown values and inverse clamping.
The policy selects preparation/tick rules; it does not duplicate arithmetic engines.
An explicitly requested unbounded inverse remains available to viewport navigation.

Domain, viewport and destination range remain distinct. Piecewise domains retain every
knot; viewport edits cannot replace the domain by two endpoints. Positional projection
normalizes the prepared scale's mapped viewport endpoints into the destination range,
and reverses the same composition for inversion. Arbitrary output values have only
applicable capabilities: classifier extents, ordinal lookup, band starts/width/step;
sequential/diverging scales never advertise a general numeric inverse. Rounded or
constant mappings do not promise exact round trips. Chart geometry rejects non-finite
results before scene publication.

## Serialization, interpolation and time boundaries

New standalone scale descriptors start at version 1 with an explicit family and
compatibility policy. Plot definition version 5 will be introduced with the first
new retained scale capability; readers continue to accept versions 1–4 under their
existing meanings. Legacy wire inputs cannot silently opt into D3 policy. Unknown
variants, future versions, invalid configuration and missing resource revisions reject
before changing a live chart. SP-01 changes no production wire reader or public API.

Range factories consume `InterpolationFactory`/`Interpolator` and owned shared values;
colors consume CLR-04 Paint descriptors and one final lowering boundary. Native custom
factories and registered portable resources carry identity/version and deterministic
resource ownership. Registration remains the existing extension system's responsibility.
No host-language callback or renderer object is placed in a portable descriptor.

Local calendars consume a supplied resource with zone identifier, tzdata identity,
covered UTC interval, initial offset and ordered UTC transition/offset records. No
system timezone, clock, network or filesystem is read by core. UTC is explicit. Local
floor/ceil/offset and formatting share the same resource and distinguish folds/gaps;
requests outside coverage diagnose. The oracle retains Node tzdata/ICU versions and
rules for UTC, New York, Berlin and Lord Howe, including non-hour transitions.
D3 overlap uses exact millisecond Date values; existing integer-origin submillisecond
projection remains independently tested and does not inherit Date's precision limit.
Locale resources are explicit, immutable and revisioned; SP-05 implements numeric
specifiers and SP-06 time formatting against the pinned reference.

All knots, parameters, range kind, factory/resource identity, policy, locale and zone
revision participate in relevant cache keys. Palette-only edits cannot retrain numeric
positions. Distribution scales train from eligible post-stat samples, independently
of viewport; corrections and retention must match a fresh batch. SP-07 supplies full
host/interaction/native/publication acceptance and WP-22 owns measured budgets.

## SP-02 implementation clarification

Full-domain positional projection uses the authored numeric range endpoints, so a
repeated first domain knot can leave the initial output interval unused just as in
the reference. An explicit viewport uses the scale-mapped viewport endpoints and
retains interior domain knots. Constant output spans project to the destination
midpoint. This separates valid standalone constant mappings from exact inverse claims.
Numeric knot axes currently support after-statistics projection; a before-statistics
grammar profile must explicitly select `coordinate_scale` until GG-04 supplies its
shared pre-stat transform policy. Silent omission of that population stage rejects.


## SP-03 implementation clarification

`OrdinalScale<K,V>` accepts native ordered keys and generic output values. `train`
returns an independent catalog and only appends under `OrdinalUnknown::Implicit`;
`map` is always read-only, returning undefined for an untrained implicit key. Explicit
unknown values cannot add keys. `ScaleKey` preserves null, boolean, number, signed
integer, unsigned integer, timestamp and text variants. Floating keys use SameValueZero
(NaN keys coalesce, as do signed zeros); other types never coerce. Exact 64-bit keys
serialize as canonical decimal strings using the existing portable integer codec.

`CategoryScale<K>`, `BandScale` and `PointScale` share one prepared spacing kernel.
Existing recipe options select Legacy placement; `BandSpec`/`PointSpec` select D3
placement, stable deduplication, alignment, round/rangeRound and defined finite negative
padding. D3 padding-inner caps at one and alignment clamps to [0,1]. Lower band starts
match D3 in either direction; oriented extents preserve descending dodge offsets.
Chart wrappers retain complete category catalogs independently of visible windows.
Prepared key lookup and point inspection use logarithmic search. Points expose zero
bandwidth and category lookup, never a numerical inverse.

`AxisScale::D3Band` and `D3Point` require definition version five and use the existing
resolved band/point routes for guides, projection, dodge and navigation. Host standalone
constructors and method qualification remain SP-07. Native generic descriptors have no
host objects; their strict versioned host envelopes are also owned by SP-07.

## SP-04 implementation clarification

`ContinuousScale` and `NumericScale` share knot selection and normalization. The typed
continuous range consumes the existing interpolation factory; sequential, diverging
and empirical rank scales separate `ScaleNormalizer` from output sampling. Native
custom outputs use the existing `Sample<T>` contract through `map_with`; versioned
registered chart interpolation remains WP-IP06's integration responsibility.
Sequential/diverging families deliberately expose no general numeric inverse.

`ClassifierScale<V>` prepares quantile/quantize breakpoints once. `ThresholdScale<K,V>`
keeps nonnumeric ordered inputs and generic outputs. Equality uses right bisection;
duplicate outputs use their first occurrence for inverse queries. `ScaleExtent` records
membership separately from optional bounds and tagged NaN. Quantile sample sorting
excludes missing/NaN observations while retaining infinities. Its reference quantile
arithmetic is shared with the rank scale; the earlier robust statistical quantile has
a different overflow contract and keeps its historical arithmetic. Thresholds retain
reference binary-search behavior for authored descending cuts without claiming that
these represent sorted intervals. A singleton quantize range retains the reference's
undefined upper inverse endpoint; an empty quantize range diagnoses its RangeError.

Scale-generated IEEE parameters use an internal route through compiled interpolation
kernels. Public animation sampling still requires finite parameters. This distinction
preserves defined constant/color results for singular scale transforms; undefined
piecewise selection diagnoses at the operation. Typed `None` remains missing, while
D3 diverging's null-to-zero coercion is compared with an explicit numeric zero input.
Empty rank identity outputs preserve negative zero and singleton ranks preserve NaN.

Mapped color and numeric style scales use `MappedScaleSpec`. `ScaleTraining::Eligible`
collects the complete eligible post-stat population across layers sharing the scale ID,
before any mark mapping or viewport projection. Corrections rebuild from that population.
Color guides retain actual intervals, midpoint and full mapping contract so factories
with identical endpoint colors cannot silently share an incompatible guide. Numeric
outputs support size, opacity and stroke width; existing numeric axes remain the
compatible positional route. Width/color must be constant within a line or filled run.

Source threshold/ordinal inputs retain exact typed keys, including integers beyond
binary64 precision; statistical numeric fields supply numeric keys. Prepared discrete
paints are parsed once. Encoded paint values stay floating through after-scale expressions
and opacity multiplication, then lower once to existing sRGB8 scene styles. Earlier byte
palette mapping remains explicit. Numeric style fields and mapped color scales require
Plot definition v5. Standalone host envelopes/method proofs remain SP-07; this package
adds no Python or JavaScript mapping arithmetic.

## SP-05 numeric tick and formatter implementation

`tick_candidates` and `tick_step` expose the reference data-space grid. Numeric,
continuous typed, sequential/diverging and quantize methods consume these shared
helpers; logarithms use signed major/minor candidates and independent label suppression.
The count is a numerical hint, while a separate hard budget rejects excess output
without substituting a coarser grid. Layout retains its bounded count and collision
policy. Empty log labels retain their tick marks and bypass text measurement; collision
thinning does not alter standalone candidates. Legacy recipe ticks/formatting remain stable.

`NumericFormat` retains the entire specifier and inline `NumericLocale`; their values
constitute configuration identity, with no process locale or mutable global formatter.
All D3 numeric types, sign/padding/grouping, precision inference, SI and fixed-prefix
formatting use one prepared formatter. Significant and fixed decimal rounding share
an exact binary-to-decimal kernel with upward decimal ties. The existing shortest
number formatter remains unchanged; its optional fixed conversion is not used because
1e-25 at 20 places was experimentally incorrect. The added pinned corpus covers that
case and extreme/subnormal inputs. Log base-e powers use the existing correctly rounded
exponential dependency after the former libm call differed at exp(1).

Axis `numeric_format` requires definition v5 and validates even an empty population.
It cannot coexist with legacy `number_format` or explicit tick labels; categories/time
reject it. Nice is an immutable outer-endpoint edit, preserves a diverging midpoint,
and rebuilds quantize breakpoints. Rank/quantile scales expose their actual quantile
methods and reject continuous tick/nice requests. SP-07 still owns actual standalone
host and integrated scale qualification; WP-AX02 consumes these shared algorithms.

## SP-06 calendar and time implementation

`Calendar` owns floor/ceil/round/offset, filtered interval ranges, automatic selection
and nice. Its shared Gregorian components also serve the legacy UTC scale; there is
one civil-date implementation. Local operations require `TimeZoneRules`: version one,
zone identity, owner revision, tzdata identity, inclusive UTC millisecond coverage,
initial offset in seconds east of UTC, and strictly ordered transitions. Rules are
immutable and supplied explicitly. Missing coverage diagnoses rather than falling
back to a machine timezone. Fold resolution chooses the earlier instant; gaps shift
forward. Day/month/year offsets use wall time, while minute/hour offsets use elapsed
time. Default weeks start Sunday; Monday remains an explicit interval.

`TimeScale` composes integer-origin domain normalization with the existing continuous
range factory and numerical inverse. Native timestamps remain integer seconds,
milliseconds, microseconds or nanoseconds. Submillisecond automatic ticks/default
labels preserve that quantum. Spans/origin differences beyond 2^53 ticks diagnose
precision loss. Calendar operations additionally require the common Date range and,
for local operations, supplied resource coverage. Numerical geometry can retain larger
integer epochs independently of calendar formatting.

On exactly representable millisecond Date domains, inverse compatibility evaluates
the shared weighted absolute numeric inverse before Date truncation. This deliberately
preserves reference rounding near epoch-millisecond boundaries; forward geometry
still subtracts an integer origin first. Finer/larger native domains retain the exact
origin for the inverse too. Reference invalid Dates become checked numerical-domain
diagnostics because the timestamp result is an integer. Fractional primitive numeric
inputs to D3 time scales require an explicitly finer native unit; the millisecond
integer API does not silently truncate them.

`TimeFormat` owns explicit locale strings and compiled directive patterns. Conditional
formatting and custom labels use the same `Calendar` and resource revision as tick
placement. Legacy UTC grids stay unchanged unless a calendar scale is selected;
a `time_format` on legacy UTC changes labels using UTC. The new `Calendar` axis variant
and any `time_format` require definition v5. Numeric/custom guide formatters cannot be
combined with time formatting. Time axes require numeric linear/rounded range factories,
retain piecewise knots, and share numeric projection, inversion and navigation.

Standalone time envelopes are version one with canonical integer strings and bounded
portable decoding. Actual Python uses native integers; WASM uses bigint timestamp
arguments/results. Thin owned handles execute the shared core, including calendar
operations, copy/disposal and strict envelope loading. The SP-06 runtime proof uses
revision 7 of the pinned fixed-zone resources and revision 42 in retained DST chart
fixtures. Public scale facade/type coverage across all families remains SP-07.


## SP-07 integration decisions

An owned `StandaloneScale` dispatches the shared family kernels through one strict
portable operation interface and one thin handle per host. No host arithmetic or
second scale engine is introduced. Native typed family APIs remain available.

Mapped eligible populations are frozen after statistics and before encoding across
all participating facet panels. Rendering then consumes those same populations.
This preserves free positional axes without separately training shared color/style
scales. Broadcast/chart-scope tables contribute once; numeric duplicates retain their
sample weight. Ordinal order is authored keys followed by layer, panel and row order.
Data correction/removal/retention rebuilds the current population; navigation does not.

Numeric threshold domains select numeric source conversion in primary authoring;
explicit typed integer/string domains select exact keys. This prevents host column
inference from changing the meaning of numeric cuts. Classifier guide labels compact
binary64 decimal noise with adaptive significant digits; exact interval metadata is
unchanged. These decisions are covered by the SP-07 population and host figure proofs.
