"""GG-03 numerical host comparison and byte-exact independently authored publication."""
import hashlib
import json
import math
import sys
from pathlib import Path
rust, python, wasm, output = map(Path, sys.argv[1:])
def same(a,b,path=''):
    if isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),path
        for k in a:same(a[k],b[k],path+'/'+k)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),path
        for i,(x,y) in enumerate(zip(a,b)):same(x,y,path+'/'+str(i))
    elif isinstance(a,(int,float)) and not isinstance(a,bool):
        assert isinstance(b,(int,float)) and math.isclose(a,b,rel_tol=2e-12,abs_tol=2e-12),(path,a,b)
    else:assert a==b,(path,a,b)
for file,count in [('symbols.json',104),('updates.json',8)]:
    a=json.loads((python/file).read_text());b=json.loads((wasm/file).read_text());assert len(a)==count;same(a,b,file)
records=[]
for name,hosts in [('glyphs',[rust,python,wasm]),('independent',[python,wasm])]:
    for fmt in ['svg','png']:
        content=(hosts[0]/f'{name}.{fmt}').read_bytes()
        for host in hosts[1:]:assert content==(host/f'{name}.{fmt}').read_bytes(),(name,fmt,host)
        records.append({'file':f'{name}.{fmt}','sha256':hashlib.sha256(content).hexdigest(),'hosts':len(hosts),'verdict':'PASS'})
output.write_text(json.dumps({'glyph_records':104,'update_states':8,'numeric_tolerance':{'absolute':2e-12,'relative':2e-12},'publications':records},indent=2)+'\n')
print('PASS GG-03: 104 Python/WASM glyph records, 8 update states, byte-identical three-host glyph SVG/PNG and two-host independent paint SVG/PNG.')
