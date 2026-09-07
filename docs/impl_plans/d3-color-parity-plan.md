# D3 color feature parity

Phase 2 coordination: [combined implementation plan](phase-2-parity-implementation-plan.md).
This document retains its detailed inventory and package ownership; the combined plan
owns cross-lane scheduling and ggplot2 integration.

Date: 7 September 2026. Assignment: compare plans and implementation with d3-color
and plan delivery. Documentation only; baseline `1cb9557` plus existing uncommitted
action/state/binding and shape/axis/scale planning changes.
Requirements: COL-01–06; acceptance catalog FIX-C01 and gate G-COLOR.

## Outcome and reference boundary

**Current implementation does not have d3-color feature parity.** It has byte paints,
palette mapping and presentation themes. This plan makes color parity required for
production while preserving the historical WP-11/13/14 and G2 acceptance scope.
Planning completion does not close any COL requirement.

Target the [documented d3-color API](https://d3js.org/d3-color), reviewed on
7 September 2026, and standalone module **3.1.0**, identified by its
[package metadata](https://raw.githubusercontent.com/d3/d3-color/v3.1.0/package.json).
The website's D3 version is a different, umbrella package version. CLR-01 must record
the reference commit, integrity hash and license with the executable oracle.

The [export list](https://raw.githubusercontent.com/d3/d3-color/v3.1.0/src/index.js)
contains eight functions: color, rgb, hsl, lab, gray, hcl, lch and cubehelix. Required
coverage includes their overloads, channels, opacity, inherited methods, RGB/HSL clamp
and the deprecated hex alias. Rust names and ownership may differ; observable color
operations must remain available in Rust and actual Python/WASM adapters.

This is the linked **d3-color** scope. The concurrent [interpolation plan](d3-interpolate-parity-plan.md) WP-IP04 owns
color interpolation and gamma/hue-path policies; the [scale plan](d3-scale-parity-plan.md)
SP-04 consumes it and owns scale integration.
COL owns the parsing/conversion/formatting engine that interpolation consumes.
The concurrent [scale-chromatic plan](d3-scale-chromatic-parity-plan.md) owns the
separately required palette catalog (CHR/CP/FIX-21); this plan uses COL/CLR/FIX-C01
to avoid ownership collisions. External color-space extensions, Delta-E packages
and full modern CSS Color parsing are not implied by d3-color parity. Do not describe completion as parity with every D3 color-related package.

## Source-backed gap matrix

Fail means a capability is absent or incompatible with the target, not that the old
alpha contract failed. Uncertain means this review has no complete runtime proof.
Line references describe the baseline and can move during ongoing work.

| Priority / capability / requirement | Live evidence and concrete gap | Verdict | Owner |
| --- | --- | --- | --- |
| P1 / CSS parsing / COL-01 | [scene.rs](../../crates/chart-core/src/scene.rs), line 21, only exposes four u8 fields; [lib.rs](../../crates/chart-core/src/lib.rs) exports no color utility module. `steelblue`, `#fea2` and `hsl(120,50%,20%)` cannot be authored as color values through a shared parser. | Fail | CLR-02 |
| P1 / RGB/HSL values / COL-02 | Scene Color cannot retain fractional RGB, arbitrary opacity or out-of-gamut channels. A brighter steelblue can have blue >255 before formatting; byte storage loses information required by subsequent operations. No HSL value/conversion API exists. | Fail | CLR-02 |
| P1 / Lab, gray, HCL/LCh, Cubehelix / COL-02 | No corresponding exported types or conversion kernels. [theme.rs](../../crates/chart-core/src/theme.rs), line 249, applies a weighted-byte grayscale presentation policy; it does not construct Lab gray(l). | Fail | CLR-03 |
| P2 / Channels/copy/brighter/darker / COL-03 | Byte Color is Copy, but no space-aware channel manipulation or brightness operations exist. Plain Rust copying is useful infrastructure, not method parity. | Fail | CLR-02/03 |
| P2 / displayable/clamp/CSS output / COL-04 | No standalone methods. [SVG encoder](../../crates/chart-export/src/svg.rs), line 32, formats resolved byte paint; that cannot validate or format an unclamped HSL/Lab value. | Fail | CLR-02/03 |
| P1 / Color values in authoring / COL-05 | [Style](../../crates/chart-core/src/grammar/definition.rs), line 548, [ThemePatch](../../crates/chart-core/src/theme.rs), [rich text](../../crates/chart-core/src/typography.rs) and [palettes](../../crates/chart-core/src/scales/color.rs) all consume scene Color. Need one shared lowering boundary and portable value descriptors. | Fail | CLR-04 |
| Current palette behavior / SCL-03 | [ColorScale](../../crates/chart-core/src/scales/color.rs), lines 8 and 131, cycles categories or linearly blends equally spaced RGBA byte stops. The focused existing test passes its midpoint/missing contract. Color interpolation controls belong to WP-IP04 and SP-04 integration. | Pass for tested old subset; Fail for expanded interpolation | WP-IP04/SP-04 with CLR-03 |
| P1 / Host operations / COL-05 | [Python](../../crates/chart-python/src/lib.rs) and [WASM](../../crates/chart-wasm/src/lib.rs) expose Chart sessions, not standalone color operations. Chart JSON acceptance would not by itself close this gap. | Fail | CLR-04 |
| P1 / Cross-destination proof / COL-06 | [native.rs](../../crates/gpui-charts/src/native.rs), line 205, and [encode.rs](../../crates/chart-export/src/encode.rs), line 63, consume byte RGBA. No FIX-C01 oracle, color-value roundtrip or new color artifact comparison exists. | Uncertain for final fidelity; Fail for required evidence | CLR-05 |

## Shared design and compatibility contract

1. Add a small public `chart-core::color` module with binary64 Rgb, Hsl, Lab, Hcl
   (LCh constructor alias) and Cubehelix values plus a tagged ColorValue. Names are
   provisional; keep all algorithms in core. Existing scene Color remains an explicit
   unpremultiplied sRGB8 paint boundary. No additional crate or host color library is
   needed solely to model these values.
2. Parse explicitly at authoring/ingestion, never during per-mark painting. Preserve
   the parsed RGB/HSL variant. Typed constructors retain their supplied channels;
   parse failure is distinct from a constructor/conversion result with undefined
   channels. Space tags replace instanceof; typed channel updates replace arbitrary
   JS property assignment. Owned copies/builders replace mutable object identity.
   Do not reproduce unrelated JS coercion or arbitrary metadata properties.
3. Retain fractional and out-of-gamut values through conversions and manipulation.
   Undefined hue/chroma and non-finite results have observable operation-specific
   behavior. Core math may use IEEE values; portable color scalars need an explicit
   finite/NaN/+infinity/-infinity representation. Never silently serialize NaN as
   JSON null or equate it with parse failure. This color-value contract does not
   relax DAT-05's requirement for finite painter coordinates.
4. Provide one conversion-to-paint operation: convert to RGB, apply the specified
   display fallback/clamp/rounding, then quantize alpha to the existing byte coverage.
   No intermediate byte roundtrip for Lab/HCL conversion or interpolation. Preserve
   the original authored value and space in definitions/semantic metadata. Existing
   byte paints retain exact meaning. Document final 8-bit quantization (at most half
   a byte step for finite in-range channels, alpha at most 1/510); color utility
   results and CSS strings retain their full specified precision. This is not an
   ICC, wide-gamut, CMYK or monitor-calibration claim.
5. Accept explicit CSS/color-value inputs at constant styles, theme tokens, palettes,
   annotations, rich text, candle colors, gradient stops and output backgrounds.
   Resolve once through the shared core engine. Mapped-color precedence, existing
   grayscale presentation policy, missing swatches and alpha compositing stay explicit.
   D3 gray(l) is a separate Lab constructor; do not silently change print themes.
6. CLR-01 records a conforming ADR and wire migration before changing shared types.
   Coordinate with active action/state schema work: preserve all v1 byte-color inputs,
   reject unknown variants/versions, and choose the smallest explicit versioned
   extension consistent with BND-01. Do not bump the wire version in this planning task.
7. WP-IP04 owns interpolation and consumes CLR-03 kernels; CLR-04 owns shared color-value
   descriptors and paint lowering. SP-01, WP-IP01 and CLR-01 share one development oracle lockfile
   and reference identity. Palette-only changes invalidate paint/legend caches but not
   stats or positional domains. Resolve parsing/conversion resources outside mark loops.
   Guide and gradient integration must use the same interpolation result as marks;
   non-sRGB gradients cannot be approximated by two endpoint sRGB stops without an
   explicit bounded-error representation and inspected evidence.

## Required operation catalog

CLR-01 turns this inventory into method-level fixtures, including inherited methods
and aliases rather than counting only documentation headings.

| API group | Required coverage and discriminating cases |
| --- | --- |
| color(specifier) | CSS names and aliases including rebeccapurple/transparent; 3/4/6/8-digit hex; comma RGB/RGBA integer or percentage forms; HSL/HSLA; case/whitespace normalization. Lock reference grammar for signs, exponents and malformed inputs. Invalid lengths/names, currentColor, CSS variables, modern space/slash forms and lab()/oklch() strings must not be accidentally accepted by the compatibility parser. |
| Constructors/conversions | Numeric channels with optional opacity; color and CSS input; same-space copies; RGB conversion from every space; direct Lab↔HCL conversion; hcl(h,c,l) versus lch(l,c,h); gray(l,opacity). Preserve opacity and undefined achromatic channels. Test typed construction separately from parsed zero-alpha input. |
| Manipulation | Read/update all channels and opacity; copy with typed overrides; brighter/darker with omitted, zero, positive, fractional and negative k; source remains unchanged. RGB scales channels; HSL/Cubehelix scale lightness; Lab/HCL add/subtract lightness. |
| Predicates and normalization | displayable on every space, including HSL's own rule; RGB clamp rounds channels, HSL clamp wraps hue and bounds saturation/lightness; opacity rules and non-finite fallbacks. Predicate must not mutate or clamp the source. |
| Output | formatHex, formatHex8, formatRgb, formatHsl, toString and deprecated hex behavior. Exact lowercase hex, punctuation, percent units, opaque/alpha form, rounding and numeric-string policy. Test repeated formatting without mutation and constructor→format→parse within the declared output precision. |

The pinned [RGB/HSL source](https://raw.githubusercontent.com/d3/d3-color/v3.1.0/src/color.js)
defines parsing, special values and boundary behavior. The pinned
[Cubehelix source](https://raw.githubusercontent.com/d3/d3-color/v3.1.0/src/cubehelix.js)
defines its conversion coefficients. The retrieved
[Lab/HCL source](https://raw.githubusercontent.com/d3/d3-color/main/src/lab.js)
uses D50 constants and direct cylindrical conversion; CLR-01 must verify these against
the 3.1.0 tarball before freezing fixtures (the tagged Lab URL could not be fetched
during this review). Source adaptation must retain the
[ISC notice](https://raw.githubusercontent.com/d3/d3-color/v3.1.0/LICENSE).

## Work packages and dependency order

| Package | Prerequisites | Requirements | Deliverables and exit evidence | Provisional effort |
| --- | --- | --- | --- | --- |
| CLR-01 — Contract and reference oracle | WP-14 | COL-01–06, ARC-04, BND-01, QLT-02 | ADR for typed values, exceptional channels, formatting, paint lowering and migration; pinned Node oracle and committed FIX-C01 cases with an executable Rust reader/comparator. Complete export/method disposition and expected-gap report; gap counts do not pass parity. | 2–3 days |
| CLR-02 — RGB/HSL, parsing and common operations | CLR-01 | COL-01–04 | Public core values, bounded parser/names, conversion/copy/brightness, predicates, clamp and common formatters. RGB/HSL method fixtures pass, including exact strings and exceptional states; no byte storage in math. | 3–5 days |
| CLR-03 — Lab, HCL/LCh and Cubehelix | CLR-02 | COL-02–04 | Shared D50/sRGB conversion and space-specific manipulation; all eight constructors and inherited operations pass independent and oracle cases. Direct same-space/Lab-HCL paths preserve information. This supplies WP-IP04 and SP-04's color kernels. | 3–5 days |
| CLR-04 — Authoring, portable operations and paint integration | CLR-03 | COL-05, BND-01/03/04, THM-01/02/03, SCN-03 | Versioned color descriptors, every paint input, one explicit lowering function, public standalone Python/WASM operations backed by Rust; actual host method/roundtrip tests, v1 migration, malformed descriptors and non-finite tags. Native/export consume the same resolved paint. | 3–5 days |
| CLR-05 — Integrated parity acceptance | CLR-04, SP-04, WP-20 | COL-01–06, QLT-02/03/04, SCN-04 | Complete FIX-C01 report, actual Rust/Python/WASM and native/SVG/PDF/PNG evidence, color-only updates versus fresh batch, coherent snapshots, measured parse/conversion/paint costs and API/support docs. G-COLOR passes only with no missing required behavior. | 2–4 days |

Total: **13–22 additional engineer-days**, provisional, excluding WP-IP04 interpolation,
SP-04 scale integration and final WP-21/22 hardening. Re-estimate
after CLR-01's exceptional-value and formatting experiments. This is execution order,
not an instruction to launch agents or interrupt the existing WP-15 assignment.

CLR-01 → CLR-02 → CLR-03 → CLR-04 → CLR-05. WP-IP04 additionally requires CLR-03; SP-04 consumes those shared kernels;
CLR-05 consumes SP-04 and WP-20, but never requires SP-07 or WP-21. WP-21/22
require CLR-05; WP-23 inherits them. This keeps the dependency graph acyclic and
allows standalone color work before interaction/streaming is complete.

## FIX-C01 acceptance matrix

Every case records operation, source space, input, expected output/diagnostic,
requirement, oracle identity, arithmetic/string tolerance and Rust/Python/WASM verdict.
Stored expectations run without Node/network; Node is only needed to regenerate the
reference fixture set. Failing, skipped or unsupported defined operations remain open.

| Case group | Independent expectations and required evidence |
| --- | --- |
| Parsing | Exhaustive reference name table and hex lengths, each functional grammar, whitespace/case, malformed strings and bounded oversized inputs. `#fea2` gives channels 255/238/170 and alpha 34/255. `transparent` retains undefined channels with zero opacity; invalid parse is a different result. |
| Floating RGB and HSL | `rgb(10%,20%,30%)` retains 25.5/51/76.5 before output. HSL 120°, 0.5, 0.2 converts to RGB 25.5/76.5/25.5. Cover hue wraps, achromatic endpoints, opacity outside [0,1], channel boundaries and copy isolation. |
| Perceptual spaces | D50 primary-color references, neutral Lab values, gray endpoints, LCh argument order, direct Lab↔HCL and RGB roundtrips; Cubehelix primaries, neutral/endpoint singularities and published coefficients. A D65 Lab implementation must not be mistaken for a matching D50 result. |
| Manipulation | k=0 identity; brightness defaults and negative/fractional k in each space; unclamped out-of-gamut results; opacity unchanged. Compare operation composition before any paint quantization. |
| Predicates/clamp/format | RGB -0.5 and just below it, 255.5 and just below it; half-integer rounding; hue ±360 and multiple turns; NaN/±infinity per field; transparent and zero-alpha parsed versus directly constructed values. Exact format strings including alpha 0.5 and exponent/negative-zero boundaries; legacy hex equals formatHex. |
| Authoring and hosts | Same CSS/value used in marks, theme/rich text, annotation, candles, gradients and export background resolves consistently. Every constructor/method runs in actual Python and Node WASM, including exceptional tags, invalid parse, independent copies and version rejection. No font/window required for standalone operations. |
| Integration and updates | Color change preserves prepared numeric data, domains, targets and dataset revisions; scene/definition revisions and paint caches update coherently. Include retained snapshots, mapped colors/legends, grayscale output and SP-04 perceptual ramps. Native/publication swatches and translucent overlaps are inspected on multiple backgrounds. |

Use exact equality for parse success/type, special-value tags, predicates, bytes and
format strings. For well-conditioned conversion channels begin with an absolute and
relative bound of 1e-10 and justify each operation against independently computed
anchors; hue comparisons use circular distance only when hue is defined. Cross-space
roundtrips need separate bounds derived from matrix/trigonometric conditioning.
Do not widen a global epsilon to mask incorrect white points or early quantization.
Floating CSS serialization needs a tested shared numeric formatter compatible with the
reference, rather than Rust/Python default string conversion. Rendering tolerances
separate declared sRGB8 quantization and compositing from mathematical color results.

Inspect actual native charts, SVG output, PDF and PNG/rasterized PDF; stored screenshots
alone cannot prove conversion correctness. Existing FIX-07/12/13/15/16 stay unchanged.
Add representative color conversion, palette preparation and repeated color-update
measurements to WP-22's existing performance protocol; no invented pass budget here.

## Evidence from this planning review

Environment: `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026). Base revision
`1cb955740c2dad2607b0a2330201125294cab5d0`; result is uncommitted documentation.

- Inspected specification/implementation plan/status and live color, theme, authoring,
  portable, native/export and existing test surfaces; compared official documentation
  and retrieved source. No executable D3 comparison was run.
- `mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`:
  **1 passed, 0 failed, 3 filtered out**. This proves only the existing subset.
- Documentation link, whitespace and requirement/dependency checks are recorded in
  the [status ledger](../implementation-status.md).

No implementation, dependencies, schemas or visual baselines changed. No fresh color
binding runtime, native/export artifact, Linux or performance gate was run. Tagged
Lab source retrieval remains CLR-01's reference verification task. All COL requirements
and G-COLOR remain open. Next package within this plan: **CLR-01**.
