# Authoring plots and live charts

Use `chart_core::prelude::*` for ordinary data, plot and component builders. The
[capability register](primary-authoring-api.md) records delivered coverage and future
qualification requirements; the [ledger](implementation-status.md) records actual checks. The default
profile is `Profile::LibraryV1`. `Profile::Ggplot2_4_0_3` selects the implemented
stage, grouping, orientation and geometry-theme policies described below. Capability
acceptance remains per package; selecting this profile does not imply full ggplot2 parity.

## Compatibility stages

Set `.profile(Profile::Ggplot2_4_0_3)` on a Rust plot, or
`.profile("Ggplot2_4_0_3")` in Python/JavaScript. The canonical definition retains
resolved policy version 1 and the pinned ggplot2 source hash. `edit().profile(...)`
advances the definition revision; already prepared output keeps its captured policy.
Legacy definitions retain their authored operations and explicit styles.

In this profile, positional `.scale(...)` transforms and censors values outside
explicit scale domains before statistics. `.oob(ScaleOob::Squish)` clamps and `Keep`
retains finite values. `.coordinate_scale(...)` transforms after statistics;
`.viewport(...)` changes the view without filtering the statistic population.
Transformed-space metadata prevents the destination from transforming values twice.
Shared named transforms use the same ordering. Consumers of one shared statistic must
agree on the positional scale context used by that statistic; incompatible contexts
reject and require separately authored transforms.

Eligible discrete source aesthetics infer an interaction group, including color and
missing categories. Explicit `.group(...)`/`.group_all()` and stat-level overrides
select the population. Numeric `group` fields preserve numeric identity. Horizontal
orientation is explicit on layers; the profile also infers it for a y-only histogram
or a bar whose y is categorical and x is not. Unmeasured categorical bars count rows;
a supplied measure preserves the column recipe. Registered geometry orientation
awaits its extension contract.

`source_expr(...)`, `stat_expr(...)`, `bin_expr(...)`, `after_scale_expr(...)` and
`from_theme(...)` build typed, bounded expression graphs in Rust, Python and JavaScript.
Python supports arithmetic operators; JavaScript uses `.add()`, `.mul()`, `.div()` and
other fluent operations. Source expressions also work in `filter(...)`. Graph types,
forward/cyclic references and node/operation budgets are checked before evaluation.
Source reductions use the selected registered dataset before chart filters/facets;
generated reductions use the prepared layer population. After-stat expressions read
back-transformed generated coordinates and their result receives the positional scale
once. These explicit population rules adapt expression evaluation to the owned-data API.

Post-scale expressions read the same snapshot of resolved aesthetics, with size and
color outputs currently supported. Size output requires point/rule geometry. For
example, `points().after_scale(scale_aes().size(after_scale_expr(AfterScaleAesthetic::Size)
* 2.))` doubles the resolved size. `from_theme(ThemeRead::Accent)` reads
`theme().geometry(...)` tokens. The default geometry theme uses point size 1.5 and
line width 0.5; explicitly authored sizes/colors take precedence. Additional independent
fill/stroke/alpha/shape/linewidth mappings and physical size/area semantics belong to GG-03.

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
omits the guide title. Use `generic_title()` for the scale's generic Color/Value
label, or `title("...")` for explicit text. Automatic compatible guides remain available.

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

## Standalone paths and fixed path annotations

`chart_core::path` owns the finite-input d3-path 3.1.0 builder, immutable numeric
geometry and formatter. `path()` / `Path::new()` are unrounded; `path_round()` uses
three output digits. `Path::with_digits(Some(n))` floors finite nonnegative digits,
with values above 15 selecting unrounded output. Precision never changes numeric
geometry, current/start state, replay or captured annotations.

```rust
use chart_core::prelude::*;
let mut curve = path();
curve.move_to(0., 0.)?;
curve.arc_to(10., 0., 10., 10., 2.)?;
let svg = curve.to_svg()?;
let annotation = vector_path("corner", curve.geometry());
// Compose with plot(data).layer(annotation), or plot.edit().annotation(annotation).
Ok::<(), chart_core::Diagnostic>(())
```

The eight drawing methods include lines, both Béziers, center/angle arcs, tangent
arcs, signed rectangles and close. Angles use radians and clockwise y-down sweep by
default. Authoring preserves empty/move-only paths and the reference's unusual
implicit-start/post-close state. A serializer-only bare segment remains serializable
but rejects scene submission without an initial move. Invalid/non-finite arguments,
intermediate overflow and exhausted operation/command/output/replay/subdivision limits
produce structured diagnostics. Each operation and owned batch is atomic.

`geometry()` returns an owned snapshot. `PathGeometry::transformed`, `lower` and
`bounds` use explicit coordinate-error/work limits. `vector_path(id, geometry)`
adds one fixed annotation; its local geometry uses destination units and its anchor
can use data, panel, figure or output coordinates. Fill/stroke/overflow are explicit;
`.transform(...)` applies local geometry before anchor placement. These paths have
annotation identity and empty source targets. Subsequent builder edits cannot mutate
an existing plot, figure request or snapshot. Path sinks receive numeric commands;
an external sink error stops replay and may leave its accepted prefix.

Python exposes `Path`, `path`, `path_round`, snake-case drawing methods, `copy`,
`apply_batch`, `to_svg`, `result`, `replay` and `dispose`. JavaScript exposes the same
operations with camelCase aliases and `toString`; `free` releases its WASM handle.
Both use `vector_path(id, path)` / `vectorPath(id, path)` to capture an annotation.
Host replay calls a supplied sink with owned numeric command records. Callbacks are
not retained by charts. Host coordinates are typed numbers; JS coercion/truthiness
and non-finite SVG output are excluded safety adaptations. Null/None digits explicitly
select unrounded output, while an omitted pathRound argument defaults to three.

`PathRequest` / `Path::from_json` is a standalone version-one operation envelope with
`digits`, `limits` and `operations`; unknown versions reject. Definitions/compositions
containing retained paths use version two, and scene results containing `VectorPath`
report version two. Legacy definition bytes and existing `PathCommand` validation
remain compatible. SVG preserves analytic arcs on the ordinary text-preserving route;
native and outlined/publication conversion use shared bounded numerical geometry.
Curved dashing remains a separate renderer/style capability.

Run `scripts/run_path_proofs.py` with wasm-bindgen CLI 0.2.128 for actual three-host
proofs. The complete binding runner includes these path cases. The native consumer is
`cargo run -p chart-gallery --example path_authoring --locked`.

## Standalone floating colors

Rust `chart_core::color` and the public Python/WASM `color`, `rgb`, `hsl`, `lab`,
`hcl`, `lch`, `gray` and `cubehelix` constructors share one floating color engine.
Parsing supports the pinned d3-color CSS grammar; `color(text)` returns None/null
when text does not parse. Model constructors also accept color text or another
owned color. Channels retain out-of-gamut and undefined values until formatting.

```python
from finstack_chart import hsl, lab

with hsl(210, 0.5, 0.4, opacity=0.75) as authored:
    print(authored.format_hex8())
    with lab(authored) as converted:
        print(converted.channels())
        saved = converted.to_json()
```

`copy`, `with_channel`, `convert`, `brighter`, `darker` and `clamp` return independent
values; `clamp` is defined for RGB/HSL. Python uses snake_case and WASM additionally
exposes camelCase aliases. Explicit standalone descriptors preserve exceptional
channels and signed zero with named tags and reject unknown versions. All authored
paint inputs accept these values: layer colors, theme tokens, palettes, rich text,
annotations, candles, gradient endpoints and output backgrounds. Definitions retain
the original space/channels; prepared scenes use sRGB bytes. For example:

```python
shade = lab(65, 45, 50, opacity=0.6)
figure = plot(data).aes(aes().x("x").y("y")).layer(points().color(shade)).build()
options = export_options(440, 340).background(hsl(210, 0.1, 0.97))
```

The same color can appear inside a gradient or palette without an intermediate hex
conversion. Rust uses `color::Paint::from_css(text)?` or passes typed colors to fluent
setters; direct raw definition fields accept byte colors with `.into()`. Continuous
floating palettes use the shared RGB interpolation engine; legacy byte palettes keep
their previous blending semantics. Expanded scale interpolation controls have their
own SP-04 acceptance. Floating paint first appears in wire version four; the envelope selects the minimum
version required by all retained capabilities.

## Numeric scale knots and compatible defaults

Rust's `scales::NumericScale` provides immutable linear, signed power/sqrt,
negative-domain log, symlog, identity and radial mapping. `NumericScaleSpec::d3`
selects matching standalone defaults; `map`, `invert`, `invert_output`, `nice`,
`with_domain` and `with_range` preserve independent copies. Missing and undefined
results use the shared typed value boundary. Non-finite geometry is rejected before
publication. For a piecewise positional axis:

```rust
use chart_core::{prelude::*, scales::NumericScale};
let scale = NumericScale::linear()
    .with_domain([0.0, 10.0, 100.0])?
    .with_range([0.0, 50.0, 100.0])?;
let figure = plot(data)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().scale(scale_numeric(scale.spec().clone())))
    .build()?;
```

The middle knot occupies half the destination range. An explicit viewport retains
the interior knots, and navigation uses that same mapping. `nice(count)` is an
explicit immutable domain edit; it does not run automatically because a viewport
changed. Existing `scale_linear`/`scale_log` recipe defaults retain their historical
policy. Compatible numeric axes require definition version five. In a grammar profile
that transforms before statistics, select `coordinate_scale(scale_numeric(...))`;
shared pre-stat integration for these new families remains GG-04. Shared typed output
scales, numeric ticks/formatters and explicit calendars are implemented. Complete
standalone scale facade and integration qualification remains SP-07. This does not certify full
D3 scale parity.

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


### D3 categorical scales

Existing `scale_band()` and `scale_point()` retain their recipe spacing.
`scale_band_d3(BandSpec::default())` and `scale_point_d3(PointSpec::default())`
provide D3 defaults, with explicit alignment and integer rounding. Supply categories
with `.categories(...)`, or use the source's stable first-seen catalog. Duplicate
D3 categories retain the first occurrence. Axis `.range(start,end)` sets destination
units; `round: true` applies D3 rangeRound behavior there. Band inner padding may reach
one for zero-width bands. These axes use definition version five.

Standalone `CategoryScale<K>::band(spec, range)` and `::point(spec, range)` preserve
any native ordered key type. `map` returns D3's lower band start, while `center`,
`extent`, `step` and `bandwidth` support positioning and inspection. Extents follow
the range's orientation. `ScaleKey` provides portable typed identity, including exact
64-bit integers; numeric signed zeros and NaNs follow SameValueZero membership.

`OrdinalScale<K,V>` cycles any typed range by the stable category index. Construct
an `OrdinalSpec` with an explicit unknown value or `OrdinalUnknown::Implicit`, then
call `train(keys)` before lookup. Training produces a new scale; `map` never mutates
a catalog. An empty range or untrained implicit key yields `None`. Clone and
reconfiguration leave earlier scales unchanged. The same operations are available through the owned Python/WASM `StandaloneScale` facade below.

### Distribution and typed output scales

`ClassifierScale<V>` prepares sample quantiles or equal-width quantize buckets from a
`ClassifierSpec<V>`. `ThresholdScale<K,V>` accepts typed ordered cutpoints. Both support
unknown outputs and inverse intervals; equality enters the next bucket. Quantile input
populations exclude missing/NaN samples. Empty quantile populations retain undefined
breakpoints; empty quantize ranges reject. `ScaleExtent` distinguishes an absent range
value from an unbounded/undefined endpoint. Outputs and keys need not be strings.

`ContinuousScale` retains arbitrary input knots and typed range values with a shared
`InterpolationFactory`. `InterpolatedScale` combines sequential, diverging or rank
normalization with `ScaleRangeFunction::Identity` or a shared `InterpolationSpec`.
Diverging normalization has three authored inputs, including an independent midpoint.
Use `ScaleNormalizer::map_with` for a native custom `Sample<T>` implementation. These
interpolated families do not advertise a general numerical inverse.

Chart mappings use the same `MappedScaleSpec`:

```rust
use chart_core::{prelude::*, scales::*, interpolate::*};
let color = MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(
    InterpolatedScaleSpec {
        normalization: NormalizationSpec::Diverging {
            family: NumericFamily::Linear,
            domain: [Number(-10.), Number(0.), Number(100.)],
            clamp: true,
        },
        output: ScaleRangeFunction::Interpolate(InterpolationSpec::Piecewise {
            factory: InterpolationFactory::new(FactoryKind::Value),
            values: ["red", "white", "blue"].map(|s| Value::Text(s.into())).to_vec(),
        }),
        unknown: Value::Missing,
    },
));
// Add `.scale(color_mapped("change", color))` to a Plot builder, and select it with
// `.aes(aes().color("change_column").color_scale("change"))`.
```

For a quantile, rank or implicit ordinal mapping, set `training: ScaleTraining::Eligible`
to train from eligible post-stat observations across layers and facet panels sharing
the scale identity. Repeated broadcast/chart-scope presentations contribute their
population once; repeated numeric observations remain separate quantile samples.
Ordinal training starts from authored keys, then appends observed keys in layer, panel
and row order. Explicit unknown policies do not extend their catalog.
Viewport changes do not retrain this population. Authored training uses the retained
sample vector. Corrected data and retention changes rebuild the current population.
Guides carry intervals, diverging midpoint and interpolation identity independently
of label text; unbounded labels use `< value` or `>= value`. Classifier labels start
with six significant digits and increase precision until distinct cutpoints have
distinct labels. Exact interval endpoints remain available in guide metadata.

A layer's `.numeric_scale(NumericAesthetic::Size, "field", spec)` maps point radius or
rule size. `Opacity` multiplies paint coverage and `StrokeWidth` controls width separately
from radius. Pass a `StatField` instead of a field name for generated values. These
mappings require finite numeric or missing outputs; opacity must be in the closed unit
interval. Missing/nonpositive sizes exclude the mark. Widths and colors must be constant
within a line/filled run. Floating alpha is multiplied before its single byte conversion.
Numeric threshold cutpoints select checked numeric source conversion, including integer
columns. Explicit integer, timestamp or string cutpoints retain typed-key comparisons;
use exact integer cutpoints for identities beyond binary64 precision. In Python,
`domain=[2.0, 5.0]` selects numeric cutpoints while `[2, 5]` retains integer-key identity;
match exact key types deliberately. Mapped color and
numeric style descriptors require definition v5.

## Numeric ticks and formatting

Standalone numeric scales expose raw candidates independently of label formatting and
layout. The count is a hint; the second argument is a hard output budget, which raises
an error without silently changing the grid. Log candidates include minor values whose
default labels can be empty. Layout preserves those minor marks and may thin overlapping
labels. Sequential/diverging and quantize scales use the same helpers; rank scales
expose quantiles instead. Nice edits only outer endpoints and preserves a diverging center.

```rust
use chart_core::{scales::NumericScale, typography::{NumericFormat, NumericLocale}};

let scale = NumericScale::linear().with_domain([0.0, 1.0])?;
assert_eq!(scale.ticks(4.0, 100)?, vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
let labels = scale.tick_format(4.0, Some("+.1%"), NumericLocale::default())?;
assert_eq!(labels.format(0.2), "+20.0%");
let axis = chart_core::plot::x_axis().numeric_format(NumericFormat {
    specifier: ",.2f".into(),
    locale: NumericLocale::default(),
});
```

The specifier grammar is `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.
Types include binary/octal/decimal/hex, fixed/scientific/general, rounded significant,
percentage, SI, and literal text. Unknown letters use trimmed general notation, as does
the reference. `NumericFormat::prepare_prefix(value)` fixes an SI prefix from a reference
value. A prepared `c` formatter accepts bounded literal strings through `format_text`.
Locales explicitly supply punctuation, currency, sign, NaN text and optional digit
substitutions. A missing grouping resource (`None`) disables grouping; an explicitly
empty group vector retains D3's empty-group result. These descriptors require v5 only
when embedded in a plot. Existing `number_format()` recipes keep their earlier meaning.

### Explicit calendar time scales

`scale_utc()` retains the existing recipe grid. Select `scale_calendar(TimeScaleSpec)`
for D3-compatible UTC or supplied local intervals and labels:

```rust,ignore
let time = TimeScaleSpec {
    domain: vec![start_ms, end_ms],
    zone: CalendarZone::Local(std::sync::Arc::new(supplied_rules)),
    ..TimeScaleSpec::default()
};
let guide = x_axis()
    .scale(scale_calendar(time)
        .calendar_interval(CalendarInterval::new(CalendarUnit::Hour)))
    .time_format(TimeFormat {
        pattern: Some("%H:%M %Z".into()),
        ..TimeFormat::default()
    });
```

The source timestamp unit must agree with the scale. Rules contain explicit coverage,
transitions, tzdata identity and owner revision; core never reads a system timezone.
The same rules place ticks and format labels, including repeated fall hours and skipped
spring hours. `TimeScale::ticks`, `nice`, `tick_format`, `map` and numeric `invert` are
available independently of charts. Calendar count hints have a separate hard output
budget. `nice` and reconfiguration return independent scales. Finer integer units keep
microsecond/nanosecond resolution; unsupported calendar coverage or lost integer
precision returns a diagnostic. These time descriptors require definition v5.


### Owned standalone scales in Python and WASM

`StandaloneScale` uses the Rust family kernels for all 26 constructor families. Python
accepts keyword options; JavaScript accepts an options object. Methods that configure,
train or nice return a new owned scale. Getters and mapped values are independent
outputs; dispose handles explicitly when finished.

```python
from finstack_chart import StandaloneScale, ScaleKey, scale_numeric, color_mapped

x = StandaloneScale("linear", domain=[0., 10., 100.], range=[0., 50., 100.])
assert x.map(10.) == 50.
assert x.invert(50.) == 10.
axis = scale_numeric(x)
colors = StandaloneScale("ordinal", range=["#28587b", "#c77c35"])
guide = color_mapped("regions", colors, "Eligible")
exact = colors.train([ScaleKey("Unsigned", 18446744073709551615)])
```

```javascript
const {StandaloneScale, scale_numeric, color_mapped} = require("./authoring.cjs");
const x = new StandaloneScale("linear", {
  domain: [0, 10, 100], range: [0, 50, 100]
});
const axis = scale_numeric(x);
const colors = new StandaloneScale("ordinal", {range: ["#28587b", "#c77c35"]});
const guide = color_mapped("regions", colors, "Eligible");
```

Available family names are `linear`, `log`, `pow`, `sqrt`, `symlog`, `identity`,
`radial`, `ordinal`, `band`, `point`, `quantile`, `quantize`, `threshold`,
`sequential`, `sequential_log`, `sequential_pow`, `sequential_sqrt`,
`sequential_symlog`, `sequential_quantile`, `diverging`, `diverging_log`,
`diverging_pow`, `diverging_sqrt`, `diverging_symlog`, `utc`, and `local`.
Local time requires a supplied versioned zone. Timestamp domains, maps, ticks and
inverses use Python integers or JavaScript BigInt in the declared source unit.
Fractional millisecond primitives require a finer declared unit.

Use `map`, `invert`, `invert_extent`/`invertExtent`, `ticks`, `format`, `nice`,
`domain`, `range`, `configure`, `copy` and `to_json`/`toJson` for supported operations.
Classifiers expose `thresholds`; empirical ranks expose `quantiles` and sampled
`range` values; bands/points expose `step`, `bandwidth`, `extent`
and `center`. Calendar scales also expose `floor`, `ceil`, `round_time`/`roundTime`
and `offset`. Unsupported operations return capability diagnostics. Sequential,
diverging, category and classifier families do not acquire a numeric inverse.

`MISSING` (Python) and `undefined` (JavaScript) represent missing values separately
from `None`/`null`. Inverse extents include a `found` flag; missing endpoints are
unbounded/undefined, while an explicit null key remains null. `ScaleKey` preserves
exact signed/unsigned/timestamp categories and distinguishes them from numbers.
`train` replaces JavaScript's implicit ordinal mutation with explicit immutable
training. Shared interpolation factories and owned interpolators can supply typed
ranges; chart axes require numeric outputs and diagnostics enforce capability limits.

`scale_numeric`, `scale_calendar`, `scale_band_d3`, `scale_point_d3` and `color_mapped`
connect these descriptors to primary chart authoring. Layer `numeric_scale` accepts
field names, owned fields, source expressions and statistical-field descriptors.
Python/WASM perform no mapping, tick, calendar or inverse arithmetic.

### Named chromatic catalog

`chart_core::scales::chromatic` provides checked `SchemeId` and `InterpolatorId`
identities. `scheme(id, size, reverse)` returns a fresh array of canonical colors:
fixed categorical schemes require no size; each Brewer scheme requires an exact
supported size. `ChromaticRamp::new(ChromaticSpec { id, reverse })` prepares a pure
evaluator. `evaluate(t)` accepts finite parameters and preserves the named ramp's
clamp, lookup or wrapping behavior; reversal evaluates at `1-t`. Named discrete
schemes are separate from interpolator samples.

Python exposes `chromatic_catalog()`, `chromatic_scheme("Blues", 5)` and
`chromatic("Viridis")`; JavaScript also exposes `chromaticCatalog` and
`chromaticScheme`. A scheme query returns independent owned `ColorValue` objects;
a ramp returns an owned `Interpolator[ColorValue]`. Its `quantize(n)` uses the shared
bounded sampling contract: n must be at least two, both endpoints are included, and
returned colors are independent owners. The shared 200,000-value budget includes the
returned array, and aggregate output budgets still apply. Counts zero and one diagnose. Dispose/free these owners using
the same lifecycle as other color/interpolator values. Fixed categorical names such
as Category10 omit `size`; Brewer sizes select actual authored arrays, without
resampling or nearest-size fallback. Catalog metadata lists every supported size.

Pass `chromatic("RdBu")` as the `interpolator` of a `StandaloneScale("diverging",
domain=[-10, 0, 100], ...)`. The center is normalized to 0.5. `reverse=True` reverses
the ramp independently of a descending domain. `color_mapped(...).palette_scheme(
{"id": "Blues", "size": 5})` attaches a named discrete palette to a mapped ordinal,
classifier, threshold or continuous range. Sequential/diverging/rank outputs use the
named interpolator route. Eligible population training remains shared across panels.

Definitions containing either named route require wire v6. The full guide mapping
retains identity, reversal, domain/transform and missing policy; discrete metadata
also includes catalog version and size and is checked against the canonical range.
Legend swatches evaluate that same mapping. Chart null/NaN/Inf color observations use
the declared missing color; direct ramp evaluation diagnoses non-finite parameters.
Final native/publication certification is recorded separately under CP-05.
