#!/usr/bin/env python3
"""FIX-I01 actual public Python constructors, samples, ownership and pinned transforms."""
import json,math,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(Path(sys.argv[1]).resolve()),str(ROOT/'packages/python')]
import finstack_chart as c
fixture=json.loads((ROOT/'fixtures/parity/d3-interpolate/cases.json').read_text())
transforms=json.loads((ROOT/'fixtures/parity/d3-interpolate/transforms.json').read_text())
def scalar(v):
    if isinstance(v,dict):return {'NaN':float('nan'),'Infinity':float('inf'),'-Infinity':-float('inf'),'-0':-0.0}[v['number']]
    return v
def tagged(v):
    if math.isnan(v):return {'number':'NaN'}
    if math.isinf(v):return {'number':'Infinity' if v>0 else '-Infinity'}
    if v==0 and math.copysign(1,v)<0:return {'number':'-0'}
    return v
def decode(v,colors):
    kind=v['kind'];value=v.get('value')
    if kind=='Missing':return c.MISSING
    if kind=='Null':return None
    if kind in ('Boolean','Text'):return value
    if kind=='Number':return scalar(value)
    if kind=='Date':return c.date_value(scalar(value))
    if kind=='Color':
        result=c.ColorValue.from_json(json.dumps({'version':1,'value':value}));colors.append(result);return result
    if kind=='NumericArray':return c.numeric_array(value['element'],map(scalar,value['values']))
    if kind=='Array':return [decode(v,colors) for v in value]
    return {k:decode(v,colors) for k,v in value.items()}
def encode(v):
    if isinstance(v,c.MissingValue):return {'kind':'Missing'}
    if v is None:return {'kind':'Null'}
    if isinstance(v,bool):return {'kind':'Boolean','value':v}
    if isinstance(v,(int,float)):return {'kind':'Number','value':tagged(v)}
    if isinstance(v,str):return {'kind':'Text','value':v}
    if isinstance(v,c.InterpolationDate):return {'kind':'Date','value':tagged(v.milliseconds)}
    if isinstance(v,c.NumericArray):return {'kind':'NumericArray','value':{'element':v.kind,'values':list(map(tagged,v.values))}}
    if isinstance(v,c.ColorValue):return {'kind':'Color','value':v.value()}
    if isinstance(v,list):return {'kind':'Array','value':list(map(encode,v))}
    return {'kind':'Record','value':{k:encode(v) for k,v in v.items()}}
def compare(a,e,path,exact=False):
    if isinstance(e,(int,float)) and not isinstance(e,bool):
        assert isinstance(a,(int,float)) and not isinstance(a,bool),(path,a,e)
        assert a==e if exact else abs(a-e)<=1e-12+1e-12*abs(e),(path,a,e)
    elif isinstance(e,list):
        assert isinstance(a,list) and len(a)==len(e),(path,a,e)
        for i,(a,e) in enumerate(zip(a,e)):compare(a,e,f'{path}/{i}',exact)
    elif isinstance(e,dict):
        assert isinstance(a,dict) and set(a)==set(e),(path,a,e)
        exact=exact or e.get('kind')=='Date' or e.get('element') not in (None,'Float64Array')
        for key in e:compare(a[key],e[key],path+'/'+key,exact)
    else:assert a==e,(path,a,e)
def name(op):return re.sub(r'(?<!^)(?=[A-Z])','_',op).lower()
def construct(case,owned):
    args=[decode(v,owned) for v in case['args']];op=case['op'];options=case['config']
    if op=='quantize':return getattr(c,name(options.get('factory','interpolateNumber')))(*args)
    if op=='piecewise':return c.piecewise(getattr(c,name(options['factory'])),args[0]) if 'factory' in options else c.piecewise(args[0])
    factory=getattr(c,name(op))
    if 'gamma' in options:factory=factory.gamma(scalar(options['gamma']))
    if 'rho' in options:factory=factory.rho(scalar(options['rho']))
    return factory(*args)
results=[]
for case in fixture['cases']:
    owned=[];f=None
    try:
        f=construct(case,owned)
        if case['op']=='quantize':
            actual=encode(f.quantize(case['config']['count']))
            assert 'adaptation' not in case,case['id'];compare(actual,case['expected'],case['id']);results.append({'id':case['id'],'samples':actual});continue
        assert 'adaptation' not in case,case['id']
        samples=[]
        for t,expected in zip(case['times'],case['expected']):
            actual=f.sample_value(t);compare(actual,expected,case['id']);compare(encode(f.sample(t)),expected,case['id']+'/public');samples.append(actual)
        if 'duration' in case:compare(f.duration,case['duration'],case['id']+'/duration');assert f.scheduling_duration==abs(f.duration)
        restored=c.Interpolator.from_json(f.to_json());copied=f.copy();held=f.sample_value(.25);f.dispose();f=None
        compare(restored.sample_value(.25),held,case['id']+'/restored');compare(copied.sample_value(.25),held,case['id']+'/copy');restored.dispose();copied.dispose()
        results.append({'id':case['id'],'samples':samples,'duration':case.get('duration')})
    except (c.ChartError,TypeError,ValueError,OverflowError) as error:
        assert 'adaptation' in case,(case['id'],error)
        results.append({'id':case['id'],'adaptation':case['adaptation']})
    finally:
        if f is not None:f.dispose()
        for color in owned:color.dispose()

def canonical(a,e,path):
    pattern=r'([-+]?(?:\d+\.?\d*|\.?\d+)(?:[eE][-+]?\d+)?)'
    assert re.sub(pattern,'#',a)==re.sub(pattern,'#',e),(path,a,e)
    compare([float(v) for v in re.findall(pattern,a)],[float(v) for v in re.findall(pattern,e)],path)
motion=[]
for case in transforms['cases']:
    factory=c.interpolate_transform_css if case['syntax']=='css' else c.interpolate_transform_svg
    try:raw=factory(case['a'],case['b'])
    except c.ChartError:
        assert 'adaptation' in case,case['id'];motion.append({'id':case['id'],'adaptation':case['adaptation']});continue
    assert 'adaptation' not in case,case['id']
    resolved=factory(*case['matrices']);samples=[]
    for t,e in zip(case['times'],case['expected']):
        canonical(resolved.sample(t),e['text'],case['id'])
        for f in (raw,resolved):
            m=f.sample_transform(t)
            for a,b in zip(m,e['matrix']):assert abs(a-b)<=1e-5+1e-6*abs(b),(case['id'],a,b)
        samples.append({'text':resolved.sample(t),'matrix':resolved.sample_transform(t)})
    raw.dispose();resolved.dispose();motion.append({'id':case['id'],'samples':samples})

f=c.interpolate_array([0,[0]],[10,[20]])
held=f(.25);samples=f.quantize(3);samples[0][1][0]=999
assert held==[2.5,[5.]] and samples[-1]==[10.,[20.]]
copy=f.copy();f.dispose();assert copy(.5)==[5.,[10.]];copy.dispose()
try:f.sample(.5);raise AssertionError('Disposed operation accepted')
except c.ChartError:pass
for invalid in ['{"version":2,"spec":{"operation":"Discrete","values":[{"kind":"Null"}]}}','{"version":1,"spec":{"operation":"Unknown"}}','{"version":1,"extra":1,"spec":{"operation":"Discrete","values":[{"kind":"Null"}]}}']:
    try:c.Interpolator.from_json(invalid);raise AssertionError('Malformed descriptor accepted')
    except c.ChartError:pass
f=c.interpolate_number(0,1)
try:f.sample(float('nan'));raise AssertionError('Nonfinite t accepted')
except c.ChartError:pass
try:c.quantize(f,1);raise AssertionError('Invalid sample count accepted')
except c.ChartError:pass
f.dispose()
try:c.piecewise(lambda a,b:None,[0,1]);raise AssertionError('Unregistered callback accepted')
except TypeError:pass
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'cases':results,'transforms':motion},indent=2,allow_nan=False)+'\n')
print(f'PASS FIX-I01 Python: {len(results)} value/configuration cases, {len(motion)} browser transform cases, owned samples, descriptors and disposal.')
