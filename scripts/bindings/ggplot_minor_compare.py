"""Compare minor snapshot structure exactly and binary64 values at oracle tolerances."""
import json
import math
import sys
from pathlib import Path
base=Path(sys.argv[1]);a=json.loads((base/'python/records.json').read_text());b=json.loads((base/'wasm/records.json').read_text())
maximum=0.;different=0

def compare(a,b,path='root'):
    global maximum,different
    if isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),path
        for key in a:compare(a[key],b[key],path+'.'+key)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),path
        for i,(x,y) in enumerate(zip(a,b)):compare(x,y,f'{path}[{i}]')
    elif isinstance(a,(float,int)) and not isinstance(a,bool):
        assert isinstance(b,(float,int)) and not isinstance(b,bool),path
        assert math.isclose(a,b,rel_tol=1e-11,abs_tol=1e-11),(path,a,b)
        if a!=b:different+=1;maximum=max(maximum,abs(a-b))
    else:assert a==b,(path,a,b)
compare(a,b)
result={'records':len(a),'relative_tolerance':1e-11,'absolute_tolerance':1e-11,'differing_binary64_fields':different,'maximum_absolute_difference':maximum}
(base/'comparison.json').write_text(json.dumps(result,indent=2))
print('PASS',result)
