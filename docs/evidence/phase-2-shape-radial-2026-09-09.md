# WP-S04 radial and link qualification complete

Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf` plus the retained Phase 2
working tree. SHP-05 and the radial/link portion of SHP-09 pass at the source/runtime
snapshots below. WP-S02/03 prerequisites are accepted. WP-S07/08 and G-SHAPE remain open;
this package does not establish general polar coordinates or hierarchy layout.

## Delivered behavior

`LineRadial` and `AreaRadial` reuse the shared materialized readers and all applicable
built-in curve protocols. The surface includes defined gaps, independent boundaries,
four boundary helpers, constants, curve parameters and precision controls. `Link`
uses the two-point line lifecycle; horizontal/vertical factories select BumpX/BumpY.
`LinkRadial` owns the reference radial tangent construction. Endpoint selectors accept
either endpoint or a constant row. Rust supports fallible native accessors and custom
curve factories; portable descriptors contain no executable callbacks.

The primary Rust/Python/WASM chart builders share the canonical compiler and version-7
shape descriptors. Radial radii use destination units around a projected data center;
Cartesian links interpolate projected endpoints. Both link endpoints retain one edge
identity. Source anchors remain separate from control points. Named numeric channels
accept source, statistical and literal values at their appropriate stages.

## Accepted evidence

- The independent pinned D3 corpus contains **40 point conversions and 697 paths**:
  seams, signed/zero radii, degeneracy, masks, boundary defaults, swapped/constant links,
  parameters and precision 0/3/12/unrounded. Regeneration repeats byte for byte. Rust,
  actual macOS/Linux Python and Node/WASM pass. Rounded SVG text is exact; six unrounded
  cases compare decimal coordinates under the predeclared `2e-12 * max(1, abs(expected))`
  tolerance. Topology/order are exact. No oracle values or tolerances changed.
- All **697 chart paths** pass in actual macOS/Linux Python and WASM, including keys
  above 2^53, endpoint selection, control exclusion, annulus holes, clipping, generated
  aggregates, facet budgets and versioned round trips. The six core integration tests
  also pass on macOS and Linux.
- Each actual host passes **760 append/upsert/remove/retention comparisons**, covering
  48 curve/parameter descriptors on applicable line/area routes and four link routes.
  Gaps, signed-radius corrections, logarithmic centers, facets, retained captures and
  exact identities are exercised. The macOS, Linux and WASM update records match exactly.
- Three themes at 300/600 DPI pass Rust/Python/WASM publication comparisons. PNG and PDF
  bytes and all nongeometric scene values agree exactly. Raw coordinate differences are
  retained, maximum `5.684341886080802e-14`. SVG serialization differs at those coordinates;
  independent SVG rasterizations agree byte for byte. All six macOS SVGs equal the
  previously rasterized WASM SVGs. PNG dimensions are 2500×3083 or 5000×6167.
- SVG/PDF/PNG output and [native output](phase-2-shape-radial/integrated/native.png) were
  inspected. All ten panels show the expected radial gaps, signed curve, annulus holes,
  Cartesian/radial links, source colors and legible labels. Native frame telemetry confirms
  actual painting. The task-owned native example was closed after capture.
- Strict TypeScript and mypy positive consumers pass; each host rejects all six intended
  negative consumers. WASM generation used matching CLI 0.2.128 and an actual Node runtime.
- The final geometry-budget revision passes **339 core + 54 export Linux tests/doctests**
  and macOS repository checks. The subsequent full macOS workspace run passes **404
  tests/doctests, zero failures/ignored**, including the initial independent-axis work
  that proceeded during the runtime startup wait. Current repository checks also pass.

Commands and logs are retained in [standalone evidence](phase-2-shape-radial/) and
[integrated evidence](phase-2-shape-radial/integrated/). The committed runners are
`scripts/bindings/shape_radial{,_interaction,_updates,_gallery}.{py,cjs}` and
`shape_radial_compare.py`; core tests are `shape_radial` and `shape_radial_integration`.
Repository gates were `mise run fmt`, `mise run check` and `mise run test`.
Linux used aarch64 `rust:1.97.1-bookworm`, read-only source/cached dependencies, the
separate task-owned target and disabled network. Source, runtime and artifact SHA-256
manifests identify each accepted stage.

## Corrected defects and validation boundaries

Facet preparation previously charged two vertices for an atomic shape path. A real
Python counterexample admitted two quarter sectors requiring ten commands/anchors with
a limit of seven. The compiler now rejects seven and accepts ten. Numeric literals
also incorrectly required source rows; literal evaluation and eligible scale training
now work over generated statistical rows while actual source reads retain stage checks.

Layout had a separate bypass: a five-command/anchor sector was admitted at a projection
limit of two, and two sectors at five. The shared `layout::work` preflight now charges
commands, anchors, all facet panels and inset replays against one figure-wide budget
before destination callbacks. [Budget evidence](phase-2-shape-radial/integrated/layout-budget/)
retains the counterexamples, exact-limit/one-below tests, zero callbacks on rejection,
and **eight actual adapter cases plus all 697 chart interactions** in freshly rebuilt
macOS/Linux Python and WASM modules.

The large path/update/publication proofs and native capture retain their own pre-final-
budget source/runtime identities; the budget guard changes work rejection and has its
own current-source runtime proofs. The final macOS workspace run additionally includes
the then-current axis identity changes. These are explicit validation stages, not a
claim that every artifact came from one identical binary. macOS startup stalled in the
system loader before chart execution and later resumed. Both normally linked and
separately re-signed modules resumed; no system security setting was changed and no
signing workaround is claimed as a demonstrated fix.

Next: WP-S07 registered custom-protocol/public-portability qualification, then WP-S08
integrated shape acceptance. All other Phase 2 requirements retain their ledger state.
