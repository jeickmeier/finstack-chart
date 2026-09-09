"""FIX-S05 actual Python extension, independent radial/link corpus and ownership."""
import json,sys,math,re
from pathlib import Path
root=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(Path(sys.argv[1]).resolve()),str(root/'packages/python')]
import finstack_chart as c
corpus=json.loads((root/'fixtures/shapes/radial.json').read_text())
def compare(a,b):
    if isinstance(a,(int,float)) and not isinstance(a,bool) and isinstance(b,(int,float)):
        assert math.isfinite(a) and abs(a-b)<=2e-12*max(1,abs(b)),(a,b)
    elif isinstance(a,(tuple,list)):
        assert len(a)==len(b)
        for x,y in zip(a,b):compare(x,y)
    elif isinstance(a,dict):
        assert a.keys()==b.keys()
        for k in a:compare(a[k],b[k])
    else:assert a==b,(a,b)
for p in corpus['points']:compare(c.point_radial(p['angle'],p['radius']),p['point'])
types=dict(lineRadial=c.ShapeLineRadial,areaRadial=c.ShapeAreaRadial,link=c.ShapeLink,linkRadial=c.ShapeLinkRadial)
negative=unrounded=0
for test in corpus['cases']:
    generator=types[test['family']](test['config'])
    decoded=types[test['family']](generator.config());assert decoded.config()==generator.config();decoded.dispose()
    if test['helper']:
        old=generator;generator=old.boundary(test['helper']);old.dispose()
    copy=generator.copy();generator.dispose()
    if not test['finite']:
        try:copy.generate(test['input'])
        except c.ChartError as e:assert e.code=='CHART_NUMERICAL_DOMAIN'
        else:raise AssertionError(test['id'])
        negative+=1;copy.dispose();continue
    p=copy.generate(test['input']);original=p.result();saved=p.copy();expected=c.Path(3);expected.apply_batch(test['operations']);compare(original['geometry']['commands'],expected.result()['geometry']['commands'])
    if 'digits' in test['config'] and test['config']['digits'] is None and not test['helper']:
        numbers=lambda s:[float(v) for v in re.findall(r'[-+]?(?:\d+\.?\d*|\.\d+)(?:e[-+]?\d+)?',s,re.I)]
        compare(numbers(p.to_svg()),numbers(test['svg'] or ''));unrounded+=1
    else:assert p.to_svg()==(test['svg'] or ''),test['id']
    again=copy.generate(test['input']);assert again.result()==original;copy.dispose();p.move_to(999,888);p.dispose();assert saved.result()==original;again.dispose();saved.dispose();expected.dispose()
for Type,config in [(c.ShapeAreaRadial,{'curve':{'kind':'Bundle'}}),(c.ShapeLineRadial,{'x':{'Column':0}}),(c.ShapeLinkRadial,{'curve':{'kind':'Linear'}}),(c.ShapeLink,{'source':'Node'})]:
    try:Type(config)
    except c.ChartError:pass
    else:raise AssertionError(config)
for a,r in [(float('nan'),1),(0,float('inf'))]:
    try:c.point_radial(a,r)
    except c.ChartError:pass
    else:raise AssertionError((a,r))
bounded=c.ShapeLinkRadial({'limits':{'max_points':1}})
try:bounded.generate({'source':[0,1],'target':[2,3]})
except c.ChartError:pass
else:raise AssertionError('max_points')
bounded.dispose()
print('PASS Python radial:',len(corpus['points']),'points;',len(corpus['cases']),'paths;',unrounded,'unrounded numerical comparisons;',negative,'nonfinite diagnostics; copy, disposal and retained output.')
