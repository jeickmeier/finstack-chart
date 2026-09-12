#!/usr/bin/env python3
"""Retain only the viridisLite seed tables absent from the shared D3 catalog."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
fixture = ROOT / 'fixtures/parity/ggplot2/palettes.json'
document = json.loads(fixture.read_text())
d3 = (ROOT / 'crates/chart-core/src/scales/chromatic/data.rs').read_text()
source = '//! Quantized viridisLite 0.4.3 seed tables (MIT); see VIRIDIS-LICENSE.\n'
shared = dict(A='Magma', B='Inferno', C='Plasma', D='Viridis')
for seed in document['viridis_seeds']:
    colors = [int(value[1:], 16) for value in seed['rgb']]
    assert len(colors) == 256
    option = seed['option']
    if option in shared:
        body = re.search(r'InterpolatorId::' + shared[option] + r' => Some\(&\[(.*?)\]\)', d3, re.S)[1]
        assert colors == [int(value, 16) for value in re.findall(r'0x([\da-fA-F]+)', body)]
        continue
    source += f'pub(super) const {option}: [u32; 256] = [\n'
    for start in range(0, 256, 8):
        source += '    ' + ', '.join(f'0x{value:06X}' for value in colors[start:start+8]) + ',\n'
    source += '];\n'
(ROOT / 'crates/chart-core/src/scales/chromatic/ggplot_data.rs').write_text(source)
files = ['tools/reference/r/palettes.R', 'tools/reference/r/palette_records.py',
         'tools/reference/r/run.py', 'tools/reference/r/renv.lock',
         'fixtures/parity/ggplot2/palettes.json', 'fixtures/parity/ggplot2/sources.json',
         'crates/chart-core/src/scales/chromatic/ggplot_data.rs',
         'crates/chart-core/src/scales/chromatic/VIRIDIS-LICENSE']
manifest = dict(owner='GG-04', reference=document['reference'], cases=len(document['cases']),
                shared_d3_seed_tables=shared, comparison='Exact RGBA bytes for every captured sample',
                files={name: hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in files})
(ROOT / 'fixtures/parity/ggplot2/palettes-manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
print(f"PASS {len(document['cases'])} palette cases, four verified shared tables, four retained viridisLite tables")
