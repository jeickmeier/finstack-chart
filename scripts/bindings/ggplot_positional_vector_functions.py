"""FIX-GG04: actual positional vector mappings, guides and retained host edits."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
shared="--shared" in sys.argv[3:]
cases=[]
for filename,summary in [('positional-pipeline-functions',False),('positional-pipeline-domains',False),('positional-pipeline-statistics',True)]:
 for t in json.loads((ROOT/f'fixtures/parity/ggplot2/{filename}.json').read_text())['cases']:
  cases.append(dict(t,summary=summary,axis=t.get('axis','y'),limit_mode=t.get('limit_mode','full')))
if shared:cases=[t for t in cases if t["summary"]]
def data_for(t):
 return c.Data.columns({'v':c.column([float(v) if v in ('Infinity','-Infinity') else v for v in t['inputs']],kind='float64').nullable(True),'i':c.column(t.get('groups',list(range(1,len(t['inputs'])+1))),kind='int64')})
def layer(t):
 p=c.points().name('marks')
 if shared:return p.from_transform('mean').after_stat(c.stat_aes().x(1.).y('Mean'))
 return p.stat(c.summary().x('v').group('i')).after_stat(c.stat_aes().x(1.).y('Mean')) if t['summary'] else p
def build(t,d):
 scale={'identity':c.scale_linear,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 if t['limit_mode']!='automatic':scale=scale.domain(4.,4.) if t['limit_mode']=='constant' else scale.domain(1.,10.)
 axis=(c.x_axis() if t['axis']=='x' else c.y_axis()).scale(scale).guide_geometry({'labels':'Preserve'}).oob_function({'operation':{'id':t.get('operation','example.scale_vector'),'version':'1'},'parameters':{'mode':'default' if t['mode']=='default' else 'oob_'+t['mode']}})
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').layer(layer(t)).aes(c.aes().x('v' if t['axis']=='x' else 'i').y('i' if t['axis']=='x' else 'v'))
 if shared:draft=draft.transform(c.transform('mean',c.summary().x('v').group('i'))).transform(c.transform('copy',c.identity_stat()).from_transform('mean')).layer(c.points().name('copy').from_transform('copy').after_stat(c.stat_aes().x(2.).y('Mean')))
 return (draft.x_axis(axis) if t['axis']=='x' else draft.y_axis(axis)).build()
def scalar(v):return v.get('number') if isinstance(v,dict) else v
def compare(a,b):
 if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert math.isclose(a,b,rel_tol=0,abs_tol=5e-14),(a,b)
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b),(a,b)
  for x,y in zip(a,b):compare(x,y)
 else:assert a==b,(a,b)
def check(t,chart,frame):
 state=chart.semantics()['layers'][0];positions=state['point_positions'];dim=0 if t['axis']=='x' else 1
 compare([scalar(p[dim]) for p in positions],[v for v in t['result']['mapped'] if v is not None])
 if not t['summary']:compare([scalar(p[1-dim]) for p in positions],[i+1 for i,v in enumerate(t['result']['mapped']) if v is not None])
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Bottom' if dim==0 else 'Left'))['ticks']
 expected=[(v,label) for v,label in zip(t['result']['breaks'],t['result']['labels']) if isinstance(v,(int,float))]
 compare([tick['label'] for tick in ticks],[label or '' for _,label in expected])
 values=[tick['value']['Number'] for tick in ticks]
 compare([(-v if t['transform']=='reverse' else math.log10(v) if t['transform']=='log10' else v) for v in values],[v for v,_ in expected])
 result={'positions':positions,'ticks':ticks,'domains':state['domains']}
 if shared:
  layers=chart.semantics()['layers'];assert len(layers)==2
  for i,other in enumerate(layers):
   compare([scalar(p[1]) for p in other['point_positions']],[v for v in t['result']['mapped'] if v is not None])
   compare([scalar(p[0]) for p in other['point_positions']],[i+1]*len(positions))
  result['layers']=[{'positions':other['point_positions'],'domains':other['domains'],'operations':[{'operation':op['operation'],'counts':op['counts']} for op in other['operations']]} for other in layers]
 return result
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==40
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**check(t,chart,frame)})
   sample=t['population']=='ordinary' and ((not t['summary'] and t['axis']=='x' and ((t['limit_mode'] in ('automatic','full') and t['mode']=='index') or (t['limit_mode']=='constant' and t['mode']=='default'))) or (t['summary'] and (shared or t['transform']=='identity') and t['mode'] in ('default','reverse','index')))
   if state=='original' and sample:
    for fmt in ('svg','pdf','png'):(out/f"{'summary' if t['summary'] else 'point'}-{t['transform']}-{t['limit_mode']}-{t['mode']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(index,t,str(error));assert error.code=='CHART_SCHEMA_CONFLICT',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(96 if shared else 720),len(records)
for axis in (('y',) if shared else ('x','y')):
 for limit_mode in (('full',) if shared else ('automatic','full')):
  selected={t['population']:t for t in cases if t['summary']==shared and t['axis']==axis and t['limit_mode']==limit_mode and t['transform']=='identity' and t['mode']=='index'};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','empty'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    ar=output.request(chart,options);owned.append(ar);af=ar.prepare();owned.append(af);br=output.request(batch,options);owned.append(br);bf=br.prepare();owned.append(bf);actual=check(t,chart,af);assert actual==check(t,batch,bf);assert p.to_json()==wire and held.scene()==scene
    records.append({'axis':axis,'limit_mode':limit_mode,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
for variant in ('missing_registration','native_only','invalid_parameters','downgrade'):
 t=copy.deepcopy(cases[0]);owned=[]
 if variant=='missing_registration':t['operation']='example.missing_vector'
 if variant=='native_only':t['operation']='example.native_scale_vector'
 if variant=='invalid_parameters':t['mode']='invalid'
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  if variant=='downgrade':wire=json.loads(wire);wire['version']=39;restored=c.Plot.from_json(json.dumps(wire),registry);owned.append(restored)
  raise AssertionError('expected rejection: '+variant)
 except c.ChartError as error:records.append({'rejection':variant,'code':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(102 if shared else 732),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));output.dispose();options.dispose();registry.dispose();print('PASS',len(records),'positional vector host states')
