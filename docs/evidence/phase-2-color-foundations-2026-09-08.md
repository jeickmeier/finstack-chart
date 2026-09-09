# CLR-01–03 color foundations

Date: 8 September 2026. Revision: `fab2505` plus the uncommitted Phase 2 work;
[source hashes](phase-2-color-foundations/source-hashes.json) identify this slice.
Requirements: COL-01–04 / FIX-C01. **CLR-01, CLR-02 and CLR-03 COMPLETE** for
reference and standalone parsing/math/formatting scope. CLR-04 paint integration,
CLR-05 and G-COLOR remain open.

The [351-case pinned corpus](../../fixtures/parity/d3-color/cases.json) covers all
148 CSS names, all eight constructors/overloads, five color models, conversions,
immutable copies, 18 brightness operations per valid input, all common formatters,
predicates, clamps and explicit exceptional channels. Separate regeneration matches
all three fixture files byte for byte. Core uses the exact pinned grammar and D50
coefficients, bounded parsing and strict versioned descriptors. Existing byte paints
keep their meaning; the standalone floating object is not yet accepted at every
chart paint input. [ADR-016](../adr/016-color-values-and-paint-boundary.md) records it.

| Evidence | Result |
| --- | --- |
| Rust offline corpus and independent RGB/HSL/D50/paint/grammar anchors | Three tests pass, including every matrix operation and exact paint hex8. [Log](phase-2-color-foundations/color-rust.log). |
| Actual Python and WASM | Both execute all 351 cases, matching numeric channels at the declared 1e-10 absolute/relative bound and strings, tags, predicates and bytes exactly. Strict malformed descriptors, independent copies and disposal pass. [Python results](phase-2-color-foundations/python.json), [WASM results](phase-2-color-foundations/wasm.json). |
| Primary authoring/runtime types | Entire existing primary proof plus 13 GG-02 figures per host, new color constructors/methods, strict Python/TypeScript consumers and five intentional invalid type cases pass. [Log](phase-2-color-foundations/primary.log), [environment](phase-2-color-foundations/environment.json). |
| Repository / macOS | `mise run check` and `mise run test` pass: 279 tests in 40 nonempty suites, zero ignored. [Check](phase-2-color-foundations/check.log), [tests](phase-2-color-foundations/tests.log), [counts](phase-2-color-foundations/test-counts.json). This run preceded interpolation implementation. |
| Linux | Actual Rust 1.97.1 container, networking disabled, read-only source/registry: three color and three existing path oracle tests pass after the shared power fix. [Log](phase-2-color-foundations/linux.log). This is a focused run, not a fresh aggregate Linux gate. |

Three discriminating failures were fixed without changing expected results: system
`atan2` changed Cubehelix paint bytes on macOS; default Rust number formatting chose
a different shortest decimal at `2^-25`; WASM power rounding changed HSL formatting
after Lab conversion. The shared implementation now pins already-locked `libm` and
`pxfm`, plus dependency-free `ryu-js` 1.0.3. One ECMAScript formatter serves colors
and paths; license/dependency checks pass. This is corpus-qualified platform behavior,
not a proof that every transcendental result is identical on every input or platform.

No new native window or color paint publication was claimed for these standalone
packages. Those require CLR-04/05 and SP-04. Broader color/scale/chromatic/ggplot2 and
performance gates remain open. Next: interpolation foundations and the complete
authored color/paint boundary, followed by scale and integrated acceptance packages.
