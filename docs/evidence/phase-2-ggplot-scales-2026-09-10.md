# GG-04 progress — 10 September 2026

Status: IN PROGRESS. Starting revision `a6caa39`; changes remain in the working tree.
GG-03, SP-06, CP-04 and CLR-04 prerequisites are accepted. This record covers the
current GG2-03/FIX-GG04 scale-policy slice and does not close GG-04 or G-GGPLOT.

## Binned mapping and registered label order — 13 September 2026

At `6e74ae6` plus working-tree changes, pinned `GuideBins$extract_key` and
`GuideColoursteps$extract_key` map before calling `get_labels`. The explicit
selection adapter previously called registered labels first. It now maps bins'
midpoints before cut/endpoint labels and stepped cuts before cut labels, followed
by midpoint mapping. Existing binned candidate APIs keep their previous contract.

`tools/reference/r/binned-guide-label-pipelines.R` captures 108 actual ggplot2 4.0.3
builds/draws under R 4.6.1 in `fixtures/parity/ggplot2/binned-guide-label-pipelines.json`:
three channels, two guide classes, three populations, two rescalers and three
label callbacks. `binned_registered_labels_match_108_draws` matches complete
successful callback order and values, retained key labels/mapped values and mark
output, and reproduces all 24 errors (84 successes). The original 432 guide
selections remain covered. All 23 native tests in `ggplot_pipeline_functions`,
`ggplot_binned_numeric`, `ggplot_binned_guides` and `ggplot_binned_label_functions`
pass (`/tmp/ggplot-binned-registered-regressions.log`), with focused strict Clippy
passing (`/tmp/ggplot-binned-registered-clippy.log`).

Fresh Linux Python and wasm32/Node builds run the shared pipeline proof with
`binned --guide-label-pipelines`; each passes 268 states: 192 original/restored-and-
edited success/error records, 72 indexed/missing-label replacements versus fresh
batch with immutable held publications, and four rejection cases. Actual logs:
`/tmp/ggplot-binned-registered-{python,wasm-build,wasm-host}.log`. Exact JSON and
54 SVG/PDF/PNG bytes pass at `target/ggplot-binned-registered/comparison.json`.
All 54 files exactly match inspected publications from the bins/steps slice,
recorded in `target/ggplot-binned-registered/inspection-matches.json`. Existing
interval swatches do not prove reference guide presentation. No new envelope
version is needed beyond v44. No fresh aggregate/native-window gate is claimed.

Next: default guide adaptation and registered break composition, then remaining
constructor defaults/fallbacks/forwarding. GG-04 and GG-05–19 remain unfinished.

## Explicit bins and colorsteps scale selection — 13 September 2026

Scope: GG2-03/FIX-GG04, at `6e74ae6` plus working-tree changes. The unchanged
`binned-guide-selection.json` supplies all 432 pinned draws. New `BinnedBins` and
`BinnedSteps` choices retain distinct scale-side behavior through wire v44. The
shared binned mapper evaluates midpoint batches for bins and cut/midpoint batches
for stepped color. Endpoint keys retain missing values. The numeric/value adapter
suppresses color-only stepped guides before population training, including its
consequence for automatic limits. Source/scale values remain in guide candidates;
tests normalize their boundary positions when comparing reference guide keys.
No second host mapping engine is introduced.

Evidence:

- `cargo test -p chart-core --test ggplot_pipeline_functions --test ggplot_binned_numeric --test ggplot_binned_guides --locked -j 2 --target-dir target/ggplot-positional-wasm`: 21 tests pass, including 108 bins, 108 stepped and the prior 216 hidden/legend draws; 383 successes and 49 expected errors across that fixture. Successful callback sequences, key values/labels/mapped values, marks and size values match independent reference values at absolute tolerance `5e-14`. New descriptors round-trip and reject envelope v43. Log `/tmp/ggplot-bins-steps-regressions.log`.
- Fresh Linux Python (`rust:1.97.1-bookworm`, extension-module/extension-proof) and fresh wasm32/Node (wasm-bindgen 0.2.128) execute the expanded `ggplot_pipeline_functions.{py,cjs}` with `binned --guide-selection`. Both pass 873 states: original/restored-and-edited cases, 54 hidden/bins/steps callback-limit replacements and four registration/parameter/envelope rejections. Original definitions and held publication scenes remain unchanged through replacements. Logs `/tmp/ggplot-bins-steps-{python,wasm-build,wasm-host}.log`.
- `ggplot_palette_compare.py` passes exact JSON and all 168 SVG/PDF/PNG bytes, recorded at `target/ggplot-bins-steps/comparison.json`. Python output is `target/ggplot-temporal-precision/linux-target/bins-steps-python`; WASM output is `target/ggplot-bins-steps/wasm`.
- All 18 new color renders inspected at `target/ggplot-bins-steps/inspection/review-{1,2,3}.png`. Marks, bounds, labels and existing interval swatches agree across formats with no clipping. `inspection-matches.json` records 150 exact matches to previously inspected hidden-guide output. These swatches are not GG-05 bins or colorsteps presentation acceptance.
- Focused Clippy with `-D warnings` and repository graph/link checks pass (`/tmp/ggplot-bins-steps-{clippy,repository}.log`). The earlier Linux 710-test aggregate predates v44; this evidence does not claim a fresh aggregate or macOS native-window run.

Next: reconcile registered break/label ordering and constructor default selection,
then remaining constructor defaults/fallbacks/forwarding. GG-04 and GG-05–19 stay open.

## Hidden and legend binned constructor selection — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. The immutable
`binned-guide-selection.json` corpus has 432 draws. Its 216 hidden/legend cases
now execute through the primary native and both host routes (195 successes,
21 errors). Native checks compare successful callback sequences and retained
keys/labels/mapped output, mark count/colors and numeric sizes. The unchanged
bins/colorsteps half remains unqualified. No production scale implementation
changed in this slice.

- Eight native pipeline tests pass, retaining the original 288-case primary proof;
  focused strict Clippy passes. Logs: `/tmp/ggplot-guide-selection-native.log`,
  `/tmp/ggplot-guide-selection-sequences.log`, `/tmp/ggplot-guide-selection-clippy.log`.
- Actual current Linux Python and WASM modules from the mixed-source build pass
  433 states: 411 original/edited/error records, 18 identity-limit replacements
  and four rejection checks. Exact manifests and all 108 publication files match.
  Source helpers now accept `binned --guide-selection`; the primary runner includes
  this proof. Logs: `/tmp/ggplot-guide-selection-{python,wasm}.log`.
- Artifacts: `target/ggplot-guide-selection/{comparison.json,wasm,inspection}` and
  `target/ggplot-temporal-precision/linux-target/guide-selection-python`. All six
  contact pages inspect the 36 automatic-limit SVG/PDF/PNG files. The 36 callback
  files equal them exactly; the 36 authored-limit files equal the previously
  inspected `target/ggplot-binned-legend/pipeline-wasm` files. These publications
  exercise hidden guides; visible legend keys have semantic evidence here.
- The full Linux run exits successfully with 710 tests, zero failures/ignored,
  across 146 executable targets and three doctest targets. It includes the final
  mixed-source implementation, but predates the eighth pipeline test above.
  Log: `/tmp/ggplot-mixed-source-linux-tests.log`.

Next: explicit bins/colorsteps scale-side selection and remaining constructor
reconciliation. GG-05 owns full guide presentation; no cumulative gate closes.

## Constructor coverage reconciliation — 13 September 2026

Revision `6e74ae6` plus working-tree changes. The existing coverage index now
includes all 276 inherited method signatures and captured fields across the 11
scale classes. A direct JSON comparison against the captured source verifies all
152 export names, 960 function formal occurrences, class method names/arguments,
class fields and existence of every linked Rust test target. This is source-index
validation only. Completed helper and population proofs supersede the older open
statements in the coverage narrative; constructor defaults/fallbacks/forwarding
and full class equivalence remain unqualified. Scale-side guide selection remains
GG-04; title callbacks and presentation have their GG-05 owner. No runtime gate
is newly passed by this documentation change.

## Mixed facet routes in identity source chains — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04 and GRA-03/07/08.
The source-population owner combines ancestor filters and intersects explicit panel
targets. Any matched non-chart ancestor preserves row-to-panel filtering. Later
vector stages use that same ancestry to retain source insertion order. The initial
same-target/chart-scope restriction below is superseded; generated statistical
ancestors still retain their distinct schema.

- All 28 focused native tests pass: 19 vector and nine facet. The new test adds 240
  configurations across five mixed routes, fixed/free, direct/shared statistics,
  continuous/binned and six callbacks. Routes include broadcast ancestry, equivalent
  reordered panel lists, matched-to-broadcast, broadcast-to-matched and chart-wide
  identity input. Retained empty panels are included in the independent direct
  baseline; free binned empty-vector classification retains its expected rejection.
  Log: `/tmp/ggplot-mixed-routes-final-native.log`.
- Fresh Linux Python and WASM each pass 520 original/restored/replacement states.
  All JSON and 108 publications match exactly. Seventy-two files match previously
  inspected panel/scope output; the two matched-route variants match one another,
  and the remaining six distinct charts were inspected in SVG/PDF/PNG (inspection
  contact pages 9–11). No R API or pixel-equivalence claim is added by these internal
  composition checks.
- Artifacts: `target/ggplot-mixed-source-vectors/{wasm,comparison.json,inspection}`
  and `target/ggplot-temporal-precision/linux-target/mixed-source-vector-python`.
  Logs: `/tmp/ggplot-mixed-routes-{python,wasm-host}.log`. Primary runner includes
  `--mixed-routes`, 520 states and 108 files. Strict Clippy, rustdoc and repository
  checks pass. A fresh full Linux aggregate is running at
  `/tmp/ggplot-mixed-source-linux-tests.log`; the 706-pass aggregate predates this.
- Remaining: complete source-backed constructor/formal reconciliation. Existing
  built-in and custom statistics reject generated statistical input independently
  of scales; identity consumers of generated results remain supported and tested.
  This slice does not close GG-04 or qualify later GG packages.

## Source-preserving identity chains — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04 and GRA-08.
Positional source-vector resolution traverses identity ancestors to their dataset
and combines their source filters. Row keys remain those of the source dataset.
A statistical ancestor retains its generated schema; this does not implement
statistics over generated statistical rows. In faceted charts, this adapter
currently requires matching target descriptors and matching chart/non-chart scope
across the source-preserving chain, otherwise it reports UnsupportedCapability.

- All 27 focused native tests pass (18 vector, nine facet), including 96 chain
  configurations compared against direct filtered source populations. Independent
  index checks assert [1,2]/[3,4] for fixed scales and [1,2]/[1,2] for free scales.
  Positive/negative sentinel rows excluded by separate ancestors never reach the
  callback population. Log: `/tmp/ggplot-source-chain-final-native.log`.
- Fresh actual Python/WASM each pass 208 states; all records and 48 publications
  agree exactly. Those publications also match the already inspected panel/scope
  files byte-for-byte. The primary proof runner includes `--identity-chain`.
  Artifacts: `target/ggplot-source-chain-vectors/{wasm,comparison.json}` and
  `target/ggplot-temporal-precision/linux-target/source-chain-vector-python`.
  Logs: `/tmp/ggplot-source-chain-{python,wasm-host}.log`.
- Strict focused Clippy, core/export rustdoc and repository graph checks pass.
  This is focused evidence; the 706-test Linux aggregate predates this extension.
  Existing source-statistic schema rejection is visible in `grammar/statistics.rs`
  and custom-statistic source requirements in `grammar/extensions.rs`. Mixed-scope
  identity composition and full constructor reconciliation remain open; GG-04
  is not accepted by this slice.

## Panel targeting and chart-wide source scope — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04 and GRA-03/07.
Explicit panel targets reuse broadcast expansion over their active subset in facet
order, independently of authored target-list order. Chart-wide source scope uses
that same full-input adapter. Only matched, non-chart source populations sort by
source insertion at subsequent vector stages. No wire version changes.

- All 17 native vector tests pass (`/tmp/ggplot-panel-scope-final-native.log`).
  The new composition checks cover 96 configurations: two scopes, fixed/free,
  direct/shared summary, continuous/binned and six callback modes. They compare
  targeted/chart-wide composition against the already reference-qualified broadcast
  owner, including empty/error results and populated callback sequences. These are
  internal scope-contract checks, not a new ggplot reference API claim.
- Fresh actual Python/WASM each pass 208 original/restored/replacement states.
  All records and 48 publications match exactly; all 16 charts were inspected.
  Binned publication cases use explicit limits containing index outputs. Continuous
  summary index callbacks can move marks outside their pre-callback trained range;
  those panels correctly retain axes with clipped marks, rather than a no-data label.
- Artifacts: `target/ggplot-panel-scope-vectors/{wasm,comparison.json,inspection}`
  and `target/ggplot-temporal-precision/linux-target/panel-scope-vector-python`.
  Logs: `/tmp/ggplot-panel-scope-{python,wasm-host}.log`. The primary runner includes
  the 208-state/48-file proof. Focused Clippy passes; the full Linux aggregate below
  predates this change. Shared chart-wide aggregate outputs retain the existing
  explicit presentation-target validation.
- Remaining: source-preserving transform chains and constructor reconciliation.
  Core statistics already reject aggregation over generated statistical rows;
  inspect that existing boundary before treating it as a missing scale operation.
  GG-04 remains open.

## Broadcast positional callback populations — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Source vector
results are cached by panel for broadcast layers and shared source statistics.
Fixed-scale callback populations preserve panel-block order; free scales receive
one population per panel. Matched layers retain source insertion order. The
existing registry owns callback recycling and bin classification.

- All 216 numeric and 432 binned reference cases pass, including shared-statistic
  variants, across 15 native vector tests (`/tmp/ggplot-broadcast-vectors-binned-native.log`).
- Fresh Linux Python and WASM pass 384 direct, 188 shared, 752 binned and 364
  shared-binned states. Manifests and all 108 SVG/PDF/PNG files agree exactly;
  all 36 charts were inspected. Artifacts are under
  `target/ggplot-{broadcast-vectors,shared-broadcast-vectors,binned-broadcast-vectors,shared-binned-broadcast-vectors}/{wasm,comparison.json,inspection}`.
- Matched-facet regressions pass 425/186/704/346 states in both fresh hosts.
  All 108 files also match the previously inspected matched-facet baselines.
  Outputs are `target/ggplot-broadcast-matched-{direct,shared,binned,shared-binned}/wasm`
  and `target/ggplot-temporal-precision/linux-target/matched-*-regression-python`.
- Focused Clippy and strict core/export rustdoc pass. The full Linux core/export
  aggregate passes 706 tests across 149 executables with zero failures or ignores
  (`/tmp/ggplot-broadcast-linux-tests.log`); it predates the panel/scope extension. The primary
  authoring runner includes all four new proof variants.
- References qualify callback populations, selected Y coordinates and guides.
  Summary publications explicitly use constant X (two constants for shared
  consumers); no default ggplot summary-X geometry or pixel equivalence is claimed.
  Explicit panel targets, chart-wide/generated-input callback composition and
  full constructor reconciliation remain open. GG-04 remains open.

## Typed limit helper constructors — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. `xlim` and `ylim`
select numeric reversal, authored category order, Date or UTC datetime scales
through the existing builders. `temporal_limits` retains exact timestamp endpoints
under wire v43. Missing endpoints retrain before statistics and after positions.
Unit conversion subtracts the origin using i128 before floating conversion, with
an integer precision-boundary check. Descending temporal limits stay chronological.

- All 48 captured numeric/character/date/datetime cases run through actual helper
  constructors. The four native test functions include two origins, mixed endpoint
  units, exact nanosecond replacement, wrong-type/precision errors and v43 claims.
  Nine focused helper/temporal-aesthetic tests pass after correcting the existing
  automatic-paint version assertion (`/tmp/ggplot-limit-helpers-dispatch-native.log`).
- Fresh Linux Python and WASM each pass 181 states: 52 valid configurations in
  original/restored/edited forms, 12 arity rejections, eight replacements and five
  exact nanosecond/precision states. All manifests and 48 publications agree
  exactly. All 16 charts were inspected in SVG/PDF/PNG. The proof pages use explicit
  oriented axis ranges inside a 640-point square page; an earlier undersized test
  page was corrected before qualification.
- Artifacts: `target/ggplot-typed-limits/{comparison.json,wasm,inspection}` and
  `target/ggplot-temporal-precision/linux-target/typed-limit-helper-python`.
  Logs: `/tmp/ggplot-limit-helper-dispatch-{python,wasm-host}.log`.
- Fresh temporal aesthetic precision regressions pass 180 exactly matching records;
  blank regressions pass 96 states and 54 exactly matching, previously inspected
  publications. These runs use the final helper build. Strict focused Clippy,
  core/export rustdoc and repository graph checks pass. The earlier full macOS run
  had 630 passes and one outdated v25 assertion across 128 executables; its five-test
  suite passes after the v42 metadata expectation is corrected. No fresh full
  aggregate pass is claimed after the wrappers or subsequent broadcast work.
- The primary authoring proof runner includes the 181-state/48-file helper proof.
  Full constructor/formal reconciliation and broadcast/chart-wide callback
  populations remain open; this does not close GG-04.

## Blank layers and automatic paint ownership — 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. `Geom::Blank`
retains optional x/y and endpoint training, creates no marks or inspection targets,
and contributes no legend of its own. A shared visible paint scale still includes
blank-layer training. Implicit ggplot paint ownership is keyed by aesthetic,
retains the first title and palette, and is explicitly serialized under wire v42.
Blank-only definitions without automatic paint remain v41. Editing an automatic
palette and appending a differently named field preserves that shared scale.

- `ggplot_scale_limit_helpers` passes all 30 pinned expansion cases and two
  independent endpoint/edit regressions (three test functions). The latest focused
  authoring/aesthetic run passes 30 tests; the additional edit regression passes
  separately (`/tmp/ggplot-blank-automatic-edit-native.log`,
  `/tmp/ggplot-blank-palette-edit-regression.log`).
- Fresh Linux Python and WASM/Node each pass 96 states, including restored and
  edited definitions, six retained replacements, scene immutability and R point
  colors. Their manifests and all 54 SVG/PDF/PNG files agree exactly. All 18 charts
  have been inspected in all three formats. Artifacts:
  `target/ggplot-blank-colors/{comparison.json,wasm,inspection}`; Python artifacts:
  `target/ggplot-temporal-precision/linux-target/blank-color-helper-python`.
  Logs: `/tmp/ggplot-blank-color-python.log`,
  `/tmp/ggplot-blank-color-wasm-host.log`. The primary runner includes this proof.
- Strict focused Clippy and core/export rustdoc pass:
  `/tmp/ggplot-blank-color-{clippy,doc}.log`. The latest macOS core aggregate is
  running. Prior 618-test macOS and 689-test Linux aggregates predate this slice.
- The temporal all-missing automatic viewport regression is corrected using the
  existing prepared limit metadata. Eight native extension tests and fresh
  Python/WASM proofs pass 1,328 explicit, 457 automatic and 1,727 vector states.
  All 102 publications agree exactly. Of these, eight SVG/PDF files differ from
  prior inspected outputs because censored singleton points no longer create
  off-viewport circles. PNG pixels are unchanged; the four affected charts were
  inspected in all formats under `target/ggplot-blank-{temporal,auto-temporal}/inspection`.

This qualifies the captured expansion behavior, not GG-05 colorbar classes or the
48 typed-limit constructor cases. Broadcast/chart-wide callbacks and complete
constructor reconciliation also remain open; GG-04 is unfinished.

## Joint positional-bin callbacks — focused qualification, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. The source
population adapter retains limit-vector arity and classifies callback output.
The existing break registry now distinguishes NULL domains from empty numeric
domains; NULL break returns remain distinct through reset training. Inverse-log
mapping uses the existing portable math dependency after a one-ULP host mismatch.

- All 1,728 new `positional-pipeline-binned-compositions.R/json` reference cases
  pass native comparisons: three transforms, points/means, four limit functions,
  three break controls, four populations and six OOB modes. The dimension guard
  and portable inverse also pass all 14 tests across positional vector and binned
  break suites (`/tmp/ggplot-binned-compositions-dimensions-native.log`).
- A separate `positional-binned-joint-dimensions.json` capture adds coordinate-stage
  `break_positions()` evidence to the original 1,120 build-stage cases, preserving
  the original fixture. Eight previously accepted routes fail at that later R
  stage. Core now rejects all-missing functional bin dimensions instead of
  inventing a viewport. The qualified host proof is 2,744 states (2,706 base plus
  38 retained); it supersedes the earlier 2,752-state build-only interpretation.
- Fresh Linux Python and WASM each pass 2,692 new joint states, 2,744 corrected
  dimension states and 1,164 unfaceted regression states. All manifests agree
  exactly, with no inverse-log tolerance needed. All 84 corresponding SVG/PDF/PNG
  files agree byte-for-byte. The 36 new joint and 12 dimension publications have
  been inspected; the 36 unfaceted images retain their earlier inspected result.
  Logs: `/tmp/ggplot-binned-compositions-python-dimensions.log`,
  `/tmp/ggplot-binned-compositions-wasm-dimensions.log`,
  `/tmp/ggplot-binned-joint-dimensions-wasm-host.log` and
  `/tmp/ggplot-binned-compositions-wasm-unfaceted.log`.
- Artifacts: `target/ggplot-binned-compositions/{comparison.json,wasm,inspection}`,
  `target/ggplot-binned-joint-dimensions/{comparison.json,wasm,inspection}` and
  `target/ggplot-binned-vectors/comparison.json`. Fresh Python artifacts are under
  `target/ggplot-temporal-precision/linux-target/` in `binned-composition-python`,
  `binned-joint-dimensions-python` and `binned-vector-python`.
- The primary host runner includes the new joint proof and corrected dimension
  count. The full Linux core/export run passes all 689 tests across 144
  executables, with no failures or ignored tests
  (`/tmp/ggplot-binned-compositions-linux-regressions.log`). Strict Clippy passes
  (`/tmp/ggplot-binned-compositions-clippy-final.log`). These aggregate runs
  compiled before the subsequent blank-layer implementation and do not certify it.
  The macOS aggregate passed 618 tests across 124 executables and strict rustdoc
  passed. Subsequent temporal extension failures are corrected in the section above.

All 152 captured constructor source contracts now match the formal inventory and
are linked from the coverage index. `scale-limit-helpers.R/json` adds 48 typed-limit
and 24 expansion-helper reference cases; these are observations, not implementation
acceptance. Blank training layers, typed helper dispatch, broadcast/chart-wide
callback sources and complete constructor reconciliation remain open. GG-04 is open.

## Positional-bin facet vectors — qualified captured slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Empty callback
vectors fail immediately at bin classification, before later panel callbacks.
Wholly empty layers skip callbacks. Free panel training defers callback-driven
empty classification to that stage. Shared nodes reuse one ordered source vector.

- Pinned `positional-pipeline-binned-facets.R/json` capture: 432 direct cases,
  fixed/free y, occupied/retained panels, three transforms, ordinary/missing/empty
  input, six vector modes and points/means. All cases pass exact callback order,
  values and guide comparisons. The 216 mean cases also pass through two shared
  consumers. All 40 focused native tests pass across vector, bins, bin break/label,
  stage and facet suites (`/tmp/ggplot-binned-facet-vectors-{native,regressions}.log`).
- Fresh Linux Python and WASM/Node each pass 704 direct and 346 shared states,
  including eight direct and four shared retained replacements. The prior
  unfaceted proof also passes all 1,164 states after the error-stage refinement.
  Exact host manifests agree. Logs:
  `/tmp/ggplot-binned-facet-vectors-python.log`,
  `/tmp/ggplot-binned-facet-vectors-wasm-host.log`,
  `/tmp/ggplot-shared-binned-facet-vectors-wasm-host.log` and
  `/tmp/ggplot-binned-vectors-wasm-regression.log`.
- All 36 direct and 18 shared facet SVG/PDF/PNG files agree byte-for-byte and
  have been inspected on six direct and three shared contact sheets. Artifacts:
  `target/ggplot-binned-facet-vectors/{comparison.json,wasm,inspection}`,
  `target/ggplot-shared-binned-facet-vectors/{comparison.json,wasm,inspection}`;
  Python files are in `target/ggplot-temporal-precision/linux-target/` under
  `binned-facet-vector-python` and `shared-binned-facet-vector-python`.
- Strict Clippy and rustdoc pass. Both flag variants are registered in the
  primary host runner. Its whole aggregate and the full Linux native suites
  have not been rerun after this slice.

Joint callback compositions, broadcast/chart-wide vector sources, constructor
reconciliation and cumulative GG-04 acceptance remain open.

## Positional-bin vector callbacks — qualified captured slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Source callback
ranges come from the trained bin owner. The common population adapter invokes the
registry, applies the bin missing policy, classifies returned values with the
existing cut owner and recycles results across the original rows. Final mapping
interpolates statistical bin coordinates; it does not invoke OOB again.

- Pinned `tools/reference/r/positional-pipeline-binned-vectors.R` captures 648
  cases in `fixtures/parity/ggplot2/positional-pipeline-binned-vectors.json`:
  identity/reverse/log10, point x/y and means, automatic/explicit limits, nice/fixed
  cuts, ordinary/missing/empty source populations, six vector modes. Every mode
  explicitly supplies a callback; the test's `default` callback censors, whereas
  the built-in positional-bin default remains squish.
- All 648 native cases pass callback arguments/counts, mapped values and guides.
  All 29 focused native tests across vector, bin, binned break/label and stage
  integration suites pass (`/tmp/ggplot-binned-vectors-{native,regressions}.log`).
- Fresh Linux Python and WASM/Node each pass 1,164 exactly equal states, including
  12 retained replacements, descriptor round-trips and held scene immutability.
  The first host attempt omitted the required `oob` descriptor field; corrected
  descriptors pass without changing production behavior. Logs:
  `/tmp/ggplot-binned-vectors-python-host.log` and
  `/tmp/ggplot-binned-vectors-wasm-host.log`.
- All 36 export files agree byte-for-byte; all 12 charts were visually inspected
  in SVG/PDF/PNG across six contact sheets. Artifacts:
  `target/ggplot-binned-vectors/{comparison.json,wasm,inspection}` and
  `target/ggplot-temporal-precision/linux-target/binned-vector-python`.
- Strict Clippy and rustdoc pass. The whole Linux aggregate and the whole primary
  host runner have not been rerun after this slice; the new host proof is
  registered in that runner. The preceding 685-test aggregate covers the temporal
  and publication changes before positional-bin callbacks were added.

Faceted/shared bin vectors, joint limit/break/vector compositions and the complete
constructor/formal inventory remain unqualified. GG-04 is still in progress.

## Temporal positional vector functions — qualified captured slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. The common
population adapter converts callback inputs, limits and outputs at the temporal
unit boundary. Native timestamp source admission retains its exact-span guard;
checked represented-number conversions handle distant callback offsets. Empty
Date/datetime defaults and duration guide errors retain reference behavior.

- Pinned R 4.6.1 / ggplot2 4.0.3 capture: 324 duration/date/datetime point/mean
  cases, automatic/explicit limits, ordinary/missing/empty populations, six vector
  return modes. `tools/reference/r/positional-pipeline-temporal.R` produces
  `fixtures/parity/ggplot2/positional-pipeline-temporal.json` through the pinned runner.
- Native: all 972 temporal unit configurations pass coordinates, limits, guides
  and callback arguments. All 26 focused temporal/vector/stage regressions and
  the separate represented-offset precision test pass. Logs:
  `/tmp/ggplot-temporal-vectors-{native,regressions,precision}.log`.
- Fresh Linux Python extension and WASM/Node module each pass 1,727 states,
  including 36 retained replacement comparisons and held-scene immutability.
  Exact cross-host manifests and 36 publication files agree; all 12 charts were
  visually inspected in SVG/PDF/PNG on six contact sheets. Artifacts:
  `target/ggplot-temporal-vectors/{comparison.json,wasm,inspection}` and
  `target/ggplot-temporal-precision/linux-target/temporal-vector-python`.
  Logs: `/tmp/ggplot-temporal-vectors-python-final.log` and
  `/tmp/ggplot-temporal-vectors-wasm-host-final.log`.
- Publication preflight and encoders omit fully clipped point circles before
  backend precision conversion. The immutable scene retains the source points;
  partially clipped circles still render and precision checks remain strict.
  All six export authoring tests pass, including the new far-point regression
  with actual SVG/PDF/PNG encoding (`/tmp/ggplot-temporal-vectors-export.log`).
- Final strict core/export/example Clippy, strict rustdoc, formatting, repository
  validation and diff checks pass. The complete Linux core/export regression
  run passes all 685 tests across 144 executables, with zero failures or ignored
  tests (`/tmp/ggplot-temporal-vectors-linux-regressions.log`).
  The new temporal proof is registered in the primary runner, whose whole
  aggregate has not been rerun.

This qualifies the captured six vector modes and temporal units, not arbitrary
fractional epoch callbacks, joint callback compositions or the remaining complete
constructor inventory. Primary binned positional callbacks are the next slice.
GG-04 and G-GGPLOT remain open.

## Shared positional facet vectors — matched numeric slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Shared source
binding now runs through one helper after consumer context resolution in either
unfaceted or panel preparation. Its per-node sample cache reuses complete ordered
fixed/free populations across panel definitions. The common prepared-table
population-operation accessor looks through identity consumers for empty-summary
mapping/rejection and empty-panel retention.

- All 25 focused native tests pass (`ggplot_positional_vector_functions`, `facets`,
  `ggplot_stages`; locked, `target/ggplot-authored-rust`, two jobs). The shared facet
  helper covers 108 existing pinned mean cases: fixed/free-Y, identity/reverse/log10,
  ordinary/missing/empty and six callback modes. It compares complete source and
  selected-Y callback sequences, every consumer's panel coordinates and guides.
  The initial four all-missing summary false successes were fixed by the shared
  population-provenance rule; reference expectations were unchanged.
- Rebuilt actual Python and WASM pass 186 exactly equal shared-facet states:
  182 original/edited outcomes plus four retained replacement-versus-fresh outcomes.
  Both consumers' coordinates, domains and operation identities/counts are checked,
  as are wire v40 restoration, guides and immutable held output. Actual unfaceted
  shared regression passes 102 states and 27 publication bytes. Actual unshared
  facet regression passes 425 states and 36 publications, also unchanged from the
  previously inspected facet artifacts.
- All 18 shared-facet SVG/PDF/PNG files match byte-for-byte; six fixed/free-Y by
  identity/reverse/logarithmic index charts were inspected in
  `target/ggplot-shared-facet-vectors/inspection/review-1.png` through `review-3.png`.
  Each populated panel shows both consumers at X=1 and X=2. The empty C panel
  preserves its no-data label and fixed/free axis behavior. Destination appearance
  is inspected; no ggplot2 pixel equivalence is claimed.
- Artifacts/manifests: `target/ggplot-shared-facet-vectors/{comparison.json,
  unfaceted-comparison.json,unshared-comparison.json,wasm,unfaceted-wasm,unshared-wasm}`;
  Python files under `target/ggplot-temporal-precision/linux-target/` in
  `shared-facet-vector-python`, `shared-vector-python`, `shared-facet-unshared-python`.
  Logs: `/tmp/ggplot-shared-facet-vectors-{native,linux,wasm-host,clippy,doc}.log`.
  Strict all-target core/export/example Clippy, rustdoc, formatting and repository
  checks pass. The primary proof runner registers the new `--shared` route.
- Clean Linux `cargo test --offline --locked -j 2 -p chart-core --lib --tests
  --target-dir /target` in `rust:1.97.1-bookworm` passes all 612 tests across 124
  unit/integration executables (5m 11s build). Doctests/full workspace and the
  complete primary host runner were not part of this command. The first attempted
  aggregate run mixed the earlier library with the newly added facet test; it is
  superseded by this clean run.

This qualifies matched shared numeric source statistics and identity consumers.
Broadcast/chart-wide populations, generated-input statistics, primary temporal/
duration/binned positional vectors and GG-04 remain unqualified. Next: remaining
primary positional families and constructor forwarding/rejection reconciliation.

## Shared positional vector transforms — unfaceted numeric slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Shared transform
resolution compares all consumer source-scale contexts before binding each node's
source callback once. Binding uses the node's dataset, filters, scope and target.
Each consuming layer still runs final positional mapping. Identity operations no
longer hide summary provenance when excluding typed empty aggregates from the
reference positional population; inspection retains those aggregates.

- Native: `mise exec -- cargo test -p chart-core --locked --test
  ggplot_positional_vector_functions --test ggplot_stages --test facets
  --target-dir target/ggplot-authored-rust -j 2`: all 24 tests pass. The mean-summary
  reference helper covers all 54 pinned cases through both direct and shared routes,
  with exact full callback counts/arguments and coordinate comparisons. A separate
  regression checks filtered input, reversed node declaration order and conflict
  rejection without callback evaluation. Log: `/tmp/ggplot-shared-vectors-native.log`.
- Actual rebuilt Python and WASM: `scripts/bindings/ggplot_positional_vector_functions`
  (`.py`/`.cjs`) with `--shared`: 102 states each (96 original/edited outcomes,
  two replacement-versus-fresh outcomes and four rejection contracts). Assertions
  cover both layers' coordinates, domains, operation identities/counts, guides,
  wire v40 round trips and immutable held publications. Fresh versus retained
  comparison excludes allocation/revision identities, which necessarily differ.
  Logs: `/tmp/ggplot-shared-vectors-linux.log` and
  `/tmp/ggplot-shared-vectors-wasm-host.log`; rebuild logs retain initial verification
  attempts before that identity comparison was corrected.
- Exact host equality and all 27 publication bytes pass:
  `target/ggplot-shared-vectors/comparison.json`. Python artifacts:
  `target/ggplot-temporal-precision/linux-target/shared-vector-python`; WASM artifacts:
  `target/ggplot-shared-vectors/wasm`. All nine identity/reverse/logarithmic by
  default/reverse/index charts were inspected across SVG/PDF/PNG in five sheets,
  `target/ggplot-shared-vectors/inspection/review-1.png` through `review-5.png`.
  Shared layers preserve the paired X positions; index outputs outside the authored
  transformed domain are correctly clipped. This is destination inspection, not
  ggplot2 pixel equivalence.
- Strict all-target core/export/example Clippy, `RUSTDOCFLAGS='-D warnings'` rustdoc,
  formatting, repository checks and `git diff --check` pass. The proof is registered
  in `scripts/run_primary_authoring_proofs.py`; the complete primary runner has not
  been rerun. Broader Linux core regression execution is pending at this cutoff.

This qualifies unfaceted shared numeric source statistics and identity consumers.
Faceted shared callbacks, generated-input statistics, temporal/duration/binned
positional vectors and overall GG-04 remain unqualified. No gate is advanced.

## Positional facet vectors — matched numeric slice, 13 September 2026

Revision `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Facet preparation
now resolves all panel scales before binding source vectors. An execution-local
cache shares each layer/input/axis result across panel preparation. Fixed scales
map source points in insertion order; free scales map ordered panel populations,
including empty levels whenever the layer has observations. Statistical outputs
retain panel/group order. Final mapping waits for every shared/free domain to train.
One adapter combines results, restores layer order and applies the reference's
whole-layer recycling rule. This matters for shortened results: two singleton
outputs for four interleaved rows recycle after reordering, while two outputs for
five rows fail. Wholly empty source summaries retain their empty facet catalog.

The pinned R scripts `positional-pipeline-facets.R` and
`positional-pipeline-facet-arities.R` capture **246 cases**: fixed/free-y point and
mean-summary mappings, three transforms, ordinary/missing/empty populations,
unused levels, default/reverse/index/short/empty/NULL results, plus singleton,
one-populated-panel and two-populated-panel arity boundaries. All source and
selected generated-Y callback vectors/ranges, per-panel mapped Y coordinates and
complete requested guide values/labels pass. Typed summary X is explicitly constant;
the reference's group-valued X and unused ymin/ymax calls are not compared. Point X
is also compared in both hosts. This is semantic and export consistency evidence,
not a pixel comparison against ggplot2 rendering.

All **33 focused native and Linux tests pass** (positional vectors, compositions,
stages, missing limits, reverse and facets). Rebuilt Python/WASM each pass **425
states**: 417 original/edited outcomes and eight replacement-versus-fresh states
retaining prior prepared output and authored interchange. Every parsed record is
exactly equal and all **36 publication files are byte-identical**. All 36 PNG/SVG/PDF
outputs were inspected in six sheets; fixed empty panels retain shared ticks and
free empty panels suppress ticks. Records: `target/ggplot-positional-facets/wasm`,
`target/ggplot-temporal-precision/linux-target/positional-facet-python`;
manifest `target/ggplot-positional-facets/comparison.json`;
inspection `target/ggplot-positional-facets/inspection/review-1.png` through `review-6.png`.
Runners `scripts/bindings/ggplot_positional_facet_vectors.py` / `.cjs` are registered
in the primary proof runner; the entire aggregate runner has not been rerun.

Logs: `/tmp/ggplot-positional-facets-{native-final,regressions-linux,python-host,wasm-host}.log`.
These runtime builds precede an equivalent empty-row predicate cleanup requested by
Clippy. Final strict all-target core/example/export Clippy, rustdoc, formatting,
repository and diff checks pass (`/tmp/ggplot-positional-facets-clippy-final.log`,
`/tmp/ggplot-positional-facets-doc.log`, `/tmp/ggplot-positional-facets-repository.log`).
The adapter explicitly rejects broadcast/chart-wide facet source populations and
shared transform inputs. Temporal/duration/binned positional callbacks, further
compositions, constructor forwarding and GG-04/GG-05–19 remain unqualified.

## Positional OOB vector functions — qualified numeric slice, 13 September 2026

Revision: `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Positional
operations use the existing pure vector registry. Initial source mapping retains
results by durable row identity in execution-local `Numeric::Scaled` state.
Validation checks registration/configuration without executing vector callbacks.
The compiler's shared post-statistic limit-training pass performs the second
mapping before geometry. Automatic limits freeze at that training stage; empty
automatic populations suppress ticks. Singleton results recycle; incompatible
result lengths fail. Wire v40 retains authored axis operation selection.

The pinned 108 point cases plus 216 automatic/fixed/constant-domain cases pass
mapped-coordinate and complete callback-sequence comparisons. Guide checks use
explicit `Preserve` presentation to compare the full requested tick set. Another
54 mean-summary cases pass source and selected generated-y mapping. The typed
summary proof selects y; the reference's unused all-missing ymin/ymax callbacks
are outside that comparison. These fixtures were captured through the pinned
R runner; new captures explicitly open a temporary PDF device.

Actual Python and WASM pass **732 exactly equal states**: 720 original/edited
primary outcomes, eight data replacement-versus-fresh comparisons retaining old
prepared publications, and four registration/portability/parameter/version
rejections. Both produce **36 byte-identical, inspected publications**. Fixed
transformed domains correctly clip out-of-range index results, including the
blank reverse/full sample. Records and manifest:
`target/ggplot-positional-vector/wasm/records.json`,
`target/ggplot-temporal-precision/linux-target/positional-vector-python/records.json`,
`target/ggplot-positional-vector/comparison.json`. Six inspection sheets are under
`target/ggplot-positional-vector/inspection/`. Runners:
`scripts/bindings/ggplot_positional_vector_functions.py` and `.cjs`.

Linux point/summary comparisons and stage/missing-limit/reverse/composition
regressions pass. Strict all-target core/example/export Clippy and rustdoc pass.
All 23 final native tests and repository/diff checks pass. Both this proof and the
continuous composition proof are registered in `scripts/run_primary_authoring_proofs.py`;
the full aggregate runner has not been rerun in this slice. Logs:
`/tmp/ggplot-positional-vector-{guides-linux,linux-build,clippy,doc,final-native}.log`.
At this earlier cutoff the source adapter rejected facets and shared transform graphs. Duration,
temporal and binned positional vector callbacks, additional joint limit/break
compositions, constructor forwarding, GG-04 and later packages remain unqualified.

## Transformed limit/vector compositions — qualified slice, 13 September 2026

Revision: `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Captured
`continuous-pipeline-compositions.json` supplies 1,728 primary combinations of
three transformations, eight limit policies, three channels, three populations,
four vector modes and hidden/automatic guides. All 1,272 successes and 456 errors
match native preparations. Another test compares 144 focused callback-input and
limit-vector sequences. Fixtures were not changed.

The shared pipeline retains full transformed limit-vector arity. Registered
rescalers may ignore bounds; default rescaling validates bounds when invoked.
NULL limit returns fail reverse/nonlinear transforms. Constant limit operations
can declare that they do not force the training-domain argument; empty hidden
preparations retain their operation identity without invoking unused mapping.
Existing operations preserve domain-forcing behavior by default.

Actual Python and WASM runners `scripts/bindings/ggplot_pipeline_compositions.py`
and `.cjs` pass **3,012 exactly equal states**: original/edited successful plots,
expected failures and 12 update-versus-fresh states with immutable prepared
publications. Both hosts produce **36 byte-identical publications**. All were
inspected across PNG and rasterized SVG/PDF; marks, axes and clipping agree.
Records and manifest: `target/ggplot-continuous-compositions/wasm/records.json`,
`target/ggplot-temporal-precision/linux-target/compositions-python/records.json`,
`target/ggplot-continuous-compositions/comparison.json`; six review sheets under
`target/ggplot-continuous-compositions/inspection/`. Host logs:
`/tmp/ggplot-compositions-{python,wasm}-host.log`.

Focused native/Linux pipeline, palette, aesthetic, break and numeric-limit tests
pass (`/tmp/ggplot-continuous-compositions-{regressions,linux}.log`). Rebuilt Python
also passes the existing 1,914 numeric-limit/binned/continuous pipeline states.
The host builds precede a mechanical removal of an unnecessary `Box<Vec<_>>`;
final native validation (both tests) and strict all-target Clippy pass after that
cleanup (`/tmp/ggplot-compositions-{final-native,clippy}.log`). Rustdoc,
formatting and repository checks pass. This evidence does not qualify positional
OOB callbacks, all binned limit compositions, remaining constructor forwarding,
GG-04 as a whole or subsequent GG-05–19.

## Scalar sampling, empty preparations and explicit binned legends — 13 September 2026

Revision: `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. Scalar numeric
and paint sampling use the selected palette index. Empty binned preparations defer
sampling until observations or guide keys need it. Explicit `BinnedLegend` retains
point-key semantics and uses wire v39; existing default binned guides retain their
own path. Native tests reject downgrading the new descriptor to v38.

Seven native pipeline tests and six example limit tests pass, covering 96 captured
singleton samples, retained guide values/labels/mapped output, and 504 actual
reference limit/rescaler builds. The focused Linux pipeline, palette, aesthetic,
break-function and numeric-limit regressions pass (`/tmp/ggplot-binned-legend-linux.log`).
Both actual hosts pass **1,348 numeric-limit states**, **283 binned pipeline states**
and **283 continuous pipeline states**. Parsed records agree exactly; all **105
publication files** agree between hosts and with previously inspected files.
Manifests are `target/ggplot-binned-legend/{numeric,pipeline,continuous}-comparison.json`.
The host module is rebuilt under `target/ggplot-binned-legend/wasm-module`.

The numeric primary proof now uses `limit-function-builds.json` (378 builds) and
`limit-rescaler-builds.json` (126 builds). These preserve the original R callback
body and constructor defaults. Earlier 1,408-state records treated some standalone
mapping outcomes as successful primary guides; they do not establish that claim.
The corrected total is 1,348, with 33 publications because the reversed binned
constructor correctly fails. Original standalone fixtures remain unchanged.
Case 154 (empty binned function limits) now passes the actual reference build.
The source-only 432-guide-class inventory is not fully qualified by these tests.

Strict all-target core/example/export Clippy, rustdoc with warnings denied,
formatting, repository checks and diff checks pass. Logs use
`/tmp/ggplot-binned-legend-{native,clippy,doc,repository}.log`. All 604 native unit
and integration tests pass. The full command exits unsuccessfully because two
doctests lost their `rlib` to an overlapping rebuild; even its compile-fail doctests
are not counted as valid evidence. All six doctests pass on a sequential Linux
rerun (`/tmp/ggplot-positional-vector-linux.log`) with the later positional draft.
This does not turn the earlier native command into a full pass. The earlier
609-test Linux run predates this follow-up. Remaining
GG-04 work includes transformed callback/limit compositions and constructor
forwarding; full guide presentation remains GG-05. Cumulative gates remain open.

## Binned OOB/rescaler vector functions — qualified slice, 13 September 2026

Revision: `6e74ae6` plus working-tree changes; GG2-03/FIX-GG04. The existing
binned mapper now shares numeric callback stages with the continuous mapper.
Observation and cut rescaling remain separate ordered calls. Palette midpoints
retain callback order while the existing threshold search receives sorted cuts.
Short/empty/NULL rescaled cuts select a singleton bin; visible guides enforce
candidate arity. Non-NULL palette results are cached per immutable preparation.
The same v38 descriptor and Rust example serve both actual hosts.

All **19 focused native tests** pass: 288 raw/source callback sequences, 288
primary draw outcomes, all prior palette regressions, 765 independently captured
alpha-byte boundary cases and shared-cache ownership. The cache is shared across
prepared clones and replaced with a fresh population; NULL remains uncached.
Reference alpha conversion now rounds ties to even. The boundary capture preserves
17-digit decimal strings so serialization cannot move a test off its exact tie.
Both Python and JavaScript expectation helpers independently pass all 765 cases.
Older alpha expectation helpers used round-half-up and were corrected; raw source
fixtures and tolerances remain unchanged.

Both actual Python and WASM pass 283 binned and 283 continuous vector states,
with exact parsed JSON and 36 byte-identical publications per family. Manifests:
`target/ggplot-binned-vector-pipeline/{comparison,continuous-comparison}.json`.
The fresh focused Linux run passes all 19 tests; runtime logs are
`/tmp/ggplot-binned-vector-linux-focused-final.log` and
`/tmp/ggplot-binned-vector-wasm-final.log`. All 36 binned publications
were inspected across independent SVG/Poppler rasters and PNGs. Strict native
core/example/export Clippy, rustdoc with warnings denied, formatting and repository
checks pass. Logs: `/tmp/ggplot-binned-vector-{focused,clippy,doc-strict,fmt,repository}.log`.
The first full Linux attempt compiled the pre-fix cache owner while later test
compilation picked up the new clone regression; it failed that regression. The fresh focused build passes; **all 609 Linux core tests pass** in the complete
rerun at `/tmp/ggplot-binned-vector-linux-full-final.log`. The initial attempted full run
is **not** recorded as passing.

Broader actual Python/WASM alpha regressions pass 1,057 binned palette, 1,240
continuous palette, 370 binned numeric, 180 temporal precision and 440 temporal
states: 3,287 exact records and 78 byte-identical publications. All 78 also match
the previously inspected files under `target/ggplot-binned-vector-palettes`,
`target/ggplot-continuous-vector-palettes`, `target/ggplot-binned-styles` and
`target/ggplot-temporal-aesthetics`. The older
numeric-limit primary proof exposed a separate source-contract gap. New
`limit-function-builds.R/json` retains the original input-recording callback body
and captures **378 actual builds**, rather than treating standalone mapping as
primary guide success. A reversed-limit binned guide correctly fails in both the
reference and core. Default `GuideBins` on an empty population can also succeed
where standalone mapping fails; that constructor/guide-selection path is still
unresolved in that run (case 154); the follow-up above resolves it and replaces
the primary host proof.
A first capture omitted input recording and therefore changed R lazy argument
forcing; it was replaced before use as an oracle. The original standalone fixture
is preserved. `scale-constructor-contracts.R/json` captures all 152 export bodies
and inherited methods for source-backed constructor reconciliation; it is source
evidence, not acceptance. `binned-guide-selection.R/json` additionally captures
432 actual draws (383 successes, 49 errors) across explicit guide classes and
limits. The immediate empty-function-limit fix is to defer binned sampling when
there are no observations and no guide keys; explicit guide-class presentation
remains GG-05. The reference guide-method probe is
`/tmp/ggplot-guide-selection-source.log`.

## Continuous OOB/rescaler vector functions — qualified slice, 13 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
`ScaleVectorOperation` selects one installed pure numeric vector function, with
borrowed transformed observations/limits and an explicit OOB or rescaling stage.
The continuous mapper owns transformation, OOB, rescaling, ordered uniqueness and
palette lookup. Callback results retain length and NULL separately; a singleton
recycles across marks, while an empty rescaler result supplies missing aesthetics.
NULL palette output still selects geometry defaults. Guide vectors require exact
candidate arity before outside keys are removed. Both normalized envelopes use v38;
legacy definitions retain their existing minimum versions. The example extension
supplies the same Rust implementation to both hosts.

The implementation also preserves infinite mapped reference point sizes in native
styles and canonical Number JSON. Their glyphs have empty paths, retaining rows
without nonfinite publication geometry. The pinned Grid nonfinite documentation,
PDF probes and 78 fresh ragg raster cases cover all 26 reference point shapes with
negative infinity, a finite control and positive infinity. Initial raster capture
counted named white pixels incorrectly; RGB conversion corrected the capture before
using the fixture. Finite controls paint, while both infinities paint zero pixels.

Evidence:

- `scale-pipeline-functions.json` and `scale-pipeline-draws.json`, captured by the corresponding scripts under `tools/reference/r/`, include 144 continuous cases (117 successes, 27 reference errors) and 144 still-unimplemented binned cases. Draw capture calls the reference renderer. Logs: `/tmp/ggplot-scale-pipeline-functions.log` and `/tmp/ggplot-vector-pipeline-draws.log`.
- `point-nonfinite-sizes.R` captures 78 actual ragg draws in `point-nonfinite-sizes.json`; `/tmp/ggplot-point-nonfinite-sizes.log`. Separate PDF probes under `target/ggplot-vector-pipeline-source` also produce no drawing commands for either infinity; their PNG hashes are equal and differ from the finite control.
- The first two tests in `ggplot_pipeline_functions.rs` pass on native and Linux: all 144 complete callback sequences/values/guide errors and all 144 primary JSON/draw outcomes, including point counts, colors and finite/nonfinite sizes. All four pipeline/glyph/resource tests pass natively and on Linux. The Linux final command passes 39 focused tests; the native focused run passes the corresponding 37 tests before adding the two glyph/resource tests, and the final four-test run passes in `/tmp/ggplot-vector-pipeline-focused-final.log`. Native logs: `/tmp/ggplot-vector-pipeline-native-final.log`; Linux: `/tmp/ggplot-vector-pipeline-linux-final.log`. The glyph harness was corrected to use the source reference identity scale rather than the documented finite-only `shape_value` shortcut, and to compare numeric R JSON with the f64 field semantically. No reference cases were removed.
- Actual Linux Python and WASM each pass **283 states**, including 18 updated/fresh comparisons, held publication immutability and four unavailable/native-only/malformed/downgraded-operation rejections. Logs: `/tmp/ggplot-vector-pipeline-{python,wasm}.log`; WASM build: `/tmp/ggplot-vector-pipeline-wasm-build.log`. The expected state count was corrected from 292 to 283 using the fixture's 27 reference errors; no cases were removed.
- `ggplot_palette_compare.py ... 283 36` confirms exact parsed JSON equality and **36 byte-identical SVG/PDF/PNG files**. Manifest: `target/ggplot-vector-pipeline/comparison.json`. Records: `target/ggplot-temporal-precision/linux-target/vector-pipeline-python` and `target/ggplot-vector-pipeline/wasm`. All six inspection sheets under `target/ggplot-vector-pipeline/inspection` were viewed: missing/transparent alpha, reordered colors, missing sizes and infinite-size suppression agree across formats. Native/source numbers use absolute 5e-14 for 15-digit R captures.
- Strict core/example/export Clippy, core rustdoc with warnings denied, formatting and repository checks pass: `/tmp/ggplot-vector-pipeline-{clippy,doc,fmt,repository}.log`. All **607 native core tests** pass in `/tmp/ggplot-vector-pipeline-full-core.log`. Long pre-test process startup waits also occurred outside the sandbox; a read-only sample showed `_dyld_start` before test code. The suite completed without failures.
- The shared comparison runner now accepts an explicit publication count, preserving the prior 27-file default. The aggregate primary runner registers the new Python/WASM/comparison steps but has not been executed in full.

Binned and positional vector callbacks remain unimplemented; 144 binned and 108
positional builds are source-only. `continuous-pipeline-compositions.R` additionally
captures 1,728 source-only transformed/OOB/rescaler/limit combinations (1,272 successes,
456 errors); `/tmp/ggplot-continuous-pipeline-compositions.log`. These include empty
limits that custom callbacks can ignore, so default normalizer validation must not
unconditionally reject them in the future composition implementation. Other constructor forwarding/compositions and
cumulative acceptance remain open. Publication inspection here establishes these
mapped mark outcomes, not full reference guide composition or exact device layout.
One diagnostic R probe regenerated the already-modified `Rplots.pdf`; its previous
working-tree bytes were not backed up. All committed capture scripts added in this
slice use explicit temporary or raster devices.

## Continuous vector palette callbacks — qualified slice, 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
Four native/Linux tests cover **675 pinned continuous cases** (618 successes and
57 reference errors): 540 constructor/map
cases, 54 missing-color draws and 81 vector-sensitive two-layer builds. Native tests
compare normalized callback arguments, first-occurrence ordering, raw maps, guide
values/labels, primary JSON round trips, color, size, alpha and source error outcomes.
The two-layer test exercises actual primary plots as well as one shared prepared scale.

The existing palette registry and interpolated normalizer own the calculation.
Continuous batches deduplicate normalized inputs in first-seen order, match repeated
missing values and signed zero, ignore output names and replace missing/short outputs
with na.value. NULL means no mapped aesthetic vector: geometry receives its defaults.
Visible guides reject NULL before filtering outside keys. Retained guide `mapped`
values avoid pointwise reevaluation; grammar parses each unique color output once.
The descriptor remains wire v37. The native example adds pure index/length/first
palette variants, shared by both proof hosts.

Evidence:

- Pinned ggplot2 4.0.3 source: `palette-functions.json`, `vector-palette-lookup.json`, `vector-palette-missing-paint.json`, and `vector-palette-batches.json`, captured by the corresponding scripts under `tools/reference/r/` during the prior palette work.
- Native: `cargo test -p chart-core --test ggplot_continuous_palette_functions --test ggplot_binned_palette_functions --test ggplot_palette_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-continuous-palette-tests.log`. All 13 tests pass. Initial host testing found that a NULL size palette incorrectly dropped points; the implementation now supplies the layer's resolved default size and the native primary test independently checks every size/alpha case.
- Linux/Python: offline Rust 1.97.1 Bookworm, all 13 focused tests, actual Python extension with `extension-module,extension-proof`, and `scripts/bindings/ggplot_continuous_palette_functions.py`; `/tmp/ggplot-continuous-palette-final-linux-python.log`. Records: `target/ggplot-temporal-precision/linux-target/continuous-vector-palette-python`.
- WASM: `chart-wasm` with `extension-proof`, wasm-bindgen 0.2.128 and Node; `/tmp/ggplot-continuous-palette-final-{wasm-build,wasm}.log`. Module and records: `target/ggplot-continuous-vector-palettes/{wasm-module,wasm}`.
- Each actual host passes **1,240 states**, including 81 independently authored two-layer plots, 24 updated/fresh comparisons, immutable held publication checks and four registry/wire rejection contracts. Native/source numeric comparisons use absolute 5e-14 for 15-digit R JSON. Host states compare exactly after excluding generated object identities. Shared-scale native tests verify exact callback input sequences; repeated pure evaluation across independently compiled primary layers is not an evaluation-count equivalence claim.
- `ggplot_palette_compare.py` confirms **27 byte-identical publication files**. The SHA-256 manifest is `target/ggplot-continuous-vector-palettes/comparison.json`. All five inspection sheets, covering independent SVG/Poppler rasterizations alongside PNGs, were inspected: color/size/alpha ordering, missing-alpha retention and NA/green/transparent replacements agree across formats. Final rebuild hashes still match every inspected file.
- All **603 core tests** pass (`/tmp/ggplot-continuous-palette-full-core.log`). Strict core/example Clippy, core rustdoc with warnings denied, `cargo fmt --all -- --check`, and `scripts/check_repository.py` pass in the corresponding `final-clippy`, `doc`, `fmt`, and `repository` logs. An attempted nonexistent `fmt-check` mise task was corrected to the cargo formatting command. The proof runners are registered in `scripts/run_primary_authoring_proofs.py`; the aggregate runner has not been executed in full.

GG-04 remains in progress. Constructor reconciliation identified arbitrary vector
`oob`/`rescaler` callbacks as the next unimplemented contract.
`tools/reference/r/scale-pipeline-functions.R` captures 288 pinned public builds
(234 successes, 54 errors) in `scale-pipeline-functions.json`;
`/tmp/ggplot-scale-pipeline-functions.log` passes. This is source evidence only,
including binned rescaling of breaks separately from observations. The companion
`positional-pipeline-functions.R` captures 108 positional builds (84 successes,
24 errors) across x/y, identity/reverse/log transforms and six OOB operations;
`/tmp/ggplot-positional-pipeline-functions.log` passes. Position OOB functions run
before and after statistics; singleton outputs recycle, while empty/NULL outputs
remove the required positional aesthetic. These 108 cases are also source-only. Remaining
constructor forwarding/compositions and cumulative acceptance still follow.
Full guide composition, non-color legends and the known nonlinear endpoint-label
omission remain GG-05; host metadata equality does not prove those visual contracts.

## Binned vector palette callbacks — qualified slice, 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
Four focused native and Linux tests cover **675 pinned binned cases**, comprising
516 successful reference builds and 159 errors. The original constructor fixture
supplies 180 cases; `vector-palette-lookup.json` adds 360 hidden/restricted-guide
cases, `vector-palette-missing-paint.json` adds 54 source point-grob comparisons,
and `vector-palette-batches.json` adds 81 vector-sensitive two-layer cases. The last
test shares one prepared scale across both input vectors and verifies callback
arguments, raw values and selected guide keys. It does not establish evaluation
counts across independently compiled primary layers.

`ScalePaletteDomain` extends the existing pure registry with a borrowed normalized
vector. The binned owner passes interval midpoints in reference order, ignores output
names, substitutes na.value for short/empty/missing results, and preserves NULL as
a distinct guide-construction error. Empty hidden layers skip callback evaluation.
All arithmetic and callback execution remain in core; host descriptors retain v37.
Primary authoring/round trips pass all 540 main/lookup outcomes, and explicit NA,
green and transparent replacements match the source's point counts and colors.

Evidence:

- Sources: `python3 tools/reference/r/run.py tools/reference/r/vector-palette-{lookup,missing-paint,batches}.R`; `/tmp/ggplot-vector-palette-{lookup,missing,batches}.log`. These scripts also capture **675 continuous cases** that remain source-only, including first-occurrence uniqueness per layer and separate guide vectors.
- Native: `cargo test -p chart-core --test ggplot_binned_palette_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-binned-vector-native.log`. All four tests pass. The discrete callback regression tests also pass.
- Linux/Python: offline Rust 1.97.1 Bookworm, four binned plus five discrete tests, then the actual Python extension build; `/tmp/ggplot-binned-vector-linux-python.log`. The final runtime proof passes in `/tmp/ggplot-binned-vector-python.log`; records are under `target/ggplot-temporal-precision/linux-target/binned-vector-palette-python`.
- WASM: extension-proof build, wasm-bindgen 0.2.128 and Node; `/tmp/ggplot-binned-vector-{wasm-build,wasm}.log`; module and records under `target/ggplot-binned-vector-palettes/{wasm-module,wasm}`.
- Both hosts pass **1,057 states**, including 24 updates with fixed limits compared with fresh preparation, immutable held publications, and four rejection cases. The comparison excludes generated object identities; scale definitions, guide metadata and styles compare exactly. Native raw numeric checks use absolute 5e-14 tolerance for the source's 15-digit JSON encoding. Host style checks independently compare color, size and resolved alpha against the source; guide metadata agreement between hosts does not prove reference legend composition.
- `scripts/bindings/ggplot_palette_compare.py` confirms exact parsed state equality and **27 byte-identical PNG/SVG/PDF files**; SHA-256 evidence is `target/ggplot-binned-vector-palettes/comparison.json`. Independent SVG/Poppler rasterizations in `inspection/review-1.png` through `review-5.png` were inspected: binned color/size/alpha ordering, retained missing-alpha points and NA versus green/transparent replacement agree across formats.
- All **599 core tests**, including doctests, passed in `/tmp/ggplot-binned-vector-full-core-retry.log`. The first run stalled before the reverse-axis binary printed its test count; that binary passed all seven tests in isolation and the complete retry passed. A final diagnostic refinement classifies NULL guide keys as validation failure; all nine focused binned/discrete tests and both rebuilt hosts pass afterward in `/tmp/ggplot-binned-vector-final-{native,linux-python,wasm}.log`. The complete suite was not repeated after this diagnostic-only refinement.
- Final strict core/example Clippy passes in `/tmp/ggplot-binned-vector-final-clippy.log`; denied-warning rustdoc, formatting and repository checks pass in `/tmp/ggplot-binned-vector-{doc,fmt,repository}.log`. Final state equality was rechecked, and all 27 publication hashes still match the inspected files. The proof runners are registered in the aggregate primary-authoring script; the aggregate runner has not been run in full.

Continuous vector callbacks, further palette/guide/limit compositions and complete
constructor reconciliation remain GG-04. Explicit legend composition, non-color
legends and the known nonlinear endpoint-label omission remain GG-05.

## Discrete palette callbacks — qualified slice, 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
The pinned public-constructor corpus contains 180 count-palette cases across color,
size and alpha, 360 hidden/restricted-guide cases, and 54 explicit missing-color
cases with source point-grob counts. In total there are 441 successful builds and
153 expected errors. Five focused Rust tests compare callback counts, raw mapping,
keys/labels, primary serialization/preparation, actual retained point colors/counts,
malformed names, output resource bounds and invalid trained metadata.

The shared ordinal owner retains the full trained domain. A named palette uses the
first matching name without manual-palette limit intersection. An unnamed short
palette appends one fallback slot and raises only on an actual out-of-range mark or
selected guide lookup; hidden or restricted guides do not sample unused keys. A
returned NA remains missing even when na.value supplies green or transparent paint.
Unmatched inputs/names and the appended slot use na.value instead. Sorted retained
fallback indices preserve this distinction through immutable preparation and wire
round trips. Empty hidden charts do not evaluate a palette on fabricated data.

Wire v37 captures the pure installed `ScalePaletteOperation`. Registration validates
identity, portability, parameter bounds, names and one aggregate borrowed value/name
budget. Scalar interpolation remains unchanged. `missing_paint_is_na` distinguishes
reference discrete NA paint from a real transparent color; an explicit primary
color replacement clears that setting. Prepared non-color scale metadata remains
inspectable even when every mark is missing. Numeric expected values use absolute
5e-14 tolerance for the source JSON's 15-digit decimal encoding; names, categories,
missingness, outcomes and callback counts are exact.

Evidence:

- Sources: `python3 tools/reference/r/run.py tools/reference/r/{palette-functions,discrete-palette-lookup,discrete-palette-missing-paint}.R`; fixtures with the corresponding names under `fixtures/parity/ggplot2`; `/tmp/ggplot-palette-functions-reference.log`, `/tmp/ggplot-palette-lookup-reference.log` and `/tmp/ggplot-palette-missing-reference.log`. The first fixture also captures 360 numeric/binned cases awaiting implementation.
- Native: `mise exec -- cargo test -p chart-core --test ggplot_palette_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-palette-native.log`.
- Linux/Python: offline Rust 1.97.1 Bookworm, focused tests then `cargo build -p chart-python --features extension-module,extension-proof`; `/tmp/ggplot-palette-linux-python.log`. Runtime records: `target/ggplot-temporal-precision/linux-target/discrete-palette-python`.
- WASM: `cargo build -p chart-wasm --features extension-proof --target wasm32-unknown-unknown`, wasm-bindgen 0.2.128 and Node; `/tmp/ggplot-palette-{wasm-build,wasm}.log`; `target/ggplot-discrete-palette-functions/{wasm-module,wasm}`.
- Actual hosts have passed 1,063 states, including 24 replacements compared with fresh preparation and four rejection cases (missing registration, native-only publication, invalid parameters, and a downgraded envelope). The final rebuilds include the aggregate validation change and all five focused Linux tests pass.
- All **27 PNG/SVG/PDF files are byte-identical**. `scripts/bindings/ggplot_palette_compare.py` records exact JSON equality for 1,063 states and per-file SHA-256 in `target/ggplot-discrete-palette-functions/comparison.json`. Independent SVG rasterization and Poppler PDF rendering are inspected in `target/ggplot-discrete-palette-functions/inspection/review-1.png` through `review-5.png`. Named color/size/alpha ordering and NA versus green/transparent replacement agree across the three formats. Non-color legend composition remains GG-05.
- Strict Clippy, denied-warning rustdoc and repository checks pass in `/tmp/ggplot-palette-{clippy,doc,repository}.log`; all **595 core tests**, including doctests, pass in `/tmp/ggplot-palette-full-core.log`. Formatting passes. The scripts are added to the aggregate primary runner; that runner has not been executed in full.

This is a bounded discrete palette slice. Continuous and binned vector palettes,
additional palette/guide/limit compositions and complete constructor reconciliation
remain open. The known GG-05 nonlinear endpoint-label omission remains open.

## Temporal minor callbacks — qualified slice, 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
The pinned source corpus contains 4,800 cases: 2,830 successful builds and 1,970
expected errors. Native execution compares all cases in seconds, milliseconds,
microseconds and nanoseconds (19,200 comparisons), plus 60 width overrides in four
units (240 comparisons). The two focused Linux tests also pass.

Date and datetime callbacks receive typed expanded limits and optional typed major
vectors, retaining intrinsic automatic-break names before label formatting. Major
selection metadata stays internal to guide resolution. The shared registry exposes
`major_break_names`; the existing v36 operation wire format remains unchanged.
NULL/untyped temporal results retain reference errors, zero ranges bypass minor
functions, and explicit width replaces the callback. Fractional Date minor values
are preserved. Missing/nonfinite candidates have no drawable position. Unbounded
axes retain finite semantic major inputs even when publication has no finite range.

The corpus exposed and fixes an empty-datetime fallback: under the reference profile,
its default extent is one second in every source unit, rather than one source tick.
A constant temporal range reduces an explicit empty major vector to the constant;
NULL suppression remains empty. Temporal major and minor functions share the typed
limit resolver. Projection normalizes within the callback coordinate frame before
converting to the retained calendar frame, avoiding distant-epoch endpoint loss.

Numeric callback and minor-value comparisons allow at most four binary64 epsilon
units at the larger of the authored absolute origin, expected value, and one. This
bounds cancellation when the empty-panel epoch-zero range is represented relative to
an authored 2024 origin; Date uses days, datetime seconds. Metadata classes, zones,
names, array lengths and outcomes are exact. Drawable positions use absolute 1e-7
pixels; major labels are exact. Large/fractional offsets remain numeric metadata
instead of claiming exact integer timestamps. No fixture values were changed.

Evidence:

- Source: `tools/reference/r/positional-temporal-minor-break-functions.R`, run through `python3 tools/reference/r/run.py`, with and without `--overrides`; `/tmp/ggplot-temporal-minor-reference.log` and `/tmp/ggplot-temporal-minor-overrides-reference.log`; fixtures `positional-temporal-minor-break-functions.json` and `positional-temporal-minor-break-overrides.json` under `fixtures/parity/ggplot2`.
- Native: `mise exec -- cargo test -p chart-core --test ggplot_positional_temporal_minor_break_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-temporal-minor-core.log`.
- Linux: `/tmp/ggplot-temporal-minor-linux-python.log`, offline Rust 1.97.1 Bookworm.
- Full core passes 590 tests, including doctests; strict Clippy, denied-warning rustdoc and repository checks pass. Logs: `/tmp/ggplot-temporal-minor-{full-core,clippy,doc,repository}.log`.
- Actual Python/WASM each pass **30,545 main states and 480 width states**, including 25 replacements compared to fresh layouts. Held scenes and plot JSON remain unchanged. Logs: `/tmp/ggplot-temporal-minor-{linux-python,wasm-build,wasm,width-wasm}.log`.
- Records compare exactly with JSON numbers interpreted as binary64 on both hosts. There are 320 large integral minor offsets whose decimal/scientific spellings differ; all resolve to exactly the same binary64 value, with no ULP tolerance. Timestamp integer strings, labels, positions, keys and array shapes remain exact.
- Python records: `target/ggplot-temporal-precision/linux-target/{positional-temporal-minor-break-python,positional-temporal-minor-width-python}`. WASM records/comparisons: `target/ggplot-positional-temporal-minor-break-functions/{wasm,comparison.json}` and `target/ggplot-positional-temporal-minor-width/{wasm,comparison.json}`.
- All **24 PNG/SVG/PDF files are byte-identical**, and independent SVG/Poppler renders were inspected in each target's `inspection/review-1.png` and `inspection/review-2.png`. Spaced Date, UTC, New York spring and New York fall retain correct major labels and point positions; the spring missing hour and fall repeated hour agree. Minor-line painting remains GG-05.
- Scripts are integrated into the aggregate primary runner, which has not been run in full.

GG-04 remains in progress. Next: palette callbacks and complete constructor/argument
reconciliation. The new source-only `tools/reference/r/palette-functions.R` and
`fixtures/parity/ggplot2/palette-functions.json` capture 540 continuous/discrete/binned
colour/size/alpha builds (414 successes, 126 errors), including full/short/empty/NULL,
named and missing outputs; `/tmp/ggplot-palette-functions-reference.log` passes.
Palette implementations and host proofs are not yet supplied for this corpus.
GG-05 must also fix the previously
recorded nonlinear endpoint-label omission and qualify minor-line painting.

## Discrete minor callbacks and binned exclusion — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
All 300 discrete source builds match two native routes (band/point), covering five
populations, automatic/empty/suppressed majors, both function signatures, five result
modes and default/zero expansion. Discrete callbacks receive expanded numeric category
bounds and mapped major coordinates. They run on zero ranges. Out-of-range/nonfinite
outputs are omitted by the shared category projection. NULL major suppression remains
NULL on populated scales and becomes an empty vector on empty scales. The registry
preserves this distinction with an optional major input.

The pinned public `scale_x_binned()` constructor rejects its `minor_breaks` argument:
all 300 captured combinations fail before any callback. Registered binned minor
policies return `UnsupportedCapability`; no ignored-callback parity is claimed for
that constructor. This is a recorded unsupported public argument, not a missing
implementation of a supported binned callback.

Actual Python/WASM each pass **1,520 exactly matching states**, including 20 data
replacements compared against fresh batches, with held scenes and plot JSON retained.
All 12 PNG/SVG/PDF outputs are byte-identical and inspected via independent SVG and
Poppler rendering. The band/point, spaced/constant samples retain the expected major
labels and point positions. This does not certify minor-line painting, which remains
GG-05. All four focused native tests and Linux equivalents pass; strict core/extension
Clippy, denied-warning rustdoc and repository checks pass. Full core: **588 passed**.

Evidence:

- Source: `python3 tools/reference/r/run.py tools/reference/r/positional-other-minor-break-functions.R`; `fixtures/parity/ggplot2/positional-other-minor-break-functions.json`; `/tmp/ggplot-other-minor-reference.log`.
- Native: `mise exec -- cargo test -p chart-core --test ggplot_positional_minor_break_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-other-minor-core.log`.
- Checks: `/tmp/ggplot-other-minor-{full-core,clippy,doc,repository}.log`.
- Actual hosts: `/tmp/ggplot-other-minor-{linux-python,wasm-build,wasm}.log`; `scripts/bindings/ggplot_positional_other_minor_break_functions.py` / `.cjs` and exact comparator; aggregate runner includes these commands but has not been rerun.
- Python records: `target/ggplot-temporal-precision/linux-target/positional-other-minor-break-python`; WASM, comparison and inspected views: `target/ggplot-positional-other-minor-break-functions/{wasm,comparison.json,inspection/review-1.png,inspection/review-2.png}`.

Next: typed temporal minor callbacks. Source-only capture at
`fixtures/parity/ggplot2/positional-temporal-minor-break-functions.json` contains 4,800
cases (2,830 successes; 1,970 expected errors), with 60 successful width overrides in
`positional-temporal-minor-break-overrides.json`. Automatic major selection passes
names to two-argument minor functions, so retained selection metadata must expand.
These temporal cases are not yet implemented or qualified. GG-04 remains open.

## Joint numeric positional major/minor callbacks — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
The new 1,600-case source corpus combines registered fixed/domain major functions
with both minor signatures, four transformations, five populations, trained/fixed
limits, five minor result modes and default/zero expansion. All 1,258 successes and
342 expected NULL-transform errors match native behavior. Both callback argument
vectors, major labels and independently computed major/minor positions are checked.
An unbounded identity axis with a fixed major function exposed the missing retained
selection: the minor function must receive `[10,5,1,1]` even when no majors are drawable.

Guide resolution now retains its semantic candidate vector before projection.
Internal axis resolution carries that vector through each layout pass; additional
guides retain their own selection. Public axis structs and wire v36 remain unchanged.
Minor callbacks consume the existing selection, with no second major evaluation.
The explicit-vector fallback for unbounded axes is removed. Callback scheduling
across layout passes remains distinct from R build scheduling.

Actual Linux Python and WASM each pass **2,898 states**, including 40 updated/fresh
layouts preserving held scenes and serialization. Labels, positions and nonnumeric
structure match exactly. Fourteen raw major and eighteen raw minor inverse-log values
differ by `8.881784197001252e-16` (one ULP). The major exception is restricted to
log10 domain-returning callbacks and one ULP; it does not permit other semantic or
position differences. Original numeric-minor proofs still pass 5,756 states per
host with their original 34 raw minor inverse-log differences.

Twelve joint PNG/SVG/PDF files are byte-identical and were inspected using independent
SVG and Poppler PDF rendering. **Open GG-05 defect:** samples 412 (sqrt) and 812
(log10) paint only the interior label 5 although their guide snapshots retain correctly
positioned labels 10, 5, 1, 1. Identity/reverse samples paint the expected endpoint
labels. Byte equality therefore certifies cross-host publication agreement, not correct
nonlinear endpoint painting. Minor guide painting likewise remains GG-05 scope.

Full core after the joint production change: **587 passed**. Focused Linux tests,
strict core/proof-extension Clippy, denied-warning rustdoc and repository/format checks
pass. The final focused native test also independently checks major positions. No
aggregate authoring or cumulative GG-04 gate is claimed.

Evidence:

- Source: `python3 tools/reference/r/run.py tools/reference/r/positional-minor-break-functions.R --joint`; fixture `fixtures/parity/ggplot2/positional-minor-break-joint-functions.json`; `/tmp/ggplot-positional-minor-break-joint-reference.log`.
- Native: `mise exec -- cargo test -p chart-core --test ggplot_positional_minor_break_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-joint-minor-final-focused.log`.
- Checks: `/tmp/ggplot-joint-minor-{full-core,clippy,doc,repository}.log`.
- Actual Linux/Python: `/tmp/ggplot-joint-minor-linux-python.log`; WASM: `/tmp/ggplot-joint-minor-{wasm-build,wasm,base-wasm}.log`.
- Both `scripts/bindings/ggplot_positional_minor_break_functions.py` / `.cjs` and comparator accept `--joint`; all are in `scripts/run_primary_authoring_proofs.py`.
- Python records: `target/ggplot-temporal-precision/linux-target/positional-minor-break-joint-python`; WASM, comparison and inspection: `target/ggplot-positional-minor-break-joint-functions/{wasm,comparison.json,inspection/review-1.png,inspection/review-2.png}`.

GG-04 remains open. The next source capture (`positional-other-minor-break-functions.R`)
contains 300 successful discrete cases and 300 rejected binned-constructor calls;
implementation and host qualification are still in progress. Temporal minor callbacks,
remaining callback families and constructor reconciliation remain required.

## Numeric positional minor-break callbacks — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
`MinorBreaks::Registered` reuses the installed pure break registry in wire v36.
`CustomScaleBreaks::accepts_major_breaks()` captures whether a minor selector receives
its second argument; `ScaleBreaksInput.major_breaks` is absent on ordinary major
selection. Minor callbacks receive inverse-transformed expanded limits and, for the
two-argument form, finite major breaks in source coordinates. No count argument is
supplied. The existing core operation bounds, registry identity and native/portable
checks apply to primary axes and additional guides. Python/WASM declarations expose
the new policy; interpreter objects and executable strings do not enter core.

All 3,200 pinned R 4.6.1 / ggplot2 4.0.3 builds match native outcomes, callback
arguments, major labels, minor values and positions: 2,516 successes and 684 expected
NULL-transform errors. Four transforms, five populations, trained/fixed limits,
automatic/fixed/empty/suppressed majors, one/two-argument callbacks, five result modes
and default/zero expansion are covered. Source numeric comparisons allow `3e-12`
relative-to-max-one error; independent fixed-range positions allow `1e-8` logical
pixels. Zero transformed ranges bypass the minor function. Raw infinite candidates
can reach the function while unprojectable outputs remain absent from drawable
snapshots. Callback scheduling is not equated with R's build passes.

The same corpus exposed and now qualifies default major-label behavior with missing
and infinite candidates. Positional censoring shares the optional numeric label
owner used by aesthetic guides. On zero ranges, reference fixed vectors—including
empty vectors—reduce to one major break. Suppression uses the existing exact empty
coupled-tick selection and remains distinct. These are adapter routes for this
corpus, not a claim that all constructor/label combinations are reconciled.

Actual Python and WASM each pass 5,756 states, including 40 data replacements whose
updated layouts match fresh batches and preserve held scenes and plot serialization.
Every label, position and nonnumeric structure matches exactly. There are 34 raw
minor inverse-log differences, at most `8.881784197001252e-16`; the comparator permits
only minor tick value differences within the source tolerance. All 12 publication
files are byte-identical and inspected through PNG, independent SVG rendering and
Poppler PDF rendering. These validate the retained chart publication; minor guide
painting remains GG-05 scope.

Final full core: **586 tests passed**. Focused Linux callback regressions, strict
Clippy for core and the proof extension, rustdoc with denied warnings and repository
checks pass. Native boundary checks cover v36 round-trips, rejected v35 downgrade,
missing/native-only registrations, extra guides and result budgets. The aggregate
authoring runner includes these proofs, but has not been rerun as an aggregate.

Evidence:

- Reference: `python3 tools/reference/r/run.py tools/reference/r/positional-minor-break-functions.R`; `/tmp/ggplot-positional-minor-break-reference.log`.
- Native: `mise exec -- cargo test -p chart-core --test ggplot_positional_minor_break_functions --locked --target-dir target/ggplot-authored-rust`; `/tmp/ggplot-positional-minor-break-core.log`.
- Full core, Clippy, docs and repository: `/tmp/ggplot-positional-minor-break-{full-core,clippy,doc,repository}.log`.
- Actual Linux/Python and WASM: `/tmp/ggplot-positional-minor-break-{linux-python,wasm-build,wasm}.log`; scripts `ggplot_positional_minor_break_functions.py` / `.cjs` and the narrow `ggplot_positional_minor_break_compare.py` under `scripts/bindings/`.
- Records: `target/ggplot-temporal-precision/linux-target/positional-minor-break-python` and `target/ggplot-positional-minor-break-functions/wasm`. Comparison and inspected renders: `target/ggplot-positional-minor-break-functions/{comparison.json,inspection/review-1.png,inspection/review-2.png}`.

GG-04 remains in progress. Joint registered major/minor selection is not qualified:
a separate 1,600-case source corpus now captures the semantic major vector even when
an unbounded axis cannot draw it. That composition is next, followed by temporal,
discrete/binned minor behavior, remaining callbacks and constructor reconciliation.

## Temporal positional break functions — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
The existing v34 positional operation now selects typed Date/datetime candidates.
The registry receives expanded origin-relative limits, exact timestamp metadata,
the supplied timezone resource and the reference `n` capability. Untyped/NULL
results are rejected; names, duplicate values, Date flooring, missing/infinite
candidates and registered/default labels retain the source behavior. A zero range
bypasses the callback. Date/datetime constructors in ggplot2 have no `n.breaks`
formal: explicit counts in this corpus set the inherited scale field, and the
adapter uses `GuideTickArguments.count`. This does not claim constructor parity.

The pinned R 4.6.1 / ggplot2 4.0.3 corpus contains 4,500 primary cases, 180
width/format precedence cases and 50 zero-range cases. Across seconds, milliseconds,
microseconds and nanoseconds, 18,920 native comparisons check 9,760 successes and
9,160 expected errors. Actual callback inputs, types, timezone, names, count identity,
visible labels and break values match the source; finite positions use a `1e-8`
logical-pixel bound over an independently fixed `[100, 540]` range. Callback count
is not tied to R's build schedule; every actual call is checked against its source
input. The Date/datetime limits use an explicit exact origin in all four units.

Fractional datetime callback values retain numeric offsets with axis timestamp
metadata and use the existing fractional guide projection. They are accepted only
for selected typed callback results, not as arbitrary untyped timestamp input.
Source observation mapping and exact subsecond viewport policy remain unchanged.
Reference epoch rounding is shared for visibility comparisons. Fractional Date offsets round downward
before timestamp conversion and then floor to whole dates. Default timestamp formatting shares the aesthetic
guide owner. Width selection retains outside candidates until formatting; repeated
hours use the reference wall-label parse's earlier instant. The unchanged fine-time
and promoted-minor-midpoint regression tests pass.

Actual Python and WASM each passed 27,025 main states, 1,280 precedence states
and 400 zero-range states; all records are exactly equal. The 25 source replacements additionally compare fresh and updated layouts,
including the reference empty-data label error, and preserve held scenes and plot
serialization. All 12 Python/WASM publication files are byte-identical and were
inspected as PNG, independently rendered SVG and Poppler-rendered PDF. Named duplicate
`first`/`again` labels intentionally overlap under the explicit Preserve policy;
Date's floored outside endpoint is absent. This slice is qualified; the final core
refresh passes all 584 tests, alongside strict Clippy, rustdoc and repository checks.

Evidence and commands:

- Source runner: `python3 tools/reference/r/run.py tools/reference/r/positional-temporal-break-functions.R`, with independent `--overrides` and `--zero-range` captures. Logs: `/tmp/ggplot-positional-temporal-break-{reference,overrides-reference,zero-reference}.log`.
- Focused native and Linux tests include `ggplot_positional_temporal_break_functions`, the existing temporal label and break tests, `ggplot_time` and `ggplot_minor_breaks`. Logs: `/tmp/ggplot-positional-temporal-break-{core,regression,linux-python}.log`.
- Full core, strict Clippy, rustdoc and repository checks: `/tmp/ggplot-positional-temporal-break-{full-core,clippy,doc,repository}.log`. The final complete core run passed 584 tests.
- Actual adapters: `scripts/bindings/ggplot_positional_temporal_break_functions.py` / `.cjs`, with the same optional controls and a focused `--replacements-only` check. Python records: `target/ggplot-temporal-precision/linux-target/positional-temporal-break{-python,-overrides-python,-zero-python,-replacements-python}`. WASM records: `target/ggplot-positional-temporal-break-functions/{wasm-final,overrides/wasm,zero/wasm,replacements/wasm}`.
- Exact host comparison: `target/ggplot-positional-temporal-break-functions/comparison.json`. Publication review: `target/ggplot-positional-temporal-break-functions/inspection/review-{1,2}.png`, containing all 12 renders. The aggregate primary-authoring runner includes these routes; it has not been rerun as an aggregate.

GG-04 remains in progress. Minor-break functions, palette callbacks and complete
constructor/cross-argument reconciliation remain open. A separate 3,200-case numeric
minor-callback source corpus is captured; its implementation is not qualified here.

## Joint positional binned limit and break functions — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. Requirements: GG2-03/FIX-GG04.
The v35 positional bin selector now composes with registered limits: initial limits
feed one shared cut selection, initial classification is checked, then the retained
cuts feed the post-statistic limit reset. Scalar classification boundaries are cached
before reset, avoiding a second population scan. The shared numeric limit evaluator
retains NULL versus empty results where this composition needs that distinction.

All 1,120 pinned source builds match 1,680 native route comparisons (1,034 successful,
646 expected errors). They cover four transformations, five populations, show-limits
on/off, seven limit operations, named/domain cuts, and both default/registered labels.
Successful callback arguments, names, mapped values and finite panel/guide positions
match the reference. Every actual limits callback input is checked against a source
input; multiplicity is not claimed. Invalid limit vectors can fail typed axis preflight
before a pure label callback that R attempts; callback scheduling on rejected frames
is outside this proof. Existing independent break-only fixtures remain unchanged.

Source edge behavior is retained: NULL limit assignment can fail where an empty vector
succeeds; a partly missing infinite range has no upper censor bound; an empty source
with no selected cuts retains its raw empty/missing limits and the core's empty-range
`[+Inf, -Inf]` sentinel. No finite extent is invented. Empty compiled layers have no
source classification calls. Nonempty invalid cut boundaries fail before reset.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-binned-joint-functions.R`;
  fixture `fixtures/parity/ggplot2/positional-binned-joint-functions.json`;
  `/tmp/ggplot-positional-binned-joint-reference.log`.
- Native focused corpus: `/tmp/ggplot-positional-binned-joint-core.log`;
  all 581 full core tests including doctests pass:
  `/tmp/ggplot-positional-binned-joint-full-core.log`.
- Offline Linux runs joint and independent positional break/bin/label and mapped-break
  regressions and rebuilds the real Python extension:
  `/tmp/ggplot-positional-binned-joint-linux.log`.
- Actual Python/WASM each pass 2,752 original/edited/replacement states with stable v35
  round-trips and 38 supported population replacements. Two reversed empty-source
  function cases are reference errors, covered by the corpus rather than successful
  replacement cases. Held scenes remain immutable; replacement guides match batches.
  Scripts use `ggplot_positional_binned_break_functions.{py,cjs} --joint`;
  logs `/tmp/ggplot-positional-binned-joint-{python,wasm,wasm-build}.log`.
- Records: `target/ggplot-temporal-precision/linux-target/positional-binned-joint-python/`
  and `target/ggplot-positional-binned-joint-functions/wasm/`; `comparison.json` in the
  latter parent checks exact labels/positions and source-backed inverse-log value
  tolerance. Eleven publications are byte-identical; one SVG differs only in numeric
  tick metadata. All four PNG/SVG/Poppler PDF samples were inspected in `inspection/`.
  The reversed fixed-function example correctly has missing mapped points and retains
  its guide; it is not the same policy as authored fixed limits on a reversed scale.
- Strict Clippy and rustdoc pass:
  `/tmp/ggplot-positional-binned-joint-{clippy,doc}.log`.

Joint positional binned functions are qualified within these boundaries. Temporal
positional/minor-break functions, palette callbacks and constructor reconciliation
remain GG-04 work. The aggregate primary authoring runner includes joint proofs but
was not rerun in full. GG-04, GG-05–19 and cumulative gates remain open.

## Named break results with default labels — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Pinned numeric, binned, positional and Date/datetime break fixtures now accept
`--default-names` to select a focused named-result corpus without changing their
original registered-label expectations. The first native probes reproduced numeric
labels such as `10.0` where the source returned `last`, and incorrect binned color
key visibility for named infinite candidates. The shared numeric owner now retains
returned names as default labels with the normal label-byte budget. Named binned
color keys use the same parsed/censored candidate owner as function-labelled keys.
Temporal explicit formats and positional explicit numeric formats keep precedence.
No new wire version or host computation was introduced.

All 300 focused source builds match 600 native comparisons (four timestamp units),
including 44 expected binned errors. Thirty further temporal width/format builds
match 120 native comparisons. Actual Python and WASM each pass 1,396 states with
exactly equal records: numeric 160, binned 116, positional 80, temporal 800, and
format/width overrides 240. Original and edited plots retain their v33/v34 wire
round-trips. The four numeric positional samples yield 12 byte-identical PNG/SVG/PDF
publications; all were independently rendered and inspected. Duplicate `first` and
`again` labels overlap intentionally under the explicit `Preserve` guide policy.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/{numeric,binned,positional,temporal}-break-functions.R --default-names`
  and `temporal-break-overrides.R --default-names`, run separately;
  fixtures `*-break-default-names.json` and `temporal-break-default-overrides.json`;
  logs `/tmp/ggplot-break-default-names-*-reference.log`.
- Reproductions: `/tmp/ggplot-break-default-names-before.log`; fixes:
  `/tmp/ggplot-break-default-names-core.log`,
  `/tmp/ggplot-break-default-names-positions-temporal.log` and
  `/tmp/ggplot-break-default-names-overrides-core.log`.
- All 579 full core tests including doctests pass:
  `/tmp/ggplot-break-default-names-full-core.log`; the later override test passes
  separately. Linux runs all four break test targets plus the final override:
  `/tmp/ggplot-break-default-names-linux.log` and
  `/tmp/ggplot-break-default-names-overrides-linux.log`.
- Fresh Python/WASM builds and actual runs:
  `/tmp/ggplot-break-default-names-{python,linux,wasm-build}.log`,
  `/tmp/ggplot-break-default-names-{numeric,binned,positional,temporal,overrides}-wasm.log`.
- Records: `target/ggplot-temporal-precision/linux-target/break-default-names/`
  and `target/ggplot-break-default-names/`; exact comparisons are `comparison.json`
  and `overrides-comparison.json` in the latter directory. The positional
  `inspection/` directory retains the inspected PNG/SVG/Poppler PDF renders.
- Strict Clippy and rustdoc: `/tmp/ggplot-break-default-names-{clippy,doc}.log`.

This closes the reproduced default-name interaction in the qualified break routes.
It does not close joint positional limit/break functions, temporal positional/minor
break functions, palette callbacks, constructor reconciliation, GG-04 or GG-05–19.
The aggregate authoring runner includes these proofs but was not rerun in full.

## Binned positional break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Wire v35 adds registered selectors to the positional bin policy and retains selected
names in prepared classification caches. One nonempty-population evaluation supplies
classification, post-statistic interval mapping and guide cuts; empty views select
against the expanded panel. Callback count capability follows the shared binned
owner, including `n.breaks` preference and zero counts independent of automatic nice
selection. Serialization checks registration without evaluating selectors. Native-only
operations cannot be serialized; missing registrations, downgraded versions and
competing fixed cuts are rejected.

The 5,760 pinned source builds cover four transformations, five populations, full or
absent limits, show-limits on/off, three function signatures, three counts, four result
modes and registered/default labels. Registered labels run through both axis and scale
policy routes. All 8,640 native route comparisons pass (5,940 successes, 2,700 expected
errors), including callback inputs/counts/names, panel ranges, visible labels and mapped
source values. Added limits precede named cuts when sorting/deduplicating and acquire
blank default labels, as in the reference. Nonfinite populations retain their names.
The original raw selector remains authored; prepared caches contain materialized cuts.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-binned-break-functions.R`;
  `fixtures/parity/ggplot2/positional-binned-break-functions.json`;
  `/tmp/ggplot-positional-binned-break-reference.log`.
- Native corpus and wire tests: `/tmp/ggplot-positional-binned-break-core.log`;
  the final mapped-value assertions pass in `/tmp/ggplot-positional-binned-break-mapping.log`.
- Linux corpus, position-bin, positional-label and prior break-function regressions:
  `/tmp/ggplot-positional-binned-break-linux.log`; final mapping assertions:
  `/tmp/ggplot-positional-binned-break-mapping-linux.log`.
- Full native core: all 575 tests including doctests pass,
  `/tmp/ggplot-positional-binned-break-full-core.log`. Strict Clippy and rustdoc pass:
  `/tmp/ggplot-positional-binned-break-{clippy,doc}.log`.
- Fresh actual Python/WASM: each passes 14,620 states, including 40 replacements,
  stable v35 round-trips, batch/update guide equality and held-scene immutability.
  Scripts: `scripts/bindings/ggplot_positional_binned_break_functions.{py,cjs}`;
  logs: `/tmp/ggplot-positional-binned-break-{python,wasm,wasm-build}.log`.
- Records: `target/ggplot-temporal-precision/linux-target/positional-binned-break-python/`
  and `target/ggplot-positional-binned-break-functions/wasm/`. The committed comparator
  checks all structure, labels and positions exactly. Inverse-log tick values differ
  by at most `8.881784197001252e-16`, below the source-backed `3e-12 * max(1, abs(value))`
  tolerance. Eleven publications are byte-identical; the twelfth is SVG with only
  that numeric tick metadata difference. Its rendered geometry/text is identical.
  Comparison: `target/ggplot-positional-binned-break-functions/comparison.json`.
- All four PNGs, four independently rendered SVGs and four Poppler PDF renders were
  inspected at `target/ggplot-positional-binned-break-functions/inspection/`: named
  middle labels, blank endpoint labels and transformed point placement are retained.

This slice does not qualify joint positional limit/break functions (explicitly
unsupported), temporal positional/minor-break functions, palette callbacks, or the
remaining constructor inventory. Named default labels in earlier numeric/temporal
break routes have focused follow-up fixtures and currently failing native probes;
that follow-up remains open. Cumulative GG-04, GG-05–19 and publication/platform gates
remain open; the aggregate authoring proof runner was updated but not rerun in full.

## Discrete positional break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The v34 guide selector now supports automatic categorical, band and point axes through
the shared discrete guide owner. It receives the complete trained category domain,
without count or temporal arguments, and applies existing matching, first-duplicate
and named-label semantics. Untrained empty scales skip break/label callbacks; trained
scales with no selected keys still evaluate positional label functions, unlike mapped
discrete color-guide selection. Default labels retain returned names and the existing
missing-category `NA` display policy. Functions do not alter category mapping.

The pinned `positional-discrete-break-functions.R` corpus has 400 ggplot2 4.0.3 /
R 4.6.1 builds across five populations, trained/explicit limits, drop/missing-level
policies, domain/mixed-named/numeric/empty/NULL outputs and registered/default labels.
All cases pass natively over automatic/band/point axes; registered labels are exercised
through both axis formatting and positional scale policy, giving 1,800 comparisons.
Tests check callback inputs/names, empty/trained bypass, visible keys and labels.
As with numeric positional functions, callback input/result equivalence is qualified;
internal invocation multiplicity across R construction and native layout passes is not.

Actual Python/WASM each pass 3,660 states: 3,600 original/edited states plus 60 retained
versus fresh-batch layout replacements. Records are exactly equal and v34 definitions
round-trip unchanged. Held scenes and original wire remain unchanged after replacement.
Twelve SVG/PDF/PNG files match byte for byte across hosts. Four PNGs and four
Poppler-rendered PDF pages were inspected: explicit category order, missing-category
placement, indexed labels and returned first-duplicate names are retained.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-discrete-break-functions.R`;
  fixture `fixtures/parity/ggplot2/positional-discrete-break-functions.json`;
  `/tmp/ggplot-positional-discrete-break-reference.log`.
- Native core: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`;
  573 tests including doctests pass, `/tmp/ggplot-positional-discrete-break-full-core.log`.
- Four focused native/Linux tests across `ggplot_positional_discrete_break_functions`,
  `ggplot_positional_break_functions`, and `ggplot_positional_discrete_label_functions`
  pass; `/tmp/ggplot-positional-discrete-break-core-final.log` and
  `/tmp/ggplot-positional-discrete-break-linux.log`. Final key assertions also pass
  in native/Linux; `/tmp/ggplot-positional-discrete-break-key-test.log` and
  `/tmp/ggplot-positional-discrete-break-key-linux.log`.
- Fresh actual Python extension (`extension-module,extension-proof`) in offline Linux
  arm64 Rust 1.97.1; `scripts/bindings/ggplot_positional_discrete_break_functions.py`;
  `target/ggplot-temporal-precision/linux-target/positional-discrete-break-python/`;
  `/tmp/ggplot-positional-discrete-break-python.log`.
- Fresh WASM `extension-proof` build with wasm-bindgen 0.2.128 Node module;
  `scripts/bindings/ggplot_positional_discrete_break_functions.cjs`;
  `target/ggplot-positional-discrete-break-functions/wasm/`;
  `/tmp/ggplot-positional-discrete-break-wasm-build.log`,
  `/tmp/ggplot-positional-discrete-break-wasm.log`.
- Exact record/publication comparisons at
  `target/ggplot-positional-discrete-break-functions/comparison.json`;
  inspected PNG samples 40/41/42/43 and PDF previews under
  `target/ggplot-positional-discrete-break-functions/pdf-preview/`.
  The primary-authoring aggregate includes this runner, but the full aggregate was
  not rerun.
- Strict core/example all-target Clippy and core rustdoc pass;
  `/tmp/ggplot-positional-discrete-break-clippy.log`,
  `/tmp/ggplot-positional-discrete-break-doc.log`.
  Formatting, repository checks, Python/Node syntax and `git diff --check` pass.

GG-04 remains IN PROGRESS. Binned/temporal positional break functions, minor-break
functions, palette callbacks and complete constructor/argument reconciliation remain
open, followed by GG-05–19 and cumulative gates. Secondary/additional-guide and
arbitrary combined argument parity are not established by this primary-axis corpus.
Next: binned positional functions shared by classification and guide selection.

## Numeric positional break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
`GuideStyle.breaks_function` and axis/guide builder methods select reference major
candidates from retained panel limits using the shared registered break engine.
The v34 descriptor requires explicit registration; original definitions remain
immutable and older definitions keep their earlier minimum versions. Native-only
functions reject portable serialization, missing registrations reject import/build,
and a v33 downgrade rejects. Replacing function selection with authored tick values
clears the operation and its version requirement.

Numeric selection uses inverse-transformed expanded panel limits, including reversed
order and nonfinite limits. It forwards the optional count only to `n` registrations.
Returned names remain aligned through positional censoring, duplicate/missing results
reach registered labels, and transform-specific NULL failures precede labels.
Unsupported axis families reject before evaluation. Mark mapping remains owned by the
retained scale. Layout may evaluate pure functions during multiple measurement passes;
this slice qualifies callback inputs/results, not ggplot2's internal invocation
multiplicity or error-path fallback calls.

The pinned `positional-break-functions.R` corpus has 1,440 ggplot2 4.0.3 / R 4.6.1
builds across identity/sqrt/log10/reverse, five populations, trained/explicit limits,
three signatures, three counts and domain/mixed-named/empty/NULL outputs. Native tests
match 648 successful panels and 792 expected rejections. Every observed native call
is checked against the source's panel-selection call (3e-12 numeric tolerance),
including label inputs/names. Finite-panel visible labels and transformed tick values
match the reference. An extra native/Linux test checks registration, portability,
round-trip/downgrade rejection and clearing function selection without evaluation.

Actual Python/WASM each pass 2,128 states: 1,296 original/edited successes, 792 errors
and 40 replacements. Each replacement compares newly prepared retained-chart guide
ticks against fresh batch layout and preserves the original wire and held scene.
Host records are exactly equal; 12 SVG/PDF/PNG files are byte-identical across hosts.
Four PNGs and four Poppler-rendered PDF pages were inspected: marks, reversed mapping
and endpoint/midpoint labels appear as expected. The explicit `Preserve` label policy
intentionally overlaps duplicate `3/7` and `4/7` labels at the same endpoint; this is
not a collision-layout acceptance claim.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-break-functions.R`;
  `fixtures/parity/ggplot2/positional-break-functions.json`;
  `/tmp/ggplot-positional-break-reference.log`.
- Native core: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`;
  571 tests including doctests pass, `/tmp/ggplot-positional-break-full-core.log`.
  The added wire/registration test subsequently passes with the corpus test;
  `/tmp/ggplot-positional-break-contract.log`.
- Nine focused native/Linux tests across `ggplot_positional_break_functions`,
  `ggplot_positional_label_functions`, `ggplot_break_functions`, and
  `ggplot_temporal_break_functions` pass; `/tmp/ggplot-positional-break-core-final.log`
  and `/tmp/ggplot-positional-break-linux.log`. Both positional break tests subsequently
  pass in offline Linux arm64 Rust 1.97.1; `/tmp/ggplot-positional-break-contract-linux.log`.
- Fresh actual Python extension with `extension-module,extension-proof`, then
  `scripts/bindings/ggplot_positional_break_functions.py`;
  `target/ggplot-temporal-precision/linux-target/positional-break-python/`;
  `/tmp/ggplot-positional-break-python.log`.
- Fresh WASM `extension-proof` build and wasm-bindgen 0.2.128 Node module, then
  `scripts/bindings/ggplot_positional_break_functions.cjs`;
  `target/ggplot-positional-break-functions/wasm/`;
  `/tmp/ggplot-positional-break-wasm-build.log`, `/tmp/ggplot-positional-break-wasm.log`.
- Exact comparisons: `target/ggplot-positional-break-functions/comparison.json`;
  inspected PNG samples 17/377/737/1097 and PDF previews at
  `target/ggplot-positional-break-functions/pdf-preview/`.
  The proof is included in `scripts/run_primary_authoring_proofs.py`; its full aggregate
  run was not repeated.
- Strict core/example all-target Clippy and core rustdoc pass;
  `/tmp/ggplot-positional-break-clippy.log`, `/tmp/ggplot-positional-break-doc.log`.
  Formatting, repository, Python/Node syntax and `git diff --check` pass.

GG-04 remains IN PROGRESS. Discrete/binned/temporal positional break functions,
minor-break functions, palette callbacks and complete constructor/argument
reconciliation remain open, followed by GG-05–19 and cumulative acceptance gates.
Additional-guide/secondary-axis function parity is not claimed by this primary-axis
corpus. Next: discrete positional break selection.

## Temporal break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Date/datetime color and size guides now evaluate registered break functions after
population training. `ScaleBreaksInput.temporal` supplies the existing immutable
origin/unit/Date/calendar context; domain values remain origin-relative numeric
coordinates. `ScaleBreaksOutput.temporal` declares the typed result representation.
The temporal route requires the supplied representation and rejects NULL/untyped
numeric results before label evaluation, preserving the pinned transform contract.
No host executable objects or ambient timezone lookup enter core. These native
metadata additions do not change the v33 portable descriptor.

Continuous temporal selection supplies optional `n` only to registrations accepting
it. Constant domains bypass the function; empty populations without limits bypass
both functions; all-missing populations preserve their nonfinite callback limits.
Returned names and duplicate/missing/nonfinite candidates reach labels before guide
visibility filtering. Explicit `date_breaks` widths override break functions;
`date_labels` formatting independently overrides label callbacks.

The pinned `temporal-break-functions.R` corpus has 4,500 ggplot2 4.0.3 / R 4.6.1
builds across color/size, Date and UTC/New York datetime, spring/fall DST windows,
five populations, trained/explicit limits, three callback signatures, three counts
and typed domain/mixed/empty/NULL/untyped results. Native comparisons run each case
in seconds/milliseconds/microseconds/nanoseconds: 12,240 successes and 5,760 expected
rejections. They check exact callback counts, typed limits, count/name metadata,
label inputs, visible labels and size guide values. An additional
`temporal-break-overrides.R` corpus contributes 30 source builds / 120 successful
native unit cases for width/format/combined precedence.

Actual Python/WASM each pass 30,280 function states (24,480 successful original/edited
states, 5,760 errors and 40 replacements) plus 240 override states. Records are exactly
equal across hosts. Definitions round-trip as v33; replacements compare retained and
fresh batch guides while preserving original wire and held publication scenes.
The host proofs use core-installed example callbacks; native tests inspect callback
metadata directly. Reference mapped values are retained in fixtures but are not
independently compared in this slice. No new publication files or painting claims.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/temporal-break-functions.R`
  and the same runner with `tools/reference/r/temporal-break-overrides.R`;
  fixtures `fixtures/parity/ggplot2/temporal-break-functions.json` and
  `fixtures/parity/ggplot2/temporal-break-overrides.json`;
  `/tmp/ggplot-temporal-break-reference.log`, `/tmp/ggplot-temporal-break-overrides-reference.log`.
- Native: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`;
  569 tests including doctests pass, `/tmp/ggplot-temporal-break-full-core.log`.
  The subsequently added override test passes alongside the main corpus test in
  `/tmp/ggplot-temporal-break-overrides-core.log` (no later production change).
- Offline Linux arm64 Rust 1.97.1: eight focused tests across
  `ggplot_temporal_break_functions`, `ggplot_break_functions`, and
  `ggplot_temporal_label_functions` pass; fresh Python extension built using
  `extension-module,extension-proof`; `/tmp/ggplot-temporal-break-linux.log`.
  Both temporal break tests subsequently pass with the added override corpus;
  `/tmp/ggplot-temporal-break-overrides-linux.log`.
- Actual Python: `scripts/bindings/ggplot_temporal_break_functions.py` against
  `target/ggplot-temporal-precision/linux-target/python-module`, followed by the
  same script with final argument `overrides`; records under the same Linux target's
  `temporal-break-python/` and `temporal-break-overrides-python/` directories;
  `/tmp/ggplot-temporal-break-python.log` and the override Linux log above.
- Actual WASM: fresh `cargo build -p chart-wasm --features extension-proof --target wasm32-unknown-unknown --locked --target-dir target/ggplot-positional-wasm`,
  wasm-bindgen 0.2.128 Node module; `scripts/bindings/ggplot_temporal_break_functions.cjs`
  with and without final argument `overrides`; records at
  `target/ggplot-temporal-break-functions/wasm/records.json` and
  `target/ggplot-temporal-break-functions/overrides/wasm/records.json`;
  `/tmp/ggplot-temporal-break-wasm-build.log`, `/tmp/ggplot-temporal-break-wasm.log`,
  `/tmp/ggplot-temporal-break-overrides-wasm.log`.
- Exact host comparisons: `target/ggplot-temporal-break-functions/functions-comparison.json`
  and `target/ggplot-temporal-break-functions/overrides-comparison.json`.
  Both proof modes are included in `scripts/run_primary_authoring_proofs.py`;
  the complete aggregate runner was not rerun.
- Strict core/example all-target Clippy and core rustdoc pass:
  `/tmp/ggplot-temporal-break-clippy.log`, `/tmp/ggplot-temporal-break-doc.log`.
  Formatting, repository checks, host-script syntax and `git diff --check` pass.

GG-04 remains IN PROGRESS. Positional and minor-break functions, palette callbacks,
complete constructor/argument reconciliation and cumulative gates remain open.
Temporal binned and arbitrary cross-argument combinations are not qualified by this
continuous color/size corpus. Next: positional break selection, then the remaining
GG-04 contracts before GG-05–19.

## Numeric binned break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Binned callbacks execute once after population/limit training, suppress automatic
break generation and domain extension, and retain their result as explicit prepared
cuts for both mapping and guides. Callback limits are inverse-transformed and sorted;
missing inverse endpoints are dropped. Count capability is captured at registration:
binned selection supplies `n.breaks` preferentially, otherwise `n`, with default five.
`ScaleBreaksInput.count_argument` identifies the supplied argument without inspecting
host executable objects. Named results retain names through label selection. Empty
populations without authored limits bypass evaluation; constants do not. NULL retains
its transform-specific error behavior. Prepared names are bounded and must align with
prepared cuts. Definitions remain immutable v33 descriptors; build/import/serialization
do not evaluate callbacks. Competing fixed breaks reject.

The pinned `binned-break-functions.R` corpus contains 2,880 ggplot2 4.0.3 / R 4.6.1
builds: color via `scale_colour_steps`, size via generic `binned_scale`/`pal_area`,
four transforms, five populations, trained/explicit limits, three callback signatures,
three requested counts and domain/mixed-named/empty/NULL results. Native tests compare
1,800 successful builds and 1,080 rejected builds, exact callback arguments/counts,
label inputs/names, color labels, size boundary values/labels and mapped values for
both channels (3e-12 numeric tolerance; exact resolved color bytes). The source's
constant color-key row-count failure precedes label evaluation; constant size guides
retain two keys at the same source boundary. A focused test separately verifies
`n.breaks` preference when both names are accepted, one-time evaluation shared across
repeated sampling/guide selection, immutable source descriptors and fixed-break
rejection. Its precedence expectation comes from the pinned `ScaleBinned$get_breaks`
implementation; it is not another generated corpus case.

Actual Python and WASM each pass 4,686 states: 1,800 successful original/edited pairs,
1,080 expected failures and six replacements. Host records compare guide semantics
against the source, round-trip v33 definitions, rebuild edits and compare retained
charts against fresh batch charts across missing, empty and restored populations.
Mapped-value comparisons are native; host records are exactly equal across adapters.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/binned-break-functions.R`;
  fixture `fixtures/parity/ggplot2/binned-break-functions.json`;
  `/tmp/ggplot-binned-break-reference.log`.
- Native full core: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`;
  567 tests including doctests pass, `/tmp/ggplot-binned-break-full-core.log`.
  Final metadata changes pass ten focused tests across `ggplot_break_functions`,
  `ggplot_binned_label_functions`, and `ggplot_label_functions`;
  `/tmp/ggplot-binned-break-final-core.log`. The subsequently added precedence/caching
  test passes with all five break-function tests in `/tmp/ggplot-binned-break-count-core.log`.
- Offline Linux arm64 Rust 1.97.1: the same ten focused tests and an actual Python
  extension rebuilt with `extension-module,extension-proof`, then
  `scripts/bindings/ggplot_binned_break_functions.py`;
  `/tmp/ggplot-binned-break-linux-final.log`.
- Actual WASM: extension-proof build, repository-local wasm-bindgen 0.2.128 nodejs
  output, all `packages/wasm/*.cjs`/`*.cts` adapters copied, then
  `scripts/bindings/ggplot_binned_break_functions.cjs`;
  `/tmp/ggplot-binned-break-wasm-build.log` and `/tmp/ggplot-binned-break-wasm.log`.
- Outputs: `target/ggplot-binned-break-functions/wasm/records.json`,
  `target/ggplot-temporal-precision/linux-target/binned-break-python/records.json`,
  and `target/ggplot-binned-break-functions/comparison.json`: 4,686 exact records,
  including six replacements per host.

Strict core/example Clippy, core rustdoc, formatting, repository validation,
Python/Node syntax and diff checks pass. The default PATH wasm-bindgen was 0.2.122 and
rejected the 0.2.128 module; the successful proof used the existing pinned local binary.
The aggregate primary proof runner includes this pair but has not been rerun in full.
No new publication/guide painting, warning-text parity, all constructor forwarding,
temporal/positional break callbacks or combined callback precedence is claimed.
GG-04 and cumulative gates remain open; temporal/positional/minor-break callbacks,
palette callbacks and constructor reconciliation are next.

## Discrete break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The existing pure break registry now supports reference discrete mapped scales. Core
passes the trained domain without a count, converts returned keys through the existing
discrete selector, retains the first duplicate and its name, and supplies selected
values/names to registered labels. Named results also drive default labels. Untrained
empty domains bypass break evaluation; empty selection bypasses label evaluation.
Competing authored breaks/names reject. Wire v33 and registry installation contracts
are unchanged; no host executable object enters core.

`discrete-break-functions.R` pins 600 ggplot2 4.0.3 / R 4.6.1 builds: six channels
(color, shape, size, alpha, linewidth, linetype), five populations, trained/explicit
limits, domain/mixed/empty/NULL/numeric return modes and default/indexed labels.
Native tests compare exact callback arguments/counts, names, selected non-color keys
and labels, plus color labels. Numeric-to-text matching covers 1 and missing values;
it does not establish every R coercion or arbitrary typed return. Actual Python/WASM
each pass 1,230 states (600 original and 600 edited plus 30 replacements), with exact
record equality and stable v33 round-trips. Replacement checks compare each retained
chart to a fresh batch chart across nullable, all-missing, empty and numeric-text data.

Evidence:

- Source: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/discrete-break-functions.R`;
  fixture `fixtures/parity/ggplot2/discrete-break-functions.json`.
- Native: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`;
  566 tests including doctests pass, log `/tmp/ggplot-discrete-break-core.log`.
- Offline Linux: `ggplot_break_functions`, `ggplot_discrete_guides`, and
  `ggplot_label_functions` integration targets: 12 tests pass. Actual Python extension
  rebuilt with `extension-module,extension-proof`, then
  `scripts/bindings/ggplot_discrete_break_functions.py`; `/tmp/ggplot-discrete-break-linux.log`.
- Actual WASM: extension-proof build, wasm-bindgen 0.2.128 nodejs output and
  `scripts/bindings/ggplot_discrete_break_functions.cjs`; logs
  `/tmp/ggplot-discrete-break-wasm-build.log` and `/tmp/ggplot-discrete-break-wasm.log`.
- Outputs: `target/ggplot-discrete-break-functions/wasm/records.json`,
  `target/ggplot-temporal-precision/linux-target/discrete-break-python/records.json`,
  and `target/ggplot-discrete-break-functions/comparison.json`.

Strict core/example Clippy (`/tmp/ggplot-discrete-break-clippy.log`) and core rustdoc
(`/tmp/ggplot-discrete-break-doc.log`) pass, as do
format, repository validation, Python/Node syntax and `git diff --check`.

The primary aggregate runner includes the new host proofs but has not been rerun in
full. No new publication/guide painting or warning-text parity is claimed. GG-04 and
cumulative gates remain open. Binned/temporal/positional/minor break callbacks, palette
callbacks and constructor argument reconciliation remain. The binned reference corpus
has been generated for the next slice; it is not implementation acceptance evidence.

## Numeric continuous break functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Core owns registered break selection through `CustomScaleBreaks`, bounded values/names
and captured count capability. `MappedScaleSpec.breaks_function` selects an installed
operation after population/limit training, at guide selection. It requires a reference
continuous scale and rejects competing fixed breaks or unsupported guide families.
Portable definitions require wire v33 and explicit installation; authoring, importing
and serialization validate without evaluating callbacks. Constant and untrained empty
domains skip evaluation. Complete returned vectors preserve names, duplicate/missing/
nonfinite values and reference order through label selection. NULL is distinct from
an empty numeric vector: sqrt/log/reverse transformation rejects NULL. Continuous
count is supplied only when the registered implementation accepts `n`; captured native
capability avoids introspecting executable source or embedding host objects in core.

The pinned `numeric-break-functions.R` oracle contains 2,880 builds: color through
`scale_colour_continuous`, size through generic `continuous_scale` with `pal_area`,
four transforms, five populations, trained/explicit limits, three callback signatures,
three requested counts and domain/mixed-named/empty/NULL results. This deliberately
uses the generic size constructor: `scale_size_continuous` rejects `n.breaks`, even
when NULL; constructor argument reconciliation remains open. Source results contain
2,448 successes and 432 transformation errors. Native tests compare exact invocation
counts, argument presence, input order, returned names, label vectors, color labels
and visible size keys (numeric tolerance 3e-12 absolute/relative). Registered-operation
absence, duplicate installation, native-only serialization and wire downgrade reject
without executing a callback.

Actual Python/WASM each pass 5,336 states: original/edited successful plots, expected
errors, stable v33 round-trips and eight replacement-versus-batch checks. Records are
exactly equal across hosts. Shared example operations exercise the same core registry;
no Python/JS implementation of scale arithmetic is added. Evidence:

- Oracle: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/numeric-break-functions.R`;
  committed fixture `fixtures/parity/ggplot2/numeric-break-functions.json`.
- Native focused: `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust --test ggplot_break_functions`.
- Offline Linux: focused `ggplot_break_functions`, `ggplot_label_functions`,
  `ggplot_continuous_guides`, then actual Python extension build and
  `scripts/bindings/ggplot_break_functions.py`; log `/tmp/ggplot-break-linux.log`.
- WASM extension-proof build, wasm-bindgen 0.2.128, then
  `scripts/bindings/ggplot_break_functions.cjs`; logs `/tmp/ggplot-break-wasm-build.log`
  and `/tmp/ggplot-break-wasm.log`.
- Outputs: `target/ggplot-numeric-break-functions/wasm/records.json`,
  `target/ggplot-temporal-precision/linux-target/numeric-break-python/records.json`,
  `target/ggplot-numeric-break-functions/comparison.json`.

The aggregate primary proof runner includes this pair but has not been rerun in full.
No new guide painting/publication or warning-text parity is claimed. Discrete, binned,
temporal and positional/minor break functions, palette callbacks and constructor
argument reconciliation remain open; GG-04 and cumulative gates stay IN PROGRESS.
All 565 core tests (including doctests), 14 focused Linux tests, four export
`scale_distributions` regressions, strict core/example Clippy and core rustdoc pass.
The final two Linux registry tests also pass in `/tmp/ggplot-break-linux-registration.log`.
Full core: `/tmp/ggplot-break-all-core.log`; Clippy: `/tmp/ggplot-break-clippy.log`;
rustdoc: `/tmp/ggplot-break-doc.log`; export: `/tmp/ggplot-break-export.log`.
`mise run fmt`, `python3 scripts/check_repository.py`, host-script syntax and
`git diff --check` pass. The initial full-core invocation exposed a missing explicit
`None` in the benchmark descriptor after the schema extension; the benchmark and
export test literals now include the optional field, and final suites pass.

## Non-color temporal labels and single callback execution — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Date/datetime size, alpha and linewidth now retain temporal guide candidates and
labels in the existing numeric-guide DTO. Temporal guide evaluation moves out of
mapped-scale sampling validation into the actual guide consumer. This removes the
extra color-label invocation during scale construction and prevents the equivalent
duplicate on the new non-color metadata route. Descriptor/type/sampling validation
remains in its existing owners; standalone callers resolve guides explicitly.

- `tools/reference/r/noncolor-temporal-label-functions.R` captures **1,200** actual
  R 4.6.1 / ggplot2 4.0.3 builds across three channels, Date/datetime, UTC and supplied
  New York spring/fall transitions, five populations, four break/format controls and
  four callback modes. Native primary tests execute all cases in seconds,
  milliseconds, microseconds and nanoseconds: **3,540 successes / 1,260 expected errors**.
  They compare exact source values, label vectors, callback count/order, generated
  names, Date/POSIXct classes and timezone context, including explicit-format
  precedence. v32 round trips invoke no callbacks. The original 400-case color
  corpus also now requires exactly the reference callback count and retains its
  actual color-legend label assertions; its colorbar coordinates are not inferred
  from source-domain candidates.
- `ggplot_temporal_label_functions` and `ggplot_temporal_guides` pass **4 focused
  macOS tests**; `/tmp/ggplot-noncolor-temporal-label-native.log`. Adding
  `ggplot_label_functions` gives **9 passing Linux tests** in the offline arm64
  Rust 1.97.1 environment; `target/ggplot-temporal-precision/linux-target/noncolor-temporal-label-linux.log`.
  Full macOS core passes **563 tests including doctests**;
  `/tmp/ggplot-noncolor-temporal-label-all-core.log`. Strict core/example all-target
  Clippy, strict core rustdoc and format/repository/syntax/diff checks pass;
  `/tmp/ggplot-noncolor-temporal-label-{clippy,doc,fmt,repository}.log`.
- Actual rebuilt Python/WASM each pass **8,400 exactly matching non-color states**,
  including 60 replacement-versus-fresh checks with frozen scenes and unchanged
  original authored definitions. The temporal color regression passes **2,800 states**
  per host; its 12 SVG/PDF/PNG publications match across hosts and are byte-identical
  to previously inspected `target/ggplot-temporal-labels/wasm` artifacts.
  Comparison: `target/ggplot-noncolor-temporal-labels/comparison.json`.
  Non-color outputs: `target/ggplot-noncolor-temporal-labels/wasm` and
  `target/ggplot-temporal-precision/linux-target/noncolor-temporal-label-python`;
  corresponding color outputs use `color-wasm` and `noncolor-temporal-color-python`.
  Build/run logs use the same temporal-label prefixes as the other non-color slices.

This qualifies the sampled temporal scale semantics and single-callback execution;
non-color guide painting and full colorbar layout remain GG-05. The complete aggregate
host runner was not run. Next: break/minor-break functions, palette callbacks and
constructor/argument reconciliation, including remaining cross-argument precedence.
GG-04 remains IN PROGRESS; no cumulative gate closes.

## Non-color binned label preparation — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The shared binned candidate parser now supplies non-color guide keys. It censors
source callback inputs, labels the limits independently, combines endpoint and cut
labels, and rejects the reference's empty-after-censoring and incompatible key/label
length cases. The prepared numeric-guide DTO retains source/scale boundaries and
labels. Equally spaced guide placement and endpoint display remain GG-05.

Guide evaluation now uses the already trained mapping before mark geometry checks.
This fixes a reproduced ordering mismatch: log-binned explicit cuts can map an
infinite size, but ggplot2 evaluates both label callbacks before reporting a guide
length error. Preparing guide metadata after mark validation incorrectly skipped the
callbacks. Discrete, continuous and binned guides share one internal owner and reuse
the mapped scale directly; no second mapping compilation or palette engine is added.

- The pinned `noncolor-numeric-label-functions.R` corpus now has **3,240** records:
  the earlier 1,440 continuous cases plus **1,800 binned** cases, including default
  labels. Native binned results match **939 successes / 861 expected errors**.
  Checks include exact callbacks/order/names/missingness, independently labelled
  limits, exact labels, and independently extracted transformed boundaries with
  3e-12 absolute or relative tolerance. Normalized guide coordinates and painting
  are not inferred from these boundary checks. Source and R method inspection logs:
  `/tmp/ggplot-noncolor-numeric-label-reference.log`,
  `/tmp/ggplot-noncolor-guide-source.log`, `/tmp/ggplot-noncolor-bins-source.log`.
- Four focused targets (`ggplot_label_functions`, `ggplot_binned_label_functions`,
  `ggplot_binned_numeric`, `ggplot_binned_styles`) pass **8 tests** on macOS and
  offline Linux arm64 Rust 1.97.1. Logs: `/tmp/ggplot-noncolor-binned-label-native.log`
  and `target/ggplot-temporal-precision/linux-target/noncolor-binned-label-linux.log`.
  Full macOS core passes **562 tests including doctests**;
  `/tmp/ggplot-noncolor-binned-label-all-core.log`. Strict core/example all-target
  Clippy, strict core rustdoc, formatting/repository/syntax/diff checks pass;
  `/tmp/ggplot-noncolor-binned-label-{clippy,doc,fmt,repository}.log`.
- Actual rebuilt Python/WASM each pass **2,748 exactly matching label states**,
  including nine replacement-versus-fresh checks. Registered policies require v32;
  defaults retain their existing version. Every authored round trip is exact.
  Outputs: `target/ggplot-noncolor-binned-labels/wasm` and
  `target/ggplot-temporal-precision/linux-target/noncolor-binned-label-python`.
  Build/run logs use the same prefixes with `wasm-build`, `wasm`, `python-build`,
  and `python` suffixes. `target/ggplot-noncolor-binned-labels/comparison.json`
  records exact host equality and publication regression results.
- The existing numeric host proof now compares the fixture's **full build result**,
  qualifying two previously deferred area-guide failures (constant explicit cuts;
  all-missing data with one authored limit). It passes **370 states**, including
  38 expected errors. Independent standalone Rust raw mapping checks remain intact;
  fixture values/tolerances are unchanged. Existing count-palette host proofs pass
  **392 states**. All 24 SVG/PDF/PNG regression publications match across hosts and
  are byte-identical to the already inspected `target/ggplot-binned-styles/wasm`
  artifacts. No new rendered guide-label claim is made.

The aggregate runner includes the new corpus but was not run in full. Remaining
label work is temporal non-color metadata and callback-count reconciliation, followed
by break/minor-break functions, palette callbacks and complete constructor/argument
reconciliation. GG-04 remains IN PROGRESS; GG-05 owns guide rendering/composition.

## Non-color continuous label preparation — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Primary size, alpha and linewidth mappings retain continuous guide candidates in
`PreparedLayer::numeric_value_guides` and the portable semantic DTO. The existing
continuous resolver evaluates the complete source-space label vector before applying
key visibility. Discrete and numeric records share one internal prepared-guide owner;
no new interpolation, label arithmetic or wire descriptor is introduced. Temporal and
binned guide preparation remain outside this specific extension.

- `tools/reference/r/noncolor-numeric-label-functions.R` generated **2,880** actual
  pinned reference builds at this checkpoint (extended by the later binned slice). This slice qualifies the **1,440 continuous** cases:
  three channels, four transforms, five populations, trained/explicit limits,
  default/explicit/empty breaks and four callback modes. **1,044** succeed and
  **396** reject label-length mismatch. Native primary tests compare exact labels,
  callback order/names/missingness and source/transformed values using 3e-12 absolute
  or relative tolerance. Source log: `/tmp/ggplot-noncolor-numeric-label-reference.log`;
  focused log: `/tmp/ggplot-noncolor-continuous-label-native.log`.
- `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`
  passes **561 tests including doctests**;
  `/tmp/ggplot-noncolor-continuous-label-all-core.log`. Offline Linux arm64 Rust 1.97.1
  passes **16 tests** across `ggplot_label_functions`, `ggplot_continuous_guides` and
  `ggplot_style_palettes`; `target/ggplot-temporal-precision/linux-target/noncolor-continuous-label-linux.log`.
  An initial invocation named a nonexistent numeric-aesthetics test target; that
  invocation is not evidence. The corrected targets above pass. Strict core/example
  all-target Clippy, strict core rustdoc, formatting, repository, syntax and diff
  checks pass; `/tmp/ggplot-noncolor-continuous-label-{clippy,doc,fmt,repository}.log`.
- Actual rebuilt Python/WASM each pass **2,496 states**, including v32 round trips,
  layer edits, expected errors and **12** replacement-versus-fresh checks. All records
  match exactly across hosts; `target/ggplot-noncolor-continuous-labels/comparison.json`.
  Outputs: `target/ggplot-noncolor-continuous-labels/wasm` and
  `target/ggplot-temporal-precision/linux-target/noncolor-continuous-label-python`.
  Build/run logs use `/tmp/ggplot-noncolor-continuous-label-{wasm-build,wasm}.log` and
  `target/ggplot-temporal-precision/linux-target/noncolor-continuous-label-{python-build,python}.log`.

Non-color key painting remains GG-05, and no fresh visual/native GPUI claim is made.
The aggregate runner includes the corpus but was not run in full. The reference's
remaining binned guide cases require a second endpoint-label callback and separate
key assembly. Binned/temporal non-color compositions, break/minor-break functions,
palette callbacks and complete constructor reconciliation remain open; GG-04 stays
IN PROGRESS.

## Non-color discrete label preparation — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
Size, alpha, linewidth, shape and linetype now evaluate discrete guide labels during
primary preparation and retain selected keys/optional labels in
`PreparedLayer::discrete_value_guides` and the portable semantic DTO. Color and
non-color routes share `MappedScale::discrete_guide_entries` and the existing
vector resolver. Disabled/hidden identity guides and explicit empty breaks suppress
callbacks. Mapping, prepared scale descriptors and definition v32 remain unchanged.

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/noncolor-discrete-label-functions.R`
  generates **800** pinned R 4.6.1 / ggplot2 4.0.3 actual builds. All five channels
  have identical selected-key, callback-input/name and result rules across ordinary,
  nullable, all-missing and empty populations; trained/explicit limits; default,
  explicit, named and empty breaks; indexed, missing, short, empty and named labels.
  **695** builds succeed and **105** reject empty label results for nonempty keys.
  Source log: `/tmp/ggplot-noncolor-discrete-label-reference.log`.
- The primary Rust corpus checks all 800 results, exact callback vectors/names,
  v32 round trips and callback deferral through serialization. Short callbacks
  recycle according to the existing reference resolver. Three regression targets
  (`ggplot_label_functions`, `ggplot_discrete_guides`, `ggplot_discrete_null`) pass
  **12 tests** on macOS and offline Linux arm64 Rust 1.97.1. Logs:
  `/tmp/ggplot-noncolor-discrete-label-native.log` and
  `target/ggplot-temporal-precision/linux-target/noncolor-discrete-label-linux.log`.
- `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`
  passes **560 tests including doctests**;
  `/tmp/ggplot-noncolor-discrete-label-all-core.log`. Strict core/example all-target
  Clippy, strict core rustdoc, formatting, repository, syntax and diff checks pass;
  `/tmp/ggplot-noncolor-discrete-label-{clippy,doc,fmt,repository}.log`.
- Rebuilt actual Python and WASM adapters each pass **1,515 states**: successful
  original/edited results, expected errors and **20** replacement-versus-fresh
  checks across all five channels. Definition round trips remain exact. Every
  cross-host record matches exactly; `target/ggplot-noncolor-discrete-labels/comparison.json`.
  WASM module/output: `target/ggplot-noncolor-discrete-labels/{wasm-module,wasm}`;
  Python output/build log: `target/ggplot-temporal-precision/linux-target/noncolor-discrete-label-python`
  and `noncolor-discrete-label-python-build.log`. WASM logs:
  `/tmp/ggplot-noncolor-discrete-label-{wasm-build,wasm}.log`.

This qualifies guide selection, callback execution, retained metadata and errors.
Non-color guide layout/painting remains GG-05; no new visual or native GPUI claim is
made. The aggregate host runner includes this corpus but was not run in full.
Continuous/binned non-color labels, break/minor-break functions, palette callbacks
and complete constructor/argument reconciliation remain open. GG-04 remains IN PROGRESS.

## Registered positional policy routes — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
This closes the primary policy-route limitation recorded by the earlier discrete and
binned label slices. The axis formatter and `GgplotDiscretePosition.guide.labels` /
`GgplotBinnedPosition.labels` registration routes now share final guide evaluation.
Training selects geometric cuts/categories without calling registered labels and
retains the authored policy for later evaluation. Explicit guide formatters take
precedence. Registry validation and definition v32 detection include positional policy
labels; no schema field or host callback implementation was added.

- The existing discrete 640-case/two-family and binned 1,600-case corpora execute both
  routes in Rust. Native tests add missing-registry and v31 downgrade rejection,
  v32 round trips, native-only serialization/point-unit rejection, zero callback calls
  during training/serialization, and explicit formatter override behavior.
  `mise exec -- cargo test -p chart-core --test ggplot_position_policy_registration
  --test ggplot_positional_discrete_label_functions --test ggplot_positional_binned_label_functions
  --test ggplot_label_functions --test ggplot_binned_label_functions --test ggplot_position_bins
  --locked --target-dir target/ggplot-authored-rust` passes **13** tests;
  `/tmp/ggplot-policy-label-routes-regression.log`. Offline Linux arm64 Rust 1.97.1
  passes the first three targets; `target/ggplot-temporal-precision/linux-target/policy-label-routes-linux.log`.
- Strict core/example-extension all-target Clippy, strict core rustdoc, formatting,
  repository/syntax/diff checks pass; `/tmp/ggplot-policy-label-routes-{clippy,doc,fmt,repository}.log`.
- Python and WASM are rebuilt with the extension proof; their expanded original/edited
  corpora pass **4,700 discrete** and **4,944 binned** states per host. Both assert v32
  on registered policy routes and exact authored round trips. Within each host,
  axis/policy state records match exactly after removing the route label.
  Discrete cross-host states match exactly; binned values/positions retain the earlier
  independent 3e-12 tolerance, with the same small logarithmic round-trip differences
  repeated over the second route. All other fields remain exact.
  `target/ggplot-policy-label-routes/comparison.json` records this comparison.
- All **30** policy-route SVG/PDF/PNG files match across hosts and match the prior
  inspected axis-route bytes. This reuses the identical discrete/binned inspection
  artifacts, including the documented Preserve-policy overlap; no new image or
  collision-avoidance claim is inferred.

Python logs/artifacts: `target/ggplot-temporal-precision/linux-target/policy-label-routes-python-build.log`
and `policy-{discrete,binned}-label-python{.log,/}`. WASM module/artifacts:
`target/ggplot-policy-label-routes/{wasm-module,discrete-wasm,binned-wasm}/`;
logs `/tmp/ggplot-policy-label-routes-wasm-build.log` and
`/tmp/ggplot-policy-{discrete,binned}-label-wasm.log`.

The aggregate runner assertions are updated but the full aggregate was not rerun.
This qualifies primary authored policy routing, not a new standalone training API or
additional GPUI capture. Other non-color labels, break/minor-break and palette
functions and remaining constructor contracts remain open. GG-04 stays IN PROGRESS.

## Binned positional labels and reset ranges — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The 1,600-case `positional-binned-label-functions.json` captures actual ggplot2 4.0.3
panel breaks, labels, ranges and callback vectors. Four transformations, five
populations, automatic/full limits, both show-limits settings, four break controls and
five label modes produce 872 successful layouts and 728 expected rejections. Of these,
1,280 cases use registered callbacks; 320 retain default formatting. Rust independently
checks callback input order/length/names, omission/error boundaries, finite panel ranges,
normalized tick positions and exact visible labels. Nonfinite-panel geometry is not
certified by successful label construction.

The existing bin owner now keeps infinite classification sentinels out of finite
post-reset panel training, and shown limits are regenerated after reset. Source-empty
metadata uses the existing prepared policy flag. Untrained primary views select fresh
cuts from their expanded viewport using the shared bin-break resolver; their source
classification remains empty. This also fixes default empty-axis labels. Both default
and registered guides censor against the panel viewport. The shared equal-break
sequence now calculates its step before multiplying, matching R `seq`; the previous
arithmetic order placed a reverse show-limits candidate on the wrong side of a strict
boundary. `/tmp/ggplot-bin-sequence.log` records the pinned R method and hexadecimal
sequence evidence; no fixture tolerance or expected value was changed to hide it.

Evidence:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-binned-label-functions.R`
  captures 1,600 cases; `/tmp/ggplot-positional-binned-label-reference.log`.
- `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`
  passes all **558** macOS tests including doctests;
  `/tmp/ggplot-positional-binned-label-all-core.log`. An earlier concurrent incremental
  rebuild invalidated a doctest library path; this final sequential run passes.
- Offline Linux arm64 Rust 1.97.1: `cargo test -p chart-core --test
  ggplot_positional_binned_label_functions --test ggplot_position_bins --test
  ggplot_binned_label_functions --locked --offline -j2 --target-dir /target` passes
  **9** tests; `target/ggplot-temporal-precision/linux-target/positional-binned-label-linux.log`.
- Strict core/example-extension all-target Clippy, strict core rustdoc, formatting,
  repository/syntax and diff checks pass;
  `/tmp/ggplot-positional-binned-label-{clippy,doc,fmt,repository}.log`.
- Rebuilt Linux Python (`extension-module,extension-proof`) and WASM (`extension-proof`,
  wasm-bindgen 0.2.128) execute `ggplot_positional_binned_label_functions.{py,cjs}`:
  **2,472 states**, comprising 1,744 successful original/edited layouts and 728 expected
  rejections. Both independently check source labels and finite normalized tick
  positions (absolute tolerance 3e-12). Definitions round-trip through actual hosts.
  Logs: `target/ggplot-temporal-precision/linux-target/positional-binned-label-{python-build.log,python.log}`
  and `/tmp/ggplot-positional-binned-label-{wasm-build,wasm}.log`.
- `target/ggplot-positional-binned-labels/comparison.json` records exact labels/errors
  and scoped numeric agreement. Logarithmic inverse/forward evaluation differs in
  **108 tick-value fields** (maximum 1.78e-15) and **12 position fields** (maximum
  5.69e-14 pixels). Raw tick values use the independent Rust/source 3e-12 relative or
  absolute tolerance; positions use 3e-12 after normalization by the 440-pixel range.
  All other fields match exactly. `raw-differences.json` retains the differences.
- All **18** selected SVG/PDF/PNG files match byte-for-byte. Three inspection sheets
  show indexed/missing/reverse labels, empty default formatting, the correctly censored
  reverse equal boundary, and explicit cuts with shown limits. Sample 30 has expected
  overlap under Preserve; no collision-avoidance claim. No clipping observed.

The aggregate primary-authoring runner includes this corpus and the field-specific
comparison but was not rerun. No new update-layout or GPUI-capture claim is made.
Registered labels authored directly inside positional scale policies still require
registry/constructor reconciliation; this qualifies the axis formatter route and the
listed default-label cases. Other non-color labels, break/minor-break and palette
functions and remaining constructor contracts stay open. GG-04 remains IN PROGRESS.

## Discrete positional label functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The 640-case `positional-discrete-label-functions.json` captures ggplot2 4.0.3 panel
breaks/labels and actual callback vectors/names: five populations, automatic/explicit
limits, factor-level dropping, missing translation, automatic/explicit/empty/named
breaks and indexed/missing/short/empty callback results. Band and point primary axes
each pass 535 successful layouts and 105 expected rejections. Rust checks every
callback input and name vector, callback omission for untrained domains, and recycled
visible labels on finite panel ranges. Nonfinite-panel geometry remains unqualified.

`layout/guide_discrete.rs` adapts the retained provider domain to the existing
`GgplotDiscreteGuide` resolver. Break intersection, deduplication, names and recycling
have one owner. The resolver now accepts caller bounds and an explicit trained-state
context internally: an all-missing untranslated domain invokes the empty-vector
callback, while a truly untrained domain skips it. Positional missing labels retain
ticks with empty text. Portable callback validation runs before evaluation.

Evidence:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-discrete-label-functions.R`
  captures 640 reference cases; `/tmp/ggplot-positional-discrete-label-reference.log`.
- Core targets `ggplot_positional_discrete_label_functions`, `ggplot_discrete_position`,
  `ggplot_discrete_limits`, `ggplot_discrete_secondary`, `ggplot_label_functions`,
  `ggplot_positional_label_functions`, `ggplot_positional_temporal_label_functions`
  pass 17 tests on macOS and offline Linux arm64 Rust 1.97.1. Commands use
  `cargo test -p chart-core --test NAME ... --locked`, macOS target
  `target/ggplot-authored-rust`, Linux `--offline -j2 --target-dir /target`.
  Logs: `/tmp/ggplot-positional-discrete-label-regression.log` and
  `target/ggplot-temporal-precision/linux-target/positional-discrete-label-linux.log`.
  The expanded named-break corpus also passes separately on macOS;
  `/tmp/ggplot-positional-discrete-label-focused.log`.
- Strict core/example-extension Clippy, strict core rustdoc, formatting and repository
  checks pass; `/tmp/ggplot-positional-discrete-label-{clippy,doc,fmt,repository}.log`.
- Rebuilt Python (`extension-module,extension-proof`) and WASM (`extension-proof`,
  wasm-bindgen 0.2.128) run the new `ggplot_positional_discrete_label_functions.{py,cjs}`
  proofs: **2,350 identical states**, comprising 2,140 successful original/edited
  layouts and 210 expected rejections. Definitions round-trip through the real host
  serializers. Explicit missing tick keys use the typed `MissingCategory` object,
  preserving the ordinary string category of the same spelling.
  Python logs/artifacts: `target/ggplot-temporal-precision/linux-target/positional-discrete-label-{python-build.log,python.log,python/}`;
  WASM logs: `/tmp/ggplot-positional-discrete-label-{wasm-build,wasm}.log`.
- `target/ggplot-positional-discrete-labels/comparison.json` records exact equality and
  12 byte-identical publications. The two inspection sheets show complete indexed
  labels, an omitted middle label with retained tick, recycled single-result labels,
  and an explicit missing-label case. SVG/PDF/PNG were inspected; no clipping observed.

The aggregate runner includes this corpus but was not rerun. No new update-layout
claim or GPUI capture is made. Direct `GgplotDiscretePosition.guide.labels` registered
policies still need constructor/registry reconciliation; this slice qualifies the
primary-axis `GuideFormatter::Registered` route. Binned and other non-color labels,
break/minor-break and palette functions remain open. GG-04 remains IN PROGRESS.

## Temporal positional label functions — 12 September 2026

Revision: `6e74ae6` plus preserved working-tree changes. Requirements: GG2-03/FIX-GG04.
The 400-case `positional-temporal-label-functions.json` oracle captures actual ggplot2
4.0.3 panel breaks/labels, callback vectors, Date/POSIXct classes, zone and automatic
names. It covers five Date/UTC/New York gap/fold contexts, five populations, four
break/format controls and four callback return modes. The Rust test executes each
case at second/millisecond/microsecond/nanosecond resolution: 740 successful layouts
and 860 expected rejections. Every observed callback matches the source vector and
metadata; finite panel ranges also match visible labels. Nonfinite-panel geometry is
not certified by label success alone.

Calendar axes accept an empty authored domain to infer endpoints from the positioned
population. Validation and resolution reuse the existing time engine; explicit
knots and standalone empty-domain rejection retain their behavior. ADR 018 records
this additive constructor meaning. The empty population and all-missing population
remain distinct; automatic all-missing time breaks fail before a label callback.
Explicit date formatting takes precedence over registered labels.

Evidence:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-temporal-label-functions.R`
  captures the pinned 400-case fixture; `/tmp/ggplot-positional-temporal-label-reference.log`.
- `mise exec -- cargo test -p chart-core --locked --target-dir target/ggplot-authored-rust`
  passes all 556 macOS tests including doctests; `/tmp/ggplot-positional-temporal-all-core.log`.
- Offline Linux arm64 `rust:1.97.1-bookworm`, `cargo test -p chart-core --test
  ggplot_positional_temporal_label_functions --test ggplot_positional_label_functions
  --locked --offline -j2 --target-dir /target` passes both corpus tests;
  `target/ggplot-temporal-precision/linux-target/positional-temporal-label-linux.log`.
- Core/example-extension strict Clippy (all targets), core rustdoc, formatting and
  repository checks pass; `/tmp/ggplot-positional-temporal-label-{clippy,doc,fmt,repository}.log`.
- Linux Python rebuilt with `extension-module,extension-proof`; WASM rebuilt with
  `extension-proof` and wasm-bindgen 0.2.128. The new
  `scripts/bindings/ggplot_positional_temporal_label_functions.{py,cjs}` execute **2,365
  identical states**: 1,480 successful original/edited layouts, 860 expected errors,
  and 25 replacement/batch domain checks. Replacement compares exact absolute
  timestamps after reconciling retained versus freshly selected integer origins;
  held publication scenes and authored definitions remain unchanged. These checks
  do not claim updated-frame layout equivalence.
  Python logs/artifacts: `target/ggplot-temporal-precision/linux-target/positional-temporal-label-{python-build.log,python.log,python/}`;
  WASM logs: `/tmp/ggplot-positional-temporal-label-{wasm-build,wasm}.log`.
- `target/ggplot-positional-temporal-labels/comparison.json` confirms 2,365 exact records
  and 12 byte-identical SVG/PDF/PNG publications. Inspection sheets and observations
  are in its `inspection/` directory. Inspected Date missing labels retain indices
  3/7 and 5/7; UTC/NY gap indices run 2/6 through 5/6; the NY fold visibly repeats
  01:00 at two distinct positions. No clipping observed in the selected publications.

The corpus is wired into the aggregate primary-authoring runner; that aggregate and
fresh GPUI captures were not run. Discrete/binned positional callbacks, other non-color
labels, break/minor-break and palette functions and remaining constructor contracts
remain open. GG-04 and cumulative parity gates stay IN PROGRESS.

## Numeric positional label vectors — 12 September 2026

Revision `6e74ae6` plus working-tree changes. The existing axis guide selector now
retains automatic numeric candidates for registered ggplot2 labels. The shared axis
path censors transformed out-of-panel candidates to missing source values before
calling the existing formatter registry; duplicate occurrences and missing labels
retain their indices. Empty populations invoke the callback with an empty vector,
while an all-missing unbounded scale retains its reference explicit candidates.
Other guide profiles preserve their existing selection/formatting contract.

`tools/reference/r/positional-label-functions.R` captures 480 pinned panel label
calls: four transforms (identity/sqrt/log10/reverse), five populations, absent/full
limits, automatic/explicit/empty breaks and four label functions. The core target
checks every callback input (3e-12 numeric tolerance), exact names, visible labels,
and all 188 success / 292 rejection boundaries. Repeated internal invocations may
differ in count but must have the same reference inputs. This slice covers numeric
primary axes with default expansion; temporal/discrete/binned axes and additional
callback/format precedence remain open.

Validation:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-label-functions.R`
  passes 480 cases under R 4.6.1 / ggplot2 4.0.3;
  `/tmp/ggplot-positional-label-reference.log`.
- `mise exec -- cargo test -p chart-core --test axis_ticks --test ggplot_reverse --test ggplot_positional_label_functions --locked --target-dir target/ggplot-authored-rust`
  passes 16 tests; `/tmp/ggplot-positional-label-regression.log`. The same three
  targets pass in offline Linux Rust 1.97.1;
  `target/ggplot-temporal-precision/linux-target/positional-label-linux.log`.
- Clippy for all targets of core and the extension example, strict core rustdoc,
  format/repository/diff and Python/Node syntax checks pass;
  `/tmp/ggplot-positional-label-{clippy,doc,fmt,repository}.log`.
- Rebuilt Python and WASM execute `scripts/bindings/ggplot_positional_label_functions.{py,cjs}`:
  376 original/edited successful layouts, 292 expected `CHART_SCHEMA_CONFLICT`
  rejections and 40 replacement states. Replacements compare current limits/domains
  with fresh builds and retain immutable held scenes/authored JSON; this does not
  certify freshly laid-out updated scenes. Dataset identities/revisions naturally
  differ across new builds and are not semantic comparison fields.
  Python logs/artifacts: `target/ggplot-temporal-precision/linux-target/positional-label-{python-build.log,python.log,python/}`;
  WASM logs: `/tmp/ggplot-positional-label-{wasm-build,wasm}.log`;
  module/output: `target/ggplot-positional-labels/{wasm-module,wasm}/`.
- All 708 records agree exactly across hosts; 15 SVG/PDF/PNG outputs are byte-identical.
  `target/ggplot-positional-labels/comparison.json` records the comparison.
  All 15 publications were inspected in `inspection/review-{1,2,3}.png`: censored
  first candidates correctly leave labels starting at 2/5, reverse labels run in
  reversed spatial order, and an explicit duplicate/missing-label example retains
  3/8 and 5/8 with no visible clipping. This is scoped label/export validation,
  not complete ggplot2 appearance or guide-composition parity.

The aggregate primary proof runner includes the new corpus but was not rerun as a
whole. No fresh full-workspace gate was run for this slice. GG-04 and G-GGPLOT remain
open; continue remaining positional/non-color labels and callback argument contracts.

## Binned color-scale label vectors — 12 September 2026

Revision `6e74ae6` plus working-tree changes. The existing registered guide-label
path now supports binned cut vectors. Primary color-guide preparation follows the
pinned `parse_binned_breaks`/`GuideColoursteps` contract for function labels: remove
missing transformed cuts, censor finite outsiders before inverse-source callback
inputs, retain duplicate occurrences, and select visible labels by default equal-step
key positions. The standalone scale candidate path retains raw cuts. Both paths use
the same registry and cardinality/text-budget checks as continuous callbacks.
The cut selection remains in the binned scale owner; GG-05 owns guide presentation.

`tools/reference/r/binned-label-functions.R` and matching JSON capture **640**
reference builds and their direct scale calls independently. Primary cases span
identity/sqrt/log10/reverse transforms; ordinary/constant/partly missing/all-missing/
empty populations; absent/full limits; nice/equal/explicit/empty cuts; indexed/
missing/short/empty labels. The core target compares every callback value and name
with the build calls and compares visible `numeric_breaks` labels against guide keys:
**378 successes, 262 expected rejections**. Float source values use 3e-12 relative/
unit-floor tolerance; metadata and labels are exact. The same target also compares all **640 direct scale cases** independently: 260
successes and 380 expected rejections, exact callback count/names, raw values within
3e-12 and exact label results. Direct empty vectors invoke the callback; empty primary
color guides skip it. Core owns this distinction through the common consumer-specific
callback policy; there is no added host wrapper.

Validation:

- `mise exec -- cargo test -p chart-core --test ggplot_binned_guides
  --test ggplot_binned_label_functions --test ggplot_label_functions
  --test ggplot_temporal_label_functions --locked --target-dir target/ggplot-authored-rust`:
  **14 PASS**; `/tmp/ggplot-binned-label-focused-final.log`.
- Same four targets in offline `rust:1.97.1-bookworm`: **14 PASS**;
  `target/ggplot-temporal-precision/linux-target/binned-label-linux-focused-final.log`.
- Rebuilt Python (`extension-module,extension-proof`) and WASM (`extension-proof`,
  wasm-bindgen 0.2.128) each pass **1,042** states with
  `scripts/bindings/ggplot_binned_label_functions.py`/`.cjs`: 756 original/edited
  successes, 262 expected rejection cases, 24 retained-data replacements compared
  with fresh builds. Held publication scene and authored wire remain unchanged.
  Portable primary definitions round-trip at wire 32.
- Cross-host comparison: **28** inverse-transform numeric fields differ within
  the existing 3e-12 tolerance (for example 5.500000000000001 versus 5.5); all other
  fields, including labels, visibility, styles and errors, are exact. These are
  recorded individually in `target/ggplot-binned-labels/comparison.json`. The
  aggregate primary runner uses the same field-specific comparison; no fixture or
  tolerance was changed. The aggregate runner itself was not rerun.
- **12 byte-identical** publications under `target/ggplot-binned-labels/wasm` and
  `target/ggplot-temporal-precision/linux-target/binned-label-python`; all inspected
  in `target/ggplot-binned-labels/inspection/review-{1,2}.png`. The inspection report
  explicitly records the current interval swatches, raw endpoint formatting
  (sqrt sample prints 12.500000000000002), and absent color-step callback text in
  this renderer. Callback labels are verified in semantic `numeric_breaks`;
  **these publications do not prove color-step visual or label-display parity**.
- All-target core/example Clippy, strict rustdoc, format/repository/diff and host
  script syntax checks pass; `/tmp/ggplot-binned-label-{clippy-final,doc-final,fmt-final,repository-final}.log`.

Full workspace tests are earlier evidence, not a fresh run for this slice. Alternate
color-step guide options, direct-scale host exposure, non-color/positional
compositions and callback combinations with authored limit functions remain open.
GG-04 remains IN PROGRESS; color-step layout and label display remain GG-05 work.

## Temporal color-scale label vectors — 12 September 2026

Revision `6e74ae6` plus working-tree changes. `GuideTemporalContext` adds exact
origin/unit and Date versus datetime interpretation plus explicit immutable calendar
rules to the existing guide formatter input. Temporal values remain origin-relative
numbers, preserving missing/fractional candidates without forcing an integer stamp.
The per-value adapter also retains this context. Temporal guide selection supplies
reference-generated break names to the registered vector function before visibility
filtering; an explicit date format takes precedence. Existing temporal selection,
formatting and registry owners are reused, with no new wire version beyond 32.

The pinned `tools/reference/r/temporal-label-functions.R` and matching JSON contain
400 color-scale builds: Date, UTC datetime and New York spring/fall transitions;
spaced, constant, partly missing, all-missing and empty populations; automatic,
explicit and empty candidates or overriding date format; indexed, missing, short
and empty callback results. The new core target compares input values, Date/POSIXct
class interpretation, timezone and names exactly against each recorded callback,
and primary guide labels/rejections in seconds/milliseconds/microseconds/nanoseconds.
It passes 1,180 successful cases and 420 expected rejections. This does not compare
ggplot's normalized colorbar key coordinates to raw timestamps.

Validation:

- `mise exec -- cargo test -p chart-core --test ggplot_temporal_label_functions
  --test ggplot_temporal_guides --test ggplot_label_functions --test axis_ticks
  --locked --target-dir target/ggplot-authored-rust`: **13 PASS**, including all
  1,600 new fixture/unit combinations; `/tmp/ggplot-temporal-label-focused.log`.
- Same four targets in offline `rust:1.97.1-bookworm`: **13 PASS**;
  `target/ggplot-temporal-precision/linux-target/temporal-label-linux-focused.log`.
- Actual Python extension rebuilt with `extension-module,extension-proof`; actual
  WASM rebuilt with `extension-proof` and wasm-bindgen 0.2.128. The new
  `scripts/bindings/ggplot_temporal_label_functions.py`/`.cjs` each pass **2,800**
  records: 2,360 original/edited successes, 420 expected rejections, 20 retained-data
  replacements compared with fresh builds. Prior authored wire and held publication
  scene stay unchanged. Wire 32 round-trips in both hosts.
- Exact cross-host records and **12 byte-identical** SVG/PDF/PNG publications:
  `target/ggplot-temporal-labels/comparison.json`. Python artifacts live under
  `target/ggplot-temporal-precision/linux-target/temporal-label-python`; WASM under
  `target/ggplot-temporal-labels/wasm`. All 12 inspected in
  `target/ggplot-temporal-labels/inspection/review-{1,2}.png`, with observations in
  `inspection.json`: missing Date labels, ordered callback indices and repeated
  New York fold hours remain legible without clipping across all three formats.
- All-target core/example Clippy and strict rustdoc pass; logs
  `/tmp/ggplot-temporal-label-{clippy,doc}.log`. Rustfmt, repository checker,
  Python/Node syntax and diff checks pass. The mistakenly requested nonexistent
  `mise run check-repository` task was replaced with the actual
  `python3 scripts/check_repository.py` command; final log
  `/tmp/ggplot-temporal-label-repository-final.log`.

The aggregate primary runner includes this corpus but was not rerun. Full workspace
regressions are the previous slice's evidence, not fresh evidence for this change.
The current square-key publication is not colorbar visual equivalence (GG-05).
Binned/non-color/positional callback compositions, break/minor-break functions,
palette callbacks and all constructor arguments remain open. GG-04 stays IN PROGRESS.

## Registered color-scale label vectors — 12 September 2026

Revision `6e74ae6` plus working-tree changes. `GgplotGuideLabels::Registered` uses
the existing versioned guide registry. `CustomGuideFormatter::format_labels` receives
the complete selected semantic vector and optional discrete break names; its default
adapter preserves existing per-value implementations and their decreasing text
budget. Prepared mappings retain the guide registry snapshot and authored operation,
not materialized labels. Minimum primary/portable wire version is 32 when a mapped
scale retains this callback. Missing/native-only registrations and stale versions
reject before executing callback code; explicit installation remains mandatory.

The pinned `guide-label-functions.R`/`.json` captures 180 continuous/discrete cases:
identity/sqrt/log10/reverse, finite/constant limits, automatic/explicit/empty breaks,
ordinary/nullable/empty discrete populations, named/deduplicated break selection and
indexed/nullable/short/empty/named callback results. It records both direct scale
calls and built guide keys. The source-space vectors and input names match with
`3e-12` numeric tolerance. Primary Rust matches all 146 successful builds and 34
rejections. Continuous labels see outside/nonfinite candidates before visibility
filtering; discrete inputs are intersected/deduplicated and retain break names.
Discrete guide-key assignment recycles a compatible shorter result. Output names
from a callback do not become discrete named replacements. Empty guide builds skip
callbacks, distinct from direct scale-method invocation with an empty vector.

Executed in the repository, Darwin arm64, Rust 1.97.1:

- R 4.6.1/ggplot2 4.0.3 capture: `/tmp/ggplot-guide-label-functions-reference.log`.
- `cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-authored-rust`
  via mise: all 606 then-current tests/doctests pass (`/tmp/ggplot-scale-label-full.log`).
  The final two-test label target also passes, including the added registry rejection
  test (`/tmp/ggplot-scale-label-registry.log`). All eight axis-tick tests cover the
  vector formatter through layout, including missing labels and output limits.
- All-target core/extension Clippy with `-D warnings`, strict rustdoc, formatting,
  repository and diff checks pass. Logs `/tmp/ggplot-scale-label-{clippy-final,doc,fmt,repository}.log`.
- Fresh actual Python/WASM run `scripts/bindings/ggplot_label_functions.{py,cjs}`:
  each passes 338 exactly matching states (326 original/edit/rejection states plus
  12 replacements compared with fresh batch semantics). Prior frame scenes and
  authored wire remain immutable; replacement frame layout is not newly compared.
  Fifteen SVG/PDF/PNG files match byte-for-byte, and all fifteen are visually inspected.
  No clipping or alignment problem is observed in these samples; nullable/recycled
  labels and pre-visibility callback indices appear consistently across formats.
- Python uses offline Linux arm64, with logs and outputs under
  `target/ggplot-temporal-precision/linux-target/scale-label-*`. WASM output, hashes,
  and three inspected contact sheets are under `target/ggplot-scale-labels/`.
  `comparison.json` and `inspection/inspection.json` record the comparisons/review.
- The offline Linux full Rust suite passes all 606 then-current tests/doctests.
  The final two-test registry target also passes on Linux
  (`scale-label-linux-registry.log`), including the added rejection control.
  The primary-authoring runner includes this proof; its cumulative execution is not
  claimed by these focused runs.

This qualifies the tested continuous/discrete color-scale callback path and shared
vector contract. Temporal/binned callbacks, non-color guide consumers, positional
scale-label policies, further population/composition cases and constructor precedence
remain unqualified. Current categorical key presentation does not certify GG-05
colorbars or complete guide composition. GG-04 and G-GGPLOT remain open.

## Calendar callbacks with supplied timezone rules — 12 September 2026

Revision `6e74ae6` plus working-tree changes. Calendar positional axes now participate
in the shared temporal callback collector, requiring the scale and source units to
agree. Callback scale resolution preserves the authored `CalendarZone` instead of
constructing a UTC calendar. Exact source origins, numeric mapping and calendar
selection remain in their existing owners. No new public/wire/host policy is added.

The first primary probe reproduced an UnsupportedCapability rejection on a valid
Calendar callback. `positional-calendar-functions.R`/`.json` now supplies 216 pinned
builds: UTC and America/New_York, the 2024 spring gap and fall fold, points/mean
summaries, spaced/constant/missing populations, identity/reversed/fixed callbacks and
three OOB policies. Primary Rust passes these across milliseconds/microseconds/
nanoseconds: 588 successful builds and 60 reference rejections, with the existing
`3e-12` position tolerance and exact labels. Separate controls reject mismatched
source/scale units and insufficient supplied timezone coverage. The explicit resource
contains the two 2024 US transitions and bounded coverage; no system timezone lookup
is introduced or historical timezone equivalence claimed.

Executed in the repository, Darwin arm64, Rust 1.97.1:

- R 4.6.1/ggplot2 4.0.3 capture: `/tmp/ggplot-calendar-time-reference.log`, 216 builds.
- `mise exec -- cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-authored-rust`:
  603 tests/doctests pass. Log `/tmp/ggplot-calendar-time-full.log`.
- All-target core/extension Clippy (`-D warnings`), strict core rustdoc, repository,
  format/diff checks pass. Logs `/tmp/ggplot-calendar-time-{clippy,doc,repository,fmt}.log`.
- Rebuilt actual Python and WASM each pass 444 exactly matching states through
  `scripts/bindings/ggplot_position_temporal.{py,cjs}` with final `calendar` argument:
  412 original/edited/rejection states plus 32 replacements compared with fresh retained
  semantics and immutable prior captures. Replacement frame layout is not newly
  compared. Authored timezone descriptors survive wire-29 round-trips.
  Python runs in offline Linux arm64; logs and output under
  `target/ggplot-temporal-precision/linux-target/calendar-time-python{,-build}.log`
  and `calendar-time-python/`. WASM uses wasm-bindgen 0.2.128; logs
  `/tmp/ggplot-calendar-time-wasm{,-build}.log`.
- Twelve SVG/PDF/PNG files match byte-for-byte. All were inspected with independent
  SVG/PDF rasterization: spring labels skip 02:00 and fall labels show two 01:00
  instants; UTC controls retain elapsed-hour labels. Reports/modules/publications and
  two inspection sheets are under `target/ggplot-calendar-time/`. No clipping was
  observed in these four samples; this does not close general guide presentation.

The cumulative primary runner includes this proof but was not run as a whole.
Offline Linux Rust regression also passes all 603 tests/doctests; log
`target/ggplot-temporal-precision/linux-target/calendar-time-linux-full.log`. Calendar callback combinations with
facets, arbitrary secondary transforms and other statistics are not newly qualified.
GG-04 remains IN PROGRESS. Next: the coverage reconciliation's unresolved callback
and constructor contracts, then GG-05–19.

## Automatically inferred timestamp callbacks — 12 September 2026

At `6e74ae6` plus working-tree changes, automatic axes infer timestamp origin/unit
metadata before evaluating positional limit callbacks. The shared source collector
retains that metadata even for empty input and rejects mixed numeric/timestamp
populations in either layer order. Numeric automatic axes retain numeric callbacks;
authored numeric-limit vectors still require numeric sources. No public fields,
serialized versions, host arithmetic or additional scale engines are introduced.

The first focused test reproduced `SchemaConflict` for an ordinary timestamp point
population on an automatic axis. After the correction, the existing pinned datetime
fixtures pass through the automatic form: 252 ordinary cases across three units
(363 successes/393 rejections), 36 fractional-summary cases across three units (108
successes), and 420 fixed/free facet cases across three units (687 successes/573
rejections). The units are milliseconds, microseconds and nanoseconds. Mixed-source
rejection and ordinary numeric automatic controls pass in both layer orders.
The pinned R runtime confirms POSIXct `scale_type` selects datetime/continuous and
implicit axes match explicit datetime mapping/ranges in a direct control;
`/tmp/ggplot-auto-time-source.log` records it. No fixture or tolerance was weakened.

Executed in the repository, Darwin arm64, Rust 1.97.1:

- `mise exec -- cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-authored-rust`:
  601 tests/doctests pass. Final log `/tmp/ggplot-auto-time-full-final.log`.
  The initial full run caught a test expecting mixed-source failure after build;
  the API correctly rejects during build. The final test accepts that earlier stage.
- All-target core/extension Clippy (`-D warnings`), strict core rustdoc,
  format/repository/diff checks pass. Logs `/tmp/ggplot-auto-time-{clippy-final,doc,fmt,repository}.log`.
- Rebuilt Python and WASM run `ggplot_position_temporal.{py,cjs}` and
  `ggplot_position_facets.{py,cjs}` with final `automatic` argument. They pass 457
  temporal and 665 facet states each, exactly matching across hosts. This includes
  12 temporal and 16 facet source replacements against fresh retained semantics,
  original/edited definitions, wire-29 round-trips and immutable captures. Replacement
  frame layout is not newly compared by these loops.
- All 30 SVG/PDF/PNG publications are byte-identical between hosts and to the earlier
  inspected explicit-datetime publications. No new visual difference is introduced.
  Report/modules/WASM outputs: `target/ggplot-auto-time/`; Python outputs and logs:
  `target/ggplot-temporal-precision/linux-target/auto-time-{python,facet-python}/`
  and `auto-time-{python,facet-python,python-build}.log`. WASM logs:
  `/tmp/ggplot-auto-time-{wasm,facet-wasm,wasm-build}.log`.
  Python uses offline Linux arm64 and WASM uses wasm-bindgen 0.2.128.

The cumulative primary runner includes both automatic proofs but was not run as a
whole. Offline Linux Rust regression also passes all 601 tests/doctests; log
`target/ggplot-temporal-precision/linux-target/auto-time-linux-full.log`. Calendar axes with explicit
local-zone resources remain a separate open callback path. GG-04 stays IN PROGRESS;
next are that path and the coverage index's callback/constructor contracts, followed
by GG-05–19.

## Authored limits through statistics and facets — 12 September 2026

Revision `6e74ae6` plus working-tree changes. The new
`positional-facet-authored-limits.R`/`.json` captures 720 pinned builds: points and
mean summaries, identity/sqrt/reverse transforms, fixed/free axes, balanced/constant/
missing/retained-empty panels, automatic/complete/partial/reversed limits and three
OOB policies. The shared primary test passes 714 successful builds and six reference
rejections with `3e-12` numeric tolerance and exact labels.

The first run reproduced a failure for an all-missing free square-root panel:
round-tripping its infinite transformed viewport through the inverse and forward
transforms yielded an invalid guide domain. Authored numeric guides now pass their
already transformed bounds to the existing continuous guide selector. No palette,
statistic, transform or break-selection kernel is duplicated; no wire version changes.
The earlier callback path is unchanged.

Executed on Darwin arm64, Rust 1.97.1, in the repository workspace:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-facet-authored-limits.R`:
  720 builds, R 4.6.1/ggplot2 4.0.3. Log `/tmp/ggplot-facet-authored-reference.log`.
- `mise exec -- cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-authored-rust`:
  598 tests/doctests pass. Log `/tmp/ggplot-facet-authored-full.log`.
- All-target core/extension Clippy (`-D warnings`), strict core rustdoc, formatting,
  repository and diff checks pass. Logs `/tmp/ggplot-facet-authored-{clippy,doc-strict,fmt,repository}.log`.
- Rebuilt actual Python and WASM run `ggplot_position_facets.{py,cjs}` with final
  `authored` argument. Each passes 1,482 exactly matching states: 1,434 original/edit/
  rejection states and 48 live source replacements compared with fresh retained panel
  semantics, preserving captured frames and authored wire. Replacement layout itself
  is not newly compared by this loop. Both hosts require wire 31 for the authored field.
  Python runs in offline Linux arm64 (`rust:1.97.1-bookworm`), build/test logs under
  `target/ggplot-temporal-precision/linux-target/facet-authored-python{,-build}.log`.
  WASM uses wasm-bindgen 0.2.128; logs `/tmp/ggplot-facet-authored-wasm{,-build}.log`.
- All 36 publication files match byte-for-byte. All were inspected in six contact
  sheets using native PNG and independent SVG/PDF rendering. Reports and WASM files:
  `target/ggplot-facet-authored/`; Python files:
  `target/ggplot-temporal-precision/linux-target/facet-authored-python/`.
  Existing No data text and left-edge summary tick clipping remain presentation
  differences under GG-05/GG-12. This is not a reference visual-equivalence claim.

The cumulative authoring runner now includes this slice but was not run as a whole.
The offline Linux Rust regression run also passes all 598 tests/doctests; log
`target/ggplot-temporal-precision/linux-target/facet-authored-linux-full.log`. The new evidence qualifies
these point/mean facet combinations; other statistics and geometry families retain
their package-specific contracts. GG-04 remains IN PROGRESS. Next: timestamp inference
and the unresolved callback/constructor contracts in the coverage reconciliation.

## Authored numeric positional limits — qualified slice, 12 September 2026

At `6e74ae6` plus working-tree changes, `AxisBuilder::numeric_limits` supplies an
optional pair of nullable source endpoints. The shared authored-bounds owner fills
transformed missing endpoints from each pre/post-statistic population and sorts
complete transformed pairs; registered callbacks retain their distinct order/arity.
Empty post-statistic populations retain a separate guide-suppression marker, including
shared/free facet training scopes. Primary wire version 31 preserves this policy and
rejects downgrades; the legacy profile rejects the new ggplot policy. Core host dispatch
and Python/WASM declarations expose the same builder.

The pinned R `ScaleContinuous` implementation confirms that an untrained scale with
nonfinite authored endpoints uses `[0, 1]` and no guide breaks. The new
`positional-authored-limits.R`/`.json` corpus captures 648 builds covering partial,
reversed, constant and transformed-invalid limits over finite, constant, mixed,
nonfinite, missing and empty populations. It supplements the 1,728 exact-view corpus;
no earlier expected value or tolerance was weakened.

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64 and offline Linux
arm64 (`rust:1.97.1-bookworm`), Rust 1.97.1:

- The primary `unbounded_positions` test target passes all **1,728 exact-view cases**
  (1,368 successes, 360 rejections), the **648 new authored cases**, the previous
  **1,350 callback/binned cases**, and a separate wire/profile rejection test.
  Log: `/tmp/ggplot-authored-primary-final.log`; R log: `/tmp/ggplot-authored-oracle.log`.
- `mise exec -- cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-positional-wasm`:
  **597 tests/doctests pass**. Log: `/tmp/ggplot-authored-native-final-full.log`.
  The equivalent offline Linux full suite passes **596**, followed by the four final
  `unbounded_positions` tests including the subsequently added wire/profile test.
  Logs under `target/ggplot-temporal-precision/linux-target/`:
  `authored-linux-full.log`, `authored-linux-final-primary.log`.
- Final all-target core/extension Clippy (`-D warnings`), strict rustdoc, formatting,
  repository/diff checks pass; strict mypy and TypeScript declaration consumers pass.
  Logs: `/tmp/ggplot-authored-{clippy-final,rustdoc,fmt-final,repository-final,mypy,tsc}.log`.
- Rebuilt Python (`extension-module,extension-proof`) and WASM (`extension-proof`,
  wasm-bindgen 0.2.128) each pass **4,428 identical states** and two explicit
  downgrade/profile rejections. This includes 36 live source replacements against
  fresh batches, transitions through empty continuous populations, and retained
  frame/authored-wire checks. Scripts: `scripts/bindings/ggplot_unbounded_positions.{py,cjs}`
  with final `authored` argument. Python logs under the Linux target: `authored-python-final-build.log`
  and `authored-python-final.log`;
  WASM logs: `/tmp/ggplot-authored-wasm-final{,-build}.log`.
- All **54 SVG/PDF/PNG files match byte-for-byte**. The previous 36 inspected files
  are unchanged; all 18 new files were inspected in three contact sheets. Final builds
  reproduce every inspected hash. Artifacts/reports: `target/ggplot-authored/`,
  including `comparison.json` and `inspection/inspection.json`. Empty scales suppress
  ticks but retain the existing No data annotation; that remains a presentation
  difference, not a reference-rendering claim.

This slice qualifies ordinary points; the new authored field with other statistics
and free facets still needs direct reference qualification. The cumulative
primary-authoring runner includes these proofs but was not run as a whole. GG-04
remains IN PROGRESS. Next: reconcile all 152 owned exports and inherited arguments
against current evidence, including these cross-stage limits and remaining timestamp
controls, then continue GG-05–19. No cumulative gate closes.

## Unbounded coordinate views — qualified slice, 12 September 2026

Revision `6e74ae6` plus working-tree changes, cwd `/Users/jeickmeier/Projects/finstack-chart`,
Darwin arm64 and offline Linux arm64 (`rust:1.97.1-bookworm`), Rust 1.97.1.
The shared projection kernel passes 735 finite reference coordinate ranges and 643
infinite positions, plus independent reversed/constant viewport checks. The expanded
reference capture contains 1,728 R builds; a separate 1,728-build capture disables
coordinate expansion to match the exact `AxisBuilder::viewport` contract.

Core retains infinite ordinary-point coordinates in `UnboundedPoint` until projection
resolves both axes to finite scene coordinates. Population limits remain independent
from the coordinate viewport. Primary Rust passes 1,350 exact-view reference cases:
1,044 successes and 306 rejections, covering all 864 binned cases and 486 continuous
cases. Continuous cases use registered constant callbacks only where their semantics
agree with the authored reference vectors. The other 378 authored-vector cases remain
open; callback equivalence is not presumed for empty or transformed-missing domains.
Point and guide positions use the existing `3e-12` tolerance.

Earlier callback and missing-value fixtures captured `panel$rescale`, which omits R's
final Cartesian infinite-point squishing. Their generators now additionally capture
`coordinate_positions`; every prior captured fact was verified unchanged. Nine
callback-argument and 36 missing-value cases distinguish these stages. The corresponding
Rust and host checks now consume final coordinates and retain infinite prepared data.

Executed qualification:

- `mise exec -- cargo test -p chart-core -p chart-extension-example --locked`
  using `target/ggplot-positional-wasm`, and the corresponding offline Docker command
  with `--offline -j2 --target-dir /target`: **594 tests/doctests passed on each platform**.
  Logs: `/tmp/ggplot-unbounded-native-full.log`, `/tmp/ggplot-unbounded-linux-final-full.log`.
- All-target core/extension Clippy with `-D warnings`, strict rustdoc, formatting,
  repository and diff checks passed. Logs: `/tmp/ggplot-unbounded-{clippy,rustdoc,fmt,repository}.log`.
- Rebuilt Python extension (`extension-module,extension-proof`) and WASM
  (`extension-proof`, wasm-bindgen 0.2.128) each pass **2,426 identical states**, including
  32 source replacements compared with fresh batches and retained-frame/wire checks.
  Scripts: `scripts/bindings/ggplot_unbounded_positions.{py,cjs}`; logs:
  `/tmp/ggplot-unbounded-python-samples.log`, `/tmp/ggplot-unbounded-wasm.log`.
- Callback regressions pass **1,584 identical states and 48 identical publications**;
  all previous publication hashes are unchanged. Missing-value regressions pass
  **2,540 states within the existing absolute/relative `3e-12` tolerance**. The same
  132 numeric host differences remain; 28 of 30 publications match byte-for-byte,
  with the other two SVGs passing the unchanged numeric comparator.
- All **36 new byte-identical SVG/PDF/PNG files** were visually inspected, together
  with the three changed logarithmic missing-value files against their predecessors.
  Default panel ranges match clipping boundaries; infinite points render at finite
  panel edges. The three changed files contain the newly retained infinite point.
  Reports, hashes and seven inspection contact sheets are retained under
  `target/ggplot-unbounded/` (`comparison.json`, `missing-comparison.json`,
  `inspection/inspection.json`).

The cumulative primary-authoring runner includes these proofs but was not rerun as a
whole. Empty-bin NoData text remains a presentation difference owned by GG-05/GG-12.
Mapped glyphs and other infinite geometry remain GG-07 work; default coordinate-view
expansion remains GG-13 work. GG-04 stays IN PROGRESS. Next: authored numeric limit
vectors, remaining timestamp controls and full export/argument inventory reconciliation,
then the requested GG-05–19 continuation. No cumulative gate is advanced.

## Free-facet positional bins — 12 September 2026

`positional-facet-bin-functions.json` captures 1,152 actual reference builds with
identity/sqrt/reverse transforms, automatic or registered limits, fixed/free axes,
points/summaries, constant/missing/empty panels and all three OOB policies. Primary
Rust passes 626 successes and 526 reference rejections; the prior numeric/temporal
facet matrices also pass. The two new boundaries are an untrained retained free-bin
panel and a faceted summary with no usable observations in any panel. Standalone
zero-row bins retain their previously qualified fallback. Log:
`/tmp/ggplot-facet-bins-primary.log`. Final full core/extension validation passes
**591 tests/doctests**, with all-target Clippy, strict rustdoc, formatting, repository
and diff checks passing. Logs: `/tmp/ggplot-facet-bins-{full,clippy,rustdoc,fmt,repository}.log`;
revision, environment and locked target directory are unchanged from the facet slice below.

Rebuilt actual Python/WASM adapters each pass **5,504 identical states**, including
**144 source replacements** versus fresh batches and immutable captures. All **93
SVG/PDF/PNG publications are byte-identical and inspected**; the previous 57 hashes
are unchanged. Twelve new binned point/summary scenes cover both sharing policies and
identity/sqrt/reverse transforms, with legible, unclipped guides. Logs:
`/tmp/ggplot-facet-bins-{python,wasm-build,wasm}.log`; comparison and inspection reports
remain under `target/ggplot-position-facets`. The primary runner includes all 93 file
comparisons; the cumulative runner was not executed.

Next: the separately captured `positional-unbounded-viewports.json` corpus has 1,728
actual finite-coordinate-view builds (1,368 successes, 360 errors). It records final
coordinate-transformed points, including infinite source positions projected onto
finite panel edges. This is preparation evidence only; implementation remains pending.
Remaining timestamp controls and all 152 GG-04-owned export/argument mappings in the
643-export baseline inventory still require reconciliation. No cumulative gate is advanced.

## Faceted positional callbacks — 12 September 2026

The 1,008-build `positional-facet-functions.json` matrix passes all **597 successful
builds and 411 expected errors** through primary Rust. Identity/sqrt/reverse position
scales train source values and positioned statistics on the shared panel union or
separate free panel populations. The compiler retains positioned rows before final
mapping, avoiding per-panel reset of shared functions. Source filtering uses the
existing facet key/target/scope rules. Per-panel semantic snapshots retain optional
`positional_limits`; authored wire 29 remains unchanged.

The separate `positional-facet-temporal-functions.json` supplement captures 1,260
Date/datetime/duration builds: 717 successes and 543 reference rejections. Each passes
at millisecond, microsecond and nanosecond timestamp resolutions through the same
compiler, with no additional production change. Raw temporal summaries use the
previous two-absolute-epoch-ULP tolerance; projected points/ticks use `3e-12` and labels
match exactly. The primary target also checks source filtering and the explicit panel
catalog against independently expected `[1,3]` source limits.

At `6e74ae6` plus working-tree changes, final full core/extension validation passes
**590 tests/doctests**. All-target Clippy, strict rustdoc, formatting, repository and
diff checks pass. Logs: `/tmp/ggplot-facet-temporal-{primary,full,clippy,rustdoc,fmt,repository}.log`.
Commands use `--locked --target-dir target/ggplot-positional-wasm`; environment remains
Darwin arm64 / Rust 1.97.1, with actual Linux Python and pinned-bindgen Node/WASM adapters.

Both rebuilt hosts pass **3,678 identical records**, including **96 source replacements**
versus fresh batches and immutable captures. All **57 SVG/PDF/PNG files match byte-for-byte
and are inspected** across nineteen scenes; the previous 21 numeric publication hashes
are unchanged. Numeric shared/free, transformed constant, retained-empty, Date/datetime
and duration fractional summary scenes have legible, unclipped labels. The existing
`No data` placeholder in an empty panel remains a later facet presentation gap.
Logs: `/tmp/ggplot-facet-temporal-{python,wasm-build,wasm}.log`. Artifacts:
`target/ggplot-position-facets/{wasm,comparison.json,inspection}` and
`target/ggplot-temporal-precision/linux-target/position-facet-python`. The primary runner
includes the complete matrices and 57 file comparisons; the cumulative runner was not
executed. No cumulative gate is advanced. Free-facet bins, remaining positional scale
boundaries and inventory reconciliation remain open before GG-04 closure and GG-05–19.

## Fractional timestamp statistic projection — 12 September 2026

The separate 72-build thirds/sevenths supplement reproduced a Date summary position
of `0.5000000000121261` where ggplot2 reports `0.5`. The common time projection now
rounds a reference statistic in absolute Date/POSIXct coordinates, consistently with
the reference-expanded viewport, while retaining the exact source origin and precise
prepared observation. All 72 reference builds pass at three source resolutions in
`fractional_temporal_statistics_match_reference`; the prior temporal/duration matrix
and precision guard also pass (four primary tests). Log:
`/tmp/ggplot-fractional-temporal-primary.log`.

Qualification at `6e74ae6` plus working-tree changes on Darwin arm64 / Rust 1.97.1:
full core/extension validation passes **587 tests/doctests**, all-target Clippy and
strict rustdoc pass, and formatting/repository/diff checks pass. Commands retain
`--locked --target-dir target/ggplot-positional-wasm`; logs are
`/tmp/ggplot-fractional-temporal-{full,clippy,rustdoc,fmt,repository}.log`.
Rebuilt Linux Python and pinned-bindgen Node/WASM each pass **1,328 identical states**,
including 36 replacements versus fresh batches and immutable captures. All **48
SVG/PDF/PNG files match byte-for-byte**; the previous 36 hashes are unchanged. Both
new contact sheets were inspected: the fractional summaries have centered visible
points and no temporal ticks, as the reference specifies. Logs are
`/tmp/ggplot-fractional-temporal-{python,wasm-build,wasm}.log`; comparison and inspection
reports are under `target/ggplot-position-temporal`. The primary runner includes the
supplement; the cumulative runner was not executed.

The next implementation remains shared/free facet callbacks; the 1,008-build facet
corpus is captured only. No cumulative gate is advanced.

## Duration positional callbacks — 12 September 2026

The 252-build duration supplement in `positional-duration-functions.json`, captured by
`tools/reference/r/positional-duration-functions.R`, passes **150 successes and 102
reference rejections**. Duration callbacks preserve the reference's NULL input on an
empty population, which permits fixed results where Date/datetime rejects before
calling the function. Unbounded automatic guides now reject as the reference does;
the accepted singleton negative infinity uses hms's literal `-Inf:NaN:NaNNaN` through
the shared duration formatter. No host-specific scale implementation was added.

Full core/extension validation now passes **586 tests/doctests** across 115 targets,
`/tmp/ggplot-duration-callback-full.log`; all-target Clippy and strict rustdoc pass in
`/tmp/ggplot-duration-callback-{clippy,rustdoc}.log`. The three-test primary target
includes 1,512 Date/datetime unit comparisons, 252 duration builds and the independent
precision-loss guard. Rebuilt Python/WASM each pass **1,184 identical states**, including
36 source replacements against fresh batches and immutable captures. All **36
SVG/PDF/PNG publications are byte-identical and inspected**; previous 24 hashes are
unchanged. Logs: `/tmp/ggplot-duration-callback-{python,wasm}.log`. Artifacts and the
updated inspection/comparison reports use the same `target/ggplot-position-temporal`
and Python directories below. The primary acceptance runner includes the duration
supplement; the cumulative runner was not executed.

Next qualification probes are fractional timestamp statistics whose means do not
land on source quanta (72 separately captured R builds), followed by shared/free
faceted callbacks (1,008 captured builds, 597 successes and 411 errors). Their new
fixtures and capture scripts are preparation evidence only; no acceptance is claimed.
GG-04 remains IN PROGRESS, followed by inventory reconciliation and GG-05–19.

## Date/datetime positional callbacks — 12 September 2026

GG2-03/FIX-GG04 at `6e74ae6` plus the working tree: Date/UTC positional
limit functions now train before statistics and again after positions. The shared
numeric callback evaluator receives exact timestamp origins and units. Ordered
results retain singleton, fractional and missing endpoints; timestamp projections
preserve their representation. Temporal population collection rejects offsets beyond
exact f64 integer precision before training, including with censor/squish policies.
`TimeAxisScale::resolve_function` uses the existing numeric mapping engine with
origin-relative endpoints and reference absolute Date/POSIXct expansion. Its integer
domain encloses fractional endpoints; the mapping retains their precise offsets.
Automatic non-finite temporal guide rejection and the accepted `NA,-Inf` empty guide
match the pinned builds. Portable authored controls retain version 29; execution-local
origin metadata is not accepted from serialized definitions.

`tools/reference/r/positional-temporal-functions.R` captures **504** ggplot2 4.0.3 /
R 4.6.1 builds in `fixtures/parity/ggplot2/positional-temporal-functions.json`: Date and
UTC datetime, points/mean summaries, six populations, seven limit callbacks and three
OOB policies. The independent fixture is unchanged from capture. The primary Rust
test checks **242 successes and 262 rejections** at each of millisecond, microsecond
and nanosecond resolution (**1,512 comparisons**), including expanded ranges and
normalized point/tick positions at `3e-12`, labels and timestamp metadata. Comparing
R's absolute-double summary observations to retained offsets allows two absolute
reference ULPs; this does not relax normalized point/tick comparisons. A separate
counterexample checks precision-loss rejection for all three OOB policies.

Executed on Darwin arm64 with Rust 1.97.1; Python is the offline Rust 1.97.1 bookworm
container, and WASM uses wasm-bindgen CLI 0.2.128 and Node 24.14.0:

- `cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-positional-wasm`:
  **584 tests/doctests PASS**, 115 targets; `/tmp/ggplot-temporal-callback-full.log`.
  Subsequent test-only unit expansion and the added precision guard pass in
  `--test positional_temporal_limits` (two tests),
  `/tmp/ggplot-temporal-callback-final-primary.log`.
- All-target core/extension Clippy with `-D warnings`, rustdoc, formatting,
  repository checks and `git diff --check`: PASS;
  `/tmp/ggplot-temporal-callback-{clippy-final,rustdoc,fmt,repository}.log`.
- Fresh Python and WASM each pass **770 states**: 504 oracle configurations, two
  states per successful configuration, and 24 source replacements against fresh
  batches, including transitions through rejected empty populations and recovery.
  Authored JSON round trips, held scenes and immutable source definitions remain
  unchanged. Logs: `/tmp/ggplot-temporal-callback-python-final.log` and
  `/tmp/ggplot-temporal-callback-wasm.log`.
- Host records match exactly; **24 SVG/PDF/PNG publications are byte-identical**.
  Artifacts: `target/ggplot-position-temporal/wasm`,
  `target/ggplot-temporal-precision/linux-target/position-temporal-python`;
  comparison/hashes: `target/ggplot-position-temporal/comparison.json`.
  All outputs were inspected as native PNG, resvg SVG and Poppler PDF, recorded in
  `target/ggplot-position-temporal/inspection/inspection.json` with four review sheets.
- `scripts/run_primary_authoring_proofs.py` now runs both temporal host proofs, the
  primary Rust tests and exact record/publication comparisons. The cumulative runner
  was not executed in this slice.

Qualification is Date/UTC, explicitly shared origins, and unfaceted callback axes.
Host temporal inputs in this proof use milliseconds; other source units are covered
by the primary Rust matrix. Calendar resources, automatic timestamp origin inference,
faceted callbacks, duration callbacks, other registered arguments and the full
inventory reconciliation remain open. GG-04 stays IN PROGRESS; this does not close
GG-05–19, G-GGPLOT, G-PARITY or cumulative production gates.

## Duration missing-value qualification — 12 September 2026

The pinned implementation confirms that `scale_x_time` / `scale_y_time` expose
`na.value` but do not forward it to `datetime_scale`: the constructed scale keeps
`NA`. `AxisBuilder::missing_value` now retains this authored no-op for duration axes;
numeric scales keep their replacement behavior. Automatic duration guides reject
untrained/all-missing populations as in the reference. Direct method capture is in
`/tmp/ggplot-duration-missing-method.log`.

`tools/reference/r/positional-duration-missing.R` captures **192** additional point/
summary configurations in `fixtures/parity/ggplot2/positional-duration-missing.json`.
All **144 successes / 48 reference errors** pass the common primary test, including
coordinates, ranges, labels and tick positions. Full core/extension validation passes
**583 tests/doctests** and all-target Clippy passes. Logs:
`/tmp/ggplot-duration-missing-{oracle,primary,full,clippy}.log`.

Fresh Python/WASM pass the combined **2,540 states** (1,344 reference configurations
and twenty replacement/fresh-batch checks), with matching structures and the declared
numeric tolerance. **Thirty publications** are inspected: 28 are byte-identical and
the same two logarithmic SVGs retain twelve sub-picounit numeric differences. All 24
prior publication hashes are unchanged. The six new duration SVG/PDF/PNG files are
byte-identical; independent renders are inspected in
`target/ggplot-position-missing/inspection/duration-review.png`. Updated comparison
and inspection manifests remain in the same artifact directory. Runtime logs:
`/tmp/ggplot-duration-missing-{python,wasm,comparison}.log`.

GG-04 stays IN PROGRESS. Next: actual Date/datetime positional callbacks. Their new
504-case R corpus is captured by `tools/reference/r/positional-temporal-functions.R`
in `fixtures/parity/ggplot2/positional-temporal-functions.json`; no implementation or
acceptance is claimed from capture alone. Faceted callbacks, unbounded viewport
handling, remaining inventory arguments and GG-05–19 remain open.

## Positional missing-value qualification — 12 September 2026

Revision: `6e74ae6` plus working-tree changes. GG2-03/FIX-GG04 now has a shared
`AxisBuilder::missing_value(Option<Number>)` control, Python `Axis.missing_value` and
WASM snake/camel aliases. Definition version 30 preserves authored axis controls and
nested `ScaleProjection` controls; older envelopes cannot silently discard them.
Replacement is already in transformed units, applies after OOB handling, and is reused
for source values, statistics and authored optional endpoints. Missing values are not
confused with infinities. The control requires the ggplot2 pre-statistic numeric/duration
stage; binned scales retain their existing dedicated missing policy. Host drafts box
both existing builder variants to keep the expanded builder out of the inline enum;
this changes no authoring behavior.

The reference exposed and fixed additional shared-stage defects: an empty summary's
inspection row must not create a positional mark; callback limits must retrain before
post-statistic mapping; empty automatic scales have no default ticks; nonempty but
all-missing automatic populations retain unbounded ranges; negative square-root
replacement coordinates must stay in transformed units. Post-statistic callback
training now takes transformed observations directly, avoiding an unnecessary
inverse/forward round trip. A compensated residual pass corrects mean rounding; the
logarithmic negative-replacement summary previously differed by one ULP and was
incorrectly censored to the replacement. The independently captured R result is
`-0.2614393726401688`. This uses the existing sum/mean owner and the same residual
correction principle as [R's mean implementation](https://github.com/wch/r-source/blob/trunk/src/main/summary.c).

Reference and validation:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/positional-missing.R`:
  PASS on R 4.6.1 / ggplot2 4.0.3. The fixture contains **1,152** point/summary builds,
  covering identity/sqrt/reverse/log10, mixed/constant/all-missing/empty populations,
  absent/zero/five/negative replacement, automatic/fixed/function limits and all
  three OOB policies, plus **two nullable endpoint cases**. Log:
  `/tmp/ggplot-missing-oracle-final.log`.
- `cargo test -p chart-extension-example --test positional_missing --test positional_limits --locked --target-dir target/ggplot-positional-wasm`:
  PASS eight tests. The new tests check all **1,032 successful builds / 120 rejections**,
  finite coordinates, panel ranges, visible tick positions/labels, wire round-trips,
  endpoint mapping and stage guards. Log: `/tmp/ggplot-missing-primary-final.log`.
- `cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-positional-wasm`:
  PASS **582 tests/doctests**, no failures or ignored tests. Log:
  `/tmp/ggplot-missing-full-final.log`. The same target's all-target Clippy and strict
  rustdoc pass: `/tmp/ggplot-missing-clippy-final.log`, `/tmp/ggplot-missing-rustdoc.log`.
- Rebuilt Linux Python (offline Rust 1.97.1 container) and WASM (wasm-bindgen 0.2.128,
  Node 24.14.0) each pass **2,196 states**: original/edited round-trips of the full
  reference matrix and twelve replacement/fresh-batch checks preserving captured
  scenes. Logs: `/tmp/ggplot-missing-python-final.log`, `/tmp/ggplot-missing-wasm-final.log`.
  The executable proofs are `scripts/bindings/ggplot_position_missing.py` and `.cjs`.
- `scripts/bindings/compare_position_missing.py` compares complete record structures
  and numbers using `3e-12` absolute/relative tolerances. There are 120 non-identical
  record numbers and twelve SVG attribute numbers; the largest absolute difference
  is `2.5579538487363607e-13`. All **16 PNG/PDF files and six SVGs** are byte-identical;
  the other two SVGs have identical XML structure/text and only the recorded numeric
  attribute differences. No reference fixture or comparison number is rounded.
- All **24 publications** were inspected: native PNGs, independent SVG rasterizations,
  and Poppler PDF renders. The initial sample canvas was too short for its explicitly
  authored y range; final samples use 640 by 640 points. Artifacts and hashes:
  `target/ggplot-position-missing/{wasm,comparison.json,inspection/inspection.json}`;
  the four inspected sheets are `inspection/final-review-1.png` through `-4.png`.
  Python outputs are in `target/ggplot-temporal-precision/linux-target/position-missing-python`.
- Strict TypeScript/mypy consumers, formatting, repository checks and diff checks
  pass: `/tmp/ggplot-missing-{tsc-final,mypy-final,fmt-check,repository}.log`.
  The primary authoring runner includes the new matrix and tolerant comparison;
  its full cumulative run is not claimed here.

This qualifies finite positional missing replacement and the stated reference
rejections. Infinite geometry is still excluded at the finite-geometry boundary;
finite viewport handling over unbounded populations, duration-specific qualification,
temporal/faceted positional callbacks, other inventory arguments and cumulative
native/platform gates remain open. GG-04 is IN PROGRESS. Next: duration missing-value
qualification, then the remaining callbacks and inventory, followed by GG-05–19.

## Positional callback log/OOB qualification — 12 September 2026

A supplemental pinned matrix now covers **756** additional continuous/binned cases:
logarithmic transforms and explicit censor/squish/keep OOB controls. The original
252 cases and all 48 stage records remain unchanged. The supplemental primary tests
pass **219 continuous successes / 159 rejections** and **177 binned successes / 201
rejections**, comparing coordinates, final limit vectors, ranges and guide positions
and labels. No production change was required; the existing shared policies matched.

Both current rebuilt hosts pass the combined **1,584 matching states** (1,008 oracle
configurations plus 25 replacement/fresh-batch states). **48 publications** match
byte for byte. Six new logarithmic SVG/PDF/PNG files were inspected; the prior 42
files retain their inspected hashes. The test harness was corrected to retain the
immutable builder returned by `oob` before final execution.

Evidence: `fixtures/parity/ggplot2/positional-limit-function-arguments.json`, generated
by `tools/reference/r/positional-limit-functions.R`; primary/core log
`/tmp/ggplot-function-arguments-primary-final.log`; actual host logs
`/tmp/ggplot-function-arguments-{python,wasm}.log`; numeric/hash comparison
`target/ggplot-position-functions/argument-comparison.json`; inspection sheet
`target/ggplot-position-functions/inspection/argument-review.png`. The primary runner
includes both matrices and new export comparisons. The prior full-core qualification
remains the implementation gate; this extension changes tests/reference/evidence only.

**GG-04 remains IN PROGRESS.** Symlog/custom transforms, temporal/faceted positional
callbacks, missing-value controls, finite viewports over unbounded populations,
remaining registered arguments and cumulative acceptance remain open. Next: positional
missing-value controls, then the remaining GG-04 inventory and GG-05–19.

## Binned positional callback integration — 12 September 2026

GG2-03/FIX-GG04 at `6e74ae6` plus working-tree changes now supports numeric
binned positional limit functions through the same primary Rust/Python/WASM axis
API. Initial ordered limits drive OOB handling and classification; the scale resets
from those limits and cached cuts, evaluates the function again, and maps statistic
indices into the retained intervals. Statistic values do not retrain these bins.
The shared binned break selector owns cut generation; no second bin engine or host
callback evaluator was added. Wire 29 retains the captured function-limit vector.

All **126 binned primary R cases** match: **74** have renderable point/range/guide
results, **49** reject during the reference build, and **three** empty-population
builds reject when the reference projects the panel. Tests preserve that stage
distinction in the oracle. The original 126 continuous cases remain unchanged.
The oracle adds **24 binned summary/shared-layer cases**, bringing that stage matrix
to 48; prepared coordinates and final limits match throughout. Re-running the pinned
R script preserved all previously captured observations exactly.

Validation and artifacts:

- `cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-positional-wasm` passes **571 unit/integration tests**. Its final doctest step lost an rlib during overlapping builds; the isolated `--doc` rerun passes all **six doctests**. Logs: `/tmp/ggplot-binned-functions-full.log` and `/tmp/ggplot-binned-functions-doc-tests.log`. The final primary target passes its three tests, including all 48 stage cases (`/tmp/ggplot-binned-functions-primary-final.log`).
- Final all-target core/external Clippy passes (`/tmp/ggplot-binned-functions-clippy-final.log`). A redundant box around the retained vector was removed after the first Clippy run; final hosts were rebuilt after that correction.
- Both rebuilt hosts pass **432 exactly matching states**, covering 252 original/edited oracle configurations and 25 source replacements versus fresh batches with immutable captures. Logs: `/tmp/ggplot-binned-functions-python-final.log` and `/tmp/ggplot-binned-functions-wasm-final.log`.
- **42 SVG/PDF/PNG files** match byte for byte. All eighteen new binned files were inspected using original PNGs and independently rasterized SVG/PDF; the 24 continuous files retain their previously inspected bytes. Comparison: `target/ggplot-position-functions/binned-comparison.json`; contact sheets: `target/ggplot-position-functions/inspection/binned-review-{1,2}.png`. The primary proof runner now includes these comparisons.
- Strict Python/TypeScript binned callback consumers and repository checks pass (`/tmp/ggplot-binned-functions-{mypy,tsc,repository}.log`). The whole cumulative primary proof runner and native-window/transition acceptance were not rerun.

**GG-04 remains IN PROGRESS.** Faceted/timestamp positional callbacks, finite viewports
over unbounded populations, log/symlog and broader OOB qualification, remaining
registered arguments and cumulative destination/platform gates remain open. Next:
qualify logarithmic and OOB callback behavior, continue the remaining GG-04 inventory,
then GG-05–19. Empty-panel presentation remains a GG-05 acceptance gap.

## Continuous positional callback integration — 12 September 2026

At `6e74ae6` plus working-tree changes, the primary numeric axis now accepts an
owned `ScaleLimitsOperation` through `AxisBuilder::limits_function`, Python
`Axis.limits_function` and WASM `limits_function`/`limitsFunction`. Definition wire
**29** retains this selection. The existing evaluator and numeric family conversion
remain shared with aesthetic callbacks. A bounded positioned-row stage allows all
layers to train a common second population before geometry is emitted. Original
plot JSON remains immutable; prepared charts retain ordered, transformed limit
vectors separately and expose them through `positional_limits` inspection metadata.

The pinned R corpus now also records guide/point positions and 24 summary/shared-layer
cases. Removing only these added observations reproduces all 252 original records
exactly. Primary tests match **all 126 continuous cases** (81 successful builds and
45 reference rejections) for identity, square-root and reverse transforms, plus
**24 summary/shared-layer cases**. Successful cases compare finite prepared points,
raw final limit vectors, finite panel ranges, visible tick positions and labels.
The reference singleton infinite panel range is represented by equal infinite
endpoints for projection while its original limit-vector arity is retained.

The host proof exposed a visible infinite tick in the constant-infinite panel case.
The shared unbounded projection now maps this tick to the midpoint. Numeric semantic
values retain the existing finite JSON encoding and use canonical `Number` transport
for infinities. Scene wire **16** marks nonfinite guide identities; destination
geometry remains finite and NaN guide metadata is rejected. This is static guide
qualification, not a transition/native-window acceptance claim.

Executed in the repository (Darwin arm64 Rust 1.97.1; offline Linux Python adapter;
Node 24.14.0 and wasm-bindgen CLI 0.2.128):

- `cargo test -p chart-core -p chart-extension-example --locked --target-dir target/ggplot-positional-wasm`: **576 tests/doctests passed**, none failed/ignored; `/tmp/ggplot-position-function-full-final.log`. The external `positional_limits` target also passes after the final inspection assertion changes (`/tmp/ggplot-position-primary.log`).
- Core/external all-target Clippy, warning-denied core rustdoc, formatting and repository checks pass (`/tmp/ggplot-position-function-clippy-final.log`, `/tmp/ggplot-position-function-doc.log`, `/tmp/ggplot-position-function-fmt-check.log`, `/tmp/ggplot-position-function-repository.log`).
- Rebuilt Linux Python and WASM primary adapters: **222 matching states each**, including original/edited wire round-trips, expected layout rejections, fifteen source replacements versus fresh preparation, and immutable captured scenes. Logs: `/tmp/ggplot-position-function-python-final.log` and `/tmp/ggplot-position-function-wasm.log`.
- **24 SVG/PDF/PNG files** match byte for byte and were inspected, including the
  signed infinite guide cases. Numeric equality, file hashes and inspection records:
  `target/ggplot-position-functions/comparison.json`; views in its `inspection/`
  directory. Python artifacts are under
  `target/ggplot-temporal-precision/linux-target/position-functions-python`.
- Strict Python and TypeScript consumers pass (`/tmp/ggplot-position-function-mypy.log`,
  `/tmp/ggplot-position-function-tsc.log`). The TypeScript declarations now include the
  already implemented `scaleSqrt` and `scaleReverse` factory aliases. The primary
  proof runner includes the new callback tests, runtimes, file comparisons and types;
  the entire cumulative runner was not rerun for this slice.

Binned, faceted and timestamp positional callbacks remain explicitly unsupported.
Finite viewports over unbounded callback limits are also rejected rather than ignored.
Log/symlog and broader OOB/argument combinations require their remaining primary
qualification. Registered break/minor/label/palette/OOB/rescaler/transform argument
reconciliation, free-facet bins, remaining category/time controls and cumulative
native/platform gates remain open. **GG-04 and GG-05–19 are not complete.** Next:
implement the binned reset/cached-cut population contract, then continue the remaining
GG-04 argument inventory and downstream packages.

## Positional callback groundwork — 11 September 2026

The 252-panel positional oracle is captured. The existing shared numeric evaluator
now matches the first callback input in all **238 invoked cases** and preserves all
**14 pre-invocation errors**. This is first-stage evidence only: the actual primary
positional callback adapter, post-statistic retraining, output coordinates, expanded
ranges and guides remain unimplemented/unqualified.

The numeric callback evaluation was separated from aesthetic policy materialization,
and filtered positional source collection was extracted from the existing binned
training path. Both remain internal helpers with no new wire version or host API.
Missing observations remain in the collected population; row work stays bounded.
The extraction preserves the existing transform, limit order and arity behavior.

Executed on Darwin arm64, Rust 1.97.1, revision `a6caa39` plus working-tree changes:

- `mise exec -- cargo test -p chart-core --test ggplot_position_bins --locked --target-dir target/ggplot-positional-wasm`: **7 passed**, including 2,400 direct reference records, 960 primary panels, statistic/filter and replacement contracts (`/tmp/ggplot-positional-shared-bins.log`).
- `mise exec -- cargo test -p chart-extension-example --test numeric_limits --test discrete_limits --locked --target-dir target/ggplot-positional-wasm`: **9 passed** after helper extraction (`/tmp/ggplot-positional-shared-test.log`).
- The numeric target was rerun after adding the positional input comparison: **5 passed**, with the existing numeric/temporal matrices retained (`/tmp/ggplot-positional-input-contract.log`).
- All-target core/external Clippy and formatting pass (`/tmp/ggplot-positional-shared-clippy.log`, `/tmp/ggplot-positional-shared-fmt.log`).

The 574-test full-suite and 1,408-state host qualification below precede this internal
extraction; actual hosts/publications were not rerun for it. GG-04 and G-GGPLOT remain
open. Next: integrate shared limit evaluation before and after statistics, retaining
execution state separately from authored plot JSON, then validate the complete
positional oracle and rebuilt hosts/destinations. GG-05–19 remain outstanding.

## Temporal limit callback extension — 11 September 2026

The shared callback now receives optional `ScaleLimitsInput.temporal` context with
an exact origin, timestamp unit and Date/datetime distinction. Values and function
results remain numeric offsets in that unit. The existing temporal guide owner
handles returned limits, including singleton formatting. Empty populations fail
before callback invocation, matching all fourteen reference cases.

The 84-case pinned oracle passes **336 Rust configurations**, across four timestamp
units, for limits, values, guide breaks/labels, JSON restoration and replacement
versus fresh training. The callback observer checks all seventy invoked reference
inputs and the exact nanosecond origin; all fourteen uninvoked cases remain uninvoked.
Five numeric and four discrete external tests pass
(`/tmp/ggplot-temporal-limit-test2.log`). Core/external all-target Clippy passes
(`/tmp/ggplot-temporal-limit-clippy.log`).

Rebuilt WASM/Node passes **1,408 states** including the earlier 892-state numeric
lane. Temporal host coverage uses four units for exact whole-unit inputs and three
units for fractional datetime inputs: integer seconds cannot represent these values.
The direct Rust offset API still tests fractional offsets in seconds. Six temporal
reference guide errors are distinguished from successful direct mapping. Rebuilt Python also passes all 1,408 states, exactly matching WASM. The full Linux
core/external suite passes **574 tests/doctests, 0 failed, 0 ignored**
(`/tmp/ggplot-temporal-limit-linux-python.log`). All **36 SVG/PDF/PNG files** match
both hosts byte-for-byte; the eighteen earlier files also match their inspected hashes.
Sixteen temporal live replacements match fresh batches while held publication scenes
and authored plot JSON remain unchanged. Log: `/tmp/ggplot-temporal-limit-wasm4.log`.
Six new temporal PNG samples are inspected:
fixed limits retain four increasing points, reversed limits omit all points, and
single limits retain the upper observation. Six independent Date SVG/PDF rasters are inspected; the six datetime rasters are
byte-identical to those inspected files. Rustdoc and repository checks pass
(`/tmp/ggplot-temporal-limit-doc.log`, `/tmp/ggplot-temporal-limit-repository.log`).
Formatting and diff checks pass. Comparison records and inspected hashes are in
`target/ggplot-numeric-limits/`. GG-04 stays open.

Next: positional callbacks. The new `positional-limit-functions.R` oracle captures
252 primary R panels, including pre/post-stat training differences, limits, values,
expanded ranges and axis candidates. Generation passes
(`/tmp/ggplot-positional-limit-oracle.log`); implementation/equivalence remain open.

## Registered numeric limit functions — 11 September 2026

The shared pure callback contract is now `CustomScaleLimits` / `ScaleLimitsInput` /
`ScaleLimitsOperation`, installed with `register_scale_limits` and selected by
`MappedScaleSpec::with_limits_function`. There is one registry owner for numeric and
categorical domains. Wire 28 retains the `limits_function` field; this uncommitted
slice generalizes the earlier discrete-only names. Registered operation identities
remain unchanged. Numeric training retains the returned source-coordinate vector in
`resolved_numeric_limits`, including singleton, empty and missing results, and
recomputes it on replacement. It does not sort function results or substitute trained
endpoints for missing output. Fixed-limit semantics remain separate.

All **378 numeric R cases** pass limits, mapping, guide breaks/labels and serialized
retraining checks: continuous size, binned size and numeric identity, under identity,
square-root and reverse transforms. The second external test checks every callback
input reached by R (357 cases; reverse inverse(NULL) fails before invocation in 21),
and rejects categorical callback output on a numeric scale. Singletons preserve the
reference lower-bound-only censor, half-range normalization and recycled binned
scalar; singleton binned guide candidates retain missing values. Empty identity and
square-root callback inputs preserve NULL versus an empty numeric vector.

Actual rebuilt Linux Python 3.11.2 and WASM/Node 24.14.0 each pass **677 exactly
matching states**: 269 successful cases restored/edited twice, 109 expected numeric
errors, and 30 live replacement states versus fresh batches. Held publication scenes
and authored plot JSON remain unchanged by replacements. Identity alpha is observed
before paint clamping through the independently checked after-scale size expression
`2 + 0.1 * coalesce(alpha, 0)`; its final paint alpha is checked separately.
Eighteen SVG/PDF/PNG files match byte-for-byte. All six PNGs and independent SVG/PDF
rasterizations were inspected: reversed continuous limits omit all points; a single
continuous limit retains only the upper observation; binned reverse/single limits
produce the expected equal-size points. Axis text and marks are unclipped.

The discrete callback proofs still pass **198 exact states** and twelve unchanged
publication files. A script-name collision had replaced the earlier positional-limit
proof scripts; they are restored as `ggplot_positional_limits.py/.cjs`, with distinct
aggregate stages/output directories. Both rebuilt hosts pass their **3,582 exact
states** (3,558 configurations and 24 replacements). Their twelve publications match
both hosts and the earlier inspected `target/ggplot-discrete-order/wasm` files exactly.
The aggregate runner is updated, but has not been run end-to-end.

Executed at starting revision `a6caa39` plus retained working-tree changes:

- Offline Linux `cargo test -p chart-core -p chart-extension-example --locked --offline -j2 --target-dir /target`: **571 passed, 0 failed, 0 ignored**, including doctests. Log: `/tmp/ggplot-numeric-limit-linux-python.log` (the subsequent first host-script attempt failed on an incompatible Label channel; the corrected Alpha proof below passes).
- Darwin focused `cargo test -p chart-extension-example --test numeric_limits --test discrete_limits --locked --target-dir target/ggplot-positional-wasm`: callback tests pass; the later callback-input test also passes. Logs: `/tmp/ggplot-numeric-limit-test3.log`, `/tmp/ggplot-numeric-limit-test4.log`.
- Core/external all-target Clippy: PASS, `/tmp/ggplot-numeric-limit-clippy.log`.
- Rebuilt Python numeric/discrete/positional runs: PASS, `/tmp/ggplot-numeric-limit-python-final.log`.
- Rebuilt WASM runs: PASS, `/tmp/ggplot-numeric-limit-wasm3.log`, `/tmp/ggplot-numeric-limit-discrete-wasm.log`, `/tmp/ggplot-restored-positional-wasm.log`.
- Host records/publications: `target/ggplot-temporal-precision/linux-target/{numeric,discrete,positional}-limits-python`; WASM outputs under `target/ggplot-numeric-limits/{wasm,discrete-wasm,positional-wasm}`. Exact comparisons: `target/ggplot-numeric-limits/comparison.json`; independent inspection: its `inspection/` directory.

This covers the extent rescaler and named transforms above, not every callback
combination with maximum/midpoint rescaling, temporal domains, positional limits or
break/label callbacks. Timestamp callbacks reject pending a typed temporal domain.
These remaining arguments, GG-05 presentation and cumulative platform/export gates
keep GG-04 open. Next: qualify the other rescalers and remaining registered arguments.

The subsequent `limit-rescalers.R` fixture adds 126 range/maximum/midpoint cases
using an identity output palette. Focused tests now pass 504 numeric cases in total,
including mapping-versus-guide errors and serialized replacement training. Fifteen
initial differences identified maximum-rescaler arity/missing semantics; the final
remaining difference required selecting a single bin before a nonfinite endpoint
was rescaled. Seven focused numeric/discrete tests pass in
`/tmp/ggplot-limit-rescalers-after2.log`. A fourth numeric test checks primary
NaN output: ggplot2 treats it as missing while the legacy profile retains its error.
Both rebuilt hosts now pass **892 exactly matching states** (358 successful cases
restored/edited twice, 146 expected errors and 30 replacements). All eighteen
publication files match both hosts and the saved inspected SHA-256 digests. Logs:
`/tmp/ggplot-limit-rescalers-linux-python.log` and
`/tmp/ggplot-limit-rescalers-wasm2.log`. Final all-target Clippy and rustdoc pass
(`/tmp/ggplot-limit-rescalers-final-clippy.log`,
`/tmp/ggplot-limit-rescalers-final-doc.log`). The stable-source full-suite run passes **573 tests/doctests, 0 failed, 0 ignored**
(`/tmp/ggplot-limit-rescalers-final-full.log`); the 571-test result above predates
these fixes. This qualification precedes the next temporal callback changes.

The next temporal callback oracle contains 84 pinned Date/datetime cases, including
fractional inputs, returned classes, reverse/singleton/missing results and empty
populations. R fails before callback invocation for all fourteen empty-population
cases. Oracle generation passes (`/tmp/ggplot-temporal-limit-oracle.log`); Rust and
WASM equivalence are recorded in the newer section above; Python qualification is
recorded in the newer section above.

## Registered discrete limit functions — 11 September 2026

A compiled external implementation now supplies discrete `limits` functions through
`CustomScaleLimits`, a bounded registry entry selected by an exact operation/version
and declarative parameters. It receives the existing factor/drop/NA-trained domain;
core retains sole ownership of training, mapping and guides. NULL and empty-vector
results remain distinct. Fixed limits are replaced by the function result. Retraining
uses the new population rather than the previously materialized result. Wire 28
retains the selection, and portable restoration requires the matching registration.
Native-only implementations prepare locally but cannot serialize to portable charts.
Guide-disabled identity scales pass NULL to the callback even with nonempty data;
they retain observed raw values separately for pooling and direct mapping.

The pinned 468-case R fixture includes **90 discrete/identity cases**, all passing
external Rust comparisons of limits, mapped values and guide labels. The other 378
numeric/binned/identity cases are qualified by the newer slice above. Four external tests also cover missing/wrong-version registrations,
invalid parameters, registry copying, native-only serialization, factor/NA inputs,
replacement training, exact-number precision and output count/text budgets.

Rebuilt Python (`extension-module,extension-proof`, Linux Python 3.11.2) and WASM
(`extension-proof`, Node 24.14.0) each pass **198 matching primary states**:
original/restored-and-edited mapping plus two expected NULL-result failures and
twenty live replacements through constant/missing/all-missing/empty populations.
Replacement results match fresh batches and held publication frames remain unchanged.
Identity values are retained through the Label aesthetic on points; this proves
shared value transport, not GG-08 text rendering. Twelve SVG/PDF/PNG files match
byte for byte. All four PNGs and independent SVG/PDF rasterizations were inspected.
Discrete reverse/fixed samples show the expected symbols and omissions; identity
samples retain all points. GG-05 guide presentation is not covered by these images.

Commands and artifacts:

- `cargo test -p chart-extension-example --test discrete_limits --locked --offline`
  in the established offline Linux container: four tests pass.
- `cargo clippy -p chart-core -p chart-extension-example --all-targets --locked -- -D warnings`: PASS.
- `scripts/bindings/ggplot_discrete_limits.py` and `.cjs`: PASS; host results under
  `target/ggplot-temporal-precision/linux-target/discrete-limits-python` and
  `target/ggplot-discrete-limits/wasm`.
- `target/ggplot-discrete-limits/comparison.json` records the exact comparison;
  `inspection/*-{svg,pdf}.png` holds independently rendered inspection images.
- The primary proof runner includes this stage; the aggregate runner has not been run.

The complete core/external suite passed **569 tests/doctests** with zero failures
or ignored tests before the final hidden-identity input correction. The four focused
external tests, now including all ninety R cases, and rebuilt 198-state host proofs
pass after that correction. Rustdoc passed before the final correction; repository,
formatting and diff checks pass. Numeric callbacks, other remaining
GG-04 arguments, GG-05 presentation and cumulative host/platform gates remain open.

## Binned numeric mapping and physical linewidth — 11 September 2026

The additional 216 numeric R cases cover size, area, alpha and linewidth. Of these,
**204** correspond to the generic binned policy; twelve `right=FALSE` calls are
rejected by R's size/linewidth convenience wrappers, which lack that argument.
Their raw records remain in the fixture. The generic policy corresponds to
`binned_scale`, which accepts closure selection. R fallback palettes are explicitly
installed for direct scale sampling, as plot construction otherwise installs them.
Constant-bin scalar output is recycled across input rows at the chart boundary.
The direct Rust comparison passes without changing palette or bin arithmetic.

Raw mapping is recorded separately from `ggplot_build`: constant-area explicit
breaks and all-missing area with one authored limit map successfully but R's binned
guide construction fails. These were deferred guide requirements at this checkpoint; the later non-color
binned-label slice qualifies their primary rejection without changing raw mapping. Rejecting their
valid scale mapping would conflate the two contracts. Python/WASM each pass
**372 numeric states**, including 36 mapping errors, and produce twelve identical
numeric publication files. Explicit strokes make rule styles observable through
the existing semantic DTO; this does not change mapped numeric values.

Visual inspection then exposed a real physical-width mismatch: nonpoint reference
linewidths were still destination units. The existing line, area, ribbon, bar, rule
and rectangle families now convert implicit reference widths at shared scene projection.
R's unit is `72.27 / 25.4` logical pixels per linewidth, or three quarters of that
in publication points; zero becomes the physical 0.01-point PDF hairline. Explicit
`aesthetic_units` overrides and LibraryV1 retain their unit contract. Constant and
mapped zero widths are admitted before projection. No new public field or wire
version is needed for this profile correction. Point size/stroke conversion remains
owned by its existing path.

The 30-record device fixture supplies grob linewidths and PDF-rounded widths.
Rectangles/polygons explicitly request an outline; nested line grobs are traversed.
The core test checks **96** constant/mapped, logical-pixel/point layouts for segment,
line, path and rectangle, plus explicit destination-unit compatibility. Six polygon
records await GG-07 geometry and are not claimed as implemented. The focused suite
passes **17 tests** (aesthetics and both binned suites). Actual rebuilt hosts each pass
**96 additional states** whose SVG stroke widths match R-derived physical values to
`2e-12`; their twelve publications also match byte for byte.

Current total: **860 matching Python/WASM states and 36 byte-identical publications**
in `target/ggplot-binned-styles/{wasm,inspection,comparison.json}` and the Linux Python
folder documented below. All twelve PNG samples and their independent SVG/PDF
rasterizations were inspected, including the changed dash and linewidth samples.
All-target Clippy and repository/format/whitespace checks pass. The final full Linux
core suite passes **547 tests**, zero failed/ignored, in
`target/ggplot-binned-styles/full-core-linewidth.log`. This includes the physical-width
correction and both binned suites.
No R pixel equivalence, complete guide presentation, aggregate primary runner or fresh
macOS/native runtime qualification is claimed. GG-04 and cumulative gates remain open.

## Binned count palettes — 11 September 2026

`GgplotBinnedPalette` adds filled/hollow shape, linetype and Brewer fermenter
selection to the existing binned policy. The resolved interval count selects the
palette once, through the discrete palette generators; existing threshold search,
closure, training, limits, guide candidates and labels remain shared. Palette overflow
uses the configured missing value, including Brewer's grey50. The count palette
requires an identity range function and is rejected on positional bin policies.
Manual named-key behavior is unchanged. Chart/plot wire **27** records this optional
capability; old payloads omit it. Value-channel preparation now checks sampling
validity on empty layers, as numeric and color preparation already did.

The pinned **216 R builds** cover solid/hollow symbols, thirteen-pattern linetypes,
Blues Brewer colors, six populations and nine controls. The core fixture checks
mapping, JSON, guide candidates and labels. The focused suite passed **22 tests**
across binned styles, existing discrete styles, binned guides and degenerate mappings
before the final empty-layer integration correction. Actual rebuilt Python and WASM
now each pass **392 states**, including forty expected errors, roundtrip and replacement
layers. Primary `value_scale` and `color_mapped` authoring supply the descriptors;
Brewer publication maps both fill and stroke to make the selected colors visible.

The two hosts produce exactly equal records and **twelve byte-identical SVG/PDF/PNG
files**. All four PNGs and independent SVG/PDF rasterizations were inspected: symbol
sequence/overflow, hollow symbols, dash changes and nine Brewer bins are visible.
No binned-guide presentation or R pixel equality is claimed; GG-05 remains open.
Artifacts: `target/ggplot-binned-styles/{wasm,wasm-module,inspection,comparison.json}`
and `target/ggplot-temporal-precision/linux-target/binned-styles-python`.
Commands: `tools/reference/r/run.py tools/reference/r/binned-style-palettes.R`,
`cargo test -p chart-core --test ggplot_binned_styles --test ggplot_style_palettes
--test ggplot_binned_guides --test ggplot_degenerate_mapping --locked --offline`,
and `scripts/bindings/ggplot_binned_styles.py` / `.cjs`. Linux uses the cached offline
Rust 1.97.1 image, Python 3.11.2; WASM uses Node 24.14.0 and bindgen 0.2.128.
The final Linux full core suite passed **545 tests**, with zero failures/ignored,
including the empty-layer correction. All-target Clippy, formatting, repository and
whitespace checks passed. The full log is
`target/ggplot-temporal-precision/linux-target/binned-styles-full-core.log`.
Aggregate primary proofs and macOS/native runtime have not been requalified.
GG-04 remains IN PROGRESS. Subsequent numeric mapping and physical-width work is qualified separately above;
the 545-test result predates the linewidth correction.

## Date/datetime aesthetic guide arguments — 11 September 2026

`GgplotTemporalGuideArguments` now distinguishes automatic, null, empty, explicit
and character-width breaks; pretty counts, hidden/explicit labels and R date formats
share the existing guide and formatter owners. Date short-span padding and uncropped
width enclosures are produced in the existing calendar selectors. Positional callers
retain viewport censoring. Constant domains replace explicit/empty candidates with
one break; null candidates bypass that rule and nonfinite generation errors. Authored
partial/full limits keep the common population policy. Date palette arithmetic uses
absolute days independently of guide selection. `with_timestamp_normalization`
configures that arithmetic; `with_guide` does not change palette values.

Chart/plot **wire 26** retains non-default temporal arguments or Date arithmetic;
ordinary datetime defaults remain 25. Standalone Date normalization uses **wire 5**,
datetime normalization 4. Optional explicit format storage is boxed so its locale
does not inflate every guide variant. Old default payloads omit the new arguments.

The pinned **156 R cases** cover two temporal kinds, six populations and thirteen
controls, including build/guide-query/drawing failures. One Rust test runs four source
units, full candidates/labels, mapping, JSON and replacement layers. Authored numeric
scales are explicitly reattached when replacing a layer, as required by the existing
replacement API. Empty populations have no drawable guide; a raw R `get_labels()`
error on an empty explicit label vector is not a drawable-guide error. The core test
checks this boundary rather than claiming the combined guide API is an R scale object.
Final focused Date, pretty-time and width-string regressions pass; the full offline
Linux core suite passes **543 tests/doctests**, zero failed or ignored.

Actual rebuilt Linux Python 3.11.2 and WASM each pass **1,144 guide states**, including
JSON, replacement, standalone Date/datetime round trips and expected errors. The
existing 180 precision and 440 default states also pass: **1,764 exact matching records**
per host. Both hosts produce **21 byte-identical SVG/PDF/PNG files**. Three new PNGs and
six independent SVG/PDF renders were inspected, and final output hashes retain all
inspected bytes. Numeric guide painting remains GG-05 scope; the large-point sample
still clips at the panel edge. No R visual-equivalence or fresh macOS/native run is
claimed. Artifacts: `target/ggplot-temporal-arguments/{wasm,inspection,comparison.json}`
and `target/ggplot-temporal-precision/linux-target/temporal-arguments-python`.

Final logs: `target/ggplot-temporal-precision/linux-target/temporal-arguments-full-core.log`,
`temporal-arguments-focused-final.log`, `/tmp/ggplot-temporal-arguments-linux-final.log`,
`/tmp/ggplot-temporal-arguments-wasm-{final,precision,defaults}.log`,
`/tmp/ggplot-temporal-arguments-{clippy-final,mypy,tsc,repository}.log`.
All-target core Clippy, strict Python/TypeScript consumers, formatting, repository
and diff checks pass. Scoped stages are in the primary proof runner; the aggregate
runner has not been run.

The **60-build width extension** now qualifies calendar units, fractional alignments
and malformed strings, using microseconds for fractional-second through multi-year
spans. R accepts trailing fields only on second widths, not minute widths. Rust and
WASM both reproduced the incorrect acceptance of `2 mins extra`; the shared aesthetic
width adapter now rejects it. Two focused Rust tests pass all **216 R cases** and their
JSON/replacement checks. Rebuilt Linux Python and WASM each pass **1,856 exact matching
states** (1,236 argument/width states plus the 620 temporal regressions), with all 21
inspected publication bytes unchanged. Logs are `/tmp/ggplot-temporal-widths-*.log`
and `target/ggplot-temporal-precision/linux-target/temporal-widths-focused.log`;
`target/ggplot-temporal-arguments/width-comparison.json` retains counts and hashes.
All-target Clippy, formatting and repository/diff checks pass. The full 543-test run
above predates this single parser correction; the latest Rust execution is focused.
Binned shape/linetype palette generation, remaining arguments/callbacks, guide
presentation and cumulative gates keep GG-04 open.

## Absolute timestamp palette arithmetic — 11 September 2026

The next temporal counterexample is absolute-double precision: at epoch 1704067200,
R treats spans at or below 0.0001 seconds as numerically constant, whereas rescaling
exact origin-relative offsets produced a full palette. At 0.01 seconds the reference
also retains the rounding from absolute POSIXct values. The retained older WASM
module reproduces case 70: radius 3.5 instead of R's 3.5000298021731755
(`/tmp/ggplot-temporal-precision-before-wasm.log`). `NormalizationSpec::Ggplot`
now optionally records the exact origin and source unit, converts only reference
normalization/OOB arithmetic to absolute seconds, and retains original source data and
domains. This context is independent of guide visibility. The shared normalizer owns
the conversion; color and numeric consumers use the same result. Chart/plot wire 25
and standalone scale wire 4 retain this context; descriptors without it retain their
previous version and semantics. Only linear range rescaling accepts this context.

`temporal-aesthetic-precision.R` captures **180 pinned R builds**: three epochs,
six spans, five aesthetics and visible/hidden guides. Actual rebuilt WASM passes all
180 cases, plus the existing **440 temporal states**. Rebuilt Linux Python 3.11.2
passes the same 620 cases. Both hosts' records match exactly and their twelve
publication files are byte-identical to each other and to the inspected timestamp
default artifacts. Records are in `target/ggplot-temporal-precision/{wasm,linux-target/python}`;
`comparison.json` retains counts and hashes. Logs use `/tmp/ggplot-temporal-precision-`
(`oracle.log`, `wasm.log`, `clippy.log`, `repository.log`) and
`/tmp/ggplot-temporal-defaults-after-precision-wasm.log`. All-target core compile,
Clippy, formatting and repository/diff checks pass. The primary proof runner includes
both new host scripts; the aggregate runner has not been executed.

Offline Linux core validation passes **542 tests/doctests**, zero failed or ignored,
including all five focused temporal regressions. The macOS test executables and Python
import remained delayed at startup and were stopped after offline proofs passed;
this does not establish a fresh macOS runtime pass. The older aggregate retry reached `ggplot_discrete_null` and stalled
at `_dyld_start` (112 KB, `/tmp/ggplot-temporal-discrete-null-sample.txt`). Superseded
aggregate/before/intermediate runs were stopped; no assertion result is inferred.
Full core testing passed in the cached
`rust:1.97.1-bookworm` Linux image, with networking disabled, read-only source and
registry mounts, and a task-owned target (`/tmp/ggplot-temporal-precision-linux.log`).
The Python binding proof also passed offline in that image
(`/tmp/ggplot-temporal-precision-linux-python.log`).
No system protections or settings were changed. Date-specific short spans, authored
temporal arguments, guide presentation and the cumulative GG-04 gate remain open.

## Automatic timestamp aesthetics — 11 September 2026

Point-size inference previously omitted `Numeric::Timestamp`, leaving raw timestamp
offsets as point sizes. Automatic size now uses the existing reference palette,
matching alpha and linewidth inference. Numeric edit reconstruction retains timestamp
origins and scale identities. Temporal guide metadata retains the exact origin, input
unit and explicit calendar resource, requiring wire **24**. Its candidate selection
reuses the existing datetime selector before viewport censoring. Empty observations
produce no candidates; an all-missing temporal population fails guide preparation, as
in R. Authored numeric scales retain their chosen guide policy.

The pinned `temporal-aesthetic-defaults.R` corpus has **60 builds**: Date/datetime,
six populations, and size/alpha/linewidth/color/fill. Date inputs in this corpus are
materialized as exact UTC timestamp columns; this does not establish all Date-specific
selection rules. Three focused Rust tests pass all four timestamp units, mapped values,
full candidate labels, JSON round-trips and edits, plus deliberate non-default origins
and authored-scale isolation. Python and WASM each pass **440 states** and produce
identical records and **twelve byte-identical SVG/PDF/PNG files**. The host style check
compares alpha after its documented byte quantization; Rust checks the numeric palette
before lowering. Typed Python and TypeScript timestamp consumers pass.

Artifacts are `target/ggplot-temporal-aesthetics/{python,wasm,inspection,comparison.json}`.
All four PNGs and independent SVG/PDF renders were inspected. Temporal color labels
are present. Numeric/fill legend presentation is still absent in these samples and
the largest size sample clips at the panel edge: GG-05 remains open. Date-specific
short spans, explicit temporal guide arguments and local-calendar argument parity
remain to be reconciled. No native-window, Linux or cumulative binding-runner acceptance
is claimed. The focused runner is added to `run_primary_authoring_proofs.py`.

Validation logs use `/tmp/ggplot-temporal-aesthetics-`: `oracle.log`,
`final-focused.log`, `python-final.log`, `wasm-final.log`, `mypy.log`, `tsc.log`,
`clippy.log` and `repository.log`. All-target Clippy, formatting and diff checks pass.
Final rebuilt host files retain all inspected hashes. The full-core attempt remains
**INCOMPLETE**: after 13 unit tests passed, the `actions` integration executable
stalled before the test harness started. A one-second process sample showed only
`_dyld_start` and a 112 KB footprint (`/tmp/ggplot-temporal-actions-sample.txt`). A
fresh build in `target/axis-rust-proof`, a copied executable and an unrestricted
`--list` attempt initially reproduced the startup delay. The fresh build eventually
started and all nine `actions` tests passed. Task-owned duplicate diagnostics were
terminated. No test assertion failure was reported; this is not a full-suite pass.
The initial unrestricted compiler run also stalled; `-j 2` restored compiler progress.
Retain `full-final.log` and `/tmp/ggplot-temporal-actions-isolated.log` as incomplete
validation evidence. The aggregate retry in `full-retry.log` was superseded by the precision work above.

## Materialized positional palettes — 11 September 2026

`GgplotDiscretePosition::palette` now retains a numeric vector in trained-domain
order. This represents a reference palette returning a fixed vector; absent palette
retains one-based indices. Values can be reversed, uneven, repeated or missing. The
vector must cover the resolved domain, and unused tail values do not train its range.
The shared expansion owner combines mapped discrete limits with observed continuous
coordinates; the provider projects marks/guide keys through the same spacing kernel.
Duplicate guides use mapped coordinates rather than reconstructing ordinal indices.
Version **23** is required when a palette is retained; version 22 still suffices for
ordinary nullable positional policies. Numeric secondary formatter overrides also
reuse the existing numeric formatter and retained viewport.

The pinned R script `tools/reference/r/discrete-position-palettes.R` records **144
primary panels plus 116 nested secondary outcomes**. Three Rust tests compare direct
mapping/ranges and **520 primary/secondary band/point configurations**, plus four
replacement states against fresh batches with held-capture checks. Missing palette
coordinates can remove the discrete expanded range; a remaining constant continuous
range maps infinities to its midpoint, matching the reference. Too-short palettes,
empty unresolved ranges and invalid secondary ranges reject. Finite comparisons use
`2e-14` absolute tolerance; typed missing/infinite values are checked separately.

Fresh Python/WASM each pass **520 configurations**, with exact matching records and
**twelve byte-identical SVG/PDF/PNG publications**. Reverse/spread/repeated/nonfinite
samples and independent SVG/PDF renders were inspected. Repeated coordinates visibly
overlap their retained labels; this is recorded, not hidden by a baseline change.
Artifacts, inspection files and comparison hashes: `target/ggplot-discrete-palette-functions/`.

Final Darwin arm64 / Rust 1.97.1 / `a6caa39` plus working-tree evidence:

- Full core: **537 tests/doctests PASS**, zero failures/ignored
  (`/tmp/ggplot-discrete-palette-full.log`). Eleven focused positional/secondary tests
  pass (`/tmp/ggplot-discrete-palette-focused.log`).
- All-target core Clippy: PASS (`/tmp/ggplot-discrete-palette-clippy.log`).
- Actual host build/proof logs: `/tmp/ggplot-discrete-palette-{python,wasm}-build.log`
  and `/tmp/ggplot-discrete-palette-{python,wasm}.log`.
- Strict Python and TypeScript positional consumers: PASS
  (`/tmp/ggplot-discrete-palette-mypy.log`, `/tmp/ggplot-discrete-palette-tsc.log`).

A materialized vector does not execute a population-dependent R callback on update.
Such callback/registered-operation adaptations and the remaining scale inventory,
host replacement, native/Linux and cumulative package acceptance stay open. GG-04
and G-GGPLOT remain open; GG-05 owns complete guide composition/presentation.

## Discrete secondary guides — 11 September 2026

The existing secondary-axis authoring API now accepts identity duplicates of reference
band/point scales. The retained primary spacing owns index mapping and viewport bounds;
secondary guides independently select numeric/category breaks and labels, preserve null
identity and apply reference three-decimal normalized rounding. Authored primary breaks
inherit numeric labels unless primary labels/names were supplied. Nonidentity transforms,
unbounded primary ranges, recursive sources and layer bindings remain rejected.
No wire fields, host mapping engine or implicit inverse are added.

`discrete-secondary.R` records **176 pinned R panels**, including missing/empty
populations, authored category limits, translation, hidden/explicit labels, numeric and
category breaks, inherited primary breaks/labels and reference errors. The Rust test
checks **352 band/point configurations** through JSON, with `2e-14` absolute numeric
value/position tolerance. R's sampled inverse can perturb a zero break by about `3e-21`;
this does not change its label or rounded position. Nonfinite numeric break entries are
omitted before the finite `ScaleValue` authoring boundary in these proof adapters.
A second test checks eight ordinary band/point configurations across both orientations
and reversed ranges, including unavailable secondary inversion.

Fresh Python/WASM each pass **352 configurations** with exactly matching metadata and
**nine byte-identical SVG/PDF/PNG files**. Explicit missing-category candidates use the
object form `{"MissingCategory": null}`; ordinary strings retain literal category meaning.
Inherited, numeric and explicit-label samples and independent SVG/PDF renders were
inspected. Artifacts and comparison hashes: `target/ggplot-discrete-secondary/`.
The cumulative primary-authoring runner includes this proof; that entire runner was
not rerun in this slice.

On Darwin arm64 / Rust 1.97.1 / `a6caa39` plus working-tree changes:

- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/discrete-secondary.R`: PASS 176 panels.
- `mise exec -- cargo test -p chart-core --test ggplot_discrete_secondary --locked --target-dir target/ggplot-scale-qualification`: PASS two tests.
- Full core: **533 tests/doctests PASS**, zero failures/ignored; this run precedes only
  the additional orientation/direction test. Log: `/tmp/ggplot-discrete-secondary-full.log`.
- All-target core Clippy: PASS (`/tmp/ggplot-discrete-secondary-clippy.log`).
- Fresh host builds and actual proofs: `/tmp/ggplot-discrete-secondary-{python,wasm}-build.log`
  and `/tmp/ggplot-discrete-secondary-{python,wasm}.log`.

GG-04/G-GGPLOT remain open. Custom positional palettes and the remaining inventory
arguments, host replacement scenarios, native/Linux and cumulative acceptance remain.
Guide crowding and empty-panel presentation retain GG-05 ownership.

## Positional null identity, factors and limits — 11 September 2026

The core positional policy now retains null separately from the literal text `NA`.
`GgplotDiscretePosition` reuses the existing discrete-domain, guide and expansion
owners. `NullableCategorical` preserves layer-local ordinals through eligible domain
training and named-domain union. `ScaleValue::MissingCategory` carries identity through
projection, guides and inspection. The existing checked positional provider and spacing
kernel handle finite destinations; no host mapping engine or sentinel label is added.

`AxisBuilder::discrete_policy` and the actual Python/WASM methods expose factor levels,
drop/translation, nullable authored limits and guide selection. Wire version 22 is
required for retained policies or authored missing-category guide/annotation values;
downgraded envelopes reject. Automatic ggplot null categories require no new authored
field. Explicit D3 band/point axes still omit null positions, with a focused control.

The pinned `position-null-categories.R` fixture contains **576 actual R configurations**:
four populations, three character/factor representations, six limit policies and
both drop, translation and break modes. Its 24 asserted empty-input/empty-limit errors
are reproduced. Direct Rust policy checks compare exact typed domains/labels and
reference mappings/ranges, with `2e-14` absolute position tolerance. Primary Rust checks
cover **1,152 band/point configurations**, JSON round-trips and version rejection.
Six focused tests also cover reversed default axes, 16 replacement states against fresh
frames, immutable captures, and explicit D3 point/band controls. Replacement comparisons
preserve a common layer identity so full scene items and guides can be compared exactly.

Fresh Python and WASM each pass **1,152 configurations** using actual source point
primitives/targets and guide values. Records match exactly; six SVG/PDF/PNG files match
byte-for-byte across hosts. Both PNG samples and their independent SVG/PDF renders were
inspected: translating null gives four distinct positions for five source points
(two null points coincide); disabling translation gives three points. Literal and missing
`NA` retain separate guide identities even when their displayed labels match.

Executed on Darwin arm64, Rust 1.97.1, revision `a6caa39` plus working-tree changes:

- `mise exec -- cargo test -p chart-core --test ggplot_discrete_position --locked --target-dir target/ggplot-scale-qualification`: six tests PASS.
- Final full core: **531 tests/doctests PASS without exclusions**. All-target core
  Clippy, repository, formatting and diff checks PASS.
- Actual host builds use `target/axis-rust-proof` and `target/ggplot-positional-wasm`;
  wasm-bindgen CLI is the repository-local pinned 0.2.128 executable. The default
  0.2.122 CLI was rejected before generation; no dependency was changed.
- `scripts/bindings/ggplot_discrete_position.py` and `.cjs`: 1,152 records each PASS.
  Strict Python and TypeScript policy/reset consumers PASS. The aggregate primary
  runner includes these consumers; the whole aggregate runner was not rerun.
- Artifacts: `target/ggplot-position-null/comparison.json`, `python/`, `wasm/`,
  `inspection/`; logs `/tmp/ggplot-null-{final-focused,clippy-final,full-final,
  python-final-build,wasm-final-build,python-final-proof,wasm-final-proof,ts,mypy-final}.log`.

This completes the tested nullable positional policy slice, **not GG-04**. Nullable
navigation windows, numeric inputs to discrete axes,
remaining callbacks/scale arguments and broader geometry/facet contracts stay open.
The new replacement checks are Rust runtime evidence; host replacement acceptance for
this positional slice remains open. Native window/Linux acceptance was not rerun.
GG-05–19 retain their prerequisite and acceptance gates.

## Nullable positions with continuous limits — 11 September 2026

`GgplotDiscretePosition::train_with_continuous_limits` now passes authored numeric
category-index limits to the shared expansion owner. Primary band/point axes use this
path for nullable categories and explicit discrete policies. The limits change the
viewport without changing typed category identity or training; existing axis authoring
and wire version 22 remain sufficient.

The independent `position-null-continuous-limits.R` corpus adds **1,152 R panels**:
four populations, three level representations, automatic/nullable authored category
limits, both translation modes, eight continuous-limit policies and three expansions.
The combined corpus now has **1,728 direct records and 3,456 primary configurations**.
All six focused Rust tests pass with the existing `2e-14` absolute tolerance.

The new oracle records raw point-grob entry counts separately from entries with finite
coordinates: R retains three undefined entries for a mixed population under infinite
limits, while no points are drawable. The fixture was generated with 17-digit numeric
precision after default JSON rounding exceeded the unchanged tolerance for an off-panel
coordinate near -23. Neither mapping expectations nor test tolerances were relaxed.

Fresh actual Python/WASM runs each pass **3,456 configurations**; the saved runtime
records were also checked against the final full-precision fixture. All records match
exactly between hosts. **Nine SVG/PDF/PNG files** match byte-for-byte, including an
asymmetrically expanded axis with null first in the authored domain. Its PNG and
independently rendered SVG/PDF were inspected. Existing original samples were retained.
Artifacts: `target/ggplot-position-null/{python,wasm,inspection,comparison.json}`.
Logs: `/tmp/ggplot-null-limits-{reference,focused-final,python-build,wasm-build,
python-final,wasm-final,clippy,fmt,repository,diff}.log`. Actual host builds use the
same isolated targets and pinned wasm-bindgen 0.2.128 described above. The primary proof
runner includes both corpora and all nine file comparisons; its aggregate was not run.
No host declaration or wire change was needed for this extension. Final full core
validation passes **531 tests/doctests without exclusions**
(`/tmp/ggplot-null-limits-full-final.log`); all-target Clippy, formatting, repository
and diff checks pass.

Nullable navigation, minor candidates, numeric discrete inputs, host replacement and
remaining scale arguments stay open. GG-04 and GG-05–19 acceptance is unchanged.

## Nullable positional minor candidates — 11 September 2026

Explicit numeric minor candidates now use shared reference category spacing for
nullable band/point axes. The checked provider exposes an optional category-index
minor capability, rejects nonfinite provider destinations, and clips finite results
to its range. Providers without the capability reject explicit requests. Automatic
categorical minors remain empty. No new host authoring or wire field is required.

`position-null-minor-breaks.R` adds **192 actual R panels**, including nullable authored
limits, missing-category translation, finite/unbounded/contracted views and finite,
missing and infinite minor candidates. Combined positional qualification is now
**1,920 direct records and 3,840 primary configurations**. The primary Rust checks
compare exact minor values and positions at `2e-14` absolute tolerance. The new
provider boundary test also covers descending destinations and nonfinite outputs.

Fresh Python/WASM each pass **3,840 configurations**, including minor snapshot values
and positions at `1e-12` absolute tolerance; all complete records match exactly.
Nine publication files match between hosts and retain the hashes of the already
inspected samples. Full core passes **532 tests/doctests without exclusions**;
all-target core Clippy, formatting, repository and diff checks pass. Logs:
`/tmp/ggplot-null-minor-{reference,core,full,python-build,wasm-build,python,wasm,
clippy,fmt,repository,diff}.log`; artifacts remain in `target/ggplot-position-null/`.
The aggregate proof runner is updated but has not been rerun in full.

This qualifies minor selection/transport, not GG-05 minor painting or full guide
composition. Next, reconcile remaining GG-04 arguments against the pinned inventory;
the discrete secondary-axis argument is a confirmed uncovered reference behavior.
Nullable navigation, numeric discrete inputs, host replacements and cumulative
platform/feature acceptance remain open.

## Implemented and checked

- Untrained discrete colour lookup now uses the reference's lazy two-slot `[0,1]`
  palette without adding guide entries or retained domain keys. Named manual palettes
  and trained empty populations retain their separate behavior. A short manual palette
  can prepare an empty chart; only a subsequent lookup requests two colours and reports
  insufficient values. Numeric/logical queries can match text levels under the ggplot
  discrete policy; direct typed lookup and other profiles retain their precedence.
  `untrained-discrete-lookups.R` supplies **64 direct colour lookup cases** and **32
  actual empty R charts**, covering hue/short/manual/named palettes, factor levels,
  missing translation and explicit breaks. Rust reproduces the old transparent output
  for text `0` where R returns `#F8766D`, then passes the matrix and primary empty charts.
  The two-slot palette is evaluated only for queries against a never-trained population;
  ordinary mark sampling keeps its prepared palette. No new wire state is introduced.
  All **525 core tests/doctests without exclusions** and all-target Clippy pass using
  `target/ggplot-scale-qualification`. Fresh Python/WASM each retain **874 exactly
  matching records**; eighteen SVG/PDF/PNG files are identical to each other and the
  previously inspected identity/null publications. This is host regression evidence;
  direct untrained scale lookups are qualified through Rust, not a new standalone host API.
  Repository/format/diff checks pass. Artifacts: `target/ggplot-untrained/comparison.json`;
  logs `/tmp/ggplot-untrained-{full-core,clippy,python-build,wasm-build,python-test,
  wasm-test,repository,fmt,diff}.log`, plus
  `/tmp/ggplot-untrained-discrete-{reference,before,focused,final-focused}.log`.
  Positional missing-category identity and the remaining GG-04 arguments stay open.

- Nullable identity colours now distinguish a missing source key from the literal
  R colour `NA`. Missing keys remove point marks and inspection targets through the
  existing ggplot missing-aesthetic stage; transparent literal colours remain marks.
  Position training retains both contributions. `identity-null-paints.R` supplies
  **512 valid-colour configurations**, including actual R point-grob counts, nullable
  limits/factor levels, drop/translation controls, hidden/visible guides and four
  populations. Primary Rust JSON round-trips match the oracle. Twelve focused tests
  across `ggplot_discrete_null` and `ggplot_identity` pass using
  `--target-dir target/axis-rust-proof`; the earlier default-target run was interrupted
  while compiling and is not passing evidence. Fresh Python/WASM each pass **874
  identical records**: 256 hue and 512 identity configurations, 32 immutable replacement
  states versus fresh batches, 72 position panels and two retained-state wire cases.
  All eighteen SVG/PDF/PNG files match across hosts. The new identity sample has been
  inspected in all three formats with embedded PDF fonts; the fifteen earlier inspected
  publications are unchanged byte-for-byte. No new public authoring API or wire version
  was introduced. Artifacts: `target/ggplot-identity-null/comparison.json` and `inspection/`;
  logs `/tmp/ggplot-identity-null-{reference,focused-final,python-build,wasm-build,
  python-test-final,wasm-test-final,repository,fmt,diff}.log`. Repository/format/diff
  checks pass. The full 523-test run below predates this identity correction; this slice
  uses focused runtime tests. All-target Clippy passes from the fresh
  `target/ggplot-scale-qualification` directory (`/tmp/ggplot-identity-null-clippy-clean.log`).
  Interrupted runs in older target directories were stalled in compiler dependency-directory
  enumeration; a one-second stack sample established that boundary. Numeric identity
  colours and complete missing-fill/stroke geometry behavior are not newly qualified.
  Untrained discrete fallback lookups and positional missing-category identity remain
  GG-04 work; GG-04 and G-GGPLOT remain open.

- Nullable discrete domains now preserve explicitly authored or retained factor
  missing-level slots without consuming palette colors. A missing key remains
  distinct from the text `NA`. **512 direct R records** cover hue and identity
  domains, nullable limits, factor/drop controls, missing translation and four
  populations. Empty factor training skips its levels; an untrained zero-row
  population differs from a trained empty palette. Missing paint with translation
  disabled is suppressed, except where the reference's empty-palette early return
  uses `na.value`. **256 primary hue cases** verify marks and legend metadata through
  JSON. Identity nullable behavior here is direct typed-scale evidence; its complete
  primary paint acceptance is not claimed.
  Inspection then reproduced a positional range regression: missing-color removal
  shrank x from `[0,3]` to `[0,1]`. The compiler now retains finite position training
  before paint removal. **72 actual R panels** verify both numeric/category axes,
  mapped coordinates, guide positions/labels and omitted marks. Category training
  retains checked source ordinals; no invisible marks or inspection targets are added.
  The optional retained-category pointer costs eight bytes per prepared layer on this
  64-bit target, with allocation only when missing paint suppresses category positions.
  Fresh Python/WASM each pass **346 exactly matching records**: 256 hue cases,
  sixteen replacement states against fresh batches, 72 actual scene panels and two
  wire cases. Retained `empty_population=true` requires version 21 and rejects a
  downgrade to 20. False is omitted, preserving ordinary version 17 policies.
  Five SVG/PDF/PNG samples are byte-identical across hosts; all fifteen files were
  visually inspected via PNG and independent SVG/PDF rendering, with PDF fonts embedded.
  Artifacts: `target/ggplot-discrete-null/comparison.json`, `publication-checks.json`
  and `inspection/`. Final **523 core tests/doctests without exclusions** and all-target
  Clippy pass, as do repository/format/diff checks. Final logs use
  `/tmp/ggplot-discrete-null-{core-final,final-focused,clippy-qualified,python-build-final,
  python-test-final,wasm-build-final,wasm-test-final,repository-final,fmt-final,diff-final}.log`.
  Producers: `tools/reference/r/discrete-null-domains.R` and `missing-paint-positions.R`.
  The full primary aggregate runner, Linux and a native window were not rerun.
  Complete identity paint, untrained fallback lookup, positional missing-category
  identity and GG-05 guide presentation remain open.

- Automatic character position domains now sort using the pinned reference's C
  collation only for the ggplot2 profile. Explicit domain order and retained source
  ordinals are unchanged. The live Python counterexample placed `c` at `0.1875`
  instead of R's `0.8125`; six R records now pass 30 actual panels across auto,
  band and point axes, both range directions and explicit-domain controls.
  The expanded actual Python/WASM proof passes **3,582 exactly matching records**;
  twelve SVG/PDF/PNG files match across hosts and the previously inspected outputs
  byte-for-byte. `target/ggplot-discrete-order/comparison.json` records this check.
  All **four focused Rust tests**, all-target core Clippy and repository/format/diff
  checks pass. Logs use `/tmp/ggplot-discrete-order-{focused,python-build,python-test,
  wasm-build,wasm-test,clippy,repository,fmt,diff}.log`. The complete core suite was
  not repeated for this ordering correction; its 518-test result below predates it.
  Source: `tools/reference/r/discrete-order.R`; fixture: `discrete-order.json`.
  Missing category identity and nullable authored domains remain open investigations.

- Discrete `continuous_limits` authoring now passes **756 direct R records** and
  **3,528 primary configurations**, including expected rejection cases, plus 24
  replacement states compared with fresh batches. Finite vectors reduce to their
  range; nullable vectors require two endpoints; empty and infinite vectors retain
  reference exceptional ranges. Boolean host inputs normalize into canonical numbers.
  Observed category positions participate in expansion, including the distinction
  between all-excluded observations and zero source rows. Band/point spacing shares
  finite and unbounded projection, omits undefined points and disables unavailable
  category lookup. JSON version 20 retains this policy and rejects downgrades.
  Fresh Python/WASM each pass **3,552 records**, exactly equal across hosts, including
  immutable retained frames and replacement versus fresh batches. Four PNGs are
  byte-identical. SVG/PDF/PNG samples were visually inspected, with embedded PDF fonts.
  Artifacts: `target/ggplot-discrete-limits/comparison.json`, `publication-checks.json`
  and `inspection/`. Reversed destinations retain legacy major ordering; reference
  label-to-position associations are checked independently. Coincident infinite-limit
  labels overlap and empty plots retain the current NoData presentation; full GG-05
  presentation equivalence is not claimed.
  Final **518 core tests/doctests without exclusions**, all-target Clippy, strict
  Python/TypeScript declarations and repository/format/diff checks pass on Darwin
  arm64. Logs: `/tmp/ggplot-discrete-limits-core.log`, `-clippy.log`, `-python-test.log`,
  `-wasm-test.log`, `-python-types.log`, `-typescript.log` and corresponding check logs.
  Strict core rustdoc also passes after correcting literal range notation in three
  comments (`/tmp/ggplot-discrete-limits-rustdoc.log`).
  The aggregate primary proof runner, Linux and a native window were not rerun for
  this slice. Remaining scale arguments and cumulative acceptance stay open.

- Minor-break selection now covers automatic numeric/temporal subdivision,
  suppression, explicit numeric and nullable timestamp candidates, and time-width
  strings through the common `MinorBreaks` policy. The shared rule passes 112 direct
  numeric records and 448 primary numeric panels, including unsorted major values,
  missing/infinite candidates, wide expansion and reverse/log/square-root transforms.
  Temporal checks cover 30 width panels (four expected errors), 35 automatic panels
  and 25 explicit timestamp panels. Thirty-six discrete R records pass through
  216 primary configurations across auto/band/point scales and both range directions.
  Discrete automatic minors are empty; numeric minors use the shared category-index
  spacing and expanded-range censoring. Explicit timestamp order and fractional
  Date minors are preserved; missing values are omitted. The ggplot2 profile floors
  authored Date **major** candidates to UTC days, including before the epoch, while
  the legacy Date factory retains its existing source-value contract.
  `MinorGuideTick` keeps an optional raw value and a finite position: square-root
  expansion can lack a real inverse, and half-nanosecond positions have no exact raw
  timestamp. Four precision cases verify exact epoch origins and source-unit
  promotion; three invalid-input cases verify type, unit and resource rejection.
  Rust/Python/WASM authoring and JSON use version 19 for explicit minor policies.
  Dense time-width proofs explicitly select a 4096-tick budget; defaults are unchanged.
  Fresh Python and WASM each pass all **761 records**. Structure, strings and missing
  values agree exactly; seven binary64 fields differ by at most
  `1.7763568394002505e-15`, within the existing `1e-11` reference tolerance.
  `target/ggplot-minor/comparison.json` records the comparison. Final core validation
  passes **514 tests/doctests without exclusions**, all-target Clippy, strict Python
  and TypeScript consumers, repository checks, formatting and diff checks.
  Evidence: `/tmp/ggplot-minor-core-all.log`, `-clippy-all.log`, `-python-test.log`,
  `-wasm-test.log`, `-python-types.log`, `-typescript.log` and the repository/format
  check logs sharing that prefix.
  The primary proof runner now includes these cases and declarations; its complete
  aggregate run is not claimed. This slice qualifies selection metadata, with no
  claim of minor painting/animation or complete GG-05 guide composition. Arbitrary
  minor callbacks, remaining scale arguments and cumulative package gates stay open.

- Positional binned scales pass 2,400 direct R pre/post-statistic records,
  all 960 primary panels (including infinite limits), 96 grouped-statistic/filter
  cases, live replacement versus fresh batches, and nested v18 projection checks.
  The integration captures source cuts before statistics and reference reset limits
  before mapping generated indices into intervals. Automatic panel labels format
  the retained panel breaks. Replacement covers four transforms and four populations,
  including empty/constant input; original definitions/captures remain unchanged.
  Rust/Python/WASM `scale_binned` uses the same core owner and wire version 18.
  Fresh Python and WASM each pass all 960 panels with identical records and four
  byte-identical PNGs. The identity, square-root, reversed and unbounded SVG/PDF/PNG
  samples were inspected; finite scene geometry is retained. The unbounded sample
  has no points and uses the current renderer's No data placeholder; this is not
  a claim of complete ggplot2 presentation equivalence. Artifacts are under
  `target/ggplot-positional/{python,wasm,rendered}`. Full core validation passes 507
  tests/doctests with no exclusions, all seven focused tests pass with the final
  drawable-coordinate/inverse assertions, and all-target core Clippy passes.
  Logs: `/tmp/ggplot-positional-{core-final,final-test,clippy,python-test,wasm-test}.log`.
  Strict mypy and TypeScript consumers also pass; the primary authoring proof runner
  includes the positional matrix, host comparisons and declaration checks.
  `mise exec -- python3 scripts/check_repository.py` and `git diff --check` pass.
  Unbounded ranges use the shared rescaler and omit undefined positions; they do
  not introduce nonfinite destination geometry or claim a finite inverse. The
  R oracle retains both panel candidates and drawable guide positions, plus final
  coordinate positions. This corrects the earlier assumption that these infinite
  limits anchor points at the panel edge: R actually produces NaN coordinates.
  Free-facet populations, finite viewports over unbounded bin geometry, remaining
  scale arguments and cumulative destination acceptance stay open. GG-04 and
  G-GGPLOT remain open.

- Primary authored-limit mapping now matches 2,016 actual R chart builds with
  guides suppressed, including 762 expected mapping failures. The same oracle retains
  default-guide construction outcomes, including 408 failures after successful hidden-
  guide mapping; these remain GG-05 acceptance evidence, not a GG-04 rendering pass.
  Primary JSON construction uses the common mapped-scale engine. R's hidden binned
  guide maps from the trained limits captured before automatic break extension; a
  visible guide selects its breaks first. Training now preserves that distinction
  without a new wire field or mutable cache. Empty hidden-guide layers prepare without
  sampling unavailable bins; an actual sampling request still rejects. Population
  retraining replaces earlier state and matches independent batch training for four
  target populations, with the original descriptor unchanged. The six degenerate/
  authored-limit tests pass in `/tmp/ggplot-authored-primary-test.log`.

- Fractional binned counts now pass 288 pinned R records across four transforms,
  three domains, authored/automatic limits and equal/nice selectors. Desired counts
  use binary64 in the existing enum, preserving integer JSON inputs. Equal selection
  uses R's rounded-up `seq(length.out = n.breaks + 2)` length; nice selection delegates
  the fractional desired count to the shared log/extended selector. Nice counts below
  one are rejected rather than running R's potentially nonterminating search. Count,
  result and sampling budgets remain bounded. The nine binned and six degenerate tests
  pass (`/tmp/ggplot-fractional-tests.log`), including invalid/fractional budget cases
  and empty hidden-guide compile-versus-sample behavior. The combined change passes
  all 499 core tests/doctests (`target/ggplot-primary-fractional-full-core.log`) and
  all-target Clippy (`/tmp/ggplot-primary-fractional-clippy.log`). A subsequent
  primary authoring regression caught break selection on a placeholder domain before
  actual population training. Named color-scale structural validation now compiles
  the range and normalizer and checks budgets without selecting bins. The constant
  `[4,4]` population with desired nice count 1.1 prepares the reference colors/labels;
  the actual nonconstant `[1,10]` population still rejects. Sixteen focused tests and
  fresh all-target Clippy pass (`/tmp/ggplot-primary-structure-{tests,clippy}.log`);
  all 500 core tests/doctests pass on this final structural-validation change
  (`target/ggplot-primary-structure-full-core.log`, zero failures/ignored tests).

  Fresh Python and WASM modules each pass 2,016 primary authored-limit cases and 288
  fractional-bin cases (2,304 records, including 774 expected failures). Every host
  record agrees, and three PNG samples are byte-identical. The corresponding PNG,
  SVG and PDF renders have been inspected for colors, marks, labels and clipping.
  Binned publication still uses the existing interval swatches; complete stepped
  guide construction and the retained R failures remain GG-05. Builds, proofs and
  artifacts are under `target/ggplot-authored-scales`; the actual scripts are
  `scripts/bindings/ggplot_scale_limits.py` and `.cjs`. Both are included in the
  primary authoring proof runner; its aggregate invocation was not run. Other host
  paths, native/Linux, remaining arguments and complete GG-04 acceptance remain open.

- Authored-limit population extension is in qualification. The isolated R generator
  `tools/reference/r/authored-limit-populations.R` records 2,688 cases across four
  policies, four transforms, four populations, 21 limit configurations and automatic/
  explicit breaks. All current reference guide candidates/labels/visibility, mapping
  values/rejections and JSON round-trips pass the focused `authored_limits` test in
  `ggplot_degenerate_mapping` (`/tmp/ggplot-authored-limits-test.log`). This is new
  code after the 494-test qualification below. All 120 tests across the 26 related
  ggplot test groups pass (`/tmp/ggplot-authored-limits-related.log`); all 495 core
  tests/doctests pass (`target/ggplot-authored-limits-full-core.log`) and all-target
  core Clippy passes (`/tmp/ggplot-authored-limits-clippy.log`). The extension permits exceptional authored limits, preserves partial-limit
  orientation and ordered OOB operations, shares finite/nonfinite break selection,
  and uses the reference log base for both guides and normalization. A compiled
  transformed-bound override represents empty/nonfinite populations without losing
  exceptional endpoints through a raw inverse. Square-root inversion rejects negative
  transformed values. Binned selection now shares one transformed-limit algorithm
  across populations, with metadata retained for mappings that cannot sample.
  The primary-chart and population-retraining extension above adds newer code. Actual
  hosts/destinations and remaining GG-04 arguments remain to be qualified; no package
  or cumulative gate is closed.

- Zero logarithmic and constant-domain mapping now pass 256 pinned R guide/mapping
  records. Binned cuts deduplicate in transformed data space before normalization;
  distinct cuts collapsing to one normalized position reject sampling, matching R.
  A non-sampleable prepared state retains metadata when no comparable normalized
  cut exists. Constant `[0,0]` logarithmic breaks return zero through the common
  log selector; nonconstant ranges starting at zero still reject automatic selection.
  No alternate mapping engine is added. The reference covers four transforms,
  continuous/identity/nice-binned/equal-binned policies, four domains and four break
  modes. R's scalar constant-bin result is compared using its layer recycling rule.
  `tools/reference/r/degenerate-bin-mapping.R` generated
  `fixtures/parity/ggplot2/degenerate-bin-mapping.json` with the isolated R runner.
  Three focused tests check all 256 records, 192 primary color charts with exact
  byte paints and errors, JSON round-trips, and missing-color defaults/overrides.
  These join ten related passing tests in `/tmp/ggplot-degenerate-test.log`.
  The named `color_mapped` builder now uses R's RGB 127 `grey50` default when the
  descriptor selects reference semantics, matching `ggplot_color_default`.
  A reference palette alone retains the independent builder default, and explicit
  missing-color overrides survive JSON. The earlier 24 primary population tests
  now compare the correct RGB 127 output; the immutable R `grey50` records did not
  change. Full-core validation passes 494 tests/doctests, zero failed/ignored, in
  `target/ggplot-degenerate-full-core.log` with
  `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`.
  Core all-target Clippy passes in `/tmp/ggplot-degenerate-clippy.log` with the
  `target/axis-rust-proof` target directory. Fresh host/destination
  acceptance, nonfinite authored limits and remaining GG-04 arguments stay open.

- Transformed missing-limit handling extends across continuous, numeric identity,
  and nice/equal binned guides. One shared endpoint filter classifies NaN transforms
  as omitted limits without altering the authored descriptor. Empty/nonfinite guide
  population checks use those effective limits. Binned metadata can remain available
  when mapping is invalid: a compiled sampling flag rejects incomplete nonfinite
  mapping for numeric/paint output, matching R's invalid-break failure. Fully valid
  authored endpoints still permit mapping. No new wire fields or per-mark transform
  reconstruction are added.
  `tools/reference/r/transformed-missing-limits.R` records 320 scale cases across
  square-root/log10, finite/empty/missing/infinite populations, five partial/invalid
  limit configurations and automatic/explicit breaks. It also records 24 actual
  `ggplot_build`/`ggplot_gtable` cases, including valid-limit controls, missing-color
  paints and guide presence. Both matrices, JSON round-trips and primary retraining
  pass in `ggplot_missing_limits`; 23 related tests pass with
  `CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --test ggplot_missing_limits --test ggplot_binned_guides --test ggplot_continuous_guides --test ggplot_identity --locked`
  (`/tmp/ggplot-missing-limits-test.log`). Full-core validation passes 491
  tests/doctests with zero failed/ignored in `target/ggplot-missing-limits-full-core.log`
  using `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`.
  Core all-target Clippy passes (`/tmp/ggplot-missing-limits-clippy.log`), as do
  formatting, repository and diff checks.
  Zero logarithmic endpoints, nonfinite authored limits, fresh host/destination
  acceptance and remaining GG-04 arguments stay open.

- Binned scale boundaries now pass 228 pinned R records covering integer counts
  zero/one/two/three/five, constant and nonconstant domains, authored/automatic
  limits, and negative square-root/logarithmic limits with a finite population.
  NaN transformed limits use the corresponding trained endpoint while preserving
  the authored limit descriptor and suppression of automatic extension. Structural
  validation no longer runs a break search on a fabricated empty population;
  selection uses the actual population. The shared extended-break owner accepts
  count one in its degenerate-range sequence branch and continues rejecting one
  for an ordinary nonconstant search. Zero nice counts and excessive counts reject
  under the bounded portable contract; the oracle does not invoke the reference
  zero-count nice search because it can fail to terminate. Fractional binned counts
  remain outside the existing integer descriptor.
  `tools/reference/r/binned-boundaries.R` generated
  `fixtures/parity/ggplot2/binned-boundaries.json` with the isolated R runner.
  Eight focused tests pass, including all 228 break/label/visibility/transformed-limit
  records, primary chart/JSON handling, unchanged paints versus omitted limits,
  retained original limits, retraining and count budgets. Evidence:
  `CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --test ggplot_binned_guides --test ggplot_breaks --locked`
  (`/tmp/ggplot-binned-boundaries-test.log`). Full-core validation passes 489
  tests/doctests, zero failed/ignored, in `target/ggplot-binned-boundaries-full-core.log`
  using `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`.
  All-target core Clippy passes (`/tmp/ggplot-binned-boundaries-clippy.log`), as do
  formatting and repository checks. These are core checks, not fresh host acceptance.
  Empty/nonfinite invalid-limit populations are covered by the subsequent extension
  above. Logarithmic zero limits, fractional counts, fresh hosts/destinations and full
  acceptance remain open.

- Explicit `GgplotTimeFormat` / `GuideFormatter::GgplotTime` compiles R date/time
  patterns onto the shared calendar/token renderer. The supplied C-locale default,
  aliases, ISO/week fields, UTC name/numeric offset, fractional `%OS`, escaped
  percent signs, literal unknown fields and multiline labels are distinct from D3
  patterns. Reference labels reproduce binary64 seconds and first-only `%OS`
  precision capped at six; selected source timestamps remain exact. Local `%Z`
  requires abbreviation resources not yet represented by the calendar descriptor,
  and `%s` depends on R process timezone; these return explicit unsupported errors.
  This is a bounded R C-locale pattern profile, not universal libc/locale parity.
  `time-formats.R` produces 220 independent R labels under R 4.6.1, LC_TIME=C,
  digits.secs=0 and TZ=UTC. Source microseconds are serialized as decimal strings
  because JSON numeric output rounded a 16-digit source identity during initial
  fixture validation; the labels and source inputs were not changed. All 220 label
  bytes, primary guide timestamp/label/JSON round-trips, supplied locale/offsets,
  unsupported patterns, byte bounds and retained D3 semantics pass. The duration
  extension preserves original binary64 seconds in the same renderer, avoiding
  integer nanosecond resampling and its range limit. `duration-formats.R` records
  40 actual hms scale panels with negative/fractional durations, large epochs,
  aliases and escaped directives. Five tests in `ggplot_time_formats` cover all
  260 reference records, primary values and wire v17 round-trips, all four axis
  sides, and a supplied timezone window with a subsecond transition.
  Multiline plain labels are measured and painted as separate lines; the complete
  block stays outside the axis. Existing per-line font and output budgets apply.
  This qualifies this label path, not full GG-14 typography equivalence.

  Fresh Python and WASM builds pass 246 date/duration publication cases each.
  Fourteen tab-containing cases explicitly assert `CHART_MISSING_RESOURCE` because
  the supplied font has no tab glyph; their R label bytes still pass core tests.
  All 260 host records match. Three representative PNGs are byte-identical between
  hosts; their SVG, PDF and PNG outputs are inspected, including the final multiline
  block placement. Artifacts and build/proof/render logs are under
  `target/ggplot-formats/{python,wasm}` and `target/ggplot-formats/*.log`.
  Strict Python and TypeScript authoring checks pass. The actual host proofs run
  `scripts/bindings/ggplot_formats.py` and `.cjs` using newly built extension-proof
  adapters and wasm-bindgen CLI 0.2.128. The primary proof runner includes these
  tests; its aggregate invocation was not run for this slice.
  Final validation on Darwin arm64 / Rust 1.97.1:
  `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`
  passes 487 tests/doctests, zero failed/ignored, across 90 groups
  (`target/ggplot-formats-qualified-core.log`).
  `CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings`
  passes (`/tmp/ggplot-formats-qualified-clippy.log`). Remaining scale arguments,
  other GG-04 host paths, native/Linux and complete acceptance remain open.


- `GuideTickArguments.width` accepts reference decimal width strings through the
  primary axis and shared guide path. Parsing resolves into existing elapsed/calendar
  selectors, preserving singular/plural units, fractional multipliers, reference
  integer sequence truncation and the different handling of trailing fields.
  Fractional minute alignment uses the shared integer lattice with rounded alignment
  and a separate step; Date scales floor aligned day candidates before censoring.
  Durations share one unit-to-seconds conversion with structured widths. The literal
  remains in the authored definition and round-trips through wire v17. Python and
  WASM declarations include the optional string; runtime host qualification remains open.
  `time-width-strings.R` records 135 actual R Date/datetime/duration panels and
  rejection cases. Exact tick identities/labels and primary JSON checks pass. An
  additional test retains a nanosecond origin above 2^53, verifies exact projected
  ticks, bounded generation, below-quantum rejection and explicit empty selection.
  All 482 core tests/doctests pass in `target/ggplot-width-strings-full-core.log`;
  core all-target Clippy, repository, formatting and diff checks pass. Export tests
  compile in `/tmp/ggplot-width-strings-export-check.log`; this is compilation only.
  Logs for the other final checks use `/tmp/ggplot-width-strings-*.log`.
  Full R numeric-string syntax, nonpositive-width corner cases, widths finer than
  source precision, remaining arguments and fresh host/destination acceptance are
  not certified by this matrix. Existing shared D3 formatting semantics are retained.

- Binned population metadata now distinguishes zero rows and all-nonfinite rows from
  finite fallback mapping. A further 192 pinned R cases cover three populations,
  four transforms, absent/partial/full limits and nice/equal/explicit/empty breaks.
  They preserve empty partial-limit errors, infinite and missing candidates,
  logarithmic equal cuts with one infinite transformed endpoint, labels and censoring.
  Equal cut generation remains one shared helper. JSON, retraining and budget checks
  pass, and primary empty binned charts suppress absent-limit guides while rejecting
  incomplete limits. The generator is `tools/reference/r/binned-populations.R`, run
  with the pinned R runner. All 480 core tests/doctests pass in
  `target/ggplot-binned-population-full-core.log`; final core all-target Clippy,
  repository, formatting and diff checks pass with logs under
  `/tmp/ggplot-binned-population-{clippy,repository,fmt-check}.log`.
  The population matrix uses default count five and finite authored limit values;
  invalid transformed limits, other count boundaries, complete guide presentation,
  remaining scale arguments and fresh host/destination acceptance remain open.

- Binned cuts and labels now use `GgplotScaleGuide::Binned` and the existing binned
  population policy. The resolver preserves explicit order, duplicates and outside
  candidates without applying the continuous constant-domain override. Both families
  share vector-wide label formatting, length validation and resource guards.
  Prepared `ColorLegend.numeric_breaks` retains the exact candidates for downstream
  composition, while continuous legends reuse this vector when building entries.
  Empty binned break selections suppress guides. Primary JSON checks preserve labels,
  candidate order and unchanged marks, and verify guide colors against the same mapping.
  A pinned 288-record R matrix found and now verifies square-root endpoint round-trips
  and constant-reverse limit ordering; the shared binned policy fixes both. Independent
  R palette samples additionally verify five square-root boundary colors and two
  constant-reverse colors in `ggplot_binned_guides.rs`. The reference command trains
  `scale_colour_steps(transform=tr)` with `s$train(s$transform(x))`, calls
  `s$get_breaks()`, then `s$map(s$transform(x))`, for `x=c(1,2.5,5,7.5,10)` with
  `tr="sqrt"` and `x=c(4,4)` with `tr="reverse"`.
  Full core validation passes 477 tests/doctests in
  `target/ggplot-binned-guide-full-core.log`; the subsequently added independent
  palette test passes with all three focused tests in
  `/tmp/ggplot-binned-guide-edge-colors.log`. Final all-target core Clippy, repository,
  formatting and diff checks pass. Export tests compile in
  `/tmp/ggplot-binned-guide-export-check.log`; this is compilation only.
  Actual color-step presentation, `show.limits` composition, binned empty/nonfinite
  populations, remaining arguments and fresh host/destination acceptance remain open.

- Nonempty populations with no finite transformed observations retain their guide
  state independently of finite fallback mapping limits. Continuous and numeric
  identity guides match 192 pinned R records across missing/infinite populations,
  identity/sqrt/log/reverse transforms, absent/partial/full limits and automatic/
  explicit/empty breaks. Candidates retain vector-wide labels and use the original
  reference censor bounds; the logarithmic lower-only automatic-break error is
  preserved. JSON and populated/nonfinite/repopulated batch checks pass, as do tick
  and label budget guards before censoring. A primary all-missing-color chart keeps
  both marks and suppresses automatic and explicit guides without changing marks.
  The earlier zero-row identity retraining cases now explicitly use eligible training.
  All 475 core tests/doctests pass in `target/ggplot-nonfinite-full-core.log`.
  Core all-target Clippy, repository, formatting and diff checks pass; logs are
  `/tmp/ggplot-nonfinite-{clippy,repository,fmt-check}.log`. The oracle generator is
  `tools/reference/r/nonfinite-continuous-guides.R`, run through the pinned R runner.
  This covers the stated transforms with default counts and finite authored limits;
  invalid transformed-limit edge cases, remaining scale arguments, binned guide
  composition and fresh host/destination acceptance remain open.

- Zero-row numeric identity guides now retain whether training saw any rows, separately
  from their finite extent. Empty and untrained identity guides suppress fallback ticks
  unless both limits are authored; explicit trained finite extents remain usable.
  The continuous empty-population oracle now includes identity scales: eighteen
  linear reference cases cover automatic/explicit/empty breaks and absent/partial/full
  limits. JSON round-trips and populated-to-empty-to-populated training match in both
  families. All 473 core tests/doctests pass in
  `target/ggplot-empty-identity-full-core.log`; all-target core Clippy, repository
  checks, formatting and diff checks pass, with logs under
  `/tmp/ggplot-empty-identity-{clippy,repository,fmt-check}.log`.
  The later nonfinite-population slice above qualifies the distinct candidate rules.
  Fresh host/destination execution and transformed empty-limit edge cases are open.

- Zero-row continuous population training now retains `empty_population` in the
  existing boxed scale policy. Guide selection does not mistake the mapping fallback
  domain for observations; two fully authored limits still generate reference guides.
  Nine pinned R cases in `empty-continuous-guides.json` check absent/partial/full
  limits with automatic/explicit/empty breaks. Rust checks also cover serialization,
  populated-to-empty-to-populated replacement and primary zero-row color preparation.
  This change does not certify all-missing populations or empty numeric identity
  scales: R retains a distinct non-finite range for nonempty all-missing populations,
  and partially authored limits require separate handling. Those cases remain open.
  Final `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p
  chart-core --locked` passes 473 tests/doctests in
  `target/ggplot-empty-guide-full-core.log`. All-target core Clippy passes in
  `/tmp/ggplot-empty-guide-clippy.log`; repository and formatting checks also pass.


- Continuous aesthetic guides now select reference automatic or explicit candidates
  and format the full source-value vector before testing guide visibility. Linear,
  square-root, logarithmic and reverse transforms reuse the existing arithmetic,
  extended/log break selectors and numeric formatter. The checked inverse API is
  unchanged; a crate-private raw inverse serves reference exceptional candidates.
  The 160 records in `continuous-guides.json` cover outside/missing/infinite values,
  constant domains (including empty authored break vectors), hidden/explicit labels,
  desired counts and label-length errors. Three focused tests additionally verify
  primary logarithmic guide labels and mark colors, immutable marks through guide
  edits and JSON round-trips, full guide suppression, numeric identity guides and
  tick/label resource guards. `MappedScaleSpec.guide` now contains the shared
  `GgplotScaleGuide` discriminant (Hidden, Discrete or Continuous) in the same optional
  box; both families share `GgplotGuideLabels` validation. Palette-only D3 mappings
  retain their previous default guide policy unless explicitly opted into a reference
  guide or scale policy. Primary preparation checks the actual selected guide size.
  Existing discrete and identity tests pass after integration. Final
  `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core
  --locked` passes 471 tests/doctests in `target/ggplot-continuous-guide-full-core.log`.
  Core all-target Clippy, repository checks, formatting and diff checks pass.
  Final `cargo check -p chart-export --tests --locked` also passes; this is
  compilation evidence only, in `/tmp/ggplot-continuous-guide-export-check.log`.
  These are candidate/scale and primary preparation checks; colorbars, multi-aesthetic
  guide composition, continuous date/time labels, minor breaks, arbitrary callbacks,
  mathematical labels and fresh host/destination acceptance remain open.

- Discrete aesthetic guide arguments now retain ordered explicit breaks, optional
  break names, automatic/hidden/positional/named labels and empty key selections in
  the primary mapped-scale descriptor. The shared resolver intersects the prepared
  domain, deduplicates breaks at their first occurrence and retains original label
  positions; named labels use the reference last replacement. Default reference
  boolean and missing-category labels use R spellings. Sixty pinned hue/manual/
  identity scale records validate selection and label/rejection rules. Thirty more
  manual-scale records validate implicit palette naming from explicit breaks,
  duplicate-name first matching, explicit limits and insufficient-value errors.
  These repairs preserve named-palette domain restriction separately from names
  inferred from breaks. Four focused tests additionally verify primary JSON
  round-trips, guide colors sampled from marks, unchanged complete marks for hue
  guide edits, exact typed key retention and resource guards. The existing five
  scale tests also pass. Final `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec
  -- cargo test -p chart-core --locked` passes 468 tests/doctests; log:
  `target/ggplot-discrete-guide-full-core.log`. Core all-target Clippy, repository
  checks, formatting and diff checks pass. `cargo check -p chart-export --tests
  --locked` passes after the descriptor addition; this is compilation only.
  An earlier overlapping build invalidated two doctest dependency paths; the final
  isolated-target run above passes all doctests. Reference fixtures are `discrete-guides.json` and
  `manual-guide-mapping.json`; generators are under `tools/reference/r/`.
  This is scale-level selection evidence, not complete guide composition, R callback
  arguments, continuous guide controls, mathematical labels or actual host/export
  presentation acceptance. Numeric category text beyond the tested exact values
  remains unqualified. The optional boxed guide adds one pointer per scale
  descriptor and no per-mark fields; ADR 014 records the localized lint expectation.

- Numeric and discrete identity descriptors now preserve raw aesthetic outputs,
  with transformation preceding numeric identity mapping. The existing positional
  transform arithmetic and discrete factor-domain helper remain the shared owners.
  Guide training, limits, factor/drop and missing-level controls are separate from
  mapping; all observed colour values are pooled even when limits exclude them.
  Default identity colour guides are hidden. Seven focused tests pass 26 base R
  records and 96 limit/factor controls, including NA versus NaN, infinities,
  untrained partial-limit fallback, reverse/sqrt/log transforms, and unseen discrete
  values. Primary colour and linewidth charts pass v17 serialization and actual
  prepared-mark checks; colours preserve R names and transparent RGB channels and
  linewidth remains unchanged outside guide limits. All-target core Clippy passes.
  The alpha extension passes 24 point-grob paints and 168 further after-scale
  arithmetic paints. Reference nonpositional field reads retain IEEE inputs;
  reference alpha stays raw through scalar arithmetic/reductions, then finite
  coverage saturates, either infinity becomes zero, and NA/NaN retains the original
  paint alpha. The ordinary checked expression evaluator is unchanged. Reference
  `NA^0` and `1^NA` identities are handled before missing propagation. Non-alpha
  geometry outputs still require finite values. Full core validation passes 464
  tests/doctests after the alpha extension. Reference numeric colour indexes, complete
  identity guide presentation/breaks, all missing/nonfinite geometry policies and
  actual Python/WASM/native/export acceptance remain open. A standalone unobserved
  colour sample parses its text on demand; eligible chart observations use the
  prepared pool. Exact integer keys outside binary64's safe domain and timestamp
  identity aesthetic values reject explicitly rather than silently narrowing.

- R color parsing adds the complete 657-name grDevices catalog and a bounded
  `color::parse_r` / `parse_r_with_palette` adapter. It shares hexadecimal byte
  decoding with authored CSS paint, while retaining R name/space/case rules and RGB
  at zero alpha. Numeric string indexes use the fixed R 4.6.1 palette or an explicitly
  supplied palette; core never reads graphics-process state. Forty-two further
  parsing records match R; one overflowing numeric-string index intentionally rejects
  with `PrecisionLoss` instead of adopting the observed platform conversion. This
  portability boundary remains outside equivalence. Reference discrete/manual color
  scales pool text through the R parser once at preparation; eight independent
  ggplot2 selections check `gray`, `green`, numbered names, spaces, transparency and
  palette indexes. Twenty-four related tests pass, including unchanged CSS/color and
  interpolation contracts. Final all-target Clippy passes. Actual host execution, general
  identity scales and numeric-valued R palette inputs remain open.

- Elapsed duration scales use numeric seconds and the existing linear mapping through
  `scale_duration()` / `AxisScale::Duration`, retained in wire v17. The shared extended
  break search accepts the reference duration step preferences; it is not duplicated.
  Default labels preserve hours beyond 24, vector-wide padding and up to six fractional
  digits. The formatter rejects microsecond magnitudes outside the exact f64 integer
  range. Fixed-second widths reuse the common guide selection path with bounded numeric
  duration candidates. The pinned oracle adds hms 1.1.4 and pkgconfig 2.0.3 only to the
  isolated development R lockfile. Forty-five break configurations, 30 automatic panels,
  12 fixed-second panels, 12 named-width panels, 12 pattern-label panels, nine secondary
  panels and seven additional label vectors pass three focused Rust tests, including
  primary JSON round-trips. Named duration widths use fixed lengths (month = 31 days,
  year = 365 days), independent of calendar progression. Explicit shared time formatters
  project numeric seconds onto the UTC epoch only for formatting; four portable patterns
  match the R oracle. Secondary duration axes inherit duration selection and labels,
  including negative affine factors. Exact source-value strings retain R binary
  residues that its decimal JSON numbers otherwise hide. Tests explicitly preserve all
  ticks; they exposed and now verify the reference single-break rule on zero-range
  numeric panels. All 455 core tests/doctests pass before the named-width/pattern/secondary
  additions; the final related duration/secondary/date/time/expansion/axis run passes
  31 tests. The Python/WASM constructor
  surfaces are wired to the same Rust builder; actual host runs remain pending.
  Full R format-string compatibility, automatic host duration coercion and complete
  argument/destination acceptance remain open. Final all-target Clippy and formatting checks pass.

- Reference point sizes now retain finite zero/negative values instead of dropping
  their observations. Geometry uses R's point-size/outline combination, suppressing
  nonpositive effective glyph dimensions while retaining source identity and selected
  scale values. Reference mapped/default point sizes and `.size()` default to millimeters;
  explicit `.radius()` retains its literal-radius contract, and explicit units and legacy
  validation remain authoritative. R's point outline parameter resolves to half its
  physical width. The portable reference point policy retains the PDF device's
  0.01-big-point zero-width hairline across destinations; other R device hairline
  implementations are not yet qualified. Canonical explicit `AreaSize` geometry keeps
  its equivalent-area contract. Nine R configurations / 63 glyph inputs check the
  selected values, graphics font sizes, exact circular radii and outline widths,
  including PDF-extracted zero-width evidence. The combined 34-test aesthetic/scale/
  stage/style-palette run passes. An initial full-suite mismatch exposed the explicit
  `.radius()` boundary; its original expectation is preserved by the corrected implementation.
  All 452 core tests/doctests and all-target Clippy pass. The retained reference PDFs have not yet
  received visual acceptance, and actual host/export/native requalification remains open.

- Date/datetime secondary guides now share the retained primary time mapping with a
  checked additive offset. The existing `secondary` API uses seconds for datetime
  and days for Date, with factor one; offsets must fit exact source units. Numeric
  transformations and non-unit factors reject for time sources. Independent calendar
  selection passes 54 pinned R panels spanning negative/zero/positive offsets,
  expanded/unexpanded ranges and supplied New York spring-gap/autumn-fold resources.
  A second regression checks retained edits, resolution
  rejection, coordinate-capability rejection and v17/legacy-envelope behavior.
  Native navigation excludes this guide-only variant. Five combined secondary tests
  pass. Full core validation passes 450 tests/doctests before the final fixture-only
  expansion; the expanded focused run and final all-target Clippy pass.
  Actual host/destination acceptance remains open.

- Timestamp population limits now participate in the existing scale stage under the
  ggplot profile. `ScaleProjection.timestamp` retains exact limits, origin and an
  optional authored unit; it is mutually exclusive with numeric limits/transforms.
  Source censor/squish compare integers before floating projection, and generated
  timestamp coordinates reapply the same limits without losing their representation.
  Eight pinned R Date/datetime summary cases distinguish censor, squish, keep and
  coordinate-only limits. Three focused tests also cover extreme i64 observations,
  exact origins beyond 2^53, retained edits, unit/schema rejection and standalone v17
  serialization. Keep performs a read-only source precision preflight; this adds an
  O(n) validation pass for retained timestamp populations and needs performance
  qualification. Unrepresentable kept observations reject instead of becoming missing.
  The combined 38-test stage/time/date/reverse run and 448 full core tests/doctests
  pass. All-target core Clippy passes; actual host acceptance remains open.

- `scale_date()` adds an explicit Date policy over exact UTC timestamp inputs,
  independent of the chart profile. It uses `TimeAxisScale` with day-based expansion
  and retains original timestamp units in data, inspection and geometry. Eighty-one
  R records pass through both second and millisecond source representations (162
  primary configurations), checking ranges, automatic breaks/labels, fractional-day
  positions and JSON round trips. Twelve further R panels verify day/week/month/year
  widths. Date day widths use an epoch-day lattice; datetime day widths retain their
  lower-local-boundary anchor. Both share one checked integer-lattice routine.
  Short Date ranges select whole-day candidates; longer ranges share the automatic
  calendar selector. Custom Date time labels format the UTC calendar date at midnight
  while retaining the original fractional-day tick position. The Date factory requires
  v17 even outside the ggplot profile, rejects numeric inputs and sub-day reference
  widths, and preserves original plots after edits. Canonical host dispatch and both
  host factory/declaration lists expose `scale_date`; actual host execution is open.
  This does not add language-specific R/Python Date-object coercion or infer a timezone
  from metadata. Limits/OOB, temporal secondary guides, minor guides and full argument
  reconciliation still require acceptance work.

- Automatic datetime breaks and default labels now use a reference selection adapter
  over the existing calendar and formatter. It retains the selected label pattern
  with its exact timestamp candidates, supports seconds through millennial intervals,
  and distinguishes elapsed weeks from local calendar-day/month progression. Five
  hundred four pinned R cases pass at the scale layer and through primary authoring
  and JSON round trips: six density hints (including 2.5), negative/modern epochs,
  short/constant spans, multi-century ranges and supplied New York DST resources.
  Thirty further primary panels verify the automatic policy after default/asymmetric/
  negative expansion. Default density restoration, custom formatting, retained edits,
  explicit empty selection and UTC/calendar interval preservation pass. Unrepresentable
  fractional constant-break identities reject instead of rounding. Requested density
  is bounded to 1–128, returned count has its own budget, and candidate search has a
  16,384-operation cap. Supplied calendars must cover the selection's surrounding
  boundaries. Core never consults the machine timezone. The primary automatic family
  remains UTC; local behavior is selected with an authored supplied calendar.
  This slice does not close width-string/date-format compatibility,
  temporal secondary axes, minor guides or refreshed host/destination acceptance.

- Datetime expansion now applies in seconds on calendar, UTC and automatically
  inferred timestamp axes. `TimeAxisScale` retains fractional viewport boundaries
  relative to its exact integer origin; `viewport()` exposes their integer enclosure,
  `relative_viewport()` exposes the actual bounds, and `tick_bounds()` excludes source
  quanta outside the view. Sub-quantum views can have no ticks. The existing numeric
  owner handles projection, inverse and outside policies. Explicit viewports override
  expansion; constant domains retain proportional second coordinates. The reference
  near-zero test uses absolute epoch magnitude while source mapping remains exact.
  Sixteen R datetime records pass across three primary authoring forms; three further
  nanosecond-source cases pass the modern-epoch near-zero rule. R range offsets are
  serialized relative to the epoch to avoid JSON losing microsecond detail. Range
  comparisons allow two binary64 epoch ULPs (2^-21 seconds at these modern epochs);
  origin-relative arithmetic is separately checked directly.
  Sixteen paired numeric R records exposed crossed expansion endpoints under negative
  contraction; one shared `continuous_viewport` helper now sorts the final panel range
  for linear, nonlinear and datetime consumers while raw `expand` semantics remain.
  Additional checks cover omit/clamp/extend, exact identities beyond 2^53, inverse,
  explicit viewport overrides, integer overflow and fractional width anchors. Authored
  UTC intervals reuse `UtcScale::ticks_in` after reference expansion. Existing time
  fixtures now explicitly author their reference's zero expansion. Actual host and
  destination acceptance and remaining Date controls remain open.

- Reference calendar-width progression is available as `GuideTickArguments.time_width`
  using the shared calendar interval descriptor. Unlike D3 `interval` field filtering,
  this policy floors the lower limit to one base calendar unit, then advances by the
  multiplier. Seconds/minutes use epoch lattices; hours/days/Monday weeks use elapsed
  durations; months/years use the supplied calendar. Sixteen pinned R records match
  exact timestamps and labels through primary authoring and JSON round trips, including
  seven-hour/two-day DST crossings and two-month/two-year local progression. Widths
  reject unsupported units and zero steps; argument choices are mutually exclusive.
  Both elapsed and calendar paths check candidate counts before allocating outputs.
  Two additional tests cover reversed/pre-epoch bounds, budgets, v17 enforcement,
  retained plots after edits, empty explicit selections and invalid numeric axes.
  Canonical dispatch and Python/TypeScript declarations expose `time_width`; actual
  host execution remains open. This structured width form does not yet parse R width
  strings, support `DSTdays`/quarters as authored width units, or certify historical
  timezone ambiguity rules beyond the supplied cases.

- Secondary axes now select breaks independently in alternate units under the
  ggplot2 profile, inheriting the primary transformation's default break function.
  Thirty-six affine R cases pass, including five logarithmic-break rejection cases,
  decreasing conversions, constant domains and linear/log/reverse/sqrt primaries.
  Sixteen additional R cases pass for authored cubic, positive square and increasing
  or decreasing piecewise transforms. `AxisBuilder.secondary_transform` uses the
  shared numeric engine with strictly monotone matching knots; clamping, rounding,
  repeated/nonmonotone knots, noninvertible families and collapsed finite precision
  reject. The optional descriptor requires wire version 17 and is exposed through
  canonical host dispatch and both host declarations. Refreshed hosts remain open.
  Secondary labels format the complete candidate list before out-of-range censoring.
  Reference guide placement uses a 1000-position sampled inverse and rounds normalized
  positions to three decimals, matching all captured reference positions at 2e-12
  relative/absolute tolerance. Exact semantic mapping remains separately verified
  against independent inverse formulas at the same tolerance. The sampled guide
  inverse adds a fixed 1000-knot resource per ggplot secondary axis; performance
  qualification remains open. The animation mapper consumes the same guide placement.
  Three focused tests also check explicit labels, edits/reset, retained original plots,
  wire rejection and precision collapse. Legacy affine guides retain inherited ticks
  unless independent tick configuration is authored. Secondary guides still bind a
  primary numeric axis and do not expose an independent navigation inverse; date/time
  secondary axes remain open.

- Fixed elapsed-second break selection is available as `GuideTickArguments.seconds`
  on UTC and supplied-calendar axes, independently of the chart profile. Thirty
  pinned ggplot2 panel records pass through primary authoring, serialization and
  layout: subsecond, pre-epoch, constant, uneven seven-second and DST gap/fold cases.
  Tick identities and label bytes match exactly. The shared formatter's `%Z` numeric
  offset is selected explicitly to match R's `%z`; this is not general R format-string
  compatibility. Integer lattice arithmetic retains timestamps beyond 2^53, rejects
  fractional source quanta, and checks the tick budget before enumeration. Three
  focused tests also cover primary edits, explicit empty selection, numeric-axis
  rejection and v17 minimum-wire enforcement. Python/TypeScript declarations expose
  the option; refreshed host execution remains open. This is the fixed-second form
  of `date_breaks`, including elapsed widths such as 3600 seconds. Calendar-width
  width-string parsing and other date/time arguments remain open.

- 18 additional pinned scales 1.4.0 records verify filled/hollow shape palettes and
  the 13-entry linetype palette, including empty populations and overflow. Overflow
  maps to typed missing values instead of cycling. Primary rule styles and primary
  point glyphs consume these through the existing value scale engine. Point shape
  constants override mappings; missing shapes omit marks, and shapes 21–25 retain
  transparent default fill. Five style tests pass in a combined 32-test scale/stage/aesthetic run.
  The new Shape value channel requires envelope version 17. Python/TypeScript
  declarations include it; actual host execution of this addition is still open.
  Automatic `aes().shape()` and `aes().linetype()` select discrete defaults under the
  ggplot profile; populations share scale identities across layers, and layer edits
  preserve default shape mappings. Numeric inputs reject. Palette warnings and shape
  guides remain open. Automatic alpha and linewidth mappings select continuous or
  ordinal reference policies through the numeric scale owner, with distinct scale
  identities, shared populations and retained defaults through layer edits. Constant
  parameters override these mappings before training. Both host declarations and
  canonical host dispatch expose the new aesthetic methods; fresh host runtimes
  are not yet qualified.

- The pinned R oracle supplies 766 palette cases, including all eight viridisLite
  options, uneven/unsorted/duplicate gradient positions, transparent colors and
  infinite parameters. Every captured output matches exact RGBA bytes.
- 93 independent scale records cover continuous training/limits/OOB, discrete
  factor levels and missing categories, named/unnamed manual palettes, sizing,
  log/sqrt/reverse transforms 32 binned configurations and ordinal numeric outputs. Constant limits include
  infinite and missing input; collapsed-bin scalar recycling is checked for every row. The numerical sizing
  tolerance is 2e-12; paint outputs match exact bytes. Missing discrete output is
  also checked as a typed missing value.
- 284 independent linear/logarithmic break records check candidate cardinality and
  values against scales 1.4.0 / labeling 0.4.3, at relative/absolute tolerance 2e-12.
  Counts include fractional requests. The ggplot normalizer selects these algorithms;
  positional numeric axes now select these algorithms under the ggplot profile.
- 499 whole-vector numeric label cases match the reference default R formatting,
  including fixed/scientific selection, shared decimal places and large/small values.
  Finite inputs and total label bytes are checked. Explicit D3 guide profiles and
  authored numeric formatters retain their independent selection rules.
- Trained color and numeric scales are checked against the compile category budget
  before preparing their trained palette. Binned descriptors account for their cuts;
  a primary chart regression rejects nice bins that exceed a two-entry budget.
- The finite expansion adapter has 120 pinned R cases covering asymmetric fractions,
  offsets, contractions, reversed limits and numerically constant domains. Automatic,
  explicit linear and logarithmic ggplot-profile axes use reference expansion; 32
  independent ggplot-built panels check ranges, breaks and labels together, plus
  explicit navigation-window preservation. `AxisSpec.expansion` supplies asymmetric
  overrides in transformed units and requires envelope version 17. Trained limits
  remain separate from the expanded viewport. Authored legacy nice/padding policies
  run before this reference viewport policy. Four constant linear/log panels with
  zero expansion check midpoint mapping, inverse and single-tick behavior against R,
  including projection of coordinates already transformed before statistics. Time
  expansion remains open.
- Discrete expansion now uses category centers 1 through N and the reference 0.6
  additive default in the shared band/point spacing kernel. The standalone policy
  retains wider continuous extents. 36 R range cases and nine R panel records pass;
  54 automatic/band/point configurations cover both destination directions, zero and
  asymmetric expansion, lookup, centers and guide positions. These projection tests
  explicitly preserve guide labels: the LibraryV1 thinning default still removes
  endpoint labels with zero expansion, and its ggplot presentation default remains
  GG-05 work. Continuous geometry extent training is not integrated by this slice.
  Explicit D3 categorical scale policies retain their separate spacing selection.
  The primary axis builder exposes reference expansion with profile-default reset.
- Primary point size now selects numeric or ordinal reference palettes. Regressions
  cover zero input, missing-size recovery after scaling, independent stroke width,
  constant radius, serialization and layer edits. Authored size mappings remain
  available for edits and are removed only from the execution mapping.
- Reverse positional scales now use the shared pre-stat transform engine. Sixteen
  pinned R panel cases pass: automatic/explicit limits, input order, constants,
  asymmetric expansion, original-unit breaks/labels, transformed observations and
  normalized projection. Inversion and Censor/Squish/Keep policies pass focused checks.
  Explicit histogram edges reverse before binning; the existing left-closed policy
  matches the corresponding R case. The R right-closed default remains GG-06 work.
  Primary Rust/Python/WASM authoring exposes `scale_reverse`; actual refreshed host
  execution for GG-04 remains open. Nested authored reverse projections require v17
  even when no positional axis is retained.
- Square-root positional scales pass 20 R panel records through primary authoring,
  including negative observations, zero/constant domains, explicit limits, source-unit
  breaks/labels and asymmetric expansion. The mapping retains expanded transformed
  viewports independently of their valid source-unit portion, so negative padding
  remains visible without creating invalid source breaks. `scale_sqrt` also supports
  explicit coordinate-only staging; negative source values are omitted at projection.
  Inverse tests cover zero-boundary roundoff, clamped nonzero boundaries, invalid
  negative coordinates and finite-range overflow/underflow. Refreshed hosts and
  non-point geometry/statistic-family qualification remain open.
- A primary-API regression checks omitted unknown point color, recovery through
  `after_scale`, constant-color precedence and transparent missing fill with a
  retained outline. Earlier GG-02 and GG-03 regressions pass.

Color conversions live in `color`, scalar spline/power interpolation in `interpolate`,
and palette/training/bin policies in `scales`. Four viridis seed tables were verified
identical to the existing catalog and reused. The remaining four retain their MIT
notice. Extended and log break adaptations retain labeling/scales MIT notices.

Definition/plot envelopes require version 17 when retaining the new scale policies;
standalone reference interpolation/normalization requires version 3. Existing
capabilities retain their earlier minimum versions. The ggplot2 profile now selects
trained numeric gradients and discrete hue defaults. Missing-aesthetic handling waits
until after post-scale expressions for point and line consumers.

## Commands and environment

Reference: R 4.6.1 r90187, ggplot2 4.0.3, scales 1.4.0, viridisLite 0.4.3,
labeling 0.4.3, from the existing pinned source/lock manifest.

```text
mise exec -- python tools/reference/r/run.py tools/reference/r/palettes.R
python3 tools/reference/r/palette_records.py
mise exec -- python tools/reference/r/run.py tools/reference/r/scales.R
mise exec -- python tools/reference/r/run.py tools/reference/r/style-palettes.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/reverse-position.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/sqrt-position.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/time-seconds.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/secondary-affine.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/secondary-transform.R
CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --test ggplot_secondary --locked
CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --test ggplot_time --test axis_ticks --locked
CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --test ggplot_reverse --test ggplot_expansion --test ggplot_stages --locked
CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --test ggplot_style_palettes --test ggplot_aesthetics --locked
mise exec -- python tools/reference/r/run.py tools/reference/r/breaks.R
mise exec -- python tools/reference/r/run.py tools/reference/r/expansion.R
mise exec -- python tools/reference/r/run.py tools/reference/r/labels.R
CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --locked
CARGO_TARGET_DIR=target/axis-rust-proof mise exec -- cargo test -p chart-core --test ggplot_scales --test ggplot_stages --test ggplot_aesthetics --test ggplot_palettes --test ggplot_breaks --test ggplot_expansion --test ggplot_labels --test ggplot_discrete_expansion --test scales --test full_scales --locked
```

The 120 expansion records, 32 panel records and 499 label vectors pass focused macOS
runs. The combined 49-test run passes, including retained legacy scale tests and an
explicit D3 guide override regression.
An earlier cached offline Linux run passed the original 760 palette cases and
46 scale records; it does not qualify subsequent changes. Current core Clippy passes
across all targets (including the collapsed and discrete expansion changes). The full
macOS chart-core suite and doctests passed 417 tests after the style-default changes
(`target/ggplot-style-full-core.log`). The reverse slice passed the combined
17-test reverse/expansion/stage run. The subsequent square-root slice passes seven
focused positional tests. Full macOS core tests/doctests pass 424 tests after both
transform additions (`target/ggplot-positional-full-core.log`, fresh
`CARGO_TARGET_DIR=target/ggplot-core-tests`); all-target core Clippy also passes.
The fixed-time slice passes 427 full core tests/doctests with zero failures or ignored
tests (`target/ggplot-time-full-core.log`) and all-target core Clippy. Formatting passes.
The secondary-axis slice then passes 430 full core tests/doctests with zero failures
or ignored tests (`target/ggplot-secondary-full-core.log`) and all-target core Clippy.
Formatting also passes after the final sampled-guide projection change.
The calendar-width slice passes 432 full core tests/doctests with zero failures or
ignored tests (`target/ggplot-time-widths-full-core.log`) and all-target core Clippy.
Reference generation: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/time-widths.R`.
Focused validation: `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --test ggplot_time --locked` (five tests).
The datetime-expansion slice passes 437 full core tests/doctests with zero failures
or ignored tests (`target/ggplot-time-expansion-full-core.log`) and all-target core Clippy.
Formatting passes. Reference generation uses `tools/reference/r/time-expansion.R`;
ten focused time tests pass, alongside the existing expansion and reverse-axis tests.
The automatic-datetime slice passes 442 full core tests/doctests with zero failures
or ignored tests (`target/ggplot-time-pretty-full-core.log`) and all-target core Clippy.
Five focused tests cover 534 primary reference panels; formatting passes. Reference
scripts are `tools/reference/r/time-pretty.R` and `tools/reference/r/time-pretty-expansion.R`.
The Date slice passes 445 full core tests/doctests with zero failures or ignored tests
(`target/ggplot-date-full-core.log`) and all-target core Clippy. Formatting passes.
Three focused Date tests pass; reference scripts are `tools/reference/r/date-scales.R`
and `tools/reference/r/date-widths.R`. An initial Date-width mismatch identified the
separate epoch-day alignment rule; the corrected shared implementation passes all 12
width records without changing their expected outputs.
The timestamp-population slice passes 448 full core tests/doctests with zero failures
or ignored tests (`target/ggplot-time-population-full-core.log`). All-target core
Clippy passes (`/tmp/ggplot-time-population-clippy.log`); its initial run identified
an unnecessary lazy fallback and an obsolete lint expectation, both corrected.
No refreshed host, native-window or Linux qualification is claimed for these additions.

The secondary-time slice passes 450 full core tests/doctests with zero failures or
ignored tests (`target/ggplot-secondary-time-full-core.log`). The subsequent fixture-only
expansion from 36 to 54 panels passes all five secondary tests
(`CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --test ggplot_secondary --locked`).
The first local-calendar run rejected insufficient supplied resource coverage; the
test resource now covers January 2024 through February 2025 with both independently
specified 2024 transitions. Expected R output was unchanged. Reference generation:
`mise exec -- python3 tools/reference/r/run.py tools/reference/r/secondary-time.R`.
Repository dependency/host-isolation/local-link validation passes
(`mise exec -- python3 scripts/check_repository.py`, `/tmp/ggplot-secondary-time-repository.log`);
this graph check is not native/host/Linux execution.

The point-size slice passes 452 full core tests/doctests with zero failures or ignored
tests (`target/ggplot-point-sizes-full-core.log`) and all-target core Clippy
(`/tmp/ggplot-point-sizes-clippy.log`). The full run includes canonical definitions
without primary source-grammar metadata. Reference generation uses
`mise exec -- python3 tools/reference/r/run.py tools/reference/r/point-sizes.R`;
the nine retained PDFs are under `target/ggplot-point-sizes-reference/`.
Repository dependency/host-isolation/local-link checks pass
(`/tmp/ggplot-point-sizes-repository.log`). These checks do not certify fresh host,
export/native, Linux or other-R-device hairline behavior.

Reference fixtures and generation scripts are committed-path artifacts under
`fixtures/parity/ggplot2` and `tools/reference/r`. The palette manifest records hashes
and the verified shared seed identities.

Duration validation: `target/ggplot-duration-full-core.log` records 455 passed and
zero failed/ignored tests before the final duration extensions;
`target/ggplot-duration-related.log` records 31 passing related tests after those
extensions. Final all-target core Clippy passes (`/tmp/ggplot-duration-clippy.log`),
and formatting checks pass. The reference is generated with
`mise exec -- python3 tools/reference/r/run.py tools/reference/r/durations.R`.
The dependency/link check passes (`/tmp/ggplot-duration-repository.log`). These are
core/source checks; they do not establish actual Python/WASM or destination acceptance.

R color validation: `mise exec -- python3 tools/reference/r/run.py tools/reference/r/color-names.R`
generates the independent catalog/parser/manual fixtures;
`mise exec -- python3 tools/reference/r/color_name_records.py` retains the sorted named
color facts in the runtime table. `target/ggplot-color-names-related.log` records 24
passing tests across color names, ggplot scales/palettes/style palettes, CSS color
parity, interpolation colors and paint integration. All-target core Clippy passes
(`/tmp/ggplot-color-names-clippy.log`). This slice has not received
fresh full-workspace, Python/WASM, native/export or Linux acceptance.

## Open acceptance work

Complete remaining channel/geometry-specific defaults, identity/manual/discrete
policies and resource checks, positional break/label and
expansion policies, date/time controls and secondary-axis requirements. Reconcile
all reference arguments against GG-00's inventory. Qualify real Python/WASM consumers,
updates, publication/native output, full repository checks and current Linux/macOS
runs before closing GG-04. Other geometry families retain their GG-07 default/missing
behavior work; no complete geometry-family equivalence is claimed here.

## Identity scale slice — 11 September 2026

Starting revision `a6caa39`, working-tree changes. Oracle commands:
`mise exec -- python3 tools/reference/r/run.py tools/reference/r/identity-scales.R`
(26 records) and `mise exec -- python3 tools/reference/r/run.py tools/reference/r/identity-controls.R`
(48 numeric and 48 discrete control records). Immutable expectations are
`fixtures/parity/ggplot2/identity-scales.json` and `identity-controls.json` in the same
folder. They use isolated R 4.6.1 / ggplot2 4.0.3. Numeric transformed comparisons
allow `2e-13 * max(1, abs(expected))`; typed missing/NaN and categorical values are
checked separately, and scene colour bytes are exact.

`CARGO_TARGET_DIR=target/ggplot-check mise exec -- cargo test -p chart-core --test ggplot_identity --locked`
passes seven tests (including the alpha extension). `CARGO_TARGET_DIR=target/ggplot-check mise exec -- cargo clippy -p chart-core --all-targets --locked -- -D warnings`
passes (`/tmp/ggplot-identity-clippy.log`). Full core validation passes 462 tests/doctests
with `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`
(`target/ggplot-identity-full-core.log`). Repository checks pass
(`/tmp/ggplot-identity-repository.log`). Package and cumulative gates stay open.

### Identity alpha and reference after-scale extension

`tools/reference/r/identity-alpha.R` records two independent point grobs / 24 paint
values, and `identity-alpha-expressions.R` records 14 expressions / 168 paint values.
Both execute through the isolated R runner. Expected artifacts are
`fixtures/parity/ggplot2/identity-alpha.json` and `identity-alpha-expressions.json`.
These check actual R point-grob colours against primary Rust prepared marks, including
infinities, finite out-of-range values, missing coverage, scalar operations, reductions
and power identities. They are not inspected native/export artifacts.

Final `CARGO_TARGET_DIR=target/ggplot-core-tests mise exec -- cargo test -p chart-core --locked`
passes 464 tests/doctests (`target/ggplot-identity-alpha-full-core.log`). Final all-target
core Clippy passes (`/tmp/ggplot-identity-alpha-clippy.log`), as does the repository
check (`/tmp/ggplot-identity-alpha-repository.log`). Source-stage expression exceptional
values, complete non-alpha geometry rules, numeric colour indexes and actual
host/destination acceptance remain unqualified by this slice.


## Registered breaks with explicit constructor guides — 13 September 2026

Revision: `6e74ae6` plus working-tree changes. FIX-GG04, scale break/label composition.
The source-captured defaults differ: `binned_scale` selects bins, whereas color steps
wrappers explicitly select colorsteps. No channel-inferred default was installed.
The new native constructor-guide route reuses all 2,960 pinned binned break fixtures,
including transformations, missing/constant/empty inputs, named/missing/infinite cuts,
three callback signatures, count forwarding and automatic versus registered labels.
It reproduced loss of break names when bins deferred registered labels using Hidden.
The shared candidate owner now preserves parsing and names while deferring only the
formatter call. All 1,836 successes and 1,124 reference errors reproduce. Legacy
candidate behavior retains its separate coverage.

Evidence:
- `mise exec -- cargo test -p chart-core --test ggplot_break_functions --test ggplot_pipeline_functions --test ggplot_binned_guides --test ggplot_binned_numeric --locked --target-dir target/ggplot-positional-wasm -j2`: 30 passed; `/tmp/ggplot-constructor-breaks-regressions.log`.
- Fresh Docker Rust 1.97.1 Python extension and native-built wasm32 extension:
  `scripts/bindings/ggplot_binned_break_functions.{py,cjs} --constructor-guides`,
  also with `--default-names`: 4,686 + 116 states per host, exact comparison in
  `target/ggplot-constructor-breaks/comparison.json`. Logs:
  `/tmp/ggplot-constructor-breaks-python.log`,
  `/tmp/ggplot-constructor-breaks-wasm-build.log`,
  `/tmp/ggplot-constructor-breaks-wasm-host.log`.
- Strict Clippy for all chart-core tests and repository checks pass: `/tmp/ggplot-constructor-breaks-clippy.log`, `/tmp/ggplot-constructor-breaks-repository.log`.
- The primary proof runner retains both constructor-guide host routes.

Limits: no new drawing or native-window inspection, no full constructor acceptance,
no guide painting claim. Constructor defaults, palette fallback and argument forwarding
remain to reconcile before GG-04 closes and GG-05–19 continue.


## Omitted generic binned guide — 13 September 2026

FIX-GG04 / GG2-03 at `6e74ae6` plus working-tree changes. The reference capture
now additionally omits `guide` from `binned_scale` in 108 draws, across three
aesthetics, three populations, three limit policies and four vector operations.
Its complete callback sequence and result match the corresponding explicit bins
case in every draw (87 successes / 21 errors). In particular a generic color
binned scale inherits bins, unlike the named colorsteps constructors. Typed host
lowering uses the existing `BinnedBins` descriptor; no runtime default changed.

Evidence:
- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/binned-guide-selection.R`:
  540 captured draws; `/tmp/ggplot-constructor-default-capture.log`.
- `mise exec -- cargo test -p chart-core --test ggplot_pipeline_functions --locked --target-dir target/ggplot-positional-wasm -j 2`:
  12 passed; `/tmp/ggplot-constructor-default-native.log`.
- `scripts/bindings/ggplot_pipeline_functions.{py,cjs} --guide-selection` using the
  existing current Linux Python and wasm32 extensions: 1,068 states per host;
  `/tmp/ggplot-constructor-default-{python,wasm}.log`. Exact records and 168 byte-identical
  publication files: `target/ggplot-constructor-default/comparison.json`.
- All 168 files match inspected `target/ggplot-bins-steps/wasm` output;
  `target/ggplot-constructor-default/inspection-matches.json`.
- The primary authoring runner expects the expanded 1,068-state route.

Limits: the lowering is explicit in the proof adapter; this does not establish a
new named authoring API or theme-selected palette fallback. No new native window
inspection or guide presentation acceptance. GG-04 and subsequent stages remain open.


## Named numeric binned constructor keys — 13 September 2026

FIX-GG04 / GG2-03 at `6e74ae6` plus working-tree changes. The pinned numeric
capture now records actual built guide keys for all 216 cases. The 204 applicable
size/area/alpha/linewidth cases compare these keys through the primary Rust plot
path and both hosts. The named defaults lower to explicit bins; NULL breaks lower
to Hidden. Size/linewidth reject `right` at the named reference wrapper, so those
12 cases are still excluded from the generic descriptor's mapping comparison.
No existing expected values or tolerances changed. Full reference guide layout is
not established by these key assertions.

Evidence:
- `mise exec -- python3 tools/reference/r/run.py tools/reference/r/binned-numeric-palettes.R`:
  216 captured builds; `/tmp/ggplot-numeric-constructor-capture.log`.
- `mise exec -- cargo test -p chart-core --test ggplot_binned_numeric --locked --target-dir target/ggplot-positional-wasm -j 2`:
  one nonempty test validates 204 standalone and primary cases;
  `/tmp/ggplot-numeric-constructor-native.log`.
- `scripts/bindings/ggplot_binned_numeric.{py,cjs} --constructor-guides`:
  370 exact Python/WASM states and 12 publication files;
  `target/ggplot-numeric-constructor/comparison.json`,
  `/tmp/ggplot-numeric-constructor-{python,wasm}.log`.
- All files match inspected `target/ggplot-binned-styles/wasm` output;
  `target/ggplot-numeric-constructor/inspection-matches.json`.
- Focused numeric/pipeline strict Clippy passes:
  `/tmp/ggplot-numeric-constructor-clippy.log`.
- The primary proof runner retains the additional constructor-guide host route.

Limits: no new production/wire behavior, theme-palette selection, custom numeric
ranges, complete named argument forwarding or native window inspection. Existing
host binaries remain current because the changes are captures and assertions only.
GG-04 and all subsequent requested stages remain open.


## Numeric range and maximum forwarding — 13 September 2026

FIX-GG04 / GG2-03 at `6e74ae6` plus working-tree changes. The named numeric
constructor capture now contains 288 builds; the prior 216 records compare
exactly unchanged against the saved preceding capture. The 72 new cases use
ascending/reversed/constant ranges and area maximum 9/2/0 across all six retained
populations. Existing PowerRange descriptors reproduce 276 applicable cases in
the native test, both standalone mapping and primary guide keys. Python's style
comparison now applies the existing reference alpha-byte clamping rule when a
custom alpha range exceeds one; underlying mapped guide numbers remain unclamped.

Evidence:
- Pinned R capture: `/tmp/ggplot-numeric-range-capture.log` (288 builds).
- Focused native numeric test: `/tmp/ggplot-numeric-range-native.log` (276 cases);
  strict Clippy `/tmp/ggplot-numeric-range-clippy.log` passes.
- Current Python/WASM constructor and legacy descriptor routes: 502 states each;
  exact parsed records and 12 publication files per route match in
  `target/ggplot-numeric-range/comparison.json`. All 24 files also match inspected
  `target/ggplot-binned-styles/wasm` files. Host logs:
  `/tmp/ggplot-numeric-range-{python,wasm,legacy-python,legacy-wasm}.log`.
- The previously running Docker Rust 1.97.1 Linux core/export aggregate completed
  successfully: 716 tests across 149 result summaries, zero failures;
  `/tmp/ggplot-constructor-breaks-linux.log`. It predates the latest numeric range
  assertions; the focused run above covers those additions.

Limits: no production/wire change, new native window inspection, theme-selected
palette fallback, malformed range argument acceptance or full constructor coverage.
All requested cumulative gates remain open.


## Registered theme palette selection — 13 September 2026

FIX-GG04 / GG2-03 at `6e74ae6` plus working-tree changes. New
`scale-palette-selection.json` captures 216 actual reference draws: three scale
families, three channels, explicit/custom/built-in/invalid fallbacks, supplied or
absent theme palette, and ordinary/missing/empty populations. The reference retains
explicit palettes and otherwise selects the theme before the constructor fallback.
The invalid numeric fallback is ignored by the reference when a built-in aesthetic
fallback exists; typed lowering uses that existing fallback descriptor.

The compiler resolves registered theme selections before population training, on a
temporary definition. Authored definitions, fallback operations, original plots and
held exports remain immutable. Wire v45 and theme v3 reject downgraded readers.
The test also reproduced built-in continuous paint missingness using nonexistent
pooled indices, and `pow(-Inf, 0.5)` retaining infinity where reference `sqrt` gives
missing. Shared paint batching and square-root evaluation now match the reference.

Evidence:
- `tools/reference/r/scale-palette-selection.R` via the pinned R runner:
  216 draws; `/tmp/ggplot-palette-selection-capture.log`.
- `cargo test -p chart-core` with `ggplot_palette_selection`, `ggplot_pipeline_functions`,
  `ggplot_scales`, `ggplot_binned_numeric`, `ggplot_palettes`, `ggplot_palette_functions`,
  `ggplot_continuous_palette_functions`, `ggplot_binned_palette_functions`,
  `interpolate_scalar`, `interpolate_descriptors`, `scale_interpolated`, under mise,
  `--locked --target-dir target/ggplot-positional-wasm -j 2`: 42 passed;
  `/tmp/ggplot-theme-palette-regressions.log`.
- Fresh Linux Python and wasm32 extensions with extension-proof:
  `/tmp/ggplot-theme-palette-python.log`, `/tmp/ggplot-theme-palette-wasm-build.log`.
  Final host routes: `/tmp/ggplot-theme-palette-python-host.log`,
  `/tmp/ggplot-theme-palette-wasm-host.log`.
- 619 exactly equal Python/WASM records: 432 original/restored/layer-edit states,
  72 theme edits, 108 data replacements matching fresh batches/held snapshots and
  seven registration/parameter/name/theme-version/wire-version rejections.
  `target/ggplot-theme-palette/comparison.json`; 54 byte-identical publication files.
- All 18 scenes inspected across PNG plus independent SVG/PDF rendering:
  `target/ggplot-theme-palette/inspection/review-1.png` through `review-9.png`.
  The zero-alpha continuous supplied palette is intentionally transparent and
  matches the captured RGBA values. Other samples retain the expected colors,
  sizes, counts and unclipped layout across formats.
- Strict core tests Clippy, repository checks, mypy and TypeScript pass:
  `/tmp/ggplot-theme-palette-{clippy,repository,mypy,tsc}.log`.
  The primary authoring runner now retains the host and type-consumer routes.

Limits: registered theme palettes only; no built-in theme palette coercion,
full element inheritance, multi-aesthetic/alias execution qualification or new
native-window inspection. Existing 716-test Linux evidence predates v45. GG-04 and
subsequent cumulative stages remain open.


## Ordered theme palette lookup and aliases — 13 September 2026

FIX-GG04 / GG2-03 at `6e74ae6` plus working-tree changes. The palette selection
capture now contains 324 draws. The prior 216 compare exactly unchanged against
the preceding capture. The additional 108 cover size-before-alpha, alpha-before-size,
missing-first lookup, alias-only `palette.color`, color-over-colour precedence,
and the `color` aesthetic alias, across continuous/binned/discrete scales,
ordinary/missing/empty populations and absent/supplied contexts.

Evidence:
- Pinned R runner: `/tmp/ggplot-palette-lookup-capture.log`, 324 captured draws.
- Native `ggplot_palette_selection` test: `/tmp/ggplot-palette-lookup-native.log`,
  all 324 cases and explicit-setter override behavior pass; strict focused Clippy
  `/tmp/ggplot-palette-lookup-clippy.log` passes.
- Current Python/WASM hosts: `/tmp/ggplot-palette-lookup-{python,wasm}.log`,
  979 states per host (648 original/restored/layer-edit, 108 theme edits,
  216 data replacements, seven rejections). Exact records and 54 files:
  `target/ggplot-palette-lookup/comparison.json`. All files match inspected v45
  outputs: `target/ggplot-palette-lookup/inspection-matches.json`.
- The initial Linux aggregate exposed three remaining explicit scale initializers
  in export distribution tests and the scale benchmark. They now retain an empty
  lookup list, preserving their prior behavior. Initial failure log:
  `/tmp/ggplot-theme-palette-linux-initial.log`; corrected aggregate is running.

Limits: no new palette/theme production behavior, built-in palette coercion,
complete named-constructor coverage, guide presentation or native-window claim.
The family coverage index gains related test links and retains unqualified
complete-contract status. GG-04 and subsequent cumulative stages remain open.


## Theme color vectors and active paint lookup — 13 September 2026

Revision: `6e74ae6` plus working-tree changes. FIX-GG04 / GG2-03 / GG2-10.
`tools/reference/r/theme-palette-values.R` captures 180 actual ggplot2 4.0.3 draws
against scales 1.4.0: continuous/binned/discrete, ten palette vectors, ordinary/
missing/empty populations, and absent/grey50 missing replacement. Native tests match
110 successes and 70 reference errors. Invalid colors in unused discrete empty
palettes remain lazy, matching the reference. Missing discrete palette slots do not
become input-NA replacement colors. Wire v46/theme v4 retain these descriptors.

A separate native counterexample first failed with an earlier constant-overridden
fill mapping diverting the next active color theme selection. The mutable walk now
uses the same active-channel filter as the read-only walk; the regression passes.
No reference expectations or visual baselines changed.

Validation:
- 44 focused Rust regressions: `/tmp/ggplot-theme-values-regressions.log`.
- Strict all-target core/export Clippy: `/tmp/ggplot-theme-values-clippy.log`.
- Fresh Linux Python and WASM extension builds; registered route: 979 equal states
  and 54 identical publication files, `target/ggplot-theme-values/registered-comparison.json`.
- Color-vector route (`ggplot_palette_selection.py/.cjs --color-vectors`): 438 equal
  states, including reference errors, round trips, layer/theme edits, repeated data
  replacement versus fresh batches, stable held exports, and version/key rejections.
  `target/ggplot-theme-values/vectors-comparison.json` records 54 identical files.
- All 54 vector files inspected in nine contact pages under
  `target/ggplot-theme-values/inspection/`; SVG rendered independently and PDF with
  Poppler. Missing/short discrete palettes omit the captured marks; gradients and
  alpha values display consistently without clipping.
- Strict Python/TypeScript consumer examples pass: `/tmp/ggplot-theme-values-mypy.log`
  and `/tmp/ggplot-theme-values-tsc.log`.

The current Linux aggregate is pending in `/tmp/ggplot-theme-values-linux.log`.
The primary cumulative runner has been extended but not run. This does not qualify
named theme palettes, complete theme inheritance, automatic constructor lookup,
full guide presentation, native windows, or all GG-04 contracts.


## Automatic/default palette selection — 13 September 2026

Revision `6e74ae6` plus working-tree changes; FIX-GG04 / GG2-03 / GG2-10.
`default-palette-selection.R` captures 56 actual draws: automatic and explicit
continuous/discrete color/fill and continuous/ordinal numeric constructors, explicit
numeric ranges, area/radius, and absent/supplied palettes. `ggplot_default_palettes`
first reproduced an automatic color mapping ignoring the theme. Default factories
now retain lookup, with automatic fill using its own key. Explicit ranges and
area/radius bypass lookup as captured. The reference capture also records actual
alpha bytes, avoiding a host-derived half-tie rounding assumption.

52 focused Rust tests pass (`/tmp/ggplot-default-palettes-regressions.log`); strict
all-target core/export/example Clippy passes (`/tmp/ggplot-default-palettes-clippy.log`).
Fresh Python and WASM match all 168 original/round-trip/layer-edit/theme-edit states
and 30 byte-identical publication files (`target/ggplot-default-palettes/comparison.json`).
All files inspected through five contact pages in that directory's `inspection/`;
SVG rendered independently and PDF with Poppler. Themed alpha, size, fill and color
are consistent and unclipped. Guide presentation remains GG-05.

Fresh retained palette routes pass 979 registered and 438 color-vector states, with
54 files each exactly matching the preceding inspected v46 routes. Comparisons are
`registered-comparison.json` and `vectors-comparison.json` under the same directory.
The cumulative primary runner is extended but not executed. The first Linux run
passed 719 tests then failed export doctest E0460 because a concurrent Python build
replaced artifacts. A rerun without overlapping builds is pending in
`/tmp/ggplot-default-palettes-linux.log`. Named palettes and full constructor argument
reconciliation remain open; these proofs do not close GG-04.


## Complete pinned named palette registry — 13 September 2026

Revision `6e74ae6` plus working-tree changes; FIX-GG04 / GG2-03 / GG2-10.
`named-theme-palettes.R` captures all 138 installed names, an unknown name and mixed-case
viridis across continuous/binned/discrete ordinary/missing/empty draws: 1,260 cases,
1,251 successes and nine errors. `ggplot_named_palettes` matches every draw with both
string and singleton-vector wire forms. Theme v5/wire v47 retain names. The source
registry metadata and deterministic data generator reuse all existing shared families
and add only 79 HCL ramps and 14 manual tables; ADR-018 records provenance/coercion.

Validation:
- Native named test passes (`/tmp/ggplot-named-palettes-native.log`). The focused
  default, degenerate-mapping, color-name, named and vector targets pass (13 tests
  total). The vector test's singleton version assertion was corrected; its successful
  rerun is `/tmp/ggplot-named-palettes-vectors-regression.log`. This executable was
  delayed at `_dyld_start` before the test harness ran, then passed all three tests.
- Fresh Linux Python and WASM match 4,183 states per wire form: round trips, layer/
  theme edits, repeated replacements versus batches, held exports and rejections.
  `target/ggplot-named-palettes/comparison.json` and `scalar-comparison.json` match
  exactly and record 54 byte-identical files each.
- All 54 files inspected in nine contact pages under that directory's `inspection/`,
  including independent SVG and Poppler PDF rendering. Captured ordinary/missing
  hue, Brewer and viridis marks agree across formats without clipping.
- Strict all-target core/export/example Clippy and strict consumer types pass:
  `/tmp/ggplot-named-palettes-clippy.log`, `-mypy.log`, `-tsc.log`.
- Deterministic regeneration of `ggplot_named_data.rs` matches exactly.
- Fresh existing host routes pass 438 vector, 979 registered and 168 default states.

The prior Linux default run caught a test that deliberately removed `ggplot` policy
without clearing the new lookup metadata; the setup now clears both. The v47 Linux
aggregate is running with `--no-fail-fast` in `/tmp/ggplot-named-palettes-linux.log`.
The cumulative primary runner is extended but not executed. Complete constructor
reconciliation, shape/linetype defaults and GG-04 closure remain open.


## Style defaults and temporal theme exception — 13 September 2026

At `6e74ae6` plus uncommitted changes, FIX-GG04 / GG2-03 / GG2-10: actual reference
captures qualify 36 shape/linetype and 60 Date/datetime palette draws. Native style
selection initially ignored the supplied shape palette; timestamp paint initially
applied a theme palette that the reference bypasses. Both shared automatic selectors
are corrected. Explicit solid/hollow shapes bypass lookup; temporal numeric defaults
retain lookup. The temporal test covers four timestamp units.

- `mise exec -- cargo test -p chart-core --test ggplot_default_palettes --test
  ggplot_default_style_palettes --test ggplot_scale_limit_helpers --test ggplot_stages
  --test ggplot_style_palettes --test ggplot_temporal_aesthetics --locked` passed
  26 tests (`/tmp/ggplot-default-temporal-theme-native.log`).
- Strict all-target core/export/example Clippy passes
  (`/tmp/ggplot-default-temporal-theme-clippy.log`).
- Fresh macOS Python extension built with `extension-module,extension-proof`;
  `ggplot_default_palettes.py/.cjs --temporal` pass 180 states and 30 byte-identical
  publication files (`target/ggplot-default-temporal-theme/comparison.json`).
  All 30 inspected in five contact pages, independently rendering SVG and PDF.
- The earlier fresh style host proof matches 144 states and 36 inspected files
  (`target/ggplot-default-style/comparison.json`). Both routes now join the primary
  runner; this addition does not claim cumulative execution.

Temporal linewidth uses segments so the mapped aesthetic is operative. This retains
all 60 mapped reference values from the initial point capture; ignored point-linewidth
missing-row behavior remains a GG-07 follow-up. The prior Linux aggregate passed
718 tests with three stale default-version assertions now corrected. Docker's fresh
Python attempt failed before build (HTTP 500, then missing socket), and the installed
app cannot launch (`kLSNoExecutableErr`). No new Linux aggregate pass is claimed.
Constructor reconciliation and GG-04 remain open.

Current macOS Python/WASM retained regressions also pass: style 144 states / 36
byte-identical files, blank layers 96 / 54, and temporal precision 180 exact records.
Comparisons reside in `target/ggplot-default-temporal-theme/{style,blank}-comparison.json`
and `precision-{python,wasm}/precision-records.json`.


## Public binned count palettes and hue white point — 13 September 2026

Revision `6e74ae6` plus uncommitted changes; FIX-GG04 / GG2-03 / GG2-10. The
108-draw `binned-constructor-palettes.json` capture qualifies 90 successful public
color/fill binned constructor draws and 18 unknown-name errors. Explicit named/color
vector palettes use the resolved bin count. The new v48 descriptor delegates to the
existing discrete count owner. Short results retain grey50 fallback; names resolve
with the captured constructor error behavior, including empty data.

This proof reproduced a shared 16-color hue error (`#0CB702` versus `#0BB702`).
Actual farver `as_white_ref('D65')` installs chromaticities (0.31271, 0.32902), not
the rounded XYZ constants in the previous polar-Luv helper. Correcting that parameter
choice passes 140 new independent count/offset/reversal/dark reference palettes.
ADR-018 records source and parameter provenance. Fixtures/tolerances were preserved.

- Five native tests pass: 108 constructor draws, 140 hue palettes, 1,260 named
  draws in both wire forms, and the existing 766-case palette regression plus shared
  interpolation proof (`/tmp/ggplot-binned-constructor-hue-corrected.log`).
- Fresh macOS Python and WASM match 288 states and 30 byte-identical SVG/PDF/PNG
  files (`target/ggplot-binned-constructor-palettes/comparison.json`). This includes
  immutable round trips/layer edits, repeated missing/empty/ordinary replacements
  against fresh batches, and held publication snapshots.
- All 30 files inspected in five independently rendered contact pages. Rerendering
  after the final white-point correction exactly preserves all five inspected pages.
- Fresh retained named-palette hosts match 4,183 states and 54 byte-identical files
  (`named-comparison.json` under the same directory).
- Strict all-target core/export/example Clippy passes on the final white-point
  correction (`/tmp/ggplot-binned-constructor-clippy.log`); the macOS aggregate
  passes 725 tests (`/tmp/ggplot-binned-constructor-macos-aggregate.log`) before
  the following registered callback NULL correction.
  The preceding v47 macOS aggregate passed 723 tests; Linux remains unavailable.
- The primary runner includes the two new native targets and 288-state host route;
  cumulative execution is not claimed.

The next reference capture covers public constructor palette functions. Complete
constructor forwarding and GG-04 remain open; this bounded result does not close
GG-05–19.


## Binned constructor function results — 13 September 2026

Revision `6e74ae6` plus uncommitted changes; FIX-GG04 / GG2-03 / GG2-10.
`binned-constructor-functions.json` captures 108 public count-function draws across
color/fill, ordinary/missing/empty data, three requested bin counts and six result
shapes. Native tests exercise both count adaptation and generic vector paths,
including exact callback arguments. The capture reproduced NULL incorrectly using
grey50: NULL now remains distinct from empty/short results through shared batch
preparation, mapped rows and guide rejection. ADR-018 records the boundary.

- Six focused native tests and strict all-target core/export/example Clippy pass.
- Fresh macOS Python and WASM pass 324 exactly matching states and 36 byte-identical
  publications; retained built-in palettes pass 288 states and 30 files. Results:
  `/private/tmp/finstack-chart-proof-20260913/functions/{comparison,builtin-comparison}.json`.
- All 36 function SVG/PDF/PNG files were inspected in six contact pages under
  `functions/inspection`. The rebuilt independent resvg inspector required an explicit
  Noto Sans serif fallback for SVG family aliases; after correcting that inspection
  setup, text and marks agree with PNG/Poppler PDF. No baseline changed.
- `CARGO_TARGET_DIR=/private/tmp/finstack-chart-proof-20260913/build mise exec -- cargo test -p chart-core -p chart-export --locked`
  passes 726 tests/doctests with zero failures on Darwin arm64
  (`/tmp/ggplot-binned-functions-macos-aggregate.log`).
- The primary runner includes both new constructor routes; its cumulative execution
  remains unclaimed. Linux Docker remains unavailable.

During this work prior `target` artifacts, R packages and helper executables
disappeared externally. Earlier recorded paths are historical and no longer retained.
Pinned R packages and wasm-bindgen 0.2.128 were restored under
`/private/tmp/finstack-chart-tools`; actual hosts were rebuilt in the proof directory
above. Re-running the 108-draw R generator reproduced the committed fixture exactly.
`run.py` now accepts optional `CHART_REFERENCE_R_WORK` and
`CHART_REFERENCE_R_LIBRARY` paths to keep the reference tools outside volatile builds.
Complete constructor forwarding and cumulative acceptance remain open; GG-04 and
GG-05–19 are unfinished.


## Explicit ordinal default paint — 13 September 2026

At `6e74ae6` plus uncommitted changes, `ggplot_color_ordinal` selects the existing
viridis count palette, bypasses theme lookup and retains NA paint. Twenty pinned
R draws pass ordinary/singleton/missing/all-missing/empty color/fill populations
with themes absent or supplied. NA color removes points; NA fill retains the
shape-21 outline. No wire version or source data representation was added.

Three focused native tests pass, including the prior 56 default draws and 60
temporal draws over four timestamp units. Actual v48 Python/WASM modules match
60 states and 30 byte-identical SVG/PDF/PNG files, all inspected after aligning
the point recipe with reference shape 21. These host descriptors exercise existing
shared lowering; the new Rust factory is validated natively. Results and five
inspection pages: `/private/tmp/finstack-chart-proof-20260913/ordinal`.
Strict all-target core/export/example Clippy passes before the test-only shape
alignment (`/tmp/ggplot-ordinal-defaults-clippy.log`); the primary runner includes
this route but cumulative execution remains unclaimed. The 726-test macOS aggregate
predates this additive helper. Linux remains unavailable.

Next: ordinal type-vector interpolation and remaining constructor forwarding.
Automatic ordered-factor dispatch and complete GG-04 acceptance remain open.


## Ordinal type-vector palettes — 13 September 2026

Revision `6e74ae6` plus uncommitted changes; FIX-GG04 / GG2-03 / GG2-10. The new
`OrdinalColors` count descriptor retains ordinal `type` color vectors and delegates
their inclusive ramp to the established Lab gradient owner. Named gradient
palettes now reuse the same count helper. Single color names remain lazy, so
unknown names only reject when actual paint is needed; empty vectors reject
even with empty data. Wire v49 retains the descriptor, including nested binned
count forms, and downgraded envelopes reject. ADR-018 records this ownership.

- Five native tests pass: 70 independent R draws, the 1,260-case named palette
  test in both wire forms, and all three default-constructor tests
  (`/tmp/ggplot-ordinal-types-native.log`).
- Fresh Python/WASM builds match 154 immutable/replacement-versus-batch states
  and 30 byte-identical SVG/PDF/PNG files. All 30 inspected in five contact pages.
  Artifact root: `/private/tmp/finstack-chart-proof-20260913/ordinal-types`;
  `comparison.json` records equality.
- Strict all-target core/export/example Clippy passes
  (`/tmp/ggplot-ordinal-types-clippy.log`). Repository/dependency/link checks pass.
- The primary runner includes the new route; cumulative execution is not claimed.
  The preceding 726-test aggregate predates ordinal additions; Linux is unavailable.

Remaining constructor/formal reconciliation and full GG-04 acceptance stay open.
No automatic ordered-factor or arbitrary R function/option execution is claimed.


## Qualitative type lists — 13 September 2026

Revision `6e74ae6` plus working-tree changes. `pal_qualitative` selects the first
shortest vector with length at least the trained nonmissing count. If no vector is
sufficient it calls the hue palette with the supplied hue parameters. Selected
names match the complete domain without the manual constructor's name-based limit
filter. Unknown colors fail only if a mark uses them. `GgplotDiscretePalette::Qualitative`
retains typed vectors/names and hue parameters in v50 and reuses the manual/hue
owners; prior claimed wire envelopes reject the new capability.

- `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library Rscript tools/reference/r/qualitative-type-palettes.R`:
  **PASS 80** ggplot2 4.0.3 / R 4.6.1 draws. Fixture
  `fixtures/parity/ggplot2/qualitative-type-palettes.json` covers both paint channels,
  shortest/tied/too-short vectors, named and duplicate keys, empty vectors/lists,
  unknown paint, and five populations. Two used-unknown-name draws reject.
- `CARGO_TARGET_DIR=/private/tmp/finstack-chart-proof-20260913/build mise exec -- cargo test -p chart-core --test ggplot_qualitative_types --test ggplot_ordinal_types --test ggplot_default_palettes --locked`:
  **PASS 5 tests**, log `/tmp/ggplot-qualitative-types-native.log`.
- Fresh extension-proof Python and wasm32 builds use the same external target,
  pinned wasm-bindgen 0.2.128, and host scripts `ggplot_binned_constructor_palettes`
  with `--qualitative-types`: **PASS 206 states and 48 publications per host**.
  `ggplot_palette_compare.py ... 206 48` passes exact record and file equality.
  Paths: `/private/tmp/finstack-chart-proof-20260913/qualitative-types/{python,wasm,comparison.json}`;
  logs `/tmp/ggplot-qualitative-types-{python,wasm}.log`. All eight
  `inspection/review-*.png` pages were visually inspected using the supplied Noto
  font and independent SVG/Poppler rasterizers: expected points, colors and axes
  are visible and consistent across PNG/SVG/PDF.
- Strict all-target core/export/example Clippy and `scripts/check_repository.py`
  **PASS**; logs `/tmp/ggplot-qualitative-types-{clippy,repository}.log`.

The preceding ordinal v49 aggregate completes with **728 macOS core/export tests
and doctests passing**, log `/tmp/ggplot-ordinal-types-macos-aggregate.log`. It does
not cover this subsequent policy. The primary runner includes the new 206/48 route
but has not run cumulatively. Fresh Linux, complete constructor forwarding,
ordered-factor dispatch and full GG-04 acceptance remain open.


## Discrete paint constructor forwarding — 13 September 2026

Revision `6e74ae6` plus working-tree changes. Existing Hue/Grey/Brewer/Viridis
policies and `ColorScaleBuilder::missing` retain the reference public constructors'
missing defaults and explicit palette parameters. The authored recipes bypass
theme lookup when a constructor supplies its own palette. No new production
implementation or wire version was required.

- `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library Rscript tools/reference/r/discrete-paint-constructors.R`:
  **PASS 80** actual ggplot2 4.0.3 / R 4.6.1 draws, committed to
  `fixtures/parity/ggplot2/discrete-paint-constructors.json`. Both channels, four
  constructors, default/custom controls and five populations cover hue endpoints,
  chroma/luminance/start/direction, grey endpoints, named divergent Brewer reversal,
  and viridis alpha/interval/direction/option.
- `CARGO_TARGET_DIR=/private/tmp/finstack-chart-proof-20260913/build mise exec -- cargo test -p chart-core --test ggplot_discrete_paint_constructors --locked`:
  **PASS 1** test covering all 80 draws and JSON reconstruction. Strict Clippy on
  that target passes. Logs `/tmp/ggplot-discrete-paint-constructors-{native,clippy}.log`.
- Actual v50 Python/WASM modules, rebuilt in the preceding qualitative slice,
  run `ggplot_binned_constructor_palettes.{py,cjs} --discrete-constructors`:
  **PASS 208 states per host**, including layer edits, missing/empty/restored data,
  fresh-batch comparison and immutable publication snapshots.
  `ggplot_palette_compare.py ... 208 48` passes exact states and **48 byte-identical
  SVG/PDF/PNG files**. Paths:
  `/private/tmp/finstack-chart-proof-20260913/discrete-constructors/{python,wasm,comparison.json}`.
  All eight `inspection/review-*.png` pages were inspected: expected paint, marks,
  fonts and axes remain visible and consistent across all three formats.

The route is added to the primary runner; cumulative execution was not performed.
Constructor-wide rejection/alias reconciliation and continuous/binned paint
forwarding remain open, as do GG-04 and subsequent stages.


## Continuous/binned paint constructor forwarding — 13 September 2026

Revision `6e74ae6` plus working-tree changes. Reference viridis_c samples six
colors before Lab interpolation; distiller samples seven Brewer colors. The new
`PaletteSpec::CountGradient` delegates to the existing count palette and Gradient
owners, retaining optional anchor positions and enforcing the existing 65,536
palette budget. The new recipe requires plot v51, standalone interpolation v4,
and standalone scale v6. Existing recipes retain their earlier versions.

The independent capture exposed custom gradientn singleton alpha 157 versus
reference 156 at the exact 156.5 byte midpoint. Reference `colour_ramp` encodes
alpha through farver's ties-to-even conversion; `Gradient::sample` previously used
the general D3 paint conversion. It now shares `color::d65::alpha_byte` with the
already verified numeric reference alpha path. No fixture/tolerance was weakened.

- `R_LIBS_USER=/private/tmp/finstack-chart-tools/r-library Rscript tools/reference/r/continuous-paint-constructors.R`:
  **PASS 200** actual draws, captured in
  `fixtures/parity/ggplot2/continuous-paint-constructors.json`: both paint channels;
  gradient/gradient2/gradientn/distiller/viridis_c and their stepped/count counterparts;
  default/custom palette controls and five populations. Includes colors alias,
  uneven/alpha stops, midpoint, Brewer direction/type, viridis interval/option/alpha,
  and ignored viridis_b values. Forty singleton/all-missing binned outcomes reject.
- Core `ggplot_continuous_paint_constructors`, `ggplot_palettes`,
  `ggplot_named_palettes`, `ggplot_ordinal_types`: **PASS 6 tests**, including
  200 new reference draws, 766 palette cases, 1,260 named draws in both forms,
  and 70 ordinal draws. New standalone envelope/roundtrip/budget checks pass.
  Log `/tmp/ggplot-continuous-paint-constructors-native.log`.
- `ggplot_pipeline_functions reference_alpha_matches_all_765_byte_boundary_cases`:
  **PASS 1 test / 765 boundaries**; log `/tmp/ggplot-count-gradient-alpha-regression.log`.
- Fresh actual v51 extension-proof Python/WASM builds and existing constructor
  runners with `--continuous-constructors`: **PASS 480 lifecycle states per host**.
  Exact state and **120 byte-identical SVG/PDF/PNG file** comparison passes;
  `/private/tmp/finstack-chart-proof-20260913/continuous-constructors/{python,wasm,comparison.json}`.
  All twenty `inspection/review-*.png` contact pages were inspected with supplied
  Noto font and independent SVG/Poppler rasterizers: paint, labels and marks align.
  Logs `/tmp/ggplot-continuous-paint-constructors-{python,wasm}.log`.
- Strict all-target core/export/example Clippy **PASS**, log
  `/tmp/ggplot-continuous-paint-constructors-clippy.log`.

The primary runner includes 480/120 execution; cumulative execution and fresh
Linux remain open. macOS core/export aggregate **PASS 732 tests/doctests**, log
`/tmp/ggplot-count-gradient-macos-aggregate.log`. The full primary runner is now
executing with the same external target and output directory
`/private/tmp/finstack-chart-proof-20260913/cumulative`, log
`/tmp/ggplot-count-gradient-cumulative.log`; completion remains open.
Full constructor rejection/adaptation reconciliation and GG-04 acceptance remain open.


## Gradient remapping cardinality correction — 13 September 2026

At `6e74ae6` plus working-tree changes, `pal_gradient_n` source inspection and a
new 96-draw capture establish that `values` defines an independent remapping
vector. It need not have the same length as the colors. The shared Lab owner now
uses `length(values)` for its normalized coordinates, preserving sorting and the
mean-coordinate treatment of duplicate positions. No fixture or tolerance changed.

- `tools/reference/r/gradient-remap-constructors.R` captures both paint channels,
  gradientn/stepsn/distiller/viridis_c, shorter/longer/duplicate/descending values,
  and ordinary/missing/empty populations. The new test fails before the fix with
  `Gradient positions must be finite and match the colors`
  (`/tmp/ggplot-gradient-remap-before.log`).
- Scoped native proof **PASS**, including all 96 new draws, 200 existing constructor
  draws and the existing palette suite; `/tmp/ggplot-gradient-remap-native.log`.
- Fresh Python/WASM **PASS 288 exact lifecycle states**, including replacement,
  retained captures, disposal and wire round-trips. All **96 SVG/PDF/PNG files are
  byte-identical**. Evidence and comparison are under
  `/private/tmp/finstack-chart-proof-20260913/gradient-remap`; all sixteen contact
  pages were inspected with explicit Noto fonts and independent rasterizers.
- Strict core/export/example all-target Clippy **PASS**
  (`/tmp/ggplot-gradient-remap-clippy.log`); formatting and focused whitespace pass.

The cumulative runner was stopped before changing the engine. Its initial failure
was an older GG-02 host assertion comparing millimeters with publication points;
explicit 72/25.4 conversion repairs both 13-figure host proofs without changing the
reference fixture. The earlier 732-test aggregate predates the remapping fix.
Nonfinite positions and empty-input rejection timing remain open, as do full GG-04
acceptance and GG-05–19. The primary runner now includes the new 288/96 route.


## Nonfinite gradient remapping — 13 September 2026

At `6e74ae6` plus working-tree changes, gradient positions use the shared portable
`Number` type. The original coordinate indices survive removal of NaN positions;
infinite endpoints and duplicate means follow `stats::approxfun`. A NaN remapped
value also remains missing in a single-color ramp. Special positions require
interpolation v5, standalone scale v7 and plot v52. Finite recipes retain older
versions. Descriptor identity includes NaN and signed zero through the shared owner.

- `tools/reference/r/gradient-nonfinite-constructors.R` captures **156 draws**,
  covering NaN leading/middle positions, positive/negative/both infinities,
  duplicate infinities and single-color missing remapping through both channels.
- Native **PASS** all new draws, the earlier 96/200 captures, palette regressions,
  and standalone identity/downgrade rejection tests
  (`/tmp/ggplot-gradient-nonfinite-native.log`).
- Fresh Python/WASM **PASS 468 exact lifecycle states and 156 byte-identical
  SVG/PDF/PNG publications**. All twenty-six contact pages inspected with explicit
  Noto fonts and independent rasterizers. Evidence:
  `/private/tmp/finstack-chart-proof-20260913/gradient-nonfinite`.
- Strict all-target core/export/example Clippy **PASS**
  (`/tmp/ggplot-gradient-nonfinite-clippy.log`).

The primary runner includes the 468/156 route. Cumulative and fresh aggregate
qualification remain open; invalid-position timing on empty inputs is next.
GG-04 and GG-05–19 remain open.


## Gradient position rejection timing — 13 September 2026

At `6e74ae6` plus working-tree changes, the reference palette wrapper compiles its
colors but delays invalid-position errors until evaluation. The strict standalone
`Gradient::new` still validates its explicit positions immediately. Empty reference
color populations skip their fabricated type-validation sample; all-missing vectors
still validate. Empty binned populations skip reference gradient/registered sampling
for synthetic boundaries. Named count palettes still validate selection on empty
inputs: the initial aggregate caught an overbroad bypass, which was narrowed while
preserving the existing unknown-name reference fixture.

- `tools/reference/r/gradient-invalid-constructors.R` captures **200 outcomes** across
  four constructors, both channels, five invalid position vectors and five source
  populations. The initial 120-case proof reproduced the empty failure before the
  fix (`/tmp/ggplot-gradient-invalid-before.log`).
- Native **PASS 200 outcomes**, 96 finite/156 nonfinite/200 original constructor
  draws, binned-guide and palette regressions. The final guard refinement additionally
  passes both 108-case public count-palette/count-callback targets:
  `/tmp/ggplot-gradient-invalid-{native,regressions}.log`.
- Actual Python/WASM **PASS 240 states and six byte-identical SVG/PDF/PNG files**;
  the two empty samples were visually inspected. Evidence under
  `/private/tmp/finstack-chart-proof-20260913/gradient-invalid`. These modules precede
  the final count-palette guard refinement; the cumulative rebuild must cover it.

The fresh macOS aggregate is executing at
`/tmp/ggplot-gradient-values-macos-aggregate.log`. The earlier 732-test result predates
all gradient-values corrections. Cumulative host execution and complete GG-04
argument reconciliation remain open, as do GG-05–19.


Final empty-gradient preparation refinement preserves later standalone sampling by
reusing `BinnedVectorPipeline` and its existing immutable-population cache. The
2,688-case authored-limit population corpus caught the lost later-sampling behavior;
it and the named count-palette/callback targets pass after the fix. No expected
values or tolerances changed. The full macOS core/export run now **PASS 736 tests
and doctests**, `/tmp/ggplot-gradient-values-macos-aggregate.log`; strict all-target
Clippy and repository checks pass. The cumulative runner is rebuilding and running
under `/private/tmp/finstack-chart-proof-20260913/cumulative`, log
`/tmp/ggplot-count-gradient-cumulative.log`. Completion remains open.


## Numeric range constructors and implicit circle publication — 13 September 2026

At `6e74ae6` plus working-tree changes, the reference capture
`tools/reference/r/numeric-paint-constructors.R` adds **160 actual draws/outcomes**:
size, area, radius, alpha, linewidth and their three public binned variants;
default/custom/reversed/zero settings; ordinary/singleton/missing/all-missing/empty
populations. Linewidth uses segments because points ignore that aesthetic in R.
The 24 binned singleton/all-missing reference rejections are retained.

The native mapping proof passed but publication exposed implicit circles using the
unconverted radius, including a rejected zero-radius scene primitive. Projection
now shares `reference_point_radius` with explicitly selected symbols, including
unbounded coordinate projection. Empty glyphs retain their prepared rows and emit
no primitive, as existing empty symbol paths do. Legacy explicit radius semantics
are preserved. The device-radius regression uses independently captured R font
sizes at three stroke settings in points and logical pixels.

- Native **PASS** the 160-outcome target and all sixteen aesthetic tests:
  `/tmp/ggplot-numeric-radius-native.log`.
- Actual fresh Python/WASM **PASS 432 exact lifecycle/rejection states and 96
  byte-identical publications**, comparison and modules under
  `/private/tmp/finstack-chart-proof-20260913/numeric-constructors`.
- All sixteen contact pages (32 charts, PNG and independent SVG/PDF rasterizers)
  inspected. Default/custom/reversed/zero size behavior is visible; zero alpha
  paints nothing. Zero-linewidth hairlines have device-dependent raster visibility.
- Both existing default-palette host routes **PASS 168 states and 30 publications**.
- Strict all-target core/export/example Clippy **PASS**:
  `/tmp/ggplot-numeric-radius-clippy.log`.

The primary runner includes the new 432/96 route. The earlier cumulative run stopped
on stale temporal wire assertions (now exact v42 paint/v45 numeric-theme assertions;
both 440-state host routes pass), and its continuation was stopped before the
renderer correction. A completely fresh cumulative run is executing at
`/tmp/ggplot-numeric-cumulative.log`, output
`/private/tmp/finstack-chart-proof-20260913/cumulative-numeric`. Fresh aggregate
execution is at `/tmp/ggplot-numeric-radius-macos-aggregate.log`. No cumulative or
complete constructor acceptance is claimed; GG-04 and GG-05–19 remain open.


Final native qualification for this slice: **PASS 738 macOS core/export tests and
doctests across 161 targets**; repository and formatting checks pass. The cumulative
run encountered a stage assertion that still interpreted an R size as a radius.
The unchanged R size/stroke values now feed its graphics-parameter formula in both
host assertions; both stage routes pass. The same-engine run continues in
`/tmp/ggplot-numeric-cumulative-resume.log`, retaining previously passed initial
subprocess results. Full cumulative completion remains open.


The full sizing inventory adds `scale_size_binned_area`: **180 outcomes**, **484
exact host states**, and **108 byte-identical publications** now pass. Binned area
singleton input succeeds where the other binned constructors reject; all 28 captured
rejections remain explicit. Native extension evidence:
`/tmp/ggplot-numeric-binned-area-native.log`.

Inspection of the four added samples found a real empty-glyph status defect:
negative binned area output retained rows but emitted no glyph, and layout added
“No data”. The pre-fix image is retained as
`numeric-constructors/inspection/before-empty-glyph-status.png`. Reference-profile
layout now distinguishes retained, unpainted populations with no projection
omissions from genuinely empty populations. The new implicit/explicit-symbol
regression and all seventeen aesthetic, twenty layout and one numeric-constructor
tests pass (`/tmp/ggplot-empty-glyph-status-native.log`). The cumulative run was
stopped before this engine correction; a fresh aggregate and host rebuild are
running. Prior 738 aggregate and 484/108 host evidence precede this final fix.


The final status fix now passes rebuilt-host **484/108** comparison. Hash comparison
with the preceding publications identifies exactly three changed files, the reversed
binned-area SVG/PDF/PNG. Reinspection of their contact page confirms the erroneous
label is removed; the other 105 files are unchanged. Evidence:
`numeric-constructors/empty-glyph-publication-changes.json`. Strict Clippy and
repository checks pass. A fresh cumulative run is at
`/tmp/ggplot-empty-glyph-cumulative.log`, output
`/private/tmp/finstack-chart-proof-20260913/cumulative-empty-glyph`; the aggregate
passes **739 tests/doctests across 161 targets**, with no failing summaries, at
`/tmp/ggplot-empty-glyph-macos-aggregate.log` (exit 0). Cumulative host completion
remains open at this update.

### Joint colour/fill scale populations — 13 September 2026

`tools/reference/r/shared-paint-aesthetics.R` captures fifteen actual builds/draws
under the pinned R 4.6.1 / ggplot2 4.0.3 installation. One scale governs both
colour and fill across continuous, binned, discrete, manual and identity policies;
ordinary, missing and empty populations test joint-domain training and retained
paint behavior. The new `ggplot_shared_paint_aesthetics` Rust target passes all
fifteen cases, with immutable JSON preservation. No shared-engine change was needed.

Actual Python/WASM scripts `ggplot_shared_paint_aesthetics.{py,cjs}` use the freshly
built modules from `cumulative-empty-glyph`; each passes 45 original/layer-edit/
theme-edit states with JSON restoration. `ggplot_palette_compare.py ... 45 15`
passes exact state equality and fifteen byte-identical publication files under
`/private/tmp/finstack-chart-proof-20260913/shared-paint`. All three contact pages
in `inspection/` were viewed after rendering SVG with the explicit Noto font and
PDF with Poppler; point stroke/fill distinctions agree across the three formats.
Logs: `/tmp/ggplot-shared-paint-{native,python,wasm}.log`. The primary runner includes
the route; its currently running process predates that addition. The earlier
739-test aggregate also predates this additional test. This evidence qualifies
joint paint mapping, not GG-05 guide merging/key composition or GG-04 completion.
