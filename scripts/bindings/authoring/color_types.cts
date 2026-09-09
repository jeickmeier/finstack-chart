import * as c from '../../../target/color-proof/wasm-module/authoring.cjs';
const parsed=c.color('steelblue');
if(parsed){
 const copied=parsed.copy({opacity:.5}).convert('Lab');
 const value:number=copied.channel('l');
 const text:string=copied.formatHsl();
 c.ColorValue.fromJson(copied.toJson());c.rgb(copied);
}
c.rgb(1,2,3,.5).brighter().clamp();
c.hsl(120,.5,.2).darker(-1);c.lab(50,30,40).convert('Hcl');
c.hcl(30,40,50);c.lch(50,40,30);c.gray(50,.5);c.cubehelix(30,.5,.2).displayable();
// @ts-expect-error incomplete numeric channels
c.rgb(1,2);
// @ts-expect-error numeric channels have fixed types
c.hsl('red','green','blue');
// @ts-expect-error parser takes text
c.color(2);
// @ts-expect-error gray requires numeric lightness
c.gray('red');
// @ts-expect-error supported space identity
c.rgb(1,2,3).convert('unknown');

const paint=c.lab(50,20,30);
c.points().color(paint).style(c.style().mark(paint).panel(paint));
c.text_style().color(paint);
c.theme().geometry({ink:paint,paper:paint,accent:paint});
c.color_continuous('ramp',0,1).palette([paint,'red']).missing(paint);
c.color_discrete('group').palette([paint]).missing(paint);
c.export_options(400,300).background(paint);
