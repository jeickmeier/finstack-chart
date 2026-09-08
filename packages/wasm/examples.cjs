'use strict';
// Primary components for the compiled extension-proof example registry; no supplied code is evaluated.
const c=require('./authoring.cjs');
function withExtensions(builder){return new c.PlotBuilder(builder._inner.with_example_extensions());}
function densityHistogram(field,edges){return c.customStat('example.density_histogram',1n,{edges:Array.from(edges)}).fieldParameter('input',field);}
function chamferedBars(native=false){return c.rectangle().geometry(native?'example.native_bars':'example.chamfered_bars',1n,null).afterStat(c.statAes().x({Custom:'left'}).y(0).x2({Custom:'right'}).y2({Custom:'density'}));}
function loadPlot(value){return c.Plot._from_example_json(value);}
module.exports={withExtensions,densityHistogram,chamferedBars,loadPlot};
