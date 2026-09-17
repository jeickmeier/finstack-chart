"""GG17 actual raster encoding through Python's primary authoring API."""
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
data=c.Data.columns({'x':[0.,1.,2.],'y':[1.,3.,2.]})
p=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points()).build()
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for dpi in [72,144]:
    options=c.export_options(360.,240.).dpi(dpi).raster_device({'tiff_compression':'Lzw','tiff_predictor':True})
    request=output.request(p,options);frame=request.prepare()
    for fmt in ['jpeg','tiff','bmp','png','ps','eps','tex','emf']:(out/f'device-{dpi}.{fmt}').write_bytes(frame.export(fmt))
    for compression in ['Jpeg','Lzma','Zstd','Webp']:
        extra_options=c.export_options(360.,240.).dpi(dpi).raster_device({'tiff_compression':compression})
        extra_request=output.request(p,extra_options);extra=extra_request.prepare()
        (out/f'codec-{dpi}-{compression.lower()}.tiff').write_bytes(extra.export('tiff'))
        extra.dispose();extra_request.dispose();extra_options.dispose()
    pages=frame.pages();pages.append(frame)
    frame.dispose();request.dispose();options.dispose()
    for fmt in ['ps','pdf','tiff']:(out/f'pages-{dpi}.{fmt}').write_bytes(pages.export(fmt))
    pages.dispose()
output.dispose();p.dispose();data.dispose()
print('PASS 30 Python device publications at 72/144 DPI.')
