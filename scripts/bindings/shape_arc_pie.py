"""FIX-S04 actual Python arc/pie constants, ownership and materialized source identity."""
from pathlib import Path
import sys,json,math,re
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
corpus=json.loads((ROOT/'fixtures/shapes/arc-pie.json').read_text())
def snake(k):return re.sub(r'(?<!^)(?=[A-Z])','_',k).lower()
def near(a,b):
    if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert abs(a-b)<=2e-12*max(1,abs(b)),(a,b)
    elif isinstance(a,list):
        assert len(a)==len(b)
        for x,y in zip(a,b):near(x,y)
    elif isinstance(a,dict):
        assert a.keys()==b.keys()
        for k in a:near(a[k],b[k])
    else:assert a==b,(a,b)
negative=0
for case in corpus['arcs']:
    config={snake(k):v for k,v in case['config'].items()};datum={snake(k):v for k,v in case['input'].items()}
    arc=c.ShapeArc(config);copy=arc.copy();assert arc.config()==copy.config();arc.dispose()
    if not case['finite']:
        try:copy.generate(datum);assert False,case['id']
        except c.ChartError as error:assert error.code=='CHART_NUMERICAL_DOMAIN'
        copy.dispose();negative+=1;continue
    path=copy.generate(datum);retained=path.copy();expected=c.Path(3).apply_batch(case['operations'])
    near(path.result()['geometry']['commands'],expected.result()['geometry']['commands']);near(copy.centroid(datum),case['centroid']);assert path.to_svg()==case['svg']['3'],case['id']
    again=copy.generate(datum);assert again.result()==path.result();again.dispose();copy.dispose();original=retained.result();path.move_to(998,997);path.dispose();assert retained.result()==original;retained.dispose();expected.dispose()
count=0
for case in corpus['pies']:
    if case['order']=='dataDescending':continue # native comparator fixture; portable custom protocols belong to WP-S07.
    pie=c.ShapePie({'order':{'default':'ValuesDescending','none':'Input','valuesAscending':'ValuesAscending'}[case['order']],'angles':{'start_angle':case['start'],'end_angle':case['end'],'pad_angle':case['pad']}})
    owner=pie.copy();pie.dispose();data=case['data'];values=[d['value'] for d in data];result=owner.layout(data,values);again=owner.layout(data,values);owner.dispose();assert result==again
    expected=[{snake(k):v for k,v in arc.items()} for arc in case['result']];near(result,expected)
    if data:assert result[0]['data']['id']==data[0]['id'];result[0]['data']['label']='mutated';assert again[0]['data']==data[0]
    count+=1
assert count==144 and negative==1
constant=c.ShapeArc({'inner_radius':0,'outer_radius':10,'start_angle':0,'end_angle':math.pi/2});p=constant.generate();assert p.to_svg()=='M0,-10A10,10,0,0,1,10,0L0,0Z';p.dispose();constant.dispose()
assert c.ShapePie({'value':2}).layout([4,7])[0]['value']==2
for Type,config in [(c.ShapeArc,{'wrong':1}),(c.ShapePie,{'order':'Unknown'})]:
    try:Type(config);assert False
    except c.ChartError:pass
for run in [lambda:c.ShapeArc().generate(),lambda:c.ShapePie().layout(['a'],[]),lambda:c.ShapePie({'limits':{'max_points':0}}).layout([1]),lambda:c.ShapeArc({'limits':{'max_points':0}}).centroid()]:
    try:run();assert False
    except c.ChartError:pass
print('PASS Python arc/pie: 620 arc cases (619 finite, one checked overflow), 144 portable pie layouts, exact metadata, constants, copies, disposal and independent paths. 48 native comparator fixtures remain Rust-only until WP-S07.')
