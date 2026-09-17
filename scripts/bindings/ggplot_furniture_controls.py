"""Independent GG14 Python title/caption/tag authors."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(8):
    position='plot' if mode==1 else 'panel'
    location,tag={2:('margin','topleft'),3:('margin','bottomright'),4:('panel','bottomleft'),5:('panel','top'),6:('plot','topright')}.get(mode,('plot','topright'))
    elements={name:{'Value':{'Text':position}} for name in ['plot.title.position','plot.caption.position']}
    elements['plot.tag.location']={'Value':{'Text':location}}
    elements['plot.tag.position']={'Value':{'Vector':[{'Number':.5},{'Number':.5}]} if mode==5 else {'Text':tag}}
    if mode==7:elements['plot.tag']='Blank'
    data=c.Data.columns(dict(x=[0.,1.,2.,3.],y=[1.,3.,2.,4.],group=['a','a','b','b']))
    builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).theme(c.theme().reference_preset('Grey',{}).update_elements({'elements':elements})).title(c.title('Figure alignment')).subtitle(c.subtitle('Explicit panel or plot span')).caption(c.caption('Caption aligned to the same selected span')).tag(c.rich_text('A'))
    if mode==6:builder=builder.facet(c.facet_wrap('group'))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600,360).dpi(144));frame=request.prepare()
    (out/f'furniture-{mode}.plot.json').write_text(wire);(out/f'furniture-{mode}.scene.json').write_text(json.dumps(frame.scene()))
    for fmt in ['svg','pdf','png']:(out/f'furniture-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python furniture controls: eight authors, 24 publications')
