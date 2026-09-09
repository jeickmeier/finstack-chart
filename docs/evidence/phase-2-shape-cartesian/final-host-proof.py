from pathlib import Path
import os,subprocess,sys,shutil
root=Path.cwd();out=root/'target/shape-cartesian-final-hosts';out.mkdir(exist_ok=True);modules=root/'target/shape-cartesian-primary';env=dict(os.environ,MYPYPATH=str(root/'packages/python'))
def run(name,args,expected=0):
 print('RUN',name,flush=True);p=subprocess.run(list(map(str,args)),env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(out/(name+'.log')).write_text(p.stdout);print(p.stdout[-1500:],flush=True);assert p.returncode==expected,(name,p.returncode)
for host,ext,cmd,owner in [('python','py',[sys.executable],modules/'python-module'),('wasm','cjs',['node'],modules/'wasm-module')]:
 for name,args in [('shape_cartesian',[]),('shape_cartesian_gallery',[out/host]),('shape_cartesian_updates',[out/(host+'-updates.json')]),('shape_cartesian_interaction',[])]:run(host+'-'+name,cmd+[root/'scripts/bindings'/(name+'.'+ext),owner]+args)
shutil.copytree(root/'target/path-proof/linux-target/shape-cartesian-linux/rust',out/'rust',dirs_exist_ok=True)
run('compare',[sys.executable,'scripts/bindings/shape_cartesian_compare.py',out])
consumer=out/'typing';consumer.mkdir(exist_ok=True);(consumer/'shape.cts').write_text((root/'scripts/bindings/authoring/shape_types.cts').read_text().replace('../../../packages/wasm/','../../shape-cartesian-primary/wasm-module/'))
run('typescript',['node',os.environ['TSC_JS'],'--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape.cts'])
base=[sys.executable,'-m','mypy','--strict','--cache-dir',out/'mypy-cache'];run('python-types',base+['scripts/bindings/authoring/shape_typing.py']);run('python-types-invalid',base+['scripts/bindings/authoring/shape_typing_invalid.py'],1)
assert 'Found 5 errors in 1 file' in (out/'python-types-invalid.log').read_text()
print('PASS final compiled macOS Python and WASM: 829 cases per host, interaction, 64 updates each, galleries and types.',flush=True)
