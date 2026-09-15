"""FIX-GG05 actual primary authoring and immutable composed-guide publication."""
import json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(10):
    data=c.Data.columns({'x':[1.,4.,10.,100.],'g':['A','B','C','D'],'f':['one','one','two','two']})
    b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.))
    if mode<6:
        position=['Right','Left','Top','Bottom',{'Inside':{'x':.5,'y':.5}},{'Inside':{'x':.5,'y':.5}}][mode]
        options={'position':position,'ncol':2,'reverse':mode%2==1,'override_aes':{'size':4.}}
        b=b.aes(c.aes().x('x').y(1.).color('g').shape('g')).layer(c.points().legend({})).legend(c.legend().aesthetic('Color').options(options)).legend(c.legend().aesthetic('Shape').options(options))
        if mode==5:b=b.facet(c.facet_wrap('f').collect_guides(False))
    elif mode>=8:
        scale={'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0.,1.],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[1.,6.],'exponent':.5,'absolute':False}},'unknown':{'kind':'Missing'}}},'ggplot':{'Binned':{'oob':'Squish','right':True,'limits':[0.,20.],'breaks':{'Explicit':[5.,10.]}}},'guide':{'BinnedBins':'Automatic'},'colorbar_options':{'show_limits':mode==8}}
        b=b.layer(c.points().numeric_scale('Size','x',scale)).legend(c.legend().aesthetic('Size').options({'position':'Right' if mode==8 else 'Bottom','reverse':mode==9}))
    elif mode==7:
        b=b.layer(c.points()).legend(c.legend().custom({'id':'999','bounds':[0.,0.,40.,20.],'paths':[{'geometry':{'commands':[{'MoveTo':[0.,0.]},{'LineTo':[40.,0.]},{'LineTo':[20.,20.]},'Close']},'fill':'#123456','stroke':None}],'options':{'title':'Custom vector','position':'Left'}}))
    else:
        b=b.layer(c.points()).x_axis(c.x_axis().scale(c.scale_log(10.).domain(1.,100.)).label('Primary logarithmic axis').ggplot_axis({'n_dodge':2,'check_overlap':True,'cap':'Both','stack_order':0,'stack_spacing':6.})).guide(c.axis_guide('outer','x').side('Bottom').label('Stacked log ticks').ggplot_axis({'stack_order':1,'logticks':{'expanded':False}}))
    p=b.y_axis(c.y_axis().visible(False)).build();wire=p.to_json();assert json.loads(wire)['version']==(71 if mode==7 else 70 if mode==6 else 69)
    restored=c.Plot.from_json(wire);p.dispose();data.dispose()
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    if mode<6:assert sum(i.get('guide',{}).get('role')=='LegendKey' for i in scene['items'])==4
    elif mode>=8:
        assert sum(i.get('guide',{}).get('role')=='LegendKey' for i in scene['items'])==3
        assert sum(i.get('guide',{}).get('role')=='LegendTick' for i in scene['items'])==(4 if mode==8 else 2)
    elif mode==7:assert sum(i.get('guide',{}).get('scope')==['legend','custom'] for i in scene['items'])==2
    else:assert sum(i.get('guide',{}).get('scope')==['logtick'] for i in scene['items'])==19
    (out/f'guide-{mode}.plot.json').write_text(wire);(out/f'guide-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'guide-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose()
output.dispose();print('PASS Python guide composition: ten authors, 30 publications, owned controls and roundtrip.')
