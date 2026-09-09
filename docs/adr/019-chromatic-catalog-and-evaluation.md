# ADR-019: named chromatic tables and evaluation

Status: accepted for Phase 2 implementation, 9 September 2026.
Requirements: CHR-01–06, ARC-04, BND-01. Owner: CP-01–05.

The [chromatic plan](../impl_plans/d3-scale-chromatic-parity-plan.md) requires the
complete d3-scale-chromatic 3.1.0 surface: 38 scheme exports / 218 actual arrays and
38 interpolators. Its pinned source, d3-color 3.1.0 and d3-interpolate 3.0.1 share the
existing development oracle lockfile. Source hashes, original CSS, RGBA expectations
and D3/ColorBrewer notices are retained. These are development references and add no
production dependency or I/O requirement.

`chart-core::scales::chromatic` owns checked scheme/interpolator IDs, static scheme
and lookup tables, metadata and catalog-specific recipes. Sized Brewer tables stay
independent; neither truncation nor ramp sampling may manufacture a named size.
Scheme queries return owned arrays. The continuous evaluator prepares immutable
shared interpolation resources once. Brewer curves use the existing RGB basis;
Cubehelix recipes use the existing long-hue interpolator and color conversion.
Lookup/polynomial/cyclic kernels live only in this catalog owner. No alternate
color or scale math is added to bindings.

A scheme query specifies a typed ID, optional size and reversal. Fixed schemes
require no size; Brewer schemes require one of their exact recorded sizes. Unknown
IDs and mismatched sizes diagnose. A version-one chromatic ramp descriptor contains
its typed interpolator ID and reversal; the version pins catalog meaning. Evaluation
accepts finite binary64 t, including outside [0,1], and applies each reference
function's own clamp/wrap behavior. Reverse means evaluate at 1-t, separately from
descending scale domains. Non-finite direct input diagnoses; chart null/NaN/Inf
continues through the scale's missing policy. Reference invalid outcomes remain
explicit evidence, not fabricated colors.

The existing interpolator descriptor gains one chromatic variant consumed by all
scale families, marks and guides. This reference is immutable and contains no host
objects. The interpolation and chromatic modules may call one another through their
checked constructors, but compiled chromatic recipes only use existing non-chromatic
primitives, so evaluation cannot recurse. Shared scale training/normalization remains
in SP-04. Mapped legend identity already retains the full descriptor. Accurate labeled
sampled swatches are sufficient; continuous strips would need separate bounded-error
and lookup-jump evidence.

Plot definitions containing the new descriptor require wire v6; existing v1–v5 input
meaning remains unchanged. Standalone interpolation descriptors retain version one
with an additional checked variant, as with explicit tagged capability extension;
older decoders reject unknown variants. Standalone chromatic operations have their
own version-one payload and reject unknown versions. Python/WASM expose catalog,
query, evaluate, serialization and owned-copy behavior by calling the same core.

The acceptance comparator uses exact final RGBA, preserving original reference CSS
without requiring its spelling as the public chromatic return type. Dense samples,
adjacent basis/lookup knots, analytically located byte transitions, very large finite
inputs, reverse and explicit non-finite diagnostics all have independent oracle rows.
No global one-byte waiver is permitted. Grayscale is presentation after canonical
color evaluation and cannot change these expectations. CP-05 adds actual hosts,
charts, updates, native/publication inspection and performance measurements; CP-01's
inventory and expected-gap checks alone cannot pass G-CHROMATIC.

## Implemented integration

Named discrete palettes retain checked canonical values plus `SchemeSpec` metadata;
this preserves identity without a second range evaluator. Named continuous output
reuses the owned interpolator variant. Guide equality includes the full mapping even
when a reversed cyclic ramp has identical endpoint swatches. The chart missing-value
policy also excludes non-finite numeric ordinal keys from eligible training; exact
finite keys and unnamed legacy behavior remain intact. Public host tests cover the
complete catalog and composition matrix, including typed Python float cutpoints,
shared bounded sampling, theme conversion and retained updates.

Finite source values can normalize to infinity or NaN under log/power transforms.
The internal scale evaluator preserves each reference recipe at those parameters;
the public ramp evaluator still rejects non-finite inputs. A valid reference color
is retained, including saturated lookup endpoints. Undefined reference strings from
NaN lookup, Cividis or Turbo results diagnose instead of manufacturing a color.
The separate 304-case transformed oracle fixes this distinction through standalone
and actual chart scenes; source NaN/Inf values still follow missing-paint policy.
