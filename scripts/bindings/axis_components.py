"""AXIS-05: independently authored primary styles, retained roles and publication."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
formatter=lambda mode:{'Registered':{'operation':{'id':'example.guide_format','version':1},'parameters':{'mode':mode}}}
styles={'domain':{'color':'#173f66','width':2.,'dashes':[8.,3.]},'ticks':{'color':'#77889999','width':1.5},'labels':{'color':'#174c75','font_size':12.},'per_tick':[{'index':2,'label':{'color':'#cc4400','font_size':18.}},{'index':3,'line':{'visible':False}},{'index':4,'line':{'color':'#cc4400','dashes':[2.,3.]}}]}
data=c.Data.columns({'x':[0.,.25,.5,.75,1.],'y':[1.,2.,3.,2.,1.]},keys=[9007199254741001+i for i in range(5)],name='ticks')
p=(c.plot(data).with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points())
 .x_axis(c.x_axis().scale(c.scale_linear().domain(0.,1.)).range(500.,100.).guide_components(styles).guide_profile('D3_3_0_0')
  .guide_geometry({'inner':-180.,'outer':12.,'padding':8.,'clip_ticks':True})
  .tick_values([0.,.25,.5,.5,.75,1.]).tick_format({'Labels':['zero','','mid','','','one']}))
 .y_axis(c.y_axis().range(280.,100.).visible(False))
 .guide(c.axis_guide('top','x').side('Top').guide_components({'domain':{'visible':False},'ticks':{'color':'#228844'},'labels':{'color':'#225533','font_size':11.}})
  .tick_size_inner(-18.).tick_size_outer(-12.).tick_padding(-8.).tick_offset(0.).guide_profile('D3_3_0_0').tick_values([0.,.5,1.]).tick_format(formatter('Indexed')))
 .guide(c.axis_guide('lower','x').side('Bottom').translate(15.,40.).guide_components({'domain':{'color':'#228844','width':2.},'labels':{'color':'#225533','font_size':12.},'per_tick':[{'index':2,'label':{'visible':False}}]})
  .tick_size(0.).tick_padding(3.).guide_profile('D3_3_0_0').tick_values([0.,.5,1.]).tick_format(formatter('Same')))
 .title(c.title('Independent domain, tick and label styles')).build())
# Changing caller-owned styles after construction cannot affect the retained plot.
styles['domain']['width']=99.
wire=p.to_json();assert json.loads(wire)['version']==14
loaded=c.Plot.from_json(wire,registry);assert loaded.to_json()==wire
(out/'components.plot.json').write_text(wire)
registry.dispose();p.dispose()
for name,text in [('text','preserve'),('outline','outline')]:
 for dpi in [300,600]:
  request=output.request(loaded,c.export_options(600.,400.).dpi(dpi).text(text).basis('current'));frame=request.prepare()
  scene=frame.scene();assert scene['version']==14
  components=[i for i in scene['items'] if 'guide' in i]
  assert sum(i['guide']['role']=='Domain' for i in components)==2
  assert sum(i['guide']['role']=='Line' for i in components)==11
  assert sum(i['guide']['role']=='Label' for i in components)==8
  assert all(not targets for i,targets in zip(scene['items'],scene['targets']) if 'guide' in i)
  assert all(i['primitive']['DashedPath']['stroke']['width']==2. for i in components if i['guide']['role']=='Domain' and 'DashedPath' in i['primitive'])
  prefix=f'{name}-{dpi}'
  (out/f'{prefix}.scene.json').write_text(json.dumps(scene));(out/f'{prefix}.guides.json').write_text(json.dumps(frame.guides()))
  for fmt in ['svg','pdf','png']:(out/f'{prefix}.{fmt}').write_bytes(frame.export(fmt))
  frame.dispose();request.dispose()
# Per-occurrence typography uses the common supplied font/shaper, including rotation.
typo={'labels':{'font_size':12.,'typography':{'text':'ignored','size':1.25}},'per_tick':[{'index':1,'label':{'font_size':18.,'rotation':30.,'typography':{'text':'ignored','size':1.,'weight':400,'tabular':True}}}]}
td=c.Data.columns({'x':[0.,1.],'y':[0.,1.]})
tp=c.plot(td).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().guide_profile('D3_3_0_0').tick_values([0.,.5,1.]).guide_components(typo)).y_axis(c.y_axis().visible(False)).build()
tr=output.request(tp,c.export_options(400.,300.).dpi(72).basis('current'));tf=tr.prepare();ts=tf.scene()
glyphs=[i['primitive']['GlyphRun'] for i in ts['items'] if i.get('guide',{}).get('role')=='Label']
assert [g['run']['font_size'] for g in glyphs]==[15.,18.,15.]
assert [g['rotation'] for g in glyphs]==[0.,30.,0.]
(out/'typography.plot.json').write_text(tp.to_json());(out/'typography.scene.json').write_text(json.dumps(ts))
for fmt in ['svg','pdf','png']:(out/f'typography.{fmt}').write_bytes(tf.export(fmt))
tf.dispose();tr.dispose();tp.dispose();td.dispose()
# Invalid styles fail at the common builder boundary, and reset removes v14 capability.
for bad in [{'ticks':{'width':0.}},{'labels':{'font_size':-1.}},{'ticks':{'dashes':[1.,2.,3.]}},{'per_tick':[{'index':1},{'index':1}]}]:
 try:c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().guide_components(bad)).build()
 except c.ChartError:pass
 else:raise AssertionError('invalid component style accepted')
reset=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().guide_components({'labels':{'font_size':18.}}).guide_components(None)).build()
assert json.loads(reset.to_json())['version']<14
reset.dispose();loaded.dispose();data.dispose();output.dispose()
print('PASS Python AX04: v14 styles/roles, owned input, invalid/reset controls, text/outline SVG/PDF/PNG at 300/600 DPI.')
