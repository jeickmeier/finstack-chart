import * as c from '../../../target/ggplot-stages/primary/wasm-module/authoring.cjs';
const data=c.Data.columns({x:[1,2],y:[10,100]});
const source=c.sourceExpr(data.field('y')).mul(2).add(1);
const fraction=c.binExpr('Count').div(c.binExpr('Count').sum());
const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(source))
 .layer(c.points().stat(c.summary().x(source)).afterStat(c.statAes().x(1).y(c.statExpr('Mean').mul(2))))
 .layer(c.histogram().afterBin(c.binAes().y(fraction)))
 .layer(c.points().afterScale(c.scaleAes().size(c.afterScaleExpr('Size').mul(2)).color(c.fromTheme('Accent'))))
 .theme(c.theme().geometry({accent:'#1256ab',point_size:1.5}))
 .yAxis(c.yAxis().coordinateScale(c.scaleLog(10)).oob('Keep')).build();
plot.edit().profile('LibraryV1').build();
// @ts-expect-error A generated accessor cannot enter a source expression.
c.aes().x(c.statExpr('Mean'));
// @ts-expect-error Source reads cannot enter the generated stage.
c.statAes().y(c.sourceExpr('y'));
// @ts-expect-error Bin fields have their own generated schema.
c.binAes().y(c.statExpr('Count'));
// @ts-expect-error Scale modifiers cannot read source columns.
c.scaleAes().size(c.sourceExpr('y'));
// @ts-expect-error Binary expressions preserve their read stage.
c.sourceExpr('x').add(c.statExpr('Mean'));
