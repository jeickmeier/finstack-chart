# ADR-015 — Checked path authoring, retained geometry and destination replay

Date: 8 September 2026. Status: Accepted for WP-P01–04.
Requirements: PTH-01–06; shared foundation for SHP-01–10.

`chart_core::path` owns one checked builder and immutable numeric result. The pinned
d3-path 3.1.0 sequence semantics and `1e-6` branch predicates apply to finite inputs.
Source/license hashes, all constructors and all eight methods are retained in
`fixtures/parity/d3-path/`; JavaScript coercion and non-finite output are explicit
typed/safety adaptations. Non-finite arguments, intermediate overflow, negative radii
and exhausted limits reject atomically. A batch is one atomic mutation.

Authoring state is distinct from SVG/destination state. In particular implicit arc
starts do not initialize d3-path's remembered subpath start. Closing them may clear
the remembered current point although the serialized geometry has an active point.
Keep that observable distinction; do not repair serializer-only sequences such as a
bare `lineTo`. Destination validation rejects a missing initial move explicitly.

The retained command buffer includes analytic SVG circular arcs and signed relative
rectangle edges. Precision affects only serialization. `path()` and `Path::new()`
are unrounded; `path_round()` uses three digits. Explicit finite nonnegative digit
values are floored; values above 15 select unrounded output. Negative ties use the
reference's round-toward-positive-infinity rule. Negative zero serializes as zero.
Equivalent number spelling is compared numerically for trigonometric cases; canonical
rounding/scientific-notation cases have exact-string checks.

Owned operations commit a small pending command list only after checking input,
geometry and aggregate budgets. Immutable results own their commands. External sinks
receive borrowed commands in order; a sink error stops replay and may leave that
external sink with its accepted prefix. The core owned buffer and batch API remain
atomic. Command, operation, output-byte and destination-subdivision limits are explicit.

Existing `scene::PathCommand` validation and legacy publication defaults remain
unchanged. WP-P04 integrates retained path geometry through an explicitly versioned
destination route, preserving analytic arcs for SVG and bounded numerical lowering
for native paint. Repeated closes and continuation must preserve closed joins; an
unconditional move after close is not an equivalent stroke. Geometry transforms,
bounds, clips, text outlines and snapshot ownership must be validated together.
No SVG parsing is permitted in normal native painting.

Path construction supplies no source-row identities. A scene consumer carries caller
metadata without assigning source rows to controls or subdivisions. WP-S01 consumes
this implementation after WP-P04; it does not create a second builder or formatter.
New standalone operation envelopes have their own version history. Legacy chart and
scene envelopes cannot silently acquire an unsupported retained-path interpretation.

Retained f64 coordinates must survive JSON interchange exactly. The pinned serde_json
1.0.151 dependency enables its `float_roundtrip` feature (no version change or new dependency).
Without it, `45.235783453256964` decoded as `45.23578345325696`, and reconstructing an
identical affine path appeared to change the definition and advanced its revision.
The exact geometry round-trip regression retains the original expected control value.

Recomputing trigonometric geometry on another platform may differ by a few ULPs.
The operation-specific numeric tolerance applies to that computation; definition
identity remains exact. Cross-host reconstruction advances the revision when the
actual numeric definition changes, even if translated output coordinates coincide.
Same-host reconstruction and unchanged JSON interchange preserve exact identity.
Runtime proofs check these revision rules independently of tolerant geometry comparison.

WP-P01 fixes the contract and reproducible corpus. It does not close any runtime or
rendering gate. FIX-P01–06 and actual Python/WASM/native/publication evidence remain
required through WP-P04 before G-PATH can pass.
