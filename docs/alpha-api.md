# Alpha API and G2 coverage

WP-14 freezes the implemented alpha boundary for further work, with source-compatible
changes reviewed explicitly. It does not promise production SemVer stability, published
packages, completed streaming or release support. [Specification](spec/gpui-charts-specification.md)
semantics govern names; [status](implementation-status.md) owns the remaining packages.

The [primary authoring plan](impl_plans/primary-authoring-api-plan.md), AUT-01–09 and
[ADR-013](adr/013-primary-authoring-api.md) now define the successor public surface
over an assumed completed original WP-01–23 baseline. All features must become
accessible through the primary builders/runtime; the table below remains the current
historical alpha API, not the proposed developer entry point. The refactor is planned,
with migration and G-AUTH evidence still open; no new API is available merely because
its sketch appears in the plan.

The scale PASS row below records the original 0.1.0 alpha subset. The required
0.2.0 [D3 scale parity work](impl_plans/d3-scale-parity-plan.md), SCL-06–08/FIX-20,
remains open through SP-01–07 and G-SCALE; this alpha matrix does not certify it.

## Authoring and ownership

| Surface | Chosen alpha API and contract |
| --- | --- |
| Data | `TypedDataBuilder`/`NormalizedBatch`, `DataStore`, immutable `SnapshotHandle`, stable `RowKey`, atomic `Transaction`; [data ADR](adr/004-immutable-data-and-transactions.md). |
| Grammar | `ChartDefinition`, `Layer`, `SourceAes`/`BinAes`/`StatAes`, `Statistic`, `Position`, `Compiler`; convenience histogram/bar/cell/OHLC recipes lower to the same engine. [Compiling typed recipe](../crates/chart-core/src/grammar/mod.rs). |
| Preparation | `PreparedChart`, named transform tables, exact source/aggregate/derived targets, separate generated schemas; source/statistical/destination stages stay distinct. [Statistics](statistics-contract.md). |
| Scales/layout | `AxisSpec`, `ResolvedAxis`, `Cartesian`, `LayoutRequest`, `layout`, `LaidOutChart`; explicit destination `TextMeasurer`, named scales and capabilities. [Families](scale-geometry-contract.md), [facets](facet-layout-contract.md). |
| Design | `ThemeSpec`, versioned theme cascade, `FigureComposition`, supplied explicit fonts; [typography/composition](theme-typography-composition-contract.md). |
| Native | `NativeFont::load`, `ChartInput`, retained `ChartView`; optional Kit theme snapshot adapter. Hosts own window/input/control closures. [Standalone example](../examples/chart-gallery/src/main.rs), [composition examples](../examples/chart-gallery/examples/family_gallery.rs). |
| Export | `FigureSnapshot::capture`, immutable resources, `export(Format)`, physical publication profile, preview scene; bytes-only SVG/PDF/PNG independent of GPUI. [Compiling publication](../crates/chart-export/examples/publication_export.rs). |
| Extensions | Explicit `ExtensionRegistry`, `CustomStat`, `CustomGeom`, `GeometryInteraction`, `NativePainterRegistry`; [contract](extension-contract.md), [public example](../examples/custom-extension/src/lib.rs). |
| Portable proof | Strict version 1 envelopes, exact decimal integer IDs/time, registered operation versions, `Session`/`PortableChart`; actual Python/WASM adapters share core. [Wire/lifetime contract](portable-contract.md). |

Constructor defaults remain explicit in Rustdoc and the linked contracts: identity stat
and position, source-data statistical space, x-ordered lines split at invalid rows,
zero-baseline area, named primary x/y axes, viewport-independent populations, and empty
extension registries. Auto bins default to 30 equal-width bins; explicit edges are
left-closed/right-open with the final right endpoint included. No hidden scale transform
changes a statistic. OHLC consumes supplied prices, validates their bounds and assigns
up color when close >= open; mapped semantic color takes precedence when supplied.

Diagnostic codes are shared by all adapters: validation/schema/precision/numerical-domain,
resource limits, disposed/stale revision and unsupported capability. A failed update
preserves the prior valid snapshot; ignored invalid values are counted and diagnosed.
The [extension contract](extension-contract.md) adds precise registration/output failure
cases. Public struct additions and enum variants affect literal construction/exhaustive
matches; the [changelog](../CHANGELOG.md) records alpha changes.

## Required alpha feature matrix

PASS below means the cited G2 feature evidence, not later interaction/performance/platform
certification. All 36 current declarative cases execute in Rust, Python and actual Node
WASM; exported scene tolerance is 1e-10 points, semantic tolerance 1e-12 and SVG bytes exact.
Each proof also retains independent expected values, rather than only comparing hosts.

| Feature / IDs | Current public surface | Acceptance evidence | G2 |
| --- | --- | --- | --- |
| Points/scatter, lines/gaps, rules, rectangles; GRA-01/02/06, FIX-01 | Source/generated layers and ordinary scene primitives | [WP-05](evidence/wp-05-completion-2026-09-06.md), [WP-07](evidence/wp-07-completion-2026-09-06.md), [WP-11](evidence/wp-11-completion-2026-09-07.md), shared binding base/family cases | PASS |
| Explicit/auto histogram, count, grouped summaries/quantiles, OLS; GRA-03/04/08, FIX-02/04/05 | Separate generated schemas and exact membership/model targets | [WP-10](evidence/wp-10-completion-2026-09-06.md), `fixtures/statistics` cases, corrected batch comparisons | PASS |
| Mixed-sign stack/normalize, dodge, deterministic jitter; GRA-05, FIX-03 | Position before domains; display jitter after projection | [WP-10](evidence/wp-10-completion-2026-09-06.md), [WP-11](evidence/wp-11-completion-2026-09-07.md) | PASS |
| Area/ribbon, interval/grouped/stacked bars, cells/heatmaps, supplied candle/volume; GRA-06 | Builtin geometry, validated endpoints and volume | [WP-11](evidence/wp-11-completion-2026-09-07.md), WP-14 independent candle-direction colors | PASS |
| Linear/log/symlog, UTC/calendar/session, band/point, discrete/continuous color, secondary guides; SCL-01–05, FIX-07 | Named resolved scales, exact time units and explicit capabilities | [WP-06](evidence/wp-06-completion-2026-09-06.md), [WP-11](evidence/wp-11-completion-2026-09-07.md), WP-14 coordinate/guide errors | PASS |
| Facet wrap/grid, shared/free scales, aligned panes and collected guides; GRA-07/08, LAY-01/03, FIX-06 | Stable typed panels and broadcast/targeted layers | [WP-12](evidence/wp-12-completion-2026-09-07.md), seven actual facet cases | PASS |
| Text/annotations, custom themes, furniture/insets, explicit fonts, rotated/rich/tabular text; THM-01–03, LAY-02–04, FIX-12/13 | Shared theme cascade and composition anchors; static annotation output | [WP-13](evidence/wp-13-completion-2026-09-07.md), three actual composition cases, inspected native/Kit | PASS |
| SVG/PDF/PNG and exact publication preview; EXP-01/02/04, SCN-03, FIX-13 | Vector native/export primitives, explicit output resources | [WP-08](evidence/wp-08-completion-2026-09-06.md), [WP-13](evidence/wp-13-completion-2026-09-07.md), WP-14 vector checks | PASS |
| Custom stat/geom, guides/targets, native-only painter failure; ARC-03, GRA-08, INT-06, FIX-17 | Public external-style example, known registration and exact generated schema | [WP-14](evidence/wp-14-completion-2026-09-07.md), nine focused tests, actual three-host extension fixture, native keyboard/paint inspection | PASS |
| Portable alpha builtins, known extension IDs, ownership/errors; BND-01–04, FIX-15/16/17 | Shared envelopes and Rust engine in PyO3/wasm-bindgen | [WP-09](evidence/wp-09-completion-2026-09-06.md) plus 36 current cases; [WP-14](evidence/wp-14-completion-2026-09-07.md) | PASS |

G2 is passed for these cumulative Cartesian/publication alpha contracts. General action
ownership/gestures, navigation and selection, linked interaction and annotation editing,
retention/queues, scheduling/dense representation and live-export capture remain WP-15–20.
Linux execution, full accessibility/fidelity/lifetime hardening, measured PERF gates and
release provenance/license decisions remain WP-21–23. Other desktop hosts, browser
renderers and packaged Python viewers remain outside the stated release scope.
