# D3 scale-chromatic feature parity

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Assignment: review implementation and plan full parity;
documentation only. Baseline: `1cb955740c2dad2607b0a2330201125294cab5d0` plus existing
action/state/binding and shape/scale/axis planning edits. Specification: 0.3.0.
Requirements: CHR-01–06, SCL-03/06–08, THM-02, ARC-04, BND-01/03/04, QLT-02/03/04.
Acceptance: FIX-21 and G-CHROMATIC, with existing FIX-06/08/12–16 regressions.

## Outcome and reference boundary

**Full parity is absent.** The current engine has user-supplied discrete palettes and
piecewise RGB-byte interpolation. It has no named D3 catalog, Brewer spline ramps or
the other chromatic interpolators. The prior scale plan explicitly excluded the
catalog; this addition makes it required production scope. Historical WP-11/13/G2
acceptance remains valid for its recorded behavior. This planning task implements no
color features and passes no parity gate.

Reference: [official documentation](https://d3js.org/d3-scale-chromatic), retrieved
7 September 2026, and the [3.1.0 exports](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/index.js).
The [package metadata](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/package.json)
identifies the standalone version; the website's D3 7.9.0 label is not this module's
version. CP-01 pins source commits, integrity hashes and the complete oracle lockfile,
including d3-interpolate 3.0.1 and d3-color 3.1.0; use d3-scale 4.0.2 for composition
cases and share the SP-01 fixture tooling. These are development references, not Rust
runtime dependencies. Preserve the upstream [license notices](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/LICENSE),
including both D3 and ColorBrewer provenance, when adapting algorithms or tables.

Parity means equivalent typed capabilities and output colors. JavaScript export
spelling, mutable array identity and CSS string whitespace are not required. Canonical
RGBA bytes are the comparison surface; preserve original D3 strings in oracle evidence.
All schemes and interpolators must be independently usable from Rust, Python and WASM
as well as usable in charts. This does not require the entire d3-color/d3-interpolate
API, a CSS parser or a browser renderer. Shared interpolation already required by SP-04
has one owner; no second engine is introduced here.

## Complete export inventory

Names below are D3 suffixes: a scheme entry means `scheme<Name>` and an interpolator
entry means `interpolate<Name>`. Each name and each supported size needs its own
machine-readable fixture row. There are **76 exports: 38 schemes and 38 interpolators**.

| Family | Names and cardinalities | Scheme exports | Interpolator exports |
| --- | --- | --- | --- |
| Categorical | Category10 (10), Observable10 (10), Tableau10 (10), Accent (8), Dark2 (8), Paired (12), Pastel1 (9), Pastel2 (8), Set1 (9), Set2 (8), Set3 (12) | 11 fixed arrays | 0 |
| Sequential, single hue | Blues, Greens, Greys, Oranges, Purples, Reds | 6; every k=3–9 | 6 |
| Sequential, multiple hues, Brewer | BuGn, BuPu, GnBu, OrRd, PuBu, PuBuGn, PuRd, RdPu, YlGn, YlGnBu, YlOrBr, YlOrRd | 12; every k=3–9 | 12 |
| Diverging, Brewer | BrBG, PRGn, PiYG, PuOr, RdBu, RdGy, RdYlBu, RdYlGn, Spectral | 9; every k=3–11 | 9 |
| Sequential, lookup | Viridis, Inferno, Magma, Plasma | 0 | 4 |
| Sequential, polynomial | Cividis, Turbo | 0 | 2 |
| Sequential, Cubehelix | CubehelixDefault, Warm, Cool | 0 | 3 |
| Cyclical | Rainbow, Sinebow | 0 | 2 |

The fixed and size-indexed schemes total **218 discrete arrays** (11 + 18×7 + 9×9).
Sources: [categorical](https://d3js.org/d3-scale-chromatic/categorical),
[sequential](https://d3js.org/d3-scale-chromatic/sequential),
[diverging](https://d3js.org/d3-scale-chromatic/diverging),
[cyclical](https://d3js.org/d3-scale-chromatic/cyclical), and the pinned exports above.
There are no D3 `schemeViridis`, `schemeTurbo` or variable-size Category10 exports.
Sampling an interpolator is useful composition, but must not be advertised as an
official discrete scheme or used to manufacture Brewer tables.

## Source review and ranked gaps

Paths/lines describe the inspected working tree; concurrent edits may move them.
Fail means missing capability confirmed in source, not a regression against 0.1.0.
Uncertain means the required runtime evidence has not been produced.

| Priority / requirements | Evidence and counterexample | Verdict | Fix owner |
| --- | --- | --- | --- |
| P1 / CHR-01/02 | [color.rs:8–28](../../crates/chart-core/src/scales/color.rs) accepts only raw palettes. No built-in IDs, exact size tables or standalone catalog query; selecting Observable10 or Blues[3] is inexpressible without caller-supplied colors. | Fail | CP-02 |
| P1 / CHR-03 | [color.rs:123–137](../../crates/chart-core/src/scales/color.rs) linearly interpolates equally spaced palette bytes. D3 Brewer ramps use a cubic basis over the largest table. Passing Blues[9] as the palette cannot reproduce the ramp interior. | Fail | CP-03 |
| P1 / CHR-03 | Same implementation cannot represent Viridis lookup jumps, Turbo/Cividis polynomials, long-hue Cubehelix or cyclical Rainbow/Sinebow. Endpoint matching alone conceals these differences. | Fail | CP-03 |
| P1 / CHR-04 | [grammar/colors.rs:56–73,164–197](../../crates/chart-core/src/grammar/colors.rs) branches only on the two current color families. Named interpolators must compose with the sequential/diverging/classifier families planned in SP-04. For diverging [-10,0,100], 0 must use t=0.5, not 1/11. | Fail | SP-04 + CP-04 |
| P2 / CHR-04 | [ColorLegend:30–43,139–191](../../crates/chart-core/src/scales/color.rs) stores palette entries and a continuous flag; [facets.rs:40–51,77–97](../../crates/chart-core/src/layout/facets.rs) compares/paints those entries. Ramp identity, reversal, evaluation and lookup discontinuities cannot be reconstructed from that metadata. | Fail | CP-04 |
| P1 / CHR-05 | [portable/wire.rs:8–19](../../crates/chart-core/src/portable/wire.rs) transports the existing definition; [Python](../../crates/chart-python/src/lib.rs) and [WASM](../../crates/chart-wasm/src/lib.rs) expose chart sessions, not chromatic query/evaluate operations. Enum serialization alone would leave standalone parity unavailable. | Fail | CP-04 |
| P1 / CHR-06 | [full_scales.rs:143–216](../../crates/chart-core/tests/full_scales.rs) proves black/white midpoint and missing/category behavior, not a named reference catalog. No chromatic differential inventory was found. | Fail | CP-01/05 |
| P2 / CHR-06 | Color bytes already have shared scene/native/export routes, but named ramps have no actual binding, rendered, update or performance proof. Existing output evidence cannot certify unimplemented descriptors. | Uncertain | CP-05 |

Reusable foundations: [scene Color](../../crates/chart-core/src/scene.rs) is already
unpremultiplied sRGB RGBA bytes; mapped colors are resolved in core; category order and
missing style are explicit. These are useful existing contracts, not a chromatic Pass.

## Behavioral and ownership decisions

1. Keep catalog data and pure lookup/evaluation in `chart-core::scales`, preferably a
   small `chromatic` submodule. Immutable static tables and checked typed IDs suffice;
   no new crate, mandatory I/O, registry framework or interpreter dependency is needed.
   Expose catalog metadata, categorical lookup, sized Brewer lookup and `evaluate(t)`.
   Reject unknown IDs, wrong family/size combinations and unsupported k values with
   contextual diagnostics. Returned owned host arrays cannot mutate shared tables.
2. Discrete schemes return exact source order and bytes, alpha 255. Do not truncate a
   maximum-size scheme or sample a continuous ramp to implement smaller Brewer sizes.
   Example: Blues[3] is #deebf7, #9ecae1, #3182bd; it is not the two endpoints and midpoint
   of interpolateBlues. The [Blues source](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-single/Blues.js)
   defines each size separately.
3. Use the actual [Brewer ramp](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/ramp.js):
   interpolateRgbBasis over the largest array, with endpoint control extrapolation and
   t clamped by the [basis evaluator](https://github.com/d3/d3-interpolate/blob/v3.0.1/src/basis.js).
   Keep intermediate channels as f64 and round/clamp once at the output boundary.
   Do not reuse geometric spline tessellation, substitute linear RGB, or require the
   shape parity workstream to implement this four-value scalar polynomial.
4. [Viridis/Inferno/Magma/Plasma](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/viridis.js)
   use 256-entry floor-indexed lookup with endpoint saturation. Preserve jumps at i/256;
   do not smooth them. [Turbo](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/turbo.js)
   and [Cividis](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/cividis.js)
   use their pinned polynomial approximations, clamped t and rounded/clipped channels;
   substituting another library's same-named palette is not evidence of compatibility.
5. [CubehelixDefault](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/cubehelix.js),
   [Warm/Cool/Rainbow](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/rainbow.js)
   and [Sinebow](https://github.com/d3/d3-scale-chromatic/blob/v3.1.0/src/sequential-multi/sinebow.js)
   require their exact hue paths and formulas. Warm/Cool/Default use long-hue Cubehelix;
   Rainbow wraps finite t outside [0,1]; Sinebow uses periodic sine-squared channels.
   No universal clamping wrapper may erase these differences. Test large finite t
   against pinned results rather than assuming numerically exact periodicity there.
6. Separate domain normalization/unknown/outside policy (SP-04) from the pure
   interpolator (CP-03). Cover every finite t whose reference result is a valid color,
   including outside [0,1]. Reject non-finite direct inputs explicitly; chart null/NaN/Inf
   values retain the declared missing color. Store undefined/invalid reference outcomes
   as tagged fixture results, never JSON NaN. Preserve current v1 continuous behavior
   (including clamp=false meaning missing outside) through explicit legacy policy.
7. Provide explicit reversal: reverse discrete array order; evaluate continuous ramps
   at 1-t. Descending domain normalization is a separate operation, and reversing both
   is tested. Optional bounded sampling can reuse shared interpolation utilities; n=0,
   n=1, endpoint selection and invalid budgets need declared semantics if exposed.
   No user callback is required to select any built-in scheme.
8. The [color plan](d3-color-parity-plan.md) CLR-03 owns RGB/Cubehelix conversion;
   WP-IP04 consumes it and owns generic color interpolation; SP-04 owns sequential/diverging
   normalization, distributions and base legend representation. CP-03 consumes those
   primitives and owns catalog-specific recipes and lookup/polynomial algorithms.
   CP-01 agrees this boundary with SP-01 before either edits shared interfaces.
   SP-04 must not depend on CP-04/05; that would create a cycle.
9. CP-04 extends descriptors and existing metadata with stable catalog/interpolator
   identity and revision, k, reversal, mapping domain/transform, outside/missing policy.
   Resolve once per configuration, not per mark. Legend samples evaluate the same
   function as marks; explicit sampled swatches are sufficient when labeled accurately.
   If rendering a continuous strip, preserve lookup discontinuities and use bounded,
   tested approximation for other ramps, not a gradient through three palette entries.
   Shared-guide equality/cache keys include the full descriptor. Theme monochrome
   conversion happens after canonical color evaluation; it cannot alter oracle results.
10. Color-only reconfiguration preserves numerical/statistical preparation, source
    identities and positional domains. Legend text/layout may invalidate as necessary.
    Extend actual Python/WASM operations for catalog listing, discrete lookup and
    evaluation, and test chart descriptors in the same core engine. Coordinate BND-01
    schema migration with SP-01/04 and ongoing action work: read existing v1 definitions
    with unchanged meaning; reject unknown newer variants on older readers. Do not pick
    a wire version merely from the specification's document version.

## Ordered work packages

Each package includes focused tests, docs and ledger evidence. Effort below is a
provisional incremental budget in engineer-days, including review; it excludes SP-04's
shared interpolation/scale work and already-assigned WP-15–20 work. Re-estimate after
CP-01. This is a schedule, not an instruction to launch agents.

| Package | Prerequisites | Requirements | Owned deliverables | Exit evidence | Effort |
| --- | --- | --- | --- | --- | --- |
| CP-01 — Reference contract and oracle | WP-14 | CHR-01/03/05/06, ARC-04 | Chromatic ADR/contract, 76-export manifest, source/license pins, shared development oracle integration, stored fixture reader | Reproducible generation, complete 218-array inventory, explicit expected-gap report and compatibility/migration decisions; no parity claim | 1–2 |
| CP-02 — Exact discrete catalog | CP-01 | CHR-01/02 | Static tables, typed IDs/metadata and checked native query API in core | Every array/order/byte matches; invalid k/ID/family and mutation isolation cases pass | 1–2 |
| CP-03 — Complete interpolator catalog | CP-02, SP-04 | CHR-01/03 | All 38 evaluators using shared RGB/Cubehelix primitives; parameter and reversal policy | All interpolator fixtures and independent boundary cases pass, including lookup discontinuities and cyclic/outside behavior | 3–5 |
| CP-04 — Chart, guide and portable integration | CP-03 | CHR-04/05, SCL-03, THM-02, BND-01/03/04 | Color descriptors, compiler/cache/legend integration, schema migration, standalone Python/WASM query/evaluate APIs | Real three-language execution; asymmetric domains, classifiers, missing values, reversal, guide identity and preserved v1 fixtures | 2–4 |
| CP-05 — Integrated certification | CP-04, SP-07, WP-20 | CHR-01–06, QLT-02/03/04 | Complete FIX-21 report, native/export gallery, live-update fixtures, API/support/provenance documentation and measured costs | G-CHROMATIC passes only with complete reference/host/update/visual evidence and no open required row | 2–3 |

Total incremental budget: **9–16 engineer-days**, with dependency wait time additional.
CP-01/02 can begin while scale work proceeds. CP-03 waits for the shared SP-04 API and
proofs. CP-04 may extend its host proof before general scale certification, but CP-05
requires SP-07's accepted scale composition and WP-20's coherent exports. CP-05 precedes
WP-21/22; WP-23 documents both G-SCALE and G-CHROMATIC. No dependency on shape or axis
certification is added, and the active WP-15 assignment remains independent.

## FIX-21 acceptance matrix

| Case | Discriminating inputs | Required evidence |
| --- | --- | --- |
| Catalog completeness | Exactly the 76 exports above, all 218 arrays, every fixed length and every valid Brewer k; invalid k=0/1/2/10/12 as applicable, wrong family and unknown names | Exact manifest/length/order/RGBA equality; structured diagnostics; no silent fallback or synthesized size |
| Brewer ramps | All 27 ramps at endpoints, dense t=i/4096, each basis knot and adjacent representable values | Pinned RGBA results plus independent four-control cubic cases; discrete tables and sampled ramps distinguished |
| Lookup ramps | All 4×256 bins, both sides of every i/256 boundary, t=0/1, negative and >1 | Exact table/index/byte equality, including final endpoint and saturating tails |
| Analytic and Cubehelix | All 7 remaining interpolators; dense t grid, t=-1.25/-0.25/0/0.5/1/1.25/2.25, large finite values and output rounding boundaries | Polynomial, long-hue and cyclical behavior against oracle; independent anchor cases; no global clamp shortcut |
| Invalid and reverse | null, NaN, +/-Inf direct and through chart mapping; reverse arrays versus 1-t; descending domain and double reversal | Explicit typed diagnostics/missing policy, finite scene colors, reversal matches reference composition |
| Scale composition | Category reuse/reorder/overflow/unknown; sequential variants; diverging [-10,0,100]; quantile/quantize/threshold equality/tails | SP-04 normalizations/classification compose with exact catalog colors; no second training engine |
| Guides and themes | Shared/free facets, same ID with different interpolator/k/reverse, editorial/terminal/grayscale, updated legend | Full compatible identity and accurate colors; palette update leaves positions/statistics unchanged; post-color theme conversion verified separately |
| Public operations and wire | Equivalent standalone catalog/query/evaluate plus chart definitions in Rust/Python/WASM; v1 inputs, unknown variants, copies and disposed sessions | Actual runtime outputs, API/declaration coverage and migration errors; original scheme storage cannot be mutated through returned arrays |
| Updates, snapshots and artifacts | Palette switches plus append/upsert/remove/retention; export during update; categorical, Brewer, lookup, diverging and cyclical gallery panels | Fresh batch equality and one captured revision; inspected native and SVG/PDF/PNG; logical legend colors agree with marks |

Compare exact final RGBA bytes by default; CSS hex versus rgb spelling is normalized
only by the fixture tool. Intermediate f64 helper comparisons use justified,
operation-specific tolerances, not permission for arbitrary one-byte visual differences.
If platform math crosses a rounding boundary, investigate and document the exact case;
do not widen every fixture or silently replace the upstream expectation. Images test
integration, not numerical equality: raster/font tolerance is separate from color bytes.

Measure catalog footprint, evaluator allocation and large color-mapped cell/point
preparation plus repeated palette changes in the existing PERF-01–05 protocol. Record
workload, hardware, elapsed time and allocation/memory evidence; do not invent a new
performance target or infer speed from static arrays. Add a capability task only after
its acceptance runner exists. Rust tests must consume committed oracle fixtures offline.

## Planning evidence and next action

The review read the specification, implementation plan, status, scale plan, current
color/compiler/legend/theme/portable paths and existing color tests. Official docs and
pinned source were inspected; source retrieval required a read-only network-enabled
command after sandbox DNS was unavailable. No D3 execution or differential comparison
was performed. This is source evidence and a plan, not an implementation certification.

Exact local checks and their results are recorded in the status ledger. All CHR-01–06
implementation requirements remain open. **Next: CP-01**, agreeing the shared oracle
and RGB/Cubehelix boundary with SP-01/04, then CP-02's exact discrete catalog.

## Delivery record — 9 September 2026

CP-01–05 are complete and G-CHROMATIC passes for the retained typed FIX-21 snapshot,
including the 304-case exceptional-normalization supplement.
See the [retained integration snapshot](../evidence/phase-2-chromatic-integration-2026-09-09.md).
The historical assessment above describes entry gaps. Exact catalog/ramp operations,
all scale composition families, checked metadata/v6 migration, actual hosts, updates,
theme conversion, native/publication inspection and measured costs now have evidence.
WP-21/22 and the other Phase 2 gates remain separate acceptance work.
