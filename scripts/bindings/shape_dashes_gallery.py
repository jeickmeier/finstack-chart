"""FIX-S09 complete dashed curves restored from Rust-authored definitions."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
source=Path(sys.argv[2]);out=Path(sys.argv[3]);output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for preset in ['Editorial','Terminal','Grayscale']:
    folder=out/preset;folder.mkdir(parents=True,exist_ok=True);wire=(source/preset/'figure.plot.json').read_text();p=c.Plot.from_json(wire);assert p.to_json()==wire;(folder/'figure.plot.json').write_text(p.to_json())
    for dpi in [300,600]:
        request=output.request(p,c.export_options(680.,640.).dpi(dpi));frame=request.prepare();scene=frame.scene();assert sum('ShapePath'in item['primitive']for item in scene['items'])==39
        assert sum('dashes' in item['primitive'].get('ShapePath',{}) for item in scene['items'])==20
        (folder/f'figure-{dpi}.scene.json').write_text(json.dumps(scene))
        for extension in ['svg','pdf','png']:(folder/f'figure-{dpi}.{extension}').write_bytes(frame.export(extension))
        frame.dispose();request.dispose()
    p.dispose()
print('PASS Python dashed publication: retained curve dash patterns, three themes and 300/600 DPI SVG/PDF/PNG.')
