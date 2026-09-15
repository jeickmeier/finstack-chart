"""FIX-GG04: binned shape default/NULL/theme selection through the shared engine."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/binned-style-defaults.json').read_text())['cases'];registry=c.ExtensionRegistry.example()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def operation(mode):return {'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':mode,'channel':'size'}}
def layer(t):
 policy={'palette':{'Shape':{'solid':t['route']!='hollow'}}} if t['route']!='null' else {}
 policy.update(oob='Squish',breaks={'Nice':5.},right=True,limits=None)
 scale={'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0.,1.],'reverse':False,'rescaler':'Range'}},'output':'Identity','unknown':{'kind':'Missing'}}},'training':'Eligible','ggplot':{'Binned':policy},'guide':{'BinnedBins':'Automatic'}}
 if t['route']=='null':scale.update(palette_function=operation('reject'),palette_theme_aesthetics=['shape'])
 return c.points().name('marks').value_scale('Shape','v',scale)
def theme(t):return c.theme() if t['theme_mode']=='absent' else c.theme().scale_palettes({'palette.shape.continuous':operation('constant' if t['theme_mode']=='vector' else 'repeat_count')})
def check(t,chart):
 data=chart.semantics()['layers'][0];wanted=[v for v in t['result']['mapped'] if v is not None];actual=data.get('aesthetics',[])
 assert len(actual)==len(wanted),(t,actual)
 for row,v in zip(actual,wanted):assert row['Shape']=={'kind':'Number','value':v},(t,row,v)
 keys=[v for g in data.get('numeric_value_guides',{}).values() for v in g if v['visible']];expected=t['result']['guides']
 assert len(keys)==(len(expected[0]['values']) if expected else 0),(t,keys)
 for i,key in enumerate(keys):
  assert key.get('label')==expected[0]['labels'][i],(t,key)
  v=expected[0]['mapped'][i];assert key.get('mapped')==({'kind':'Missing'} if v is None else {'kind':'Number','value':v}),(t,key,v)
 return {'aesthetics':actual,'keys':keys}
for index,t in enumerate(cases):
 owned=[]
 try:
  try:
   d=c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='float64').nullable(True)},name='data');owned.append(d)
   p=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x('x').y(1.)).layer(layer(t)).theme(theme(t)).build();owned.append(p)
   chart=p.chart();owned.append(chart);chart.semantics()
  except c.ChartError as error:
   assert 'error' in t['result'],(t,error);records.append({'index':index,'rejected':error.code});continue
  assert 'error' not in t['result'],t
  wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build() if state=='layer_edit' else restored.edit().theme(theme(t)).build()
   if state!='original':owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,**check(t,chart)});assert restored.to_json()==wire
  if t['population']=='ordinary':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['route']}-{t['theme_mode']}.{fmt}").write_bytes(frame.export(fmt))
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==134,len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS 134 binned shape default states and 30 publication files')
