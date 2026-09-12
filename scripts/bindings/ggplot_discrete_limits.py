"""FIX-GG04 registered discrete limit functions through actual primary hosts."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def descriptor(case):
 s={'training':'Eligible','limits_function':{'operation':{'id':'example.discrete_limits','version':'1'},'parameters':{'mode':case['control']}}}
 if case['kind']=='identity':s['function']={'GgplotDiscreteIdentity':{'limits':None,'levels':None,'drop':True,'na_translate':True,'guide':case['guide'],'observed':[]}}
 else:s.update({'function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'ggplot':{'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'Shape':{'solid':True}}}}})
 return s
cases=[x for x in json.loads((ROOT/'fixtures/parity/ggplot2/limit-functions.json').read_text())['cases'] if 'transform' not in x]
for index,case in enumerate(cases):
 owned=[];failure=case['result'].get('error');identity=case['kind']=='identity';channel='Label' if identity else 'Shape'
 try:
  inputs=case['inputs'];data=c.Data.columns({'x':c.column(list(map(float,range(len(inputs)))),kind='float64'),'v':c.column(inputs,kind='string').nullable(True)});owned.append(data)
  def layer():return c.points().name('marks').value_scale(channel,'v',descriptor(case))
  p=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).build();owned.append(p)
  wire=p.to_json();assert json.loads(wire)['version']==28
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=chart.semantics()['layers'][0];assert not failure,(index,failure)
   wanted=[v for v in case['result']['values'] if identity or v is not None]
   aesthetics=actual.get('aesthetics',[]);assert len(aesthetics)==len(wanted),(index,aesthetics,wanted)
   for row,want in zip(aesthetics,wanted):assert row.get(channel)==({'kind':'Missing'} if want is None else {'kind':'Text' if identity else 'Number','value':want}),(index,row,want)
   records.append({'index':index,'state':state,'aesthetics':aesthetics})
   if case['guide'] and case['population']=='spaced' and case['control'] in ('reverse','fixed') and state=='original':
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/f"{case['kind']}-{case['control']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  diagnostic=json.loads(str(error));assert failure,(index,diagnostic);assert diagnostic['code']=='CHART_VALIDATION',(index,diagnostic);records.append({'index':index,'error':diagnostic['code']})
 finally:
  for obj in reversed(owned):obj.dispose()
for kind in ('discrete','identity'):
 for control in ('append','fixed'):
  owned=[]
  try:
   selected={t['population']:t for t in cases if t['guide'] and t['kind']==kind and t['control']==control}
   identity=kind=='identity';channel='Label' if identity else 'Shape'
   def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='string').nullable(True)})
   def build(d):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(c.points().name('marks').value_scale(channel,'v',descriptor(selected['spaced']))).build()
   original=data_for(selected['spaced']);owned.append(original);p=build(original);owned.append(p);wire=p.to_json();chart=p.chart();owned.append(chart)
   request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('constant','missing','all_missing','empty','spaced'):
    t=selected[population];replacement=data_for(t);owned.append(replacement)
    tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    actual=chart.semantics()['layers'][0].get('aesthetics',[]);assert actual==batch.semantics()['layers'][0].get('aesthetics',[])
    expected=[v for v in t['result']['values'] if identity or v is not None];assert len(actual)==len(expected)
    for row,want in zip(actual,expected):assert row[channel]==({'kind':'Missing'} if want is None else {'kind':'Text' if identity else 'Number','value':want})
    assert held.scene()==scene and p.to_json()==wire
    records.append({'kind':kind,'control':control,'replacement':population,'aesthetics':actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==198,len(records)
(out/'discrete-limit-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose();print('PASS Python:',len(records),'registered discrete limit states.')
