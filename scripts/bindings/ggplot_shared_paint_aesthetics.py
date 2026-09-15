"""FIX-GG04: one scale trains the joint colour/fill population through actual hosts."""
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
cases = json.loads((ROOT / 'fixtures/parity/ggplot2/shared-paint-aesthetics.json').read_text())['cases']
registry = c.ExtensionRegistry.example()
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(640, 360).dpi(96).basis('current')
records = []
def descriptor(family):
    s = {'training':'Eligible', 'guide':'Hidden', 'function':{'Ordinal':{'domain':[], 'range':[], 'unknown':{'Explicit':None}}}, 'ggplot':{'Discrete':{'limits':None, 'levels':None, 'drop':True, 'na_translate':True, 'palette':{'Hue':{'h':[15,375], 'chroma':100, 'luminance':65, 'start':0, 'reverse':False}}}}}
    if family in ('continuous','binned'):
        s['function'] = {'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range'}}, 'output':{'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}}, 'unknown':{'kind':'Missing'}}}
        s['ggplot'] = {'Continuous':{'limits':None,'oob':'Censor'}} if family == 'continuous' else {'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':5},'right':True}}
    if family == 'manual':
        s['ggplot']['Discrete']['palette'] = {'Manual':{'values':[{'kind':'Text','value':v} for v in ('#FF0000','#008000','#0000FF')], 'names':None}}
    if family == 'identity':
        del s['ggplot']
        s['function'] = {'GgplotDiscreteIdentity':{'limits':None,'levels':None,'drop':True,'na_translate':True,'guide':False,'observed':[]}}
    return s
def layer(): return c.points().name('marks').size(4).aesthetic_value('Shape', {'kind':'Number','value':21})
def paint(raw):
    if raw is None: return {'red':0,'green':0,'blue':0,'alpha':0}
    v = '#7F7F7F' if raw == 'grey50' else raw
    return dict(zip(('red','green','blue','alpha'), [int(v[i:i+2],16) for i in (1,3,5)] + [int(v[7:9],16) if len(v)==9 else 255]))
for index, t in enumerate(cases):
    owned=[]
    try:
        kind='float64' if t['family'] in ('continuous','binned') else 'string'
        d=c.Data.columns({'x':c.column(list(map(float,range(len(t['colour'])))),kind='float64'),'colour':c.column(t['colour'],kind=kind),'fill':c.column(t['fill'],kind=kind)},name='data'); owned.append(d)
        p=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x('x').y(1).color('colour').fill('fill').color_scale('shared').fill_scale('shared')).scale(c.color_mapped('shared',descriptor(t['family']))).layer(layer()).build(); owned.append(p)
        wire=p.to_json(); restored=c.Plot.from_json(wire,registry); owned.append(restored); assert restored.to_json()==wire
        for state in ('original','layer_edit','theme_edit'):
            current=restored if state=='original' else (restored.edit().layer('marks',layer()).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build())
            if state!='original': owned.append(current)
            chart=current.chart(); owned.append(chart)
            styles=chart.semantics()['layers'][0].get('styles',[])
            wanted=[(a,b) for a,b in zip(t['result']['colour'],t['result']['fill']) if a is not None]
            assert len(styles)==len(wanted)==t['result']['mark_count'],t
            for s,(a,b) in zip(styles,wanted): assert s['color']==paint(a) and s['fill']==paint(b),(t,s,a,b)
            records.append({'index':index,'state':state,'styles':styles}); assert restored.to_json()==wire
        if t['population']=='ordinary':
            request=output.request(restored,options); owned.append(request); frame=request.prepare(); owned.append(frame)
            for fmt in ('svg','pdf','png'): (out/f"{t['family']}.{fmt}").write_bytes(frame.export(fmt))
    finally:
        for o in reversed(owned): o.dispose()
assert len(records)==45
(out/'records.json').write_text(json.dumps(records,sort_keys=True))
options.dispose(); output.dispose(); registry.dispose()
print('PASS 45 joint paint states and 15 publication files')
