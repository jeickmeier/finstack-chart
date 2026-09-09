from pathlib import Path
import subprocess,os,shutil
root=Path('/workspace');out=root/'target/shape-cartesian-linux';out.mkdir(exist_ok=True)
env=dict(os.environ,PYO3_PYTHON='/usr/bin/python3')
def run(name,args):
 print('RUN',name,flush=True);r=subprocess.run(args,cwd=root,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(out/(name+'.log')).write_text(r.stdout);print(r.stdout[-2200:],flush=True);assert r.returncode==0,(name,r.returncode)
run('core',['cargo','test','-p','chart-core','--lib','--test','shape_cartesian','--test','shape_integration','--test','shape_foundation','--test','path_lowering','--test','inspection','--locked','--offline'])
run('python-build',['cargo','build','-p','chart-python','--features','extension-module,extension-proof','--locked','--offline'])
module=out/'python-module';module.mkdir(exist_ok=True);shutil.copy2(root/'target/debug/libchart_python.so',module/'chart_python.so')
run('python',['python3','scripts/bindings/shape_cartesian.py',str(module)])
run('interaction',['python3','scripts/bindings/shape_cartesian_interaction.py',str(module)])
run('updates',['python3','scripts/bindings/shape_cartesian_updates.py',str(module),str(out/'updates.json')])
run('gallery',['python3','scripts/bindings/shape_cartesian_gallery.py',str(module),str(out/'python')])
run('export',['cargo','run','-p','chart-export','--example','shape_cartesian','--locked','--offline','--',str(out/'rust')])
