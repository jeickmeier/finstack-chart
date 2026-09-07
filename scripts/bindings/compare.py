"""Compare actual native, Python and WASM results plus independent fixture expectations."""
import hashlib
import json
import math
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
        assert layer(name)['operations'][0]['incremental']=={'append':False,'window':False,'correction':False,'full_recompute':True}
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
for data, scenes in zip(statistics,scenes):
    def family(name): return data['family-'+name]['layers']
    def marks(name,layer='1'): return [i['primitive'] for i in scenes['family-'+name]['items'] if i['layer']==layer]
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
    labels=[i['primitive']['Text']['text'] for i in scenes['family-utc-leap']['items'] if 'Text' in i['primitive']]
    assert any('2024-02-29' in t for t in labels)
    assert family('stacked-bars')[0]['domains']['y']=={'minimum':-5.,'maximum':5.}
for name in statistics[0]:
    if name.startswith('family-'):
        pdf=str(paths[0]/f'statistics-{name}.pdf')
        assert not subprocess.check_output(['pdfimages','-list',pdf],text=True).strip().splitlines()[2:]
print('PASS WP-11: independent log/run gaps, area/ribbon boundaries, color/missing metadata, price/volume validation, supplied sessions, leap day, signed stack and vector PDF expectations')
