# SP-02 numeric mapping and named-axis integration

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 changes.
SP-02 is COMPLETE for its SCL-01/02/03/06/07 and DAT-05 package scope.
[Source hashes](phase-2-numeric-scales/source-hashes.json) identify the qualified
snapshot. SP-03–07 and G-SCALE remain OPEN.

The shared scale module now provides immutable numeric descriptors and prepared
piecewise mapping/inversion. Linear, signed power/sqrt, negative-domain log,
symlog, identity and signed radial behavior preserve knots, reversed domains,
constant/repeated endpoints, rounding, unknown values and copies. Finite base values
below one and exponent zero/negative are evaluated per operation; undefined results
are tagged or diagnosed without inventing finite outputs. Explicit `nice(count)`
edits domain ends, retaining interior knots. A shared ECMAScript power boundary fixes
unit bases with infinite exponents, exposed by exponent-zero inverse fixtures.

Existing finite linear recipes delegate to the shared checked legacy mapping helper
without adding per-axis allocations. Their domain expansion, tick grid, precision
handling and unbounded navigation inverse remain unchanged. Compatible named numeric
axes retain full domain knots and compose the same mapping with destination and
viewport projection. A repeated initial knot retains its unused initial output
interval; constant output spans map to the destination midpoint. Plot and normalized
definitions carrying these axes require version five; old envelopes reject them.

| Evidence | Result |
| --- | --- |
| Numeric reference | All 161 numeric-range configurations in the frozen corpus pass map/inverse and five explicit nice counts each, plus descriptor round trips. Remaining color/typed-range configurations are assigned to SP-04. Typed null-to-number coercion is excluded; identity unknown inverses use a typed result. [Core](phase-2-numeric-scales/core.log), [nice](phase-2-numeric-scales/nice.log). |
| Independent math | Piecewise midpoint, radial area/radius relation, inverse clamp, unbounded inverse, copy isolation, bad ordering and version rejection pass. Same focused core log. |
| Named axes/publication | Nine independently specified configurations cover every numeric family, constant domains and repeated knots. Shared axis values match scene point positions and SVG exports; v5 round trips and v4 capability rejection pass. [Log](phase-2-numeric-scales/export.log). This is numerical/publication execution, not inspected visual certification. |
| Navigation | New pointer-anchored piecewise zoom maps [0,10,100] through output [0,50,100], yielding viewport [5,55] while retaining the data value 10 at the pointer. Clamped inverse and original domains remain coherent. All six navigation tests pass. [Log](phase-2-numeric-scales/navigation.log). |
| Regression | Nine foundational scales, four existing full-scales, 21 authoring/host-dispatch/portable and the focused new cases pass: 43 macOS tests total across the listed runs. [Core](phase-2-numeric-scales/core.log), [authoring](phase-2-numeric-scales/authoring.log). |
| Repository | `mise run check`, rustfmt and whitespace checks pass, including all-target Clippy, rustdoc and core WASM compilation. [Check](phase-2-numeric-scales/check.log). |
| Linux | Actual Rust 1.97.1 container, offline with read-only sources, passes 20 numeric/legacy-scale/color/interpolation tests. [Log](phase-2-numeric-scales/linux.log). |

Full D3 candidate ticks/specifier formatting remain SP-05; current new-axis guide
candidates use the declared existing bounded grid. Before-stat grammar profiles must
explicitly choose `coordinate_scale` for these numeric knot axes; GG-04 owns their
shared pre-stat integration and silent stage omission rejects. No actual standalone
Python/WASM numeric method proof, native capture, complete visual acceptance, aggregate
Linux gate or performance budget is claimed. Those remain SP-07/WP-21/22. Next: SP-03
ordinal, band and point semantics, sharing prepared spacing with legacy projection.
