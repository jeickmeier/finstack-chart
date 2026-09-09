"""FIX-C01 all standalone color operations through the actual Python extension."""
from pathlib import Path
import json,math,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
cases=json.loads((ROOT/'fixtures/parity/d3-color/cases.json').read_text())['cases']
def scalar(v): return {'NaN':float('nan'),'Infinity':float('inf'),'-Infinity':-float('inf'),'-0':-0.0}[v['number']] if isinstance(v,dict) else v
def snapshot(value):
    result=value.value();value.dispose();return result
def compare(actual,expected,label):
    if type(actual) in (int,float) and type(expected) in (int,float):assert math.isclose(actual,expected,rel_tol=1e-10,abs_tol=1e-10),(label,actual,expected)
    elif isinstance(expected,dict):
        assert actual.keys()==expected.keys(),(label,actual,expected)
        for k in expected:compare(actual[k],expected[k],label+'/'+k)
    elif isinstance(expected,list):
        assert len(actual)==len(expected),(label,actual,expected)
        for i,(a,e) in enumerate(zip(actual,expected)):compare(a,e,label+'/'+str(i))
    else:assert actual==expected,(label,actual,expected)
results=[]
for case in cases:
    i=case['input'];e=case['expected']
    value=c.color(i['parse']) if 'parse' in i else getattr(c,i['constructor'])(i['css']) if 'css' in i else getattr(c,i['constructor'])(*[scalar(v) for v in i['args']])
    if value is None:
        assert e is None;results.append({'id':case['id'],'result':None});continue
    result={'value':value.value(),'conversions':{space:snapshot(getattr(c,space)(value)) for space in ['rgb','hsl','lab','hcl','cubehelix']},'displayable':value.displayable(),
            'formats':{'formatHex':value.format_hex(),'formatHex8':value.format_hex8(),'formatRgb':value.format_rgb(),'formatHsl':value.format_hsl(),'toString':str(value),'hex':value.hex()},
            'copy':snapshot(value.copy({k:scalar(v) for k,v in e['copyPatch'].items()})),'copyPatch':e['copyPatch'],
            'brightness':[dict(method=op['method'],k=op['k'],value=snapshot(getattr(value,op['method'])() if op['k'] is None else getattr(value,op['method'])(scalar(op['k'])))) for op in e['brightness']],
            'clamp':snapshot(value.clamp()) if value.space() in ('Rgb','Hsl') else None,'sourceAfter':value.value()}
    compare(result,e,case['id'])
    wire=value.to_json();copied=c.ColorValue.from_json(wire);assert copied.to_json()==wire
    value.dispose();assert copied.to_json()==wire;copied.dispose()
    results.append({'id':case['id'],'result':result})
for call in [lambda:c.color('x'*4097),lambda:c.rgb(1,2),lambda:c.ColorValue.from_json('{"version":2,"value":{"space":"Rgb","channels":{"r":1,"g":2,"b":3,"opacity":1}}}')]:
    try:call()
    except (c.ChartError,ValueError):pass
    else:raise AssertionError('Expected malformed constructor/descriptor rejection')
v=c.rgb(1,2,3)
try:v.copy({'h':4.})
except c.ChartError:pass
else:raise AssertionError('wrong-space channel accepted')
assert v.channel('r')==1.;v.dispose()
try:v.format_hex()
except c.ChartError:pass
else:raise AssertionError('disposed color accepted')
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(results,indent=2)+'\n')
print(f'PASS FIX-C01 Python: {len(cases)} cases, every constructor/method, exceptional tags, copies, strict descriptors and disposal.')
