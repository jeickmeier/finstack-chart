from pathlib import Path
import os,subprocess,shutil,sys
root=Path.cwd();out=root/'target/chromatic-final-hosts';out.mkdir(exist_ok=True);module=out/'python-module';module.mkdir(exist_ok=True);
if '--wasm-only' not in sys.argv:shutil.copy2(root/'target/debug/libchart_python.dylib',module/'chart_python.so')
wasm=out/'wasm-module'
def run(name,args):
 print('RUN',name,flush=True);p=subprocess.run(list(map(str,args)),text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(out/(name+'.log')).write_text(p.stdout);print(p.stdout[-1800:],flush=True);assert p.returncode==0,(name,p.returncode)
run('wasm-generate',['/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen',root/'target/path-proof/linux-target/wasm32-unknown-unknown/debug/chart_wasm.wasm','--target','nodejs','--out-dir',wasm])
for name in ('authoring.cjs','authoring.d.cts','interpolation.cjs','interpolation.d.cts','scales.cjs','scales.d.cts','examples.cjs','examples.d.cts'):shutil.copy2(root/'packages/wasm'/name,wasm/name)
for lang,cmd,mod,suffix in [('python',[sys.executable],module,'py'),('wasm',['node'],wasm,'cjs')]:
 if lang=='python' and '--wasm-only' in sys.argv:continue
 for name in ['color','interpolate','scales','chromatic','chromatic_composition','chromatic_transformed','chromatic_gallery','chromatic_updates']:
  run(name+'-'+lang,cmd+[root/'scripts/bindings'/f'{name}.{suffix}',mod,out/('publication/'+lang if name=='chromatic_gallery' else name+'-'+lang+'.json')]+(['--public'] if name=='scales' else []))
run('chromatic-memory',['node','--expose-gc',root/'scripts/bindings/chromatic_memory.cjs',wasm,out/'chromatic-memory.json'])
shutil.copytree(root/'target/path-proof/linux-target/chromatic-linux-primary/rust',out/'publication/rust',dirs_exist_ok=True)
print('PASS final source: selected actual host runtimes, all color/interpolation/scale/chromatic matrices, scenes, ownership and updates.',flush=True)
