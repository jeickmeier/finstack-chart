"""CLR-04 actual Python paint authors, immutable snapshots and all input routes."""
from pathlib import Path
import json,sys,math
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
background=c.hsl(210.,.1,.97)
options=c.export_options(440.,340.).dpi(96).background(background).basis('current')
def ink():return c.lab(28.,7.,-18.)
def heading(text):return c.title(text).style(c.text_style().color(ink()).size(1.15))
def data():return c.Data.columns({'x':[1.,2.,3.],'y':[1.,2.,1.5]})
figures=[]
swatches=(c.plot(c.Data.columns({'one':[1.]})).title(heading('Color spaces at the paint boundary'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(0.,6.)).visible(False))
 .y_axis(c.y_axis().scale(c.scale_linear().domain(0.,2.)).visible(False))
 .theme(c.theme().preset('Editorial').style(c.style().gradient({'direction':'Horizontal','start':c.hsl(40.,.4,.98),'end':c.hsl(210.,.3,.94)}))))
for i,(name,value) in enumerate([('RGB',c.rgb(230.25,55.75,80.5)),('HSL',c.hsl(155.,.75,.38)),('Lab',c.lab(65.,45.,50.)),('HCL',c.hcl(270.,70.,60.)),('Cubehelix',c.cubehelix(210.,1.3,.55))]):
 x=float(i+1)
 swatches=swatches.layer(c.points().aes(c.aes().x(x).y(1.15)).color(value).size(19.)).layer(c.labels().id(name).at(x,.65).text(name).style(c.text_style().color(value)))
 value.dispose()
figures.append(('spaces',swatches.build()))
ramp=c.Data.columns({'x':[i/10. for i in range(11)],'y':[1.]*11})
figures.append(('continuous',c.plot(ramp).aes(c.aes().x('x').y('y'))
 .layer(c.points().aes(c.aes().color('x').color_scale('ramp')).size(12.))
 .scale(c.color_continuous('ramp',0.,1.).palette([c.lab(65.,70.,65.),c.hcl(240.,70.,65.)]).missing(c.hsl(0.,0.,.5)))
 .legend(c.legend().scale('ramp').title('Floating RGB')).title(heading('One ramp for marks and guide stops'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(-.1,1.1)))
 .y_axis(c.y_axis().scale(c.scale_linear().domain(0.,2.)).visible(False)).theme(c.theme().preset('Editorial')).build()))
candles=c.Data.columns({'x':[1.,2.,3.,4.],'open':[2.,4.,3.,5.],'close':[4.,3.,5.,4.],'low':[1.,2.,2.,3.],'high':[5.,5.,6.,6.]})
figures.append(('candles',c.plot(candles).aes(c.aes().x('x').y('open').y2('close').low('low').high('high'))
 .layer(c.ohlc().width(12.).color(c.rgb(40.5,45.5,50.5)).candle_colors({'up':c.hcl(150.,55.,55.),'down':c.hcl(25.,65.,55.)}))
 .title(heading('Directional colors retain their space'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(.5,4.5))).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,7.))).theme(c.theme().preset('Editorial').geometry(ink=ink(),paper=c.gray(98.),accent=c.hcl(270.,50.,55.),point_size=1.5,line_width=.5)).build()))
def rich(text):return c.rich_text(text).style(c.text_style().color(ink()))
path=c.path().move_to(0.,0.).line_to(30.,0.).line_to(15.,25.).close_path()
figures.append(('furniture',c.plot(data()).aes(c.aes().x('x').y('y'))
 .layer(c.points().size(8.).style(c.style().mark(c.hsl(210.,.7,.5))))
 .layer(c.vector_path('triangle',path).fill(c.lab(65.,50.,45.,.55)).stroke({'color':c.hcl(20.,55.,45.),'width':2.}).anchor({'Output':{'x':310.,'y':155.}}))
 .layer(c.labels().id('label').at(1.4,1.5).text('Retained annotation').style(c.text_style().color(c.hcl(300.,40.,40.))))
 .title(c.title('Rich text and authored paint').rich(rich('Rich text and authored paint')))
 .subtitle(c.subtitle('Shared color descriptors').style(c.text_style().color(c.hsl(210.,.5,.35))))
 .caption(c.caption('Caption / source / footnote').style(c.text_style().color(c.lab(40.,20.,-20.))))
 .source_note(c.source_note('Source: supplied values').style(c.text_style().color(c.gray(40.))))
 .footnote(c.footnote('Final scene uses sRGB8').style(c.text_style().color(c.hcl(100.,20.,35.))))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(.5,3.5)).rich_label(rich('Authored x'))).y_axis(c.y_axis().scale(c.scale_linear().domain(.5,2.5)).rich_label(rich('Authored y')))
 .theme(c.theme().preset('Editorial').style(c.style().foreground(ink()).annotation(ink()).focus(c.hcl(90.,60.,70.)).selection(c.hcl(300.,40.,60.)).grid(c.gray(90.)).panel(c.gray(99.)))).build()))
records=[]
for name,plot in figures:
 wire=plot.to_json();assert json.loads(wire)['version']==4
 copied=c.Plot.from_json(wire);assert copied.to_json()==wire;copied.dispose()
 request=output.request(plot,options);frame=request.prepare();scene=frame.scene()
 (out/f'{name}.plot.json').write_text(wire);(out/f'{name}.scene.json').write_text(json.dumps(scene,indent=2))
 for fmt in ('svg','pdf','png'):(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
 plot.dispose();repeated=request.prepare();assert repeated.scene()==scene;repeated.dispose()
 records.append({'id':name,'version':4,'retained_after_disposal':True})
 frame.dispose();request.dispose()
# Every direct and nested style input accepts owned values, CSS and explicit descriptors.
value=c.hcl(-0.,-0.,50.,.4);descriptor=json.loads(value.to_json())
assert descriptor['value']['channels']['h']=={'number':'-0'}
style=c.style()
for field in ('background','panel','foreground','grid','mark','annotation','focus','selection'):style=getattr(style,field)(value)
plot=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points().color('rebeccapurple')).theme(c.theme().style(style)).build()
assert json.loads(plot.to_json())['definition']['theme']['plot']['mark']==descriptor
# A wide finite channel must remain binary64 in nested authored values.
wide=c.rgb(1e100,0.,0.)
wide_plot=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points().color(wide)).build()
assert json.loads(wide_plot.to_json())['definition']['layers'][0]['style']['color']['value']['channels']['r']==1e100
# Exercise host/output and interaction tokens through retained layout options.
layout=c.layout_options().host_style(c.style().background(value)).output_style(c.style().panel(value))
request=output.request(plot,c.export_options(440.,340.).layout(layout).background(value));frame=request.prepare();assert frame.scene()['items']
# A floating discrete palette must retain descriptors and lower both categories and missing paints.
d=c.Data.columns({'x':[1.,2.,3.],'y':[1.,2.,3.],'group':c.categorical(['A','B','C'])})
p=c.plot(d).aes(c.aes().x('x').y('y').color('group').color_scale('d')).layer(c.points()).scale(c.color_discrete('d').palette([c.lab(60.,40.,30.),value]).domain(['A','B']).missing(c.rgb(1.2,2.3,3.4))).build()
f=output.request(p,options).prepare();assert len([i for i in f.scene()['items'] if 'Point' in i['primitive'] and i.get('layer') is not None])==3
for bad in [{'version':2,'value':descriptor['value']},{'red':1.5,'green':0,'blue':0,'alpha':255},'currentColor']:
 try:c.points().color(bad)
 except (c.ChartError,ValueError):pass
 else:raise AssertionError('Malformed paint accepted')
value.dispose()
try:c.points().color(value)
except c.ChartError:pass
else:raise AssertionError('Disposed color accepted')
# A paint-only edit keeps the previously presented snapshot and retained current request.
live_plot=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points()).theme(c.theme().style(c.style().mark('red'))).build()
live=live_plot.chart();live_frame=live.present(output,options);before=live.revisions()
old=live.request(output,options.basis('presented'))
updated=live_plot.edit().theme(c.theme().style(c.style().mark(c.hsl(120.,1.,.5)))).build()
live.apply_plot(updated,before['definition']);after=live.revisions()
assert after['store']==before['store'] and after['definition']==before['definition']+1
current=live.request(output,options.basis('current'));still_old=live.request(output,options.basis('presented'))
fresh=output.request(updated,options)
live.dispose();live_plot.dispose();updated.dispose();live_frame.dispose()
def paints(request):
 frame=request.prepare()
 result=[i['primitive']['Point']['fill'] for i in frame.scene()['items'] if 'Point' in i['primitive'] and i.get('layer') is not None]
 frame.dispose();return result
assert paints(old)==paints(still_old)==[{'red':255,'green':0,'blue':0,'alpha':255}]*3
assert paints(current)==paints(fresh)==[{'red':0,'green':255,'blue':0,'alpha':255}]*3
records.append({'id':'live-color-edit','retained_presented_and_current':True,'data_revision_preserved':True})
(out/'checks.json').write_text(json.dumps(records,indent=2)+'\n')
print('PASS CLR-04 Python: four independently authored publication figures, every paint input, strict descriptors and retained ownership.')
