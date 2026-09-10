import {Owned,ShapeRegistry} from './authoring.cjs';
import {InterpolationInput,InterpolationResult,Interpolator,BinaryInterpolationFactory,NumericInterpolationFactory,ColorInterpolationFactory} from './interpolation.cjs';
export type ScaleFamily='linear'|'log'|'pow'|'sqrt'|'symlog'|'identity'|'radial'|'ordinal'|'band'|'point'|'quantile'|'quantize'|'threshold'|'sequential'|'sequential_log'|'sequential_pow'|'sequential_sqrt'|'sequential_symlog'|'sequential_quantile'|'diverging'|'diverging_log'|'diverging_pow'|'diverging_sqrt'|'diverging_symlog'|'utc'|'local';
export type ScaleTimeUnit='Seconds'|'Milliseconds'|'Microseconds'|'Nanoseconds';
export class ScaleKey {
  constructor(kind:'Integer'|'Unsigned'|'Timestamp',value:bigint);
  constructor(kind:'Number',value:number);
  constructor(kind:'Text',value:string);
  constructor(kind:'Boolean',value:boolean);
  constructor(kind:'Null',value?:null);
  readonly kind:string; readonly value:unknown;
}
export type ScaleArgument=undefined|null|boolean|number|bigint|string|ScaleKey;
export interface CalendarInterval {unit:'SourceTick'|'Millisecond'|'Second'|'Minute'|'Hour'|'Day'|'UnixDay'|'Month'|'Year'|{Week:'Sunday'|'Monday'};step:number;}
export interface NumericLocale {decimal?:string;thousands?:string|null;grouping?:readonly number[]|null;currency?:readonly [string,string];numerals?:readonly string[]|null;percent?:string;minus?:string;nan?:string;}
export interface TimeLocale {dateTime?:string;date?:string;time?:string;periods?:readonly [string,string];days?:readonly string[];shortDays?:readonly string[];months?:readonly string[];shortMonths?:readonly string[];}
export interface TimeZoneRules {version:1;zone:string;revision:string;tzdata:string;coverage:{start:string;end:string};initial_offset_seconds:number;transitions:readonly {at_millis:string;offset_seconds:number}[];}
export type CalendarZone='Utc'|{Local:TimeZoneRules};
export interface InterpolationFactorySpec {kind:'Value'|'Number'|'Round'|'String'|'Date'|'Array'|'NumberArray'|'Object'|'Hue'|'Rgb'|'Hsl'|'HslLong'|'Lab'|'Hcl'|'HclLong'|'Cubehelix'|'CubehelixLong';gamma?:number;}
export interface ScaleOptions {
  domain?:Iterable<ScaleArgument>;range?:Iterable<InterpolationInput>;clamp?:boolean;round?:boolean;unknown?:InterpolationInput;implicit?:boolean;
  base?:number;exponent?:number;constant?:number;padding?:number;padding_inner?:number;padding_outer?:number;align?:number;
  factory?:InterpolationFactorySpec|BinaryInterpolationFactory<unknown>|NumericInterpolationFactory|ColorInterpolationFactory;
  interpolator?:Interpolator<unknown>;unit?:ScaleTimeUnit;zone?:CalendarZone;
}
export interface InverseExtent {found:boolean;lower:ScaleArgument;upper:ScaleArgument;}
export interface ScaleFormatOptions {count?:number;specifier?:string|null;pattern?:string|null;locale?:NumericLocale|TimeLocale;}
export class StandaloneScale extends Owned {
  constructor(family?:ScaleFamily,options?:ScaleOptions & {registry?:ShapeRegistry});
  static from_json(text:string,registry?:ShapeRegistry):StandaloneScale;
  static fromJson(text:string,registry?:ShapeRegistry):StandaloneScale;
  static from_spec(spec:Readonly<Record<string,unknown>>,registry?:ShapeRegistry):StandaloneScale;
  static fromSpec(spec:Readonly<Record<string,unknown>>,registry?:ShapeRegistry):StandaloneScale;
  to_json():string;toJson():string;
  spec():Record<string,unknown>;
  mapped(training?:'Authored'|'Eligible'):Record<string,unknown>;
  copy():StandaloneScale;
  configure(options:ScaleOptions):StandaloneScale;
  reconfigure(spec:Readonly<Record<string,unknown>>,registry?:ShapeRegistry):StandaloneScale;
  domain():ScaleArgument[];range():InterpolationResult[];
  map(value?:ScaleArgument):InterpolationResult;
  map_value(value?:ScaleArgument):Record<string,unknown>;mapValue(value?:ScaleArgument):Record<string,unknown>;
  invert(value:number):InterpolationResult|bigint;
  invert_extent(value:InterpolationInput):InverseExtent;invertExtent(value:InterpolationInput):InverseExtent;
  ticks(count?:number,options?:{interval?:CalendarInterval;budget?:number}):ScaleArgument[];
  format(value:ScaleArgument,options?:ScaleFormatOptions):string;
  nice(count?:number,options?:{interval?:CalendarInterval}):StandaloneScale;
  train(values:Iterable<ScaleArgument>):StandaloneScale;
  thresholds():(number|undefined)[];quantiles(count:number):(number|undefined)[];
  step():number;bandwidth():number;
  extent(value:ScaleArgument):[number,number]|undefined;center(value:ScaleArgument):number|undefined;
  floor(value:bigint,interval:CalendarInterval):bigint;
  ceil(value:bigint,interval:CalendarInterval):bigint;
  round_time(value:bigint,interval:CalendarInterval):bigint;roundTime(value:bigint,interval:CalendarInterval):bigint;
  offset(value:bigint,interval:CalendarInterval,steps?:number):bigint;
}
