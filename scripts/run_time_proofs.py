#!/usr/bin/env python3
"""SP-06 focused Rust/Python/WASM calendar proof; uses installed pinned toolchains."""
import os,sys,subprocess,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT/'target/scale-time-proof'
out.mkdir(parents=True,exist_ok=True)
cli=os.environ.get('WASM_BINDGEN','wasm-bindgen')
assert subprocess.check_output([cli,'--version'],text=True).strip()=='wasm-bindgen 0.2.128'
env=dict(os.environ,PYO3_PYTHON=sys.executable,CHART_TIME_ARTIFACTS=str(out/'rust'))
(out/'environment.txt').write_text(f'python={sys.version}\nexecutable={sys.executable}\n'+subprocess.check_output(['rustc','-Vv'],text=True)+subprocess.check_output(['node','--version'],text=True)+subprocess.check_output([cli,'--version'],text=True))
def run(name,args):
 print('RUN',name,flush=True)
 with (out/(name+'.log')).open('w') as log:subprocess.run(list(map(str,args)),cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 print((out/(name+'.log')).read_text()[-1000:],flush=True)
run('core',['cargo','test','-p','chart-core','--test','scale_calendar','--locked'])
run('export',['cargo','test','-p','chart-export','--test','scale_calendar','--locked'])
run('python-build',['cargo','build','-p','chart-python','--features','extension-module','--locked'])
module=out/'python-module';module.mkdir(exist_ok=True)
shutil.copy2(ROOT/'target/debug'/('libchart_python.dylib' if sys.platform=='darwin' else 'libchart_python.so'),module/'chart_python.so')
run('python',[sys.executable,ROOT/'scripts/bindings/time.py',module,out/'python',out/'rust'])
run('wasm-build',['cargo','build','-p','chart-wasm','--target','wasm32-unknown-unknown','--locked'])
run('wasm-generate',[cli,ROOT/'target/wasm32-unknown-unknown/debug/chart_wasm.wasm','--target','nodejs','--out-dir',out/'wasm-module'])
for name in ['authoring.cjs','authoring.d.cts','interpolation.cjs','interpolation.d.cts','scales.cjs','scales.d.cts']:shutil.copy2(ROOT/'packages/wasm'/name,out/'wasm-module'/name)
run('wasm',['node',ROOT/'scripts/bindings/time.cjs',out/'wasm-module',out/'wasm',out/'rust'])
assert (out/'python/checks.json').read_bytes()==(out/'wasm/checks.json').read_bytes()
print('PASS SP-06 retained calendar runtime proof',out)
