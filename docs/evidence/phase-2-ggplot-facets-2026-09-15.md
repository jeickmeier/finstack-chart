# GG-12 facet semantics and layout qualification

Status: implemented shared core and host authoring controls; qualification in progress. This is not package acceptance. Working tree based on `96d044d82654184aad2806a5b5ef01847d5e7678`, with concurrent GG09 changes retained. Root coordinates final broad checks and package gate.

## Contract and ownership

`FacetSpec.reference: Option<FacetPolicy>` selects reference catalog, population and furniture behavior. New `Profile::Ggplot2_4_0_3` authors infer the default policy; deserialized definitions with no policy preserve legacy explicit catalogs, broadcast requirements and independent free-grid panels. Explicit `.reference(policy)` selects facet semantics without changing unrelated chart/stat profiles. Definition wire capability is 76 (root-owned detection).

One bounded live source planner supplies existing panel compilation: multiple wrap variables; nested tuples within grid sides and crossing across sides; explicit level catalogs and drop behavior; missing identities; typed margin wildcards distinct from literal `(all)`; partial missing-field broadcast by canonical field name; live source updates; explicit catalog locking through `.order`. Complete/partial tuple sets are bounded to 256 rather than storing every source row. Existing original source row targets are retained in each panel/margin.

Free grid x populations share columns and y populations share rows. The same identities feed positional limit/OOB callback populations, automatic bin scopes and final domain merging. `shrink=false` adds bounded pre-stat position contributions without painting source rows. The shared stat pipeline and scale projection readers remain the calculation owners.

Layout consumes the existing shared solver and measurer: eight wrap direction encodings, grid table order reversal, proportional expanded-domain space, wrap free-space normalization diagnostics, four wrap strip sides, switched grid strips, independent interior axes/labels, value/both/context/lookup/wrapped labellers. Registered strip labellers reuse installed `CustomGuideFormatter` vector registrations: logical category values (missing stays `MissingCategory`) and canonical variable names, checked cardinality/bounds/portability. Typed PanelKey identity remains independent of displayed text. Parsed mathematical labellers remain GG14-owned.

## Independent source

`tools/reference/r/facet-controls.R` generated `fixtures/parity/ggplot2/facet-controls.json` using local R 4.6.1 / ggplot2 4.0.3. All 38 cases completed both build and draw on an 8×6 inch PDF device. Fixture records catalogs, membership, scale-sharing IDs, expanded ranges, strip/axis labels and layout units. Source log: `/private/tmp/gg12-facet-reference.log`. See the earlier [inventory](gg12-facet-inventory-2026-09-15.md) for exact independent observations and source command.

`tools/reference/r/facet-edge-controls.R` adds seven successful pinned build/draw cases in `fixtures/parity/ggplot2/facet-edge-controls.json`: four grid switch settings, reversed table order, partial-data new levels, and numeric facet variables. Log: `/private/tmp/gg12-edge-reference.log`. The gtable positions independently establish that switched strips sit between the plot and exterior axes.

## Focused verification

- `mise exec -- cargo test -p chart-core --test ggplot_facet_controls --test facets --locked`: 12 new controls and nine preserved legacy facet tests passed; `/private/tmp/gg12-facet-focused.log`.
- Final `mise exec -- cargo test -p chart-core --test ggplot_facet_controls --locked`: 13 tests passed, including generic host controls and versioned replay; `/private/tmp/gg12-final-focused.log`. A subsequent focused run passes all 14 tests including source-backed strip/axis ordering, plot-aligned strip bounds and retained guide translations; `/private/tmp/gg12-strip-order-fix.log`.
- Tests assert 4/12/16/72 multivariable catalogs, exact 24/10/16 margin identities, original margin population counts, literal `(all)` distinction, partial broadcast across differently ordered schemas and new levels, source/update/batch panel identity/domain agreement, shrink=true/false summary ranges, new author vs old-wire semantics, six pinned panel directions, four measured strip sides, registered formatter invocation, interior axis/label independence, and 9:1 actual plot-width ratios at 800 and 1200 Points.
- A scoped core Clippy identified facet formatting/argument-count warnings and unrelated concurrent GG09 warnings; owned facet warnings and compiler needless borrow were corrected. No whole-workspace Clippy success is claimed here.

## Publication and native artifacts prepared

Seventeen independent authors are implemented in `examples/common/ggplot_facet_controls.rs`, `scripts/bindings/ggplot_facet_controls.py`, and `.cjs`. Each requests 800×560 Points at 144 dpi with an explicit 0.1×0.1 minimum plot size so very thin proportional panels remain drawable without changing the engine’s default compact policy, compares original/replay scenes and produces 51 SVG/PDF/PNG files named `facet-0` through `facet-16`, plus plot/scene JSON. Rust wrapper: `crates/chart-export/examples/ggplot_facet_controls.rs`.

Native gallery: `examples/chart-gallery/examples/ggplot_facet_controls_native.rs`, cases 0/5/10/12/14/16 over the same Rust author helper and supplied Noto Sans font; an optional command-line mode 0–16 draws one author at 1440×680 for individual inspection. B owns the combined native/host build, this owner captures and inspects the facet app window afterward. Baseline actual Python/WASM runs each produced 51 publications successfully, but they predate the final strip placement correction and explicit minimum plot request. Final cross-runtime qualification is pending the coordinated delta build.

The initial native diagnostic capture (`/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_21-47-23.png`, trace `/private/tmp/gg12-native-initial.log`) exposed strips spanning whole cells and sitting outside axes, plus compact suppression of thin panels. It is not an accepted visual baseline. The source-backed shared solver correction now translates exterior guides past the reserved strips and aligns strip backgrounds with actual plot extents; the proof requests explicitly allow thin plots. Final native, source-render inspection and consolidated gate results will be appended after the rebuilt artifact is inspected.

## Final native inspection and corrective regression pass

The coordinated native build passed (`/private/tmp/gg09-gg12-native-final-build.log`). All 17 individual author windows were captured after non-null stamped `scene-frame-painted` events and inspected. Every owned process was closed. Capture manifest: `/private/tmp/gg12-native-final/manifest.jsonl`; individual traces: `/private/tmp/gg12-native-final/facet-N.log`. Modes 0/4/6–13/16 confirm data/summary separation, direction ordering, four strip sides, wrapped lookup labels, interior axis controls and unequal free widths after the strip correction. Dense modes intentionally preserve authored guide ticks, including overlap where available panel height is very small.

Inspection identified one further source mismatch: empty panels painted the general engine’s “No data” message. The shared solver now suppresses that message only for panels whose parent uses reference facet policy, while retaining `LayoutStatus::NoData`, plot geometry and legacy messages. Changed native/publication modes are 1/2/3/5/14/15. Their existing screenshots are diagnostic for this final change; targeted recapture remains pending. The other eleven inspected modes are accepted for the tested facet controls.

`/private/tmp/gg12-empty-panel-delta.log` records 15 facet tests, 17 colorbar tests and three legend tests passing, including reference-blank versus legacy-message behavior and preserved nonfacet wire expectations. Faceted authors in those previous tests now correctly require capability 76.

The first consolidated workspace run exposed combined vector recycling regressions in the new free-facet path. Corrected assembly supplies all shared-scale groups together to the existing `map_populations` evaluator. Margin occurrences retain source ordering by margin combination and original row, including repetition through the opposite axis’s margin panels; explicit occurrence addresses preserve panel-specific source targets. It does not add a vector evaluator.

Independent `tools/reference/r/facet-vector-sharing.R` generated six cases in `fixtures/parity/ggplot2/facet-vector-sharing.json`, with successful identity/reverse draws and the reference scalar-length rejection. Source log: `/private/tmp/gg12-vector-sharing-reference.log`. All 20 positional-vector tests (19 unchanged plus the six-case grid/margin sentinel) and all three scale-limit helper tests pass in `/private/tmp/gg12-vector-margin-delta.log`. The latter explicitly requests all axes because it validates every panel’s guide catalog; default suppression remains covered independently. Exact source callback populations, final panel points and scalar rejection are asserted.

Final host/publication equality, six changed native recaptures and consolidated acceptance remain root-coordinated. No GG12 package acceptance is claimed in this provisional report.

## Accepted native correction captures

The final3 build passed (`/private/tmp/gg09-gg12-native-final3-build.log`). Changed modes 1/2/3/5/14/15 were recaptured and individually inspected after stamped paint events; empty panels are now blank, with points, margins, axes and strip controls retained. Together with the eleven unchanged inspected modes, all 17 native authors are qualified for these controls. All owned app processes closed successfully. The manifest is append-only; use the latest entry for each mode. Earlier affected captures remain diagnostic evidence, not accepted baselines.

| Mode | Final screenshot |
| --- | --- |
| 0 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-02-36.png` |
| 1 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-13-59.png` |
| 2 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-14-02.png` |
| 3 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-14-06.png` |
| 4 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-02-56.png` |
| 5 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-14-09.png` |
| 6 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-02-59.png` |
| 7 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-01.png` |
| 8 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-02.png` |
| 9 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-24.png` |
| 10 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-25.png` |
| 11 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-27.png` |
| 12 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-28.png` |
| 13 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-42.png` |
| 14 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-14-12.png` |
| 15 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-14-16.png` |
| 16 | `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_22-03-46.png` |

Actual final host/publication equality and the consolidated workspace/package gate remain root-owned; native qualification is complete.
