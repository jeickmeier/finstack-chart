# ADR-017 — Shared interpolation and typed values

Date: 8 September 2026. Status: Accepted for WP-IP01–05.
Requirements: ITP-01–08. Reference: d3-interpolate 3.0.1, d3-color 3.1.0.

## Ownership and profile

`chart-core::interpolate` compiles endpoints and options once and samples explicit
finite `t` without clocks, I/O or host objects. Scales own normalization and domain
policies. Color owns parsing, conversion and formatting. Interpolation owns range
blending, scalar splines, transform decomposition and zoom trajectories. Existing
checked scale arithmetic and exact integer timestamps/IDs keep their contracts.

Numbers use binary64, including tagged NaN/infinities and signed zero. Missing and
null are distinct. Typed values include booleans, text, epoch-millisecond dates,
floating colors, Number-based numeric arrays, general arrays and string-keyed records.
Finite sampling parameters may extrapolate; nonfinite parameters reject explicitly.
Depth is limited to 32, aggregate value nodes to 200,000 and text/descriptor bytes to
4 MiB. Sampling checks the resulting value budget before returning owned output.

| Target | Source conversion and result |
| --- | --- |
| Missing, null, boolean | Constant target, independent of source. |
| Number | Number/date payload, boolean 0/1, null 0, missing NaN, or bounded numeric text; other kinds reject. Invalid numeric text becomes NaN. |
| Text | Recognized CSS target uses RGB interpolation; otherwise pair numeric tokens in target text. Primitive source text uses explicit ECMAScript spelling; colors use shared RGB formatting; dates and other structured source text reject. |
| Date | Number/date/primitive numeric conversion, then TimeClip at ±8.64e15 ms and truncation toward zero. Invalid date stays tagged. |
| Color | Color or parsed text source; unparseable text/missing source has undefined color channels. Other kinds reject. |
| Numeric array | Source numeric/general array or null/missing empty source. Preserve target type/length and untouched target tail; blend common prefix before destination casts. |
| General array | Source array or null/missing empty source; recursively dispatch common prefix and retain target tail. |
| Record | Source record or numeric-index fields of an array; other sources are empty; retain target keys, recursively dispatch shared keys, omit source-only keys. No prototype inheritance. |

Typed numeric arrays preserve Float32 rounding and Float64 values; Int8/16/32 and
Uint8/16/32 truncate and wrap; Uint8Clamped clamps and rounds nearest with even ties.
BigInt arrays, DataView, custom coercion/prototypes and arbitrary host callbacks are
outside this typed profile and reject at ingestion. They are not numeric-array parity.

Samples own their results, including nested values. An explicit destination overwrite
API may reuse capacity; earlier owned samples and quantized results stay independent.
Portable descriptors use strict versioned tagged kinds; no numerical NaN becomes JSON
null. Default generic dispatch recognizes a color string before numeric-token text.

## Per-operation behavior

Numbers/round and piecewise extrapolate; rounding matches JS ties toward positive
infinity, including negative zero. Open basis clamps and needs at least two controls;
closed basis wraps and accepts one or more controls. Discrete saturates and needs one
value. Piecewise needs two values and compiles each adjacent pair exactly once.
Quantize accepts integer counts 2–200,000 subject to aggregate output node/byte bounds, and samples both endpoints. Other upstream
empty/singleton outputs are retained in the corpus with named diagnostic adaptations.

Positive finite gamma is accepted for RGB and both Cubehelix routes; nonpositive or
nonfinite gamma rejects. Default gamma is one. Hue interpolation normalizes its output
and short/long routes retain the reference's distinct antipodal behavior. RGB splines
are opaque, sharing scalar splines without alpha interpolation. Quantization occurs
only on explicit formatting/paint conversion, never between color-space operations.

Transforms accept finite 2D matrices or bounded absolute CSS/SVG transform lists.
CSS absolute lengths/angles and SVG centered rotation are supported headlessly;
percentages/context-dependent values require an explicitly resolved matrix. Reject
malformed syntax, 3D/perspective and nonfinite matrices. Browser CSS length/angle and SVG parser rounding is measured separately from binary64 decomposition and string templates. Headless parsing retains authored binary64 precision; resolved-matrix templates and tokens are compared independently from browser-resolved geometry.
Reflections, singular inputs and shortest rotation retain source operation order.

Zoom requires finite centers and positive finite widths. Finite rho is floored at
1e-3 as upstream does. Signed reference duration remains available; a separately
named scheduling duration is the absolute reference duration. Nonfinite sampled views return a diagnostic
instead of entering scene geometry. This adds no animation scheduler.

## Reference and acceptance

The shared Node lock pins all 27 exports, three gamma factories, rho and duration.
[Numeric/value/color/zoom cases](../../fixtures/parity/d3-interpolate/cases.json)
are independent Node outputs captured before the next mutable reference sample.
[Transform cases](../../fixtures/parity/d3-interpolate/transforms.json) are generated
with DOMMatrix and SVG consolidation in Chromium 151.0.7922.34. Manifests retain
source and binary hashes, locale/timezone, platform and operation-specific tolerances.
Both generators regenerate byte for byte. D3/browser packages are development-only.

WP-IP02–05 qualify the corresponding operations; WP-IP06 supplies real public host
and chart consumers. WP-IP07 and G-INTERPOLATE require all cross-lane evidence. The
reference inventory alone does not pass them. Native custom factories are allowed;
portable custom operations require the existing versioned registry, and unregistered
closures fail serialization. Rendering, scheduling and disposal stay in their owners.
