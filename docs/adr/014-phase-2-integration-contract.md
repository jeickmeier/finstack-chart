# ADR-014 — Phase 2 profiles, references and coverage ownership

Date: 8 September 2026. Status: Accepted integration direction for P2-00.
Requirements: GG2-01/12, ARC-03/04, BND-01, QLT-02/05.
Baseline: `fab2505951061eaafe9adb52c86b248ee0dfa6bf` (primary authoring implementation).
This decision introduces no runtime API, dependency or wire-version change.

## One definition carries execution semantics

New compatibility policies lower into the canonical `ChartDefinition`, including
their immutable profile/version provenance. Do not introduce a separate mutable
profile on Chart, Session, a worker or Output. The current primary envelope's
`LibraryV1` field remains unchanged until GG-02 implements migration. Today it has
only one supported value; passing definitions through runtime/export is sufficient
for that baseline, not for a future profile selected only on Plot.

GG-02 owns the common profile contract. It resolves default grouping, stage order,
bin closure, aesthetic units and theme policies into explicit normalized semantics.
Core evaluates those semantics; hosts cannot select different defaults. The profile
identity records provenance and participates in definition equality/revision and
cache invalidation even when a particular plot resolves to equal numeric output.
Layout/device profiles remain distinct destination inputs.

D3 lane owners add explicit versioned descriptors for standalone algorithms and
their component consumers. Selecting a D3 descriptor does not globally change
unrelated chart/stat defaults. GG adapters reuse those kernels with explicit
reference policies. Libraries and version numbers alone never select behavior.
The existing LibraryV1 definition path preserves all current defaults.

Every effective policy edit uses the existing definition revision fence and current
runtime source. Old prepared scenes and Presented captures retain their original
definition/resources; Current uses the latest committed definition. Definition,
resource and layout changes invalidate their existing dependent caches. No profile
change may reuse an incompatible worker result or mutate a retained request.

## Migration and resources

The integrator owns the common migration; the first package needing new semantic
wire fields implements it with tests. Keep existing version-1 readers and defaults.
Write a new envelope version for new required constructs or changed interpretation;
never emit new semantics disguised as a legacy payload. Normalize legacy inputs
through an explicit conversion. Primary and portable envelopes have separate version
histories; no package independently increments both or redefines existing fields.
Downgrade succeeds only when the definition is exactly representable, otherwise it
returns a structured unsupported-capability diagnostic. Record the concrete version
and migration in the delivering package, not speculatively in this entry package.

Existing explicit resource descriptors/registries remain the boundary for fonts,
locale/calendar/CRS data and registered operations. The owning package adds version,
content digest, license/source provenance and validated limits where needed. Missing
or mismatched resources reject before evaluation. Core performs no resource lookup
or downloads. Captures retain the exact immutable resources and registrations used.

WP-AX01 owns guide/scale identity separation: preserve ScaleId-based layer bindings,
primary x/y identities and navigation targets while adding distinct guide identities.
Migrate AxisBuilder/AxisHandle, names, serialization and host navigation together.
The guide-to-scale relation must be explicit; guide IDs never become data scale IDs.
SP-01 supplies the shared scale capability contract. This work does not wait for
final G-AUTH, G-GGPLOT or G-PARITY certification.

## Shared reference workspace and fixture convention

Use one development-only Node workspace at `tools/reference/node/`. Its package.json
declares all eight plan-pinned D3 modules; package-lock.json pins transitive sources
and integrity. A runtime manifest pins Node and any browser oracle/runtime used.
The first D3 entry package creates and verifies the lock; subsequent lanes extend
that same workspace. No npm dependency enters a Rust production package.

GG-00 owns `tools/reference/r/`, with an exact R/runtime manifest, renv.lock and
source/dependency checksums for ggplot2 4.0.3 and its required capability dependencies.
It records platform/native-library constraints separately from R package versions.
Neither reference workspace or lock is claimed to exist or execute in P2-00.

Store checked outputs under `fixtures/parity/<reference>/`. Each corpus manifest
records reference release/source digest, dependency-lock digest, generator revision,
input/output digests, licenses, platform, locale, timezone, RNG and supplied fonts or
other resources. Every case carries a stable reference-item ID, operation/options,
input, expected output or diagnostic, requirement/package/FIX IDs and the comparison
rule. Encode large integers, non-finite values and missing values explicitly, using
the existing portable conventions where applicable. Do not encode process addresses
or incidental object names as semantic identities.

Compare exact topology/order/categories/IDs/strings first; numeric tolerances are
operation-specific with units and rationale. Keep numeric/semantic comparisons
separate from inspected SVG/PDF/PNG/native artifacts. Regenerate into a temporary
directory and compare digests before accepting changes. Rust tests consume committed
fixtures offline; R/Node generators never run as a prerequisite for ordinary cargo
tests. Missing oracle/runtime evidence leaves its row open.

## Coverage and delivery ownership

The [coverage register](../phase-2-coverage.md) links existing inventories to AP-00
rows, semantic owners, fixtures and required surfaces. Lane entry packages expand
their source inventories to method/argument/default rows; GG-00 owns that expansion
for ggplot2. P2-00 does not claim those exhaustive oracle inventories are finished.
Every new reference item must acquire an existing or new semantic owner before its
lane can close. One item can have several consumers but only one kernel owner.

AP-00–08's delivered builders/runtime/hosts are reused. Each semantic package owns
its Rust public operations, shared host dispatch, actual Python/WASM exports and
declarations, primary usage, fixture comparison and destination evidence. GG-18
broadens combinations; it does not postpone first-use binding evidence. A future
standalone utility remains directly callable as well as usable by plot components.

The integrator checks cross-lane edges from the combined plan, especially CLR-04 →
SP-04 and the absence of backward certification dependencies. AP-09's remaining
performance qualification stays with its owner. Pure contracts/kernels and the
existing legend acceptance can proceed on the committed baseline independently.
Completed P2-00 certifies this handoff only; no feature or cumulative gate closes.

## GG-02 implementation of the migration

The continuation implements the common contract with optional canonical
`ExecutionSemantics` and capability-checked definition version 3. The primary profile
must match that object; LibraryV1 retains absent semantics and legacy wire defaults.
Layer/shared-transform source grammar preserves grouping and positional inputs.
Current and Presented capture continue using the existing immutable definition owners.
Shared statistic consumers with incompatible positional scale contexts explicitly reject.
The source-expression population is the registered dataset before chart filters/facets;
computed-expression population is the prepared layer. Post-scale outputs use one
pre-modifier snapshot. Independent aesthetic units and remaining statistic defaults
continue in their owning GG packages; this stage implementation does not certify them.

## GG-03 independent styles and version 16

Independent paints reuse `ColorEncoding` and the existing color scale engine; numeric
and typed text/linetype channels reuse `NumericEncoding` and `MappedScale`. Their
trained descriptors remain attached to prepared layers. Constants suppress the
corresponding active mapping and grouping input. In the ggplot profile, an explicit
linewidth suppresses mapped linewidth before training; post-scale expressions still
read a single resolved snapshot. Solid ggplot line segments use each start row's
style after ordering. Variable styles on non-solid grouped lines reject, as in the
pinned reference.

Style fields for fill, stroke, alpha, units and line type are optional and omitted in
legacy wires. Their presence, reference glyphs or the new channel descriptors requires
version 16. The resolved byte-color style grows from 24 to 56 bytes on the checked
64-bit targets; this is an explicit cost of the independent channels, while colors
remain four bytes. No constant-memory or unchanged-throughput claim is made for this
extension. The shared path/scene renderer owns filled outlines and dash hit geometry;
no host duplicates symbol construction or unit conversion.

`SymbolKind::Ggplot(0..=25)` extends the WP-S05 symbol kernel with R glyph topology.
Coordinates are qualified against numerical paths extracted from the pinned R PDF
device at four sizes. `AreaSize` is equivalent-circle area; legacy point `Size` is
radius. Explicit millimeters and points convert at the destination boundary using
96 logical pixels or 72 publication points per inch. Reference default size scales,
zero policies and palette adaptation belong to GG-04; text-channel retention does not
claim completion of GG-08 text geometry. LibraryV1 keeps its existing run-style rules.

## GG-04 scale-stage and time-guide ownership

Version 17 carries reference scale-stage policies, Date/calendar-width controls and
secondary time interpretation. Timestamp limits compare/clamp exact integers before
origin-relative floating conversion; generated statistics retain their timestamp
representation. Keep rejects unrepresentable retained values, with a documented
source-validation pass. Date uses the shared timestamp engine with day-based expansion
and calendar selection; it does not introduce a second date data representation.

Secondary Date/datetime guides reuse a translated copy of the primary resolved time
mapping and the shared calendar selector. The existing affine authoring entry accepts
factor one and exact-source-unit additive offsets (seconds for datetime, days for
Date). Numeric secondary guides retain their separate sampled reference inverse;
time guides do not use that numeric approximation. Secondary guides cannot bind
coordinates, become navigation targets or supply independent coordinate inversion.

Reference point sizing interprets default/mapped size and primary `.size()` in
millimeters unless explicit units override it. Source/inspection style values remain
selected aesthetic values; the common glyph kernel receives R's combined size and
outline dimension. Nonpositive effective dimensions create empty geometry without
invalidating the observation. Explicit `.radius()` and canonical `AreaSize` keep their
literal-radius/equivalent-area contracts. The reference PDF device's 0.01-big-point
zero-outline minimum is a portable physical hairline policy; qualification against
other R devices is a separate open boundary. This conversion does not alter standalone
shared symbol-area APIs or the LibraryV1 default.

Elapsed durations retain numeric seconds and use the existing linear mapping through
`AxisScale::Duration` (wire v17). They do not reinterpret seconds as calendar timestamps.
The existing extended-break search owns both numerical and duration preferred-step
lists. Default hms labels share fractional precision, hour width and right alignment
across the vector; formatting rejects microsecond magnitudes beyond the exact f64
integer range. Fixed-second duration widths use a bounded numeric lattice. The R oracle
pins hms 1.1.4 as a development-only dependency and retains exact decimal strings for
binary input values whose residues affect reference label precision. Named duration
widths are fixed elapsed lengths, including 31-day months and 365-day years. Explicit
shared time formatters interpret durations against the UTC epoch for labels only;
full R format-string equivalence is a separate acceptance boundary. Secondary duration
guides inherit duration selection and formatting over the existing numeric secondary
mapping, including non-unit and negative affine factors.

R named-color facts and parsing live in the existing shared color owner. The R adapter
shares byte-level hexadecimal decoding with authored CSS paint but preserves R-specific
names, special transparent spellings and hidden RGB channels. Numeric string indexes
use a pinned R 4.6.1 default palette or an explicit caller palette, with no graphics
process state. The portable index domain is positive signed 32-bit integers after
truncation; overflow rejects rather than adopting platform-specific R conversion.
Reference discrete/manual scale text is parsed once during color preparation, leaving
per-mark paint lookup pooled and the CSS compatibility path unchanged.

Identity aesthetic descriptors separate raw mapping from guide population metadata.
Numeric identity uses the same raw transform arithmetic as the checked positional
transform, preserving exceptional outputs at the scale API; geometry still applies
its own eligibility rules. Discrete identity shares factor-domain ordering with
ordinary reference scales, but retains every observed value for paint preparation
regardless of guide limits. Default identity guides are hidden; guide domains never
censor raw outputs. Standalone samples outside the prepared paint population may
parse on demand, while eligible chart rows use pooled values. This does not change
the existing D3 numeric identity scale's NaN/unknown contract.

Reference nonpositional field reads retain IEEE values until scale evaluation. Alpha
retains its raw value through reference after-scale arithmetic and reductions, with
finite saturation deferred to paint lowering. R's paint conversion maps either
infinity to zero coverage and leaves original coverage for NA/NaN. The existing
checked expression evaluator remains the default; reference after-scale execution
selects the shared evaluator's infinite-number policy and R missing-power identities.
This does not qualify exceptional-number source-stage expressions or permit nonfinite
geometry dimensions. Source positions and legacy aesthetic inputs keep finite reads.

Discrete scale guide selection is retained as an optional boxed argument descriptor.
It consumes the same prepared domain and samples the same mapping as marks. Break
intersection/order and label selection are shared by ordinary and identity scales;
manual palettes reuse the existing value owner after reference break-name assignment.
Duplicate manual names match their first value; named label replacements match their
last value, as independently observed in R. No guide computation enters per-mark
mapping. The additional optional pointer makes `ColorScale<Color>` 256 bytes on the
current 64-bit target (previously 248). A localized `large_enum_variant` expectation
preserves the existing by-value mapped-scale authoring API; boxing that entire API
solely for this eight-byte change would impose a broader migration. Full guide
presentation and numeric/text formatting acceptance remain separate work.

The optional guide box now holds a shared `GgplotScaleGuide` discriminant for hidden,
discrete or continuous guide policies; this adds no further inline scale storage.
Numeric guide candidates preserve outside values through vector-wide formatting,
then expose visibility separately from their raw and transformed values. The default
reference selectors reuse the axis break/format owners. The raw nonlinear inverse
is crate-private and shares arithmetic with the unchanged checked inverse API.
Guide edits never replace the scale mapping; manual-break palette naming remains
an explicit reference constructor behavior. Merely selecting a ggplot2 color palette
on a D3 descriptor does not opt that descriptor into reference guide semantics.

Zero-row continuous aesthetic training retains one boolean in the existing boxed
GG-04 population policy. A fallback mapping extent cannot establish guide observations;
fully authored limits can establish a guide independently. The flag is serialized,
replaced on each eligible batch and consumed only by reference guide selection.
No per-mark storage or legacy D3 guide behavior changes.

Numeric identity guide training also retains whether it observed any rows, independently
of its optional finite extent. This preserves zero-row versus all-nonfinite populations
and prevents fallback mapping domains from generating empty identity guides. The
serialized flag is replaced per batch and does not affect raw identity outputs.

Nonempty all-nonfinite continuous training retains a second flag in the existing
boxed scale policy; identity training derives this state from its row flag and absent
finite extent. Guide selection preserves reference infinite censor bounds independently
of finite fallback mark normalization. Both paths share numeric candidate labeling and
resource checks; no mark data or interpolator is duplicated.

Binned scale labels share the numeric candidate formatter but retain break selection
in the existing binned policy; constant-domain continuous selection is not reused.
Prepared legends retain a bounded numeric candidate vector, shared with continuous
entry construction and available to later guide composition. This adds per-guide
metadata only. R transformation round-trips precede binned endpoint comparison, and
constant reverse training preserves the reference limit order. Color-step rendering
and limit-label composition remain GG-05 work.

Binned population flags preserve reference candidate semantics independently from the
finite fallback used by mark mapping. Empty partial-limit failures and all-nonfinite
candidate behavior are evaluated at guide preparation. Equal-cut arithmetic remains
shared between ordinary training and nonfinite guide selection; no second bin mapper
or palette sampler is introduced.

Reference width strings are normalized during guide selection, leaving authored text
and scale projection immutable. Integer time-lattice arithmetic is shared; only the
reference alignment/truncation policy differs. Date candidates floor their aligned day
anchor before censoring, and elapsed widths reuse a single unit conversion. The
optional string is in the existing v17 guide argument object; no per-mark fields or
host-local date arithmetic are introduced.

Reference transformed limits that become NaN use the trained endpoint, while the
original authored descriptor remains immutable. Structural numeric-policy validation
checks the descriptor without selecting breaks on an invented empty population;
actual training owns selection. The common extended-break implementation handles its
single-result degenerate sequence before rejecting count one for a nonconstant search.
No new scale engine or host-specific selection path is introduced.

Empty/nonfinite guide population checks share the transformed missing-endpoint filter.
Binned metadata and sampling eligibility remain distinct: the prepared mapping retains
one sampling-validity flag and rejects numeric/paint sampling when incomplete nonfinite
limits cannot define bins. This preserves standalone break metadata without publishing
colors from a fallback range that R cannot map. Fully specified finite transformed
limits continue to establish mapping independently of a finite observation population.

Binned cuts deduplicate in transformed data space before rescaling. Distinct cuts
that collapse after normalization invalidate sampling while retaining guide metadata;
this is represented within the prepared mapping, not a new serialized engine.
Named mapped-color builders select R's grey50 missing color when their descriptor
opts into reference semantics. A reference palette alone does not opt a descriptor
into those semantics, and explicit missing-color overrides remain authoritative.

Authored exceptional limits use one transformed-bound preparation path. Fully supplied
comparable endpoints sort in transformed space; partial endpoints retain their authored
positions when filled from the population. Ordered OOB operations preserve crossed
bounds. Compiled reference normalization can retain transformed bounds whose raw
inverse loses information, including empty and nonfinite populations; authored wire
limits and population flags reconstruct that state without a new serialized engine.
The reference normalizer and guides use the same logarithm base. Square-root inverse
values below zero are missing. Binned finite/nonfinite selection and continuous
finite/nonfinite selection each share their existing selector and labeling path.

Hidden binned guides preserve R's first-mapping order: mapping uses population limits
captured before automatic break extension. Visible guides prepare their breaks first
and map from the extended limits. This distinction is resolved in immutable training,
using the existing guide policy and prepared cuts; it does not introduce a mutable
palette cache. Empty hidden-guide populations skip selection/sampling while structural
count budgets remain enforced; invoking an unavailable sampler still fails explicitly.
Default colorbar construction and its failures remain GG-05's responsibility.

Binned desired counts are binary64 values in the existing Equal/Nice variants, matching
the continuous guide count representation. Equal selection rounds the requested
sequence length up; nice selection passes the original fractional target to the shared
selector. Integer wire inputs remain accepted. Nice counts below one are outside the
bounded portable contract because the reference search may not terminate.

Named color-scale definition validation cannot choose binned cuts from its placeholder
normalization domain. It validates the population policy, descriptor, normalizer,
range registrations and resource bounds; population preparation selects actual cuts.
This allows a fractional count that is valid for constant data while preserving the
reference rejection for a nonconstant trained domain. Actual preparation and sampling
continue to enforce the full compiled mapping contract.

## GG-04 positional binned scale lifecycle

Positional bins use the numeric population, transformation, break, label and
threshold owners in `chart-core::scales`. Their first mapping produces bin indices
for statistics. Reference reset retains the initial cuts as the trained range,
reapplies authored endpoints, and maps generated indices or fractional positions
back into intervals. Those two boundary sets can differ under partial reversed
limits; a single cached mapping cannot implement both stages. Prepared state is
immutable and captured once per shared axis population. Population replacement
rebuilds it; geometry and guides do not retrain it. Automatic panel labels format
only the selected panel candidates. Explicit user labels retain their associations.

Primary `scale_binned` and retained bin projections require envelope version 18,
including projections nested in mappings, statistics and filters without an axis.
Python and WASM delegate to the same core constructor. Finite panel and statistic
comparisons qualify the current slice. Infinite positional ranges retain Number-encoded endpoints in a separate resolved
range adapter using the shared ggplot rescaler. Undefined projections are omitted;
finite scene geometry and the ordinary Bounds contract remain unchanged. These
ranges explicitly expose no numeric inverse or generic pan/zoom. R's coordinate
stage produces NaN for the tested infinite point positions, so no extended scene
point type is needed. Free-facet populations and finite viewports over unbounded
bin geometry remain acceptance gaps.

## GG-04 minor guide selection

Optional `MinorBreaks` authoring uses envelope version 19 and the shared guide
configuration for Rust, Python and WASM. It supports automatic subdivision, hidden
minors, explicit numeric or nullable timestamp candidates and reference date/time
width strings. Discrete numeric minors use the existing category-spacing owner;
automatic discrete minors are empty. Minor
snapshots retain positions independently of major labels and do not retrain scales.
A minor's raw `ScaleValue` is optional: transformed square-root expansion can have
no real inverse, and a midpoint can fall below nanosecond timestamp resolution.
Neither case permits dropping its finite drawable position or inventing a raw value.

Temporal subdivision uses origin-relative coordinates from `TimeAxisScale`.
Representable fractional timestamps promote to a finer integer unit; epoch values
never pass through binary64 to perform that promotion. Explicit Date major breaks in the ggplot2 profile
follow `ScaleContinuousDate$get_breaks` and floor to UTC days before labels and minor
selection, with Euclidean flooring before epoch zero. Time-width minors reuse the
existing calendar/duration selector and authored resource budget. This is selection
metadata acceptance; minor painting, animation and complete guide composition remain
GG-05 work. The completed selection slice passes 514 core tests/doctests and 761 actual cases
per host, including precision and rejection boundaries. Complete package, native
presentation and cumulative parity acceptance remain separate gates.

## GG-04 discrete continuous limits

Optional `AxisSpec.continuous_limits` uses envelope version 20. Canonical values
are Number vectors; host booleans normalize to zero/one before shared authoring.
Finite vectors reduce to their range, missing endpoints require a two-element
vector, and empty/infinite vectors preserve the reference exceptional limits.
Category-index expansion belongs to the shared spacing kernel and combines the
configured range with actual observed positions. No observations and observations
entirely excluded by the category domain have different reference ranges.

Spacing retains finite Bounds or an exceptional Number range, using the common
reference rescaler. Undefined projections produce no point; finite scene geometry
and Bounds contracts remain unchanged. Unbounded spacing disables category lookup
and inversion. Minor censoring shares this range; major label associations remain
stable even where destination ordering differs. Replacement rebuilds immutable
prepared state. The slice passes 756 R records, 3,552 cases per actual host and all
518 core tests/doctests. Coincident label layout and NoData presentation remain
GG-05 work; complete scale and destination acceptance remain open.

Automatic band/point character domains in the ggplot2 profile use the pinned C
collation; explicit domains retain authored order. Sorting occurs in resolved scale
options, leaving source ordinals and other profiles intact. Six reference ordering
records extend the actual host proof to 3,582 matching cases. This does not add
locale-dependent sorting or conflate missing categories with the text `NA`.

## GG-04 nullable discrete populations and unpainted positions

Shared discrete domain training preserves explicit missing-level positions and factor
ordering independently of palette cardinality. Missing levels do not consume palette
colors. Automatic missing translation controls trained levels; explicit limits retain
an authored missing slot even when translation is disabled. The typed Null key remains
distinct from the text `NA`. Zero source rows do not train factor levels.

Discrete policies retain whether their latest eligible population had zero rows.
This distinguishes the reference's untrained fallback from a nonempty population that
trained an empty palette. The empty-palette mapping returns `na.value` before missing
translation; otherwise disabled translation suppresses unmatched paint. False state is
omitted from serialized policies. Explicit retained true state requires envelope 21,
using the common mapped-scale traversal for every aesthetic owner; ordinary policies
retain version 17. Both primary and definition envelopes reject downgrade to version 20.

Position training precedes missing-paint removal in the ggplot2 profile. The compiler
retains finite numeric contributions and checked categorical source ordinals before
clearing nonpainted point/symbol/line/rule coordinates, including rule endpoints.
Categorical ordinals remain tied to the original immutable layer catalog. An optional
boxed pair of ordinal sets adds one pointer per prepared layer (eight bytes here) and
allocates only for suppressed category positions. These contributions affect scale
training without creating drawable or inspectable marks. Replacement recomputes them;
retained frames remain immutable. The slice passes 512 direct scale records, 72 R
position panels, 346 cases per actual host and 523 core tests/doctests. It does not close
identity primary paint, positional missing-category or complete guide presentation gates.

### Nullable identity colours

A discrete identity colour retains the distinction between a raw missing key and the
literal R colour `NA`. Both lower to transparent paint, but only the missing key enters
the existing missing-aesthetic removal stage. This classification ignores identity guide
and `na.translate` controls, which do not alter its raw mapping. Position domains retain
omitted rows through the existing shared training path. No new wire state or host-side
mapping engine is required. FIX-GG04 evidence covers 512 valid-colour R configurations,
actual point-grob counts, primary Rust JSON and actual Python/WASM replacement checks;
other missing-paint geometry contracts remain governed by their acceptance evidence.

### Untrained discrete lookup defaults

The retained empty-population flag distinguishes reference fallback lookup limits from
an empty trained palette. The prepared descriptor and guide domain stay empty; actual
queries lazily request the ordinary palette's two fallback colours. This preserves empty
chart preparation for short manual palettes, whose insufficient-value error belongs to
lookup. Named manual palettes retain their implicit limits behavior. Numeric/logical
queries can fall back to reference text-level matching only under the ggplot discrete
policy, after direct typed lookup. No new serialized state or per-row palette work is
introduced for trained populations. FIX-GG04 qualifies colour queries and empty charts
against 64 direct R records and 32 primary cases, with full-core and actual host regression
evidence in the scale ledger.

### Typed positional missing categories

GG-04 uses the existing `ScaleKey::Null` identity for its positional policy and a
nullable layer catalog for source ordinals. `ScaleValue::MissingCategory` survives
projection and guide/inspection transport without colliding with literal `NA` text.
The policy shares discrete training/guide owners; an internal checked positional
provider shares band/point spacing, centering and destination validation. The adapter
contains no second numerical scale implementation. Ordinary string catalogs remain
available, and explicit D3 band/point projection retains null omission.

Authored factor/drop/translation, nullable limits and break policies use definition
wire version 22; older envelopes reject retained new fields or authored missing-category
values. Python/WASM call the same Rust builder. Acceptance is recorded in the GG-04
scale ledger; new core APIs or passing direct fixtures alone do not close the package.

Nullable positional continuous limits use the same category-index expansion kernel as
ordinary categorical axes. They do not retrain keys or require a second policy field:
the existing axis continuous-limit authoring feeds the typed policy's training method.

Explicit numeric minors on typed categorical providers use an optional provider
capability backed by the same spacing kernel. The checked boundary requires finite
positions within the destination range; minor candidates do not alter category training.

Discrete secondary axes retain their source axis and reuse its reference spacing.
A provider may expose its category-index viewport for duplicate-guide selection;
missing capability rejects, and finite viewport validation stays in the layout owner.
Duplicate guides expose numeric values, while primary category/null identity remains
unchanged. Only literal identity transforms are allowed. Candidate selection reuses
primary guide policies, formatting uses the existing formatter, and normalized guide
rounding is independent of mark mapping. This adds no serialized fields or host engine.

Materialized positional palettes use the existing nullable positional policy and
shared numeric spacing/expansion owners. Their vector is fixed in trained-domain
order, must cover that domain and requires wire version 23; it is not an R callback.
A prepared provider exposes the raw category coordinate for duplicate guides, avoiding
ordinal reconstruction when positions repeat or differ from 1..n. A missing discrete
range can leave an observed continuous range; empty/unresolved and secondary-degenerate
ranges reject according to the pinned reference. Explicit numeric duplicate labels
use the existing numeric guide formatter.

Automatic nonpositional timestamp scales retain an integer origin, source unit and
explicit calendar in `GgplotScaleGuide::Temporal` (wire 24). Candidate selection reuses
the datetime selector with censoring deferred to the aesthetic guide; positional
selectors keep their existing viewport censoring. Timestamp normalization now separately retains origin/unit metadata in
`NormalizationSpec::Ggplot` (chart/plot wire 25, standalone scale wire 4). It projects
relative ticks to absolute seconds only for reference palette and OOB arithmetic,
including the reference constant-range tolerance and double rounding. Source identity
and domains remain relative/exact; removing a guide cannot change palette values.
This context supports linear range rescaling; other normalization policies reject it. Inferred guides use UTC, independently of source timezone
metadata; explicit local rules are resources, not implicit system dependencies.
Authored numeric scale guide policies are preserved. Date-specific argument coverage
and guide presentation remain separate open requirements in the scale evidence.

Temporal aesthetic selection now has explicit Date/datetime arguments, with null
breaks distinct from an empty vector. The existing pretty selector can return padded
Date candidates without changing positional censoring; width guides reuse calendar
floors/progression and integer lattices with fullseq enclosure endpoints. Guide labels
still use the common continuous label/visibility contract and explicit R formatter.
Palette normalization remains independent of guide setters: Date normalization uses
absolute days, datetime normalization uses absolute seconds, and both retain exact
source origins/units. Non-default guide arguments or Date arithmetic require chart/
plot wire 26; standalone Date normalization requires wire 5. Absent new controls retain
the previous default payload/version. Optional format/locale storage is boxed.

Binned aesthetic count palettes retain an optional `GgplotBinnedPalette` in the
existing policy (chart/plot wire 27). The common bin classifier selects the interval
count, then calls the existing discrete shape/linetype/Brewer generator. Missing
palette outputs use the same authored unknown value as missing observations.
An identity range function avoids competing palette sources; positional bin policies
reject this aesthetic-only field. Named manual palettes keep their category matching
semantics. Value-channel compilation validates prepared sampling even for an empty
layer, matching numeric and color compilation. This does not add guide presentation.

Reference linewidth conversion belongs to shared scene projection, alongside the
existing point conversion. For the existing nonpoint reference geometry families,
an absent explicit aesthetic unit selects the R graphics linewidth unit and zero
selects its physical PDF hairline. An explicit unit remains authoritative. Prepared
numeric styles stay in aesthetic units, so palette and after-scale semantics are not
changed by the destination. Builtin validation admits zero reference widths before
projection; native/export/hit geometry consume the same positive final stroke.

Registered discrete limit functions are installed native implementations selected by
an exact operation identity and bounded JSON parameters. Core passes the common
trained key domain after factor/drop/NA handling and consumes the returned vector;
it does not evaluate source code from payloads. NULL remains distinct from an empty
vector. Callback results replace authored fixed limits and are recomputed for every
new eligible population. Chart/plot wire 28 preserves the selection, with registry
ownership retained across edits, updates and held publication snapshots. Missing
registrations, unsupported versions and native-only portable requests reject through
the shared extension validation path. The external proof implementation owns its
function body; core retains mapping and guide algorithms. Numeric limit functions
and other scale function arguments remain separate unfinished GG-04 requirements.

Numeric limit functions share the pure typed-key callback registry with discrete
limits (`CustomScaleLimits`, `ScaleLimitsInput`, `ScaleLimitsOperation`). The
`limits_function` selection owns no executable payload. The numeric consumer
retains raw result arity and missing values separately from fixed-limit fallback;
replacement training discards materialized output. A singleton may map successfully
while its binned guide candidates are missing. Numeric identity output remains
independent of guide validity. Temporal callback domains remain explicitly unsupported
until their typed epoch/unit contract is implemented.


Temporal limit callbacks use the same registry and origin-relative numeric values.
`ScaleLimitsInput.temporal` supplies the existing exact origin/unit descriptor and
Date-versus-datetime semantics. Function output uses the same offset representation;
no absolute epoch is rounded through a floating-point callback value. Nonfinite
trained endpoints remain representable. The temporal transform rejects an untrained
NULL domain before invoking the function, including functions with fixed results,
as the pinned reference does. Temporal function guides reuse the existing calendar
owner after raw result arity validation. The implementation remains pure native
registration selected by the portable operation identity; no host interpreter enters
core. Positional limits and other callback arguments remain separate open work.
