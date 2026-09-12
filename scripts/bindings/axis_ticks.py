"""AXIS-02/03 actual primary Python guide selection, formatting and captured records."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(600.,400.).dpi(72).basis('current')
def formatter(mode,name='example.guide_format',version=1):
 return {'Registered':{'operation':{'id':name,'version':version},'parameters':{'mode':mode}}}
def configure(axis,config):
 for source,method in [('tickSize','tick_size'),('tickSizeInner','tick_size_inner'),('tickSizeOuter','tick_size_outer'),('tickPadding','tick_padding'),('offset','tick_offset')]:
  if source in config:axis=getattr(axis,method)(config[source])
 if 'arguments' in config:
  args=config['arguments'];axis=axis.tick_arguments({'count':args[0] if args else None,'specifier':args[1] if len(args)>1 else None})
 if 'interval' in config:
  name,step=config['interval'];axis=axis.tick_arguments({'interval':{'unit':{'utcDay':'Day','utcMinute':'Minute','utcYear':'Year'}[name],'step':step}})
 if 'values' in config:axis=axis.tick_values(config['values'])
 if 'formatter' in config:axis=axis.tick_format(None if config['formatter'] is None else formatter(config['formatter']))
 return axis
records=[]
for profile in json.loads((ROOT/'fixtures/axes/reference.json').read_text())['profiles']:
 for case in profile['cases']:
  s=case['scale'];kind=s['kind'];domain=s['domain'];horizontal=case['side'] in ['Top','Bottom']
  data=c.Data.columns({'value':c.categorical(domain) if kind in ['Band','Point'] else c.timestamps(domain,'ms','UTC') if kind=='Utc' else domain,'other':[1.]*len(domain)})
  axis=(c.x_axis() if horizontal else c.y_axis()).side(case['side']).guide_profile('D3_3_0_0')
  owned=None
  if kind in ['Band','Point']:
   spec={'domain':domain,'align':s.get('align',.5),'round':s.get('round',False)}
   if kind=='Band':spec.update(padding_inner=s.get('padding',0),padding_outer=s.get('padding',0));scale=c.scale_band_d3(spec)
   else:spec['padding']=s.get('padding',0);scale=c.scale_point_d3(spec)
  elif kind=='Utc':scale=c.scale_utc().time_domain(*domain)
  else:
   owned=c.StandaloneScale(kind.lower(),domain=domain,**{k:s[k] for k in ['base','constant','exponent'] if k in s});scale=c.scale_numeric(owned)
  axis=axis.scale(scale)
  if 'range' in s:axis=axis.range(*s['range'])
  elif kind=='Identity':axis=axis.range(*domain)
  axis=configure(axis,case['config'])
  for index,state in enumerate(case['states']):
   if index:axis=configure(axis,case['config']['reset'])
   builder=c.plot(data).with_registry(registry).aes(c.aes().x('value' if horizontal else 'other').y('other' if horizontal else 'value')).layer(c.points())
   p=(builder.x_axis(axis) if horizontal else builder.y_axis(axis)).build()
   wire=p.to_json();assert json.loads(wire)['version']==(13 if any(k in case['config'] for k in ['tickSize','tickSizeInner','tickSizeOuter','tickPadding','offset']) else 11)
   frame=output.request(p,options.layout(c.layout_options().device_scale(profile['device_scale']))).prepare()
   guides=[g for g in frame.guides()['guides'] if g['spec'].get('profile')=='D3_3_0_0'];assert len(guides)==1
   actual=[{'value':t['value'],'label':t['label']} for t in guides[0]['ticks']]
   expected=[{'value':{'Category':t['value']} if kind in ['Band','Point'] else {'Timestamp':{'value':str(t['value']['Timestamp']),'unit':'Milliseconds'}} if kind=='Utc' else {'Number':t['value']},'label':t['text']['label']} for t in state['ticks']]
   assert actual==expected,(case['id'],index,actual,expected)
   for tick,reference in zip(guides[0]['ticks'],state['ticks']):
    coordinate=float(reference['attributes']['transform'].removeprefix('translate(').removesuffix(')').split(',')[0 if horizontal else 1])
    assert abs(tick['position']-coordinate)<=1e-9,(case['id'],tick,coordinate)
   records.append({'case':case['id'],'state':index,'ticks':actual})
   frame.dispose();p.dispose()
  if owned:owned.dispose()
  data.dispose()
assert len(records)==376
(out/'reference-records.json').write_text(json.dumps(records,indent=2))
# Exact timestamps survive the primary value conversion and captured guide JSON.
base=9007199254740993
data=c.Data.columns({'x':c.timestamps([base,base+1000000000],'ns','UTC'),'y':[0.,1.]})
axis=c.x_axis().guide_profile('D3_3_0_0').tick_arguments({'interval':{'unit':'Millisecond','step':1}}).tick_values([{'Timestamp':{'value':str(base+1),'unit':'Nanoseconds'}}]).tick_format(formatter('Context'))
p=c.plot(data).with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).build()
frame=output.request(p,options).prepare();tick=[g for g in frame.guides()['guides'] if g['spec'].get('profile')=='D3_3_0_0'][0]['ticks'][0]
assert tick['value']['Timestamp']['value']==str(base+1) and tick['label']==f'1:0:{base+1}'
frame.dispose();p.dispose();data.dispose()
def reject(fn,code):
 try:fn()
 except c.ChartError as e:assert e.code==code,(e,code)
 else:raise AssertionError('Expected '+code)
data=c.Data.columns({'x':[0.,1.],'y':[0.,1.]})
def invalid_format(name='example.guide_format',version=1,mode='Same'):
 return c.plot(data).with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().guide_profile('D3_3_0_0').tick_format(formatter(mode,name,version))).build()
reject(lambda:invalid_format(version=2),'CHART_UNSUPPORTED_CAPABILITY')
reject(lambda:invalid_format(mode='Unknown'),'CHART_VALIDATION')
native=invalid_format('example.native_guide_format');reject(native.to_json,'CHART_UNSUPPORTED_CAPABILITY');reject(lambda:output.request(native,options).prepare(),'CHART_UNSUPPORTED_CAPABILITY');native.dispose();data.dispose()
# An independently authored publication with blank/repeated labels and one registered formatter.
data=c.Data.columns({'x':[0.,.25,.5,.75,1.],'y':[1.,2.,3.,2.,1.]},keys=[9007199254741001+i for i in range(5)],name='ticks')
p=(c.plot(data).with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points())
 .x_axis(c.x_axis().scale(c.scale_linear().domain(0.,1.)).range(100.,500.).guide_profile('D3_3_0_0').tick_values([0.,.25,.5,.5,.75,1.]).tick_format({'Labels':['zero','','mid','','','one']}))
 .y_axis(c.y_axis().range(280.,100.).visible(False))
 .guide(c.axis_guide('top','x').side('Top').guide_profile('D3_3_0_0').tick_values([0.,.5,1.]).tick_format(formatter('Indexed')))
 .guide(c.axis_guide('lower','x').side('Bottom').translate(0.,32.).guide_profile('D3_3_0_0').tick_values([0.,.5,1.]).tick_format(formatter('Same')))
 .title(c.title('Independent values and labels')).build())
# Authored fixture IDs are independent of the preceding reference-corpus allocations.
descriptor=json.loads(p.to_json())
for index,name in enumerate(['top','lower']):
 descriptor['definition']['guides'][index]['id']=str(1026+index)
 descriptor['guides'][name]=str(1026+index)
p.dispose();p=c.Plot.from_json(json.dumps(descriptor),registry)
wire=p.to_json();loaded=c.Plot.from_json(wire,registry);assert loaded.to_json()==wire
request=output.request(p,options);registry.dispose();p.dispose();frame=request.prepare()
(out/'ticks.plot.json').write_text(wire);(out/'ticks.guides.json').write_text(json.dumps(frame.guides(),indent=2));(out/'ticks.scene.json').write_text(json.dumps(frame.scene()))
for fmt in ['svg','pdf','png']:(out/f'ticks.{fmt}').write_bytes(frame.export(fmt))
second=output.request(loaded,options).prepare();assert second.guides()==frame.guides();assert second.export('png')==frame.export('png')
second.dispose();loaded.dispose();frame.dispose();request.dispose();data.dispose();output.dispose()
print('PASS Python AX02: 372 reference cases / 376 states, exact values/order/labels, independent resets, typed timestamps above 2^53, v11 registry round trip and retained SVG/PDF/PNG.')
