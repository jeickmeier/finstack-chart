"""Independent primary Python symbol gallery across every built-in theme."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
kinds=['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','Asterisk','Diamond2','Square2','Triangle2']
for preset in ['Editorial','Terminal','Grayscale']:
    folder=out/preset;folder.mkdir(exist_ok=True)
    draft=c.plot(c.Data.columns({'x':[0.,5.],'y':[0.,15.]})).aes(c.aes().x('x').y('y')).theme(c.theme().preset(preset)).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,5.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,15.)).visible(False)).title(c.title(f'Symbols / area and stroke size / {preset}'))
    for i,kind in enumerate(kinds):
        y=13.5-i;d=c.Data.columns({'x':[2.,3.,4.],'y':[y,y,y],'size':[16.,64.,256.]},name=f'symbol-{i}',keys=[9007199254741001+i*10,9007199254741003+i*10,9007199254741005+i*10])
        draft=draft.layer(c.shape_symbol().name(f'symbol-{i}').data(d).symbol_kind(kind).symbol_paint('Fill' if i<7 else 'Stroke').shape_value('AreaSize','size').color('#2162a8' if i<7 else '#b55037')).layer(c.labels().id(f'symbol-label-{i}').at(.1,y).text(kind).style(c.text_style().size(.85)))
    for i,label in enumerate(['16','64','256']):draft=draft.layer(c.labels().id(f'size-{i}').at(2.+i,14.4).text(label).style(c.text_style().size(.85)))
    d=c.Data.columns({'x':[2.,3.,4.],'y':[.4,.4,.4],'size':[16.,64.,256.],'kind':['A','B','C']},name='mapped')
    draft=draft.layer(c.shape_symbol().name('mapped').data(d).symbol_types('kind',['A','B','C'],['Circle','Square','Plus']).symbol_title('Type').shape_value('AreaSize','size').symbol_size_guide('Area',[16.,64.,256.]).color('#2162a8')).layer(c.labels().id('mapped-label').at(.1,.4).text('Mapped').style(c.text_style().size(.85)))
    p=draft.build();(folder/'figure.plot.json').write_text(p.to_json());requests=[(dpi,output.request(p,c.export_options(600.,740.).dpi(dpi))) for dpi in [300,600]];p.dispose()
    for dpi,request in requests:
        frame=request.prepare();scene=frame.scene();assert len([i for i in scene['items'] if 'ShapePath' in i['primitive']])==42
        for i,item in enumerate(scene['items']):
            if 'ShapePath' in item['primitive']:assert len(item['primitive']['ShapePath']['anchors'])==len(scene['targets'][i])==1
        (folder/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
        for fmt in ['svg','pdf','png']:(folder/f'figure-{dpi}.{fmt}').write_bytes(frame.export(fmt))
        frame.dispose();request.dispose()
print('PASS Python symbol gallery: 13 types at three sizes, type/size guides and all three themes at 300/600 DPI; immutable exports after plot disposal.')
