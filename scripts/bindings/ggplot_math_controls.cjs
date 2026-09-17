// GG14 independent supplied-font mathematical author/replay publications.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const fontdir=path.join(ROOT,'fixtures/math/fonts'),output=new c.Output(fs.readFileSync(path.join(fontdir,'DejaVuSerif.ttf'))),fonts={regular:output.primary_font()};
for(const [key,name] of [['italic','DejaVuSerif-Italic.ttf'],['bold','DejaVuSerif-Bold.ttf'],['bold_italic','DejaVuSerif-BoldItalic.ttf'],['symbol','DejaVuMathTeXGyre.ttf']])fonts[key]=output.register_font(fs.readFileSync(path.join(fontdir,name)));
const fallback=output.register_font(fs.readFileSync(path.join(fontdir,'DejaVuSans.ttf'))),cases=[['x[i]^2','frac(alpha,beta)','sqrt(x,3)','sum(x[i],i==1,n)'],['hat(x)','widehat(x+y)','widetilde(x+y)','ring(x)'],['bgroup("(",atop(x,y),")")','integral(f(x)*dx,a,b)','bold(x)+italic(y)','phantom(x)*y'],['alpha %in% A','x %->% y','scriptstyle(x[i])','group(langle,x,rangle)'],['plain(P)(X==x)','frac(1,sqrt(2*pi))*e^{-x^2/2}','bolditalic(x)','list(alpha,beta,gamma)']];
cases.push(cases[4],cases[4],cases[4],['paste("café",frac(alpha,beta))', 'paste("κόσμος",sqrt(x))', 'paste("мир",x[i]^2)', 'paste("naïve",hat(y))']);
for(let mode=0;mode<cases.length;mode++){
 const data=c.Data.columns({x:new Float64Array([1,3,1,3]),y:new Float64Array([3,3,1,1]),label:cases[mode],group:c.categorical(['alpha','beta','alpha','beta'])}),style=c.text_style().fallback(fallback),headline=c.math_text('paste(plain(Plotmath),": ",frac(alpha^2,beta))',fonts).style(style),ticks=c.text_style().math(fonts).fallback(fallback);
 let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points().text_geom({math:fonts,units:'Points',size:16,angle:mode===8?30:0}).text_label('label')).x_axis(c.x_axis().scale(c.scale_linear().domain(0,4)).ticks([[{Number:1},'alpha'],[{Number:3},'frac(1,2)']]).text_style(ticks).rich_label(c.math_text('sum(x[i],i==1,n)',fonts).style(style))).y_axis(c.y_axis().scale(c.scale_linear().domain(0,4)).rich_label(c.math_text('sqrt(y)',fonts).style(style))).title(c.title('').rich(headline));
 if(mode===5)builder=builder.facet(c.facet_wrap('group').reference({labeller:{math:fonts}}));
 else if(mode===6)builder=builder.layer(c.points().aes(c.aes().color('group').y(0.5))).legend(c.legend().aesthetic('Color').options({math:fonts}));
 else if(mode===7)builder=builder.layer(c.labels().panel_at(null,0.5,0.5).rich(c.math_text('frac(alpha,beta)',fonts).style(style)));
 const p=builder.build();
 const wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`math-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`math-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`math-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM math controls: nine authors, 27 publications.');
