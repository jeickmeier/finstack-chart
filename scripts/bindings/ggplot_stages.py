"""FIX-GG02 actual Python authors, R numeric checks and immutable capture/profile transitions."""
import json
import math
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
corpus=json.loads((ROOT/'fixtures/parity/ggplot2/stages.json').read_text())
reference={v['id']:v['layers'][0]['columns'] for v in corpus['cases']}
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(420,280).dpi(96).basis('current')
records=[]
def figure(data):return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'))
def marks(frame):return [i['primitive']['Point'] for i in frame.scene()['items'] if 'Point' in i['primitive'] and i.get('layer') is not None]
def retain(name,plot):
    request=output.request(plot,options);frame=request.prepare()
    (out/f'{name}.plot.json').write_text(plot.to_json())
    (out/f'{name}.scene.json').write_text(json.dumps(frame.scene(),indent=2))
    for fmt in ('svg','pdf','png'):(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
    return request,frame
for name,axis in [
    ('log_mean',c.y_axis().scale(c.scale_log(10))),
    ('coordinate_log_mean',c.y_axis().coordinate_scale(c.scale_log(10))),
    ('scale_limit_mean',c.y_axis().scale(c.scale_linear().domain(1,10))),
    ('coordinate_zoom_mean',c.y_axis().viewport(1,10)),
    ('squish_mean',c.y_axis().scale(c.scale_linear().domain(1,10)).oob('Squish')),
    ('keep_mean',c.y_axis().scale(c.scale_linear().domain(1,10)).oob('Keep')),
]:
    data=c.Data.columns({'x':[1.,1.,1.],'y':[1.,10.,100.]})
    plot=figure(data).layer(c.points().stat(c.summary().x('y')).after_stat(c.stat_aes().x(1.).y('Mean'))).y_axis(axis).build()
    chart=plot.chart();semantics=chart.semantics();domain=semantics['layers'][0]['domains']['y']
    expected=reference[name]['y'][0]
    assert math.isclose(domain['minimum'],expected,rel_tol=1e-12,abs_tol=1e-12),(name,domain,expected)
    assert math.isclose(domain['maximum'],expected,rel_tol=1e-12,abs_tol=1e-12)
    request,frame=retain(name,plot);records.append({'id':name,'y':domain,'point_count':len(marks(frame))})
    frame.dispose();request.dispose();chart.dispose();plot.dispose();data.dispose()
# Shared graph aliases retain the same operation population as inline statistics.
data=c.Data.columns({'x':[1.,1.,1.],'y':[1.,10.,100.]})
for axis,expected in [(c.y_axis().scale(c.scale_log(10)),1.),(c.y_axis().coordinate_scale(c.scale_log(10)),37.)]:
    shared=c.transform('mean',c.summary().x('y'))
    plot=(figure(data).transform(shared).transform(c.transform('copy',c.identity_stat()).from_transform(shared))
          .layer(c.points().from_transform('copy').after_stat(c.stat_aes().x(1.).y('Mean'))).y_axis(axis).build())
    chart=plot.chart();assert math.isclose(chart.semantics()['layers'][0]['domains']['y']['minimum'],expected,abs_tol=1e-12)
    chart.dispose();plot.dispose()
plot=figure(data).layer(c.points().filter(c.filter(c.source_expr('y')*2.).maximum(20.))).build()
chart=plot.chart();assert chart.semantics()['layers'][0]['domains']['y']['maximum']==10.
chart.dispose();plot.dispose();data.dispose()
# Horizontal recipes and generated histogram expressions use the same stage kernel.
hist_data=c.Data.columns({'value':[1.,2.,5.,20.,50.,200.,500.]})
plot=(c.plot(hist_data).profile('Ggplot2_4_0_3').aes(c.aes().y('value'))
      .layer(c.histogram().breaks([1.,10.,100.,1000.])).y_axis(c.y_axis().scale(c.scale_log(10))).build())
chart=plot.chart();rows=chart.semantics()['layers'][0]['rows']['Binned']
assert [int(r['count']) for r in rows]==reference['horizontal_log_histogram']['count']
request,frame=retain('horizontal_log_histogram',plot)
assert len([i for i in frame.scene()['items'] if i.get('layer') is not None and 'Rectangle' in i['primitive']])==3
for v in [frame,request,chart,plot]:v.dispose()
fraction=c.bin_expr('Count')/c.bin_expr('Count').sum()
plot=(c.plot(hist_data).profile('Ggplot2_4_0_3').aes(c.aes().x('value'))
      .layer(c.histogram().breaks([1.,10.,100.,1000.]).after_bin(c.bin_aes().y(fraction))).build())
chart=plot.chart();domain=chart.semantics()['layers'][0]['domains']['y'];assert math.isclose(domain['maximum'],3./7.,abs_tol=1e-12)
request,frame=retain('after_stat_expression',plot)
for v in [frame,request,chart,plot,hist_data]:v.dispose()
category_data=c.Data.columns({'category':c.categorical(['A','A','B'])})
plot=(c.plot(category_data).profile('Ggplot2_4_0_3').aes(c.aes().y('category')).layer(c.bars().orientation('Horizontal'))
      .y_axis(c.y_axis().scale(c.scale_band())).build())
chart=plot.chart();rows=chart.semantics()['layers'][0]['rows']['Statistical'];assert [int(r['count']) for r in rows]==reference['horizontal_count']['count']
request,frame=retain('horizontal_count',plot)
rectangles=[i['primitive']['Rectangle']['bounds'] for i in frame.scene()['items'] if i.get('layer') is not None and 'Rectangle' in i['primitive']]
assert len(rectangles)==2
for v in [frame,request,chart,plot,category_data]:v.dispose()
data=c.Data.columns({'x':[1.,1.,1.],'y':[1.,10.,100.]})
plot=figure(data).aes(c.aes().x(c.source_expr(data.field('x'))+1.).y(c.source_expr('y')*2.)).layer(c.points()).build()
chart=plot.chart();d=chart.semantics()['layers'][0]['domains'];assert d['x']=={'minimum':2.,'maximum':2.};assert d['y']=={'minimum':2.,'maximum':200.}
request,frame=retain('source_expression',plot);frame.dispose();request.dispose();chart.dispose();plot.dispose()
plot=figure(data).layer(c.points().after_scale(c.scale_aes().size(c.after_scale_expr('Size')*2.))).build()
request,frame=retain('after_scale_expression',plot)
# R gg_par fontsize combines size (.pt) and stroke (.stroke/2); circle radius is 3/8 fontsize.
assert all(math.isclose(p['radius'], (size * 72.27 + stroke * 48.) / 25.4 * .375, rel_tol=1e-12)
           for p, size, stroke in zip(marks(frame), reference['after_scale_expression']['size'], reference['after_scale_expression']['stroke'], strict=True))
frame.dispose();request.dispose();plot.dispose()
plot=figure(data).theme(c.theme().geometry(accent='#1256ab')).layer(c.points().after_scale(c.scale_aes().color(c.from_theme('Accent')))).build()
request,frame=retain('theme_expression',plot)
assert all(p['fill']=={'red':18,'green':86,'blue':171,'alpha':255} for p in marks(frame))
frame.dispose();request.dispose();plot.dispose()
# A statistic expression reads back-transformed values, and an old capture retains its profile.
plot=figure(data).layer(c.points().size(2.).color('#1256ab').stat(c.summary().x('y')).after_stat(c.stat_aes().x(1.).y(c.stat_expr('Mean')*2.))).y_axis(c.y_axis().scale(c.scale_log(10).domain(1,100))).build()
chart=plot.chart();domain=chart.semantics()['layers'][0]['domains']['y'];assert math.isclose(domain['minimum'],reference['log_after_stat_expression']['y'][0],abs_tol=1e-12)
static_request,static_frame=retain('log_after_stat_expression',plot)
presented=chart.present(output,options);old_points=marks(presented)
current_request=chart.request(output,options)
legacy=plot.edit().profile('LibraryV1').build();assert chart.apply_plot(legacy,0)
presented_options=options.basis('presented');presented_request=chart.request(output,presented_options);presented_frame=presented_request.prepare()
new_request=chart.request(output,options);new_frame=new_request.prepare()
assert marks(presented_frame)==old_points
assert marks(new_frame)!=old_points
old_frame=current_request.prepare();assert marks(old_frame)==old_points
old_bytes=old_frame.export('svg')
chart.dispose();plot.dispose();legacy.dispose();data.dispose();output.dispose()
assert old_frame.export('svg')==old_bytes
assert marks(static_frame)==old_points
(out/'captures.json').write_text(json.dumps({'old':old_points,'new':marks(new_frame),'retained_after_disposal':True},indent=2))
for item in [static_request,static_frame,presented,current_request,presented_options,presented_request,presented_frame,new_request,new_frame,old_frame,options]:item.dispose()
# Stage mismatches reject at the shared component boundary.
try:c.aes().x(c.stat_expr('Mean'))
except c.ChartError:pass
else:raise AssertionError('generated expression accepted as a source read')
(out/'results.json').write_text(json.dumps(records,indent=2))
print('PASS FIX-GG02 Python: 13 independently authored stage figures, R numeric/style expectations, stage rejection, static/Presented/Current capture and disposal.')
