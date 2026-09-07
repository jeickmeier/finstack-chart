# Developer guide for the original chart scope

This guide covers the implemented Cartesian/native/headless baseline through the original
WP-01–23 assignment. The [support matrix](support-matrix.md) gives executed platform
coverage. Expanded parity and primary-authoring work does not become available merely
because it appears in the [specification](spec/gpui-charts-specification.md).

## Build and run

From the repository root, provision the pinned tools with `mise install`, then run:

```sh
mise run check
mise run test
mise exec -- cargo run -p chart-gallery --locked
```

Native examples require macOS Apple Silicon, a graphical login and the selected full
Xcode installation. The common core/text/export packages also run on Linux. No chart
initialization requires an API key, network service or finance engine. The first Cargo
build can download locked dependencies; subsequent offline builds need that cache.
`check` includes formatting, license/source policy, native/Kit builds, Clippy, rustdoc and
core WASM compilation. It does not substitute for actual binding-runtime checks.

## Author and share a chart

Use durable dataset/field/layer/row identities. Normalize typed rows once with
`TypedDataBuilder`, or supply validated `NormalizedBatch` columns. The compiler consumes
`ChartDefinition`, an immutable `DataStore` snapshot and `ChartState`. Core calculation
is independent of display dimensions. Destination `layout` consumes a supplied text
measurer and produces one immutable scene with exact source/aggregate/derived targets.

The compiling [grammar tutorial](../crates/chart-core/src/grammar/mod.rs) demonstrates
native typed data, a histogram, shared preparation and destination layout. Its recipe
lowers to the same bin statistic and rectangle layer available to a layered definition.
Use [family fixtures](../fixtures/families/README.md) for lines, points, area/ribbon,
heatmap cells, grouped/stacked bars, supplied OHLC/volume and scale policies. See
[facets](facet-layout-contract.md) for grouping scope and shared/free scales and
[typography/composition](theme-typography-composition-contract.md) for rich labels, themes,
figure furniture, insets and physical layout.

`ChartInput::new` compiles owned input before a native mount. Load supplied font bytes
with `NativeFont::load` before resolving that family in GPUI; retain one `ChartView`
entity per chart. Change its data/definition/layout through its methods. Recreating the
data store, compiler or entity inside every `Render` discards ownership and cache benefits.
The [gallery](../examples/chart-gallery/src/main.rs) is a runnable retained-host example.
Kit is an optional host adapter; it has no separate chart compiler or statistics.

## Update and interact

`DataStore::apply` validates an ordered transaction against exact source-epoch and
per-dataset/schema revisions. Inspect `Applied`, `AlreadyApplied`, `Rejected` or `Conflict`;
queue admission and numeric preparation are different acknowledgements. Upsert replaces
whole rows. Count/event-time retention, watermarks, replay horizons and category order are
explicit. Immutable old snapshots remain valid until their last owner releases them.

For native background preparation, submit a committed snapshot with `ChartView::queue_data`.
One active and one newest pending preparation are bounded independently of accepted source
operations. Compatible work can advance while later data is pending. Incompatible view,
definition or resource completions cannot overwrite the current scene. `set_data` remains
a synchronous path. The [streaming contract](streaming-contract.md) and
[scheduling/density contract](scheduling-density-contract.md) describe exact incremental
paths, batch fallbacks, visible line/candle reduction and disposal.

Painting, queries and actions use the acknowledged scene. Pure hover queries reuse its
index and exact retained lookup. A viewport changes presentation, not the population of
a summary/model. `ActionRequest` carries origin and revision/scene fences; controlled
hosts must accept the expected revision, not a stale saved reply. Freeze retains the
accepted preparation and presentation settings while resize reprojects it; resume admits
the newest coherent source. Follow-tail, history, pinning and freeze are distinct states.
Read [state/actions](state-action-contract.md), [interaction](interaction-contract.md)
and [host tools](host-tools-contract.md) before integrating gestures or linked views.

Runnable workflows:

| Example | What to exercise |
| --- | --- |
| `family_gallery` | Original geometry and scale families |
| `actions_gallery` | Controlled actions, preview/cancel/commit, undo and freeze |
| `interaction_gallery` | Indexed hover, zoom/pan and selection |
| `host_tools_gallery` | Linked charts/table, snapped editing and custom controls |
| `streaming_gallery` | Admission/commit, retention, history and follow |
| `scheduling_gallery` | Four-chart bounded workers and density; `--isolated-redraw` runs the regression |
| `live_export_gallery` | Captured publication while later atomic updates continue |
| `hardening_gallery` | Frozen resize, malformed-resource recovery and entity release |

Run any entry with `mise exec -- cargo run -p chart-gallery --example NAME --locked`.
Pass example arguments after a second `--`. Inspect the native output; a successful build
alone does not certify a UI or accessibility flow.

## Publish bytes from a coherent snapshot

The exporter accepts owned font resources and a `PublicationProfile` with point dimensions,
DPI, visible/full-domain view and text/outline policy. `FigureRequest` captures immutable
inputs; preparation creates a `FigureSnapshot`; encoding returns bytes and a reproducibility
manifest. The caller saves files. No GPUI event loop or system-font scan is needed.

```sh
mise exec -- cargo run -p chart-export --example publication_export --locked -- artifacts/publication
mise exec -- cargo run -p chart-export --example composition_proof --locked -- artifacts/composition
```

For a live native export, call `capture_presented`, transfer its source/state/resource
capture into `FigureRequest`, and submit to a bounded `ExportQueue` on a caller-selected
executor. A job retains one source revision even if the chart later moves. Cancellation,
failures and disposal release queue ownership; running work releases its snapshot on exit.
Use [live-export guidance](live-export-contract.md) for clean versus interactive capture,
resource identity, queue limits and host save timing.

SVG/PDF retain supported vector marks; PNG uses explicit physical dimensions and DPI.
Font embedding permissions still apply. PDF/X, CMYK/spot-color press workflows, tagged-PDF
accessibility and arbitrary native painters are outside the supported export scope.
Native-only painters return an explicit unsupported-export diagnostic.

## Add an extension through the common engine

The [external-style example crate](../examples/custom-extension/src/lib.rs) implements a
checked density histogram and chamfered bars using public APIs. Follow its boundaries:

1. Implement `CustomStat::descriptor`, `schema` and `evaluate`. Validate parameters before
   allocation, declare typed generated fields, honor the supplied budget and preserve exact
   memberships/model identity. Declare only implemented incremental capabilities.
2. Register an immutable implementation under a qualified operation name and exact positive
   version in `ExtensionRegistry`. Pass the registry to the compiler; JSON never installs code.
3. Consume generated fields through generated accessors. A named transform can feed both a
   custom geometry and a builtin layer without another numerical engine.
4. For `CustomGeom`, return shared point/rule/rectangle/polygon geometry plus explicit hit
   regions, semantic values, selection policy and keyboard order. Use a host-owned native
   painter only when the explicit absence of portable/export support is acceptable.
5. Exercise malformed versions, schema/identity errors, exact target lookup and real outputs.
   The [extension fixture instructions](../fixtures/extensions/README.md) run the public
   example tests, native gallery and actual proof adapters.

The [extension contract](extension-contract.md) defines registration limits, provenance,
coordinate capabilities and errors. Callback implementations are trusted native code;
core validates their returned structures but cannot make their internal computation safe.

## Exercise the proof adapters

Read the [portable contract](portable-contract.md) and [binding setup](../fixtures/bindings/README.md).
The runner needs Node 24.14.0, Poppler and wasm-bindgen-cli **0.2.128** in addition to mise:

```sh
WASM_BINDGEN=/absolute/path/to/wasm-bindgen mise run bindings-proof artifacts/bindings
```

This builds and imports the actual PyO3 extension and runs generated Node WASM glue, then
compares shared semantic/action/scene/export/lifetime fixtures. Returned strings/bytes own
their storage. Do not replace exact decimal 64-bit identities or timestamps with JavaScript
Numbers. Python/WASM constructors and wire operations are proof APIs; wheel/npm/browser
viewer/notebook distribution and the separately planned primary-authoring API are not
certified by these results.

## Diagnose and hand off

Keep the first structured diagnostic and its revision/resource context. A recoverable error
retains the prior valid state where possible; empty output is not a successful replacement.
Use the [status ledger](implementation-status.md) and linked reports for requirement-level
evidence and limitations. Preserve fixture values/tolerances and investigate a regression
before regenerating any visual baseline. Performance requires the actual recorded workload,
not repeated screenshots or a zero-test package. Package publishing remains separate from
local development and requires the ownership/license decisions in
[ADR-010](adr/010-package-and-release-policy.md).
