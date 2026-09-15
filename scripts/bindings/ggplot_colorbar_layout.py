"""FIX-GG05: actual Python colorbar geometry, lifecycle and immutable publication."""
import json
import math
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
sampling = '--sampling' in sys.argv
demand = '--demand' in sys.argv
constant = '--constant' in sys.argv
orientation = '--orientation' in sys.argv
presentation = '--presentation' in sys.argv
step_controls = '--steps-controls' in sys.argv
default_all = '--default-steps-all' in sys.argv
default_steps = default_all or '--default-steps' in sys.argv
boundaries = '--steps-boundaries' in sys.argv or step_controls or default_steps
steps = '--steps' in sys.argv or boundaries
alpha = '--alpha' in sys.argv
display = '--display' in sys.argv or alpha or steps
reference = json.loads((ROOT / ('fixtures/parity/ggplot2/colorbar-boundaries.json' if sampling else 'fixtures/parity/ggplot2/colorbar-layout.json')).read_text())['cases']
if sampling: reference = [q for q in reference if q['display']=='raster' and not q['reverse'] and q['direction']=='vertical']
if demand: reference = []
if constant: reference = json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-constant.json').read_text())['cases']
if orientation:
    reference = [q for q in json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-boundaries.json').read_text())['cases'] if q['display']=='raster']
if presentation:
    reference = json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-presentation.json').read_text())['cases']
    for q in reference: q['result']['values']=q['result']['keys']
if display:
    reference = json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-display.json').read_text())['cases']
    for q in reference: q['result']['values']=q['result']['keys']
    reference += [q for q in json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-boundaries.json').read_text())['cases'] if q['display']!='raster']
if alpha:
    reference = json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-alpha.json').read_text())['cases']
    for q in reference: q['result']['values']=q['result']['keys']
if steps:
    reference=[q for q in json.loads((ROOT/'fixtures/parity/ggplot2/colorsteps-layout.json').read_text())['cases'] if q['even_steps'] and not q['show_limits']]
    for q in reference:
        q['display']='rectangles';q['nbin']=None
        r=q['result'];r['decor_colors']=[c for _,c in sorted(zip([min(a,b) for a,b in zip(r['decor']['min'],r['decor']['max'])],r['decor']['colour']))]
        r['values']=r['key']['.value'];r['labels']=r['key']['.label']
if boundaries:
    reference=[q for q in json.loads((ROOT/('fixtures/parity/ggplot2/colorsteps-controls.json' if step_controls else 'fixtures/parity/ggplot2/colorsteps-default-boundaries.json' if default_steps else 'fixtures/parity/ggplot2/colorsteps-boundaries.json')).read_text())['cases'] if not(q['family']=='binned' and q['mode']=='null')]
    if default_steps and not default_all: reference=[q for q in reference if q['population']!='constant']
    for q in reference:
        q.update(palette='asymmetric',display='rectangles',nbin=None,direction='vertical',reverse=False,constant=q['population']=='constant')
        r=q['result']
        if 'error' not in r:
            r['decor_colors']=(r.get('decor') or {}).get('colour',[])
            r['values']=(r.get('key') or {}).get('.value',[]);r['labels']=(r.get('key') or {}).get('.label',[])
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
records = []
def paint(text):
    if text == 'grey50': return dict(red=127,green=127,blue=127,alpha=255)
    return dict(red=int(text[1:3], 16), green=int(text[3:5], 16), blue=int(text[5:7], 16), alpha=int(text[7:9],16) if len(text)==9 else 255)
def scale(palette, hidden=False, nbin=None, constant=False, controls=None):
    colors, values = {
        'ordinary': (['#000000', '#ffffff'], [0., 1.]),
        'transparent': (['#0000ff20', '#ffffff80', '#ff0000e0'], [0., .2, 1.]),
        'asymmetric': (['#0000ff', '#ffffff', '#ff0000'], [0., .2, 1.]),
        'discontinuous': (['#ff0000', '#ff0000', '#0000ff', '#0000ff'], [0., .499, .5, 1.]),
    }[palette]
    result = {**({'colorbar_options':{'nbin':nbin}} if nbin is not None else {}), 'training':'Eligible', 'function':{'Interpolated':{
        'normalization':{'Sequential':{'family':'Linear', 'domain':[-2.,8.], 'clamp':False}},
        'output':{'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[paint(v) for v in colors], 'values':values}}}},
        'unknown':{'kind':'Missing'}}},
        'ggplot':{'Continuous':{'limits':([2.,2.] if display and constant else [3.,3.] if constant else [-2.,8.]), 'oob':'Censor'}},
        'guide':'Hidden' if hidden else {'Colorbar':{'breaks':([2.] if display and constant else [3.] if constant else [-2.,0.,3.,8.]), 'labels':('Hidden' if controls and controls['hidden_labels'] else 'Automatic')}}}
    if controls: result.setdefault('colorbar_options', {}).update({k:v for k,v in controls.items() if k not in ('hidden_labels','steps_family','steps_endpoints','steps_default','boundary')})
    if steps and controls:
        cuts=[-2.,0.,3.,8.] if controls['steps_endpoints'] else [0.,3.]
        if controls['steps_family']=='binned':
            result['function']['Interpolated']['normalization']={'Ggplot':{'family':'Linear','domain':[-2.,8.],'reverse':False,'rescaler':'Range'}}
            result['ggplot']={'Binned':{'limits':[-2.,8.],'breaks':{'Explicit':cuts},'oob':'Squish','right':True}}
            result['guide']='Hidden' if hidden else {'Binned' if controls['steps_default'] else 'BinnedSteps':'Automatic'}
        else: result['guide']='Hidden' if hidden else {'ContinuousSteps':{'breaks':cuts,'labels':'Automatic'}}
    if boundaries and controls:
        q=controls['boundary']; cuts=q['breaks']; limits=q['limits']
        if isinstance(cuts,list): cuts=[{'number':'NaN'} if v is None else {'number':v} if isinstance(v,str) else v for v in cuts]
        result['function']['Interpolated']['normalization']={'Ggplot':{'family':'Linear','domain':limits,'reverse':False,'rescaler':'Range'}}
        if q['family']=='binned':
            result['ggplot']={'Binned':{'limits':limits,'breaks':{'Nice':5.} if cuts=='automatic' else {'Explicit':cuts},'oob':'Squish','right':True}}
            result['guide']='Hidden' if hidden else {'Binned' if default_steps else 'BinnedSteps':'Automatic'}
        else:
            result['ggplot']={'Continuous':{'limits':limits,'oob':'Censor'}}
            result['guide']='Hidden' if hidden or q['mode']=='null' else {'ContinuousSteps':{'breaks':None if cuts=='automatic' else cuts,'labels':'Automatic'}}
    return result
def author(palette, channel, facet, nbin=None, constant=False, controls=None):
    data = c.Data.columns({'x':[1.,2.,3.,4.,1.,2.,3.,4.], 'v':([2.]*8 if display and constant else [-2.,0.,3.,8.,-2.,0.,3.,8.]), 'f':['A']*4+['B']*4})
    if boundaries and controls and controls['boundary']['population']=='empty':
        data.dispose();data=c.Data.columns({'x':c.column([],kind='float64'),'v':c.column([],kind='float64'),'f':c.column([],kind='string')})
    aes = c.aes().x('x').y(1.)
    aes = getattr(getattr(aes, channel)('v'), channel+'_scale')('v')
    builder = c.plot(data).profile('Ggplot2_4_0_3').aes(aes).scale(c.color_mapped('v', scale(palette, nbin=nbin, constant=constant, controls=controls))).layer(c.points())
    if facet != 'single': builder = builder.facet(c.facet_wrap('f').collect_guides(facet == 'collected'))
    plot = builder.build(); data.dispose(); aes.dispose(); builder.dispose()
    return plot
def close(a,b):
    assert math.isclose(a,b,rel_tol=0.,abs_tol=3e-12),(a,b)
def inspect(plot, case, channel, facet, state):
    chart = plot.chart(); semantic = chart.semantics(); chart.dispose()
    assert len(semantic['layers']) == (1 if facet == 'single' else 2)
    styles = [style for layer in semantic['layers'] for style in layer.get('styles',[])]
    expected = [paint(v) for v in case['result']['mapped']] * 2
    assert [s[channel] for s in styles] == expected,(case['palette'],channel,facet,[s[channel] for s in styles],expected)
    request = output.request(plot, c.export_options(600,360).dpi(144)); frame = request.prepare(); scene = frame.scene()
    direction=case['direction'].capitalize()
    paired=[(i['primitive']['SampledGradientRectangle'],i.get('clip')) for i in scene['items'] if 'SampledGradientRectangle' in i['primitive']]
    if not (step_controls and not case['even_steps']) and (len(case['result']['decor_colors'])==1 or (display and case.get('constant') and case['display']=='gradient')):
        for item in scene['items']:
            r=item['primitive'].get('Rectangle')
            if r is not None and (abs(r['bounds']['height']/r['bounds']['width']-20/3)<1e-10 or abs(r['bounds']['width']/r['bounds']['height']-20/3)<1e-10):
                paired.append((dict(bounds=r['bounds'],direction=direction,colors=[r['fill']]),item.get('clip')))
    if step_controls and not case['even_steps']:
        cells=[(i['primitive']['Rectangle'],i.get('clip')) for i in scene['items'] if i.get('layer') is None and 'Rectangle' in i['primitive'] and i.get('clip') is not None]
        if cells:
            top=min(r['bounds']['origin']['y'] for r,_ in cells);bottom=max(r['bounds']['origin']['y']+r['bounds']['height'] for r,_ in cells)
            b=dict(origin=dict(x=cells[0][0]['bounds']['origin']['x'],y=top),width=cells[0][0]['bounds']['width'],height=bottom-top)
            assert len(cells)==len(case['result']['bar']['height'])
            for (r,_),height in zip(cells,case['result']['bar']['height']):close(r['bounds']['height']/b['height'],height)
            paired.append((dict(bounds=b,direction=direction,colors=[r['fill'] for r,_ in reversed(cells)],mode='Steps'),cells[0][1]))
    bars=0 if state=='hidden' or (boundaries and (not case['result']['decor_colors'] or (step_controls and case['result']['bar'] is None))) else 2 if facet=='local' else 1
    assert len(paired)==bars,(case['palette'],channel,facet,state,len(paired))
    normalized=[]
    expected_ticks=[v for v in case['result'].get('tick_positions',case['result']['values']) if v is not None]
    expected_labels=[label for label,value in zip(case['result']['labels'],case['result']['values']) if value is not None]
    for ramp,clip in paired:
        assert ramp['direction']==direction
        expected_colors=[paint(v) for v in case['result']['decor_colors']]
        if display and case.get('constant') and case['display']=='gradient': expected_colors=expected_colors[:1]
        if display and case['display']=='gradient' and len(expected_colors)>1: assert ramp['mode']=='Endpoints'
        assert ramp['colors']==(expected_colors if direction=='Horizontal' else list(reversed(expected_colors)))
        if display and case['display']=='rectangles' and len(expected_colors)>1: assert ramp['mode']=='Steps'
        b=ramp['bounds'];bottom=b['origin']['y']+b['height'];keys=[]
        for item in scene['items']:
            p=item['primitive'].get('Path')
            if p is None or len(p['commands'])!=4:continue
            first=p['commands'][0].get('MoveTo');third=p['commands'][2].get('MoveTo')
            if first is None or third is None:continue
            if direction=='Horizontal':
                if first['y']!=b['origin']['y'] or not(b['origin']['y']<third['y']<bottom):continue
                position=(first['x']-b['origin']['x'])/b['width']
            else:
                if first['x']!=b['origin']['x'] or not(b['origin']['x']<third['x']<b['origin']['x']+b['width']):continue
                position=(bottom-first['y'])/b['height']
            if -1e-12<=position<=1+1e-12:keys.append(position)
        assert len(keys)==len(expected_ticks),(case,keys)
        for a,bvalue in zip(keys,expected_ticks):close(a,bvalue)
        labels=[i['primitive']['Text']['text'] for i in scene['items'] if i.get('clip')==clip and 'Text' in i['primitive'] and i['primitive']['Text']['text'] in ('-2','0','2','3','4','6','8','Inf','-Inf')]
        assert labels==expected_labels,(case,labels)
        normalized.append(keys)
    record={'palette':case['palette'],'channel':channel,'facet':facet,'state':state,'styles':styles,'keys':normalized,'gradients':[p for p,_ in paired]}
    if orientation or presentation or display:record['parameters']={k:case.get(k) for k in ('nbin','direction','reverse','lower','upper','labels') + (('display','constant') if display else ()) + (('alpha',) if alpha else ()) + (('family','endpoints','guide_kind') if steps else ()) + (('population','mode') if boundaries else ())}
    records.append(record)
    return request,frame,scene
for pi,case in enumerate(reference):
    channels=[['color','fill','stroke'][pi%3]] if orientation or display else ['color'] if constant or presentation else ['color','fill','stroke']
    facets=[['single','collected','local'][(pi//3)%3]] if orientation or display else [['single','collected','local'][pi%3]] if presentation else ['single'] if constant else ['single','collected','local']
    if boundaries: facets=['single']
    controls=dict(direction=case['direction'].capitalize(),reverse=case['reverse'],draw_lower_limit=case.get('lower',True),draw_upper_limit=case.get('upper',True),hidden_labels=case.get('labels')=='hidden') if orientation or presentation or display else None
    if display and not steps: controls['display']=case['display'].capitalize()
    if steps: controls.update(steps_family=case['family'],steps_endpoints=case.get('endpoints',False),steps_default=case.get('guide_kind')=='default')
    if boundaries: controls['boundary']=case
    if step_controls: controls.update(even_steps=case['even_steps'],show_limits=case['show_limits'])
    if alpha and case['alpha'] is not None: controls['alpha']=case['alpha']
    is_constant=constant or (display and case.get('constant',False))
    for channel in channels:
        for fi,facet in enumerate(facets):
            if boundaries and 'error' in case['result']:
                try:
                    rejected=author(case['palette'],channel,facet,case.get('nbin'),is_constant,controls)
                    rejected_request=output.request(rejected,c.export_options(600,360)); rejected_frame=rejected_request.prepare()
                except c.ChartError: records.append({'family':case['family'],'population':case['population'],'mode':case['mode'],'error':True})
                else: raise AssertionError(('expected rejection',case))
                continue
            plot=author(case['palette'],channel,facet,case.get('nbin'),is_constant,controls); wire=plot.to_json()
            if sampling or constant or orientation or presentation or display:
                version=68 if step_controls and (not case['even_steps'] or case['show_limits']) else 67 if alpha and case['alpha'] is not None else 66 if display and not steps and case['display']!='raster' else 65 if controls else 64
                assert json.loads(wire)['version']==version
                stale=json.loads(wire);stale['version']=version-1
                try: c.Plot.from_json(json.dumps(stale))
                except c.ChartError: pass
                else: raise AssertionError('stale sampling version accepted')
            request,frame,before=inspect(plot,case,channel,facet,'initial')
            loaded=c.Plot.from_json(wire); rq,fr,_=inspect(loaded,case,channel,facet,'roundtrip');fr.dispose();rq.dispose();loaded.dispose()
            hidden=plot.edit().scale(c.color_mapped('v',scale(case['palette'],True,case.get('nbin'),is_constant,controls))).build()
            rq,fr,_=inspect(hidden,case,channel,facet,'hidden');fr.dispose();rq.dispose()
            restored=hidden.edit().scale(c.color_mapped('v',scale(case['palette'],nbin=case.get('nbin'),constant=is_constant,controls=controls))).build()
            rq,fr,_=inspect(restored,case,channel,facet,'restored');fr.dispose();rq.dispose();restored.dispose();hidden.dispose()
            assert plot.to_json()==wire and frame.scene()==before
            publish=(case['nbin']==5 or (case['nbin']>0 and case['lower'] and case['upper'] and case['labels']=='automatic')) if presentation else orientation or constant or channel==['color','fill','stroke'][(pi+fi)%3]
            if display: publish=(pi<80 and (case['nbin'] is None or case['nbin']==2.5)) or (pi>=80 and case['palette']=='discontinuous' and case['nbin']==5)
            if steps: publish=not boundaries or (case['population']=='ordinary' and bool(case['result'].get('bar')))
            if alpha: publish=pi%4==(pi//4)%4
            if publish:
                name=f"{case['palette']}-{channel}-{facet}" + (f"-nbin-{case['nbin']:g}" if sampling or constant else "")
                if orientation or presentation or display:name=f"{'step-controls' if step_controls else 'default-steps' if default_steps else 'step-boundaries' if boundaries else 'steps' if steps else 'alpha' if alpha else 'display' if display else 'orientation' if orientation else 'presentation'}-{pi:03}"
                (out/f'{name}.scene.json').write_text(json.dumps(before));(out/f'{name}.plot.json').write_text(wire)
                for fmt in ['svg','pdf','png']:(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
            frame.dispose();request.dispose();plot.dispose()
if demand:
    for case in json.loads((ROOT / 'fixtures/parity/ggplot2/colorbar-demand.json').read_text())['cases']:
        values=[] if case['population']=='empty' else [-2.,0.,3.,8.]
        data=c.Data.columns({'v':c.column(values,kind='float64')})
        aes=c.aes().x(1.).y(1.).fill('v').fill_scale('v')
        spec=scale('ordinary',case['hidden'], {'number':'NaN'} if case['nbin']=='NA' else case['nbin'])
        if not case['hidden']:
            if case['selection']=='no_breaks': spec['guide']['Colorbar']['breaks']=[]
            if case['selection']=='no_labels': spec['guide']['Colorbar']['labels']='Hidden'
        builder=c.plot(data).profile('Ggplot2_4_0_3').aes(aes).scale(c.color_mapped('v',spec)).layer(c.points())
        plot=builder.build();wire=plot.to_json();loaded=c.Plot.from_json(wire)
        request=output.request(loaded,c.export_options(600,360))
        try: frame=request.prepare()
        except c.ChartError:
            assert 'error' in case['result'],case
            result={'error':True}
        else:
            assert 'error' not in case['result'],case
            scene=frame.scene()
            bars=sum('SampledGradientRectangle' in i['primitive'] or ('Rectangle' in i['primitive'] and abs(i['primitive']['Rectangle']['bounds']['height']/i['primitive']['Rectangle']['bounds']['width']-20/3)<1e-10) for i in scene['items'])
            assert bars==case['result']['guide_count'],case
            result={'bars':bars};frame.dispose()
        records.append({k:case[k] for k in ('nbin','population','selection','hidden')}|{'result':result})
        request.dispose();loaded.dispose();plot.dispose();builder.dispose();aes.dispose();data.dispose()
output.dispose()
assert len(records)==(711 if step_controls else 93 if default_all else 80 if default_steps else 180 if boundaries else 72 if steps else 384 if alpha else 704 if display else 192 if orientation else 512 if presentation else 60 if demand else 48 if constant else 432 if sampling else 108)
(out/'records.json').write_text(json.dumps(records))
print(f'PASS Python GG-05: {len(records)} geometry/lifecycle states and {len(list(out.glob("*.png")))*3} publication files.')
