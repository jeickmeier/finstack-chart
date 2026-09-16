"""FIX-GG09 independent Python precomputed distribution geometry authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
red={'red':255,'green':0,'blue':0,'alpha':255};gold={'red':255,'green':215,'blue':0,'alpha':255}
for mode in range(15):
    if mode>=11:
        data=c.Data.columns({'x':[0.,0.,.2,1.,1.2,2.,2.,2.]});mapping=c.aes()
        layer=c.density().stat(c.density_stat().input('x')).recipe({'Density':{'outline':['Upper','Lower','Both','Full'][mode-11]}}).fill(gold).alpha(.2)
    elif mode>=8:
        data=c.Data.columns({'x':[1.]*8,'y':[0.,1.,1.,2.,3.,4.,5.,20.]})
        mapping=c.aes()
        layer=c.boxplot().stat(c.boxplot_stat().input(data.field('y')).x('x')) if mode==8 else c.violin().stat(c.violin_stat().input(c.source_expr(data.field('y'))).x('x')) if mode==9 else c.dotplot().stat(c.dotplot_stat().input('y'))
    elif mode<4:
        data=c.Data.columns({'x':[1.,2.],'middle':[2.5,3.],'lo':[.75,2.],'hi':[3.25,4.],'min':[0.,1.],'max':[4.,5.],'nl':[1.1034641071565687,1.5868050382201329],'nu':[3.8965358928434313,4.413194961779867],'relative':[math.sqrt(8),math.sqrt(5)]},keys=[10,20])
        spec={'width':.75,'notch':mode==1,'notch_width':.3,'staple_width':0. if mode==0 else .7,'variable_width':mode==2,'source_outliers':[{'row':'10','values':[20.]}]}
        if mode==2:spec.update(median={'color':red,'linewidth':1.},whisker={'line_type':'Dashed'},outlier={'shape':21,'fill':gold,'size':3.})
        layer=c.rule().recipe({'Boxplot':spec})
        for channel,field in [('Lower','lo'),('Upper','hi'),('Middle','middle'),('WhiskerLower','min'),('WhiskerUpper','max'),('NotchLower','nl'),('NotchUpper','nu'),('RelativeWidth','relative')]:layer=layer.recipe_value(channel,field)
        mapping=c.aes().x('x').y('middle').group('x')
        if mode==3:mapping=c.aes().x('middle').y('x').group('x');layer=layer.orientation('Horizontal')
    elif mode==4:
        data=c.Data.columns({'x':[1.]*5,'y':[0.,1.,2.,3.,4.],'width':[.1,.7,1.,.5,.1],'q':[None,.25,.5,.75,None]},keys=[1,2,3,4,5])
        mapping=c.aes().x('x').y('y');layer=c.rule().recipe({'Violin':{'width':.8,'quantile':{'line_type':'Dashed','color':red}}}).recipe_value('ViolinWidth','width').recipe_value('QuantileFlag','q')
    else:
        data=c.Data.columns({'bin':[0.,1.,2.],'count':[3.,1.,4.]},keys=[1,2,3]);mapping=c.aes().x(1.).y('bin') if mode==6 else c.aes().x('bin').y(0.)
        layer=c.rule().recipe({'Dotplot':{'bin_axis':'Y' if mode==6 else 'X','stack':'CenterWhole' if mode==7 else 'Up','stack_ratio':.8,'dot_size':.7}}).recipe_value('Count','count').recipe_value('BinWidth',.5)
    p=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(layer).build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'distribution-{mode}.plot.json').write_text(wire);(out/f'distribution-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'distribution-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python distribution geometry: fifteen authors, 45 publications.')
