"""FIX-S07 actual Python stack kernels, source ownership and exceptional endpoints."""
from pathlib import Path
import sys,json,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
from finstack_chart._interpolation import _un_number
corpus=json.loads((ROOT/'fixtures/shapes/stack.json').read_text());assert len(corpus['cases'])==435
for case in corpus['cases']:
    config={k:case[k] for k in ['keys','order','offset','missing']};generator=c.ShapeStack(config);copy=generator.copy();generator.dispose();data=[dict(id=str(9007199254741001+2*i),values=values) for i,values in enumerate(case['matrix'])];actual=copy.layout(data,case['matrix']);again=copy.layout(data,case['matrix']);copy.dispose()
    for result in [actual,again]:
        assert len(result)==len(case['series'])
        for got,expected in zip(result,case['series']):
            assert got['key']==expected['key'] and got['index']==expected['index'] and len(got['points'])==len(expected['points'])
            for point,want in zip(got['points'],expected['points']):
                assert point['data']==want['data']
                for name in ['y0','y1']:
                    a,b=point[name],_un_number(want[name]);assert (math.isnan(a)and math.isnan(b))or(abs(a-b)<=2e-12*max(1,abs(b))and(a!=0 or math.copysign(1,a)==math.copysign(1,b))),(case['id'],name,a,b)
    if data:
        data[0]['id']='changed';assert all(not s['points']or s['points'][0]['data']['id']=='9007199254741001' for s in actual)
    try:copy.layout(case['matrix']);assert False
    except c.ChartError:pass
for config in [{'keys':['a','b'],'order':{'Explicit':[0,0]}},{'keys':['a'],'value':float('inf')},{'keys':['a'],'limits':{'max_series':0}}]:
    try:c.ShapeStack(config);assert False
    except (c.ChartError,ValueError):pass
for config,values in [({'keys':['a'],'missing':'Error'},[[None]]),({'keys':['a']},[[1.,2.]]),({'keys':['a','b'],'offset':'Expand'},[[sys.float_info.max,sys.float_info.max]]),({'keys':['a','b'],'offset':'Wiggle','limits':{'max_work':3}},[[1.,1.]])]:
    g=c.ShapeStack(config)
    try:g.layout(values);assert False
    except c.ChartError:pass
    g.dispose()
g=c.ShapeStack({'keys':['a'],'value':7.});result=g.layout([{'id':9007199254741001,'number':'NaN'}],[[None]]);g.dispose();assert result[0]['points'][0]['data']=={'id':9007199254741001,'number':'NaN'} and result[0]['points'][0]['y1']==7.
print('PASS Python stack: 435 independent order/offset/missing cases, exact keys/ranks/source metadata, signed zero/NaN, copies/repeated/disposal independence and bounded diagnostics.')
