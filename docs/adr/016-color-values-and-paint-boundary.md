# ADR-016 — Floating color values and the paint boundary

Date: 8 September 2026. Status: Accepted for CLR-01–04.
Requirements: COL-01–06, BND-01/03/04, ARC-04. Reference: d3-color 3.1.0.

## Values and operations

`chart-core::color` owns binary64 RGB, HSL, D50 Lab, HCL/LCh and Cubehelix values,
CSS parsing, conversions, brightness, predicates, clamping and formatting. The
reference's eight constructors have typed Rust equivalents and owned host values.
Only the pinned CSS grammar is accepted; parsing is bounded at ingestion. Parse
failure is distinct from a valid color with undefined channels. Same-space copies
and direct Lab/HCL conversions preserve the source information.

Public channel values remain floating point, including undefined/out-of-gamut values.
Portable finite channels are JSON numbers; exceptional channels use an explicit
`{"number":"NaN"}`, `{"number":"Infinity"}`, `{"number":"-Infinity"}` or
`{"number":"-0"}` tag. The shared binary64 codec also serves interpolation; signed
zero cannot be allowed to disappear during a host JSON round trip.
Unknown tags, extra fields and malformed descriptors reject. A standalone descriptor
has version 1 and a tagged color value. This does not change the finite coordinate
requirements or scene paint bytes. Equality treats two undefined channels as the same
retained value, while numerical operations keep IEEE behavior.

Floating CSS output follows ECMAScript number-string conventions; byte channels use
D3's half-up rounding and clamping. A dedicated test corpus distinguishes formatter
behavior from approximate channel-conversion comparisons. Hosts never implement color
math or stringify floating channels themselves. Source adaptations retain the ISC notice.
The core directly pins `libm = 0.2.16`, already present in the workspace lockfile, for
portable reference-compatible `atan2`. A reproduced macOS system-libm last-bit
difference changed two quantized Cubehelix paint channels in FIX-C01; using this
pure-Rust implementation fixes that case without relaxing expected strings or bytes.
The core also pins the dependency-free `ryu-js = 1.0.3` safe formatter
([upstream API](https://docs.rs/ryu-js/1.0.3/ryu_js/)). A fixture with opacity `2^-25`
reproduced a shortest-decimal tie where Rust's default printer disagrees with
ECMAScript. One shared formatter now serves color CSS and D3 path serialization;
no expected strings are normalized. Its Apache-2.0/BSL-1.0 licensing is allowed by
the existing dependency policy. The core also directly pins already-locked
`pxfm = 0.1.30` for the three conversion power operations. The WASM system power
implementation put the cube roots of two XYZ channels one ULP below the reference,
changing the exact HSL output after Lab conversion. The shared power implementation
passes that regression on native Rust, Python and WASM without changing fixtures.
Its BSD-3-Clause licensing is allowed by the existing dependency policy. Other
conversion arithmetic retains the source operation order and is qualified by the
full corpus on the required hosts.

## Authoring and publication migration

Existing `scene::Color` stays the unpremultiplied sRGB8 paint boundary. CLR-04 uses
an explicit authored paint input that accepts old byte colors or a versioned floating
color descriptor. Definitions retain the authored space/channels; a preparation step
resolves them once through the shared conversion-to-paint function. Prepared scenes
and native/export consumers use byte colors. The new definition capability
version is 4; older definitions keep their exact byte inputs and versions. Standalone
color operations do not require a chart, font, window or publication object.

Apply the same authored input at styles, themes, palettes, annotations, rich text,
candles, gradient stops and output backgrounds. New input descriptors must not be
silently discarded in primary/portable round trips or retained capture metadata.
Constant paint preparation stays outside per-mark rendering. Color-only edits preserve
numerical statistics/domains and invalidate the dependent paint/legend preparation.

Final in-range quantization error is at most half one RGB byte step and 1/510 in alpha.
Formatting/conversion operations retain the reference's precision before that boundary.
The existing grayscale print policy remains distinct from Lab `gray(l)`. Interpolation
and gamma/hue-path policies belong to WP-IP04; this module supplies their color kernels.
This decision makes no ICC, wide-gamut or monitor-calibration claim.

## Acceptance

The shared pinned Node workspace generates the [351-case oracle](../../fixtures/parity/d3-color/cases.json)
and [method/source manifest](../../fixtures/parity/d3-color/manifest.json), including all
148 names and explicit exceptional states. Rust runs the committed corpus offline.
CLR-02/03 qualify math and strings; CLR-04 qualifies all host/input routes; CLR-05 waits
for SP-04 and the full native/publication/update matrix. Fixture existence does not
close G-COLOR or missing operations.

## Implemented paint inputs and compatibility

`color::Paint` is an untagged input union of the exact legacy byte object and a
version-one `ColorDescriptor`. CSS is parsed at ingestion into that union. Existing
six/eight-digit hex syntax retains byte semantics; other supported CSS forms retain
the parsed floating value. Unknown fields, malformed channels and unsupported
versions reject. Primary Python/WASM encoders serialize owned colors as descriptors,
including nested palettes, text and gradients; disposed owners reject immediately.

Authoring uses `Style<Paint>`, `ThemePatch<Paint>`, `GeometryTheme<Paint>`,
`CandleColors<Paint>` and `ColorScale<Paint>`. Generic structs default to the legacy
byte type for independent Rust use. Fluent setters accept `Into<Paint>`; direct
assignments to raw definition fields use `.into()` for byte values. Prepared marks,
resolved theme caches, glyph paint, vector primitives and native/export rendering
still contain `scene::Color`. Reusable `map_color(s)` methods adapt the color parameter
without copying a second set of structure fields into a competing schema.

A floating continuous palette compiles shared RGB interpolators before its row loop;
guide stops sample those same interpolators. Fully byte-authored palettes retain their
previous operation order and rounding. Additional interpolation routes and normalized
scale families remain SP-04's work. Palette and constant-color edits retain numerical
tables and positional domains, rebuilding dependent mark/guide paint.

Primary and normalized definitions use capability version four whenever any retained
paint is floating, including rich axis/furniture text and post-scale literals. Existing
theme geometry and path-composition subversions keep their independent meanings.
Publication profiles and live-export Begin overrides use version two when carrying
floating presentation paints; profile background is a new optional version-two field.
Retained requests and metadata preserve the authored descriptors after owner disposal.

The signed-zero reference correction preserves all 351 case identities and algorithms.
The previous oracle serialized negative zero as zero, including its nominal signed-zero
inputs. It now classifies the sign before JSON serialization and reconstructs it before
calling D3. The stronger comparison also requires positive-zero alpha/unit clamps
where JavaScript `Math.max` and `value || 0` prescribe them; reference values and
operation-specific tolerances are not normalized to hide that distinction.
