"""FIX-S02/03 actual Python Cartesian ownership, defaults, masks and all curve kernels."""
from pathlib import Path
import json,sys,math
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
cases=json.loads((ROOT/'fixtures/shapes/cartesian.json').read_text())['cases']
for seed in json.loads((ROOT/'fixtures/shapes/cases.json').read_text())['cases']:
    if seed['family'] not in ('line','area'):continue
    settings=dict(seed['settings']);curve={'kind':settings.pop('curve')[5:]}
    for key in ['alpha','beta','tension']:
        if key in settings:curve[key]=settings.pop(key)
    cases.append(dict(id=seed['id'],family=seed['family'],input=seed['input'],config=dict(settings,curve=curve),helper=None,finite=True,operations=seed['operations'],svg=seed['svg_digits']['3']))
def compare(a,b):
    if isinstance(a,(int,float)) and isinstance(b,(int,float)):
        assert abs(a-b)<=2e-12*max(1,abs(b)),(a,b)
    elif isinstance(a,list):
        assert len(a)==len(b)
        for x,y in zip(a,b):compare(x,y)
    elif isinstance(a,dict):
        assert a.keys()==b.keys()
        for k in a:compare(a[k],b[k])
    else:assert a==b
negative=0
for case in cases:
    generator=(c.ShapeLine if case['family']=='line' else c.ShapeArea)(case['config'])
    if case['helper']:
        old=generator;generator=old.boundary(case['helper']);old.dispose()
    copy=generator.copy();generator.dispose()
    if not case['finite']:
        try:copy.generate(case['input']);assert False,case['id']
        except c.ChartError as error:assert error.code=='CHART_NUMERICAL_DOMAIN'
        negative+=1;copy.dispose();continue
    p=copy.generate(case['input']);original=p.result();saved=p.copy()
    expected=c.Path(3).apply_batch(case['operations']);compare(original['geometry']['commands'],expected.result()['geometry']['commands'])
    assert p.to_svg()==(case['svg'] or ''),case['id']
    again=copy.generate(case['input']);assert again.result()==original
    copy.dispose();p.move_to(999,888);p.dispose();assert saved.result()==original
    again.dispose();saved.dispose();expected.dispose()
for Type,config in [(c.ShapeArea,{'curve':{'kind':'Bundle'}}),(c.ShapeLine,{'curve':{'kind':'Basis','tension':1}}),(c.ShapeLine,{'x':{'Column':-1}})]:
    try:Type(config);assert False,config
    except c.ChartError:pass
print('PASS Python Cartesian:',len(cases),'cases including',negative,'nonfinite-reference diagnostics; actual generator, boundary, copy, repeated output and independent Path ownership.')
