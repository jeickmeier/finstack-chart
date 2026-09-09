"""FIX-GG01: execute the Rust matrix inputs through the actual primary Python adapter.

Arguments: built Python module directory, Rust artifact directory, output directory.
The Rust test supplies independent legend-count/color/provenance expectations.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c

inputs, out = map(Path, sys.argv[2:4])
out.mkdir(parents=True, exist_ok=True)
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
files = sorted(inputs.glob('*.plot.json'))
assert len(files) == 24, len(files)
for file in files:
    name = file.name.removesuffix('.plot.json')
    plot = c.Plot.from_json(file.read_text())
    chart = plot.chart()
    if name.startswith('hidden-'):
        chart.legend_visible(False)
    width, height = (130, 65) if name.startswith('tight-') else (500, 300)
    options = c.export_options(width, height).dpi(96).basis('current')
    request = chart.request(output, options)
    frame = request.prepare()
    scene = frame.scene()
    expected = json.loads((inputs / f'{name}.scene.json').read_text())
    assert scene == expected, name
    (out / f'{name}.scene.json').write_text(json.dumps(scene, indent=2))
    for fmt in ('svg', 'pdf', 'png'):
        (out / f'{name}.{fmt}').write_bytes(frame.export(fmt))
    # Exercise host component dispatch as well as the primary interchange path.
    if name.startswith('two-entry-'):
        edited = plot.edit().legend(c.legend().scale('series').untitled()).build()
        chart.apply_plot(edited, 0)
        edited_request = chart.request(output, options)
        edited_frame = edited_request.prepare()
        texts = [i['primitive']['Text']['text'] for i in edited_frame.scene()['items'] if 'Text' in i['primitive']]
        assert 'Series' not in texts and 'Color' not in texts and texts.count('Alpha') == 1
        edited_frame.dispose(); edited_request.dispose(); edited.dispose()
        generic = plot.edit().legend(c.legend().scale('series').generic_title()).build()
        generic_request = output.request(generic, options)
        generic_frame = generic_request.prepare()
        texts = [i['primitive']['Text']['text'] for i in generic_frame.scene()['items'] if 'Text' in i['primitive']]
        assert texts.count('Color') == 1 and 'Series' not in texts and texts.count('Alpha') == 1
        generic_frame.dispose(); generic_request.dispose(); generic.dispose()
    frame.dispose(); request.dispose(); chart.dispose(); plot.dispose(); options.dispose()
output.dispose()
print('PASS FIX-GG01 Python: 24 exact scenes, 72 exports and 6 host untitled/generic edits.')
