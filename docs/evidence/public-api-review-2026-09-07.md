# Public API usability review — 7 September 2026

The current API exposes a capable chart engine, but does not yet meet the owner's goal
of authoring charts as simply as ggplot2. The main problem is the amount of engine
configuration required before expressing a chart. Retain the shared engine and add a
small authoring surface that owns routine setup and lowers into existing contracts.

This is a review of the live working tree over
`127fe2d4853f89b62ba59a17248485e3c378ba60`, including existing uncommitted work.
No implementation, dependency, fixture or existing default was changed. Only this
review and the required status-ledger entry were added. Severity below ranks work
against the owner's usability goal; it does not label intentional alpha contracts
as newly discovered runtime regressions.

The comparison uses ggplot2 4.0.3's official
[plot construction](https://ggplot2.tidyverse.org/reference/ggplot.html),
[aesthetic mappings](https://ggplot2.tidyverse.org/reference/aes.html) and
[saving](https://ggplot2.tidyverse.org/reference/ggsave.html) documentation, retrieved
7 September 2026. Its relevant strengths are shared plot data/mappings, composable
layers, local overrides and a short saving operation. Matching R syntax or its `+`
operator is not necessary to reproduce those benefits in Rust.

## Findings

### API-01 — High: the first chart requires manual engine identities and ingestion setup

Requirements: SCP-01, DAT-01/02, GG2-10; **Fail for the requested simplicity goal**.
The public [grammar example](../../crates/chart-core/src/grammar/mod.rs), lines 8–30,
requires dataset/field/layer IDs, source keys, revisions, a schema version, source
epoch, typed snapshot materialization and a data store for a four-value histogram.
[TypedDataBuilder](../../crates/chart-core/src/grammar/typed.rs), lines 27–31 and
64–70, takes both a field ID and name and starts from an already-created snapshot.
[ChartDefinition::new](../../crates/chart-core/src/grammar/definition.rs), line 778,
requires an authored revision. There is no default dataset on the definition;
`Layer::new`, line 644, requires data and identity on every layer.

Counterexample: adding points over a line with the same data/mappings still requires
another `LayerId`, dataset reference, geometry choice and empty or repeated mappings.
`ChartDefinition::mapped` already handles positional inheritance; it should be reused.
The cost is learning lifecycle concepts before the basic data → aesthetics → layers
workflow. Provide an owned plot/data builder with name-based fields or typed accessors,
default data and generated identities. Make keys/handles available for advanced use;
retain supplied keys and revision fences for live updates. Generated keys must be
persistent identities, not recomputed row offsets after sorting or replacement.

### API-02 — High: mapped aesthetics are split across incompatible authoring routes

Requirements: GRA-01, GG2-02/03; **Fail for complete inherited aesthetics**.
[SourceAes](../../crates/chart-core/src/grammar/definition.rs), lines 269–287, inherits
coordinates, size and group, but has no color mapping. Color lives separately on
`Layer`, line 638, and [ColorEncoding](../../crates/chart-core/src/grammar/colors.rs),
lines 24–33, requires a scale ID, input kind, complete scale and optional title.
Source size means point radius or stroke width in destination units, rather than
independently trained size and linewidth aesthetics. Fill/stroke cannot be independently
mapped through this surface.

Counterexample: coloring both a line and its points by `series` requires configuring
each layer's color encoding and separately supplying the line group. The source
compiler explicitly uses `aes.group` or `Grouping::All`
([compiler.rs](../../crates/chart-core/src/grammar/compiler.rs), lines 710 and 833).
Color alone does not establish separate lines. This is an intentional legacy default,
but an unexpected extra rule for a ggplot-style authoring surface.

Provide one inherited aesthetic builder with clear field mapping versus constant
style, independent aesthetic channels and automatic compatible scale selection.
Keep typed source/after-stat boundaries internally and expose an explicit after-stat
operation when needed. Put inferred grouping and changed scale/stat defaults in the
planned compatibility profile; do not silently change legacy definitions. A façade
alone cannot close missing aesthetic semantics in GG2-02/03.

### API-03 — Medium: routine customization requires editing protocol structures

Requirements: GRA-07, THM-01, GG2-07/09/10; **Fail for concise customization**.
[FacetSpec](../../crates/chart-core/src/grammar/facets.rs), lines 47–65, requires
fields, a complete ordered `PanelKey` catalog, layout, empty-panel policy, scale
policy, gap and guide collection. Undeclared observed keys reject. A caller asking
for panels by `region` must enumerate that region population first.

Themes and figure composition are public optional fields on
[ChartDefinition](../../crates/chart-core/src/grammar/definition.rs), lines 754–774;
its builder methods, lines 776–810, cover only mappings, transforms, axes and layers.
Axis titles live in [AxisSpec](../../crates/chart-core/src/layout/types.rs), line 87;
plot titles live in [FigureComposition](../../crates/chart-core/src/composition.rs),
line 137. Common changes require discovering and constructing several distinct types.

Add `facet_wrap(field)`, labels, theme and primary-scale conveniences with deliberate
defaults. Infer the initial facet catalog from data, with explicit order/drop/free-scale
overrides and a defined policy for new categories on updates. Reuse the current
deterministic panel machinery. Keep low-level catalog control for fixed dashboards.
Prefer setters/builders over requiring public struct literals; broad future field
additions should not force ordinary application code to be rewritten.

### API-04 — Medium: there is no single authored plot value shared by presentation and export

Requirements: ARC-01/02, EXP-01/02, GG2-10/11; **Fail for one simple end-to-end workflow**.
[ChartInput::new](../../crates/gpui-charts/src/view.rs), lines 44–70, already hides
compiler, initial state and layout defaults, which is a good native boundary. It
still takes the definition and data snapshot separately.
[FigureSnapshot::capture](../../crates/chart-export/src/snapshot.rs), lines 69–83,
takes definition, source, state, fonts and profile. The
[publication example](../../crates/chart-export/examples/publication_export.rs),
lines 14–37, further relies on fixture helpers for data and resources before exporting
bytes and writing them. The absence of automatic filesystem writes is intentional.

Carry the definition and owned data in one authored plot value. Let native and export
adapters consume that value with their explicit resource context. Configure supplied
fonts once per destination context; offer short bytes export and a host-owned save
helper with physical dimensions. Do not move filesystem access or system-font discovery
into core, add global current-plot state, or recreate a retained native chart per frame.
The existing snapshot and native adapter should remain the implementation boundaries.

### API-05 — Medium: the entry documentation teaches engine internals before chart authoring

Requirements: QLT-05 and WP-14's compiling public examples; **Fail for onboarding**.
[README](../../README.md), lines 12–37, leads with specification, implementation,
infrastructure and contributor setup. The crate-root
[Rustdoc example](../../crates/chart-core/src/lib.rs), lines 1–35, draws a raw scene
rectangle; its introduction still says layout/export are later boundaries. The grammar
example is a valid compiler demonstration but is too much setup for a first chart.
[Alpha API](../alpha-api.md) is an ownership/coverage matrix, not a developer tutorial.

Lead the public docs with one runnable line-plus-points chart using ordinary application
data and a rendered result. Follow with grouped color, histogram, facets, labels/theme,
native mount and export. Keep the current low-level examples as advanced documentation.
Use a small curated prelude within an existing crate rather than adding a package solely
to re-export everything. Document field names, defaults and useful validation messages
alongside the examples. Publish only compiling syntax.

### API-06 — Medium, future product boundary: bindings expose the wire protocol as the API

Requirements: BND-03/04, GG2-10; **Fail for ergonomic host authoring; not a failure of
the intentionally minimal binding-proof scope**.
[Python](../../crates/chart-python/src/lib.rs), lines 23–33, and
[WASM](../../crates/chart-wasm/src/lib.rs), lines 19–27, construct charts from three
JSON strings plus font bytes. Python errors carry diagnostic JSON in `args[0]`.
The [Python proof](../../scripts/bindings/python_proof.py), lines 14–18, loads separate
chart/data/profile fixture files to create its first chart.

When offering developer-facing Python/TypeScript authoring, provide host-native
builders/data adapters and structured error properties over the existing owned Rust
boundary. Preserve JSON as interchange. Resolve fields/compile statistics in the shared
engine rather than growing independent host compilers. Full packaging or viewer work
is not implied by this review.

## Recommended direction and acceptance

Use one small fluent vocabulary. The following is a **design sketch only**, not an
implemented or compile-verified API. `data` represents ordinary named columns supplied
through the proposed data adapter; `output` is a separately configured export context
with explicit font bytes.

```rust,ignore
let plot = Plot::new(data)
    .aes(aes().x("date").y("price").color("series"))
    .layer(line())
    .layer(points().size(2.0))
    .facet(facet_wrap("region"))
    .labels(labels().title("Prices").y("USD"))
    .theme(theme_minimal());

let svg = output.svg(&plot, Size::mm(180.0, 120.0))?;
```

Here `.aes(...color(...))` maps a field; `.size(2.0)` sets a constant style. Dataset
and aesthetic overrides belong on layers. Both named-column and typed-row routes
must lower to the same validated definition/data and retain precision/null metadata.
The compatibility profile must have a documented identity and defaults even if a
constructor supplies it. Do not turn the façade into another statistical engine.

Keep these existing contracts: heterogeneous layers and mapping override validation,
typed generated fields, shared recipe/compiler semantics, immutable snapshots,
exact keys/time, explicit resources, structured diagnostics and host isolation.
The focused grammar tests below pass those tested boundaries. Conciseness must not
weaken any of them.

Recommended next slice: specify and implement the simple plot/data/layer path within
the existing authoring owner, then extend it for aesthetics and customization under
the appropriate parity work packages. The current [GG-16](../impl_plans/phase-2-parity-implementation-plan.md)
combines late extensibility work and authoring conveniences. Define the everyday
authoring acceptance examples earlier; retain semantic prerequisites for each feature.
This recommendation does not resequence the implementation plan or authorize fixes.

Acceptance should use external-style compiling examples with all imports and data
setup visible, and independent expected results where behavior changes:

| Developer task | Proposed usability/equivalence gate |
| --- | --- |
| First scatter or line | No manual dataset/field/layer IDs, revisions, schema epoch or compiler; typed rows and named columns both work. |
| Add an overlay | One layer addition; default data/aesthetics reused; explicit independent-data override works. |
| Map categorical color | One mapping; profile-appropriate grouping and scale/legend behavior; no caller palette/domain inventory required. |
| Histogram | Select x and optionally bin count/width; no manual generated count/endpoints; same shared statistic/provenance. |
| Facet and label | One facet request and labels call; deterministic inferred panels and tested new-category behavior. |
| Export and native mount | Same plot value; destination resources configured once; actual inspected output and retained native lifecycle. |
| Invalid field/type | Error identifies the authored field/layer/aesthetic and a correction, with structured diagnostics retained. |

Overall verdict: **Fail for the requested ggplot2-like developer experience today**.
The underlying tested composition contracts pass within their alpha scope. Actual
developer time-to-first-chart and proposed façade usability remain **Uncertain**;
no user study or proposed-API implementation was performed. Required GG2 parity and
production gates remain open.

## Verification

Working directory `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1 (`8bab26f4f`, 14 July 2026).

- `mise exec -- cargo test -p chart-core --doc --locked`: **4 passed** (two runnable
  examples and two intended compile failures), zero failed.
- `mise exec -- cargo test -p chart-core --test grammar --test facets --locked`:
  **29 passed** (20 grammar, nine facets), zero failed.

These tests establish existing contracts, not the proposed API, ggplot2 differential
parity, actual Python/WASM execution or native/export visual quality. No broad test
matrix, benchmark or usability study was run. Documentation verification is recorded
in the status-ledger entry for this review.
