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
import {GuideGeometry,layoutOptions} from '../../../packages/wasm/authoring.cjs';
const geometry:GuideGeometry={inner:-40,outer:6,padding:-3,offset:.5,labels:'Preserve',overflow:'Clip',clip_ticks:true};
a.guideGeometry(geometry).tickSize(-6).tickSizeInner(0).tickSizeOuter(6).tickPadding(-3).tickOffset(null);
g.guideGeometry(geometry).tickSize(-6).tickSizeInner(0).tickSizeOuter(6).tickPadding(-3).tickOffset(null);
layoutOptions().deviceScale(2);
// @ts-expect-error explicit label policy
xAxis().guideGeometry({labels:'Automatic'});
import {GuideComponents} from '../../../packages/wasm/authoring.cjs';
const components:GuideComponents={domain:{color:'#113355',width:2,dashes:[4,2]},labels:{font_size:18},per_tick:[{index:2,line:{visible:false}}]};
a.guideComponents(components).guideComponents(null); g.guideComponents(components).guideComponents(null);
// @ts-expect-error numeric stroke width
xAxis().guideComponents({ticks:{width:'wide'}});
// @ts-expect-error explicit label component property
xAxis().guideComponents({labels:{weight:'bold'}});
