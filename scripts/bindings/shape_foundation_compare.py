"""FIX-S01 identical independently authored shapes through all three runtime scenes."""
from pathlib import Path
import json,sys
root=Path(sys.argv[1])
for host in ['python','wasm']:
    for dpi in [300,600]:
        name=f'figure-{dpi}'
        assert json.loads((root/'rust'/f'{name}.scene.json').read_text())==json.loads((root/host/f'{name}.scene.json').read_text()),(host,dpi,'scene')
        assert (root/'rust'/f'{name}.png').read_bytes()==(root/host/f'{name}.png').read_bytes(),(host,dpi,'png')
print('PASS FIX-S01: identical full scenes and PNG bytes across Rust/Python/WASM at 300/600 DPI. Native/SVG/PDF inspection is separate.')
