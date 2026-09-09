"""FIX-S07/09 actual tidy chart stacks against independently pinned endpoints."""
from pathlib import Path
import sys,json,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
from finstack_chart._interpolation import _un_number
cases=json.loads((ROOT/'fixtures/shapes/stack.json').read_text())['cases'];key=9007199254741001
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(400.,200.).dpi(72).layout(c.layout_options().padding(0))
def near(a,b):
    if isinstance(a,(float,int)) and isinstance(b,(float,int)):assert abs(a-b)<=2e-12*max(1.,abs(b)),(a,b)
    elif isinstance(a,list):assert len(a)==len(b),(a,b);[near(x,y)for x,y in zip(a,b)]
    elif isinstance(a,dict):assert a.keys()==b.keys(),(a,b);[near(a[k],b[k])for k in a]
    else:assert a==b,(a,b)
checked=0;precision_diagnostics=0
for case in cases:
 n=len(case['keys'])
 if not n:continue
 rows=[(i,g,v)for i,row in enumerate(case['matrix'])for g,v in enumerate(row)if v is not None][::-1]
 d=c.Data.columns({'x':c.column([float(r[0])for r in rows],kind='float64'),'y':c.column([float(r[2])for r in rows],kind='float64'),'g':c.column([r[1]for r in rows],kind='int64')},keys=[key+r[0]*n+r[1]for r in rows])
 for area in [False,True]:
    layer=(c.shape_area()if area else c.bars()).position(c.shape_stack(list(range(n))).stack_order(case['order']).stack_offset(case['offset']).stack_missing(case['missing']))
    p=c.plot(d).aes(c.aes().x('x').x2('x').y('y').y2(0.).group('g')).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).build();wire=p.to_json();assert json.loads(wire)['version']==7
    q=c.Plot.from_json(wire);q.dispose();request=output.request(p,options);p.dispose()
    try:frame=request.prepare()
    except c.ChartError as error:
        assert error.code=='CHART_PRECISION_LOSS'and any(abs(_un_number(p[k]))>1e30 for s in case['series']for p in s['points']for k in ['y0','y1'])
        precision_diagnostics+=1;request.dispose();continue
    scene=frame.scene();observed=[]
    for item,targets in zip(scene['items'],scene['targets']):
        if not targets:continue
        ids=[int(t['Source']['key'])for t in targets];observed+=ids;g=(ids[0]-key)%n;sample=(ids[0]-key)//n
        if area:
            shape=item['primitive']['ShapePath'];points=case['series'][g]['points'];runs=[];run=[]
            for i,point in enumerate(points):
                y1=_un_number(point['y1'])
                if math.isnan(y1):
                    if run:runs.append(run);run=[]
                else:run.append(i)
            if run:runs.append(run)
            run=next(r for r in runs if sample in r);expected_ids=[key+i*n+g for i in run if case['matrix'][i][g]is not None];assert ids==expected_ids
            upper=[[i*100.,200.-_un_number(points[i]['y1'])*50.]for i in run];lower=[[i*100.,200.-_un_number(points[i]['y0'])*50.]for i in reversed(run)]
            commands=[{'MoveTo':upper[0]}]+[{'LineTo':p}for p in upper[1:]+lower]+['Close'];near(shape['geometry']['commands'],commands)
            near([[p['x'],p['y']]for p in shape['anchors']],[[i*100.,200.-_un_number(points[i]['y1'])*50.]for i in run if case['matrix'][i][g]is not None])
        else:
            point=case['series'][g]['points'][sample];a=200.-_un_number(point['y0'])*50.;b=200.-_un_number(point['y1'])*50.
            if a==b:near(item['primitive']['Rule']['from']['y'],a)
            else:bounds=item['primitive']['Rectangle']['bounds'];near(bounds['origin']['y'],min(a,b));near(bounds['height'],abs(a-b))
    assert sorted(observed)==sorted(key+i*n+g for i,g,_ in rows),(case['id'],area);checked+=1;frame.dispose();request.dispose()
 d.dispose()
assert checked+precision_diagnostics==810
# A zero-filled absent cell contributes path geometry but no focusable observation.
d=c.Data.columns({'x':[0.,1.,2.,0.,2.],'y':[1.,1.,1.,2.,2.],'g':c.column([1,1,1,0,0],kind='int64')},keys=[key+i for i in range(5)])
p=c.plot(d).aes(c.aes().x('x').x2('x').y('y').y2(0.).group('g')).layer(c.shape_area().position(c.shape_stack([0,1]).stack_missing('Zero'))).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).build();chart=p.chart();p.dispose();f=chart.present(output,options);assert sum(len(x)for x in f.scene()['targets'])==5
hits=chart.inspect(25.,150.,mode='Containment')['targets'];assert hits and all(int(t['identity']['Source']['key'])in [key+i for i in range(5)]for t in hits);chart.focus(hits[0]);chart.present(output,options).dispose();assert not chart.inspect(-1.,100.,mode='Containment')['targets'];f.dispose();chart.dispose()
# Generated statistics retain their aggregate membership through stacked bars.
d=c.Data.columns({'group':['a','b','b']})
layer=c.bars().stat(c.count().group('group')).after_stat(c.stat_aes().x(2.).y('Count').y2(0.)).position(c.shape_stack(['a','b']))
p=c.plot(d).layer(layer).build();request=output.request(p,options);f=request.prepare();targets=[t for ts in f.scene()['targets']for t in ts if 'Aggregate'in t];assert [len(t['Aggregate']['members'])for t in targets]==[1,2];f.dispose();request.dispose();p.dispose()
print(f'PASS Python stack: {checked} reference chart comparisons plus {precision_diagnostics} explicit off-scale publication-precision diagnostics, exact sparse path commands/source anchors, signed bar extents, reversed input, wire roundtrip, clipping/focus without synthetic targets and generated aggregate membership.')
