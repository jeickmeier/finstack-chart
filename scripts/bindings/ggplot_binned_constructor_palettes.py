"""Explicit reference constructor count palettes through actual Python authoring."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
functions='--functions' in sys.argv
invalid='--gradient-invalid' in sys.argv
nonfinite='--gradient-nonfinite' in sys.argv
remap='--gradient-remap' in sys.argv
continuous='--continuous-constructors' in sys.argv or remap or nonfinite or invalid
constructors='--discrete-constructors' in sys.argv
qualitative='--qualitative-types' in sys.argv
ordinal='--ordinal-types' in sys.argv or qualitative or constructors
cases=json.loads((ROOT/'fixtures/parity/ggplot2'/('gradient-invalid-constructors.json' if invalid else 'gradient-nonfinite-constructors.json' if nonfinite else 'gradient-remap-constructors.json' if remap else 'continuous-paint-constructors.json' if continuous else 'discrete-paint-constructors.json' if constructors else 'qualitative-type-palettes.json' if qualitative else 'ordinal-type-palettes.json' if ordinal else 'binned-constructor-functions.json' if functions else 'binned-constructor-palettes.json')).read_text())['cases'];records=[];registry=c.ExtensionRegistry.example()
if constructors or continuous:
 for t in cases:t['palette_name']=t['constructor']+'-'+t['configuration']
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def data_for(t):return c.Data.columns({'x':c.column([float(i) for i in range(len(t['inputs']))],kind='float64'),'v':c.column(t['inputs'] if ordinal else [float(v) if v is not None else None for v in t['inputs']],kind='string' if ordinal else 'float64').nullable(True)},name='data')
def layer():return (c.points().aesthetic_value('Shape',{'kind':'Number','value':21}) if functions or ordinal or continuous else c.points()).name('marks')
def build(t,d):
 palette=None if functions or ordinal or continuous else {'Named':t['palette'][0]} if len(t['palette'])==1 else {'Values':[{'kind':'Color','value':{'space':'Rgb','channels':{'r':int(v[1:3],16),'g':int(v[3:5],16),'b':int(v[5:7],16),'opacity':1}}} for v in t['palette']]}
 scale={'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':'Identity','unknown':{'kind':'Missing'}}},'ggplot':{'Binned':{'palette':{'Discrete':palette},'limits':None,'oob':'Squish','breaks':{'Nice':t.get('count',5)},'right':True}},'guide':'Hidden'}
 if continuous:
  name=t['constructor'];custom=t['configuration']=='custom';binned=name.startswith('steps') or name in ('fermenter','viridis_b');values=t['args'].get('values') if isinstance(t['args'],dict) else None
  if name in ('distiller','fermenter','viridis_c','viridis_b'):
   count_palette={'Viridis':{'option':'Magma' if custom else 'Viridis','begin':.2 if custom else 0,'end':.8 if custom else 1,'alpha':.5 if custom else 1,'reverse':custom}} if name.startswith('viridis') else {'Brewer':{'id':'RdBu' if custom else 'Blues','reverse':not custom}}
   if binned:scale['ggplot']['Binned']['palette']={'Discrete':count_palette};recipe=None
   else:recipe={'CountGradient':{'palette':count_palette,'count':7 if name=='distiller' else 6,'values':values}}
  else:
   colors=(['red','blue'] if custom else ['#132B43','#56B1F7']) if name in ('gradient','steps') else (['red','white','blue'] if custom else ['#832424','white','#3A3A98']) if name in ('gradient2','steps2') else ['#ff000040','#ffffff80','#0000ffcc'] if custom else ['red','white','blue']
   if isinstance(t['args'],dict) and 'colours' in t['args']:colors=t['args']['colours'] if isinstance(t['args']['colours'],list) else [t['args']['colours']]
   recipe={'Gradient':{'colors':colors,'values':values}}
  if not binned:scale['ggplot']={'Continuous':{'limits':None,'oob':'Censor'}}
  elif recipe is not None:scale['ggplot']['Binned']['palette']=None
  if recipe is not None:scale['function']['Interpolated']['output']={'Interpolate':{'operation':'GgplotPalette','spec':recipe}}
  if name in ('gradient2','steps2'):scale['function']['Interpolated']['normalization']['Ggplot']['rescaler']={'Midpoint':1 if custom else 0}
 if ordinal:
  scale['function']={'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}}
  scale['ggplot']={'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'OrdinalColors':t.get('palette',[])}}}
  scale['missing_paint_is_na']=not qualitative
  if qualitative:scale['ggplot']['Discrete']['palette']={'Qualitative':{'palettes':[{'values':p['values'],'names':None if p['names'] is None else [{'Text':v} for v in p['names']]} for p in t['palette']],'fallback':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}}}
 if constructors:
  custom=t['configuration']=='custom'
  chosen={'hue':{'Hue':{'h':[30,300] if custom else [15,375],'chroma':50 if custom else 100,'luminance':80 if custom else 65,'start':10 if custom else 0,'reverse':custom}},'grey':{'Grey':{'start':.1 if custom else .2,'end':.9 if custom else .8}},'brewer':{'Brewer':{'id':'RdBu' if custom else 'Blues','reverse':custom}},'viridis_d':{'Viridis':{'option':'Magma' if custom else 'Viridis','begin':.2 if custom else 0,'end':.8 if custom else 1,'alpha':.5 if custom else 1,'reverse':custom}}}[t['constructor']]
  scale['ggplot']['Discrete']['palette']=chosen
  scale['missing_paint_is_na']=t['constructor'] in ('brewer','viridis_d')
 if functions:
  scale['ggplot']['Binned']['palette']=None
  scale['palette_function']={'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':t['mode'],'channel':'colour','count':True}}
 m=c.aes().x('x').y(1.);m=m.fill('v').fill_scale('v') if t['channel']=='fill' else m.color('v').color_scale('v')
 authored=c.color_mapped('v',scale)
 if constructors and t['constructor']=='grey':authored=authored.missing('#FF0000')
 return c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(m).scale(authored).layer(layer()).build()
def paint(v):
 if v is None:return {'red':0,'green':0,'blue':0,'alpha':0}
 v={'white':'#FFFFFF','red':'#FF0000','blue':'#0000FF','green':'#00FF00','black':'#000000','yellow':'#FFFF00','orange':'#FFA500','purple':'#A020F0','grey50':'#7F7F7F'}.get(v,v)
 return dict(zip(('red','green','blue','alpha'),[int(v[i:i+2],16) for i in (1,3,5)]+[int(v[7:9],16) if len(v)==9 else 255]))
def check(t,chart):
 styles=chart.semantics()['layers'][0].get('styles',[]);assert len(styles)==t['result']['point_count'],t
 wanted=[v for v in t['result']['mapped'] if v is not None or t['channel']=='fill'] if ordinal or continuous else t['result']['point_colours']
 for style,v in zip(styles,wanted):assert style['fill' if t['channel']=='fill' else 'color']==paint(v),(t,style,v)
 return styles
for index,t in enumerate(cases):
 owned=[]
 try:
  try:
   d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert constructors or continuous or json.loads(wire)['version']==(50 if qualitative else 49 if ordinal else 37 if functions else 48)
   if nonfinite:assert json.loads(wire)['version']==52
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   chart=restored.chart();owned.append(chart);styles=check(t,chart) if 'error' not in t['result'] else chart.semantics()
  except c.ChartError:
   assert 'error' in t['result'],t;records.append({'index':index,'state':'error'});continue
  assert 'error' not in t['result'],t
  records.append({'index':index,'state':'original','styles':styles})
  edited=restored.edit().layer('marks',layer()).build();owned.append(edited);ec=edited.chart();owned.append(ec);records.append({'index':index,'state':'layer_edit','styles':check(t,ec)})
  if invalid and t['population']=='empty' and t['configuration']=='zero' and t['constructor']=='gradientn':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-empty-invalid.{fmt}").write_bytes(frame.export(fmt))
  if t['population']=='ordinary':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame);scene=frame.scene()
   if ordinal or continuous or t['count']==5:
    for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-{t['mode'] if functions else t['palette_name']}.{fmt}").write_bytes(frame.export(fmt))
   for population in ('missing','empty','ordinary'):
    expected=next(x for x in cases if all(x[k]==t[k] for k in (('channel','palette_name') if ordinal or continuous else ('channel','mode' if functions else 'palette_name','count'))) and x['population']==population);replacement=data_for(expected);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(d,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    bp=build(expected,replacement);owned.append(bp);batch=bp.chart();owned.append(batch);actual=check(expected,chart);assert actual==check(expected,batch);assert restored.to_json()==wire and frame.scene()==scene;records.append({'index':index,'state':'replacement','population':population,'styles':actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(240 if invalid else 468 if nonfinite else 288 if remap else 480 if continuous else 208 if constructors else 206 if qualitative else 154 if ordinal else 324 if functions else 288),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print(f'PASS {len(records)} constructor palette states and {6 if invalid else 156 if nonfinite else 96 if remap else 120 if continuous else 48 if qualitative or constructors else 36 if functions else 30} publication files')
