"""FIX-GG04 actual Python exceptional limits, hidden bins and fractional counts."""
import json
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(640, 360).dpi(96).basis('current')
records = []
def wire_number(value):
    return {'number':value} if isinstance(value,str) else value
def descriptor(case, hidden):
    family = {'sqrt': {'Pow': {'exponent': .5}}, 'log10': {'Log': {'base': 10.}}}.get(case['transform'], 'Linear')
    kind = case['kind'] if 'kind' in case else ('binned_nice' if case['nice'] else 'binned_equal')
    limits = None if case['limits'] is None else [wire_number(v) for v in case['limits']]
    cuts = {'Explicit': [wire_number(v) for v in case['cuts']]} if case.get('mode') == 'explicit' else {('Equal' if kind == 'binned_equal' else 'Nice'): case.get('count', 5)}
    policy = {'Continuous': {'limits': limits, 'oob': 'Censor'}} if kind == 'continuous' else {'Binned': {'limits': limits, 'oob': 'Squish', 'breaks': cuts, 'right': True}}
    return {'training': 'Eligible', 'guide': 'Hidden' if hidden else {'Binned': 'Automatic'}, 'ggplot': policy,
        'function': {'Interpolated': {'normalization': {'Ggplot': {'family': family, 'domain': [1,10], 'reverse': case['transform'] == 'reverse', 'rescaler': 'Range'}},
        'output': {'Interpolate': {'operation': 'GgplotPalette', 'spec': {'Gradient': {'colors': [{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}], 'values': None}}}}, 'unknown': {'kind': 'Missing'}}}}
def paint(value):
    if value == 'grey50': return {'red':127,'green':127,'blue':127,'alpha':255}
    assert value.startswith('#') and len(value) == 7, value
    return dict(zip(('red','green','blue','alpha'), [int(value[i:i+2],16) for i in (1,3,5)] + [255]))
for kind, filename in [('authored','authored-limit-populations.json'), ('fractional','binned-fractional-counts.json')]:
    fixture = json.loads((ROOT / 'fixtures/parity/ggplot2' / filename).read_text())
    for index, case in enumerate(fixture['primary' if kind == 'authored' else 'cases']):
        hidden = kind == 'authored'
        values = {'finite':[1.,10.], 'empty':[], 'missing':[None,None], 'infinite':[float('inf'),float('-inf')]}.get(case.get('population'), case.get('domain'))
        expected = case['result'] if hidden or 'error' in case['result'] else case['result']['mapping']
        owned = []
        try:
            data = c.Data.columns({'x': c.column([float(i) for i in range(len(values))], kind='float64'), 'v': c.column(values, kind='float64')}); owned.append(data)
            plot = (c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.).color('v').color_scale('v'))
                .scale(c.color_mapped('v', descriptor(case, hidden))).layer(c.points()).build()); owned.append(plot)
            wire = plot.to_json(); assert json.loads(wire)['version'] == 17
            restored = c.Plot.from_json(wire); owned.append(restored); assert restored.to_json() == wire
            chart = restored.chart(); owned.append(chart); semantic = chart.semantics()
            assert 'error' not in expected, (kind,index,case)
            layer = semantic['layers'][0]; colors = [s['color'] for s in layer.get('styles', [])]
            wanted = [paint(v) for v in expected['colors']]
            if len(wanted) == 1: wanted *= len(values)
            assert colors == wanted, (kind,index,colors,wanted)
            record = {'kind':kind,'index':index,'colors':colors}
            if hidden:
                assert layer.get('color_legend') is None
            else:
                labels = case['result']['labels']
                entries = layer['color_legend'].get('numeric_breaks', []) if layer.get('color_legend') else []
                actual = [e['label'] for e in entries]
                assert [e['visible'] for e in entries] == case['result']['visible'], (kind,index,entries)
                assert actual == labels, (kind,index,actual,labels)
                record['labels'] = actual
            records.append(record)
            sample = None
            if hidden and case['transform'] == 'identity' and case['population'] == 'finite' and case['mode'] == 'auto':
                if case['kind'] == 'binned_nice' and case['limits'] is None: sample = 'hidden-bins'
                if case['kind'] == 'continuous' and case['limits'] == [1,'Infinity']: sample = 'infinite-limit'
            if not hidden and case['transform'] == 'identity' and case['domain'] == [1,10] and case['limits'] is None and case['count'] == 2.5 and case['nice']: sample = 'fractional-bins'
            if sample:
                request = output.request(restored, options); owned.append(request)
                frame = request.prepare(); owned.append(frame)
                (out / f'{sample}.plot.json').write_text(wire)
                (out / f'{sample}.scene.json').write_text(json.dumps(frame.scene(),indent=2))
                for fmt in ('svg','pdf','png'): (out / f'{sample}.{fmt}').write_bytes(frame.export(fmt))
        except c.ChartError as error:
            assert 'error' in expected, (kind,index,case,error)
            records.append({'kind':kind,'index':index,'error':error.code})
        finally:
            for value in reversed(owned): value.dispose()
output.dispose()
assert len(records) == 2304
(out / 'records.json').write_text(json.dumps(records,indent=2))
print('PASS Python GG04: 2016 authored-limit chart cases, 288 fractional-bin records, v17 and three SVG/PDF/PNG samples.')
