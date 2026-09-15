"""Exact cross-host proof for discrete minor callbacks and binned exclusion."""
import json
import sys
from pathlib import Path
python,wasm,output=map(Path,sys.argv[1:4])
a=json.loads((python/'records.json').read_text());b=json.loads((wasm/'records.json').read_text())
assert len(a)==len(b)==1520
assert a==b
files=sorted(p for p in wasm.iterdir() if p.suffix in ('.svg','.pdf','.png'))
assert len(files)==12 and all(p.read_bytes()==(python/p.name).read_bytes() for p in files)
output.parent.mkdir(parents=True,exist_ok=True)
output.write_text(json.dumps({'exact_records':len(a),'exact_publications':[p.name for p in files]},indent=2))
print('PASS 1520 exact states; 12 byte-equal publications.')
