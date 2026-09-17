"""FIX-GG11 independent Python spatial statistic and geometry authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(16):
    if 6<=mode<=9:
        n=2 if mode==9 else 5
        x=[];y=[];z=[]
        for row in range(n):
            for col in range(n):
                px=float(col)-(2. if n==5 else 0.);py=float(row)-(2. if n==5 else 0.)
                x.append(px);y.append(py);z.append(None if mode==8 and row==2 and col==2 else (0. if row==col else 2.) if mode==9 else px*px+py*py)
        data=c.Data.columns({'x':x,'y':y,'z':z})
        filled=mode in [7,8]
        levels={'breaks':[.5,1.,1.5] if mode==9 else [.5,2.,4.] if filled else [1.,2.,4.], 'bins':None,'binwidth':None}
        stat=c.spatial_stat({'Contour':{'levels':levels,'filled':filled}}).input('z')
        layer=(c.contour_filled() if filled else c.contour()).stat(stat)
    else:
        data=c.Data.columns({'x':[0.,.25,.5,1.,1.5,2.,2.],'y':[0.,.75,1.,.5,1.5,2.,0.],'z':[1.,2.,4.,8.,16.,32.,64.],'w':[1.,2.,0.,3.,1.,2.,1.]})
        axis={'breaks':None,'bins':30,'options':{'binwidth':1.,'boundary':0.}}
        if mode in [0,1]:
            stat=c.spatial_stat({'Rectangular':{'axes':[axis,axis],'summary':'Mean' if mode==1 else None,'drop':False}})
            layer=c.bin2d().stat(stat.input('z') if mode==1 else stat.weight('w'))
        elif mode in [2,3]:layer=c.hex().stat(c.spatial_stat({'Hexagonal':{'binwidth':[1.,1.],'bins':[30,30],'summary':'Median' if mode==3 else None,'drop':True}}).input('z').weight('w'))
        elif mode in [4,5,14,15]:
            filled=mode in [5,15]
            stat=c.spatial_stat({'Density':{'bandwidth':[1.,2.],'adjust':[1.,1.],'n':[25,25],'contour':None if mode==14 else {'breaks':None,'bins':None,'binwidth':None},'contour_var':'Normalized' if mode==15 else 'Density','filled':filled}}).weight('w')
            layer=(c.bin2d() if mode==14 else c.contour_filled() if filled else c.density2d()).stat(stat)
        elif mode in [10,11,12]:layer=c.ellipse().stat(c.spatial_stat({'Ellipse':{'kind':['Normal','T','Euclidean'][mode-10],'level':.8,'segments':51}}).weight('w'))
        else:layer=c.hex().stat(c.identity_stat()).recipe({'Hexagon':{'width':.4,'height':.3}}).fill('#D4A843')
    builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer)
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'spatial-{mode}.plot.json').write_text(wire);(out/f'spatial-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'spatial-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python spatial controls: sixteen authors, 48 publications.')
