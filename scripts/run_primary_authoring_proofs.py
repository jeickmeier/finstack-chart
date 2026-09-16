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
from build_primary_modules import python_module, wasm_module

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
module=python_module(ROOT,output,target,run)
run('python-primary',[sys.executable,ROOT/'scripts/bindings/authoring/python_proof.py',module,output/'python'])
wasm=wasm_module(ROOT,output,target,run,cli)
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
identity_callbacks=output/'discrete-identity-functions'
run('identity-callbacks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_identity_functions.py',module,identity_callbacks/'python'])
run('identity-callbacks-wasm',[node,ROOT/'scripts/bindings/ggplot_identity_functions.cjs',wasm,identity_callbacks/'wasm'])
run('identity-callbacks-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',identity_callbacks/'python',identity_callbacks/'wasm',identity_callbacks/'comparison.json','577','18'])

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
        if family == 'binned' and control == 'reverse':
            continue  # The actual reference plot rejects this guide; raw mapping remains covered.
        for fmt in ('svg','pdf','png'):
            assert (numeric_limits/'python'/f'{family}-{control}.{fmt}').read_bytes() == (numeric_limits/'wasm'/f'{family}-{control}.{fmt}').read_bytes()
# Primary positional callbacks train before statistics and after positions.
position_functions=output/'ggplot-position-functions'
run('position-functions-core',['cargo','test','-p','chart-extension-example','--test','positional_limits','--locked'])
run('position-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_functions.py',module,position_functions/'python'])
run('position-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_position_functions.cjs',wasm,position_functions/'wasm'])
assert json.loads((position_functions/'python/position-function-records.json').read_text()) == json.loads((position_functions/'wasm/position-function-records.json').read_text())
for prefix in ('','binned-'):
    for transform in ('identity','sqrt'):
        for control in ('identity','fixed','single'):
            for fmt in ('svg','pdf','png'):
                assert (position_functions/'python'/f'{prefix}{transform}-{control}.{fmt}').read_bytes() == (position_functions/'wasm'/f'{prefix}{transform}-{control}.{fmt}').read_bytes()
for prefix in ('','binned-'):
    for fmt in ('svg','pdf','png'):
        assert (position_functions/'python'/f'{prefix}log10-identity.{fmt}').read_bytes() == (position_functions/'wasm'/f'{prefix}log10-identity.{fmt}').read_bytes()
for transform in ('identity','reverse'):
    for fmt in ('svg','pdf','png'):
        assert (position_functions/'python'/f'{transform}-infinite-guide.{fmt}').read_bytes() == (position_functions/'wasm'/f'{transform}-infinite-guide.{fmt}').read_bytes()
src=(ROOT/'scripts/bindings/authoring/ggplot_position_functions_types.cts').read_text().replace('../../../target/ggplot-position-functions/wasm-module/','../wasm-module/')
(consumer/'ggplot-position-functions.cts').write_text(src)
run('typescript-position-functions',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-position-functions.cts'])
run('python-position-functions-types',base+[ROOT/'scripts/bindings/authoring/ggplot_position_functions_typing.py'])
# Positional missing replacement uses the same engine in every authoring adapter.
position_missing=output/'ggplot-position-missing'
run('position-missing-core',['cargo','test','-p','chart-extension-example','--test','positional_missing','--locked'])
run('position-missing-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_missing.py',module,position_missing/'python'])
run('position-missing-wasm',[node,ROOT/'scripts/bindings/ggplot_position_missing.cjs',wasm,position_missing/'wasm'])
run('position-missing-comparison',[sys.executable,ROOT/'scripts/bindings/compare_position_missing.py',position_missing/'python',position_missing/'wasm',position_missing/'comparison.json'])
# Date/UTC callbacks retain exact source metadata and fractional limit vectors.
position_temporal=output/'ggplot-position-temporal'
run('position-temporal-core',['cargo','test','-p','chart-extension-example','--test','positional_temporal_limits','--locked'])
run('position-temporal-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_temporal.py',module,position_temporal/'python'])
run('position-temporal-wasm',[node,ROOT/'scripts/bindings/ggplot_position_temporal.cjs',wasm,position_temporal/'wasm'])
assert json.loads((position_temporal/'python/position-temporal-records.json').read_text()) == json.loads((position_temporal/'wasm/position-temporal-records.json').read_text())
for kind in ('date','datetime','duration'):
    for context,population,control in [('points','fractional','identity'),('summary','fractional','identity'),('points','all_missing','missing_lower'),('points','spaced','single')]:
        for fmt in ('svg','pdf','png'):
            name=f'{kind}-{context}-{population}-{control}.{fmt}'
            assert (position_temporal/'python'/name).read_bytes() == (position_temporal/'wasm'/name).read_bytes()
for kind in ('date','datetime'):
    for population in ('thirds','sevenths'):
        for fmt in ('svg','pdf','png'):
            name=f'{kind}-summary-{population}-identity.{fmt}'
            assert (position_temporal/'python'/name).read_bytes() == (position_temporal/'wasm'/name).read_bytes()
# Shared/free facet callbacks retain per-panel limits through updates and publication.
position_facets=output/'ggplot-position-facets'
run('position-facets-core',['cargo','test','-p','chart-extension-example','--test','positional_facet_limits','--locked'])
run('position-facets-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_facets.py',module,position_facets/'python'])
run('position-facets-wasm',[node,ROOT/'scripts/bindings/ggplot_position_facets.cjs',wasm,position_facets/'wasm'])
assert json.loads((position_facets/'python/position-facet-records.json').read_text()) == json.loads((position_facets/'wasm/position-facet-records.json').read_text())
for policy in ('fixed','free'):
    samples=[('points','identity','balanced'),('summary','identity','balanced'),('summary','sqrt','constant')]
    if policy=='fixed': samples.append(('points','reverse','empty_panel'))
    for context,transform,population in samples:
        for fmt in ('svg','pdf','png'):
            name=f'{context}-{transform}-{policy}-{population}.{fmt}'
            assert (position_facets/'python'/name).read_bytes() == (position_facets/'wasm'/name).read_bytes()
for kind in ('date','datetime','duration'):
    for policy in ('fixed','free'):
        for context,population in [('points','balanced'),('summary','fractional')]:
            for fmt in ('svg','pdf','png'):
                name=f'{context}-{kind}-{policy}-{population}.{fmt}'
                assert (position_facets/'python'/name).read_bytes() == (position_facets/'wasm'/name).read_bytes()
for transform in ('identity','sqrt','reverse'):
    for policy in ('fixed','free'):
        for context in ('points','summary'):
            for fmt in ('svg','pdf','png'):
                name=f'{context}-binned_{transform}-{policy}-balanced.{fmt}'
                assert (position_facets/'python'/name).read_bytes() == (position_facets/'wasm'/name).read_bytes()
temporal_breaks=output/'ggplot-temporal-break-functions'
for suffix,args,count in [('',[],30280),('-overrides',['overrides'],240)]:
    dest=temporal_breaks/('overrides' if args else 'functions')
    run('temporal-breaks-python'+suffix,[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_break_functions.py',module,dest/'python',*args])
    run('temporal-breaks-wasm'+suffix,[node,ROOT/'scripts/bindings/ggplot_temporal_break_functions.cjs',wasm,dest/'wasm',*args])
    temporal_break_records=json.loads((dest/'python/records.json').read_text())
    assert len(temporal_break_records)==count
    assert temporal_break_records == json.loads((dest/'wasm/records.json').read_text())

binned_breaks=output/'ggplot-binned-break-functions'
run('binned-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_break_functions.py',module,binned_breaks/'python'])
run('binned-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_break_functions.cjs',wasm,binned_breaks/'wasm'])
binned_break_records=json.loads((binned_breaks/'python/records.json').read_text())
assert len(binned_break_records)==4686
assert binned_break_records == json.loads((binned_breaks/'wasm/records.json').read_text())

for suffix, args, count in [('', [], 4686), ('-names', ['--default-names'], 116)]:
    dest=output/('ggplot-binned-constructor-breaks'+suffix)
    run('binned-constructor-breaks-python'+suffix,[sys.executable,ROOT/'scripts/bindings/ggplot_binned_break_functions.py',module,dest/'python','--constructor-guides',*args])
    run('binned-constructor-breaks-wasm'+suffix,[node,ROOT/'scripts/bindings/ggplot_binned_break_functions.cjs',wasm,dest/'wasm','--constructor-guides',*args])
    records=json.loads((dest/'python/records.json').read_text())
    assert len(records)==count
    assert records==json.loads((dest/'wasm/records.json').read_text())

discrete_breaks=output/'ggplot-discrete-break-functions'
run('discrete-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_discrete_break_functions.py',module,discrete_breaks/'python'])
run('discrete-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_discrete_break_functions.cjs',wasm,discrete_breaks/'wasm'])
discrete_break_records=json.loads((discrete_breaks/'python/records.json').read_text())
assert len(discrete_break_records)==1230
assert discrete_break_records == json.loads((discrete_breaks/'wasm/records.json').read_text())

numeric_breaks=output/'ggplot-numeric-break-functions'
run('numeric-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_break_functions.py',module,numeric_breaks/'python'])
run('numeric-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_break_functions.cjs',wasm,numeric_breaks/'wasm'])
numeric_break_records=json.loads((numeric_breaks/'python/records.json').read_text())
assert len(numeric_break_records)==5336
assert numeric_break_records == json.loads((numeric_breaks/'wasm/records.json').read_text())

noncolor_temporal_labels=output/'ggplot-noncolor-temporal-labels'
run('noncolor-temporal-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_noncolor_temporal_label_functions.py',module,noncolor_temporal_labels/'python'])
run('noncolor-temporal-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_noncolor_temporal_label_functions.cjs',wasm,noncolor_temporal_labels/'wasm'])
noncolor_temporal_records=json.loads((noncolor_temporal_labels/'python/records.json').read_text())
assert len(noncolor_temporal_records)==8400
assert noncolor_temporal_records == json.loads((noncolor_temporal_labels/'wasm/records.json').read_text())
noncolor_binned_labels=output/'ggplot-noncolor-binned-labels'
run('noncolor-binned-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_noncolor_binned_label_functions.py',module,noncolor_binned_labels/'python'])
run('noncolor-binned-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_noncolor_binned_label_functions.cjs',wasm,noncolor_binned_labels/'wasm'])
noncolor_binned_records=json.loads((noncolor_binned_labels/'python/records.json').read_text())
assert len(noncolor_binned_records)==2748
assert noncolor_binned_records == json.loads((noncolor_binned_labels/'wasm/records.json').read_text())
interval_labels=output/'ggplot-continuous-interval-labels'
run('interval-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_interval_label_functions.py',module,interval_labels/'python'])
run('interval-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_interval_label_functions.cjs',wasm,interval_labels/'wasm'])
run('interval-labels-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',interval_labels/'python',interval_labels/'wasm',interval_labels/'comparison.json','2720','102'])
noncolor_continuous_labels=output/'ggplot-noncolor-continuous-labels'
run('noncolor-continuous-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_noncolor_continuous_label_functions.py',module,noncolor_continuous_labels/'python'])
run('noncolor-continuous-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_noncolor_continuous_label_functions.cjs',wasm,noncolor_continuous_labels/'wasm'])
noncolor_continuous_records=json.loads((noncolor_continuous_labels/'python/records.json').read_text())
assert len(noncolor_continuous_records)==2496
assert noncolor_continuous_records == json.loads((noncolor_continuous_labels/'wasm/records.json').read_text())
noncolor_discrete_labels=output/'ggplot-noncolor-discrete-labels'
run('noncolor-discrete-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_noncolor_discrete_label_functions.py',module,noncolor_discrete_labels/'python'])
run('noncolor-discrete-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_noncolor_discrete_label_functions.cjs',wasm,noncolor_discrete_labels/'wasm'])
factor_discrete_labels=output/'ggplot-factor-discrete-labels'
run('factor-discrete-labels-core',['cargo','test','-p','chart-core','--test','ggplot_label_functions','factor_discrete_labels_preserve_drop_and_missing_policies','--locked'])
run('factor-discrete-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_noncolor_discrete_label_functions.py',module,factor_discrete_labels/'python','--factors'])
run('factor-discrete-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_noncolor_discrete_label_functions.cjs',wasm,factor_discrete_labels/'wasm','--factors'])
run('factor-discrete-labels-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',factor_discrete_labels/'python',factor_discrete_labels/'wasm',factor_discrete_labels/'comparison.json','1360','39'])
noncolor_discrete_records=json.loads((noncolor_discrete_labels/'python/records.json').read_text())
assert len(noncolor_discrete_records)==1515
assert noncolor_discrete_records == json.loads((noncolor_discrete_labels/'wasm/records.json').read_text())
scale_labels=output/'ggplot-scale-labels'
run('scale-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_label_functions.py',module,scale_labels/'python'])
run('scale-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_label_functions.cjs',wasm,scale_labels/'wasm'])
assert json.loads((scale_labels/'python/records.json').read_text()) == json.loads((scale_labels/'wasm/records.json').read_text())
label_publications=[p.name for p in (scale_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(label_publications)==15
for name in label_publications:
    assert (scale_labels/'python'/name).read_bytes() == (scale_labels/'wasm'/name).read_bytes()
binned_position_labels=output/'ggplot-positional-binned-labels'
run('binned-position-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_binned_label_functions.py',module,binned_position_labels/'python'])
run('binned-position-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_binned_label_functions.cjs',wasm,binned_position_labels/'wasm'])
binned_position_python=json.loads((binned_position_labels/'python/records.json').read_text())
binned_position_wasm=json.loads((binned_position_labels/'wasm/records.json').read_text())
assert len(binned_position_python)==len(binned_position_wasm)==4944
for left,right in zip(binned_position_python,binned_position_wasm):
    assert len(left.get('ticks',[]))==len(right.get('ticks',[]))
    for a,b in zip(left.get('ticks',[]),right.get('ticks',[])):
        if a['value']!=b['value']:
            assert set(a['value'])==set(b['value'])=={'Number'}
            x,y=a['value']['Number'],b['value']['Number']
            assert isinstance(x,(int,float)) and isinstance(y,(int,float)) and abs(x-y)<=3e-12*max(1.,abs(x),abs(y))
            a['value']=b['value']
        assert abs(a['position']-b['position'])/440<=3e-12
        a['position']=b['position']
    assert left==right
for states in [binned_position_python,binned_position_wasm]:
    assert [{k:v for k,v in r.items() if k!='route'} for r in states if r['route']=='axis']==[{k:v for k,v in r.items() if k!='route'} for r in states if r['route']=='policy']
binned_position_files=[p for p in (binned_position_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(binned_position_files)==18
for artifact in binned_position_files:assert artifact.read_bytes()==(binned_position_labels/'wasm'/artifact.name).read_bytes()

discrete_position_labels=output/'ggplot-positional-discrete-labels'
run('discrete-position-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_discrete_label_functions.py',module,discrete_position_labels/'python'])
run('discrete-position-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_discrete_label_functions.cjs',wasm,discrete_position_labels/'wasm'])
discrete_position_python=json.loads((discrete_position_labels/'python/records.json').read_text())
assert len(discrete_position_python)==4700 and discrete_position_python==json.loads((discrete_position_labels/'wasm/records.json').read_text())
assert [{k:v for k,v in r.items() if k!='route'} for r in discrete_position_python if r['route']=='axis']==[{k:v for k,v in r.items() if k!='route'} for r in discrete_position_python if r['route']=='policy']
discrete_position_files=[p for p in (discrete_position_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(discrete_position_files)==12
for artifact in discrete_position_files:assert artifact.read_bytes()==(discrete_position_labels/'wasm'/artifact.name).read_bytes()

temporal_position_labels=output/'ggplot-positional-temporal-labels'
run('temporal-position-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_temporal_label_functions.py',module,temporal_position_labels/'python'])
run('temporal-position-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_temporal_label_functions.cjs',wasm,temporal_position_labels/'wasm'])
temporal_position_python=json.loads((temporal_position_labels/'python/records.json').read_text())
assert len(temporal_position_python)==2365 and temporal_position_python==json.loads((temporal_position_labels/'wasm/records.json').read_text())
temporal_position_files=[p for p in (temporal_position_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(temporal_position_files)==12
for artifact in temporal_position_files:assert artifact.read_bytes()==(temporal_position_labels/'wasm'/artifact.name).read_bytes()

# Minor callbacks share the break registry and preserve source arity through both hosts.
minor_callbacks=output/'ggplot-positional-minor-break-functions'
run('minor-callbacks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_minor_break_functions.py',module,minor_callbacks/'python'])
run('minor-callbacks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_minor_break_functions.cjs',wasm,minor_callbacks/'wasm'])
run('minor-callbacks-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_minor_break_compare.py',minor_callbacks/'python',minor_callbacks/'wasm',minor_callbacks/'comparison.json'])
joint_minor_callbacks=output/'joint-minor-callbacks'
run('joint-minor-callbacks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_minor_break_functions.py',module,joint_minor_callbacks/'python','--joint'])
run('joint-minor-callbacks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_minor_break_functions.cjs',wasm,joint_minor_callbacks/'wasm','--joint'])
run('joint-minor-callbacks-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_minor_break_compare.py',joint_minor_callbacks/'python',joint_minor_callbacks/'wasm',joint_minor_callbacks/'comparison.json','--joint'])
other_minor_callbacks=output/'other-minor-callbacks'
run('other-minor-callbacks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_other_minor_break_functions.py',module,other_minor_callbacks/'python'])
run('other-minor-callbacks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_other_minor_break_functions.cjs',wasm,other_minor_callbacks/'wasm'])
run('other-minor-callbacks-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_other_minor_break_compare.py',other_minor_callbacks/'python',other_minor_callbacks/'wasm',other_minor_callbacks/'comparison.json'])
for temporal_minor_flag, temporal_minor_name in [([], 'temporal-minor-callbacks'), (['--overrides'], 'temporal-minor-width')]:
 temporal_minor_callbacks=output/temporal_minor_name
 run(temporal_minor_name+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_temporal_minor_break_functions.py',module,temporal_minor_callbacks/'python',*temporal_minor_flag])
 run(temporal_minor_name+'-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_temporal_minor_break_functions.cjs',wasm,temporal_minor_callbacks/'wasm',*temporal_minor_flag])
 run(temporal_minor_name+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_temporal_minor_break_compare.py',temporal_minor_callbacks/'python',temporal_minor_callbacks/'wasm',temporal_minor_callbacks/'comparison.json',*temporal_minor_flag])
discrete_palettes=output/'discrete-palette-functions'
run('discrete-palette-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_functions.py',module,discrete_palettes/'python'])
run('discrete-palette-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_palette_functions.cjs',wasm,discrete_palettes/'wasm'])
run('discrete-palette-functions-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',discrete_palettes/'python',discrete_palettes/'wasm',discrete_palettes/'comparison.json'])
binned_palettes=output/'binned-vector-palettes'
run('binned-vector-palettes-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_palette_functions.py',module,binned_palettes/'python'])
run('binned-vector-palettes-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_palette_functions.cjs',wasm,binned_palettes/'wasm'])
run('binned-vector-palettes-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_palettes/'python',binned_palettes/'wasm',binned_palettes/'comparison.json','1057'])
continuous_palettes=output/'continuous-vector-palettes'
run('continuous-vector-palettes-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_palette_functions.py',module,continuous_palettes/'python'])
run('continuous-vector-palettes-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_palette_functions.cjs',wasm,continuous_palettes/'wasm'])
run('continuous-vector-palettes-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',continuous_palettes/'python',continuous_palettes/'wasm',continuous_palettes/'comparison.json','1240'])
vector_pipeline=output/'continuous-vector-pipeline'
run('continuous-vector-pipeline-python',[sys.executable,ROOT/'scripts/bindings/ggplot_pipeline_functions.py',module,vector_pipeline/'python'])
run('continuous-vector-pipeline-wasm',[node,ROOT/'scripts/bindings/ggplot_pipeline_functions.cjs',wasm,vector_pipeline/'wasm'])
run('continuous-vector-pipeline-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',vector_pipeline/'python',vector_pipeline/'wasm',vector_pipeline/'comparison.json','283','36'])

binned_pipeline=output/'binned-vector-pipeline'
run('binned-vector-pipeline-python',[sys.executable,ROOT/'scripts/bindings/ggplot_pipeline_functions.py',module,binned_pipeline/'python','binned'])
run('binned-vector-pipeline-wasm',[node,ROOT/'scripts/bindings/ggplot_pipeline_functions.cjs',wasm,binned_pipeline/'wasm','binned'])
run('binned-vector-pipeline-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_pipeline/'python',binned_pipeline/'wasm',binned_pipeline/'comparison.json','283','36'])

theme_palettes=output/'theme-palettes'
run('theme-palette-core',['cargo','test','-p','chart-core','--test','ggplot_palette_selection','--locked'])
run('theme-palette-python',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_selection.py',module,theme_palettes/'python'])
run('theme-palette-wasm',[node,ROOT/'scripts/bindings/ggplot_palette_selection.cjs',wasm,theme_palettes/'wasm'])
run('theme-palette-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',theme_palettes/'python',theme_palettes/'wasm',theme_palettes/'comparison.json','979','54'])
continuous_guides=output/'continuous-guide-selection'
run('continuous-guide-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.py',module,continuous_guides/'python'])
run('continuous-guide-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.cjs',wasm,continuous_guides/'wasm'])
run('continuous-guide-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',continuous_guides/'python',continuous_guides/'wasm',continuous_guides/'comparison.json','243','27'])
colorbar_guides=output/'continuous-colorbar-selection'
run('continuous-colorbar-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.py',module,colorbar_guides/'python','--colorbar'])
run('continuous-colorbar-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.cjs',wasm,colorbar_guides/'wasm','--colorbar'])
run('continuous-colorbar-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_guides/'python',colorbar_guides/'wasm',colorbar_guides/'comparison.json','351','63'])
# GG-05 default sampled colorbars: native export and both actual host adapters.
colorbar_layout=output/'ggplot-colorbar-layout'
run('colorbar-layout-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_layout/'rust'])
run('colorbar-layout-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_layout/'python'])
run('colorbar-layout-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_layout/'wasm'])
run('colorbar-layout-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_layout/'python',colorbar_layout/'wasm',colorbar_layout/'comparison.json','108','27',colorbar_layout/'rust'])
colorbar_sampling=output/'ggplot-colorbar-sampling'
run('colorbar-sampling-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_sampling/'rust','--sampling'])
run('colorbar-sampling-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_sampling/'python','--sampling'])
run('colorbar-sampling-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_sampling/'wasm','--sampling'])
run('colorbar-sampling-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_sampling/'python',colorbar_sampling/'wasm',colorbar_sampling/'comparison.json','432','108',colorbar_sampling/'rust'])
run('colorbar-demand-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_sampling/'demand-python','--demand'])
run('colorbar-demand-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_sampling/'demand-wasm','--demand'])
run('colorbar-demand-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_sampling/'demand-python',colorbar_sampling/'demand-wasm',colorbar_sampling/'demand-comparison.json','60','0'])
run('colorbar-constant-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_sampling/'constant-rust','--constant'])
run('colorbar-constant-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_sampling/'constant-python','--constant'])
run('colorbar-constant-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_sampling/'constant-wasm','--constant'])
run('colorbar-constant-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_sampling/'constant-python',colorbar_sampling/'constant-wasm',colorbar_sampling/'constant-comparison.json','48','36',colorbar_sampling/'constant-rust'])
colorbar_presentation=output/'ggplot-colorbar-presentation'
run('colorbar-orientation-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_presentation/'orientation-rust','--orientation'])
run('colorbar-orientation-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_presentation/'orientation-python','--orientation'])
run('colorbar-orientation-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_presentation/'orientation-wasm','--orientation'])
run('colorbar-orientation-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_presentation/'orientation-python',colorbar_presentation/'orientation-wasm',colorbar_presentation/'orientation-comparison.json','192','144',colorbar_presentation/'orientation-rust'])
run('colorbar-presentation-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_presentation/'presentation-rust','--presentation'])
run('colorbar-presentation-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_presentation/'presentation-python','--presentation'])
run('colorbar-presentation-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_presentation/'presentation-wasm','--presentation'])
run('colorbar-presentation-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_presentation/'presentation-python',colorbar_presentation/'presentation-wasm',colorbar_presentation/'presentation-comparison.json','512','120',colorbar_presentation/'presentation-rust'])
colorbar_display=output/'ggplot-colorbar-display'
run('colorbar-display-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_display/'rust','--display'])
run('colorbar-display-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_display/'python','--display'])
run('colorbar-display-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_display/'wasm','--display'])
run('colorbar-display-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_display/'python',colorbar_display/'wasm',colorbar_display/'comparison.json','704','120',colorbar_display/'rust'])
colorbar_alpha=output/'colorbar-alpha';colorbar_alpha.mkdir(exist_ok=True)
run('colorbar-alpha-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorbar_alpha/'rust','--alpha'])
run('colorbar-alpha-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorbar_alpha/'python','--alpha'])
run('colorbar-alpha-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorbar_alpha/'wasm','--alpha'])
run('colorbar-alpha-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorbar_alpha/'python',colorbar_alpha/'wasm',colorbar_alpha/'comparison.json','384','72',colorbar_alpha/'rust'])
colorsteps=output/'colorsteps';colorsteps.mkdir(exist_ok=True)
run('colorsteps-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorsteps/'rust','--steps'])
run('colorsteps-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorsteps/'python','--steps'])
run('colorsteps-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorsteps/'wasm','--steps'])
run('colorsteps-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorsteps/'python',colorsteps/'wasm',colorsteps/'comparison.json','72','54',colorsteps/'rust'])

for guide_mode, guide_states, guide_pubs in [('steps-controls',711,180),('default-steps-all',93,27)]:
    guide_output=output/('ggplot-'+guide_mode)
    run(guide_mode+'-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',guide_output/'rust','--'+guide_mode])
    run(guide_mode+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,guide_output/'python','--'+guide_mode])
    run(guide_mode+'-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,guide_output/'wasm','--'+guide_mode])
    run(guide_mode+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',guide_output/'python',guide_output/'wasm',guide_output/'comparison.json',str(guide_states),str(guide_pubs),guide_output/'rust'])
src=(ROOT/'scripts/bindings/authoring/ggplot_guide_types.cts').read_text().replace('../../../packages/wasm/',str(wasm)+'/')
(consumer/'ggplot-guides.cts').write_text(src)
run('typescript-ggplot-guides',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-guides.cts'])
run('python-ggplot-guide-types',base+[ROOT/'scripts/bindings/authoring/ggplot_guide_typing.py'])
composition=output/'ggplot-guide-composition'
run('guide-composition-rust',['cargo','run','-p','chart-export','--example','ggplot_guide_composition','--locked','--',composition/'rust'])
run('guide-composition-python',[sys.executable,ROOT/'scripts/bindings/ggplot_guide_composition.py',module,composition/'python'])
run('guide-composition-wasm',[node,ROOT/'scripts/bindings/ggplot_guide_composition.cjs',wasm,composition/'wasm'])
for file in (composition/'rust').iterdir():
    if file.suffix in ('.svg','.pdf','.png'):
        assert file.read_bytes()==(composition/'python'/file.name).read_bytes()==(composition/'wasm'/file.name).read_bytes(),file
colorsteps_boundaries=output/'ggplot-colorsteps-boundaries'
run('colorsteps-boundaries-rust',['cargo','run','-p','chart-export','--example','ggplot_colorbar_layout','--locked','--',colorsteps_boundaries/'rust','--steps-boundaries'])
run('colorsteps-boundaries-python',[sys.executable,ROOT/'scripts/bindings/ggplot_colorbar_layout.py',module,colorsteps_boundaries/'python','--steps-boundaries'])
run('colorsteps-boundaries-wasm',[node,ROOT/'scripts/bindings/ggplot_colorbar_layout.cjs',wasm,colorsteps_boundaries/'wasm','--steps-boundaries'])
run('colorsteps-boundaries-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',colorsteps_boundaries/'python',colorsteps_boundaries/'wasm',colorsteps_boundaries/'comparison.json','180','48',colorsteps_boundaries/'rust'])


temporal_selection=output/'temporal-guide-selection'
run('temporal-guide-selection-core',['cargo','test','-p','chart-core','--test','ggplot_temporal_guide_selection','--locked'])
run('temporal-guide-selection-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.py',module,temporal_selection/'python'])
run('temporal-guide-selection-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.cjs',wasm,temporal_selection/'wasm'])
run('temporal-guide-selection-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',temporal_selection/'python',temporal_selection/'wasm',temporal_selection/'comparison.json','7680','198'])
run('temporal-step-controls-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.py',module,temporal_selection/'controls-python','--steps-controls'])
run('temporal-step-controls-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.cjs',wasm,temporal_selection/'controls-wasm','--steps-controls'])
assert json.loads((temporal_selection/'controls-python/records.json').read_text()) == json.loads((temporal_selection/'controls-wasm/records.json').read_text())
for source in (temporal_selection/'controls-python').iterdir():
    if source.suffix in ('.svg','.pdf','.png'):
        assert source.read_bytes() == (temporal_selection/'controls-wasm'/source.name).read_bytes(), source
run('temporal-interval-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.py',module,temporal_selection/'labels-python','--interval-labels'])
run('temporal-interval-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_guide_selection.cjs',wasm,temporal_selection/'labels-wasm','--interval-labels'])
run('temporal-interval-labels-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',temporal_selection/'labels-python',temporal_selection/'labels-wasm',temporal_selection/'labels-comparison.json','21872','180'])
interval_guides=output/'continuous-interval-guides'
run('continuous-interval-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.py',module,interval_guides/'python','--intervals'])
run('continuous-interval-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.cjs',wasm,interval_guides/'wasm','--intervals'])
run('continuous-interval-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',interval_guides/'python',interval_guides/'wasm',interval_guides/'comparison.json','216','72'])
interval_transforms=output/'continuous-interval-transforms'
run('continuous-interval-transform-core',['cargo','test','-p','chart-extension-example','--test','continuous_interval_transforms','--locked'])
run('continuous-interval-transform-python',[sys.executable,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.py',module,interval_transforms/'python','--transforms'])
run('continuous-interval-transform-wasm',[node,ROOT/'scripts/bindings/ggplot_continuous_guide_selection.cjs',wasm,interval_transforms/'wasm','--transforms'])
run('continuous-interval-transform-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',interval_transforms/'python',interval_transforms/'wasm',interval_transforms/'comparison.json','324','216'])
generic_guides=output/'generic-guide-suppression'
run('generic-guide-python',[sys.executable,ROOT/'scripts/bindings/ggplot_generic_guide_suppression.py',module,generic_guides/'python'])
run('generic-guide-wasm',[node,ROOT/'scripts/bindings/ggplot_generic_guide_suppression.cjs',wasm,generic_guides/'wasm'])
run('generic-guide-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',generic_guides/'python',generic_guides/'wasm',generic_guides/'comparison.json','972','108'])
theme_vectors=output/'theme-vectors'
run('theme-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_selection.py',module,theme_vectors/'python','--color-vectors'])
run('theme-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_palette_selection.cjs',wasm,theme_vectors/'wasm','--color-vectors'])
run('theme-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',theme_vectors/'python',theme_vectors/'wasm',theme_vectors/'comparison.json','438','54'])
default_palettes=output/'default-palettes'
run('default-palettes-core',['cargo','test','-p','chart-core','--test','ggplot_default_palettes','--locked'])
run('default-palettes-python',[sys.executable,ROOT/'scripts/bindings/ggplot_default_palettes.py',module,default_palettes/'python'])
run('default-palettes-wasm',[node,ROOT/'scripts/bindings/ggplot_default_palettes.cjs',wasm,default_palettes/'wasm'])
run('default-palettes-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',default_palettes/'python',default_palettes/'wasm',default_palettes/'comparison.json','168','30'])
numeric_constructors=output/'numeric-constructors'
run('numeric-constructors-core',['cargo','test','-p','chart-core','--test','ggplot_numeric_paint_constructors','--locked'])
run('numeric-constructors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_default_palettes.py',module,numeric_constructors/'python','--numeric-constructors'])
run('numeric-constructors-wasm',[node,ROOT/'scripts/bindings/ggplot_default_palettes.cjs',wasm,numeric_constructors/'wasm','--numeric-constructors'])
run('numeric-constructors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',numeric_constructors/'python',numeric_constructors/'wasm',numeric_constructors/'comparison.json','484','108'])
shared_paint=output/'shared-paint-aesthetics'
run('shared-paint-core',['cargo','test','-p','chart-core','--test','ggplot_shared_paint_aesthetics','--locked'])
run('shared-paint-python',[sys.executable,ROOT/'scripts/bindings/ggplot_shared_paint_aesthetics.py',module,shared_paint/'python'])
run('shared-paint-wasm',[node,ROOT/'scripts/bindings/ggplot_shared_paint_aesthetics.cjs',wasm,shared_paint/'wasm'])
run('shared-paint-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',shared_paint/'python',shared_paint/'wasm',shared_paint/'comparison.json','45','15'])
# GG-04 built-in transforms, guide controls, finite secondary axes and wire versions.
run('builtin-transforms-core',['cargo','test','-p','chart-core','--test','ggplot_transform_contracts','--locked'])
for route, flags, states, files in [('original', [], 342, 171), ('labels', ['--labels'], 348, 342), ('secondary', ['--secondary'], 87, 81)]:
    transforms=output/'builtin-transforms'/route
    run('builtin-transforms-'+route+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,transforms/'python']+flags)
    run('builtin-transforms-'+route+'-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,transforms/'wasm']+flags)
    run('builtin-transforms-'+route+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',transforms/'python',transforms/'wasm',transforms/'comparison.json',str(states),str(files)])
identity_transforms=output/'identity-transform-guides'
run('identity-transform-guides-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,identity_transforms/'python','--identity-guides'])
run('identity-transform-guides-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,identity_transforms/'wasm','--identity-guides'])
run('identity-transform-guides-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',identity_transforms/'python',identity_transforms/'wasm',identity_transforms/'comparison.json','696','24'])
identity_vectors=output/'identity-vector-transforms'
run('identity-vector-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','identity_vector_transform_mapping','--locked'])
run('identity-vector-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,identity_vectors/'python','--identity-vectors'])
run('identity-vector-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,identity_vectors/'wasm','--identity-vectors'])
run('identity-vector-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',identity_vectors/'python',identity_vectors/'wasm',identity_vectors/'comparison.json','288','18'])
identity_functions=output/'identity-vector-functions'
run('identity-functions-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','identity_vector_callbacks','--locked'])
run('identity-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,identity_functions/'python','--identity-functions'])
run('identity-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,identity_functions/'wasm','--identity-functions'])
run('identity-functions-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',identity_functions/'python',identity_functions/'wasm',identity_functions/'comparison.json','864','18'])
vector_functions=output/'vector-scale-functions'
run('vector-functions-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','vector_scale_callbacks','--locked'])
run('vector-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,vector_functions/'python','--scale-functions'])
run('vector-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,vector_functions/'wasm','--scale-functions'])
run('vector-functions-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',vector_functions/'python',vector_functions/'wasm',vector_functions/'comparison.json','3168','39'])
null_breaks=output/'vector-null-breaks'
run('transform-null-core',['cargo','test','-p','chart-core','--lib','builtin_null_and_typed_empty','--locked'])
run('vector-null-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','vector_null_breaks','--locked'])
run('vector-null-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,null_breaks/'python','--null-breaks'])
run('vector-null-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,null_breaks/'wasm','--null-breaks'])
run('vector-null-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',null_breaks/'python',null_breaks/'wasm',null_breaks/'comparison.json','1056','36'])
position_functions=output/'vector-position-callbacks'
run('vector-position-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','vector_position_callbacks','--locked'])
run('vector-position-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,position_functions/'python','--position-functions'])
run('vector-position-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,position_functions/'wasm','--position-functions'])
run('vector-position-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',position_functions/'python',position_functions/'wasm',position_functions/'comparison.json','180','27'])
vector_position_explicit=output/'vector-position-explicit'
run('vector-position-explicit-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','vector_position_explicit','--locked'])
run('vector-position-explicit-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,vector_position_explicit/'python','--position-explicit'])
run('vector-position-explicit-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,vector_position_explicit/'wasm','--position-explicit'])
run('vector-position-explicit-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',vector_position_explicit/'python',vector_position_explicit/'wasm',vector_position_explicit/'comparison.json','360','54'])
vector_position_minors=output/'vector-position-minors'
run('vector-position-minors-core',['cargo','test','-p','chart-extension-example','--test','vector_transform_plots','vector_position_minor','--locked'])
run('vector-position-minors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,vector_position_minors/'python','--position-minors'])
run('vector-position-minors-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,vector_position_minors/'wasm','--position-minors'])
run('vector-position-minors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',vector_position_minors/'python',vector_position_minors/'wasm',vector_position_minors/'comparison.json','600','36'])
transform_wire=output/'builtin-transform-wire'
run('builtin-transform-wire-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transform_wire.py',module,transform_wire/'python'])
run('builtin-transform-wire-wasm',[node,ROOT/'scripts/bindings/ggplot_transform_wire.cjs',wasm,transform_wire/'wasm'])
assert json.loads((transform_wire/'python/states.json').read_text()) == json.loads((transform_wire/'wasm/states.json').read_text())
composed=output/'composed-transforms'
run('composed-transforms-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,composed/'python','--compositions'])
run('composed-transforms-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,composed/'wasm','--compositions'])
run('composed-transforms-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',composed/'python',composed/'wasm',composed/'comparison.json','543','177'])
composed_wire=output/'composed-transform-wire'
run('composed-transform-wire-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transform_wire.py',module,composed_wire/'python','--compositions'])
run('composed-transform-wire-wasm',[node,ROOT/'scripts/bindings/ggplot_transform_wire.cjs',wasm,composed_wire/'wasm','--compositions'])
assert json.loads((composed_wire/'python/states.json').read_text()) == json.loads((composed_wire/'wasm/states.json').read_text())
probability_transforms=output/'probability-transforms'
run('probability-transforms-native',['cargo','test','-p','chart-extension-example','--test','probability_transforms','--locked'])
run('probability-transforms-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,probability_transforms/'python','--probability'])
run('probability-transforms-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,probability_transforms/'wasm','--probability'])
run('probability-transforms-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',probability_transforms/'python',probability_transforms/'wasm',probability_transforms/'comparison.json','108','36'])
registered_transforms=output/'registered-transforms'
run('registered-transforms-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,registered_transforms/'python','--registered'])
run('registered-transforms-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,registered_transforms/'wasm','--registered'])
run('registered-transforms-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',registered_transforms/'python',registered_transforms/'wasm',registered_transforms/'comparison.json','108','36'])
transform_minors=output/'transform-minors'
run('transform-minors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,transform_minors/'python','--minors'])
run('transform-minors-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,transform_minors/'wasm','--minors'])
run('transform-minors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',transform_minors/'python',transform_minors/'wasm',transform_minors/'comparison.json','360','36'])
transform_vectors=output/'transform-vectors'
run('transform-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,transform_vectors/'python','--vectors'])
run('transform-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,transform_vectors/'wasm','--vectors'])
run('transform-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',transform_vectors/'python',transform_vectors/'wasm',transform_vectors/'comparison.json','180','54'])
vector_statistic_styles=output/'vector-statistic-styles'
run('vector-statistic-styles-python',[sys.executable,ROOT/'scripts/bindings/ggplot_vector_statistic_styles.py',module,vector_statistic_styles/'python'])
run('vector-statistic-styles-wasm',[node,ROOT/'scripts/bindings/ggplot_vector_statistic_styles.cjs',wasm,vector_statistic_styles/'wasm'])
run('vector-statistic-styles-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',vector_statistic_styles/'python',vector_statistic_styles/'wasm',vector_statistic_styles/'comparison.json','44','36'])
retained_vector_styles=output/'retained-vector-styles'
run('retained-vector-styles-python',[sys.executable,ROOT/'scripts/bindings/ggplot_vector_statistic_styles.py',module,retained_vector_styles/'python','--retained'])
run('retained-vector-styles-wasm',[node,ROOT/'scripts/bindings/ggplot_vector_statistic_styles.cjs',wasm,retained_vector_styles/'wasm','--retained'])
run('retained-vector-styles-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',retained_vector_styles/'python',retained_vector_styles/'wasm',retained_vector_styles/'comparison.json','44','36'])
retained_vectors=output/'retained-vectors'
run('retained-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transforms.py',module,retained_vectors/'python','--vectors','--retained'])
run('retained-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_transforms.cjs',wasm,retained_vectors/'wasm','--vectors','--retained'])
run('retained-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',retained_vectors/'python',retained_vectors/'wasm',retained_vectors/'comparison.json','180','54'])
registered_transform_wire=output/'registered-transform-wire'
run('registered-transform-wire-python',[sys.executable,ROOT/'scripts/bindings/ggplot_transform_wire.py',module,registered_transform_wire/'python','--registered'])
run('registered-transform-wire-wasm',[node,ROOT/'scripts/bindings/ggplot_transform_wire.cjs',wasm,registered_transform_wire/'wasm','--registered'])
assert json.loads((registered_transform_wire/'python/states.json').read_text()) == json.loads((registered_transform_wire/'wasm/states.json').read_text())
binned_shape_defaults=output/'binned-shape-defaults'
run('binned-shape-defaults-native',['cargo','test','-p','chart-extension-example','--test','binned_style_defaults','--locked'])
run('binned-shape-defaults-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_style_defaults.py',module,binned_shape_defaults/'python'])
run('binned-shape-defaults-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_style_defaults.cjs',wasm,binned_shape_defaults/'wasm'])
run('binned-shape-defaults-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_shape_defaults/'python',binned_shape_defaults/'wasm',binned_shape_defaults/'comparison.json','134','30'])

position_palettes=output/'positional-palette-functions'
run('position-palettes-core',['cargo','test','-p','chart-extension-example','--test','positional_palette_functions','--locked'])
run('position-palettes-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_palette_functions.py',module,position_palettes/'python'])
run('position-palettes-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_palette_functions.cjs',wasm,position_palettes/'wasm'])
run('position-palettes-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',position_palettes/'python',position_palettes/'wasm',position_palettes/'comparison.json','298','18'])
secondary_guides=output/'secondary-guide-functions'
run('secondary-guide-functions-core',['cargo','test','-p','chart-extension-example','--test','secondary_guide_functions','--locked'])
run('secondary-guide-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_secondary_guide_functions.py',module,secondary_guides/'python'])
run('secondary-guide-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_secondary_guide_functions.cjs',wasm,secondary_guides/'wasm'])
run('secondary-guide-functions-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',secondary_guides/'python',secondary_guides/'wasm',secondary_guides/'comparison.json','1266','24'])
default_style=output/'default-style-palettes'
run('default-style-core',['cargo','test','-p','chart-core','--test','ggplot_default_style_palettes','--locked'])
run('default-style-python',[sys.executable,ROOT/'scripts/bindings/ggplot_default_style_palettes.py',module,default_style/'python'])
run('default-style-wasm',[node,ROOT/'scripts/bindings/ggplot_default_style_palettes.cjs',wasm,default_style/'wasm'])
run('default-style-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',default_style/'python',default_style/'wasm',default_style/'comparison.json','144','36'])
default_ordinal=output/'default-ordinal-palettes'
run('default-ordinal-python',[sys.executable,ROOT/'scripts/bindings/ggplot_default_palettes.py',module,default_ordinal/'python','--ordinal'])
run('default-ordinal-wasm',[node,ROOT/'scripts/bindings/ggplot_default_palettes.cjs',wasm,default_ordinal/'wasm','--ordinal'])
run('default-ordinal-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',default_ordinal/'python',default_ordinal/'wasm',default_ordinal/'comparison.json','60','30'])
default_temporal=output/'default-temporal-palettes'
run('default-temporal-python',[sys.executable,ROOT/'scripts/bindings/ggplot_default_palettes.py',module,default_temporal/'python','--temporal'])
run('default-temporal-wasm',[node,ROOT/'scripts/bindings/ggplot_default_palettes.cjs',wasm,default_temporal/'wasm','--temporal'])
run('default-temporal-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',default_temporal/'python',default_temporal/'wasm',default_temporal/'comparison.json','180','30'])
run('named-palettes-core',['cargo','test','-p','chart-core','--test','ggplot_named_palettes','--locked'])
for scalar in (False,True):
    named_palettes=output/('named-palettes-scalar' if scalar else 'named-palettes')
    flags=['--named-palettes']+(['--scalar-names'] if scalar else [])
    run(named_palettes.name+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_selection.py',module,named_palettes/'python',*flags])
    run(named_palettes.name+'-wasm',[node,ROOT/'scripts/bindings/ggplot_palette_selection.cjs',wasm,named_palettes/'wasm',*flags])
    run(named_palettes.name+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',named_palettes/'python',named_palettes/'wasm',named_palettes/'comparison.json','4183','54'])
ordinal_types=output/'ordinal-type-palettes'
run('ordinal-types-core',['cargo','test','-p','chart-core','--test','ggplot_ordinal_types','--locked'])
run('ordinal-types-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,ordinal_types/'python','--ordinal-types'])
run('ordinal-types-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,ordinal_types/'wasm','--ordinal-types'])
run('ordinal-types-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',ordinal_types/'python',ordinal_types/'wasm',ordinal_types/'comparison.json','154','30'])
qualitative_types=output/'qualitative-type-palettes'
run('qualitative-types-core',['cargo','test','-p','chart-core','--test','ggplot_qualitative_types','--locked'])
run('qualitative-types-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,qualitative_types/'python','--qualitative-types'])
run('qualitative-types-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,qualitative_types/'wasm','--qualitative-types'])
run('qualitative-types-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',qualitative_types/'python',qualitative_types/'wasm',qualitative_types/'comparison.json','206','48'])
discrete_constructors=output/'discrete-paint-constructors'
run('discrete-constructors-core',['cargo','test','-p','chart-core','--test','ggplot_discrete_paint_constructors','--locked'])
run('discrete-constructors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,discrete_constructors/'python','--discrete-constructors'])
run('discrete-constructors-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,discrete_constructors/'wasm','--discrete-constructors'])
run('discrete-constructors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',discrete_constructors/'python',discrete_constructors/'wasm',discrete_constructors/'comparison.json','208','48'])
continuous_constructors=output/'continuous-paint-constructors'
run('continuous-constructors-core',['cargo','test','-p','chart-core','--test','ggplot_continuous_paint_constructors','--locked'])
run('continuous-constructors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,continuous_constructors/'python','--continuous-constructors'])
run('continuous-constructors-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,continuous_constructors/'wasm','--continuous-constructors'])
run('continuous-constructors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',continuous_constructors/'python',continuous_constructors/'wasm',continuous_constructors/'comparison.json','480','120'])
gradient_remap=output/'gradient-remap'
run('gradient-remap-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,gradient_remap/'python','--gradient-remap'])
run('gradient-remap-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,gradient_remap/'wasm','--gradient-remap'])
run('gradient-remap-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',gradient_remap/'python',gradient_remap/'wasm',gradient_remap/'comparison.json','288','96'])
gradient_nonfinite=output/'gradient-nonfinite'
run('gradient-nonfinite-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,gradient_nonfinite/'python','--gradient-nonfinite'])
run('gradient-nonfinite-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,gradient_nonfinite/'wasm','--gradient-nonfinite'])
run('gradient-nonfinite-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',gradient_nonfinite/'python',gradient_nonfinite/'wasm',gradient_nonfinite/'comparison.json','468','156'])
gradient_invalid=output/'gradient-invalid'
run('gradient-invalid-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,gradient_invalid/'python','--gradient-invalid'])
run('gradient-invalid-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,gradient_invalid/'wasm','--gradient-invalid'])
run('gradient-invalid-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',gradient_invalid/'python',gradient_invalid/'wasm',gradient_invalid/'comparison.json','240','6'])
binned_constructors=output/'binned-constructor-palettes'
run('binned-constructor-core',['cargo','test','-p','chart-core','--test','ggplot_binned_constructor_palettes','--test','ggplot_hue_counts','--locked'])
run('binned-constructor-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,binned_constructors/'python'])
run('binned-constructor-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,binned_constructors/'wasm'])
run('binned-constructor-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_constructors/'python',binned_constructors/'wasm',binned_constructors/'comparison.json','288','30'])
binned_count_functions=output/'binned-constructor-functions'
run('binned-count-functions-core',['cargo','test','-p','chart-core','--test','ggplot_binned_constructor_functions','--locked'])
run('binned-count-functions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.py',module,binned_count_functions/'python','--functions'])
run('binned-count-functions-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_constructor_palettes.cjs',wasm,binned_count_functions/'wasm','--functions'])
run('binned-count-functions-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_count_functions/'python',binned_count_functions/'wasm',binned_count_functions/'comparison.json','324','36'])
src=(ROOT/'scripts/bindings/authoring/ggplot_palette_selection_types.cts').read_text().replace('../../../target/ggplot-theme-palette/wasm-module/','../wasm-module/')
(consumer/'ggplot-palette-selection.cts').write_text(src)
run('typescript-palette-selection',tsc+['--noEmit','--strict','--target','es2022','--lib','es2022,esnext.disposable','--module','nodenext',consumer/'ggplot-palette-selection.cts'])
run('python-palette-selection-types',base+[ROOT/'scripts/bindings/authoring/ggplot_palette_selection_typing.py'])

binned_selection=output/'binned-guide-selection'
run('binned-guide-selection-python',[sys.executable,ROOT/'scripts/bindings/ggplot_pipeline_functions.py',module,binned_selection/'python','binned','--guide-selection'])
run('binned-guide-selection-wasm',[node,ROOT/'scripts/bindings/ggplot_pipeline_functions.cjs',wasm,binned_selection/'wasm','binned','--guide-selection'])
run('binned-guide-selection-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_selection/'python',binned_selection/'wasm',binned_selection/'comparison.json','1068','168'])
binned_label_pipelines=output/'binned-guide-label-pipelines'
run('binned-guide-label-pipelines-python',[sys.executable,ROOT/'scripts/bindings/ggplot_pipeline_functions.py',module,binned_label_pipelines/'python','binned','--guide-label-pipelines'])
run('binned-guide-label-pipelines-wasm',[node,ROOT/'scripts/bindings/ggplot_pipeline_functions.cjs',wasm,binned_label_pipelines/'wasm','binned','--guide-label-pipelines'])
run('binned-guide-label-pipelines-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',binned_label_pipelines/'python',binned_label_pipelines/'wasm',binned_label_pipelines/'comparison.json','268','54'])

for name, runner, count in [('continuous-compositions','ggplot_pipeline_compositions',3012), ('positional-vectors','ggplot_positional_vector_functions',732), ('positional-facet-vectors','ggplot_positional_facet_vectors',425), ('temporal-positional-vectors','ggplot_temporal_vector_functions',1727), ('binned-positional-vectors','ggplot_binned_vector_functions',1164)]:
    destination=output/name
    run(name+'-python',[sys.executable,ROOT/f'scripts/bindings/{runner}.py',module,destination/'python'])
    run(name+'-wasm',[node,ROOT/f'scripts/bindings/{runner}.cjs',wasm,destination/'wasm'])
    run(name+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',destination/'python',destination/'wasm',destination/'comparison.json',str(count),'36'])

for name, flags, count, files in [('panel-scope-vectors', [], 208, 48), ('source-chain-vectors', ['--identity-chain'], 208, 48), ('mixed-source-vectors', ['--mixed-routes'], 520, 108)]:
    destination=output/name
    run(name+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_panel_scope_vectors.py',module,destination/'python',*flags])
    run(name+'-wasm',[node,ROOT/'scripts/bindings/ggplot_panel_scope_vectors.cjs',wasm,destination/'wasm',*flags])
    run(name+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',destination/'python',destination/'wasm',destination/'comparison.json',str(count),str(files)])

for name, flags, count, files in [('binned-facet-vectors',['--binned'],704,36), ('shared-binned-facet-vectors',['--binned','--shared'],346,18), ('broadcast-vectors',['--broadcast'],384,36), ('shared-broadcast-vectors',['--broadcast','--shared'],188,18), ('binned-broadcast-vectors',['--broadcast','--binned'],752,36), ('shared-binned-broadcast-vectors',['--broadcast','--binned','--shared'],364,18)]:
    destination=output/name
    run(name+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_facet_vectors.py',module,destination/'python',*flags])
    run(name+'-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_facet_vectors.cjs',wasm,destination/'wasm',*flags])
    run(name+'-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',destination/'python',destination/'wasm',destination/'comparison.json',str(count),str(files)])

typed_helpers=output/'typed-scale-limit-helpers'
run('typed-helpers-python',[sys.executable,ROOT/'scripts/bindings/ggplot_typed_limit_helpers.py',module,typed_helpers/'python'])
run('typed-helpers-wasm',[node,ROOT/'scripts/bindings/ggplot_typed_limit_helpers.cjs',wasm,typed_helpers/'wasm'])
run('typed-helpers-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',typed_helpers/'python',typed_helpers/'wasm',typed_helpers/'comparison.json','181','48'])

named_helpers=output/'named-scale-limit-helpers'
run('named-helpers-python',[sys.executable,ROOT/'scripts/bindings/ggplot_typed_limit_helpers.py',module,named_helpers/'python','--named-limits'])
run('named-helpers-wasm',[node,ROOT/'scripts/bindings/ggplot_typed_limit_helpers.cjs',wasm,named_helpers/'wasm','--named-limits'])
run('named-helpers-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',named_helpers/'python',named_helpers/'wasm',named_helpers/'comparison.json','181','48'])

blank_helpers=output/'blank-scale-helpers'
run('blank-helpers-python',[sys.executable,ROOT/'scripts/bindings/ggplot_blank_helpers.py',module,blank_helpers/'python'])
run('blank-helpers-wasm',[node,ROOT/'scripts/bindings/ggplot_blank_helpers.cjs',wasm,blank_helpers/'wasm'])
run('blank-helpers-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',blank_helpers/'python',blank_helpers/'wasm',blank_helpers/'comparison.json','96','54'])

joint_bin_vectors=output/'joint-positional-bin-vectors'
run('joint-bin-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_vector_functions.py',module,joint_bin_vectors/'python','--joint'])
run('joint-bin-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_vector_functions.cjs',wasm,joint_bin_vectors/'wasm','--joint'])
run('joint-bin-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',joint_bin_vectors/'python',joint_bin_vectors/'wasm',joint_bin_vectors/'comparison.json','2692','36'])

shared_vectors=output/'shared-positional-vectors'
run('shared-positional-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_vector_functions.py',module,shared_vectors/'python','--shared'])
run('shared-positional-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_vector_functions.cjs',wasm,shared_vectors/'wasm','--shared'])
run('shared-positional-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',shared_vectors/'python',shared_vectors/'wasm',shared_vectors/'comparison.json','102','27'])

shared_facets=output/'shared-positional-facet-vectors'
run('shared-positional-facet-vectors-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_facet_vectors.py',module,shared_facets/'python','--shared'])
run('shared-positional-facet-vectors-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_facet_vectors.cjs',wasm,shared_facets/'wasm','--shared'])
run('shared-positional-facet-vectors-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_palette_compare.py',shared_facets/'python',shared_facets/'wasm',shared_facets/'comparison.json','186','18'])

# Temporal positional callbacks preserve typed values and source formatter precedence.
for name, flag, count in [('functions',None,27025), ('overrides','--overrides',1280), ('zero','--zero-range',400)]:
    temporal_breaks=output/('ggplot-positional-temporal-break-'+name)
    args=[] if flag is None else [flag]
    run('temporal-position-breaks-'+name+'-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_temporal_break_functions.py',module,temporal_breaks/'python',*args])
    run('temporal-position-breaks-'+name+'-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_temporal_break_functions.cjs',wasm,temporal_breaks/'wasm',*args])
    records=json.loads((temporal_breaks/'python/records.json').read_text())
    assert len(records)==count and records==json.loads((temporal_breaks/'wasm/records.json').read_text())
    if flag is None:
        publications=[p for p in (temporal_breaks/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
        assert len(publications)==12
        for artifact in publications:assert artifact.read_bytes()==(temporal_breaks/'wasm'/artifact.name).read_bytes()

# Named break results also supply default labels; explicit temporal formats retain precedence.
for family, script, count in [('numeric','ggplot_break_functions',160), ('binned','ggplot_binned_break_functions',116), ('positional','ggplot_positional_break_functions',80), ('temporal','ggplot_temporal_break_functions',800)]:
    named=output/('ggplot-'+family+'-break-default-names')
    run(f'{family}-break-default-names-python',[sys.executable,ROOT/'scripts/bindings'/f'{script}.py',module,named/'python','--default-names'])
    run(f'{family}-break-default-names-wasm',[node,ROOT/'scripts/bindings'/f'{script}.cjs',wasm,named/'wasm','--default-names'])
    named_records=json.loads((named/'python/records.json').read_text())
    assert len(named_records)==count and named_records==json.loads((named/'wasm/records.json').read_text())
    if family=='positional':
        artifacts=[p for p in (named/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
        assert len(artifacts)==12
        for artifact in artifacts:assert artifact.read_bytes()==(named/'wasm'/artifact.name).read_bytes()

named_overrides=output/'ggplot-temporal-break-default-overrides'
run('named-overrides-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_break_functions.py',module,named_overrides/'python','overrides','--default-names'])
run('named-overrides-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_break_functions.cjs',wasm,named_overrides/'wasm','overrides','--default-names'])
named_override_records=json.loads((named_overrides/'python/records.json').read_text())
assert len(named_override_records)==240 and named_override_records==json.loads((named_overrides/'wasm/records.json').read_text())

joint_binned=output/'ggplot-positional-binned-joint-functions'
run('joint-binned-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_binned_break_functions.py',module,joint_binned/'python','--joint'])
run('joint-binned-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_binned_break_functions.cjs',wasm,joint_binned/'wasm','--joint'])
run('joint-binned-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_binned_break_compare.py',joint_binned/'python',joint_binned/'wasm',joint_binned/'comparison.json','--joint'])

positional_binned_breaks=output/'ggplot-positional-binned-break-functions'
run('positional-binned-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_binned_break_functions.py',module,positional_binned_breaks/'python'])
run('positional-binned-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_binned_break_functions.cjs',wasm,positional_binned_breaks/'wasm'])
run('positional-binned-breaks-compare',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_binned_break_compare.py',positional_binned_breaks/'python',positional_binned_breaks/'wasm',positional_binned_breaks/'comparison.json'])

positional_discrete_breaks=output/'ggplot-positional-discrete-break-functions'
run('positional-discrete-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_discrete_break_functions.py',module,positional_discrete_breaks/'python'])
run('positional-discrete-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_discrete_break_functions.cjs',wasm,positional_discrete_breaks/'wasm'])
positional_discrete_break_records=json.loads((positional_discrete_breaks/'python/records.json').read_text())
assert len(positional_discrete_break_records)==3660
assert positional_discrete_break_records == json.loads((positional_discrete_breaks/'wasm/records.json').read_text())
positional_discrete_break_files=[p for p in (positional_discrete_breaks/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(positional_discrete_break_files)==12
for artifact in positional_discrete_break_files:assert artifact.read_bytes()==(positional_discrete_breaks/'wasm'/artifact.name).read_bytes()

positional_breaks=output/'ggplot-positional-break-functions'
run('positional-breaks-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_break_functions.py',module,positional_breaks/'python'])
run('positional-breaks-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_break_functions.cjs',wasm,positional_breaks/'wasm'])
positional_break_records=json.loads((positional_breaks/'python/records.json').read_text())
assert len(positional_break_records)==2128
assert positional_break_records == json.loads((positional_breaks/'wasm/records.json').read_text())
positional_break_files=[p for p in (positional_breaks/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(positional_break_files)==12
for artifact in positional_break_files:assert artifact.read_bytes()==(positional_breaks/'wasm'/artifact.name).read_bytes()

positional_labels=output/'ggplot-positional-labels'
run('positional-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_positional_label_functions.py',module,positional_labels/'python'])
run('positional-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_positional_label_functions.cjs',wasm,positional_labels/'wasm'])
positional_python=json.loads((positional_labels/'python/records.json').read_text())
assert len(positional_python)==708
assert positional_python == json.loads((positional_labels/'wasm/records.json').read_text())
positional_publications=[p.name for p in (positional_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(positional_publications)==15
for name in positional_publications:
    assert (positional_labels/'python'/name).read_bytes() == (positional_labels/'wasm'/name).read_bytes()
binned_labels=output/'ggplot-binned-labels'
run('binned-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_label_functions.py',module,binned_labels/'python'])
run('binned-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_label_functions.cjs',wasm,binned_labels/'wasm'])
binned_python=json.loads((binned_labels/'python/records.json').read_text())
binned_wasm=json.loads((binned_labels/'wasm/records.json').read_text())
assert len(binned_python)==len(binned_wasm)==1042
for a,b in zip(binned_python,binned_wasm):
    assert len(a.get('entries',[]))==len(b.get('entries',[]))
    for ae,be in zip(a.get('entries',[]),b.get('entries',[])):
        for field in ('value','transformed'):
            if ae[field]!=be[field]:
                assert isinstance(ae[field],(float,int)) and isinstance(be[field],(float,int))
                assert abs(ae[field]-be[field])<=3e-12*max(1,abs(ae[field]))
                be[field]=ae[field]
    assert a==b
binned_label_publications=[p.name for p in (binned_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(binned_label_publications)==12
for name in binned_label_publications:
    assert (binned_labels/'python'/name).read_bytes() == (binned_labels/'wasm'/name).read_bytes()
temporal_labels=output/'ggplot-temporal-labels'
run('temporal-labels-python',[sys.executable,ROOT/'scripts/bindings/ggplot_temporal_label_functions.py',module,temporal_labels/'python'])
run('temporal-labels-wasm',[node,ROOT/'scripts/bindings/ggplot_temporal_label_functions.cjs',wasm,temporal_labels/'wasm'])
assert json.loads((temporal_labels/'python/records.json').read_text()) == json.loads((temporal_labels/'wasm/records.json').read_text())
temporal_label_publications=[p.name for p in (temporal_labels/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(temporal_label_publications)==12
for name in temporal_label_publications:
    assert (temporal_labels/'python'/name).read_bytes() == (temporal_labels/'wasm'/name).read_bytes()
calendar_time=output/'ggplot-calendar-time'
run('calendar-time-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_temporal.py',module,calendar_time/'python','calendar'])
run('calendar-time-wasm',[node,ROOT/'scripts/bindings/ggplot_position_temporal.cjs',wasm,calendar_time/'wasm','calendar'])
assert json.loads((calendar_time/'python/position-temporal-records.json').read_text()) == json.loads((calendar_time/'wasm/position-temporal-records.json').read_text())
calendar_publications=[p.name for p in (calendar_time/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(calendar_publications)==12
for name in calendar_publications:
    assert (calendar_time/'python'/name).read_bytes() == (calendar_time/'wasm'/name).read_bytes()
auto_time=output/'ggplot-auto-time'
for proof,script,record_name in [('temporal','ggplot_position_temporal','position-temporal-records.json'),('facets','ggplot_position_facets','position-facet-records.json')]:
    destination=auto_time/proof
    run(f'auto-time-{proof}-python',[sys.executable,ROOT/f'scripts/bindings/{script}.py',module,destination/'python','automatic'])
    run(f'auto-time-{proof}-wasm',[node,ROOT/f'scripts/bindings/{script}.cjs',wasm,destination/'wasm','automatic'])
    assert json.loads((destination/'python'/record_name).read_text()) == json.loads((destination/'wasm'/record_name).read_text())
    publications=[p.name for p in (destination/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
    assert len(publications)==(18 if proof=='temporal' else 12)
    for name in publications:
        assert (destination/'python'/name).read_bytes() == (destination/'wasm'/name).read_bytes()
authored_facets=output/'ggplot-facet-authored'
run('authored-facets-python',[sys.executable,ROOT/'scripts/bindings/ggplot_position_facets.py',module,authored_facets/'python','authored'])
run('authored-facets-wasm',[node,ROOT/'scripts/bindings/ggplot_position_facets.cjs',wasm,authored_facets/'wasm','authored'])
assert json.loads((authored_facets/'python/position-facet-records.json').read_text()) == json.loads((authored_facets/'wasm/position-facet-records.json').read_text())
for policy in ('fixed','free'):
    for transform in ('identity','sqrt','reverse'):
        for context in ('points','summary'):
            for fmt in ('svg','pdf','png'):
                name=f'{context}-{transform}-{policy}-missing.{fmt}'
                assert (authored_facets/'python'/name).read_bytes() == (authored_facets/'wasm'/name).read_bytes()
unbounded_positions=output/'ggplot-unbounded'
run('unbounded-positions-core',['cargo','test','-p','chart-extension-example','--test','unbounded_positions','--locked'])
run('unbounded-positions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_unbounded_positions.py',module,unbounded_positions/'python'])
run('unbounded-positions-wasm',[node,ROOT/'scripts/bindings/ggplot_unbounded_positions.cjs',wasm,unbounded_positions/'wasm'])
assert json.loads((unbounded_positions/'python/unbounded-position-records.json').read_text()) == json.loads((unbounded_positions/'wasm/unbounded-position-records.json').read_text())
for kind,controls in [('continuous',['upper']),('binned',['upper','finite'])]:
    for transform in ('identity','sqrt','reverse','log10'):
        for control in controls:
            for fmt in ('svg','pdf','png'):
                sample_control='lower' if kind=='binned' and transform=='log10' and control=='finite' else control
                name=f'{kind}-{transform}-{sample_control}.{fmt}'
                assert (unbounded_positions/'python'/name).read_bytes() == (unbounded_positions/'wasm'/name).read_bytes()
authored_positions=output/'ggplot-authored'
run('authored-positions-python',[sys.executable,ROOT/'scripts/bindings/ggplot_unbounded_positions.py',module,authored_positions/'python','authored'])
run('authored-positions-wasm',[node,ROOT/'scripts/bindings/ggplot_unbounded_positions.cjs',wasm,authored_positions/'wasm','authored'])
assert json.loads((authored_positions/'python/unbounded-position-records.json').read_text()) == json.loads((authored_positions/'wasm/unbounded-position-records.json').read_text())
authored_publications=[p.name for p in (authored_positions/'python').iterdir() if p.suffix in ('.svg','.pdf','.png')]
assert len(authored_publications)==54
for name in authored_publications:
    assert (authored_positions/'python'/name).read_bytes() == (authored_positions/'wasm'/name).read_bytes()
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
binned_numeric_constructors=output/'binned-numeric-constructors'
run('binned-numeric-constructor-python',[sys.executable,ROOT/'scripts/bindings/ggplot_binned_numeric.py',module,binned_numeric_constructors/'python','--constructor-guides'])
run('binned-numeric-constructor-wasm',[node,ROOT/'scripts/bindings/ggplot_binned_numeric.cjs',wasm,binned_numeric_constructors/'wasm','--constructor-guides'])
left=json.loads((binned_numeric_constructors/'python/binned-numeric-records.json').read_text())
right=json.loads((binned_numeric_constructors/'wasm/binned-numeric-records.json').read_text())
assert left == right and len(left) == 502
for sample in ('size','area','alpha','linewidth'):
    for fmt in ('svg','pdf','png'):
        assert (binned_numeric_constructors/'python'/f'numeric-{sample}.{fmt}').read_bytes() == (binned_numeric_constructors/'wasm'/f'numeric-{sample}.{fmt}').read_bytes()
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
