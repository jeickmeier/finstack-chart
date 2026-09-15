"""Compare actual palette states and publication bytes without host tolerances."""
import hashlib,json,sys
from pathlib import Path
py,wasm,out=map(Path,sys.argv[1:4]);out.parent.mkdir(parents=True,exist_ok=True)
native=Path(sys.argv[6]) if len(sys.argv)>6 else None
a=json.loads((py/'records.json').read_text());b=json.loads((wasm/'records.json').read_text())
expected=int(sys.argv[4]) if len(sys.argv)>4 else 1063
assert len(a)==len(b)==expected
for index,(left,right) in enumerate(zip(a,b)):assert left==right,(index,left,right)
files={}
for p in sorted(wasm.iterdir()):
 if p.suffix in ('.svg','.pdf','.png'):
  content=p.read_bytes()
  if native is not None:assert (native/p.name).read_bytes()==content,('native',p.name)
  assert (py/p.name).read_bytes()==content,p.name;files[p.name]=hashlib.sha256(content).hexdigest()
expected_files=int(sys.argv[5]) if len(sys.argv)>5 else 27
assert len(files)==expected_files,len(files)
out.write_text(json.dumps({'states':len(a),'comparison':'exact parsed JSON equality','publication_files':files,**({'native_publication_comparison':'byte-identical'} if native is not None else {})},indent=2)+'\n')
print(f'PASS: {len(a)} exactly equal Python/WASM states and {len(files)} byte-identical publication files.')
