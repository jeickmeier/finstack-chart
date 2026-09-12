"""FIX-GG04 actual Python date/duration label consumers over pinned R records."""
import json
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(1200, 360).dpi(96).basis('current')
records = []
for kind, filename in [('datetime', 'time-formats.json'), ('duration', 'duration-formats.json')]:
    cases = json.loads((ROOT / 'fixtures/parity/ggplot2' / filename).read_text())['cases']
    for index, case in enumerate(cases):
        if kind == 'datetime':
            value = int(case['microseconds'])
            data = c.Data.columns({'x': c.timestamps([value - 1000000, value + 1000000], 'us', 'UTC'), 'y': [0., 1.]})
            values = [{'Timestamp': {'value': str(value), 'unit': 'Microseconds'}}]
            expected = [case['label']]
            axis = c.x_axis()
        else:
            seconds = [float(v) for v in case['seconds']]
            data = c.Data.columns({'x': seconds, 'y': [0., 1.]})
            values = [{'Number': v} for v in seconds]
            expected = case['labels']
            axis = c.x_axis().scale(c.scale_duration())
        axis = (axis.tick_values(values)
                .tick_format({'GgplotTime': {'pattern': case['pattern']}})
                .guide_geometry({'labels': 'Preserve'}))
        plot = (c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points())
                .x_axis(axis).y_axis(c.y_axis().visible(False)).build())
        wire = plot.to_json(); assert json.loads(wire)['version'] == 17
        restored = c.Plot.from_json(wire); assert restored.to_json() == wire
        request = output.request(restored, options)
        if any('\t' in label for label in expected):
            try: request.prepare()
            except c.ChartError as error: assert error.code == 'CHART_MISSING_RESOURCE', error
            else: raise AssertionError('Publication must reject unsupported tab glyphs explicitly')
            records.append({'kind': kind, 'index': index, 'unsupported': 'tab glyph'})
            for value in (request, restored, plot, data): value.dispose()
            continue
        frame = request.prepare()
        guides = [g for g in frame.guides()['guides'] if g['spec']['side'] == 'Bottom']
        assert len(guides) == 1, guides
        ticks = guides[0]['ticks']
        assert [t['label'] for t in ticks] == expected, (kind, index, ticks, expected)
        assert [t['value'] for t in ticks] == values, (kind, index, ticks, values)
        records.append({'kind': kind, 'index': index, 'values': values, 'labels': expected})
        if (kind, index) in [('datetime', 0), ('datetime', 21), ('duration', 1)]:
            name = f'{kind}-format-{index}'
            (out / f'{name}.plot.json').write_text(wire)
            (out / f'{name}.guides.json').write_text(json.dumps(frame.guides(), indent=2))
            for fmt in ('svg', 'pdf', 'png'):
                (out / f'{name}.{fmt}').write_bytes(frame.export(fmt))
        for value in (frame, request, restored, plot, data): value.dispose()
output.dispose()
assert len(records) == 260
assert sum("unsupported" in record for record in records) == 14
(out / 'records.json').write_text(json.dumps(records, indent=2))
print('PASS Python GG04: 246 R date/duration cases with exact values/v17/publication; 14 tab-glyph rejections.')
