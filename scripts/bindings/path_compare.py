"""Compare path topology/state/numbers and scene metadata across actual runtimes."""
import json, math, re, sys
from pathlib import Path
root=Path(sys.argv[1])
def same(a,b,where='root'):
    if isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),where
        for key in a:same(a[key],b[key],where+'.'+key)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),where
        for i,(x,y) in enumerate(zip(a,b)):same(x,y,f'{where}[{i}]')
    elif isinstance(a,(int,float)) and not isinstance(a,bool):
        assert isinstance(b,(int,float)) and abs(a-b)<=2e-12*max(1,abs(a)),(where,a,b)
    elif where.endswith('.svg'):
        tokenize=lambda s:re.findall(r'[A-DF-Za-df-z]|[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?',s)
        x,y=tokenize(a),tokenize(b);assert len(x)==len(y),where
        for i,(x,y) in enumerate(zip(x,y)):
            if x.isalpha():assert x==y,(where,x,y)
            else:same(float(x),float(y),where+f'.number{i}')
    else:assert a==b,(where,a,b)
reference=json.loads((root/'rust/paths.json').read_text());assert len(reference)==86
for host in ('python','wasm'):
    same(reference,json.loads((root/host/'paths.json').read_text()))
    original=json.loads((root/'rust/figure.plot.json').read_text())
    rebuilt=json.loads((root/host/'figure.plot.json').read_text())
    # Cross-platform libm may differ by ULPs. Revisions track exact definitions;
    # same-host reconstruction must be a no-op (asserted by both runtime scripts).
    original_definition=original['definition'].copy();rebuilt_definition=rebuilt['definition'].copy()
    original_definition.pop('revision');rebuilt_definition.pop('revision')
    changed=original_definition!=rebuilt_definition
    expected=str(int(original['definition']['revision'])+int(changed))
    assert rebuilt['definition']['revision']==expected
    same(original_definition,rebuilt_definition)
    for dpi in (300,600):
        a=json.loads((root/'rust'/f'figure-{dpi}.scene.json').read_text())
        b=json.loads((root/host/f'figure-{dpi}.scene.json').read_text())
        assert b['stamp']['definition']==expected
        assert a['stamp']['definition']==original['definition']['revision']
        same(a['stamp'] | {'definition':expected},b['stamp'])
        same({k:v for k,v in a.items() if k!='stamp'},{k:v for k,v in b.items() if k!='stamp'})
print('PASS FIX-P01–06: 86 full operation traces and all retained scene geometry/targets across Rust, Python and WASM.')
