from pathlib import Path
import subprocess,os,shutil
root=Path('/workspace');out=root/'target/chromatic-linux-primary';out.mkdir(exist_ok=True)
env=dict(os.environ,PYO3_PYTHON='/usr/bin/python3')
def run(name,args):
    print('RUN',name,flush=True)
    r=subprocess.run(args,cwd=root,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (out/(name+'.log')).write_text(r.stdout);print(r.stdout[-2000:],flush=True);assert r.returncode==0,(name,r.returncode)
run('core',['cargo','test','-p','chart-core','--lib','--test','chromatic','--test','chromatic_integration','--test','color_parity','--test','interpolate_color','--test','reference_inventory','--locked','--offline'])
run('python-build',['cargo','build','-p','chart-python','--features','extension-module,extension-proof','--locked','--offline'])
module=out/'python-module';module.mkdir(exist_ok=True);shutil.copy2(root/'target/debug/libchart_python.so',module/'chart_python.so')
for name in ['color','interpolate','chromatic','chromatic_composition','chromatic_transformed','chromatic_gallery','chromatic_updates']:
    run(name,['python3',str(root/'scripts/bindings'/f'{name}.py'),str(module),str(out/(name if name=='chromatic_gallery' else name+'.json'))])
run('export',['cargo','run','-p','chart-export','--example','chromatic_proof','--locked','--offline','--',str(out/'rust')])
run('wasm-build',['cargo','build','-p','chart-wasm','--features','extension-proof','--target','wasm32-unknown-unknown','--locked','--offline'])
run('benchmark',['cargo','run','-p','chart-core','--example','chromatic_benchmark','--release','--locked','--offline'])
