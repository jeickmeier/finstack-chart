"""FIX-GG06: actual host count/summary authoring, owned fields and portable replay."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(5):
    d=c.Data.columns({'x':[1.,1.,2.,2.,4.,4.],'y':[1.,3.,4.,8.,2.,6.],'w':[2.,-1.,0.,3.,0.,-2.],'g':['A','A','A','A','B','B']})
    if mode==0: stat=c.count().ggplot_count().x('x').group('g').count_weight(d.field('w'))
    else:
        stat=c.summary().x('x').y('y').group('g').summary_helper({'MeanSe':{'mult':1.}})
        if mode in [2,3]:stat=stat.summary_bins({'breaks':[0.,2.,5.],'bins':30,'options':{'closed':'Right' if mode==2 else 'Left'}})
        if mode==4:stat=stat.summary_helper({'MeanClBootSeeded':{}})
    p=c.plot(d).layer(c.points().stat(stat)).x_axis(c.x_axis().scale(c.scale_linear().domain(-1.,6.))).y_axis(c.y_axis().scale(c.scale_linear().domain(-4.,12.))).build();wire=p.to_json();assert json.loads(wire)['version']==72
    restored=c.Plot.from_json(wire);d.dispose()
    scenes=[]
    for candidate in [p,restored]:
        request=output.request(candidate,c.export_options(600.,360.));frame=request.prepare();scenes.append(frame.scene())
        if candidate is restored:
            for fmt in ['svg','pdf','png']:(out/f'summary-{mode}.{fmt}').write_bytes(frame.export(fmt))
        frame.dispose();request.dispose()
    assert scenes[0]==scenes[1]
    (out/f'summary-{mode}.scene.json').write_text(json.dumps(scenes[0]));(out/f'summary-{mode}.plot.json').write_text(wire)
    p.dispose();restored.dispose()
output.dispose()
expected=json.loads((ROOT/'fixtures/parity/ggplot2/count-summary-controls.json').read_text())['extra']['kept_limits']
d=c.Data.columns({'x':[1.,2.,10.]})
p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(c.histogram().stat(c.bin().bins(3).ggplot_bin({}))).x_axis(c.x_axis().numeric_limits([0.,5.]).oob('Keep')).build()
q=c.Plot.from_json(p.to_json());records=[]
for current in [p,q]:
    chart=current.chart();rows=chart.semantics()['layers'][0]['rows']['Binned'];assert len(rows)==len(expected)
    record=[]
    for row,want in zip(rows,expected):
        actual={'xmin':row['start'],'xmax':row['end'],'count':row['statistics']['count']}
        for key,value in actual.items():assert abs(value-want[key])<1e-12,(key,value,want[key])
        record.append(actual)
    records.append(record);chart.dispose()
assert records[0]==records[1]
(out/'kept-limits.json').write_text(json.dumps(records));q.dispose();p.dispose();d.dispose()
print('PASS explicit limits with OOB Keep: original/replay numeric R sentinel.')
print('PASS Python five count/summary authors, original/replay scene equality and 15 publications.')

reference=json.loads((ROOT/'fixtures/parity/ggplot2/count-summary-controls.json').read_text())['extra']
for mode in ['fixed','free_x','free_y','overlay']:
    d=c.Data.columns({'y':[1.,2.]} if mode=='overlay' else {'y':[0.,1.,100.,101.],'g':['A','A','B','B']})
    builder=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().y('y')).layer(c.histogram().orientation('Horizontal').stat(c.bin().x('y').bins(3).ggplot_bin({})))
    overlay=None
    if mode=='overlay':
        overlay=c.Data.columns({'x':[1.,1.],'y':[0.,10.]},name='overlay')
        builder=builder.layer(c.points().data(overlay).aes(c.aes().x('x').y('y')))
    else:builder=builder.facet(c.facet_wrap('g').free_x(mode=='free_x').free_y(mode=='free_y'))
    p=builder.build();q=c.Plot.from_json(p.to_json());records=[]
    for current in [p,q]:
        chart=current.chart();semantic=chart.semantics();panels=semantic['panels'] or [semantic]
        rows=[row for panel in panels for row in panel['layers'][0]['rows']['Binned']]
        expected=reference['horizontal_'+mode];assert len(rows)==len(expected)
        record=[]
        for row,want in zip(rows,expected):
            actual={'ymin':row['start'],'ymax':row['end'],'count':row['count']}
            for key,value in actual.items():assert abs(float(value)-want[key])<1e-12,(mode,key,value,want[key])
            record.append(actual)
        records.append(record);chart.dispose()
    assert records[0]==records[1]
    (out/('horizontal-'+mode+'.json')).write_text(json.dumps(records));q.dispose();p.dispose();d.dispose()
    if overlay is not None:overlay.dispose()
print('PASS horizontal physical-axis training: fixed/free_x/free_y facets and mixed-orientation overlay, original/replay R sentinels.')
