"""FIX-21 independently authored chart colors for every normalization/classifier family."""
from pathlib import Path
import json,math,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
corpus=json.loads((ROOT/'fixtures/parity/d3-scale-chromatic/composition.json').read_text())
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(320.,160.)
samples=0
for case in corpus['cases']:
    selection=case['selection'];opts=case['options'].copy();ramp=None
    if case['family']!='ordinal':opts['domain']=[float(v) for v in opts['domain']] # JavaScript Number oracle; Python integers are distinct exact category keys.
    if 'ramp'in selection:opts['interpolator']=ramp=c.chromatic(selection['ramp'],selection['reverse'])
    scale=c.StandaloneScale(case['family'],**opts)
    mapping=c.color_mapped('named',scale).missing('#0b16212c')
    if 'scheme'in selection:mapping=mapping.palette_scheme({'id':selection['scheme'],'size':selection['size'],'reverse':selection['reverse']})
    values=[{'NaN':math.nan,'Infinity':math.inf,'-Infinity':-math.inf}[s['input']['number']] if isinstance(s['input'],dict) else s['input'] for s in case['samples']]
    # Empty/invalid reference outcomes are diagnostics, never a substitute color.
    assert all(s['expected'] is not None for s in case['samples'])
    p=c.plot(c.Data.columns({'x':[float(i) for i in range(len(values))],'value':values})).aes(c.aes().x('x').y(1.).color('value').color_scale('named')).layer(c.points()).scale(mapping).build()
    request=output.request(p,options);frame=request.prepare();paints=[i['primitive']['Point']['fill'] for i in frame.scene()['items'] if 'Point'in i['primitive'] and i.get('layer') is not None]
    actual=[[v['red'],v['green'],v['blue'],v['alpha']] for v in paints];expected=[s['expected'] for s in case['samples']]
    assert actual==expected,(case['id'],actual,expected);samples+=len(actual)
    for v in [frame,request,p,scale]:v.dispose()
    if ramp:ramp.dispose()
assert samples==1268
Path(sys.argv[2]).write_text(json.dumps({'cases':90,'samples':samples,'exact':'actual scene RGBA'},indent=2)+'\n')
print('PASS FIX-21 Python: 90 composed mappings / 1268 exact scene colors across all sequential/diverging/rank/classifier families.')
# Theme conversion occurs after named evaluation and applies equally to marks and guides.
for theme,mark,ends in [('Editorial',[33,145,140],[[68,1,84],[253,231,37]]),('Terminal',[33,145,140],[[68,1,84],[253,231,37]]),('Grayscale',[121]*3,[[21]*3,[222]*3])]:
    ramp=c.chromatic('Viridis');scale=c.StandaloneScale('sequential',domain=[0.,1.],interpolator=ramp)
    p=(c.plot(c.Data.columns({'x':[.5]})).aes(c.aes().x('x').y(1.).color('x').color_scale('named')).layer(c.points())
       .scale(c.color_mapped('named',scale)).legend(c.legend().scale('named')).theme(c.theme().preset(theme)).build())
    request=output.request(p,options);frame=request.prepare();scene=frame.scene()['items'];rgb=lambda v:[v['red'],v['green'],v['blue']]
    marks=[rgb(i['primitive']['Point']['fill']) for i in scene if 'Point'in i['primitive'] and i.get('layer') is not None]
    chips=[rgb(i['primitive']['Rectangle']['fill']) for i in scene if 'Rectangle'in i['primitive'] and i.get('clip') is not None and i['primitive']['Rectangle']['bounds']['width']==i['primitive']['Rectangle']['bounds']['height']]
    assert marks==[mark] and chips==ends,(theme,marks,chips)
    for v in [frame,request,p,scale,ramp]:v.dispose()
print('PASS FIX-21 Python: editorial/terminal/grayscale preserve canonical evaluation and convert both marks and guide swatches.')
