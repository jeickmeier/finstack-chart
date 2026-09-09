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
target=Path(env.get('CARGO_TARGET_DIR',ROOT/'target')).resolve()

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
lib=target/'debug'/('libchart_python.dylib' if sys.platform=='darwin' else 'libchart_python.so')
shutil.copy2(lib,module/'chart_python.so')
run('python-primary',[sys.executable,ROOT/'scripts/bindings/authoring/python_proof.py',module,output/'python'])
run('wasm-primary-build',['cargo','build','-p','chart-wasm','--features','extension-proof','--target','wasm32-unknown-unknown','--locked'])
wasm=output/'wasm-module'
run('wasm-primary-generate',[cli,target/'wasm32-unknown-unknown/debug/chart_wasm.wasm','--target','nodejs','--out-dir',wasm])
for file in ('authoring.cjs','authoring.d.cts','interpolation.cjs','interpolation.d.cts','scales.cjs','scales.d.cts','examples.cjs','examples.d.cts'):shutil.copy2(ROOT/'packages/wasm'/file,wasm/file)
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
# GG-02 uses independently authored figures in each host and the pinned R oracle.
stages=output/'stages'
run('stages-rust',['cargo','run','-p','chart-export','--example','ggplot_stage_proof','--locked','--',stages/'rust'])
run('stages-python',[sys.executable,ROOT/'scripts/bindings/ggplot_stages.py',module,stages/'python'])
run('stages-wasm',[node,ROOT/'scripts/bindings/ggplot_stages.cjs',wasm,stages/'wasm'])
src=(ROOT/'scripts/bindings/authoring/stages_types.cts').read_text().replace('../../../target/ggplot-stages/primary/wasm-module/','../wasm-module/')
(consumer/'stages.cts').write_text(src)
run('typescript-stages',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'stages.cts'])
run('python-stages-types',base+[ROOT/'scripts/bindings/authoring/stages_typing.py'])
invalid=run('python-stages-types-invalid',base+[ROOT/'scripts/bindings/authoring/stages_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid
# CLR standalone values execute all pinned constructor/method cases through real hosts.
run('color-python',[sys.executable,ROOT/'scripts/bindings/color.py',module,output/'color-python.json'])
run('color-wasm',[node,ROOT/'scripts/bindings/color.cjs',wasm,output/'color-wasm.json'])
src=(ROOT/'scripts/bindings/authoring/color_types.cts').read_text().replace('../../../target/color-proof/wasm-module/','../wasm-module/')
(consumer/'color.cts').write_text(src)
run('typescript-color',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'color.cts'])
run('python-color-types',base+[ROOT/'scripts/bindings/authoring/color_typing.py'])
invalid=run('python-color-types-invalid',base+[ROOT/'scripts/bindings/authoring/color_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid
# All standalone interpolation families share the same primary owned host facade.
run('interpolate-python',[sys.executable,ROOT/'scripts/bindings/interpolate.py',module,output/'interpolate-python.json'])
run('interpolate-wasm',[node,ROOT/'scripts/bindings/interpolate.cjs',wasm,output/'interpolate-wasm.json'])
src=(ROOT/'scripts/bindings/authoring/interpolate_types.cts').read_text().replace('../../../target/interpolate-proof/wasm-module/','../wasm-module/')
(consumer/'interpolate.cts').write_text(src)
run('typescript-interpolate',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'interpolate.cts'])
run('python-interpolate-types',base+[ROOT/'scripts/bindings/authoring/interpolate_typing.py'])
invalid=run('python-interpolate-types-invalid',base+[ROOT/'scripts/bindings/authoring/interpolate_typing_invalid.py'],expected=1)
assert 'Found 6 errors in 1 file' in invalid
# CLR-04 shared authored paint and retained publication inputs.
paint=output/'paint'
run('paint-rust',['cargo','run','-p','chart-export','--example','paint_proof','--locked','--',paint/'rust'])
run('paint-python',[sys.executable,ROOT/'scripts/bindings/paint.py',module,paint/'python'])
run('paint-wasm',[node,ROOT/'scripts/bindings/paint.cjs',wasm,paint/'wasm'])
# CLR-05 perceptual ramps, alpha backgrounds and grayscale publication.
color_acceptance=output/'color-acceptance'
run('color-acceptance-rust',['cargo','run','-p','chart-export','--example','color_acceptance_proof','--locked','--',color_acceptance/'rust'])
run('color-acceptance-python',[sys.executable,ROOT/'scripts/bindings/color_acceptance.py',module,color_acceptance/'python'])
run('color-acceptance-wasm',[node,ROOT/'scripts/bindings/color_acceptance.cjs',wasm,color_acceptance/'wasm'])
# SP-07 complete standalone operations plus independent chart integration.
scales=output/'scales'
run('scales-core',['cargo','test','-p','chart-core','--test','scale_operations','--test','scale_population','--test','scale_calendar','--test','scale_numeric','--locked'])
run('scales-rust',['cargo','run','-p','chart-export','--example','scale_proof','--locked','--',scales/'rust'])
run('scales-python',[sys.executable,ROOT/'scripts/bindings/scales.py',module,scales/'python-scales.json'])
run('scales-wasm',[node,ROOT/'scripts/bindings/scales.cjs',wasm,scales/'wasm-scales.json'])
assert json.loads((scales/'python-scales.json').read_text())==json.loads((scales/'wasm-scales.json').read_text())
# Replay the full calendar corpus through the public owned scale facade.
env['CHART_TIME_ARTIFACTS']=str(scales/'time-rust')
run('scales-time-rust',['cargo','test','-p','chart-export','--test','scale_calendar','--locked'])
env.pop('CHART_TIME_ARTIFACTS')
run('scales-time-python',[sys.executable,ROOT/'scripts/bindings/time.py',module,scales/'time-python',scales/'time-rust','--public'])
run('scales-time-wasm',[node,ROOT/'scripts/bindings/time.cjs',wasm,scales/'time-wasm',scales/'time-rust','--public'])
run('scales-gallery-python',[sys.executable,ROOT/'scripts/bindings/scale_gallery.py',module,scales/'python'])
run('scales-gallery-wasm',[node,ROOT/'scripts/bindings/scale_gallery.cjs',wasm,scales/'wasm'])
run('scales-interaction-python',[sys.executable,ROOT/'scripts/bindings/scale_interaction.py',module,scales/'python-interaction.json'])
run('scales-interaction-wasm',[node,ROOT/'scripts/bindings/scale_interaction.cjs',wasm,scales/'wasm-interaction.json'])
assert json.loads((scales/'python-interaction.json').read_text())==json.loads((scales/'wasm-interaction.json').read_text())
run('scales-updates-python',[sys.executable,ROOT/'scripts/bindings/scale_updates.py',module,scales/'python-updates.json'])
run('scales-updates-wasm',[node,ROOT/'scripts/bindings/scale_updates.cjs',wasm,scales/'wasm-updates.json'])
assert json.loads((scales/'python-updates.json').read_text())==json.loads((scales/'wasm-updates.json').read_text())
run('scales-memory',[node,'--expose-gc',ROOT/'scripts/bindings/scale_memory.cjs',wasm,scales/'memory.json'])
src=(ROOT/'scripts/bindings/authoring/scale_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'scales.cts').write_text(src)
run('typescript-scales',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'scales.cts'])
run('python-scales-types',base+[ROOT/'scripts/bindings/authoring/scale_typing.py'])
invalid=run('python-scales-types-invalid',base+[ROOT/'scripts/bindings/authoring/scale_typing_invalid.py'],expected=1)
assert 'Found 6 errors in 1 file' in invalid
print(f'PASS FIX-AUTH07 primary runtime and type proofs: {output}. Artifact inspection remains a separate recorded check.')

# CP-04 complete named catalog and ramp corpus through public owned APIs.
run('chromatic-core',['cargo','test','-p','chart-core','--test','chromatic','--test','chromatic_integration','--locked'])
run('chromatic-python',[sys.executable,ROOT/'scripts/bindings/chromatic.py',module,output/'chromatic-python.json'])
run('chromatic-wasm',[node,ROOT/'scripts/bindings/chromatic.cjs',wasm,output/'chromatic-wasm.json'])
chromatic=output/'chromatic'
run('chromatic-rust',['cargo','run','-p','chart-export','--example','chromatic_proof','--locked','--',chromatic/'rust'])
run('chromatic-gallery-python',[sys.executable,ROOT/'scripts/bindings/chromatic_gallery.py',module,chromatic/'python'])
run('chromatic-gallery-wasm',[node,ROOT/'scripts/bindings/chromatic_gallery.cjs',wasm,chromatic/'wasm'])
run('chromatic-updates-python',[sys.executable,ROOT/'scripts/bindings/chromatic_updates.py',module,output/'chromatic-updates-python.json'])
run('chromatic-updates-wasm',[node,ROOT/'scripts/bindings/chromatic_updates.cjs',wasm,output/'chromatic-updates-wasm.json'])
run('chromatic-memory-wasm',[node,'--expose-gc',ROOT/'scripts/bindings/chromatic_memory.cjs',wasm,output/'chromatic-memory.json'])
invalid=run('python-chromatic-types-invalid',base+[ROOT/'scripts/bindings/authoring/chromatic_typing_invalid.py'],expected=1)
assert 'Found 4 errors in 1 file' in invalid
run('chromatic-composition-python',[sys.executable,ROOT/'scripts/bindings/chromatic_composition.py',module,output/'chromatic-composition-python.json'])
run('chromatic-composition-wasm',[node,ROOT/'scripts/bindings/chromatic_composition.cjs',wasm,output/'chromatic-composition-wasm.json'])
run('chromatic-transformed-python',[sys.executable,ROOT/'scripts/bindings/chromatic_transformed.py',module,output/'chromatic-transformed-python.json'])
run('chromatic-transformed-wasm',[node,ROOT/'scripts/bindings/chromatic_transformed.cjs',wasm,output/'chromatic-transformed-wasm.json'])

# WP-S01 independent shape reference contexts consume the shared path engine.
shape_foundation=output/'shape-foundation'
run('shape-foundation-core',['cargo','test','-p','chart-core','--test','shape_foundation','--locked'])
run('shape-foundation-rust',['cargo','run','-p','chart-export','--example','shape_foundation','--locked','--',shape_foundation/'rust'])
run('shape-foundation-python',[sys.executable,ROOT/'scripts/bindings/shape_foundation.py',module,shape_foundation/'python'])
run('shape-foundation-wasm',[node,ROOT/'scripts/bindings/shape_foundation.cjs',wasm,shape_foundation/'wasm'])
run('shape-foundation-compare',[sys.executable,ROOT/'scripts/bindings/shape_foundation_compare.py',shape_foundation])

# WP-S02 checked generators and projected chart routes share the same core engine.
shape=output/'shape-cartesian'
run('shape-cartesian-core',['cargo','test','-p','chart-core','--test','shape_cartesian','--test','shape_integration','--locked'])
run('shape-cartesian-rust',['cargo','run','-p','chart-export','--example','shape_cartesian','--locked','--',shape/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('shape-cartesian-'+host,command+[ROOT/'scripts/bindings'/('shape_cartesian.'+extension),owner])
    run('shape-cartesian-gallery-'+host,command+[ROOT/'scripts/bindings'/('shape_cartesian_gallery.'+extension),owner,shape/host])
    run('shape-cartesian-updates-'+host,command+[ROOT/'scripts/bindings'/('shape_cartesian_updates.'+extension),owner,shape/(host+'-updates.json')])
    run('shape-cartesian-interaction-'+host,command+[ROOT/'scripts/bindings'/('shape_cartesian_interaction.'+extension),owner])
run('shape-cartesian-compare',[sys.executable,ROOT/'scripts/bindings/shape_cartesian_compare.py',shape])
src=(ROOT/'scripts/bindings/authoring/shape_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape.cts').write_text(src)
run('shape-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape.cts'])
run('shape-python-types',base+[ROOT/'scripts/bindings/authoring/shape_typing.py'])
invalid=run('shape-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid

# WP-S03 arc/pie kernels, primary chart routes and actual host contracts.
shape=output/'shape-arc'
run('shape-arc-core',['cargo','test','-p','chart-core','--test','shape_arc_pie','--test','shape_arc_integration','--locked'])
run('shape-arc-rust',['cargo','run','-p','chart-export','--example','shape_arc','--locked','--',shape/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    for name,args in [('shape_arc_pie',[]),('shape_arc_gallery',[shape/host]),('shape_arc_updates',[shape/(host+'-updates.json')]),('shape_arc_interaction',[])]:
        run(name+'-'+host,command+[ROOT/'scripts/bindings'/(name+'.'+extension),owner]+args)
run('shape-arc-compare',[sys.executable,ROOT/'scripts/bindings/shape_arc_compare.py',shape])
src=(ROOT/'scripts/bindings/authoring/shape_arc_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape-arc.cts').write_text(src)
run('shape-arc-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape-arc.cts'])
run('shape-arc-python-types',base+[ROOT/'scripts/bindings/authoring/shape_arc_typing.py'])
invalid=run('shape-arc-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_arc_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid

# WP-S05 area-symbol kernels, mappings, guides and actual host contracts.
shape=output/'shape-symbol'
run('shape-symbol-core',['cargo','test','-p','chart-core','--test','shape_symbol','--test','shape_symbol_integration','--locked'])
run('shape-symbol-rust',['cargo','run','-p','chart-export','--example','shape_symbol','--locked','--',shape/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    for name,args in [('shape_symbol',[]),('shape_symbol_gallery',[shape/host]),('shape_symbol_updates',[shape/(host+'-updates.json')]),('shape_symbol_interaction',[])]:
        run(name+'-'+host,command+[ROOT/'scripts/bindings'/(name+'.'+extension),owner]+args)
run('shape-symbol-compare',[sys.executable,ROOT/'scripts/bindings/shape_symbol_compare.py',shape])
src=(ROOT/'scripts/bindings/authoring/shape_symbol_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape-symbol.cts').write_text(src)
run('shape-symbol-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape-symbol.cts'])
run('shape-symbol-python-types',base+[ROOT/'scripts/bindings/authoring/shape_symbol_typing.py'])
invalid=run('shape-symbol-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_symbol_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid
# WP-S06 reference orders/offsets and primary sparse stack positions.
shape=output/'shape-stack'
run('shape-stack-core',['cargo','test','-p','chart-core','--test','shape_stack','--test','shape_stack_integration','--locked'])
run('shape-stack-rust',['cargo','run','-p','chart-export','--example','shape_stack','--locked','--',shape/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    for name,args in [('shape_stack',[]),('shape_stack_gallery',[shape/host]),('shape_stack_updates',[shape/(host+'-updates.json')]),('shape_stack_interaction',[])]:
        run(name+'-'+host,command+[ROOT/'scripts/bindings'/(name+'.'+extension),owner]+args)
run('shape-stack-compare',[sys.executable,ROOT/'scripts/bindings/shape_stack_compare.py',shape])
src=(ROOT/'scripts/bindings/authoring/shape_stack_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape-stack.cts').write_text(src)
run('shape-stack-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape-stack.cts'])
run('shape-stack-python-types',base+[ROOT/'scripts/bindings/authoring/shape_stack_typing.py'])
invalid=run('shape-stack-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_stack_typing_invalid.py'],expected=1)
assert 'Found 6 errors in 1 file' in invalid

# FIX-S05/09 radial/link controls, chart projection, provenance and retained publication.
run('shape-radial-core',['cargo','test','-p','chart-core','--test','shape_radial','--test','shape_cartesian','--test','shape_radial_integration','--locked'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('shape-radial-'+host,command+[ROOT/'scripts/bindings'/('shape_radial.'+extension),owner])
src=(ROOT/'scripts/bindings/authoring/shape_radial_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape-radial.cts').write_text(src)
run('shape-radial-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape-radial.cts'])
run('shape-radial-python-types',base+[ROOT/'scripts/bindings/authoring/shape_radial_typing.py'])
invalid=run('shape-radial-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_radial_typing_invalid.py'],expected=1)
assert 'Found 6 errors in 1 file' in invalid

radial=output/'shape-radial'
run('shape-radial-rust-gallery',['cargo','run','-p','chart-export','--example','shape_radial','--locked','--',radial/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('shape-projection-budget-'+host,command+[ROOT/'scripts/bindings'/('shape_projection_budget.'+extension),owner])
    run('shape-radial-interaction-'+host,command+[ROOT/'scripts/bindings'/('shape_radial_interaction.'+extension),owner])
    run('shape-radial-updates-'+host,command+[ROOT/'scripts/bindings'/('shape_radial_updates.'+extension),owner,radial/('updates-'+host+'.json')])
    run('shape-radial-gallery-'+host,command+[ROOT/'scripts/bindings'/('shape_radial_gallery.'+extension),owner,radial/host])
run('shape-radial-compare',[sys.executable,ROOT/'scripts/bindings/shape_radial_compare.py',radial])

# FIX-S08/09 registered native protocols and retained chart/publication consumers.
custom=output/'shape-custom'
run('shape-custom-core',['cargo','test','-p','chart-extension-example','--test','shapes','--test','shape_charts','--locked'])
run('shape-custom-rust-gallery',['cargo','run','-p','chart-export','--example','shape_custom','--locked','--',custom/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('shape-custom-'+host,command+[ROOT/'scripts/bindings'/('shape_custom.'+extension),owner])
    run('shape-custom-updates-'+host,command+[ROOT/'scripts/bindings'/('shape_custom_updates.'+extension),owner,custom/('updates-'+host+'.json')])
    run('shape-custom-gallery-'+host,command+[ROOT/'scripts/bindings'/('shape_custom_gallery.'+extension),owner,custom/'rust',custom/host])
run('shape-custom-compare',[sys.executable,ROOT/'scripts/bindings/shape_radial_compare.py',custom])
src=(ROOT/'scripts/bindings/authoring/shape_custom_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'shape-custom.cts').write_text(src)
run('shape-custom-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'shape-custom.cts'])
run('shape-custom-python-types',base+[ROOT/'scripts/bindings/authoring/shape_custom_typing.py'])
invalid=run('shape-custom-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/shape_custom_typing_invalid.py'],expected=1)
assert 'Found 3 errors in 1 file' in invalid

# WP-S08 retained curved dashes preserve original geometry and semantic anchors.
dash=output/'shape-dashes'
run('shape-dashes-core-export',['cargo','test','-p','chart-core','-p','chart-export','--test','shape_dashes','--locked'])
run('shape-dashes-rust',['cargo','run','-p','chart-export','--example','shape_dashes','--locked','--',dash/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('shape-dashes-'+host,command+[ROOT/'scripts/bindings'/('shape_dashes.'+extension),owner])
    run('shape-dashes-gallery-'+host,command+[ROOT/'scripts/bindings'/('shape_dashes_gallery.'+extension),owner,dash/'rust',dash/host])
run('shape-dashes-compare',[sys.executable,ROOT/'scripts/bindings/shape_radial_compare.py',dash])
