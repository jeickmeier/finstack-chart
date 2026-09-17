"""GG16 independent Python primary authors, ownership and reference-vector checks."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(10):
    if mode==8:
        data=c.Data.columns(dict(x=[0.,1.,2.],y=[2.,5.,10.],w=[1.,2.,1.]))
        options={'method':{'Registered':{'operation':{'id':'example.prescribed_slope','version':'1'},'parameters':{'slope':2.,'envelope':1.25}}},'terms':None,'n':80,'full_range':False,'se':True,'xseq':[-1.,0.,1.,2.,3.],'level':.8}
        builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').layer(c.smooth().stat(c.model_stat(options).x('x').y('y').weight('w')))
    elif mode==4:
        data=registry.materialize('example.xy_recipe',1,dict(x=[0.,1.,2.],y=[2.,4.,3.]))
        builder=c.autoplot(data,'example.xy_recipe',1,dict(line=True),registry)
    else:
        x=[-3.,-2.,-1.,0.,1.,2.,3.]
        groups=c.cut_number(x,3) if mode==1 else c.cut_width(x,2.,center=0.) if mode==2 else c.cut_interval(x,n=3)
        clone=groups.copy();assert groups.value()==clone.value();groups.dispose()
        try: groups.value();raise AssertionError('disposed result succeeded')
        except c.ChartError:pass
        levels=clone.value()["levels"];column=clone.column();clone.dispose()
        data=c.Data.columns(dict(x=x,y=[1.,4.,2.,5.,3.,6.,4.],group=column));column.dispose()
        layer=c.points().key_glyph('example.diamond_key',1,dict(padding=.1)) if mode==3 else c.points()
        builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').scale(c.color_discrete('groups').domain(levels)).aes(c.aes().x('x').y('y').color('group').color_scale('groups')).layer(layer)
    if mode==5: builder=builder.legend(c.legend().scale("groups").registered("example.strip_guide",1,{}))
    if mode==6: builder=builder.facet(c.facet_wrap("group").registered("example.reverse_facets",1,{"columns":2}))
    if mode==7: builder=builder.registered_coordinate("example.wave_coordinate",1,{"amplitude":.12}).layer(c.line()).layer(c.line().independent().data(c.Data.columns(dict(x=[-3.,3.],y=[3.5,3.5]),name="wave-span")).aes(c.aes().x("x").y("y")))
    if mode==9: builder=builder.facet(c.facet_wrap("group").reference({"labeller":{"registered":{"operation":{"id":"example.facet_labels","version":"1"},"parameters":"context"}}}))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire,registry)
    request=output.request(restored,c.export_options(600,360).dpi(144));frame=request.prepare()
    original_request=output.request(p,c.export_options(600,360).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==frame.scene();original_frame.dispose();original_request.dispose()
    (out/f'extension-{mode}.plot.json').write_text(wire);(out/f'extension-{mode}.scene.json').write_text(json.dumps(frame.scene()))
    for fmt in ['svg','pdf','png']:(out/f'extension-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
records=[]
for case in json.loads((ROOT/'fixtures/parity/ggplot2/vector-helper-controls.json').read_text())['cases']:
    x=case['x'] if isinstance(case['x'],list) else [case['x']];options=case['options'];op=case['operation']
    if op=='resolution':
        value=c.resolution(x,zero=options['zero'],integer=case['integer']);assert abs(value-case['result']['value'])<1e-14;records.append(value);continue
    options={('ordered' if k=='ordered_result' else k):v for k,v in options.items()}
    try:
        result=getattr(c,op)(x,**options);value=result.value();result.dispose()
        assert 'error' not in case['result'],case['name']
        for key in ['codes','levels','ordered']:assert value[key]==case['result'][key],case['name']
        records.append(value)
    except (c.ChartError,ValueError):
        assert 'error' in case['result'],case['name'];records.append({'error':True})
(out/'records.json').write_text(json.dumps(records))
registry.dispose();output.dispose();print('PASS Python extension controls: ten authors, 30 publications and 35 vector cases')

summary=c.summarize([1.,None,2.,3.]);assert summary[0]==2.
assert abs(summary[1]-(2.-(1./3.)**.5))<1e-14
assert c.summarize([None])==[None,None,None]
# Native-only callbacks cannot cross the portable boundary, even when installed.
owned_registry=c.ExtensionRegistry.example();kept=owned_registry.copy();owned_registry.dispose()
d=c.Data.columns(dict(x=[0.,1.],y=[1.,2.],g=['a','b']))
base=c.plot(d).with_registry(kept).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('g').color_scale('groups')).scale(c.color_discrete('groups'))
for builder in [base.layer(c.points().key_glyph('example.native_diamond_key',1,{'padding':.1})),base.layer(c.points()).legend(c.legend().scale('groups').registered('example.native_strip_guide',1,{})),base.layer(c.points()).facet(c.facet_wrap('g').registered('example.native_reverse_facets',1,{'columns':1})),base.layer(c.points()).registered_coordinate('example.native_wave_coordinate',1,{'amplitude':.1})]:
    value=builder.build()
    try: value.to_json();raise AssertionError('native-only callback serialized')
    except c.ChartError:pass
    value.dispose()
try: kept.materialize('example.native_xy_recipe',1,dict(x=[1.],y=[2.]));raise AssertionError('native-only materialization executed')
except c.ChartError:pass
retained=base.layer(c.points()).registered_coordinate('example.wave_coordinate',1,{'amplitude':.1}).build();kept.dispose();d.dispose()
assert 'example.wave_coordinate' in retained.to_json();retained.dispose()
print('PASS Python native-only protocol rejection and registry copy/disposal retention')
# Facet labellers share installed guide registry, typed context and strict strip cardinality.
r=c.ExtensionRegistry.example();d=c.Data.columns(dict(x=[0.,1.],y=[1.,2.],g=['a','b']))
for operation,mode in [('example.facet_labels','short'),('example.facet_labels','oversize'),('example.native_facet_labels','context')]:
    p=c.plot(d).with_registry(r).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).facet(c.facet_wrap('g').reference({'labeller':{'registered':{'operation':{'id':operation,'version':'1'},'parameters':mode}}})).build()
    out_engine=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
    try:
        if operation.startswith('example.native'):p.to_json()
        else:out_engine.request(p,c.export_options(600,360)).prepare()
        raise AssertionError('invalid facet labeller succeeded')
    except c.ChartError:pass
    out_engine.dispose();p.dispose()
r.dispose();d.dispose()
