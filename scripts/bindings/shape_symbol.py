"""FIX-S06 actual Python symbols, source palettes and bounded owned paths."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
corpus=json.loads((ROOT/'fixtures/shapes/symbol.json').read_text())
def near(a,b):
    if isinstance(a,(int,float)) and isinstance(b,(int,float)): assert abs(a-b)<=2e-12*max(1,abs(b)),(a,b)
    elif isinstance(a,list):
        assert len(a)==len(b)
        for a,b in zip(a,b):near(a,b)
    elif isinstance(a,dict):
        assert a.keys()==b.keys()
        for k in a:near(a[k],b[k])
    else:assert a==b
for case in corpus['cases']:
    config={'kind':case['kind'],'size':case['size']}
    if case['size']<0:
        try:c.ShapeSymbol(config);assert False
        except c.ChartError as e:assert e.code=='CHART_NUMERICAL_DOMAIN'
        continue
    g=c.ShapeSymbol(config);copy=g.copy();assert g.config()==copy.config();g.dispose()
    p=copy.generate();saved=p.copy();expected=c.Path(3).apply_batch(case['operations'])
    near(p.result()['geometry'],expected.result()['geometry']);assert p.to_svg()==case['svg']['3'],case['id']
    again=copy.generate();assert again.result()==p.result();again.dispose();copy.dispose();initial=saved.result();p.move_to(10,20);p.dispose();assert saved.result()==initial;saved.dispose();expected.dispose()
fill,stroke=c.ShapeSymbol.palettes();assert list(fill)==corpus['palettes']['fill'] and list(stroke)==corpus['palettes']['stroke']
a=c.ShapeSymbol({'kind':'X'});b=c.ShapeSymbol({'kind':'Times'});p=a.generate();q=b.generate();assert p.result()==q.result();p.dispose();q.dispose();a.dispose();b.dispose()
for config in [{'kind':'Unknown'},{'bad':0},{'digits':-1}]:
    try:c.ShapeSymbol(config);assert False
    except c.ChartError:pass
for config in [{'limits':{'max_points':0}},{'limits':{'path':{'max_operations':0,'max_commands':100,'max_svg_bytes':1000,'max_replay_commands':100,'max_subdivisions':100}}}]:
    g=c.ShapeSymbol(config)
    try:g.generate();assert False
    except c.ChartError as e:assert e.code=='CHART_RESOURCE_LIMIT'
    g.dispose()
print('PASS Python symbols: 156 reference cases, palettes, aliases, repeated/copy/disposal ownership, independent numeric paths and bounded failures.')
