"""Capture complex projection cases in independent R processes, preserving the sequential oracle."""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import json, os, subprocess, sys, tempfile
root = Path(__file__).resolve().parents[3]
original = json.loads((root / 'fixtures/parity/ggplot2/mapproj-controls.json').read_text())
cases = [c for c in original['cases'] if c['method'] in ['guyou', 'square', 'tetra', 'hex', 'eisenlohr']]
work = Path(tempfile.mkdtemp(prefix='gg15-isolated-', dir='/private/tmp'))
def capture(pair):
    index, case = pair
    source, target = work / f'{index}-in.json', work / f'{index}-out.json'
    source.write_text(json.dumps(case))
    result = subprocess.run([sys.executable, str(root / 'tools/reference/r/run.py'), str(root / 'tools/reference/r/mapproj-isolated-one.R'), str(source), str(target)], cwd=root, text=True, capture_output=True)
    if result.returncode: raise RuntimeError(result.stderr + result.stdout)
    return json.loads(target.read_text())
with ThreadPoolExecutor(max_workers=4) as executor: results = list(executor.map(capture, enumerate(cases)))
output = {'reference':original['reference'], 'isolation':'fresh R process for every call; previous sequential fixture retained to expose initializer state', 'cases':results}
(root / 'fixtures/parity/ggplot2/mapproj-isolated-controls.json').write_text(json.dumps(output, indent=2, allow_nan=False)+'\n')
print(f'Captured {len(results)} fresh-process cases; temporary artifacts {work}')
