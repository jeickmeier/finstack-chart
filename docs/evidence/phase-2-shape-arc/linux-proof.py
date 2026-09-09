from pathlib import Path
import subprocess,os,shutil
root=Path('/workspace');out=root/'target/shape-arc-linux';out.mkdir(exist_ok=True);env=dict(os.environ,PYO3_PYTHON='/usr/bin/python3')
def run(name,args):
 print('RUN',name,flush=True);p=subprocess.run(args,cwd=root,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(out/(name+'.log')).write_text(p.stdout);print(p.stdout[-2500:],flush=True);assert p.returncode==0,(name,p.returncode)
run('core',['cargo','test','-p','chart-core','--test','shape_arc_pie','--test','shape_arc_integration','--test','shape_integration','--locked','--offline'])
run('python-build',['cargo','build','-p','chart-python','--features','extension-module,extension-proof','--locked','--offline'])
module=out/'python-module';module.mkdir(exist_ok=True);shutil.copy2(root/'target/debug/libchart_python.so',module/'chart_python.so')
run('python',['python3','scripts/bindings/shape_arc_pie.py',str(module)])

run('interaction',['python3','scripts/bindings/shape_arc_interaction.py',str(module)])
run('gallery',['python3','scripts/bindings/shape_arc_gallery.py',str(module),str(out/'python')])
run('updates',['python3','scripts/bindings/shape_arc_updates.py',str(module),str(out/'python-updates.json')])
run('export',['cargo','run','-p','chart-export','--example','shape_arc','--locked','--offline','--',str(out/'rust')])
