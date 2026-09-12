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
for file in ('authoring.cjs','authoring.d.cts','interpolation.cjs','interpolation.d.cts','scales.cjs','scales.d.cts','examples.cjs','examples.d.cts','hierarchy.cjs','hierarchy.d.cts'):shutil.copy2(ROOT/'packages/wasm'/file,wasm/file)
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
# GG-04 explicit R date/time patterns and multiline publication labels.
formats=output/'ggplot-formats'
run('formats-core',['cargo','test','-p','chart-core','--test','ggplot_time_formats','--locked'])
run('formats-python',[sys.executable,ROOT/'scripts/bindings/ggplot_formats.py',module,formats/'python'])
run('formats-wasm',[node,ROOT/'scripts/bindings/ggplot_formats.cjs',wasm,formats/'wasm'])
assert json.loads((formats/'python/records.json').read_text()) == json.loads((formats/'wasm/records.json').read_text())
src=(ROOT/'scripts/bindings/authoring/ggplot_formats_types.cts').read_text().replace('../../../target/ggplot-formats/wasm-module/','../wasm-module/')
(consumer/'ggplot-formats.cts').write_text(src)
run('typescript-formats',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-formats.cts'])
run('python-formats-types',base+[ROOT/'scripts/bindings/authoring/ggplot_formats_typing.py'])
# GG-04 exceptional limit populations and fractional binned counts.
scale_limits=output/'ggplot-scale-limits'
run('scale-limits-core',['cargo','test','-p','chart-core','--test','ggplot_binned_guides','--test','ggplot_degenerate_mapping','--locked'])
run('scale-limits-python',[sys.executable,ROOT/'scripts/bindings/ggplot_scale_limits.py',module,scale_limits/'python'])
run('scale-limits-wasm',[node,ROOT/'scripts/bindings/ggplot_scale_limits.cjs',wasm,scale_limits/'wasm'])
assert json.loads((scale_limits/'python/records.json').read_text()) == json.loads((scale_limits/'wasm/records.json').read_text())
for sample in ('infinite-limit','hidden-bins','fractional-bins'):
    assert (scale_limits/'python'/f'{sample}.png').read_bytes() == (scale_limits/'wasm'/f'{sample}.png').read_bytes()
# GG-04 positional bins include finite and unbounded reference panels.
positional=output/'ggplot-positional'
run('positional-core',['cargo','test','-p','chart-core','--test','ggplot_position_bins','--locked'])
run('positional-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_bins.py',module,positional/'python'])
run('positional-wasm',[node,ROOT/'scripts/bindings/ggplot_position_bins.cjs',wasm,positional/'wasm'])
assert json.loads((positional/'python/records.json').read_text()) == json.loads((positional/'wasm/records.json').read_text())
for sample in ('identity','sqrt','reverse','unbounded'):
    name=sample+'-positional-bins.png'
    assert (positional/'python'/name).read_bytes() == (positional/'wasm'/name).read_bytes()
src=(ROOT/'scripts/bindings/authoring/ggplot_position_bins_types.cts').read_text().replace('../../../target/ggplot-positional/wasm-module/','../wasm-module/')
(consumer/'ggplot-position-bins.cts').write_text(src)
run('typescript-positional',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-position-bins.cts'])
run('python-positional-types',base+[ROOT/'scripts/bindings/authoring/ggplot_position_bins_typing.py'])
# GG-04 numeric and temporal minor selection shares core semantics across hosts.
minor=output/'ggplot-minor'
run('minor-core',['cargo','test','-p','chart-core','--test','ggplot_minor_breaks','--locked'])
run('minor-python',[sys.executable,ROOT/'scripts/bindings/ggplot_minor_breaks.py',module,minor/'python'])
run('minor-wasm',[node,ROOT/'scripts/bindings/ggplot_minor_breaks.cjs',wasm,minor/'wasm'])
run('minor-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_minor_compare.py',minor])
src=(ROOT/'scripts/bindings/authoring/ggplot_minor_types.cts').read_text().replace('../../../target/ggplot-minor/wasm-module/','../wasm-module/')
(consumer/'ggplot-minor.cts').write_text(src)
run('typescript-minor',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-minor.cts'])
run('python-minor-types',base+[ROOT/'scripts/bindings/authoring/ggplot_minor_typing.py'])
# GG-04 category-index continuous limits and immutable population updates.
discrete_limits=output/'ggplot-positional-limits'
run('positional-limits-ranges',['cargo','test','-p','chart-core','--lib','category_index_ranges_match_reference_authored_limits','--locked'])
run('positional-limits-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_limits','--locked'])
run('positional-limits-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_limits.py',module,discrete_limits/'python'])
run('positional-limits-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_limits.cjs',wasm,discrete_limits/'wasm'])
assert json.loads((discrete_limits/'python/records.json').read_text()) == json.loads((discrete_limits/'wasm/records.json').read_text())
for sample in ('unused-levels','one-infinite-endpoint','undefined-positions','excluded-population'):
    assert (discrete_limits/'python'/f'{sample}.png').read_bytes() == (discrete_limits/'wasm'/f'{sample}.png').read_bytes()
src=(ROOT/'scripts/bindings/authoring/ggplot_discrete_limits_types.cts').read_text().replace('../../../target/ggplot-discrete-limits/wasm-module/','../wasm-module/')
(consumer/'ggplot-discrete-limits.cts').write_text(src)
run('typescript-discrete-limits',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-discrete-limits.cts'])
run('python-discrete-limits-types',base+[ROOT/'scripts/bindings/authoring/ggplot_discrete_limits_typing.py'])
nullable_discrete=output/'ggplot-discrete-null'
run('discrete-null-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_null','--test','ggplot_untrained_discrete','--locked'])
run('discrete-null-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_null.py',module,nullable_discrete/'python'])
run('discrete-null-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_null.cjs',wasm,nullable_discrete/'wasm'])
assert json.loads((nullable_discrete/'python/records.json').read_text()) == json.loads((nullable_discrete/'wasm/records.json').read_text())
for sample in ['authored-missing-first','factor-missing-middle','suppressed-missing','empty-palette','unpainted-categories','identity-missing-first']:
    assert (nullable_discrete/'python'/f'{sample}.png').read_bytes() == (nullable_discrete/'wasm'/f'{sample}.png').read_bytes()

nullable_position=output/'ggplot-discrete-position'
run('discrete-position-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_position','--locked'])
run('discrete-position-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_position.py',module,nullable_position/'python'])
run('discrete-position-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_position.cjs',wasm,nullable_position/'wasm'])
assert json.loads((nullable_position/'python/records.json').read_text()) == json.loads((nullable_position/'wasm/records.json').read_text())
for sample in ('True','False','expanded'):
    for fmt in ('svg','pdf','png'):
        assert (nullable_position/'python'/f'null-positions-{sample}.{fmt}').read_bytes() == (nullable_position/'wasm'/f'null-positions-{sample}.{fmt}').read_bytes()

secondary_discrete=output/'ggplot-discrete-secondary'
run('discrete-secondary-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_secondary','--locked'])
run('discrete-secondary-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_secondary.py',module,secondary_discrete/'python'])
run('discrete-secondary-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_secondary.cjs',wasm,secondary_discrete/'wasm'])
assert json.loads((secondary_discrete/'python/records.json').read_text()) == json.loads((secondary_discrete/'wasm/records.json').read_text())
for sample in ('inherit','numeric','explicit'):
    for fmt in ('svg','pdf','png'):
        assert (secondary_discrete/'python'/f'discrete-secondary-{sample}.{fmt}').read_bytes() == (secondary_discrete/'wasm'/f'discrete-secondary-{sample}.{fmt}').read_bytes()

position_palettes=output/'ggplot-discrete-position-palette'
run('discrete-position-palette-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_position_palette','--locked'])
run('discrete-position-palette-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_position_palette.py',module,position_palettes/'python'])
run('discrete-position-palette-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_position_palette.cjs',wasm,position_palettes/'wasm'])
assert json.loads((position_palettes/'python/records.json').read_text()) == json.loads((position_palettes/'wasm/records.json').read_text())
for sample in ('reverse','spread','repeated','nonfinite'):
    for fmt in ('svg','pdf','png'):
        assert (position_palettes/'python'/f'position-palette-{sample}.{fmt}').read_bytes() == (position_palettes/'wasm'/f'position-palette-{sample}.{fmt}').read_bytes()

temporal_aesthetics=output/'ggplot-temporal-aesthetics'
run('temporal-aesthetics-core',['cargo','test','-p','chart-core','--test','ggplot_temporal_aesthetics','--locked'])
run('temporal-aesthetics-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_aesthetics.py',module,temporal_aesthetics/'python'])
run('temporal-aesthetics-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_aesthetics.cjs',wasm,temporal_aesthetics/'wasm'])
assert json.loads((temporal_aesthetics/'python/records.json').read_text()) == json.loads((temporal_aesthetics/'wasm/records.json').read_text())
for sample in ('size','alpha','colour','fill'):
    for fmt in ('svg','pdf','png'):
        assert (temporal_aesthetics/'python'/f'temporal-{sample}.{fmt}').read_bytes() == (temporal_aesthetics/'wasm'/f'temporal-{sample}.{fmt}').read_bytes()

run('temporal-precision-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_precision.py',module,temporal_aesthetics/'python'])
run('temporal-precision-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_precision.cjs',wasm,temporal_aesthetics/'wasm'])
assert json.loads((temporal_aesthetics/'python/precision-records.json').read_text()) == json.loads((temporal_aesthetics/'wasm/precision-records.json').read_text())

run('temporal-guides-core',['cargo','test','-p','chart-core','--test','ggplot_temporal_guides','--locked'])
run('temporal-guides-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_guides.py',module,temporal_aesthetics/'python'])
run('temporal-guides-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_guides.cjs',wasm,temporal_aesthetics/'wasm'])
assert json.loads((temporal_aesthetics/'python/guide-records.json').read_text()) == json.loads((temporal_aesthetics/'wasm/guide-records.json').read_text())
for sample in ('date-short-default','datetime-short-width','date-constant-null_breaks'):
    for fmt in ('svg','pdf','png'):
        assert (temporal_aesthetics/'python'/f'{sample}.{fmt}').read_bytes() == (temporal_aesthetics/'wasm'/f'{sample}.{fmt}').read_bytes()

discrete_limits=output/'ggplot-discrete-limits'
run('discrete-limits-core',['cargo','test','-p','chart-extension-example','--test','discrete_limits','--locked'])
run('discrete-limits-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_limits.py',module,discrete_limits/'python'])
run('discrete-limits-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_limits.cjs',wasm,discrete_limits/'wasm'])
assert json.loads((discrete_limits/'python/discrete-limit-records.json').read_text()) == json.loads((discrete_limits/'wasm/discrete-limit-records.json').read_text())
for sample in ('discrete-reverse','discrete-fixed','identity-reverse','identity-fixed'):
    for fmt in ('svg','pdf','png'):
        assert (discrete_limits/'python'/f'{sample}.{fmt}').read_bytes() == (discrete_limits/'wasm'/f'{sample}.{fmt}').read_bytes()
# Numeric callback limits retain raw vector arity and source-domain replacement.
numeric_limits=output/'ggplot-numeric-limits'
run('numeric-limits-core',['cargo','test','-p','chart-extension-example','--test','numeric_limits','--locked'])
run('numeric-limits-python',[sys.executable,ROOT/'scripts/bindings/ggplot_numeric_limits.py',module,numeric_limits/'python'])
run('numeric-limits-wasm',[node,ROOT/'scripts/bindings/ggplot_numeric_limits.cjs',wasm,numeric_limits/'wasm'])
assert json.loads((numeric_limits/'python/numeric-limit-records.json').read_text()) == json.loads((numeric_limits/'wasm/numeric-limit-records.json').read_text())
for family in ('continuous','binned','date','datetime'):
    for control in ('reverse','fixed','single'):
        for fmt in ('svg','pdf','png'):
            assert (numeric_limits/'python'/f'{family}-{control}.{fmt}').read_bytes() == (numeric_limits/'wasm'/f'{family}-{control}.{fmt}').read_bytes()
binned_styles=output/'ggplot-binned-styles'
run('binned-styles-core',['cargo','test','-p','chart-core','--test','ggplot_binned_styles','--locked'])
run('binned-styles-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_styles.py',module,binned_styles/'python'])
run('binned-styles-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_styles.cjs',wasm,binned_styles/'wasm'])
assert json.loads((binned_styles/'python/binned-style-records.json').read_text()) == json.loads((binned_styles/'wasm/binned-style-records.json').read_text())
for sample in ('solid','hollow','linetype','brewer'):
    for fmt in ('svg','pdf','png'):
        assert (binned_styles/'python'/f'{sample}.{fmt}').read_bytes() == (binned_styles/'wasm'/f'{sample}.{fmt}').read_bytes()

run('binned-numeric-core',['cargo','test','-p','chart-core','--test','ggplot_binned_numeric','--locked'])
run('binned-numeric-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_numeric.py',module,binned_styles/'python'])
run('binned-numeric-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_numeric.cjs',wasm,binned_styles/'wasm'])
run('line-widths-python',[sys.executable,ROOT/'scripts/bindings/ggplot_line_widths.py',module,binned_styles/'python'])
run('line-widths-wasm',[node,ROOT/'scripts/bindings/ggplot_line_widths.cjs',wasm,binned_styles/'wasm'])
for record in ('binned-numeric-records.json','line-width-records.json'):
    assert json.loads((binned_styles/'python'/record).read_text()) == json.loads((binned_styles/'wasm'/record).read_text())
for sample in ('numeric-size','numeric-area','numeric-alpha','numeric-linewidth','width-segment','width-line','width-path','width-rect'):
    for fmt in ('svg','pdf','png'):
        assert (binned_styles/'python'/f'{sample}.{fmt}').read_bytes() == (binned_styles/'wasm'/f'{sample}.{fmt}').read_bytes()

src=(ROOT/'scripts/bindings/authoring/ggplot_discrete_position_types.cts').read_text().replace('../../../target/ggplot-position-null/wasm-module/','../wasm-module/')
(consumer/'ggplot-discrete-position.cts').write_text(src)
run('typescript-discrete-position',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-discrete-position.cts'])
run('python-discrete-position-types',base+[ROOT/'scripts/bindings/authoring/ggplot_discrete_position_typing.py'])

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

# AXIS-01 independent guides and retained positional providers.
axis=output/'axis-provider'
run('axis-core',['cargo','test','-p','chart-core','--test','axis_guides','--test','axis_providers','--locked'])
run('axis-provider-rust',['cargo','run','-p','chart-export','--example','axis_provider_proof','--locked','--',axis/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('axis-guides-'+host,command+[ROOT/'scripts/bindings'/('axis_guides.'+extension),owner])
    run('axis-provider-'+host,command+[ROOT/'scripts/bindings'/('axis_providers.'+extension),owner,axis/host])
for extension in ('svg','pdf','png'):
    assert len({(axis/host/('provider.'+extension)).read_bytes() for host in ('rust','python','wasm')})==1
src=(ROOT/'scripts/bindings/authoring/axis_provider_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'axis-provider.cts').write_text(src)
run('axis-provider-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'axis-provider.cts'])
run('axis-provider-python-types',base+[ROOT/'scripts/bindings/authoring/axis_provider_typing.py'])
invalid=run('axis-provider-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/axis_provider_typing_invalid.py'],expected=1)
assert 'Found 3 errors in 1 file' in invalid

# AXIS-02 independent selection and formatting over the AXIS-01 guide contract.
axis_ticks=output/'axis-ticks'
run('axis-ticks-core',['cargo','test','-p','chart-core','--test','axis_ticks','--locked'])
run('axis-ticks-rust',['cargo','run','-p','chart-export','--example','axis_tick_proof','--locked','--',axis_ticks/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('axis-ticks-'+host,command+[ROOT/'scripts/bindings'/('axis_ticks.'+extension),owner,axis_ticks/host])
for extension in ('svg','pdf','png'):
    assert len({(axis_ticks/host/('ticks.'+extension)).read_bytes() for host in ('rust','python','wasm')})==1
src=(ROOT/'scripts/bindings/authoring/axis_tick_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'axis-ticks.cts').write_text(src)
run('axis-ticks-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'axis-ticks.cts'])
run('axis-ticks-python-types',base+[ROOT/'scripts/bindings/authoring/axis_tick_typing.py'])
invalid=run('axis-ticks-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/axis_tick_typing_invalid.py'],expected=1)
assert 'Found 3 errors in 1 file' in invalid

# IP06 registered factories, immutable scale consumers and explicit sampled publication.
ip=output/'interpolation-integration'
run('interpolation-registered-core',['cargo','test','-p','chart-core','--test','interpolate_registered','--locked'])
run('interpolation-integration-rust',['cargo','run','-p','chart-export','--example','interpolation_proof','--locked','--',ip/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('interpolation-integration-'+host,command+[ROOT/'scripts/bindings'/('interpolation_integration.'+extension),owner,ip/host])
run('interpolation-integration-compare',[sys.executable,ROOT/'scripts/bindings/interpolation_integration_compare.py',ip])
src=(ROOT/'scripts/bindings/authoring/interpolation_integration_types.cts').read_text().replace('../../../target/interpolation-integration/wasm-module/','../wasm-module/')
(consumer/'interpolation-integration.cts').write_text(src)
run('interpolation-integration-typescript',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'interpolation-integration.cts'])
run('interpolation-integration-python-types',base+[ROOT/'scripts/bindings/authoring/interpolation_integration_typing.py'])
invalid=run('interpolation-integration-python-types-invalid',base+[ROOT/'scripts/bindings/authoring/interpolation_integration_typing_invalid.py'],expected=1)
assert 'Found 5 errors in 1 file' in invalid

# AXIS-04/05 signed geometry, portable components and one retained publication route.
axis_components=output/'axis-components'
run('axis-components-core',['cargo','test','-p','chart-core','--test','axis_components','--test','axis_ticks','--locked'])
run('axis-components-publication',['cargo','test','-p','chart-export','--test','axis_components','--locked'])
run('axis-geometry-rust',['cargo','run','-p','chart-export','--example','axis_geometry_proof','--locked','--',output/'axis-geometry'])
run('axis-components-rust',['cargo','run','-p','chart-export','--example','axis_component_proof','--locked','--',axis_components/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('axis-components-'+host,command+[ROOT/'scripts/bindings'/('axis_components.'+extension),owner,axis_components/host])
run('axis-components-compare',[sys.executable,ROOT/'scripts/bindings/axis_components_compare.py',axis_components])
# AX05 shares the core tick-join plan, explicit host samples and displayed capture.
axis_transitions=output/'axis-transitions'
run('axis-transitions-core',['cargo','test','-p','chart-core','--test','axis_transitions','--locked'])
run('axis-transitions-publication',['cargo','test','-p','chart-export','--test','axis_transitions','--locked'])
run('axis-transitions-rust',['cargo','run','-p','chart-export','--example','axis_transition_proof','--locked','--',axis_transitions/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('axis-transitions-'+host,command+[ROOT/'scripts/bindings'/('axis_transitions.'+extension),owner,axis_transitions/host])
run('axis-transitions-compare',[sys.executable,ROOT/'scripts/bindings/axis_components_compare.py',axis_transitions,'transitions'])
src=(ROOT/'scripts/bindings/authoring/axis_transition_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'axis-transitions.cts').write_text(src)
run('typescript-axis-transitions',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'axis-transitions.cts'])
run('python-axis-transitions-types',base+[ROOT/'scripts/bindings/authoring/axis_transition_typing.py'])

# HIR-01 through HIR-08: one core session and chart recipe path in actual hosts.
hierarchy=output/'hierarchy';hierarchy.mkdir(exist_ok=True)
run('hierarchy-requests',[sys.executable,ROOT/'scripts/hierarchy/requests.py',hierarchy/'requests.json'])
run('hierarchy-rust',['cargo','run','-p','chart-extension-example','--example','hierarchy_proof','--locked','--',hierarchy/'requests.json',hierarchy/'rust.json'])
run('hierarchy-python',[sys.executable,ROOT/'scripts/bindings/hierarchy.py',hierarchy/'requests.json',hierarchy/'python.json',module])
run('hierarchy-wasm',[node,ROOT/'scripts/bindings/hierarchy.cjs',wasm,hierarchy/'requests.json',hierarchy/'wasm.json'])
run('hierarchy-compare',[sys.executable,ROOT/'scripts/hierarchy/compare.py',hierarchy/'rust.json',hierarchy/'python.json',hierarchy/'wasm.json'])
run('hierarchy-padding-requests',[sys.executable,ROOT/'scripts/hierarchy/padding.py','requests',hierarchy/'padding-requests.json'])
run('hierarchy-padding-rust',['cargo','run','-p','chart-extension-example','--example','hierarchy_proof','--locked','--',hierarchy/'padding-requests.json',hierarchy/'padding-rust.json'])
run('hierarchy-padding-python',[sys.executable,ROOT/'scripts/bindings/hierarchy.py',hierarchy/'padding-requests.json',hierarchy/'padding-python.json',module])
run('hierarchy-padding-wasm',[node,ROOT/'scripts/bindings/hierarchy.cjs',wasm,hierarchy/'padding-requests.json',hierarchy/'padding-wasm.json'])
run('hierarchy-padding-compare',[sys.executable,ROOT/'scripts/hierarchy/padding.py','compare',hierarchy/'padding-rust.json',hierarchy/'padding-python.json',hierarchy/'padding-wasm.json'])
run('hierarchy-control-requests',[sys.executable,ROOT/'scripts/hierarchy/controls.py','requests',hierarchy/'control-requests.json'])
run('hierarchy-control-rust',['cargo','run','-p','chart-extension-example','--example','hierarchy_proof','--locked','--',hierarchy/'control-requests.json',hierarchy/'control-rust.json'])
run('hierarchy-control-python',[sys.executable,ROOT/'scripts/bindings/hierarchy.py',hierarchy/'control-requests.json',hierarchy/'control-python.json',module])
run('hierarchy-control-wasm',[node,ROOT/'scripts/bindings/hierarchy.cjs',wasm,hierarchy/'control-requests.json',hierarchy/'control-wasm.json'])
run('hierarchy-control-compare',[sys.executable,ROOT/'scripts/hierarchy/controls.py','compare',hierarchy/'control-rust.json',hierarchy/'control-python.json',hierarchy/'control-wasm.json'])
hierarchy_charts=hierarchy/'charts'
run('hierarchy-chart-rust',['cargo','run','-p','chart-export','--example','hierarchy_chart_proof','--locked','--',hierarchy_charts/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('hierarchy-chart-'+host,command+[ROOT/'scripts/bindings'/('hierarchy_chart.'+extension),owner,hierarchy_charts/host])
    run('hierarchy-updates-'+host,command+[ROOT/'scripts/bindings'/('hierarchy_updates.'+extension),owner,hierarchy/(host+'-updates.json')])
    run('hierarchy-ownership-'+host,command+[ROOT/'scripts/bindings'/('hierarchy_ownership.'+extension),owner,hierarchy/(host+'-ownership.json')])
run('hierarchy-updates-compare',[sys.executable,ROOT/'scripts/hierarchy/updates_compare.py',hierarchy/'python-updates.json',hierarchy/'wasm-updates.json'])
run('hierarchy-chart-compare',[sys.executable,ROOT/'scripts/bindings/axis_components_compare.py',hierarchy_charts,'hierarchy'])
src=(ROOT/'scripts/bindings/authoring/hierarchy_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'hierarchy.cts').write_text(src)
run('typescript-hierarchy',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'hierarchy.cts'])
run('python-hierarchy-types',base+[ROOT/'scripts/bindings/authoring/hierarchy_typing.py'])
# GG-03: independent channels and reference glyphs use the same live host builds.
aesthetics=output/'ggplot-aesthetics';aesthetics.mkdir(exist_ok=True)
run('ggplot-aesthetics-rust',['cargo','run','-p','chart-export','--example','ggplot_aesthetics','--locked','--',aesthetics/'rust'])
for host,extension,command,owner in [('python','py',[sys.executable],module),('wasm','cjs',[node],wasm)]:
    run('ggplot-aesthetics-'+host,command+[ROOT/'scripts/bindings'/('ggplot_aesthetics.'+extension),owner,aesthetics/host])
    run('ggplot-aesthetic-updates-'+host,command+[ROOT/'scripts/bindings'/('ggplot_aesthetic_updates.'+extension),owner,aesthetics/host/'updates.json'])
run('ggplot-aesthetics-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_aesthetic_compare.py',aesthetics/'rust',aesthetics/'python',aesthetics/'wasm',aesthetics/'comparison.json'])
src=(ROOT/'scripts/bindings/authoring/ggplot_aesthetic_types.cts').read_text().replace('../../../packages/wasm/','../wasm-module/')
(consumer/'ggplot-aesthetics.cts').write_text(src)
run('typescript-ggplot-aesthetics',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-aesthetics.cts'])
run('python-ggplot-aesthetics-types',base+[ROOT/'scripts/bindings/authoring/ggplot_aesthetic_typing.py'])
