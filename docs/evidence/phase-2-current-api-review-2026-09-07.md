# Phase 2 plan against the current package API

Date: 7 September 2026. Review baseline: `b631f0e6d7e41722b5774433d616f704234157d7`
plus the active primary-authoring working tree. This reviews plan applicability and
public API integration; it is not full algorithm, binding or release certification.
Source line references describe the inspected working tree and may move with concurrent work.

## Verdict

**The parity scope remains applicable, but its first implementation handoff is stale.**
The current API already supplies primary data/plot/component builders, typed runtime
ownership and publication destinations. The Phase 2 plan's newer AUT/AP addendum correctly
requires future capabilities through this surface. Extend those owners; do not implement
another authoring layer or re-extract a runtime from Session.

The earlier missing single-panel legend now has a shared implementation and a passing
ordinary/faceted export regression. Treat GG-01 as acceptance reconciliation and remaining
edge/visual verification, rather than a new legend-painter implementation. This review
does not close its entire FIX-GG01 matrix or change package/gate states.

## Required planning correction

**P2 — Replace the stale GG-01 implementation kickoff.** Related contracts: GG2-04,
GRA-07, SCL-05, LAY-03, THM-03. The [Phase 2 first handoff](../impl_plans/phase-2-parity-implementation-plan.md#8-estimates-risks-and-first-handoff)
still directs implementation of the old defect and coordination with active WP-16.
Current [engine.rs](../../crates/chart-core/src/layout/engine.rs), lines 514–537,
prepares and finishes a single-panel legend; [facets.rs](../../crates/chart-core/src/layout/facets.rs),
lines 185–239, reuses the shared painter. The
[export regression](../../crates/chart-export/tests/authoring.rs), line 241,
checks ordinary/faceted labels and source provenance and passed in this review.

Impact: following the old kickoff would duplicate active authoring work and use an
obsolete assignment boundary. Reconcile GG-01 with that implementation/evidence first;
run the still-required empty/hidden/tight/shared/incompatible-guide and inspected-output
cases before closing it. Start new parity engineering at P2-00/GG-00 and the D3 entry
packages, coordinating with the current AP owner. Keep the original review/probe as
historical evidence and refresh the operational handoff separately.

## Current API routing and remaining scope

Pass below means the stated existing route has source or focused runtime evidence;
it never means complete D3/ggplot2 equivalence. Partial/Fail concern the proposed parity
capability, not a regression against LibraryV1.

| Phase 2 owner | Current public route / source | Assessment and integration action |
| --- | --- | --- |
| P2-00 / GG-16 | `chart_core::prelude`, `Data`, `plot`, `Plot`, `Chart`; [exports](../../crates/chart-core/src/lib.rs), [plot](../../crates/chart-core/src/plot/mod.rs), [runtime](../../crates/chart-core/src/runtime.rs) | Pass for primary foundation in focused tests. Reuse AP-00's capability register and AP-01 ownership; GG-16 adds extension/dispatch semantics, not another facade. |
| GG-02 | `Profile::LibraryV1`, `PlotBuilder::profile`, inherited `aes`, `group`/`group_all`, `after_stat`, `after_bin`; [mapping](../../crates/chart-core/src/plot/mapping.rs), [stat](../../crates/chart-core/src/plot/stat.rs) | Partial. Typed generated-stage reads already exist. General expressions, after-scale evaluation, inferred groups and compatibility policies remain new work. Preserve compile-time separation of source and generated fields. |
| GG-03 | `AesBuilder` x/y/endpoints/low/high/size/group/color and `LayerBuilder::style`; [mapping](../../crates/chart-core/src/plot/mapping.rs) | Fail for full independent aesthetic parity. Fill/stroke/alpha/shape/linetype/size/linewidth training still needs kernels and new options in these existing builders. Constant theme symbols do not close it. |
| SP / CLR / IP / CP / GG-04 | `ScaleBuilder`, numeric/time/category constructors, `color_discrete`, `color_continuous`; [axis](../../crates/chart-core/src/plot/axis.rs), [color](../../crates/chart-core/src/plot/color.rs), [scales](../../crates/chart-core/src/scales/mod.rs) | Partial. Existing builders wrap current scale families and byte palettes. Full D3 families/color values/interpolation/catalogs remain open. Add standalone public family APIs and integrate them into these components. |
| AX / GG-05 | `AxisBuilder`, `AxisHandle`, `x_axis`, `y_axis`, separate `legend`; [axis](../../crates/chart-core/src/plot/axis.rs), [legend](../../crates/chart-core/src/plot/composition.rs) | Partial. Shared single/facet legend painting now exists. Guide identity split, colorbars, key glyphs, independent guide geometry/styles and transitions remain required. |
| GG-06 | `bin`, `count`, `summary`, `fit`, `stack`, `dodge`, `jitter`, `transform`, `filter`; [stat](../../crates/chart-core/src/plot/stat.rs), [transform](../../crates/chart-core/src/plot/transform.rs) | Partial. Existing stat/position/transform authoring is reusable; weighted/reference bin policies, density fields, dodge2/nudge/jitter-dodge are additions. |
| P / S / GG-07/09/10/11 | `points`, `line`, `area`, `ribbon`, `bars`, `ohlc`, `rule`, `rectangle`, `cells`, `histogram`, `volume`; [layers](../../crates/chart-core/src/plot/layer.rs) | Partial. Existing recipes lower to the same grammar. Complete paths/shapes and analytical kernels are still absent from the current public family surface. Keep all new recipes on those shared kernels. |
| GG-08 / GG-14 | `labels`, `callout`, title/subtitle/notes/panel letters, rich text/style; [composition](../../crates/chart-core/src/plot/composition.rs), [text](../../crates/chart-core/src/plot/text.rs) | Partial. `labels()` constructs one fixed annotation, not a source/stat-row text geom. Mapped labels/math remain open. Preserve AUT-04's separate title/axis/legend components. |
| GG-12 | `facet_wrap`, `facet_grid`, inferred catalog, order/empty/free/gap/collect controls; [facet](../../crates/chart-core/src/plot/facet.rs) | Partial; focused catalog tests pass. Multiple facet variables, margins, shrink, proportional space and richer labellers remain new work. Do not count existing catalog inference as absent. |
| GG-13/15 / H | `coord_cartesian` and shared coordinate capability; [options](../../crates/chart-core/src/plot/options.rs), [coordinates](../../crates/chart-core/src/layout/coordinates.rs) | Partial Cartesian foundation; full transformed/polar/geographic coordinates and hierarchy engines remain open. Extend primary components alongside standalone kernels. |
| GG-17/18 | `Output`, `export_options`, `CaptureBasis`, `FigureRequest`, artifact `save`; [export](../../crates/chart-export/src/authoring.rs) | Pass for focused static/live SVG/PDF/PNG routes. Preserve Presented/Current independently of visible/full-domain and interaction capture. Additional devices remain new work; do not rebuild existing capture ownership. |
| GG-18 / AP-05/06 | `Chart::apply_plot`, transactions, actions, queries, preparation/presentation; `ChartInput::from_plot`, optional Kit mount; [runtime](../../crates/chart-core/src/runtime.rs), [native](../../crates/gpui-charts/src/view.rs), [Kit recipe](../../examples/chart-gallery/examples/family_gallery.rs) | Focused runtime tests pass. Extend invalidation, targets and captures for new semantics; do not create another queue/store/reducer. Native runtime requalification remains open in this review. |
| All portable families / AP-07 | Rust `plot::host` and export host dispatch; public [Python](../../crates/chart-python/src/lib.rs) and [WASM](../../crates/chart-wasm/src/lib.rs) wrappers | Partial infrastructure only. Actual Python/WASM wrappers currently expose JSON-based `Chart` operations; primary host-native builders and standalone parity APIs are not registered there. |

## API-specific integration prerequisites

1. **Profile lifetime — GG-02 / GG2-01/02, AUT-03/06, BND-01.** The existing
   [Profile enum](../../crates/chart-core/src/plot/mod.rs), lines 79–85, only supports
   LibraryV1. Primary [interchange](../../crates/chart-core/src/plot/wire.rs), lines
   11–25, stores that profile separately from the normalized definition.
   `Plot::chart` (plot/mod.rs:195), `Chart::apply_plot` (runtime.rs:173), and static
   `Output::request` (authoring.rs:219) carry the normalized definition into execution
   or capture, without carrying a separate semantic profile. This is not a demonstrated
   LibraryV1 defect: there is no alternative profile to lose today. Before adding D3/GG
   variants, specify whether all policies lower into the canonical definition or an
   immutable profile descriptor travels with it. Verify primary interchange, legacy
   envelopes, runtime edits, worker/cache identity, static export and both live capture
   bases. Merely extending the Profile enum cannot implement alternate semantics.

2. **Scale/guide identity migration — WP-AX01 / AXIS-01, AUT-01/04.**
   `AxisBuilder` holds an `AxisSpec`; `AxisHandle` wraps `ScaleId`, and naming an axis
   assigns a scale identity ([axis.rs](../../crates/chart-core/src/plot/axis.rs):209–261).
   `Plot::named_axes`, wire name maps, layer axis selection and host navigation consume
   those identities. The planned many-guides-to-one-scale split must update these
   concrete consumers in addition to low-level layout types. Keep existing axis names
   and layer scale bindings valid during migration; prove a shared top/bottom pair
   through primary Rust and actual host builders. Do not make layer coordinates bind
   to a decorative guide identity.

3. **Host-native delivery — AP-07 plus each semantic package / AUT-07, BND-03/04.**
   Python registers only `Chart` and `ChartError` (lib.rs:127–130); its constructor
   receives definition/data/publication JSON strings (lib.rs:23–33). WASM's exported
   constructor has the same boundary (lib.rs:19–27). New Rust host-dispatch types are
   not proof of usable Python/JavaScript exports. Coordinate AP-07's public naming,
   registrations, conversions and declaration/stub production before claiming primary
   binding reachability. Each capability should test its actual host-native operation
   and primary chart route, while retaining legacy-envelope compatibility tests.
   Pure kernels need not wait for G-AUTH; neither G-AUTH nor G-PARITY should depend on
   the other's final certification.

These are concrete implementation prerequisites and currently open capability surfaces,
not newly reproduced runtime failures. The plan already includes a primary-API rule;
the next plan revision should attach these source owners and cases to its package
handoffs and avoid charging for authoring foundations already delivered under AP.

## Verification and limitations

Executed in `/Users/jeickmeier/Projects/finstack-chart`, Darwin arm64, using the mise
toolchain and committed lockfile:

| Command | Result |
| --- | --- |
| `mise exec -- cargo test -p chart-core --test authoring --test authoring_runtime --test authoring_host --locked` | 20 passed: 11 authoring, 7 runtime, 2 Rust host-dispatch tests; zero failed/ignored |
| `mise exec -- cargo test -p chart-export --test authoring --locked` | 4 passed: publication formats, capture preconditions, 8-way live capture independence/disposal, ordinary/faceted legends; zero failed/ignored |
| `mise exec -- python3 scripts/check_repository.py` | Passed dependency/host-isolation graph and local Markdown file-link checks; not target execution or external-link/anchor validation |
| `git diff --check -- docs/implementation-status.md docs/evidence/phase-2-current-api-review-2026-09-07.md` | Passed for tracked changes; the new review document was inspected separately |

The two host-dispatch tests run as Rust tests, not Python or WASM. Export tests exercise
real encoders but this review did not visually inspect their output. No fresh actual
Python/WASM runtime, native UI, Linux execution, D3/R oracle or performance gate ran.
The working tree is concurrently changing; results belong to the test binaries built
by these commands. This is not a claim about every later edit or a published package.

Read-only review outcome: retain the Phase 2 capability scope; refresh the stale
GG-01 kickoff and map future work to the current primary API owners above. No production
code, plans, baselines, versions or gate states changed by this review. This evidence
file and the mandatory ledger entry are the only review-authored repository changes.
