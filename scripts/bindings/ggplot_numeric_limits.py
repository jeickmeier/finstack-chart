"""FIX-GG04 numeric limit callbacks through the actual Python primary adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def descriptor(t):
 s={'training':'Eligible','limits_function':{'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':t['control']}}}
 if t['kind']=='identity':s['function']={'GgplotNumericIdentity':{'transform':{'sqrt':'Sqrt','reverse':'Reverse'}.get(t['transform']),'limits':None,'guide':True,'trained':None}}
 else:
  s['function']={'Interpolated':{'normalization':{'Ggplot':{'family':{'Pow':{'exponent':0.5}} if t['transform']=='sqrt' else 'Linear','domain':[0,1],'reverse':t['transform']=='reverse','rescaler':'Maximum' if t.get('rescaler')=='maximum' else {'Midpoint':2} if t.get('rescaler')=='midpoint' else 'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[1,6],'exponent':0.5,'absolute':False}},'unknown':{'kind':'Missing'}}}
  if 'rescaler' in t:s['function']['Interpolated']['output']='Identity'
  s['ggplot']={'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':5},'right':True}} if t['kind']=='binned' else {'Continuous':{'limits':None,'oob':'Censor'}}
 if 'units' in t:
  s['function']['Interpolated']['normalization']['Ggplot']['timestamp']={'origin':str(1704067200*t['units']),'unit':t['variant'],'date':t['kind']=='date'}
  s['guide']={'Temporal':{'origin':str(1704067200*t['units']),'unit':t['variant'],'zone':'Utc','arguments':{'date':t['kind']=='date'}}}
 return s
def data_for(t):
 if 'units' in t:
  factor=t['units']*(86400 if t['kind']=='date' else 1)
  def stamp(v):
   if v is None:return 1704067200*t['units']
   whole=math.trunc(v);return whole*factor+round((v-whole)*factor)
  return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.timestamps([stamp(v) for v in t['inputs']],t['unit'],'UTC').validity([v is not None for v in t['inputs']])})
 return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column([v or 0 for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])})
def layer(t):
 p=c.points().name('marks')
 return p.numeric_scale('Alpha','v',descriptor(t)).after_scale(c.scale_aes().size(c.after_scale_expr('Alpha').coalesce(0.).mul(.1).add(2.))) if t['kind']=='identity' else p.numeric_scale('Size',{'field':'v','origin':str(1704067200*t['units'])} if 'units' in t else 'v',descriptor(t))
def build(t,d):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def check(t,actual):
 wanted=t['result']['values']
 if len(wanted)==1:wanted=wanted*len(t['inputs'])
 if t['kind']=='identity':
  rows=actual.get('styles',[]);assert len(rows)==len(wanted),(t,rows)
  for row,want in zip(rows,wanted):
   expected_size=2.+.1*(want if want is not None else 0.)
   assert abs(row['radius']-expected_size)<3e-12,(t,row,want)
   assert row['color']['alpha']==(255 if want is None else math.floor(min(1.,max(0.,want))*255+.5)),(t,row,want)
 else:
  wanted=[v for v in wanted if v is not None];rows=actual.get('styles',[]);assert len(rows)==len(wanted),(t,rows,wanted)
  for row,want in zip(rows,wanted):assert abs(row['radius']-want)<3e-12*max(abs(want),1),(t,row,want)
 return {'styles':actual.get('styles',[]),'aesthetics':actual.get('aesthetics',[])}
cases=[x for x in json.loads((ROOT/'fixtures/parity/ggplot2/limit-functions.json').read_text())['cases'] if 'transform' in x]
cases += [dict(t,transform='identity') for t in json.loads((ROOT/'fixtures/parity/ggplot2/limit-rescalers.json').read_text())['cases']]
cases += [dict(t,transform='identity',unit=unit,variant=variant,units=units) for t in json.loads((ROOT/'fixtures/parity/ggplot2/temporal-limit-functions.json').read_text())['cases'] for unit,variant,units in [('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)] if not (t['kind']=='datetime' and t['population']=='fractional' and units==1)]
for index,t in enumerate(cases):
 owned=[];failure=t['result'].get('error') or (t['result'].get('guide',{}).get('error') if 'units' in t else None)
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==28
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=chart.semantics()['layers'][0];assert not failure,(index,failure)
   records.append({'index':index,'state':state,**check(t,actual)})
   if ('units' not in t or t['units']==1000000) and 'rescaler' not in t and t['kind']!='identity' and t['population']=='spaced' and t['transform']=='identity' and t['control'] in ('reverse','fixed','single') and state=='original':
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/f"{t['kind']}-{t['control']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  diagnostic=json.loads(str(error));assert failure,(index,diagnostic);assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(index,diagnostic)
  records.append({'index':index,'error':diagnostic['code']})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==1362,len(records)
for kind in ('continuous','binned','identity','date','datetime'):
 for control in ('fixed','single'):
  owned=[]
  try:
   selected={t['population']:t for t in cases if t['kind']==kind and t['control']==control and t['transform']=='identity' and 'rescaler' not in t and ('units' not in t or t['units']==1000000)}
   original=data_for(selected['spaced']);owned.append(original);p=build(selected['spaced'],original);owned.append(p);wire=p.to_json();chart=p.chart();owned.append(chart)
   request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in (('constant','missing','all_missing','spaced') if kind in ('date','datetime') else ('constant','missing','all_missing','empty','spaced')):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    actual=check(t,chart.semantics()['layers'][0]);assert actual==check(t,batch.semantics()['layers'][0]);assert held.scene()==scene and p.to_json()==wire
    records.append({'kind':kind,'control':control,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==1408,len(records)
(out/'numeric-limit-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose();print('PASS Python:',len(records),'numeric limit states.')
