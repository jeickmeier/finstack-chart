#!/usr/bin/env python3
"""Build and execute FIX-AUTH07 primary authors, independent fixtures and host type checks.

Run under mise. WASM_BINDGEN must match the locked crate (0.2.128). TSC_JS may
name an installed TypeScript compiler; otherwise tsc must be on PATH. Python
must have mypy available (a task-local PYTHONPATH is supported). No downloads.
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
output=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT/'target/authoring'
output.mkdir(parents=True,exist_ok=True)
node=os.environ.get('NODE','node');cli=os.environ.get('WASM_BINDGEN','wasm-bindgen')
tsc=[node,os.environ['TSC_JS']] if os.environ.get('TSC_JS') else ['tsc']
version=subprocess.check_output([cli,'--version'],text=True).strip()
if version!='wasm-bindgen 0.2.128':raise SystemExit('Require wasm-bindgen-cli 0.2.128; set WASM_BINDGEN to an installed matching executable.')
env=dict(os.environ,PYO3_PYTHON=sys.executable,MYPYPATH=str(ROOT/'packages/python'))

def run(name,command,expected=0):
    print('RUN',' '.join(map(str,command)),flush=True)
    result=subprocess.run(list(map(str,command)),cwd=ROOT,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (output/(name+'.log')).write_text(result.stdout)
    print(result.stdout[-1800:],end='',flush=True)
    if result.returncode!=expected:raise SystemExit(f'{name} exited {result.returncode}; expected {expected}. See {output/name}.log')
    return result.stdout

run('typescript-version',tsc+['--version'])
run('mypy-version',[sys.executable,'-m','mypy','--version'])
(output/'environment.json').write_text(json.dumps({'python':sys.version,'executable':sys.executable,'platform':sys.platform,'wasm_bindgen':version,'node':subprocess.check_output([node,'--version'],text=True).strip(),'rust':subprocess.check_output(['rustc','-Vv'],text=True),'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()},indent=2))
run('rust-primary',['cargo','run','-p','chart-export','--example','primary_binding_proof','--locked','--',output/'native'])
run('python-primary-build',['cargo','build','-p','chart-python','--features','extension-module,extension-proof','--locked'])
module=output/'python-module';module.mkdir(exist_ok=True)
lib=ROOT/'target/debug'/('libchart_python.dylib' if sys.platform=='darwin' else 'libchart_python.so')
shutil.copy2(lib,module/'chart_python.so')
run('python-primary',[sys.executable,ROOT/'scripts/bindings/authoring/python_proof.py',module,output/'python'])
run('wasm-primary-build',['cargo','build','-p','chart-wasm','--features','extension-proof','--target','wasm32-unknown-unknown','--locked'])
wasm=output/'wasm-module'
run('wasm-primary-generate',[cli,ROOT/'target/wasm32-unknown-unknown/debug/chart_wasm.wasm','--target','nodejs','--out-dir',wasm])
for file in ('authoring.cjs','authoring.d.cts','examples.cjs','examples.d.cts'):shutil.copy2(ROOT/'packages/wasm'/file,wasm/file)
run('wasm-primary',[node,'--expose-gc',ROOT/'scripts/bindings/authoring/wasm_proof.cjs',wasm,output/'wasm'])
run('primary-compare',[sys.executable,ROOT/'scripts/bindings/authoring/compare.py',output])
consumer=output/'typing';consumer.mkdir(exist_ok=True)
src=(ROOT/'scripts/bindings/authoring/types.cts').read_text().replace('../../../target/authoring/wasm-module/','../wasm-module/')
(consumer/'consumer.cts').write_text(src)
run('typescript-primary',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'consumer.cts'])
base=[sys.executable,'-m','mypy','--strict','--cache-dir',output/'mypy-cache']
run('python-primary-types',base+[ROOT/'scripts/bindings/authoring/typing_valid.py'])
invalid=run('python-primary-types-invalid',base+[ROOT/'scripts/bindings/authoring/typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid
print(f'PASS FIX-AUTH07 primary runtime and type proofs: {output}. Artifact inspection remains a separate recorded check.')
