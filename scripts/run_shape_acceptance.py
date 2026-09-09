#!/usr/bin/env python3
"""Execute FIX-S01-09 against an already-built real Python or Node WASM module.

Usage: python3 scripts/run_shape_acceptance.py python|wasm MODULE OUTPUT [DASH_GALLERY_SOURCE]
Builds and visual/native inspection are separate acceptance evidence. This runner
uses the committed oracle; it never regenerates reference expectations.
"""
from pathlib import Path
import os,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
host,module,out=sys.argv[1:4];module=Path(module).resolve();out=Path(out).resolve();out.mkdir(parents=True,exist_ok=True)
if host not in ('python','wasm'):raise SystemExit('Expected python or wasm')
command=[sys.executable] if host=='python'else[os.environ.get('NODE','node')];extension='py'if host=='python'else'cjs'
def run(name,args=()):
    argv=command+[str(ROOT/'scripts/bindings'/f'{name}.{extension}'),str(module)]+list(map(str,args))
    print('RUN',' '.join(argv),flush=True)
    with (out/f'{name}.log').open('w')as log:result=subprocess.run(argv,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
    if result.returncode:raise SystemExit(f'{name} failed ({result.returncode}); see {out/name}.log')
    print('PASS',name,flush=True)
run('shape_foundation',[out/'foundation'])
for name in ['shape_cartesian','shape_arc_pie','shape_symbol','shape_stack','shape_radial','shape_custom','shape_dashes','shape_projection_budget']:run(name)
for family in ['cartesian','arc','symbol','stack','radial','custom']:
    if family!='custom':run(f'shape_{family}_interaction')
    run(f'shape_{family}_updates',[out/f'{family}-updates.json'])
if len(sys.argv)>4:run('shape_dashes_gallery',[Path(sys.argv[4]).resolve(),out/'dashes'])
print(f'PASS FIX-S01-09 actual {host} shape corpus, interactions, updates and retained ownership; visual/native qualification is separate.',flush=True)
