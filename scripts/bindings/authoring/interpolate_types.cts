import * as c from '../../../target/interpolate-proof/wasm-module/authoring.cjs';
const number:c.Interpolator<number>=c.interpolateNumber(2,10);
const value:number=number.sample(.5);
const samples:number[]=c.quantize(number,3);
const text:string=c.interpolateString('2px','10px').sample(.5);
const date:Date=c.interpolateDate(new Date(0),new Date(1000)).sample(.5);
const typed:c.InterpolationNumericArray=c.interpolateNumberArray([0,1],new Uint8ClampedArray([2,3])).sample(.5);
const rgb:string=c.interpolateRgb.gamma(2.2)('red','blue').sample(.5);
const piece:c.Interpolator<number>=c.piecewise(c.interpolateNumber,[0,10,0]);
const colors:c.Interpolator<string>=c.piecewise(c.interpolateRgb.gamma(2),['red','blue','white']);
const transform:string=c.interpolateTransformCss('none','translate(10px,20px)').sample(.5);
const matrix:number[]=c.interpolateTransformSvg([1,0,0,1,0,0],[1,0,0,1,10,20]).sampleTransform(.5);
const zoom:number[]=c.interpolateZoom.rho(1)([0,0,10],[0,0,1]).sample(.5);
const duration:number|undefined=c.interpolateZoom([0,0,10],[0,0,1]).schedulingDuration;
// @ts-expect-error Sampling parameters are numeric.
number.sample('0.5');
// @ts-expect-error Numeric interpolation has no gamma configuration.
c.interpolateNumber.gamma(2);
// @ts-expect-error BigInt arrays are excluded from the numeric profile.
c.numericArray('BigInt64Array',[1,2]);
// @ts-expect-error Rho is a number.
c.interpolateZoom.rho('1');
// @ts-expect-error BigInt is not a binary64 interpolation scalar.
c.interpolateNumber(1n,2n);
// @ts-expect-error Unregistered callbacks cannot be portable factories.
c.piecewise((a:number,b:number)=>c.interpolateNumber(a,b),[0,1]);
const catalog:c.ChromaticCatalog=c.chromaticCatalog();
const named:c.Interpolator<c.ColorValue>=c.chromatic('Viridis',true);
const palette:c.SchemeSpec={id:'Blues',size:3,reverse:true};
c.colorMapped('q',new c.StandaloneScale('quantile',{range:['red']})).paletteScheme(palette);
// @ts-expect-error unknown catalog name
c.chromatic('ViridisTypo');
// @ts-expect-error categorical scheme has no interpolator
c.chromatic('Category10');
// @ts-expect-error integer palette size is a number
c.chromaticScheme('Blues','3');
