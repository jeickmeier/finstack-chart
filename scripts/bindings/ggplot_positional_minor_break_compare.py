"""Compare minor callback hosts; only raw inverse-log values admit oracle tolerance."""
import json
import math
import sys
from pathlib import Path
python, wasm, output = map(Path, sys.argv[1:4])
a=json.loads((python/'records.json').read_text());b=json.loads((wasm/'records.json').read_text())
assert len(a)==len(b)==(2898 if "--joint" in sys.argv else 5756)
records=a
rounding=[]
major_rounding=[]
joint="--joint" in sys.argv
if joint:cases=json.loads((Path(__file__).resolve().parents[2]/'fixtures/parity/ggplot2/positional-minor-break-joint-functions.json').read_text())['cases']
def compare(a,b,path=''):
    if isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),path
        for key,value in a.items():compare(value,b[key],path+'/'+key)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),path
        for i,(x,y) in enumerate(zip(a,b)):compare(x,y,path+'/'+str(i))
    elif a!=b:
        major = joint and '/ticks/' in path and path.endswith('/value/Number')
        if major:
            record=records[int(path.split('/')[1])]
            case=cases[record['index']]
            assert case['transform']=='log10' and case['major']=='function_domain',(path,case)
            assert abs(a-b)<=max(math.ulp(a),math.ulp(b)),(path,a,b)
            major_rounding.append(abs(a-b))
        else:
            assert '/minor_ticks/' in path and path.endswith('/value/Number'),(path,a,b)
        assert isinstance(a,(int,float)) and isinstance(b,(int,float)),path
        assert abs(a-b)<=3e-12*max(1.,abs(b)),(path,a,b)
        if not major:rounding.append(abs(a-b))
compare(a,b)
files=sorted(p for p in wasm.iterdir() if p.suffix in ('.svg','.pdf','.png'))
assert len(files)==12 and all(p.read_bytes()==(python/p.name).read_bytes() for p in files)
output.parent.mkdir(parents=True,exist_ok=True)
output.write_text(json.dumps({'records':len(a),'major_value_rounding_count':len(major_rounding),'minor_value_rounding_count':len(rounding),'maximum_absolute_rounding':max(rounding+major_rounding,default=0),'exact_publications':[p.name for p in files]},indent=2))
print('PASS',len(a),'states; all labels/positions exact;',len(rounding),'raw minor and',len(major_rounding),'raw major inverse-log ULP differences; 12 byte-equal publications.')
