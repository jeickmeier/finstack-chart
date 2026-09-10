# D3 interpolation parity: review and implementation plan

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Mode: planning only. Baseline: `1cb955740c2dad2607b0a2330201125294cab5d0`
plus existing uncommitted action, binding, shape, scale and axis work.
Authority: [specification](../spec/gpui-charts-specification.md), ITP-01–08;
execution: [main plan](gpui-charts-implementation-plan.md);
evidence: [status ledger](../implementation-status.md).

The implementation does **not** have D3 interpolation feature parity. This plan makes
that coverage required, gives each algorithm one owner, and defines how to prove it.
Existing alpha acceptance remains valid for its recorded scope. This task changes no
Rust implementation, binding API, dependency, fixture or visual baseline.

## Implementation update — 9 September 2026

WP-IP01–06 are complete for their recorded scopes. WP-IP07 has a complete standalone
27-export/configuration catalog, fresh reference/host replay, integrated sampled
publication and finite preparation/sampling/allocation evidence. It remains PARTIAL
because WP-AX06 is unfinished; G-INTERPOLATE is not passed. See the
[integration report](../evidence/phase-2-interpolation-integration-2026-09-09.md) and
[status ledger](../implementation-status.md). The planning narrative below retains the
original review's provenance; it is not the current implementation status.

## Reference scope and sources

Target `d3-interpolate` **3.0.1**, with `d3-color` **3.1.0** proposed for the shared
oracle lock; WP-IP01 must resolve and record exact source revisions, checksums and
licenses. This is a reference-tool selection, not an adopted production dependency.
The package manifest confirms interpolation version 3.0.1 and an ISC license; its
loose d3-color dependency range is insufficient for reproducible fixtures.
[Package manifest](https://cdn.jsdelivr.net/npm/d3-interpolate@3.0.1/package.json).

The public export index enumerates 27 exports. The matrix below also includes factory
configuration and result metadata. The retrieved index and dispatch/array sources are
from upstream `main`, so WP-IP01 must reconcile them with the pinned release before
freezing fixtures. The versioned zoom source was inspected; no D3 runtime ran here.
[Export index](https://raw.githubusercontent.com/d3/d3-interpolate/main/src/index.js),
[dispatch](https://raw.githubusercontent.com/d3/d3-interpolate/main/src/value.js),
[number arrays](https://raw.githubusercontent.com/d3/d3-interpolate/main/src/numberArray.js),
[versioned zoom](https://raw.githubusercontent.com/d3/d3-interpolate/v3.0.1/src/zoom.js).

The module covers values, colors, transforms and zoom trajectories. Its public
capabilities are required as typed Rust and portable equivalents. JavaScript syntax,
prototype behavior and mutable object identity are language adaptations. This does
not add d3-transition's scheduler/selection API, d3-ease, d3-timer, a CSS cascade,
d3-zoom's entire interaction API, a palette catalog or arbitrary path morphing.
D3 shape curves remain SHP-03; scalar basis interpolation is a separate operation.
[Module scope](https://d3js.org/d3-interpolate).

## Capability inventory and current verdicts

“Fail” means a required capability is absent or demonstrably narrower; it does not
mean a defect in the previously accepted subset. “Uncertain” means existing related
behavior has no differential proof. No complete interpolation family is certified Pass.

| Reference exports / configuration | Live implementation evidence | Verdict / owner |
| --- | --- | --- |
| `interpolate` | No public interpolation module in `chart-core/src/lib.rs:36–65`; no typed target-driven dispatch or recursive value evaluator | Fail; ITP-01/02, WP-IP03 |
| `interpolateNumber`, `interpolateRound` | Private `scales/linear.rs:238` checks finite results and treats endpoints specially; no standalone factory or D3 rounding policy | Uncertain for overlapping finite numeric results; Fail for public/rounded surface; WP-IP02 |
| `interpolateString`, `interpolateDate` | Data timestamps exist, but no embedded-number string interpolation or Date-equivalent evaluator | Fail; ITP-02, WP-IP03 |
| `interpolateArray`, `interpolateNumberArray`, `interpolateObject` | Portable columns/objects serialize data; they do not interpolate nested values or preserve numeric-array element conversion | Fail; ITP-02, WP-IP03 |
| `interpolateBasis`, `interpolateBasisClosed` | No scalar multi-control-point spline evaluator; path primitives are unrelated | Fail; ITP-03, WP-IP02 |
| `interpolateDiscrete`, `piecewise`, `quantize` | `ColorScale` uses equally spaced palette segments internally; no general public composition/sampling operations | Fail; ITP-03, WP-IP02 |
| `interpolateRgb`, `.gamma` | `scales/color.rs:105–139` blends and rounds four byte channels after domain clamp/omit; no floating color input or gamma factory | Uncertain for opaque RGB overlap; Fail for full contract; ITP-04, WP-IP04 |
| `interpolateRgbBasis`, `interpolateRgbBasisClosed` | No color spline operation | Fail; ITP-04, WP-IP04 |
| `interpolateHsl`, `interpolateHslLong`, `interpolateLab`, `interpolateHcl`, `interpolateHclLong` | `scene.rs:20` stores sRGB/alpha bytes only; no interpolation color-space conversion or hue-path policy | Fail; ITP-04, WP-IP04 |
| `interpolateCubehelix`, `interpolateCubehelixLong`, both `.gamma` | No Cubehelix kernel or configurable lightness progression | Fail; ITP-04, WP-IP04 |
| `interpolateHue` | No standalone angular interpolator | Fail; ITP-04, WP-IP04 |
| `interpolateTransformCss`, `interpolateTransformSvg` | Scene geometry/layout translation supplies no public affine decomposition/interpolator or CSS/SVG transform input/output adapter | Fail; ITP-05, WP-IP05 |
| `interpolateZoom`, `.rho`, result `.duration` | State actions and viewport domains do not supply a sampled smooth camera trajectory or duration metadata | Fail; ITP-06, WP-IP05 |
| Shared Rust/Python/WASM and chart consumers | Existing portable session/proofs cover chart definitions, data and actions; no standalone interpolation proof catalog | Fail for absent API; Uncertain for future host parity; ITP-07/08, WP-IP06/07 |

Paths above are relative to [chart-core source](../../crates/chart-core/src).
Additional inspected anchors: [linear helper](../../crates/chart-core/src/scales/linear.rs),
[color mapping](../../crates/chart-core/src/scales/color.rs),
[scene color/gradient](../../crates/chart-core/src/scene.rs),
[portable wire](../../crates/chart-core/src/portable/wire.rs),
[state](../../crates/chart-core/src/state.rs),
[existing scale tests](../../crates/chart-core/tests/full_scales.rs).
The scene's two-stop gradient at `scene.rs:507` is a renderer resource, not a general
interpolation engine. Planning source anchors refer to this live checkout.

## Findings that drive the plan

| Priority / requirement | Evidence and concrete missing behavior | Required change |
| --- | --- | --- |
| P1 / ITP-01/02 | `lib.rs` exposes no interpolator. A string pair `"2px"`/`"10px"` cannot be sampled as `"6px"` at 0.5, nor can matching fields in nested objects be blended. | Public typed value factories with bounded recursive evaluation and target-shaped output. |
| P1 / ITP-04 | Byte-only `Color` cannot retain fractional opacity or missing hue channels. Current palette blending cannot express RGB gamma 2.2, Lab or long-hue interpolation. | Consume CLR color values/conversions, add floating interpolation; round only at the declared output boundary. Preserve byte-scene recipes explicitly. |
| P1 / ITP-05/06 | Current state/scene API has no transform/zoom interpolator. Component blending of arbitrary matrix entries or linear zoom width is insufficient. | Shared affine decomposition and smooth zoom kernels with parameter/duration evidence. |
| P1 / ITP-07 | SP-04 previously owned interpolation while WP-AX05 needed transition math; neither enumerated the full standalone module. | WP-IP owns interpolation; scales and axis transitions consume it. No host copy of the formulas. |
| P2 / ITP-03/08 | Existing full-scale tests have no interpolation oracle. Palette segments do not certify scalar basis, typed sampling, out-of-range behavior or output lifetime. | FIX-I01 method inventory, independent boundary cases, pinned reference results and actual binding execution. |

## Compatibility and implementation contract

1. **One core engine.** Add a focused `chart-core::interpolate` module as working slices
   land. Compile input descriptors once; sampling receives explicit `t`, performs no
   I/O and reads no clock. Native typed factories and bounded portable descriptors
   invoke the same kernels. No new crate is justified now. Share numeric primitives
   with scales where contracts match; retain checked scale normalization and DAT-05.
   Update [ADR-005](../adr/005-foundational-scales-and-layout.md) and
   [ADR-006](../adr/006-portable-specification-and-binding-proofs.md) during WP-IP01.
2. **Defined typed profile.** Native numeric input is binary64. Portable values include
   distinct missing/null, boolean, number, text, date, color, numeric arrays, general
   arrays and string-keyed records. Dispatch by the target kind, including color-string
   recognition before ordinary string interpolation; null/missing/booleans are constants.
   Preserve target-only fields/elements and omit source-only fields. Specify each
   mixed-kind pair's conversion or diagnostic; do not substitute general JS coercion.
   Bound nesting, element count, string bytes and sampled output using existing limits.
3. **Lifetime and exceptional values.** Default samples own their results. An explicit
   reusable destination may serve hot loops, with clear overwrite semantics; stored
   `quantize` samples remain independent. Do not emulate D3's accidental shared-result
   aliasing. Preserve supported numeric-array destination kinds (Float32/64 and Number-
   based signed/unsigned/clamped integer arrays) and their conversion rules, including
   wrap versus clamp and ties. BigInt/DataView are not silently accepted numeric arrays.
   Tagged non-finite/missing results at the numerical boundary must never become invalid
   scene coordinates or ambiguous JSON null. Enumerate exact diagnostic adaptations.
4. **Numbers, strings, dates and composition.** Record per-operation parameter behavior,
   not a blanket [0,1] clamp. Numeric and piecewise extrapolation, basis endpoint clamping,
   closed-basis wrapping and discrete endpoint saturation need separate tests. Match
   JS half-toward-positive-infinity rounding where observable, including negative ties.
   Date compatibility uses epoch milliseconds, truncation/time clipping and invalid-date
   classification; exact source timestamps and IDs keep their original integer units.
   String token pairing follows target text; specify numeric formatting, exponent and
   signed-zero behavior. Sample count and empty/singleton control inputs need explicit
   dispositions. Quantize requires n > 1 in the supported documented profile; other
   reference edge outputs must be inventoried rather than mistaken for valid geometry.
   [Value API](https://d3js.org/d3-interpolate/value).
5. **Colors.** Consume CLR-02/03's shared floating representation with explicit missing
   channels, parser/conversions and RGB output formatting. The
   [color plan](d3-color-parity-plan.md) owns those primitives; this lane owns blending.
   Keep color recognition local to interpolation; THM-02 mapped/style rules do not change.
   Support RGB, HSL, Lab, HCL and Cubehelix, hue short/long routes, hue normalization,
   relevant gamma factories, alpha behavior, and open/closed RGB splines. Match the
   reference's opaque RGB-spline result rather than inventing alpha spline support.
   Test parsing of named/hex/rgb(a)/hsl(a)/transparent values and invalid color fallback.
   Do not quantize Lab/HCL/gamma intermediate values to scene bytes. Scalar splines
   consume WP-IP02; no separate spline implementation per color space.
   [Color API](https://d3js.org/d3-interpolate/color).
6. **Transforms.** Core operates on finite 2D affine matrices and decomposed translation,
   rotation, x-skew and scale. Include reflections, singular cases and shortest rotation.
   Provide CSS and SVG transform-list adapters and canonical output in their respective
   units/syntax, including SVG centered rotation. Test composition order; interpolating
   six matrix entries is not decomposition parity. DOMMatrix/SVG consolidation used by
   the oracle stays in a pinned real browser harness. Absolute 2D inputs must work
   headlessly; context-dependent inputs require explicit resolved matrices/resources.
   Do not claim 3D/perspective, CSS cascade or arbitrary DOM evaluation. A missing defined
   2D transform capability remains a gap; host syntax adaptations cannot conceal it.
   [Transform API](https://d3js.org/d3-interpolate/transform).
7. **Zoom.** Use typed center-x/center-y/width values, finite centers and positive width,
   default rho sqrt(2), configurable rho and separately exposed reference duration.
   Cover near-coincident centers and extreme ratios. The pinned source floors rho at
   1e-3 and can return negative duration for coincident-center zoom-in: retain oracle
   metadata; a host uses an explicitly documented nonnegative scheduling duration.
   Do not cast a signed reference duration directly to an unsigned host duration.
   [Zoom API](https://d3js.org/d3-interpolate/zoom),
   [duration/rho implementation](https://raw.githubusercontent.com/d3/d3-interpolate/v3.0.1/src/zoom.js).
8. **Consumers and extension boundaries.** Scales own domain normalization/clamp/unknown
   policy; interpolation owns range sampling. Native custom factories remain usable;
   portable definitions name built-ins or registered versioned Rust operations, following
   the existing extension registry. Unregistered closures fail serialization explicitly.
   Resolve callbacks once, not per mark. Renderer color conversion is one declared final
   boundary. Arbitrary color ramps used in publication need controlled shared sampling
   with measured error, not an unverified two-stop backend approximation. Host clocks,
   interruption, reduced motion, disposal and presented-scene capture stay with the
   existing state/axis/runtime packages. A typed sampled transform/zoom is available
   independently of running an animation or opening a window.

## Work packages and dependency order

Owned new paths below are proposed, not files claimed to exist. Each package lands
working behavior and discriminating fixtures; do not scaffold all public APIs first.

| Package | Prerequisites | Deliverables / owned paths | Acceptance / provisional effort |
| --- | --- | --- | --- |
| WP-IP01 — Contract and reference harness | WP-14 | ITP-01/08; ADR amendments, shared oracle manifest/lock, proposed `fixtures/interpolate/`, generation/comparison runner and complete method/default/adaptation inventory | Pin 3.0.1 and color dependency, reconcile export index, generate deterministic numeric and browser-transform reference outputs, establish open-gap report; 3–5 days |
| WP-IP02 — Scalar kernels and composition | WP-IP01 | ITP-02/03; proposed `interpolate/` numeric/round/basis/discrete/piecewise/quantize kernels and typed composition boundary | Independent scalar/negative-tie/boundary/extrapolation/periodicity/sampling cases; callable standalone Rust surface; 3–5 days |
| WP-IP03 — Structured values | WP-IP02 | ITP-01/02/07; string/date/array/record factories, target dispatch, bounded owned/reusable outputs and descriptor validation | Nested target shape, numeric-array casting, token formatting, date clipping, limits and sample independence; color dispatch completes with WP-IP04; 5–8 days |
| WP-IP04 — Color interpolation | WP-IP02, CLR-03 | ITP-04; consume floating colors/parsing/conversion from the [color plan](d3-color-parity-plan.md); own hue/gamma and RGB splines in core | Every color export/configuration matches reference plus independent channel cases; explicit byte/alpha boundary; 3–5 days |
| WP-IP05 — Transform and zoom interpolation | WP-IP02 | ITP-05/06; affine decomposition, CSS/SVG adapters, smooth zoom/rho/duration and pure standalone output | Real-browser transform oracle plus headless comparisons, geometry-equivalent matrices, small-distance/ratio/negative-duration zoom cases; 6–10 days |
| WP-IP06 — Portable and chart integration | WP-IP03/04/05, CLR-04, SP-04, WP-16, WP-19, WP-20 | ITP-07; shared portable envelopes/session, Python/WASM adapters, scale/legend/theme integration, sampled transform/zoom gallery and coherent export | Actual standalone and composed Rust/Python/WASM samples; inspected native/SVG/PDF/PNG ramp/transform/zoom frames; update/fresh and lifetime checks; 4–7 days |
| WP-IP07 — Parity certification | WP-IP06, SP-07, WP-AX06, CLR-05, CP-05 | ITP-01–08; full FIX-I01 verdict catalog, cross-lane consumers, docs/support matrix and benchmark evidence | All 27 exports plus gamma/rho/duration/configuration have passing evidence for declared hosts/profile; no missing capability; 3–5 days |

Provisional total: **27–45 engineer-days** for the interpolation lane, including interpolation work
previously implied by SP-04, excluding the separately estimated CLR color engine. Re-estimate after WP-IP01 and the transform/color spike;
subtract transferred interpolation work from the scale estimate rather than counting
it twice. Existing scale/axis/runtime package estimates remain separate consumers.
No staffing or parallel-agent execution is assumed.

Color ownership handoff: CLR-01–03 own shared parsing, values, conversion and
formatting; CLR-04 owns color descriptors and paint lowering. WP-IP04 consumes CLR-03
and owns interpolation only. The revised 3–5 day WP-IP04 estimate excludes foundation
work already budgeted in CLR-01–03. CLR-05 consumes
SP-04 but does not require WP-IP06/07. Keep COL/FIX-C01 distinct from ITP fixtures.

Dependency handoff (no backward dependency on final certification):

- WP-IP01 and SP-01 coordinate one development oracle lock and descriptor boundary;
  neither waits for the other's completed implementation.
- SP-02 consumes WP-IP02 scalar/round/composition kernels. SP-04 requires WP-IP03/04
  and owns scale normalization, classifiers and guide integration, not interpolation.
- CLR-03 supplies WP-IP04 color values/conversion/formatting; CLR-04 supplies the
  WP-IP06 wire/paint boundary. CP-03 consumes WP-IP04 interpolation and owns named
  ramps; CP-04 owns their chart integration. CLR/CP certification never waits for WP-IP07.
- WP-AX05 consumes WP-IP02/05 for numeric/transform sampling and owns axis lifecycle.
  It does not wait for WP-IP06/07. WP-IP06 does not require WP-AX06 or SP-07.
- SP-07 and WP-AX06 retain their scale/axis certification roles. WP-IP07 waits for them
  plus CLR-05/CP-05 and verifies shared consumers without reopening algorithm ownership.
- WP-15–20 retain existing prerequisites and active assignments; their completed
  interfaces feed WP-IP06. WP-21 and WP-22 require WP-IP07; WP-23 inherits them.
  G-INTERPOLATE and G4 stay open until full proof exists.

## FIX-I01 acceptance matrix

The generator records versions/checksums, browser engine/OS for transforms, input type,
parameters, expected output type/value, duration, diagnostic adaptation and tolerance.
Commit results for offline Rust tests; D3 is never a runtime dependency. The reference
must not use the Rust code under test to compute expected values. Add hand-calculated
examples and metamorphic checks alongside reference snapshots.

| Cases | Discriminating acceptance |
| --- | --- |
| FIX-I01-A: inventory and dispatch | All 27 exports, 3 gamma factories, rho and duration; target boolean/null/missing, numeric versus color/text, nested mixed kinds, unsupported coercions, unknown operation/version and bounded recursion |
| FIX-I01-B: scalar and date | t = -0.25, 0, 0.25, 0.5, 1, 1.25; equal/reversed endpoints, negative half rounding, signed zero, subnormal/extreme finite inputs, classified NaN/Inf, fractional/negative epoch milliseconds and Date limits; preserve source time/ID > 2^53 |
| FIX-I01-C: strings and structures | Scientific notation, signed/decimal tokens, differing token counts and static suffixes, unmatched/empty arrays and objects, target-only fields, typed arrays (float rounding, integer wrapping, Uint8Clamped ties), repeated/out-of-order evaluation and retained sample independence |
| FIX-I01-D: splines/composition | Open endpoints, closed periodicity and C2 seam checks, non-monotone controls, zero/one/two/many values, exact discrete and piecewise breakpoints, extrapolation policy, quantize endpoint spacing, custom factory invocation and invalid counts |
| FIX-I01-E: colors | All spaces and hue routes, gamma 1 and nondefault values, invalid gamma classification, achromatic/missing channels, transparent/fractional alpha, out-of-gamut conversion, parsing/formatting, RGB-spline opacity limitation, byte conversion once |
| FIX-I01-F: transforms | Identity/none, all 2D operations, centered SVG rotate, composition order, 350-to-10 rotation, reflections/negative scale, singular matrices, CSS/SVG unit/format distinction, invalid syntax/context, matrix and transformed-point comparisons at multiple t |
| FIX-I01-G: zoom | Same/near/distant centers, equal/increasing/decreasing widths, default/custom/floored rho, invalid widths, reversed endpoints, duration sign, extreme ratio conditioning, endpoint fidelity and finite sampled views |
| FIX-I01-H: consumers and lifecycle | Identical standalone/scale samples, asymmetric diverging domain, copied interpolator/scale independence, palette changes without stat retraining, axis interruption/reduced motion, coherent sampled export, stale job/disposal, actual Python/WASM values/metadata and inspected native/publication frames |

Start finite scalar/color-space/zoom comparisons at 1e-12 absolute plus 1e-12 relative
where conditioning permits; derive separate operation bounds in WP-IP01, not one loose
visual epsilon. Integers, categories, dispatch, string templates and date milliseconds
compare exactly. Typed Float32 output compares after the specified cast. RGB string/byte
rounding boundaries compare exactly; retain floating values to diagnose differences.
Transforms compare decomposition/recomposition, mapped points and canonical syntax
separately; singular and ill-conditioned cases need explicit expectations. Date, hue,
basis and constant-target operations have family-specific endpoint behavior, so a single
“all interpolators return a at zero” invariant is invalid. Undefined upstream cases
receive named typed diagnostics and do not count as matching defined capabilities.

Measure factory construction separately from repeated sampling, with nested sizes,
allocation counts, reusable-buffer behavior and color/transform/zoom workloads. Use
WP-22's existing PERF protocol; set measured budgets before optimization. No throughput
claim follows from formula review or compile-only tests.

## Completion and evidence from this review

This planning review inspected the linked official API pages, available upstream sources,
core scale/color/scene/state/portable code and existing companion plans/ADRs.
Concurrent color/chromatic plans were incorporated, preserving specification 0.3.0
and using FIX-I01 to avoid their FIX-C01/FIX-21 identifiers. Some pinned
source requests were unavailable and shell network resolution failed; release source
reconciliation and reference execution remain explicit WP-IP01 acceptance work.
No D3 differential fixtures, runtime binding proof, native/export artifact, Linux run or
performance measurement was produced. Documentation checks and any existing-subset test
results are recorded in the status ledger, not credited as FIX-I01 passes.

Next interpolation task: **WP-IP01**. Full parity is complete only when ITP-01–08,
FIX-I01-A–H, WP-IP01–07 and G-INTERPOLATE have evidence, with the supported profile,
language adaptations and remaining platform boundaries stated in release documentation.
