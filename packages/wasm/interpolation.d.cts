import {ColorValue,Owned} from './authoring.cjs';
export type NumericArrayKind='Float32Array'|'Float64Array'|'Int8Array'|'Uint8Array'|'Uint8ClampedArray'|'Int16Array'|'Uint16Array'|'Int32Array'|'Uint32Array';
export type InterpolationNumericArray=Float32Array|Float64Array|Int8Array|Uint8Array|Uint8ClampedArray|Int16Array|Uint16Array|Int32Array|Uint32Array;
export type InterpolationInput=null|undefined|boolean|number|string|Date|ColorValue|InterpolationNumericArray|ReadonlyArray<InterpolationInput>|{readonly [key:string]:InterpolationInput};
export type InterpolationResult=null|undefined|boolean|number|string|Date|ColorValue|InterpolationNumericArray|InterpolationResult[]|{[key:string]:InterpolationResult};
export const MISSING:undefined;
export function date_value(milliseconds:number):Date;
export {date_value as dateValue};
export function numeric_array(kind:NumericArrayKind,values:Iterable<number>):InterpolationNumericArray;
export {numeric_array as numericArray};
export class Interpolator<T=InterpolationResult> extends Owned{
  static from_json(value:string):Interpolator;
  static fromJson(value:string):Interpolator;
  to_json():string;toJson():string;
  copy():Interpolator<T>;
  sample(t:number):T;
  sample_value(t:number):Record<string,unknown>;sampleValue(t:number):Record<string,unknown>;
  sample_color(t:number):ColorValue;sampleColor(t:number):ColorValue;
  sample_transform(t:number):number[];sampleTransform(t:number):number[];
  quantize(count:number):T[];
  readonly duration:number|undefined;
  readonly scheduling_duration:number|undefined;readonly schedulingDuration:number|undefined;
}
declare const factoryBrand:unique symbol;
interface BuiltinFactory{readonly [factoryBrand]:true;}
export interface BinaryInterpolationFactory<T> extends BuiltinFactory{(a:InterpolationInput,b:InterpolationInput):Interpolator<T>;}
type NumericInput=number|boolean|string|null|undefined|Date;
type ColorInput=string|ColorValue|null|undefined;
type ArrayInput=ReadonlyArray<InterpolationInput>|InterpolationNumericArray|null|undefined;
export interface NumericInterpolationFactory extends BuiltinFactory{(a:NumericInput,b:NumericInput):Interpolator<number>;}
export interface ColorInterpolationFactory extends BuiltinFactory{(a:ColorInput,b:ColorInput):Interpolator<string>;}
export interface GammaInterpolationFactory extends ColorInterpolationFactory{gamma(value:number):GammaInterpolationFactory;}
export interface ZoomInterpolationFactory{(a:ReadonlyArray<number>|InterpolationNumericArray,b:ReadonlyArray<number>|InterpolationNumericArray):Interpolator<number[]>;rho(value:number):ZoomInterpolationFactory;}
export const interpolate:BinaryInterpolationFactory<InterpolationResult>;
export const interpolate_number:NumericInterpolationFactory;
export const interpolate_round:NumericInterpolationFactory;
export const interpolate_hue:NumericInterpolationFactory;
export const interpolate_string:BuiltinFactory & ((a:number|boolean|string|null|undefined|ColorValue,b:number|boolean|string|null|undefined|ColorValue)=>Interpolator<string>);
export const interpolate_date:BuiltinFactory & ((a:NumericInput,b:NumericInput)=>Interpolator<Date>);
export interface ArrayInterpolationFactory extends BuiltinFactory{
  (a:ArrayInput,b:InterpolationNumericArray):Interpolator<InterpolationNumericArray>;
  (a:ArrayInput,b:ReadonlyArray<InterpolationInput>|null|undefined):Interpolator<InterpolationResult[]>;
}
export const interpolate_array:ArrayInterpolationFactory;
export const interpolate_number_array:typeof interpolate_array;
export const interpolate_object:BinaryInterpolationFactory<{[key:string]:InterpolationResult}>;
export const interpolate_rgb:GammaInterpolationFactory;
export const interpolate_hsl:ColorInterpolationFactory;
export const interpolate_hsl_long:ColorInterpolationFactory;
export const interpolate_lab:ColorInterpolationFactory;
export const interpolate_hcl:ColorInterpolationFactory;
export const interpolate_hcl_long:ColorInterpolationFactory;
export const interpolate_cubehelix:GammaInterpolationFactory;
export const interpolate_cubehelix_long:GammaInterpolationFactory;
export function interpolate_basis(values:ReadonlyArray<number>|InterpolationNumericArray):Interpolator<number>;
export const interpolate_basis_closed:typeof interpolate_basis;
export function interpolate_rgb_basis(values:ReadonlyArray<ColorInput>):Interpolator<string>;
export const interpolate_rgb_basis_closed:typeof interpolate_rgb_basis;
export function interpolate_discrete<T extends InterpolationInput>(values:ReadonlyArray<T>):Interpolator<T>;
export function interpolate_transform_css(a:string,b:string):Interpolator<string>;
export function interpolate_transform_css(a:ReadonlyArray<number>|InterpolationNumericArray,b:ReadonlyArray<number>|InterpolationNumericArray):Interpolator<string>;
export function interpolate_transform_svg(a:string|null|undefined,b:string|null|undefined):Interpolator<string>;
export function interpolate_transform_svg(a:ReadonlyArray<number>|InterpolationNumericArray,b:ReadonlyArray<number>|InterpolationNumericArray):Interpolator<string>;
export const interpolate_zoom:ZoomInterpolationFactory;
export function piecewise(values:ReadonlyArray<InterpolationInput>):Interpolator;
export function piecewise<T>(factory:BuiltinFactory & ((a:never,b:never)=>Interpolator<T>),values:ReadonlyArray<InterpolationInput>):Interpolator<T>;
export function quantize<T>(interpolator:Interpolator<T>,count:number):T[];
export {interpolate_number as interpolateNumber,interpolate_round as interpolateRound,interpolate_hue as interpolateHue,interpolate_string as interpolateString,interpolate_date as interpolateDate,interpolate_array as interpolateArray,interpolate_number_array as interpolateNumberArray,interpolate_object as interpolateObject,interpolate_rgb as interpolateRgb,interpolate_hsl as interpolateHsl,interpolate_hsl_long as interpolateHslLong,interpolate_lab as interpolateLab,interpolate_hcl as interpolateHcl,interpolate_hcl_long as interpolateHclLong,interpolate_cubehelix as interpolateCubehelix,interpolate_cubehelix_long as interpolateCubehelixLong,interpolate_basis as interpolateBasis,interpolate_basis_closed as interpolateBasisClosed,interpolate_rgb_basis as interpolateRgbBasis,interpolate_rgb_basis_closed as interpolateRgbBasisClosed,interpolate_discrete as interpolateDiscrete,interpolate_transform_css as interpolateTransformCss,interpolate_transform_svg as interpolateTransformSvg,interpolate_zoom as interpolateZoom};

export type ChromaticSchemeId="Accent"|"Blues"|"BrBG"|"BuGn"|"BuPu"|"Category10"|"Dark2"|"GnBu"|"Greens"|"Greys"|"Observable10"|"OrRd"|"Oranges"|"PRGn"|"Paired"|"Pastel1"|"Pastel2"|"PiYG"|"PuBu"|"PuBuGn"|"PuOr"|"PuRd"|"Purples"|"RdBu"|"RdGy"|"RdPu"|"RdYlBu"|"RdYlGn"|"Reds"|"Set1"|"Set2"|"Set3"|"Spectral"|"Tableau10"|"YlGn"|"YlGnBu"|"YlOrBr"|"YlOrRd";
export type ChromaticInterpolatorId="Blues"|"BrBG"|"BuGn"|"BuPu"|"Cividis"|"Cool"|"CubehelixDefault"|"GnBu"|"Greens"|"Greys"|"Inferno"|"Magma"|"OrRd"|"Oranges"|"PRGn"|"PiYG"|"Plasma"|"PuBu"|"PuBuGn"|"PuOr"|"PuRd"|"Purples"|"Rainbow"|"RdBu"|"RdGy"|"RdPu"|"RdYlBu"|"RdYlGn"|"Reds"|"Sinebow"|"Spectral"|"Turbo"|"Viridis"|"Warm"|"YlGn"|"YlGnBu"|"YlOrBr"|"YlOrRd";
export interface SchemeSpec {id:ChromaticSchemeId;version?:1;size?:number|null;reverse?:boolean;}
export interface SchemeInfo {id:ChromaticSchemeId;family:'Categorical'|'Sequential'|'Diverging';sizes:number[];}
export interface ChromaticCatalog {version:1;schemes:SchemeInfo[];interpolators:ChromaticInterpolatorId[];}
export function chromatic_catalog():ChromaticCatalog;
export function chromatic_scheme(name:ChromaticSchemeId,size?:number|null,reverse?:boolean):ColorValue[];
export function chromatic(name:ChromaticInterpolatorId,reverse?:boolean):Interpolator<ColorValue>;
export {chromatic_catalog as chromaticCatalog,chromatic_scheme as chromaticScheme};
