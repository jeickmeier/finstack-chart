"""Compare binned positional hosts, allowing only inverse-log tick-value rounding."""
import html
import json
import re
import sys
from pathlib import Path

python, wasm, output = map(Path, sys.argv[1:4])
a = json.loads((python / 'records.json').read_text())
b = json.loads((wasm / 'records.json').read_text())
assert len(a) == len(b) == (2744 if "--joint" in sys.argv[4:] else 14620)
rounding = []

def compare(a, b, path=''):
    if isinstance(a, dict):
        assert isinstance(b, dict) and a.keys() == b.keys(), path
        for key, value in a.items():
            compare(value, b[key], path + '/' + key)
    elif isinstance(a, list):
        assert isinstance(b, list) and len(a) == len(b), path
        for index, (x, y) in enumerate(zip(a, b)):
            compare(x, y, path + '/' + str(index))
    elif a != b:
        assert path.endswith('/value/Number'), (path, a, b)
        assert isinstance(a, (int, float)) and isinstance(b, (int, float))
        # Forward/inverse log coordinates use host transcendental implementations.
        assert abs(a-b) <= 3e-12 * max(1., abs(b)), (path, a, b)
        rounding.append(abs(a-b))

compare(a, b)
artifacts = sorted(f for f in wasm.iterdir() if f.suffix in ('.png', '.pdf', '.svg'))
assert len(artifacts) == 12
exact, metadata_rounding = [], []
for file in artifacts:
    before, after = (python / file.name).read_bytes(), file.read_bytes()
    if before == after:
        exact.append(file.name)
        continue
    assert file.suffix == '.svg', file
    before, after = before.decode(), after.decode()
    pattern = r'data-value="([^"]*)"'
    left, right = re.findall(pattern, before), re.findall(pattern, after)
    assert len(left) == len(right)
    for x, y in zip(left, right):
        compare({'value': json.loads(html.unescape(x))}, {'value': json.loads(html.unescape(y))})
    assert re.sub(pattern, '', before) == re.sub(pattern, '', after)
    metadata_rounding.append(file.name)
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps({'records': len(a), 'tick_value_rounding_count': len(rounding),
    'maximum_absolute_rounding': max(rounding, default=0), 'exact_publications': exact,
    'svg_metadata_rounding_only': metadata_rounding}, indent=2))
print('PASS', len(a), 'host states;', len(exact), 'byte-equal publications;',
      len(metadata_rounding), 'SVG metadata-only inverse-log differences')
