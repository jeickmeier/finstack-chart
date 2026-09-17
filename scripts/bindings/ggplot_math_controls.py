"""GG14 independent supplied-font mathematical author/replay publications."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
fontdir=ROOT/'fixtures/math/fonts'
output=c.Output((fontdir/'DejaVuSerif.ttf').read_bytes())
fonts={'regular':output.primary_font()}
for key,name in [('italic','DejaVuSerif-Italic.ttf'),('bold','DejaVuSerif-Bold.ttf'),('bold_italic','DejaVuSerif-BoldItalic.ttf'),('symbol','DejaVuMathTeXGyre.ttf')]:fonts[key]=output.register_font((fontdir/name).read_bytes())
fallback=output.register_font((fontdir/'DejaVuSans.ttf').read_bytes())
cases=[['x[i]^2','frac(alpha,beta)','sqrt(x,3)','sum(x[i],i==1,n)'],['hat(x)','widehat(x+y)','widetilde(x+y)','ring(x)'],['bgroup("(",atop(x,y),")")','integral(f(x)*dx,a,b)','bold(x)+italic(y)','phantom(x)*y'],['alpha %in% A','x %->% y','scriptstyle(x[i])','group(langle,x,rangle)'],['plain(P)(X==x)','frac(1,sqrt(2*pi))*e^{-x^2/2}','bolditalic(x)','list(alpha,beta,gamma)']]
cases += [cases[-1]] * 3
cases.append(['paste("café",frac(alpha,beta))', 'paste("κόσμος",sqrt(x))', 'paste("мир",x[i]^2)', 'paste("naïve",hat(y))'])
for mode,labels in enumerate(cases):
    data=c.Data.columns({'x':[1.,3.,1.,3.],'y':[3.,3.,1.,1.],'label':labels,'group':c.categorical(['alpha','beta','alpha','beta'])})
    style=c.text_style().fallback(fallback)
    headline=c.math_text('paste(plain(Plotmath),": ",frac(alpha^2,beta))',fonts).style(style)
    ticks=c.text_style().math(fonts).fallback(fallback)
    builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points().text_geom({'math':fonts,'units':'Points','size':16.,'angle':30. if mode==8 else 0.}).text_label('label')).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).ticks([[{'Number':1.},'alpha'],[{'Number':3.},'frac(1,2)']]).text_style(ticks).rich_label(c.math_text('sum(x[i],i==1,n)',fonts).style(style))).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.)).rich_label(c.math_text('sqrt(y)',fonts).style(style))).title(c.title('').rich(headline))
    if mode==5:builder=builder.facet(c.facet_wrap('group').reference({'labeller':{'math':fonts}}))
    elif mode==6:builder=builder.layer(c.points().aes(c.aes().color('group').y(0.5))).legend(c.legend().aesthetic('Color').options({'math':fonts}))
    elif mode==7:builder=builder.layer(c.labels().panel_at(None,0.5,0.5).rich(c.math_text('frac(alpha,beta)',fonts).style(style)))
    p=builder.build()
    wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'math-{mode}.plot.json').write_text(wire);(out/f'math-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'math-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python math controls: nine authors, 27 publications.')
