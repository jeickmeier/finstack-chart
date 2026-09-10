import {ExtensionRegistry, ShapeRegistry, Scale, PlotBuilder, scaleRegistered, scale_registered, plot, Data, xAxis} from '../../../packages/wasm/authoring.cjs';
const registry:ShapeRegistry=ExtensionRegistry.example();
const scale:Scale=scaleRegistered('example.fold',1n,{limit:10});
const exact:Scale=scale_registered('example.fold','1',{limit:10});
const p:PlotBuilder=plot(Data.columns({x:new Float64Array([1])})).withRegistry(registry);
xAxis().coordinateScale(scale);
// @ts-expect-error version must be an exact integer representation
scaleRegistered('example.fold',false,{});
// @ts-expect-error callbacks are not portable parameters
scaleRegistered('example.fold',1,()=>1);
// @ts-expect-error explicit registered-code owner required
p.withRegistry('not a registry');
void [p,exact];
