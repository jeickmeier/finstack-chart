#!/usr/bin/env python3
"""Focused GG package runtime proof; build once, execute only assigned capability matrices.

This is scoped evidence, not a replacement for the cumulative primary runner.
Run with mise; WASM_BINDGEN must identify the locked 0.2.128 CLI.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from build_primary_modules import python_module, wasm_module

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('output',type=Path)
parser.add_argument('--reuse-modules',action='store_true',help='Reuse existing compiled modules for author/declaration-only corrections.')
parser.add_argument('--package',action='append',choices=['GG-06','GG-07','GG-08','GG-09','GG-12'],required=True)
args=parser.parse_args();ROOT=Path(__file__).resolve().parents[1];output=args.output.resolve();output.mkdir(parents=True,exist_ok=True)
cli=os.environ.get('WASM_BINDGEN','wasm-bindgen');node=os.environ.get('NODE','node')
assert subprocess.check_output([cli,'--version'],text=True).strip()=='wasm-bindgen 0.2.128'
env=dict(os.environ,PYO3_PYTHON=sys.executable,MYPYPATH=str(ROOT/'packages/python'))
target=Path(env.get('CARGO_TARGET_DIR',ROOT/'target')).resolve()
(output/'environment.json').write_text(json.dumps({'python':sys.version,'platform':sys.platform,'wasm_bindgen':cli,'node':subprocess.check_output([node,'--version'],text=True).strip(),'rust':subprocess.check_output(['rustc','-Vv'],text=True),'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'scope':args.package},indent=2))
def run(name,command):
 print('RUN',' '.join(map(str,command)),flush=True)
 result=subprocess.run(list(map(str,command)),cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
 (output/(name+'.log')).write_text(result.stdout);print(result.stdout[-1800:],end='',flush=True)
 if result.returncode:raise SystemExit(f'{name} failed; see {output/(name+".log")}')
if args.reuse_modules:
 import shutil
 module=output/'python-module';wasm=output/'wasm-module'
 assert (module/'chart_python.so').is_file() and (wasm/'chart_wasm_bg.wasm').is_file(), 'Build modules before reusing them.'
 for name in ['authoring.cjs','authoring.d.cts']:shutil.copy2(ROOT/'packages/wasm'/name,wasm/name)
else:
 module=python_module(ROOT,output,target,run);wasm=wasm_module(ROOT,output,target,run,cli)
scopes={'GG-06':['ggplot_bin_stat_controls','ggplot_position_controls','ggplot_count_summary'], 'GG-07':['ggplot_interval_recipes','ggplot_surface_recipes','ggplot_recipe_marks','ggplot_stroke_controls'], 'GG-08':['ggplot_text_marks'], 'GG-09':['ggplot_distribution_geometries','ggplot_univariate_controls'], 'GG-12':['ggplot_facet_controls']}
consumer=output/'typing';consumer.mkdir(exist_ok=True)
source=(ROOT/'scripts/bindings/authoring/ggplot_stats_text_types.cts').read_text().replace('../../../target/ggplot-packages/wasm-module/','../wasm-module/')
(consumer/'consumer.cts').write_text(source)
tsc=[node,env['TSC_JS']] if env.get('TSC_JS') else ['tsc']
run('typescript-stats-text',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'consumer.cts'])
run('python-stats-text-types',[sys.executable,'-m','mypy','--strict','--cache-dir',output/'mypy-cache',ROOT/'scripts/bindings/authoring/ggplot_stats_text_typing.py'])
results={}
if 'GG-07' in args.package:
 for host, command, extension, owner in [('python',[sys.executable],'py',module),('wasm',[node],'cjs',wasm)]:
  run('ggplot_recipe_host-'+host,command+[ROOT/'scripts/bindings'/('ggplot_recipe_host.'+extension),owner,output/'ggplot_recipe_host'/host])
 for record in ['records.json','count-expression.json']:
  assert json.loads((output/'ggplot_recipe_host/python'/record).read_text())==json.loads((output/'ggplot_recipe_host/wasm'/record).read_text())
for package in dict.fromkeys(args.package):
 for scope in scopes[package]:
  base=output/scope
  for host,command,extension,owner in [('python',[sys.executable],'py',module),('wasm',[node],'cjs',wasm)]:
   run(scope+'-'+host,command+[ROOT/'scripts/bindings'/(scope+'.'+extension),owner,base/host])
  run(scope+'-rust',['cargo','run','-p','chart-export','--example',scope,'--locked','--',base/'rust'])
  left=base/'python';right=base/'wasm'
  record_names=['records.json','kept-limits.json']+[p.name for p in left.glob('horizontal-*.json')]
  for name in record_names:
   if (left/name).exists():assert json.loads((left/name).read_text())==json.loads((right/name).read_text()),name
  files=[p for p in left.iterdir() if p.suffix in ('.svg','.pdf','.png')];assert files
  for path in files:
   assert path.read_bytes()==(right/path.name).read_bytes(),path
   assert path.read_bytes()==(base/'rust'/path.name).read_bytes(),path
  for path in left.glob('*.scene.json'):
   assert json.loads(path.read_text())==json.loads((right/path.name).read_text()),path
  results[scope]={'publications_per_host':len(files),'status':'passed'}
(output/'comparisons.json').write_text(json.dumps(results,indent=2)+'\n')
print('PASS focused packages:', ', '.join(dict.fromkeys(args.package)))
