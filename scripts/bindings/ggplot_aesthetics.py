"""FIX-GG03: actual host styles, glyph ownership and immutable publication."""
import json
import math
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
reference = json.loads((ROOT / 'fixtures/parity/ggplot2/aesthetics.json').read_text())
records = []
for case in reference['symbols']:
    config = {'kind': {'Ggplot': case['pch']}, 'size': math.pi * (.375 * case['device_size']) ** 2}
    symbol = c.ShapeSymbol(config); copy = symbol.copy(); symbol.dispose()
    path = copy.generate(); saved = path.copy(); before = saved.result()
    again = copy.generate(); assert again.result() == before
    path.move_to(99, 99); path.dispose(); copy.dispose(); assert saved.result() == before
    records.append({'pch': case['pch'], 'device_size': case['device_size'], 'geometry': before['geometry']})
    saved.dispose(); again.dispose()
for code in [26, 31, 255]:
    try: c.ShapeSymbol({'kind': {'Ggplot': code}}).generate()
    except c.ChartError: pass
    else: raise AssertionError('invalid glyph accepted')
(out / 'symbols.json').write_text(json.dumps(records))
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
data = c.Data.columns({'x': [1., 3.], 'y': [2., 2.], 'inside': ['A', 'B'], 'outside': ['B', 'A'], 'area': [1., 4.], 'alpha': [.5, 1.]}, keys=[9007199254741001, 9007199254741003])
def builder():
    return (c.plot(data).profile('Ggplot2_4_0_3')
        .aes(c.aes().x('x').y('y').fill('inside').fill_scale('inside').stroke('outside').stroke_scale('outside'))
        .scale(c.color_discrete('inside').domain(['A', 'B']).palette(['#ff0000', '#0000ff']))
        .scale(c.color_discrete('outside').domain(['A', 'B']).palette(['#000000', '#008000'])))
p = builder().layer(c.points().name('points').radius(5).linewidth(2).shape_value('Alpha', data.field('alpha'))).build()
wire = p.to_json(); assert json.loads(wire)['version'] == 16
loaded = c.Plot.from_json(wire); assert loaded.to_json() == wire
chart = loaded.chart(); semantic = chart.semantics(); styles = semantic['layers'][0]['styles']
assert [s['fill'] for s in styles] == [{'red':255,'green':0,'blue':0,'alpha':128}, {'red':0,'green':0,'blue':255,'alpha':255}]
assert [s['stroke']['green'] for s in styles] == [128, 0]
assert len(semantic['layers'][0]['paint_legends']) == 2
assert all(s['stroke_width'] == 2 and s['radius'] == 5 for s in styles)
(out / 'independent.semantics.json').write_text(json.dumps(semantic))
edited = p.edit().layer('points', c.points().fill('#123456')).build()
echart = edited.chart(); assert len(echart.semantics()['layers'][0]['paint_legends']) == 1
assert p.to_json() == wire; echart.dispose(); edited.dispose()
scale = c.StandaloneScale('linear')
q = (c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()
    .value_scale('TextSize', c.source_expr(data.field('area')).sum(), scale)
    .aesthetic_value('FontFace', {'kind':'Text','value':'bold'})).build())
qc = q.chart(); values = qc.semantics()['layers'][0]['aesthetics']
assert values == [{'FontFace':{'kind':'Text','value':'bold'},'TextSize':{'kind':'Number','value':5.}}] * 2
qc.dispose(); q.dispose(); scale.dispose()
for layer in [c.points().alpha(1.1), c.points().line_type({'Custom': 0}),
              c.points().aesthetic_value('FontFace', {'kind':'Text','value':'unknown'})]:
    try: c.plot(data).aes(c.aes().x('x').y('y')).layer(layer).build()
    except c.ChartError: pass
    else: raise AssertionError('invalid aesthetic accepted')
    layer.dispose()
for name, layer in [('area', c.points().shape_value('AreaSize', data.field('area')).stroke('#000000').linewidth(2)), ('radius', c.points().aes(c.aes().size('area')).stroke('#000000'))]:
    q = c.plot(data).aes(c.aes().x('x').y('y')).layer(layer).build(); qc = q.chart(); s = qc.semantics()['layers'][0]['styles']
    assert math.isclose((s[1]['radius'] / s[0]['radius']) ** 2, 4 if name == 'area' else 16)
    qc.dispose(); q.dispose(); layer.dispose()
# A complete glyph gallery exercises resolved fill/stroke modes through the scene and exporters.
gallery = (c.plot(data).aes(c.aes().x('x').y('y'))
    .x_axis(c.x_axis().scale(c.scale_linear().domain(-.5, 6.5)))
    .y_axis(c.y_axis().scale(c.scale_linear().domain(-.5, 3.5))))
for code in range(26):
    d = c.Data.columns({'x': [float(code % 7)], 'y': [float(3 - code // 7)]}, keys=[9007199254741001 + code], name=f'glyph-{code}')
    layer = c.shape_symbol().data(d).symbol_kind({'Ggplot':code}).symbol_size(100).fill('#ff0000').stroke('#0000ff').linewidth(1)
    gallery = gallery.layer(layer); d.dispose(); layer.dispose()
g = gallery.title(c.title('R point glyphs 0–25: independent red fill and blue outline')).build()
for name, plot in [('independent', loaded), ('glyphs', g)]:
    request = output.request(plot, c.export_options(640, 400).dpi(144)); frame = request.prepare()
    scene = frame.scene(); (out / f'{name}.scene.json').write_text(json.dumps(scene)); (out / f'{name}.plot.json').write_text(plot.to_json())
    for fmt in ['svg', 'pdf', 'png']: (out / f'{name}.{fmt}').write_bytes(frame.export(fmt))
    if name == 'glyphs':
        marks = [i['primitive']['ShapePath'] for i in scene['items'] if i.get('layer') is not None and 'ShapePath' in i['primitive']]
        assert len(marks) == 26
        for code, mark in enumerate(marks):
            assert (mark['fill'] is not None) == (code >= 15)
            assert (mark['stroke'] is not None) == (code <= 14 or code >= 19)
            assert len(mark['anchors']) == 1
    frame.dispose(); request.dispose()
chart.dispose(); p.dispose(); loaded.dispose(); g.dispose(); data.dispose(); output.dispose()
print('PASS Python GG-03: 104 glyph ownership cases, independent paints/alpha/metadata, area/radius ratios, v16/edit capture and 26-glyph SVG/PDF/PNG gallery.')
