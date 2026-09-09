"""Independently authored Python stack gallery using the primary chart engine."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());offsets=['None','Expand','Diverging','Silhouette','Wiggle']
for preset in ['Editorial','Terminal','Grayscale']:
    folder=out/preset;folder.mkdir(exist_ok=True);rows=[]
    for oi,offset in enumerate(offsets):
      for kind,form in enumerate(['Area','Bars']):
       for sample in range(6):
        for g,name in enumerate(['A','B','C']):
         if sample==2 and g==1:continue
         rows.append((float(sample),-2. if sample==3 and g==2 else float((sample+2*g)%4+1),name,offset,form,2*oi+kind,9007199254741001+oi*100+kind*30+sample*3+g))
    data=c.Data.columns({'x':[r[0]for r in rows],'y':[r[1]for r in rows],'group':c.categorical([r[2]for r in rows]),'offset':c.categorical([r[3]for r in rows]),'form':c.categorical([r[4]for r in rows]),'slot':[r[5]for r in rows]},keys=[r[6]for r in rows],name='stacks')
    draft=c.plot(data).aes(c.aes().x('x').x2('x').y('y').y2(0.).group('group').color('group')).theme(c.theme().preset(preset)).title(c.title(f'Stack offsets / InsideOut / {preset}')).subtitle(c.subtitle('Signed heights, a missing sample, and explicit zero filling')).facet(c.facet_grid('offset','form').free_y(True).gap(12.)).x_axis(c.x_axis().scale(c.scale_linear().domain(-.5,5.5))).scale(c.color_discrete('group').domain(['A','B','C'])).legend(c.legend().scale('group').title('Series'))
    for oi,offset in enumerate(offsets):
      for kind in range(2):
        layer=c.shape_area()if kind==0 else c.bars().width(15.);slot=float(2*oi+kind)
        draft=draft.layer(layer.name(f'{offset}-{kind}').filter(c.filter('slot').minimum(slot).maximum(slot)).position(c.shape_stack(['A','B','C']).stack_order('InsideOut').stack_offset(offset).stack_missing('Zero')))
    p=draft.build();(folder/'figure.plot.json').write_text(p.to_json());requests=[(dpi,output.request(p,c.export_options(600.,740.).dpi(dpi)))for dpi in [300,600]];p.dispose()
    for dpi,request in requests:
        f=request.prepare();scene=f.scene();assert sum('ShapePath'in i['primitive']for i in scene['items'])==15;assert sum(len(t)for t in scene['targets'])==170
        for i,item in enumerate(scene['items']):
            if 'ShapePath'in item['primitive']:assert len(item['primitive']['ShapePath']['anchors'])==len(scene['targets'][i])
        (folder/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
        for fmt in ['svg','pdf','png']:(folder/f'figure-{dpi}.{fmt}').write_bytes(f.export(fmt))
        f.dispose();request.dispose()
print('PASS Python stack gallery: five offsets, InsideOut rank, sparse areas/bars, 170 actual targets, three themes and immutable 300/600 DPI outputs.')
