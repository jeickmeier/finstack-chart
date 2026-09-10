import {xAxis,axisGuide,GuideTickArguments,GuideFormatter,Axis,Guide} from '../../../packages/wasm/authoring.cjs';
const args:GuideTickArguments={count:2.5,specifier:'.1%'};
const fmt:GuideFormatter={Registered:{operation:{id:'example.guide_format',version:1n},parameters:{mode:'Indexed'}}};
const a:Axis=xAxis().guideProfile('D3_3_0_0').tickArguments(args).tickValues([0,.5,1]).tickFormat(fmt);
const g:Guide=axisGuide('top','x').guide_profile('D3_3_0_0').tick_values([]).tick_values(null).tick_format(null);
// @ts-expect-error exact profile required
xAxis().guideProfile('D3');
// @ts-expect-error finite number count shape
xAxis().tickArguments({count:'five'});
// @ts-expect-error semantic label strings required
axisGuide('top','x').tickFormat({Labels:[3]});
void [a,g];
