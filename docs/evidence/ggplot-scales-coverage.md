# GG-04 reference coverage reconciliation

Revision `6e74ae6` plus the qualified working-tree changes, 14 September 2026.
The owned GG-04 scale contracts are qualified; complete reference export acceptance
retains the named GG-05/GG-16 and GG-19 gates.
[The coverage index](ggplot-scales-coverage.json) enumerates all
152 GG-04 exports from the immutable ggplot2 4.0.3 inventory: 141 functions,
11 ggproto objects, 960 formal argument occurrences and 57 distinct argument names.
It now maps all 960 formal occurrences, 276 inherited method occurrences and
160 inherited fields to computational controls, typed language adaptations or
named downstream owners. Every control links its implementation and scoped test
evidence. This mapping does not itself pass a runtime or complete export gate.
[Executed slice evidence](phase-2-ggplot-scales-2026-09-10.md) owns numerical results,
commands, host executions and inspected artifacts.

The captured source now enumerates inherited methods and fields for all 11 classes
in the coverage index, in addition to the 141 function signatures. This makes the
class contract visible without treating shared method names as equivalent behavior.
The immutable inventory's OPEN labels remain baseline labels, not a current count
of absent implementations. The following historical argument rows describe bounded
proofs; the reconciliation below supersedes their stale population/helper gaps.

Current integrated validation passes 862 macOS tests/doctests and full
`mise run check`. The final cumulative host proof passed all 672 declared commands
and inline
assertions against 6,343 baseline path hashes, verified after completion.
[Final qualification](phase-2-ggplot-scales-2026-09-14.json) records the source,
environment, log hashes and validation boundaries. The structural reconciliation
passes all 152 exports, 960 formal
occurrences, 276 inherited method occurrences and 160 fields; it does not replace
runtime qualification or the explicitly named downstream guide/extension gates.

The following slice snapshots are ordered newest first. Their test counts and
pending-next-slice statements describe each recorded revision; later slices
supersede those gaps. The first snapshot and the current integrated result above
own the present status:

- Explicit and minor positional vector selection is qualified and integrated in
  eleven paths. The 144 explicit and 240 minor source builds pass final native
  checks; fresh hosts match 360 and 600 records, with 90 inspected byte-equal files.
  The previous callback proof (180/27) and explicit proof (360/54) are unchanged.
  All 18 vector and 14 related guide tests and strict Clippy pass. The isolated
  860-test aggregate precedes the final guide-profile guard; combined main checks
  pass 862 tests/doctests and full `mise run check`. Final cumulative qualification
  passes all 672 declared commands and inline assertions against that source. This
  supersedes the explicit-vector gap recorded below.

- Positional vector callback dispatch is qualified and integrated in ten paths:
  72 source builds, 858 isolated macOS tests/doctests, strict Clippy, 180 matching
  host lifecycle/error records and 27 inspected byte-equal files. Complete inverse
  domains, NULL dispatch, batch break/label context and empty-panel suppression
  supersede the concrete positional gap below. The preceding 1,056-state proof
  and its 36 files are unchanged. Combined main validation passes 860 tests/doctests and full `mise run check`.
  A separate explicit positional vector-break/coupled-label gap is reproduced
  in 144 newly captured source builds; its isolated correction is pending.

- NULL transform/break dispatch is qualified and integrated in thirteen paths:
  384 primary builds, 116 raw built-in NULL/empty outcomes, 1,056 matching host
  records and 36 inspected byte-equal files. The isolated suite passes 857
  tests/doctests and strict Clippy; the preceding 3,168-state proof is unchanged.
  The positional callback caller is the next concrete audit, with 72 source builds
  captured. Final combined aggregate and cumulative qualification remain pending.

- Continuous/binned vector callback composition is qualified and integrated in
  fourteen verified paths: 1,152 primary reference builds, complete callback-input
  and mapped-key checks, 3,168 equal host lifecycle/error records and 39 inspected,
  byte-equal files. Per-layer training and guide palette coordinates are corrected;
  registered NULL forward dispatch preserves built-in rejection behavior. The
  isolated macOS suite passes 855 tests/doctests with strict Clippy, and the prior
  864-state identity callback proof is unchanged. The combined main suite passes
  857 tests/doctests and full `mise run check`. A further NULL break-callback
  dispatch gap is reproduced in the next isolated slice; final cumulative
  qualification and complete export reconciliation remain pending.

- Discrete colour/fill identity callback demand and NULL-versus-empty domains are
  qualified and integrated: 192 source builds, six native tests, 577 matching host
  lifecycle/wire records and 18 inspected byte-equal publications. The old 198-state
  discrete-limit proof and 12 files are unchanged. Retained NULL metadata requires
  v63; the isolated macOS suite passes 854 tests/doctests with strict Clippy. The
  combined main aggregate passes 856 tests/doctests and full `mise run check`.
  Fill legend painting remains GG-05. A newly
  captured continuous/binned vector-callback corpus is the next scale-side audit.

- The complete frozen-baseline primary-authoring runner passes all 621 logged
  commands. After its source verification, all six qualified slices below were
  integrated from immutable snapshots (57 source/reference paths). Main formatting
  and repository checks pass; the combined macOS suite passes 854 tests/doctests
  and full `mise run check` passes. Historical
  integration-pending statements below are superseded by this transfer. The final
  cumulative proof must use the combined source before GG-04 can close.

- Identity vector callback limits/breaks now have a qualified draft correction:
  per-layer transformed extents, whole inverse domains/results and hidden-guide
  demand match 288 primary builds, including callback inputs and NULL identity.
  Fresh hosts pass 864 lifecycle states and 18 inspected, byte-identical files;
  the prior 288-state vector proof and its 18 publications remain unchanged.
  The draft macOS aggregate passes 852 tests/doctests with strict Clippy and
  formatting. Eleven paths are frozen for integration. Discrete identity callback
  demand, main integration and the final cumulative aggregate remain open.

- Vector identity transforms now have a qualified draft correction for complete
  source batches, pooled layer training, authored limits and guide candidates.
  Ninety-six reference builds match native behavior and 288 host lifecycle states;
  all 18 publications were inspected. Reauthoring an unchanged numeric scale also
  retains its shared identity. The draft macOS aggregate passes 851 tests/doctests,
  with strict core/example Clippy; the prior 696-state identity proof and 24 files
  are unchanged. Nine paths are frozen for integration. The callback slice above
  extends these cases; numeric guide painting remains GG-05.

- Identity-scale transform dispatch now has a concrete qualified draft correction:
  the shared transform family replaces the linear fallback, and identity labels
  retain complete candidates before guide filtering. All 232 pinned builds across
  29 built-in transforms match native mapping, guide values and exact labels;
  fresh hosts match 696 lifecycle states and 24 inspected publication files.
  This repairs reverse/reciprocal missing candidates and two log-format precision
  discrepancies. All 777 states and 594 publications per host in the existing
  original/label/secondary modes remain unchanged from the main cumulative run.
  Eight source/reference paths are frozen for integration; numeric identity guide
  painting remains GG-05, and main aggregate qualification remains pending.

- `transform` reconciliation has qualified and integrated the built-in and composition slices.
  The built-in owner passes 29 configurations and 116 actual positional/paint
  cases, guide-count/registered-label controls and finite secondary guides.
  Composition passes 13 configurations and 195 positional/continuous/binned
  paint plots, including domain trimming, inherited counts and population errors.
  Actual Python/WASM records match, and publication samples are inspected; all
  594 refreshed built-in files remain unchanged after composition. The status
  ledger records exact commands, wire versions, transfer state and aggregate
  evidence. A registered pointwise owner is now integrated for 36 actual charts
  and 24 separate scale-query outcomes, with 108 matching host lifecycle states,
  36 inspected publications and versioned standalone/native rejection proofs.
  The status ledger records the 817-test macOS aggregate and declaration boundary.
  Transform-owned minor callbacks now pass 120 reference charts, six extra registered
  override routes, callback/budget/zero-range checks, 360 matching host lifecycle
  states and 36 inspected publication files. The final macOS aggregate passes 819
  tests/doctests. Minor painting remains guide work. Length-preserving vector
  transforms now pass 1,776 reference charts covering position/paint/size, limits,
  source layers/facets and generated count position/styles. The latest macOS
  aggregate passes 833 tests/doc tests; fresh hosts match 180 base-vector states
  and 44 generated-style states, with inspected publications. Both hosts also
  pass generated-style version-56 metadata round trips and downgrade rejection
  under replacement training. Broader generated statistics,
  callback combinations and remaining argument combinations remain open; see the status ledger for exact evidence boundaries.

- Probability transforms now have an explicit registered quantile/CDF adaptation.
  Four uniform/exponential configurations add 36 source chart draws and 120 raw
  numeric outcomes to the existing normal/logistic proof. Four native tests and
  strict example Clippy pass; actual hosts match 108 lifecycle states and 36
  inspected, byte-identical publications. This qualifies the callback-pair route
  and domain metadata without adding a statistical distribution dependency to core.
  The adapter does not claim every R distribution implementation. Guide-candidate
  labels use explicit native visibility adaptation; full guide presentation remains
  GG-05. The status ledger records the source, integration and cumulative boundary.

- Joint colour/fill `aesthetics` now has fifteen actual build/draw cases across
  continuous, binned, discrete, manual and identity scales, including missing and
  empty populations. One shared scale identity passes 45 matching host lifecycle
  states and fifteen inspected publications without engine changes. Guide merging
  and multi-aesthetic key composition remain GG-05 work.

- Numeric `range`/`max_size` forwarding matches 180 actual outcomes through nine
  constructors and five populations, with 484 exact host states and 108 inspected
  publications. The final layout fix removes an incorrect empty-population label
  from retained, unpainted glyphs; only its three publication files changed. Implicit-circle projection now shares the reference size/stroke
  conversion with explicit symbols and preserves empty glyph rows. The fresh macOS
  aggregate passes 739 tests/doctests across 161 targets after that status fix;
  cumulative host qualification is in progress; uncaptured argument combinations
  remain unqualified.

- Invalid gradient-position timing matches 200 outcomes: empty plots succeed while
  other captured populations reject. Both hosts match 240 states and six inspected
  publications before the final deferred-sampling refinement. That refinement
  additionally passes the 2,688-case standalone population corpus and the full
  736-test macOS aggregate; cumulative host rebuilding is in progress.

- Nonfinite gradient remapping now matches 156 further draws, 468 exact host states
  and 156 inspected publications. Shared portable numbers retain NaN/infinities
  and signed zero under v52. The subsequent timing proof is recorded above.

- Gradient `values` now remaps independently of color cardinality, matching 96
  additional constructor draws, 288 exact host states and 96 inspected publications.
  Shorter/longer vectors, duplicate positions and descending vectors are qualified
  for the captured populations. Subsequent nonfinite and timing proofs are recorded
  above.

- Continuous and binned paint constructor forwarding now matches 200 draws,
  480 exact host states and 120 inspected publications. Fixed-count gradients
  retain six-anchor viridis_c and seven-anchor distiller behavior. The discovered
  alpha midpoint error is repaired with shared reference ties-to-even encoding;
  all 765 existing alpha boundaries still pass. Full argument/rejection adaptation
  remains unqualified.

- Discrete hue/grey/Brewer/viridis constructor defaults and explicit settings now
  match 80 draws, 208 exact host states and 48 inspected publications using the
  existing palette/missing-paint owners. This closes the captured recipe boundary,
  without certifying uncaptured rejection/alias combinations.

- Qualitative `type` vectors/lists now match 80 actual reference draws, 206 exact
  Python/WASM lifecycle states and 48 inspected, byte-identical publications.
  Shortest sufficient selection, ties, named/duplicate lookup and hue fallback
  are qualified for the captured cases. This does not qualify all constructor
  dispatch, R function forwarding or process options.

- `xlim`, `ylim` and `expand_limits` have actual typed helper proofs. The first two
  cover 48 captured constructor cases, 181 matching host states and 48 inspected
  publications; blank-layer training covers 30 captured cases, 96 matching host
  states and 54 inspected publications. They retain their bounded captured-contract
  qualification in the JSON index.
- Positional vectors now cover temporal and binned inputs, selected statistics,
  fixed/free facets, shared source statistics, broadcast/panel/chart scope and
  identity-source chains. The latest mixed-route proof covers 240 native
  configurations, 520 matching host states and 108 inspected publications. Older
  statements below that these populations are unsupported are superseded. This is
  not certification of arbitrary statistical extension compositions.
- Direct generic-constructor reconciliation now links `continuous_scale`,
  `discrete_scale` and `binned_scale` to the existing `scale-palette-selection`
  corpus. Its `do.call` invokes each generic constructor directly: 108 draws per
  family, 324 total, cover explicit/theme/fallback/built-in palette precedence,
  invalid fallback rejection, ordered and aliased aesthetic lookup over
  ordinary/missing/empty populations. The 979 host states and 54 inspected files
  are recorded in the existing ledger; no new runtime gate is claimed here.
  All these cases author `guide='none'`, so they do not qualify default guide
  selection or `breaks=NULL` suppression. The subsequent continuous selection
  capture now qualifies 81 default/legend/hidden draws, 243 matching host states
  and 27 inspected publications, including NULL/empty suppression. The subsequent colorbar/interval selections and generic suppression are now
  integrated: 324 generic draws, 117 colorbar draws, 72 linear interval host cases
  and 108 transformed interval draws have scoped native/actual-host proofs and
  inspected publications in the status ledger. Interval label callbacks add 1,080
  source/native builds and 2,720 exact host states, with 108 inspected publication
  files. This qualifies their scale-side computation; full guide painting remains
  GG-05.
  Other argument controls still need per-constructor reconciliation. Typed aliases and R metaprogramming
  syntax follow the adaptation below; computational behavior remains in scope
  under GG2-03/GG2-10.
- Theme selection now qualifies the captured registered fallback/explicit precedence,
  ordered aesthetic lookup, color aliases, explicit vectors and all 138 pinned named
  palettes. Named forms match 1,260 source draws and 4,183 host states per form;
  automatic/default color/fill/size/alpha/linewidth behavior matches 56 draws and
  168 host states. These supersede earlier blanket claims that palette lookup is
  absent. Shape/linetype defaults add 36 draws and 144 host states; temporal default palette
  selection adds 60 draws and 180 host states. Explicit ordinal defaults add 20 draws
  and 60 host states; ordinal type vectors add 70 draws and 154 host states.
  Public binned palettes add 108 draws and 288 host states, and count functions add
  another 108 draws and 324 host states. Remaining constructor forwarding and
  complete element inheritance still need their own reconciliation.
- Date/datetime legend, colorbar, interval and hidden selections now match all 648
  source draws in four timestamp units, including twelve expected source errors.
  Existing calendar, interval and colorbar owners retain exact labels, mapped keys
  and ramp colors. Actual hosts match 7,680 replay/edit/error records with 198
  byte-identical publications; 36 new triplets were inspected and 90 previous files
  remain unchanged. Version-60 temporal interval selection is integrated. The same
  modules pass the 2,720-state numeric interval-label regression. Temporal interval
  callbacks and format precedence additionally match 2,160 captured draws in four
  units: exact callback names, metadata, endpoint calls and labels, with 21,872
  matching host records and 180 inspected byte-identical publications. The expanded
  macOS workspace passes 844 tests. Rendered interval, colorbar and numeric guide
  presentation remains GG-05. See the ledger for
  commands, source provenance and the separate cumulative qualification boundary.

- Scale class selection/metadata belongs to GG-04. Guide title callbacks, layout,
  colorbars, stepped guides and presentation belong to GG-05. Captured `make_title`
  and `make_sec_title` methods therefore need explicit cross-package evidence;
  deferring presentation does not qualify absent scale-side selection behavior.

### Current computational reconciliation

The JSON index is the per-export checklist: `argument_contracts` resolves every
formal occurrence; `inherited_computation_contracts` resolves every method;
`field_contracts` resolves inherited state. Shared groups retain one implementation
and evidence mapping. The following rows summarize those explicit links, replacing
the former blanket forwarding and uncaptured-combination placeholders.

| Controls | Implemented contract and evidence boundary |
| --- | --- |
| Limits, OOB, NA, category retention and expansion | Shared numeric/discrete/binned/temporal population policies, registered vector limits/OOB, factor/null catalogs, positional continuous limits and expansion. Existing source-backed tests include authored limits, fixed/free facets, staged statistics and replacement versus fresh training. Broader geometry/statistic acceptance belongs to its work package and GG-18/19. |
| Breaks, minor breaks, labels and counts | Typed default/hidden/explicit/registered operations, full vector callback inputs, names, NULL/empty distinctions, pre-filter labels and interval endpoints. The integrated positional/continuous/binned/identity/temporal proofs supersede earlier scalar and absent-callback claims. |
| Date/time controls | Explicit time/locale resources, date/time widths and patterns, fractional source precision, calendar boundaries, and captured DST folds/gaps. Environment/resource coverage remains stated in each proof. |
| Transform, rescaler and binned controls | Shared built-ins, registered and composed transformations, batch/NULL protocols, binned closure/count/boundary policies. The final ten integrated slices qualify the concrete callback and palette defects identified in reconciliation. |
| Palette selection and parameters | Explicit/theme/fallback/default precedence, named catalogs, qualitative/ordinal policies, count/vector callbacks, constructor numeric/style policies and independent gradient knot vectors. Alias syntax resolves to the same authored descriptor. Source-backed constructor captures include the data-dependent rejection cases listed in their fixtures. |
| Size, shape and linetype | Area/radius/size-zero and numeric ranges, solid/hollow and binned shape policies, nullable/factor mappings and shared callback labels. Varying-linetype geometry rejections remain recorded against geometry work; guide glyph painting belongs to GG-05. |
| Secondary axes | Numeric/discrete/temporal selection, monotone transforms, callback metadata and pre-filter labels, including source rejection timing. Title callbacks and guide layout belong to GG-05. |
| Aesthetics and guide selection | Shared channel training, hidden/legend/colorbar/bins/steps selection, mapped keys, ramps and interval labels are scale computations. Guide title/order/placement, numeric keys, colorbar/step rendering and collection are GG-05. |
| Class state and methods | Built-in class computations map to the same population, transform, mapping and guide-selection owners. Owned prepared state, fresh training and retained captures adapt clone/reset/cache behavior. Custom class authoring remains GG-16. R printing/call objects are diagnostic syntax, not another chart engine. |

These are mapped computational contracts, pending the combined-source workspace
and actual-host results. Full export acceptance also includes the explicit
GG-05/GG-16 and GG-19 owners; the index does not certify those packages early.

### Constructor syntax and computational ownership

The captured bodies of all 141 functions distinguish forwarding syntax from
computational controls. Apply the phase-2 plan's declared language adaptation when
reconciling the argument rows; do not introduce a second R-call interpreter:

| Captured form | Typed adaptation and acceptance boundary |
| --- | --- |
| `colours` / `colors`, `trans` / `transform`, `scale_*_continuous` aliases | Author one resolved palette/transform descriptor. Alias spellings are not required by the plan; palette sampling and transformation behavior remain required. R's duplicate-argument dispatch errors do not imply another descriptor field. |
| `scale_name`, `call` | Deprecated naming and R call-stack diagnostics do not select chart computations. Native diagnostics carry their own code/context; no R call object is stored in core. |
| `...` | Forward meaningful controls to the existing typed scale, guide and resource owners. Rust signatures and host descriptor validation enforce the portable schema. An R wrapper's forwarding syntax is not an additional callback protocol. |
| `super` and ggproto methods | Built-in scale families select shared training/mapping policies. Registered operations retain the required limit/break/label/palette/OOB computations. New class/extension authoring remains assigned to GG-16; guide title/layout methods remain GG-05. This assignment does not certify unimplemented computations in either package. |
| `type` as a function, formula/lambda controls | A fully authored typed scale or registered operation expresses the computation. Qualitative vectors/lists and palette count/vector functions have separate captured proofs; merely renaming an R function is insufficient evidence. |
| `x`, named `lims(...)`, `scale_type(x)` | Typed columns and explicit scale/limit builders own data dispatch and limits. `named-limit-dispatch.json` captures 300 named/public constructor comparisons and two reference compositions. The 48 positional outcomes pass the typed Rust builders and 181 matching actual host states with 48 inspected publications. Nonpositional computations use the selected scale-family evidence; arbitrary third-party R dispatch is excluded by the plan. |
| `aesthetics` | Bind the same scale identity to the selected typed channels. Joint colour/fill training now has direct source, native and host evidence above; guide/key collection remains GG-05. |

The index records these language adaptations separately from numerical controls.
Every numerical group links the source-backed tests for defaults, precedence,
rejections and callbacks; the scale ledger records their executions and limits.

Next: finish combined-source qualification, review the mapped contracts against its
results, then advance the next ready package in the existing prerequisite order.
GG-05 presentation and GG-16 extension authoring are explicit dependencies for full
export acceptance. This reconciliation adds no work-package backlog.

Validation for this reconciliation: every one of the 152 indexed exports retains
its captured signature and inherited member list; all 960 formal occurrences,
276 inherited method occurrences and 160 fields have mappings; every linked
implementation/test path exists. Runtime results remain in the executed scale
ledger; the interrupted run is retained, and the restarted cumulative proof uses
the integrated explicit/minor correction.
