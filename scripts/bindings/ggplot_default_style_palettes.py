"""Reference shape/linetype default selection through actual portable hosts."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/default-style-palettes.json').read_text())['cases'];registry=c.ExtensionRegistry.example()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def theme(t):return c.theme().scale_palettes({f"palette.{t['channel']}.discrete":{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':'constant','channel':'size' if t['channel']=='shape' else 'linetype'}}}) if t['theme_mode']=='supplied' else c.theme()
def layer(t):
 p=(c.points() if t['channel']=='shape' else c.rule()).name('marks')
 if t['route']!='automatic':
  scale={'function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'training':'Eligible','ggplot':{'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'Shape':{'solid':t['route']!='hollow'}} if t['channel']=='shape' else 'LineType'}}}
  if t['route']=='constructor':scale['palette_theme_aesthetics']=[t['channel']]
  p=p.value_scale('Shape' if t['channel']=='shape' else 'LineType','v',scale)
 return p
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'end':c.column([float(i)+.5 for i in range(len(t['inputs']))],kind='float64'),'v':c.column(t['inputs'],kind='string').nullable(True)},name='data')
def build(t,d):
 m=c.aes().x('x').y(1.);m=m.shape('v') if t['channel']=='shape' else m.x2('end').y2(1.).linetype('v')
 return c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(m).layer(layer(t)).theme(theme(t)).build()
def check(t,chart):
 actual=chart.semantics()['layers'][0];rows=actual.get('aesthetics',[]);wanted=[v for v in t['result']['mapped'] if v is not None or t['channel']=='linetype'];key='Shape' if t['channel']=='shape' else 'LineType';assert len(rows)==len(wanted),(t,rows)
 for row,v in zip(rows,wanted):assert row[key]==({'kind':'Missing'} if v is None else {'kind':'Number' if key=='Shape' else 'Text','value':v}),(t,row,v)
 return {'aesthetics':rows,'styles':actual.get('styles',[])}
def matching(x,t):return all(x[k]==t[k] for k in ('channel','route','theme_mode'))
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   expected=t
   if state=='original':current=restored
   elif state=='layer_edit':current=restored.edit().layer('marks',layer(t)).build();owned.append(current)
   else:
    expected=next(x for x in cases if x['channel']==t['channel'] and x['route']==t['route'] and x['population']==t['population'] and x['theme_mode']!=t['theme_mode']);current=restored.edit().theme(theme(expected)).build();owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,**check(expected,chart)});assert restored.to_json()==wire
  if t['population']=='ordinary':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame);scene=frame.scene()
   for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-{t['route']}-{t['theme_mode']}.{fmt}").write_bytes(frame.export(fmt))
   chart=restored.chart();owned.append(chart)
   for population in ('missing','empty','ordinary'):
    expected=next(x for x in cases if matching(x,t) and x['population']==population);replacement=data_for(expected);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(d,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    batch_plot=build(expected,replacement);owned.append(batch_plot);batch=batch_plot.chart();owned.append(batch);actual=check(expected,chart);assert actual==check(expected,batch);assert restored.to_json()==wire and frame.scene()==scene;records.append({'index':index,'state':'replacement','population':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==144,len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS 144 style palette states and 36 publication files')
