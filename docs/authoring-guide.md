# Authoring plots and live charts

Use `chart_core::prelude::*` for ordinary data, plot and component builders. The
[capability register](primary-authoring-api.md) records delivered coverage and future
qualification requirements; the [ledger](implementation-status.md) records actual checks. The default
profile is the library's existing semantics. Expanded ggplot2/D3 profiles require their
own semantic packages.

## Data and layers

```rust
use chart_core::prelude::*;

let data = Data::rows([(1.0, 10.0), (2.0, 12.0), (3.0, 11.0)])
    .field("time", |row| row.0)
    .field("value", |row| row.1)
    .build()?;
let authored = plot(data)
    .aes(aes().x("time").y("value"))
    .layer(line().name("prices"))
    .layer(points().size(3.0))
    .title(title("Prices"))
    .subtitle(subtitle("Daily observations"))
    .x_axis(x_axis().label("Time"))
    .y_axis(y_axis().label("USD"))
    .layer(labels().id("peak").at(2.0, 12.0).text("Peak").offset(0.0, 14.0))
    .build()?;
```

Columns accept signed/unsigned integers, floating values, booleans, strings and optional
values. Use `categorical(...)` for categorical coordinate scales and
`timestamps(ticks, TimeUnit::Nanoseconds, "UTC")` for exact timestamps. `column(values)`
adds validity, original display strings, labels and units. Row accessors execute once
per batch. Materialization owns the values; changing a caller's array cannot alter a
built chart. Explicit keys are retained; omitted keys are allocated. A keyless replacement
therefore creates new row identities.

Use optional `Data::columns().identity(id)` (Python/JS `identity=`/`identity:`) when
importing a known dataset whose identity must survive reauthoring. Seeded jitter hashes
the exact source identity, so fixture or cross-process comparisons must preserve it.
Ordinary authors omit the option. Within one Plot, references to a shared dataset use
clones of that Data; distinct owners with the same imported identity reject.

Names resolve once against the selected dataset. `data.field("value")?` returns an
owner-checked handle when explicit reuse is useful. `.layer(points().data(other))` adds
an independent source and still inherits mappings by name. Unknown names and foreign
handles fail at build with a structured diagnostic. Build validates structure; it does
not execute statistics, geometry or layout.

## Statistics, appearance and composition

| Intent | Primary components |
| --- | --- |
| Marks and recipes | `line`, `points`, `area`, `ribbon`, `bars`, `volume`, `ohlc`, `rule`, `rectangle`, `cells`, `histogram` |
| Statistics | `.stat(bin().x("x").breaks(edges))`, `count`, `summary`, `fit`, `identity_stat`; `.after_bin(bin_aes()...)` / `.after_stat(stat_aes()...)` |
| Positions and filtering | `.position(stack(order).normalize(true))`, `dodge`, seeded `jitter`; `.filter(filter("x").maximum(2.0))` |
| Shared calculations | `.transform(transform("fit", fit().x("x").y("y")))`; layers use `.from_transform("fit")` |
| Appearance and guides | source `.aes(aes().color("series").group("series"))`; `color_discrete`, `color_continuous`; `.legend(legend().scale("series").title("Series"))` |
| Positional scales | `.x_axis(x_axis().scale(scale_log(10.0)))`; linear, symlog, band, point, UTC and supplied-session families |
| Facets | `.facet(facet_wrap("region").columns(2))` or `facet_grid("region", "scenario")`; explicit order, empty policy, free axes and layer targets |
| Typography | `text_style`, `text_run`, `rich_text`; explicit fonts, fallback, language, direction, size, rotation and line spacing |
| Figure furniture | separate `title`, `subtitle`, `caption`, `source_note`, `footnote`, `panel_letter`, `labels`, `callout`, `inset` |
| Design and layout | `theme().preset(NamedTheme::Editorial)`, plot/layer `style`, axis controls and destination `layout_options` |

Source grouping and appearance are independent. Constants such as point radius and line
width stay on the layer. Map generated output with `stat_aes`/`bin_aes`, whose field types
cannot enter source mappings. `.color_group("scale")` explicitly maps resolved group
identities. Color-scale domains/palettes own key order and mapping; `legend().untitled()`
omits the guide title. Automatic compatible guides remain available.

Bar/volume recipes default to zero when y2 is unmapped; explicit y2 mappings take
precedence. Coordinates use the layer's named x/y axes. Axis viewports zoom without
filtering a statistic's population. An affine secondary axis is a guide over a primary
numeric scale, not an independent layer coordinate scale. UTC values preserve their
integer unit; session scales consume an explicitly supplied calendar.

Fixed labels create one annotation, independent of source row count. Callouts add an
explicit leader endpoint. Labels can use data, panel, figure or output coordinates;
notes and axis titles remain separate components. Insets reference prepared layer
handles and do not rerun statistics for an inset population. Mapped shape/linetype,
mathematical text and broader coordinate families remain gated by their semantic work.

## Live state, updates and edits

```rust
let mut chart = authored.chart()?;
let append = Data::columns()
    .column("time", [4.0])
    .column("value", [14.0])
    .build()?;
let transaction = chart.transaction()?.append("data", append).build()?;
let receipt = chart.commit(transaction)?;

let edited = authored.edit().title(title("Updated prices")).build()?;
chart.apply_plot(&edited, authored.definition().revision)?;
```

An ordinary `Chart` owns its store and bounded ingestion queue. Dataset handles,
append/upsert/replace/remove, count/event-time retention, watermark and category reset
are transaction operations. Commits have explicit revision/replay outcomes. Enqueue
acceptance does not mean the data committed; drain through `commit_next` and inspect its
outcome. `stream_options` sets queue limits and overload policy. Host adapters expose
`stream_status()` for full accounting/reconciliation and `pinned()` for the current or
historical pinned value.

Use `chart.external_view()` for another view over the same immutable committed source;
it starts with independent view state and keeps the original names/handles. Admit later
commits explicitly with `view.accept_from(&chart)` (Python `accept_from(chart)`, JavaScript
`acceptFrom(chart)`). Only the source owner commits; an external view retains its admitted
snapshot after the owner is disposed.

Definition edits preserve current live data, replay state and pending ingestion, even
when the original immutable Plot predates a successful append. Replacing data is a
separate transaction. Rejected edits/transactions leave the prior valid state intact.
Advanced integrations can use `Chart::from_external` with immutable committed snapshots;
that route never creates a second writable store authority.

Chart exposes typed commands, selection/inspection, pinned navigation, controlled state,
annotation editing and links. `annotation_edit` and `link` configure those operations;
`render_options` controls existing dense line/candle representations. A native `ChartView`
retains the same runtime and owns tasks, paint acknowledgement and native factories.
Scheduling keeps one active and one newest pending preparation. Fonts, UI objects and
worker threads remain outside core.

## Native and publication destinations

A supplied `NativeFont::from_bytes(...)` plus `ChartInput::from_plot(...)` mounts a static
plot. `ChartInput::from_chart(...)` adopts an existing live Chart. Input builders accept
layout/density, tooltip, controls, command/event and accessibility hooks. Kit's
`gpui_charts_kit::chart_input` applies a Kit theme to this same destination. Retain the
`ChartView` entity; do not rebuild a plot on every render.

```rust
use chart_export::{Output, PageSize};
let output = Output::new(font_bytes)?;
let svg = output.svg(&authored, PageSize::millimeters(180.0, 120.0)?)?;
```

`Output` retains explicit fonts for many plots. `export_options` configures physical
page size, DPI, text mode, background, budgets and layout. Capture a request first, then
prepare and encode it, or submit it to a bounded `ExportQueue`. Saving a path is an
explicit host operation. Static exports use the Plot's source and initial state.

Live capture has three independent choices: **Presented/Current** basis,
**visible/full-domain** view, and **interaction inclusion**. Presented is the live default
and requires an acknowledged frame; it preserves that frame's source and origin stamp.
Current can capture before first paint and uses committed inputs without inventing a
presentation stamp. Requests and jobs retain their inputs through later updates, edits
and view disposal. Headless export never needs a GPUI event loop.

## Python and JavaScript

Python uses the source package plus the built PyO3 extension:

```python
import finstack_chart as c

data = c.Data.columns({"time": [1.0, 2.0, 3.0], "value": [10.0, 12.0, 11.0]})
plot = (c.plot(data).aes(c.aes().x("time").y("value"))
    .layer(c.line()).layer(c.points().size(3.0))
    .title(c.title("Prices")).build())
chart = plot.chart()
```

JavaScript loads `authoring.cjs` beside the generated WASM module:

```javascript
const c = require('./authoring.cjs');
const data = c.Data.columns({time: new Float64Array([1, 2, 3]),
                             value: new Float64Array([10, 12, 11])});
const plot = c.plot(data).aes(c.aes().x('time').y('value'))
  .layer(c.line()).layer(c.points().size(3)).title(c.title('Prices')).build();
const chart = plot.chart();
```

Hosts forward field resolution, defaults, validation and execution to Rust. Python
integers and JavaScript BigInt preserve exact 64-bit values. Unsafe JS numbers reject;
use explicit timestamps and column kinds for ambiguous or all-null data. `Data.rows`
materializes accessor callbacks once. `Output` receives bytes; Python `.save` and explicit
Node filesystem calls can write the returned SVG/PDF/PNG bytes.

Handles support `dispose()`. Retained requests and editors can survive their source
runtime's disposal. JavaScript also exposes `free()` for deterministic wrapper release;
in sustained explicit-lifetime loops, free intermediate persistent builders as well as
final handles. Ordinary garbage collection timing is host-controlled. Python Rust-only
execution detaches from the interpreter. Errors expose `code`, `diagnostic`, `context`
and `correction`. Stubs and TypeScript declarations accompany the executable proof.

## Migration from low-level entry points

| Previous entry point | Primary route |
| --- | --- |
| TypedRows / TypedDataBuilder / manual schema and keys | `Data::rows` / `Data::columns`, named fields and optional keys |
| ChartDefinition / Layer / mapping mutation | `plot(data)`, component builders, `.build()`, immutable `.edit()` |
| Compiler plus separate store/reducer/queue | retained `Chart`; native `ChartView` adopts that runtime |
| `ChartInput::new(definition, source, font)` | `ChartInput::from_plot` / `from_chart`; explicit external-source Chart for existing store owners |
| FigureSnapshot::capture plus manually built request/profile | supplied-font `Output`, `export_options`, immutable request and `ExportQueue` |
| Hand-authored chart/data/state envelopes in host tutorials | `finstack_chart` or `authoring.cjs` Data/Plot/Chart/Output |

Existing specialist paths, legacy constructors and version-1 envelope fields remain
supported. Portable Session is a forwarding compatibility adapter over Chart. The
primary Plot interchange envelope independently records names, profile and exact source
values; `Plot::to_json` / `from_json` do not serialize extension implementations.
Registered extension loading remains explicit.

The planned migration release is 0.2.0; removal is no earlier than 0.3.0 and requires
consumer/deprecation evidence. The workspace remains unpublished 0.1.0 during this work.
No broad visibility removal is necessary to make authoring primary. Expert compiler,
wire, resource and standalone helper modules remain available; new tutorials should
start with builders. New features must extend these components or add a new builder,
including applicable host wrappers/declarations and proof cases before acceptance.
