import * as c from '../../../packages/wasm/authoring.cjs';
function author(data:c.Data):c.Plot {
 const layer=c.shape_symbol().symbolKind({Ggplot:21}).fill('#ff0000').stroke('#000000').alpha(.5).radius(2).linewidth(1).line_type('Dashed').aesthetic_units('Millimeters')
  .shape_value('Alpha',data.field('alpha')).shape_value('StrokeWidth',data.field('width'))
  .value_scale('TextSize',c.source_expr(data.field('width')).sum(),new c.StandaloneScale('linear'))
  .aesthetic_value('FontFace',{kind:'Text',value:'bold'})
  .after_scale(c.scale_aes().fill(c.after_scale_expr('Color')).stroke(c.after_scale_expr('Fill')).alpha(c.after_scale_expr('Alpha')).linewidth(c.after_scale_expr('LineWidth')));
 return c.plot(data).aes(c.aes().x('x').y('y').fill('fill').stroke('stroke')).layer(layer).build();
}
void author;
function temporalDefaults():c.Plot {
 const data=c.Data.columns({x:[0,1],y:[0,1],when:c.timestamps([1704067200000000000n,1704067209000000000n],'ns','UTC').validity([true,true])});
 return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').size('when').alpha('when').linewidth('when').fill('when').color('when'))
  .layer(c.points().name('marks')).build().edit().layer('marks',c.points().stroke('#000000')).build();
}
void temporalDefaults;
