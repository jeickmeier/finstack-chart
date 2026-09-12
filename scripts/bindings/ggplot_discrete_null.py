"""FIX-GG04: actual nullable hue domains, mapped missing values and immutable replacement."""
import json
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current')
cases=[v for v in json.loads((ROOT/'fixtures/parity/ggplot2/discrete-null-domains.json').read_text())['cases'] if v['kind']=='hue'];records=[]
cases += json.loads((ROOT/'fixtures/parity/ggplot2/identity-null-paints.json').read_text())['cases']
def key(v):return 'Null' if v is None else {'Text':v}
def descriptor(case):
    if case.get('kind') == 'identity':
        return {'training':'Eligible','function':{'GgplotDiscreteIdentity':{'limits':None if case['limits'] is None else [key(v) for v in case['limits']],'levels':None if case['levels'] is None else [key(v) for v in case['levels']],'drop':case['drop'],'na_translate':case['na_translate'],'guide':case['guide'],'observed':[]}}}
    return {'training':'Eligible','function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'ggplot':{'Discrete':{'limits':None if case['limits'] is None else [key(v) for v in case['limits']],'levels':None if case['levels'] is None else [key(v) for v in case['levels']],'drop':case['drop'],'na_translate':case['na_translate'],'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}}}}}
def data_for(case):
    values=case['inputs'];return c.Data.columns({'x':c.column([float(i) for i in range(len(values))],kind='float64'),'v':c.column(values,kind='string').nullable(True)},keys=list(range(100,100+len(values))))
def build(data,case):return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(case))).layer(c.points()).build()
def paint(v):
    if v is None:return {'red':0,'green':0,'blue':0,'alpha':0}
    if v in ('red4','blue','green','orange','NA'):
        return dict(zip(('red','green','blue','alpha'),{'red4':[139,0,0,255],'blue':[0,0,255,255],'green':[0,255,0,255],'orange':[255,165,0,255],'NA':[255,255,255,0]}[v]))
    if v=='grey50':return {'red':127,'green':127,'blue':127,'alpha':255}
    assert v.startswith('#') and len(v)==7,v
    return dict(zip(('red','green','blue','alpha'),[int(v[i:i+2],16) for i in (1,3,5)]+[255]))
def check(chart,case):
    layer=chart.semantics()['layers'][0];colors=[v['color'] for v in layer.get('styles',[])];wanted=[paint(v) for v in case['result']['training_paints'] if v is not None]
    assert colors==wanted,(case,colors,wanted)
    legend=layer.get('color_legend');entries=legend['entries'] if legend else [];domain=legend['mapping']['function'].get('Ordinal',{}).get('domain') if legend else []
    identity=case.get('kind')=='identity'
    if identity: assert len(colors)==case['result']['point_count']
    wanted_entries=[['NA' if label is None else label,paint('NA' if identity and color is None else color)] for label,color in zip(case['result']['labels'],case['result']['guide_paints'])]
    if identity and not case['guide']: wanted_entries=[]
    assert entries==wanted_entries,(case,entries,wanted_entries)
    if not identity: assert domain==[key(v) for v in case['result']['breaks']],(case,domain)
    return {'colors':colors,'entries':entries,'domain':domain}
for index,case in enumerate(cases):
    owned=[]
    try:
        data=data_for(case);owned.append(data);plot=build(data,case);owned.append(plot);wire=plot.to_json();restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
        chart=restored.chart();owned.append(chart);records.append({'index':index,**check(chart,case)})
        sample=None
        if case['kind']=='identity' and case['population']=='mixed' and case['levels'] is None and case['drop'] and case['na_translate'] and case['limits_name']=='first' and case['guide']:sample='identity-missing-first'
        if case['kind']=='hue' and case['population']=='mixed' and case['drop']:
            if case['levels'] is None and case['na_translate'] and case['limits_name']=='first':sample='authored-missing-first'
            if case['levels'] is not None and case['na_translate'] and case['limits_name']=='auto':sample='factor-missing-middle'
            if case['levels'] is None and not case['na_translate'] and case['limits_name']=='first':sample='suppressed-missing'
            if case['levels'] is None and not case['na_translate'] and case['limits_name']=='only':sample='empty-palette'
        if sample:
            request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
            for fmt in ('svg','pdf','png'):(out/f'{sample}.{fmt}').write_bytes(frame.export(fmt))
            (out/f'{sample}.scene.json').write_text(json.dumps(frame.scene(),indent=2))
    finally:
        for value in reversed(owned):value.dispose()
for kind in ('hue','identity'):
    for limits_name in ('auto','first'):
        for translate in (False,True):
            selected={v['population']:v for v in cases if v['kind']==kind and (kind=='hue' or v['guide']) and v['levels'] is None and v['limits_name']==limits_name and v['na_translate']==translate and v['drop']}
            owned=[]
            try:
                original=data_for(selected['mixed']);owned.append(original);plot=build(original,selected['mixed']);owned.append(plot);wire=plot.to_json();chart=plot.chart();owned.append(chart)
                request=output.request(plot,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
                for population in ('finite','missing','empty','mixed'):
                    case=selected[population];replacement=data_for(case);owned.append(replacement);updates=chart.transaction();owned.append(updates);builder=updates.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);receipt=chart.commit(update);assert 'Applied' in receipt,receipt
                    fresh=build(replacement,case);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(chart,case);assert actual==check(batch,case)
                    assert held.scene()==scene and plot.to_json()==wire
                    records.append({'kind':kind,'replacement':population,'limits':limits_name,'translate':translate,**actual})
            finally:
                for value in reversed(owned):value.dispose()
position_cases=json.loads((ROOT/'fixtures/parity/ggplot2/missing-paint-positions.json').read_text())['cases']
for index,case in enumerate(position_cases):
    owned=[]
    try:
        columns={axis:c.categorical(case[axis]) if case[axis+'_kind']=='category' else c.column(case[axis],kind='float64') for axis in ('x','y')}
        columns['v']=c.column(case['values'],kind='string').nullable(True)
        data=c.Data.columns(columns,keys=list(range(100,104)));owned.append(data)
        spec=descriptor({**case,'drop':True,'levels':None});spec['guide']='Hidden'
        plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('v').color_scale('v')).scale(c.color_mapped('v',spec)).layer(c.points()).x_axis(c.x_axis().range(100,540)).y_axis(c.y_axis().range(300,60)).build();owned.append(plot)
        request=output.request(plot,options);owned.append(request);frame=request.prepare();owned.append(frame);scene=frame.scene();points=[None]*4
        for item,targets in zip(scene['items'],scene['targets']):
            if not targets or 'Source' not in targets[0]:continue
            point=item['primitive']['Point'];row=int(targets[0]['Source']['key'])-100;assert points[row] is None
            points[row]=[(point['center']['x']-100)/440,(300-point['center']['y'])/240]
            assert point['fill']==paint(case['result']['colors'][row])
        for row,point in enumerate(points):
            if case['result']['colors'][row] is None:assert point is None,(case,row,point)
            else:
                assert point is not None,(case,row)
                assert all(abs(point[i]-case['result'][axis+'_positions'][row])<1e-12 for i,axis in enumerate(('x','y'))),(case,row,point)
        guides=frame.guides()['guides'];labels={}
        for axis,side,start,span in [('x','Bottom',100,440),('y','Left',300,-240)]:
            guide=next(g for g in guides if g['spec']['side']==side);labels[axis]=sorted(t['label'] for t in guide['ticks']);assert labels[axis]==sorted(case['result'][axis+'_labels'])
            for label,position in zip(case['result'][axis+'_labels'],case['result'][axis+'_major_positions']):
                tick=next(t for t in guide['ticks'] if t['label']==label);assert abs((tick['position']-start)/span-position)<1e-12,(case,tick,position)
        records.append({'position_index':index,'points':points,'labels':labels})
        if case['x_kind']=='category' and case['y_kind']=='numeric' and case['population']=='mixed' and case['limits_name']=='first' and not case['na_translate']:
            for fmt in ('svg','pdf','png'):(out/f'unpainted-categories.{fmt}').write_bytes(frame.export(fmt))
    finally:
        for value in reversed(owned):value.dispose()
empty_case=next(case for case in cases if case['population']=='empty' and case['limits_name']=='auto' and case['levels'] is None)
for retained in (False,True):
    owned=[]
    try:
        data=data_for(empty_case);owned.append(data);spec=descriptor(empty_case);spec['ggplot']['Discrete']['empty_population']=retained
        plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',spec)).layer(c.points()).build();owned.append(plot);wire=plot.to_json();encoded=json.loads(wire);version=21 if retained else 17;assert encoded['version']==version
        restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
        if retained:
            encoded['version']=20;rejected=False
            try:
                downgraded=c.Plot.from_json(json.dumps(encoded));owned.append(downgraded)
            except c.ChartError as error:
                assert json.loads(str(error))['code']=='CHART_UNSUPPORTED_CAPABILITY';rejected=True
            assert rejected
        else:assert 'empty_population' not in wire
        records.append({'retained_empty':retained,'version':version})
    finally:
        for value in reversed(owned):value.dispose()
assert len(records)==874,len(records)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose()
print('PASS Python: 256 hue and 512 identity nullable domains, 32 replacements, 72 position panels, two wire cases and six SVG/PDF/PNG samples.')
