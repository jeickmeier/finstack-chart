"""AXIS-06 independent Python authoring, compiled samples and retained ownership."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
def axis(domain,values,labels):
 return c.x_axis().scale(c.scale_linear().domain(0.,domain)).range(50.,450.).guide_profile('D3_3_0_0').tick_values(values).tick_format({'Labels':labels})
data=c.Data.columns({'x':[0.,.5,1.],'y':[0.,1.,0.]})
a=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis(1.,[0.,.5,1.],['zero','half','one'])).y_axis(c.y_axis().visible(False)).build()
b=a.edit().x_axis(axis(2.,[0.,1.,2.],['ZERO','ONE','TWO'])).build()
cplot=b.edit().x_axis(axis(4.,[0.,2.,4.],['zero again','two again','four'])).build()
plots=[a,b,cplot]
for i,p in enumerate(plots):(out/f'plot-{i}.json').write_text(p.to_json())
for mode in ['text','outline']:
 figures=[output.request(p,c.export_options(900.,300.).text('preserve' if mode=='text' else 'outline').dpi(144)).prepare() for p in plots]
 plan=figures[1].guide_transition(figures[0]);mid=plan.sample(.5);interrupt=figures[2].guide_transition(mid)
 for name,frame in [('start',plan.sample(0.)),('mid',mid),('end',plan.sample(1.)),('interrupt-start',interrupt.sample(0.)),('interrupt-mid',interrupt.sample(.5)),('interrupt-end',interrupt.sample(1.))]:
  prefix=f'{mode}-{name}'
  for ext,value in [('scene',frame.scene()),('guides',frame.guides()),('presentation',frame.presentation())]:(out/f'{prefix}.{ext}.json').write_text(json.dumps(value))
  for fmt in ['svg','pdf','png']:(out/f'{prefix}.{fmt}').write_bytes(frame.export(fmt))
  frame.dispose()
 assert plan.sample(1.).scene()==figures[1].scene()
 for f in figures:f.dispose()
 # The compiled plan owns its old/new snapshots after input handles are disposed.
 assert len(plan.sample(.5).presentation()[0]['frame']['ticks'])==4
 for fraction in [float('nan'),float('inf'),-.1,1.1]:
  try:plan.sample(fraction)
  except c.ChartError:pass
  else:raise AssertionError('invalid fraction accepted')
 plan.dispose()
 try:plan.sample(.5)
 except c.ChartError:pass
 else:raise AssertionError('disposed plan accepted')
 interrupt.dispose()
chart=a.chart();opts=c.export_options(900.,300.).dpi(144)
initial=chart.present(output,opts)
chart.apply_plot(b,int(chart.revisions()['definition']))
target=chart.present(output,opts);plan=target.guide_transition(initial);mid=plan.sample(.5)
chart.acknowledge_frame(mid)
frozen=chart.request(output,opts.basis('displayed')).prepare()
assert frozen.scene()==mid.scene() and frozen.guides()==mid.guides()
try:chart.acknowledge_frame(initial)
except c.ChartError:pass
else:raise AssertionError('stale frame accepted')
chart.dispose()
try:chart.acknowledge_frame(mid)
except c.ChartError:pass
else:raise AssertionError('disposed runtime accepted')
output.dispose();assert len(plan.sample(.5).presentation()[0]['frame']['ticks'])==4
print('PASS Python AX05: sampled publications, interruption, final/reduced motion, capture, stale/disposed handles and retained resources')
