"""GG2-03: actual blank training layers, host round-trips and immutable publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
cases=json.loads((ROOT/'fixtures/parity/ggplot2/scale-limit-helpers.json').read_text())['expansions']
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current');records=[]
def primary(t):
 category=t['family']=='category'
 return c.Data.columns({'x':c.column(['b','c','b','c'] if category else [1.,2.,3.,4.],kind='category' if category else 'float64'),'y':c.column(['b','c','b','c'] if category else [2.,4.,6.,8.],kind='category' if category else 'float64'),'g':c.column(['A','A','B','B'],kind='string')})
def extra(t,reverse=False):
 n=t['result']['helper_rows']
 return c.Data.columns({name:c.column([values[(n-1-i if reverse else i)%len(values)] for i in range(n)],kind='category' if t['family']=='category' else 'float64') for name,values in t['arguments'].items()},name='expansion')
def layer(t,e):
 a=c.aes()
 for name in t['arguments']:a=getattr(a,'color' if name=='colour' else name)(name)
 return c.blank().name('expansion').data(e).independent().aes(a).facet_target('Broadcast')
def build(t,d,e):
 base=c.aes().x('x').y('y')
 if t['axes']=='colour_shared':base=base.color('y')
 p=c.plot(d).profile('Ggplot2_4_0_3').aes(base).layer(c.points()).layer(layer(t,e))
 if t['facet']!='none':p=p.facet(c.facet_wrap('g').free_x(t['facet']=='free').free_y(t['facet']=='free'))
 return p.build()
def domain(layers,axis):
 # Compare training membership; fresh and retained dictionaries may use different ordinals.
 spaces=[l['domains'][axis+'_space'] for l in layers if l['domains'][axis+'_space'] is not None]
 if spaces and isinstance(spaces[0],dict) and any(k in spaces[0] for k in ('Categorical','NullableCategorical')):
  values=[]
  for space in spaces:values.extend(next(iter(space.values()))['categories'])
  return sorted(set(values))
 extents=[l['domains'][axis] for l in layers if l['domains'][axis] is not None]
 return [min(e['minimum'] for e in extents),max(e['maximum'] for e in extents)] if extents else None
def check(t,chart,frame):
 s=chart.semantics();panels=s['panels'] or [{'key':None,'layers':s['layers']}]
 assert len(panels)==len(t['result']['panels'])
 all_layers=[l for p in panels for l in p['layers']];painted=0;result=[]
 for p,w in zip(panels,t['result']['panels']):
  ls=p['layers'];assert len(ls)==2
  assert ls[1]['targets']==[] and ls[1]['invalid_geometry']==0
  painted+=len(ls[0]['targets'])
  trained=all_layers if t['facet']=='fixed' else ls
  actual={a:domain(trained,a) for a in ['x','y']};assert actual==w,(t,actual,w)
  labels=None
  if t['axes'] in ('colour','colour_shared'):
   legend=ls[1]['color_legend'];labels=[v[0] for v in legend['entries']] if t['family']=='category' else [v['label'] or '' for v in legend['numeric_breaks']]
   assert labels==t['result']['colour_labels']
  result.append({'key':p['key'],'domains':actual,'blank_training':{a:domain([ls[1]],a) for a in ['x','y']},'color_labels':labels})
 assert painted==4
 # Guide IDs and data owners are intentionally absent from the comparison record.
 guides=[{'scope':g['scope'],'side':g['spec']['side'],'ticks':g['ticks']} for g in frame.guides()['guides']]
 point_colors=None
 if t['axes']=='colour_shared':
  point_colors=[i['primitive']['Point']['fill'] for i in frame.scene()['items'] if i['layer'] is not None and 'Point' in i['primitive']]
  expected=[{'red':int(h[1:3],16),'green':int(h[3:5],16),'blue':int(h[5:7],16),'alpha':255} for h in t['result']['point_colours']]
  assert point_colors==expected
 if t['axes']=='colour':
  palette=[v[1] for v in panels[0]['layers'][1]['color_legend']['entries']]
  assert not any(i['layer'] is None and i['primitive'].get('Rectangle',{}).get('fill') in palette for i in frame.scene()['items'])
 return {'panels':result,'guides':guides,'point_colors':point_colors}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=primary(t);e=extra(t);owned.extend([d,e]);p=build(t,d,e);owned.append(p)
  wire=p.to_json();assert json.loads(wire)['version']==(45 if t['axes'].startswith('colour') else 41)
  restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
  edited=restored.edit().layer('expansion',layer(t,e)).build();owned.append(edited)
  for state,current in [('original',p),('restored',restored),('edited',edited)]:
   chart=current.chart();request=output.request(current,options);frame=request.prepare();owned.extend([chart,request,frame])
   held=frame.scene();records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and t['axes'] in ('xy','colour','colour_shared'):
    for fmt in ['svg','pdf','png']:(out/f"{t['axes']}-{t['family']}-{t['facet']}.{fmt}").write_bytes(frame.export(fmt))
   assert frame.scene()==held
  if t['axes']=='xy':
   replacement=extra(t,True);owned.append(replacement)
   tx=chart.transaction();owned.append(tx);builder=tx.replace(e,replacement);owned.append(builder);update=builder.build();owned.append(update)
   assert 'Applied' in chart.commit(update)
   fresh=build(t,d,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
   ar=output.request(chart,options);br=output.request(batch,options);owned.extend([ar,br]);af=ar.prepare();bf=br.prepare();owned.extend([af,bf])
   actual=check(t,chart,af);assert actual==check(t,batch,bf)
   assert af.export('png')==bf.export('png')
   assert frame.scene()==held and p.to_json()==wire
   records.append({'index':index,'state':'replaced_reversed',**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==96
(out/'records.json').write_text(json.dumps(records));options.dispose();output.dispose()
print('PASS',len(records),'blank-layer host states')
