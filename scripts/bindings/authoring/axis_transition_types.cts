import * as c from '../../../packages/wasm/authoring.cjs';
function capture(target:c.FigureSnapshot,previous:c.FigureSnapshot,chart:c.Chart,output:c.Output):c.FigureSnapshot {
  const plan:c.FigureTransition=target.guideTransition(previous);
  const sample=plan.sample(.5);
  const guides:Record<string,unknown>[]=sample.presentation();
  if(!guides.length)throw Error('missing guides');
  chart.acknowledgeFrame(sample);
  // @ts-expect-error fractions are numeric
  plan.sample('mid');
  // @ts-expect-error transition sources must be actual immutable figures
  target.guideTransition(chart);
  return chart.request(output,c.exportOptions(900,300).basis('displayed')).prepare();
}
