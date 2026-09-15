import {StandaloneScale,ScaleKey,CalendarInterval,InterpolationResult,interpolateLab,interpolateRgb} from '../../../packages/wasm/authoring.cjs';
const s=new StandaloneScale('linear',{domain:[0,10,100],range:[0,0.5,1],clamp:true});
const inverse:InterpolationResult|bigint=s.invert(0.5);
const label:string=s.format(10,{specifier:'.2f'});
s.configure({round:true}).nice(4).dispose();
new StandaloneScale('linear',{range:['red','blue'],factory:interpolateLab});
new StandaloneScale('diverging',{domain:[-1,0,2],interpolator:interpolateRgb('red','blue')});
new StandaloneScale('ordinal',{domain:['a',1,1n,new ScaleKey('Unsigned',18446744073709551615n)],range:['low','high']}).train(['later']);
const t=new StandaloneScale('utc',{domain:[1700000000000000001n,1700000000000000101n],unit:'Nanoseconds'});
const interval:CalendarInterval={unit:'SourceTick',step:10};
const exact:bigint=t.offset(1700000000000000001n,interval);
t.ticks(10,{interval});t.format(exact,{pattern:'%f'});
// @ts-expect-error unknown family
new StandaloneScale('unknown');
// @ts-expect-error unsupported domain object
new StandaloneScale('linear',{domain:[{}]});
// @ts-expect-error unknown source unit
new StandaloneScale('utc',{unit:'fortnights'});
// @ts-expect-error exact timestamps require bigint
t.floor(100,interval);
// @ts-expect-error clamp requires boolean
s.configure({clamp:'yes'});
// @ts-expect-error exact integer key requires bigint
new ScaleKey('Unsigned','18446744073709551615');
void [inverse,label,exact];

import {scale_transform, x_axis} from "../../../packages/wasm/authoring.cjs";
x_axis().scale(scale_transform({Ggplot:{transform:{BoxCox:{p:0.5,offset:2}}}}));
