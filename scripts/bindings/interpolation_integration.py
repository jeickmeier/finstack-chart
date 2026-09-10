"""IP06 actual registered interpolation consumers, explicit motion and retained publication."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example()
def factory(mode,native=False):return c.registered_interpolation(registry,'example.native_interpolation' if native else 'example.interpolation',1,{'mode':mode})
def reject(fn):
 try:fn()
 except (c.ChartError,ValueError,TypeError,OverflowError):pass
 else:raise AssertionError('Expected rejection')
f=factory('SquaredNumber');i=f(0.,100.);copy=f.copy();f.dispose();reject(lambda:f(0.,1.));assert i.quantize(5)==[0.,6.25,25.,56.25,100.]
wire=i.to_json();assert json.loads(wire)['version']==2;reject(lambda:c.Interpolator.from_json(wire));restored=c.Interpolator.from_json(wire,registry);assert restored(.5)==25.
piece=c.piecewise(copy,[0.,100.,200.]);assert piece.quantize(5)==[0.,25.,100.,125.,200.]
for version in [2,9007199254740993,True,1.5]:reject(lambda:c.registered_interpolation(registry,'example.interpolation',version,{'mode':'SquaredNumber'}))
reject(lambda:c.registered_interpolation(registry,'example.interpolation',1,{'mode':'Bad'}))
native=factory('SquaredNumber',True);native_i=native(0.,100.);reject(native_i.to_json)
for family in ['linear','utc']:
 domain=[0.,10.] if family=='linear' else [9007199254740993,9007199254741993]
 s=c.StandaloneScale(family,registry=registry,domain=domain,range=[0.,100.],factory=copy,**({'unit':'Nanoseconds'} if family=='utc' else {}))
 assert s.map(5. if family=='linear' else 9007199254741493)==25.
 r=c.StandaloneScale.from_json(s.to_json(),registry);s.dispose();assert r.map(domain[0])==0.;r.dispose()
for family,domain,x,expected in [('sequential',[0.,10.],5.,25.),('diverging',[-10.,0.,100.],50.,56.25),('sequential_quantile',[0.,1.,2.,3.,4.],2.,25.)]:
 s=c.StandaloneScale(family,registry=registry,domain=domain,interpolator=i,**({} if family=='sequential_quantile' else {'unknown':None,'clamp':True}));assert s.map(x)==expected and s.map() is (c.MISSING if family=='sequential_quantile' else None);assert s.map(200.)==100.;s.dispose()
ns=c.StandaloneScale('linear',registry=registry,factory=native);reject(ns.to_json);ns.dispose()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(600.,400.).dpi(72).basis('current')
def figure(t,native=False,rows=None):
 color_factory=factory('LabColor',native);size_factory=factory('SquaredNumber',native)
 color=c.StandaloneScale('linear',registry=registry,factory=color_factory,range=['#d54a3a','#28689b']);size=c.StandaloneScale('linear',registry=registry,factory=size_factory,range=[5.,13.])
 rows=rows if rows is not None else [(9007199254741001+j,j/20.) for j in range(21)]
 data=c.Data.columns({'x':[r[1] for r in rows]},keys=[r[0] for r in rows],name='interpolation')
 transform=c.interpolate_transform_svg('translate(0,0) rotate(-25) scale(0.7)','translate(40,10) rotate(65) scale(1.3)');zoom=c.interpolate_zoom([0.,0.,100.],[40.,20.,50.]);view=zoom(t)
 triangle=c.Path().move_to(-24.,18.).line_to(0.,-25.).line_to(24.,18.).close_path();camera=c.Path().rect(-35.,-20.,70.,40.)
 p=(c.plot(data).with_registry(registry).aes(c.aes().x('x').y(1.).color('x').color_scale('perceptual'))
  .layer(c.points().numeric_scale('Size','x',size)).scale(c.color_mapped('perceptual',color)).legend(c.legend().scale('perceptual').title('Registered Lab ramp'))
  .x_axis(c.x_axis().scale(c.scale_linear().domain(0.,1.)).range(65.,415.))
  .y_axis(c.y_axis().scale(c.scale_linear().domain(0.,2.)).range(155.,65.).visible(False))
  .layer(c.vector_path('sampled-transform',triangle).transform(transform.sample_transform(t),max_error=.001,max_commands=10000).anchor({'Output':{'x':155.,'y':255.}}).fill({'red':213,'green':74,'blue':58,'alpha':220}))
  .layer(c.vector_path('sampled-zoom',camera).transform([100./view[2],0.,0.,100./view[2],-view[0],-view[1]],max_error=.001,max_commands=10000).anchor({'Output':{'x':395.,'y':255.}}).fill(None).stroke({'color':{'red':40,'green':104,'blue':155,'alpha':255},'width':2.}))
  .layer(c.labels().id('transform-label').output_at(135.,330.).text('Sampled transform'))
  .layer(c.labels().id('zoom-label').output_at(340.,330.).text('Sampled zoom'))
  .title(c.title(f'Shared interpolation / t = {t:.2f}')).theme(c.theme().preset('Editorial')).build())
 for v in [color_factory,size_factory,color,size,data,transform,zoom,triangle,camera]:v.dispose()
 return p
for t in [0.,.5,1.]:
 folder=out/f'frame-{t:g}';folder.mkdir(exist_ok=True);p=figure(t);wire=p.to_json();assert json.loads(wire)['version']==12
 loaded=c.Plot.from_json(wire,registry);request=output.request(p,options);p.dispose();frame=request.prepare();other=output.request(loaded,options).prepare();assert frame.export('png')==other.export('png')
 # Every mark is sampled directly; its final bytes equal the standalone floating factory.
 ramp_factory=factory('LabColor');ramp=ramp_factory('#d54a3a','#28689b');marks=[v['primitive']['Point'] for v in frame.scene()['items'] if v.get('layer') is not None and 'Point' in v['primitive']];assert len(marks)==21
 for j,mark in enumerate(marks):
  sample=ramp(j/20.);hex8=sample.format_hex8();sample.dispose();assert hex8=='#'+''.join(f"{mark['fill'][k]:02x}" for k in ['red','green','blue','alpha']);assert abs(mark['radius']-(5.+8.*(j/20.)**2))<1e-12
 ramp.dispose();ramp_factory.dispose()
 (folder/'interpolation.plot.json').write_text(wire);(folder/'interpolation.scene.json').write_text(json.dumps(frame.scene()));(folder/'interpolation.guides.json').write_text(json.dumps(frame.guides()))
 for fmt in ['svg','pdf','png']:(folder/f'interpolation.{fmt}').write_bytes(frame.export(fmt))
 frame.dispose();other.dispose();request.dispose();loaded.dispose()
p=figure(.5,True);reject(p.to_json);reject(lambda:output.request(p,options).prepare());p.dispose()
rows=[(9007199254741001,0.),(9007199254741002,.5),(9007199254741003,1.)];p=figure(.5,rows=rows);chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));held=old.prepare();png=held.export('png');held.dispose()
for step in range(4):
 tx=chart.transaction().id(f'ip-{step}')
 if step<2:
  row=(9007199254741004,.75) if step==0 else (9007199254741002,.25)
  if step==0:rows.append(row)
  else:rows[1]=row
  batch=c.Data.columns({'x':[row[1]]},keys=[row[0]],name='interpolation');tx=tx.append('interpolation',batch) if step==0 else tx.upsert('interpolation',batch);batch.dispose()
 elif step==2:rows.pop(0);tx=tx.remove('interpolation',[9007199254741001])
 else:rows.pop(0);tx=tx.retain_count('interpolation',2)
 tx=tx.build();assert 'Applied' in chart.commit(tx);tx.dispose();fresh=figure(.5,rows=rows);a=chart.request(output,options).prepare();b=output.request(fresh,options).prepare();assert a.export('png')==b.export('png');a.dispose();b.dispose();fresh.dispose()
chart.dispose();registry.dispose();frame=old.prepare();assert frame.export('png')==png;frame.dispose();old.dispose();last=copy(0.,100.);assert last(.5)==25.;last.dispose()
for v in [copy,i,restored,piece,native,native_i,output]:v.dispose()
print('PASS IP06 Python: registered/piecewise/time factories, v2/v12, native-only rejection, three retained frames, four update/fresh PNG comparisons and disposal.')
