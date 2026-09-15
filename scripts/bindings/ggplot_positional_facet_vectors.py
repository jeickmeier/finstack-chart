"""FIX-GG04: actual faceted positional vector mappings, guides and retained host edits."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
shared="--shared" in sys.argv[3:]
binned="--binned" in sys.argv[3:]
broadcast="--broadcast" in sys.argv[3:]
cases=[]
for filename in (('positional-pipeline-binned-broadcast-facets' if binned else 'positional-pipeline-broadcast-facets',) if broadcast else ('positional-pipeline-binned-facets',) if binned else ('positional-pipeline-facets','positional-pipeline-facet-arities')):
 cases.extend(json.loads((ROOT/f'fixtures/parity/ggplot2/{filename}.json').read_text())['cases'])
if shared:cases=[t for t in cases if t["kind"]=="mean"]
def data_for(t):
 return c.Data.columns({'v':c.column([float(v) if v in ('Infinity','-Infinity') else v for v in t['inputs']],kind='float64').nullable(True),'i':c.column(list(range(1,len(t['inputs'])+1)),kind='int64'),'g':c.column(t['groups'],kind='int64'),'f':c.column(t['facets'],kind='string')})
def layer(t):
 p=c.points().name('marks')
 if broadcast:p=p.facet_target('Broadcast')
 if shared:return p.from_transform('mean').after_stat(c.stat_aes().x(1.).y('Mean'))
 return p.stat(c.summary().x('v').group('g')).after_stat(c.stat_aes().x(1.).y('Mean')) if t['kind']=='mean' else p
def build(t,d):
 scale={'identity':c.scale_linear,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 if binned:scale=c.scale_binned({'bins':{'breaks':{'Nice':3.},'oob':'Squish','right':True},'transform':{'reverse':'Reverse','log10':{'Log':{'base':10.}}}.get(t['transform'])})
 axis=c.y_axis().scale(scale).guide_geometry({'labels':'Preserve'}).oob_function({'operation':{'id':'example.scale_vector','version':'1'},'parameters':{'mode':'default' if t['mode']=='default' else 'oob_'+t['mode']}})
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').layer(layer(t)).aes(c.aes().x('i').y('v')).y_axis(axis).facet(c.facet_wrap('f').columns(3).order([['A'],['B']] if t.get('panels')=='occupied' else [['A'],['B'],['C']]).free_y(t['scales']=='free_y'))
 if shared:
  target='Broadcast' if broadcast else 'Match'
  draft=draft.transform(c.transform('mean',c.summary().x('v').group('g')).facet_target(target)).transform(c.transform('copy',c.identity_stat()).from_transform('mean').facet_target(target)).layer(c.points().name('copy').from_transform('copy').facet_target(target).after_stat(c.stat_aes().x(2.).y('Mean')))

 return draft.build()
def scalar(v):return v.get('number') if isinstance(v,dict) else v
def compare(a,b):
 if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert math.isclose(a,b,rel_tol=0,abs_tol=5e-14),(a,b)
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b),(a,b)
  for x,y in zip(a,b):compare(x,y)
 else:assert a==b,(a,b)
def check(t,chart,frame):
 panels=chart.semantics()['panels'];guides=frame.guides()['guides'];result=[]
 assert len(panels)==len(t['result']['panels'])==(2 if t.get('panels')=='occupied' else 3)
 for panel,wanted in zip(panels,t['result']['panels']):
  state=panel['layers'][0];positions=state['point_positions']
  compare([scalar(p[1]) for p in positions],[v for v in wanted['mapped'] if v is not None])
  if t['kind']=='point':compare([scalar(p[0]) for p in positions],[x for x,v in zip(wanted['x'],wanted['mapped']) if v is not None])
  ticks=next(g for g in guides if g['scope']==[{'Panel':panel['key']}] and g['spec']['side']=='Left')['ticks']
  expected=[(v,label) for v,label in zip(wanted['breaks'],wanted['labels']) if isinstance(v,(int,float))]
  compare([tick['label'] for tick in ticks],[label or '' for _,label in expected])
  values=[tick['value']['Number'] for tick in ticks]
  compare([(-v if t['transform']=='reverse' else math.log10(v) if t['transform']=='log10' else v) for v in values],[v for v,_ in expected])
  entry={'key':panel['key'],'positions':positions,'ticks':ticks,'domains':state['domains'],'limits':panel.get('positional_limits',{})}
  if shared:
   assert len(panel['layers'])==2
   for i,other in enumerate(panel['layers']):
    compare([scalar(p[1]) for p in other['point_positions']],[v for v in wanted['mapped'] if v is not None])
    compare([scalar(p[0]) for p in other['point_positions']],[i+1]*len(positions))
   entry['layers']=[{'positions':other['point_positions'],'domains':other['domains'],'operations':[{'operation':op['operation'],'counts':op['counts']} for op in other['operations']]} for other in panel['layers']]
  result.append(entry)
 return {'panels':result}
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
   sample=t['population']=='ordinary' and t['mode']=='index' and (not binned or t['panels']=='occupied')
   if state=='original' and sample:
    for fmt in ('svg','pdf','png'):(out/f"{t['kind']}-{t['scales']}-{t['transform']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(index,t,str(error));assert error.code in ('CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN'),(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(((360 if shared else 744) if binned else (184 if shared else 376)) if broadcast else ((342 if shared else 696) if binned else (182 if shared else 417))),len(records)
for kind in (('mean',) if shared else ('point','mean')):
 for scales in ('fixed','free_y'):
  selected={t['population']:t for t in cases if t['kind']==kind and t['scales']==scales and t['transform']=='identity' and t['mode']=='index' and (not binned or t['panels']=='occupied')};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','empty'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    ar=output.request(chart,options);owned.append(ar);af=ar.prepare();owned.append(af);br=output.request(batch,options);owned.append(br);bf=br.prepare();owned.append(bf);actual=check(t,chart,af);assert actual==check(t,batch,bf);assert p.to_json()==wire and held.scene()==scene
    records.append({'kind':kind,'scales':scales,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(((364 if shared else 752) if binned else (188 if shared else 384)) if broadcast else ((346 if shared else 704) if binned else (186 if shared else 425))),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));output.dispose();options.dispose();registry.dispose();print('PASS',len(records),'positional facet vector host states')
