"""Compare actual palette states and publication bytes without host tolerances."""
import hashlib,json,re,sys
from pathlib import Path
py,wasm,out=map(Path,sys.argv[1:4]);out.parent.mkdir(parents=True,exist_ok=True)
native=Path(sys.argv[6]) if len(sys.argv)>6 else None
def publication_identity(content):
 # Primary authors allocate process-local guide IDs. Preserve their equality and
 # ordering while comparing every other SVG byte, including logical tick values.
 ids={}
 return re.sub(rb'data-guide-id="([0-9]+)"',lambda m:b'data-guide-id="'+str(ids.setdefault(m[1],len(ids))).encode()+b'"',content)
a=json.loads((py/'records.json').read_text());b=json.loads((wasm/'records.json').read_text())
expected=int(sys.argv[4]) if len(sys.argv)>4 else 1063
assert len(a)==len(b)==expected
for index,(left,right) in enumerate(zip(a,b)):assert left==right,(index,left,right)
files={}
for p in sorted(wasm.iterdir()):
 if p.suffix in ('.svg','.pdf','.png'):
  content=p.read_bytes()
  if native is not None:
   other=(native/p.name).read_bytes()
   assert (publication_identity(other)==publication_identity(content) if p.suffix=='.svg' else other==content),('native',p.name)
  assert (py/p.name).read_bytes()==content,p.name;files[p.name]=hashlib.sha256(content).hexdigest()
expected_files=int(sys.argv[5]) if len(sys.argv)>5 else 27
assert len(files)==expected_files,len(files)
out.write_text(json.dumps({'states':len(a),'comparison':'exact parsed JSON equality','publication_files':files,**({'native_publication_comparison':'PNG/PDF byte-identical; SVG exact after bijective process-local guide-ID normalization'} if native is not None else {})},indent=2)+'\n')
print(f'PASS: {len(a)} exactly equal Python/WASM states and {len(files)} byte-identical publication files.')
