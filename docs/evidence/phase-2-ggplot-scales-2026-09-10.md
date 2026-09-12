# GG-04 progress — 10 September 2026

Status: IN PROGRESS. Starting revision `a6caa39`; changes remain in the working tree.
GG-03, SP-06, CP-04 and CLR-04 prerequisites are accepted. This record covers the
current GG2-03/FIX-GG04 scale-policy slice and does not close GG-04 or G-GGPLOT.

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
guide construction fails. These remain GG-05 guide requirements. Rejecting their
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
Artifacts, inspection files and comparison hashes: `target/ggplot-discrete-palette/`.

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
