import * as c from '../../../target/interpolation-integration/wasm-module/authoring.cjs';
const registry=c.ExtensionRegistry.example();
const factory:c.RegisteredInterpolationFactory=c.registeredInterpolation(registry,'example.interpolation',1n,{mode:'SquaredNumber'});
const value:c.Interpolator<c.InterpolationResult>=factory(0,100);
const piece:c.Interpolator<c.InterpolationResult>=c.piecewise(factory,[0,100,200]);
const scale=new c.StandaloneScale('linear',{registry,factory});
c.StandaloneScale.fromJson(scale.toJson(),registry);
c.Interpolator.fromJson(value.toJson(),registry);
factory.copy().dispose();
// @ts-expect-error version is exact integer syntax, never bool
c.registeredInterpolation(registry,'example.interpolation',true,{});
// @ts-expect-error no arbitrary host callbacks
c.registeredInterpolation(registry,'example.interpolation',1,()=>{});
// @ts-expect-error installed registry handle required
new c.StandaloneScale('linear',{registry:'bad'});
// @ts-expect-error installed registry handle required
c.Interpolator.fromJson('{}','bad');
// @ts-expect-error callbacks lack the installed factory identity
c.piecewise((a:number,b:number)=>c.interpolateNumber(a,b),[0,1]);
