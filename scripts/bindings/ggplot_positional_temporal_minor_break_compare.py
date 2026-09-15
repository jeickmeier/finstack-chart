"""Cross-host temporal minor callback records and publication proof."""
import json
import sys
from pathlib import Path
python,wasm,output=map(Path,sys.argv[1:4])
overrides='--overrides' in sys.argv[4:]
# JSON numeric scale values are binary64 on both hosts; timestamp integers are strings.
# ECMAScript may print a large integral double without exponent notation.
a=json.loads((python/'records.json').read_text(),parse_int=float);b=json.loads((wasm/'records.json').read_text(),parse_int=float)
assert len(a)==len(b)==(480 if overrides else 30545)
assert a==b
files=sorted(p for p in wasm.iterdir() if p.suffix in ('.svg','.pdf','.png'))
assert len(files)==12 and all(p.read_bytes()==(python/p.name).read_bytes() for p in files)
output.parent.mkdir(parents=True,exist_ok=True)
output.write_text(json.dumps({'exact_records':len(a),'numeric_comparison':'Exact binary64 JSON numbers; timestamp integer strings remain exact','exact_publications':[p.name for p in files]},indent=2))
print(f'PASS {len(a)} exact states; 12 byte-equal publications.')
