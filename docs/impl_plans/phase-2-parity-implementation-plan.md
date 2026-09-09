# Phase 2 — Complete D3 and ggplot2 capability parity

Document version: 0.5.0. Date: 7 September 2026.
State: implementation authorized; package outcomes belong to the status ledger.
Original planning baseline: `127fe2d4853f89b62ba59a17248485e3c378ba60` plus its working tree.
Implementation baseline: `fab2505951061eaafe9adb52c86b248ee0dfa6bf`, the committed
primary authoring implementation. [P2-00 coverage](../phase-2-coverage.md) and
[ADR-014](../adr/014-phase-2-integration-contract.md) supersede the historical
[current API review](../evidence/phase-2-current-api-review-2026-09-07.md) where noted.
Authority: [specification](../spec/gpui-charts-specification.md#21-phase-2-parity-scope).
Execution states and evidence: [status ledger](../implementation-status.md).

The subsequent [primary authoring API plan](primary-authoring-api-plan.md), under
specification 0.5.0 / AUT-01–09, assumes original WP-01–23 completion and owns AP-00–09
for that refactor. It makes this primary surface mandatory for every delivered parity
capability. Algorithm inventories and package ownership below remain unchanged;
GG-16 supplies extension/recipe semantics through that surface, not a separate late
façade. Additional parity packages are not declared complete by the baseline assumption.
Capability acceptance feeds G-PARITY and G-AUTH independently; neither gate depends on
the other. The release containing the refactor requires both where parity is in scope.

## 1. Outcome and relationship to Phase 1

Deliver the complete capability inventories from all eight D3 plans and the ggplot2
review through one Rust engine, with native, headless and actual Python/WASM evidence.
The D3 documents remain the detailed owners of their packages. This secondary plan owns
the combined sequence, shared interfaces, ggplot2 packages and cumulative acceptance.
It does not copy their method inventories or reset their package IDs.

Phase 1 is the foundation and interactive streaming program, WP-01–20 in the
[original plan](gpui-charts-implementation-plan.md). The ledger now records original
WP-01–23 completion, with original-scope limitations retained; the primary-authoring
AP-00–08 implementation is complete; AP-09 performance qualification remains open.
Those historical results do not certify the changed API
or expanded parity. Reuse accepted WP-14 and runtime interfaces, checking affected
behavior against the current AP implementation at integration. Pure kernel and oracle
work may proceed independently of final authoring certification.

WP-21–23 remain the single final hardening/performance/release sequence for the expanded
scope, after Phase 2 capability acceptance. Their original-scope completion is retained;
new workloads and changed paths require requalification. This preserves the requirement that
D3 parity precede G4. Neither G2 nor G3 is renamed a production release. Phase 2 expands
that final release to include the ggplot2 capability profile; it does not create a
second release process or treat planning as permission to publish packages.

The original assignment was documentation only. The owner subsequently authorized
implementation after Review API simplicity finished and committed its code. That
handoff is satisfied by `fab2505`; proceed one reviewable package slice at a time.
Existing source, review edits and historical acceptance limitations are preserved.

## 2. Complete input register and traceability

All parity review documents found under `docs/impl_plans/` and `docs/evidence/` at the
baseline are represented here. Their source observations may predate subsequent edits;
each implementation owner must reproduce applicable gaps before changing behavior.

| Input / reference baseline | Existing package owner | Requirements / acceptance | Phase 2 use |
| --- | --- | --- | --- |
| [D3 path review and plan](d3-path-parity-plan.md), 3.1.0 | WP-P01–04 | PTH-01–06; FIX-P01–06; G-PATH | Shared builder, arcs, replay, precision and renderer foundation |
| [D3 shape review and plan](d3-shape-parity-plan.md), 3.2.0 | WP-S01–08 | SHP-01–10; FIX-S01–09; G-SHAPE | Generators, curves, symbols, pies, stacks and links |
| [D3 color review and plan](d3-color-parity-plan.md), 3.1.0 | CLR-01–05 | COL-01–06; FIX-C01; G-COLOR | Floating color values, parsing, conversion and paint lowering |
| [D3 interpolation review and plan](d3-interpolate-parity-plan.md), 3.0.1 | WP-IP01–07 | ITP-01–08; FIX-I01; G-INTERPOLATE | Scalar/structured/color/transform/zoom interpolation |
| [D3 scale review and plan](d3-scale-parity-plan.md), 4.0.2 | SP-01–07 | SCL-06–08; FIX-20; G-SCALE | All 26 factories, operations, formatting and explicit calendars |
| [D3 scale-chromatic review and plan](d3-scale-chromatic-parity-plan.md), 3.1.0 | CP-01–05 | CHR-01–06; FIX-21; G-CHROMATIC | All 76 exports and 218 discrete arrays plus evaluator semantics |
| [D3 axis review and plan](d3-axis-parity-plan.md), 3.0.0 | WP-AX01–06 | AXIS-01–07; FIX-19; G-AXIS | Guide identity, tick policy, components and transitions |
| [D3 hierarchy review and plan](d3-hierarchy-parity-plan.md), 3.1.2 | WP-H01–08 | HIR-01–08; FIX-H01; G-HIERARCHY | Topology, operations, all layouts/tilers/helpers and history |
| [ggplot2 review](../evidence/ggplot2-parity-review-2026-09-07.md), 4.0.3 | GG-00–19 below | GG2-01–12; FIX-GG00–19; G-GGPLOT | Full chart capability coverage and reference behavior |
| [ggplot2 retained probe](../evidence/ggplot2-parity-2026-09-07/probe.rs) and [output](../evidence/ggplot2-parity-2026-09-07/probe.log) | GG-01/02 | GGP-01/05 findings | Historical defect/default evidence; reconcile the delivered legend fix and retain grouping comparisons |
| [Current package API review](../evidence/phase-2-current-api-review-2026-09-07.md) | P2-00, GG-01/02/18, WP-AX01, AP-07 | AUT-01/03–07, BND-01/03/04 | Current owner mapping, legend acceptance, profile lifetime, axis migration and host proof requirements; source observations retain their review revision |

| ggplot2 review finding | Required closure owner | Concrete completion evidence |
| --- | --- | --- |
| GGP-01, missing single-panel legends | GG-01, regression retained in GG-05 | Single panel and equivalent one-panel facet both paint the same logical guide |
| GGP-02, statistical defaults/breadth | GG-02/06/09/10/11 | Stage, closure, weight, generated-field and model comparisons |
| GGP-03, aesthetic semantics | GG-02/03/04 | Independent fill/stroke/alpha/type/size/linewidth, area semantics and staged expressions |
| GGP-04, analytical layers | GG-07/09/10/11 | Built-in statistics plus geoms, provenance, guides and host proofs |
| GGP-05, inferred groups | GG-02 | Discrete-aesthetic interaction, explicit override and unchanged legacy grouping |
| GGP-06, guides/colors/facets/time | D3 CLR/IP/SP/CP/AX, GG-04/05/12/14 | Reference-specific color/guide/calendar/facet behavior after shared kernels |
| GGP-07, coordinates/extensions | GG-13/15/16 | Actual coordinate transforms, clipping/guides and registered extension consumers |
| GGP-08, missing certification | GG-00/18/19, WP-21–23 | Pinned method/argument matrix, differential outputs and inspected destination evidence |

The review's GGP IDs remain findings. GG2 IDs in the specification are the new
requirements; FIX-GG IDs below are planned fixtures. No historical test count closes them.

## 3. Definition of full parity

**D3:** complete observable capabilities of the eight pinned modules, including their
standalone APIs and configuration/custom protocols. This does not claim all of D3:
force, Sankey, chord, selection, general transition/timer/zoom products and DOM APIs
are not introduced by these reviews. Their required consumers still receive the
algorithms specified by the eight inventories.

**ggplot2:** complete chart feature and behavioral capabilities of the pinned 4.0.3
reference through a declared typed compatibility profile. Include every documented
built-in geom/stat/position/scale/guide/facet/coordinate/theme family and its meaningful
parameters, defaults, generated fields and extension hooks. Include analytical fitting,
density, geography, polar/radial coordinates and mathematical text; these are Phase 2
requirements, not an unassigned future backlog. The profile must be named in claims.

GG-00 must enumerate the release exports, documented arguments, inherited controls,
aliases and supported dependency-backed methods, rather than using the family review
as an exhaustive checklist. The [reference index](https://ggplot2.tidyverse.org/reference/index.html)
and [tagged namespace](https://raw.githubusercontent.com/tidyverse/ggplot2/v4.0.3/NAMESPACE)
were checked during this planning task; exact source hashes and a runnable R lock remain
GG-00 deliverables. A newly discovered defined chart capability is assigned to one of
the packages below and keeps G-GGPLOT open until implemented.

Language adaptations are explicit: typed expressions/builders replace R quosures,
formulas and `+`; registered Rust operations replace serialized closures/ggproto objects;
owned resources replace grid objects, process globals and default devices. Alias
spellings, bundled demonstration datasets and arbitrary third-party R package execution
are not required. However, a documented computation or drawing capability cannot be
excluded merely because its R implementation uses a callback. Supply an equivalent
native/registered protocol and prove it with an external consumer. Autoplot/fortify
capabilities map to typed data/recipe dispatch, without promising the whole R ecosystem.

Maintain three explicit policies over shared kernels: existing chart defaults,
D3-compatible descriptors and the ggplot2-compatible profile. Existing definitions
retain source-space statistics, bin closure, explicit grouping, radius size and current
theme behavior. Reference defaults apply only when selected; serialize resolved policy
and resource identity. Profile switching is a semantic change and invalidates the
appropriate prepared data, guides, layout and snapshots. Never regenerate old baselines
to make a new profile appear backward compatible.

## 4. Shared architecture and ownership

| Shared contract | Sole implementation owner | Consumers / integration rule |
| --- | --- | --- |
| Path construction, arcs, numeric replay, SVG precision | WP-P01–04 | WP-S, hierarchy recipes, GG geoms/coordinates/text; no second sink or serializer |
| Shape generators, curve families, symbol geometry, pie/stack layouts | WP-S01–07 | GG-07/08/09/13/15; ggplot2 policy is an adapter, not an alternate shape engine |
| Color values and conversion | CLR-01–04 | IP, SP, CP, GG-04; no intermediate byte quantization |
| Range interpolation and sampling | WP-IP02–05 | SP, CP, AX, GG-04/05; no duplicated blend or scalar spline kernels |
| Domain mapping, inverse capabilities, ticks/formatting, calendars | SP-01–06 | AX, GG-04/05/12/13; explicit versioned time/locale resources |
| D3 palette catalog | CP-01–04 | GG-04 adds only reference-specific missing catalogs/policies, with provenance |
| Axis component identities/geometry/transition lifecycle | WP-AX01–05 | GG-05/12/13/14; categorical/continuous legends remain shared guide consumers |
| Grammar staging, generated fields, grouping, aesthetic resolution | GG-02/03 | Every GG stat/geom/position; one compiler and dependency graph |
| General legend allocation and components | GG-01 acceptance / GG-05 extensions | Reuse the delivered single/faceted painter; extend it for D3 color metadata and GG guides |
| Coordinate projection, subdivision, inverse and clipping protocol | GG-13 | GG-15 geography and all geoms; radial shape projection alone is insufficient |
| Topology and hierarchy layout/history | WP-H01–08 | Existing core recipes and extension consumers; not required by GG statistical kernels |
| Wire migration, operation registry and host adapters | One integration owner; each delivering package supplies its descriptors/tests | BND-01/02 and ADR-006/012; no independently chosen envelope versions |
| Revisions, scheduling, updates and coherent snapshots | WP-15–20 | Every new family; exact batch fallback until incremental equivalence is proven |

### 4.1 Integration through the current primary API

Use [AP-00's capability register](../primary-authoring-api.md) and the
[current API mapping](../evidence/phase-2-current-api-review-2026-09-07.md#current-api-routing-and-remaining-scope)
as the starting inventory. P2-00 links reference coverage to those rows; it does not
create a second authoring backlog. Revalidate active code before assigning a slice.

- `chart-core/src/plot/` owns Data/Plot and typed components. Extend existing mappings,
  stat/position, scale/color, facet and composition builders in their semantic package.
  Keep checked source/generated fields, explicit `group_all`, inferred facet catalogs
  and separate annotation/title/axis-label/legend components. Existing `after_stat` and
  `after_bin` reads are foundations; GG-02 adds the remaining expression semantics.
  Existing fixed annotations do not satisfy GG-08's source/stat-row text geoms.
- `chart-core/src/runtime.rs` and its modules own the retained Chart. Session delegates
  to it. Extend the same store/compiler/reducer/queue and invalidation contracts;
  retain native worker/presentation ownership in `gpui-charts`.
- `chart-export/src/authoring.rs` owns Output, export options and static/live request
  acquisition. GG-17/18 extend existing destinations and captures; preserve Presented
  versus Current independently of visible/full-domain and interaction inclusion.
- Standalone path/shape/color/interpolation/scale/hierarchy families retain their D3
  owners and direct APIs, feeding the same primary components. AP foundations do not
  establish these missing kernels or reference-equivalent defaults.
- AP-07 owns shared Python/WASM syntax, converters and declarations; each semantic
  package adds its public operations, component options and actual host tests. Host
  authoring wrappers and actual baseline proofs are delivered in `fab2505`, superseding
  the review's earlier JSON-only snapshot. Extend registrations, exports and declarations;
  neither Rust dispatch tests nor wrapper source alone establishes host acceptance.

Core remains synchronous and usable without GPUI, interpreters, mandatory threading,
system fonts or I/O. Keep algorithms within existing crate ownership. Evaluate mature
numerical/projection/text/encoding dependencies through focused capability spikes and
ADRs; only split a crate for demonstrated dependency/isolation benefit. GIS resources,
fonts, images and clocks are supplied by hosts. An optional adapter may isolate a heavy
dependency, but required parity is not achieved on a surface where it remains absent.

A D3 name shared with an R package is not evidence of identical numerics. In particular,
GG-04 must verify the reference's Lab white point, gamut/alpha handling, palette sampling
and viridis variants against R dependencies. Reuse CLR/IP primitives with explicit
parameters or extend them once; do not assume D3's D50 Lab or lookup ramps equal the
ggplot2 defaults. D3 and ggplot2 stacking/grouping/size defaults likewise stay distinct.

## 5. Entry package and coordinated execution waves

**P2-00 — Integration contract and coverage register.** Prerequisite: WP-14 evidence.
Requirements: GG2-01/12, ARC-03/04, BND-01, QLT-02/05. Owns planning/ADR/reference-tool
coordination only. Assign one integrator; reconcile the current tree, make a common
profile/wire/resource decision, and create a coverage register linking reference item
→ requirement → package → fixture → surface → verdict/evidence. Coordinate the eight
existing lane-entry contracts and GG-00, without replacing or charging for their work
again. Commit no empty public API forest. Exit: owner/dependency map, migration strategy,
reference lock layout and explicitly open gap register. Use one shared pinned Node
reference workspace for the D3 modules and browser oracle, plus a separate pinned R
environment for ggplot2; share fixture metadata/comparison conventions across them.
Link AP-00's delivered capability rows and active AP ownership. Record the remaining
GG-01 acceptance cases; assign GG-02 profile lifetime, WP-AX01 primary identity migration
and each package's AP-07 host integration before handing off those consumers. Resolve
each interface when its package starts; a missing host proof does not block independent
kernels. Proposed effort: 2–4 days, excluding already completed AP contract work.

Delivered entry contract: [ADR-014](../adr/014-phase-2-integration-contract.md) and
[coverage register](../phase-2-coverage.md), against committed API baseline `fab2505`.
Reference lock creation and method/argument expansion remain with the lane entries
and GG-00; this integration handoff does not execute or certify their oracles.

Default execution is one reviewable slice at a time; waves describe dependency order,
not an instruction to start agents or assume staffing. Independent ready kernels may
be assigned separately. Entries within a wave still follow their package prerequisites.

| Wave | Work | Exit / dependent work unlocked |
| --- | --- | --- |
| A — Lock contracts and reconcile gaps | P2-00; D3 entry packages WP-P01, CLR-01, WP-IP01, SP-01, CP-01, WP-AX01, WP-H01; GG-00 and GG-01 acceptance | Reproducible reference inputs, compatibility/resource boundaries and remaining legend acceptance evidence. WP-S01 still waits for WP-P04. |
| B — Shared mathematical foundations | WP-P02–04; CLR-02–04; WP-IP02–05; SP-02/03; CP-02; WP-H02; GG-02 | Accepted path/color/scalar/value/category/topology contracts; grammar stages and profile migration |
| C — D3 families and grammar consumers | WP-S01–07; SP-04–06; CP-03/04; WP-AX02–04; WP-H03–06; GG-03/04/06/07/08 | Complete kernels, independent aesthetics and initial statistical/geometry routes |
| D — ggplot2 breadth and presentation | GG-05/09/10/11/12/13/14/15/16/17; begin each only when its prerequisites pass | Full analytical layers, guides/facets/coordinates, math/geography, authoring and devices |
| E — Live and cross-host integration | WP-16–20 accepted; SP-07, CLR-05, CP-05, WP-AX05/06, WP-S08, WP-H07/08, WP-IP06/07; GG-18/19 | Eight D3 gates and G-GGPLOT; G-PARITY capability gate |
| F — One final release sequence | WP-21 and WP-22, then WP-23 | Supported-platform fidelity, measured performance, docs/migrations and G4 |

The D3 package tables retain all their local edges. In addition, make **CLR-04 an
explicit prerequisite of SP-04**: the scale plan's deliverables consume its descriptors
and paint lowering even though its older prerequisite list omitted that edge. It adds
no cycle: CLR-04 requires CLR-03, while CLR-05 consumes SP-04.

Critical cross-lane chains to preserve:

- CLR-03 + WP-IP02 → WP-IP04; WP-IP03/04 + CLR-04 + SP-02/03 → SP-04.
- SP-04 → SP-05 → SP-06; SP-03/05/06 + WP-AX01 → WP-AX02 → WP-AX03/04.
- WP-P04 → WP-S01; WP-S03/04 + WP-H03–06 + WP-16/18 → WP-H07.
- WP-IP02/05 + WP-AX04 + WP-15/19 → WP-AX05; WP-AX05 + SP-07 + WP-20 → WP-AX06.
- SP-07 + CP-04 + WP-20 → CP-05; CLR-04 + SP-04 + WP-20 → CLR-05.
- WP-IP06 + SP-07 + WP-AX06 + CLR-05 + CP-05 → WP-IP07.

Do not make SP-07, CLR-05, CP-05 or WP-AX06 depend on WP-IP07. Do not make D3
certification depend on GG-19. Those backward edges would turn shared consumers into
cycles. GG integration can begin using accepted kernels before all D3 gates close;
GG-19 checks their accepted results before the cumulative gate.

## 6. ggplot2 work packages

Every package owns focused core fixtures, versioned descriptors, a real public consumer,
relevant binding tests, documentation and ledger updates as its implementation lands.
GG-18/19 broaden evidence across combinations; they are not permission to defer all
portable APIs, provenance or visual checks until the end. Paths below name existing
ownership areas; new modules should appear only with working behavior.

### GG-00 — Reference inventory and executable oracle

Prerequisite: P2-00. Requirements: GG2-01/12. Owns proposed `fixtures/ggplot2/` and
development reference scripts. Pin ggplot2 4.0.3 source plus R, scales, colorspace,
farver, viridisLite/RColorBrewer and required model/spatial/device dependency versions
as applicable to the final inventory. Record checksums, licenses, platform/font/RNG/
timezone inputs and session information. A missing R executable blocks regeneration,
not independent Rust development from committed fixtures; no oracle has run here.

Generate export/argument/default/generated-field coverage and comparable built-layer,
scale, guide, panel and geometry records, plus SVG/PDF/PNG reference artifacts. Avoid
unstable R object addresses and grob names as semantic identity. FIX-GG00 passes when
regeneration is deterministic, offline Rust comparison works, every inventory row has
an owner and genuine gaps are reported as open. Pinning a version is not parity.

### GG-01 — Reconcile shared legend acceptance

Prerequisite: WP-12/13 evidence; independent of P2-00/GG-00 and D3 delivery.
Requirements: GG2-04, GRA-07, SCL-05, LAY-03, THM-03. Owns `layout/engine.rs`,
`layout/facets.rs` and common guide layout. The current engine already shares
single/faceted measurement and painting, with a passing ordinary/faceted labels and
source-provenance regression in `chart-export/tests/authoring.rs`. Retain GGP-01's
probe as historical evidence and reconcile it against that implementation. Do not
extract another painter. Preserve semantic guide identity, clipping and tight-layout
diagnostics. Complete FIX-GG01: two-entry scatter guide, one-panel facet control,
empty/hidden guide, tight layout, shared and incompatible guides; assert scene content
and inspect SVG/PDF/PNG. Repair only reproduced remaining failures. The focused
regression does not close this full matrix; acceptance reconciliation can proceed
independently of P2-00 and coordinate with the active AP owner.

Delivered 8 September 2026: the [GG-01 acceptance report](../evidence/phase-2-entry-and-legends-2026-09-08.md)
records the complete specified matrix, repaired empty/untitled cases, actual host
proofs and inspected publication artifacts. Full guide extensions remain GG-05.

### GG-02 — Compatibility profile, stages and inferred grouping

Prerequisites: GG-00. Requirements: GG2-01/02. Owns grammar/compiler and portable
profile contract, extending `plot/mapping.rs`, `plot/stat.rs` and `plot/wire.rs`.
Reuse existing source/after-stat/after-bin typed reads and add the remaining source,
after-stat, after-scale and theme-derived expression semantics with checked types,
bounded operations and cycle detection. Define
scale transform/OOB/limits before statistics where the ggplot2 profile requires it;
coordinate limits remain post-stat view operations. Carry transformed-space metadata
to prevent double application. Infer groups from eligible discrete aesthetics with
explicit override; define inheritance, default stat/geom pairing and orientation.
Keep R evaluation in the oracle only. FIX-GG02: log histogram versus coordinate-log,
scale limit versus zoom, colored lines without explicit group, multiple discrete
aesthetics, stat-specific group override, missing values and legacy v1 round trips.

Before introducing a profile beyond `Profile::LibraryV1`, implement
[ADR-014](../adr/014-phase-2-integration-contract.md): resolved execution policies and
immutable profile provenance live in the canonical definition. Today primary interchange stores profile separately;
`Plot::chart`, `Chart::apply_plot` and `Output::request` pass normalized definitions.
This is an integration prerequisite, not a demonstrated LibraryV1 defect. Implement
the selected policy consistently through runtime edits, worker/cache identity, wire
migration and static/live capture. FIX-GG02 must verify primary and legacy envelope
round trips, profile changes after an edit, static output and both Presented/Current
captures, including retained old snapshots after later changes/disposal. Preserve
resource/profile provenance; adding enum variants alone does not change semantics.

### GG-03 — Independent aesthetic encodings

Prerequisites: GG-02, SP-04, WP-S05. Requirements: GG2-03. Owns grammar mappings,
resolved styles and generated schemas. Add independently trained fill, stroke/color,
alpha, shape, linetype, size, linewidth and text-related channels, with aesthetic units
and constant/mapped precedence. Separate area/radius/linewidth semantics. Share mapped
symbol geometry with WP-S05, extending it once for reference-specific glyphs if needed.
FIX-GG03: sizes 1/4 under area mapping yield area ratio 1:4; legacy radius mapping
retains 1:16; independent outline/fill/alpha and size/width changes retain correct hits,
source identity and legend metadata in Rust/Python/WASM.

### GG-04 — ggplot2 scale and palette policies

Prerequisites: GG-03, SP-06, CP-04, CLR-04. Requirements: GG2-03. Owns adapters in
scales/grammar, not a new scale engine. Cover positional/nonpositional continuous,
discrete, binned, manual and identity policies; expansion, limits/OOB/NA, breaks/labels,
rescaling, area/radius sizing, reversible and transformed secondary axes, date/time
controls. Implement ggplot2-specific palettes and gradient behavior absent from CP,
including all reference viridis options and hue/grey/Brewer policies. FIX-GG04: uneven
gradient stops, reference Lab midpoint, bins/tails, unused levels, size-zero behavior,
custom monotone secondary transformation and DST folds/gaps with explicit resources.
Validate every reference argument and rejection rule in GG-00's inventory.

### GG-05 — Complete guides and legend composition

Prerequisites: GG-01, GG-04, WP-AX04. Requirements: GG2-04. Owns shared guide grammar,
layout and component metadata. Add continuous colorbars, stepped/binned guides,
multi-aesthetic keys, per-layer legend inclusion/overrides, key glyphs, title/order/
direction/reversal/placement and compatibility-aware collection. Add ggplot2 axis,
log-tick, stacked-axis and custom guide policies through AX components; angular guides
complete with GG-13. Sample the same mapping as marks, preserving discontinuities.
FIX-GG05: single/faceted equivalents, mixed fill/shape/size keys, asymmetric diverging
colorbar, binned interval labels, inside/outside guides and crowded/clipped labels;
inspect components and vector/raster output. No stop-swatches-only colorbar claim.

### GG-06 — Bin/count/summary and position semantics

Prerequisites: GG-02, SP-03, WP-S06. Requirements: GG2-02/05. Owns statistics and
position adapters. Add bin closure, boundaries/centers/binwidth, padding, weights,
count/density/ncount/ndensity fields; grouped/binned summaries and summary helpers.
Provide nudge, dodge2, jitter-dodge and all reference dodge/stack/fill orientation,
width/preserve/reverse/vjust controls. Reuse compatible stack kernels; do not equate
D3 expand with legacy normalize or ggplot2 fill by name alone. Make RNG algorithm,
seed and draw-order policy explicit; a stable-key jitter policy is not an exact R RNG
claim. FIX-GG06: `[0,1,2]` at breaks `[0,1,2]` yields profile counts `[2,1]` and legacy
`[1,2]`, weighted/nonuniform density integrals, signed/zero stacks, variable-width
dodge2, empty slots, jitter reproducibility and retention versus fresh computation.

### GG-07 — Primitive and interval recipe completion

Prerequisites: GG-03, GG-06, WP-S02/03/05. Requirements: GG2-06. Owns built-in geoms
and recipes over shared shapes. Complete intervals/errorbars/crossbars/pointranges,
reference lines, steps, segments/curves/spokes, grouped polygons with holes, rugs,
blank layers that still train scales, count marks, tiles/rectangles/raster and related
orientation/arrow/stroke/fill controls. Layer/stat overrides remain composable.
FIX-GG07: horizontal and vertical intervals, data-space slopes, three step modes,
hole misses, arrow endpoints, raster pixel extents and blank-domain contribution;
native/headless scenes and targets agree. Include source/stat-derived variants.

### GG-08 — Data-driven text, labels and annotations

Prerequisites: GG-03, WP-P04. Requirements: GG2-06/09. Owns text geoms/composition
and the existing text service boundary. Map labels over source and generated rows;
support label boxes, padding, justification, rotation, overlap controls, units,
nudging and custom/raster annotations. Resource and output-size budgets replace the
assumption that all data labels fit the authored 256-annotation limit. FIX-GG08 checks
row counts, stat labels, empty/Unicode text, rotated boxes, dense overlap and physical
sizing in native/text-and-outline exports. Mathematical expressions follow GG-14.

### GG-09 — Distributional and one-dimensional analytical layers

Prerequisites: GG-06/07. Requirements: GG2-05/06. Owns built-in boxplot, KDE/density,
violin/dotplot, ECDF, QQ/QQ-line, function sampling, unique/connect and area alignment
statistics plus recipes. Specify each generated schema, weighting, estimator/bandwidth,
kernel/grid/boundary, trim/normalization, grouping/orientation and degeneracy policy.
Quantiles alone do not implement whiskers/outliers; normalized histograms are not KDE.
FIX-GG09 uses independent Tukey/ECDF/quantile anchors, numerical density mass checks,
pinned estimator curves, tied/sparse/constant samples, grouped align output and
function/distribution registrations. Prove membership versus derived provenance.

### GG-10 — Smoothers, confidence bands and quantile regression

Prerequisites: GG-06/07. Requirements: GG2-05/06. Owns reusable chart-statistical model
operations and adapters. Deliver weighted linear/generalized-linear models, LOESS,
the reference automatic GAM route and quantile regression; support documented model
parameters, formula capabilities through typed model terms/registrations, prediction
grids and uncertainty output. Spike numerical dependencies first; no finance engine
or interpreter is permitted in core. A registered arbitrary model hook supplements
the built-ins, rather than replacing required default algorithms.

The [smoothing reference](https://ggplot2.tidyverse.org/reference/geom_smooth.html)
selects LOESS below 1,000 observations and a GAM otherwise using the largest group;
GG-00 pins that behavior and its model dependencies. FIX-GG10 covers both sides of
the threshold across facets, weighted exact linear examples, confidence levels,
link-scale intervals, nonlinear curves, quantile objectives, rank deficiency and
solver convergence diagnostics. Test zoom versus filtering and explicit model overrides.
Unknown formula/model options remain open capability rows, not silent OLS fallbacks.

### GG-11 — Two-dimensional statistics and contours

Prerequisites: GG-06/07/09. Requirements: GG2-05/06. Owns rectangular/hex bin counts
and summaries, 2D KDE, contour/isoband and ellipse statistics/geoms. Define grid/kernel/
bandwidth, weighting, threshold/interpolation and contour topology contracts. Reuse
shared paths and filled polygons. FIX-GG11: independently known cell/hex membership,
saddle cells, closed rings/holes, discontinuous grids, empty bins, degenerate covariance
and known ellipses; inspect density contour/filled output and derived inspection targets.
Bounds and budgets count generated grid cells/segments, not just input rows.

### GG-12 — Facet semantics and layout breadth

Prerequisites: GG-02, GG-05, GG-06. Requirements: GG2-07. Owns facet grammar/layout.
Add multiple row/column variables, combinations/nesting, margins, shrink, proportional
free space, row/column scale-sharing constraints, panel direction/order, strip position,
interior axes/labels and labeller descriptors/registrations. Retain stable panel keys
and explicit legacy broadcast semantics; compatibility inference is profile-owned.
FIX-GG12: absent combinations, NA/unused levels, raw versus stat domain with shrink,
unequal domain spans, margin aggregates without accidental double counting, missing
facet fields and wrapped labels. Parsed labellers complete with GG-14. Verify resize,
selection/provenance and shared-guide identity in actual hosts.

### GG-13 — Cartesian, transformed and polar/radial coordinates

Prerequisites: GG-07, GG-12, WP-S04, WP-AX04. Requirements: GG2-08. Owns core
coordinate protocol and projection/clip/hit integration. Add fixed aspect, flip,
post-stat nonlinear coordinates and full polar/radial panels, angular/radial guides,
partial circles, inner radius, direction/start and label rotation controls. Separate
scale transform from coordinate transform and source filtering from clipping.
Subdivide paths adaptively with destination-unit error/work bounds; handle seams,
holes, noninvertible mappings and partial inverse capability explicitly. FIX-GG13:
data circles remain circles under fixed aspect, flipped intervals, transformed straight
segments, radial bars/ribbons, full/partial seams, inner-hole misses, guide placement
and consistent pointer/keyboard/export geometry. Share subdivision across geoms.

### GG-14 — Theme hierarchy and mathematical typography

Prerequisites: GG-05/08/12/13. Requirements: GG2-09. Owns theme/text/layout contracts.
Add the reference element inheritance, blank/inherit-blank, relative units, margins,
component/geom defaults, all complete theme presets and per-guide/strip/axis controls.
Use explicit theme contexts for get/set/update/replace behavior rather than process
global state. Implement typed mathematical layout covering the reference parsed-label
capabilities, including scripts, fractions, operators, accents and delimiters; supply
fonts explicitly and preserve logical text/accessibility. Reuse the text service for
labels, axes, strips and annotations; do not substitute literal TeX strings.
FIX-GG14: inheritance/blank/relative-unit matrices, identical stats across themes,
math in each consumer, multilingual/rotated labels, measured bounds and inspected
native plus SVG/PDF text/outline and 300/600 DPI output.

### GG-15 — Geographic layers and coordinates

Prerequisites: GG-07/08/13. Requirements: GG2-08. Owns typed geometry input,
projection service and map/sf adapters. Add feature geometries/multiparts/holes,
map joins/borders, sf-equivalent stat/label/coordinate extraction and projected panels,
CRS/default-CRS/axis-order/limit rules, graticules, clipping and antimeridian behavior.
GG-00 inventories documented map/projection methods; implementation includes equivalent
capabilities without requiring an R sf object. Supply projection resources explicitly;
record the selected dependency and resource provenance in an ADR after a WASM/native
spike. No implicit map downloads, filesystem lookup or GIS engine in unrelated charts.
FIX-GG15: known projected anchors, mixed CRS, dateline-crossing polygons and holes,
graticule labels, non-sf overlays, invalid geometry, selection and high-resolution
exports across hosts. Unavailable required projections keep parity open.

### GG-16 — Extensibility and authoring conveniences

For the primary-API refactor, AP-03/04 integrate these capabilities into the main
builders and AP-07 owns the host-native syntax. GG-16 owns the extension/dispatch
semantics and their evidence; it must not introduce a competing authoring surface.

Prerequisites: GG-02/05/07/12/13. Requirements: GG2-10. Owns registry/public builders,
typed recipe dispatch and external examples. Extend existing stat/geom registration
to scales, coordinates, facets, guides, labellers, models and key glyphs with declared
schemas, training/inversion, resource/invalidation and error contracts. Add inspectable
built-layer/plot data, labels/alt text, composable typed authoring, materialization/
autoplot/autolayer-style dispatch and vector helpers (cut/resolution/summary) through
the common engine. No R object system or second Python/JS compiler.
FIX-GG16 exercises an external custom example at each extension boundary through Rust
and registered Python/WASM, malformed outputs, callback context, native-only rejection,
deterministic repeated preparation and copy/disposal. GG-18 supplies later model/math/
geographic extension combinations; a generic registration struct is insufficient.
Reuse delivered composition, labels/alt text and built-layer inspection routes where
present; charge only the missing dispatch/protocol semantics and their integration.

### GG-17 — Saving and device capability completion

Prerequisites: GG-08/14/15, WP-20. Requirements: GG2-11. Owns chart-export and explicit
host save adapters. Extend existing Output/ExportOptions/FigureRequest and artifact
save routes. Preserve SVG/PDF/PNG; inventory and deliver reference device
capabilities for JPEG/TIFF/BMP, PostScript/EPS, supported metafile/TeX routes and custom
device hooks with a per-platform matrix. The [saving contract](https://ggplot2.tidyverse.org/reference/ggsave.html)
supplies dimensions/units/scale/DPI/background/size-limit and filename/device behavior;
the host resolves path creation, device selection and an explicit current figure.
Do not emulate hidden R session state. Spike unsupported encoders and validate licenses
before adopting them. Device-specific support must match its declared reference host;
Windows-only devices do not imply a Windows GPUI product. WASM retains its scene/SVG
baseline; explicitly record other device availability rather than inventing runtime support.
FIX-GG17 checks physical dimensions, extension selection, transparency/color policy,
fonts, vector retention, multi-page/numbered output where supported, independent
decoding and inspection, custom device errors and export during ingestion. Required
device rows cannot disappear into a generic unsupported result.

### GG-18 — Full grammar, update and host integration

Prerequisites: GG-03–17, WP-16/17/18/19/20. Requirements: GG2-02–12. Owns integrated
fixtures, gallery, portable adapters and generated API/schema evidence. Combine geoms,
stats, aesthetics, guides, facets, coords, themes, extensions and snapshot outputs;
test actual Rust/Python/Node WASM behavior, single-thread execution, large IDs, resource
lifetimes and migrations. FIX-GG18 compares append/upsert/remove/retention to batch,
source/filter/zoom differences, selection of aggregates/derived paths, coherent old
snapshots, invalid update rollback and disposal. Recompute global models exactly until
an optimization has independent equivalence and measurements. Native interactions
and publication must consume the same presented geometry and semantic revision.
Exercise primary Data/Plot/Chart/Output usage and standalone operations through actual
package exports, conversions and generated declarations/stubs. Reuse AP-07 syntax and
proof infrastructure and retain legacy interchange compatibility. Carry alternate
profile policies and migrated scale/guide identities through these paths. Verify the
Presented/Current × visible/full-domain × interaction-inclusion capture matrix with
new semantics; existing export coverage is the starting regression, not work to rebuild.

### GG-19 — Capability certification and handoff

Prerequisites: GG-00–18, WP-P04, WP-S08, SP-07, CP-05, CLR-05, WP-IP07,
WP-AX06, WP-H08. Requirements: GG2-01–12. Owns final coverage report and capability
evidence, not another algorithm implementation. FIX-GG19 joins the export/argument
inventory with actual evidence; every built-in capability is equivalent or has a
tested language adaptation preserving capability. Missing/failed/uncertain required
rows block G-GGPLOT. Report unsupported environments and precision adaptations
separately from missing features. Prepare workload definitions/budgets for WP-22 and
cross-platform visual corpus for WP-21. Final production certification remains G4.

## 7. Acceptance protocol and cumulative gates

Each FIX-GGnn belongs to GG-nn. Store reference version/hash, profile, operation and
parameter settings, stage-space metadata, data/seed/resource identity, expected output
or diagnostic, requirement IDs, numerical tolerance and destination verdict. Expected
failures establish the backlog; they are not passing fixtures. Each family requires
standalone statistical/scale output where meaningful plus actual composed chart output.

Use four independent evidence layers:

1. **Semantic:** reference built-layer results and independent analytical anchors;
   compare membership, group/panel identity, schemas, breaks and strings exactly;
   justify numerical tolerances per estimator/operation and conditioning.
2. **Geometry/layout:** resolved transforms, interval/path topology, panel allocations,
   guides, physical units and text metrics with declared fonts. A matching image does
   not certify the statistical population or correct interval closure.
3. **Destinations:** actual Rust/Python/WASM calls and inspected macOS native,
   SVG/PDF/PNG artifacts; device-specific proofs on their supported hosts. Record
   raster/font/antialias tolerances separately from numerical or color differences.
4. **Updates/resources:** batch equivalence, reference RNG policy, equivalent-history
   resquarify checks, coherent capture, failure/disposal, bounded allocations/work and
   measured supported workloads. Fresh squarify is not an oracle for retained history.

| Gate | Prerequisites and meaning |
| --- | --- |
| G-GGPLOT | GG-19 accepted; all GG2-01–12 / FIX-GG00–19 capability evidence complete |
| G-PARITY | G3, G-PATH, G-SHAPE, G-COLOR, G-INTERPOLATE, G-SCALE, G-CHROMATIC, G-AXIS, G-HIERARCHY and G-GGPLOT accepted; unlocks final WP-21/22 acceptance |
| G4 | G-PARITY plus WP-21 supported-platform/fidelity, WP-22 measured performance and WP-23 API/schema/migration/release evidence; final Phase 2 production readiness |

Retain PERF-01–05 and the existing D3 supplemental workloads. Add size-series for
weighted bins, KDE grids, LOESS/GAM/quantile fits, 2D density/contours, many facet/guide
components, mathematical labels and projected geometry. Record source rows, generated
vertices/cells, model iterations, allocations, cold/update/layout/hit/export costs
and retained resources. Establish workload-specific numerical/work/latency budgets
before acceptance; do not apply simple-line frame targets indiscriminately to global
model fitting or remove expensive families from the coverage claim.

Implementation uses existing `mise run fmt`, `mise run check`, `mise run test` and
`mise run bindings-proof`, expanded with real family cases. macOS/Linux core/export
execution, actual native interaction, reference regeneration and device inspection
need their respective environments. Add named parity tasks only when runners exist.
Never count a missing R, native display, encoder or WASM capability as a passing skip.

## 8. Estimates, risks and first handoff

Keep the original D3 estimates as provisional lane evidence: shape 8–13 developer-weeks;
path 9–16 days overlaps that foundation; color 13–22 days; interpolation 27–45 days
includes work transferred out of scales; chromatic 9–16 days; axis 14–26 days;
hierarchy 35–57 days; scale needs re-estimation after SP-01. Do not sum overlapping
path/shape, scale/interpolation or oracle/host integration work twice.

New ggplot2-only planning allowances below exclude D3 kernels, Phase 1, P2-00 and
WP-21–23. These are historical allowances before the current API reconciliation,
including focused fixtures/review/host integration. P2-00 must subtract delivered AP
foundations and legend work, assign shared binding work once and estimate only remaining
integration/acceptance. They are not current remaining-effort or elapsed-time promises.

| Packages | Additional engineer-days | Main uncertainty |
| --- | --- | --- |
| GG-00/01 | 5–8 / 1–3 (historical) | Oracle reproducibility; GG-01 now needs acceptance reconciliation and any reproduced residual fixes |
| GG-02/03/04/05 | 8–14 / 5–9 / 6–10 / 7–12 | Stages, migration, reference color policies and guide layout |
| GG-06/07/08 | 8–13 / 6–10 / 4–7 | Weighted/position defaults and geometric/text consumers |
| GG-09/10/11 | 12–20 / 20–35 / 14–24 | Estimator/model dependencies, contour numerics and topology |
| GG-12/13/14 | 8–14 / 12–20 / 15–25 | Scale sharing, transformed clipping and mathematical text |
| GG-15/16/17 | 18–30 / 8–14 / 10–20 | CRS portability, external protocols and device capabilities |
| GG-18/19 | 10–18 / 5–9 | Full cross-host/update combinations and inventory completeness |

Historical GG allowance totals **182–315 engineer-days**, plus P2-00's 2–4 days.
Remaining effort is unestimated pending the delivered-work reconciliation. Re-estimate
after GG-00 and the model/math/geography/device spikes; unverified dependency capabilities and any
newly inventoried arguments remain explicit cost/schedule risks. Do not convert the
original 16–24-week project estimate into a Phase 2 promise. Resource limits and
performance evidence determine supported workload sizes, not permission to omit families.

P2-00's integration contract and GG-01's specified acceptance are delivered against
the committed primary API baseline; see the [entry/legend report](../evidence/phase-2-entry-and-legends-2026-09-08.md).
Next parity engineering handoff: **GG-00 and the existing D3 entry packages** in
prerequisite order. Preserve the AP implementation and link its delivered capability
rows. No duplicate runtime extraction, authoring facade or legend rewrite belongs in
this handoff; full G-AUTH is not a prerequisite for pure parity kernels.
The model, math, geographic and device spikes are bounded implementation tasks inside
their packages once their prerequisite contracts exist, not reasons to pause this plan.

## 9. Planning evidence and limitations

Read all eight D3 plans, the ggplot2 review and retained probe references, current
specification/main plan/ledger, ADR-005/006/012 and repository validation entry point.
Checked the official ggplot2 index (displayed 4.0.3), tagged namespace, aesthetic-stage,
smoothing and saving documentation. One direct geographic documentation URL failed;
GG-00 must verify the complete release source and dependency lock for that family.

The original planning task provided source synthesis and intended acceptance, not an
executed oracle or a fresh implementation audit. Exact documentation validation commands/results are
recorded in the ledger. No R/D3 suite, Rust feature test, native UI, export inspection,
Python/WASM runtime, Linux execution or performance benchmark was run by this planning
assignment. No implementation gate was advanced; historical review evidence retains
its original revision and scope.

The current API review subsequently ran 24 focused Rust tests and records their exact
revision, commands and limitations. This plan revision consumes that evidence and
rechecks live source ownership, including ongoing host-wrapper additions; it does not
rerun or expand those test claims. Documentation validation for this update belongs in
the ledger. Package and gate completion remain evidence-driven and unchanged here.
