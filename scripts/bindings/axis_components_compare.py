"""Shared publication artifact comparison across three real hosts."""
from pathlib import Path
import hashlib,json,math,sys
root=Path(sys.argv[1]);native=root/'rust';result=[]
def same(a,b,path=''):
 if isinstance(a,dict):
  assert isinstance(b,dict) and a.keys()==b.keys(),path
  for k in a:same(a[k],b[k],path+'/'+k)
 elif isinstance(a,list):
  assert isinstance(b,list) and len(a)==len(b),path
  for i,(x,y) in enumerate(zip(a,b)):same(x,y,path+f'/{i}')
 elif isinstance(a,float):assert isinstance(b,(int,float)) and math.isclose(a,b,rel_tol=0.,abs_tol=1e-9),(path,a,b)
 else:assert a==b,(path,a,b)
for source in sorted(native.iterdir()):
 if source.suffix not in ['.json','.svg','.pdf','.png']:continue
 for host in ['python','wasm']:
  target=root/host/source.name
  if source.suffix=='.json':same(json.loads(source.read_text()),json.loads(target.read_text()),host+'/'+source.name)
  else:assert source.read_bytes()==target.read_bytes(),(host,source.name,'publication bytes differ')
 result.append({'file':source.name,'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'verdict':'PASS'})
variants=['start','mid','end','interrupt-start','interrupt-mid','interrupt-end'] if len(sys.argv)>2 and sys.argv[2]=='transitions' else [300,600]
if len(sys.argv)>2 and sys.argv[2]=='hierarchy':
 for source in native.glob('*-text.png'):assert source.read_bytes()==source.with_name(source.name.replace('-text.png','-outline.png')).read_bytes()
else:
 for variant in variants:assert (native/f'text-{variant}.png').read_bytes()==(native/f'outline-{variant}.png').read_bytes()
(root/'host-comparison.json').write_text(json.dumps({'results':result,'json_geometry_absolute_tolerance':1e-9,'semantic_labels_values_roles_and_ids':'exact','publications':'byte-identical Rust/Python/WASM; text/outline PNG identical'},indent=2))
print(f'PASS {len(result)} three-host artifacts: semantic JSON and byte-identical SVG/PDF/PNG.')
