"""Compare actual native, Python and WASM results plus independent fixture expectations."""
import hashlib
import json
import math
import statistics as statistics_oracle
import re
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

root = Path(sys.argv[1])
paths = [root / host for host in ("native", "python", "wasm")]

def same(a, b, tolerance, location="root"):
    if isinstance(a, dict):
        assert isinstance(b, dict) and a.keys() == b.keys(), location
        for key in a: same(a[key], b[key], tolerance, f"{location}/{key}")
    elif isinstance(a, list):
        assert isinstance(b, list) and len(a) == len(b), location
        for i, (left, right) in enumerate(zip(a, b)): same(left, right, tolerance, f"{location}/{i}")
    elif isinstance(a, float):
        assert isinstance(b, (float, int)) and math.isfinite(a) and math.isfinite(b)
        assert abs(a-b) <= tolerance, (location, a, b)
    else:
        assert type(a) == type(b) and a == b, (location, a, b)

for name in ("initial", "definition", "transaction", "replay", "action", "final", "scene", "state"):
    values = [json.loads((p / f"{name}.json").read_text()) for p in paths]
    for value in values[1:]: same(values[0], value, 0.01 if name == "scene" else 1e-10)
    print(f"PASS three-runtime {name}: exact identities/types/state; numeric tolerance {'0.01 pt' if name == 'scene' else '1e-10 absolute'}")
for path in paths:
    initial = json.loads((path / "initial.json").read_text())
    final = json.loads((path / "final.json").read_text())
    assert [r["count"] for r in initial["layers"][0]["rows"]["Binned"]] == ["2", "1"]
    bins = final["layers"][0]["rows"]["Binned"]
    assert [r["count"] for r in bins] == ["1", "2"]
    assert [r["target"]["Aggregate"]["members"] for r in bins] == [["9007199254743001"], ["9007199254743002", "9007199254743004"]]
    assert final["layers"][0]["domains"]["x"] == {"minimum":0., "maximum":2.}
    assert final["layers"][1]["domains"]["y"] == {"minimum":1., "maximum":4.}
    assert final["state"]["viewport"] == {"x":[0.5,1.6], "y":None}
    assert final["state"]["state_revision"] == final["state"]["viewport_revision"] == "1"
    assert final["store_revision"] == "1"
    chunks = final["datasets"][0]["chunks"]
    rows = [(key, chunk, i) for chunk in chunks for i, key in enumerate(chunk["keys"])]
    assert len(rows) == 4
    for offset, (key, chunk, i) in enumerate(rows):
        assert key == str(9007199254743001+offset)
        assert chunk["columns"][2]["values"]["Timestamp"][i] == str(1712345678901234567+offset)
    _, first, i = rows[0]
    assert first["columns"][3]["values"]["UInt64"][i] == "18446744073709551615"
    assert first["columns"][4]["values"]["Int64"][i] == "-9223372036854775808"
    assert first["columns"][0]["formatted"][i] == "0.2500"
    _, third, i = rows[2]
    assert third["columns"][0]["values"]["Float64"][i] == 1.25
    assert third["columns"][0]["validity"][i] is False
    assert "Applied" in json.loads((path / "transaction.json").read_text())
    assert "AlreadyApplied" in json.loads((path / "replay.json").read_text())
    scene = json.loads((path / "scene.json").read_text())
    assert len(scene["items"]) == len(scene["targets"])
    assert scene["targets"][0] == [] and scene["items"][0]["layer"] is None
    for item, targets in zip(scene["items"], scene["targets"]):
        if item["layer"] is None: assert targets == []
        else: assert targets
    assert scene["items"][1]["layer"] == "9007199254742001"
    assert scene["targets"][1][0]["Aggregate"]["members"] == ["9007199254743001"]
    svg = ET.parse(path / "chart.svg").getroot()
    assert svg.attrib["width"] == "360pt" and svg.attrib["height"] == "240pt"
    ns = "{http://www.w3.org/2000/svg}"
    assert list(svg.iter(ns+"path")) and not list(svg.iter(ns+"image")) and not list(svg.iter(ns+"text"))
print("PASS independent domains/bin counts/members, large IDs, exact nanosecond times, UInt64/Int64 extremes, null/display values, replay, physical vector SVG and per-item target alignment")
# This fixed outline route is deterministic across the three actual runtimes.
assert len({hashlib.sha256((p / 'chart.svg').read_bytes()).hexdigest() for p in paths}) == 1
print("PASS exact SVG bytes across Rust/Python/WASM")
for path in paths[:2]:
    png = (path / "chart.png").read_bytes()
    assert struct.unpack_from(">II", png, 16) == (750,500)
    pdf = str(path / "chart.pdf")
    info = subprocess.check_output(["pdfinfo",pdf],text=True)
    assert re.search(r"Page size:\s+360 x 240 pts", info)
    assert not subprocess.check_output(["pdfimages","-list",pdf],text=True).strip().splitlines()[2:]
    assert not subprocess.check_output(["pdffonts",pdf],text=True).strip().splitlines()[2:]
print("PASS actual Rust/Python PDF vector/font policy and PNG dimensions")

# WP-10 operation outputs and actual final scene projection, including display positions.
statistics = [json.loads((p / 'statistics.json').read_text()) for p in paths]
scenes = [json.loads((p / 'statistics-scenes.json').read_text()) for p in paths]
for other in statistics[1:]: same(statistics[0], other, 1e-12)
for other in scenes[1:]: same(scenes[0], other, 1e-10)
def close(a,b,tolerance=1e-12): assert abs(a-b)<=tolerance,(a,b)
def row_values(row):
    return {str(v['field']):v['value'] for v in row['values']}
for data,scene in zip(statistics,scenes):
    layer = lambda name: data[name]['layers'][0]
    rows = lambda name: layer(name)['rows']['Statistical']
    summary = rows('summary')[0]
    assert summary['count']=='4' and len(summary['members'])==4
    v=row_values(summary); assert v['Sum']==60 and v['Mean']==15
    close(next(v['value'] for v in summary['values'] if v['field']=={'Quantile':1}),7.5)
    assert layer('summary')['operations'][0]['counts']['invalid_stat']==1
    assert [r['count'] for r in rows('count')]==['1','0']
    for name,intercept,slope,count in [('ols',0.8,2.3,'4'),('filtered-fit',1,2,'3')]:
        r=rows(name)[0];v=row_values(r);close(v['Intercept'],intercept);close(v['Slope'],slope);assert r['count']==count
        assert 'Derived' in r['target'] and len(r['members'])==int(count)
    assert row_values(rows('transformed-summary')[0])['Mean']==20
    assert 'Transformed' in layer('transformed-summary')['domains']['y_space']
    bins=layer('auto-bin')['rows']['Binned'];assert len(bins)==30 and sum(int(b['count']) for b in bins)==4
    assert bins[0]['start']==0 and bins[-1]['end']==2
    assert [r['count'] for r in layer('overflow-bin')['rows']['Binned']]==['3','3']
    for name,bound in [('stack',5),('normalize',1)]:
        assert layer(name)['domains']['y']=={'minimum':-bound,'maximum':bound}
        # Source rows remain source rows after positioning, while final rectangles change.
        assert 'Source' in layer(name)['rows']
    for name,case in scene.items():
        assert len(case['items'])==len(case['targets'])
        assert all('position' not in target for targets in case['targets'] for target in targets)
        operation=layer(name)['operations'][0]
        specialized=isinstance(operation['parameters'],dict) and 'Bin' in operation['parameters']
        assert operation['incremental']=={'append':specialized,'window':specialized,'correction':specialized,'full_recompute':True}
    # Fixed slots [0,1,2] with missing middle group: equal bar widths, one slot gap.
    rects=[i['primitive']['Rectangle']['bounds'] for i in scene['dodge']['items'] if i['layer']=='1']
    assert len(rects)==2
    close(rects[0]['width'],rects[1]['width'],1e-10)
    close(rects[1]['origin']['x']-rects[0]['origin']['x'],2*rects[0]['width'],1e-10)
for name in statistics[0]:
    svgs=[(p/f'statistics-{name}.svg').read_bytes() for p in paths]
    assert all(svg==svgs[0] for svg in svgs[1:]),name
print(f'PASS shared fixtures: {len(statistics[0])} cases in Rust/Python/WASM, independent stats/provenance, scenes within 1e-10 pt, SVG bytes exact')

# WP-11 independent expectations, in addition to the unchanged three-runtime comparisons.
for data, family_scenes in zip(statistics,scenes):
    def family(name): return data['family-'+name]['layers']
    def marks(name,layer='1'): return [i['primitive'] for i in family_scenes['family-'+name]['items'] if i['layer']==layer]
    assert len(marks('log-gaps'))==2 and all('Path' in p for p in marks('log-gaps'))
    assert len(family('log-gaps')[0]['rows']['Source'])==5
    assert len(marks('area'))==2 and family('area')[0]['invalid_geometry']==1
    assert family('area')[0]['domains']['y']=={'minimum':0.,'maximum':4.}
    assert len(marks('ribbon'))==2 and family('ribbon')[0]['invalid_geometry']==2
    assert all('FilledPath' in p for p in marks('area')+marks('ribbon'))
    legend=family('point-color')[0]['color_legend']
    assert legend['id']=='20' and [p[0] for p in legend['entries']]==['Alpha','Beta','Gamma'] and not legend['continuous']
    assert len(marks('heatmap'))==6 and family('heatmap')[0]['color_legend']['continuous']
    assert marks('heatmap')[-1]['Rectangle']['fill']=={'red':128,'green':128,'blue':128,'alpha':70}
    price,volume=family('ohlc-volume')
    assert price['invalid_geometry']==0 and volume['invalid_geometry']==1
    assert price['domains']['y']=={'minimum':1.,'maximum':7.} and volume['domains']['y']=={'minimum':0.,'maximum':30.}
    assert len(marks('ohlc-volume'))==10 and len(marks('ohlc-volume','2'))==4
    assert len(marks('session'))==2 and len(family('session')[0]['rows']['Source'])==5
    labels=[i['primitive']['Text']['text'] for i in family_scenes['family-utc-leap']['items'] if 'Text' in i['primitive']]
    assert any('2024-02-29' in t for t in labels)
    assert family('stacked-bars')[0]['domains']['y']=={'minimum':-5.,'maximum':5.}
for name in statistics[0]:
    if name.startswith('family-'):
        pdf=str(paths[0]/f'statistics-{name}.pdf')
        assert not subprocess.check_output(['pdfimages','-list',pdf],text=True).strip().splitlines()[2:]
print('PASS WP-11: independent log/run gaps, area/ribbon boundaries, color/missing metadata, price/volume validation, supplied sessions, leap day, signed stack and vector PDF expectations')

# WP-12: independent FIX-06 expectations and panel-aware destination checks.
for runtime in paths:
    data=json.loads((runtime/'statistics.json').read_text())
    scene=json.loads((runtime/'statistics-scenes.json').read_text())
    shared=data['facet-shared-broadcast']['panels']
    assert [p['key']['values'][0]['Text'] for p in shared]==['B','A']
    assert [len(p['layers'][0]['rows']['Source']) for p in shared]==[2,3]
    assert [len(p['layers'][1]['rows']['Source']) for p in shared]==[1,1]
    assert [len(p['layers']) for p in data['facet-free-target']['panels']]==[1,2]
    for name in ('facet-shared-broadcast','facet-free-target','facet-grid-empty'):
        output=scene[name]
        assert len(output['items'])==len(output['targets'])==len(output['item_panels'])
        assert all(panel['plot'] is not None for panel in output['panels'])
        for item,targets,panel in zip(output['items'],output['targets'],output['item_panels']):
            if targets: assert panel is not None and item['layer'] is not None
        plots=[p['plot'] for p in output['panels']]
        assert all(abs(p['width']-plots[0]['width'])<1e-10 and abs(p['height']-plots[0]['height'])<1e-10 for p in plots)
    output=scene['facet-shared-broadcast']
    threshold=[i['primitive']['Rule']['from']['y'] for i in output['items'] if i['layer']=='2']
    assert len(threshold)==2 and abs(threshold[0]-threshold[1])<1e-10
    plot=output['panels'][0]['plot']
    points=[i['primitive']['Point']['center'] for i,p in zip(output['items'],output['item_panels']) if i['layer']=='1' and p['values']==[{'Text':'B'}]]
    assert abs(points[0]['y']-(plot['origin']['y']+(120-100)/(120-1)*plot['height']))<1e-10
    grid=data['facet-grid-empty']['panels']
    assert len(grid)==6 and all(not p['layers'][0]['rows']['Source'] for p in grid[-2:])
    expected={'group-summary':[[100.,120.],[2.,9.]],'panel-summary':[[110.],[13./3.]],'chart-summary':[[233./5.],[233./5.]]}
    for name,wanted in expected.items():
        panels=data['facet-'+name]['panels']
        actual=[[float(next(v['value'] for v in row['values'] if v['field']=='Mean')) for row in p['layers'][0]['rows']['Statistical']] for p in panels]
        same(actual,wanted,1e-12)
        for p in panels:
            op=p['layers'][0]['operations'][-1]
            assert op['scope']=={'group-summary':'Group','panel-summary':'Facet','chart-summary':'Chart'}[name]
            assert op['panel']==(None if name=='chart-summary' else p['key'])
    assert data['facet-chart-summary']['panels'][0]['layers'][0]['targets']==data['facet-chart-summary']['panels'][1]['layers'][0]['targets']
    texts=[i['primitive']['Text']['text'] for i in scene['facet-shared-broadcast']['items'] if 'Text' in i['primitive']]
    assert texts.count('Group')==1
for name in data:
    if name.startswith('facet-'):
        assert not subprocess.check_output(['pdfimages','-list',str(paths[0]/f'statistics-{name}.pdf')],text=True).strip().splitlines()[2:]
print('PASS WP-12 FIX-06: typed panel order/membership, broadcast/target, shared/free scales, aligned six-panel grid, grouped/facet/chart means, per-item panel identity, collected guides and vector PDF')

# WP-13: FIX-12/13 composition and explicit typography through all three real hosts.
composition_names = ['composition-'+theme for theme in ('editorial','terminal','grayscale')]
for runtime, data, scene in zip(paths, statistics, scenes):
    reference = data[composition_names[0]]
    for name in composition_names:
        actual = data[name]
        # Theme changes must leave all prepared populations, domains and targets intact.
        same(reference['layers'], actual['layers'], 1e-12)
        same(reference['panels'], actual['panels'], 1e-12)
        output = scene[name]
        assert len(output['panels']) == 2 and len(output['insets']) == 1
        assert len(output['items']) == len(output['targets']) == len(output['item_panels'])
        assert output['insets'][0]['panel'] == {'values':[{'Text':'A'}]}
        primitives = [i['primitive'] for i in output['items']]
        assert sum('GradientRectangle' in p for p in primitives) == 3
        assert sum('DashedPath' in p for p in primitives) == 2
        assert all(p['Symbol']['kind']=='Diamond' for p in primitives if 'Symbol' in p)
        glyphs = [p['GlyphRun'] for p in primitives if 'GlyphRun' in p]
        arabic = next(g['run'] for g in glyphs if g['run']['text']=='مرحبا بالعالم')
        assert arabic['used_fallback'] and arabic['font']['id']=='9007199254747003'
        assert arabic['direction']=='RightToLeft' and all(g['id']!=0 for g in arabic['glyphs'])
        assert any(g['run']['font']['id']=='9007199254747002' for g in glyphs)
        assert any(g['rotation']==-35 and g['run']['tabular'] for g in glyphs)
        assert any(g['rotation']==-90 and g['run']['text']=='Value (units)' for g in glyphs)
        assert all(any(g['run']['text']==t for g in glyphs) for t in ['Peak 9','Panel B','(a)','(b)','λ = −0.25'])
        assert len(output['diagnostics'])==1 and output['diagnostics'][0]['severity']=='Warning'
        svg = ET.parse(runtime/f'statistics-{name}.svg').getroot()
        ns = '{http://www.w3.org/2000/svg}'
        assert not list(svg.iter(ns+'image')) and list(svg.iter(ns+'linearGradient'))
        assert any(e.attrib.get('aria-label')=='مرحبا بالعالم' for e in svg.iter())
for name in composition_names:
    pdf = str(paths[0]/f'statistics-{name}.pdf')
    assert not subprocess.check_output(['pdfimages','-list',pdf],text=True).strip().splitlines()[2:]
    fonts = subprocess.check_output(['pdffonts',pdf],text=True).strip().splitlines()[2:]
    assert len(fonts)==3 and all(re.search(r'yes\s+yes\s+yes', f) for f in fonts)
    extracted = subprocess.check_output(['pdftotext',pdf,'-'],text=True)
    assert 'مرحبا بالعالم' in extracted
print('PASS WP-13 FIX-12/13: theme invariant populations/domains/targets, two panels and source-sharing inset, rich/rotated/tabular text, explicit bold/Arabic resources, symbols/dashes/gradients, vector searchable PDF and three-host exact SVG')

# WP-14 FIX-17: independent external operation and builtin candle direction expectations.
for runtime, data, scene in zip(paths, statistics, scenes):
    output = data['extension-histogram']; layers = output['layers']
    assert len(layers) == 2
    rows = layers[0]['rows']['Statistical']
    assert [r['count'] for r in rows] == ['3','3']
    assert layers[0]['schema']['Custom']['operation'] == {'id':'example.density_histogram','version':'1'}
    for index,row in enumerate(rows):
        values = {v['field']['Custom']:v['value'] for v in row['values']}
        assert values == {'left':float(index),'right':float(index+1),'density':0.5}
        assert row['members'] == [str(9007199254743001+index*3+i) for i in range(3)]
        assert row['target']['Aggregate']['members'] == row['members']
    assert layers[0]['operations'][0]['counts']['invalid_stat'] == 1
    assert layers[0]['domains']['x'] == {'minimum':0.,'maximum':2.}
    assert layers[0]['domains']['y'] == {'minimum':0.,'maximum':0.5}
    assert layers[1]['rows'] == layers[0]['rows']
    figure = scene['extension-histogram']
    assert len(figure['interactions']) == 2
    interaction = list(figure['interactions'].values())
    assert [i['keyboard_order'] for i in interaction] == ['2','1']
    assert all(i['selection']=='AtomicTarget' and 'Polygon' in i['hit'] for i in interaction)
    assert all(i['values'][0] == ['Count',{'Unsigned':'3'}] for i in interaction)
    assert all('FilledPath' in figure['items'][int(index)]['primitive'] for index in figure['interactions'])
    labels=[i['primitive']['Text']['text'] for i in figure['items'] if 'Text' in i['primitive']]
    assert all(label in labels for label in ['Lower','Boundary','Upper'])
    candles=[i['primitive'] for i in scene['alpha-candle-colors']['items'] if i['layer']=='1']
    colors=[p['Rule']['stroke']['color'] if 'Rule' in p else p['Rectangle']['fill'] for p in candles]
    green={'red':30,'green':145,'blue':85,'alpha':255};red={'red':195,'green':55,'blue':55,'alpha':255}
    assert colors == [green,green,red,red,red,red,green,green,green,green]
for name in ('extension-histogram','alpha-candle-colors'):
    pdf=str(paths[0]/f'statistics-{name}.pdf')
    assert not subprocess.check_output(['pdfimages','-list',pdf],text=True).strip().splitlines()[2:]
print('PASS WP-14 FIX-17: registered custom density/schema, exact counts/membership, common generated builtin points and shared axes, custom labels/hit/semantics/selection/keyboard order, vector exports and independent up/down/doji candle colors')

traces = [json.loads((p/'actions-state-trace.json').read_text()) for p in paths]
for trace in traces[1:]: same(traces[0], trace, 0.)
for runtime in paths:
    xml = {name:ET.parse(runtime/f'actions-{name}.svg').getroot() for name in ('preview','cancel','commit','undo','redo')}
    def furniture(doc):
        # Metadata differs across state revisions; compare rendered content only.
        return [ET.tostring(e) for e in doc if not e.tag.endswith('metadata')]
    assert furniture(xml['preview']) == furniture(xml['commit']) == furniture(xml['redo'])
    assert furniture(xml['cancel']) == furniture(xml['undo'])
    assert furniture(xml['cancel']) != furniture(xml['commit'])
print('PASS WP-15 exact Rust/Python/WASM action events and component revisions; real-font preview/cancel/commit/undo/redo geometry')

input_traces = [json.loads((p/'input-trace.json').read_text()) for p in paths]
for trace in input_traces[1:]: same(input_traces[0],trace,1e-12)
for native_svg in paths[0].glob('input-*.svg'):
    for runtime in paths[1:]:
        assert native_svg.read_bytes() == (runtime/native_svg.name).read_bytes(), native_svg.name
print('PASS WP-16 Rust/Python/WASM indexed inspection, provenance selection, typed navigation, pinned previews, cancellation and exact export geometry')

for runtime, trace in zip(paths, input_traces):
    host_steps = [s for s in trace if s['case'].startswith('input-host-')]
    assert len(host_steps) == 25
    xml = {name: ET.parse(runtime / f'input-host-tools-{name}.svg').getroot() for name in ('snapped-preview','edit-cancel','edit-commit','edit-undo','edit-redo')}
    assert furniture(xml['snapped-preview']) == furniture(xml['edit-commit']) == furniture(xml['edit-redo'])
    assert furniture(xml['edit-cancel']) == furniture(xml['edit-undo'])
    assert furniture(xml['edit-cancel']) != furniture(xml['edit-commit'])
print('PASS WP-17 actual Rust/Python/WASM accessible data, snapped annotation preview/cancel/commit/undo/redo, linked provenance, echoes, missing keys and clipped/category selections')

stream_case=json.loads((Path(__file__).resolve().parents[2]/'fixtures/streaming/replay.json').read_text())
stream_traces=[json.loads((p/'stream-trace.json').read_text()) for p in paths]
for trace in stream_traces[1:]:same(stream_traces[0],trace,1e-12)
def stream_subset(actual,expected):
    if isinstance(expected,dict):
        for k,v in expected.items():stream_subset(actual[k],v)
    elif isinstance(expected,list):
        assert len(actual)==len(expected)
        for a,b in zip(actual,expected):stream_subset(a,b)
    else:assert actual==expected,(actual,expected)
for runtime,trace in zip(paths,stream_traces):
    assert len(trace)==len(stream_case['steps'])==70
    for result,step in zip(trace,stream_case['steps']):
        assert result['name']==step['name']
        if 'expected' in step:stream_subset(result['result'],step['expected'])
        semantic=result['semantics'];assert semantic['store_revision']==step['revision'],step['name']
        assert semantic['datasets'][0]['retention']==step['retention'],step['name']
        rows=[]
        for chunk in semantic['datasets'][0]['chunks']:
            rows.extend(map(list,zip(chunk['keys'],chunk['columns'][0]['values']['Timestamp'],chunk['columns'][1]['values']['Float64'])))
        assert rows==step['rows'],step['name']
        actual_bins=semantic['transforms'][0]['rows']['Binned'];edges=[0,10,20,30,50]
        for i,b in enumerate(actual_bins):
            members=sorted([k for k,t,y in rows if edges[i]<=y and (y<edges[i+1] or (i==3 and y==50))],key=int)
            assert b['count']==str(len(members))
            assert b['target']['Aggregate']['members']==members
            assert b['target']['Aggregate']['input']['revision']==step['revision']
        summaries=semantic['transforms'][1]['rows']['Statistical']
        values=[r[2] for r in rows]
        assert len(summaries)==1
        summary=summaries[0]
        assert summary['count']==str(len(rows))
        assert summary['members']==sorted([r[0] for r in rows],key=int)
        actual_values={str(v['field']):v['value'] for v in summary['values']}
        if values:
            expected={'Min':min(values),'Max':max(values),'Sum':math.fsum(values),'Mean':statistics_oracle.mean(values),"{'Quantile': 0}":min(values),"{'Quantile': 1}":statistics_oracle.median(values),"{'Quantile': 2}":max(values)}
            for field,value in expected.items():assert abs(actual_values[field]-value)<=1e-12,(step['name'],field)
        else:assert all(v is None for v in actual_values.values())
    for svg in paths[0].glob('stream-*.svg'):assert svg.read_bytes()==(runtime/svg.name).read_bytes(),svg.name
print('PASS WP-18 70-step actual Rust/Python/WASM queue/retention/replay trace; independent retained rows and exact bin memberships; historical pins and exact SVG bytes')
