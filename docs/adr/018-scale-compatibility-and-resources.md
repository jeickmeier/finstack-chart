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


## GG-04 inferred calendar-axis domains

A calendar axis may author `TimeScaleSpec.domain = []` to request population-trained
endpoints. `scale_calendar` and axis validation validate the remaining options using
a temporary two-knot domain; layout resolves the actual extent through the existing
integer-origin time scale. Explicit knots retain their existing behavior, and
standalone `TimeScale` still rejects an empty domain. This extends an existing wire
value without adding a field or a parallel scale implementation.

Under the ggplot2 profile, temporal axes distinguish an empty population from an
all-missing population. Empty data passes an empty vector to a registered primary-axis
label function; all-missing automatic time breaks reject the unbounded domain before
calling labels. Temporal label inputs carry Date/POSIXct units, supplied timezone
resources and automatic candidate names. Censoring precedes the callback, preserving
candidate length and label occurrence indices. `date_labels` formatting bypasses the
registered label function. These contracts have a pinned 400-case panel oracle.


### Registered continuous break selection (12 September 2026)

`MappedScaleSpec.breaks_function` selects a pure versioned operation at guide selection,
after shared population/limit training. Core captures whether the implementation
accepts the continuous `n` argument when registered, bounds returned keys/names, and
preserves names through the label-vector callback. Empty results, NULL results,
constant-domain bypass and transformation errors retain separate reference behavior.
Wire v33 requires explicit registration and never carries executable host objects.
The first qualified route is numeric continuous scales; unsupported families and
competing fixed breaks reject until their own source contracts are implemented.
See the numeric-break section of the [GG-04 evidence](../evidence/phase-2-ggplot-scales-2026-09-10.md).

Discrete reference scales also use the registered break selector. They pass the trained
domain without a count, select returned values using the existing discrete matching
rules, retain first-duplicate names, and skip registered labels when the selected
vector is empty. This preserves positional label behavior by applying the empty-vector
bypass only to the mapped discrete break-function route. Six-channel source and actual
host proofs are recorded in the same GG-04 evidence document.

Numeric binned reference scales consume their registered break result during shared
population training, before mapping and guide selection. The prepared copy carries
explicit cuts and optional aligned names; the authored descriptor remains unchanged.
This prevents reevaluation from giving mapping and guides inconsistent cuts. Binned
callbacks receive sorted source limits and prefer the registered `n.breaks` capability
over `n`; input metadata identifies the supplied argument. Automatic domain extension
is suppressed for function cuts. Constant color/size guide behavior and nonfinite
mapping failures remain distinct, as recorded by the pinned source corpus and actual
host proofs in the same GG-04 evidence document.

Temporal continuous break functions reuse the immutable temporal context already
used for label callbacks. Limits and typed results use origin-relative coordinates
with an explicit origin/unit/Date representation. Native output metadata distinguishes
typed Date/datetime vectors from untyped numbers and NULL without serializing host
objects or executable code. Temporal selection requires the supplied representation;
constant/empty bypass and width/format precedence remain in the temporal guide owner.
This extends the shared native callback contract without a portable wire-version
change. The temporal break section of the GG-04 evidence records source and host proofs.

Numeric positional major-break functions live in `GuideStyle` alongside independent
values, counts and formatting. Selection reads the retained expanded panel viewport
and never retrains or changes mark mapping. It reuses the existing break registry,
which is captured by prepared charts including facet snapshots. Wire v34 adds the
selector to primary/additional guide descriptors; fixed tick replacement clears it.
The numeric primary-axis proof qualifies callback values and names, not internal
invocation multiplicity across measurement passes. Other positional families remain
explicitly unsupported until their corresponding source contracts are qualified.

Discrete positional break functions now use the same v34 guide selector over the
retained categorical provider domain. They reuse discrete matching and label ownership,
including named first duplicates, rather than rebuilding category positions. Trained
empty selections continue to invoke positional label functions, preserving their
separate contract from mapped aesthetic guides. Automatic, band and point source/host
proofs are recorded in the GG-04 evidence; binned and temporal routes remain separate.

Binned positional selectors use the shared binned break owner and wire v35. The raw
axis owns the registered operation; prepared caches retain cuts/names and cannot
carry executable identities. Zero callback counts bypass automatic nice-break count
restrictions. Mapping and guide selection share the nonempty population cut result;
untrained views select against their expanded panel. Added limits precede cuts when
resetting names, yielding blank endpoint labels. Generated blank labels are valid;
user-authored custom-tick validation remains unchanged. Joint limit/break selectors
are explicitly unsupported pending the joint source contract. The GG-04 evidence
records source/native/host validation and inverse-log metadata rounding boundaries.

Named numeric and temporal break outputs now retain their source default labels.
The shared numeric guide owner applies names with label-byte validation. Named binned
color outputs enter the existing parsed/censored key path, preserving infinite key
visibility rather than applying raw candidate visibility. Explicit format policies
remain authoritative, including temporal width/format controls. This is a semantic
correction within the existing wire versions; default-name fixtures run separately
from the unchanged original registered-label corpora.

Joint positional bin limit/break operations now qualify under v35. Initial cut
classification precedes post-statistic limit reset; cached scalar boundaries avoid
repeating the population scan. The shared limit evaluator can retain NULL separately
from an empty vector for this consumer without changing the existing vector wrappers.
An empty source with empty cuts retains empty raw limits and the unbounded empty-range
sentinel, not a finite fallback. Single retained infinite endpoints leave the missing
upper censor bound absent. Invalid-frame label callback scheduling is not a contract:
typed endpoint preflight may reject before formatting. Source/native/host evidence
and the finite guide-position and inverse-log tolerance boundaries are in GG-04.

Temporal positional break callbacks reuse the v34 operation and registry. Typed
origin-relative results can retain fractional source ticks for guide projection;
exact observation mapping and subsecond viewports remain unchanged. Default formats
share the temporal aesthetic guide owner. Width callbacks retain uncropped candidates
through formatting; wall-label reconstruction resolves repeated hours to the earlier
instant. Numeric offsets are accepted as temporal values only for the typed selector
path. Explicit counts adapt the inherited ggplot2 scale field, since its temporal
positional constructors expose no `n.breaks` formal. GG-04 owns acceptance evidence.

Registered minor selection uses the existing pure break registry under wire v36.
The captured arity capability controls the optional major-break vector; major
selection leaves that input absent, and minor selection does not supply a count.
Limits and major values are inverse-transformed before evaluation, then minor results
are transformed and discarded against the panel. Zero ranges bypass minor callbacks.
Default positional numeric labels share the aesthetic optional-label owner, retaining
missing and infinite candidates through censoring. Fixed empty vectors reduce to the
constant major on a zero range; exact empty coupled ticks express suppression in the
qualified adapter. GG-04 records the bounded matrix, raw inverse-log tolerances and
remaining joint-selector and nonnumeric-minor work; minor painting belongs to GG-05.

Joint positional major/minor qualification retains the semantic major vector before
projection in internal layout state. Minor callbacks consume that selection even on
unbounded axes; no callback is reevaluated to recover omitted drawable ticks. Public
axis structures and wire v36 are unchanged. Nonlinear publication endpoint omissions
observed during this qualification remain an explicit GG-05 rendering gate.

Discrete minor callbacks share primary category coordinates and range projection with
secondary guides; zero ranges still invoke the callback. The optional major argument
preserves populated NULL suppression versus empty vectors. The public reference binned
constructor rejects minor callbacks; registered binned minor policies remain explicitly
unsupported. This does not change the existing fixed-minor compatibility routes.

Temporal minor functions share typed expanded limits with major functions. Internal
major selection retains intrinsic names independently of final labels; the pure
registry supplies these names only for functions accepting the major argument.
Fractional Date minors are not floored. Projection first normalizes in the callback
coordinate frame so distant authored origins do not shift panel endpoints. Under the
reference profile the empty datetime extent is one second across source units; an
explicit empty major list on a zero range reduces to the constant, while suppressed
majors stay empty. Existing portable minor operations remain wire v36.


### Registered discrete palettes and missing paint (12 September 2026)

Wire v37 selects a pure installed count palette by qualified identity and bounded
parameters. Shared discrete training owns its input count and ordinal domain; named
results use first-name matching without manual-palette implicit limits. The trained
range retains sorted fallback-slot indices so missing palette elements remain NA,
while unmatched names/inputs and a short palette's appended slot use na.value.
Out-of-range errors occur at mark or selected-guide lookup, avoiding errors from
unused entries in hidden/restricted guides. No host implements palette arithmetic.

Explicit reference discrete NA paint has a retained flag distinct from transparent
paint. Primary explicit color replacement clears that flag. This preserves missing
point removal, replacement paint and guide semantics through the same core path.
The value owner validates the aggregate borrowed palette output before preparation;
registered callbacks never arrive as executable JSON. Continuous and binned vector
palette callbacks require subsequent qualification, as recorded in the evidence.


### Vector palettes and pipeline operations (13 September 2026)

The v37 palette registry also accepts complete normalized vectors. Continuous
mapping deduplicates in first-occurrence order; binned mapping supplies interval
midpoints through the existing bin owner. Native/source and host evidence for both
families is recorded in the scale ledger.

Wire v38 introduces one shared pure numeric-vector operation registry for OOB and
rescaling stages. Call sites supply an explicit stage, transformed vector and limits;
parameters and outputs are bounded, and portable preparation rejects native-only
identities. Callback identity belongs to the authored descriptor; no interpreter or
second scale engine is introduced. Continuous mapping retains callback output arity:
mark assignment recycles singletons and fills empty vectors with missing values,
while guide assignment requires exact length. NULL palettes remain distinct from
empty rescaler output. Binned and positional integration are subsequent work.

Reference point sizes retain infinity through canonical Number serialization for
inspection and after-scale behavior. Their prepared glyph is empty, so finite scene
geometry is preserved without changing the source row count. Reference raster cases
cover this boundary for every numeric point shape. Authoring still rejects invalid
constant sizes; other profiles retain their existing finite-value rules.

### 13 September 2026 — positional OOB vector stages

Wire v40 permits a registered `oob_function` on a positional axis. It reuses the
v38 numeric vector registry. Schema validation checks its identity and parameters;
preparation executes it over ordered source values and then generated coordinates.
Source results are stored by durable row identity in execution-local scaled
mapping state and never enter interchange. The existing compiler pass for shared
post-statistic limits owns final range training and the second vector call.
Automatic empty populations retain fallback limits but have no guide ticks.
The first implemented boundary is numeric point and selected mean-summary mappings;
facets/shared transforms and temporal/binned positional routes require subsequent
source-backed qualification. Complete guide comparison uses explicit `Preserve`
presentation; adaptive tick thinning is a separate presentation choice.

### 13 September 2026 — transformed limit-vector composition

Numeric vector stages retain the complete transformed limit vector, including
empty, singleton and missing-endpoint results. A registered rescaler can ignore
these bounds; the default rescaler validates them when it executes. This preserves
the reference's lazy evaluation rather than rejecting every unusual limit result
at scale construction. Reverse and nonlinear transformations reject NULL limits.

`CustomScaleLimits::requires_domain` defaults to true. A constant operation can
return false to avoid forcing an otherwise unavailable inverse-transformed training
domain. Empty hidden preparations retain authored callback identity while deferring
unused evaluation. These rules reuse the numeric-limit and vector-stage owners and
do not add another normalization engine.

### 13 September 2026 — binned vector stage ownership

Continuous and binned callback pipelines share one ordered numeric-stage owner.
Binned scale preparation retains transformed cuts; each mapping rescales the
observation and cut vectors separately. Cut lookup uses the existing threshold
search, while palette midpoint order follows the unsorted rescaler output.
Non-NULL palette results are shared by clones of the same immutable preparation;
retraining creates a new cache. NULL results remain uncached. A callback is required
to be pure; this cache does not promise a global single evaluation under concurrency.

Reference alpha bytes use ties-to-even rounding, checked against 765 captured
boundary values with round-trip decimal inputs. Generic color conversion retains
its existing contract. Explicit guide-class selection still needs reconciliation:
standalone binned mapping, a generic legend and the constructor's default binned
guide can have different empty-population evaluation paths.

### 13 September 2026 — explicit binned legend and deferred empty sampling

`BinnedLegend` selects point-key mapping independently from the default binned
guide. Definitions retaining this variant require wire v39; older envelopes reject
it. Both guide forms share the prepared binned mapper. Legend key assignment checks
raw palette arity before labels and visibility filtering. Scalar callers resolve
the returned palette index through the same batch assignment policy.

Empty binned populations retain their emptiness through limit callbacks. Preparing
such a scale does not by itself demand a valid sample; actual observations and
guide keys validate when mapped. This preserves reference build outcomes that
differ from standalone sampling without suppressing real sampling failures.


### 13 September 2026 — positional facet vector populations

Resolve panel scale contexts before evaluating positional source vectors. Reuse
execution-local samples across panel preparation; keep them outside interchange.
Train every final shared/free domain before generated vector mapping. Apply each
registered operation to its complete scale population, including empty free panels
when the logical layer is nonempty. Combine and restore layer order before applying
whole-layer return-length recycling. Source points retain insertion order across
panels; statistical outputs retain panel/group order. These policies belong to the
shared positional adapter and reuse the existing vector registry and training pass.
Matched numeric point/selected-summary fixtures qualify this boundary. Broadcast,
chart-wide and shared-transform inputs remain explicit unsupported capabilities.


### Shared positional source vectors (13 September 2026)

A shared source transform owns one callback population. Consumer scale-context
compatibility is resolved before callback evaluation; the node's own filters and
input select that population. Final mapping remains per consuming layer. Identity
operations preserve the upstream summary's empty-positional-population policy,
while the typed inspection table retains empty aggregates. This extends the
execution-local source sampling adapter without changing public wire v40.
The first qualified slice is unfaceted numeric source statistics and identity
consumers. Shared facets and generated-input statistics are separate open boundaries.


### Matched shared facet vectors (13 September 2026)

The shared-node source binder now also runs after all facet axis populations are
resolved. Per-node execution-local caches retain samples across panel definitions;
source and final mapping share the existing ordered fixed/free adapter. A prepared
table exposes its last non-identity population operation internally, so identity
consumers consistently retain summary empty-population and panel-retention policies.
Matched numeric shared facets are qualified by the scale evidence record; broadcast,
chart-wide and generated-input vector statistics remain separate boundaries.


### Temporal positional vector boundaries (13 September 2026)

The existing numeric vector registry receives temporal values in reference units
(days for Date, seconds for datetime/duration). The common population adapter owns
conversion to and from retained origin-relative values; hosts do not implement
another mapping pipeline. Checked represented-number timestamp conversion permits
exactly representable distant callback offsets without widening the source-span
admission rule. The scale evidence records the tested unit/mode boundary.

Fully clipped point circles are omitted before publication precision preflight
and encoding. This conservative circle-bounds check preserves partial circles,
scene contents and existing precision rejection for visible geometry.


### Positional-bin vector evaluation (13 September 2026)

Positional binned OOB callbacks use the ordered source population adapter once,
then the existing bin classifier. Their returned values already occupy transformed
space and must not pass through the scalar OOB/transform policy again. The existing
post-statistic interval mapper remains the sole owner of final bin coordinates.


The bin vector adapter validates an empty return at classification before invoking
later free-panel populations. An entirely empty layer skips the population mapper.
This differs from continuous-vector return recycling and is owned by bin semantics.


### Joint positional-bin callbacks (13 September 2026)

Retain the source limit vector separately from the reset panel range so OOB
callbacks receive its original arity. Scalar classification uses returned values,
not an eagerly classified input population. `ScaleBreaksInput::domain_is_null`
distinguishes NULL limits from an empty numeric vector; bin reset also preserves
the selector's NULL result without adding another portable envelope field.
The shared inverse-log transformation uses the existing `libm` dependency for
identical host results after a one-ULP native/WASM difference was reproduced.


### Blank scale-training layers (GG-04, 13 September 2026)

Pinned `expand_limits()` creates a non-inheriting blank data layer. The canonical
`Geom::Blank` therefore accepts optional positional aesthetics, uses ordinary
statistic/scale/facet training, and records unpainted categorical membership and
finite extents without producing geometry or inspection targets. It is exposed by
one `blank()` builder in all three authoring surfaces. New definitions require
wire v41; earlier definitions retain their existing capability version. Typed
rectangular data and explicit broadcast composition supply helper populations;
full helper constructor convenience and rejection contracts remain unqualified.


### Automatic paint ownership (GG-04, 13 September 2026)

A blank layer mapping another field must train the existing aesthetic scale.
Ggplot automatic color/fill/stroke scales therefore share one owner per aesthetic;
explicit named scales retain their declared ownership. The first automatic owner
retains the palette and title used by later layers. `ColorEncoding.automatic`
records this authoring provenance, defaults to false for older payloads and selects
wire v42 when present. Edits recover the owner from this metadata, including a
subsequent palette edit, rather than infer it from palette equality. Blank geometry
alone requires v41. The legend collector skips blank layers while other layers
still consume their shared training values. Full guide classes remain GG-05.


### Partial temporal population limits (GG-04, 13 September 2026)

`AxisSpec.temporal_limits` reuses exact `ScaleValue::Timestamp` endpoints and
selects wire v43. It applies to Date, UTC and calendar scale families, independently
of a viewport. Missing endpoints are filled from each current population through
the same pre-statistic and post-position limit owner as numeric endpoints. Endpoint
units are converted and the exact source origin subtracted with i128 arithmetic
before floating conversion; offsets beyond the existing 2^53 precision boundary
reject. This avoids narrowing epoch timestamps to floating point. Descending
temporal endpoints remain chronological, matching typed ggplot Date/POSIXct
helpers; numeric reverse limits use the existing reverse transform. Numeric,
category and temporal helper semantics reuse their existing scale owners. An axis
cannot combine temporal endpoints with numeric endpoints or a limits callback.


### Broadcast positional callback populations (GG-04, 13 September 2026)

Broadcast source rows repeat in panel order before fixed-scale vector callbacks.
Free scales invoke the common population mapper separately for each panel. An
execution-local cache therefore retains one row-key map per panel; it does not
change the portable wire. Matched layers continue to order by source insertion.
Post-statistic and positioned populations retain panel blocks for broadcast
layers so index and recycling callbacks preserve the same ordering. Shared source
statistics reuse this adapter and the existing operation registry. Expanded row
counts are checked before allocation. Explicit panel subsets, chart-wide scope
and statistics over generated inputs remain separately diagnosed pending their
population contracts and evidence.


### Explicit panel targets and chart-wide source vectors (13 September 2026)

The broadcast adapter accepts an active panel subset and preserves facet order.
Untargeted panels consume no source callback observations and retain an empty
execution-local sample map. Chart-wide source statistics bypass matched facet
filtering and use the same repeated full-input population adapter. Subsequent
vector stages retain panel blocks for chart-wide source rows as for broadcast
rows. This does not relax the existing rule that generated chart-wide aggregates
require an explicitly broadcast or targeted presentation layer. Existing wire
fields already express these choices; no additional envelope version is needed.


### Source-preserving identity chains (GG-04, 13 September 2026)

The positional source-vector adapter resolves identity-transform ancestors to the
same dataset and retains their filters before invoking callbacks. It does not
reinterpret generated statistical fields as source columns. Faceted identity
chains intersect all ancestor panel targets and retain matched filtering when any
non-chart ancestor matches panel keys. The same source-population traversal controls
source insertion ordering at later vector stages, including a broadcast consumer
of matched source rows. Chart-wide ancestors do not erase downstream filtering.
Generated-statistic input support remains governed by the existing statistic schema
contracts. No portable field or wire version is added.


### Explicit binned guide selection (GG-04, 13 September 2026)

Wire v44 distinguishes explicit bins and colorsteps from existing binned and legend
selection. Scale candidates retain source/scale values; presentation owns normalized
positions. Bins invoke the common mapper on interval midpoints, while colorsteps
invoke it on cuts and interval midpoints. Non-color aesthetics suppress colorsteps
before training because guide selection affects the reference's automatic-limit
timing. Typed authored definitions retain the requested selection; prepared
non-color descriptors contain the effective hidden guide. Full presentation and
collection remain GG-05. Qualification of this adapter is bounded by the captured
selection matrix in the GG-04 evidence record.


## Registered theme palette fallback (wire v45)

Reference `ScalesList$set_palettes` first preserves an explicit palette; otherwise
it searches the declared aesthetic order in the theme and then uses the constructor
fallback. Preserve that distinction in `MappedScaleSpec.palette_theme_aesthetics`:
an empty list means explicit selection; a nonempty list requests lookup while
retaining the current built-in or registered fallback. Explicit palette setters
clear the request. `ThemeSpec.scale_palettes` retains registered operations under
reference palette keys; theme v3 and definition v45 prevent older readers from
silently discarding this behavior. The existing extension registry validates
identity, parameters and destination capability, including unused theme entries.

The compiler resolves selection into a temporary definition before scale training.
It retains the authored definition for serialization and reuse. Lookup copies an
operation descriptor; existing scale owners perform count/vector evaluation,
missing handling, mapping and guide sampling. No host palette engine or global
mutable theme context is introduced. Python/WASM use the same theme command and
mapped descriptor.

This slice qualifies registered theme palettes for continuous/discrete/binned
color, size and alpha, including captured ordered size/alpha lookup, absent first
aesthetics and color aliases. Built-in theme palette values/coercions, complete
element inheritance and all guide presentation remain separate contracts.


## Theme color vectors (wire v46)

Theme v4 adds color vectors alongside the existing registered-operation wire shape.
Continuous and binned selection lowers to the shared reference Lab gradient;
discrete selection lowers to a count palette that pads unavailable entries with
missing values. This differs from a strict manual scale and therefore has a distinct
`Values` descriptor. Palette missing values remain separate from input missing-value
replacement. Authored vectors, including absent entries, survive serialization.
Selection and evaluation remain in core. The mutable selection walker filters
constant-overridden paint channels identically to the read-only walker.

Qualification covers the captured 180 draws and actual Python/WASM edit, replacement,
round-trip and publication proofs. Named palette coercion and automatic constructor
fallback selection remain open and are not implied by color-vector support.


## Default constructor theme selection

Automatic color mapping and the default color constructor retain `colour` lookup;
automatic fill changes that key to `fill`. Continuous and ordinal size, alpha and
linewidth defaults retain their corresponding keys. Explicit numeric ranges clear
lookup; area/radius defaults are explicit palettes and never request it. The pinned
56-draw constructor matrix and actual host edit proofs qualify this distinction.
Default descriptors therefore require v45 even when no theme is currently supplied:
a later immutable theme edit must still affect the retained fallback selection.


## Named theme palettes (wire v47)

The pinned scales 1.4.0 registry contains 138 names. Name lookup is case-insensitive;
strings and one-element color vectors retain the same selection behavior. Theme v5
and wire v47 preserve that distinction from explicit vectors. The immutable core
registry delegates hue, grey, 35 Brewer families and eight viridis names to their
existing owners. It adds the missing 79 fixed HCL ramps and 14 fixed manual palettes.
Continuous coercion samples a discrete palette at its declared maximum (255 for
hue/grey/viridis), then uses the common Lab gradient. Discrete coercion samples a
continuous ramp at evenly spaced positions. Count palettes retain short/overflow
semantics rather than being replaced by a strict manual scale.

Provenance: `named-theme-palettes.R` captures the exact registry installed by scales
`init_palettes`, whose registration order is HCL, base, viridis, Brewer, optional
dichromat, grey and hue. In the pinned environment the 79 HCL entries contain
`grDevices::hcl.colors(31, palette=...)` output; the 14 manual entries contain the
registered base palette outputs. `named-palette-catalog.json` retains these values
and classifications; `extract_named_palettes.py` emits deterministic immutable data.
The new tables contain those palette outputs, not sampled chart results or a copied
HCL interpolation implementation. Existing shared-family code/tables keep their
existing provenance and licenses. No R runtime, process-global registry or mandatory
I/O enters the production engine. Unused unknown names remain lazy as in the reference.


## Style defaults and temporal theme exception

Automatic shape and linetype defaults retain theme lookup; explicit solid/hollow
shape palettes bypass it. Timestamp color/fill defaults use the reference's explicit
gradient and therefore clear theme lookup. Timestamp size/alpha/linewidth retain
the numeric default lookup. These decisions follow the pinned constructors and
36 style plus 60 Date/datetime draws. No new palette engine or wire version is needed.


## Explicit binned constructor count palettes (wire v48)

Public binned paint constructors wrap their explicit discrete palettes with
`pal_binned`; the wrapper evaluates the palette at the number of bin midpoints.
`GgplotBinnedPalette::Discrete` delegates to the same discrete count owner used by
ordinary discrete scales, while generic binned vector functions retain their existing
normalized-domain contract. Version 48 distinguishes the new retained descriptor.

A 16-color hue counterexample exposed a separate polar-Luv parameter mismatch. The
pinned farver runtime installs D65 from chromaticities x=0.31271, y=0.32902; its actual
white reference is X=95.042854537718071, Y=100, Z=108.890037079812814. Core now derives
that white point in its existing polar-Luv helper. The 140-palette capture retains
the independent reference values. Source: scales `pal_hue`, farver `as_white_ref`,
and [farver 2.1.2 conversion source](https://github.com/thomasp85/farver/blob/v2.1.2/src/Conversion.cpp).
This changes scientific parameters in the existing equations, without copying a
conversion implementation or changing the separate Lab/D50 policies.


## Public binned count-function adaptation

The registered vector interface can adapt a reference count palette by passing the
number of normalized bin midpoints to that palette. The portable example's optional
`count` parameter demonstrates this lowering; it adds no new core descriptor or
wire version. Exact callback counts and outputs are compared to 108 pinned draws
in both ordinary and vector-stage routes.

A binned NULL result must remain distinct from a short or empty vector: reference
data-frame assignment creates missing observations, bypassing `na.value`. Color
therefore suppresses the affected points; fill remains transparent with an outline.
The row adapters perform this binned-specific assignment while other scale families
retain their existing NULL-vector behavior. Visible binned guides reject a NULL
palette through a diagnostic instead of assuming values exist.


## Explicit ordinal paint default constructor

`ggplot_color_ordinal` lowers the public ordinal color/fill default to the existing
viridis discrete palette with missing paint enabled and theme selection absent.
Reference `scale_colour_ordinal`/`scale_fill_ordinal` delegate to viridis discrete
constructors, whose NA default differs from the general hue factory's grey50.
The adapter requires no new wire capability or data variant. This explicit factory
does not claim automatic ordered-factor dispatch or arbitrary R constructor execution.


## Ordinal type-vector adaptation (wire v49)

`GgplotDiscretePalette::OrdinalColors` retains reference ordinal `type` color names
and samples their inclusive Lab ramp at the trained category count. It delegates
to the existing gradient owner; named gradient palettes share the same count
sampling helper. A one-element vector remains a lazy text paint, matching
`colour_ramp`'s constant branch, including unknown-name errors only when a mark
needs the color. Empty vectors reject even with empty data. This does not convert
R functions or process options into executable core state.

Wire v49 covers the new descriptor directly and when nested in a binned count
adapter. Older claimed envelopes reject; existing constructor descriptors retain
their existing versions. The 70-draw pinned capture includes alpha and transparent
colors, one/zero colors, missing/empty populations and invalid names.


## Qualitative type-list adaptation (wire v50)

`GgplotDiscretePalette::Qualitative` retains a list of `GgplotQualitativeColors`
vectors and supplied hue fallback. Select the first shortest vector sufficient
for the trained nonmissing category count. Reuse manual exact-name matching for
the selected vector, including first duplicate precedence, without applying the
manual scale constructor's name-based domain filter. Short vectors use the shared
hue implementation. The core representation uses typed strings and keys; R list
evaluation and process-global options are not embedded in portable state. Wire
v50 identifies the retained policy and older claimed envelopes reject it.


## Count gradients and reference alpha encoding (wire v51)

`PaletteSpec::CountGradient` retains a fixed anchor count, the existing discrete
palette descriptor and optional positions. Compilation samples once through the
count owner, then delegates to the existing Lab gradient. This represents public
viridis_c's six-color and distiller's seven-color construction without host color
precomputation or a second interpolation engine. It carries standalone interpolation
v4, standalone scale v6 and plot v51; older claimed envelopes reject it.

The captured gradient alpha midpoint requires farver ties-to-even encoding, shared
with numeric reference alpha mappings in `color::d65::alpha_byte`. General D3 color
quantization retains its contract; this conversion is applied at the reference
palette boundary. The 200-draw constructor corpus and 765-byte-boundary regression
check both paths without changing the independent expected results.


### Reference gradient remapping values (wire v52)

Gradient remapping positions use the shared `Number` representation. The reference
normalizes their own cardinality independently of color anchors, removes NA pairs
after assigning coordinates, averages duplicate positions and retains infinite
endpoints. The Lab gradient remains the sole owner of sampling. NaN remapping
results produce missing paint even for a one-color ramp.

Finite recipes retain their prior envelope versions. Special positions (NaN,
infinities or signed zero) require interpolation v5, standalone scale v7 and plot
v52; downgraded envelopes reject them. Reusing `Number` also preserves descriptor
equality and JSON identity for NaN instead of relying on IEEE NaN equality.


Reference palette wrappers defer invalid-position errors to evaluation, matching
`pal_gradient_n` on empty inputs. The strict direct gradient constructor still rejects
invalid positions. Empty plot preparation skips fabricated reference-gradient
samples, while all-missing vectors and named count-palette lookup retain their
reference errors. This is a preparation-policy correction within the existing
wire capability; it adds no palette engine or callback protocol.


Implicit reference circles and explicit reference symbols share one size/stroke
conversion. Projection applies it before point-stroke device conversion, including
unbounded-coordinate points; nonpositive resulting glyphs emit no primitive while
retaining prepared rows. This fixes publication behavior within the existing
reference profile and leaves explicit legacy radii unchanged. Numeric constructor
range/max_size adapters continue to use the existing power-range owner.


### Built-in reference transform ownership (plot v53, standalone v8)

The GG-04 draft uses one owned `GgplotTransform` descriptor for scalar forward,
inverse, domain metadata and population validation. Both positional
`ScaleTransform::Ggplot` and aesthetic `NumericFamily::Ggplot` delegate to it.
Existing D3/native transform descriptors retain their contracts. Normal quantiles
use bounded inversion of the existing normal CDF; no interpreter or distribution
runtime enters core. Deterministic math functions are checked against the pinned
reference, including endpoint rounding that changes guide censoring.

The existing reference projection adapter retains transformed limits and viewport
endpoints. Finite inverse capabilities require a one-to-one branch with finite,
round-tripping endpoints; infinite or invalid inverse branches stay unavailable.
Secondary guides consume that same inverse and the existing sampled guide mapping.
They inherit a primary reference count when no secondary count is authored.
Break selection stays with the existing continuous-guide and logarithmic-break
owners, including decreasing logarithmic exponent sequences.

Plots carrying these descriptors require v53; standalone numeric, continuous or
interpolated standalone descriptors require v8 and reject downgrade envelopes.
The direct numeric envelope uses v2 for this family and retains v1 for its earlier
families. The existing
registered formatter interface carries custom label functions. The fixed-two-
decimal example callback demonstrates reference rounding without changing the
existing numeric formatter contracts or claiming the full `label_number` API.
The implementation/status ledger records draft transfer and acceptance separately;
composed and registered transforms are not implied by this built-in slice.


### Composed reference transforms (plot v54, standalone v9, direct numeric v3)

Composition extends the same transform descriptor with an owned vector. Scale
families therefore implement Clone rather than Copy; scalar evaluation borrows
the descriptor instead of cloning vectors per observation. Validation rejects
empty compositions and limits recursive depth and operation count using the
existing resource policy. Forward evaluation follows declaration order; inverse
evaluation reverses it. Population validation follows each intermediate stage,
so the Box-Cox rejection contract applies after earlier transformations.

The composed domain follows the pinned dependency's transformed-domain
intersection, then inverse traversal. Guide selection trims finite transformed
limits to that domain before generating candidates. Composed default breaks
inherit the first transform's callback, including its fixed default count; the
reference composed callback does not forward an authored count. Existing guide
and logarithmic break owners retain responsibility for selection and formatting.

Composed descriptors require plot v54, standalone v9, and direct numeric v3;
ordinary built-ins retain v53/v8/v2 and legacy families retain their earlier
versions. This serializes data only, with no host callback objects in core.
Registered transforms and additional distribution constructors remain separate
open contracts. The ledger records when the isolated draft is transferred.

## Registered pointwise transformations

`CustomTransformFactory` resolves an exact `TransformOperation` and bounded JSON
parameters into an owned `PointwiseTransform`. The registry captures its descriptor
at installation. The kernel supplies forward/inverse arithmetic, a captured source
domain, an inverse-branch declaration and optional population, default-break and
default-label behavior. Core retains immutable prepared ownership; Python, browser
objects and executable source never enter the descriptor. Trusted native callbacks
remain subject to the existing purity contract; storage bounds cannot preempt them.

`GgplotTransform::Registered` stores a boxed selection so built-in transform and
mapping enums retain their prior size. Compiler preparation resolves selections
before semantic training, including retained positional projections; standalone
copies, reconfiguration and option changes retain their registry. Direct native
authors resolve a descriptor through `ExtensionRegistry::resolve_transform` before
calling its arithmetic. Missing registrations fail validation. Inversion requires
a declared finite one-to-one branch. The existing transform engine continues to
own scale mapping, domain trimming, population handling and guide selection.

Transform-owned defaults yield to explicit guide settings. A composition inherits
its first transform's break function with the reference one-argument count policy;
its default formatter is the composition's ordinary numeric formatter. This slice
does not provide transform-owned minor callbacks or arbitrary vector-coupled R
functions. Their coverage remains explicit in the status ledger.

The minimum envelopes are primary/definition version 55, standalone version 10 and
direct numeric version 4 when a registered selection occurs, including nested
compositions and retained binned state. Existing built-in versions remain unchanged.
Portable serialization/loading checks exact installed identities and portable
capability; serialized JSON cannot install a factory. The optional external-style
proof crate reuses the workspace-pinned `libm` for deterministic cubic inversion.

Reference captures distinguish successful chart build/draw from later direct scale
queries. In particular, an empty chart can draw successfully while its custom
formatter rejects a direct empty label query. Qualification and publication
inspection are recorded in [the status ledger](../implementation-status.md).

### Transform-owned minor defaults — 14 September 2026

The captured pointwise kernel may supply a bounded minor-break vector from finite
transformed major values, expanded transformed limits and the subdivision count.
The existing minor resolver owns precedence, zero-range suppression, output limits,
viewport filtering, inverse metadata and projection. Explicit numeric or registered
minor policies and hidden minors bypass this default. Composition resets it to the
shared regular algorithm, matching the pinned dependency. Retain selected major
order before sorting drawable ticks; the callback receives semantic selection order.
The example installs a separate `example.scale_transform_minor` version 1 identity,
leaving the original example transform parameters unchanged. Existing registered
transform and retained-minor envelopes already represent this selection; an older
registry rejects the unknown example operation rather than silently approximating it.

Reference transform projections use trained constant limits before reference
expansion, with explicit legacy padding/nice options retaining their existing path.
Transformed minor candidates enter the retained projection directly, including the
registered explicit-minor route. This avoids applying forward arithmetic twice.
These decisions establish minor selection and metadata; minor tick/grid painting is
owned by guide implementation. Whole-vector-coupled forward/inverse arithmetic is
not represented by the pointwise protocol.

### Population-dependent transform arithmetic — 14 September 2026

`PreparedTransform` extends the installed transform protocol with ordered,
length-preserving forward/inverse batches and an explicit `is_pointwise` declaration.
`PointwiseTransform` remains a source-compatible alias. Existing implementations use
the scalar defaults; population-dependent implementations supply both batch methods.
Core checks output lengths and propagates callback errors even for empty batches.
Such transforms do not claim scalar inverse capabilities. Composition forwards the
same vector through each member, including domain and authored-limit endpoint pairs.
This supersedes the pointwise-only boundary above without introducing a second
registry or mapping engine. Length-changing/recycling R transforms remain outside
this protocol's current qualification.

Source transforms retain each layer/input vector, including missing/nonfinite entries,
before panel selection. Shared paint and numeric sampling reuse that ordering.
Generated positional aesthetics are encoded across panels before transformation,
then pass through the existing OOB, palette and position stages. Already transformed
statistical fields retain their value-space provenance and bypass a second forward
transform. Empty source inputs retain existing callback training semantics; panel
selection only removes training when it excludes a nonempty source population.

Trained mapped scales may retain transformed bounds because inverse endpoint
arithmetic cannot reconstruct a population-dependent training extent. This metadata
requires a registered reference normalization and primary/definition version 56;
authored selections without retained metadata keep version 55. Metadata validation
and old-version rejection are exercised by the external example tests. Qualification
is bounded by the recorded fixtures: generated counts and source paint/size facets
are covered; other generated style populations and callback combinations remain
open until independently checked. Host and publication results belong to the ledger.

Generated non-pointwise style populations retain panel and ordinal identities within
one layer batch. The shared sampler uses those identities to map colors and numeric
styles back to their encoded rows after transforming the combined generated vector.
This extends source-row batching without changing the serialized scale contract.
Count-to-paint and count-to-size fixtures qualify ordinary/fixed-facet and reverse
composition behavior; broader generated statistics and callback combinations remain open.

Probability transforms use the same explicit registered transform contract: a pure
quantile operation, its CDF inverse and probability domain `[0, 1]`. The external
example qualifies uniform/exponential pairs against pinned R captures alongside
built-in normal/logistic proofs. R function-name lookup and parameter forwarding
are adapted to a versioned typed operation; core does not own a general statistical
distribution library. Raw scale-query labels and visibility-filtered guide entries
remain distinct API contracts. Binned guide selection must match the reference
before training; hiding a guide can change the first mapping's limits.


Continuous colorbar selection uses the existing continuous break descriptor and
shared transformed-value mapping pipeline. Definition version 57 explicitly retains
this selection; older definitions reject it. A color legend may retain 300 decoration
samples in transformed scale space, matching the pinned GuideColourbar default.
Samples use the same OOB, rescaler and palette operations as marks, without applying
the forward transform twice. Empty, NULL or entirely invisible keys suppress the
ramp; numeric non-color aesthetics suppress colorbar guides. Empty decoration data
is omitted from serialization to preserve existing ordinary legend metadata.

This scale-side contract retains data for GG-05. Current publication layout still
paints ordinary swatches; colorbar layout, key positioning and configurable guide
presentation remain unqualified. The selection fixture and host proof therefore
certify callback batches and retained samples, not rendered ggplot2 colorbar parity.


Continuous interval selections retain `ContinuousBins` and `ContinuousSteps` with
the existing break/label descriptor at definition version 58. Their keys use the
same interval parsing, endpoint handling, midpoint sampling and labels as binned
scale guides. Selecting these guides does not bin the continuous mark mapping.
Automatic/function breaks mask scale endpoints before interval key mapping;
authored numeric endpoints remain available. Non-color aesthetics suppress steps.
The retained key values remain source-space values; interval positioning, endpoint
visibility and decoration presentation belong to GG-05. The primary selection
capture qualifies automatic, NULL, empty and uneven authored breaks on linear
scales. Additional log10 and cardinality/center vector-transform captures qualify
trained-limit use and inverse label batches. Midpoints and parsed keys enter the
shared transformed-value pipeline directly, including built-in palettes.

Reference palette pooling distinguishes source NA from arithmetic NaN. An internal
quiet-NaN payload carries missing provenance through numeric mapping; numeric wire
transport remains canonical, and source nullable fields recreate that provenance
on replay. Pool keys merge signed zeros but retain those two missing classes.
An index-based palette makes the distinction visible in marks and host checks.
This does not extend the wire vocabulary to encode R-specific NA values in arbitrary
callback parameters. Registered label combinations and other guide controls still
require their own reference evidence.


Continuous interval labels discard censored break boundaries before inserting
limit labels. A constant continuous bin still evaluates the registered break and
endpoint label batches and validates their lengths, then removes the key whose
ordinal position is undefined. Legacy binned constant-key handling stays separate
within the existing shared guide helper. The 1,080-case interval-label capture
retains raw parsed boundaries independently from the visible ordinal keys; key
comparisons use the visible prefix without altering captured reference output.


Temporal colorbar selection retains `TemporalColorbar(GgplotTemporalGuide)` at
definition version 59. It shares Date/datetime candidate generation, calendar
formatting and registered labels with temporal point guides, and uses the existing
300-sample colorbar pipeline for decoration. Non-color aesthetics suppress this
selection. The typed constructor adaptation selects it for Date/datetime color
defaults or explicit colorbars; explicit legends retain `Temporal`. Ramp values
retain the normalizer's absolute Date days/POSIX seconds, while temporal key values
remain offsets in the declared source unit. Full colorbar presentation is GG-05.

Temporal interval selection retains `TemporalBins` and `TemporalSteps` at definition
version 60. These share the existing interval key and midpoint pipeline and the
calendar break/label owner. Date/POSIX break vectors mask endpoints for both
automatic and explicit candidates; constant bins with no surviving cuts reject.
Keys retain timestamp offsets, while interval palette batches convert to absolute
Date days/POSIX seconds exactly once. Interior automatic labels retain calendar
pretty labels; separately generated endpoints use the temporal default formatter.
The 648-draw fixture covers UTC Date/datetime, three aesthetics, four units,
fixed/inferred limits and ordinary/constant/empty inputs. It does not certify
interval-specific callback or DST combinations, or GG-05 guide presentation.

Temporal interval label callbacks preserve the common calendar selector's break
names through masking/censoring and carry the existing timestamp context. Explicit
`date_labels` has precedence over registered label functions, including separately
formatted interval endpoints. The 2,160-case capture checks five callback result
modes, automatic/explicit/empty breaks, format precedence and endpoint calls in
four units. This extends UTC interval label evidence without changing timezone
resource ownership or claiming all interval DST/callback compositions.
