# WP-AX06 integrated certification — 9 September 2026

Status: COMPLETE for AXIS-01–07/FIX-19 and G-AXIS, within the declared platform/profile boundary.
Revision: working tree over `a6caa39`. The ledger remains authoritative for gate status.

## Capability verdict matrix

The reference is d3-axis 3.0.0 with the locked shared scale/format/time/selection/transition
packages. [Inventory](../../fixtures/axes/inventory.json) records all four constructors,
all ten methods and both device-scale defaults. [Static provenance](../../fixtures/axes/manifest.json)
and [timed provenance](../../fixtures/axes/transitions-manifest.json) retain source hashes,
licenses, environment and generator identity. Normal Rust tests read stored expectations.

| Requirement / case | Implementation and independent acceptance | Verdict |
| --- | --- | --- |
| AXIS-01 / FIX-19-A | Separate GuideId/ScaleId; shared retained providers, four sides, translation/replacement, copied configuration/reset and old-definition migrations. Core `axis_guides`/`axis_providers`; actual registered provider authoring and retained exports. [Entry](phase-2-axis-entry-2026-09-09.md), [provider](phase-2-axis-provider-2026-09-09.md). | PASS |
| AXIS-02 / FIX-19-B | Shared tick resolver preserves explicit order/duplicates/empty selection and bypasses enumeration. Arguments continue to affect default formatting. Count hints are separate from hard budgets. [Tick evidence](phase-2-axis-ticks-2026-09-09.md). | PASS |
| AXIS-02/03 / FIX-19-C | 372 pinned cases / 376 states include applicable numeric families, negative/reversed/degenerate domains, fractional count hints, log variants, blank minor labels and symlog. Actual Python/WASM independently replay the stored corpus; Rust offline tests use the same expected records. | PASS |
| AXIS-03 / FIX-19-D | Independent formatting/reset, SI/grouped/percent/scientific, supplied locale, registered original-value/index/list callbacks, repeated/empty labels and explicit errors. Native-only registrations cannot serialize. Tick/provider tests and three-host scripts retain original semantics. | PASS |
| AXIS-02/03 / FIX-19-E | Shared UTC/supplied-zone calendar ticks include DST gaps/folds and interval boundaries; band/point fallback, padding/rounding/reversed ranges; exact integer source timestamps beyond 2^53 have independent assertions outside D3's time range. [Tick evidence](phase-2-axis-ticks-2026-09-09.md). | PASS |
| AXIS-04 / FIX-19-F | All sides, signed sizes/padding, independent caps, translation, range endpoints, centering and device-scale offset. Numeric reference comparison plus inspected native/SVG/PDF/PNG. [Geometry evidence](phase-2-axis-geometry-2026-09-09.md). | PASS |
| AXIS-04/05 / FIX-19-G | Explicit preserve/hide/thin policy, bounds/budgets, independent component styles and rich labels, empty decoration targets, facets/insets/secondary units. Existing scene/text/paint paths; 26 matching host artifacts and inspected text/outline 300/600 DPI publications. [Component evidence](phase-2-axis-components-2026-09-09.md). | PASS |
| AXIS-06 / FIX-19-H | 41 D3 timed cases / 125 samples cover joins/order/duplicates, enter/update/exit, caps/offset/side, interruption and exact final layout. Native clock, reduced motion, disposal and retained displayed export; explicit three-host sampling/owner/stale-state checks. [Transition evidence](phase-2-axis-transitions-2026-09-09.md). | PASS |
| AXIS-07 / FIX-19-I | Actual Rust/Python/WASM execution and inspected native macOS device-scale two; reference and portable geometry at scales one/two. SVG text/outline components, vector PDF and 300/600 DPI PNG. Fresh supported headless Linux and repository qualification pass. | PASS |

## Declared compatibility and platform boundary

LibraryV1 remains the default; D3_3_0_0 is an explicit per-guide profile. Primary API
versions 8/10/11/13/14 introduce independent guides/providers/ticks/geometry/styles;
older definitions still decode and unconfigured definitions retain earlier envelopes.
Scene v15 adds optional sampled lifecycle identity/opacity. Checked typed values and
registered native operations replace JavaScript coercion and arbitrary interpreter
callbacks. The legacy session calendar scale has no D3 counterpart. Secondary-unit
conversions retain their checked policy; they are not an alternate D3 scale family.

Native inspection used the available macOS Retina display (scale two). No scale-one
physical native display or Linux native renderer is certified. Both scale-factor
policies execute in reference/core/portable proofs; Linux support is headless core/export.
Outline output exposes logical roles/labels while text is represented by positioned
paths. Native primitive alpha and SVG/PDF group opacity retain continuous semantic
opacity; their overlap compositing and OS text rasterization are renderer specific.
This certification does not claim identical pixels across renderers.

## Final qualification

- **449 macOS workspace and 448 Linux headless tests/doctests pass**, zero failures.
  Full `mise run check` passes, including strict lint/docs and standalone WASM core.
  Logs and source/runtime identities are in the [transition evidence](phase-2-axis-transitions/).
- Fresh actual Rust/Python/WASM replay passes independent guides, registered providers,
  all 372 static reference cases / 376 states, component styling and transitions.
  [Six provider/tick publications](phase-2-axis-certification/provider-tick-comparison.json),
  [26 component artifacts](phase-2-axis-certification/axis_components/host-comparison.json)
  and [75 transition artifacts](phase-2-axis-transitions/host-comparison.json) agree.
  JSON tolerances remain operation-specific and labels/identities exact.
- Cumulative comparison caught process-dependent guide IDs in the older tick example.
  Its independently authored inputs now assign the same two explicit guide IDs before
  rendering in each host. No expected geometry, tolerance or visual baseline changed;
  exact SVG/PDF/PNG comparison passes. Final fixture formatting and focused lint pass.
- The interpolation consumer replay passes [18 three-host comparisons](phase-2-axis-certification/interpolation_integration/host-comparison.json),
  plus updated-vs-fresh/retained-owner cases. Axis plans use shared scalar/transform
  interpolation; the native trace passes interruption, reduced motion, disposal and
  exact acquisition-versus-exported geometry. This fulfills the WP-IP07 axis prerequisite.

Repository checks initially exposed two unescaped intervals in new rustdoc; the corrected
complete run passes. Native artifacts were visually inspected, including independently
rasterized SVG/PDF and the moving scene exported after disposal. Native clock submission
counts are not FPS certification. Expanded WP-21/22/23 and global G-PARITY/G4 remain open.
