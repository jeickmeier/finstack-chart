"""Compare FIX-GG04 host records and publications with the declared numeric tolerance."""
import hashlib
import json
import math
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

python, wasm, destination = map(Path, sys.argv[1:4])
differences = []

def compare(a, b, location):
    if isinstance(a, dict) and isinstance(b, dict):
        assert a.keys() == b.keys(), location
        for key in a:
            compare(a[key], b[key], f'{location}.{key}')
    elif isinstance(a, list) and isinstance(b, list):
        assert len(a) == len(b), location
        for i, (x, y) in enumerate(zip(a, b)):
            compare(x, y, f'{location}[{i}]')
    elif isinstance(a, (float, int)) and isinstance(b, (float, int)) and not isinstance(a, bool) and not isinstance(b, bool):
        assert a == b or math.isclose(a, b, rel_tol=3e-12, abs_tol=3e-12), (location, a, b)
        if a != b:
            differences.append({'location': location, 'python': a, 'wasm': b, 'absolute_difference': abs(a-b)})
    else:
        assert a == b, (location, a, b)

records = [json.loads((root/'position-missing-records.json').read_text()) for root in (python, wasm)]
assert len(records[0]) == len(records[1]) == 2540
compare(*records, 'records')
files = sorted(p.name for p in python.iterdir() if p.suffix in ('.svg', '.pdf', '.png'))
assert len(files) == 30
publications = {}
for name in files:
    a, b = [(root/name).read_bytes() for root in (python, wasm)]
    identical = a == b
    if not identical:
        assert name.endswith('.svg'), name
        x, y = list(ET.fromstring(a).iter()), list(ET.fromstring(b).iter())
        assert len(x) == len(y), name
        for i, (left, right) in enumerate(zip(x, y)):
            assert (left.tag, left.text, left.tail, left.attrib.keys()) == (right.tag, right.text, right.tail, right.attrib.keys()), (name, i)
            for key in left.attrib:
                first, second = left.attrib[key], right.attrib[key]
                if first != second:
                    assert key in {'x', 'y', 'x1', 'y1', 'x2', 'y2', 'cx', 'cy', 'r', 'width', 'height'}, (name, i, key)
                    compare(float(first), float(second), f'{name}[{i}].{key}')
    publications[name] = {'byte_identical': identical, 'python_sha256': hashlib.sha256(a).hexdigest(), 'wasm_sha256': hashlib.sha256(b).hexdigest()}
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_text(json.dumps({'matching_states': len(records[0]), 'relative_tolerance': 3e-12, 'absolute_tolerance': 3e-12, 'numeric_differences': differences, 'publications': publications}, indent=2))
print('PASS', len(records[0]), 'states;', sum(v['byte_identical'] for v in publications.values()), 'byte-identical publications;', len(differences), 'numeric differences within tolerance.')
