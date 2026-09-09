"""CP-04 exact catalog/ramp proof through public Python owners."""
from pathlib import Path
import json,math,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
corpus=json.loads((ROOT/'fixtures/parity/d3-scale-chromatic/cases.json').read_text())
catalog=c.chromatic_catalog();assert catalog['version']==1 and len(catalog['schemes'])==len(catalog['interpolators'])==38
catalog['schemes'][0]['sizes'].append(999);assert 999 not in c.chromatic_catalog()['schemes'][0]['sizes']
def color_hex(v):
    try:return v.format_hex8()
    finally:v.dispose()
def expected(v):return '#'+''.join(f'{x:02x}' for x in v)
def reject(call):
    try:call()
    except (c.ChartError,ValueError,TypeError,OverflowError):pass
    else:raise AssertionError('Expected rejection')
for row in corpus['schemes']:
    colors=c.chromatic_scheme(row['name'],row['size']);assert list(map(color_hex,colors))==list(map(expected,row['rgba']))
    assert list(map(color_hex,c.chromatic_scheme(row['name'],row['size'],True)))==list(map(expected,reversed(row['rgba'])))
    reject(lambda:c.chromatic_scheme(row['name'],0))
n=0
for row in corpus['ramps']:
    ramp=c.chromatic(row['name']);wire=ramp.to_json();copy=c.Interpolator.from_json(wire)
    reverse=c.chromatic(row['name'],True)
    reject(lambda:ramp.sample(None))
    for count in [0,1,200001]:reject(lambda:ramp.quantize(count))
    anchors={sample[0]:sample[2] for sample in row['samples'] if type(sample[0]) in (int,float) and sample[0] in (0.,.5,1.)}
    assert list(map(color_hex,ramp.quantize(3)))==[expected(anchors[t]) for t in [0.,.5,1.]]
    for t,css,rgba in row['samples']:
        n+=1
        t={'NaN':math.nan,'Infinity':math.inf,'-Infinity':-math.inf,'-0':-0.}.get(t.get('number'),0.) if isinstance(t,dict) else t
        if not math.isfinite(t) or rgba is None:reject(lambda:ramp.sample(t));continue
        actual=color_hex(ramp.sample(t));assert actual==expected(rgba),(row['name'],t,actual,rgba)
    for t in [0.,.123456789,.5,1.]:assert color_hex(reverse.sample(t))==color_hex(copy.sample(1-t))
    ramp.dispose();assert copy.to_json()==wire;reject(lambda:ramp.sample(.5));copy.dispose();reverse.dispose()
assert n==160666
for call in [lambda:c.chromatic('bad'),lambda:c.chromatic_scheme('Viridis'),lambda:c.chromatic_scheme('Blues'),lambda:c.chromatic_scheme('Category10',10),lambda:c.chromatic_scheme('Blues',3.5)]:reject(call)
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'schemes':218,'samples':n,'catalog':c.chromatic_catalog(),'exact':'sRGB8 RGBA','ownership':'copies, disposal, reversal and isolated results'},indent=2)+'\n')
print(f'PASS CP-04 Python: 218 exact scheme arrays and {n} ramp rows, catalog metadata, reversal, invalid inputs, copies and disposal.')
