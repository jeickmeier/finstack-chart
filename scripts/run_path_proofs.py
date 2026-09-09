#!/usr/bin/env python3
"""Build and execute FIX-P01–06 through actual Rust/Python/WASM; no downloads."""
import json,os,shutil,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT/'target/path-proof'
out.mkdir(parents=True,exist_ok=True)
cli=os.environ.get('WASM_BINDGEN','wasm-bindgen');node=os.environ.get('NODE','node')
version=subprocess.check_output([cli,'--version'],text=True).strip()
if version!='wasm-bindgen 0.2.128':raise SystemExit('Set WASM_BINDGEN to the matching 0.2.128 CLI.')
env=dict(os.environ,PYO3_PYTHON=sys.executable)
def run(name,args):
    print('RUN',' '.join(map(str,args)),flush=True)
    with (out/(name+'.log')).open('w') as log:subprocess.run(list(map(str,args)),cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    print((out/(name+'.log')).read_text()[-2000:],flush=True)
(out/'environment.json').write_text(json.dumps(dict(python=sys.version,platform=sys.platform,wasm_bindgen=version,node=subprocess.check_output([node,'--version'],text=True).strip(),rust=subprocess.check_output(['rustc','-Vv'],text=True),revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()),indent=2))
run('rust',['cargo','run','-p','chart-export','--example','path_binding_proof','--locked','--',out/'rust'])
run('python-build',['cargo','build','-p','chart-python','--features','extension-module','--locked'])
module=out/'python-module';module.mkdir(exist_ok=True)
library=ROOT/'target/debug'/('libchart_python.dylib' if sys.platform=='darwin' else 'libchart_python.so')
shutil.copy2(library,module/'chart_python.so')
run('python',[sys.executable,ROOT/'scripts/bindings/path.py',module,out/'rust',out/'python'])
run('wasm-build',['cargo','build','-p','chart-wasm','--target','wasm32-unknown-unknown','--locked'])
wasm=out/'wasm-module'
run('wasm-generate',[cli,ROOT/'target/wasm32-unknown-unknown/debug/chart_wasm.wasm','--target','nodejs','--out-dir',wasm])
for name in ('authoring.cjs','authoring.d.cts','interpolation.cjs','interpolation.d.cts','scales.cjs','scales.d.cts'):shutil.copy2(ROOT/'packages/wasm'/name,wasm/name)
run('wasm',[node,ROOT/'scripts/bindings/path.cjs',wasm,out/'rust',out/'wasm'])
run('compare',[sys.executable,ROOT/'scripts/bindings/path_compare.py',out])
print('PASS path runtime proof. Native and publication artifact inspection remain separately recorded.')
