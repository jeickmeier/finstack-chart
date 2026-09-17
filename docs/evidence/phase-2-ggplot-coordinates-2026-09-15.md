# GG-13 coordinate implementation evidence — 2026-09-15

Status: implementation and focused qualification complete; combined host and repository acceptance remain pending; native visual/presentation qualification is recorded below. This report does not close GG-13 or a global parity gate. The root integration owner maintains the ledger gate.

## Contract and shared ownership

The optional definition-level `CoordinateSpec` leaves legacy definitions unchanged. Cartesian views implement typed limits, independent expansion edges, flips, reversal and fixed ratios. Transformed views use existing `GgplotTransform` descriptors after statistics and retain expansion in transformed space. Polar and radial policies share one resolved map while retaining their distinct expansion, aspect, direction/reversal, clipping and guide rules. New coordinate authoring selects definition version 78; warped raster hit coverage selects scene version 22.

`coordinate_resolve` reads existing axis calculation spaces. Timestamp limits subtract integer origins before floating conversion and do not undergo viewport censoring; session limits retain the supplied calendar's closed-session policy. Runtime axis windows supersede authored coordinate views. No coordinate operation changes statistical populations or scale training.

`coordinate_path` supplies bounded destination subdivision using transformed control enclosures, carried analytic-arc lowering error, pinned libm and explicit work/depth limits. Physical point dimensions, stem widths and arrow lengths are resolved in destination units. Shared preclipping lowers radial annulus/sector geometry before publication, native paint and inspection; interior contours use bounded geometric tests to avoid quadratic clipping. Polar retains rectangular clipping as in the pinned source.

The same map serves public `Coordinates`, composition data anchors, custom `HitGeometry::Path`, pointer inversion and navigation. Radial center/overlapping branches expose unavailable/ambiguous inversion; rectangular radial pan/region gestures are explicitly unsupported. Anchored radial zoom requires a unique inverse. A provider without retained coordinate-space endpoints and backend-native opaque paint are explicit unsupported capabilities rather than guessed mappings.

Root-owned raster sampling inverse-maps original cells, uses bounded nearest/bilinear premultiplied-alpha sampling, and retains per-target output coverage. The inner hole and invalid sector pixels are absent from both paint and inspection. Root-owned guide selection uses the existing typed tick engine over the coordinate view; angular/radial placement, aspect and theme-grid integration are shared core code. Secondary guides retain primary scale identity.

## Independent source evidence

- `tools/reference/r/coordinate-controls.R` → `fixtures/parity/ggplot2/coordinate-controls.json`: 24 independently built, projected, munched and drawn cases under R 4.6.1 / ggplot2 4.0.3. Original source capture: `/private/tmp/gg13-coordinate-reference.log`.
- `tools/reference/r/coordinate-guide-controls.R` → `fixtures/parity/ggplot2/coordinate-guide-controls.json`: nine source guide cases retaining seam labels, primary/secondary radii, partial panels, angle controls and actual draw outcomes.
- Source clip policy and dotplot capability probes are retained in `/private/tmp/gg13-polar-clip.log` and `/private/tmp/gg13-dot-coord.log`.
- Read-only prior inventory: `docs/evidence/gg13-coordinate-inventory-2026-09-15.md`.

## Focused execution

Commands run from the repository with the existing locked toolchain:

```sh
mise exec -- cargo test -p chart-core --lib coordinate --locked
mise exec -- cargo test -p chart-core --test ggplot_coordinate_controls --test ggplot_coordinate_guides --test ggplot_coordinate_raster --locked
mise exec -- cargo run -p chart-export --example ggplot_coordinate_controls --locked -- /private/tmp/gg13-rust-publications
mise exec -- cargo clippy -p chart-core --lib --locked -- -D warnings
```

- Coordinate-filtered library tests: **16 pass**, `/private/tmp/gg13-coordinate-monotone2.log`. Includes all 24 source projections, full/multiple-turn subdivision, work limits, raster coverage, compound custom hits through full/partial annuli, runtime-window precedence, exact 17-nanosecond timestamp limits outside a censored viewport, and rejection of a discontinuous reciprocal branch. Two filtered tests predate GG-13.
- Public coordinate controls: **5 pass**; guides: **3 pass**; raster inspection: **1 pass**, `/private/tmp/gg13-public-final.log`. Includes actual shared layout, immutable replay, physical arrow length, guide roles, fixed-aspect/free-facet rejection and unique inverse navigation.
- Independent Rust authors: **14 pass**, **42 SVG/PDF/PNG publications**, original/replay scene JSON exact equality; `/private/tmp/gg13-rust-publications7.log`.
- Core Clippy reported no coordinate findings, but failed on seven model-owner lints; `/private/tmp/gg13-clippy-final.log`. Those findings were handed to the integration owner; this is not a passing repository check.

The publication authors are `examples/common/ggplot_coordinate_controls.rs`, with independent mirrored Python/WASM scripts `scripts/bindings/ggplot_coordinate_controls.py` and `.cjs`. Cases 0–5 cover full/partial angular guides, reversal, explicit label angles, secondary inner guides and fixed ratios. Cases 6–13 cover flipped intervals, sqrt-transformed arrows, polar columns, clipped annulus lines/points, warped raster cells, partial-sector gradient panels, coordinate zoom over count statistics, and flipped/reversed categorical positions.

All 14 PNG outputs, all 14 actual PDF pages rendered through Poppler, and all 14 SVG files independently rendered through macOS Quick Look were visually inspected. Contact sheets: `/private/tmp/gg13-coordinate-contact.png`, `/private/tmp/gg13-coordinate-pdf-contact.png`, and `/private/tmp/gg13-coordinate-svg-contact.png`. Files are `/private/tmp/gg13-rust-publications/coordinate-{0..13}.{svg,pdf,png}`; independent SVG preview log: `/private/tmp/gg13-svg-preview.log`.

## Remaining acceptance work

The native gallery `ggplot_coordinate_controls_native` accepts `GG13_MODES` as comma-separated indices; planned batches are 0–5, 6–11 and 12–13. Fresh actual Python/WASM module execution, cross-host publication comparison, declarations consumers and the shared repository/workspace gates remain with the coordinated integration run. Native inspection is complete as recorded below. Distribution-dot compatibility is qualified by `/private/tmp/gg13-dot-correction.log`: Cartesian coordinates map bin/center positions before building physical circles and stacks, including flip; nonlinear coordinates explicitly reject the combination disclaimed by the pinned upstream implementation. All seven model lint fixes are now reported by their owner; aggregate checks remain pending. No Linux or global certification is claimed here.

## Native qualification and model inspection correction — 2026-09-16

The direct executable workflow (without the stalled app wrapper) built both galleries with:

```sh
mise exec -- cargo build -p chart-gallery --example ggplot_coordinate_controls_native --example ggplot_model_controls_native --locked
```

Corrected build log: `/private/tmp/gg13-gg10-native-corrected-build.log`. All 14 coordinate modes have actual non-null presented stamps. Screenshots were captured with the installed screenshot helper and individually inspected:

- Modes 0–5: `/private/tmp/gg13-native-0-5.png` and `.log`.
- Modes 6–9: `/private/tmp/gg13-native-6-11.png` and `.log`; the failed mode11 in this initial screenshot is diagnostic evidence, not acceptance.
- Corrected modes 10–11: `/private/tmp/gg13-native-10-11-corrected.png` and `.log`.
- Modes 12–13: `/private/tmp/gg13-native-12-13.png` and `.log`.
- Machine-readable presented stamps: `/private/tmp/gg13-native-presented-evidence.json`.

Mode11 initially exceeded the unchanged image-work budget because sampling multiplied the two-sample baseline by Retina device scale. The shared raster owner corrected this to `max(2, device_scale)` and qualified 480×320 at DPR2 under the existing budget. The corrected image rendered correctly, but the native adapter's per-pixel quad expansion took about 24.6 seconds for its first frame. The native adapter owner corrected this with exact-color horizontal/vertical rectangle coalescing and transparent-pixel omission; two tests pass, including exhaustive reconstruction of 19,683 mask/alpha combinations and a 600,000-cell sector represented by 448 rectangles (`/private/tmp/gg13-native-raster-coalescing.log`). Final modes10/11 were rebuilt and inspected at `/private/tmp/gg13-native-10-11-coalesced.png` and `.log`: both have non-null stamps and null diagnostics. Observed debug first-frame interval improved from 24,629 ms to 948 ms; subsequent recorded frames were 684/694/724 ms versus 24,338 ms before. These are observed diagnostic intervals under concurrent development load, not benchmark or 60 Hz certification. The exact-color correction does not alter shared scene pixels or export/host arithmetic. All newly owned windows were closed.

The six model panels (0,1,2,3,7,8) initially painted successfully but could not construct inspection: `Shape source-anchor and target counts differ.` The common BandRun projection mirrors prediction targets along both sides of a ribbon; independent paint conversion incorrectly used the original unmirrored count when generating path anchors. `layout/project.rs` now passes the actual mirrored target count for this branch. Original model provenance is unchanged. `gpui-charts/view.rs` now records inspector construction errors instead of silently discarding them, and the gallery probe records diagnostics.

The public regression in `ggplot_model_authors.rs` creates the six native-sized frames and requires successful inspection, exactly 41 keyboard prediction identities per model, and original-target membership for every inspection result. It passes in `/private/tmp/gg10-model-inspection4.log`. The rebuilt six-panel native model gallery passes visual inspection with non-null stamps and no diagnostics: `/private/tmp/gg10-native-models-corrected.png` and `.log`. All three stale root model processes and all newly owned gallery processes were closed. Fresh cross-host model outputs must include the corrected path-anchor arrays.
