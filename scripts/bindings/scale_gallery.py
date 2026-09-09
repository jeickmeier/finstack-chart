"""SP-07 independently authored Python scale figures and retained publication."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(720.,420.).dpi(144)
figures=[]
x=c.StandaloneScale('linear',domain=[0.,10.,100.],range=[0.,50.,100.])
y=c.StandaloneScale('sqrt',domain=[0.,16.])
figures.append(('numeric',c.plot(c.Data.columns({'x':[0.,5.,10.,55.,100.],'y':[1.,4.,9.,16.,9.]},name='numeric'))
 .aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points().size(4.))
 .x_axis(c.x_axis().scale(c.scale_numeric(x))).y_axis(c.y_axis().scale(c.scale_numeric(y)))
 .axis(c.x_axis().name('secondary').side('Top').secondary('x',.1,0.).numeric_format({'specifier':'.1f'}))
 .title(c.title('Piecewise x, square-root y')).theme(c.theme().preset('Editorial')).build()))
ordinal=c.StandaloneScale('ordinal',range=['#28587b','#c77c35'])
figures.append(('categorical',c.plot(c.Data.columns({'category':c.categorical(['A','B','C','A','B','C']),'region':c.categorical(['North']*3+['South']*3),'value':[2.,5.,8.,20.,35.,50.]},name='categories'))
 .aes(c.aes().x('category').y('value').color('region').color_scale('regions')).layer(c.points().size(7.))
 .x_axis(c.x_axis().scale(c.scale_band_d3({'padding_inner':.3,'padding_outer':.15,'align':.2,'round':True})))
 .scale(c.color_mapped('regions',ordinal,'Eligible')).legend(c.legend().scale('regions').title('Region'))
 .facet(c.facet_wrap('region').columns(2).free_y(True)).title(c.title('Bands with free y facets')).theme(c.theme().preset('Editorial')).build()))
quantile=c.StandaloneScale('quantile',range=['#abc9d8','#4385a5','#163c55'])
size=c.StandaloneScale('threshold',domain=[5.,10.],range=[3.,6.,9.])
figures.append(('classifier',c.plot(c.Data.columns({'x':[0.,1.,2.,3.,4.,5.,6.,7.,8.],'value':[0.,0.,1.,2.,3.,5.,8.,13.,21.]},name='distribution'))
 .aes(c.aes().x('x').y('value').color('value').color_scale('quantiles'))
 .layer(c.points().numeric_scale('Size','value',size))
 .scale(c.color_mapped('quantiles',quantile,'Eligible')).legend(c.legend().scale('quantiles').title('Sample terciles'))
 .title(c.title('Exact quantiles and threshold size')).theme(c.theme().preset('Editorial')).build()))
diverging=c.StandaloneScale('diverging',domain=[-10.,0.,100.],range=['#bb4a43','#f8f5ee','#28587b'],clamp=True)
opacity=c.StandaloneScale('linear',domain=[-10.,100.],range=[.4,1.],clamp=True)
figures.append(('diverging',c.plot(c.Data.columns({'x':[-10.,-5.,0.,25.,50.,75.,100.],'y':[1.,2.,3.,2.,1.,2.,3.]},name='diverging'))
 .aes(c.aes().x('x').y('y').color('x').color_scale('asymmetric'))
 .layer(c.points().size(9.).numeric_scale('Opacity','x',opacity))
 .scale(c.color_mapped('asymmetric',diverging)).legend(c.legend().scale('asymmetric').title('Center = 0'))
 .title(c.title('An asymmetric diverging guide')).theme(c.theme().preset('Editorial')).build()))
zone={'Local':{'version':1,'zone':'America/New_York','revision':'42','tzdata':'2025c','coverage':{'start':'1672531200000','end':'1735689600000'},'initial_offset_seconds':-18000,'transitions':[
 {'at_millis':'1678604400000','offset_seconds':-14400},{'at_millis':'1699164000000','offset_seconds':-18000},
 {'at_millis':'1710054000000','offset_seconds':-14400},{'at_millis':'1730613600000','offset_seconds':-18000}]}}
start=1730606400000
time=c.StandaloneScale('local',domain=[start,start+5*3600000],zone=zone)
figures.append(('local-time',c.plot(c.Data.columns({'time':c.timestamps([start+i*3600000 for i in range(6)],'ms','UTC'),'value':[1.,2.,1.5,3.,2.,4.]},name='local-time'))
 .aes(c.aes().x('time').y('value')).layer(c.line()).layer(c.points().size(3.))
 .x_axis(c.x_axis().scale(c.scale_calendar(time).calendar_interval({'unit':'Hour','step':1})).time_format({'pattern':'%H:%M %Z'}))
 .y_axis(c.y_axis().visible(False)).title(c.title('One elapsed hour through the fold')).theme(c.theme().preset('Editorial')).build()))
for name,plot in figures:
 wire=plot.to_json();assert json.loads(wire)['version']==5
 copy=c.Plot.from_json(wire);assert copy.to_json()==wire;copy.dispose()
 (out/f'{name}.plot.json').write_text(wire)
 request=output.request(plot,options);plot.dispose();frame=request.prepare()
 (out/f'{name}.scene.json').write_text(json.dumps(frame.scene(),indent=2))
 for fmt in ['svg','pdf','png']:(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
 frame.dispose();request.dispose()
# Typed field/expression routes share the exact same core numeric mapping as field names.
owned=c.Data.columns({'x':[0.,1.],'y':[1.,2.]})
for source in [owned.field('y'),c.source_expr(owned.field('y'))]:
 p=c.plot(owned).aes(c.aes().x('x').y('y')).layer(c.points().numeric_scale('Size',source,c.StandaloneScale('linear',domain=[1.,2.],range=[3.,6.]))).build()
 frame=output.request(p,options).prepare()
 assert [i['primitive']['Point']['radius'] for i in frame.scene()['items'] if 'Point' in i['primitive'] and i.get('layer') is not None]==[3.,6.] # the retained scene is in points; raster export applies dpi
 frame.dispose();p.dispose()
print('PASS SP-07 Python: five independently authored v5 scale figures, typed source routes and retained SVG/PDF/PNG.')
