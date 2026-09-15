"""FIX-GG04/GRA-03/07: explicit panel and chart-scope vector composition."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
mixed_routes="--mixed-routes" in sys.argv[3:]
identity_chain=mixed_routes or "--identity-chain" in sys.argv[3:]
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/"fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes());options=c.export_options(640,360).dpi(96).basis("current")
def build(d,kind,free,shared,binned,mode,special):
 target=({'Panels':[{'values':[{'Text':v}]} for v in ['C','A']]} if kind=='panels' else 'Match') if special else 'Broadcast'
 if route==3:target='Broadcast' if special else 'Match'
 elif route==4:target='Match'
 scope='Chart' if special and kind=='chart' else 'Facet'
 presentation='Broadcast' if shared and kind=='chart' else target
 layer=c.points().name('marks')
 if shared:layer=layer.from_transform('mean').after_stat(c.stat_aes().x(1.).y('Mean'))
 elif identity_chain and special:layer=layer.from_transform('upper')
 elif identity_chain:layer=layer.filter(c.filter('v').minimum(0.)).filter(c.filter('v').maximum(100.))
 scale=c.scale_binned({'bins':{'limits':[0.,5.],'oob':'Squish','right':True,'breaks':{'Nice':3.}}}) if binned else c.scale_linear()
 axis=c.y_axis().scale(scale).oob_function({'operation':{'id':'example.scale_vector','version':'1'},'parameters':{'mode':mode if mode=='default' else 'oob_'+mode}})
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('i').y('v')).layer(layer.facet_target(presentation).scope(scope)).y_axis(axis).facet(c.facet_wrap('f').order([['A'],['B'],['C']] if (special and kind=='panels') or route in (3,4) else [['A'],['C']]).free_y(free))
 ancestor='Broadcast' if route in (1,4,5) else {'Panels':[{'values':[{'Text':v}]} for v in ['A','C']]} if route==2 else 'Match' if route==3 else target
 if identity_chain and special:draft=draft.transform(c.transform('lower',c.identity_stat()).facet_target(ancestor).scope('Chart' if route==5 else scope).filter(c.filter('v').minimum(0.))).transform(c.transform('upper',c.identity_stat()).from_transform('lower').facet_target(ancestor).scope(scope).filter(c.filter('v').maximum(100.)))
 mean=c.transform('mean',c.summary().x('v'))
 if identity_chain:mean=mean.from_transform('upper') if special else mean.filter(c.filter('v').minimum(0.)).filter(c.filter('v').maximum(100.))
 if shared:draft=draft.transform(mean.facet_target(target).scope(scope)).layer(c.points().from_transform('mean').facet_target(presentation).scope(scope).after_stat(c.stat_aes().x(2.).y('Mean')))
 return draft.build()
def data(values):return c.Data.columns({'v':c.column([999.,*values,-99.] if identity_chain else values,kind='float64'),'i':c.column([99.,1.,2.,-99.] if identity_chain else [1.,2.],kind='float64'),'f':c.column(['A','A','C','C'] if identity_chain else ['A','C'],kind='string')})
def state(chart):
 try:
  panels=chart.semantics()['panels'];result=[]
  for p in panels:
   if p['key']=={'values':[{'Text':'B'}]}:
    assert all(not layer['point_positions'] for layer in p['layers'])
   else:result.append([[point for point in layer['point_positions']] for layer in p['layers']])
  return {'positions':result}
 except c.ChartError as e:
  assert e.code in ('CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN');return {'error':e.code}
for route in (range(1,6) if mixed_routes else (0,)):
 for kind in (('panels',) if mixed_routes else ('panels','chart')):
  for free in (False,True):
   for shared in (False,True):
    for binned in (False,True):
     for mode in ('index','reverse','short','empty','null','default'):
      owned=[]
      try:
       d=data([10.,20.]);owned.append(d);base=build(d,kind,free,shared,binned,mode,False);owned.append(base);bc=base.chart();owned.append(bc);expected=state(bc)
       p=build(d,kind,free,shared,binned,mode,True);owned.append(p);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
       for label,current in [('original',p),('restored',restored)]:
        chart=current.chart();owned.append(chart);actual=state(chart);assert actual==expected,(kind,free,shared,binned,mode,actual,expected);records.append({'route':route,'kind':kind,'free':free,'shared':shared,'binned':binned,'mode':mode,'state':label,**actual})
       if mode=='index':
        if 'error' not in expected:
         request=output.request(p,options);owned.append(request);frame=request.prepare();owned.append(frame)
         prefix=f'{route}-' if mixed_routes else ''
         for fmt in ('svg','pdf','png'):(out/f'{prefix}{kind}-{int(free)}-{int(shared)}-{int(binned)}.{fmt}').write_bytes(frame.export(fmt))
        chart=p.chart();owned.append(chart);replacement=data([4.,8.]);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(d,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
        fresh=build(replacement,kind,free,shared,binned,mode,True);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=state(chart);assert actual==state(batch);assert p.to_json()==wire;records.append({'route':route,'kind':kind,'free':free,'shared':shared,'binned':binned,'mode':mode,'state':'replaced',**actual})
      finally:
       for obj in reversed(owned):obj.dispose()
assert len(records)==(520 if mixed_routes else 208),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,separators=(',',':')))
output.dispose();options.dispose();registry.dispose();print('PASS',len(records),'panel scope vector states')
