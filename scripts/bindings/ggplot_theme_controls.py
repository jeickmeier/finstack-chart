"""FIX-GG14 independent Python precomputed theme controls authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(16):
    data=c.Data.columns({'x':[1.,2.,3.,4.],'y':[2.,1.,4.,3.],'g':['A','A','B','B']})
    presets=['Grey','Bw','Linedraw','Light','Dark','Minimal','Classic','Void','Test']
    themed=c.theme().reference_preset(presets[min(mode,8)],{})
    if mode==9:themed=c.theme().reference_preset('Grey',{'base_size':16.,'ink':'#123456','paper':'#F8EEDD','accent':'#D020A0'})
    elif mode==10:themed=themed.update_elements({'complete':False,'elements':{'axis.text':'Blank','axis.text.x':{'Element':{'kind':'Text','properties':{'inherit.blank':{'Bool':False},'colour':{'Text':'red'},'size':{'Relative':1.5}}}}}})
    elif mode==11:themed=themed.update_elements({'complete':False,'elements':{'axis.ticks.length.y':{'Value':{'Unit':[{'value':-2.,'unit':'mm'}]}},'axis.line.y':{'Element':{'kind':'Line','properties':{'colour':{'Text':'#008080'},'linewidth':{'Number':1.},'arrow':{'Arrow':{'angle':25.,'length_mm':4.,'ends':'Both','closed':True}},'arrow.fill':{'Text':'red'},'lineend':{'Text':'square'},'linetype':{'Text':'dashed'}}}}}})
    if mode==12:themed=themed.update_elements({'complete':False,'elements':{'panel.spacing.x':{'Value':{'Unit':[{'value':8.,'unit':'mm'}]}},'legend.position':{'Value':{'Text':'bottom'}},'strip.background':{'Element':{'kind':'Rect','properties':{'fill':{'Text':'#CCEEDD'}}}}}})
    elif mode==13:themed=themed.update_elements({'complete':False,'elements':{'axis.text.theta':{'Element':{'kind':'Text','properties':{'colour':{'Text':'#CC2200'},'size':{'Number':14.}}}}}})
    if mode==14:
        themed=c.theme().reference_preset('Test',{'base_family':'Fixture Font'}).fonts([{'family':'Fixture Font','face':'plain','font':{'id':'1','revision':'0','kind':'Font','byte_len':str((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').stat().st_size)},'weight':400}]).update_elements({'complete':False,'elements':{'axis.text.x':{'Element':{'kind':'Text','properties':{'angle':{'Number':35.}}}}}})
    if mode==15:themed=themed.update_elements({'complete': False, 'elements': {'legend.frame': {'Element': {'kind': 'Rect', 'properties': {'fill': 'Missing', 'colour': {'Text': 'red'}}}}, 'legend.ticks': {'Element': {'kind': 'Line', 'properties': {'colour': {'Text': '#00AA55'}, 'linewidth': {'Number': 1.0}}}}, 'legend.title.position': {'Value': {'Text': 'left'}}, 'legend.position': {'Value': {'Text': 'bottom'}}}})
    if mode==12:themed=themed.update_elements({'complete': False, 'elements': {'panel.widths': {'Value': {'Unit': [{'value': 1.0, 'unit': 'null'}, {'value': 3.0, 'unit': 'null'}]}}, 'strip.placement': {'Value': {'Text': 'outside'}}, 'strip.clip': {'Value': {'Text': 'off'}}, 'legend.box': {'Value': {'Text': 'horizontal'}}, 'legend.box.just': {'Value': {'Text': 'top'}}, 'legend.spacing.x': {'Value': {'Unit': [{'value': 6.0, 'unit': 'mm'}]}}, 'legend.title.position': {'Value': {'Text': 'left'}}, 'legend.key.spacing.x': {'Value': {'Unit': [{'value': 2.0, 'unit': 'mm'}]}}}})
    context=c.ThemeContext(themed);captured=context.get();context.update({"complete":False,"elements":{}});previous=context.set(captured);previous.dispose();context.replace({"complete":False,"elements":{}});themed=context.get();context.dispose();captured.dispose()
    builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points().aes(c.aes().size("x")) if mode==12 else c.points()).title(c.title('Thème • Δοκιμή • Пример' if mode==14 else 'Reference theme')).subtitle(c.subtitle('Inheritance and physical controls')).caption(c.caption('Explicit shared text service')).theme(themed)
    if mode==12:builder=builder.aes(c.aes().x("x").y("y").color("g").color_scale("groups")).scale(c.color_discrete("groups").domain(["A","B"])).facet(c.facet_wrap("g").columns(2).reference({}))
    elif mode==13:builder=builder.coordinate({"Radial":{}})
    if mode==14:builder=builder.layer(c.points().text_defaults().text_label("g"))
    if mode==15:builder=builder.aes(c.aes().x("x").y("y").color("x").color_scale("values")).scale(c.color_mapped("values",{'training': 'Eligible', 'function': {'Interpolated': {'normalization': {'Sequential': {'family': 'Linear', 'domain': [1.0, 4.0], 'clamp': False}}, 'output': {'Interpolate': {'operation': 'GgplotPalette', 'spec': {'Gradient': {'colors': ['#112244', '#FFCC22'], 'values': [0.0, 1.0]}}}}, 'unknown': {'kind': 'Missing'}}}, 'ggplot': {'Continuous': {'limits': [1.0, 4.0], 'empty_population': False, 'nonfinite_population': False, 'oob': 'Censor'}}, 'guide': {'Colorbar': {'breaks': [1.0, 2.0, 3.0, 4.0]}}}))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'theme-{mode}.plot.json').write_text(wire);(out/f'theme-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'theme-{mode}.{fmt}').write_bytes(frame.export(fmt))
    if mode==14:
        for dpi in [300,600]:
            for text in ['preserve','outline']:
                variant_request=output.request(restored,c.export_options(600.,360.).dpi(dpi).text(text));variant=variant_request.prepare();assert variant.scene()==scene
                for fmt in ['svg','pdf','png']:(out/f'theme-{mode}.{text}-{dpi}.{fmt}').write_bytes(variant.export(fmt))
                variant.dispose();variant_request.dispose()
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python theme controls: sixteen authors, 60 publications including text/outline at300/600dpi.')
