"""AXIS-01: actual shared guide ownership, named navigation, edits and retained exports."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(600.,400.).dpi(72).basis('current')
def guide(name,source,side,dx,dy):
 return c.axis_guide(name,source).side(side).translate(dx,dy).ticks([(2.,name+' two'),(8.,name+' eight')])
def positions(frame):
 items=frame.scene()['items'];result={}
 for i,item in enumerate(items):
  text=item['primitive'].get('Text')
  if text and text['text'].split(' ')[0] in ['top','lower','far']:
   rule=items[i-1]['primitive']['Rule'];assert rule['from']['x']==rule['to']['x'];result[text['text']]=rule['from']
 return result
def check(values,top=(180.,420.),lower=(190.,430.),far=(200.,440.)):
 for name,expected in [('top',top),('lower',lower),('far',far)]:
  for label,x in zip(['two','eight'],expected):assert abs(values[name+' '+label]['x']-x)<1e-10,(name,label,values,x)
 assert abs(values['far two']['y']-values['lower two']['y']-30.)<1e-10
base=9007199254741001
d=c.Data.columns({'x':[0.,5.,10.],'y':[1.,2.,3.]},keys=[base,base+1,base+2],name='source')
p=(c.plot(d).aes(c.aes().x('x').y('y')).layer(c.points())
 .x_axis(c.x_axis().scale(c.scale_linear().domain(0.,10.)).range(100.,500.).visible(False))
 .y_axis(c.y_axis().range(300.,100.).visible(False))
 .axis(c.x_axis().name('other').side('Top').scale(c.scale_linear().domain(0.,20.)).range(100.,500.).visible(False))
 .guide(guide('top','x','Top',0.,0.)).guide(guide('lower','x','Bottom',10.,30.)).guide(guide('far','x','Bottom',20.,60.)).build())
wire=p.to_json();payload=json.loads(wire);assert payload['version']==8
assert len(payload['definition']['axes'])==3 and len(payload['definition']['guides'])==3
assert len(set(g['scale'] for g in payload['definition']['guides']))==1
roundtrip=c.Plot.from_json(wire);assert roundtrip.to_json()==wire
request=output.request(p,options);frame=request.prepare();check(positions(frame));assert sorted(int(t['Source']['key']) for group in frame.scene()['targets'] for t in group)==[base,base+1,base+2];png=frame.export('png');frame.dispose()
f=output.request(roundtrip,options).prepare();assert f.export('png')==png;f.dispose();roundtrip.dispose()
edited=p.edit().guide(guide('top','other','Top',0.,0.)).build();changed=json.loads(edited.to_json())
assert changed['guides']==payload['guides'];assert changed['axes']==payload['axes']
assert changed['definition']['guides'][0]['scale']==payload['axes']['other']
f=output.request(edited,options).prepare();check(positions(f),top=(140.,260.));f.dispose();edited.dispose()
chart=p.chart();chart.present(output,options).dispose();chart.set_windows(chart.range('x',{'Numeric':[0.,20.]})['windows'])
f=chart.present(output,options);check(positions(f),top=(140.,260.),lower=(150.,270.),far=(160.,280.));f.dispose()
f=request.prepare();assert f.export('png')==png;f.dispose()
try:c.plot(d).layer(c.points()).guide(guide('bad','missing','Top',0.,0.)).build()
except c.ChartError as e:assert e.code=='CHART_MISSING_RESOURCE'
else:raise AssertionError('Missing shared scale accepted')
chart.dispose();p.dispose();d.dispose();request.dispose();output.dispose()
print('PASS Python independent guides: shared positions, two translated guides on one side, named scale navigation, stable guide identities during scale replacement, v8 round trips, exact source IDs and retained publication.')
