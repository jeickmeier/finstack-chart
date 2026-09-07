# D3 scale feature parity

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Assignment: assess current plans/code and plan parity;
documentation only. Baseline: `1cb9557`, with pre-existing action/state/host edits.
Requirements: SCL-01–08, DAT-02/05, BND-01/03/04, QLT-02. Acceptance: FIX-20,
with FIX-06/07/08/10/13/15/16 regression coverage.

## Outcome and scope

**The implementation and the original 0.1.0 plan do not have D3 scale feature parity.**
WP-06/11 delivered their narrower contracts. This plan adds a required production
workstream, SP-01–07, without reopening those historical completion reports or
implementing scale behavior in this planning task. G-SCALE and G4 cannot pass until this workstream
passes. G2's existing acceptance describes the 0.1.0 alpha only.

Reference target: [D3 scale documentation](https://d3js.org/d3-scale), retrieved
7 September 2026, and the [d3-scale 4.0.2 export surface](https://github.com/d3/d3-scale/blob/v4.0.2/src/index.js).
The [package metadata](https://github.com/d3/d3-scale/blob/v4.0.2/package.json) identifies
the standalone module version and its dependencies. D3's website version, 7.9.0, is
the umbrella library version. SP-01 must lock the oracle's transitive packages,
source commit, integrity hashes, license and timezone/locale inputs before generating
acceptance fixtures. No D3 package is added to production dependencies.

Parity means equivalent scale capabilities and observable results on equivalent
typed inputs. It covers standalone scales and their applicable chart aesthetics,
guides, interaction capabilities and portable descriptors. It does not require JS
method spelling, coercion, object identity, a DOM or a browser renderer. The complete
d3-scale-chromatic catalog is separately required by CHR-01–06 and the
[chromatic plan](d3-scale-chromatic-parity-plan.md); CP-01–05 own its delivery.
Custom interpolation must remain possible;
limiting the feature set to the current byte-color palette is insufficient.

Required D3 operations may not disappear under a blanket "Rust is different" exception.
SP-01 records a method-level disposition: equivalent implementation, typed adaptation,
or genuinely undefined/non-finite reference behavior with a diagnostic. Unsupported
defined behavior remains a gap. Mutable D3 authoring is represented by builders or
new immutable descriptors; prepared scenes remain immutable. Matching defaults must
be available through D3-compatible constructors/descriptors, even where recipes retain
their current explicit defaults.

## Current implementation versus target

Verdicts concern **full family parity**, not whether the current implementation meets
its original specification. **Fail** means a source-confirmed missing capability or
different contract. **Uncertain** means matching evidence is absent. No complete family
receives Pass from the existing tests alone.

| D3 surface | Current evidence and gap | Verdict | Delivery |
| --- | --- | --- | --- |
| Linear | [LinearScale](../../crates/chart-core/src/scales/linear.rs) maps/inverts two numeric endpoints. Missing arbitrary knots, output interpolation/rounding and unknown outputs. Clamp applies only forward; tick selection differs. | Fail | SP-02/05 |
| Pow, sqrt | [ScaleTransform](../../crates/chart-core/src/scales/nonlinear.rs) has only log/symlog. No exponent family or sqrt convenience. | Fail | SP-02 |
| Identity, radial | No implementations in the [scale module](../../crates/chart-core/src/scales/mod.rs). Identity behavior is not an ordinary range remap; radial interpolates squared radii and needs its own output transform. | Fail | SP-02 |
| Log | Positive-only, base > 1; ticks/nice use transformed linear grid. Missing negative-domain support and D3 log-specific tick/label behavior. | Fail | SP-02/05 |
| Symlog | Matching forward/inverse formula for positive threshold; ticks/nice are in transformed space rather than D3's data-space linear tick contract. Full D3 parameter/edge coverage unproved. | Fail | SP-02/05 |
| UTC and local time | [UtcScale](../../crates/chart-core/src/scales/utc.rs) preserves integer origins and calendar steps. No local calendar engine, time nice, D3 interval selection or default multi-resolution formatting; current weeks start Monday. | Fail | SP-06 |
| Band | [BandScale](../../crates/chart-core/src/scales/band.rs) has centers/oriented extents and padding. Missing align, rounding, explicit step/bandwidth API and zero-width bands; singleton leftover-space behavior differs. | Fail | SP-03 |
| Point | [PointScale](../../crates/chart-core/src/scales/color.rs) has centers/lookup and padding. Missing align, rounding, step/zero-bandwidth API; default padding differs. | Fail | SP-03 |
| Ordinal and scaleImplicit | [ColorScale::Discrete](../../crates/chart-core/src/scales/color.rs) supplies string-category palette cycling, retained training and missing color. No general typed range or authoring-time implicit-domain operation. | Fail | SP-03 |
| Sequential, SequentialLog/Pow/Sqrt/Symlog | Current continuous color offers only two-endpoint normalization and equally spaced palette stops. No general interpolator-based family/variants. | Fail | SP-04 |
| SequentialQuantile | Summary quantiles exist, but no sample-rank scale, range sampling or quantiles(n) scale API. | Fail | SP-04 |
| Diverging, DivergingLog/Pow/Sqrt/Symlog | No independently authored three-point domain/midpoint with transform-aware interpolation. Three palette colors do not supply that domain. | Fail | SP-04 |
| Quantile | No sample-trained discrete scale, breakpoints or invertExtent. A grouped quantile statistic does not supply scale semantics. | Fail | SP-04 |
| Quantize | No equal-width discrete mapping, thresholds or invertExtent. Histogram bins are upstream statistics, not a replacement. | Fail | SP-04 |
| Threshold | No authored ordered cutpoints, discrete range or unbounded inverse extents. | Fail | SP-04 |
| tickFormat and scale formatting | [NumberFormat](../../crates/chart-core/src/typography.rs) supports fixed/scientific/percent, explicit precision and two locales. Missing D3 specifier coverage, inferred precision/SI prefixes and log/time format contracts. | Fail | SP-05/06 |
| copy, getters and reconfiguration | Existing scales derive Clone and expose some getters. Independent callback/catalog configuration, every family and host round trips have no D3 comparison coverage. | Uncertain | SP-01–07 |
| End-to-end parity | [AxisScale/ResolvedScale](../../crates/chart-core/src/layout/types.rs), color/guide metadata and portable proofs cover the existing subset. No pinned d3-scale differential catalog. | Fail | SP-01/07 |

The inventory covers all 26 exported scale factories plus `scaleImplicit` and the
top-level `tickFormat` helper. The authoritative behavioral sources are
[linear/identity/radial](https://d3js.org/d3-scale/linear),
[power](https://d3js.org/d3-scale/pow), [log](https://d3js.org/d3-scale/log),
[symlog](https://d3js.org/d3-scale/symlog), [time](https://d3js.org/d3-scale/time),
[ordinal](https://d3js.org/d3-scale/ordinal), [band](https://d3js.org/d3-scale/band),
[point](https://d3js.org/d3-scale/point), [sequential](https://d3js.org/d3-scale/sequential),
[diverging](https://d3js.org/d3-scale/diverging), [quantile](https://d3js.org/d3-scale/quantile),
[quantize](https://d3js.org/d3-scale/quantize) and [threshold](https://d3js.org/d3-scale/threshold).
SP-01 inventories inherited methods from pinned source as well as documentation headings.

## Priority contract changes

These source-derived examples are discriminating proposed fixtures, not claims that
a D3 runtime comparison was executed in this review.

Baseline source locations: `scales/linear.rs:206` (inverse) and `:266` (tick step),
`scales/nonlinear.rs:21` (parameter validation) and `:229` (ticks),
`scales/band.rs:99` (centers), `scales/color.rs:8` (color families) and `:192`
(point options), `scales/utc.rs:127` (interval selection), `layout/types.rs:39`
(axis families), and `typography.rs:243` (formatting). All are under
`crates/chart-core/src/`; linked files are in the matrix above.

| Priority / IDs | Concrete difference and impact | Planned resolution |
| --- | --- | --- |
| P1 / SCL-06 | Piecewise domain [0,10,100], range [0,50,100] maps 10 to 50; two endpoints map it to 10. Unequal domain segments are currently inexpressible. | Validated ordered knot vectors and per-segment interpolation; preserve knots through viewport/navigation. |
| P1 / SCL-07 | Domain [0,10], range [0,100], clamp enabled: current invert(200) returns 20; D3 returns 10. Brushing/dragging can report different source limits. | Share clamping semantics in forward/inverse; retain an explicitly named unbounded inverse for navigation where needed. |
| P1 / SCL-03/06 | Domain [-100,-1] is valid for D3 log and rejected here. | Sign-consistent log branches, zero-crossing validation, valid negative baselines and host fixtures. |
| P1 / SCL-06 | Diverging [-10,0,100] must put 0 at interpolator parameter 0.5. Current continuous color puts it at 1/11. | Three authored domain values; each side normalized independently after its transform. |
| P2 / SCL-07 | For [0,1] and count 4, current numeric ticks are [0,0.5,1]; D3 produces [0,0.2,0.4,0.6,0.8,1]. Sharing a 1/2/5 vocabulary is not tick parity. | Implement the reference count/step selection and nice algorithm; separate candidate generation from layout thinning. |
| P2 / SCL-07 | A singleton band, range [0,100], inner padding 0.5, outer 0: current center is 25; D3's center alignment gives center 50 and width 50. | Account for leftover span before aligning; test singleton, empty, reversed and rounded bands. |
| P1 / SCL-04/06 | Elapsed UTC seconds cannot represent local day boundaries across a DST transition merely by relabeling ticks. | Supplied timezone rules for floor/ceil/offset and local ticks/nice; preserve exact timestamp projection. |

Other required compatibility changes: constant/repeated domains, constant ranges,
duplicate category deduplication, defaults, range-length rules, missing input policies,
explicit nice requests on authored domains, threshold boundary equality and empty
sample/range behavior. Keep the existing training and invalid-painter guarantees.
Defined finite reference results must be supported or remain visibly incomplete;
do not automatically reject all degenerate cases as invalid.

## Implementation shape and compatibility

1. Extend `chart-core::scales`; keep one mapping engine. Separate domain training,
   pure resolved mapping, tick/format generation and layout thinning. Reuse checked
   normalization, integer time origins and the existing quantile primitive where their
   contracts match. Do not add a scale crate or run D3 at render time.
2. Share transform/normalization helpers across positional, sequential and diverging
   variants. Radial is an output/radius transform, not a polar-coordinate feature.
   Keep ordinal and threshold outputs generic in native Rust. Portable descriptors
   cover numeric, color, text, boolean, timestamp and bounded structured values as
   appropriate; do not stringify category identity or ordered threshold inputs.
3. Consume the shared interpolation engine from the
   [interpolation parity plan](d3-interpolate-parity-plan.md), ITP-01–08. WP-IP02 owns
   scalar/rounded and composition kernels; WP-IP03 owns typed values; WP-IP04 owns color
   interpolation, hue/gamma and splines, consuming CLR-02/03 parsing/conversion. Scales own domain normalization, range
   configuration and integration. Custom factories use the common native/registered
   contract; no separate interpolation implementation lives in scales or bindings.
4. Expose family-specific capabilities: numerical inverse where meaningful, inverse
   extent for classifiers, category lookup, bandwidth/step, ticks and formatters.
   Sequential/diverging scales have no general numeric inverse. Rounded/degenerate
   mappings must not advertise exact round-trip guarantees.
5. Preserve recipe defaults through explicit options, not a second legacy engine.
   D3-compatible descriptors preserve supplied knots and constant-domain mapping;
   automatic chart training may still expand constants for a useful axis. Keep domain,
   viewport and range separate. Define explicit nice as an authored operation, distinct
   from automatic nice policy that never overrides explicit domains.
6. Version changed wire meaning under BND-01. SP-01 decides the migration before API
   edits: existing v1 inputs must retain meaning via explicit old policies or a documented
   migration, and unknown new variants must reject on older readers. Add new operations
   to the actual Python/WASM proof surface; serialization alone does not expose a usable
   standalone map/invert/ticks API. Resolve catalogs/interpolators once, not per mark.
7. Update cache keys and guide compatibility for knots, transform parameters, rounding,
   unknown/outside policy, output type, interpolator ID/version, locale and timezone
   resource revision. Color-only changes cannot retrain position. Quantile scales train
   from the declared eligible post-stat population and retrain after corrections/eviction;
   viewport zoom alone must not change their sample population.

## Work packages and acceptance

All stages own documentation and focused fixtures alongside the named implementation.
One integrator owns shared scale/axis/wire interfaces. This is a dependency plan, not
an instruction to launch agents. Implement SP-01 first; subsequent independent branches
may be scheduled by the owner. Do not move unrelated WP-15 work into this changeset.

### SP-01 — Freeze the compatibility contract and oracle

Prerequisite: WP-14. Requirements: SCL-06/07/08, ARC-04, BND-01, QLT-02.
Owns: scale contract/ADR-005 amendment, fixture manifest, a development-only Node
oracle generator and Rust fixture reader. No public unimplemented API scaffolding.

- Pin d3-scale 4.0.2 and transitive dependencies in an isolated fixture tool lockfile;
  record source/license provenance and a reproducible regeneration command.
- Enumerate constructors, inherited methods, defaults and parameter validity, including
  count 0/1, fractional counts, empty/constant/repeated inputs, base/exponent limits,
  non-finite results and range/domain length mismatches. Define typed unknown/unbounded
  results rather than JSON NaN/Infinity.
- Record every current difference as a tracked FIX-20 case and method disposition.
  Agree descriptor migration and timezone/interpolation resource boundaries in the ADR.
- Store independently checked inputs/results, operation tolerances and oracle provenance.
  Ordinary Rust tests consume committed JSON without Node or network access.

Exit: deterministic oracle regeneration, an executable comparator and a complete
expected-gap report. Existing differences stay open; expected failures cannot count as
feature acceptance. This stage establishes measurement, not parity.

### SP-02 — Continuous mapping and numeric families

Prerequisites: SP-01, WP-IP02. Requirements: SCL-01/02/03/06/07, DAT-05.
Owns: `scales/linear.rs`, `nonlinear.rs`, family helpers and numeric axis integration.

- Add piecewise numeric mapping/inversion, range rounding, unknown handling, matching
  clamp behavior, explicit nice operation plumbing and independent configuration copies.
- Add signed power/sqrt, identity, radial and negative-domain log. Cover finite base
  values between zero and one as well as above one where reference behavior is defined;
  classify exponent zero/negative and singular inverses per operation, not by broad bans.
- Provide representable constant-domain/range and repeated-knot behavior; validate
  ordering and capability limitations. Check overflow and near-zero precision separately
  from well-scaled D3 cases.

Exit: numeric FIX-20 mapping/inversion cases pass, including rounded ties, unequal knots,
descending orientation, extrapolation and copy isolation. Existing FIX-07 remains valid
under documented recipe policies. Public scales execute through named numeric axes.

### SP-03 — Ordinal, band and point completeness

Prerequisite: SP-01. Requirements: SCL-01/06/07, DAT-02.
Owns: categorical scale helpers, category authoring and corresponding axis descriptors.

- Add generic ordinal ranges and explicit unknown versus implicit-domain construction.
  Resolve implicit additions in deterministic authoring/training order, then freeze the
  catalog for scene use; inspection must never mutate it.
- Add alignment, rounding/rangeRound, step/bandwidth and all padding controls. Band
  starts and widths use D3 semantics; retain center/oriented-extent convenience APIs.
- Handle repeated categories, empty/singleton domains, zero widths, reversed ranges,
  large typed keys and default presets without changing source identity.

Exit: exact category/membership/order tests, independently calculated spacing fixtures,
and matching band/point output for all alignment/rounding combinations. FIX-06/07 and
category updates preserve expected identity, dodge slots and hit geometry.

### SP-04 — Interpolation, distribution and color families

Prerequisites: SP-02, SP-03, CLR-04, WP-IP03, WP-IP04. Requirements: SCL-03/06/07, GRA-03, BND-01.
Owns: color/classifier scales, mapped aesthetics and legend data; consumes shared interpolation.

Phase 2 makes CLR-04 explicit here because this package consumes its portable color
descriptors and shared paint lowering, in addition to CLR-03's kernels. See the
[combined dependency plan](phase-2-parity-implementation-plan.md#5-entry-package-and-coordinated-execution-waves).

- Integrate WP-IP03/04 typed interpolation and generic range values from the implementation shape.
  Consume WP-IP04's RGB basis/Cubehelix interpolation used by CP-03, backed by CLR-03's
  floating-point color conversion. Share CLR-04's descriptors and paint lowering;
  retain undefined-channel semantics until final paint quantization. Coordinate with
  CP-01/CLR-01 without depending on later chromatic integration or certification.
- Add sequential linear/log/pow/sqrt/symlog, sequential quantile, and diverging
  linear/log/pow/sqrt/symlog. Include range/rangeRound/interpolator configuration where
  supplied by the reference; sequential-quantile range sampling and quantiles(n) are
  distinct from sequential continuous methods.
- Add quantile, quantize and threshold using a shared ordered-breakpoint lookup where
  appropriate. Thresholds also accept ordered nonnumeric inputs. Preserve equality
  direction, tails, repeated cuts, unknown output and inverse extents, including duplicate
  range values and absent/unbounded endpoints.
- Extend guide metadata for real breakpoints, three-point centers and intervals;
  changing the palette must not change classifier boundaries or positional training.
  Apply numeric output scales to size/opacity/stroke-width as well as position where
  compatible, instead of implementing every new family only as color.

Exit: distribution/tie/tail, asymmetric diverging, custom interpolation, unknown and
legend fixtures match; fresh batch and corrected-data quantile training agree. Core
maps typed outputs without renderer-specific values or per-row validation overhead.

### SP-05 — Numeric ticks, nice and formatting

Prerequisites: SP-02, SP-04. Requirements: SCL-07, LAY-01/02, THM-03.
Owns: tick/format helpers, `typography.rs`, axis and guide preparation.

- Match numeric count/step/nice behavior and standalone tickFormat, with independent
  precision inference, specifier and explicit locale descriptors. Include SI, percentage,
  signed/grouped/scientific labels and the full referenced numeric specifier surface.
- Implement log major/minor ticks and label suppression; use data-space linear ticks/nice
  for pow/symlog and the appropriate radial/identity/quantize helpers.
- Keep raw candidate ticks and formatting observable separately from bounded layout
  collision thinning. Separate caller count hints from hard resource budgets.

Exit: exact tick arrays where representable and exact label strings against the pinned
oracle, including sign/zero/tiny/large domains and precision boundaries. Layout can thin
labels without silently changing the standalone scale result.

### SP-06 — UTC and explicit local calendar parity

Prerequisites: SP-02, SP-05. Requirements: SCL-04/06/07, DAT-05, BND-01.
Owns: time interval/format helpers, timezone resource contract and time axis descriptors.

- Add matching automatic interval selection, calendar-aware every(k), explicit ticks/nice
  and default conditional/custom time formatting. Cover milliseconds through years,
  two-day steps and Sunday-based default weeks; retain explicit Monday-week support.
- Provide local time using host-supplied versioned timezone rules, with no hidden system
  timezone or I/O. The host may resolve a machine default explicitly before preparing
  the chart. Floor/ceil/offset, DST folds/gaps and labels use the same resource.
- Keep integer-origin projection and all supported source units. Compare with D3 within
  its millisecond Date range; independently validate submillisecond and large-integer
  precision beyond that overlap. Session compression remains a separate feature.

Exit: UTC and local ticks/nice/invert match fixed-zone fixtures across leap/month/year
boundaries and both DST transitions. Actual Python/WASM use the same timezone data
revision; a formatter-only local-time adapter does not pass.

### SP-07 — Integration and parity release proof

Prerequisites: SP-03–06 and WP-16, WP-18, WP-20 for interaction/update/live-export proof.
Requirements: SCL-01–08, BND-01/03/04, SCN-04, QLT-02/03/04.
Owns: compiler/layout/scene capability integration, portable scale operation proofs,
native/export gallery, fixture report and support/API documentation.

- Exercise each applicable scale through named axes, mapped aesthetics, shared/free
  facets, secondary guides, legends and physical publication layout.
- Verify pointer-anchored zoom, brushing, inspection and linked domains against retained
  resolved scales; unsupported inverses return capability diagnostics. No duplicated
  interpolation formula in hit testing or host code.
- Execute standalone map/invert/invertExtent/ticks/format/reconfigure and chart fixtures
  in Rust, Python and single-threaded Node WASM. Test migration, unknown descriptors,
  large IDs/timestamps, disposed handles and independent copies.
- Compare append/upsert/remove/retention with fresh batch results, including ordinal
  training and exact quantile samples; inspect coherent exports during updates.
- Inspect actual native charts plus SVG/PDF/PNG for representative numeric, categorical,
  classifier, diverging and local-time guides. Measure piecewise lookup, category lookup,
  quantile retraining and allocation behavior; feed results into WP-22's declared budgets.

Exit (G-SCALE): every required FIX-20 method/case passes on its declared surface with no hidden
skips; typed adaptations and precision limits are documented. WP-21 rechecks the complete
fixture/platform matrix; WP-22 verifies performance; WP-23 documents the actual support.
No "D3 scale parity" claim until this evidence exists.

## Evidence protocol and sequencing

Each FIX-20 case records family/method, authored input, expected result/diagnostic,
oracle package/resource versions, Rust/Python/WASM disposition and requirement ID.
Use exact comparison for identities, unknowns, thresholds selected, rounded coordinates
and strings; justify arithmetic tolerances per operation. For well-scaled binary64
mapping start from 1e-12 relative/absolute expectations, but derive stricter exact or
ULP bounds where possible. Measure transformed round-trip conditioning; do not widen a
global tolerance to cover an unexplained algorithm difference. Scene tolerances remain
separate from scale-data tolerances. Millisecond fixtures compare integer times exactly;
submillisecond inversion uses independently justified source-tick bounds.

The [main plan](gpui-charts-implementation-plan.md) owns project traceability and gate
dependencies. The sequence is SP-01 -> SP-02/SP-03 -> SP-04 -> SP-05 -> SP-06 -> SP-07.
SP-02 also requires WP-IP02; SP-04 requires WP-IP03/04. Interpolation algorithms and
FIX-I01 belong to WP-IP; share its oracle lock. WP-IP07 consumes SP-07, so SP-07 must
not wait for final interpolation certification. Remove transferred interpolation work
from this lane when estimating the combined scope.
SP-04 additionally requires CLR-03 from the color plan; CLR-05 consumes SP-04 without
requiring SP-07. WP-21/22 require both color and scale acceptance.
WP-15–20 can continue under their existing assignment; SP-07 additionally needs their
consuming behavior. WP-21/22 must include SP-07, and WP-23 inherits those dependencies.
The concurrent axis plan owns FIX-19; this scale plan uses FIX-20. SP-03/05/06
supply band metrics and numeric/time tick/format APIs to WP-AX02. Axis argument
precedence, geometry, styling and transitions stay in WP-AX01–06. Share the
d3-scale oracle manifest and timezone/locale resources across both workstreams.
Re-estimate delivery after SP-01: the original 16–24 week budget excludes this added
scope, especially typed interpolation, formatting and local calendar services.

Intended implementation checks: focused FIX-20 core tests; `mise run check`,
`mise run test`, `mise run bindings-proof`; fixture oracle regeneration; actual native
gallery and SVG/PDF/PNG inspection; macOS/Linux core execution and WP-22 performance.
Add a scale-oracle mise task only when its acceptance runner exists.

## Checks executed for this planning review

Working directory: `/Users/jeickmeier/Projects/finstack-chart`; macOS/Darwin arm64,
Rust 1.97.1. Review baseline `1cb9557`; final documentation changes are uncommitted.

- Inspected specification, implementation plan, ledger, ADR-005, scale/geometry and
  alpha contracts; scale implementations, axis types/dispatch, formatting and test coverage.
- Retrieved the official D3 family documentation and pinned source export/algorithm
  references. This was documentation/source research, not an executed D3 differential run.
- `mise exec -- cargo test -p chart-core --test scales --test full_scales --locked`:
  **13 passed, 0 failed** (9 foundational, 4 full-scale tests).
- Documentation link/dependency and whitespace checks are recorded in the status ledger.

No implementation, fixture baseline, dependency or wire version was changed. No fresh
Python/WASM, native/export, Linux or performance gate was run. Next scale task: SP-01.
