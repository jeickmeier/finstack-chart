# Implementation status

Updated: 11 September 2026. Specification version: 0.5.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`; WP-04 at `d0a6c48`.
WP-05 is committed at `3cf1b33`; WP-06 at `f8657fb`; WP-07/08 at `fac148a`.
WP-09/10 are committed at `c5ec829`; WP-11 at `a6fb2ea`; WP-12 at `63dbcc2`.
Original reports retain the revision context from their evidence runs.

## Optional Kit crate withdrawal — 11 September 2026

Infrastructure slice: delete `gpui-charts-kit` and keep Kit as an optional gallery
recipe. [ADR-023](adr/023-withdraw-optional-kit-crate.md) records the boundary.
A-KIT is WITHDRAWN. Native/export AP-06 evidence is unchanged; hosts map Kit tokens
into `ThemePatch` and `ChartInput::from_plot`. The composition gallery inlines the
former adapter. Historical evidence snapshots and prior clippy `-p gpui-charts-kit`
logs remain dated records.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:

- `python3 scripts/check_repository.py`: PASS workspace edges, host isolation, single
  GPUI identity and local Markdown links. Members are eight packages; Kit is absent.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy -p chart-gallery --example family_gallery --example host_bootstrap --features kit --locked -- -D warnings`: PASS.
- `cargo check -p gpui-charts --locked` and `cargo check -p chart-gallery --example family_gallery --locked`: PASS.
- `cargo check -p gpui-charts-kit --locked`: fails with no matching package, as intended.
- `mise run check`: repository/deny/fmt/workspace check and Clippy passed; rustdoc
  then failed on pre-existing `[0,1]` intra-doc links in `interpolate/spline.rs` and
  `scales/ggplot_identity.rs` from the open ggplot2 assignment. Not part of this slice.

No Python/WASM, native window, or publication artifacts were re-run. GG-04 and G-AUTH
stay open. Next action: continue the authorized ggplot2 scale work; no Kit library
crate remains to maintain.

## Remaining-work reconciliation — 11 September 2026

Status-only review at `a6caa39` plus current working-tree changes: P2-00 and GG-00–03
are complete; GG-04 is in progress; GG-05–19 are not started as acceptance packages.
Existing baseline implementations within those packages remain reusable. All eight
D3 prerequisite gates have accepted scoped evidence; they are not a new implementation
backlog. G-GGPLOT/G-PARITY and the expanded production gates remain open.

The full package backlog is owned by [GG-04 through GG-19 in the Phase 2 plan](impl_plans/phase-2-parity-implementation-plan.md#gg-04--ggplot2-scale-and-palette-policies).
Current confirmed scale gaps and qualification limits are in the
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).
The pinned inventory still has 643 `OPEN` rows, including delivered capabilities;
it is an inventory baseline, not a current missing-feature count. Completing the
export/argument-to-evidence reconciliation remains required before claiming an
exhaustive API-level residual or closing GG-04/GG-19. This review read the live plan,
ledger, inventory and selected scale/guide source; it ran no feature/runtime tests.
Next: positional pre/post-statistic limit integration and remaining argument audit,
then the package-specific work and acceptance in the existing plan.

## Active ggplot2 assignment — 10 September 2026

Active follow-on: binned count/numeric palettes and reference physical linewidths
now pass 860 states per rebuilt Python/WASM host and 36 byte-identical, inspected
publication files. Wire 27 retains count palettes; linewidth uses the profile's
physical conversion and zero hairline at shared scene projection. The final Linux core suite
passes 547 tests, with Clippy and repository checks also passing. R guide-build
failures are distinguished from successful scale mapping. Details and limits:
[scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md). Next: remaining
GG-04 arguments/registered callbacks. GG-04, GG-05
presentation and cumulative gates remain open; no fresh macOS/native pass is claimed.

Registered discrete limit functions now pass all 90 pinned R cases, four external
contract tests and 198 matching states per rebuilt Python/WASM host. Wire 28 retains
the installed operation selection. Twelve publication files match byte for byte;
PNG and independently rasterized SVG/PDF inspection is complete. Core/external
Clippy passes. The full core/external suite passed 569 tests before the final
hidden-identity callback correction; focused tests and rebuilt hosts pass afterward.
Numeric limit callbacks now also pass all 378 pinned R cases and 677 exactly matching
states per rebuilt host, with eighteen identical, inspected publications. The shared
API is `CustomScaleLimits` / `with_limits_function`; wire 28 uses `limits_function`.
The full Linux core/external suite passes 571 tests/doctests; all-target Clippy passes.
The prior positional proof scripts are restored under `ggplot_positional_limits` after
a filename collision: both rebuilt hosts pass 3,582 matching states and twelve
unchanged publications. Discrete callbacks retain 198 matching host states and twelve
unchanged publications. Other rescalers, temporal/positional callbacks and remaining
registered arguments are next. The 547-test/860-state results above predate callback changes. Details and artifact
paths are in the scale evidence. GG-04 and cumulative gates remain open.

Follow-on rescaler work: 126 additional pinned R cases now pass focused external
checks (504 numeric cases total). Maximum rescaling preserves valid maps with
missing/empty limit vectors; binned single-boundary selection precedes rescaling.
The primary grammar now treats ggplot2 numeric NaN output as missing; a focused
test preserves legacy rejection. Both rebuilt hosts pass 892 exactly matching states
and eighteen publications identical to the inspected files. Final Clippy/rustdoc
pass; the stable-source full-suite passes 573 tests/doctests with no failures or
ignored tests. This qualification precedes the next temporal callback changes. Temporal callbacks now pass the 84-case Date/datetime oracle over 336 Rust
configurations and 1,408 WASM states including the existing numeric lane. The shared
callback context retains exact origin/unit and Date semantics; empty domains reject
before invocation. Both actual hosts match all 1,408 states and 36 inspected publication files. The
full Linux core/external suite passes 574 tests/doctests without failures or ignores;
Clippy, rustdoc, formatting and repository checks pass. Positional callbacks are next.
Their 252-panel oracle now verifies 238 first callback inputs and fourteen errors
before invocation through the shared evaluator. Internal callback evaluation and
filtered positional collection were extracted for reuse; seven positional-bin and
nine external callback tests pass, with five numeric tests rerun after the input
comparison was added. Core/external Clippy and formatting pass. The 574-test/full-host
qualification predates this internal extraction. Post-statistic callback retraining,
primary positional output and other registered arguments remain open; GG-05–19
remain outstanding. See the scale evidence for commands and the next integration step.

Latest extension: materialized positional palettes pass 144 primary R panels plus
116 nested secondary outcomes, 520 configurations per actual host, twelve identical
inspected publication files and four Rust replacement states. Full core passes 537
tests/doctests; all-target Clippy and strict Python/TypeScript consumers pass. Wire 23
retains authored numeric palettes. Remaining argument/callback reconciliation and
cumulative destination/platform acceptance keep GG-04 open.

Latest GG-04 slice (11 September): discrete secondary axes pass 176 pinned R panels
through 352 band/point JSON configurations in Rust and each actual Python/WASM host.
Both hosts produce identical records and nine identical inspected SVG/PDF/PNG exports.
Full core passes 533 tests/doctests; two focused tests additionally cover reversed ranges
and both orientations. Clippy passes. Starting revision remains `a6caa39` plus retained
working-tree changes. GG-04 is IN PROGRESS; GG-05–19 and cumulative gates remain open.
Next: reconcile custom positional palettes and remaining GG-04 arguments with the
pinned inventory. Details: [scale evidence](evidence/phase-2-ggplot-scales-2026-09-10.md).

The owner authorized continuing through GG-03–19. GG-03 completed on
accepted GG-02, SP-04 and WP-S05 prerequisites, starting at `a6caa39` with the
previously qualified axis/interpolation/hierarchy changes retained in the working tree.
GG-03 is COMPLETE: [acceptance evidence](evidence/phase-2-aesthetics-2026-09-10.md)
records twelve focused regressions, 104 reference glyphs, actual Python/WASM ownership
and eight update states per host, matching Rust/Python/WASM publication, inspected
PDF/native output, 487 macOS and 468 Linux tests/doctests, and a passing repository
check. The resolved style grows from 24 to 56 bytes for independent channels; this
explicit cost is recorded in ADR 014. G-GGPLOT and G-PARITY remain open. The latest guide slices pass 60
discrete selection/label records, 30 manual break-to-palette records and 160 continuous
guide cases, plus eighteen zero-row continuous/identity cases including JSON and
retraining, and 192 all-nonfinite candidate/label/censor cases. Primary guide edits
preserve marks. Binned candidates and labels pass 288 R records, primary metadata
and independent boundary-color checks, plus 192 binned population cases and 135 primary time-width string panels. Explicit R date/time formatting additionally passes 220 reference labels and primary
axis/JSON/resource checks, plus 40 primary duration-format panels. Fresh Python/WASM
proofs each pass 246 publication cases and 14 explicit tab-glyph rejections, with
identical records/PNGs and inspected SVG/PDF/PNG samples. Multiline labels pass all
four axis sides and supplied subsecond timezone-resource checks. All 487 core
tests/doctests and all-target core Clippy pass on the final code. Full guide presentation,
remaining scale arguments and actual host/destination acceptance remain open. The latest authored-limit extension passes 2,688 focused R cases, 120 related tests, all 495 core tests/doctests and all-target core Clippy; the hidden-guide/fractional extension passes all 499 core tests/doctests. Fresh Python/WASM each pass 2,304 matching records with three identical PNGs and inspected SVG/PDF/PNG samples. The final structural-validation fix passes 16 focused tests, all 500 core tests/doctests and all-target Clippy. Degenerate-domain handling additionally passes 256 R guide/mapping records and 192 primary chart cases, including constant-zero logarithmic bins, collapsed-cut rejection and correct reference missing-color defaults; all 494 core tests/doctests and Clippy pass. Transformed missing-limit population handling additionally passes 320 scale records and 24 actual R chart cases, including mapping rejections and fully specified-limit controls; all 491 core tests/doctests and Clippy pass. Binned count and transformed missing-limit boundaries additionally pass 228 R records and eight focused tests, including primary JSON/retraining checks; all 489 core tests/doctests and Clippy pass. Positional bins now pass 2,400 direct R records, all 960 primary panels (including infinite limits), 96 statistic/filter cases and live replacement versus fresh batches. Fresh Python/WASM each pass 960 matching records with four identical PNGs and inspected SVG/PDF renders; version 18 retains the shared positional state. Full core validation passes 507 tests/doctests without exclusions, final focused tests pass, and all-target Clippy passes. R's infinite-limit cases produce undefined coordinates and omitted points; an unbounded range adapter preserves that behavior with finite destinations and explicitly unavailable inversion. Free-facet populations, finite viewports over unbounded bin geometry and remaining scale arguments stay open. Minor selection now covers numeric, temporal and discrete policies, with 761 matching fresh Python/WASM records within the reference tolerance, 514 passing core tests/doctests, all-target Clippy and strict host declarations. Four precision cases preserve exact timestamp origins and promote fractional units; Date major flooring is isolated to the ggplot2 profile. Version 19 retains nullable raw minor values and finite positions. Repository, formatting and diff checks pass. Discrete continuous-limit control now passes 756 direct R records, 3,528 primary configurations and 24 replacement states. Fresh Python/WASM each pass 3,552 exactly matching records and four byte-identical PNGs, with SVG/PDF/PNG inspection completed. Version 20 retains the authored vector and distinguishes all-excluded observations from zero rows. All 518 core tests/doctests, all-target Clippy, strict host declarations and repository/format/diff checks pass. Automatic character-order training now follows pinned C collation only in the ggplot2 profile, preserving explicit domains. Six new R records pass 30 actual panels, bringing both host proofs to 3,582 exactly matching records. All four focused Rust tests and all-target Clippy pass; twelve publication files match each other and the previously inspected artifacts byte-for-byte. The latest change uses focused tests; the full 518-test run above predates this ordering correction. Nullable authored/factor domains preserve missing-level order and distinguish zero rows from a trained empty palette. The new 512-record direct oracle and 256 primary hue cases pass; 72 R position panels verify that missing paint preserves numeric/category training. Both hosts pass 346 exactly matching records and fifteen byte-identical publication files, all SVG/PDF/PNG samples inspected. All 523 core tests/doctests, all-target Clippy and repository/format/diff checks pass. Explicit retained empty-population state uses version 21; ordinary policies remain version 17. Identity primary missing colours now pass 512 actual R point-count configurations and twelve focused Rust tests. Both hosts pass 874 matching records, including 32 replacement states, with eighteen identical inspected publication files. The full 523-test run above predates this identity correction; all-target Clippy passes from a fresh target directory. Untrained fallback colour lookups now pass 64 direct R cases and 32 empty primary charts, preserving lazy short-manual-palette errors and empty guides. All 525 core tests/doctests and all-target Clippy pass from the fresh qualification target. Rebuilt hosts retain 874 identical records and eighteen unchanged inspected publications. Positional null identity now retains typed catalogs through domain union, projection, guides and inspection. The 576-case R matrix passes direct policy checks and 1,152 primary band/point configurations. Wire 22 retains factor/drop/translation/nullable-limit policies. Both actual hosts pass 1,152 exactly matching records and six identical inspected SVG/PDF/PNG files. Six focused Rust tests additionally cover 16 replacement states, immutable captures, reversed axes and explicit D3 null-omission controls. All 531 core tests/doctests, all-target Clippy, strict host declarations and repository/format/diff checks pass. Nullable navigation/continuous-limit combinations, broader geometry/facet and host replacement acceptance remain open. Coincident labels and empty-panel presentation remain GG-05 gaps; minor painting/animation, arbitrary callbacks and remaining scale arguments stay open. GG-04 is IN PROGRESS: [the current scale slice](evidence/phase-2-ggplot-scales-2026-09-10.md) passes 766 palette, Nullable continuous-limit/expansion combinations now add 1,152 pinned R panels, bringing direct coverage to 1,728 records and primary coverage to 3,456 configurations. Fresh Python/WASM each pass 3,456 exact-matching records and nine identical publication files, including inspected expanded-axis SVG/PDF/PNG. Full core remains 531 passing tests/doctests; all-target Clippy and repository checks pass. Evidence and next open minor/navigation work are recorded in the linked GG-04 scale ledger. Nullable numeric minor selection adds 192 reference panels: 1,920 direct records, 3,840 matching primary Python/WASM configurations, nine unchanged inspected publication files and 532 passing core tests/doctests. Clippy and repository checks pass. Next: reconcile the remaining GG-04 arguments, including discrete secondary axes, against the pinned inventory.
93 scale and 284 break records, plus focused point/after-scale and trained-bin budget regressions.
Automatic and authored linear/logarithmic axes now use reference expansion and
default breaks/labels: 120 expansion records, 499 label vectors and 32 complete panel
records pass, including collapsed linear/log viewports with midpoint projection.
Discrete range policies pass 36 R cases and 54 axis configurations against nine R
panels. Fixed elapsed-second time breaks and supplied-zone DST labels pass 30 R
panel cases; structured calendar widths pass 16 additional R panels with exact
timestamps/labels, bounded generation and v17 serialization. Datetime expansion passes
16 R cases through three primary forms, three fine-resolution cases and 16 paired
numeric contraction cases; fractional views retain exact source timestamps. Automatic
datetime selection and labels pass 534 primary R panels, including default restoration,
format overrides and supplied DST resources. Date axes pass 81 R records in two
source units plus 12 width panels through the shared timestamp engine. Timestamp
population limits pass eight R summary cases and focused exact-integer/stage regressions,
448 full core tests/doctests and all-target core Clippy. Numeric secondary guides pass 36 affine and 16 custom-transform R
cases, with independent breaks/labels and reference sampled/rounded placement.
Secondary Date/datetime guides pass 54 further R panels, including supplied DST
resources, plus edit, exact-unit and v17 rejection checks. Full core validation passes
450 tests/doctests before the final fixture expansion; the expanded focused run passes.
Reference point size units, zero/negative observation retention and outline conversion
pass nine R configurations / 63 glyph inputs, 452 core tests/doctests and all-target
core Clippy. Explicit `.radius()` retains its literal-radius contract. Portable point hairlines use the reference
PDF device's physical minimum; other R device hairline equivalence remains open.
Elapsed duration scales pass 45 break configurations, 66 primary and nine secondary
panels plus seven additional label vectors, reusing linear mapping and the extended
break kernel. The isolated oracle now pins hms 1.1.4. Named duration widths, four UTC
label patterns, inherited secondary labels, zero-range selection and v17 round-trips
pass. Full core validation passes 455 tests/doctests before the final duration extensions;
31 related tests and final all-target Clippy pass after them. Actual host acceptance
and complete format/argument compatibility remain open.
R color parsing now covers 657 named colors and 42 parsing records, with an explicit
portable rejection for overflowing palette indexes. Reference manual text colors pass
eight ggplot2 selections, 24 related tests and all-target Clippy. Numeric/discrete
identity mappings now pass 122 R records and primary colour/linewidth integration,
with raw output independent of guide limits and hidden default colour guides.
Reference alpha lowering and after-scale arithmetic pass 192 additional point-grob
paint comparisons, including infinite values and missing-value power identities.
All-target core Clippy and 464 full core tests/doctests pass. Numeric colour
indexes, complete identity guide presentation and actual host/destination acceptance
remain open.
Geometry-extent training, remaining time policies and ggplot guide-presentation defaults
remain open (zero-expansion projection tests explicitly preserve endpoint labels).
Remaining policies and full package acceptance stay open; the other authorized
packages follow in prerequisite order.

## Completed axis, interpolation and hierarchy assignment — 10 September 2026

The owner authorized final interpolation certification followed by axis and hierarchy.
WP-AX03–06 are COMPLETE and G-AXIS passes for the declared D3 typed/profile and supported
platform boundary. [Integrated axis certification](evidence/phase-2-axis-certification-2026-09-09.md)
records all FIX-19 verdicts, 372 static cases / 376 states, 125 timed samples,
actual Rust/Python/WASM and inspected native/SVG/PDF/PNG, exact retained capture,
449 macOS/448 Linux tests and a full repository-check pass.

WP-IP07 and G-INTERPOLATE now pass: the existing 27-export/reference/configuration
catalog and measured integration evidence are joined by the qualified axis consumer and
fresh 18-artifact host replay. [Final interpolation certification](evidence/phase-2-interpolation-integration-2026-09-09.md)
records that closure. H01–08 are COMPLETE and G-HIERARCHY now passes for the declared
finite typed surface. [Hierarchy final acceptance](evidence/phase-2-hierarchy-integration-2026-09-10.md)
records 898 independent oracle cases plus 27 control sequences per Rust/Python/WASM
surface, all 65 method/control/FIX verdicts, nine inspected native projections, 81 matching
three-host artifacts, update/replay/ownership proofs and measured resource handoff.
Final qualification passes 474 macOS and 455 Linux tests/doctests, strict host typing and
full repository checks. Starting revision `a6caa39`; changes remain in the working tree.
Earlier stop boundaries and remaining-work counts below are historical.

| Active remaining lane | Packages | Count |
| --- | --- | ---: |
| Axis, interpolation, hierarchy | None — assigned work complete | 0 |
| ggplot2 (outside this assignment) | GG-03–19 | 17 |
| Phase 2 total | 17 remaining of 71; 54 accepted | 17 |

All eight D3 lane gates pass. The hierarchy high-fanout hit query measured 18.49 ms p95
at 1,000 nodes; WP-22 retains optimization and interactive-latency/allocation/presentation
qualification. This component parity acceptance does not pass PERF-01–05.

Global WP-21/22/23, G-PARITY/G4 and other release gates remain open.

## Interpolation handoff — 9 September 2026

WP-IP06 is COMPLETE. Registered factories now feed shared interpolation, scale,
color/legend/theme and Rust/Python/WASM consumers, with inspected native/SVG/PDF/PNG
sampled frames and retained update/capture proofs. The
[interpolation report](evidence/phase-2-interpolation-integration-2026-09-09.md) records
commands, source identity, acceptance evidence and validation limits.

WP-IP07 is PARTIAL: its 27-export/configuration verdict catalog, fresh reference replay,
host/integration evidence and allocation/timing profile are complete, but WP-AX06 is a
required unfinished prerequisite. G-INTERPOLATE remains NOT PASSED. WP-AX03–06 were
not started by this assignment. The fresh repository check passed, including the
previously stalled standalone WASM metadata check; that requalification item is resolved.

| Lane | Remaining packages | Count | Next prerequisite action |
| --- | --- | ---: | --- |
| Axis | WP-AX03–06 | 4 | AX03 is ready; outside this interpolation assignment. |
| Hierarchy | WP-H01–08 | 8 | H01 is ready. |
| Interpolation | WP-IP07 final certification | 1 | Complete AX06, then qualify the axis transition consumer and final cross-lane gate. |
| ggplot2 | GG-03–19 | 17 | Retain the listed prerequisite order. |
| **Phase 2 total** | **30 remaining of 71; 41 accepted** | **30** | **No further package started in this task.** |

Expanded WP-21/22/23, AP-09/G-AUTH, G-GGPLOT, G-PARITY and G4 remain separate open
qualification work. The following axis handoff is retained as historical context.

## Phase 2 axis handoff — 9 September 2026

The owner narrowed the active assignment to finish WP-AX01 and WP-AX02, update the
remaining-work list and stop before WP-AX03. Their integrated implementation and focused
acceptance are complete; the [axis report](evidence/phase-2-axis-ticks-2026-09-09.md) records final repository validation and its limits. The D3 profile entry, checked
providers and independent tick selection/formatting are covered. Full D3 axis geometry,
components, transitions and certification remain open.

The following is the remaining Phase 2 todo list after these two package acceptances:

| Lane | Remaining packages | Count | Next prerequisite action |
| --- | --- | --- | --- |
| Axis | WP-AX03–06 | 4 | AX03 consumes accepted AX02; outside this assignment. |
| Hierarchy | WP-H01–08 | 8 | H01 reference/contract harness is ready. |
| Interpolation | WP-IP06–07 | 2 | IP06 integration; IP07 also requires AX06. |
| ggplot2 | GG-03–19 | 17 | GG-03 and GG-06 have their listed prerequisites; retain each package's dependency order. |
| **Phase 2 total** | **31 remaining of 71; 40 accepted after this handoff** | **31** | **Stop here; do not start another package in this task.** |

The remaining standalone WASM core metadata check stalled after the other repository steps passed and is explicitly incomplete; fresh actual WASM runtime proofs pass. Retain this check as a requalification item.

Expanded WP-21/22/23 requalification remains additional to those 31 packages. AP-09 /
G-AUTH's existing native performance gate is also separate. Five of eight D3 lane gates
pass; G-AXIS, G-HIERARCHY and G-INTERPOLATE remain open, alongside G-GGPLOT, G-PARITY
and G4. Package details and evidence below remain authoritative; this list does not
relax prerequisites or claim final production acceptance.

Earlier accepted work remains recorded below: WP-S01–08/G-SHAPE, WP-P01–04/G-PATH,
GG-00–02, the color/chromatic/scale lanes and WP-IP01–05.
The [oracle entry report](evidence/phase-2-oracles-2026-09-08.md) records 643 ggplot2
exports, 57 locked R sources, 32 reproducible records and 96 inspected artifacts.
The [path acceptance report](evidence/phase-2-paths-2026-09-08.md) records 86 reference
sequences, eight focused tests, actual three-host proofs, inspected native/publication
output, 261 macOS tests and 255 Linux core/export tests. Final repository, primary API
and aggregate binding proofs pass. These results do not close other D3, ggplot2 or
performance gates. GG-02 and color foundation results are recorded below.
SP-04 is COMPLETE for shared typed interpolation/distribution mapping, numeric aesthetics and guide metadata.
SP-05 is COMPLETE for exact numeric ticks/labels, locale specifiers, nice and publication integration.
SP-06 is COMPLETE for shared calendars/time axes and actual same-revision host proofs.
SP-07 is COMPLETE and G-SCALE passes for the declared typed FIX-20 scope.
[Integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md) records 661
cases / 19,562 operations per surface, complete public calendar replay, actual keyed
updates and interactions, five inspected native/publication figures, exact three-host
RGBA output, 343 macOS tests, 40 focused Linux tests and passing repository/type checks.
Piecewise/category/quantile timings and a stable WASM memory plateau feed WP-22;
no sustained-load or aggregate platform release gate is closed.
CLR-05 is COMPLETE and G-COLOR passes for COL-01–06 / FIX-C01.
[Integrated color evidence](evidence/phase-2-color-acceptance-2026-09-09.md) records
all 351 reference cases, actual host methods and paint inputs, three independently
authored perceptual/alpha/grayscale figures, mapped palette cache/target invariants,
11 passing macOS/Linux tests, inspected native/SVG/PDF/PNG and measured component/update
costs. CP-01 is COMPLETE: the [entry report](evidence/phase-2-chromatic-entry-2026-09-09.md) records the complete 76-export / 218-array / 38-interpolator oracle with 160,666 samples, byte-identical regeneration and the Rust reader. CP-02/03 are COMPLETE for core scope: [foundation evidence](evidence/phase-2-chromatic-foundations-2026-09-09.md) records exact tables, all 160,666 sampled rows, 27,434 exact V8 trigonometric anchors, 19 macOS / 21 Linux tests and passing repository checks. Fixed fused trigonometry resolved reproduced byte-rounding differences. Integrated certification is recorded immediately below.
CP-04/05 are COMPLETE and G-CHROMATIC passes for CHR-01–06 / FIX-21. The final 304-case exceptional-normalization matrix passes on Rust and actual macOS/Linux Python and Node WASM.
[Integrated chromatic evidence](evidence/phase-2-chromatic-integration-2026-09-09.md)
records 218 exact arrays and 160,666 ramp rows in actual Rust/Python/WASM, 90 composed
scale cases / 1,268 exact scene colors, v6 migration and checked guide identity,
24 update steps per host, 352 macOS suite tests plus focused macOS/Linux supplements,
five inspected native/SVG/PDF/PNG figures, cross-platform exact publication, a stable
18,000-owner WASM plateau and measured catalog/evaluation/update costs. A reproduced
non-finite ordinal-key training shift was fixed without changing legacy typed keys.
Final fresh primary runtime/type proofs, rebuilt native inspection and repository checks pass. No global release or
sustained-load gate is inferred from these measurements.
WP-S01 is COMPLETE for entry/foundation scope. Its [acceptance report](evidence/phase-2-shape-foundation-2026-09-09.md)
records all 63 exports/methods/defaults, 333 exact numeric contexts, 42 layout records,
actual macOS/Linux Python and Node WASM, identical 300/600 DPI scenes/PNG output,
and inspected native/SVG/PDF/PNG sector, hole and external-sink geometry. This consumes
G-PATH. WP-S02 is COMPLETE: [829 Cartesian cases](evidence/phase-2-shape-cartesian-2026-09-09.md),
all 20 curves, general areas, actual hosts and inspected output; the full macOS suite
passes 369 tests. WP-S03 is IN PROGRESS: [arc/pie implementation and qualification](evidence/phase-2-shape-arc-2026-09-09.md)
passes Linux core/Python and final WASM oracle, interaction, updates and publication;
macOS build/launch qualification is pending. While those loader waits persist, the independent
Earlier shape-stage progress (superseded by WP-S08 acceptance below): WP-S05 symbol implementation is IN PROGRESS on its accepted WP-S01 prerequisite: [156 fixtures per host, actual area/type guides, 104 updates per host and exact inspected publication](evidence/phase-2-shape-symbol-2026-09-09.md) pass. WP-S06 is IN PROGRESS: [complete stack kernels and tidy bars/areas](evidence/phase-2-shape-stack-2026-09-09.md) pass 435 numerical cases per host, 1,620 core tidy comparisons, 480 updates per host, exact inspected three-host publication and 385 Linux tests/doctests. Native/macOS qualification continues for these families before the remaining ordered shape gates. G-SHAPE remains open. GG-03 retains its WP-S05 prerequisite. All authorized Phase 2 work continues.


## CLR-01 reference and CLR-02/03 color math — 8 September 2026

[ADR-016](adr/016-color-values-and-paint-boundary.md) defines floating values, explicit
exceptional tags, parser/formatter behavior and the authored-value/byte-paint boundary.
The d3-color 3.1.0 oracle has 351 cases: all 148 names, eight constructors, every
inherited method, RGB/HSL clamps and exceptional channels. Separate regeneration
matches the corpus and manifest byte for byte. CLR-01 is COMPLETE for entry scope;
G-COLOR remains open.

CLR-02/03 are COMPLETE. The new core color module implements bounded CSS parsing,
RGB/HSL/D50 Lab/HCL/LCh/Cubehelix, copy/brightness, conversions, predicates, formatters
and explicit descriptor round trips. All 351 cases pass in actual Rust/Python/WASM,
including exact strings/bytes. Shared atan2, power and ECMAScript formatter corrections
fixed reproduced platform differences without weakening fixtures. Repository checks,
279 macOS tests, six focused Linux tests and complete primary runtime/type proofs pass.
[Retained evidence](evidence/phase-2-color-foundations-2026-09-08.md) records the source,
commands and limits. CLR-04 is COMPLETE: every paint input retains floating values, strict wire migrations pass, and Rust/Python/WASM/native/publication use the shared lowering boundary.
[Paint evidence](evidence/phase-2-paint-2026-09-09.md) records four independently authored figures, exact cross-host RGBA output, inspected SVG/PDF/native, retained live snapshots, 298 macOS tests, 14 focused Linux tests and passing repository/type checks. CLR-05 and G-COLOR are now qualified in the [integrated color report](evidence/phase-2-color-acceptance-2026-09-09.md).

## WP-IP01 reference and interpolation foundations — 8 September 2026

[ADR-017](adr/017-shared-interpolation-values.md) freezes target dispatch, exceptional
values, owned lifetimes, operation-specific bounds and explicit typed adaptations.
All 27 pinned exports, three gamma factories, rho and duration are inventoried. The
Node oracle generates 370 scalar/value/color/zoom cases; real Chromium 151.0.7922.34
generates 36 CSS/SVG transform pairs with source/binary hashes. Both regenerate byte
for byte. WP-IP01 is COMPLETE for contract/reference scope; G-INTERPOLATE stays open.

The new shared scalar/composition module passes 56 applicable reference cases plus
independent rounding, spline seam, factory-count and owned-sample tests. The floating
color interpolation module passes 106 applicable cases with exact CSS output, plus
independent hue/gamma/alpha anchors. Structured values pass 191 cases including the complete
source/target kind matrix and explicit adaptations; 17 zoom and 36 browser transform cases
pass. Strict standalone descriptors, every public factory/configuration, actual Python/WASM
samples, six invalid type cases per host, copies and disposal pass. Linux passes all 11
interpolation kernel tests plus six color/path regressions; two new descriptor tests pass
on macOS. Clippy, repository checks and combined primary runtime/type proofs pass.
[Retained evidence](evidence/phase-2-interpolation-foundations-2026-09-09.md) identifies this source snapshot and validation limits.
Chart paint/scale/axis consumers, inspected integrated publication and final parity gates
remain outstanding; no G-INTERPOLATE closure is claimed.

## GG-02 stage implementation and acceptance — 8 September 2026

Against `fab2505` plus the uncommitted continuation, canonical profile provenance,
scale/stat ordering, inferred discrete groups, horizontal recipes and typed bounded
expression graphs are implemented in the shared core. Ten stage tests and four
expression tests pass. The independent R 4.6.1 / ggplot2 4.0.3 corpus has 22 cases
and repeats byte for byte, including log/coordinate order, scale limits/zoom,
back-transform-before-after-stat expressions, missing/explicit groups and orientation.
Shared named statistics now apply the same source policies as inline operations;
conflicting consumer scale contexts explicitly reject.

The regular primary proof runner includes all new stage authors and positive/negative
Python/TypeScript consumers. Actual Rust/Python/WASM execution passes 13 figures per
host, source-expression filters and shared aliases. Static/Presented/Current capture
and profile edits preserve old requests after disposal. All 13 three-host PNGs have
identical RGBA channels. Native four-panel rendering, publication PNGs and rasterized
PDFs have been inspected; every PDF uses the supplied embedded Noto Sans.

GG-02 is COMPLETE for its assigned stage scope. `mise run check` passes; macOS
276 tests and Linux core/export 270 tests pass with none ignored. Actual
primary and aggregate binding proofs pass. The [acceptance report](evidence/phase-2-stages-2026-09-08.md)
retains commands, outputs, source hashes, images and limitations. Source reductions explicitly use the registered dataset before chart
filters/facet splits, while generated reductions use the prepared layer population.
Post-scale outputs cover size/color (size for point/rule); independent aesthetics,
physical units and complete default family behavior remain in GG-03 and later owners.
No broader ggplot2 or cross-library gate closes from this package alone.

## Integrated shape acceptance — 9 September 2026

[WP-S08](evidence/phase-2-shape-acceptance-2026-09-09.md) is COMPLETE at the retained
source snapshot over `fab2505951061eaafe9adb52c86b248ee0dfa6bf`. All 63 exports and
220 methods have per-item Pass verdicts under the finite typed d3-shape 3.2.0 profile.
Fresh macOS/Linux Python and Node/WASM generator, interaction and 1,544-update runs
pass; all host update records agree. There are 420 macOS workspace and 419 Linux
core/export/extension tests-doctests, and repository plus actual binding checks pass.
Native and nine publication images were inspected. All PNG/PDF bytes agree; two
Terminal SVG comparisons preserve control-coordinate differences no larger than
5.684341886080802e-14 within the existing tolerance and rasterize identically.
The remaining theme dash gap and two acceptance regressions are fixed without changing
oracle expectations. G-SHAPE passes; G-PARITY, G4 and WP-21/22/23 remain open.
WP-AX01/02 integration is recorded in the current axis handoff above. The owner's stop boundary is before WP-AX03.

## Phase 2 handoff verified; P2-00 integration contract delivered — 8 September 2026

The owner-requested predecessor **Review API simplicity** finished with completed
turns and no reported turn error. Verified `fab2505951061eaafe9adb52c86b248ee0dfa6bf`
(`Add primary chart authoring and shared host runtime`, 8 September 2026 23:11:16 UTC)
is HEAD and contains the primary Rust/Python/WASM implementation and completion
evidence. `git merge-base --is-ancestor fab2505 HEAD` passed. The task reader exposed
no final message for those completed turns; commit content, date and the committed
completion report establish the handoff. AP-09 performance qualification remains
open; it is not a prerequisite for independent Phase 2 contracts/legend acceptance.
The waiting heartbeat was paused for implementation and then deleted after its
handoff condition was fulfilled; it no longer polls the predecessor.

P2-00 is COMPLETE for its entry scope: [ADR-014](adr/014-phase-2-integration-contract.md)
records canonical-definition policy/provenance, compatible migration, explicit resource
ownership and the shared Node/separate R reference-lock layout. The
[coverage register](phase-2-coverage.md) links all eight lane inventories and GG-00–19
to requirements, semantic owners, AP-00 routes, fixtures, surfaces and open gaps.
It excludes delivered API foundations from remaining effort and preserves the combined
plan's dependency edges. Lane entries/GG-00 still own concrete reference installation,
locks and exhaustive method/argument fixtures; no oracle ran for this entry package.

Validation: `mise exec -- python3 scripts/check_repository.py` passed workspace/host
isolation and local Markdown file links on Darwin arm64. This is contract validation,
not runtime/parity certification. Baseline worktree contained the earlier Phase 2
plans/review/ledger edits; these are retained. No production dependency, portable
version or package publication changed. GG-01's remaining shared legend acceptance
is delivered in the following entry. G-AUTH, all D3 gates, G-GGPLOT, G-PARITY and
expanded G4 remain OPEN.

## GG-01 shared legend acceptance delivered — 8 September 2026

GG-01 / FIX-GG01 is COMPLETE for its specified scene and inspected SVG/PDF/PNG
matrix against `fab2505` plus this uncommitted slice. Shared layout now omits empty
guides, and primary `legend().untitled()` omits the title while retaining keys.
`generic_title()` explicitly retains the legacy Color/Value fallback in Rust, Python
and WASM. Existing primary fixture authors now use that explicit fallback; independent
expected fixtures and comparators remain unchanged. No second guide engine was added.

The [acceptance report](evidence/phase-2-entry-and-legends-2026-09-08.md) and
[retained evidence](evidence/phase-2-legends/README.md) cover 24 ordinary/collected/local
facet cases, including empty/hidden/tight/shared/incompatible guides and untitled
builds/edits. Actual Python and WASM each match all 24 Rust portable scenes exactly,
produce 72 exports and pass six host-dispatched untitled/generic edits. All 24 direct
PNGs and independently rendered PDFs were visually inspected. The full primary
runtime/type proof passes, including baseline component-family equivalence and replay.
Final `mise run check` and `mise run test` pass on Darwin arm64: 253 tests, zero
failed/ignored. The focused export run passes six tests including the 24-case matrix.
Final outputs equal the retained inspected artifacts; logs and source/artifact hashes
identify this validation snapshot.

The initial full test run exposed reliance on the old `untitled()` behavior in fixture
authors; that failure is retained alongside the corrected final evidence. Full guides,
colorbars and advanced collection remain GG-05. No fresh native-window, Linux, R oracle
or sustained performance run is claimed. Next package: GG-00's reference inventory/oracle
or a ready independent D3 entry; cumulative gates and AP-09 remain OPEN.

## Primary authoring implemented; final performance qualification pending — 8 September 2026

AP-00–08 are complete for the delivered LibraryV1 baseline. Data/Plot/Chart/Output,
all current component families, typed live operations, native/Kit/export destinations,
actual Python/WASM syntax and production-consumer migration are implemented. The
[completion report](evidence/primary-authoring-completion-2026-09-08.md) maps the
[35-row register](primary-authoring-api.md) to source, independent fixture and host evidence.
Revision: `b631f0e6d7e41722b5774433d616f704234157d7` plus this implementation;
the report's source inventory identifies the tested files. Unrelated Phase 2 edits remain
separate. All packages stay unpublished 0.1.0 and portable schemas stay version 1.

Final `mise run fmt`, `mise run check` and `mise run test` pass on Darwin arm64;
252 tests pass. Offline Linux aarch64 core/export/text tests/check/docs pass, 246 tests.
Actual `primary-authoring-proof` and `bindings-proof` pass. Primary execution covers
34 full component-family cases and 23 action + 47 input + 70 streaming steps in Rust,
Python and WASM, exact data, typed transactions, capture/disposal, detachment/memory
and strict declarations. Linux primary outputs pass the same independent comparison.
Native primary/streaming/Kit keyboard, selection, exact-value accessibility hooks,
historical pin/freeze/retention/queue/resume and Kit reset were executed and inspected.
SVG/PNG and searchable/outlined PDF artifacts were generated and inspected.

The native check exposed and fixed repeated frozen-frame acknowledgement entering the
resize-only path. The regression verifies repeat painting after live commit and failed
pending-scene admission without losing the retained source/capture policy. Updated
native execution passes. Field/identity validation and seeded-jitter fixture identity
were also corrected without changing existing expected values or tolerances.

AP-09 / G-AUTH remain open pending final native performance qualification. The initial
five primary workloads passed their applicable baseline protocols. A final-code rerun
recorded ten-line frame-work p95 21.095374 ms, followed by 20.712834 ms on repetition
(target 16.7 ms); hover remained below 4 ms. Platform submission dominates the slow
samples. Both dashboard attempts completed all 120 preparations per chart but stopped
painting before the drain snapshot; those attempts fail acceptance and are retained.
The unchanged baseline also failed the matched ten-line comparison at 21.160960 ms;
the paired primary run had fewer than 30 displayed samples. A refactor-specific
regression is not established, and a stable visible-window rerun has been requested. The ADR-008 baseline 30-minute
exception, full-assistive/other-platform limitations, dependency advisories and expanded
parity/G4 requirements remain explicit. Next action: complete the requested visible
native measurements, preserve failed traces, then close or explicitly leave G-AUTH open.

## Phase 2 implementation awaiting API commit — 7 September 2026

Historical waiting entry, superseded by the verified 8 September handoff above.

Owner authorized implementation after the task **Review API simplicity** completes
and commits its code. At this handoff its local task snapshot was active; no completion
or qualifying commit was verified. Phase 2 implementation remains pending that condition.
A follow-up in the Phase 2 task checks every five minutes, verifies the completed task's
commit and resumes from P2-00 against that baseline. It stays quiet while unchanged.
The handoff baseline remains `b631f0e6d7e41722b5774433d616f704234157d7` plus active
edits; the implementation start must record the actual qualifying revision. This
waiting step ran no feature checks and advanced no package or gate.

## Phase 2 plans reconciled with the current API — 7 September 2026

Owner-authorized planning update against `b631f0e6d7e41722b5774433d616f704234157d7`
plus active AP edits. Updated the [Phase 2 plan](impl_plans/phase-2-parity-implementation-plan.md),
[axis handoff](impl_plans/d3-axis-parity-plan.md),
[primary-authoring handoffs](impl_plans/primary-authoring-api-plan.md) and
[main integration plan](impl_plans/gpui-charts-implementation-plan.md) using the
[current API review](evidence/phase-2-current-api-review-2026-09-07.md).
Scope: GG2-01/02/04/12, AXIS-01/07, AUT-01/03–07 and BND-01/03/04.

GG-01 now reconciles the delivered shared painter and remaining FIX-GG01 edge/visual
cases. P2-00 is the first new parity handoff, linking AP-00's register and current
Data/Plot/Chart/Output owners. GG-02 specifies profile execution/wire/cache/capture
propagation; WP-AX01 includes primary handles/names/layer bindings and host navigation;
AP-07 and each semantic package share host syntax and actual export/declaration/runtime
proof responsibilities. Live Python/WASM authoring additions supersede the review's
earlier JSON-only source observation; their acceptance remains with AP-07's evidence.
Historical effort allowances now require subtraction of delivered AP/legend work before
estimating remaining effort. Original-scope WP completion is retained and expanded
requalification remains required. Historical review reports remain unchanged.

Validation in the repository on Darwin arm64 via mise:

- `mise exec -- python3 scripts/check_repository.py`: passed dependency/host-isolation
  graph and local Markdown file-link checks; no target execution or external-link/anchor validation.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/phase-2-parity-implementation-plan.md docs/impl_plans/d3-axis-parity-plan.md docs/impl_plans/primary-authoring-api-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md`:
  passed, retaining the repository's Markdown hard-break convention.
- Reviewed the updated handoffs: all eight D3 inputs, GG-00–19 and cumulative gate
  prerequisites are retained; the new API section is shared by the companion plans.

No feature tests or acceptance gates are claimed by this update. No production code, dependency,
fixture, public wire version or package/gate completion state changed in this task;
unrelated live edits were preserved. Next action: P2-00 contract/evidence reconciliation,
then GG-00 and ready D3 entry packages; GG-01 remaining acceptance can proceed independently.

## Phase 2 current API review — 7 September 2026

Reviewed the Phase 2 plan against `b631f0e6d7e41722b5774433d616f704234157d7`
plus the active primary-authoring working tree. The
[current API review](evidence/phase-2-current-api-review-2026-09-07.md) maps each
parity family to its current public owner. The capability scope remains applicable;
the GG-01 implementation kickoff is stale because ordinary and faceted legends now
share a painter and an export regression. Full GG-01 edge and visual acceptance
remains unresolved. Future packages must extend existing Data/Plot/Chart/Output,
define compatibility-profile propagation, migrate primary axis consumers with the
scale/guide identity split, and deliver actual Python/WASM exports and declarations.

On Darwin arm64, `mise exec -- cargo test -p chart-core --test authoring --test authoring_runtime --test authoring_host --locked`
passed 20 tests and `mise exec -- cargo test -p chart-export --test authoring --locked`
passed 4 tests. `mise exec -- python3 scripts/check_repository.py` passed graph and
local Markdown file-link checks; scoped `git diff --check` passed for tracked changes.
These checks cover the built Rust test binaries; Rust host dispatch
does not establish Python/WASM runtime support. No fresh native UI, Linux, external
oracle, performance or visual-inspection evidence was produced by this review.
No production code, plan or gate state was changed. Next action: reconcile the GG-01
handoff and attach the report's concrete API owners and acceptance cases to the
Phase 2 packages while coordinating with the active AP implementation.

## Primary authoring implementation in progress — 7 September 2026

Owner authorized the complete AP-00–09 plan. AP-00's baseline/public contract,
35 source-owner coverage rows, migration policy and acceptance handoff are recorded in
[the capability register](primary-authoring-api.md). Baseline remains
`b631f0e6d7e41722b5774433d616f704234157d7`; earlier owner-authorized planning edits are
preserved. No publication or parity-kernel implementation is implied.

AP-01 extracted typed Chart store/reducer/compiler/queue/query ownership; legacy JSON
Session now forwards to it, retaining eager portable preparation. New structural
validation accepts registered native-only operations without running statistics.
Owned and external-source routes remain distinct. Four new FIX-AUTH01 runtime tests
pass, including independent external views, native-only execution counting,
definition-only data/replay/queue preservation, typed/legacy histogram/replay parity.
The existing portable/streaming/scheduling/stage-cache selection passes 22 tests.

AP-02 has the first Data columns/rows, owner-scoped field/layer handles, immutable
Plot and inherited source aes/layer/histogram route. Four FIX-AUTH02 core tests pass:
independent geometry/bin membership, nullable/exact integer/timestamp preservation,
foreign-handle and missing-field diagnostics, and independent dataset overlays.
Native ChartInput::from_plot and the headless Output destination compile. Component
and publication tests, native lifecycle integration, complete runtime/binding/consumer
migration and performance acceptance remain in progress. No cumulative gate closes.

Commands on Darwin arm64 with mise Rust 1.97.1: `cargo metadata --no-deps --format-version 1 --locked`;
`cargo test -p chart-core --test portable --test streaming --test scheduling --test stage_cache --locked`;
`cargo test -p chart-core --test authoring_runtime --locked`;
`cargo test -p chart-core --test authoring --locked`;
`cargo check -p chart-export -p gpui-charts --locked` (all via `mise exec --`).
Next action: finish real publication proofs and component/runtime coverage, then the
remaining ordered packages. G-AUTH remains OPEN; the register records partial coverage.

### Continuing authoring implementation evidence

The later primary-authoring slice adds regression coverage for original-Plot edits after
live append, retry/queue preservation, retention rollback and queued epoch conflicts.
Automatic timestamp mappings now share an exact integer origin across owned datasets of
the same representation. Shared automatic color catalogs are trained before per-layer
color assignment. Facet authoring rejects ambiguous same-ordinal/different-name fields;
matching datasets must currently align facet-field schema positions or explicitly use
broadcast/panel targeting. Explicit timestamp origins remain authoritative and incompatible
origins reject. Shared automatic colors use ordered catalog union per coherent snapshot;
an explicit color domain fixes category-to-palette assignments across changing catalogs.

`mise exec -- cargo test -p chart-core --locked` passed the entire core suite and doctests
on this slice. `mise exec -- cargo test -p chart-export --test authoring --locked` passed
4 tests, including the 8-case Presented/Current × visible/full-domain × interaction matrix,
with requests executed after runtime disposal. Acknowledged destination layout is retained
as capture provenance and as the default Presented layout policy, with explicit publication
size/font/options applied. Native input now has tooltip/control/accessibility/edit/density
builders, and Kit has a primary plot mount helper. Native committed receipts remain committed
when later scheduling fails; that separate failure uses the existing diagnostic surface.

`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p gpui-charts-kit --all-targets --locked -- -D warnings`
passed before the subsequent host-binding dispatch additions. The refreshed
`mise exec -- cargo run -p chart-gallery --example primary_authoring --locked -- --headless`
produced updated `target/authoring/native-primary.{svg,pdf,png}`; PNG and a Poppler-rendered
PDF (`/private/tmp/finstack-primary-refreshed.png`) were visually inspected and the Peak
annotation is inside the plot. These checks do not certify native input/lifecycle or bindings.

AP-07 now has shared Rust host dispatch into actual typed component/draft builders,
typed host runtime/transaction/capture adapters, and primary Plot version-1 interchange
with names/profile/exact data. PyO3 and wasm-bindgen owned handles now expose these paths;
`packages/python/finstack_chart` and `packages/wasm/authoring.cjs` provide ordinary
column/row/component syntax without a separate grammar compiler. Source arrays cross
typed native vectors; Python integers and WASM BigInt retain exact 64-bit payloads.
Output resources, immutable requests/frames and deferred export queue/jobs are separate
owned handles with explicit disposal. Existing portable Chart APIs remain available.

The first smoke checks have been superseded by committed
[scripts/run_primary_authoring_proofs.py](../scripts/run_primary_authoring_proofs.py)
and `mise run primary-authoring-proof`. On Darwin arm64, Rust 1.97.1, Python 3.14.6,
Node 24.14.0, TypeScript 6.0.2, mypy 2.3.0 and wasm-bindgen 0.2.128, the actual Rust
executable, PyO3 extension and Node WASM module passed shared primary-author assertions.
The runner compares initial/final source semantics, scenes, actions, navigation,
transactions/replay, exact 64-bit timestamp/integer/null/category values, summary means
and OLS against independent expectations. Scene/navigation tolerances are 1e-9 destination
units and statistical tolerances 1e-12; opaque allocated identities are normalized only
after source identity relationships are checked. Actual SVG/PDF/PNG encoders execute.

Owned editor handles, named runtime commands, event-time retention, structured error
properties and separate public component classes now have Python stubs and TypeScript
declarations. JavaScript offers camelCase aliases. Positive typing examples pass;
five negative examples reject misplaced titles/subtitles/annotations/legends and data
replacement through definition edits. Host proofs also cover input mutation and one-time
row callbacks, display/boolean/null metadata, invalid/foreign fields, queue acceptance
versus commit, stale queued bases, update-versus-batch domains, count/event retention,
stale state commands, eight capture combinations, editor/request survival after disposal,
export-job limits/cancellation and repeated disposal. Python made 710 independent thread
steps during one 100,000-row Rust semantic call (0.891 seconds, debug build); this is a
detachment observation, not a performance claim. WASM memory stayed 6,815,744 bytes across
six batches of 100 create/dispose cycles with 1,000 rows, explicitly freeing all fluent
intermediate handles. A prior GC-timing-only run was unstable; garbage collection timing
is not a deterministic disposal guarantee.

Evidence: `target/authoring/{environment.json,*primary*.log,native,python,wasm,typing}`.
The Python/WASM PNGs were inspected; a fixture-only label offset was corrected so the
annotation remains inside the plot, then all three runtime fixtures were rerun.
Command: `WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen TSC_JS=/Users/jeickmeier/.npm/_npx/e04ecd76da0b5726/node_modules/typescript/lib/tsc.js PYTHONPATH=/Users/jeickmeier/.cache/uv/archive-v0/PKSDgIDzoTfeXd75NY8Rf/lib/python3.14/site-packages mise run primary-authoring-proof`.
`mise run fmt`, Python/export all-target Clippy and wasm32 all-target Clippy passed.
Baseline revision remains `b631f0e6d7e41722b5774433d616f704234157d7` plus these uncommitted
changes. The runner also now exercises the compiled custom density-histogram extension through
primary components in all three hosts. `custom_stat(...).field_parameter(name, field)`
resolves owner-checked source mappings only during authoring. The example owns its
`density_histogram`/`chamfered_bars` helpers and explicit registry installation/loading;
no parameter payload installs code. Independent counts/members/density and both generated
consumers match; the headless image was inspected. Native-only variants build as Rust
plots but reject portable runtime/serialization and headless preparation. Full remaining
component/standalone coverage, native lifecycle qualification, consumer migration and
AP-09 performance/platform gates remain open. The existing aggregate compatibility
runner also passed with
`WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof`:
36 stat/position/geometry cases, 23 action transitions, 47 input steps, 70 streaming steps,
40 held-capture steps, extension/schema/capability failures, rich text/facets/density and
independent vector/raster assertions across actual Rust/Python/WASM. Evidence is under
`artifacts/bindings`. This establishes preserved legacy contracts; the remaining delivered
component families still need primary-builder evidence. This shared host proof does not close every capability row or G-AUTH.

The current working tree adds grammar/stat/position/axis/color/text/theme/facet/figure
builders, named transforms, generated color mappings, grouping overrides, runtime
transaction/configuration helpers and immutable component edits. Compiler structural
validation now covers facets, axes, figure references and color stages without executing
statistics. Grid builders infer the complete row/column product. Ordinary charts and
facets reuse the same legend painter. Future GG mapped-symbol/linetype kernels remain
unimplemented and their coverage stays open.

Native ChartInput/ChartView now adopt the same typed Chart; GPUI retains tasks, a separate
worker compiler, frame acknowledgements and native resources. Committed source admission
is separate from worker preparation; an older admitted result cannot replace newer
committed data. Paint acknowledgement retains actual inspection state for Presented
capture. Supplied fonts have an automatic native resource constructor. Full native
lifecycle, sustained workloads and bindings still require requalification.

Additional commands, all in the repository on Darwin arm64 with mise Rust 1.97.1:

- `mise exec -- cargo test -p chart-core --test authoring --test facets --test layout --locked`:
  36 passed at that slice (7 authoring, 9 facets, 20 layout).
- `mise exec -- cargo test -p chart-core --test authoring --locked`: subsequently 8 passed,
  adding independent shared-transform/generated-color/reuse assertions.
- `mise exec -- cargo test -p chart-core --test authoring_runtime --test portable --test scheduling --test streaming --test stage_cache --locked`:
  27 passed, including older-worker/newer-commit and disposal checks.
- `mise exec -- cargo test -p chart-export --test authoring --locked`: 3 passed, including
  real SVG/PDF/PNG publication and ordinary/faceted legend text. A missing facet selector
  was corrected in the test and now also rejects during structural build.
- `mise exec -- cargo run -p chart-gallery --example primary_authoring --locked` and
  `mise exec -- cargo build -p chart-gallery --example primary_authoring --locked`:
  built the shared native/publication example. A sandboxed launch could not connect to
  macOS services; the compiled local example was subsequently launched outside the
  sandbox and as a temporary local app bundle for visual inspection. The actual native
  chart displayed both colored series, title/subtitle, axes, legend and caption. Keyboard
  interaction was not verified by this inspection.
- Artifacts: `target/authoring/primary-authoring.{svg,pdf,png}` and
  `target/authoring/native-primary.{svg,pdf,png}`. PNG and Poppler-rendered PDF were inspected.
  Inspection identified an annotation offset outside the default plot clip; the example
  now places that label inside the plot and awaits refreshed image inspection.
- `mise run fmt` passed at the recorded intermediate slice. Focused Clippy identified
  new clone-on-Copy/large-enum issues; fixes are applied but the final rerun is pending.
  Latest `mise exec -- cargo check -p chart-core --locked` passes after immutable edit work.

These counts precede the newest edit/configuration changes; they do not certify those
changes, actual Python/WASM execution, complete native lifecycle or performance gates.
Next: behavioral coverage for primary edits/transactions/configuration, remaining
component ownership checks, native/export consolidation, then AP-07–09. G-AUTH stays OPEN.

### Primary component-family qualification

The primary Rust, Python and WASM authors now execute 34 existing independent
statistic/position/geometry/scale/facet/composition cases through explicit builders.
The shared Rust fixture checks exact source materialization, full semantic output and
resolved marks against the original fixture definitions, rebasing only allocated
layer/dataset/field/scale identities. Python/WASM use ordinary columns and explicit
components, then compare complete semantics and scenes with Rust at the existing
1e-12/1e-9 tolerances. No fixture expectations or visual baselines were changed.
Coverage includes overflow/automatic bins, summary/count/OLS and affine/filter variants,
stack/normalization/dodge/data/display jitter, log/symlog/point/UTC/session scales,
area/ribbon/cells/OHLC/volume, group color and affine secondary axes; seven facet cases
cover catalogued empty grids, free axes, broadcasts/targeted layers and group/facet/chart
stat populations. Three publication cases cover named themes, gradients/symbols/dashes,
explicit regular/bold/Arabic fonts, rich titles/axis labels, ordered notes, panel letters,
callout/coordinate-space labels and insets using existing prepared layers.

This exposed and fixed two missing primary controls (`LayerBuilder::color_group` and
`LegendBuilder::untitled`) and bar/volume defaults discarded by `.aes(...)`. Bar baselines
are now recipe options used when y2 is unmapped; explicit y2 mappings retain precedence.
Python/WASM dispatch and declarations include the added controls. Native editorial
composition, OHLC/volume and grid artifacts were inspected; the six-panel grid's initial
400×260-point page correctly reported layout pressure and was enlarged to 600×540 points
for its proof artifact. Editorial composition uses 180×120 mm at 96 DPI.

Commands: `mise exec -- cargo test -p chart-core --test authoring_families --locked`;
`mise run primary-authoring-proof` with the task-local tool paths above; after extending
facets/composition, each actual Rust/Python/WASM producer and
`mise exec -- python3 scripts/bindings/authoring/compare.py target/authoring` ran again.
The comparison passed all 34 primary families. Outputs are
`target/authoring/{native,python,wasm}/{families,family-scenes}.json` and native PNGs.
The expanded WASM workload plateaued at 19,070,976 bytes in its six explicitly freed
batches; its higher peak includes the additional families and supplied fonts. The latest
Python detachment check recorded 627 thread steps over 0.786 seconds (debug build).
Full native lifecycle/input/Kit qualification, remaining option/runtime coverage,
consumer migration and AP-09 performance/platform gates remain open.

### Consumer migration in progress

The main native gallery now uses ordinary `Data::rows`, named recipe builders and
immutable Plot edits, with owner-derived field handles for inspection. Family/facet/
composition galleries reuse explicit recipes in `examples/common/authoring_fixtures.rs`;
the independent raw-fixture comparison remains in tests. Kit mounts these primary plots
through its supplied-theme input helper. Extension, action and interaction galleries now
use primary construction. The linked host-tools example shares Data between charts,
builds callouts, derives actual layer identities and uses `link` through native
`capture_link`/`resolve_link`; captured host-command export still uses its specialist
snapshot boundary to retain the event's exact scene.

The curated prelude exposes builders and relevant option enums without host dispatch or
ambiguous profile exports. Root Rustdoc and README now lead with Data/Plot authoring;
`docs/authoring-guide.md` documents current components, runtime/capture ownership, host
usage and the 0.2/0.3 additive migration policy. No old public module was removed.
Migration exposed named-axis insertion discarding prior primary-axis settings and missing
Rust edit x/y-axis conveniences; both are fixed. The meaningful regression checks
non-default bar baselines before/after aes, retained primary axes and stable edit handles.

`mise exec -- cargo test -p chart-core --test authoring --test authoring_families --locked`
passed 12 focused tests plus two corpus tests covering 34 cases. `cargo check` passed
main/family/Kit/extension/actions/interaction/host-tools consumers via mise. `mise run fmt`, repository graph/link validation and
`mise exec -- cargo clippy -p chart-core -p chart-export -p gpui-charts -p chart-gallery --all-targets --locked -- -D warnings` passed on the latest slice. Streaming,
scheduling/live-export/benchmark consumers, native interaction/lifecycle/Kit execution,
complete option/runtime coverage, docs/type/schema validation and AP-09 measurements
remain in progress. G-AUTH stays open.

### Primary runtime consumers and documentation

The streaming gallery now owns one Chart and uses named transaction/retention and
queue builders. The scheduling gallery and finite/sustained native benchmarks author
ordinary columns and components, commit through the mounted Chart, and keep explicit
CPU/GPU/presentation instrumentation. The live-export gallery and sustained benchmark
use Output::live_request; annotation definition edits and later commits cannot alter an
accepted request. The publication tutorial now uses Data rows, Plot and Output with
physical dimensions, supplied fonts and text/DPI options. Raw fixtures remain only in
compatibility/diagnostic proof programs and internal stage benchmarks.

Primary `external_view`/`accept_from` now preserve authored handles and definition edit
ownership while sharing committed snapshots and keeping independent reducers. Rust
checks prove pointer-shared snapshots, explicit source admission, rejection of a copied
writer and continued view preparation after writer disposal. Python/WASM expose the
same operations with fresh actual-runtime checks included in the primary proof runner.

README, the authoring guide, release guide, portable compatibility contract, historical
alpha API guide and changelog now lead with the implemented primary surface. The planned
0.2.0 migration retains old public paths and wire version 1; removal is no earlier than
0.3.0. Current packages remain 0.1.0 and unpublished. No separate Phase 2 semantic gate
is advanced. Performance requalification and remaining integrated/native checks follow.

Validation so far: native example compilation and all-example Clippy with
`--features performance,kit --locked -- -D warnings` pass. The focused core external-view
suite passes all eight tests. These are intermediate results; final aggregate and actual
host/native/performance results will be recorded separately below.

## Grouped aesthetic contract recorded in the plan — 7 September 2026

Added section 3.5 to the [primary authoring plan](impl_plans/primary-authoring-api-plan.md)
with the proposed grouped line/point example and independent group/color/fill/shape/
linetype/size/linewidth/alpha channels. The contract distinguishes group membership
from scale-selected appearance, supports same or different mapping fields, preserves
plot/layer inheritance and overrides, and keeps constants and legends with their
respective builders. Mapped styles must not require a separate layer per category.
AP-03 acceptance now covers actual resolved styles, line membership, inference and
override rules, legend keys, inspected output and AP-07 host proofs. GG-02–05 retain
semantic ownership; the example is explicitly prospective.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; outcome is uncommitted
plan/ledger edits preserving earlier work. AP-00–09 remain PLANNED, GG-02–05 remain
NOT STARTED and G-AUTH remains OPEN. Next action: AP-00's API walkthrough includes
section 3.5. Validation: `mise exec -- python3 scripts/check_repository.py`, temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py`, and
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/implementation-status.md`
passed on Darwin arm64. Documentation-only checks; no proposed API/runtime tests ran.

## Grouped aesthetic API clarification — 7 September 2026

Reviewed AUT-03 / GG2-03 against current `SourceAes`, `ColorEncoding`, `ThemePatch`
and GG-02/03. The planned primary `aes()` supports independent group, color, shape
and linetype mappings, inherited or overridden per layer. Group selects connected
observations/statistical populations; scales choose visual encodings. Current source
mappings provide explicit group and size; separate color encoding supports source
categories and prepared groups. Symbols/dashes are currently plot/layer theme values,
not mapped shape/linetype channels. GG-02/03 remain NOT STARTED; no inferred grouping
from color or complete mapped symbol/line-style support is certified by this answer.

Revision `b631f0e6d7e41722b5774433d616f704234157d7`, with existing uncommitted planning
edits preserved. Evidence is source/plan inspection only; no runtime tests ran.
Validation: `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
passed. Next action remains the recorded AP-00 walkthrough; mapped aesthetic semantics
retain GG-02/03 ownership and primary builder integration in AP-03. G-AUTH remains OPEN.

## Implemented builder inventory and future integration rule — 7 September 2026

Added a source-linked 16-family component inventory to section 3.3 of the
[primary authoring plan](impl_plans/primary-authoring-api-plan.md): explicit figure
furniture/callout/inset builders, existing marks/statistics/positions, transforms,
scales/axes/legends, facets, theme/text/layout, runtime configuration, host hooks,
publication and registered components. Builder names remain proposed. The inventory
separates implemented fixed annotations from future mapped labels and other parity
requirements; the introductory sketch now uses the existing Editorial theme.

Section 3.4, AUT-01 and ADR-013 require every future feature to extend an existing
typed builder or add a focused component within the primary API. Its own package
includes primary examples, applicable bindings/serialization updates and behavioral
evidence. Runtime operations remain Chart/destination methods. AP-00/04/08 acceptance
now carries inventory expansion, composition evidence and the ongoing integration rule.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
plan/specification/ADR/ledger edits, preserving earlier changes. Source inspection
covered the linked current core grammar/composition/layout/runtime and native/export
declarations. No runtime implementation or schema changed; AP-00–09 remain PLANNED
and G-AUTH remains OPEN. Next action: AP-00's option-level register and API walkthrough.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` and temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed documentation links,
package/fixture/ledger and requirement/dependency consistency.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime test or proposed-builder compilation ran; this is planning evidence.

## Separate annotation and figure builders — 7 September 2026

Updated the [primary authoring plan](impl_plans/primary-authoring-api-plan.md),
AUT-04 and ADR-013 to the owner's direction: labels are x/y-positioned annotations;
titles, subtitles, x/y axis labels and legends have separate builders. Section 3.1
owns the proposed routes and supersedes the older combined-labels sketch. AP-00/03/04
now require separate component examples, correct annotation identity/provenance,
automatic legends and shared text/guide implementation. FIX-AUTH04 includes component
isolation and compile-fail checks against misplaced labels-builder setters.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/specification/ADR/ledger edits, preserving prior review and plan updates.
No implementation or wire format changed; AP-00–09 remain PLANNED and G-AUTH OPEN.
Next action remains AP-00, using the revised builder boundaries. Intended acceptance
has not been executed; this is a documentation-only update.

Validation from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed; the temporary
`mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed the unchanged package,
requirement and dependency checks. `git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/spec/gpui-charts-specification.md docs/implementation-status.md`
passed. No runtime or proposed-builder compilation was run.

## Primary authoring plan review findings incorporated — 7 September 2026

Updated the [plan](impl_plans/primary-authoring-api-plan.md) and
[ADR-013](adr/013-primary-authoring-api.md) at the owner's request. APR-01–04 are
addressed in the planning contract: definition-only edits preserve current data;
owned ingestion and external committed sources retain one commit authority, with
independent view/worker/export lifetimes; Presented/Current export basis is separate
from navigation/interaction policy; structural build and native/portable validation
are separate. AP-00/01/02/05/06 now carry the corresponding handoff and regression
acceptance. The AP-00–09 sequence and AUT-01–09 remain unchanged.

Starting revision `b631f0e6d7e41722b5774433d616f704234157d7`; result is uncommitted
planning/ADR/review-follow-up/ledger documentation. Prior review edits were preserved.
The plan records the reviewed completed original-scope baseline, without reclassifying
expanded parity or original acceptance. No implementation, dependency, wire schema,
fixture or release setting changed. Review findings are addressed in the plan only;
all AP packages remain PLANNED and G-AUTH remains OPEN. No runtime tests or proposed
API execution ran for this documentation-only update. Next action: AP-00's baseline
and public-contract handoff, then AP-01 under the clarified ownership map.

Validation in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, mise Python
3.14.6: `mise exec -- python3 scripts/check_repository.py` passed repository/dependency
and local-link checks; `mise exec -- python3 /tmp/check_chart_authoring_plan.py` passed
ten packages/fixtures/ledger rows, nine requirement mappings, fifteen capability
families and an acyclic sequence. An inline `mise exec -- python3 -` check verified
the four review findings' representation and updated-document whitespace.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/impl_plans/primary-authoring-api-plan.md docs/adr/013-primary-authoring-api.md docs/implementation-status.md`
passed. These are documentation checks, not AP implementation or runtime evidence.

## Primary authoring plan checked against the current library — 7 September 2026

Outcome: reviewed the plan against clean revision
`c773fcba9a8c846322d08c9c1203fce415e91328`, including the now-completed original
WP-17–23 source and contracts. The original-WP assumption matches the ledger's
original-scope completion records; expanded parity and G-AUTH remain open.
HEAD later advanced to `b631f0e6d7e41722b5774433d616f704234157d7`; its changes were
unrelated skill additions and did not alter the reviewed library or plan files.
[Review evidence](evidence/primary-authoring-plan-review-2026-09-07.md) records
four actionable handoff gaps: APR-01 definition-only edits must preserve live data;
APR-02 map externally managed sources and worker/view lifetimes before runtime
consolidation; APR-03 retain Current/Presented export bases; APR-04 separate native
execution from portable capability validation. AUT-01/02/03/05/06/08/09, DAT, STM,
ARC, EXP and BND contracts are affected. These are plan gaps, not new demonstrated
runtime regressions. Overall architecture/reuse direction passes; the specific
handoff contracts remain Fail or Uncertain as detailed in the report.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1:

- `mise exec -- cargo metadata --no-deps --format-version 1 --locked`: succeeded;
  inspected nine workspace packages and direct dependency ownership.
- `mise exec -- cargo test -p chart-core --test streaming --test scheduling --test stage_cache --locked`:
  **16 passed**, zero failed (nine streaming, four scheduling, three cache).
- `mise exec -- cargo test -p chart-export --test live_export --test extensions --locked`:
  **9 passed**, zero failed (five live export, four extensions).
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown links; graph checks do not establish runtime support.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`:
  passed. An inline `mise exec -- python3 -` check passed new-report whitespace and
  the four APR finding identifiers; no feature acceptance was implied.

Result: uncommitted review report and this required ledger entry only. Plan,
implementation, fixtures, dependencies and prior evidence were preserved. No proposed
API compilation, actual Python/WASM proof, native/artifact inspection, Linux run or
sustained benchmark ran. Next action: tighten AP-00/01's ownership/edit/validation
handoff and AP-06's capture contract using APR-01–04; retain AP-00–09 and all open gates.

## Primary authoring API planning — 7 September 2026

Planning complete in the [primary authoring plan](impl_plans/primary-authoring-api-plan.md).
Owner direction: concise authoring is the main public API and every feature runs
through it. Per explicit instruction, the plan assumes original WP-01–23 complete;
this is a prospective input, not a change to their recorded implementation states.
Additional D3/GG work retains its scope/owners and must integrate through the primary
API when delivered. No historical gate or feature is certified by this assumption.

Specification/main-plan/migration versions advance together to 0.5.0. AUT-01–09,
FIX-AUTH00–09 and G-AUTH govern capability-complete authoring, a shared typed runtime,
data/identity, design, live interactions, native/export, bindings and migration.
[ADR-013](adr/013-primary-authoring-api.md) records accepted ownership: builders in
chart-core, existing normalized engine/runtime reused, host resources in adapters,
standalone utilities directly callable, no new façade crate or second compiler.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60` plus live edits. Result:
uncommitted plan/ADR and focused authority/traceability/ledger updates. Existing source,
reviews and parity plans were preserved. No dependencies, implementation, fixtures,
portable versions or release/publication settings changed. Those planning-only statements retain their original revision context. The current
package states below are updated by the implementation evidence above.

| Package | State | Prerequisites | Next action / evidence |
| --- | --- | --- | --- |
| AP-00 — Public contract and baseline register | COMPLETE | Assumed completed original WP-01–23 | Contract/register and migration policy in primary-authoring-api.md; FIX-AUTH00. |
| AP-01 — Shared typed runtime | COMPLETE | AP-00 | Typed owned/external Chart, independent worker caches and legacy forwarding qualified; E1/E3/E4. |
| AP-02 — Primary data/plot/layer path | COMPLETE | AP-01 | Ordinary Data/Plot route, exact ownership/diagnostics and actual native/publication examples qualified; E1/E4. |
| AP-03 — Complete grammar and aesthetics | COMPLETE | AP-02, applicable semantic owners | All delivered grammar/scale/extension families qualified through primary authors; E2. |
| AP-04 — Design/composition/specialists | COMPLETE | AP-03, applicable semantic owners | Facets/themes/text/composition and retained standalone helpers qualified; E2/E4. |
| AP-05 — Live Chart features | COMPLETE | AP-04 | 23 action, 47 input and 70 streaming steps, edits/rollback/replay and native scheduling qualified; E1/E3/E5. |
| AP-06 — Native/Kit/export integration | COMPLETE | AP-05 | Actual native/Kit input hooks, capture matrix, deferred resources and inspected SVG/PDF/PNG qualified; E4. |
| AP-07 — Host-native Python/WASM authoring | COMPLETE | AP-06 | Actual Rust/Python/WASM family/runtime proofs, typing, exact values, disposal/detachment/memory and compatibility pass; E2/E3/E5. |
| AP-08 — Consumer/docs/API migration | COMPLETE | AP-07 | Primary production consumers/docs and version-1 compatibility migration pass; E4/E5. |
| AP-09 — Full coverage and requalification | IN PROGRESS | AP-08, all target-release capability acceptance | Full register/platform proof complete; final visible native performance rerun pending; E5 / G-AUTH OPEN. |

Planning validation executed in `/Users/jeickmeier/Projects/finstack-chart`,
7 September 2026, Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links. Target graph checks are not runtime proof.
- `mise exec -- python3 /tmp/check_chart_authoring_plan.py`: passed ten package/fixture/
  ledger rows and an acyclic sequence, nine normative/traceability IDs, fifteen routing
  families, open-gate/assumed-baseline wording, synchronized 0.5.0 authority versions,
  and new-plan/ADR links and whitespace. This temporary documentation checker is not
  a feature acceptance runner; its compact requirement-ID parsing was corrected before
  the passing run.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md docs/alpha-api.md`:
  passed, retaining the repository's Markdown hard-break convention. The temporary
  checker also checks the untracked plan and ADR, which Git's tracked diff omits.

G-AUTH remains OPEN. No Rust runtime tests, actual Python/WASM execution, native/export
inspection, sustained performance run or proposed-API compilation was performed for
this documentation-only task. Next implementation action: AP-00 under the stated
completed-baseline assumption. Detailed acceptance commands are the existing scoped
and final repository gates; the plan specifies intended evidence, not results.

## Phase 2 parity implementation planning — 7 September 2026

Planning complete in the [secondary implementation plan](impl_plans/phase-2-parity-implementation-plan.md).
Scope: consolidate all eight D3 module plans and the ggplot2 review into Phase 2,
with shared ownership, prerequisites, full capability coverage and intended evidence.
Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; result is uncommitted
documentation. Existing source, review and planning changes are preserved. No runtime,
dependency, fixture baseline or portable envelope version changes were made by this task.

Specification/main-plan/migration document versions advance together to 0.4.0.
GG2-01–12 explicitly adopt ggplot2 capability scope, including models/density,
coordinates/geography, mathematical text and saving devices. D3 package inventories
remain in their existing plans. P2-00 owns integration coordination; GG-00–19 own the
new ggplot2 slices and FIX-GG00–19. CLR-04 is now an explicit SP-04 prerequisite because
the latter consumes its descriptors and paint lowering. Historical WP/G2 evidence is
unchanged; WP-16 remains under its active assignment and WP-17–20 remain prerequisites
for final integration. G-PARITY joins G3, all eight D3 gates and G-GGPLOT before the
existing WP-21–23 final certification. No implementation gate advanced.

At the original planning run, all new package owners were unassigned and implementation
evidence was absent. The table below incorporates the subsequent current API handoff
reconciliation; historical paragraphs above retain their original revision context.
READY permits the stated next action and does not certify the feature. Exact owned
work and FIX-GG cases are defined once in the plan.

| Package | State | Requirement scope | Evidence / next action |
| --- | --- | --- | --- |
| P2-00 — Integration contract and coverage register | COMPLETE | GG2-01/12, ARC-03/04, BND-01, QLT-02/05 | ADR-014 and phase-2-coverage.md; committed API baseline fab2505 reconciled. Reference locks/oracles remain with lane entries and GG-00. |
| GG-00 — Reference inventory and executable oracle | COMPLETE | GG2-01/12 | Pinned 643-export inventory, 57 R sources, 32 reproducible seed records/96 artifacts; [entry evidence](evidence/phase-2-oracles-2026-09-08.md). Semantic argument matrices remain with delivering packages. |
| GG-01 — Reconcile shared legend acceptance | COMPLETE | GG2-04, GRA-07, SCL-05, LAY-03, THM-03 | FIX-GG01: 24 cases, exact actual Python/WASM scenes, 216 exports and inspected PNG/PDF sheets; empty-guide and untitled defects repaired. Evidence: phase-2-entry-and-legends-2026-09-08.md. Full guides remain GG-05. |
| GG-02 — Compatibility profile, stages and inferred grouping | COMPLETE | GG2-01/02 | [Stage acceptance](evidence/phase-2-stages-2026-09-08.md); primary/binding proofs pass. Later ggplot2 families remain separate. |
| GG-03 — Independent aesthetic encodings | COMPLETE | GG2-03 | [Accepted](evidence/phase-2-aesthetics-2026-09-10.md): 104 R glyph records, 12 core regressions, actual hosts/update/publication/native, 487 macOS / 468 Linux tests and full check. |
| GG-04 — ggplot2 scale and palette policies | IN PROGRESS | GG2-03 | [In progress](evidence/phase-2-ggplot-scales-2026-09-10.md): 766 palette / 93 scale / 284 break / 120 expansion / 499 label / 32 numeric panel / 36 discrete range records and 54 categorical configurations pass; 18 style palette cases plus primary point/rule integration pass; 16 reverse-axis and 20 square-root panels, inverse-domain boundaries and left-closed histogram edge ordering pass; 30 fixed-time/DST-label and 16 calendar-width panels plus source-precision/budget/edit regressions pass; 36 affine and 16 custom secondary-axis cases pass, including reference guide rounding; 19 datetime-expansion and 16 paired numeric cases pass; 534 automatic datetime panels pass; 81 Date records in two source units and 12 Date-width panels pass; 445 full core tests/doctests and core Clippy pass. Duration selection/labels pass 45 break configurations and 75 primary/secondary panels. Identity mappings pass 122 R records, primary colour/linewidth charts and 192 alpha/after-scale grob paints; 464 full core tests/doctests and Clippy pass. Guide arguments pass 60 R discrete selection/label cases, 30 manual break/palette cases and 160 continuous candidate/label cases. Primary guide colors, labels, hidden guides and unchanged marks pass; all 471 core tests/doctests, Clippy and repository checks pass. Zero-row continuous and identity guide training additionally pass eighteen R records, JSON/retraining and primary empty-color checks; all 473 core tests/doctests and Clippy pass. All-nonfinite guide training additionally passes 192 reference cases, primary missing-color mark preservation, budgets and JSON/retraining checks; all 475 core tests/doctests, Clippy and repository checks pass. Binned candidates/labels additionally pass 288 R records and primary metadata/JSON/color checks; full core validation passes 477 tests/doctests, plus a subsequently added independent boundary-color test. Final Clippy and export-test compilation pass. Binned empty/nonfinite populations additionally pass 192 R records, JSON/retraining, budgets and primary empty/partial-limit checks; all 480 core tests/doctests, Clippy and repository checks pass. Time-width strings additionally pass 135 actual R Date/datetime/duration panels, wire round-trips, exact nanosecond and budget/override checks; all 482 core tests/doctests, Clippy, repository checks and export-test compilation pass. Explicit R date/time patterns additionally pass 220 reference labels, primary timestamp/label/JSON checks, supplied locale/offsets and retained D3 behavior; 40 primary R duration-format panels also pass, preserving original binary64 seconds; all 487 core tests/doctests, Clippy and repository checks pass, and export tests compile. Fresh Python/WASM formatter proofs each pass 246 publication cases and 14 explicit tab-glyph rejections; records and three PNGs match, and SVG/PDF/PNG samples are inspected. Multiline labels pass four axis sides; subsecond timezone windows retain resource coverage. Final full-core qualification passes all 487 tests/doctests and all-target core Clippy. Binned integer-count and finite-population transformed missing-limit boundaries additionally pass 228 R records and eight focused tests, including primary JSON/retraining checks. Final full-core qualification passes all 489 tests/doctests and all-target core Clippy. Transformed missing-limit population handling additionally passes 320 scale records and 24 actual R chart cases; 23 related tests, all 491 core tests/doctests and all-target core Clippy pass. Degenerate-domain handling additionally passes 256 R guide/mapping records and 192 primary chart cases, including zero logarithmic endpoints and reference missing-color defaults; 13 related tests, all 494 core tests/doctests and all-target core Clippy pass. The newer authored-limit extension passes 2,688 focused R cases, 120 related tests, all 495 core tests/doctests and all-target core Clippy; the hidden-guide/fractional extension passes all 499 core tests/doctests. Fresh Python/WASM each pass 2,304 matching records with three identical PNGs and inspected SVG/PDF/PNG samples. The final structural-validation fix passes 16 focused tests, all 500 core tests/doctests and all-target Clippy. Remaining arguments, other host paths and full acceptance are open. |
| GG-05 — Complete guides and legend composition | NOT STARTED | GG2-04 | Requires GG-01, GG-04, WP-AX04. |
| GG-06 — Bin/count/summary and position semantics | NOT STARTED | GG2-02/05 | Requires GG-02, SP-03, WP-S06. |
| GG-07 — Primitive and interval recipe completion | NOT STARTED | GG2-06 | Requires GG-03, GG-06, WP-S02/03/05. |
| GG-08 — Data-driven text, labels and annotations | NOT STARTED | GG2-06/09 | Requires GG-03, WP-P04. |
| GG-09 — Distributional and one-dimensional analytical layers | NOT STARTED | GG2-05/06 | Requires GG-06/07. |
| GG-10 — Smoothers, confidence bands and quantile regression | NOT STARTED | GG2-05/06 | Requires GG-06/07; spike model algorithms and dependencies. |
| GG-11 — Two-dimensional statistics and contours | NOT STARTED | GG2-05/06 | Requires GG-06/07/09. |
| GG-12 — Facet semantics and layout breadth | NOT STARTED | GG2-07 | Requires GG-02/05/06. |
| GG-13 — Cartesian, transformed and polar/radial coordinates | NOT STARTED | GG2-08 | Requires GG-07/12, WP-S04, WP-AX04. |
| GG-14 — Theme hierarchy and mathematical typography | NOT STARTED | GG2-09 | Requires GG-05/08/12/13. |
| GG-15 — Geographic layers and coordinates | NOT STARTED | GG2-08 | Requires GG-07/08/13. |
| GG-16 — Extensibility and authoring conveniences | NOT STARTED | GG2-10 | Requires GG-02/05/07/12/13. |
| GG-17 — Saving and device capability completion | NOT STARTED | GG2-11 | Requires GG-08/14/15 and WP-20. |
| GG-18 — Full grammar, update and host integration | NOT STARTED | GG2-02–12 | Requires GG-03–17 and WP-16–20. |
| GG-19 — Capability certification and handoff | NOT STARTED | GG2-01–12 | Requires GG-00–18 and all eight D3 certification packages; then G-GGPLOT/G-PARITY and WP-21/22. |

Planning evidence: all nine review/plan documents, normative contracts and selected
ADR/validation boundaries read; official ggplot2 4.0.3 index, tagged namespace,
aesthetic-stage, smoothing and saving references checked. No R oracle or D3 reference
suite ran. New GG-only effort is provisionally 182–315 engineer-days plus 2–4 for
P2-00, excluding shared D3, Phase 1 and final hardening; re-estimation gates are in
the plan. This is a scope-based allowance, not a delivery promise.

Validation executed in `/Users/jeickmeier/Projects/finstack-chart` on 7 September 2026,
Darwin arm64, mise Python 3.14.6:

- `mise exec -- python3 scripts/check_repository.py`: passed workspace/dependency
  isolation and local Markdown file links. Resolved target graphs are not platform
  runtime execution.
- `mise exec -- python3 /tmp/check_chart_phase2_plan.py`: passed 94 package nodes plus
  12 gate nodes with no cycles/unresolved dependencies, eight D3 inputs/backlinks,
  eight GGP findings, 20 GG package/fixture/ledger entries, 12 normative/traceability
  IDs, 231 local file/heading links, new-plan whitespace and 182–315 day arithmetic.
  This is a temporary documentation consistency check, not a feature acceptance runner.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving the
  established Markdown hard-break convention. New-plan whitespace was checked above
  because untracked files are outside `git diff --check`.

No Rust feature tests, native UI/artifact inspection, Python/WASM runtime, Linux or
performance gates were run for this planning-only assignment. Next implementation:
GG-01's current legend defect; next parity-program entry: P2-00, then GG-00 and D3
entry packages. The linked plan and these ledger rows are the retained planning result.

## ggplot2 feature parity review — 7 September 2026

Review complete against the official ggplot2 reference displaying version 4.0.3;
[full comparison and ranked findings](evidence/ggplot2-parity-review-2026-09-07.md).
Reviewed base `127fe2d4853f89b62ba59a17248485e3c378ba60` plus the concurrent working
tree. Result is uncommitted review evidence; existing inspection/state/category-window
and D3 planning edits are preserved. No production code, fixtures, baselines,
dependencies or normative parity requirements were changed by this review.

**Full ggplot2 parity is not established.** Basic Cartesian grammar/publication is a
tested subset. Missing statistical families, independent aesthetic scales, full
guides/facets/coordinates and deliberate binning/grouping/size differences remain.
GGP-01 reproduces a current defect: a non-faceted colored scatter prepares a two-entry
legend but its final scene contains no legend title. GGP-02–08 distinguish parity
gaps and absent evidence from violations of current scope. Relevant existing IDs:
GRA-01–08, SCL-01/05, LAY-01/03, THM-03, ARC-03, QLT-02/03, BND-03/04 and SCP-03.
GGP identifiers are review findings, not new normative requirements.

Evidence from `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust `1.97.1 (8bab26f4f 2026-07-14)`:

- `mise exec -- cargo test -p chart-core --test grammar --test statistics --test full_scales --test facets --locked`:
  **47 passed, 0 failed** (20 grammar, 14 statistics, 4 full_scales, 9 facets).
- `mise exec -- cargo test -p chart-export --test composition --test publication --test extensions --locked`:
  **20 passed, 0 failed** (6 composition, 10 publication, 4 extensions).
- A Rust scene probe confirmed missing single-panel legends and the explicit grouping
  requirement for category-colored lines. Retained [source](evidence/ggplot2-parity-2026-09-07/probe.rs)
  and [command/output/core binary hash](evidence/ggplot2-parity-2026-09-07/probe.log).
  Its faceted positive control painted the legend successfully.
- `mise exec -- python3 scripts/check_repository.py` passed workspace/dependency
  isolation and local Markdown links. Ledger whitespace and an inline new-report
  whitespace/finding-ID/count check passed; commands are retained in the review.

Tests certify the binaries built during this review, not subsequent concurrent edits.
No fresh R/ggplot2 oracle, Python/WASM runtime, native UI/image inspection, Linux,
performance or full matrix ran. Existing G2 evidence retains its stated scope;
no gate was advanced. Next action: fix GGP-01, then select a bounded ggplot2 capability
target and explicit compatibility policies while reusing the planned D3 foundations.

## D3 interpolation parity planning handoff

Planning is complete in [the interpolation plan](impl_plans/d3-interpolate-parity-plan.md).
The implementation does **not** have d3-interpolate parity. ITP-01–08/FIX-I01 and
G-INTERPOLATE are required by specification 0.3.0. This is uncommitted documentation
on starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; existing action/binding
changes and concurrent shape/scale/axis/color/chromatic/path planning are preserved.
CLR-02/03 own color parsing/conversion; WP-IP owns interpolation; SP owns normalization;
CP owns named ramps. No implementation, fixture, dependency or wire version was changed
by this assignment. Next interpolation package: **WP-IP01** after the accepted WP-14.

| Interpolation package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-IP01 — Contract and reference harness | COMPLETE | Core | ADR-017, 370 Node and 36 pinned Chromium cases, exact regeneration | ITP-01/08 | Requires WP-14; reconcile pinned export/source inventory and share the reference lock with color/scale/chromatic/axis. |
| WP-IP02 — Scalar kernels and composition | COMPLETE | Core/hosts | 56 reference cases, independent contracts and actual host proofs pass | ITP-02/03 | Requires WP-IP01. |
| WP-IP03 — Structured values | COMPLETE | Core/hosts | 191 reference cases, target dispatch matrix, ownership and actual hosts pass | ITP-01/02/07 | Requires WP-IP02; color dispatch joins WP-IP04 in integration. |
| WP-IP04 — Color interpolation | COMPLETE | Core/hosts | 106 reference cases, exact CSS, independent anchors and actual hosts pass | ITP-04 | Requires WP-IP02 and CLR-03; consumes the shared color engine. |
| WP-IP05 — Transform and zoom interpolation | COMPLETE | Core/hosts | 17 zoom and 36 pinned browser cases plus actual hosts pass | ITP-05/06 | Requires WP-IP02. |
| WP-IP06 — Portable and chart integration | COMPLETE | Core/hosts/publication | [Integration proof](evidence/phase-2-interpolation-integration-2026-09-09.md): registered factories, shared scale/mark/guide consumers, actual Rust/Python/WASM, three inspected native/publication states and retained updates | ITP-07 | Version 2 standalone / version 12 registered charts; native-only export rejection and registry snapshot lifetime qualified. |
| WP-IP07 — Parity certification | COMPLETE | Cross-lane acceptance | [27-export verdicts](evidence/phase-2-interpolation-integration/verdict-catalog.md), [axis consumer](evidence/phase-2-axis-certification-2026-09-09.md), fresh host replay and finite benchmark | ITP-01–08 | G-INTERPOLATE passes for the declared typed profile; release gates remain separate. |

## D3 path parity planning handoff

The [path gap review and delivery plan](impl_plans/d3-path-parity-plan.md) is complete
as documentation, reviewed at `1cb955740c2dad2607b0a2330201125294cab5d0` plus the live
working tree. The specification now requires PTH-01–06, FIX-P01–06 and G-PATH.
**Implementation does not have d3-path parity.** Missing capabilities include the
standalone builder/serializer, arcs/arcTo, signed rectangle subpaths, D3 state behavior
and precision controls. WP-P01–04 own the common foundation consumed by WP-S01;
their provisional 9–16 developer-days overlap the shape estimate. WP-S01 completion
now requires WP-P04; WP-21 and G4 require G-PATH. Existing completed packages retain
their recorded scope; concurrent action and other parity work is preserved.

Evidence: 7 September 2026, Darwin arm64, Rust 1.97.1, Node v24.14.0, working directory
`/Users/jeickmeier/Projects/finstack-chart`. Compared live sources with official docs
and pinned d3-path 3.1.0 source/exports/tests/manifest. Executed 14 exploratory sequences
and two invalid-digit cases against the pinned upstream source; its hash and findings
are in the plan. Temporary downloads are not a retained parity corpus; WP-P01 owns it.
The two commands below each passed **1 test, 0 failures**:

```sh
mise exec -- cargo test -p chart-core --test contracts numeric_paths_preserve_segments_and_reject_invalid_subpath_order --locked
mise exec -- cargo test -p chart-export --test publication plain_text_xml_escaping_curves_and_empty_clips_are_supported --locked
```

These validate existing behavior, not D3 differential parity or visual fidelity.
No Rust implementation, dependency, baseline or wire version changed in this assignment.
No new path fixtures, native/export image inspection, binding parity, Linux execution
or performance evidence was produced. All PTH requirements and G-PATH remain open.
Next action within the path assignment: **WP-P01 — Contract and reference corpus**.

Documentation checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace/dependency isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (retains existing Markdown
hard-break spaces). These checks do not execute the declared target platforms.
An inline `python3 -` consistency check also passed six PTH requirement rows, six
FIX-P fixtures, four path package rows, gate/traceability links, 12 acyclic path/shape
packages and path-plan whitespace/anchors. Result: uncommitted documentation, with
implementation and all path acceptance gates still open.

| Path package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-P01 — Contract and reference corpus | COMPLETE | Core | ADR-015; pinned 86-sequence corpus and request/result DTO | PTH-01–06, ARC-04, QLT-02 | Source/license hashes, independent expectations and standalone consumers retained; G-PATH remains open. |
| WP-P02 — Checked builder and complete geometry | COMPLETE | Core | Seven focused path tests; actual three-host traces | PTH-01/02/03/05 | All operations, owned numeric results, atomic failures, independent geometry and explicit budgets; renderer gate remains P04. |
| WP-P03 — Shared SVG path output | COMPLETE | Core/export | FIX-P04 sequence/precision checks; actual representative SVG | PTH-01/04 | Shared formatter; analytic retained arcs; legacy path output defaults preserved. |
| WP-P04 — Renderers, portable APIs and acceptance | COMPLETE | Core/native/export/bindings | [Path acceptance](evidence/phase-2-paths-2026-09-08.md), retained hashes/artifacts | PTH-01–06 | FIX-P01–06 actual three-host proof, native/publication inspection and macOS/Linux checks pass. |


## D3 color parity planning handoff

The [color review and plan](impl_plans/d3-color-parity-plan.md) is complete as
uncommitted documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`.
Current byte colors/palettes do **not** establish d3-color parity. Specification 0.3.0
now also requires COL-01–06, CLR-01–05, FIX-C01 and G-COLOR. Historical WP-11/13/14
and G2 evidence retains its original scope. Active WP-15 and concurrent scale/shape/
axis/chromatic changes are preserved. CLR identifiers and FIX-C01 avoid CP/FIX-21
catalog ownership collisions. SP-04 requires CLR-03's color kernels; CLR-05 requires
SP-04 and WP-20, without depending on SP-07 or CP-05. WP-21/22 require CLR-05.

Evidence: 7 September 2026; `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026). Official d3-color documentation and retrieved
source compared with live core, themes, grammar, wire, host and export code. Target
module 3.1.0; tagged Lab source fetch failed, so main-branch Lab was inspected and
CLR-01 must verify it against the pinned tarball. No D3 oracle was executed.
`mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`:
**1 passed, 0 failed, 3 filtered out**, proving only the existing palette subset.
`mise exec -- python3 scripts/check_repository.py` passed workspace edges,
host isolation and local Markdown file links (graph inspection, not target execution).
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed, preserving the
existing Markdown hard-break convention. Inline `python3 -` consistency checks passed:
six COL definitions/traceability rows, five CLR plan/ledger packages, FIX-C01/G-COLOR,
WP-21/22 dependencies, shared interpolation ownership and 61 acyclic package nodes.
New color-plan whitespace passed separately. Concurrent interpolation planning was
also reconciled: WP-IP04 consumes CLR-03, and SP-04 consumes WP-IP04. No implementation, fixtures,
dependency or wire version changed. No fresh color binding runtime, native/export
artifact inspection, Linux or performance proof was produced. All COL requirements
remain open. Next within this plan: **CLR-01 — contract and reference oracle**.

| Color package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CLR-01 — Contract and reference oracle | COMPLETE | Core | 351-case pinned oracle, deterministic regeneration; ADR-016 | COL-01–06, ARC-04, BND-01, QLT-02 | WP-14 prerequisite met; pin complete source/oracle and freeze typed/wire/paint contracts. |
| CLR-02 — RGB/HSL, parsing and common operations | COMPLETE | Core | Actual Rust/Python/WASM FIX-C01 and type proofs; see foundation report | COL-01–04 | Requires CLR-01. |
| CLR-03 — Lab, HCL/LCh and Cubehelix | COMPLETE | Core | Full conversion/method corpus and Linux regression pass | COL-02–04 | Requires CLR-02; supplies SP-04 color kernels. |
| CLR-04 — Authoring, portable operations and paint integration | COMPLETE | Core/hosts | [Qualified paint snapshot](evidence/phase-2-paint-2026-09-09.md): every input, migration, actual hosts/native/publication, retained live updates | COL-05, BND-01/03/04, THM-01/02/03, SCN-03 | CLR-05 awaits SP-04. |
| CLR-05 — Integrated parity acceptance | COMPLETE | Core/hosts/native/export | [Integrated evidence](evidence/phase-2-color-acceptance-2026-09-09.md) | COL-01–06, QLT-02/03/04, SCN-04 | G-COLOR passes for declared typed snapshot; release gates remain separate. |

## D3 scale-chromatic parity planning handoff

The [chromatic review and plan](impl_plans/d3-scale-chromatic-parity-plan.md) is complete
as documentation. Starting revision: `1cb955740c2dad2607b0a2330201125294cab5d0`; outcome:
uncommitted documentation, including specification/implementation/migration version
0.3.0, CHR-01–06, CP-01–05, FIX-21 and G-CHROMATIC. The implementation has raw palettes
and linear RGB interpolation, **not full scale-chromatic parity**. All six CHR requirements
remain open. The previous scale plan's catalog exclusion is replaced with explicit
chromatic ownership; SP-04 retains shared RGB/Cubehelix and scale algorithms. No source,
fixture, dependency or wire version was changed by this planning assignment. Existing
WP-15 and shape/scale/axis changes were preserved. Historical G2 acceptance is unchanged.

Evidence: 7 September 2026, `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
configured Rust 1.97.1. Official docs and pinned d3-scale-chromatic 3.1.0 source were
compared with live core color, compiler, legend, theme and portable host paths.
Read-only Python/urllib source inventory assertions passed: **76 exports, 38 schemes,
38 interpolators, 218 discrete arrays**, every export name present in the plan.
Source retrieval succeeded using network-enabled read-only execution after sandbox DNS
failure. This inspected source; it did not execute D3 or generate a numerical oracle.

Checks run:

- `mise exec -- cargo test -p chart-core --test full_scales points_and_colors_have_declared_missing_and_domain_policy --locked`: **1 passed**, 0 failed, 3 filtered out. Existing custom palette behavior only.
- `mise exec -- python3 scripts/check_repository.py`: passed workspace/host isolation and local Markdown file links; platform graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, preserving existing Markdown hard breaks.
- Inline Python documentation assertions: six normative CHR IDs and traceability rows, five CP plan/ledger packages, aligned 0.3.0 document headers and release dependencies checked.

No fresh chromatic differential, Python/WASM execution, native/export inspection,
Linux or performance evidence was produced. Estimated incremental effort is 9–16
engineer-days, excluding shared SP work; re-estimate after CP-01. Next within this
assignment: **CP-01**, reference oracle/compatibility contract, followed by CP-02's exact
catalog. CP-03 requires SP-04, and CP-05 requires SP-07/WP-20 before WP-21/22.

| Chromatic package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| CP-01 — Reference contract and oracle | COMPLETE | Core/reference | [Entry](evidence/phase-2-chromatic-entry-2026-09-09.md) | CHR-01/03/05/06, ARC-04 | All 76 exports and reference provenance pinned. |
| CP-02 — Exact discrete catalog | COMPLETE | Core | [Foundation](evidence/phase-2-chromatic-foundations-2026-09-09.md) | CHR-01/02 | All 218 actual arrays exact. |
| CP-03 — Complete interpolator catalog | COMPLETE | Core | [Foundation](evidence/phase-2-chromatic-foundations-2026-09-09.md) | CHR-01/03 | All 38 ramps and 160,666 sampled rows. |
| CP-04 — Chart, guide and portable integration | COMPLETE | Core/hosts | [Integration](evidence/phase-2-chromatic-integration-2026-09-09.md) | CHR-04/05, SCL-03, THM-02, BND-01/03/04 | v6 metadata, actual hosts, composition/themes and exceptional normalization pass. |
| CP-05 — Integrated certification | COMPLETE | Core/native/export/hosts | [G-CHROMATIC](evidence/phase-2-chromatic-integration-2026-09-09.md) | CHR-01–06, QLT-02/03/04 | Qualified snapshot; WP-21/22 release rechecks remain. |

## D3 hierarchy parity planning handoff

The [hierarchy review and delivery plan](impl_plans/d3-hierarchy-parity-plan.md) is
complete as documentation. **Implementation parity is absent**: HIR-01–06 have no
hierarchy engine/layout implementations; HIR-07/08 integrated runtime, renderer and
performance evidence remains unverified. Specification 0.3.0 now requires HIR-01–08,
FIX-H01-A–H and G-HIERARCHY. WP-H08 precedes WP-21/22 acceptance. Historical G2 and
completed Cartesian packages retain their original scope.

Review context: 7 September 2026, Darwin arm64,
`/Users/jeickmeier/Projects/finstack-chart`; started at `1cb9557` with live action and
parity-plan edits. HEAD advanced independently to `127fe2d` during review; this handoff
is uncommitted documentation on that working tree. Source and other planning changes
were preserved. The review compared the official D3 hierarchy documentation and pinned
3.1.2 exports/construction/stratify/treemap/packing source with core, grammar, layout,
scene, provenance and portable interfaces. It inventories all 16 exports, including
`Node`, all methods/controls, reference adaptations and discriminating planned cases.

No hierarchy code, wire version, dependencies or baselines changed. No Rust feature
tests, D3 differential runner, actual binding proofs, native/export visual inspection,
Linux execution or benchmarks ran for this task. Checks below validate documentation
only. G-HIERARCHY and all hierarchy implementation packages remain open.
Next within this assignment: **WP-H01**, then WP-H02; existing action and other parity
assignments retain their own next steps.

Planning validation passed:

- `mise exec -- python3 scripts/check_repository.py`: workspace edges, host isolation,
  pinned GPUI identity, optional Kit and local Markdown file links. Target graph checks
  are not target runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: no whitespace errors,
  preserving existing Markdown hard breaks.
- Inline `python3 -` consistency check: eight normative HIR IDs, eight package rows in
  the hierarchy plan and ledger, eight fixture subsets, all 16 reference export names,
  main-plan traceability, WP-21/22 dependencies, removed hierarchy deferrals, local
  hierarchy heading links and new-plan whitespace passed.

| Hierarchy package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| WP-H01 — Contract and reference harness | COMPLETE | Core contract/reference | [Entry evidence](evidence/phase-2-hierarchy-entry-2026-09-09.md): 16 exports, all methods/defaults, 441 reproducible cases, ADR-022 | HIR-01–08, ARC-04, BND-01, QLT-02 | Seed harness only; production/runtime certification remains H02–08. |
| WP-H02 — Topology, stratification and operations | COMPLETE | Core hierarchy | [Topology evidence](evidence/phase-2-hierarchy-topology-2026-09-10.md): 31 pinned cases, deep/bounded native inputs and 3 passing contracts | HIR-01/02/07 | Standalone core scope; registered/host/chart acceptance remains H07/H08. |
| WP-H03 — Tidy tree and cluster | COMPLETE | Core hierarchy | [Layout evidence](evidence/phase-2-hierarchy-layouts-2026-09-10.md): 340 reference layouts and mode/callback invariants | HIR-03 | H07/H08 retain radial/presentation/host obligations. |
| WP-H04 — Partition | COMPLETE | Core hierarchy | [Layout evidence](evidence/phase-2-hierarchy-layouts-2026-09-10.md): 42 reference partitions and own-value slack | HIR-04 | H07/H08 retain icicle/sunburst/host obligations. |
| WP-H05 — Treemap and tilers | COMPLETE | Core hierarchy | [Kernel evidence](evidence/phase-2-hierarchy-kernels-2026-09-10.md): 217 cases, retained histories and six topology reset states | HIR-06 | Standalone/native callback scope; registered/host/chart acceptance remains H07/H08. |
| WP-H06 — Packing and helpers | COMPLETE | Core hierarchy | [Kernel evidence](evidence/phase-2-hierarchy-kernels-2026-09-10.md): 191 cases plus containment/non-overlap/budget checks | HIR-05 | Standalone/native callback scope; integration and measured release evidence remain. |
| WP-H07 — Grammar, portable API and presentation | COMPLETE | Core/hosts/native/export | [Final integration evidence](evidence/phase-2-hierarchy-integration-2026-09-10.md): nine chart contracts, external registered operations, typed actual hosts, native and SVG/PDF/PNG inspection | HIR-07, BND-01/03/04, SCN-04 | Complete within assignment; preserve source-qualified evidence for release rechecks. |
| WP-H08 — Integrated parity acceptance | COMPLETE | Shared parity acceptance | [65-row catalog](evidence/phase-2-hierarchy-integration/verdict-catalog.md), 898 oracle cases plus 27 controls per host, replay/disposal/resources, 474 macOS/455 Linux tests and repository checks | HIR-01–08, QLT-02/03/04 | G-HIERARCHY passes; benchmark workloads and high-fanout query follow-up handed to WP-22. |

## D3 scale parity planning handoff

The [scale review and implementation plan](impl_plans/d3-scale-parity-plan.md) is
complete as documentation at `1cb9557` plus uncommitted documentation changes.
The implementation does **not** have D3 scale feature parity. Specification 0.2.0
now requires SCL-06–08/FIX-20 and G-SCALE; SP-01–07 own the missing families,
behavioral compatibility, oracle and end-to-end evidence. WP-06/11 and G2 retain
their original 0.1.0 acceptance. Shape/axis plans and active WP-15 edits are preserved.
Scale tick/format algorithms have one owner; WP-AX02 consumes SP-03/05/06.

Review evidence: 7 September 2026, working directory
`/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, Rust 1.97.1.
Official D3 documentation and d3-scale 4.0.2 source compared with live scale,
axis/formatting and portable contracts. `mise exec -- cargo test -p chart-core
--test scales --test full_scales --locked` passed **13 tests, 0 failed**.
These verify existing contracts, not D3 parity. No fresh D3 differential, Python/WASM,
native/export, Linux or performance evidence was produced. No scale code, fixtures,
dependencies or wire version changed. Next within the scale assignment: **SP-01**.

Planning checks passed: `mise exec -- python3 scripts/check_repository.py`
(workspace edges, host isolation and local Markdown file links), and
`git -c core.whitespace=-blank-at-eol diff --check -- docs` (preserves the existing
Markdown hard-break convention). Graph validation is not platform runtime execution.

| Scale package | State | Owner | Revision/evidence | Requirements | Next action |
| --- | --- | --- | --- | --- | --- |
| SP-01 — Compatibility contract and oracle | COMPLETE | Core/reference | [Reference and gap report](evidence/phase-2-scale-reference-2026-09-09.md): 26 factories, 461 cases, 200 formats, four zones, complete method dispositions | SCL-06/07/08, ARC-04, BND-01, QLT-02 | Measurement only; G-SCALE OPEN. Proceed SP-02. |
| SP-02 — Continuous mapping and numeric families | COMPLETE | Core | [Numeric qualification](evidence/phase-2-numeric-scales-2026-09-09.md): 161 configurations, explicit nice, named axes/navigation, v5, 43 macOS and 20 Linux tests | SCL-01/02/03/06/07, DAT-05 | D3 ticks/formatters remain SP-05; host qualification SP-07. |
| SP-03 — Ordinal, band and point | COMPLETE | Core | [Categorical qualification](evidence/phase-2-categorical-scales-2026-09-09.md): all 130 cases, exact keys, named axes/dodge/update snapshots, 55 macOS and 30 Linux tests; repository checks pass | SCL-01/06/07, DAT-02 | Standalone host/native/performance qualification remains SP-07. |
| SP-04 — Interpolation, distribution and color | COMPLETE | Core | [Distribution qualification](evidence/phase-2-distribution-scales-2026-09-09.md): 160 pinned configurations, defaults, numeric styles, interval/midpoint guides, post-stat correction/zoom and one paint boundary; 66 macOS and 36 Linux tests; checks pass | SCL-03/06/07, GRA-03, BND-01 | Registered interpolation remains WP-IP06; standalone host/native/performance qualification SP-07/WP-21/22. |
| SP-05 — Numeric ticks, nice and formatting | COMPLETE | Core | [Numeric format qualification](evidence/phase-2-numeric-format-2026-09-09.md): 269 exact tick/label configurations, 200 standalone format cases, expanded 11,176 labels, nice/locale/empty-label publication; 99 macOS and 33 Linux tests; checks pass | SCL-07, LAY-01/02, THM-03 | Standalone host qualification remains SP-07; shared API consumed by WP-AX02. |
| SP-06 — UTC and explicit local calendars | COMPLETE | Core/hosts | [Calendar qualification](evidence/phase-2-calendar-scales-2026-09-09.md): 2,750 filtered intervals, 16,632 exact custom labels, 160 mapping configurations; same-revision Python/WASM and byte-identical DST SVGs; 52 macOS/50 Linux tests and repository checks pass | SCL-04/06/07, DAT-05, BND-01 | Shared API consumed by WP-AX02; public facade/type/native/performance integration remains SP-07. |
| SP-07 — Integrated parity proof | COMPLETE | Core/hosts/native/export | [Integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md) | SCL-01–08, BND-01/03/04, SCN-04, QLT-02/03/04 | G-SCALE passes for declared typed snapshot; release gates remain separate. |

## D3 shape parity planning handoff

The owner-requested [shape parity review and plan](impl_plans/d3-shape-parity-plan.md)
is complete as documentation at base `1cb955740c2dad2607b0a2330201125294cab5d0`
plus uncommitted documentation changes. Specification/implementation/migration documents
now use version 0.2.0 and require SHP-01–10, FIX-S01–09 and G-SHAPE before G4.
Existing WP-10/11/14 and G2 passes retain their original Cartesian scope. No D3 parity
implementation or runtime pass is claimed; all SHP requirements remain open.

Evidence: 7 September 2026, Darwin arm64, working directory
`/Users/jeickmeier/Projects/finstack-chart`; official D3 documentation and pinned
d3-shape 3.2.0 exports/source compared with the live implementation. Exact source
locations and missing contracts are in the linked plan.
`mise exec -- python3 scripts/check_repository.py` passed workspace/dependency isolation
and local Markdown file links. Inline `python3 -` inventory/dependency checks passed:
10 SHP IDs, nine supplemental fixtures, eight shape ledger packages and 31 acyclic
original-plus-shape package nodes at the time of this check.
`git -c core.whitespace=-blank-at-eol diff --check -- docs/spec/gpui-charts-specification.md docs/spec/gpui-charts-migration-plan.md docs/impl_plans/gpui-charts-implementation-plan.md docs/implementation-status.md`
passed; the default whitespace check flags these documents' intentional Markdown
hard-break spaces. The new shape plan has no trailing whitespace.
The review ran no Rust feature tests, D3 oracle, native visuals or binding
runtime comparisons. Concurrent WP-15 source and status edits were preserved.
Next within the shape assignment: WP-S01 reference fixtures and shared path foundation;
the ongoing WP-15 assignment below continues independently.

## Current handoff

**The owner's original WP-11 through WP-23 assignment is complete, with a separate
commit for every package. Expanded acceptance remains OPEN.** The owner explicitly
waived the 30-minute test; its interrupted trace and numerical failure remain retained.
No package was published and no distribution license was inferred.

WP-23 supplies the [developer/release guide](release-guide.md),
[65-requirement evidence index](release-evidence.md), changelog, package/source policy,
and deterministic unpublished local source-candidate tool. The index explicitly lists
75 additional identifiers from concurrent planning as OPEN and does not certify later
clauses added to original IDs. See [WP-23 evidence](evidence/wp-23-completion-2026-09-07.md).

Final runtime evidence is **219 macOS tests, 213 Linux headless tests**, required
fmt/check/lint/docs/WASM checks and actual Rust/Python/Node WASM fixtures (36 cases,
23 actions, 47 input steps, 70 stream steps, three density cases, 40 live-export steps).
The extracted committed runtime archive passes offline repository/isolation/link checks
and fresh headless all-target compilation. Repeated archives have identical SHA-256.
The final WP-23 archive/manifest is generated under `artifacts/local-release` after commit.

[WP-22](evidence/wp-22-completion-2026-09-07.md) records ten-line p95 hover/frame work
0.092666/11.689291 ms and visible short-stream actual ingest-to-display p95 165.759375 ms.
Exact accounting and concurrent publication drain/release pass. Heavy-dashboard timings,
failed normal-window display traces, allocator high-water and the duration/visibility
limits remain explicit. These results do not certify expanded workloads or current G4.

**Next owner work:** select the separately planned parity/primary-authoring lane and its
acceptance packages. Preserve the open OS accessibility/platform/distribution boundaries,
license metadata and dependency advisories. There is no active implementation package
left in this original assignment; concurrent owner edits remain separate.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap accepted; WP-03 capability follow-up complete; dependency maintenance/release disposition remain WP-08/23. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `435e127` | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | DONE | Unassigned | `3a86189` | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | [Completion evidence](evidence/wp-03-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md); [ADR-008](adr/008-benchmark-protocol.md) | Actual proof artifacts, native lifecycle/hooks and starting profile inspected; public integration/full fixtures remain later packages. |
| WP-04 — Immutable data, schemas and transactions | DONE | Unassigned | `d0a6c48` | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | [Completion evidence](evidence/wp-04-completion-2026-09-06.md); [ADR-004](adr/004-immutable-data-and-transactions.md) | Data portions accepted; full transform/selection behavior, queued ingestion/time retention and binding runtimes remain later packages. |
| WP-05 — Grammar compiler and minimal prepared scene | DONE | Unassigned | `3cf1b33` | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | [Completion evidence](evidence/wp-05-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Known data-space geometry/semantics accepted; scales/layout and destination scene projection are WP-06; complete grammar/extensions remain later packages. |
| WP-06 — Foundational scales, ticks and layout | DONE | Unassigned | `f8657fb` | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | [Completion evidence](evidence/wp-06-completion-2026-09-06.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Foundational scale/scene/text-layout contracts accepted; actual native/export consumers are WP-07/08, full families/typography/shared layout remain WP-11–13. |
| WP-07 — Working standalone GPUI vertical slice | DONE | Unassigned | `fac148a` | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | [Completion evidence](evidence/wp-07-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Standalone line/point/bar, native text, presented-snapshot inspection, resize and disposal accepted for FIX-01/07/18 subsets; complete interaction/indexing/accessibility remain WP-15–21. |
| WP-08 — Headless export and snapshot foundation | DONE | Unassigned | `fac148a` | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | [Completion evidence](evidence/wp-08-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Basic headless formats, explicit resources and minimal FIX-13/14 accepted; full composition/live exports remain WP-13/20. |
| WP-09 — Portable schema and executable binding proofs | DONE | Unassigned | `c5ec829` | BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01 | [Completion evidence](evidence/wp-09-completion-2026-09-06.md); [ADR-006](adr/006-portable-specification-and-binding-proofs.md) | Version 1 subset and actual Rust/Python/WASM FIX-15/16 accepted; extend builtin coverage through WP-10–14 and full parity at WP-21. |
| WP-10 — Complete statistical and position semantics | DONE | Unassigned | `c5ec829` | GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02 | [Completion evidence](evidence/wp-10-completion-2026-09-06.md); [contract](statistics-contract.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Built-in stats/positions and FIX-02–05 accepted; exact full-recompute fallback declared. Extend families in WP-11 and specialized streaming in WP-18. |
| WP-11 — Required scale and geometry families | DONE | Unassigned | `a6fb2ea` | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | [Completion evidence](evidence/wp-11-completion-2026-09-07.md); [contract](scale-geometry-contract.md) | Required families and FIX-01/03/07 scope accepted through native/export/actual bindings; proceed to shared layout in WP-12. |
| WP-12 — Facets, guides and shared layout | DONE | Unassigned | `63dbcc2` | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | [Completion evidence](evidence/wp-12-completion-2026-09-07.md); [contract](facet-layout-contract.md) | FIX-06 and facet/shared-layout scope accepted through core/native/export/actual bindings; full typography and composition remain WP-13. |
| WP-13 — Full themes and publication composition | DONE | Unassigned | `f6c41c1` | THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03 | [Completion evidence](evidence/wp-13-completion-2026-09-07.md); [contract](theme-typography-composition-contract.md) | FIX-12/13 accepted through actual core/native/export/bindings; cumulative G2 remains WP-14. |
| WP-14 — Extension contracts and alpha API | DONE | Unassigned | `1cb9557` | SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05 | [Completion evidence](evidence/wp-14-completion-2026-09-07.md); [contract](extension-contract.md); [alpha matrix](alpha-api.md) | FIX-17 and cumulative G2 passed; full reducer/interaction begins WP-15. |
| WP-15 — Complete action reducer and state ownership | DONE | Unassigned | `127fe2d` | INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01 | [Completion evidence](evidence/wp-15-completion-2026-09-07.md); [contract](state-action-contract.md); [ADR-007](adr/007-actions-gestures-and-controlled-state.md) | Deterministic action/controlled/gesture/history/lifetime scope accepted through actual native/export/Python/WASM. Input producers and full G3 remain WP-16–20. |
| WP-16 — Hit testing, navigation and selection | DONE | Unassigned | `4148793` | INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05 | [Completion evidence](evidence/wp-16-completion-2026-09-07.md); [contract](interaction-contract.md) | Assigned indexed inspection/navigation/selection acceptance passes; FIX-09/10 remaining portions and G3 stay with WP-17–20. |
| WP-17 — Linked views, editable annotations and host controls | DONE | Unassigned | `6779e44` | INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06 | [Completion evidence](evidence/wp-17-completion-2026-09-07.md); [contract](host-tools-contract.md) | Original linked/editing/host acceptance passes; native accessibility limitations recorded. Full G3 remains open. |
| WP-18 — Streaming retention and incremental computation | DONE | Unassigned | `30ca2e8` | DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01 | [Completion evidence](evidence/wp-18-completion-2026-09-07.md); [contract](streaming-contract.md) | Original queue/retention/incremental/follow acceptance passes; 70-step three-host replay and native lifecycle inspected. Sustained PERF and G3 remain open. |
| WP-19 — Bounded scheduling, caches and dense representation | DONE | Unassigned | `e57246d` | STM-04, STM-05, SCN-04, GPU-02, QLT-04 | [Completion evidence](evidence/wp-19-completion-2026-09-07.md); [contract](scheduling-density-contract.md) | Original bounded-worker/cache/density acceptance passes; actual four-chart progress and three-host dense proofs. Preliminary PERF identifies index/memory bottlenecks; intermittent native redraw question retained. |
| WP-20 — Coherent exports during live interaction | DONE | Unassigned | `9d86ef7` | EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04 | [Completion evidence](evidence/wp-20-completion-2026-09-07.md); [contract](live-export-contract.md) | Original coherent capture/bounded lifetime acceptance passes; actual 40-step three-host replay and finite native exports during 400 atomic commits. Sustained PERF and native hardening remain open. |
| WP-21 — Correctness, fidelity and supported-platform hardening | DONE (original scope) | Unassigned | `4c099ee` | SCP-03, QLT-01/02/03, GPU-02/03, BND-03/04, FIX-01–18 | [Completion evidence](evidence/wp-21-completion-2026-09-07.md); [ADR-009](adr/009-supported-platform-and-accessibility.md) | Original acceptance passes including fixed native frozen resize/redraw and actual macOS/Linux/headless bindings. Expanded parity/authoring acceptance remains open and requires the separately listed packages/gates. |
| WP-22 — Measured performance and sustained-load release gate | DONE (original scope; owner duration waiver) | Unassigned | `c58f5b2` | STM-01/03/04/05, EXP-03, QLT-04, PERF-01–05 | [Completion evidence](evidence/wp-22-completion-2026-09-07.md); [ADR-008](adr/008-benchmark-protocol.md) | Visible short budgets and exact accounting pass; interrupted/occluded failures and memory limits retained. Thirty-minute duration explicitly owner-waived. Expanded workloads and current G4 stay open. |
| WP-23 — Production documentation and release readiness | DONE (original local candidate scope) | Unassigned | Included in this completion commit | SCP-01/02/03, ARC-04, BND-01, QLT-05/06 | [Completion evidence](evidence/wp-23-completion-2026-09-07.md); [release index](release-evidence.md); [guide](release-guide.md) | All original package handoffs complete; source candidate remains unpublished. Expanded parity/authoring and current production G4 remain OPEN; duration waiver, failed evidence, platform/accessibility/metadata limits are explicit. |

## Supplemental shape work packages

The [shape plan](impl_plans/d3-shape-parity-plan.md#work-packages-and-dependency-order)
owns deliverables and acceptance criteria; planning approval is not implementation evidence.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-S01 — Shape contract, oracle and path foundation | COMPLETE | Core/reference/native/export/hosts | [FIX-S01 evidence](evidence/phase-2-shape-foundation-2026-09-09.md) | SHP-01, SHP-08, SHP-10 | 63 exports, 333 contexts, 42 layouts; actual hosts and inspected destinations | Proceed WP-S02; generator families remain open. |
| WP-S02 — Cartesian generators and complete curves | COMPLETE | Core/hosts/native/export | [FIX-S02/03 evidence](evidence/phase-2-shape-cartesian-2026-09-09.md) | SHP-02, SHP-03, SHP-09 | 20 curves, 829 cases; 369 macOS tests; actual hosts, 64 updates each and inspected destinations | Curved dash styling subsequently accepted in WP-S08. |
| WP-S03 — Arc geometry and pie layout | COMPLETE | Core/hosts | WP-S01 accepted; ADR-020 | SHP-04, SHP-09 | [620 arc/192 pie Rust cases, 144 built-in host pies, Linux/WASM interactions, 64 updates and publication](evidence/phase-2-shape-arc-2026-09-09.md); implementation complete | Fresh macOS Python, inspected native output and 393-test workspace/repository checks pass; proceed WP-S04. |
| WP-S04 — Radial generators and links | COMPLETE | Core/hosts/native/export | WP-S02 and WP-S03 accepted; ADR-020 | SHP-05, SHP-09 | [40 point/697 standalone and chart paths, 760 updates per host, exact inspected publication](evidence/phase-2-shape-radial-2026-09-09.md) | Fresh macOS/Linux Python and WASM budget regressions pass; figure-wide projection preflight includes facets/insets. Native inspected; 339 core + 54 export Linux tests, 404 macOS workspace tests/doctests and repository checks pass at recorded snapshots. Proceed WP-S07. |
| WP-S05 — Complete symbol encoding | COMPLETE | Core/hosts | WP-S01 accepted; ADR-020 | SHP-06, SHP-09 | [13 types, 156 fixtures per host, 104 updates per host, mapped guides and exact inspected publication](evidence/phase-2-shape-symbol-2026-09-09.md); Linux lint/docs pass | Fresh macOS Python, inspected native output and complete workspace/repository qualification pass. |
| WP-S06 — Complete stack layouts | COMPLETE | Core/hosts/native/export | WP-S01 and WP-10 accepted; ADR-020 | SHP-07, SHP-09 | [435 numerical cases per host, 1,620 tidy comparisons, 480 updates per host, exact inspected three-host publication and 385 Linux tests/doctests](evidence/phase-2-shape-stack-2026-09-09.md) | Fresh macOS Python, 480 updates, inspected native output and complete workspace/repository qualification pass. |
| WP-S07 — Custom protocols and public portability | COMPLETE | Core/hosts/native/export | WP-S02–06 accepted | SHP-01, SHP-08, SHP-09 | [Five registered protocols, chart/legend integration, wire-v9, 72 updates per host and exact inspected publication](evidence/phase-2-shape-custom-2026-09-09.md) | Native and all three publication themes inspected; actual Python/WASM, strict types and repository checks pass. Full regression follow-up recorded with the evidence; WP-S08 subsequently accepted. |
| WP-S08 — Integrated parity acceptance | COMPLETE | Core/hosts/native/export | WP-S07, WP-16, WP-18, WP-20 accepted | SHP-09, SHP-10 | [63 exports/220 methods accepted; retained curved dashes and complete cross-family qualification](evidence/phase-2-shape-acceptance-2026-09-09.md) | 420 macOS / 419 Linux tests-doctests, 1,544 updates per actual host, native and nine publication images inspected; G-SHAPE passes. Expanded WP-21/22/23 remain open. |

## Cumulative gates

### Required D3 axis packages

The [axis parity plan](impl_plans/d3-axis-parity-plan.md) adds AXIS-01–07/FIX-19 to
specification 0.2.0. Historical WP-06/11–14 and G2 evidence retains its original scope.
The axis review/planning assignment is complete; implementation parity remains open.
WP-21 additionally requires WP-AX06, and WP-22/23 include axis performance/release proof.

| Package | State | Owner | Revision | Requirements | Evidence / next action |
| --- | --- | --- | --- | --- | --- |
| WP-AX01 — Guide contract and reference harness | COMPLETE | Core/hosts | WP-14 accepted; reference entry | AXIS-01, AXIS-07 | [372 actual-browser reference cases and complete 4-factory/10-method inventory repeat exactly](evidence/phase-2-axis-entry-2026-09-09.md). Identity, shared-scale guide resolution, primary builders/edits/name maps and version-8 migration implemented; five guide tests plus 39 existing core tests pass on Linux; actual Linux Python/WASM prove shared placement, named navigation, stable scale replacement and retained outputs. Strict positive types and macOS repository/workspace checks pass. Dedicated macOS Python and inspected native guide identity checks pass. [Registered provider acceptance](evidence/phase-2-axis-provider-2026-09-09.md) passes 21 focused macOS/Linux tests, actual macOS/Linux Python and WASM, strict declarations and byte-identical publication output plus inspected native output. [Integrated acceptance](evidence/phase-2-axis-ticks-2026-09-09.md) completes the explicit profile entry and retains final regression/validation limits. Complete geometry remains AX03. |
| WP-AX02 — Tick selection and formatting | COMPLETE | Core/hosts | Working tree over `51f2eda`; retained source hashes | AXIS-02, AXIS-03 | [Integrated acceptance](evidence/phase-2-axis-ticks-2026-09-09.md): 372 reference cases / 376 states per actual Rust/Python/WASM host, independent reset/selection/formatting, exact timestamps, registered callbacks, strict types, byte-identical publication and inspected native/SVG/PDF/PNG. 430 macOS and 430 Linux tests/doctests; final focused tests pass. Stop before AX03 as requested; aggregate-check limits remain explicit. |
| WP-AX03 — Axis geometry and bounded layout | COMPLETE | Core/hosts/native/export | WP-AX02 accepted | AXIS-04 | [Geometry acceptance](evidence/phase-2-axis-geometry-2026-09-09.md): 437 macOS tests, 372 pinned cases/376 states in Rust/Python/WASM; signed geometry, policies, facets and native/publication inspection. |
| WP-AX04 — Styling and publication components | COMPLETE | Core/hosts/native/export | WP-AX03 accepted | AXIS-05 | [Component acceptance](evidence/phase-2-axis-components-2026-09-09.md): v14 styles/roles, per-tick typography, 443 macOS tests, 26 matching three-host artifacts and inspected native/text-outline/high-DPI publication. |
| WP-AX05 — Axis updates and transitions | COMPLETE | Core/native/hosts/export | [Timed/native/capture proof](evidence/phase-2-axis-transitions-2026-09-09.md): 125 reference samples, 75 host artifacts, 449 macOS/448 Linux tests | AXIS-06 | Native clock/interruption/reduced motion/disposal and exact displayed capture pass. |
| WP-AX06 — Parity certification and documentation | COMPLETE | Integrated acceptance | [Capability matrix](evidence/phase-2-axis-certification-2026-09-09.md), 449 macOS/448 Linux tests, full repository check | AXIS-01–07 | G-AXIS passes for declared supported surface; no global release/FPS claim. |

### Gate results

| Gate | State | Evidence required next |
| --- | --- | --- |
| G0 | PASSED — architecture/capability scope | WP-01/02/03 evidence and ADRs establish the initial macOS route and starting protocol; this does not pass full requirements, FIX/PERF or release support. |
| G1 | PASSED — minimal portable core | WP-04–08 foundation/native/headless evidence plus WP-09 actual Python/WASM FIX-15/16 runtime comparison; this does not certify full grammar or production host/distribution products. |
| G2 | PASSED — Cartesian/publication alpha | [Alpha matrix](alpha-api.md) maps complete grammar/facets/themes/publication/extensions and actual portable evidence. G3/G4 retain their remaining scope. |
| G3 | PASSED (original interactive streaming scope) | WP-15–20 interaction/streaming/export plus WP-21 frozen-resize and native redraw regression fixes; actual native and Rust/Python/WASM evidence. Sustained PERF and expanded parity remain separate. |
| G-PATH | PASSED | PTH-01–06/FIX-P01–06; [WP-P04 source snapshot and evidence](evidence/phase-2-paths-2026-09-08.md). Shape and performance qualification remain separate. |
| G-AXIS | PASSED | AXIS-01–07/FIX-19; pinned reference matrix, actual bindings, native/publication inspection and transition evidence. Required before WP-21 and G4. |
| G-SHAPE | PASSED for finite typed snapshot | SHP-01–10/FIX-S01–09 through WP-S08; [integrated evidence](evidence/phase-2-shape-acceptance-2026-09-09.md). Complete per-item verdicts, actual hosts and inspected destinations; WP-21/22/23 remain open. |
| G-SCALE | PASSED for declared typed snapshot | SCL-01–08/FIX-20 through SP-07; [integrated evidence](evidence/phase-2-scale-integration-2026-09-09.md). WP-21/22 release rechecks remain open. |
| G-CHROMATIC | PASSED for retained typed snapshot | CHR-01–06/FIX-21 through CP-05; [final evidence](evidence/phase-2-chromatic-integration-2026-09-09.md). Other Phase 2/release gates remain open. |
| G-COLOR | PASSED for declared typed snapshot | COL-01–06/FIX-C01 through CLR-05; [integrated evidence](evidence/phase-2-color-acceptance-2026-09-09.md). WP-21/22 release rechecks remain open. |
| G-INTERPOLATE | PASSED | ITP-01–08/FIX-I01; all 27 exports/configuration/result controls, shared consumers, actual Rust/Python/WASM and applicable inspected native/publication evidence. WP-IP07 precedes WP-21/22. |
| G-HIERARCHY | PASSED for declared finite typed surface | HIR-01–08/FIX-H01-A–H through WP-H08; [complete method/history/host/native/publication/resource evidence](evidence/phase-2-hierarchy-integration-2026-09-10.md). Global WP-21/22/23 and PERF gates remain separate. |
| G-GGPLOT | NOT PASSED | GG2-01–12 / FIX-GG00–19 and GG-19 complete reference/host/destination capability evidence. |
| G-PARITY | NOT PASSED | G3, all eight D3 gates and G-GGPLOT; Phase 2 integrated capabilities before final WP-21/22 acceptance. |
| G4 | OPEN — local candidate only | G-PARITY and all required FIX/PERF/platform/accessibility/documentation evidence through WP-21–23, including D3 and ggplot2 supplemental workloads. |

## Axis planning evidence — 7 September 2026

Outcome: reviewed current axis implementation against D3 and added the parity plan,
normative requirements, fixture coverage, package dependencies and open gate. No axis
implementation changed. Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`;
result is uncommitted documentation. Existing action/state/binding edits and concurrent
shape planning were preserved. Source findings and external references are retained in
[the plan](impl_plans/d3-axis-parity-plan.md#evidence-from-this-planning-review).

Working directory `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026).
`mise exec -- cargo test -p chart-core --test scales --test layout --locked` passed:
29 tests (9 scales, 20 layout), zero failed. Existing behavior passes; these are not
D3 differential tests. `mise exec -- python3 scripts/check_repository.py` passed workspace
boundaries, dependency isolation and local Markdown file links. The default whitespace
check flagged the documents' existing Markdown hard-break convention; verification with
`git -c core.whitespace=-blank-at-eol diff --check -- docs` passed. An inline `python3 -`
consistency check passed seven normative axis IDs, six package rows in each of the axis
plan/main plan/ledger, requirement traceability, G-AXIS, shared-scale prerequisites and
the new plan's whitespace. These are documentation checks, not feature certification.
No new D3 fixtures, runtime binding proofs, native/export inspections, Linux execution
or performance evidence were produced. AXIS-01–07 and G-AXIS remain open.
Next within this assignment's plan: WP-AX01; existing WP-15 work continues independently.

## Interpolation planning evidence — 7 September 2026

Outcome: documented the complete 27-export d3-interpolate inventory, source-backed gaps,
ITP-01–08/FIX-I01, seven implementation packages and G-INTERPOLATE. Reconciled ownership
with the concurrent color/scale/chromatic/axis plans and preserved other live edits.
Starting revision `1cb955740c2dad2607b0a2330201125294cab5d0`; result is uncommitted
planning documentation. The [plan](impl_plans/d3-interpolate-parity-plan.md) retains
source links, retrieval limitations, compatibility decisions, estimates and intended
acceptance. No production implementation, dependency, schema or fixture was changed
by this task. ITP-01–08 and G-INTERPOLATE remain open; next action is WP-IP01.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --test scales --test full_scales --locked`:
  **13 passed, 0 failed** (9 foundational and 4 full-scale tests). These establish the
  existing subset only; they are not a D3 interpolation differential suite.
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries, host isolation and local Markdown file links; no platform runtime claim.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs`: passed, retaining the
  existing Markdown hard-break convention.
- `mise exec -- python3 /tmp/chart-interpolate-plan-check.py`: passed 27 export names,
  eight normative/traceability IDs, seven plan/ledger package rows, eight fixture groups,
  WP-21/22 dependencies, new-plan whitespace and an acyclic 30-node cross-lane model.
  This temporary planning check is not a committed feature runner or CI gate.

No D3 runtime/oracle regeneration, actual Python/WASM interpolation proof, native/export
inspection, Linux execution or performance measurement was run. Available reference
source included the upstream main export index and pinned zoom implementation; some
pinned source requests and shell DNS failed. WP-IP01 must reconcile and lock release
sources before committing acceptance fixtures. No absent evidence is recorded as a pass.

## Public API usability review — 7 September 2026

Outcome: reviewed the live public Rust authoring, data, native/export, documentation
and proof-binding surfaces against the owner's ggplot2-like simplicity goal. The
[review](evidence/public-api-review-2026-09-07.md) records six ranked findings,
source locations, current/proposed boundaries and concrete developer-workflow
acceptance criteria. Current usability fails that goal; tested existing grammar
contracts pass their scoped checks. Proposed authoring and developer usability remain
unverified. Scope: SCP-01, DAT-01/02, GRA-01/07, THM-01, ARC-01/02, EXP-01/02,
BND-03/04, QLT-05 and GG2-02/03/07/09/10/11. Existing minimal bindings are not
reclassified as failed proof adapters merely because their authoring is low-level.

Starting revision `127fe2d4853f89b62ba59a17248485e3c378ba60`; reviewed existing
uncommitted work. Result: uncommitted review report and this ledger entry only;
implementation, defaults, fixtures and unrelated edits preserved. The root repository
instruction to record every review outcome governs this ledger update.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64,
Rust 1.97.1 (`8bab26f4f`, 14 July 2026):

- `mise exec -- cargo test -p chart-core --doc --locked`: **4 passed**, zero failed
  (two runnable examples and two intended compile failures).
- `mise exec -- cargo test -p chart-core --test grammar --test facets --locked`:
  **29 passed**, zero failed (20 grammar, nine facets).
- `mise exec -- python3 scripts/check_repository.py`: passed repository/dependency
  boundaries and local Markdown file links; target graphs are not runtime execution.
- `git -c core.whitespace=-blank-at-eol diff --check -- docs/implementation-status.md`
  and `git -c core.whitespace=-blank-at-eol diff --no-index --check -- /dev/null docs/evidence/public-api-review-2026-09-07.md`:
  passed whitespace checks, retaining the existing Markdown hard-break convention.

No proposed API was implemented or compile-certified. No actual Python/WASM proof,
native/export inspection, differential reference suite, performance gate or user
study ran. GG2 and G4 remain open. Next action: prioritize a bounded plot/data/layer
authoring slice and define compiling everyday examples early in the existing parity
plan; semantic prerequisites and legacy defaults must be preserved.

## Evidence updates

Before ending every task, including reviews and partial or blocked slices, update this
ledger with its outcome and evidence. Record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
