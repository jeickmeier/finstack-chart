"""FIX-GG04 actual Python positional-bin panels, v18 and publication proofs."""
import json
import math
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(640, 360).dpi(96).basis('current')
records = []
fixture = json.loads((ROOT / 'fixtures/parity/ggplot2/positional-bin-panels.json').read_text())
for index, case in enumerate(fixture['cases']):
    expected = case['result']; owned = []
    transform = {'sqrt':'Sqrt','reverse':'Reverse','log10':{'Log':{'base':10}}}.get(case['transform'])
    breaks = {'Equal' if case['mode'] == 'equal' else 'Nice':3}
    if case['mode'] in ('explicit','empty'): breaks = {'Explicit':[-1,1,2,4,10,20] if case['mode'] == 'explicit' else []}
    if case['mode'] == 'none': breaks = None
    spec = {'bins':{'limits':None if case['limits'] is None else [({'number':v} if isinstance(v,str) else v) for v in case['limits']],'breaks':breaks,'oob':'Squish','right':case['right']},'transform':transform,'show_limits':case['show_limits']}
    try:
        data = c.Data.columns({'x':[4.,4.] if case['population'] == 'constant' else [1.,10.],'y':[1.,2.]}); owned.append(data)
        plot = (c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points())
            .x_axis(c.x_axis().scale(c.scale_binned(spec)).guide_geometry({'labels':'Preserve'})).y_axis(c.y_axis().visible(False)).build()); owned.append(plot)
        wire = plot.to_json(); assert json.loads(wire)['version'] == 18
        restored = c.Plot.from_json(wire); owned.append(restored); assert restored.to_json() == wire
        chart = restored.chart(); owned.append(chart)
        semantic = chart.semantics()
        request = output.request(restored, options); owned.append(request)
        frame = request.prepare(); owned.append(frame)
        assert 'error' not in expected, (index,case)
        finite = [v for v in expected['x'] if isinstance(v,(int,float))]
        domain = semantic['layers'][0]['domains']['x']
        if finite:
            assert math.isclose(domain['minimum'],min(finite),rel_tol=1e-11,abs_tol=1e-11), (index,domain,finite)
            assert math.isclose(domain['maximum'],max(finite),rel_tol=1e-11,abs_tol=1e-11), (index,domain,finite)
        ticks = next(g for g in frame.guides()['guides'] if g['spec']['side'] == 'Bottom')['ticks']
        assert [t['label'] for t in ticks] == expected['drawable_labels'], (index,ticks,expected)
        for actual,wanted in zip(ticks,expected['drawable_break_values']): assert math.isclose(actual['value']['Number'],wanted,rel_tol=1e-11,abs_tol=1e-11)
        records.append({'index':index,'domain':domain,'labels':[t['label'] for t in ticks]})
        sample = None
        if case['mode']=='nice' and case['population']=='finite' and case['right'] and not case['show_limits']:
            if case['limits'] is None and case['transform'] in ('identity','sqrt','reverse'):
                sample = case['transform']+'-positional-bins'
            elif case['limits'] == [1,'Infinity'] and case['transform']=='identity':
                sample = 'unbounded-positional-bins'
        if sample:
            (out / (sample+'.plot.json')).write_text(wire)
            for fmt in ('svg','pdf','png'): (out / (sample+'.'+fmt)).write_bytes(frame.export(fmt))
    except c.ChartError as error:
        assert 'error' in expected, (index,case,error)
        records.append({'index':index,'error':error.code})
    finally:
        for value in reversed(owned): value.dispose()
output.dispose()
assert len(records)==960
(out / 'records.json').write_text(json.dumps(records,indent=2))
print('PASS Python: 960 positional-bin panels, v18 and four SVG/PDF/PNG samples.')
