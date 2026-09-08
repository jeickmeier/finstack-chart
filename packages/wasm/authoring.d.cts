// Primary CommonJS proof adapter. Source identities and revisions remain exact strings in core result records.
export type JSONValue = null | boolean | number | bigint | string | ReadonlyArray<JSONValue> | {readonly [key: string]: JSONValue};
export type Options = Readonly<Record<string, unknown>>;
export type Color = string | Options;
export type Panel = ReadonlyArray<string | number | boolean | Options> | Options;
export type ScaleValue = number | string | Options;
export type MappingValue = string | number | Field | Options;
export type Format = 'svg' | 'pdf' | 'png';
export type ColumnKind = 'float64' | 'int64' | 'uint64' | 'bool' | 'string' | 'category' | 's' | 'ms' | 'us' | 'ns';
export interface DispatchOutcome { outcome: {changed: boolean; revision: string; viewport_changed: boolean}; event: Record<string, unknown> | null; }
export interface SelectionResult { targets: Array<Record<string, unknown>>; }
export interface WindowResult { windows: Record<string, unknown>; }
export interface Revisions { definition: bigint; state: bigint; epoch: bigint; store: bigint; }
export class ChartError extends Error {code: string; correction: string; context: Record<string, unknown>; diagnostic: Record<string, unknown>;}
export class Owned {protected constructor(); dispose(): void; free(): void;}
export class Field extends Owned {private readonly _field: "Field";}
export class Column extends Owned {
  nullable(value?: boolean): Column;
  validity(values: Iterable<boolean>): Column;
  formatted(values: Iterable<string | null>): Column;
  unit(value: string): Column;
  label(value: string): Column;
}
export function column(values: Iterable<number | bigint | boolean | string | null>, options?: {kind?: ColumnKind; timezone?: string}): Column;
export function categorical(values: Iterable<string | null>): Column;
export function timestamps(values: Iterable<number | bigint | null>, unit?: 's' | 'ms' | 'us' | 'ns', timezone?: string): Column;
export class Data extends Owned {
 static columns(columns: Record<string, Column | Iterable<number | bigint | boolean | string | null>>, options?: {name?: string; keys?: Iterable<number | bigint>; limits?: Options; identity?: number | bigint; schemaVersion?: number | bigint}): Data;
 static rows<T>(rows: Iterable<T>, options?: {fields?: Record<string, (row: T, index: number) => number | bigint | boolean | string | null>; name?: string; keys?: Iterable<number | bigint> | ((row: T, index: number) => number | bigint); limits?: Options; identity?: number | bigint; schemaVersion?: number | bigint}): Data;
 field(name: string): Field;
 readonly name: string;
}
export class Component extends Owned {}

export class Aes extends Component {
  private readonly _family: "Aes";
  x(value: MappingValue): this;
  y(value: MappingValue): this;
  x2(value: MappingValue): this;
  y2(value: MappingValue): this;
  low(value: MappingValue): this;
  high(value: MappingValue): this;
  size(value: MappingValue): this;
  group(value: MappingValue): this;
  color(value: MappingValue): this;
  group_all(): this;
  groupAll(): this;
  color_scale(name: string): this;
  colorScale(name: string): this;
}

export class Layer extends Component {
  color_group(scale: string): this;
  colorGroup(scale: string): this;
  private readonly _family: "Layer";
  name(name: string): this;
  data(data: Data): this;
  aes(mapping: Aes): this;
  stat(value: Stat): this;
  after_stat(mapping: StatAes): this;
  afterStat(mapping: StatAes): this;
  after_bin(mapping: BinAes): this;
  afterBin(mapping: BinAes): this;
  filter(value: Filter): this;
  position(value: Position): this;
  style(value: Style): this;
  from_transform(value: string | Transform): this;
  fromTransform(value: string | Transform): this;
  independent(): this;
  size(value: number): this;
  width(value: number): this;
  baseline(value: number): this;
  color(value: Color): this;
  axes(x: string, y: string): this;
  scope(value: string): this;
  facet_target(value: Options): this;
  facetTarget(value: Options): this;
  clip(value: boolean): this;
  connect_gaps(value: boolean): this;
  connectGaps(value: boolean): this;
  invalid(value: string): this;
  order(value: string): this;
  candle_colors(value: Options): this;
  candleColors(value: Options): this;
  bins(count: number): this;
  breaks(values: ReadonlyArray<number>): this;
  geometry(name: string, version: number | bigint, parameters: JSONValue): this;
}

export class Stat extends Component {
  field_parameter(name: string, value: MappingValue): this;
  fieldParameter(name: string, value: MappingValue): this;
  private readonly _family: "Stat";
  x(value: MappingValue): this;
  y(value: MappingValue): this;
  group(value: MappingValue): this;
  group_all(): this;
  groupAll(): this;
  bins(count: number): this;
  breaks(values: ReadonlyArray<number>): this;
  quantiles(values: ReadonlyArray<number>): this;
  required(values: ReadonlyArray<string | number | Options>): this;
  outliers(value: string): this;
  empty_sum_zero(value: boolean): this;
  emptySumZero(value: boolean): this;
  transform(factor: number, offset: number): this;
  y_transform(factor: number, offset: number): this;
  yTransform(factor: number, offset: number): this;
}

export class StatAes extends Component {
  private readonly _family: "StatAes";
  x(value: string | number | Options): this;
  y(value: string | number | Options): this;
  x2(value: string | number | Options): this;
  y2(value: string | number | Options): this;
  size(value: string | number | Options): this;
  color(value: string | Options): this;
  color_scale(name: string): this;
  colorScale(name: string): this;
  color_group(name: string): this;
  colorGroup(name: string): this;
}

export class BinAes extends Component {
  private readonly _family: "BinAes";
  x(value: string | number | Options): this;
  y(value: string | number | Options): this;
  x2(value: string | number | Options): this;
  y2(value: string | number | Options): this;
  color_group(name: string): this;
  colorGroup(name: string): this;
  size(value: string | number | Options): this;
}

export class Position extends Component {
  private readonly _family: "Position";
  normalize(value: boolean): this;
  width(value: number): this;
  displacement(x: number, y: number): this;
  units(value: string): this;
}

export class Filter extends Component {
  private readonly _family: "Filter";
  minimum(value: number): this;
  maximum(value: number): this;
}

export class Transform extends Component {
  private readonly _family: "Transform";
  data(data: Data): this;
  aes(value: Aes): this;
  filter(value: Filter): this;
  from_transform(value: string | Transform): this;
  fromTransform(value: string | Transform): this;
  scope(value: string): this;
  invalid(value: string): this;
  facet_target(value: Options): this;
  facetTarget(value: Options): this;
}

export class Scale extends Component {
  private readonly _family: "Scale";
  domain(start: number, end: number): this;
  band_padding(start: number, end: number): this;
  bandPadding(start: number, end: number): this;
  baseline(value: string): this;
  padding(value: number): this;
  point_padding(value: number): this;
  pointPadding(value: number): this;
  nice(value: boolean): this;
  nice_ticks(value: number): this;
  niceTicks(value: number): this;
  categories(values: ReadonlyArray<string>): this;
  time_domain(start: number | bigint, end: number | bigint): this;
  timeDomain(start: number | bigint, end: number | bigint): this;
  interval(value: Options): this;
}

export class Axis extends Component {
  private readonly _family: "Axis";
  name(value: string): this;
  label(value: string): this;
  side(value: string): this;
  outside(value: string): this;
  visible(value: boolean): this;
  rotation(value: number): this;
  viewport(start: number, end: number): this;
  range(start: number, end: number): this;
  scale(value: Scale): this;
  text_style(value: TextStyle): this;
  textStyle(value: TextStyle): this;
  rich_label(value: RichText): this;
  richLabel(value: RichText): this;
  format(value: NumberFormat): this;
  ticks(values: ReadonlyArray<readonly [ScaleValue, string]>): this;
  secondary(source: string, factor: number, offset: number): this;
}

export class ColorScale extends Component {
  private readonly _family: "ColorScale";
  palette(values: ReadonlyArray<Color>): this;
  domain(values: ReadonlyArray<string>): this;
  missing(value: Color): this;
  clamp(value: boolean): this;
}

export class Legend extends Component {
  untitled(): this;
  private readonly _family: "Legend";
  scale(value: string): this;
  title(value: string): this;
}

export class Facet extends Component {
  private readonly _family: "Facet";
  columns(value: number): this;
  empty(value: string): this;
  free_x(value: boolean): this;
  freeX(value: boolean): this;
  free_y(value: boolean): this;
  freeY(value: boolean): this;
  collect_guides(value: boolean): this;
  collectGuides(value: boolean): this;
  gap(value: number): this;
  order(values: ReadonlyArray<Panel>): this;
}

export class Style extends Component {
  private readonly _family: "Style";
  background(value: Color): this;
  panel(value: Color): this;
  foreground(value: Color): this;
  grid(value: Color): this;
  mark(value: Color): this;
  annotation(value: Color): this;
  focus(value: Color): this;
  selection(value: Color): this;
  font_size(value: number): this;
  fontSize(value: number): this;
  padding(value: number): this;
  gap(value: number): this;
  tick_length(value: number): this;
  tickLength(value: number): this;
  stroke_width(value: number): this;
  strokeWidth(value: number): this;
  dashes(values: ReadonlyArray<number>): this;
  color_mode(value: string): this;
  colorMode(value: string): this;
  gradient(value: string | Options): this;
  symbol(value: string | Options): this;
}

export class Theme extends Component {
  private readonly _family: "Theme";
  preset(value: string): this;
  style(value: Style): this;
  layer(layer: Layer, style: Style): this;
}

export class TextStyle extends Component {
  private readonly _family: "TextStyle";
  size(value: number): this;
  weight(value: number): this;
  font(value: Options): this;
  fallback(value: Options): this;
  color(value: Color): this;
  language(value: string): this;
  direction(value: string): this;
  tabular(value: boolean): this;
}

export class TextRun extends Component {
  private readonly _family: "TextRun";
  style(value: TextStyle): this;
}

export class RichText extends Component {
  private readonly _family: "RichText";
  style(value: TextStyle): this;
  run(value: TextRun): this;
  line(value: TextRun): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class Title extends Component {
  private readonly _family: "Title";
  style(value: TextStyle): this;
  rich(value: RichText): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class Subtitle extends Component {
  private readonly _family: "Subtitle";
  style(value: TextStyle): this;
  rich(value: RichText): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class Caption extends Component {
  private readonly _family: "Caption";
  style(value: TextStyle): this;
  rich(value: RichText): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class SourceNote extends Component {
  private readonly _family: "SourceNote";
  style(value: TextStyle): this;
  rich(value: RichText): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class Footnote extends Component {
  private readonly _family: "Footnote";
  style(value: TextStyle): this;
  rich(value: RichText): this;
  line_spacing(value: number): this;
  lineSpacing(value: number): this;
  rotation(value: number): this;
}

export class Labels extends Component {
  private readonly _family: "Labels";
  id(value: string): this;
  text(value: string): this;
  at(x: ScaleValue, y: ScaleValue): this;
  figure_at(x: number, y: number): this;
  figureAt(x: number, y: number): this;
  output_at(x: number, y: number): this;
  outputAt(x: number, y: number): this;
  offset(x: number, y: number): this;
  panel_at(panel: Panel | null, x: number, y: number): this;
  panelAt(panel: Panel | null, x: number, y: number): this;
  panel(value: Panel): this;
  axes(x: string, y: string): this;
  priority(value: number): this;
  collision(value: string): this;
  overflow(value: string): this;
  style(value: TextStyle): this;
  rich(value: RichText): this;
}

export class Callout extends Component {
  private readonly _family: "Callout";
  label(value: Labels): this;
  at(x: ScaleValue, y: ScaleValue): this;
  to_data(x: ScaleValue, y: ScaleValue): this;
  toData(x: ScaleValue, y: ScaleValue): this;
  to(value: Options): this;
  text(value: string): this;
  connector_origin(value: string): this;
  connectorOrigin(value: string): this;
  offset(x: number, y: number): this;
  style(value: TextStyle): this;
}

export class PanelLetter extends Component {
  private readonly _family: "PanelLetter";
  panel(value: Panel): this;
  style(value: TextStyle): this;
  rich(value: RichText): this;
}

export class Inset extends Component {
  private readonly _family: "Inset";
  id(value: string): this;
  panel(value: Panel): this;
  rectangle(x: number, y: number, width: number, height: number): this;
  x_view(start: number, end: number): this;
  xView(start: number, end: number): this;
  y_view(start: number, end: number): this;
  yView(start: number, end: number): this;
  guides(value: boolean): this;
  layer(value: Layer): this;
}

export class NumberFormat extends Component {
  private readonly _family: "NumberFormat";
  notation(value: string | Options): this;
  precision(value: number): this;
  locale(value: Options): this;
  grouping(value: boolean): this;
  prefix(value: string): this;
  suffix(value: string): this;
}

export class LayoutOptions extends Component {
  private readonly _family: "LayoutOptions";
  font_size(value: number): this;
  fontSize(value: number): this;
  padding(value: number): this;
  tick_length(value: number): this;
  tickLength(value: number): this;
  label_gap(value: number): this;
  labelGap(value: number): this;
  minimum_plot(value: readonly [number, number]): this;
  minimumPlot(value: readonly [number, number]): this;
  target_ticks(value: number): this;
  targetTicks(value: number): this;
  max_ticks(value: number): this;
  maxTicks(value: number): this;
  max_categories(value: number): this;
  maxCategories(value: number): this;
  max_vertices(value: number): this;
  maxVertices(value: number): this;
  limits(value: Options): this;
  host_theme(value: Options): this;
  hostTheme(value: Options): this;
  output_theme(value: Options): this;
  outputTheme(value: Options): this;
  interaction_theme(value: Options): this;
  interactionTheme(value: Options): this;
  figure_bounds(value: readonly [number, number, number, number] | null): this;
  figureBounds(value: readonly [number, number, number, number] | null): this;
  host_style(value: Style): this;
  hostStyle(value: Style): this;
  output_style(value: Style): this;
  outputStyle(value: Style): this;
}

export class RenderOptions extends Component {
  private readonly _family: "RenderOptions";
  line_bucket_width(value: number | null): this;
  lineBucketWidth(value: number | null): this;
  candle_bucket_width(value: number | null): this;
  candleBucketWidth(value: number | null): this;
  max_columns(value: number): this;
  maxColumns(value: number): this;
  candle_volume(layer: Layer, field: Field): this;
  candleVolume(layer: Layer, field: Field): this;
}

export class StreamOptions extends Component {
  private readonly _family: "StreamOptions";
  transactions(value: number): this;
  rows(value: number): this;
  bytes(value: number): this;
  overload(value: string): this;
}

export class AnnotationEdit extends Component {
  private readonly _family: "AnnotationEdit";
  part(value: string): this;
  horizontal(value: boolean): this;
  vertical(value: boolean): this;
  x(value: Options): this;
  y(value: Options): this;
  preserve_order(horizontal: boolean | null): this;
  preserveOrder(horizontal: boolean | null): this;
}

export class Link extends Component {
  private readonly _family: "Link";
  axis(source: Axis, destination: Axis): this;
  selection(value: boolean): this;
  source_panel(value: Panel): this;
  sourcePanel(value: Panel): this;
  destination_panel(value: Panel): this;
  destinationPanel(value: Panel): this;
  missing(value: string): this;
}
export function aes(): Aes;
export function points(): Layer;
export function line(): Layer;
export function area(): Layer;
export function ribbon(): Layer;
export function bars(): Layer;
export function volume(): Layer;
export function ohlc(): Layer;
export function rule(): Layer;
export function rectangle(): Layer;
export function cells(): Layer;
export function histogram(): Layer;
export function identity_stat(): Stat;
export function identityStat(): Stat;
export function bin(): Stat;
export function count(): Stat;
export function summary(): Stat;
export function fit(): Stat;
export function custom_stat(name: string, version: number | bigint, parameters: JSONValue): Stat;
export function customStat(name: string, version: number | bigint, parameters: JSONValue): Stat;
export function stat_aes(): StatAes;
export function statAes(): StatAes;
export function bin_aes(): BinAes;
export function binAes(): BinAes;
export function stack(order: ReadonlyArray<string | number | boolean | Options>): Position;
export function dodge(order: ReadonlyArray<string | number | boolean | Options>): Position;
export function jitter(seed: number | bigint): Position;
export function filter(field: string | number | Options): Filter;
export function transform(name: string, stat: Stat): Transform;
export function scale_linear(): Scale;
export function scaleLinear(): Scale;
export function scale_log(base: number): Scale;
export function scaleLog(base: number): Scale;
export function scale_symlog(threshold: number): Scale;
export function scaleSymlog(threshold: number): Scale;
export function scale_band(): Scale;
export function scaleBand(): Scale;
export function scale_point(): Scale;
export function scalePoint(): Scale;
export function scale_utc(): Scale;
export function scaleUtc(): Scale;
export function scale_session(calendar: Options): Scale;
export function scaleSession(calendar: Options): Scale;
export function x_axis(): Axis;
export function xAxis(): Axis;
export function y_axis(): Axis;
export function yAxis(): Axis;
export function color_discrete(name: string): ColorScale;
export function colorDiscrete(name: string): ColorScale;
export function color_continuous(name: string, start: number, end: number): ColorScale;
export function colorContinuous(name: string, start: number, end: number): ColorScale;
export function legend(): Legend;
export function facet_wrap(field: string): Facet;
export function facetWrap(field: string): Facet;
export function facet_grid(row: string, column: string): Facet;
export function facetGrid(row: string, column: string): Facet;
export function style(): Style;
export function theme(): Theme;
export function text_style(): TextStyle;
export function textStyle(): TextStyle;
export function text_run(text: string): TextRun;
export function textRun(text: string): TextRun;
export function rich_text(text: string): RichText;
export function richText(text: string): RichText;
export function title(text: string): Title;
export function subtitle(text: string): Subtitle;
export function caption(text: string): Caption;
export function source_note(text: string): SourceNote;
export function sourceNote(text: string): SourceNote;
export function footnote(text: string): Footnote;
export function labels(): Labels;
export function callout(): Callout;
export function panel_letter(text: string): PanelLetter;
export function panelLetter(text: string): PanelLetter;
export function inset(): Inset;
export function number_format(): NumberFormat;
export function numberFormat(): NumberFormat;
export function layout_options(): LayoutOptions;
export function layoutOptions(): LayoutOptions;
export function render_options(): RenderOptions;
export function renderOptions(): RenderOptions;
export function stream_options(): StreamOptions;
export function streamOptions(): StreamOptions;
export function annotation_edit(id: string): AnnotationEdit;
export function annotationEdit(id: string): AnnotationEdit;
export function link(origin: string): Link;
export class PlotBuilder extends Owned {
 data(value: Data): this;
 aes(value: Aes): this;
 layer(value: Layer | Labels | Callout): this;
 title(value: Title): this;
 subtitle(value: Subtitle): this;
 caption(value: Caption): this;
 source_note(value: SourceNote): this;
 sourceNote(value: SourceNote): this;
 footnote(value: Footnote): this;
 x_axis(value: Axis): this;
 xAxis(value: Axis): this;
 y_axis(value: Axis): this;
 yAxis(value: Axis): this;
 axis(value: Axis): this;
 scale(value: ColorScale): this;
 legend(value: Legend): this;
 facet(value: Facet): this;
 theme(value: Theme): this;
 panel_letter(value: PanelLetter): this;
 panelLetter(value: PanelLetter): this;
 inset(value: Inset): this;
 transform(value: Transform): this;
 profile(value: string): this;
 compile_limits(value: Options): this;
 compileLimits(value: Options): this;
 data_limits(value: Options): this;
 dataLimits(value: Options): this;
 build(): Plot;
}
export class PlotEdit extends Owned {
 layer(name: string, value: Layer): this;
 annotation(value: Labels | Callout): this;
 remove_layer(name: string): this;
 removeLayer(name: string): this;
 remove_annotation(id: string): this;
 removeAnnotation(id: string): this;
 clear_facets(): this;
 clearFacets(): this;
 title(value: Title): this;
 subtitle(value: Subtitle): this;
 caption(value: Caption): this;
 source_note(value: SourceNote): this;
 sourceNote(value: SourceNote): this;
 footnote(value: Footnote): this;
 x_axis(value: Axis): this;
 xAxis(value: Axis): this;
 y_axis(value: Axis): this;
 yAxis(value: Axis): this;
 axis(value: Axis): this;
 scale(value: ColorScale): this;
 legend(value: Legend): this;
 facet(value: Facet): this;
 theme(value: Theme): this;
 panel_letter(value: PanelLetter): this;
 panelLetter(value: PanelLetter): this;
 inset(value: Inset): this;
 transform(value: Transform): this;
 build(): Plot;
}
export function plot(data: Data): PlotBuilder;
export class Plot extends Owned {
 edit(): PlotEdit;
 chart(): Chart;
 to_json(): string;
 toJson(): string;
 static from_json(value: string): Plot;
 static fromJson(value: string): Plot;
}
export class ExportOptions extends Owned {
 dpi(value: number): this;
 page(width: number, height: number, unit: 'pt' | 'mm'): this;
 precision(value: number): this;
 max_raster_pixels(value: number | bigint): this;
 maxRasterPixels(value: number | bigint): this;
 max_output_bytes(value: number): this;
 maxOutputBytes(value: number): this;
 background(value: Color): this;
 interaction(value: Options): this;
 basis(value: 'presented' | 'current' | 'Presented' | 'Current'): this;
 view(value: 'visible' | 'full_domain' | 'VisibleView' | 'FullDomain'): this;
 text(value: 'preserve' | 'outline' | 'Preserve' | 'Outline'): this;
 layout(value: LayoutOptions): this;
}
export function export_options(width: number, height: number, unit?: 'pt' | 'mm'): ExportOptions;
export function exportOptions(width: number, height: number, unit?: 'pt' | 'mm'): ExportOptions;
export class FigureRequest extends Owned {
 prepare(): FigureSnapshot;
 manifest(): Record<string, unknown>;
}
export class FigureSnapshot extends Owned {
 scene(): Record<string, unknown>;
 manifest(): Record<string, unknown>;
 export(format: Format): Uint8Array;
}
export class Output extends Owned {
 constructor(font: Iterable<number>);
 primary_font(): Record<string, unknown>;
 primaryFont(): Record<string, unknown>;
 register_font(font: Iterable<number>): Record<string, unknown>;
 registerFont(font: Iterable<number>): Record<string, unknown>;
 request(source: Plot | Chart, options: ExportOptions): FigureRequest;
 export(source: Plot | Chart, format: Format, options: ExportOptions): Uint8Array;
}
export class ExportJob extends Owned { cancel(): boolean; run(): Uint8Array; }
export class ExportQueue extends Owned {
 constructor(options?: {maxJobs?: number; maxInputBytes?: number; maxRows?: number});
 submit(request: FigureRequest, format: Format): ExportJob;
 metrics(): Record<string, unknown>;
}
export class Transaction extends Owned {}
export class Updates extends Owned {
 id(value: string): Updates;
 append(target: Data | string, data: Data): Updates;
 upsert(target: Data | string, data: Data): Updates;
 replace(target: Data | string, data: Data): Updates;
 remove(target: Data | string, keys: Iterable<number | bigint>): Updates;
 retain_count(target: Data | string, count: number | null): Updates;
 retainCount(target: Data | string, count: number | null): Updates;
 retain_event_time(target: Data | string, field: string, width: number | bigint, watermark: number | bigint, options?: {allowedLateness?: number | bigint; late?: 'Reject' | 'Drop'}): Updates;
 retainEventTime(target: Data | string, field: string, width: number | bigint, watermark: number | bigint, options?: {allowedLateness?: number | bigint; late?: 'Reject' | 'Drop'}): Updates;
 retention(target: Data | string, policy: string | Options): Updates;
 watermark(target: Data | string, ticks: number | bigint): Updates;
 reset_categories(target: Data | string, field: string): Updates;
 resetCategories(target: Data | string, field: string): Updates;
 build(): Transaction;
}
export class Editor extends Owned {
 original(): Record<string, unknown>;
 preview(dx: number, dy: number): Record<string, unknown>;
 nudge(options?: {horizontal?: boolean; forward?: boolean; steps?: number}): Record<string, unknown>;
}
export interface CommandOptions { expected?: number | bigint; }
export interface QueryOptions { gesture?: boolean; stamp?: Options; }
export interface NavigationOptions extends QueryOptions { axes?: ReadonlyArray<string>; panel?: Panel | null; boundary?: 'ClampToDomain' | 'Extend'; }
export class Chart extends Owned {
  external_view(): Chart;
  externalView(): Chart;
  accept_from(source: Chart): Record<string, unknown>;
  acceptFrom(source: Chart): Record<string, unknown>;
 constructor(plot: Plot);
 revisions(): Revisions;
 semantics(): Record<string, unknown>;
 state(): Record<string, unknown>;
 apply_plot(plot: Plot, expected: number | bigint): boolean;
 applyPlot(plot: Plot, expected: number | bigint): boolean;
 restore_state(state: Options, expected: number | bigint): void;
 restoreState(state: Options, expected: number | bigint): void;
 act(action: string | Options, options?: CommandOptions & {origin?: string | Options}): DispatchOutcome;
 query(operation: Options, options?: QueryOptions): Record<string, unknown>;
 layer_visible(layer: string, visible: boolean, options?: CommandOptions): DispatchOutcome;
 layerVisible(layer: string, visible: boolean, options?: CommandOptions): DispatchOutcome;
 legend_visible(visible: boolean, options?: CommandOptions): DispatchOutcome;
 legendVisible(visible: boolean, options?: CommandOptions): DispatchOutcome;
 follow(mode?: 'FollowLatest' | 'InspectHistory' | 'FreezePresentation', options?: CommandOptions): DispatchOutcome;
 freeze(options?: CommandOptions): DispatchOutcome;
 resume(options?: CommandOptions): DispatchOutcome;
 reset(options?: CommandOptions): DispatchOutcome;
 undo(options?: CommandOptions): DispatchOutcome;
 redo(options?: CommandOptions): DispatchOutcome;
 clear_inspection(options?: CommandOptions): DispatchOutcome;
 clearInspection(options?: CommandOptions): DispatchOutcome;
 select(targets: ReadonlyArray<Options>, change?: 'Replace' | 'Add' | 'Remove' | 'Toggle', options?: CommandOptions): DispatchOutcome;
 hover(targets: ReadonlyArray<Options>, options?: CommandOptions): DispatchOutcome;
 focus(target?: Options | null, options?: CommandOptions): DispatchOutcome;
 pin(target?: Options | null, options?: CommandOptions): DispatchOutcome;
 set_annotation(annotation: Options, options?: CommandOptions): DispatchOutcome;
 setAnnotation(annotation: Options, options?: CommandOptions): DispatchOutcome;
 remove_annotation(id: string, options?: CommandOptions): DispatchOutcome;
 removeAnnotation(id: string, options?: CommandOptions): DispatchOutcome;
 set_windows(windows: Options, options?: CommandOptions): DispatchOutcome;
 setWindows(windows: Options, options?: CommandOptions): DispatchOutcome;
 editor(options: AnnotationEdit): Editor;
 describe(options?: QueryOptions & {offset?: number; limit?: number}): Record<string, unknown>;
 inspect(x: number, y: number, options?: QueryOptions & {radius?: number; maxGrouped?: number; mode?: 'Auto' | 'NearestX' | 'NearestPoint' | 'Containment'}): Record<string, unknown>;
 select_region(region: Options, options?: QueryOptions & {limit?: number}): SelectionResult;
 selectRegion(region: Options, options?: QueryOptions & {limit?: number}): SelectionResult;
 select_series(layer: string, options?: QueryOptions & {panel?: Panel | null; limit?: number}): SelectionResult;
 selectSeries(layer: string, options?: QueryOptions & {panel?: Panel | null; limit?: number}): SelectionResult;
 navigate(action: string | Options, options?: NavigationOptions): WindowResult;
 zoom(x: number, y: number, factor: number, options?: NavigationOptions): WindowResult;
 pan(dx: number, dy: number, options?: NavigationOptions): WindowResult;
 range(axis: string, window: string | Options, options?: QueryOptions & {panel?: Panel | null}): WindowResult;
 present(output: Output, options: ExportOptions): FigureSnapshot;
 request(output: Output, options: ExportOptions): FigureRequest;
 stream(options: StreamOptions): this;
 transaction(): Updates;
 commit(transaction: Transaction): Record<string, unknown>;
 enqueue(transaction: Transaction): Record<string, unknown> | string;
 stream_status(): Record<string, unknown>;
 streamStatus(): Record<string, unknown>;
 pinned(): Record<string, unknown> | null;
 queue_status(): Record<string, unknown>;
 queueStatus(): Record<string, unknown>;
 commit_next(): Record<string, unknown> | null;
 commitNext(): Record<string, unknown> | null;
 reset_epoch(): Record<string, unknown>;
 resetEpoch(): Record<string, unknown>;
 dense(frame: FigureSnapshot, options: RenderOptions): Record<string, unknown>;
 link_capture(component: Link, event: Options): Record<string, unknown> | null;
 linkCapture(component: Link, event: Options): Record<string, unknown> | null;
 link_resolve(component: Link, message: Options): Record<string, unknown>;
 linkResolve(component: Link, message: Options): Record<string, unknown>;
}
export { Chart as LegacyChart } from './chart_wasm.js';
